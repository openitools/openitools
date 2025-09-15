fn main() {
    let libs = ["gtk+-3.0", "webkit2gtk-4.1"];

    for lib in libs {
        if let Err(e) = pkg_config::probe_library(lib) {
            panic!("Missing system library: `{lib}` not found. Error: {e}");
        }
    }
    tauri_build::build();
}
