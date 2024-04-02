use std::pin::Pin;
use std::{collections::hash_map::Entry, ops::Div};

use async_trait::async_trait;
use futures::{future::poll_fn, StreamExt};
use libp2p_identity::PeerId;

use tracing::{debug, error, info};

use hopr_platform::time::native::current_time;

use crate::messaging::ControlMessage;

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::{MultiCounter, SimpleHistogram};
use hopr_primitive_types::prelude::AsUnixTimestamp;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_TIME_TO_PING: SimpleHistogram =
        SimpleHistogram::new(
            "hopr_ping_time_sec",
            "Measures total time it takes to ping a single node (seconds)",
            vec![0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0],
        ).unwrap();
    static ref METRIC_PING_COUNT: MultiCounter = MultiCounter::new(
            "hopr_heartbeat_pings_count",
            "Total number of pings by result",
            &["success"]
        ).unwrap();
}

const MAX_PARALLEL_PINGS: usize = 14;

/// Heartbeat send ping TX type
///
/// NOTE: UnboundedSender and UnboundedReceiver are bound only by available memory
/// in case of faster input than output the memory might run out.
///
/// The unboundedness relies on the fact that a back pressure mechanism exists on a
/// higher level of the business logic making sure that only a fixed maximum count
/// of pings ever enter the queues at any given time.
pub type HeartbeatSendPingTx = futures::channel::mpsc::UnboundedSender<(PeerId, ControlMessage)>;

/// Heartbeat get pong RX type
///
/// NOTE: UnboundedSender and UnboundedReceiver are bound only by available memory
/// in case of faster input than output the memory might run out.
///
/// The unboundedness relies on the fact that a back pressure mechanism exists on a
/// higher level of the business logic making sure that only a fixed maximum count
/// of pings ever enter the queues at any given time.
pub type HeartbeatGetPongRx =
    futures::channel::mpsc::UnboundedReceiver<(PeerId, std::result::Result<(ControlMessage, String), ()>)>;

/// Result of the ping operation.
pub type PingResult = std::result::Result<u64, ()>;

/// External behavior that will be triggered once a [PingResult] is available.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PingExternalAPI {
    async fn on_finished_ping(&self, peer: &PeerId, result: PingResult, version: String);
}

/// Configuration for the [Ping] mechanism
#[derive(Debug, Clone, PartialEq, Eq, smart_default::SmartDefault)]
pub struct PingConfig {
    /// The maximum total allowed concurrent heartbeat ping count
    #[default = 14]
    pub max_parallel_pings: usize,
    /// The timeout duration for an indiviual ping
    #[default(std::time::Duration::from_secs(30))]
    pub timeout: std::time::Duration, // `Duration` -> should be in millis,
}

/// Trait for the ping operation itself.
#[async_trait]
pub trait Pinging {
    async fn ping(&mut self, peers: Vec<PeerId>);
}

/// Implementation of the ping mechanism
#[derive(Debug)]
pub struct Ping<T: PingExternalAPI + std::marker::Send> {
    config: PingConfig,
    send_ping: HeartbeatSendPingTx,
    receive_pong: HeartbeatGetPongRx,
    external_api: T,
}

type PingStartedRecord = (u64, ControlMessage);

impl<T: PingExternalAPI + std::marker::Send> Ping<T> {
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
        debug!("Pinging peer '{}'", peer);

        let ping_challenge: ControlMessage = ControlMessage::generate_ping_request();

        self.send_ping
            .start_send((*peer, ping_challenge.clone()))
            .map(move |_| (current_time().as_unix_timestamp().as_millis() as u64, ping_challenge))
            .map_err(|_| ())
    }
}

#[async_trait]
impl<T: PingExternalAPI + std::marker::Send> Pinging for Ping<T> {
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
        let start_all_peers = current_time();
        let mut peers = peers;

        if peers.is_empty() {
            debug!("Received an empty peer list, not pinging any peers");
            return;
        }

        if let Err(e) = poll_fn(|cx| Pin::new(&self.send_ping).poll_ready(cx)).await {
            error!("The ping receiver is not listening: {}", e);
            return;
        }

