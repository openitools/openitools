use openitools_idevice::UsbmuxdProvider;
use serde::Serialize;

use super::{get_string_value_or_default, RecursiveFind};

#[derive(Serialize, Clone, Default)]
pub struct Battery {
    pub level: u64,
    pub health: f32,
    pub cycle_counts: u64,
}

pub async fn handle_device_battery(provider: &UsbmuxdProvider) -> Battery {
    let mut device_diag = match openitools_idevice::get_diag_client(provider).await {
        Ok(diag) => diag,
        Err(e) => {
            log::error!("Something went wrong while getting the diagnistics relay client: {e:?}");
            return Battery::default();
        }
    };

    let battery_plist_key = get_battery_plist_key(provider).await;
    let battery_plist = device_diag
        .ioregistry(None, Some(battery_plist_key.as_str()), None)
        .await
        .unwrap_or_default()
        .unwrap_or_default();

    let level = battery_plist
        .rfind("CurrentCapacity")
        .map_or(0, |n| n.as_unsigned_integer().unwrap_or_default());

    let cycle_counts = battery_plist
        .rfind("CycleCount")
        .map_or(0, |n| n.as_unsigned_integer().unwrap_or_default());

    let health = {
        let designed_capa = battery_plist
            .rfind("DesignCapacity")
            .map_or(0, |n| n.as_unsigned_integer().unwrap_or_default())
            as f32;

        let max_capa = battery_plist
            .rfind("NominalChargeCapacity")
            .map_or(0, |n| n.as_unsigned_integer().unwrap_or_default())
            as f32;

        ((max_capa / designed_capa) * 100.0 * 100.0).round() / 100.0
    };

    Battery {
        level,
        health,
        cycle_counts,
    }
}

async fn get_battery_plist_key(provider: &UsbmuxdProvider) -> String {
    let mut lockdownd_client = match openitools_idevice::get_lockdownd_client(provider).await {
        Ok(lockdownd) => lockdownd,
        Err(e) => {
            log::error!("Something went wrong while getting the lockdownd client: {e:?}");
            return String::default();
        }
    };

    let model = get_string_value_or_default(&mut lockdownd_client, Some("ProductType"), None)
        .await
        .unwrap_or("Unknown".into());

    let model_version = model.trim_start_matches("iPhone");
    let model_version_first_num = model_version
        .split_once(',')
        .map(|(code, _)| code.parse::<u8>().unwrap_or_default())
        .unwrap_or_default();

    if model_version_first_num <= 9 {
        // only to iPhone 7 and earlier
        "AppleARMPMUCharger".into()
    } else {
        "AppleSmartBattery".into()
    }
}
