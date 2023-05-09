use crate::constants::DEFAULT_RELAYED_CONNECTION_PING_TIMEOUT;
use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    future::{join, Future, FutureExt, Join},
    ready, Stream,
};
use libp2p::PeerId;
use pin_project_lite::pin_project;
use std::{
    cell::RefCell,
    cmp::Ordering,
    collections::HashMap,
    hash::{Hash, Hasher},
    pin::Pin,
    rc::Rc,
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
        waker: RefCell<Option<Waker>>,
        started: u64,
        ended: RefCell<Option<u64>>,
        #[pin]
        timeout: Pin<Box<dyn Future<Output = ()>>>
    }
}

impl PingFuture {
    pub fn new(timeout: Pin<Box<dyn Future<Output = ()>>>) -> Self {
        Self {
            waker: RefCell::new(None),
            started: current_timestamp(),
            ended: RefCell::new(None),
            timeout,
        }
    }

    pub(super) fn wake(&self) -> () {
        *self.ended.borrow_mut() = Some(current_timestamp());

        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }
}

impl Future for PingFuture {
    type Output = Result<u64, String>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if let Some(end) = *this.ended.borrow() {
            return Poll::Ready(Ok(end - *this.started));
        }

        if let Poll::Ready(()) = this.timeout.poll(cx) {
            return Poll::Ready(Err("timeout".into()));
        }

        *this.waker.borrow_mut() = Some(cx.waker().clone());

        Poll::Pending
    }
}

pin_project! {
    pub struct PollBorrow<F> {
        #[pin]
        fut: Rc<RefCell<F>>
    }
}

impl<F: Future> Future for PollBorrow<F> {
    type Output = F::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let mut borrowed = this.fut.borrow_mut();

        unsafe { Pin::new_unchecked(&mut *borrowed) }.poll(cx)
    }
}

pin_project! {
    pub struct Server<St> {
        #[pin]
        a: End<St>,
        a_ended: bool,
        #[pin]
        b: End<St>,
        b_ended: bool,
    }
}

pin_project! {
    pub struct End<St> {
        ping_requests: Rc<RefCell<HashMap<[u8;4],Rc<RefCell<PingFuture>>>>>,
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
        waker: RefCell<Option<Waker>>,
        ended: bool
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
            ping_requests: Rc::new(RefCell::new(HashMap::new())),
            st: Some(st),
            buffered: None,
            id,
            status_tx,
            status_rx,
            new_stream_rx,
            new_stream_tx,
            waker: RefCell::new(None),
            ended: false,
        }
    }

    pub fn is_active(&self, maybe_timeout: Option<u64>) -> Result<PollBorrow<PingFuture>, String> {
        let random_value: [u8; 4] = [0u8; 4];

        let timeout_duration = maybe_timeout.unwrap_or(DEFAULT_RELAYED_CONNECTION_PING_TIMEOUT as u64);

        let response_timeout = sleep(std::time::Duration::from_millis(timeout_duration as u64)).fuse();

        let fut = Rc::new(RefCell::new(PingFuture::new(Box::pin(response_timeout))));
        self.ping_requests.borrow_mut().insert(random_value, fut.clone());

        self.status_tx
            .unbounded_send(Box::new([
                MessagePrefix::StatusMessage as u8,
                StatusMessage::Ping as u8,
            ]))
            .map_err(|e| e.to_string())?;

        info!("sent ping");

        Ok(PollBorrow { fut })
    }

    pub fn update(&self, new_stream: St) -> Result<(), String> {
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

        *this.waker.borrow_mut() = Some(cx.waker().clone());

        info!("polling future");

        if *this.ended {
            return Poll::Ready(Ok(None));
        }

        // 1. try to deliver queued message
        // 2. check for reconnected stream
        // 3. check for status messages to be sent
        // 4. check for recent incoming messages
        // -> repeat
        Poll::Ready(loop {
            info!("loop iteration");
            if let Some(chunk) = this.buffered.take() {
                info!("taking buffered {:?}", chunk);
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
                                    *this.ended = true;

                                    *this.buffered = Some(chunk);

                                    break Ok(None);
                                }
                                y if y == ConnectionStatusMessage::Restart as u8 => {
                                    *this.buffered = Some(chunk);
                                    continue;
                                }
                                y if y == ConnectionStatusMessage::Upgraded as u8 => {
                                    // Swallow UPGRADED message for backwards-compatibility
                                    break Ok(None);
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

                                    if let Some(entry) = this.ping_requests.borrow().get(&id) {
                                        info!("ping wake");
                                        entry.borrow().wake();
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
                    *this.ended = true;
                    break Ok(None);
                }
            }
        })
    }
}