        let mut active_pings: std::collections::HashMap<PeerId, PingStartedRecord> = std::collections::HashMap::new();

        let remainder = peers.split_off(self.config.max_parallel_pings.min(peers.len()));
        for peer in peers.into_iter() {
            if let Entry::Vacant(e) = active_pings.entry(peer) {
                if let Ok(v) = self.initiate_peer_ping(&peer) {
                    e.insert(v);
                }
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
                            Ok(current_time()
                                .as_unix_timestamp()
                                .saturating_sub(std::time::Duration::from_millis(start))
                                .div(2u32))
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
                    METRIC_TIME_TO_PING.observe((duration.as_millis() as f64) / 1000.0); // precision for seconds
                    METRIC_PING_COUNT.increment(&["true"]);
                }
                Err(_) => {
                    METRIC_PING_COUNT.increment(&["false"]);
                }
            }

            self.external_api
                .on_finished_ping(&peer, result.map(|v| v.as_millis() as u64), version)
                .await;

            if current_time().duration_since(start_all_peers).unwrap_or_default() < self.config.timeout {
                while let Some(peer) = waiting.pop_front() {
                    if let Entry::Vacant(e) = active_pings.entry(peer) {
                        match self.initiate_peer_ping(&peer) {
                            Ok(v) => {
                                e.insert(v);
                            }
                            Err(_) => continue,
                        };
                    }
                }
            }

            if active_pings.is_empty() && waiting.is_empty() {
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
            timeout: std::time::Duration::from_millis(150),
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
                predicate::function(|x: &PingResult| x.is_ok()),
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
        futures::join!(pinger.ping(vec![peer]), ideal_single_use_channel);
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
                predicate::eq(peer),
                predicate::function(|x: &PingResult| x.is_err()),
                predicate::eq("version".to_owned()),
            )
            .return_const(());

        let bad_pong_single_use_channel = async move {
            if let Some((peer, challenge)) = rx_ping.next().await {
                let _ = tx_pong.start_send((peer, Ok((challenge, "version".to_owned()))));
            };
        };

        let mut pinger = Ping::new(simple_ping_config(), tx, rx, mock);
        futures::join!(pinger.ping(vec![peer]), bad_pong_single_use_channel);
    }

    #[async_std::test]
    async fn test_ping_peer_times_out_on_the_pong() {
        let (tx, mut rx_ping) = futures::channel::mpsc::unbounded::<(PeerId, ControlMessage)>();
        let (mut tx_pong, rx) =
            futures::channel::mpsc::unbounded::<(PeerId, std::result::Result<(ControlMessage, String), ()>)>();

        let peer = PeerId::random();
        let ping_config = PingConfig {
            timeout: std::time::Duration::from_millis(0),
            ..simple_ping_config()
        };

        let mut mock = MockPingExternalAPI::new();
        mock.expect_on_finished_ping()
            .with(
                predicate::eq(peer),
                predicate::function(|x: &PingResult| x.is_err()),
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

        let mut pinger = Ping::new(ping_config, tx, rx, mock);
        futures::join!(pinger.ping(vec![peer]), timeout_single_use_channel);
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
                predicate::eq(peers[0]),
                predicate::function(|x: &PingResult| x.is_ok()),
                predicate::eq("version".to_owned()),
            )
            .return_const(());
        mock.expect_on_finished_ping()
            .with(
                predicate::eq(peers[1]),
                predicate::function(|x: &PingResult| x.is_ok()),
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
                predicate::eq(peers[0]),
                predicate::function(|x: &PingResult| x.is_ok()),
                predicate::eq("version".to_owned()),
            )
            .return_const(());
        mock.expect_on_finished_ping()
            .with(
                predicate::eq(peers[1]),
                predicate::function(|x: &PingResult| x.is_ok()),
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

        let start = current_time();
        futures::join!(pinger.ping(peers), ideal_twice_usable_linearly_delaying_channel);
        let end = current_time();

        assert_ge!(
            end.duration_since(start).unwrap_or_default(),
            std::time::Duration::from_millis(ping_delay)
        );
    }
}
