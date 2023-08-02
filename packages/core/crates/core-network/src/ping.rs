use std::{time::Duration, pin::Pin, result};

use async_trait::async_trait;
use futures::{
    channel::mpsc,
    future::{
        poll_fn,
        select,
        Either,
        FutureExt, // .fuse()
    },
    pin_mut,
    stream::{FuturesUnordered, StreamExt},
};
use libp2p_identity::PeerId;

use utils_log::{debug, error, info};
use utils_metrics::histogram_start_measure;
use utils_metrics::metrics::SimpleCounter;
use utils_metrics::metrics::SimpleHistogram;
use utils_metrics::metrics::SimpleTimer;
use utils_types::traits::BinarySerializable;

use crate::errors::NetworkingError;
use crate::errors::NetworkingError::{DecodingError, Other, Timeout};
use crate::messaging::{ControlMessage, PingMessage};

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::sleep;
#[cfg(any(not(feature = "wasm"), test))]
use utils_misc::time::native::current_timestamp;

use crate::messaging::ControlMessage::Pong;
#[cfg(all(feature = "wasm", not(test)))]
use gloo_timers::future::sleep;
#[cfg(all(feature = "wasm", not(test)))]
use utils_misc::time::wasm::current_timestamp;

const PINGS_MAX_PARALLEL: usize = 14;

// TODO: NOTE: UnboundedSender and UnboundedReceiver are bound only by available memory
// in case of faster input than output the memory might run out
pub type HeartbeatSendPingTx = futures::channel::mpsc::UnboundedSender<(PeerId, ControlMessage)>;
pub type HeartbeatGetPongRx = futures::channel::mpsc::UnboundedReceiver<(PeerId, std::result::Result<ControlMessage, ()>)>;


#[cfg_attr(test, mockall::automock)]
pub trait PingExternalAPI {
    fn on_finished_ping(&self, peer: &PeerId, result: crate::types::Result);
}

/// Basic type used for internally aggregating ping results to be further processed in the
/// PingExternalAPI callbacks.
type PingMeasurement = (PeerId, crate::types::Result);


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PingConfig {
    pub max_parallel_pings: usize,
    pub network: String,
    pub normalized_version: String,
    pub timeout: Duration,
}

#[async_trait(? Send)] // not placing the `Send` trait limitations on the trait
pub trait Pinging {
    async fn ping(&mut self, peers: Vec<PeerId>);
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Ping {
    config: PingConfig,
    active_pings: std::collections::HashMap<PeerId, (u64, ControlMessage, Option<SimpleTimer>)>,
    send_ping: HeartbeatSendPingTx,
    receive_pong: HeartbeatGetPongRx,
    external_api: Box<dyn PingExternalAPI>,
    metric_time_to_ping: Option<SimpleHistogram>,
    metric_successful_ping_count: Option<SimpleCounter>,
    metric_failed_ping_count: Option<SimpleCounter>,
}

impl Ping {
    pub fn new(config: PingConfig, send_ping: HeartbeatSendPingTx, receive_pong: HeartbeatGetPongRx, external_api: Box<dyn PingExternalAPI>) -> Ping {
        let config = PingConfig {
            max_parallel_pings: config.max_parallel_pings.min(PINGS_MAX_PARALLEL),
            ..config
        };

        Ping {
            config,
            active_pings: std::collections::HashMap::new(),
            send_ping,
            receive_pong,
            external_api,
            metric_time_to_ping: if cfg!(test) {None} else {SimpleHistogram::new(
                "core_histogram_ping_time_seconds",
                "Measures total time it takes to ping a single node (seconds)",
                vec![0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0, 60.0, 90.0, 120.0, 300.0],
            )
            .ok()},
            metric_successful_ping_count: SimpleCounter::new(
                "core_counter_heartbeat_successful_pings",
                "Total number of successful pings",
            )
            .ok(),
            metric_failed_ping_count: SimpleCounter::new(
                "core_counter_heartbeat_failed_pings",
                "Total number of failed pings",
            )
            .ok(),
        }
    }

