use crate::{constants::DEFAULT_RELAYED_CONNECTION_PING_TIMEOUT, traits::DuplexStream};
use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    future::{select, Either, Future, FutureExt},
    pin_mut, ready, Stream,
};
use libp2p::PeerId;
use pin_project_lite::pin_project;
use std::{
    cmp::Ordering,
    collections::HashMap,
    hash::{Hash, Hasher},
    pin::Pin,
    task::{Context, Poll, Waker},
};

use utils_log::{error, log};

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::sleep;
#[cfg(all(feature = "wasm", not(test)))]
use gloo_timers::future::sleep;

#[cfg(any(not(feature = "wasm"), test))]
use utils_misc::time::native::current_timestamp;
#[cfg(all(feature = "wasm", not(test)))]
use utils_misc::time::wasm::current_timestamp;

#[derive(Copy, Clone, Eq)]
pub struct RelayConnectionIdentifier {
    id_a: PeerId,
    id_b: PeerId,
}

impl ToString for RelayConnectionIdentifier {
    fn to_string(&self) -> String {
        format!("{} <-> {}", self.id_a.to_string(), self.id_b.to_string())
    }
}

impl Hash for RelayConnectionIdentifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id_a.hash(state);
        self.id_b.hash(state);
    }
}

impl PartialEq for RelayConnectionIdentifier {
    fn eq(&self, other: &Self) -> bool {
        self.id_a.eq(&other.id_a) && self.id_b.eq(&other.id_b)
    }
}

impl TryFrom<(PeerId, PeerId)> for RelayConnectionIdentifier {
    type Error = String;
    fn try_from(val: (PeerId, PeerId)) -> Result<Self, Self::Error> {
        match val.0.cmp(&val.1) {
            Ordering::Equal => Err("Keys must not be equal".into()),
            Ordering::Greater => Ok(RelayConnectionIdentifier {
                id_a: val.0,
                id_b: val.1,
            }),
            Ordering::Less => Ok(RelayConnectionIdentifier {
                id_a: val.1,
                id_b: val.0,
            }),
        }
    }
}

#[repr(u8)]
pub enum MessagePrefix {
    Payload = 0x00,
    StatusMessage = 0x01,
    ConnectionStatus = 0x02,
    WebRTC = 0x03,
}

#[repr(u8)]
pub enum StatusMessage {
    Ping = 0x00,
    Pong = 0x01,
}

#[repr(u8)]
pub enum ConnectionStatusMessage {
    Stop = 0x00,
    Restart = 0x01,
    Upgraded = 0x02,
}

pin_project! {
    pub struct PingFuture {
        waker: Option<Waker>,
        done: bool,
        started: u64,
        ended: Option<u64>
    }
}

impl PingFuture {
    pub fn new() -> Self {
        Self {
            done: false,
            waker: None,
            started: current_timestamp(),
            ended: None,
        }
    }
    pub(super) fn wake(&mut self) -> () {
        self.ended = Some(current_timestamp());
        self.done = true;
        self.waker.take().unwrap().wake();
    }
}

impl Future for PingFuture {
    type Output = u64;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if *this.done {
            return Poll::Ready(this.ended.unwrap() - *this.started);
        }

        *this.waker = Some(cx.waker().clone());

        Poll::Pending
    }
}

pin_project! {
    pub struct Server<St> {
        #[pin]
        a: End<St>,
        a_ended: bool,
        new_stream_a: UnboundedSender<St>,
        #[pin]
        b: End<St>,
        b_ended: bool,
        new_stream_b: UnboundedSender<St>,
    }
}

pin_project! {
    pub struct End<St> {
        ping_requests: HashMap<[u8;4],PingFuture>,
        #[pin]
        st: Option<St>,
        buffered: Option<Box<[u8]>>,
        id: PeerId,
        status_tx: UnboundedSender<Box<[u8]>>,
        #[pin]
        status_rx: UnboundedReceiver<Box<[u8]>>,
        #[pin]
        new_stream_rx: UnboundedReceiver<St>,
        new_stream_tx: UnboundedSender<St>,
        waker: Option<Waker>
    }
}

impl<St: DuplexStream> End<St> {
    pub async fn is_active(&mut self, maybe_timeout: Option<u64>) -> Result<u64, String> {
        let random_value: [u8; 4] = [0u8; 4];

        let fut = PingFuture::new();

        let fut = self.ping_requests.insert(random_value, fut).unwrap();

        self.status_tx
            .unbounded_send(Box::new([
                MessagePrefix::StatusMessage as u8,
                StatusMessage::Ping as u8,
            ]))
            .map_err(|e| e.to_string())?;

        let timeout_duration = maybe_timeout.unwrap_or(DEFAULT_RELAYED_CONNECTION_PING_TIMEOUT as u64);

        let response_timeout = sleep(std::time::Duration::from_millis(timeout_duration as u64)).fuse();

        pin_mut!(response_timeout);

        match select(fut, response_timeout).await {
            Either::Left(x) => Ok(x.0),
            Either::Right(_) => Err("ping timeout".into()),
        }
    }

    pub fn update(&mut self, new_stream: St) -> Result<(), String> {
        match self.new_stream_tx.unbounded_send(new_stream) {
            Ok(()) => (),
            Err(e) => return Err(format!("Failed to queue new stream {}", e.to_string())),
        };

        if let Some(waker) = self.waker.take() {
            waker.wake()
        }

        Ok(())
    }

