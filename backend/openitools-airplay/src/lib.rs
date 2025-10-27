use airplay::{
    config::{Audio, MacAddr6, Video},
    playback::null::NullDevice,
};
use tokio::net::TcpListener;
mod audio;
mod discovery;
mod features;
mod transport;
mod utils;
mod video;

pub use features::FEATURES;

use crate::{audio::GstAudio, video::GstVideo};

pub async fn start() {
    gst::init().expect("gstreamer initialization");

    let mac_address = utils::random_lla_mac();

    let svc_listener = TcpListener::bind("0.0.0.0:5200").await.unwrap();
    discovery::mdns_broadcast(&mac_address.to_string());

    let cfg = airplay::config::Config::<_, _> {
        mac_addr: mac_address,
        features: FEATURES,
        model: "AppleTV3,2".into(),
        name: "OpeniTools".into(),
        fw_version: "220.68".into(),
        audio: Audio {
            device: GstAudio::default(),
            ..Default::default()
        },
        video: Video {
            device: GstVideo::default(),
            ..Default::default()
        },
        ..Default::default()
    };

    transport::serve_with_rtsp_remap(svc_listener, airplay::rtsp::RouterService::serve(cfg)).await;
}