    fn initiate_peer_ping(&mut self, peer: &PeerId) -> bool {
        if ! self.active_pings.contains_key(&peer) {
            info!("Pinging peer '{}'", peer);

            let ping_challenge: ControlMessage = ControlMessage::generate_ping_request();

            let ping_peer_timer = if let Some(metric_time_to_ping) = &self.metric_time_to_ping {
                Some(histogram_start_measure!(metric_time_to_ping))
            } else {
                None
            };
            let _ = self.active_pings.insert(peer.clone(), (current_timestamp(), ping_challenge.clone(), ping_peer_timer));
            return self.send_ping.start_send((peer.clone(), ping_challenge)).is_ok()
        }

        false
    }
}

#[async_trait(? Send)] 
impl Pinging for Ping {
    /// Performs multiple concurrent async pings to the specified peers.
    ///
    /// A sliding window mechanism is used to select at most a fixed number of concurrently processed
    /// peers in order to stabilize the pinging mechanism. Pings that do not fit into that window must
    /// wait until they can be further processed.
    ///
    /// # Arguments
    ///
    /// * `peers` - A vector of PeerId objects referencing the peers to be pinged
    /// * `send_msg` - The send function producing a Future with the reply of the pinged peer
    async fn ping(&mut self, peers: Vec<PeerId>)
    {
        let start_all_peers = current_timestamp();
        let mut peers = peers;

        if peers.is_empty() {
            debug!("Received an empty peer list, not pinging any peers");
            return ()
        }

        if let Err(e) = poll_fn(|cx| Pin::new(&mut self.send_ping).poll_ready(cx)).await {
            error!("The ping receiver is not listening: {}", e);
            return ()
        }

        let remnants = self.active_pings.len();
        if remnants > 0 {
            debug!("{} remnants from previous timeout aborted session are present", remnants)
        }

        let remainder = peers.split_off(self.config.max_parallel_pings.min(peers.len()));
        for peer in peers.into_iter() {
            self.initiate_peer_ping(&peer);
        }

        let mut waiting = std::collections::VecDeque::from(remainder);
        while let Some((peer, response)) = self.receive_pong.next().await {
            let (peer, result) = match response {
                Ok(pong) => {
                    let record = self.active_pings.remove(&peer);
                    
                    if record.is_none() {
                        error!("Received a pong for an unregistered ping, likely an aborted run");
                        continue;
                    }

                    let (start, challenge, timer) = record.expect("Should hold a value at this point");
                    let duration: std::result::Result<std::time::Duration, ()> = {
                        if ControlMessage::validate_pong_response(&challenge, &pong).is_ok() {
                            info!("Successfully pinged peer {}", peer);
                            Ok(std::time::Duration::from_millis(current_timestamp() - start))
                        } else {
                            error!("Failed to verify the challenge for ping to peer: {}", peer.to_string());
                            Err(())
                        }
                    };

                    if let Some(metric_time_to_ping) = &self.metric_time_to_ping {
                        metric_time_to_ping.record_measure(timer.unwrap());
                    }

                    (peer, duration)
                },
                Err(_) => {
                    error!("Ping to peer {} timed out", peer);
                    (peer, Err(()))
                }
            };

            match result {
                Ok(_) => {
                    if let Some(metric_successful_ping_count) = &self.metric_successful_ping_count {
                        metric_successful_ping_count.increment();
                    };
                },
                Err(_) => {
                    if let Some(metric_failed_ping_count) = &self.metric_failed_ping_count {
                        metric_failed_ping_count.increment();
                    };
                }
            }

            self.external_api.on_finished_ping(&peer, result.map(|v| v.as_millis() as u64));

            let remaining_time = current_timestamp() - start_all_peers;
            if (remaining_time as u128) < self.config.timeout.as_millis() {
                while let Some(peer) = waiting.pop_front() {
                    if self.initiate_peer_ping(&peer) {
                        break
                    }
                }
            }
        }
    }
}

// #[cfg(feature = "wasm")]
// pub mod wasm {
//     use super::*;
//     use js_sys::JsString;
//     use std::str::FromStr;
//     use wasm_bindgen::prelude::*;

//     #[wasm_bindgen]
//     struct WasmPingApi {
//         _network: String,
//         _version: String,
//         on_finished_ping_cb: js_sys::Function,
//     }

//     impl PingExternalAPI for WasmPingApi {
//         fn on_finished_ping(&self, peer: &PeerId, result: crate::types::Result) {
//             let this = JsValue::null();
//             let peer = JsValue::from(peer.to_base58());
//             let res = {
//                 if let Ok(v) = result {
//                     JsValue::from(v as f64)
//                 } else {
//                     JsValue::undefined()
//                 }
//             };

//             if let Err(err) = self.on_finished_ping_cb.call2(&this, &peer, &res) {
//                 error!(
//                     "Failed to perform on peer offline operation with: {}",
//                     err.as_string()
//                         .unwrap_or_else(|| { "Unspecified error occurred on registering the ping result".to_owned() })
//                         .as_str()
//                 )
//             };
//         }
//     }

//     /// WASM wrapper for the Ping that accepts a JS function for sending a ping message and returing
//     /// a Future for the peer reply.
//     #[wasm_bindgen]
//     pub struct Pinger {
//         pinger: Ping,
//     }

//     #[wasm_bindgen]
//     impl Pinger {
//         #[wasm_bindgen]
//         pub fn build(
//             network: String,
//             version: String,
//             on_finished_ping_cb: js_sys::Function,
//             send_msg_cb: js_sys::Function,
//         ) -> Self {
//             let api = Box::new(WasmPingApi {
//                 _network: network.clone(),
//                 _version: version.clone(),
//                 on_finished_ping_cb,
//             });

//             let config = PingConfig {
//                 network,
//                 normalized_version: version,
//                 max_parallel_pings: PINGS_MAX_PARALLEL,
//                 timeout: Duration::from_secs(30),
//             };

//             Self {
//                 pinger: Ping::new(config, api),
//                 send_msg_cb,
//             }
//         }

//         /// Ping the peers represented as a Vec<JsString> values that are converted into usable
//         /// PeerIds.
//         ///
//         /// # Arguments
//         /// * `peers` - Vector of String representations of the PeerIds to be pinged.
//         #[wasm_bindgen]
//         pub async fn ping(&self, mut peers: Vec<JsString>) {
//             let converted = peers
//                 .drain(..)
//                 .filter_map(|x| {
//                     let x: String = x.into();
//                     PeerId::from_str(&x).ok()
//                 })
//                 .collect::<Vec<_>>();

//             let message_transport =
//                 |msg: Box<[u8]>,
//                  peer: String|
//                  -> std::pin::Pin<Box<dyn futures::Future<Output = Result<Box<[u8]>, String>>>> {
//                     Box::pin(async move {
//                         let this = JsValue::null();
//                         let data: JsValue = js_sys::Uint8Array::from(msg.as_ref()).into();
//                         let peer: JsValue = <JsValue as From<String>>::from(peer);

//                         // call a send_msg_cb producing a JS promise that is further converted to a Future
//                         // holding the reply of the pinged peer for the ping message.
//                         match self.send_msg_cb.call2(&this, &data, &peer) {
//                             Ok(r) => {
//                                 let promise = js_sys::Promise::from(r);
//                                 match wasm_bindgen_futures::JsFuture::from(promise).await {
//                                     Ok(x) => {
//                                         if x.is_undefined() {
//                                             Err("Failed to send ping message".into())
//                                         } else {
//                                             debug!("transport returned {:?}", x);
//                                             let arr = js_sys::Array::from(x.as_ref());
//                                             if arr.length() > 0 {
//                                                 Ok(js_sys::Uint8Array::from(arr.get(0)).to_vec().into_boxed_slice())
//                                             } else {
//                                                 error!("transport has returned an empty response");
//                                                 Err("Empty response returned from ping transport".into())
//                                             }
//                                         }
//                                     }
//                                     Err(e) => {
//                                         error!("Failed to send ping message");
//                                         error!("{:?}", e);
//                                         Err("Failed to send ping message".into())
//                                     }
//                                 }
//                             }
//                             Err(e) => {
//                                 error!(
//                                     "The message transport could not be established: {}",
//                                     e.as_string()
//                                         .unwrap_or_else(|| {
//                                             "The message transport failed with unknown error".to_owned()
//                                         })
//                                         .as_str()
//                                 );
//                                 Err(format!("Failed to extract transport error as string: {:?}", e))
//                             }
//                         }
//                     })
//                 };

//             self.pinger.ping_peers(converted, &message_transport).await;
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messaging::ControlMessage;
    use crate::ping::Ping;
    use mockall::*;
    use more_asserts::*;

