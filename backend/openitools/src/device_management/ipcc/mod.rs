pub mod check;
pub mod install;
pub mod prep_file;
pub mod query;
use tauri::Emitter;

pub fn emit_bundle_installation_status(window: &tauri::Window, status: bool) {
    let _ = window.emit("carrier_bundle_install_status", status);
}
