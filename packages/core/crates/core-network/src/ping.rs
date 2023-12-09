use std::pin::Pin;

use async_trait::async_trait;
use futures::{future::poll_fn, StreamExt};
use libp2p_identity::PeerId;

use utils_log::{debug, error, info};

#[cfg(any(not(feature = "wasm"), test))]
use utils_misc::time::native::current_timestamp;

use crate::messaging::ControlMessage;

#[cfg(all(feature = "wasm", not(test)))]
use utils_misc::time::wasm::current_timestamp;

#[cfg(all(feature = "prometheus", not(test)))]
use utils_metrics::metrics::{SimpleCounter, SimpleHistogram};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_TIME_TO_PING: SimpleHistogram =
        SimpleHistogram::new(
            "core_histogram_ping_time_seconds",
            "Measures total time it takes to ping a single node (seconds)",
            vec![0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0, 60.0, 90.0, 120.0, 300.0],
        ).unwrap();
    static ref METRIC_SUCCESSFUL_PING_COUNT: SimpleCounter = SimpleCounter::new(
            "core_counter_heartbeat_successful_pings",
            "Total number of successful pings",
        ).unwrap();
    static ref METRIC_FAILED_PINT_COUNT: SimpleCounter = SimpleCounter::new(
            "core_counter_heartbeat_failed_pings",
            "Total number of failed pings",
        ).unwrap();
}

const MAX_PARALLEL_PINGS: usize = 14;

// TODO: NOTE: UnboundedSender and UnboundedReceiver are bound only by available memory
// in case of faster input than output the memory might run out
pub type HeartbeatSendPingTx = futures::channel::mpsc::UnboundedSender<(PeerId, ControlMessage)>;
pub type HeartbeatGetPongRx =
    futures::channel::mpsc::UnboundedReceiver<(PeerId, std::result::Result<(ControlMessage, String), ()>)>;

#[cfg_attr(test, mockall::automock)]
#[async_trait(? Send)]
pub trait PingExternalAPI {
    async fn on_finished_ping(&self, peer: &PeerId, result: crate::types::Result, version: String);
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PingConfig {
    pub max_parallel_pings: usize,
    pub timeout: u64, // `Duration` -> should be in millis,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl PingConfig {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(max_parallel_pings: usize, timeout: u64) -> Self {
        Self {
            max_parallel_pings,
            timeout,
        }
    }
}

#[async_trait(? Send)] // not placing the `Send` trait limitations on the trait
pub trait Pinging {
    async fn ping(&mut self, peers: Vec<PeerId>);
}

#[derive(Debug)]
pub struct Ping<T: PingExternalAPI> {
    config: PingConfig,
    send_ping: HeartbeatSendPingTx,
    receive_pong: HeartbeatGetPongRx,
    external_api: T,
}

type PingStartedRecord = (u64, ControlMessage);

impl<T: PingExternalAPI> Ping<T> {
    pub fn new(
        config: PingConfig,
        send_ping: HeartbeatSendPingTx,
        receive_pong: HeartbeatGetPongRx,
        external_api: T,
    ) -> Ping<T> {
        let config = PingConfig {
            max_parallel_pings: config.max_parallel_pings.min(MAX_PARALLEL_PINGS),
            ..config
        };

        Ping {
            config,
            send_ping,
            receive_pong,
            external_api,
        }
    }

    fn initiate_peer_ping(&mut self, peer: &PeerId) -> Result<(u64, ControlMessage), ()> {
        info!("Pinging peer '{}'", peer);

        let ping_challenge: ControlMessage = ControlMessage::generate_ping_request();

        self.send_ping
            .start_send((peer.clone(), ping_challenge.clone()))
            .map(move |_| (current_timestamp(), ping_challenge))
            .map_err(|_| ())
    }
}

