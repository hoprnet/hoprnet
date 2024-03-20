use async_std::task::sleep;
use async_trait::async_trait;
use futures::{
    future::{
        select,
        Either,
        FutureExt, // .fuse()
    },
    pin_mut,
};
use libp2p_identity::PeerId;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};
use validator::Validate;

use tracing::{debug, info};

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::{histogram_start_measure, metrics::SimpleHistogram};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_TIME_TO_HEARTBEAT: SimpleHistogram =
        SimpleHistogram::new(
            "hopr_heartbeat_round_time_sec",
            "Measures total time in seconds it takes to probe all other nodes",
            vec![0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0, 60.0],
        ).unwrap();
}

use hopr_platform::time::native::current_time;

use crate::constants::{DEFAULT_HEARTBEAT_INTERVAL, DEFAULT_HEARTBEAT_INTERVAL_VARIANCE, DEFAULT_HEARTBEAT_THRESHOLD};
use crate::network::Network;
use crate::ping::Pinging;

/// Configuration for the Heartbeat mechanism
#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeartbeatConfig {
    /// Round-to-round variance to complicate network sync in seconds
    #[serde_as(as = "DurationSeconds<u64>")]
    #[serde(default = "default_heartbeat_variance")]
    #[default(default_heartbeat_variance())]
    pub variance: std::time::Duration,
    /// Interval in which the heartbeat is triggered in seconds
    #[serde_as(as = "DurationSeconds<u64>")]
    #[serde(default = "default_heartbeat_interval")]
    #[default(default_heartbeat_interval())]
    pub interval: std::time::Duration,
    /// The time interval for which to consider peer heartbeat renewal in seconds
    #[serde_as(as = "DurationSeconds<u64>")]
    #[serde(default = "default_heartbeat_threshold")]
    #[default(default_heartbeat_threshold())]
    pub threshold: std::time::Duration,
}

#[inline]
fn default_heartbeat_interval() -> std::time::Duration {
    DEFAULT_HEARTBEAT_INTERVAL
}

#[inline]
fn default_heartbeat_threshold() -> std::time::Duration {
    DEFAULT_HEARTBEAT_THRESHOLD
}

#[inline]
fn default_heartbeat_variance() -> std::time::Duration {
    DEFAULT_HEARTBEAT_INTERVAL_VARIANCE
}

use std::sync::Arc;

use tracing::error;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait HeartbeatExternalApi {
    /// Get all peers considered by the `Network` to be pingable.
    ///
    /// After a duration of non-pinging based specified by the configurable threshold.
    async fn get_peers(&self, from_timestamp: std::time::SystemTime) -> Vec<PeerId>;
}

/// Implementor of the heartbeat external API.
///
/// Heartbeat requires functionality from external components in order to obtain
/// the triggers for its functionality. This class implements the basic API by
/// aggregating all necessary heartbeat resources without leaking them into the
/// `Heartbeat` object and keeping both the adaptor and the heartbeat object
/// OCP and SRP compliant.
pub struct HeartbeatExternalInteractions<T>
where
    T: hopr_db_api::peers::HoprDbPeersOperations + Sync + Send + std::fmt::Debug,
{
    network: Arc<Network<T>>,
}

impl<T> HeartbeatExternalInteractions<T>
where
    T: hopr_db_api::peers::HoprDbPeersOperations + Sync + Send + std::fmt::Debug,
{
    pub fn new(network: Arc<Network<T>>) -> Self {
        Self { network }
    }
}

#[async_trait]
impl<T> HeartbeatExternalApi for HeartbeatExternalInteractions<T>
where
    T: hopr_db_api::peers::HoprDbPeersOperations + Sync + Send + std::fmt::Debug,
{
    /// Get all peers considered by the `Network` to be pingable.
    ///
    /// After a duration of non-pinging based specified by the configurable threshold.
    async fn get_peers(&self, from_timestamp: std::time::SystemTime) -> Vec<PeerId> {
        self.network
            .find_peers_to_ping(from_timestamp)
            .await
            .unwrap_or_else(|e| {
                error!("Failed to generate peers for the heartbeat procedure: {e}");
                vec![]
            })
    }
}

/// Heartbeat mechanism providing the regular trigger and processing for the heartbeat protocol.
///
/// This object provides a single public method that can be polled. Once triggered, it will never
/// return and will only terminate with an unresolvable error or a panic.
pub struct Heartbeat<T: Pinging, API: HeartbeatExternalApi> {
    config: HeartbeatConfig,
    pinger: T,
    external_api: API,
}

