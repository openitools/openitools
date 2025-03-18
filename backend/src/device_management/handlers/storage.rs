use serde::Serialize;

use rsmobiledevice::{
    device::DeviceClient, device_info::domains::DeviceDomains, devices_collection::SingleDevice,
};

#[derive(Serialize, Clone)]
pub struct Storage {
    pub total_storage: u64,
    pub used_storage: u64,
    pub available_storage: u64,
}

pub fn handle_device_storage(device: &DeviceClient<SingleDevice>) -> Storage {
    let device_info = device.get_device_info();

    let disk_dict = device_info.get_values(DeviceDomains::DiskUsage).unwrap();
    let mut total_storage = disk_dict
        .get("TotalDiskCapacity")
        .map_or(0, |s| s.parse::<u64>().unwrap_or_default());

    total_storage /= 1e+9 as u64;

    let mut available_storage = disk_dict
        .get("AmountRestoreAvailable")
        .map_or(0, |s| s.parse::<u64>().unwrap_or_default());

    available_storage /= 1e+9 as u64;

    let used_storage = total_storage - available_storage;

    Storage {
        total_storage,
        available_storage,
        used_storage,
    }
}