#[async_trait(? Send)]
impl<T: PingExternalAPI> Pinging for Ping<T> {
    /// Performs multiple concurrent async pings to the specified peers.
    ///
    /// A sliding window mechanism is used to select at most a fixed number of concurrently processed
    /// peers in order to stabilize the pinging mechanism. Pings that do not fit into that window must
    /// wait until they can be further processed.
    ///
    /// # Arguments
    ///
    /// * `peers` - A vector of PeerId objects referencing the peers to be pinged
    async fn ping(&mut self, peers: Vec<PeerId>) {
        let start_all_peers = current_timestamp();
        let mut peers = peers;

        if peers.is_empty() {
            debug!("Received an empty peer list, not pinging any peers");
            return ();
        }

        if let Err(e) = poll_fn(|cx| Pin::new(&self.send_ping).poll_ready(cx)).await {
            error!("The ping receiver is not listening: {}", e);
            return ();
        }

        let mut active_pings: std::collections::HashMap<PeerId, PingStartedRecord> = std::collections::HashMap::new();

        let remainder = peers.split_off(self.config.max_parallel_pings.min(peers.len()));
        for peer in peers.into_iter() {
            if !active_pings.contains_key(&peer) {
                match self.initiate_peer_ping(&peer) {
                    Ok(v) => {
                        active_pings.insert(peer.clone(), v);
                    }
                    Err(_) => {}
                };
            }
        }

        let mut waiting = std::collections::VecDeque::from(remainder);

        while let Some((peer, response)) = self.receive_pong.next().await {
            let record = active_pings.remove(&peer);

            if record.is_none() {
                error!("Received a pong for an unregistered ping, likely an aborted run");
                continue;
            }

            let (peer, result, version) = match response {
                Ok((pong, version)) => {
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

                    (peer, duration, version)
                }
                Err(_) => {
                    error!("Ping to peer {} timed out", peer);
                    (peer, Err(()), "unknown".to_owned())
                }
            };

            #[cfg(all(feature = "prometheus", not(test)))]
            match result {
                Ok(duration) => {
                    METRIC_TIME_TO_PING.observe(duration.as_millis() as f64);
                    METRIC_SUCCESSFUL_PING_COUNT.increment();
                }
                Err(_) => {
                    METRIC_FAILED_PINT_COUNT.increment();
                }
            }

            self.external_api
                .on_finished_ping(&peer, result.map(|v| v.as_millis() as u64), version)
                .await;

            let remaining_time = current_timestamp() - start_all_peers;
            if (remaining_time as u128) < self.config.timeout as u128 {
                while let Some(peer) = waiting.pop_front() {
                    if !active_pings.contains_key(&peer) {
                        match self.initiate_peer_ping(&peer) {
                            Ok(v) => {
                                active_pings.insert(peer.clone(), v);
                            }
                            Err(_) => continue,
                        };
                    }
                }
            }

            if active_pings.len() == 0 && waiting.len() == 0 {
                break;
            }
        }
    }
}

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
            timeout: 150, //Duration::from_millis(150),
        }
    }

    #[async_std::test]
    async fn test_ping_peers_with_no_peers_should_not_do_any_api_calls() {
        let mock = MockPingExternalAPI::new();

        let (tx, _rx_ping) = futures::channel::mpsc::unbounded::<(PeerId, ControlMessage)>();
        let (_tx_pong, rx) =
            futures::channel::mpsc::unbounded::<(PeerId, std::result::Result<(ControlMessage, String), ()>)>();

        let mut pinger = Ping::new(simple_ping_config(), tx, rx, mock);
        pinger.ping(vec![]).await;
    }

    #[async_std::test]
    async fn test_ping_peers_with_happy_path_should_trigger_the_desired_external_api_calls() {
        let (tx, mut rx_ping) = futures::channel::mpsc::unbounded::<(PeerId, ControlMessage)>();
        let (mut tx_pong, rx) =
            futures::channel::mpsc::unbounded::<(PeerId, std::result::Result<(ControlMessage, String), ()>)>();

        let peer = PeerId::random();

        let mut mock = MockPingExternalAPI::new();
        mock.expect_on_finished_ping()
            .with(
                predicate::eq(peer),
                predicate::function(|x: &crate::types::Result| x.is_ok()),
                predicate::eq("version".to_owned()),
            )
            .return_const(());

        let ideal_single_use_channel = async move {
            if let Some((peer, challenge)) = rx_ping.next().await {
                let _ = tx_pong.start_send((
                    peer,
                    Ok((
                        ControlMessage::generate_pong_response(&challenge).expect("valid challenge"),
                        "version".to_owned(),
                    )),
                ));
            };
        };

        let mut pinger = Ping::new(simple_ping_config(), tx, rx, mock);
        futures::join!(pinger.ping(vec![peer.clone()]), ideal_single_use_channel);
    }

    #[async_std::test]
    async fn test_ping_should_invoke_a_failed_ping_reply_for_an_incorrect_reply() {
        let (tx, mut rx_ping) = futures::channel::mpsc::unbounded::<(PeerId, ControlMessage)>();
        let (mut tx_pong, rx) =
            futures::channel::mpsc::unbounded::<(PeerId, std::result::Result<(ControlMessage, String), ()>)>();

        let peer = PeerId::random();

        let mut mock = MockPingExternalAPI::new();
        mock.expect_on_finished_ping()
            .with(
                predicate::eq(peer.clone()),
                predicate::function(|x: &crate::types::Result| x.is_err()),
                predicate::eq("version".to_owned()),
            )
            .return_const(());

        let bad_pong_single_use_channel = async move {
            if let Some((peer, challenge)) = rx_ping.next().await {
                let _ = tx_pong.start_send((peer, Ok((challenge, "version".to_owned()))));
            };
        };

        let mut pinger = Ping::new(simple_ping_config(), tx, rx, mock);
        futures::join!(pinger.ping(vec![peer.clone()]), bad_pong_single_use_channel);
    }

    #[async_std::test]
    async fn test_ping_peer_times_out_on_the_pong() {
        let (tx, mut rx_ping) = futures::channel::mpsc::unbounded::<(PeerId, ControlMessage)>();
        let (mut tx_pong, rx) =
            futures::channel::mpsc::unbounded::<(PeerId, std::result::Result<(ControlMessage, String), ()>)>();

        let peer = PeerId::random();
        let mut ping_config = simple_ping_config();
        ping_config.timeout = 0;

        let mut mock = MockPingExternalAPI::new();
        mock.expect_on_finished_ping()
            .with(
                predicate::eq(peer),
                predicate::function(|x: &crate::types::Result| x.is_err()),
                predicate::eq("unknown".to_owned()),
            )
            .return_const(());

        // NOTE: timeout is ensured by the libp2p protocol handling, only error arrives
        // from the channel
        let timeout_single_use_channel = async move {
            if let Some((peer, _challenge)) = rx_ping.next().await {
                let _ = tx_pong.start_send((peer, Err(())));
            };
        };

        let mut pinger = Ping::new(simple_ping_config(), tx, rx, mock);
        futures::join!(pinger.ping(vec![peer.clone()]), timeout_single_use_channel);
    }

    #[async_std::test]
    async fn test_ping_peers_multiple_peers_are_pinged_in_parallel() {
        let (tx, mut rx_ping) = futures::channel::mpsc::unbounded::<(PeerId, ControlMessage)>();
        let (mut tx_pong, rx) =
            futures::channel::mpsc::unbounded::<(PeerId, std::result::Result<(ControlMessage, String), ()>)>();

        let peers = vec![PeerId::random(), PeerId::random()];

        let mut mock = MockPingExternalAPI::new();
        mock.expect_on_finished_ping()
            .with(
                predicate::eq(peers[0].clone()),
                predicate::function(|x: &crate::types::Result| x.is_ok()),
                predicate::eq("version".to_owned()),
            )
            .return_const(());
        mock.expect_on_finished_ping()
            .with(
                predicate::eq(peers[1].clone()),
                predicate::function(|x: &crate::types::Result| x.is_ok()),
                predicate::eq("version".to_owned()),
            )
            .return_const(());

        let ideal_twice_usable_channel = async move {
            for _ in 0..2 {
                if let Some((peer, challenge)) = rx_ping.next().await {
                    let _ = tx_pong.start_send((
                        peer,
                        Ok((
                            ControlMessage::generate_pong_response(&challenge).expect("valid challenge"),
                            "version".to_owned(),
                        )),
                    ));
                };
            }
        };

        let mut pinger = Ping::new(simple_ping_config(), tx, rx, mock);
        futures::join!(pinger.ping(peers), ideal_twice_usable_channel);
    }

    #[async_std::test]
    async fn test_ping_peers_should_ping_parallel_only_a_limited_number_of_peers() {
        let (tx, mut rx_ping) = futures::channel::mpsc::unbounded::<(PeerId, ControlMessage)>();
        let (mut tx_pong, rx) =
            futures::channel::mpsc::unbounded::<(PeerId, std::result::Result<(ControlMessage, String), ()>)>();

        let mut config = simple_ping_config();
        config.max_parallel_pings = 1;

        let ping_delay = 10u64;

        let peers = vec![PeerId::random(), PeerId::random()];

        let mut mock = MockPingExternalAPI::new();
        mock.expect_on_finished_ping()
            .with(
                predicate::eq(peers[0].clone()),
                predicate::function(|x: &crate::types::Result| x.is_ok()),
                predicate::eq("version".to_owned()),
            )
            .return_const(());
        mock.expect_on_finished_ping()
            .with(
                predicate::eq(peers[1].clone()),
                predicate::function(|x: &crate::types::Result| x.is_ok()),
                predicate::eq("version".to_owned()),
            )
            .return_const(());

        let ideal_twice_usable_linearly_delaying_channel = async move {
            for i in 0..2 {
                if let Some((peer, challenge)) = rx_ping.next().await {
                    async_std::task::sleep(std::time::Duration::from_millis(ping_delay * i)).await;
                    let _ = tx_pong.start_send((
                        peer,
                        Ok((
                            ControlMessage::generate_pong_response(&challenge).expect("valid challenge"),
                            "version".to_owned(),
                        )),
                    ));
                };
            }
        };

        let mut pinger = Ping::new(simple_ping_config(), tx, rx, mock);

        let start = current_timestamp();
        futures::join!(pinger.ping(peers), ideal_twice_usable_linearly_delaying_channel);
        let end = current_timestamp();

        assert_ge!(end - start, ping_delay);
    }
}
