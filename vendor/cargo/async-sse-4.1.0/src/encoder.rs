use async_std::io::{BufRead as AsyncBufRead, Read as AsyncRead};
use async_std::prelude::*;
use async_std::task::{ready, Context, Poll};

use std::io;
use std::pin::Pin;
use std::time::Duration;

pin_project_lite::pin_project! {
    /// An SSE protocol encoder.
    #[derive(Debug)]
    pub struct Encoder {
        buf: Box<[u8]>,
        cursor: usize,
        #[pin]
        receiver: async_channel::Receiver<Vec<u8>>,
    }
}

impl AsyncRead for Encoder {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let mut this = self.project();
        // Request a new buffer if current one is exhausted.
        if this.buf.len() <= *this.cursor {
            match ready!(this.receiver.as_mut().poll_next(cx)) {
                Some(buf) => {
                    log::trace!("> Received a new buffer with len {}", buf.len());
                    *this.buf = buf.into_boxed_slice();
                    *this.cursor = 0;
                }
                None => {
                    log::trace!("> Encoder done reading");
                    return Poll::Ready(Ok(0));
                }
            };
        }

        // Write the current buffer to completion.
        let local_buf = &this.buf[*this.cursor..];
        let max = buf.len().min(local_buf.len());
        buf[..max].clone_from_slice(&local_buf[..max]);
        *this.cursor += max;

        // Return bytes read.
        Poll::Ready(Ok(max))
    }
}

impl AsyncBufRead for Encoder {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
        let mut this = self.project();
        // Request a new buffer if current one is exhausted.
        if this.buf.len() <= *this.cursor {
            match ready!(this.receiver.as_mut().poll_next(cx)) {
                Some(buf) => {
                    log::trace!("> Received a new buffer with len {}", buf.len());
                    *this.buf = buf.into_boxed_slice();
                    *this.cursor = 0;
                }
                None => {
                    log::trace!("> Encoder done reading");
                    return Poll::Ready(Ok(&[]));
                }
            };
        }
        Poll::Ready(Ok(&this.buf[*this.cursor..]))
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        let this = self.project();
        *this.cursor += amt;
    }
}

/// The sending side of the encoder.
#[derive(Debug, Clone)]
pub struct Sender(async_channel::Sender<Vec<u8>>);

/// Create a new SSE encoder.
pub fn encode() -> (Sender, Encoder) {
    let (sender, receiver) = async_channel::bounded(1);
    let encoder = Encoder {
        receiver,
        buf: Box::default(),
        cursor: 0,
    };
    (Sender(sender), encoder)
}

impl Sender {
    async fn inner_send(&self, bytes: impl Into<Vec<u8>>) -> io::Result<()> {
        self.0
            .send(bytes.into())
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::ConnectionAborted, "sse disconnected"))
    }

    /// Send a new message over SSE.
    pub async fn send(&self, name: &str, data: &str, id: Option<&str>) -> io::Result<()> {
        // Write the event name
        let msg = format!("event:{}\n", name);
        self.inner_send(msg).await?;

        // Write the id
        if let Some(id) = id {
            self.inner_send(format!("id:{}\n", id)).await?;
        }

        // Write the data section, and end.
        let msg = format!("data:{}\n\n", data);
        self.inner_send(msg).await?;

        Ok(())
    }

    /// Send a new "retry" message over SSE.
    pub async fn send_retry(&self, dur: Duration, id: Option<&str>) -> io::Result<()> {
        // Write the id
        if let Some(id) = id {
            self.inner_send(format!("id:{}\n", id)).await?;
        }

        // Write the retry section, and end.
        let dur = dur.as_secs_f64() as u64;
        let msg = format!("retry:{}\n\n", dur);
        self.inner_send(msg).await?;
        Ok(())
    }
}
