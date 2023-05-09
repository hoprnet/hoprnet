use crate::server_new::{PingFuture, PollBorrow as PollBorrowActive, RelayConnectionIdentifier, Server};
use async_std::stream::StreamExt;
use futures::{
    future::{Future, Join},
    stream::FuturesUnordered,
};
use pin_project_lite::pin_project;
use std::{
    cell::RefCell,
    collections::HashMap,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};
use utils_log::{error, info};
use utils_misc::traits::DuplexStream;

use libp2p::PeerId;

#[cfg(feature = "wasm")]
use wasm_bindgen_futures::spawn_local;

#[cfg(not(feature = "wasm"))]
use async_std::task::spawn_local;

pin_project! {
    struct PollBorrow<St> {
        st: Rc<RefCell<Server<St>>>,
    }
}

impl<St: DuplexStream> Future for PollBorrow<St> {
    type Output = Result<(), String>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        Pin::new(&mut *this.st.borrow_mut()).poll(cx)
    }
}

pin_project! {
    struct PollActive<St> {
        server: Rc<RefCell<Server<St>>>,
        #[pin]
        fut: Option<Join<PollBorrowActive<PingFuture>, PollBorrowActive<PingFuture>>>,
    }
}

impl<St: DuplexStream> PollActive<St> {
    pub fn new(server: Rc<RefCell<Server<St>>>) -> Self {
        Self { server, fut: None }
    }
}

impl<St: DuplexStream> Future for PollActive<St> {
    type Output = Option<RelayConnectionIdentifier>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        if this.fut.is_none() {
            *this.fut = Some(this.server.borrow().both_active(None));
        }

        match this.fut.as_mut().as_pin_mut().unwrap().poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready((a, b)) => Poll::Ready(if a.is_ok() && b.is_ok() {
                // No need to prune anything if connection is alive
                None
            } else {
                Some(this.server.borrow().get_id())
            }),
        }
    }
}

struct RelayConnections<St> {
    conns: Rc<RefCell<HashMap<RelayConnectionIdentifier, Rc<RefCell<Server<St>>>>>>,
}

impl<St> ToString for RelayConnections<St> {
    fn to_string(&self) -> String {
        let items = self.conns.borrow().keys().map(|x| x.to_string()).collect::<Vec<_>>();

        let prefix: String = "RelayConnections:\n".into();

        format!(
            "{} {}",
            prefix,
            if items.len() == 0 {
                "  No relayed connections".into()
            } else {
                items.join("\n  ")
            }
        )
    }
}

