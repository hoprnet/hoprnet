use crate::{server::Server, traits::DuplexStream};
use core::{
    hash::{Hash, Hasher},
    pin::Pin,
    task::{Context, Poll},
};
use futures::{stream::FusedStream, Future, Sink, Stream, StreamExt};
use pin_project_lite::pin_project;
use std::{cell::RefCell, cmp::Ordering, collections::HashMap, rc::Rc};
use utils_log::error;

use libp2p::PeerId;

#[cfg(feature = "wasm")]
use wasm_bindgen_futures::spawn_local;

#[cfg(not(feature = "wasm"))]
use async_std::task::spawn_local;

pin_project! {
    struct RelayConnection<St> {
        #[pin]
        conn_a: Server<St>,
        buffered_a: Option<Box<[u8]>>,
        #[pin]
        conn_b: Server<St>,
        buffered_b: Option<Box<[u8]>>,
    }
}

impl<St> RelayConnection<St> {
    fn new(conn_a: Server<St>, conn_b: Server<St>) -> Self {
        Self {
            conn_a,
            buffered_a: None,
            conn_b,
            buffered_b: None,
        }
    }
}

pin_project! {
    struct SomeFuture<'a, St> {
        foo: & 'a mut  RelayConnection<St>,
    }
}

impl<'a, St: DuplexStream> Future for SomeFuture<'a, St> {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        let foo = &mut this.foo.conn_a;

        Pin::new(foo).poll_next(cx);
        // if !th<is.foo.conn_a.is_terminated() {
        //     match this.foo.conn_b.poll_next(cx) {
        //         Poll::Ready(Some(item)) => match this.conn_a.as_mut().poll_ready(cx) {
        //             Poll::Ready(Ok(())) => match this.conn_a.as_mut().start_send(item.unwrap()) {
        //                 Ok(()) => (),
        //                 Err(e) => return Poll::Ready(Err(e)),
        //             },
        //             Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
        //             Poll::Pending => {
        //                 *this.buffered_a = Some(item.unwrap());
        //                 b_next_pending = true;
        //             }
        //         },
        //         Poll::Ready(None) => match this.conn_a.as_mut().poll_flush(cx) {
        //             Poll::Ready(Ok(())) => (),
        //             Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
        //             Poll::Pending => {
        //                 b_next_pending = true;
        //             }
        //         },
        //         Poll::Pending => b_next_pending = true,
        //     };
        // }>

        Poll::Ready(())
    }
}

impl<St: DuplexStream> Future for RelayConnection<St> {
    type Output = Result<(), String>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        let mut a_send_pending = false;

        if let Some(item) = this.buffered_a.take() {
            match this.conn_a.as_mut().poll_ready(cx) {
                Poll::Ready(Ok(())) => match this.conn_a.as_mut().start_send(item) {
                    Ok(()) => (),
                    Err(e) => return Poll::Ready(Err(e)),
                },
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => {
                    *this.buffered_a = Some(item);
                    a_send_pending = true;
                }
            };
        }

        if let Some(item) = this.buffered_b.take() {
            match this.conn_b.as_mut().poll_ready(cx) {
                Poll::Ready(Ok(())) => match this.conn_b.as_mut().start_send(item) {
                    Ok(()) => (),
                    Err(e) => return Poll::Ready(Err(e)),
                },
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => {
                    *this.buffered_b = Some(item);
                    if a_send_pending {
                        // FIXME: This might produce a dead-lock
                        return Poll::Pending;
                    }
                }
            }
        }

        let mut b_next_pending: bool;

        loop {
            b_next_pending = false;

            if this.conn_a.is_terminated() && this.conn_b.is_terminated() {
                break Poll::Ready(Ok(()));
            }

            if !this.conn_a.is_terminated() {
                match this.conn_b.as_mut().poll_next(cx) {
                    Poll::Ready(Some(item)) => match this.conn_a.as_mut().poll_ready(cx) {
                        Poll::Ready(Ok(())) => match this.conn_a.as_mut().start_send(item.unwrap()) {
                            Ok(()) => (),
                            Err(e) => return Poll::Ready(Err(e)),
                        },
                        Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                        Poll::Pending => {
                            *this.buffered_a = Some(item.unwrap());
                            b_next_pending = true;
                        }
                    },
                    Poll::Ready(None) => match this.conn_a.as_mut().poll_flush(cx) {
                        Poll::Ready(Ok(())) => (),
                        Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                        Poll::Pending => {
                            b_next_pending = true;
                        }
                    },
                    Poll::Pending => b_next_pending = true,
                };
            }

            if !this.conn_b.is_terminated() {
                match this.conn_a.as_mut().poll_next(cx) {
                    Poll::Ready(Some(item)) => match this.conn_b.as_mut().poll_ready(cx) {
                        Poll::Ready(Ok(())) => match this.conn_b.as_mut().start_send(item.unwrap()) {
                            Ok(()) => (),
                            Err(e) => return Poll::Ready(Err(e)),
                        },
                        Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                        Poll::Pending => {
                            *this.buffered_b = Some(item.unwrap());
                            if b_next_pending {
                                // FIXME: This might produce a dead-lock
                                return Poll::Pending;
                            }
                        }
                    },
                    Poll::Ready(None) => match this.conn_b.as_mut().poll_flush(cx) {
                        Poll::Ready(Ok(())) => (),
                        Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                        Poll::Pending => {
                            if b_next_pending {
                                return Poll::Pending;
                            }
                        }
                    },
                    Poll::Pending => {
                        if b_next_pending {
                            return Poll::Pending;
                        }
                    }
                }
            }
        }
    }
}

