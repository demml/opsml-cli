/// Copyright (c) Shipt, Inc.
/// This source code is licensed under the MIT license found in the
/// LICENSE file in the root directory of this source tree.
use crate::api::types;
use crate::api::utils;
use anyhow::Context;
use futures_util::StreamExt;
use owo_colors::OwoColorize;
use reqwest::{self, Response};
use serde::Serialize;
use std::time::Duration;
use std::{format, path::Path};
use tokio::time::sleep;

pub struct RouteHelper {}

impl RouteHelper {
    /// async post request for metadata
    ///
    /// # Arguments
    ///
    /// * `url` - A string slice
    /// * `payload` - A string slice
    ///
    pub async fn make_post_request<T: Serialize>(
        url: &str,
        payload: &T,
    ) -> Result<Response, anyhow::Error> {
        let (client, parsed_url) = utils::create_client(url, None).await.unwrap();
        let msg = client
            .post(parsed_url)
            .json(payload)
            .send()
            .await
            .with_context(|| "failed to send post request")?;

        Ok(msg)
    }

    /// async get request for metadata
    ///
    /// # Arguments
    ///
    /// * `url` - A string slice
    ///
    pub async fn make_get_request(
        url: &str,
        params: Option<&[(&str, &str)]>,
    ) -> Result<Response, anyhow::Error> {
        let (client, parsed_url) = utils::create_client(url, params).await.unwrap();

        let msg = client
            .get(parsed_url)
            .send()
            .await
            .with_context(|| "Failed to send get request")?;

        Ok(msg)
    }

    /// Lists files associated with a model
    ///
    /// # Arguments
    ///
    /// * `rpath` - Remote path to file
    ///
    /// # Returns
    /// * `Result<types::ListFileResponse, String>` - Result of file download
    ///
    pub async fn list_files(rpath: &Path) -> Result<types::ListFileResponse, anyhow::Error> {
        let params = [("path", rpath.to_str().unwrap())];
        let response =
            RouteHelper::make_get_request(&utils::OpsmlPaths::ListFile.as_str(), Some(&params))
                .await?;

        let files = response.json::<types::ListFileResponse>().await?;

        Ok(files)
    }

    /// Downloads a stream to a file
    ///
    /// # Arguments
    ///
    /// * `response` - Response object
    /// * `filename` - Path to save file to
    ///
    /// # Returns
    /// * `Result<(), String>` - Result of file download
    ///
    pub async fn download_stream_to_file(
        response: Response,
        filename: &Path,
    ) -> Result<(), anyhow::Error> {
        let mut response_stream = response.bytes_stream();
        let mut file = tokio::fs::File::create(filename).await.with_context(|| {
            format!(
                "failed to create file for {:?}",
                filename.to_str().unwrap().red()
            )
        })?;

        while let Some(item) = response_stream.next().await {
            let chunk =
                item.with_context(|| format!("failed to read response for {:?}", filename))?;
            tokio::io::copy(&mut chunk.as_ref(), &mut file)
                .await
                .with_context(|| format!("failed to write response for {:?}", filename))?;
        }
        Ok(())
    }

    /// Downloads an artifact file. Retries up to 3 times if download fails.
    ///
    /// # Arguments
    ///
    /// * `url` - url of opsml server
    /// * `uri` - uri of model
    /// * `local_save_path` - path to save model to
    ///
    /// # Returns
    /// * `Result<(), String>` - Result of file download
    ///
    pub async fn download_file(lpath: &Path, rpath: &str) -> Result<(), anyhow::Error> {
        let params = [("path", rpath)];
        let mut attempts = 0;
        let max_attempts = 3;

        while attempts < max_attempts {
            attempts += 1;
            let response =
                RouteHelper::make_get_request(&utils::OpsmlPaths::Download.as_str(), Some(&params))
                    .await
                    .with_context(|| "failed to download model")?;

            if response.status().is_success() {
                match RouteHelper::download_stream_to_file(response, lpath).await {
                    Ok(_) => return Ok(()),
                    Err(e) if attempts < max_attempts => {
                        println!("Attempt {} failed: {}. Retrying...", attempts, e);
                        sleep(Duration::from_secs(2)).await; // Wait before retrying
                    }
                    Err(e) => return Err(e),
                }
            } else {
                let error_message = format!(
                    "Failed to download model: {}",
                    response.text().await.unwrap().red()
                );
                return Err(anyhow::anyhow!(error_message));
            }
        }

        Err(anyhow::anyhow!(
            "Failed to download file after {} attempts",
            max_attempts
        ))
    }

