use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};

use async_channel::Sender;
use futures_lite::io::{self, AsyncBufRead as BufRead, AsyncRead as Read};

/// ReadNotifier forwards [`async_std::io::Read`] and
/// [`async_std::io::BufRead`] to an inner reader. When the
/// ReadNotifier is read from (using `Read`, `ReadExt`, or `BufRead`
/// methods), it sends a single message containing `()` on the
/// channel.
#[pin_project::pin_project]
pub(crate) struct ReadNotifier<B> {
    #[pin]
    reader: B,
    sender: Sender<()>,
    has_been_read: bool,
}

impl<B> fmt::Debug for ReadNotifier<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReadNotifier")
            .field("read", &self.has_been_read)
            .finish()
    }
}

impl<B: Read> ReadNotifier<B> {
    pub(crate) fn new(reader: B, sender: Sender<()>) -> Self {
        Self {
            reader,
            sender,
            has_been_read: false,
        }
    }
}

impl<B: BufRead> BufRead for ReadNotifier<B> {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
        self.project().reader.poll_fill_buf(cx)
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        self.project().reader.consume(amt)
    }
}

impl<B: Read> Read for ReadNotifier<B> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.project();

        if !*this.has_been_read {
            if let Ok(()) = this.sender.try_send(()) {
                *this.has_been_read = true;
            };
        }

        this.reader.poll_read(cx, buf)
    }
}
