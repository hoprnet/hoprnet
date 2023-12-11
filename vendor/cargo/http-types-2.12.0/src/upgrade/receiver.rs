use futures_lite::Stream;

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::upgrade::Connection;

/// The receiving half of a channel to send an upgraded connection.
#[must_use = "Futures do nothing unless polled or .awaited"]
#[derive(Debug)]
pub struct Receiver {
    receiver: async_channel::Receiver<Connection>,
}

impl Receiver {
    /// Create a new instance of `Receiver`.
    #[allow(unused)]
    pub(crate) fn new(receiver: async_channel::Receiver<Connection>) -> Self {
        Self { receiver }
    }
}

impl Future for Receiver {
    type Output = Option<Connection>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.receiver).poll_next(cx)
    }
}
