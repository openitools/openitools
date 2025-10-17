mod device_management;
use device_management::{
    device::check_device,
    ipcc::{check::check_installing_succeed, install::install_ipcc, query::get_bundles},
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())
                .expect("Unable to add the tauri updater plugin");

            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            check_device,
            install_ipcc,
            check_installing_succeed,
            get_bundles
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_, event| match event {
            tauri::RunEvent::Exit => {
                println!("hey we exit");
            }
            tauri::RunEvent::ExitRequested { .. } => {}
            _ => {}
        });
}
