use openitools_idevice::UsbmuxdProvider;
use serde::Serialize;

#[derive(Serialize, Clone, Default)]
pub struct Storage {
    pub total: u64,
    pub used: u64,
    pub available: u64,
}

pub async fn handle_device_storage(provider: &UsbmuxdProvider) -> Storage {
    let mut lockdownd_client = match openitools_idevice::get_lockdownd_client(provider).await {
        Ok(lockdown) => lockdown,
        Err(e) => {
            log::error!("something went wrong while getting the lockdownd client: {e:?}");
            return Storage::default();
        }
    };

    let total = lockdownd_client
        .get_value(Some("TotalDiskCapacity"), Some("com.apple.disk_usage"))
        .await
        .map_or(0, |s| {
            s.as_unsigned_integer().unwrap_or_default() / 1e+9 as u64
        });

    let available = lockdownd_client
        .get_value(Some("AmountRestoreAvailable"), Some("com.apple.disk_usage"))
        .await
        .map_or(0, |s| {
            s.as_unsigned_integer().unwrap_or_default() / 1e+9 as u64
        });

    Storage {
        total,
        used: total - available,
        available,
    }
}
