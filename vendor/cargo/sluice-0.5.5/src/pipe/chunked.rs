//! Generally a ring buffer is an efficient and appropriate data structure for
//! asynchronously transmitting a stream of bytes between two threads that also
//! gives you control over memory allocation to avoid consuming an unknown
//! amount of memory. Setting a fixed memory limit also gives you a degree of
//! flow control if the producer ends up being faster than the consumer.
//!
//! But for some use cases a ring buffer will not work if an application uses
//! its own internal buffer management and requires you to consume either all of
//! a "chunk" of bytes, or none of it.
//!
//! Because of these constraints, instead we use a quite unique type of buffer
//! that uses a fixed number of growable buffers that are exchanged back and
//! forth between a producer and a consumer. Since each buffer is a vector, it
//! can grow to whatever size is required of it in order to fit a single chunk.
//!
//! To avoid the constant allocation overhead of creating a new buffer for every
//! chunk, after a consumer finishes reading from a buffer, it returns the
//! buffer to the producer over a channel to be reused. The number of buffers
//! available in this system is fixed at creation time, so the only allocations
//! that happen during reads and writes are occasional reallocation for each
//! individual vector to fit larger chunks of bytes that don't already fit.

use async_channel::{bounded, Sender, Receiver};
use futures_core::{FusedStream, Stream};
use futures_io::{AsyncBufRead, AsyncRead, AsyncWrite};
use std::{
    io,
    io::{BufRead, Cursor, Write},
    pin::Pin,
    task::{Context, Poll},
};

/// Create a new chunked pipe with room for a fixed number of chunks.
///
/// The `count` parameter sets how many buffers are available in the pipe at
/// once. Smaller values will reduce the number of allocations and reallocations
/// may be required when writing and reduce overall memory usage. Larger values
/// reduce the amount of waiting done between chunks if you have a producer and
/// consumer that run at different speeds.
///
/// If `count` is set to 1, then the pipe is essentially serial, since only the
/// reader or writer can operate on the single buffer at one time and cannot be
/// run in parallel.
pub(crate) fn new(count: usize) -> (Reader, Writer) {
    let (buf_pool_tx, buf_pool_rx) = bounded(count);
    let (buf_stream_tx, buf_stream_rx) = bounded(count);

    // Fill up the buffer pool.
    for _ in 0..count {
        buf_pool_tx
            .try_send(Cursor::new(Vec::new()))
            .expect("buffer pool overflow");
    }

    let reader = Reader {
        buf_pool_tx,
        buf_stream_rx,
        chunk: None,
    };

    let writer = Writer {
        buf_pool_rx,
        buf_stream_tx,
    };

    (reader, writer)
}

/// The reading half of a chunked pipe.
pub(crate) struct Reader {
    /// A channel of incoming chunks from the writer.
    buf_pool_tx: Sender<Cursor<Vec<u8>>>,

    /// A channel of chunk buffers that have been consumed and can be reused.
    buf_stream_rx: Receiver<Cursor<Vec<u8>>>,

    /// A chunk currently being read from.
    chunk: Option<Cursor<Vec<u8>>>,
}

impl AsyncRead for Reader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        // Read into the internal buffer.
        match self.as_mut().poll_fill_buf(cx)? {
            // Not quite ready yet.
            Poll::Pending => Poll::Pending,

            // A chunk is available.
            Poll::Ready(chunk) => {
                // Copy as much of the chunk as we can to the destination
                // buffer.
                let amt = buf.write(chunk)?;

                // Mark however much was successfully copied as being consumed.
                self.consume(amt);

                Poll::Ready(Ok(amt))
            }
        }
    }
}

