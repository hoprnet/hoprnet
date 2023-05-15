use crate::constants::DEFAULT_RELAYED_CONNECTION_PING_TIMEOUT;
use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    future::{Future, FutureExt},
    ready, Stream,
};
use libp2p::PeerId;
use pin_project_lite::pin_project;
use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    pin::Pin,
    str::FromStr,
    task::{Context, Poll, Waker},
};
use utils_log::{error, info};
use utils_misc::traits::DuplexStream;

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::sleep;
#[cfg(all(feature = "wasm", not(test)))]
use gloo_timers::future::sleep;

#[cfg(any(not(feature = "wasm"), test))]
use utils_misc::time::native::current_timestamp;
#[cfg(all(feature = "wasm", not(test)))]
use utils_misc::time::wasm::current_timestamp;

static RELAY_CONNECTION_IDENTIFIER_SEPERATOR: &str = " <-> ";

#[derive(Copy, Clone, Eq)]
pub struct RelayConnectionIdentifier {
    id_a: PeerId,
    id_b: PeerId,
}

impl ToString for RelayConnectionIdentifier {
    fn to_string(&self) -> String {
        format!(
            "{}{}{}",
            self.id_a.to_string(),
            RELAY_CONNECTION_IDENTIFIER_SEPERATOR,
            self.id_b.to_string()
        )
    }
}

impl FromStr for RelayConnectionIdentifier {
    type Err = String;
    ///
    /// ```rust
    /// use connect_relay::server_new::RelayConnectionIdentifier;
    /// use libp2p::PeerId;
    /// use std::str::FromStr;
    ///
    /// const ALICE: &'static str = "1Ank4EeHLAd3bwwtJma1WsXYSmiGgqmjkQoCUpevx67ix8";
    /// const BOB: &'static str = "1AcPsXRKVc3U64NBb4obUUT34jSLWtvAz2JMw192L92QKW";
    ///
    /// let id: RelayConnectionIdentifier = (PeerId::from_str(ALICE).unwrap(), PeerId::from_str(BOB).unwrap()).try_into().unwrap();
    ///
    /// assert!(RelayConnectionIdentifier::from_str(id.to_string().as_str()).unwrap().eq(&id));
    /// ```
    fn from_str(s: &str) -> Result<Self, String> {
        let ids = s.split(RELAY_CONNECTION_IDENTIFIER_SEPERATOR).collect::<Vec<_>>();

        // A valid identifier contains exactly 2 PeerIds
        if ids.len() != 2 {
            return Err("Not a valid relay connection identifier".into());
        }

        let a = PeerId::from_str(ids[0]).map_err(|e| e.to_string())?;
        let b = PeerId::from_str(ids[1]).map_err(|e| e.to_string())?;

        RelayConnectionIdentifier::try_from((a, b))
    }
}

impl Hash for RelayConnectionIdentifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id_a.hash(state);
        self.id_b.hash(state);
    }
}

impl PartialEq for RelayConnectionIdentifier {
    ///
    /// ```rust
    /// use connect_relay::server_new::RelayConnectionIdentifier;
    /// use libp2p::PeerId;
    /// use std::str::FromStr;
    ///
    /// const ALICE: &'static str = "1Ank4EeHLAd3bwwtJma1WsXYSmiGgqmjkQoCUpevx67ix8";
    /// const BOB: &'static str = "1AcPsXRKVc3U64NBb4obUUT34jSLWtvAz2JMw192L92QKW";
    ///
    /// let id: RelayConnectionIdentifier = (PeerId::from_str(ALICE).unwrap(), PeerId::from_str(BOB).unwrap()).try_into().unwrap();
    /// let id_copy: RelayConnectionIdentifier = (PeerId::from_str(ALICE).unwrap(), PeerId::from_str(BOB).unwrap()).try_into().unwrap();
    ///
    /// assert!(id.eq(&id_copy));
    /// ```
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
    /// Message is a payload and need to be forwarded
    Payload = 0x00,
    /// Message is a status message and handles the node <-> relay relationship
    StatusMessage = 0x01,
    /// Message is a connection status message and handles the state of the
    /// link between initiator and destination
    ConnectionStatus = 0x02,
    /// Message contains WebRTC signalling information and need to be forwarded
    WebRTC = 0x03,
}

