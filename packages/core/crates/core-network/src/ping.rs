use async_trait::async_trait;
use std::{rc::Rc, time::Duration};

use futures::{
    future::{
        select,
        Either,
        FutureExt, // .fuse()
    },
    pin_mut,
    stream::{FuturesUnordered, StreamExt},
};
use js_sys::{Array, Function, JsString, Promise, Uint8Array};
use libp2p::PeerId;

use utils_log::{debug, error, info};
use utils_metrics::{
    histogram_start_measure,
    metrics::{SimpleCounter, SimpleHistogram},
};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

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

/// Basic type used for internally aggregating ping results to be further processed in the
/// PingExternalAPI callbacks.
type PingMeasurement = (PeerId, crate::types::Result);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PingConfig {
    max_parallel_pings: usize,
    environment_id: String,
    normalized_version: String,
    timeout: Duration,
}

impl PingConfig {
    pub fn new(
        max_parallel_pings: usize,
        environment_id: String,
        normalized_version: String,
        timeout: Duration,
    ) -> Self {
        Self {
            max_parallel_pings,
            environment_id,
            normalized_version,
            timeout,
        }
    }
}

#[async_trait(?Send)]
pub trait PingExternalAPI {
    async fn run(&self, msg: Box<[u8]>, peer: &PeerId) -> Result<Box<[u8]>, String>;
}

pub struct WasmPingApi {
    send_message_cb: Function,
}

impl WasmPingApi {
    pub fn new(send_message_cb: Function) -> Self {
        Self { send_message_cb }
    }
}

#[async_trait(?Send)]
impl PingExternalAPI for WasmPingApi {
    async fn run(&self, msg: Box<[u8]>, peer: &PeerId) -> Result<Box<[u8]>, String> {
        let js_this = JsValue::null();
        let data: JsValue = js_sys::Uint8Array::from(msg.as_ref()).into();
        let peer: JsValue = JsString::from(peer.to_string()).into();

        // call a send_msg_cb producing a JS promise that is further converted to a Future
        // holding the reply of the pinged peer for the ping message.
        match self.send_message_cb.call2(&js_this, &data, &peer) {
            Ok(r) => {
                let promise = Promise::from(r);
                let data = JsFuture::from(promise)
                    .await
                    .map(|x| Array::from(x.as_ref()).get(0))
                    .map(|x| Uint8Array::from(x).to_vec().into_boxed_slice())
                    .map_err(|x| {
                        x.dyn_ref::<JsString>()
                            .map_or("Failed to send ping message".to_owned(), |x| -> String { x.into() })
                    });

                data
            }
            Err(e) => {
                error!(
                    "The message transport could not be established: {}",
                    e.as_string()
                        .unwrap_or_else(|| { "The message transport failed with unknown error".to_owned() })
                        .as_str()
                );
                Err(format!("Failed to extract transport error as string: {:?}", e))
            }
        }
        // self.render_fullscreen().await;
        // for _ in 0..4u16 {
        //     remind_user_to_join_mailing_list().await;
        // }
        // self.hide_for_now().await;
    }
}

pub struct Ping<API> {
    config: PingConfig,
    _protocol_heartbeat: [String; 2],
    on_finished_ping: Box<dyn Fn(&PeerId, crate::types::Result) + 'static>,
    send_msg: Rc<API>,
    metric_time_to_heartbeat: Option<SimpleHistogram>,
    metric_time_to_ping: Option<SimpleHistogram>,
    metric_successful_ping_count: Option<SimpleCounter>,
    metric_failed_ping_count: Option<SimpleCounter>,
}