#[derive(Copy, Clone, Eq)]
struct RelayConnectionIdentifier {
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

fn get_key_and_value<St>(
    id_a: PeerId,
    conn_a: Server<St>,
    id_b: PeerId,
    conn_b: Server<St>,
) -> Result<(RelayConnectionIdentifier, RelayConnection<St>), String> {
    match id_a.cmp(&id_b) {
        Ordering::Equal => Err("Keys must not be equal".into()),
        Ordering::Greater => Ok((
            RelayConnectionIdentifier { id_a, id_b },
            RelayConnection::new(conn_a, conn_b),
        )),
        Ordering::Less => Ok((
            RelayConnectionIdentifier { id_a: id_b, id_b: id_a },
            RelayConnection::new(conn_b, conn_a),
        )),
    }
}

struct RelayConnections<St> {
    conns: Rc<RefCell<HashMap<RelayConnectionIdentifier, Option<RelayConnection<St>>>>>,
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

        format!("{}{}", prefix, items)
    }
}

impl<St: DuplexStream> RelayConnections<St> {
    pub fn new() -> Self {
        Self {
            conns: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn create_new(&'static mut self, id_a: PeerId, stream_a: St, id_b: PeerId, stream_b: St) -> Result<(), String> {
        let id1: RelayConnectionIdentifier = (id_a, id_b).try_into().unwrap();

        {
            let conns = &self.conns;

            let conn_a = Server::new(
                stream_a,
                Box::new(move || {
                    conns.borrow_mut().remove(&id1);
                }),
                Box::new(move || {
                    conns.borrow_mut().remove(&id1);
                }),
            );

            let conn_b = Server::new(
                stream_b,
                Box::new(move || {
                    conns.borrow_mut().remove(&id1);
                }),
                Box::new(move || {
                    conns.borrow_mut().remove(&id1);
                }),
            );

            let (id, conn) = get_key_and_value(id_a, conn_a, id_b, conn_b)?;

            // let ptr = SomeFuture { foo: &mut conn };
            // spawn_local(ptr);
            let mut foo = None;
            let bar = foo.insert(conn);

            conns.borrow_mut().insert(id, foo);

            // spawn_local(async move {

            //     conns.borrow().get(&id).
            //     let foo = bar;
            //     // match bar.await {
            //     //     Ok(()) => (),
            //     //     Err(e) => {
            //     //         // conns.borrow_mut().remove(&id);
            //     //         error!("{}", e)
            //     //     }
            //     // }
            // });

            // conns.borrow_mut().insert(id, conn);

            // spawn_local(async move {
            //     match conn.await {
            //         Ok(()) => (),
            //         Err(e) => {
            //             // conns.borrow_mut().remove(&id);
            //             error!("{}", e)
            //         }
            //     }
            // });
        }

        let conns = &self.conns;
        let id = id1.clone();

        // spawn_local(async move {
        //     match conns.borrow_mut().get_mut(&id).as_mut().unwrap().await {
        //         Ok(()) => (),
        //         Err(e) => {
        //             conns.borrow_mut().remove(&id);
        //             error!("{}", e)
        //         }
        //     }
        // });

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

        // FIXME: no need for a mutable borrow
        let mut conns = self.conns.borrow_mut();

        let relay_conn = match conns.get_mut(&id) {
            Some(x) => x,
            None => {
                error!("Connection does not exist");
                return false;
            }
        };

        match source.cmp(&destination) {
            Ordering::Equal => panic!("must not happen"),
            Ordering::Greater => {
                return match relay_conn.as_mut().unwrap().conn_b.ping(maybe_timeout).await {
                    Ok(_) => true,
                    Err(e) => {
                        error!("{}", e);
                        false
                    }
                }
            }
            Ordering::Less => {
                return match relay_conn.as_mut().unwrap().conn_a.ping(maybe_timeout).await {
                    Ok(_) => true,
                    Err(e) => {
                        error!("{}", e);
                        false
                    }
                }
            }
        }
    }

    pub fn update_existing(&mut self, source: PeerId, destination: PeerId, to_source: St) -> bool {
        let id: RelayConnectionIdentifier = match (source, destination).try_into() {
            Ok(x) => x,
            Err(e) => {
                error!("{}", e);
                return false;
            }
        };

        let mut conns = self.conns.borrow_mut();

        let relay_conn = match conns.get_mut(&id) {
            Some(x) => x,
            None => {
                error!("Connection does not exist");
                return false;
            }
        };

        match source.cmp(&destination) {
            Ordering::Equal => panic!("must not happen"),
            Ordering::Greater => {
                return match relay_conn.as_mut().unwrap().conn_b.update(to_source) {
                    Ok(_) => true,
                    Err(e) => {
                        error!("{}", e);
                        false
                    }
                }
            }
            Ordering::Less => {
                return match relay_conn.as_mut().unwrap().conn_a.update(to_source) {
                    Ok(_) => true,
                    Err(e) => {
                        error!("{}", e);
                        false
                    }
                }
            }
        }
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
