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

#[tauri::command]
pub async fn get_bundles(model: String, version: String) -> Vec<Bundle> {
    let metadata = get_metadata(&model).await.unwrap();

    for firmware in metadata.fw {
        if firmware.version == version {
            let url = format!(
                "{BASE_IPCC_URL}/{}/{version}/bundles.json",
                urlencoding::encode(&model)
            );
            let response = reqwest::get(&url).await.unwrap();

            let bundles =
                serde_json::from_str::<BundlesFile>(&response.text().await.unwrap()).unwrap();

            return bundles.bundles;
        }
    }

    return vec![];
}

pub async fn get_metadata(model: &str) -> Result<Metadata, QueryError> {
    let encoded_model = urlencoding::encode(model);

    let url = format!("{BASE_IPCC_URL}/{encoded_model}/metadata.json");

    log::debug!("Fetching metadata from: {url}");

    let response = reqwest::get(&url).await?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(QueryError::NotFound);
    }

    let response_text = response.text().await.map_err(|e| {
        log::error!("failed to get response text for model {model}: {e}");
        QueryError::EmptyBody
    })?;

    serde_json::from_str(&response_text).map_err(|e| {
        log::error!("failed to deserialize metadata for model {model}: {e}");
        QueryError::DeserializeError {
            text: response_text,
            error: e,
        }
    })
}
