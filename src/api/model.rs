/// Copyright (c) Shipt, Inc.
/// This source code is licensed under the MIT license found in the
/// LICENSE file in the root directory of this source tree.
use crate::api::route_helper::RouteHelper;
use crate::api::types;
use crate::api::utils;
use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use serde_json;
use std::path::PathBuf;
use std::{fs, path::Path};

use super::types::ModelMetadata;

const MODEL_METADATA_FILE: &str = "model-metadata.json";
const NO_ONNX_URI: &str = "No onnx model uri found but onnx flag set to true";
const NO_QUANTIZE_URI: &str = "No quantize model uri found but quantize flag set to true";

pub struct ModelDownloader<'a> {
    pub name: Option<&'a str>,
    pub version: Option<&'a str>,
    pub repository: Option<&'a str>,
    pub uid: Option<&'a str>,
    pub write_dir: &'a str,
    pub ignore_release_candidates: &'a bool,
    pub onnx: &'a bool,
    pub quantize: &'a bool,
    pub preprocessor: &'a bool,
}

impl ModelDownloader<'_> {
    /// Saves metadata to json
    ///
    /// # Arguments
    ///
    /// * `metadata` - metadata to save
    /// * `path` - path to save to
    ///
    /// # Returns
    /// * `Result<(), String>` - Result of file download
    ///
    async fn save_metadata_to_json(
        &self,
        metadata: &types::ModelMetadata,
        path: &Path,
    ) -> Result<(), anyhow::Error> {
        let json_string =
            serde_json::to_string(metadata).with_context(|| "Failed to serialize metadata")?;
        fs::File::create(path).with_context(|| "Unable to create metadata file")?;
        fs::write(path, json_string).with_context(|| "Unable to write metadata file")?;
        Ok(())
    }

    /// Main function for downloading model metadata
    ///
    /// # Arguments
    ///
    /// * `args` - DownloadArgs struct
    ///
    /// # Returns
    /// * `Result<types::ModelMetadata, String>` - Result of model metadata download
    ///
    async fn get_model_metadata(&self) -> Result<types::ModelMetadata, anyhow::Error> {
        let save_path = Path::new(&self.write_dir).join(MODEL_METADATA_FILE);

        let model_metadata_request = types::ModelMetadataRequest {
            name: self.name,
            repository: self.repository,
            version: self.version,
            uid: self.uid,
            ignore_release_candidates: self.ignore_release_candidates,
        };

        let response = RouteHelper::make_post_request(
            &utils::OpsmlPaths::MetadataDownload.as_str(),
            &model_metadata_request,
        )
        .await?;

        let loaded_response = RouteHelper::load_stream_response(response).await?;
        let model_metadata: types::ModelMetadata = serde_json::from_str(&loaded_response)
            .with_context(|| "Failed to parse model Metadata")?;

        // create save path for metadata
        utils::create_dir_path(&save_path)?;
        self.save_metadata_to_json(&model_metadata, &save_path)
            .await?;

        Ok(model_metadata)
    }

    /// Sets model uri (onnx or trained model) depending on boolean
    ///
    /// # Arguments
    ///
    /// * `onnx` - Flag to download onnx model
    /// * `model_metadata` - Model metadata
    ///
    /// # Returns
    /// * `&Path` - Remote path to file
    ///
    fn get_model_uri(
        &self,
        model_metadata: &types::ModelMetadata,
    ) -> Result<PathBuf, anyhow::Error> {
        let uri = if self.onnx == &true {
            if self.quantize == &true {
                model_metadata
                    .quantized_model_uri
                    .clone()
                    .with_context(|| NO_QUANTIZE_URI.red())?
            } else {
                model_metadata
                    .onnx_uri
                    .clone()
                    .with_context(|| NO_ONNX_URI.red())?
            }
        } else {
            model_metadata.model_uri.clone()
        };

        let filepath = std::path::Path::new(&uri);

        Ok(filepath.to_owned())
    }

    /// Gets processor uri
    ///
    /// # Arguments
    ///
    /// * `model_metadata` - Model metadata
    ///
    /// # Returns
    /// * `Option<&Path>` - File path to processor or None
    ///
    fn get_preprocessor_uri(&self, model_metadata: &types::ModelMetadata) -> Option<PathBuf> {
        let uri = if model_metadata.preprocessor_uri.is_some() {
            Some(
                std::path::Path::new(&model_metadata.preprocessor_uri.as_ref().unwrap()).to_owned(),
            )
        } else if model_metadata.tokenizer_uri.is_some() {
            Some(std::path::Path::new(&model_metadata.tokenizer_uri.as_ref().unwrap()).to_owned())
        } else if model_metadata.feature_extractor_uri.is_some() {
            Some(
                std::path::Path::new(&model_metadata.feature_extractor_uri.as_ref().unwrap())
                    .to_owned(),
            )
        } else {
            None
        };

        uri.to_owned()
    }

    /// Downloads metadata
    ///
    /// # Arguments
    ///
    /// * `args` - DownloadArgs struct
    ///
    async fn get_metadata(&self) -> Result<types::ModelMetadata, anyhow::Error> {
        // check args first
        utils::check_args(self.name, self.repository, self.version, self.uid)
            .await
            .unwrap();
        let model_metadata = self.get_model_metadata().await?;

        Ok(model_metadata)
    }

    /// Downloads files associated with a model
    ///
    /// # Arguments
    ///
    /// * `rpath` - Remote path to file
    ///
    /// # Returns
    /// * `Result<(), String>` - Result of file download
    async fn download_files(&self, rpath: &Path, rpath_root: &Path) -> Result<(), anyhow::Error> {
        let rpath_files = RouteHelper::list_files(rpath).await?;

        // iterate over each file and download
        for file in rpath_files.files.iter() {
            let rpath = Path::new(file);

            let stripped_path = rpath
                .strip_prefix(rpath_root)
                .with_context(|| "Failed to create file path")?;

            let lpath = Path::new(self.write_dir).join(stripped_path);

            println!(
                "Downloading: {} from {}",
                lpath.display().to_string().green(),
                file
            );

            utils::create_dir_path(&lpath)?;
            RouteHelper::download_file(&lpath, file).await?;
        }

        Ok(())
    }

    /// Gets root to use
    ///
    /// # Arguments
    ///
    /// * `metadata` - Model metadata
    ///
    /// # Returns
    /// * `Result<PathBuf, String>` - Path to save root
    async fn get_save_root(&self, metadata: &ModelMetadata) -> Result<PathBuf, anyhow::Error> {
        let root = format!(
            "{}/{}/{}/v{}",
            utils::SaveRoot::Model.as_str(),
            metadata.model_repository,
            metadata.model_name,
            metadata.model_version
        );

        let root_path = Path::new(&root);

        Ok(root_path.to_owned())
    }

    /// Downloads preprocessor files
    ///
    /// # Arguments
    ///
    /// * `metadata` - Model metadata
    /// * `rpath_root` - Root path to save to
    ///
    /// # Returns
    /// * `Result<(), String>` - Result of file download
    async fn get_preprocessor(
        &self,
        metadata: &ModelMetadata,
        rpath_root: &Path,
    ) -> Result<(), anyhow::Error> {
        let preprocessor_rpath = self.get_preprocessor_uri(metadata);

        if preprocessor_rpath.is_some() {
            let preprocessor_rpath = preprocessor_rpath.unwrap();
            self.download_files(&preprocessor_rpath, rpath_root).await?;
        }

        Ok(())
    }

    /// Downloads a model file
    /// Will also download any associated preprocessor files
    /// Preprocessors can be tokenizer, feature extractor, or preprocessor
    async fn download_model(&self) -> Result<(), anyhow::Error> {
        let model_metadata = self.get_metadata().await?;

        let rpath_root = self.get_save_root(&model_metadata).await?;

        // Get preprocessor
        if self.preprocessor == &true {
            self.get_preprocessor(&model_metadata, &rpath_root).await?;
        }

        let model_rpath = self.get_model_uri(&model_metadata)?;

        // Get model
        self.download_files(&model_rpath, &rpath_root).await?;

        Ok(())
    }
}

