use openitools_idevice::Event;
use tauri::Emitter;

use super::handlers::{
    battery::handle_device_battery, hardware::handle_device_hardware, os::handle_device_os,
    storage::handle_device_storage,
};

#[tauri::command]
pub async fn check_device(window: tauri::Window) {
    window.emit("device_status", false).ok();

    openitools_idevice::event_subscribe(async |event| match event {
        Event::Connected => {
            println!("connected");
            log::info!("device connected");
            window.emit("device_status", true).ok();

            let device_provider = match openitools_idevice::get_provider().await {
                Ok(provider) => provider,
                Err(e) => {
                    log::error!("Something went wrong while getting the device provider: {e:?}");
                    return;
                }
            };

            window
                .emit(
                    "device_hardware",
                    handle_device_hardware(&device_provider).await,
                )
                .ok();

            window
                .emit(
                    "device_storage",
                    handle_device_storage(&device_provider).await,
                )
                .ok();

            window
                .emit(
                    "device_battery",
                    handle_device_battery(&device_provider).await,
                )
                .ok();

            window
                .emit("device_os", handle_device_os(&device_provider).await)
                .ok();
        }
        Event::Disconnected => {
            println!("disconnected");
            log::info!("device disconnected");
            window.emit("device_status", false).ok();
        }
    })
    .await;
}
