## Compile Instructions

### Prerequisites

Before building, install these system packages:

**Install commands:**

- **Arch Linux:**

```fish

        sudo pacman -Syu libimobiledevice libimobiledevice-glue usbmuxd libusbmuxd
```

- **Ubuntu / Debian:**

```fish

        sudo apt update
        sudo apt install libimobiledevice6 libimobiledevice-glue usbmuxd
```

- **Fedora:**

```fish

        sudo dnf install libimobiledevice libimobiledevice-glue usbmuxd
```

---

### Start the `usbmuxd` service

```fish

        sudo systemctl start usbmuxd
        sudo systemctl enable usbmuxd  # Optional: start at boot
```

---

### Rust Toolchain & Tauri CLI

Make sure Rust is installed via [rustup](https://rustup.rs/):

```fish

        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then install the Tauri CLI if your project requires it:

```fish

        cargo install tauri-cli
```

---

### Build & Run

Finally, build and run your project:

```fish

        cargo r # or cargo tauri dev
```

If you want to get a release build do:

```fish

        # the NO_STRIP is for appimage, tauri needs it
        NO_STRIP=true cargo b # or NO_STRIP=true cargo tauri build
```