impl<'a, St: DuplexStream + 'static> RelayConnections<St> {
    pub fn new() -> Self {
        Self {
            conns: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn create_new(&mut self, id_a: PeerId, stream_a: St, id_b: PeerId, stream_b: St) -> Result<(), String> {
        let id: RelayConnectionIdentifier = (id_a, id_b).try_into().unwrap();

        let server = Rc::new(RefCell::new(Server::new(stream_a, id_a, stream_b, id_b)));

        {
            self.conns.borrow_mut().insert(id, server.clone());
        }

        let id_to_move = id.clone();
        let conns_to_move = self.conns.clone();
        let polling = PollBorrow { st: server };

        // Start the stream but don't await its end
        spawn_local(async move {
            match polling.await {
                Ok(()) => info!(
                    "Relayed connection [\"{}>\"] ended successfully",
                    id_to_move.to_string()
                ),
                Err(e) => error!(
                    "Relayed connection [\"{}>\"] ended with error {}",
                    id_to_move.to_string(),
                    e
                ),
            }
            conns_to_move.borrow_mut().remove(&id_to_move);
        });

        Ok(())
    }

    pub async fn is_active(&self, source: PeerId, destination: PeerId, maybe_timeout: Option<u64>) -> bool {
        let id: RelayConnectionIdentifier = match (source, destination).try_into() {
            Ok(x) => x,
            Err(e) => {
                error!("{}", e);
                return false;
            }
        };

        if let Some(entry) = self.conns.borrow().get(&id) {
            return entry.borrow().is_active(source, maybe_timeout).await;
        }

        false
    }

    pub fn update_existing(&self, source: PeerId, destination: PeerId, to_source: St) -> bool {
        let id: RelayConnectionIdentifier = match (source, destination).try_into() {
            Ok(x) => x,
            Err(e) => {
                error!("{}", e);
                return false;
            }
        };

        if let Some(entry) = self.conns.borrow().get(&id) {
            match entry.borrow().update(source, to_source) {
                Ok(()) => info!(
                    "Successfully replaced old incoming connection for [\"{}>\"]",
                    id.to_string()
                ),
                Err(e) => error!(
                    "Error while replacing incoming connection for [\"{}>\"] {}",
                    id.to_string(),
                    e
                ),
            }
            return true;
        }

        false
    }

    pub fn exists(&self, source: PeerId, destination: PeerId) -> bool {
        let id: RelayConnectionIdentifier = match (source, destination).try_into() {
            Ok(x) => x,
            Err(e) => {
                error!("{}", e);
                return false;
            }
        };

        self.conns.borrow().contains_key(&id)
    }

    pub fn size(&self) -> usize {
        self.conns.borrow().len()
    }

    pub async fn prune(&self) -> usize {
        let conns = self.conns.borrow();

        let mut futs = FuturesUnordered::from_iter(conns.iter().map(|(_, conn)| PollActive::new(conn.clone())));

        let mut pruned: usize = 0;

        while let Some(x) = futs.next().await {
            if let Some(id) = x {
                pruned += 1;
                self.conns.borrow_mut().remove(&id);
            }
        }

        pruned
    }

    pub fn remove(&self, id: &RelayConnectionIdentifier) {
        let mut conns = self.conns.borrow_mut();

        conns.remove(&id);
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use js_sys::Number;
    use libp2p::PeerId;
    use std::str::FromStr;
    use utils_misc::ok_or_jserr;
    use utils_misc::streaming_iterable::{JsStreamingIterable, StreamingIterable};
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct RelayState {
        w: super::RelayConnections<StreamingIterable>,
    }

    #[wasm_bindgen]
    extern "C" {
        pub type JsPeerId;

        #[wasm_bindgen(structural, method, js_name = "toString")]
        pub fn to_string(this: &JsPeerId) -> String;
    }

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen]
        pub type RelayStateOps;

        #[wasm_bindgen(getter, js_name = "relayFreeTimeout")]
        pub fn relay_free_timeout() -> u32;
    }

    #[wasm_bindgen(js_name = "getId")]
    pub fn get_id(source: JsPeerId, destination: JsPeerId) -> Result<JsValue, JsValue> {
        let source = ok_or_jserr!(PeerId::from_str(source.to_string().as_str()))?;
        let destination = ok_or_jserr!(PeerId::from_str(destination.to_string().as_str()))?;

        let id = ok_or_jserr!(super::RelayConnectionIdentifier::try_from((source, destination)))?;

        Ok(JsValue::from(id.to_string()))
    }

    #[wasm_bindgen]
    impl RelayState {
        #[wasm_bindgen(constructor)]
        pub fn new(_ops: RelayStateOps) -> RelayState {
            RelayState {
                w: super::RelayConnections::new(),
            }
        }

        #[wasm_bindgen(js_name = "createNew")]
        pub fn create_new(
            &mut self,
            source: JsPeerId,
            destination: JsPeerId,
            to_source: JsStreamingIterable,
            to_destination: JsStreamingIterable,
        ) -> Result<JsValue, JsValue> {
            let source = ok_or_jserr!(PeerId::from_str(source.to_string().as_str()))?;
            let destination = ok_or_jserr!(PeerId::from_str(destination.to_string().as_str()))?;

            match self
                .w
                .create_new(source, to_source.into(), destination, to_destination.into())
            {
                Ok(()) => Ok(JsValue::undefined()),
                Err(e) => Err(e.into()),
            }
        }

        #[wasm_bindgen(js_name = "isActive")]
        pub async fn is_active(
            &self,
            source: JsPeerId,
            destination: JsPeerId,
            timeout: Option<Number>,
        ) -> Result<JsValue, JsValue> {
            let source = ok_or_jserr!(PeerId::from_str(source.to_string().as_str()))?;
            let destination = ok_or_jserr!(PeerId::from_str(destination.to_string().as_str()))?;

            let res = self
                .w
                .is_active(source, destination, timeout.map(|n| n.value_of() as u64))
                .await;

            Ok(res.into())
        }

        #[wasm_bindgen(js_name = "updateExisting")]
        pub fn update_existing(
            &self,
            source: JsPeerId,
            destination: JsPeerId,
            to_source: JsStreamingIterable,
        ) -> Result<JsValue, JsValue> {
            let source = ok_or_jserr!(PeerId::from_str(source.to_string().as_str()))?;
            let destination = ok_or_jserr!(PeerId::from_str(destination.to_string().as_str()))?;

            self.w.update_existing(source, destination, to_source.into());

            Ok(JsValue::undefined())
        }

        #[wasm_bindgen]
        pub fn exists(&self, source: JsPeerId, destination: JsPeerId) -> Result<JsValue, JsValue> {
            let source = ok_or_jserr!(PeerId::from_str(source.to_string().as_str()))?;
            let destination = ok_or_jserr!(PeerId::from_str(destination.to_string().as_str()))?;

            Ok(self.w.exists(source, destination).into())
        }

        #[wasm_bindgen(js_name = "relayedConnectionCount")]
        pub fn relayed_connection_count(&self) -> Number {
            // SAFETY: There won't be more than 2**32 - 1 relayed connections
            Number::from(self.w.size() as u32)
        }

        #[wasm_bindgen]
        pub async fn prune(&self) -> Number {
            // SAFETY: There won't be more than 2**32 - 1 relayed connections
            Number::from(self.w.prune().await as u32)
        }

        #[wasm_bindgen(js_name = "toString")]
        pub fn to_string(&self) -> String {
            self.w.to_string()
        }

        #[wasm_bindgen]
        pub fn remove(&self, source: JsPeerId, destination: JsPeerId) -> Result<JsValue, JsValue> {
            let source = ok_or_jserr!(PeerId::from_str(source.to_string().as_str()))?;
            let destination = ok_or_jserr!(PeerId::from_str(destination.to_string().as_str()))?;

            self.w.remove(&(source, destination).try_into()?);

            Ok(JsValue::undefined())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::server_new::{ConnectionStatusMessage, MessagePrefix, StatusMessage};

    use super::*;
    use futures::{
        channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
        future::join,
        stream::{FusedStream, Stream},
        Sink,
    };
    use libp2p::PeerId;
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
                Poll::Ready(None) => Poll::Pending,
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

    #[async_std::test]
    async fn check_if_active() {
        let (stream_a, st_a_send, st_a_receive) = TestingDuplexStream::new();
        let (stream_b, st_b_send, st_b_receive) = TestingDuplexStream::new();

        let peer_a = PeerId::from_str(ALICE).unwrap();
        let peer_b = PeerId::from_str(BOB).unwrap();

        let server = Rc::new(RefCell::new(Server::new(stream_a, peer_a, stream_b, peer_b)));

        // Start polling the stream in both directions
        let polled_borrow = PollBorrow { st: server.clone() };
        // Issue a ping request in both directions
        let polled = PollActive::new(server);

        let ((_, res), _) = join(join(polled_borrow, polled), async move {
            assert_eq!(
                join(st_a_receive.take(1).next(), st_b_receive.take(1).next()).await,
                (
                    Some(Box::new([MessagePrefix::StatusMessage as u8, StatusMessage::Ping as u8,]) as Box<[u8]>),
                    Some(Box::new([MessagePrefix::StatusMessage as u8, StatusMessage::Ping as u8,]) as Box<[u8]>)
                )
            );

            st_a_send
                .unbounded_send(Box::new([
                    MessagePrefix::StatusMessage as u8,
                    StatusMessage::Pong as u8,
                ]))
                .unwrap();

            st_b_send
                .unbounded_send(Box::new([
                    MessagePrefix::StatusMessage as u8,
                    StatusMessage::Pong as u8,
                ]))
                .unwrap();

            st_a_send
                .unbounded_send(Box::new([
                    MessagePrefix::ConnectionStatus as u8,
                    ConnectionStatusMessage::Stop as u8,
                ]))
                .unwrap();

            st_b_send
                .unbounded_send(Box::new([
                    MessagePrefix::ConnectionStatus as u8,
                    ConnectionStatusMessage::Stop as u8,
                ]))
                .unwrap();
        })
        .await;

        assert!(res.is_none());
    }
    #[async_std::test]
    async fn check_if_active_timeout() {
        let (stream_a, _, _) = TestingDuplexStream::new();
        let (stream_b, _, _) = TestingDuplexStream::new();

        let peer_a = PeerId::from_str(ALICE).unwrap();
        let peer_b = PeerId::from_str(BOB).unwrap();

        let server = Rc::new(RefCell::new(Server::new(stream_a, peer_a, stream_b, peer_b)));

        let polled = PollActive::new(server).await;

        assert!(polled.is_some(), "Passive stream should end in a timeout");
        assert!(polled.eq(&Some((peer_a, peer_b).try_into().unwrap())))
    }

    #[test]
    fn empty_state_manager() {
        let state = RelayConnections::<TestingDuplexStream>::new();

        let a = PeerId::from_str(ALICE).unwrap();
        let b = PeerId::from_str(BOB).unwrap();

        assert!(state.size() == 0, "Size of empty state object must be zero");
        assert!(
            !state.exists(a, b),
            "Empty state object must not contain any connection"
        );
    }
}
