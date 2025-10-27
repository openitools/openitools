use std::{convert::Infallible, sync::RwLock};

use airplay::playback::{
    ChannelHandle, Device, Stream,
    audio::AudioDevice,
    video::{PacketKind, VideoDevice, VideoPacket, VideoParams},
};
use gst::{
    ffi::GstPadIterIntLinkFunction,
    glib::object::{Cast as _, ObjectExt},
    prelude::{ElementExt, GstBinExtManual as _},
};
use gst_app::{AppSink, AppSinkCallbacks};
use gst_plugin_webrtc::webrtcsink::WebRTCSinkCongestionControl;

#[derive(Default)]
pub struct GstVideo {
    id: u64,
    gst_handler: RwLock<Option<GstHandler>>,
}

struct GstHandler {
    pipeline: gst::Pipeline,
    appsrc: gst_app::AppSrc,
}

impl Drop for GstHandler {
    fn drop(&mut self) {
        self.pipeline.set_state(gst::State::Null).unwrap();
    }
}

impl Device for GstVideo {
    type Params = VideoParams;
    type Stream = Self;
    type Error = Infallible;
    fn create(
        &self,
        id: u64,
        params: Self::Params,
        handle: std::sync::Weak<dyn ChannelHandle>,
    ) -> impl Future<Output = Result<Self::Stream, Self::Error>> + Send {
        // if let Some(up) = handle.upgrade() {
        //     up.close();
        // }
        println!("creating video device");
        async move {
            Ok(Self {
                id,
                gst_handler: RwLock::new(None),
            })
        }
    }
}

impl VideoDevice for GstVideo {}

fn create_gst_stream(avcc: impl AsRef<[u8]> + Send + 'static) -> GstHandler {
    let pipeline = gst::Pipeline::new();

    let caps = gst::Caps::builder("video/x-h264")
        .field("stream-format", "avc")
        .field("alignment", "au")
        .field("codec_data", gst::Buffer::from_slice(avcc))
        .build();

    let appsrc = gst_app::AppSrc::builder()
        .caps(&caps)
        .format(gst::Format::Time)
        .is_live(true)
        .do_timestamp(true)
        .build();

    let h264parse_in = gst::ElementFactory::make("h264parse").build().unwrap();
    let decoder = gst::ElementFactory::make("avdec_h264").build().unwrap();
    let videoconvert = gst::ElementFactory::make("videoconvert").build().unwrap();

    // let sink = gst::ElementFactory::make("autovideosink").build().unwrap();
    let sink = gst::ElementFactory::make("webrtcsink")
        .property("stun-server", None::<String>)
        .property("run-signalling-server", true)
        .property("do-fec", false)
        .property("do-retransmission", false)
        .property_from_str("congestion-control", "disabled")
        .build()
        .unwrap();

    // let sink = gst::ElementFactory::make("autovideosink").build().unwrap();
    // let sink = gst::ElementFactory::make("appsink").build().unwrap();
    // let appsink = sink.dynamic_cast_ref::<AppSink>().unwrap();
    // appsink.set_caps(Some(
    //     &gst::Caps::builder("video/x-raw")
    //         .field("format", "RGB")
    //         .field("pixel-aspect-ratio", "1/1")
    //         .build(),
    // ));
    // appsink.set_property("emit-signals", true);
    // appsink.set_property("sync", false); // Disable vsync for faster pulling
    //
    // appsink.set_callbacks(
    //     AppSinkCallbacks::builder()
    //         .new_sample(move |appsink| {
    //             let sample = appsink.pull_sample().unwrap();
    //             let buffer = sample.buffer_owned().ok_or(gst::FlowError::Error).unwrap();
    //             let map = buffer.map_readable().unwrap();
    //             let data: Vec<u8> = map.as_slice().to_vec(); // Raw RGB bytes
    //
    //             let info = sample.caps().ok_or(gst::FlowError::Error).unwrap();
    //             let width = info.structure(0).unwrap().get::<i32>("width").unwrap();
    //             let height = info.structure(0).unwrap().get::<i32>("height").unwrap();
    //
    //             // Emit to frontend
    //             // window.emit("video-frame", (width, height, data)).unwrap();
    //
    //             Ok(gst::FlowSuccess::Ok)
    //         })
    //         .build(),
    // );
    pipeline
        .add_many([
            appsrc.upcast_ref(),
            &h264parse_in,
            &decoder,
            &videoconvert,
            &sink,
        ])
        .unwrap();

    gst::Element::link_many([
        appsrc.upcast_ref(),
        &h264parse_in,
        &decoder,
        &videoconvert,
        &sink,
    ])
    .unwrap();

    pipeline.set_state(gst::State::Playing).unwrap();

    GstHandler { pipeline, appsrc }
}

impl Stream for GstVideo {
    type Content = VideoPacket;
    fn on_data(&self, content: Self::Content) {
        let VideoPacket { kind, payload, .. } = content;

        // println!("incoming data stream");

        match kind {
            PacketKind::AvcC => {
                // println!("incoming avcc data");
                let gst_handler = create_gst_stream(payload);
                *self.gst_handler.write().unwrap() = Some(gst_handler);
            }

            PacketKind::Payload => {
                // println!("incoming payload data");
                if let Ok(gst_handler_lock) = self.gst_handler.read() {
                    if let Some(gst_handler) = gst_handler_lock.as_ref() {
                        gst_handler
                            .appsrc
                            .push_buffer(gst::Buffer::from_slice(payload))
                            .unwrap();
                    }
                }
            }
            PacketKind::Other(_) => {
                // println!("other data")
            }
        }
    }
    fn on_ok(self) {}
    fn on_err(self, err: Box<dyn std::error::Error>) {}
}