#[repr(u8)]
pub enum StatusMessage {
    /// Used to see if stream counterparty is active
    Ping = 0x00,
    /// Used to signal to relay that counterparty is active
    Pong = 0x01,
}

#[repr(u8)]
pub enum ConnectionStatusMessage {
    /// Notifies the relay and the other relay end to end the stream
    Stop = 0x00,
    /// Notifies the other relay end about a reconnect.
    /// This is used to redo the end-to-end connection handshake.
    Restart = 0x01,
    /// Notifies the relay that the connection has been upgraded and the
    /// relayed connection is no longer needed
    Upgraded = 0x02,
}

pin_project! {
    /// Holds one end of a relayed connections
    pub struct End<St> {
        // Holds the stream
        // to be set and overwritten at runtime
        #[pin]
        st: Option<St>,
        // chunk fetched from stream
        // to be sent soon
        buffered: Option<Box<[u8]>>,
        // PeerId of the destination
        id: PeerId,
        // send status messages to other end
        status_tx: UnboundedSender<Box<[u8]>>,
        #[pin]
        // receive status messages from other end
        status_rx: UnboundedReceiver<Box<[u8]>>,
        // receive a new stream (e.g. after a reconnect)
        #[pin]
        new_stream_rx: UnboundedReceiver<St>,
        // take a new stream (e.g. after a reconnect)
        new_stream_tx: UnboundedSender<St>,
        // if true, stream is about to end, i.e. poll_close has been called
        ending: bool,
        // if true, stream is done, no more data is expected
        ended: bool,
        // stores the latest ping timeout
        ping_timeout: Option<Pin<Box<dyn Future<Output = ()>>>>,
        // if there is a running ping request, store its start timestamp
        ping_start: Option<u64>,
        // once the pong response came back, store the current timestamp
        ping_ended: Option<u64>,
        // once the pong response came back, wake the thread to process the result
        ping_waker: Option<Waker>
    }
}

impl<St: DuplexStream> End<St> {
    pub fn new(
        st: St,
        id: PeerId,
        status_tx: UnboundedSender<Box<[u8]>>,
        status_rx: UnboundedReceiver<Box<[u8]>>,
    ) -> Self {
        let (new_stream_tx, new_stream_rx) = mpsc::unbounded::<St>();

        Self {
            st: Some(st),
            buffered: None,
            id,
            status_tx,
            status_rx,
            new_stream_rx,
            new_stream_tx,
            ending: false,
            ended: false,
            ping_timeout: None,
            ping_ended: None,
            ping_start: None,
            ping_waker: None,
        }
    }

