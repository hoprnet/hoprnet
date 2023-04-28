use crate::traits::DuplexStream;
use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    Future,
};
use libp2p::PeerId;
use pin_project_lite::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

pub struct Server<St> {
    a: End<St>,
    new_stream_a: UnboundedSender<St>,
    b: End<St>,
    new_stream_b: UnboundedSender<St>,
}

pin_project! {
    pub struct End<St> {
        st: St,
        buffered: Box<[u8]>,
        id: PeerId,
        #[pin]
        status_rx: UnboundedReceiver<Box<[u8]>>,
        #[pin]
        new_stream: UnboundedReceiver<St>
    }
}

impl<St: DuplexStream> Server<St> {
    fn new(stream_a: St, peer_a: PeerId, stream_b: St, peer_b: PeerId) {
        let (status_ab_tx, status_ab_rx) = mpsc::unbounded::<Box<[u8]>>();
        let (status_ba_tx, status_ba_rx) = mpsc::unbounded::<Box<[u8]>>();

        let (new_stream_a_tx, new_stream_a_tr) = mpsc::unbounded::<St>();
        let (new_stream_b_tx, new_stream_b_rx) = mpsc::unbounded::<St>();
    }
}

impl<St: DuplexStream> Future for Server<St> {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}
