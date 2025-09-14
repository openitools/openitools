use openitools_idevice::LockdownClient;
use rsmobiledevice::{
    device::DeviceClient,
    device_info::{domains::DeviceDomains, keys::DeviceKeys},
    devices_collection::SingleDevice,
};
use serde::Serialize;

use super::get_string_value_or_default;

#[derive(Serialize, Clone)]
pub struct Hardware {
    pub model: String,
    pub model_number: String,
    pub region: String,
}
pub async fn handle_device_hardware(device: &mut LockdownClient) -> Hardware {
    let region_code = get_string_value_or_default(device, Some("RegionInfo"), None)
        .await
        .unwrap_or_default();

    let region: String = match region_code.as_str() {
        "LL/A" => "United States".into(),
        "B/A" => "Canada".into(),
        "C/A" => "Europe".into(),
        "J/A" => "Japan".into(),
        "X/A" => "China".into(),
        "ZP/A" => "Global/International".into(),
        "KH/A" => "Hong Kong".into(),
        "M/A" => "Mexico".into(),
        "A/A" => "Argentina".into(),
        "T/A" => "Taiwan".into(),
        "V/A" => "United Kingdom".into(),
        "R/A" => "Russia".into(),
        _ => "unknown".into(),
    };

    let model_number_code = get_string_value_or_default(device, Some("ModelNumber"), None)
        .await
        .unwrap_or_default();

    let model_meaning = match model_number_code.chars().next().unwrap_or_default() {
        'F' => "Refurbished Device",
        'M' => "New Device",
        'N' => "Warranty Replacement Device",
        'P' => "Personalized Device",
        '3' => "Demo Device",
        _ => "Unknown",
    };

    let model_number = format!("{model_number_code} ({model_meaning})",);

    let model = get_string_value_or_default(device, Some("ProductType"), None)
        .await
        .unwrap_or("Unknown".into());

    Hardware {
        model,
        model_number,
        region,
    }
}