    /// Issues a low-level ping to see if the counterparty is responding.
    /// If the request hits a timeout, the stream is considered stale.
    pub fn poll_active(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        maybe_timeout: Option<u64>,
    ) -> Poll<Result<u64, String>> {
        let this = self.project();

        *this.ping_waker = Some(cx.waker().clone());

        if let Some(ping_start) = this.ping_start.take() {
            if let Some(ping_end) = this.ping_ended.take() {
                this.ping_timeout.take();
                return Poll::Ready(Ok(ping_end - ping_start));
            }

            return match this.ping_timeout.as_mut().unwrap().as_mut().poll(cx) {
                Poll::Pending => {
                    *this.ping_start = Some(ping_start);
                    Poll::Pending
                }
                Poll::Ready(()) => {
                    this.ping_timeout.take();
                    Poll::Ready(Err("timeout".into()))
                }
            };
        }

        info!("ping active called no request");

        *this.ping_start = Some(current_timestamp());

        let timeout_duration = maybe_timeout.unwrap_or(DEFAULT_RELAYED_CONNECTION_PING_TIMEOUT as u64);

        *this.ping_timeout = Some(Box::pin(
            sleep(std::time::Duration::from_millis(timeout_duration as u64)).fuse(),
        ));

        this.status_tx
            .unbounded_send(Box::new([
                MessagePrefix::StatusMessage as u8,
                StatusMessage::Ping as u8,
            ]))
            .map_err(|e| e.to_string())?;

        // Poll timeout to register waker
        match this.ping_timeout.as_mut().unwrap().as_mut().poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(()) => {
                // Only happens if timeout set to 0, cleanup stuff
                this.ping_start.take();
                this.ping_timeout.take();
                this.ping_waker.take();
                Poll::Ready(Err("timeout".into()))
            }
        }
    }

    /// Overwrites the incoming stream with a fresh stream, e.g. after a reconnect.
    pub fn update(&self, new_stream: St) -> Result<(), String> {
        match self.new_stream_tx.unbounded_send(new_stream) {
            Ok(()) => (),
            Err(e) => return Err(format!("Failed to queue new stream {}", e.to_string())),
        };

        Ok(())
    }

    /// Consumes the underlying stream and handles protocol messages and
    /// forwards all relevant messages to their destination, including payload
    /// messages.
    ///
    /// Returns `Poll::Ready(Ok(None))` once stream has ended.
    pub fn poll_stream_done(
        self: Pin<&mut Self>,
        other: &mut Pin<&mut End<St>>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<()>, String>> {
        let mut this = self.project();

        if *this.ended {
            return Poll::Ready(Ok(None));
        }

        // 1. try to deliver queued message
        // 2. check for reconnected stream
        // 3. check for status messages to be sent
        // 4. check for recent incoming messages
        // -> repeat
        loop {
            info!("loop iteration");

            if *this.ending {
                let other_st = Pin::new(other.st.as_mut().unwrap());
                info!("calling poll_close");
                match other_st.poll_close(cx) {
                    Poll::Ready(_) => {
                        *this.ended = true;
                        return Poll::Ready(Ok(None));
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }

            if let Some(chunk) = this.buffered.take() {
                println!("taking buffered {:?}", chunk);
                let mut other_st = Pin::new(other.st.as_mut().unwrap());
                match other_st.as_mut().poll_ready(cx)? {
                    Poll::Ready(()) => {
                        if chunk.len() == 2
                            && chunk[0] == MessagePrefix::ConnectionStatus as u8
                            && chunk[1] == ConnectionStatusMessage::Stop as u8
                        {
                            *this.ending = true;
                        }
                        other_st.as_mut().start_send(chunk).map_err(|e| e.to_string())?;

                        // We just sent a STOP message, so close the stream
                        if *this.ending == true {
                            continue;
                        }
                    }
                    Poll::Pending => {
                        // We didn't succeed in sending chunk,
                        // so keep it for a next attempt
                        *this.buffered = Some(chunk);
                        info!("buffered {:?}", this.buffered);

                        return Poll::Pending;
                    }
                };
            }

            info!("took buffered");

            if let Poll::Ready(Some(new_stream)) = this.new_stream_rx.as_mut().poll_next(cx) {
                info!("polling new stream");

                this.st.set(Some(new_stream));

                // Drop any previously cached message
                *this.buffered = Some(Box::new([
                    MessagePrefix::ConnectionStatus as u8,
                    ConnectionStatusMessage::Restart as u8,
                ]));
                continue;
            }

            info!("polled new stream");

            if let Poll::Ready(Some(status_message)) = this.status_rx.as_mut().poll_next(cx) {
                info!("polling status message");

                *this.buffered = Some(status_message);
                continue;
            }

            info!("polled status message");

            match ready!(this.st.as_mut().as_pin_mut().unwrap().poll_next(cx)?) {
                Some(chunk) => {
                    info!("polling from stream {:?}", chunk);

                    match chunk[0] {
                        x if x == MessagePrefix::ConnectionStatus as u8 => {
                            if chunk.len() < 2 {
                                error!("unrecognizable connection status message. message is missing second byte");
                                continue;
                            }

                            match chunk[1] {
                                y if y == ConnectionStatusMessage::Stop as u8 => {
                                    info!("stop received {}", x);

                                    // Correct?

                                    *this.buffered = Some(chunk);

                                    continue;
                                }
                                y if y == ConnectionStatusMessage::Restart as u8 => {
                                    *this.buffered = Some(chunk);
                                    continue;
                                }
                                y if y == ConnectionStatusMessage::Upgraded as u8 => {
                                    // Swallow UPGRADED message for backwards-compatibility
                                    // *this.ended = true;
                                    continue;
                                }
                                y => error!("unrecognizable connection status message [{}]", y),
                            }
                        }
                        x if x == MessagePrefix::StatusMessage as u8 => {
                            if chunk.len() < 2 {
                                error!("unrecognizable status message. message is missing second byte");
                                continue;
                            }

                            match chunk[1] {
                                y if y == StatusMessage::Pong as u8 => {
                                    info!("pong received {}", x);

                                    if let Some(waker) = this.ping_waker.take() {
                                        info!("ping wake");
                                        *this.ping_ended = Some(current_timestamp());
                                        waker.wake();
                                    }
                                }
                                y if y == StatusMessage::Ping as u8 => {
                                    info!("ping received {}", x);

                                    *this.buffered = Some(Box::new([
                                        MessagePrefix::StatusMessage as u8,
                                        StatusMessage::Pong as u8,
                                    ]));
                                    continue;
                                }
                                y => error!("Unrecognizable status message [{}]", y),
                            }
                        }
                        x if (x == MessagePrefix::Payload as u8 || x == MessagePrefix::WebRTC as u8) => {
                            *this.buffered = Some(chunk);
                            continue;
                        }
                        x => {
                            info!("Received unrecognizable message [{}]", x)
                        }
                    }
                }
                None => {
                    info!("ended");
                    *this.ending = true;
                    continue;
                }
            }
        }
    }
}

pin_project! {
    /// Holds two streams that form a relayed connections
    /// from peer_a <-> relay <-> peer_b
    pub struct Server<St> {
        // stream from a->b
        #[pin]
        a: End<St>,
        // stream from b->a
        #[pin]
        b: End<St>,
    }
}

impl<St: DuplexStream> Server<St> {
    /// Creates a new relay connection
    ///
    /// This method will not `poll` the streams.
    pub fn new(stream_a: St, peer_a: PeerId, stream_b: St, peer_b: PeerId) -> Self {
        let (status_ab_tx, status_ab_rx) = mpsc::unbounded::<Box<[u8]>>();
        let (status_ba_tx, status_ba_rx) = mpsc::unbounded::<Box<[u8]>>();

        Self {
            a: End::new(stream_a, peer_a.clone(), status_ba_tx, status_ab_rx),
            b: End::new(stream_b, peer_b.clone(), status_ab_tx, status_ba_rx),
        }
    }

    /// Gets the id of the relay connections
    pub fn get_id(&self) -> RelayConnectionIdentifier {
        assert!(!self.a.id.eq(&self.b.id), "Identifier must not be equal");

        (self.a.id.clone(), self.b.id.clone()).try_into().unwrap()
    }

    /// Checks if the underlying stream to the given peer is active.
    /// This issues a low-level ping request to the given peer.
    pub fn poll_active(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        peer: PeerId,
        maybe_timeout: Option<u64>,
    ) -> Poll<bool> {
        let this = self.project();

        if peer.eq(&this.a.id) {
            match this.a.poll_active(cx, maybe_timeout) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Ok(_)) => Poll::Ready(true),
                Poll::Ready(Err(_)) => Poll::Ready(false),
            }
        } else if peer.eq(&this.b.id) {
            match this.b.poll_active(cx, maybe_timeout) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Ok(_)) => Poll::Ready(true),
                Poll::Ready(Err(_)) => Poll::Ready(false),
            }
        } else {
            Poll::Ready(false)
        }
    }

    /// Checks if both streams are active. Used to prune stale connections.
    /// This issues ping requests send to both peers.
    pub fn poll_both_active(mut self: Pin<&mut Self>, cx: &mut Context<'_>, maybe_timeout: Option<u64>) -> Poll<bool> {
        let mut a_done = false;
        let mut a_active = false; // default
        let peer_a = self.a.id.clone();

        let mut b_done = false;
        let mut b_active = false; // default
        let peer_b = self.b.id.clone();

        let mut pending = false;

        Poll::Ready(loop {
            if !a_done {
                match self.as_mut().poll_active(cx, peer_a, maybe_timeout) {
                    Poll::Ready(res_a) => {
                        a_done = true;
                        a_active = res_a;
                        pending = false;
                    }
                    Poll::Pending => {
                        pending = true;
                    }
                }
            }

            if !b_done {
                match self.as_mut().poll_active(cx, peer_b, maybe_timeout) {
                    Poll::Ready(res_b) => {
                        b_done = true;
                        b_active = res_b;
                        pending = false;
                    }
                    Poll::Pending => {
                        pending = true;
                    }
                }
            }

            if pending {
                return Poll::Pending;
            }

            if a_done && b_done {
                break a_active && b_active;
            }
        })
    }

    /// Overwrites the stream to the given peer. Used to handle reconnects.
    pub fn update(&self, peer: PeerId, st: St) -> Result<(), String> {
        if peer.eq(&self.a.id) {
            return self.a.update(st);
        } else if peer.eq(&self.b.id) {
            return self.b.update(st);
        }

        Err("Incorrect peer. None of the stored peers matches the given peer".into())
    }
}