    fn simple_ping_config() -> PingConfig {
        PingConfig {
            max_parallel_pings: 2,
            network: "test".to_owned(),
            normalized_version: "1.0a".to_owned(),
            timeout: Duration::from_millis(150),
        }
    }

    #[async_std::test]
    async fn test_ping_peers_with_no_peers_should_not_do_any_api_calls() {
        let mock = MockPingExternalAPI::new();

        let (tx, _rx_ping) = futures::channel::mpsc::unbounded::<(PeerId, ControlMessage)>();
        let (_tx_pong, rx) = futures::channel::mpsc::unbounded::<(PeerId, std::result::Result<ControlMessage, ()>)>();

        let mut pinger = Ping::new(simple_ping_config(), tx, rx, Box::new(mock));
        pinger.ping(vec![]).await;
    }

    #[async_std::test]
    async fn test_ping_peers_with_happy_path_should_trigger_the_desired_external_api_calls() {
        let (tx, mut rx_ping) = futures::channel::mpsc::unbounded::<(PeerId, ControlMessage)>();
        let (mut tx_pong, rx) = futures::channel::mpsc::unbounded::<(PeerId, std::result::Result<ControlMessage, ()>)>();

        let peer = PeerId::random();

        let mut mock = MockPingExternalAPI::new();
        mock.expect_on_finished_ping()
            .with(
                predicate::eq(peer),
                predicate::function(|x: &crate::types::Result| x.is_ok()),
            )
            .return_const(());

        let ideal_single_use_channel = async move {
            if let Some((peer, challenge)) = rx_ping.next().await {
                let _ = tx_pong.start_send((peer, Ok(ControlMessage::generate_pong_response(&challenge).expect("valid challenge"))));
            };
        };

        let mut pinger = Ping::new(simple_ping_config(), tx, rx, Box::new(mock));
        futures::join!(
            pinger.ping(vec![peer.clone()]),
            ideal_single_use_channel
        );
    }