impl AsyncBufRead for Reader {
    fn poll_fill_buf(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
        // If the current chunk is consumed, first return it to the writer for
        // reuse.
        if let Some(chunk) = self.chunk.as_ref() {
            if chunk.position() >= chunk.get_ref().len() as u64 {
                let mut chunk = self.chunk.take().unwrap();
                chunk.set_position(0);
                chunk.get_mut().clear();

                if let Err(e) = self.buf_pool_tx.try_send(chunk) {
                    // We pre-fill the buffer pool channel with an exact number
                    // of buffers, so this can never happen.
                    if e.is_full() {
                        panic!("buffer pool overflow")
                    }
                    // If the writer disconnects, then we'll just discard this
                    // buffer and any subsequent buffers until we've read
                    // everything still in the pipe.
                    else if e.is_closed() {
                        // Nothing!
                    }
                    // Some other error occurred.
                    else {
                        return Poll::Ready(Err(io::ErrorKind::BrokenPipe.into()));
                    }
                }
            }
        }

        // If we have no current chunk, then attempt to read one.
        if self.chunk.is_none() {
            // If the stream has terminated, then do not poll it again.
            if self.buf_stream_rx.is_terminated() {
                return Poll::Ready(Ok(&[]));
            }

            match Pin::new(&mut self.buf_stream_rx).poll_next(cx) {
                // Wait for a new chunk to be delivered.
                Poll::Pending => return Poll::Pending,

                // Pipe has closed, so return EOF.
                Poll::Ready(None) => return Poll::Ready(Ok(&[])),

                // Accept the new chunk.
                Poll::Ready(buf) => self.chunk = buf,
            }
        }

        // Return the current chunk, if any, as the buffer.
        #[allow(unsafe_code)]
        Poll::Ready(match unsafe { self.get_unchecked_mut().chunk.as_mut() } {
            Some(chunk) => chunk.fill_buf(),
            None => Ok(&[]),
        })
    }

    fn consume(mut self: Pin<&mut Self>, amt: usize) {
        if let Some(chunk) = self.chunk.as_mut() {
            // Consume the requested amount from the current chunk.
            chunk.consume(amt);
        }
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        // Ensure we close the primary stream first before the pool stream so
        // that the writer knows the pipe is closed before trying to poll the
        // pool channel.
        self.buf_stream_rx.close();
        self.buf_pool_tx.close();
    }
}

/// Writing half of a chunked pipe.
pub(crate) struct Writer {
    /// A channel of chunks to send to the reader.
    buf_pool_rx: Receiver<Cursor<Vec<u8>>>,

    /// A channel of incoming buffers to write chunks to.
    buf_stream_tx: Sender<Cursor<Vec<u8>>>,
}

impl AsyncWrite for Writer {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        // If the reading end of the pipe is closed then return an error now,
        // otherwise we'd be spending time writing the entire buffer only to
        // discover that it is closed afterward.
        if self.buf_stream_tx.is_closed() {
            return Poll::Ready(Err(io::ErrorKind::BrokenPipe.into()));
        }

        // Do not send empty buffers through the rotation.
        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }

        // Attempt to grab an available buffer to write the chunk to.
        match Pin::new(&mut self.buf_pool_rx).poll_next(cx) {
            // Wait for the reader to finish reading a chunk.
            Poll::Pending => Poll::Pending,

            // Pipe has closed.
            Poll::Ready(None) => Poll::Ready(Err(io::ErrorKind::BrokenPipe.into())),

            // An available buffer has been found.
            Poll::Ready(Some(mut chunk)) => {
                // Write the buffer to the chunk.
                chunk.get_mut().extend_from_slice(buf);

                // Send the chunk to the reader.
                match self.buf_stream_tx.try_send(chunk) {
                    Ok(()) => Poll::Ready(Ok(buf.len())),

                    Err(e) => {
                        if e.is_full() {
                            panic!("buffer pool overflow")
                        } else {
                            Poll::Ready(Err(io::ErrorKind::BrokenPipe.into()))
                        }
                    }
                }
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.buf_stream_tx.close();
        Poll::Ready(Ok(()))
    }
}
