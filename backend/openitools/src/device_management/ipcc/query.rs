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

pub async fn download_bundle(
    device_model: &str,
    ios_version: &str,
    bundle: &str,
) -> Result<Vec<u8>, QueryError> {
    let url = format!(
        "{BASE_IPCC_URL}/{}/{ios_version}/{bundle}.tar",
        urlencoding::encode(device_model)
    );

    let response = reqwest::get(&url).await?;

    let res_bytes = response.bytes().await?;

    Ok(res_bytes.to_vec())
}

#[tauri::command]
pub async fn get_bundles(device_model: String, ios_version: String) -> Vec<Bundle> {
    let metadata = match get_metadata(&device_model).await {
        Ok(md) => md,
        Err(e) => {
            log::error!("failed to get the metadata for ({device_model}:{ios_version}): {e}");
            return Vec::default();
        }
    };

    for firmware in metadata.fw {
        if firmware.version == ios_version {
            let url = format!(
                "{BASE_IPCC_URL}/{}/{ios_version}/bundles.json",
                urlencoding::encode(&device_model)
            );
            let response = match reqwest::get(&url).await {
                Ok(res) => res,
                Err(e) => {
                    log::error!(
                        "failed to get the bundles.json for ({device_model}:{ios_version}): {e}"
                    );
                    return Vec::default();
                }
            };

            let response_text = match response.text().await {
                Ok(res_t) => res_t,
                Err(e) => {
                    log::error!("failed to get the text of the response: {e}");
                    return Vec::default();
                }
            };

            let bundles = match serde_json::from_str::<BundlesFile>(&response_text) {
                Ok(b) => b,
                Err(e) => {
                    log::error!("failed to convert the response bundles.json to a struct: {e}");
                    return Vec::default();
                }
            };

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
