use std::ops::Div;

use async_stream::stream;
use async_trait::async_trait;
use futures::{Stream, StreamExt, channel::mpsc::UnboundedSender};
use hopr_async_runtime::prelude::timeout_fut;
#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::{MultiCounter, SimpleHistogram};
use hopr_platform::time::native::current_time;
use hopr_primitive_types::{prelude::AsUnixTimestamp, traits::SaturatingSub};
use libp2p_identity::PeerId;
use tracing::{debug, warn};

use crate::{
    errors::{NetworkingError, Result},
    messaging::ControlMessage,
};

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

/// Trait for the ping operation itself.
pub trait Pinging {
    fn ping(&self, peers: Vec<PeerId>) -> impl Stream<Item = Result<std::time::Duration>>;
}

/// External behavior that will be triggered once a ping operation result is available
/// per each pinged peer.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PingExternalAPI {
    async fn on_finished_ping(&self, peer: &PeerId, result: &Result<std::time::Duration>, version: String);
}

/// Heartbeat send ping TX type
///
/// NOTE: UnboundedSender and UnboundedReceiver are bound only by available memory
/// in case of faster input than output the memory might run out.
///
/// The unboundedness relies on the fact that a back pressure mechanism exists on a
/// higher level of the business logic making sure that only a fixed maximum count
/// of pings ever enter the queues at any given time.
pub type HeartbeatSendPingTx = UnboundedSender<(PeerId, PingQueryReplier)>;

/// Configuration for the [`Pinger`] mechanism
#[derive(Debug, Clone, PartialEq, Eq, smart_default::SmartDefault)]
pub struct PingConfig {
    /// The maximum total allowed concurrent heartbeat ping count
    #[default = 25]
    pub max_parallel_pings: usize,
    /// The timeout duration for an indiviual ping
    #[default(std::time::Duration::from_secs(30))]
    pub timeout: std::time::Duration, // `Duration` -> should be in millis,
}

/// Ping query result type holding data about the ping duration and the string
/// containg an optional version information of the pinged peer, if provided.
pub type PingQueryResult = Result<(std::time::Duration, String)>;

/// Helper object allowing to send a ping query as a wrapped channel combination
/// that can be filled up on the transport part and awaited locally by the `Pinger`.
#[derive(Debug, Clone)]
pub struct PingQueryReplier {
    notifier: UnboundedSender<PingQueryResult>,
    challenge: Box<(u64, ControlMessage)>,
}

impl PingQueryReplier {
    pub fn new(notifier: UnboundedSender<PingQueryResult>) -> Self {
        Self {
            notifier,
            challenge: Box::new((
                current_time().as_unix_timestamp().as_millis() as u64,
                ControlMessage::generate_ping_request(),
            )),
        }
    }

    /// Return a copy of the challenge for which the reply is expected
    pub fn challenge(&self) -> ControlMessage {
        self.challenge.1.clone()
    }

    /// Mechanism to finalize the ping operation by providing a [`ControlMessage`] received by the
    /// transport layer.
    ///
    /// The resulting timing information about the RTT is halved to provide a unidirectional latency.
    pub fn notify(self, pong: ControlMessage, version: String) {
        let timed_result = if ControlMessage::validate_pong_response(&self.challenge.1, &pong).is_ok() {
            let unidirectional_latency = current_time()
                .as_unix_timestamp()
                .saturating_sub(std::time::Duration::from_millis(self.challenge.0))
                .div(2u32);
            Ok((unidirectional_latency, version))
        } else {
            Err(NetworkingError::DecodingError)
        };

        if self.notifier.unbounded_send(timed_result).is_err() {
            warn!("Failed to notify the ping query result due to upper layer ping timeout");
        }
    }
}

/// Timeout-based future that will resolve to the result of the ping operation.
#[tracing::instrument(level = "trace", skip(sender, timeout))]
pub fn to_active_ping(
    peer: PeerId,
    sender: HeartbeatSendPingTx,
    timeout: std::time::Duration,
) -> impl std::future::Future<Output = (PeerId, Result<std::time::Duration>, String)> {
    let (tx, mut rx) = futures::channel::mpsc::unbounded::<PingQueryResult>();
    let replier = PingQueryReplier::new(tx);

    if let Err(e) = sender.unbounded_send((peer, replier)) {
        warn!(%peer, error = %e, "Failed to initiate a ping request");
    }

    async move {
        match timeout_fut(timeout, rx.next()).await {
            Ok(Some(Ok((latency, version)))) => {
                debug!(latency = latency.as_millis(), %peer, %version, "Ping succeeded",);
                (peer, Ok(latency), version)
            }
            Ok(Some(Err(e))) => {
                let error = if let NetworkingError::DecodingError = e {
                    NetworkingError::PingerError(peer, "incorrect pong response".into())
                } else {
                    e
                };

                debug!(%peer, %error, "Ping failed internally",);
                (peer, Err(error), "unknown".into())
            }
            Ok(None) => {
                debug!(%peer, "Ping canceled");
                (
                    peer,
                    Err(NetworkingError::PingerError(peer, "canceled".into())),
                    "unknown".into(),
                )
            }
            Err(_) => {
                debug!(%peer, "Ping failed due to timeout");
                (peer, Err(NetworkingError::Timeout(timeout.as_secs())), "unknown".into())
            }
        }
    }
}