impl<St: DuplexStream> Server<St> {
    pub fn new(stream_a: St, peer_a: PeerId, stream_b: St, peer_b: PeerId) -> Self {
        let (status_ab_tx, status_ab_rx) = mpsc::unbounded::<Box<[u8]>>();
        let (status_ba_tx, status_ba_rx) = mpsc::unbounded::<Box<[u8]>>();

        Self {
            a: End::new(stream_a, peer_a.clone(), status_ba_tx, status_ab_rx),
            a_ended: false,
            b: End::new(stream_b, peer_b.clone(), status_ab_tx, status_ba_rx),
            b_ended: false,
        }
    }

    pub fn get_id(&self) -> RelayConnectionIdentifier {
        assert!(!self.a.id.eq(&self.b.id), "Identifier must not be equal");

        (self.a.id.clone(), self.b.id.clone()).try_into().unwrap()
    }

    pub async fn is_active(&self, peer: PeerId, maybe_timeout: Option<u64>) -> bool {
        if peer.eq(&self.a.id) {
            return self.a.is_active(maybe_timeout).unwrap().await.is_ok();
        } else if peer.eq(&self.b.id) {
            return self.b.is_active(maybe_timeout).unwrap().await.is_ok();
        }

        false
    }

    pub fn both_active(&self, maybe_timeout: Option<u64>) -> Join<PollBorrow<PingFuture>, PollBorrow<PingFuture>> {
        let res = join(
            self.a.is_active(maybe_timeout).unwrap(),
            self.b.is_active(maybe_timeout).unwrap(),
        );

        res
    }

    pub fn update(&self, peer: PeerId, st: St) -> Result<(), String> {
        if peer.eq(&self.a.id) {
            return self.a.update(st);
        } else if peer.eq(&self.b.id) {
            return self.b.update(st);
        }

        Err("Incorrect peer. None of the stored peers matches the given peer".into())
    }

    pub fn get_id_a(&self) -> PeerId {
        self.a.id.to_owned()
    }

    pub fn get_id_b(&self) -> PeerId {
        self.b.id.to_owned()
    }
}

impl<St: DuplexStream> Future for Server<St> {
    type Output = Result<(), String>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        while !self.a_ended && !self.b_ended {
            info!("server polling iteration");
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

        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use futures::{stream::FusedStream, Sink, Stream, StreamExt};

    use super::*;
    use std::{str::FromStr, time::Duration};

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

            match this.end.poll(&mut this.other_end, cx) {
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
            #[pin]
            ping_fut: Option<PollBorrow<PingFuture>>
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
                ping_fut: None,
            }
        }
    }

    impl<St: DuplexStream> Future for TestingPollActive<St> {
        type Output = ();
        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let mut pending = false;

            {
                let mut this = self.as_mut().project();

                if this.ping_fut.is_none() {
                    *this.ping_fut = Some(this.end.as_mut().is_active(None).unwrap());
                }

                match this.ping_fut.as_mut().as_pin_mut().unwrap().poll(cx) {
                    Poll::Pending => {
                        info!("is_active poll::pending");

                        this.st_send
                            .unbounded_send(Box::new([
                                MessagePrefix::StatusMessage as u8,
                                StatusMessage::Pong as u8,
                            ]))
                            .unwrap();

                        pending = true;
                    }
                    Poll::Ready(_) => {
                        info!("is_active poll::ready");

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

                if let Poll::Pending = this.other_end.poll(&mut this.end, cx) {
                    pending = true;
                }
            }

            {
                let mut this = self.as_mut().project();

                if let Poll::Pending = this.end.poll(&mut this.other_end, cx) {
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

    #[async_std::test]
    async fn wake_ping_future() {
        let now = current_timestamp();

        let timeout = sleep(Duration::from_millis(1000));

        let ping_fut = PingFuture::new(Box::pin(timeout));

        assert!(ping_fut.started >= now);

        ping_fut.wake();

        let res = ping_fut.await;

        assert!(res.is_ok(), "Ping must complete");
    }

    #[async_std::test]
    async fn timeout_ping() {
        let timeout = sleep(Duration::from_millis(5));

        let ping_fut = PingFuture::new(Box::pin(timeout));

        let res = ping_fut.await;

        assert!(res.is_err(), "Ping must end up in a timeout");
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
                Box::new([MessagePrefix::WebRTC as u8, 0, 0, 0, 0]) as Box<[u8]>
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
            vec![Box::new([
                MessagePrefix::ConnectionStatus as u8,
                ConnectionStatusMessage::Restart as u8
            ]) as Box<[u8]>,]
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
            vec![Box::new([MessagePrefix::StatusMessage as u8, StatusMessage::Ping as u8]) as Box<[u8]>]
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
                Box::new([MessagePrefix::WebRTC as u8, 1, 1, 1, 1]) as Box<[u8]>
            ]
        );

        assert_eq!(
            st_b_receive.collect::<Vec<Box<[u8]>>>().await,
            vec![
                Box::new([MessagePrefix::Payload as u8, 0, 0, 0, 0]) as Box<[u8]>,
                Box::new([MessagePrefix::WebRTC as u8, 0, 0, 0, 0]) as Box<[u8]>
            ]
        );
    }
}
