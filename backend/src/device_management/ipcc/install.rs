use super::emit_bundle_installation_status;
use super::prep_file;
use super::query::download_bundle;
use rsmobiledevice::RecursiveFind;
use std::{io::Cursor, sync::Arc};

#[tauri::command]
pub async fn install_ipcc(
    window: tauri::Window,
    expected_model: String,
    expected_ios_version: String,
    bundle: String,
) {
    let window = Arc::new(window);

    if let Err(e) = run_installation(
        Arc::clone(&window),
        expected_model,
        expected_ios_version,
        bundle,
    )
    .await
    {
        log::error!("Installation failed: {e}");
        emit_bundle_installation_status(&window, false);
    }
}

async fn run_installation(
    window: Arc<tauri::Window>,
    expected_model: String,
    expected_ios_version: String,
    bundle: String,
) -> Result<(), String> {
    log::info!("IPCC installation started");

    // connect to the device
    let device_client = rsmobiledevice::device::DeviceClient::new()
        .and_then(|client| {
            client
                .get_first_device()
                .ok_or(rsmobiledevice::errors::DeviceClientError::DeviceNotFound)
        })
        // we only need it to log to the stdout
        .map_err(|e| e.to_string())?;

    let device_info = device_client.get_device_info();

    // verify device identity
    let connected_model = device_info
        .get_product_type()
        .map_err(|e| format!("Failed to get device model: {e}"))?;

    let connected_ios_version = device_info
        .get_product_version()
        .map_err(|e| format!("Got model but failed to get iOS version: {e}"))?;

    if expected_model != connected_model || expected_ios_version != connected_ios_version {
        log::info!(
            "Device mismatch: expected {connected_model}:{connected_ios_version}, got {expected_model}:{expected_ios_version}"
        );
        return Err("Connected device does not match expected model/iOS version".into());
    }

    // 3. Download and repackage the bundle
    let tar_bytes = download_bundle(&expected_model, &expected_ios_version, &bundle)
        .await
        .map_err(|e| format!("Failed to download bundle: {e}"))?;

    let zip_bytes = prep_file::repack_tar_to_zip(&tar_bytes).await;
    let mut zip_cursor = Cursor::new(zip_bytes);

    // 4. Install with callback
    let installer = device_client.get_device_installer();
    let window_clone = Arc::clone(&window);

    installer
        .install_from_reader_with_callback(&mut zip_cursor, None, move |command, status| {
            //TODO: log it better
            println!("{command:#?}");
            println!("{status:#?}");

            // emit only when status = "Complete"
            if status.rfind("Status").is_some_and(|s| s == "Complete") {
                emit_bundle_installation_status(&window_clone, true);
            }
        })
        .map_err(|e| format!("Installer error: {e}"))?;

    Ok(())
}
