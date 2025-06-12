use rsmobiledevice::device::Event;
use tauri::Emitter;

use super::handlers::{
    battery::handle_device_battery, hardware::handle_device_hardware, os::handle_device_os,
    storage::handle_device_storage,
};

#[tauri::command]
pub fn check_device(window: tauri::Window) {
    window.emit("device_status", false).ok();

    rsmobiledevice::device::event_subscribe(move |event| match event {
        Event::Connect => {
            println!("connected");
            log::info!("device connected");
            window.emit("device_status", true).ok();

            let device_client = rsmobiledevice::device::DeviceClient::new()
                .and_then(|client| {
                    client
                        .get_first_device()
                        .ok_or(rsmobiledevice::errors::DeviceClientError::DeviceNotFound)
                })
                .unwrap();

            window
                .emit("device_hardware", handle_device_hardware(&device_client))
                .ok();

            window
                .emit("device_storage", handle_device_storage(&device_client))
                .ok();

            window
                .emit("device_battery", handle_device_battery(&device_client))
                .ok();

            window
                .emit("device_os", handle_device_os(&device_client))
                .ok();
        }
        Event::Disconnect => {
            println!("disconnected");
            log::info!("device disconnected");
            window.emit("device_status", false).ok();
        }
        Event::Pair => {}
    })
    .unwrap();
}
