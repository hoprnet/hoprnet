use crate::{
    constants::DEFAULT_RELAYED_CONNECTION_PING_TIMEOUT,
    server::{MessagePrefix, RelayConnectionEndpoints, RelayConnectionIdentifier, Server, StatusMessage},
};
use futures::{
    future::{select, Either},
    pin_mut, ready, Future, Stream, StreamExt,
};
use pin_project_lite::pin_project;
use std::{
    cell::RefCell,
    cmp::Ordering,
    collections::HashMap,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
    time::Duration,
};
use utils_log::{error, info};
use utils_misc::traits::DuplexStream;

use libp2p::PeerId;

#[cfg(feature = "wasm")]
use wasm_bindgen_futures::spawn_local;

#[cfg(not(feature = "wasm"))]
use async_std::task::spawn_local;

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::sleep;
#[cfg(all(feature = "wasm", not(test)))]
use gloo_timers::future::sleep;

pin_project! {
    struct PollActive<St> {
        conns: Rc<RefCell<HashMap<RelayConnectionIdentifier, RelayConnectionEndpoints<St>>>>,
        source: PeerId,
        destination: PeerId,
    }
}

impl<St> Future for PollActive<St> {
    type Output = Result<Option<Box<[u8]>>, String>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        info!(
            "borrow_mut poll {}",
            RelayConnectionIdentifier::try_from((this.source.to_owned(), this.destination.to_owned()))
                .unwrap()
                .to_string()
        );
        match this.conns.borrow_mut().get_mut(
            &(this.source.to_owned(), this.destination.to_owned())
                .try_into()
                .unwrap(),
        ) {
            Some(endpoints) => match this.source.cmp(&this.destination) {
                Ordering::Equal => panic!("must not happen"),
                Ordering::Greater => Poll::Ready(Ok(ready!(Pin::new(&mut endpoints.ping_b_rx).poll_next(cx)))),
                Ordering::Less => Poll::Ready(Ok(ready!(Pin::new(&mut endpoints.ping_a_rx).poll_next(cx)))),
            },
            None => return Poll::Ready(Err("already deleted".into())),
        }
    }
}

/// Holds relayed connections, can
/// - check if a connection is active
/// - overwrite connection endpoint e.g. upon reconnects
/// - prune stale connections
/// - produce debug output to see current relay connections
struct RelayConnections<St> {
    conns: Rc<RefCell<HashMap<RelayConnectionIdentifier, RelayConnectionEndpoints<St>>>>,
}

