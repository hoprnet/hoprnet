use crate::{
    server_new::{RelayConnectionIdentifier, Server},
    traits::DuplexStream,
};
use futures::Future;
use pin_project_lite::pin_project;
use std::{
    cell::RefCell,
    collections::HashMap,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};
use utils_log::error;

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

struct RelayConnections<St> {
    conns: Rc<RefCell<HashMap<RelayConnectionIdentifier, Rc<RefCell<Server<St>>>>>>,
}

impl<St> ToString for RelayConnections<St> {
    fn to_string(&self) -> String {
        let prefix: String = "RelayConnections:\n".into();

        let items: String = self
            .conns
            .borrow()
            .keys()
            .map(|x| format!("  {}\n", x.to_string()))
            .collect();

        format!("{} {}", prefix, items)
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
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use std::str::FromStr;

    use crate::streaming_iterable::{JsStreamingIterable, StreamingIterable};
    use js_sys::Number;
    use libp2p::PeerId;
    use utils_misc::ok_or_jserr;
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

            let this = unsafe {
                std::mem::transmute::<
                    &mut super::RelayConnections<StreamingIterable>,
                    &'static mut super::RelayConnections<StreamingIterable>,
                >(&mut self.w)
            };

            match this.create_new(source, to_source.into(), destination, to_destination.into()) {
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
    }
}
