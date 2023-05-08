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
use utils_log::error;
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
        let prefix: String = "RelayConnections:\n".into();

        let items = self.conns.borrow().keys().map(|x| x.to_string()).collect::<Vec<_>>();

        format!("{} {}", prefix, items.join("\n  "))
    }
}

impl<'a, St: DuplexStream + 'static> RelayConnections<St> {
    pub fn new() -> Self {
        Self {
            conns: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn create_new(&mut self, id_a: PeerId, stream_a: St, id_b: PeerId, stream_b: St) -> Result<(), String> {
        let id1: RelayConnectionIdentifier = (id_a, id_b).try_into().unwrap();

        let server = Rc::new(RefCell::new(Server::new(stream_a, id_a, stream_b, id_b)));

        let id1_to_move = id1.clone();
        self.conns.borrow_mut().insert(id1, server.clone());

        let conn_to_move = self.conns.clone();
        let polling = PollBorrow { st: server };
        spawn_local(async move {
            polling.await;
            conn_to_move.borrow_mut().remove(&id1_to_move);
        });

        Ok(())
    }

    pub async fn is_active(&mut self, source: PeerId, destination: PeerId, maybe_timeout: Option<u64>) -> bool {
        let id: RelayConnectionIdentifier = match (source, destination).try_into() {
            Ok(x) => x,
            Err(e) => {
                error!("{}", e);
                return false;
            }
        };

        if let Some(entry) = self.conns.borrow_mut().get_mut(&id) {
            return entry.borrow_mut().is_active(source, maybe_timeout).await;
        }

        false
    }

    pub fn update_existing(&mut self, source: PeerId, destination: PeerId, to_source: St) -> bool {
        let id: RelayConnectionIdentifier = match (source, destination).try_into() {
            Ok(x) => x,
            Err(e) => {
                error!("{}", e);
                return false;
            }
        };

        if let Some(entry) = self.conns.borrow_mut().get_mut(&id) {
            entry.borrow_mut().update(source, to_source).unwrap();
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
            &mut self,
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
            &mut self,
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
