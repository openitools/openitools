use std::io::Cursor;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Metadata {
    pub fw: Vec<Firmware>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Firmware {
    pub version: String,
}

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("the `metadata.json` file was not found for the specified model")]
    NotFound,

    #[error("couldn't deserialize the `metadata.json` content: {error}")]
    DeserializeError {
        text: String,
        error: serde_json::Error,
    },

    #[error("no text found in the response")]
    EmptyBody,

    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Bundle {
    bundle_name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
struct BundlesFile {
    bundles: Vec<Bundle>,
}

const BASE_IPCC_URL: &str =
    "https://raw.githubusercontent.com/openitools/openitools-ipcc/refs/heads/files";

pub async fn download_bundle(device_model: &str, ios_version: &str, bundle: &str) -> Vec<u8> {
    let url = format!(
        "{BASE_IPCC_URL}/{}/{ios_version}/{bundle}.tar",
        urlencoding::encode(device_model)
    );

    let response = reqwest::get(&url).await.unwrap();

    let res_bytes = response.bytes().await.unwrap();

    res_bytes.to_vec()
}

#[tauri::command]
pub async fn get_bundles(device_model: String, ios_version: String) -> Vec<Bundle> {
    let metadata = get_metadata(&device_model).await.unwrap();

    for firmware in metadata.fw {
        if firmware.version == ios_version {
            let url = format!(
                "{BASE_IPCC_URL}/{}/{ios_version}/bundles.json",
                urlencoding::encode(&device_model)
            );
            let response = reqwest::get(&url).await.unwrap();

            let bundles =
                serde_json::from_str::<BundlesFile>(&response.text().await.unwrap()).unwrap();

            println!("{bundles:#?}");
            return bundles.bundles;
        }
    }

    vec![]
}

pub async fn get_metadata(device_model: &str) -> Result<Metadata, QueryError> {
    let encoded_model = urlencoding::encode(device_model);

    let url = format!("{BASE_IPCC_URL}/{encoded_model}/metadata.json");

    log::debug!("Fetching metadata from: {url}");

    let response = reqwest::get(&url).await?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(QueryError::NotFound);
    }

    let response_text = response.text().await.map_err(|e| {
        log::error!("failed to get response text for model {device_model}: {e}");
        QueryError::EmptyBody
    })?;

    serde_json::from_str(&response_text).map_err(|e| {
        log::error!("failed to deserialize metadata for model {device_model}: {e}");
        QueryError::DeserializeError {
            text: response_text,
            error: e,
        }
    })
}
