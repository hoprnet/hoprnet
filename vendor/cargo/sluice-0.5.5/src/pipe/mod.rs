//! Asynchronous in-memory byte buffers aimed at producer-consumer problems.
//!
//! Pipes are like byte-oriented channels that implement I/O traits for reading
//! and writing.

use futures_io::{AsyncBufRead, AsyncRead, AsyncWrite};
use std::{
    fmt,
    io,
    pin::Pin,
    task::{Context, Poll},
};

mod chunked;

/// How many chunks should be available in a chunked pipe. Default is 4, which
/// strikes a good balance of low memory usage and throughput.
const DEFAULT_CHUNK_COUNT: usize = 4;

/// Creates a new asynchronous pipe with the default configuration.
///
/// The default implementation guarantees that when writing a slice of bytes,
/// either the entire slice is written at once or not at all. Slices will never
/// be partially written.
pub fn pipe() -> (PipeReader, PipeWriter) {
    let (reader, writer) = chunked::new(DEFAULT_CHUNK_COUNT);

    (PipeReader { inner: reader }, PipeWriter { inner: writer })
}

/// The reading end of an asynchronous pipe.
pub struct PipeReader {
    inner: chunked::Reader,
}

impl AsyncRead for PipeReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl AsyncBufRead for PipeReader {
    #[allow(unsafe_code)]
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
        unsafe { self.map_unchecked_mut(|s| &mut s.inner) }.poll_fill_buf(cx)
    }

    fn consume(mut self: Pin<&mut Self>, amt: usize) {
        Pin::new(&mut self.inner).consume(amt)
    }
}

impl fmt::Debug for PipeReader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("PipeReader")
    }
}

/// The writing end of an asynchronous pipe.
pub struct PipeWriter {
    inner: chunked::Writer,
}

impl AsyncWrite for PipeWriter {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_close(cx)
    }
}

impl fmt::Debug for PipeWriter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("PipeWriter")
    }
}