    #[async_std::test]
    async fn test_ping_should_invoke_a_failed_ping_reply_for_an_incorrect_reply() {
        let (tx, mut rx_ping) = futures::channel::mpsc::unbounded::<(PeerId, ControlMessage)>();
        let (mut tx_pong, rx) = futures::channel::mpsc::unbounded::<(PeerId, std::result::Result<ControlMessage, ()>)>();

        let peer = PeerId::random();

        let mut mock = MockPingExternalAPI::new();
        mock.expect_on_finished_ping()
            .with(
                predicate::eq(peer.clone()),
                predicate::function(|x: &crate::types::Result| x.is_err()),
            )
            .return_const(());

        let bad_pong_single_use_channel = async move {
            if let Some((peer, challenge)) = rx_ping.next().await {
                let _ = tx_pong.start_send((peer, Ok(challenge)));
            };
        };

        let mut pinger = Ping::new(simple_ping_config(), tx, rx, Box::new(mock));
        futures::join!(
            pinger.ping(vec![peer.clone()]),
            bad_pong_single_use_channel
        );
    }

    #[async_std::test]
    async fn test_ping_peer_times_out_on_the_pong() {
        let (tx, mut rx_ping) = futures::channel::mpsc::unbounded::<(PeerId, ControlMessage)>();
        let (mut tx_pong, rx) = futures::channel::mpsc::unbounded::<(PeerId, std::result::Result<ControlMessage, ()>)>();

        let peer = PeerId::random();
        let mut ping_config = simple_ping_config();
        ping_config.timeout = Duration::from_millis(0);

        let mut mock = MockPingExternalAPI::new();
        mock.expect_on_finished_ping()
            .with(
                predicate::eq(peer),
                predicate::function(|x: &crate::types::Result| x.is_err()),
            )
            .return_const(());

        // NOTE: timeout is ensure by the libp2p protocol handling, only error arrives
        // from the channel
        let timeout_single_use_channel = async move {
            if let Some((peer, challenge)) = rx_ping.next().await {
                let _ = tx_pong.start_send((peer, Err(())));
            };
        };

        let mut pinger = Ping::new(simple_ping_config(), tx, rx, Box::new(mock));
        futures::join!(
            pinger.ping(vec![peer.clone()]),
            timeout_single_use_channel
        );
    }

