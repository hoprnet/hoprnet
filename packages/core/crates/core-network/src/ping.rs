use std::{time::Duration, pin::Pin, result};

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

use crate::errors::NetworkingError;
use crate::errors::NetworkingError::{DecodingError, Other, Timeout};
use crate::messaging::{ControlMessage, PingMessage};
use utils_types::traits::BinarySerializable;

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
    fn get_peers(&self, from_timestamp: u64) -> Vec<PeerId>;
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

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Ping {
    config: PingConfig,
    active_pings: std::collections::HashMap<PeerId, (u64, ControlMessage)>,
    send_ping: HeartbeatSendPingTx,
    receive_pong: HeartbeatGetPongRx,
    external_api: Box<dyn PingExternalAPI>,
    metric_time_to_heartbeat: Option<SimpleHistogram>,
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
            metric_time_to_heartbeat: SimpleHistogram::new(
                "core_histogram_heartbeat_time_seconds",
                "Measures total time it takes to probe all other nodes (in seconds)",
                vec![0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0, 60.0, 90.0, 120.0, 300.0],
            )
            .ok(),
            metric_time_to_ping: SimpleHistogram::new(
                "core_histogram_ping_time_seconds",
                "Measures total time it takes to ping a single node (seconds)",
                vec![0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0, 60.0, 90.0, 120.0, 300.0],
            )
            .ok(),
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

    /// Heartbeat loop responsible for periodically requesting peers to ping around from the 
    /// external API interface.
    /// 
    /// The loop never ends and will run indefinitely, until the program is explicitly terminated.
    /// As such, this feature should therefore be joined with other internal loops and awaited
    /// after all components have been initialized.
    pub async fn heartbeat_loop(&mut self) {
        loop {
            let heartbeat_round_timer = if let Some(metric_time_to_heartbeat) = &self.metric_time_to_heartbeat {
                Some(histogram_start_measure!(metric_time_to_heartbeat))
            } else {
                None
            };

            let start = current_timestamp();
            // let ping = futures::future::pending().fuse();

            // pin_mut!(timeout, ping);

            // TODO: select over this
            //let timeout = sleep(self.config.timeout).fuse();
            // let from_timestamp = if start > self.config.heartbeat_threshold { start - self.config.heartbeat_threshold } else { start };
            // self.external_api.get_peers(from_timestamp);
        // try {
        //     const thresholdTime = Date.now() - Number(this.config.heartbeat_threshold)
        //     log(`Checking nodes since ${thresholdTime} (${new Date(thresholdTime).toLocaleString()})`)
    
        //     await this.pinger.ping(this.networkPeers.peers_to_ping(BigInt(thresholdTime)))
        //   } catch (err) {
        //     log('FATAL ERROR IN HEARTBEAT', err)
            // sleep if pinging was too fast

            if let Some(metric_time_to_heartbeat) = &self.metric_time_to_heartbeat {
                metric_time_to_heartbeat.record_measure(heartbeat_round_timer.unwrap());
            };

            sleep(std::time::Duration::from_millis(0u64.max(current_timestamp() - start))).await
        }

    }

    /// Performs multiple concurrent async pings to the specified peers.
    ///
    /// A sliding window mechanism is used to select at most a fixed number of concurrently processed
    /// peers in order to stabilize the pinging mechanism over a fixed time window. Pings that do not
    /// fit into that time window do not produce a viable result and do not trigger an external
    /// api call to register the result.
    ///
    /// # Arguments
    ///
    /// * `peers` - A vector of PeerId objects referencing the peers to be pinged
    /// * `send_msg` - The send function producing a Future with the reply of the pinged peer
    pub async fn ping_peers(&mut self, mut peers: Vec<PeerId>)
    {
        // TODO: cannot see this lifecycle at this point
        // let ping_peer_timer = if let Some(metric_time_to_ping) = &self.metric_time_to_ping {
        //    Some(histogram_start_measure!(metric_time_to_ping))
        // } else {
        //     None
        // };
        //
        // if let Some(metric_time_to_ping) = &self.metric_time_to_ping {
        //     metric_time_to_ping.record_measure(ping_peer_timer.unwrap());
        // }

        let start_all_peers = current_timestamp();

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

                    let (start, challenge) = record.expect("Should hold a value at this point");
                    let duration: std::result::Result<std::time::Duration, ()> = {
                        if ControlMessage::validate_pong_response(&challenge, &pong).is_ok() {
                            info!("Successfully pinged peer {}", peer);
                            Ok(std::time::Duration::from_millis(current_timestamp() - start))
                        } else {
                            error!("Failed to verify the challenge for ping to peer: {}", peer.to_string());
                            Err(())
                        }
                    };

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

    fn initiate_peer_ping(&mut self, peer: &PeerId) -> bool {
        if ! self.active_pings.contains_key(&peer) {
            info!("Pinging peer '{}'", peer);

            let ping_challenge: ControlMessage = ControlMessage::generate_ping_request();
            let _ = self.active_pings.insert(peer.clone(), (current_timestamp(), ping_challenge.clone()));
            self.send_ping.start_send((peer.clone(), ping_challenge));
            return true
        }

        false
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
    use std::str::FromStr;

    fn simple_ping_config() -> PingConfig {
        PingConfig {
            max_parallel_pings: 2,
            network: "test".to_owned(),
            normalized_version: "1.0a".to_owned(),
            timeout: Duration::from_millis(150),
        }
    }

    const BAD_PEER: &'static str = "1Ank4EeHLAd3bwwtJma1WsXYSmiGgqmjkQoCUpevx67ix8";
    const PEER_DELAYED_1_MS: &'static str = "1AcPsXRKVc3U64NBb4obUUT34jSLWtvAz2JMw192L92QKW";
    const PEER_DELAYED_2_MS: &'static str = "1Ag7Agu1thSZFRGyoPWydqCiFS6tJFXLeukpVjCtwy1V8m";
    const PEER_DELAYED_10_MS: &'static str = "1AduBfEp4KpymzNGzVAmsoF5RhHReRa8LfSjBoDwwWz96s";
    const PEER_DELAYED_11_MS: &'static str = "1AWpJQyRSosNeKKZAGzA9VLue6qdC1yyUv594F6W8Jasjn";

    // Testing override
    pub async fn send_ping(msg: Box<[u8]>, peer: String) -> Result<Box<[u8]>, String> {
        let mut reply = PingMessage::from_bytes(msg.as_ref())
            .map_err(|_| DecodingError)
            .and_then(|chall| ControlMessage::generate_pong_response(&ControlMessage::Ping(chall)))
            .and_then(|msg| msg.get_ping_message().map(PingMessage::to_bytes))
            .unwrap();

        match peer.as_str() {
            BAD_PEER => {
                // let's damage the reply bytes
                for x in reply.iter_mut() {
                    if *x < u8::MAX {
                        *x = *x + 1;
                    }
                }
            }
            PEER_DELAYED_1_MS => std::thread::sleep(Duration::from_millis(1)),
            PEER_DELAYED_2_MS => std::thread::sleep(Duration::from_millis(2)),
            PEER_DELAYED_10_MS => std::thread::sleep(Duration::from_millis(10)),
            PEER_DELAYED_11_MS => std::thread::sleep(Duration::from_millis(11)),
            _ => (),
        }

        Ok(reply)
    }

    #[async_std::test]
    async fn test_ping_peers_with_no_peers_should_not_do_any_api_calls() {
        let mock = MockPingExternalAPI::new();

        let (tx, rx_ping) = futures::channel::mpsc::unbounded::<(PeerId, ControlMessage)>();
        let (tx_pong, rx) = futures::channel::mpsc::unbounded::<(PeerId, std::result::Result<ControlMessage, ()>)>();

        let mut pinger = Ping::new(simple_ping_config(), tx, rx, Box::new(mock));
        pinger.ping_peers(vec![]).await;
    }

    // #[async_std::test]
    // async fn test_ping_peers_with_happy_path_should_trigger_the_desired_external_api_calls() {
    //     let peer = PeerId::random();

    //     let mut mock = MockPingExternalAPI::new();
    //     mock.expect_on_finished_ping()
    //         .with(
    //             predicate::eq(peer),
    //             predicate::function(|x: &crate::types::Result| x.is_ok()),
    //         )
    //         .return_const(());

    //     let pinger = Ping::new(simple_ping_config(), Box::new(mock));
    //     pinger.ping_peers(vec![peer.clone()], &send_ping).await;
    // }

    // #[async_std::test]
    // async fn test_ping_should_invoke_a_failed_ping_reply_for_an_incorrect_reply() {
    //     let bad_peer = PeerId::from_str("1Ank4EeHLAd3bwwtJma1WsXYSmiGgqmjkQoCUpevx67ix8")
    //         .ok()
    //         .unwrap();

    //     let mut mock = MockPingExternalAPI::new();
    //     mock.expect_on_finished_ping()
    //         .with(
    //             predicate::eq(bad_peer),
    //             predicate::function(|x: &crate::types::Result| x.is_err()),
    //         )
    //         .return_const(());

    //     let pinger = Ping::new(simple_ping_config(), Box::new(mock));
    //     pinger.ping_peers(vec![bad_peer.clone()], &send_ping).await;
    // }

    // #[async_std::test]
    // async fn test_ping_peer_invokes_send_message_but_the_time_runs_out() {
    //     let peer = PeerId::random();
    //     let mut ping_config = simple_ping_config();
    //     ping_config.timeout = Duration::from_millis(0);

    //     let mut mock = MockPingExternalAPI::new();
    //     mock.expect_on_finished_ping()
    //         .with(
    //             predicate::eq(peer),
    //             predicate::function(|x: &crate::types::Result| x.is_err()),
    //         )
    //         .return_const(());

    //     let pinger = Ping::new(ping_config, Box::new(mock));
    //     pinger.ping_peers(vec![peer.clone()], &send_ping).await;
    // }

    // #[async_std::test]
    // async fn test_ping_peers_empty_list_will_not_trigger_any_pinging() {
    //     let mock = MockPingExternalAPI::new();
    //     let pinger = Ping::new(simple_ping_config(), Box::new(mock));

    //     pinger.ping_peers(vec![], &send_ping).await;
    // }

    // #[async_std::test]
    // async fn test_ping_peers_multiple_peers_are_pinged_in_parallel() {
    //     let peers = vec![
    //         PeerId::from_str(PEER_DELAYED_1_MS).ok().unwrap(),
    //         PeerId::from_str(PEER_DELAYED_2_MS).ok().unwrap(),
    //     ];

    //     let mut mock = MockPingExternalAPI::new();
    //     mock.expect_on_finished_ping()
    //         .with(
    //             predicate::eq(peers[0].clone()),
    //             predicate::function(|x: &crate::types::Result| x.is_ok()),
    //         )
    //         .return_const(());
    //     mock.expect_on_finished_ping()
    //         .with(
    //             predicate::eq(peers[1].clone()),
    //             predicate::function(|x: &crate::types::Result| x.is_ok()),
    //         )
    //         .return_const(());

    //     let pinger = Ping::new(simple_ping_config(), Box::new(mock));
    //     pinger.ping_peers(peers, &send_ping).await;
    // }

    // #[async_std::test]
    // async fn test_ping_peers_should_ping_parallel_only_a_limited_number_of_peers() {
    //     let mut config = simple_ping_config();
    //     config.max_parallel_pings = 1;

    //     let ping_delay_first = 10u64;
    //     let ping_delay_second = 10u64;

    //     let peers = vec![
    //         PeerId::from_str(PEER_DELAYED_10_MS).ok().unwrap(),
    //         PeerId::from_str(PEER_DELAYED_11_MS).ok().unwrap(),
    //     ];

    //     let mut mock = MockPingExternalAPI::new();
    //     mock.expect_on_finished_ping()
    //         .with(
    //             predicate::eq(peers[0].clone()),
    //             predicate::function(|x: &crate::types::Result| x.is_ok()),
    //         )
    //         .return_const(());
    //     mock.expect_on_finished_ping()
    //         .with(
    //             predicate::eq(peers[1].clone()),
    //             predicate::function(|x: &crate::types::Result| x.is_ok()),
    //         )
    //         .return_const(());

    //     let pinger = Ping::new(config, Box::new(mock));

    //     let start = current_timestamp();
    //     pinger.ping_peers(peers, &send_ping).await;
    //     let end = current_timestamp();

    //     assert_ge!(end - start, ping_delay_first + ping_delay_second)
    // }

    // #[async_std::test]
    // async fn test_ping_peer_should_not_ping_all_peers_if_the_max_timeout_is_reached() {
    //     let mut config = simple_ping_config();
    //     config.timeout = Duration::from_millis(1);
    //     config.max_parallel_pings = 1;

    //     let peers = vec![
    //         PeerId::from_str(PEER_DELAYED_10_MS).ok().unwrap(),
    //         PeerId::from_str(PEER_DELAYED_11_MS).ok().unwrap(),
    //     ];

    //     let mut mock = MockPingExternalAPI::new();
    //     mock.expect_on_finished_ping()
    //         .with(
    //             predicate::eq(peers[0].clone()),
    //             predicate::function(|x: &crate::types::Result| x.is_ok()),
    //         )
    //         .return_const(());

    //     let pinger = Ping::new(config, Box::new(mock));

    //     pinger.ping_peers(peers, &send_ping).await;
    // }
}