impl<St: DuplexStream> Future for Server<St> {
    type Output = Result<(), String>;

    /// Future that resolves once the connection has ended.
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(loop {
            let mut pending = false;

            if !self.a.ended {
                let mut this = self.as_mut().project();

                match this.a.poll_stream_done(&mut this.b, cx) {
                    Poll::Pending => {
                        pending = true;
                    }
                    Poll::Ready(Err(e)) => {
                        pending = false;
                        error!("Error iterating on relayed stream {}", e);
                    }
                    Poll::Ready(Ok(None)) => {
                        pending = false;
                    }
                    Poll::Ready(Ok(Some(()))) => (),
                }
            }

            if !self.b.ended {
                let mut this = self.as_mut().project();

                match this.b.poll_stream_done(&mut this.a, cx) {
                    Poll::Pending => {
                        pending = true;
                    }
                    Poll::Ready(Err(e)) => {
                        pending = false;
                        error!("Error iterating on relayed stream {}", e);
                    }
                    Poll::Ready(Ok(None)) => {
                        pending = false;
                    }
                    Poll::Ready(Ok(Some(()))) => (),
                }
            }

            if pending {
                return Poll::Pending;
            }

            if self.a.ended && self.b.ended {
                break Ok(());
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use futures::{stream::FusedStream, Sink, Stream, StreamExt};

    use super::*;
    use std::str::FromStr;

    const ALICE: &'static str = "1Ank4EeHLAd3bwwtJma1WsXYSmiGgqmjkQoCUpevx67ix8";
    const BOB: &'static str = "1AcPsXRKVc3U64NBb4obUUT34jSLWtvAz2JMw192L92QKW";

    pin_project! {
        struct TestingDuplexStream {
            #[pin]
            rx: UnboundedReceiver<Box<[u8]>>,
            #[pin]
            tx: UnboundedSender<Box<[u8]>>,
        }
    }

    impl TestingDuplexStream {
        fn new() -> (Self, UnboundedSender<Box<[u8]>>, UnboundedReceiver<Box<[u8]>>) {
            let (send_tx, send_rx) = mpsc::unbounded::<Box<[u8]>>();
            let (receive_tx, receive_rx) = mpsc::unbounded::<Box<[u8]>>();

            (
                Self {
                    rx: send_rx,
                    tx: receive_tx,
                },
                send_tx,
                receive_rx,
            )
        }
    }

    impl FusedStream for TestingDuplexStream {
        fn is_terminated(&self) -> bool {
            self.rx.is_terminated()
        }
    }

    impl Stream for TestingDuplexStream {
        type Item = Result<Box<[u8]>, String>;
        fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            let this = self.project();

            match this.rx.poll_next(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Some(x)) => Poll::Ready(Some(Ok(x))),
                Poll::Ready(None) => Poll::Ready(None),
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self.rx.size_hint()
        }
    }

    impl Sink<Box<[u8]>> for TestingDuplexStream {
        type Error = String;
        fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            let this = self.project();

            this.tx.poll_ready(cx).map_err(|e| e.to_string())
        }

        fn start_send(self: Pin<&mut Self>, item: Box<[u8]>) -> Result<(), Self::Error> {
            let this = self.project();

            this.tx.start_send(item).map_err(|e| e.to_string())
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            let this = self.project();

            this.tx.poll_flush(cx).map_err(|e| e.to_string())
        }

        fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            let this = self.project();

            this.tx.poll_close(cx).map_err(|e| e.to_string())
        }
    }

