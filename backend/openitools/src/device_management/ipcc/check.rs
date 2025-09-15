use openitools_idevice::UsbmuxdProvider;
use std::sync::Arc;
use tauri::{Emitter, Window};
use tokio::time::{timeout, Duration};

#[tauri::command]
pub async fn check_installing_succeed(window: Window) {
    let provider = match openitools_idevice::get_provider().await {
        Ok(p) => p,
        Err(e) => {
            log::error!("Failed to create a device provider: {e}");
            let _ = window.emit("installation_succeed_status", false);
            return;
        }
    };

    spawn_syslog_listener(window, &provider).await;
}

async fn spawn_syslog_listener(window: Window, provider: &UsbmuxdProvider) {
    let window = Arc::new(window);

    let mut syslog = match openitools_idevice::get_syslog_client(provider).await {
        Ok(s) => s,
        Err(e) => {
            log::error!("{e}");
            return;
        }
    };
    let result = timeout(Duration::from_secs(40), async {
        loop {
            let Ok(log) = syslog.next().await else {
                return false;
            };

            if log.contains("SIM is Ready") {
                log::info!("SIM ready detected in logs.");
                let _ = window.emit("installation_succeed_status", true);
                return true; // signal success
            }
        }
    })
    .await;

    match result {
        Ok(false) => {
            log::warn!("Syslog ended without SIM ready message.");
            let _ = window.emit("installation_succeed_status", false);
        }
        Err(_) => {
            log::error!("Timeout reached without SIM ready.");
            let _ = window.emit("installation_succeed_status", false);
        }
        Ok(_) => {}
    }
}