impl<T: Pinging, API: HeartbeatExternalApi> Heartbeat<T, API> {
    pub fn new(config: HeartbeatConfig, pinger: T, external_api: API) -> Self {
        Self {
            config,
            pinger,
            external_api,
        }
    }

    /// Heartbeat loop responsible for periodically requesting peers to ping around from the
    /// external API interface.
    ///
    /// The loop runs indefinitely, until the program is explicitly terminated.
    ///
    /// This feature should be joined with other internal loops and awaited after all
    /// components have been initialized.
    pub async fn heartbeat_loop(&mut self) {
        loop {
            #[cfg(all(feature = "prometheus", not(test)))]
            let heartbeat_round_timer = histogram_start_measure!(METRIC_TIME_TO_HEARTBEAT);

            let start = current_time();
            let from_timestamp = start.checked_sub(self.config.threshold).unwrap_or(start);

            info!("Starting a heartbeat round for peers since timestamp {from_timestamp:?}");
            let peers = self.external_api.get_peers(from_timestamp).await;

            // random timeout to avoid network sync:
            let this_round_planned_duration = std::time::Duration::from_millis({
                let interval_ms = self.config.interval.as_millis() as u64;
                let variance_ms = self.config.interval.as_millis() as u64;

                hopr_crypto_random::random_integer(
                    interval_ms,
                    Some(interval_ms.checked_add(variance_ms.max(1u64)).unwrap_or(u64::MAX)),
                )
            });

            let timeout = sleep(this_round_planned_duration).fuse();
            let ping = self.pinger.ping(peers).fuse();

            pin_mut!(timeout, ping);

            match select(timeout, ping).await {
                Either::Left(_) => info!("Heartbeat round interrupted by timeout"),
                Either::Right(_) => {
                    info!("Heartbeat round finished for all peers");

                    let this_round_actual_duration = current_time().duration_since(start).unwrap_or_default();

                    let time_to_wait_for_next_round =
                        this_round_planned_duration.saturating_sub(this_round_actual_duration);

                    debug!("Heartbeat sleeping for: {time_to_wait_for_next_round:?}");
                    sleep(time_to_wait_for_next_round).await
                }
            };

            #[cfg(all(feature = "prometheus", not(test)))]
            METRIC_TIME_TO_HEARTBEAT.record_measure(heartbeat_round_timer);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_heartbeat_config() -> HeartbeatConfig {
        HeartbeatConfig {
            variance: std::time::Duration::from_millis(0u64),
            interval: std::time::Duration::from_millis(5u64),
            threshold: std::time::Duration::from_millis(0u64),
        }
    }

    pub struct DelayingPinger {
        pub delay: std::time::Duration,
    }

    #[async_trait]
    impl Pinging for DelayingPinger {
        async fn ping(&mut self, _peers: Vec<PeerId>) {
            sleep(self.delay).await;
        }
    }

    #[async_std::test]
    async fn test_heartbeat_should_loop_multiple_times() {
        let config = simple_heartbeat_config();

        let ping_delay = std::time::Duration::from_millis(5u64);
        let expected_loop_count = 2u32;

        let mut mock = MockHeartbeatExternalApi::new();
        mock.expect_get_peers()
            .times(expected_loop_count as usize..)
            .return_const(vec![PeerId::random(), PeerId::random()]);

        let mut heartbeat = Heartbeat::new(config, DelayingPinger { delay: ping_delay }, mock);

        let tolerance = std::time::Duration::from_millis(2);
        futures_lite::future::race(
            heartbeat.heartbeat_loop(),
            sleep(ping_delay * expected_loop_count + tolerance),
        )
        .await;
    }

    #[async_std::test]
    async fn test_heartbeat_should_interrupt_long_running_heartbeats() {
        let config = HeartbeatConfig {
            interval: std::time::Duration::from_millis(5u64),
            ..simple_heartbeat_config()
        };

        let ping_delay = 2 * config.interval + config.interval / 2;
        let expected_loop_count = 2;

        let mut mock = MockHeartbeatExternalApi::new();
        mock.expect_get_peers()
            .times(expected_loop_count..)
            .return_const(vec![PeerId::random(), PeerId::random()]);

        let mut heartbeat = Heartbeat::new(config, DelayingPinger { delay: ping_delay }, mock);

        let tolerance = std::time::Duration::from_millis(2);
        futures_lite::future::race(
            heartbeat.heartbeat_loop(),
            sleep(config.interval * (expected_loop_count as u32) + tolerance),
        )
        .await;
    }
}
