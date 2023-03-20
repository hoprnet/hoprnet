use std::time::Duration;

use futures::{
    future::{
        Either,
        FutureExt,      // .fuse()
        select
    },
    pin_mut,
    stream::{FuturesUnordered, StreamExt}
};
use libp2p::PeerId;

use utils_log::{debug,info,error};
use utils_metrics::metrics::SimpleCounter;
use utils_metrics::metrics::SimpleHistogram;
use utils_metrics::histogram_start_measure;

#[cfg(any(not(feature = "wasm"), test))]
use utils_misc::time::native::current_timestamp;
#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::sleep as sleep;

#[cfg(all(feature = "wasm", not(test)))]
use utils_misc::time::wasm::current_timestamp;
#[cfg(all(feature = "wasm", not(test)))]
use gloo_timers::future::sleep as sleep;

const PINGS_MAX_PARALLEL: usize = 14;

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
    pub environment_id: String,
    pub normalized_version: String,
    pub timeout: Duration,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Ping {
    config: PingConfig,
    _protocol_heartbeat: [String; 2],
    external_api: Box<dyn PingExternalAPI>,
    metric_time_to_heartbeat: Option<SimpleHistogram>,
    metric_time_to_ping: Option<SimpleHistogram>,
    metric_successful_ping_count: Option<SimpleCounter>,
    metric_failed_ping_count: Option<SimpleCounter>,
}