    pub fn poll(
        self: Pin<&mut Self>,
        other: &mut Pin<&mut End<St>>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<()>, String>> {
        let mut this = self.project();

        *this.waker = Some(cx.waker().clone());

        // 1. try to deliver queued message
        // 2. check for reconnected stream
        // 3. check for status messages to be sent
        // 4. check for recent incoming messages
        // -> repeat
        Poll::Ready(loop {
            if let Some(chunk) = this.buffered.take() {
                // other.st.as_mut().as_pin_mut()
                let mut other_st = Pin::new(other.st.as_mut().unwrap());
                // let mut other_st = unsafe { Pin::new_unchecked(&mut other.st.as_deref_mut().unwrap()) };
                // let foo = other;
                match other_st.as_mut().poll_ready(cx)? {
                    Poll::Ready(()) => other_st.as_mut().start_send(chunk).map_err(|e| e.to_string())?,
                    Poll::Pending => {
                        *this.buffered = Some(chunk);
                        return Poll::Pending;
                    }
                };
            }

            if let Poll::Ready(Some(new_stream)) = this.new_stream_rx.as_mut().poll_next(cx) {
                this.st.set(Some(new_stream));

                // Drop any previously cached message
                *this.buffered = Some(Box::new([
                    MessagePrefix::ConnectionStatus as u8,
                    ConnectionStatusMessage::Restart as u8,
                ]));
                continue;
            }

            if let Poll::Ready(Some(status_message)) = this.status_rx.as_mut().poll_next(cx) {
                *this.buffered = Some(status_message);
                continue;
            }

            match ready!(this.st.as_mut().as_pin_mut().unwrap().poll_next(cx)?) {
                Some(chunk) => {
                    match chunk[0] {
                        x if (x == MessagePrefix::ConnectionStatus as u8
                            && chunk[1] == ConnectionStatusMessage::Stop as u8) =>
                        {
                            // Correct?
                            break Ok(None);
                        }
                        x if (x == MessagePrefix::StatusMessage as u8 && chunk[1] == StatusMessage::Pong as u8) => {
                            // chunk.
                            let id: [u8; 4] = match chunk.len() {
                                2 => [0u8; 4],
                                6 => chunk[2..6].try_into().unwrap(),
                                _ => {
                                    error!(
                                        "Incorrect ping id length. Received {} elements but expected 4",
                                        chunk.len() - 2
                                    );
                                    continue;
                                }
                            };

                            if let Some(entry) = this.ping_requests.get_mut(&id) {
                                entry.wake();
                            }
                        }
                        x if (x == MessagePrefix::StatusMessage as u8 && chunk[1] == StatusMessage::Ping as u8) => {
                            *this.buffered = Some(Box::new([
                                MessagePrefix::StatusMessage as u8,
                                StatusMessage::Pong as u8,
                            ]));
                            continue;
                        }
                        x if (x == MessagePrefix::Payload as u8 || x == MessagePrefix::WebRTC as u8) => {
                            *this.buffered = Some(chunk);
                            continue;
                        }
                        _ => {
                            error!("Received unrecognizable message")
                        }
                    }
                }
                None => break Ok(None),
            }
        })
    }
}

impl<St: DuplexStream> Server<St> {
    fn new(stream_a: St, peer_a: PeerId, stream_b: St, peer_b: PeerId) {
        let (status_ab_tx, status_ab_rx) = mpsc::unbounded::<Box<[u8]>>();
        let (status_ba_tx, status_ba_rx) = mpsc::unbounded::<Box<[u8]>>();

        let (new_stream_a_tx, new_stream_a_tr) = mpsc::unbounded::<St>();
        let (new_stream_b_tx, new_stream_b_rx) = mpsc::unbounded::<St>();
    }

    pub fn get_id(&self) -> RelayConnectionIdentifier {
        assert!(self.a.id.eq(&self.b.id), "Identifier must not be equal");

        (self.a.id.clone(), self.b.id.clone()).try_into().unwrap()
    }
}

impl<St: DuplexStream> Future for Server<St> {
    type Output = Result<(), String>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let mut a_pending = false;
            if !self.a_ended {
                let mut this = self.as_mut().project();

                match this.a.poll(&mut this.b, cx) {
                    Poll::Pending => a_pending = true,
                    Poll::Ready(Err(e)) => {
                        *this.a_ended = true;
                        error!("Error iterating on relayed stream {}", e);
                    }
                    Poll::Ready(Ok(None)) => {
                        *this.a_ended = true;
                    }
                    Poll::Ready(Ok(Some(()))) => (),
                }
            }

            if !self.b_ended {
                let mut this = self.as_mut().project();

                match this.b.poll(&mut this.a, cx) {
                    Poll::Pending => {
                        if a_pending {
                            // None of the substreams can make progress
                            return Poll::Pending;
                        }
                    }
                    Poll::Ready(Err(e)) => {
                        *this.b_ended = true;
                        error!("Error iterating on relayed stream {}", e);
                    }
                    Poll::Ready(Ok(None)) => {
                        *this.b_ended = true;
                    }
                    Poll::Ready(Ok(Some(()))) => (),
                }
            }
        }
    }
}
