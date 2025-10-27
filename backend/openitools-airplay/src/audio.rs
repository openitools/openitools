use std::{convert::Infallible, sync::RwLock};

use airplay::playback::{
    ChannelHandle, Device, Stream,
    audio::{AudioDevice, AudioPacket, AudioParams},
    video::{PacketKind, VideoDevice, VideoPacket, VideoParams},
};
use gst::{
    glib::object::Cast as _,
    prelude::{ElementExt, GstBinExtManual as _},
};

#[derive(Default)]
pub struct GstAudio {
    id: u64,
}

impl Device for GstAudio {
    type Params = AudioParams;
    type Stream = Self;
    type Error = Infallible;
    fn create(
        &self,
        id: u64,
        params: Self::Params,
        handle: std::sync::Weak<dyn ChannelHandle>,
    ) -> impl Future<Output = Result<Self::Stream, Self::Error>> + Send {
        println!("creating audio device");
        if let Some(up) = handle.upgrade() {
            up.close();
        }
        async move { Ok(Self { id }) }
    }
}

impl AudioDevice for GstAudio {
    fn get_volume(&self) -> f32 {
        println!("gets the volume");
        100.0
    }

    fn set_volume(&self, value: f32) {
        println!("setting the volume");
    }
}

impl Stream for GstAudio {
    type Content = AudioPacket;
    fn on_data(&self, content: Self::Content) {
        println!("incoming audio data");
    }
    fn on_ok(self) {
        println!("audio ok");
    }
    fn on_err(self, err: Box<dyn std::error::Error>) {
        eprintln!("audio error: {err:#?}");
    }
}
