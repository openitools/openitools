use std::{net::SocketAddr, str::FromStr as _};

pub use idevice::{
    IdeviceService,
    lockdown::LockdownClient,
    provider::{IdeviceProvider, UsbmuxdProvider},
    usbmuxd::{UsbmuxdAddr, UsbmuxdConnection, UsbmuxdDevice},
};
use idevice::{diagnostics_relay::DiagnosticsRelayClient, syslog_relay::SyslogRelayClient};
use tokio::time::{Duration, sleep};

pub enum Event {
    Connected,
    Disconnected,
}

async fn get_devices() -> Result<Vec<UsbmuxdDevice>, String> {
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

    Ok(devices)
}

async fn get_device() -> Result<UsbmuxdDevice, String> {
    let mut devices = get_devices().await?;

    if devices.is_empty() {
        return Err("devices are empty".into());
    }

    Ok(devices.remove(0))
}

pub async fn get_provider() -> Result<UsbmuxdProvider, String> {
    let device = get_device().await?;

    let muxaddr = UsbmuxdAddr::from_env_var()
        .map_err(|e| format!("failed to create a usbmuxd address from env: {e:?}"))?;

    Ok(device.to_provider(muxaddr, "openitools-idevice"))
}

pub async fn get_lockdownd_client(provider: &UsbmuxdProvider) -> Result<LockdownClient, String> {
    let mut lockdownd_client = LockdownClient::connect(provider)
        .await
        .map_err(|e| format!("failed to connect to lockdownd service: {e:?}"))?;

    lockdownd_client
        .start_session(
            &provider
                .get_pairing_file()
                .await
                .map_err(|e| format!("Failed to get the pairing file: {e:?}"))?,
        )
        .await
        .map_err(|e| format!("Failed to start a new lockdownd session: {e:?}"))?;

    Ok(lockdownd_client)
}

pub async fn get_diag_client(provider: &UsbmuxdProvider) -> Result<DiagnosticsRelayClient, String> {
    DiagnosticsRelayClient::connect(provider)
        .await
        .map_err(|e| format!("failed to connect to lockdownd service: {e:?}"))
}

pub async fn get_device_model(lockdownd_client: &mut LockdownClient) -> Result<String, String> {
    lockdownd_client
        .get_value(Some("ProductType"), None)
        .await
        .map(|v| v.as_string().unwrap_or_default().to_string())
        .map_err(|e| format!("failed to get the device model: {e:?}"))
}

pub async fn get_device_ios_version(
    lockdownd_client: &mut LockdownClient,
) -> Result<String, String> {
    lockdownd_client
        .get_value(Some("ProductVersion"), None)
        .await
        .map(|v| v.as_string().unwrap_or_default().to_string())
        .map_err(|e| format!("failed to get the device model: {e:?}"))
}

pub async fn get_syslog_client(provider: &UsbmuxdProvider) -> Result<SyslogRelayClient, String> {
    SyslogRelayClient::connect(provider)
        .await
        .map_err(|e| format!("failed to create a syslog relay: {e:?}"))
}

pub async fn install_package<Fut>(
    provider: &UsbmuxdProvider,
    data: impl AsRef<[u8]>,
    cb: impl Fn((u64, ())) -> Fut,
) -> Result<(), String>
where
    Fut: std::future::Future<Output = ()>,
{
    idevice::utils::installation::install_bytes_with_callback(
        provider,
        data,
        "jj.ipcc",
        None,
        cb,
        (),
    )
    .await
    .map_err(|e| format!("failed to install package: {e:?}"))
}

async fn is_device_connected() -> Result<(), String> {
    get_device().await.map(|_| ())
}

pub async fn event_subscribe<F, Fut>(func: F) -> !
where
    F: Fn(Event) -> Fut,
    Fut: Future<Output = ()>,
{
    let mut was_connected = false;

    loop {
        let is_connected = is_device_connected().await;
        match is_connected {
            Ok(()) if !was_connected => {
                was_connected = true;
                func(Event::Connected).await;
            }
            Err(_) if was_connected => {
                was_connected = false;
                func(Event::Disconnected).await;
            }
            _ => {} // no change
        }

        sleep(Duration::from_secs(1)).await;
    }
}
