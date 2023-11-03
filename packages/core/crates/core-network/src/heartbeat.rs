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
use rand::Rng;

use serde::{Deserialize, Serialize};
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

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::sleep;
#[cfg(any(not(feature = "wasm"), test))]
use utils_misc::time::native::current_timestamp;

#[cfg(all(feature = "wasm", not(test)))]
use gloo_timers::future::sleep;
#[cfg(all(feature = "wasm", not(test)))]
use utils_misc::time::wasm::current_timestamp;

use crate::constants::{DEFAULT_HEARTBEAT_INTERVAL, DEFAULT_HEARTBEAT_INTERVAL_VARIANCE, DEFAULT_HEARTBEAT_THRESHOLD};
use crate::ping::Pinging;

/// Configuration of the Heartbeat
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Validate, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    /// Round-to-round variance to complicate network sync
    pub variance: u64,
    /// Interval in which the heartbeat is triggered
    pub interval: u64,
    /// The maximum number of concurrent heartbeat pings
    pub threshold: u64,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl HeartbeatConfig {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(variance: u64, interval: u64, threshold: u64) -> HeartbeatConfig {
        HeartbeatConfig {
            variance,
            interval,
            threshold,
        }
    }
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
    rng: rand::rngs::ThreadRng,
}

impl<T: Pinging, API: HeartbeatExternalApi> Heartbeat<T, API> {
    pub fn new(config: HeartbeatConfig, pinger: T, external_api: API) -> Self {
        Self {
            config,
            pinger,
            external_api,
            rng: rand::thread_rng(),
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

            info!(
                "Starting a heartbeat round for peers since timestamp {}",
                from_timestamp
            );
            let peers = self.external_api.get_peers(from_timestamp).await;

            // random timeout to avoid network sync:
            let this_round_planned_duration_in_ms: u64 = self
                .rng
                .gen_range(
                    self.config.interval
                        ..self
                            .config
                            .interval
                            .checked_add(self.config.variance.max(1))
                            .unwrap_or(u64::MAX),
                )
                .into();

            let timeout = sleep(std::time::Duration::from_millis(this_round_planned_duration_in_ms)).fuse();
            let ping = self.pinger.ping(peers).fuse();

            pin_mut!(timeout, ping);

            match select(timeout, ping).await {
                Either::Left(_) => info!("Heartbeat round interrupted by timeout"),
                Either::Right(_) => info!("Heartbeat round finished for all peers"),
            };

            let this_round_actual_duration_in_ms = current_timestamp().checked_sub(start).unwrap_or(0u64);
            if this_round_actual_duration_in_ms < this_round_planned_duration_in_ms {
                let time_to_wait_for_next_round = this_round_planned_duration_in_ms - this_round_actual_duration_in_ms;

                debug!("Heartbeat sleeping for: {}ms", time_to_wait_for_next_round);
                sleep(std::time::Duration::from_millis(time_to_wait_for_next_round)).await
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
            variance: 0u64,
            interval: 5u64,
            threshold: 0u64,
        }
    }

    pub struct DelayingPinger {
        pub delay: u64,
    }

    #[async_trait(? Send)]
    impl Pinging for DelayingPinger {
        async fn ping(&mut self, _peers: Vec<PeerId>) {
            sleep(std::time::Duration::from_millis(self.delay)).await;
        }
    }

    #[async_std::test]
    async fn test_heartbeat_should_loop_multiple_times() {
        let config = simple_heartbeat_config();

        let ping_delay = 5u64;
        let expected_loop_count = 2;

        let mut mock = MockHeartbeatExternalApi::new();
        mock.expect_get_peers()
            .times(expected_loop_count..)
            .return_const(vec![PeerId::random(), PeerId::random()]);

        let mut heartbeat = Heartbeat::new(config, DelayingPinger { delay: ping_delay }, mock);

        futures_lite::future::race(
            heartbeat.heartbeat_loop(),
            sleep(std::time::Duration::from_millis(
                (expected_loop_count as u64) * ping_delay,
            )),
        )
        .await;
    }

    #[async_std::test]
    async fn test_heartbeat_should_interrupt_long_running_heartbeats() {
        let mut config = simple_heartbeat_config();
        config.interval = 5u64;

        let ping_delay = 2 * config.interval;
        let expected_loop_count = 2;

        let mut mock = MockHeartbeatExternalApi::new();
        mock.expect_get_peers()
            .times(expected_loop_count..)
            .return_const(vec![PeerId::random(), PeerId::random()]);

        let mut heartbeat = Heartbeat::new(config.clone(), DelayingPinger { delay: ping_delay }, mock);

        futures_lite::future::race(
            heartbeat.heartbeat_loop(),
            sleep(std::time::Duration::from_millis(
                (expected_loop_count as u64) * config.interval,
            )),
        )
        .await;
    }
}