impl<St> ToString for RelayConnections<St> {
    fn to_string(&self) -> String {
        info!("borrow to_string");
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
    /// Initiates the connection manager
    pub fn new() -> Self {
        Self {
            conns: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    /// Assembles two duplex stream to a relayed connection and stores it.
    /// Starts a background task that polls the streams.
    pub fn create_new(&mut self, id_a: PeerId, stream_a: St, id_b: PeerId, stream_b: St) -> Result<(), String> {
        let (server, endpoints) = match id_a.cmp(&id_b) {
            Ordering::Equal => panic!("must not happen"),
            Ordering::Greater => Server::new(stream_a, id_a, stream_b, id_b),
            Ordering::Less => Server::new(stream_b, id_b, stream_a, id_a),
        };

        let start_conn = {
            let conns_to_move = self.conns.clone();
            let id: RelayConnectionIdentifier = (id_a, id_b).try_into().unwrap();

            async move {
                info!("borrow_mut insert {}", id.to_string());
                conns_to_move.borrow_mut().insert(id, endpoints);

                match server.await {
                    Ok(()) => info!("Relayed connection [\"{}>\"] ended successfully", id.to_string()),
                    Err(e) => error!("Relayed connection [\"{}>\"] ended with error {}", id.to_string(), e),
                }

                info!("borrow_mut remove {}", id.to_string());
                conns_to_move.borrow_mut().remove(&id);
            }
        };

        // Start the stream but don't await its end
        spawn_local(start_conn);

        Ok(())
    }

    /// Checks if the relay connection of `source` and `destination` is active.
    /// Issues low-level ping requests on both underlying streams.
    pub async fn is_active(&self, source: PeerId, destination: PeerId, maybe_timeout: Option<u64>) -> bool {
        let id: RelayConnectionIdentifier = match (source, destination).try_into() {
            Ok(x) => x,
            Err(e) => {
                error!("{}", e);
                return false;
            }
        };

        let timeout = sleep(Duration::from_millis(
            maybe_timeout.unwrap_or(DEFAULT_RELAYED_CONNECTION_PING_TIMEOUT as u64),
        ));

        pin_mut!(timeout);

        return match source.cmp(&destination) {
            Ordering::Equal => panic!("must not happen"),
            Ordering::Greater => {
                info!("borrow send ping {}", id.to_string());
                if let Some(endpoints) = self.conns.borrow().get(&id) {
                    endpoints
                        .ping_a_tx
                        .unbounded_send(
                            Box::new([MessagePrefix::StatusMessage as u8, StatusMessage::Ping as u8]) as Box<[u8]>,
                        )
                        .unwrap();
                } else {
                    return false;
                }

                // A: A->B so we will receive from B
                match select(
                    PollActive {
                        conns: self.conns.clone(),
                        source,
                        destination,
                    },
                    timeout,
                )
                .await
                {
                    Either::Left((Ok(Some(chunk)), _)) => {
                        chunk.len() == 2
                            && chunk[0] == MessagePrefix::StatusMessage as u8
                            && chunk[1] == StatusMessage::Pong as u8
                    }
                    Either::Left((Ok(None), _)) => false,
                    Either::Left((Err(_), _)) => false,
                    Either::Right(_) => {
                        info!("low-level timeout");
                        false
                    }
                }
            }
            Ordering::Less => {
                info!("borrow send ping {}", id.to_string());
                if let Some(endpoints) = self.conns.borrow().get(&id) {
                    endpoints
                        .ping_b_tx
                        .unbounded_send(
                            Box::new([MessagePrefix::StatusMessage as u8, StatusMessage::Ping as u8]) as Box<[u8]>,
                        )
                        .unwrap();
                } else {
                    return false;
                }

                // B: B->A so we will receive from A
                match select(
                    PollActive {
                        conns: self.conns.clone(),
                        source,
                        destination,
                    },
                    timeout,
                )
                .await
                {
                    Either::Left((Ok(Some(chunk)), _)) => {
                        chunk.len() == 2
                            && chunk[0] == MessagePrefix::StatusMessage as u8
                            && chunk[1] == StatusMessage::Pong as u8
                    }
                    Either::Left((Ok(None), _)) => false,
                    Either::Left((Err(_), _)) => false,
                    Either::Right(_) => {
                        info!("low-level timeout");
                        false
                    }
                }
            }
        };
    }

    /// Overwrites the duplex stream *to* `source` by the given stream.
    /// Used to handle reconnects.
    pub fn update_existing(&self, source: PeerId, destination: PeerId, to_source: St) -> bool {
        let id: RelayConnectionIdentifier = match (source, destination).try_into() {
            Ok(x) => x,
            Err(e) => {
                error!("{}", e);
                return false;
            }
        };

        info!("borrow overwrite stream {}", id.to_string());
        if let Some(entry) = self.conns.borrow().get(&id) {
            match source.cmp(&destination) {
                Ordering::Equal => panic!("must not happen"),
                Ordering::Greater => entry.new_stream_a.unbounded_send(to_source).unwrap(),
                Ordering::Less => entry.new_stream_b.unbounded_send(to_source).unwrap(),
            }

            return true;
        }

        false
    }

    /// Checks if we have stored a relay connection between `source` and `destination`.
    /// Does not do any checks if the connection is alive.
    pub fn exists(&self, source: PeerId, destination: PeerId) -> bool {
        let id: RelayConnectionIdentifier = match (source, destination).try_into() {
            Ok(x) => x,
            Err(e) => {
                error!("{}", e);
                return false;
            }
        };

        info!("borrow exists {}", id.to_string());
        // self.conns.borrow().contains_key(&id)
        false
    }

    /// Gets the number of the currently stored connections.
    /// Can include stale connections.
    pub fn size(&self) -> usize {
        // TODO: this is weird
        info!("try_borrow length");
        0
        // match self.conns.try_borrow() {
        //     Ok(x) => x.len(),
        //     Err(_) => 0,
        // }
    }

    /// Runs through all currently stored connections, checks if they
    /// are active and drops them if the appear stale.
    pub async fn prune(&self) -> usize {
        info!("pruning connections");
        // let mut futs = FuturesUnordered::from_iter(self.conns.borrow().iter().map(|(id, conn)| PollBothActive {
        //     server: conn,
        //     id: id.clone(),
        //     maybe_timeout: None,
        // }));

        let mut pruned: usize = 0;

        // while let Some(x) = futs.next().await {
        //     if let Some(id) = x {
        //         pruned += 1;
        //         self.conns.borrow_mut().remove(&id);
        //     }
        // }

        pruned
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
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::server::{ConnectionStatusMessage, MessagePrefix, StatusMessage};

//     use super::*;
//     use futures::{
//         channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
//         future::join4,
//         stream::{FusedStream, Stream},
//         Sink,
//     };
//     use libp2p::PeerId;
//     use std::str::FromStr;

//     const ALICE: &'static str = "1Ank4EeHLAd3bwwtJma1WsXYSmiGgqmjkQoCUpevx67ix8";
//     const BOB: &'static str = "1AcPsXRKVc3U64NBb4obUUT34jSLWtvAz2JMw192L92QKW";

//     pin_project! {
//         struct TestingDuplexStream {
//             #[pin]
//             rx: UnboundedReceiver<Box<[u8]>>,
//             #[pin]
//             tx: UnboundedSender<Box<[u8]>>,
//         }
//     }

//     impl TestingDuplexStream {
//         fn new() -> (Self, UnboundedSender<Box<[u8]>>, UnboundedReceiver<Box<[u8]>>) {
//             let (send_tx, send_rx) = mpsc::unbounded::<Box<[u8]>>();
//             let (receive_tx, receive_rx) = mpsc::unbounded::<Box<[u8]>>();

//             (
//                 Self {
//                     rx: send_rx,
//                     tx: receive_tx,
//                 },
//                 send_tx,
//                 receive_rx,
//             )
//         }
//     }

//     impl FusedStream for TestingDuplexStream {
//         fn is_terminated(&self) -> bool {
//             self.rx.is_terminated()
//         }
//     }

//     impl Stream for TestingDuplexStream {
//         type Item = Result<Box<[u8]>, String>;
//         fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
//             let this = self.project();

//             match this.rx.poll_next(cx) {
//                 Poll::Pending => Poll::Pending,
//                 Poll::Ready(Some(x)) => Poll::Ready(Some(Ok(x))),
//                 Poll::Ready(None) => Poll::Pending,
//             }
//         }

//         fn size_hint(&self) -> (usize, Option<usize>) {
//             self.rx.size_hint()
//         }
//     }

//     impl Sink<Box<[u8]>> for TestingDuplexStream {
//         type Error = String;
//         fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//             let this = self.project();

//             this.tx.poll_ready(cx).map_err(|e| e.to_string())
//         }

//         fn start_send(self: Pin<&mut Self>, item: Box<[u8]>) -> Result<(), Self::Error> {
//             let this = self.project();

//             this.tx.start_send(item).map_err(|e| e.to_string())
//         }

//         fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//             let this = self.project();

//             this.tx.poll_flush(cx).map_err(|e| e.to_string())
//         }

//         fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//             let this = self.project();

//             this.tx.poll_close(cx).map_err(|e| e.to_string())
//         }
//     }

//     impl DuplexStream for TestingDuplexStream {}

//     #[async_std::test]
//     async fn check_if_both_active() {
//         let (stream_a, st_a_send, mut st_a_receive) = TestingDuplexStream::new();
//         let (stream_b, st_b_send, mut st_b_receive) = TestingDuplexStream::new();

//         let peer_a = PeerId::from_str(ALICE).unwrap();
//         let peer_b = PeerId::from_str(BOB).unwrap();

//         let server = Rc::new(RefCell::new(Server::new(stream_a, peer_a, stream_b, peer_b)));

//         // Start polling the stream in both directions
//         let poll_stream_done = PollBorrow { st: server.clone() };
//         // Issue a ping request in both directions
//         let poll_both_active = PollBothActive {
//             server,
//             id: (peer_a, peer_b).try_into().unwrap(),
//             maybe_timeout: None,
//         };

//         let ping_received_a = async {
//             assert_eq!(
//                 st_a_receive.next().await,
//                 Some(Box::new([MessagePrefix::StatusMessage as u8, StatusMessage::Ping as u8,]) as Box<[u8]>),
//                 "Must receive PING message"
//             );

//             assert!(st_a_send
//                 .unbounded_send(Box::new([
//                     MessagePrefix::StatusMessage as u8,
//                     StatusMessage::Pong as u8,
//                 ]))
//                 .is_ok(),);
//             assert!(st_a_send
//                 .unbounded_send(Box::new([
//                     MessagePrefix::ConnectionStatus as u8,
//                     ConnectionStatusMessage::Stop as u8,
//                 ]))
//                 .is_ok());

//             assert_eq!(
//                 st_a_receive.next().await,
//                 Some(Box::new([
//                     MessagePrefix::ConnectionStatus as u8,
//                     ConnectionStatusMessage::Stop as u8,
//                 ]) as Box<[u8]>),
//                 "Must receive STOP message"
//             );
//         };

//         let ping_received_b = async {
//             assert_eq!(
//                 st_b_receive.next().await,
//                 Some(Box::new([MessagePrefix::StatusMessage as u8, StatusMessage::Ping as u8,]) as Box<[u8]>),
//                 "Must receive PING message"
//             );

//             assert!(st_b_send
//                 .unbounded_send(Box::new([
//                     MessagePrefix::StatusMessage as u8,
//                     StatusMessage::Pong as u8,
//                 ]))
//                 .is_ok());
//             assert!(st_b_send
//                 .unbounded_send(Box::new([
//                     MessagePrefix::ConnectionStatus as u8,
//                     ConnectionStatusMessage::Stop as u8,
//                 ]))
//                 .is_ok());

//             assert_eq!(
//                 st_b_receive.next().await,
//                 Some(Box::new([
//                     MessagePrefix::ConnectionStatus as u8,
//                     ConnectionStatusMessage::Stop as u8,
//                 ]) as Box<[u8]>),
//                 "Must receive STOP message"
//             );
//         };

//         let (_, res, _, _) = join4(poll_stream_done, poll_both_active, ping_received_a, ping_received_b).await;

//         assert!(res.is_none());
//     }
//     #[async_std::test]
//     async fn check_if_active_timeout() {
//         let (stream_a, _, _) = TestingDuplexStream::new();
//         let (stream_b, _, _) = TestingDuplexStream::new();

//         let peer_a = PeerId::from_str(ALICE).unwrap();
//         let peer_b = PeerId::from_str(BOB).unwrap();

//         let server = Rc::new(RefCell::new(Server::new(stream_a, peer_a, stream_b, peer_b)));

//         let polled = PollBothActive {
//             server,
//             id: (peer_a, peer_b).try_into().unwrap(),
//             maybe_timeout: None,
//         }
//         .await;

//         assert!(polled.is_some(), "Passive stream should end in a timeout");
//         assert!(polled.eq(&Some((peer_a, peer_b).try_into().unwrap())))
//     }

//     #[async_std::test]
//     async fn check_if_active_timeout_immediate() {
//         let (stream_a, _, _) = TestingDuplexStream::new();
//         let (stream_b, _, _) = TestingDuplexStream::new();

//         let peer_a = PeerId::from_str(ALICE).unwrap();
//         let peer_b = PeerId::from_str(BOB).unwrap();

//         let server = Rc::new(RefCell::new(Server::new(stream_a, peer_a, stream_b, peer_b)));

//         let polled = PollBothActive {
//             server,
//             id: (peer_a, peer_b).try_into().unwrap(),
//             maybe_timeout: Some(0),
//         }
//         .await;

//         assert!(polled.is_some(), "Passive stream should end in a timeout");
//         assert!(polled.eq(&Some((peer_a, peer_b).try_into().unwrap())))
//     }

//     #[test]
//     fn empty_state_manager() {
//         let state = RelayConnections::<TestingDuplexStream>::new();

//         let a = PeerId::from_str(ALICE).unwrap();
//         let b = PeerId::from_str(BOB).unwrap();

//         assert!(state.size() == 0, "Size of empty state object must be zero");
//         assert!(
//             !state.exists(a, b),
//             "Empty state object must not contain any connection"
//         );
//     }
// }
