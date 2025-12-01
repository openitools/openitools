use std::{future::Future, pin::Pin, sync::Arc, task::ready};

use openitools_idevice::afc::{AfcClient, FileDescriptor};
use tokio::{
    io::{AsyncRead, AsyncSeek},
    sync::Mutex,
};

pub struct AfcFdReader {
    pub fd: u64,
    pub afc: Arc<Mutex<AfcClient>>,
    pub path: String,
}

impl AsyncRead for AfcFdReader {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let mut afc = Box::pin(self.afc.lock());

        let mut afc = ready!(afc.as_mut().poll(cx));

        let mut fd = Box::pin(unsafe { FileDescriptor::new(&mut afc, self.fd, self.path.clone()) });

        fd.as_mut().poll_read(cx, buf)
    }
}

impl AsyncSeek for AfcFdReader {
    fn start_seek(self: Pin<&mut Self>, position: std::io::SeekFrom) -> std::io::Result<()> {
        let mut afc = Box::pin(self.afc.lock());

        let mut afc = ready!(afc.as_mut().poll(cx));

        let mut fd = Box::pin(unsafe { FileDescriptor::new(&mut afc, self.fd, self.path.clone()) });

        fd.as_mut().start_seek(position)
    }

    fn poll_complete(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<u64>> {
        let mut afc = Box::pin(self.afc.lock());

        let mut afc = ready!(afc.as_mut().poll(cx));

        let mut fd = Box::pin(unsafe { FileDescriptor::new(&mut afc, self.fd, self.path.clone()) });

        fd.as_mut().poll_complete(cx)
    }
}
