use serde::Serialize;

use rsmobiledevice::{
    device::DeviceClient, device_info::domains::DeviceDomains, devices_collection::SingleDevice,
};

#[derive(Serialize, Clone)]
pub struct Storage {
    pub total: u64,
    pub used: u64,
    pub available: u64,
}

pub fn handle_device_storage(device: &DeviceClient<SingleDevice>) -> Storage {
    let device_info = device.get_device_info();

    let disk_hash = device_info
        .get_values(DeviceDomains::DiskUsage)
        .unwrap_or_default();

    let mut total = disk_hash
        .get("TotalDiskCapacity")
        .map_or(0, |s| s.parse::<u64>().unwrap_or_default());

    total /= 1e+9 as u64;

    let mut available = disk_hash
        .get("AmountRestoreAvailable")
        .map_or(0, |s| s.parse::<u64>().unwrap_or_default());

    available /= 1e+9 as u64;

    let used = total - available;

    Storage {
        total,
        used,
        available,
    }
}