impl<API: PingExternalAPI> Ping<API> {
    pub fn new(
        config: PingConfig,
        on_finished_ping: impl Fn(&PeerId, crate::types::Result) + 'static,
        api: API,
    ) -> Self {
        let config = PingConfig {
            max_parallel_pings: config.max_parallel_pings.min(PINGS_MAX_PARALLEL),
            ..config
        };

        Ping {
            _protocol_heartbeat: [
                // new
                format!(
                    "/hopr/{}/heartbeat/{}",
                    &config.environment_id, &config.normalized_version
                ),
                // deprecated
                format!("/hopr/{}/heartbeat", &config.environment_id),
            ],
            config,
            on_finished_ping: Box::new(on_finished_ping),
            send_msg: Rc::new(api),
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
    pub async fn ping_peers(&self, mut peers: Vec<PeerId>) {
        if peers.is_empty() {
            debug!("Received an empty peer list, not pinging any peers");
            ()
        }

        let heartbeat_round_timer = if let Some(metric_time_to_heartbeat) = &self.metric_time_to_heartbeat {
            Some(histogram_start_measure!(metric_time_to_heartbeat))
        } else {
            None
        };

        let remainder = peers.split_off(self.config.max_parallel_pings.min(peers.len()));

        let start = current_timestamp();

        let mut futs = {
            let this = &*self;

            let futs = to_futures_unordered(
                peers
                    .iter()
                    .map(|x| this.ping_peer(x.clone(), self.config.timeout))
                    .collect::<Vec<_>>(),
            );

            futs
        };

        let mut waiting = remainder.iter();
        while let Some(heartbeat) = futs.next().await {
            if let Some(v) = waiting.next() {
                let remaining_time = current_timestamp() - start;
                if (remaining_time as u128) < self.config.timeout.as_millis() {
                    futs.push(self.ping_peer(v.clone(), Duration::from_millis(remaining_time)));
                }
            }

            (self.on_finished_ping)(&heartbeat.0, heartbeat.1);
        }

        if let Some(metric_time_to_heartbeat) = &self.metric_time_to_heartbeat {
            metric_time_to_heartbeat.cancel_measure(heartbeat_round_timer.unwrap());
        };
    }

    /// Ping a single peer respecting a specified timeout duration.
    async fn ping_peer(&self, destination: PeerId, timeout_duration: Duration) -> PingMeasurement {
        info!("Pinging peer '{}'", destination);
        let sent_ping = ControlMessage::generate_ping_request();

        let ping_result: PingMeasurement = {
            let ping_peer_timer = if let Some(metric_time_to_ping) = &self.metric_time_to_ping {
                Some(histogram_start_measure!(metric_time_to_ping))
            } else {
                None
            };

            let timeout = sleep(std::cmp::min(timeout_duration, self.config.timeout)).fuse();
            let send_message_clone = self.send_msg.clone();
            let ping = async move {
                send_message_clone
                    .run(sent_ping.get_ping_message().unwrap().serialize(), &destination)
                    .await
            }
            .fuse();

            pin_mut!(timeout, ping);

            let ping_result: Result<(), NetworkingError> = match select(timeout, ping).await {
                Either::Left(_) => Err(Timeout(timeout_duration.as_secs())),
                Either::Right((v, _)) => match v {
                    Ok(received) => PingMessage::deserialize(received.as_ref())
                        .map_err(|_| DecodingError)
                        .and_then(|deserialized| {
                            ControlMessage::validate_pong_response(&sent_ping, &Pong(deserialized))
                        }),
                    Err(description) => Err(Other(format!(
                        "Ping to peer '{}' failed with: {}",
                        destination.to_string(),
                        description
                    ))),
                },
            };

            if let Some(metric_time_to_ping) = &self.metric_time_to_ping {
                metric_time_to_ping.cancel_measure(ping_peer_timer.unwrap());
            }

            match &ping_result {
                Ok(_) => info!("Successfully pinged peer {}", destination),
                Err(e) => error!("Ping to peer {} failed with error: {}", destination, e),
            }

            (
                destination,
                if ping_result.is_ok() {
                    Ok(current_timestamp())
                } else {
                    Err(())
                },
            )
        };

        match ping_result.1 {
            Ok(_) => {
                if let Some(metric_successful_ping_count) = &self.metric_successful_ping_count {
                    metric_successful_ping_count.increment();
                };
            }
            Err(_) => {
                if let Some(metric_failed_ping_count) = &self.metric_failed_ping_count {
                    metric_failed_ping_count.increment();
                };
            }
        }

        ping_result
    }
}

/// Add multiple futures into a FuturesUnordered obejct to setup concurrent execution of futures
fn to_futures_unordered<F>(mut fs: Vec<F>) -> FuturesUnordered<F> {
    let futures: FuturesUnordered<F> = FuturesUnordered::new();
    for f in fs.drain(..) {
        futures.push(f);
    }

    futures
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messaging::ControlMessage;
    use crate::ping::Ping;
    use async_trait::async_trait;
    use more_asserts::*;
    use std::str::FromStr;

    fn simple_ping_config() -> PingConfig {
        PingConfig {
            max_parallel_pings: 2,
            environment_id: "test".to_owned(),
            normalized_version: "1.0a".to_owned(),
            timeout: Duration::from_millis(150),
        }
    }

    const BAD_PEER: &'static str = "1Ank4EeHLAd3bwwtJma1WsXYSmiGgqmjkQoCUpevx67ix8";
    const PEER_DELAYED_1_MS: &'static str = "1AcPsXRKVc3U64NBb4obUUT34jSLWtvAz2JMw192L92QKW";
    const PEER_DELAYED_2_MS: &'static str = "1Ag7Agu1thSZFRGyoPWydqCiFS6tJFXLeukpVjCtwy1V8m";
    const PEER_DELAYED_10_MS: &'static str = "1AduBfEp4KpymzNGzVAmsoF5RhHReRa8LfSjBoDwwWz96s";
    const PEER_DELAYED_11_MS: &'static str = "1AWpJQyRSosNeKKZAGzA9VLue6qdC1yyUv594F6W8Jasjn";

    pub struct TestingSendPing {}

    #[async_trait(?Send)]
    impl PingExternalAPI for TestingSendPing {
        async fn run(&self, msg: Box<[u8]>, peer: &PeerId) -> Result<Box<[u8]>, String> {
            let mut reply = PingMessage::deserialize(msg.as_ref())
                .map_err(|_| DecodingError)
                .and_then(|chall| ControlMessage::generate_pong_response(&ControlMessage::Ping(chall)))
                .and_then(|msg| msg.get_ping_message().map(PingMessage::serialize))
                .unwrap();

            match peer.to_string().as_str() {
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
    }

    #[async_std::test]
    async fn test_ping_peers_with_no_peers_should_not_do_any_api_calls() {
        let pinger = Ping::new(
            simple_ping_config(),
            |_peer: &PeerId, _result: crate::types::Result| {},
            TestingSendPing {},
        );
        pinger.ping_peers(vec![]).await;
    }

    #[async_std::test]
    async fn test_ping_peers_with_happy_path_should_trigger_the_desired_external_api_calls() {
        let peer = PeerId::random();

        let pinger = Ping::new(
            simple_ping_config(),
            move |pinged_peer: &PeerId, result: crate::types::Result| {
                if !peer.eq(pinged_peer) || !result.is_ok() {
                    panic!("incorrect arguments")
                }
            },
            TestingSendPing {},
        );
        pinger.ping_peers(vec![peer.clone()]).await;
    }

    #[async_std::test]
    async fn test_ping_should_invoke_a_failed_ping_reply_for_an_incorrect_reply() {
        let bad_peer = PeerId::from_str("1Ank4EeHLAd3bwwtJma1WsXYSmiGgqmjkQoCUpevx67ix8")
            .ok()
            .unwrap();

        let pinger = Ping::new(
            simple_ping_config(),
            move |pinged_peer: &PeerId, result: crate::types::Result| {
                if !bad_peer.eq(pinged_peer) || !result.is_err() {
                    panic!("incorrect arguments")
                }
            },
            TestingSendPing {},
        );
        pinger.ping_peers(vec![bad_peer.clone()]).await;
    }

    #[async_std::test]
    async fn test_ping_peer_invokes_send_message_but_the_time_runs_out() {
        let peer = PeerId::random();
        let mut ping_config = simple_ping_config();
        ping_config.timeout = Duration::from_millis(0);

        let pinger = Ping::new(
            ping_config,
            move |pinged_peer: &PeerId, result: crate::types::Result| {
                if !peer.eq(pinged_peer) || !result.is_err() {
                    panic!("incorrect arguments")
                }
            },
            TestingSendPing {},
        );
        pinger.ping_peers(vec![peer.clone()]).await;
    }

    #[async_std::test]
    async fn test_ping_peers_empty_list_will_not_trigger_any_pinging() {
        let pinger = Ping::new(
            simple_ping_config(),
            |_pinged_peer: &PeerId, _result: crate::types::Result| {},
            TestingSendPing {},
        );

        pinger.ping_peers(vec![]).await;
    }

    #[async_std::test]
    async fn test_ping_peers_multiple_peers_are_pinged_in_parallel() {
        let peers = vec![
            PeerId::from_str(PEER_DELAYED_1_MS).ok().unwrap(),
            PeerId::from_str(PEER_DELAYED_2_MS).ok().unwrap(),
        ];

        let peers_clone = peers.clone();

        let pinger = Ping::new(
            simple_ping_config(),
            move |pinged_peer: &PeerId, result: crate::types::Result| {
                if (!peers_clone[0].eq(pinged_peer) || !result.is_ok())
                    && (!peers_clone[1].clone().eq(pinged_peer) || !result.is_ok())
                {
                    panic!("incorrect arguments")
                }
            },
            TestingSendPing {},
        );
        pinger.ping_peers(peers).await;
    }

    #[async_std::test]
    async fn test_ping_peers_should_ping_parallel_only_a_limited_number_of_peers() {
        let mut config = simple_ping_config();
        config.max_parallel_pings = 1;

        let ping_delay_first = 10u64;
        let ping_delay_second = 10u64;

        let peers = vec![
            PeerId::from_str(PEER_DELAYED_10_MS).ok().unwrap(),
            PeerId::from_str(PEER_DELAYED_11_MS).ok().unwrap(),
        ];

        let peers_clone = peers.clone();

        let pinger = Ping::new(
            config,
            move |pinged_peer: &PeerId, result: crate::types::Result| {
                if (!peers_clone[0].eq(pinged_peer) || !result.is_ok())
                    && (!peers_clone[1].clone().eq(pinged_peer) || !result.is_ok())
                {
                    panic!("incorrect arguments")
                }
            },
            TestingSendPing {},
        );

        let start = current_timestamp();
        pinger.ping_peers(peers).await;
        let end = current_timestamp();

        assert_ge!(end - start, ping_delay_first + ping_delay_second)
    }

    #[async_std::test]
    async fn test_ping_peer_should_not_ping_all_peers_if_the_max_timeout_is_reached() {
        let mut config = simple_ping_config();
        config.timeout = Duration::from_millis(1);
        config.max_parallel_pings = 1;

        let peers = vec![
            PeerId::from_str(PEER_DELAYED_10_MS).ok().unwrap(),
            PeerId::from_str(PEER_DELAYED_11_MS).ok().unwrap(),
        ];

        let peers_clone = peers.clone();

        let pinger = Ping::new(
            config,
            move |pinged_peer: &PeerId, result: crate::types::Result| {
                if !peers_clone[0].eq(pinged_peer) || !result.is_ok() {
                    panic!("incorrect arguments")
                }
            },
            TestingSendPing {},
        );

        pinger.ping_peers(peers).await;
    }
}