impl Ping {
    pub fn new(config: PingConfig, external_api: Box<dyn PingExternalAPI>) -> Ping {
        let config = PingConfig {
            max_parallel_pings: config.max_parallel_pings.min(PINGS_MAX_PARALLEL),
            ..config
        };

        Ping {
            _protocol_heartbeat: [
                // new
                format!("/hopr/{}/heartbeat/{}", &config.environment_id, &config.normalized_version),
                // deprecated
                format!("/hopr/{}/heartbeat", &config.environment_id)
            ],
            config,
            external_api,
            metric_time_to_heartbeat: SimpleHistogram::new(
                "core_histogram_heartbeat_time_seconds",
                "Measures total time it takes to probe all other nodes (in seconds)",
                vec![0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0, 60.0, 90.0, 120.0, 300.0]
            ).ok(),
            metric_time_to_ping: SimpleHistogram::new(
                "core_histogram_ping_time_seconds",
                "Measures total time it takes to ping a single node (seconds)",
                vec![0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0, 60.0, 90.0, 120.0, 300.0]
            ).ok(),
            metric_successful_ping_count: SimpleCounter::new(
                "core_counter_heartbeat_successful_pings",
                "Total number of successful pings"
            ).ok(),
            metric_failed_ping_count: SimpleCounter::new(
                "core_counter_heartbeat_failed_pings",
                "Total number of failed pings"
            ).ok(),
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
    pub async fn ping_peers<F>(&self, mut peers: Vec<PeerId>,
                                 send_msg: &impl Fn(Box<[u8]>, String) -> F)
    where
        F: futures::Future<Output =  Result<Box<[u8]>, String>>
    {
        if peers.is_empty() {
            debug!("Received an empty peer list, not pinging any peers");
            ()
        }

        let _ping_peers_timer = match &self.metric_time_to_heartbeat {
            Some(metric_time_to_heartbeat) => {
                let timer = histogram_start_measure!(metric_time_to_heartbeat);
                Some(scopeguard::guard((), move |_| { metric_time_to_heartbeat.cancel_measure(timer); }))
            },
            None => None
        };

        let remainder = peers.split_off(self.config.max_parallel_pings.min(peers.len()));

        let start = current_timestamp();
        let mut futs = to_futures_unordered(
            peers.iter()
                .map(|x| { self.ping_peer(x.clone(), self.config.timeout, send_msg)})
                .collect::<Vec<_>>()
        );

        let mut waiting = remainder.iter();
        while let Some(heartbeat) = futs.next().await {
            if let Some(v) = waiting.next() {
                let remaining_time = current_timestamp() - start;
                if (remaining_time as u128) < self.config.timeout.as_millis() {
                    futs.push(self.ping_peer(v.clone(), Duration::from_millis(remaining_time), send_msg));
                }
            }

            self.external_api.on_finished_ping(&heartbeat.0, heartbeat.1);
        }
    }

    /// Ping a single peer respecting a specified timeout duration.
    async fn ping_peer<F>(&self, destination: PeerId, timeout_duration: Duration,
                       send_msg: &impl Fn(Box<[u8]>, String) -> F) -> PingMeasurement
    where
        F: futures::Future<Output = Result<Box<[u8]>, String>>
    {
        info!("Pinging peer '{}'", destination);

        use rand::{RngCore,SeedableRng};
        let mut challenge = Box::new([0u8; 16]);
        rand::rngs::StdRng::from_entropy().fill_bytes(&mut challenge.as_mut_slice());

        let ping_result: PingMeasurement = {
            let _ping_peer_timer = match &self.metric_time_to_ping {
                Some(metric_time_to_ping) => {
                    let timer = histogram_start_measure!(metric_time_to_ping);
                    Some(scopeguard::guard((), move |_| { metric_time_to_ping.cancel_measure(timer); }))
                },
                None => None
            };

            let timeout = sleep(std::cmp::min(timeout_duration, self.config.timeout)).fuse();
            let ping = async {
                send_msg(challenge.clone(), destination.to_string()).await
            }.fuse();

            pin_mut!(timeout, ping);

            let ping_result: Result<(), String> = match select(
                timeout, ping).await
            {
                Either::Left(_) => Err(format!("The ping timed out {}s", timeout_duration.as_secs())),
                Either::Right((v, _)) => match v {
                    Ok(received) => {
                        let expected = crate::heartbeat::generate_ping_response(challenge.clone());
                        match compare(expected.as_ref(), received.as_ref()) {
                            std::cmp::Ordering::Equal => Ok(()),
                            _ => Err(format!("Received incorrect reply for challenge, expected '{:x?}', but received: {:x?}", expected.as_ref(), received.as_ref()))
                        }
                    }
                    Err(description) => {
                        Err(format!("Error during ping to peer '{}': {}", destination.to_string(), description))
                    }
                },
            };

            info!("Ping finished for peer {} with: {:?}", destination, ping_result);

            (
                destination,
                if ping_result.is_ok() { Ok(current_timestamp()) } else { Err(()) }
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

/// Compare functions to create ordering result for array of u8s
fn compare(a: &[u8], b: &[u8]) -> std::cmp::Ordering {
    if a.len() != b.len() {
        return a.len().cmp(&b.len());
    }

    for (ai, bi) in a.iter().zip(b.iter()) {
        match ai.cmp(&bi) {
            std::cmp::Ordering::Equal => continue,
            ord => return ord
        }
    }

    a.len().cmp(&b.len())
}


#[cfg(feature = "wasm")]
pub mod wasm {
    use super::*;
    use std::str::FromStr;
    use js_sys::JsString;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    #[wasm_bindgen]
    struct WasmPingApi {
        _environment_id: String,
        _version: String,
        on_finished_ping_cb: js_sys::Function,
    }

    impl PingExternalAPI for WasmPingApi {
        fn on_finished_ping(&self, peer: &PeerId, result: crate::types::Result) {
            let this = JsValue::null();
            let peer = JsValue::from(peer.to_base58());
            let res = {
                if let Ok(v) = result {
                    JsValue::from(v as f64)
                } else {
                    JsValue::undefined()
                }
            };

            if let Err(err) = self.on_finished_ping_cb.call2(&this, &peer, &res) {
                error!("Failed to perform on peer offline operation with: {}", err.as_string().unwrap().as_str())
            };
        }
    }

    /// WASM wrapper for the Ping that accepts a JS function for sending a ping message and returing
    /// a Future for the peer reply.
    #[wasm_bindgen]
    pub struct Pinger {
        pinger: Ping,
        send_msg_cb: js_sys::Function,
    }

    #[wasm_bindgen]
    impl Pinger {
        #[wasm_bindgen]
        pub fn build(environment_id: String, version: String,
                     on_finished_ping_cb: js_sys::Function,
                     send_msg_cb: js_sys::Function) -> Self {
            let api = Box::new(WasmPingApi {
                _environment_id: environment_id.clone(),
                _version: version.clone(),
                on_finished_ping_cb,
            });

            let config = PingConfig {
                environment_id,
                normalized_version: version,
                max_parallel_pings: PINGS_MAX_PARALLEL,
                timeout: Duration::from_secs(30)
            };

            Self {
                pinger: Ping::new(config, api),
                send_msg_cb
            }
        }

        /// Ping the peers represented as a Vec<JsString> values that are converted into usable
        /// PeerIds.
        ///
        /// # Arguments
        /// * `peers` - Vector of String representations of the PeerIds to be pinged.
        #[wasm_bindgen]
        pub async fn ping(&self, mut peers: Vec<JsString>) {
            let converted = peers.drain(..)
                .map(|x| {
                    let x: String = x.into();
                    PeerId::from_str(&x).ok().unwrap()
                })
                .collect::<Vec<_>>();

            let message_transport = |msg: Box<[u8]>, peer: String| -> std::pin::Pin<Box<dyn futures::Future<Output = Result<Box<[u8]>,String>>>> {
                Box::pin(async move {
                    let this = JsValue::null();
                    let data: JsValue = js_sys::Uint8Array::from(msg.as_ref()).into();
                    let peer: JsValue = JsString::from(peer.as_str()).into();

                    // call a send_msg_cb producing a JS promise that is further converted to a Future
                    // holding the reply of the pinged peer for the ping message.
                    match self.send_msg_cb.call2(&this, &data, &peer) {
                        Ok(r) => {
                            let promise = js_sys::Promise::from(r);
                            let data =
                                wasm_bindgen_futures::JsFuture::from(promise).await
                                    .map(|x| js_sys::Array::from(x.as_ref()).get(0))
                                    .map(|x| js_sys::Uint8Array::new(&x).to_vec().into_boxed_slice())
                                    .map_err(|x| x.dyn_ref::<JsString>()
                                        .map_or("Failed to send ping message".to_owned(), |x| -> String { x.into() }));

                            data
                        },
                        Err(e) => {
                            error!("The message transport could not be established: {}", e.as_string().unwrap().as_str());
                            Err(format!("Failed to extract transport error as string: {:?}", e))
                        }
                    }
                })
            };

            self.pinger.ping_peers(converted, &message_transport).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::*;
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

    // Testing override
    pub async fn send_ping(msg: Box<[u8]>, peer: String) -> Result<Box<[u8]>, String> {
        let mut reply = crate::heartbeat::generate_ping_response(msg);

        match peer.as_str() {
            BAD_PEER => {
                // let's damage the reply bytes
                for x in reply.iter_mut() {
                    if *x < u8::MAX {
                        *x = *x + 1;
                    }
                }
            },
            PEER_DELAYED_1_MS =>  std::thread::sleep(Duration::from_millis(1)),
            PEER_DELAYED_2_MS =>  std::thread::sleep(Duration::from_millis(2)),
            PEER_DELAYED_10_MS =>  std::thread::sleep(Duration::from_millis(10)),
            PEER_DELAYED_11_MS =>  std::thread::sleep(Duration::from_millis(11)),
            _ => ()
        }

        Ok(reply)
    }

    #[async_std::test]
    async fn test_ping_peers_with_no_peers_should_not_do_any_api_calls() {
        let mock = MockPingExternalAPI::new();

        let pinger = Ping::new(
            simple_ping_config(),
            Box::new(mock)
        );
        pinger.ping_peers(vec![], &send_ping).await;
    }

    #[async_std::test]
    async fn test_ping_peers_with_happy_path_should_trigger_the_desired_external_api_calls() {
        let peer = PeerId::random();

        let mut mock = MockPingExternalAPI::new();
        mock.expect_on_finished_ping()
            .with(predicate::eq(peer), predicate::function(|x: &crate::types::Result| x.is_ok()))
            .return_const(());

        let pinger = Ping::new(
            simple_ping_config(),
            Box::new(mock) );
        pinger.ping_peers(vec![peer.clone()], &send_ping).await;
    }

    #[async_std::test]
    async fn test_ping_should_invoke_a_failed_ping_reply_for_an_incorrect_reply() {
        let bad_peer = PeerId::from_str("1Ank4EeHLAd3bwwtJma1WsXYSmiGgqmjkQoCUpevx67ix8").ok().unwrap();

        let mut mock = MockPingExternalAPI::new();
        mock.expect_on_finished_ping()
            .with(predicate::eq(bad_peer), predicate::function(|x: &crate::types::Result| x.is_err()))
            .return_const(());

        let pinger = Ping::new(simple_ping_config(), Box::new(mock) );
        pinger.ping_peers(vec![bad_peer.clone()], &send_ping).await;
    }

    #[async_std::test]
    async fn test_ping_peer_invokes_send_message_but_the_time_runs_out() {
        let peer = PeerId::random();
        let mut ping_config = simple_ping_config();
        ping_config.timeout = Duration::from_millis(0);

        let mut mock = MockPingExternalAPI::new();
        mock.expect_on_finished_ping()
            .with(predicate::eq(peer), predicate::function(|x: &crate::types::Result| x.is_err()))
            .return_const(());

        let pinger = Ping::new(ping_config, Box::new(mock) );
        pinger.ping_peers(vec![peer.clone()], &send_ping).await;
    }

    #[async_std::test]
    async fn test_ping_peers_empty_list_will_not_trigger_any_pinging() {
        let mock = MockPingExternalAPI::new();
        let pinger = Ping::new(simple_ping_config(), Box::new(mock) );

        pinger.ping_peers(vec![], &send_ping).await;
    }

    #[async_std::test]
    async fn test_ping_peers_multiple_peers_are_pinged_in_parallel() {
        let peers = vec![
            PeerId::from_str(PEER_DELAYED_1_MS).ok().unwrap(),
            PeerId::from_str(PEER_DELAYED_2_MS).ok().unwrap()
        ];

        let mut mock = MockPingExternalAPI::new();
        mock.expect_on_finished_ping()
            .with(predicate::eq(peers[0].clone()), predicate::function(|x: &crate::types::Result| x.is_ok()))
            .return_const(());
        mock.expect_on_finished_ping()
            .with(predicate::eq(peers[1].clone()), predicate::function(|x: &crate::types::Result| x.is_ok()))
            .return_const(());

        let pinger = Ping::new(simple_ping_config(), Box::new(mock) );
        pinger.ping_peers(peers, &send_ping).await;
    }

    #[async_std::test]
    async fn test_ping_peers_should_ping_parallel_only_a_limited_number_of_peers() {
        let mut config = simple_ping_config();
        config.max_parallel_pings = 1;

        let ping_delay_first = 10u64;
        let ping_delay_second = 10u64;

        let peers = vec![
            PeerId::from_str(PEER_DELAYED_10_MS).ok().unwrap(),
            PeerId::from_str(PEER_DELAYED_11_MS).ok().unwrap()
        ];

        let mut mock = MockPingExternalAPI::new();
        mock.expect_on_finished_ping()
            .with(predicate::eq(peers[0].clone()), predicate::function(|x: &crate::types::Result| x.is_ok()))
            .return_const(());
        mock.expect_on_finished_ping()
            .with(predicate::eq(peers[1].clone()), predicate::function(|x: &crate::types::Result| x.is_ok()))
            .return_const(());

        let pinger = Ping::new(config, Box::new(mock) );

        let start = current_timestamp();
        pinger.ping_peers(peers, &send_ping).await;
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
            PeerId::from_str(PEER_DELAYED_11_MS).ok().unwrap()
        ];

        let mut mock = MockPingExternalAPI::new();
        mock.expect_on_finished_ping()
            .with(predicate::eq(peers[0].clone()), predicate::function(|x: &crate::types::Result| x.is_ok()))
            .return_const(());

        let pinger = Ping::new(config, Box::new(mock) );

        pinger.ping_peers(peers, &send_ping).await;
    }
}