use crate::chunked::ChunkedDecoder;
use async_dup::{Arc, Mutex};
use futures_lite::io::{AsyncRead as Read, BufReader, Take};
use std::{
    fmt::Debug,
    io,
    pin::Pin,
    task::{Context, Poll},
};

pub enum BodyReader<IO: Read + Unpin> {
    Chunked(Arc<Mutex<ChunkedDecoder<BufReader<IO>>>>),
    Fixed(Arc<Mutex<Take<BufReader<IO>>>>),
    None,
}

impl<IO: Read + Unpin> Debug for BodyReader<IO> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BodyReader::Chunked(_) => f.write_str("BodyReader::Chunked"),
            BodyReader::Fixed(_) => f.write_str("BodyReader::Fixed"),
            BodyReader::None => f.write_str("BodyReader::None"),
        }
    }
}

impl<IO: Read + Unpin> Read for BodyReader<IO> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        match &*self {
            BodyReader::Chunked(r) => Pin::new(&mut *r.lock()).poll_read(cx, buf),
            BodyReader::Fixed(r) => Pin::new(&mut *r.lock()).poll_read(cx, buf),
            BodyReader::None => Poll::Ready(Ok(0)),
        }
    }
}
