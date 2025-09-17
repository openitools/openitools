use openitools_idevice::UsbmuxdProvider;
use serde::Serialize;

use super::get_string_value_or_default;

#[derive(Serialize, Clone, Default)]
pub struct OS {
    pub ios_ver: String,
    pub build_num: String,
}

pub async fn handle_device_os(provider: &UsbmuxdProvider) -> OS {
    let mut lockdownd_client = match openitools_idevice::get_lockdownd_client(provider).await {
        Ok(lockdown) => lockdown,
        Err(e) => {
            log::error!("Something went wrong while getting the lockdownd client: {e:?}");
            return OS::default();
        }
    };

    let ios_ver = get_string_value_or_default(&mut lockdownd_client, Some("ProductVersion"), None)
        .await
        .unwrap_or_default();

    let build_num = get_string_value_or_default(&mut lockdownd_client, Some("BuildVersion"), None)
        .await
        .unwrap_or_default();

    OS { ios_ver, build_num }
}
