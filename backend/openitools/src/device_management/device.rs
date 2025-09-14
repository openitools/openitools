use std::sync::{atomic::AtomicBool, Arc};

use openitools_idevice::{Event, IdeviceProvider};
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

            let device_client = rsmobiledevice::device::DeviceClient::new().and_then(|client| {
                client
                    .get_first_device()
                    .ok_or(rsmobiledevice::errors::DeviceClientError::DeviceNotFound)
            });

            let device_provider = match openitools_idevice::get_provider().await {
                Ok(provider) => provider,
                Err(e) => {
                    log::error!("Something went wrong while getting the device provider: {e:?}");
                    return;
                }
            };

            let mut lockdownd_client =
                match openitools_idevice::get_lockdownd_client(&device_provider).await {
                    Ok(lockdownd) => lockdownd,
                    Err(e) => {
                        log::error!(
                            "Something went wrong while getting the lockdownd client: {e:?}"
                        );
                        return;
                    }
                };

            lockdownd_client
                .start_session(
                    &device_provider
                        .get_pairing_file()
                        .await
                        .unwrap_or_else(|e| panic!("Failed to get the pairing file: {e:?}")),
                )
                .await
                .unwrap_or_else(|e| panic!("Failed to start a new lockdownd session: {e:?}"));

            match device_client {
                Ok(client) => {
                    window
                        .emit(
                            "device_hardware",
                            handle_device_hardware(&mut lockdownd_client).await,
                        )
                        .ok();

                    window
                        .emit("device_storage", handle_device_storage(&client))
                        .ok();

                    window
                        .emit("device_battery", handle_device_battery(&client))
                        .ok();

                    window.emit("device_os", handle_device_os(&client)).ok();
                }

                Err(e) => {
                    log::error!("failed to create a device client, error: {e}");
                }
            }
        }
        Event::Disconnected => {
            println!("disconnected");
            log::info!("device disconnected");
            window.emit("device_status", false).ok();
        }
    })
    .await;
}
