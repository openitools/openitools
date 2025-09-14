use serde::Serialize;

use rsmobiledevice::{
    device::DeviceClient,
    device_info::{domains::DeviceDomains, keys::DeviceKeys},
    devices_collection::SingleDevice,
};

#[derive(Serialize, Clone)]
pub struct OS {
    pub ios_ver: String,
    pub build_num: String,
}

pub fn handle_device_os(device: &DeviceClient<SingleDevice>) -> OS {
    let device_info = device.get_device_info();

    OS {
        ios_ver: device_info
            .get_value(DeviceKeys::ProductVersion, DeviceDomains::All)
            .unwrap_or("unknown".into()),
        build_num: device_info
            .get_value(DeviceKeys::BuildVersion, DeviceDomains::All)
            .unwrap_or("unknown".into()),
    }
}