    /// Parses stream response
    ///
    /// # Arguments
    ///
    /// * `response` - Response object
    ///
    /// # Returns
    /// * `String` - String representation of response
    ///
    pub async fn load_stream_response(response: Response) -> Result<String, anyhow::Error> {
        let mut response_stream = response.bytes_stream();
        let mut stream_buffer = String::new();
        while let Some(item) = response_stream.next().await {
            let chunk = item.with_context(|| "failed to read stream response")?;
            let string_chunk = std::str::from_utf8(&chunk).unwrap();

            stream_buffer.push_str(string_chunk);
        }
        Ok(stream_buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_json_diff::assert_json_eq;

    use std::env;
    use std::fs;
    use tokio;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_get_request() {
        let mut download_server = mockito::Server::new_async().await;
        let url = download_server.url();

        // get files
        let files_path = "./src/api/test_utils/list_files.json";
        let files = fs::read_to_string(files_path).expect("Unable to read file");

        // mock list files
        let get_path = format!("{}/get", url);
        let mock_get_path = download_server
            .mock("GET", "/get")
            .with_status(201)
            .with_body(&files)
            .create();

        let _ = RouteHelper::make_get_request(&get_path, None)
            .await
            .unwrap();
        mock_get_path.assert();
    }

    #[tokio::test]
    async fn test_post_request() {
        let mut download_server = mockito::Server::new_async().await;
        let url = download_server.url();

        // get files
        let files_path = "./src/api/test_utils/list_files.json";
        let files = fs::read_to_string(files_path).expect("Unable to read file");

        // mock list files
        let post_path = format!("{}/post", url);
        let mock_post_path = download_server
            .mock("POST", "/post")
            .with_status(201)
            .with_body(&files)
            .create();

        let model_metadata_request = types::ModelMetadataRequest {
            name: Some("name"),
            version: Some("version"),
            uid: Some("uid"),
            repository: Some("repository"),
            ignore_release_candidates: &false,
        };

        let _ = RouteHelper::make_post_request(&post_path, &model_metadata_request)
            .await
            .unwrap();

        mock_post_path.assert();
    }

    #[tokio::test]
    async fn test_list_files() {
        let mut download_server = mockito::Server::new_async().await;
        let url = download_server.url();

        unsafe {
            env::set_var("OPSML_TRACKING_URI", url);
        }

        // get files
        let files_path = "./src/api/test_utils/list_files.json";
        let files = fs::read_to_string(files_path).expect("Unable to read file");
        let list_files: types::ListFileResponse =
            serde_json::from_str(&fs::read_to_string(files_path).expect("Unable to read file"))
                .unwrap();

        // mock list files
        let artifact_path = "/opsml/files/list?path=files";
        let mock_list_files = download_server
            .mock("GET", artifact_path)
            .with_status(201)
            .with_body(&files)
            .create();

        let file_response = RouteHelper::list_files(Path::new("files")).await.unwrap();
        mock_list_files.assert();

        // assert structs are the same
        assert_json_eq!(list_files, file_response);
    }

    #[tokio::test]
    async fn test_download_file() {
        let mut download_server = mockito::Server::new_async().await;
        let url = download_server.url();
        unsafe {
            env::set_var("OPSML_TRACKING_URI", url);
        }

        // mock model
        let get_path = "/opsml/files/download?path=metadata.json";

        let mock_download_path = download_server
            .mock("GET", get_path)
            .with_status(201)
            .with_body("{'hello': 'world'}")
            .create();

        let uid = &Uuid::new_v4().to_string();
        let file_path = format!("{}.json", uid);
        let lpath = Path::new(&file_path);

        RouteHelper::download_file(lpath, "metadata.json")
            .await
            .unwrap();

        mock_download_path.assert();

        // check path exists
        assert!(lpath.exists());

        // delte path
        fs::remove_file(lpath).unwrap();
    }
}
