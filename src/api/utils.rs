/// Copyright (c) Shipt, Inc.
/// This source code is licensed under the MIT license found in the
/// LICENSE file in the root directory of this source tree.
use anyhow::Context;
use lazy_static::lazy_static;

use owo_colors::OwoColorize;
use reqwest::Url;
use reqwest::{self};
use std::env;
use std::{format, path::Path};

lazy_static! {
    static ref OPSML_TRACKING_URI: String = match env::var("OPSML_TRACKING_URI") {
        Ok(val) =>
            if val.ends_with('/') {
                remove_suffix(&val, '/')
            } else {
                val
            },

        Err(_e) => {
            panic!(
                "{}",
                "No OPSML_TRACKING_URI found. Check your environment"
                    .bold()
                    .red()
            )
        }
    };
}

pub enum OpsmlPaths {
    ListCard,
    MetadataDownload,
    Download,
    Metric,
    ListFile,
}

impl OpsmlPaths {
    pub fn as_str(&self) -> String {
        match self {
            OpsmlPaths::ListCard => format!("{}/opsml/cards/list", *OPSML_TRACKING_URI),
            OpsmlPaths::MetadataDownload => {
                format!("{}/opsml/models/metadata", *OPSML_TRACKING_URI)
            }
            OpsmlPaths::Download => {
                format!("{}/opsml/files/download", *OPSML_TRACKING_URI)
            }
            OpsmlPaths::Metric => {
                format!("{}/opsml/metrics", *OPSML_TRACKING_URI)
            }
            OpsmlPaths::ListFile => format!("{}/opsml/files/list", *OPSML_TRACKING_URI),
        }
    }
}

pub async fn check_args(
    name: Option<&str>,
    repository: Option<&str>,
    version: Option<&str>,
    uid: Option<&str>,
) -> Result<(), anyhow::Error> {
    let common_args = [name, version, repository];
    let has_common = common_args.iter().all(|i| i.is_none());

    let has_uid = uid.is_none();

    if has_common != has_uid {
        Ok(())
    } else {
        Err(anyhow::Error::msg(
            "Please provide either a uid or a name, repository, and version",
        ))
    }
}

/// Removes the suffix from a string if it exists
///
/// # Arguments
///
/// * `s` - A string slice
/// * `suffix` - A string slice
///
pub fn remove_suffix(s: &str, suffix: char) -> String {
    match s.strip_suffix(suffix) {
        Some(s) => s.to_string(),
        None => s.to_string(),
    }
}

pub async fn create_client(
    url: &str,
    params: Option<&[(&str, &str)]>,
) -> Result<(reqwest::Client, Url), anyhow::Error> {
    let parsed_url = match params {
        Some(p) => {
            let mut url = Url::parse(url).with_context(|| "Failed to parse url")?;
            for (key, value) in p {
                url.query_pairs_mut().append_pair(key, value);
            }
            url
        }
        None => Url::parse(url).with_context(|| "Failed to parse url")?,
    };
    //let parsed_url = reqwest::Url::parse(url).with_context(|| "Failed to parse url")?;
    let client = reqwest::Client::new();

    Ok((client, parsed_url))
}

/// Create parent directories associated with path
///
/// # Arguments
///
/// * `path` - path to create
///
pub fn create_dir_path(path: &Path) -> Result<(), anyhow::Error> {
    let prefix = path
        .parent()
        .with_context(|| "Failed to get parent directory")?;
    std::fs::create_dir_all(prefix)
        .with_context(|| format!("Failed to create directory path for {:?}", prefix))?;

    Ok(())
}

pub enum SaveRoot {
    Model,
}

impl SaveRoot {
    pub fn as_str(&self) -> &'static str {
        match self {
            SaveRoot::Model => "opsml-root:/OPSML_MODEL_REGISTRY",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_suffix() {
        let test_uri_with_slash = "http://localhost:8080/";
        let test_uri_without_slash = "http://localhost:8080";
        let processed_with_slash_uri = remove_suffix(test_uri_with_slash, '/');
        let processed_without_slash_uri = remove_suffix(test_uri_without_slash, '/');
        assert_eq!(processed_with_slash_uri, "http://localhost:8080");
        assert_eq!(processed_without_slash_uri, test_uri_without_slash);
    }
}