    #[async_std::test]
    async fn test_ping_peers_multiple_peers_are_pinged_in_parallel() {
        let (tx, mut rx_ping) = futures::channel::mpsc::unbounded::<(PeerId, ControlMessage)>();
        let (mut tx_pong, rx) = futures::channel::mpsc::unbounded::<(PeerId, std::result::Result<ControlMessage, ()>)>();

        let peers = vec![PeerId::random(), PeerId::random()];

        let mut mock = MockPingExternalAPI::new();
        mock.expect_on_finished_ping()
            .with(
                predicate::eq(peers[0].clone()),
                predicate::function(|x: &crate::types::Result| x.is_ok()),
            )
            .return_const(());
        mock.expect_on_finished_ping()
            .with(
                predicate::eq(peers[1].clone()),
                predicate::function(|x: &crate::types::Result| x.is_ok()),
            )
            .return_const(());
        
        let ideal_twice_usable_channel = async move {
            for _ in 0..2 {
                if let Some((peer, challenge)) = rx_ping.next().await {
                    let _ = tx_pong.start_send((peer, Ok(ControlMessage::generate_pong_response(&challenge).expect("valid challenge"))));
                };
            }
        };

        let mut pinger = Ping::new(simple_ping_config(), tx, rx, Box::new(mock));
        futures::join!(
            pinger.ping(peers),
            ideal_twice_usable_channel
        );    
    }

    #[async_std::test]
    async fn test_ping_peers_should_ping_parallel_only_a_limited_number_of_peers() {
        let (tx, mut rx_ping) = futures::channel::mpsc::unbounded::<(PeerId, ControlMessage)>();
        let (mut tx_pong, rx) = futures::channel::mpsc::unbounded::<(PeerId, std::result::Result<ControlMessage, ()>)>();

        let mut config = simple_ping_config();
        config.max_parallel_pings = 1;

        let ping_delay = 10u64;

        let peers = vec![PeerId::random(), PeerId::random()];

        let mut mock = MockPingExternalAPI::new();
        mock.expect_on_finished_ping()
            .with(
                predicate::eq(peers[0].clone()),
                predicate::function(|x: &crate::types::Result| x.is_ok()),
            )
            .return_const(());
        mock.expect_on_finished_ping()
            .with(
                predicate::eq(peers[1].clone()),
                predicate::function(|x: &crate::types::Result| x.is_ok()),
            )
            .return_const(());

        let ideal_twice_usable_linearly_delaying_channel = async move {
            for i in 0..2 {
                if let Some((peer, challenge)) = rx_ping.next().await {
                    sleep(std::time::Duration::from_millis(ping_delay * i)).await;
                    let _ = tx_pong.start_send((peer, Ok(ControlMessage::generate_pong_response(&challenge).expect("valid challenge"))));
                };
            }
        };

        let mut pinger = Ping::new(simple_ping_config(), tx, rx, Box::new(mock));
        
        let start = current_timestamp();
        futures::join!(
            pinger.ping(peers),
            ideal_twice_usable_linearly_delaying_channel
        );  
        let end = current_timestamp();

        assert_ge!(end - start, ping_delay);
    }
}