    impl DuplexStream for TestingDuplexStream {}

    pin_project! {
        struct TestingPoll< St> {
            #[pin]
            end: End<St>,
            #[pin]
            other_end:End<St>,
        }
    }

    impl<St: DuplexStream> TestingPoll<St> {
        fn new(end: End<St>, other_end: End<St>) -> Self {
            Self { end, other_end }
        }
    }

    impl<St: DuplexStream> Future for TestingPoll<St> {
        type Output = ();
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let mut this = self.project();

            match this.end.poll_stream_done(&mut this.other_end, cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(_) => Poll::Ready(()),
            }
        }
    }

    pin_project! {
        struct TestingPollActive<St> {
            #[pin]
            end: End<St>,
            #[pin]
            other_end:End<St>,
            st_send: UnboundedSender<Box<[u8]>>,
            st_other_send: UnboundedSender<Box<[u8]>>,
            first_attempt: bool
        }
    }

    impl<St: DuplexStream> TestingPollActive<St> {
        fn new(
            end: End<St>,
            other_end: End<St>,
            st_send: UnboundedSender<Box<[u8]>>,
            st_other_send: UnboundedSender<Box<[u8]>>,
        ) -> Self {
            Self {
                end,
                other_end,
                st_send,
                st_other_send,
                first_attempt: true,
            }
        }
    }

    impl<St: DuplexStream> Future for TestingPollActive<St> {
        type Output = ();
        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let mut pending = false;

            {
                let mut this = self.as_mut().project();

                match this.end.as_mut().poll_active(cx, None) {
                    Poll::Pending => {
                        if *this.first_attempt {
                            *this.first_attempt = false;
                            this.st_send
                                .unbounded_send(Box::new([
                                    MessagePrefix::StatusMessage as u8,
                                    StatusMessage::Pong as u8,
                                ]))
                                .unwrap();
                        }

                        pending = true;
                    }
                    Poll::Ready(_) => {
                        this.st_send
                            .unbounded_send(Box::new([
                                MessagePrefix::ConnectionStatus as u8,
                                ConnectionStatusMessage::Stop as u8,
                            ]))
                            .unwrap();

                        this.st_other_send
                            .unbounded_send(Box::new([
                                MessagePrefix::ConnectionStatus as u8,
                                ConnectionStatusMessage::Stop as u8,
                            ]))
                            .unwrap();
                    }
                }
            }

            {
                let mut this = self.as_mut().project();

                if let Poll::Pending = this.other_end.poll_stream_done(&mut this.end, cx) {
                    pending = true;
                }
            }

            {
                let mut this = self.as_mut().project();

                if let Poll::Pending = this.end.poll_stream_done(&mut this.other_end, cx) {
                    pending = true;
                }
            }

            if pending {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        }
    }

    #[test]
    fn test_identifier() {
        assert!(
            (RelayConnectionIdentifier::try_from((PeerId::from_str(ALICE).unwrap(), PeerId::from_str(BOB).unwrap()))
                .is_ok())
        );

        assert!(
            (RelayConnectionIdentifier::try_from((PeerId::from_str(ALICE).unwrap(), PeerId::from_str(ALICE).unwrap()))
                .is_err())
        );

        let ab =
            RelayConnectionIdentifier::try_from((PeerId::from_str(ALICE).unwrap(), PeerId::from_str(BOB).unwrap()))
                .unwrap();

        let ba =
            RelayConnectionIdentifier::try_from((PeerId::from_str(BOB).unwrap(), PeerId::from_str(ALICE).unwrap()))
                .unwrap();

        assert!(ab.eq(&ba));
        assert!(ab.to_string().eq(&ba.to_string()));
    }

    #[async_std::test]
    async fn test_connection_end() {
        let (status_ab_tx, status_ab_rx) = mpsc::unbounded::<Box<[u8]>>();
        let (status_ba_tx, status_ba_rx) = mpsc::unbounded::<Box<[u8]>>();

        let (st, st_send, _) = TestingDuplexStream::new();
        let (st_other, _, st_other_receive) = TestingDuplexStream::new();

        let conn_a = End::new(st, PeerId::from_str(ALICE).unwrap().clone(), status_ba_tx, status_ab_rx);
        let conn_b = End::new(
            st_other,
            PeerId::from_str(ALICE).unwrap().clone(),
            status_ab_tx,
            status_ba_rx,
        );

        st_send
            .unbounded_send(Box::new([MessagePrefix::Payload as u8, 0, 0, 0, 0]))
            .unwrap();

        st_send
            .unbounded_send(Box::new([MessagePrefix::WebRTC as u8, 0, 0, 0, 0]))
            .unwrap();

        st_send
            .unbounded_send(Box::new([
                MessagePrefix::ConnectionStatus as u8,
                ConnectionStatusMessage::Stop as u8,
            ]))
            .unwrap();

        TestingPoll::new(conn_a, conn_b).await;

        assert_eq!(
            st_other_receive.collect::<Vec<Box<[u8]>>>().await,
            vec![
                Box::new([MessagePrefix::Payload as u8, 0, 0, 0, 0]) as Box<[u8]>,
                Box::new([MessagePrefix::WebRTC as u8, 0, 0, 0, 0]) as Box<[u8]>,
                Box::new([
                    MessagePrefix::ConnectionStatus as u8,
                    ConnectionStatusMessage::Stop as u8,
                ]) as Box<[u8]>
            ]
        );
    }

    #[async_std::test]
    async fn test_stream_upgrade() {
        let (status_ab_tx, status_ab_rx) = mpsc::unbounded::<Box<[u8]>>();
        let (status_ba_tx, status_ba_rx) = mpsc::unbounded::<Box<[u8]>>();

        let (st, st_send, _) = TestingDuplexStream::new();
        let (st_other, _, st_other_receive) = TestingDuplexStream::new();

        let conn_a = End::new(st, PeerId::from_str(ALICE).unwrap().clone(), status_ba_tx, status_ab_rx);
        let conn_b = End::new(
            st_other,
            PeerId::from_str(ALICE).unwrap().clone(),
            status_ab_tx,
            status_ba_rx,
        );

        // Should not forward this message
        st_send
            .unbounded_send(Box::new([MessagePrefix::Payload as u8, 0, 0, 0, 0]))
            .unwrap();

        let (st_next, st_next_send, _) = TestingDuplexStream::new();

        assert!(conn_a.update(st_next).is_ok());

        st_next_send
            .unbounded_send(Box::new([
                MessagePrefix::ConnectionStatus as u8,
                ConnectionStatusMessage::Stop as u8,
            ]))
            .unwrap();

        TestingPoll::new(conn_a, conn_b).await;

        assert_eq!(
            st_other_receive.collect::<Vec<Box<[u8]>>>().await,
            vec![
                Box::new([
                    MessagePrefix::ConnectionStatus as u8,
                    ConnectionStatusMessage::Restart as u8
                ]) as Box<[u8]>,
                Box::new([
                    MessagePrefix::ConnectionStatus as u8,
                    ConnectionStatusMessage::Stop as u8,
                ]) as Box<[u8]>
            ]
        );
    }

    #[async_std::test]
    async fn test_is_active() {
        let (status_ab_tx, status_ab_rx) = mpsc::unbounded::<Box<[u8]>>();
        let (status_ba_tx, status_ba_rx) = mpsc::unbounded::<Box<[u8]>>();

        let (st, st_send, st_receive) = TestingDuplexStream::new();
        let (st_other, st_other_send, _) = TestingDuplexStream::new();

        let conn_a = End::new(st, PeerId::from_str(ALICE).unwrap().clone(), status_ba_tx, status_ab_rx);
        let conn_b = End::new(
            st_other,
            PeerId::from_str(ALICE).unwrap().clone(),
            status_ab_tx,
            status_ba_rx,
        );

        TestingPollActive::new(conn_a, conn_b, st_send, st_other_send).await;

        assert_eq!(
            st_receive.collect::<Vec<Box<[u8]>>>().await,
            vec![
                Box::new([MessagePrefix::StatusMessage as u8, StatusMessage::Ping as u8]) as Box<[u8]>,
                Box::new([
                    MessagePrefix::ConnectionStatus as u8,
                    ConnectionStatusMessage::Stop as u8,
                ]) as Box<[u8]>
            ]
        );
    }

    #[async_std::test]
    async fn test_server() {
        let (st_a, st_a_send, st_a_receive) = TestingDuplexStream::new();
        let (st_b, st_b_send, st_b_receive) = TestingDuplexStream::new();

        let server = Server::new(
            st_a,
            PeerId::from_str(ALICE).unwrap(),
            st_b,
            PeerId::from_str(BOB).unwrap(),
        );

        st_a_send
            .unbounded_send(Box::new([MessagePrefix::Payload as u8, 0, 0, 0, 0]))
            .unwrap();

        st_a_send
            .unbounded_send(Box::new([MessagePrefix::WebRTC as u8, 0, 0, 0, 0]))
            .unwrap();

        st_a_send
            .unbounded_send(Box::new([
                MessagePrefix::ConnectionStatus as u8,
                ConnectionStatusMessage::Stop as u8,
            ]))
            .unwrap();

        st_b_send
            .unbounded_send(Box::new([MessagePrefix::Payload as u8, 1, 1, 1, 1]))
            .unwrap();

        st_b_send
            .unbounded_send(Box::new([MessagePrefix::WebRTC as u8, 1, 1, 1, 1]))
            .unwrap();

        st_b_send
            .unbounded_send(Box::new([
                MessagePrefix::ConnectionStatus as u8,
                ConnectionStatusMessage::Stop as u8,
            ]))
            .unwrap();

        assert!(server.await.is_ok());

        assert_eq!(
            st_a_receive.collect::<Vec<Box<[u8]>>>().await,
            vec![
                Box::new([MessagePrefix::Payload as u8, 1, 1, 1, 1]) as Box<[u8]>,
                Box::new([MessagePrefix::WebRTC as u8, 1, 1, 1, 1]) as Box<[u8]>,
                Box::new([
                    MessagePrefix::ConnectionStatus as u8,
                    ConnectionStatusMessage::Stop as u8,
                ]) as Box<[u8]>
            ]
        );

        assert_eq!(
            st_b_receive.collect::<Vec<Box<[u8]>>>().await,
            vec![
                Box::new([MessagePrefix::Payload as u8, 0, 0, 0, 0]) as Box<[u8]>,
                Box::new([MessagePrefix::WebRTC as u8, 0, 0, 0, 0]) as Box<[u8]>,
                Box::new([
                    MessagePrefix::ConnectionStatus as u8,
                    ConnectionStatusMessage::Stop as u8,
                ]) as Box<[u8]>
            ]
        );
    }
}
