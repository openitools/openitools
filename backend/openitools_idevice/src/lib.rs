use std::{net::SocketAddr, str::FromStr as _};

use idevice::usbmuxd::UsbmuxdConnection;
use tokio::time::{Duration, sleep};

pub enum Event {
    Connected,
    Disconnected,
}

async fn is_device_connected() -> Result<(), String> {
    let mut usbmuxd = if let Ok(var) = std::env::var("USBMUXD_SOCKET_ADDRESS") {
        let socket =
            SocketAddr::from_str(&var).map_err(|e| format!("Bad USBMUXD_SOCKET_ADDRESS: {e:?}"))?;
        let socket = tokio::net::TcpStream::connect(socket)
            .await
            .map_err(|e| format!("unable to connect to socket address: {e:?}"))?;
        UsbmuxdConnection::new(Box::new(socket), 1)
    } else {
        UsbmuxdConnection::default()
            .await
            .map_err(|e| format!("Unable to connect to usbmxud: {e:?}"))?
    };
    let devices = usbmuxd
        .get_devices()
        .await
        .map_err(|e| format!("Unable to get devices from usbmuxd: {e:?}"))?;

    if devices.is_empty() {
        return Err("devices are empty".into());
    }

    Ok(())
}

pub async fn event_subscribe<F>(func: F) -> !
where
    F: Fn(Event),
{
    let mut was_connected = false;

    loop {
        let is_connected = is_device_connected().await;
        match is_connected {
            Ok(()) if !was_connected => {
                was_connected = true;
                func(Event::Connected);
            }
            Err(_) if was_connected => {
                was_connected = false;
                func(Event::Disconnected);
            }
            _ => {} // no change
        }

        sleep(Duration::from_millis(100)).await;
    }
}
