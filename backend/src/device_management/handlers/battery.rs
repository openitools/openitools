use serde::Serialize;

use rsmobiledevice::{device::DeviceClient, devices_collection::SingleDevice, RecursiveFind};

#[derive(Serialize, Clone, Default)]
pub struct Battery {
    pub battery_level: u8,
    pub battery_health: f32,
    pub cycle_counts: u32,
}

pub fn handle_device_battery(device: &DeviceClient<SingleDevice>) -> Battery {
    let device_diag = device.get_device_diagnostic();

    let battery_plist = match device_diag.get_battery_plist() {
        Ok(plist) => plist,
        Err(_) => return Battery::default(),
    };

    let battery_level = battery_plist
        .rfind("CurrentCapacity")
        .map_or(0, |n| n.parse::<u8>().unwrap_or_default());

    let cycle_counts = battery_plist
        .rfind("CycleCount")
        .map_or(0, |n| n.parse::<u32>().unwrap_or_default());

    let _designed_capa = battery_plist
        .rfind("DesignCapacity")
        .map_or(0.0, |n| n.parse::<f32>().unwrap_or_default());

    let _max_capa = battery_plist
        .rfind("NominalChargeCapacity")
        .map_or(0.0, |n| n.parse::<f32>().unwrap_or_default());

    let battery_health = ((_max_capa / _designed_capa) * 100.0 * 100.0).round() / 100.0;

    Battery {
        battery_level,
        battery_health,
        cycle_counts,
    }
}
