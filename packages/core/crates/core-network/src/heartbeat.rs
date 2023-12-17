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

use utils_log::{debug, info};

#[cfg(all(feature = "prometheus", not(test)))]
use utils_metrics::{histogram_start_measure, metrics::SimpleHistogram};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_TIME_TO_HEARTBEAT: SimpleHistogram =
        SimpleHistogram::new(
            "core_histogram_heartbeat_time_seconds",
            "Measures total time it takes to probe all other nodes (in seconds)",
            vec![0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0, 60.0, 90.0, 120.0, 300.0],
        ).unwrap();
}

use async_std::task::sleep;
use platform::time::native::current_timestamp;

use crate::constants::{DEFAULT_HEARTBEAT_INTERVAL, DEFAULT_HEARTBEAT_INTERVAL_VARIANCE, DEFAULT_HEARTBEAT_THRESHOLD};
use crate::ping::Pinging;

/// Configuration of the Heartbeat
#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Validate, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    /// Round-to-round variance to complicate network sync [seconds]
    #[serde_as(as = "DurationSeconds<u64>")]
    pub variance: std::time::Duration,
    /// Interval in which the heartbeat is triggered [seconds]
    #[serde_as(as = "DurationSeconds<u64>")]
    pub interval: std::time::Duration,
    /// The time interval for which to consider peer heartbeat renewal [seconds]
    #[serde_as(as = "DurationSeconds<u64>")]
    pub threshold: std::time::Duration,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            interval: DEFAULT_HEARTBEAT_INTERVAL,
            threshold: DEFAULT_HEARTBEAT_THRESHOLD,
            variance: DEFAULT_HEARTBEAT_INTERVAL_VARIANCE,
        }
    }
}

/// API trait for external functionality required by the heartbeat mechanism
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait HeartbeatExternalApi {
    async fn get_peers(&self, from_timestamp: u64) -> Vec<PeerId>;
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

            let start = current_timestamp();
            let from_timestamp = start.checked_sub(self.config.threshold).unwrap_or(start);

            info!("Starting a heartbeat round for peers since timestamp {from_timestamp:?}");
            let peers = self.external_api.get_peers(from_timestamp.as_millis() as u64).await;

            // random timeout to avoid network sync:
            let this_round_planned_duration = std::time::Duration::from_millis({
                let interval_ms = self.config.interval.as_millis() as u64;
                let variance_ms = self.config.interval.as_millis() as u64;

                core_crypto::random::random_integer(
                    interval_ms,
                    Some(interval_ms.checked_add(variance_ms.max(1u64)).unwrap_or(u64::MAX)),
                )
                .unwrap_or(interval_ms)
            });

            let timeout = sleep(this_round_planned_duration).fuse();
            let ping = self.pinger.ping(peers).fuse();

            pin_mut!(timeout, ping);

            match select(timeout, ping).await {
                Either::Left(_) => info!("Heartbeat round interrupted by timeout"),
                Either::Right(_) => info!("Heartbeat round finished for all peers"),
            };

            let this_round_actual_duration = current_timestamp().saturating_sub(start);
            if this_round_actual_duration < this_round_planned_duration {
                let time_to_wait_for_next_round =
                    this_round_planned_duration.saturating_sub(this_round_actual_duration);

                debug!("Heartbeat sleeping for: {time_to_wait_for_next_round:?}ms");
                sleep(time_to_wait_for_next_round).await
            }

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

    #[async_trait(? Send)]
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

        futures_lite::future::race(heartbeat.heartbeat_loop(), sleep(ping_delay * expected_loop_count)).await;
    }

    #[async_std::test]
    async fn test_heartbeat_should_interrupt_long_running_heartbeats() {
        let config = HeartbeatConfig {
            interval: std::time::Duration::from_millis(5u64),
            ..simple_heartbeat_config()
        };

        let ping_delay = 2 * config.interval;
        let expected_loop_count = 2;

        let mut mock = MockHeartbeatExternalApi::new();
        mock.expect_get_peers()
            .times(expected_loop_count..)
            .return_const(vec![PeerId::random(), PeerId::random()]);

        let mut heartbeat = Heartbeat::new(config, DelayingPinger { delay: ping_delay }, mock);

        futures_lite::future::race(
            heartbeat.heartbeat_loop(),
            sleep(config.interval * (expected_loop_count as u32)),
        )
        .await;
    }
}
