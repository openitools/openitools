use regex::Regex;
use rsmobiledevice::{
    device::DeviceClient,
    device_syslog::{filters::FilterPart, LogFilter},
};
use std::{sync::Arc, thread, time::Duration};
use tauri::{Emitter, Window};

#[tauri::command]
pub fn check_installing_succeed(window: Window) {
    let device_client = match DeviceClient::new().and_then(|client| {
        client
            .get_first_device()
            .ok_or(rsmobiledevice::errors::DeviceClientError::DeviceNotFound)
    }) {
        Ok(client) => client,
        Err(e) => {
            log::error!("Failed to initialize device client: {e}");
            let _ = window.emit("installation_succeed_status", false);
            return;
        }
    };

    let regex = match Regex::new(r"\bSIM is Ready\b") {
        Ok(r) => r,
        Err(e) => {
            log::error!("Invalid regex pattern: {e}");
            let _ = window.emit("installation_succeed_status", false);
            return;
        }
    };

    spawn_syslog_listener(window, device_client, regex);
}

fn handle_sim_ready_log(
    window: Arc<Window>,
) -> impl Fn(rsmobiledevice::device_syslog::LogsData<'_>) + Send + Sync + 'static {
    move |_log_data| {
        log::info!("SIM ready detected in logs.");
        let _ = window.emit("installation_succeed_status", true);
    }
}

fn spawn_syslog_listener(
    window: Window,
    device_client: DeviceClient<rsmobiledevice::devices_collection::SingleDevice>,
    regex: Regex,
) {
    let window = Arc::new(window);

    thread::spawn({
        let window = Arc::clone(&window);
        move || {
            let mut syslog = device_client.get_device_syslog();
            syslog.set_filter(LogFilter::OneShot(regex), FilterPart::All);

            let on_timeout = {
                let win = Arc::clone(&window);
                move || {
                    log::warn!("SIM ready not detected within 40s.");
                    let _ = win.emit("installation_succeed_status", false);
                }
            };

            if let Err(e) = syslog.log_to_custom_with_timeout_or_else(
                {
                    let win = Arc::clone(&window);
                    handle_sim_ready_log(win)
                },
                Duration::from_secs(40),
                on_timeout,
            ) {
                log::error!("Syslog monitoring error: {e}");
                let _ = window.emit("installation_succeed_status", false);
            }
        }
    });
}