/// Implementation of the ping mechanism
#[derive(Debug, Clone)]
pub struct Pinger<T>
where
    T: PingExternalAPI + Send + Sync,
{
    config: PingConfig,
    send_ping: HeartbeatSendPingTx,
    recorder: T,
}

impl<T> Pinger<T>
where
    T: PingExternalAPI + Send + Sync,
{
    pub fn new(config: PingConfig, send_ping: HeartbeatSendPingTx, recorder: T) -> Self {
        let config = PingConfig {
            max_parallel_pings: config.max_parallel_pings,
            ..config
        };

        Pinger {
            config,
            send_ping,
            recorder,
        }
    }

    pub fn config(&self) -> &PingConfig {
        &self.config
    }
}

impl<T> Pinging for Pinger<T>
where
    T: PingExternalAPI + Send + Sync,
{
    /// Performs multiple concurrent async pings to the specified peers.
    ///
    /// A sliding window mechanism is used to select at most a fixed number of concurrently processed
    /// peers in order to stabilize the pinging mechanism. Pings that do not fit into that window must
    /// wait until they can be further processed.
    ///
    /// # Arguments
    ///
    /// * `peers` - A vector of PeerId objects referencing the peers to be pinged
    #[tracing::instrument(level = "info", skip(self, peers), fields(peers.count = peers.len()))]
    fn ping(&self, mut peers: Vec<PeerId>) -> impl Stream<Item = Result<std::time::Duration>> {
        let start_all_peers = current_time();

        stream! {
            if !peers.is_empty() {
                let remainder = peers.split_off(self.config.max_parallel_pings.min(peers.len()));
                let mut active_pings = peers
                    .into_iter()
                    .map(|peer| to_active_ping(peer, self.send_ping.clone(), self.config.timeout))
                    .collect::<futures::stream::FuturesUnordered<_>>();

                let mut waiting = std::collections::VecDeque::from(remainder);

                while let Some((peer, result, version)) = active_pings.next().await {
                    self.recorder.on_finished_ping(&peer, &result, version).await;

                    #[cfg(all(feature = "prometheus", not(test)))]
                    match &result {
                        Ok(duration) => {
                            METRIC_TIME_TO_PING.observe((duration.as_millis() as f64) / 1000.0); // precision for seconds
                            METRIC_PING_COUNT.increment(&["true"]);
                        }
                        Err(_) => {
                            METRIC_PING_COUNT.increment(&["false"]);
                        }
                    }

                    if current_time().saturating_sub(start_all_peers) < self.config.timeout {
                        if let Some(peer) = waiting.pop_front() {
                            active_pings.push(to_active_ping(peer, self.send_ping.clone(), self.config.timeout));
                        }
                    }

                    yield result;

                    if active_pings.is_empty() && waiting.is_empty() {
                        break;
                    }
                }
            } else {
                debug!("Received an empty peer list, not pinging any peers");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use futures::TryStreamExt;
    use hopr_primitive_types::traits::SaturatingSub;
    use mockall::*;
    use more_asserts::*;

    use super::*;
    use crate::{messaging::ControlMessage, ping::Pinger};

    fn simple_ping_config() -> PingConfig {
        PingConfig {
            max_parallel_pings: 2,
            timeout: std::time::Duration::from_millis(150),
        }
    }

    #[tokio::test]
    async fn ping_query_replier_should_return_ok_result_when_the_pong_is_correct_for_the_challenge()
    -> anyhow::Result<()> {
        let (tx, mut rx) = futures::channel::mpsc::unbounded::<PingQueryResult>();

        let replier = PingQueryReplier::new(tx);
        let challenge = replier.challenge.clone();

        replier.notify(
            ControlMessage::generate_pong_response(&challenge.1)?,
            "version".to_owned(),
        );

        assert!(rx.next().await.is_some_and(|r| r.is_ok()));

        Ok(())
    }

    #[tokio::test]
    async fn ping_query_replier_should_return_err_result_when_the_pong_is_incorrect_for_the_challenge()
    -> anyhow::Result<()> {
        let (tx, mut rx) = futures::channel::mpsc::unbounded::<PingQueryResult>();

        let replier = PingQueryReplier::new(tx);

        replier.notify(
            ControlMessage::generate_pong_response(&ControlMessage::generate_ping_request())?,
            "version".to_owned(),
        );

        assert!(rx.next().await.is_some_and(|r| r.is_err()));

        Ok(())
    }

    #[tokio::test]
    async fn ping_query_replier_should_return_the_unidirectional_latency() -> anyhow::Result<()> {
        let (tx, mut rx) = futures::channel::mpsc::unbounded::<PingQueryResult>();

        let replier = PingQueryReplier::new(tx);
        let challenge = replier.challenge.clone();

        let delay = std::time::Duration::from_millis(10);

        tokio::time::sleep(delay).await;
        replier.notify(
            ControlMessage::generate_pong_response(&challenge.1)?,
            "version".to_owned(),
        );

        let actual_latency = rx
            .next()
            .await
            .ok_or_else(|| anyhow::anyhow!("should contain a result value"))?
            .map_err(|_e| anyhow::anyhow!("should contain a result value"))?
            .0;
        assert!(actual_latency > delay / 2);
        assert!(actual_latency < delay);

        Ok(())
    }

    #[tokio::test]
    async fn ping_empty_vector_of_peers_should_not_do_any_api_calls() -> anyhow::Result<()> {
        let (tx, mut rx) = futures::channel::mpsc::unbounded::<(PeerId, PingQueryReplier)>();

        let ideal_channel = tokio::task::spawn(async move {
            while let Some((_peer, replier)) = rx.next().await {
                let challenge = replier.challenge.1.clone();

                replier.notify(
                    ControlMessage::generate_pong_response(&challenge).expect("valid challenge reply"),
                    "version".to_owned(),
                );
            }
        });

        let mut mock = MockPingExternalAPI::new();
        mock.expect_on_finished_ping().times(0);

        let pinger = Pinger::new(simple_ping_config(), tx, mock);

        assert!(pinger.ping(vec![]).try_collect::<Vec<_>>().await?.is_empty());

        ideal_channel.abort();

        Ok(())
    }

    #[tokio::test]
    async fn test_ping_peers_with_happy_path_should_trigger_the_desired_external_api_calls() -> anyhow::Result<()> {
        let (tx, mut rx) = futures::channel::mpsc::unbounded::<(PeerId, PingQueryReplier)>();

        let ideal_channel = tokio::task::spawn(async move {
            while let Some((_peer, replier)) = rx.next().await {
                let challenge = replier.challenge.1.clone();

                replier.notify(
                    ControlMessage::generate_pong_response(&challenge).expect("valid challenge reply"),
                    "version".to_owned(),
                );
            }
        });

        let peer = PeerId::random();

        let mut mock = MockPingExternalAPI::new();
        mock.expect_on_finished_ping()
            .times(1)
            .with(
                predicate::eq(peer),
                predicate::function(|x: &Result<std::time::Duration>| x.is_ok()),
                predicate::eq("version".to_owned()),
            )
            .return_const(());

        let pinger = Pinger::new(simple_ping_config(), tx, mock);
        pinger.ping(vec![peer]).try_collect::<Vec<_>>().await?;

        ideal_channel.abort();

        Ok(())
    }

    #[tokio::test]
    async fn test_ping_should_invoke_a_failed_ping_reply_for_an_incorrect_reply() -> anyhow::Result<()> {
        let (tx, mut rx) = futures::channel::mpsc::unbounded::<(PeerId, PingQueryReplier)>();

        let failing_channel = tokio::task::spawn(async move {
            while let Some((_peer, replier)) = rx.next().await {
                replier.notify(
                    ControlMessage::generate_pong_response(&ControlMessage::generate_ping_request())
                        .expect("valid challenge reply"),
                    "version".to_owned(),
                );
            }
        });

        let peer = PeerId::random();

        let mut mock = MockPingExternalAPI::new();
        mock.expect_on_finished_ping()
            .times(1)
            .with(
                predicate::eq(peer),
                predicate::function(|x: &Result<std::time::Duration>| x.is_err()),
                predicate::eq("unknown".to_owned()),
            )
            .return_const(());

        let pinger = Pinger::new(simple_ping_config(), tx, mock);
        assert!(pinger.ping(vec![peer]).try_collect::<Vec<_>>().await.is_err());

        failing_channel.abort();

        Ok(())
    }

    #[tokio::test]
    async fn test_ping_peer_returns_error_on_the_pong() -> anyhow::Result<()> {
        let (tx, mut rx) = futures::channel::mpsc::unbounded::<(PeerId, PingQueryReplier)>();

        let delay = std::time::Duration::from_millis(10);
        let delaying_channel = tokio::task::spawn(async move {
            while let Some((_peer, replier)) = rx.next().await {
                let challenge = replier.challenge.1.clone();

                tokio::time::sleep(delay).await;
                replier.notify(
                    ControlMessage::generate_pong_response(&challenge).expect("valid challenge reply"),
                    "version".to_owned(),
                );
            }
        });

        let peer = PeerId::random();
        let ping_config = PingConfig {
            timeout: std::time::Duration::from_millis(0),
            ..simple_ping_config()
        };

        let mut mock = MockPingExternalAPI::new();
        mock.expect_on_finished_ping()
            .times(1)
            .with(
                predicate::eq(peer),
                predicate::function(|x: &Result<std::time::Duration>| x.is_err()),
                predicate::eq("unknown".to_owned()),
            )
            .return_const(());

        let pinger = Pinger::new(ping_config, tx, mock);
        assert!(pinger.ping(vec![peer]).try_collect::<Vec<_>>().await.is_err());

        delaying_channel.abort();

        Ok(())
    }

    #[tokio::test]
    async fn test_ping_peers_multiple_peers_are_pinged_in_parallel() -> anyhow::Result<()> {
        let (tx, mut rx) = futures::channel::mpsc::unbounded::<(PeerId, PingQueryReplier)>();

        let ideal_channel = tokio::task::spawn(async move {
            while let Some((_peer, replier)) = rx.next().await {
                let challenge = replier.challenge.1.clone();

                replier.notify(
                    ControlMessage::generate_pong_response(&challenge).expect("valid challenge reply"),
                    "version".to_owned(),
                );
            }
        });

        let peers = vec![PeerId::random(), PeerId::random()];

        let mut mock = MockPingExternalAPI::new();
        mock.expect_on_finished_ping()
            .times(1)
            .with(
                predicate::eq(peers[0]),
                predicate::function(|x: &Result<std::time::Duration>| x.is_ok()),
                predicate::eq("version".to_owned()),
            )
            .return_const(());
        mock.expect_on_finished_ping()
            .times(1)
            .with(
                predicate::eq(peers[1]),
                predicate::function(|x: &Result<std::time::Duration>| x.is_ok()),
                predicate::eq("version".to_owned()),
            )
            .return_const(());

        let pinger = Pinger::new(simple_ping_config(), tx, mock);
        pinger.ping(peers).try_collect::<Vec<_>>().await?;

        ideal_channel.abort();

        Ok(())
    }

    #[tokio::test]
    async fn test_ping_peers_should_ping_parallel_only_a_limited_number_of_peers() -> anyhow::Result<()> {
        let (tx, mut rx) = futures::channel::mpsc::unbounded::<(PeerId, PingQueryReplier)>();

        let delay = 10u64;

        let ideal_delaying_channel = tokio::task::spawn(async move {
            while let Some((_peer, replier)) = rx.next().await {
                let challenge = replier.challenge.1.clone();

                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                replier.notify(
                    ControlMessage::generate_pong_response(&challenge).expect("valid challenge reply"),
                    "version".to_owned(),
                );
            }
        });

        let peers = vec![PeerId::random(), PeerId::random()];

        let mut mock = MockPingExternalAPI::new();
        mock.expect_on_finished_ping()
            .times(1)
            .with(
                predicate::eq(peers[0]),
                predicate::function(|x: &Result<std::time::Duration>| x.is_ok()),
                predicate::eq("version".to_owned()),
            )
            .return_const(());
        mock.expect_on_finished_ping()
            .times(1)
            .with(
                predicate::eq(peers[1]),
                predicate::function(|x: &Result<std::time::Duration>| x.is_ok()),
                predicate::eq("version".to_owned()),
            )
            .return_const(());

        let pinger = Pinger::new(
            PingConfig {
                max_parallel_pings: 1,
                ..simple_ping_config()
            },
            tx,
            mock,
        );

        let start = current_time();
        pinger.ping(peers).try_collect::<Vec<_>>().await?;
        let end = current_time();

        assert_ge!(end.saturating_sub(start), std::time::Duration::from_millis(delay));

        ideal_delaying_channel.abort();

        Ok(())
    }
}
