use tokio::net::TcpListener;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use tracing_subscriber::fmt::Layer;

mod audio;
mod discovery;
mod playback;
mod transport;
mod video;

#[tokio::main]
async fn main() {
    // let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));
    //
    // tracing_subscriber::registry()
    //     .with(LevelFilter::DEBUG) // <-- sets debug level globally
    //     .with(Layer::default())
    //     .init();

    tracing::info!("hello");

    gstreamer::init().expect("gstreamer initialization");

    let svc_listener = TcpListener::bind("0.0.0.0:5200").await.unwrap();
    discovery::mdns_broadcast();

    let cfg = airplay::config::Config::<_, _> {
        video: airplay::config::Video {
            device: playback::PipeDevice {
                callback: video::transcode,
            },
            ..Default::default()
        },
        audio: airplay::config::Audio {
            device: playback::PipeDevice {
                callback: audio::transcode,
            },
            ..Default::default()
        },
        ..Default::default()
    };

    transport::serve_with_rtsp_remap(svc_listener, airplay::rtsp::RouterService::serve(cfg)).await;
}
