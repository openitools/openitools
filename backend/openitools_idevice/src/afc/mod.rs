use idevice::IdeviceService;
pub use idevice::afc::{AfcClient, file::FileDescriptor, opcode::AfcFopenMode};
mod types;
pub use types::{FSTree, FileType, PathInfo};

pub async fn download_from_afc(file_paths: Vec<String>) -> Vec<String> {
    todo!()
}

pub async fn get_afc_client() -> AfcClient {
    let provider = crate::get_provider().await.unwrap();
    idevice::afc::AfcClient::connect(&provider).await.unwrap()
}
