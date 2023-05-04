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

        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
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
        #[pin]
        b: End<St>,
        b_ended: bool,
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
    pub fn new(
        st: St,
        id: PeerId,
        status_tx: UnboundedSender<Box<[u8]>>,
        status_rx: UnboundedReceiver<Box<[u8]>>,
    ) -> Self {
        let (new_stream_tx, new_stream_rx) = mpsc::unbounded::<St>();

        Self {
            ping_requests: HashMap::new(),
            st: Some(st),
            buffered: None,
            id,
            status_tx,
            status_rx,
            new_stream_rx,
            new_stream_tx,
            waker: None,
        }
    }

    pub async fn is_active(&mut self, maybe_timeout: Option<u64>) -> Result<u64, String> {
        let random_value: [u8; 4] = [0u8; 4];

        let fut = PingFuture::new();
        self.ping_requests.insert(random_value, fut);

        self.status_tx
            .unbounded_send(Box::new([
                MessagePrefix::StatusMessage as u8,
                StatusMessage::Ping as u8,
            ]))
            .map_err(|e| e.to_string())?;

        println!("sent ping");
        let timeout_duration = maybe_timeout.unwrap_or(DEFAULT_RELAYED_CONNECTION_PING_TIMEOUT as u64);

        let response_timeout = sleep(std::time::Duration::from_millis(timeout_duration as u64)).fuse();

        pin_mut!(response_timeout);

        match select(self.ping_requests.get_mut(&random_value).unwrap(), response_timeout).await {
            Either::Left(x) => Ok(x.0),
            Either::Right(_) => {
                println!("ping timeout");
                Err("ping timeout".into())
            }
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
            println!("loop iteration");
            if let Some(chunk) = this.buffered.take() {
                println!("taking buffered {:?}", chunk);
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

            println!("took buffered");

            if let Poll::Ready(Some(new_stream)) = this.new_stream_rx.as_mut().poll_next(cx) {
                println!("polling new stream");

                this.st.set(Some(new_stream));

                // Drop any previously cached message
                *this.buffered = Some(Box::new([
                    MessagePrefix::ConnectionStatus as u8,
                    ConnectionStatusMessage::Restart as u8,
                ]));
                continue;
            }

            println!("polled new stream");

            if let Poll::Ready(Some(status_message)) = this.status_rx.as_mut().poll_next(cx) {
                println!("polling status message");

                *this.buffered = Some(status_message);
                continue;
            }

            println!("polled status message");

            match ready!(this.st.as_mut().as_pin_mut().unwrap().poll_next(cx)?) {
                Some(chunk) => {
                    println!("polling from stream");

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
        assert!(self.a.id.eq(&self.b.id), "Identifier must not be equal");

        (self.a.id.clone(), self.b.id.clone()).try_into().unwrap()
    }

    pub async fn is_active(&mut self, peer: PeerId, maybe_timeout: Option<u64>) -> bool {
        if peer.eq(&self.a.id) {
            self.a.is_active(maybe_timeout).await.unwrap();
        } else if peer.eq(&self.b.id) {
            self.b.is_active(maybe_timeout).await.unwrap();
        }

        false
    }

    pub fn update(&mut self, peer: PeerId, st: St) -> Result<(), String> {
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

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        while !self.a_ended && !self.b_ended {
            println!("server polling iteration");
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
    use std::str::FromStr;

    const ALICE: &'static str = "1Ank4EeHLAd3bwwtJma1WsXYSmiGgqmjkQoCUpevx67ix8";
    const BOB: &'static str = "1AcPsXRKVc3U64NBb4obUUT34jSLWtvAz2JMw192L92QKW";
    // relays need a proper name
    const RYAN: &'static str = "1Ag7Agu1thSZFRGyoPWydqCiFS6tJFXLeukpVjCtwy1V8m";

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
                Poll::Ready(x) => Poll::Ready(()),
            }
        }
    }

    pin_project! {
        struct TestingPollActive<St> {
            #[pin]
            end: End<St>,
            #[pin]
            other_end:End<St>,
            st_send: UnboundedSender<Box<[u8]>>
        }
    }

    impl<St: DuplexStream> TestingPollActive<St> {
        fn new(end: End<St>, other_end: End<St>, st_send: UnboundedSender<Box<[u8]>>) -> Self {
            Self {
                end,
                other_end,
                st_send,
            }
        }
    }

    impl<St: DuplexStream> Future for TestingPollActive<St> {
        type Output = ();
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let mut this = self.project();

            let mut ping_sent = false;
            loop {
                let mut active_pending = false;

                if !ping_sent {
                    ping_sent = true;
                    match Box::pin(this.end.as_mut().is_active(None)).poll_unpin(cx) {
                        Poll::Pending => {
                            active_pending = true;
                        }
                        Poll::Ready(_) => {
                            this.st_send
                                .unbounded_send(Box::new([
                                    MessagePrefix::StatusMessage as u8,
                                    StatusMessage::Pong as u8,
                                ]))
                                .unwrap();

                            this.st_send
                                .unbounded_send(Box::new([
                                    MessagePrefix::ConnectionStatus as u8,
                                    ConnectionStatusMessage::Stop as u8,
                                ]))
                                .unwrap();
                        }
                    }
                }

                match this.end.as_mut().poll(&mut this.other_end, cx) {
                    Poll::Pending => {
                        if active_pending {
                            return Poll::Pending;
                        }
                    }
                    Poll::Ready(x) => {
                        if ping_sent {
                            return Poll::Ready(());
                        }
                    }
                };
            }
        }
    }

    #[async_std::test]
    async fn wake_ping_future() {
        let now = current_timestamp();

        let mut ping_fut = PingFuture::new();

        assert!(ping_fut.started >= now);
        assert!(ping_fut.ended.is_none());
        assert!(!ping_fut.done);

        ping_fut.wake();

        let res = ping_fut.await;

        assert!(res < 10);
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

        let (st, st_send, st_receive) = TestingDuplexStream::new();
        let (st_other, st_other_send, st_other_receive) = TestingDuplexStream::new();

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

        let (st, st_send, st_receive) = TestingDuplexStream::new();
        let (st_other, st_other_send, st_other_receive) = TestingDuplexStream::new();

        let mut conn_a = End::new(st, PeerId::from_str(ALICE).unwrap().clone(), status_ba_tx, status_ab_rx);
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

        let (st_next, st_next_send, st_next_receive) = TestingDuplexStream::new();

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
        let (st_other, st_other_send, st_other_receive) = TestingDuplexStream::new();

        let conn_a = End::new(st, PeerId::from_str(ALICE).unwrap().clone(), status_ba_tx, status_ab_rx);
        let conn_b = End::new(
            st_other,
            PeerId::from_str(ALICE).unwrap().clone(),
            status_ab_tx,
            status_ba_rx,
        );

        TestingPollActive::new(conn_a, conn_b, st_send).await;

        assert_eq!(
            st_receive.take(1).collect::<Vec<Box<[u8]>>>().await,
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