/// Downloads model metadata
///
/// * `name` - Name of model
/// * `repository` - repository associated with model
/// * `version` - Version of model
/// * `uid` - uid of model
/// * `url` - url of opsml server
pub async fn download_model_metadata(
    name: Option<&str>,
    version: Option<&str>,
    repository: Option<&str>,
    uid: Option<&str>,
    write_dir: &str,
    ignore_release_candidates: &bool,
) -> Result<types::ModelMetadata, anyhow::Error> {
    // check args first

    let model_downloader = ModelDownloader {
        name,
        version,
        repository,
        uid,
        write_dir,
        ignore_release_candidates,
        onnx: &false,
        quantize: &false,
        preprocessor: &false,
    };
    model_downloader.get_metadata().await
}

/// Downloads model file
///
/// * `name` - Name of model
/// * `repository` - repository associated with model
/// * `version` - Version of model
/// * `uid` - uid of model
/// * `url` - url of opsml server
/// * `write_dir` - directory to write to
/// * `no_onnx` - Flag to not download onnx model
/// * `onnx` - Flag to download onnx model
///
#[allow(clippy::too_many_arguments)]
pub async fn download_model(
    name: Option<&str>,
    version: Option<&str>,
    repository: Option<&str>,
    uid: Option<&str>,
    write_dir: &str,
    onnx: &bool,
    quantize: &bool,
    preprocessor: &bool,
    ignore_release_candidates: &bool,
) -> Result<(), anyhow::Error> {
    let model_downloader = ModelDownloader {
        name,
        version,
        repository,
        uid,
        write_dir,
        ignore_release_candidates,
        onnx,
        quantize,
        preprocessor,
    };
    model_downloader.download_model().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tokio;
    #[tokio::test]
    async fn test_download_metadata() {
        let metadata = fs::read_to_string("./src/api/test_utils/metadata.json").unwrap();
        let model_metadata: types::ModelMetadata = serde_json::from_str(&metadata).unwrap();

        // setup server
        let mut download_server = mockito::Server::new_async().await;
        let url = download_server.url();
        env::set_var("OPSML_TRACKING_URI", url);

        // get files
        let files = types::ListFileResponse {
            files: vec![model_metadata.onnx_uri.as_ref().unwrap().to_string()],
        };
        let file_response = serde_json::to_string(&files).unwrap();

        // mock metadata
        let mock_metadata_path = download_server
            .mock("POST", "/opsml/models/metadata")
            .with_status(201)
            .with_body(serde_json::to_string(&model_metadata).unwrap())
            .create();

        // mock list files
        let artifact_path = "/opsml/files/list?path=models";

        let _mock_list_path = download_server
            .mock("GET", artifact_path)
            .with_status(201)
            .with_body(&file_response)
            .create();

        // mock list files for preprocessor
        let artifact_preprocessor_path = "/opsml/files/list?path=preprocessor.json";
        let preprocessor_files = types::ListFileResponse {
            files: vec!["preprocessor.json".to_string()],
        };
        let preprocessor_file_response = serde_json::to_string(&preprocessor_files).unwrap();

        let _mock_list_path = download_server
            .mock("GET", artifact_preprocessor_path)
            .with_status(201)
            .with_body(&preprocessor_file_response)
            .create();

        // mock model
        let get_model_path = "/opsml/files/download?path=models.json";

        let mock_model_path = download_server
            .mock("GET", get_model_path)
            .with_status(201)
            .with_body(&metadata)
            .create();

        // mock model
        let get_preprocessor_path = "/opsml/files/download?path=preprocessor.json";

        let mock_preprocessor_path = download_server
            .mock("GET", get_preprocessor_path)
            .with_status(201)
            .with_body(&metadata)
            .create();

        let downloader = ModelDownloader {
            name: Some("linear-reg-model"),
            version: Some("1.1.0"),
            repository: Some("devops-ml"),
            uid: None,
            write_dir: "downloaded",
            ignore_release_candidates: &false,
            onnx: &true,
            quantize: &false,
            preprocessor: &false,
        };

        let metadata = downloader.get_metadata().await.unwrap();
        mock_metadata_path.assert();

        let save_root = downloader.get_save_root(&metadata).await.unwrap();
        assert_eq!(
            save_root.to_str().unwrap(),
            "opsml-root:/OPSML_MODEL_REGISTRY/devops-ml/linear-reg-model/v1.1.0"
        );

        // test downloading files
        let model_rpath = downloader.get_model_uri(&model_metadata).unwrap();
        assert_eq!(model_rpath.to_str().unwrap(), "models.json");

        downloader
            .download_files(Path::new("models"), Path::new(""))
            .await
            .unwrap();

        mock_model_path.assert();

        downloader
            .get_preprocessor(&metadata, Path::new(""))
            .await
            .unwrap();

        mock_preprocessor_path.assert();

        // clean up
        fs::remove_dir_all("downloaded").unwrap();
    }
}
