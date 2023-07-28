use async_trait::async_trait;
use libp2p_identity::PeerId;
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
use rand::Rng;

use serde::{Deserialize, Serialize};
use validator::Validate;

use utils_log::{error,info};
use utils_metrics::metrics::SimpleHistogram;
use utils_metrics::histogram_start_measure;

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::sleep;
#[cfg(any(not(feature = "wasm"), test))]
use utils_misc::time::native::current_timestamp;

#[cfg(all(feature = "wasm", not(test)))]
use gloo_timers::future::sleep;
#[cfg(all(feature = "wasm", not(test)))]
use utils_misc::time::wasm::current_timestamp;

use crate::ping::Pinging;

/// Configuration of the Heartbeat
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Validate, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    /// The maximum number of concurrent heartbeats
    pub heartbeat_variance: f32,
    pub heartbeat_interval: u32,
    pub heartbeat_threshold: u64,
}

#[cfg_attr(test, mockall::automock)]
pub trait HeartbeatExternalApi {
    fn get_peers(&self, from_timestamp: u64) -> Vec<PeerId>;
}


pub struct Heartbeat<T: Pinging, API: HeartbeatExternalApi> {
    config: HeartbeatConfig,
    pinger: T,
    external_api: API,
    rng: rand::rngs::ThreadRng,
    metric_time_to_heartbeat: Option<SimpleHistogram>,
}

impl<T: Pinging, API: HeartbeatExternalApi> Heartbeat<T, API> {
    pub fn new(config: HeartbeatConfig, pinger: T, external_api: API) -> Self {
        Self {
            config, pinger, external_api,
            rng: rand::thread_rng(),
            metric_time_to_heartbeat: if cfg!(test) {None} else {SimpleHistogram::new(
                "core_histogram_heartbeat_time_seconds",
                "Measures total time it takes to probe all other nodes (in seconds)",
                vec![0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0, 60.0, 90.0, 120.0, 300.0],
            )
            .ok()},
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
            let from_timestamp = if start > self.config.heartbeat_threshold { start - self.config.heartbeat_threshold } else { start };
            info!("Starting a heartbeat round for peers since timestamp {}", from_timestamp);
            let peers = self.external_api.get_peers(from_timestamp);

            // random timeout to avoid network sync:
            let timeout_in_ms: u64 = if self.config.heartbeat_variance > 1.0 {
                self.rng
                .gen_range(self.config.heartbeat_interval..(self.config.heartbeat_interval + (self.config.heartbeat_variance as u32)))
                .into()
            } else {
                self.config.heartbeat_interval as u64
            };

            let timeout = sleep(std::time::Duration::from_millis(timeout_in_ms)).fuse();
            let ping = self.pinger.ping(peers).fuse();

            pin_mut!(timeout, ping);

            let _ = match select(timeout, ping).await {
                Either::Left(_) => info!("Heartbeat round interrupted by timeout"),
                Either::Right(_) => info!("Heartbeat round finished for all peers")
            };

            if let Some(metric_time_to_heartbeat) = &self.metric_time_to_heartbeat {
                metric_time_to_heartbeat.record_measure(heartbeat_round_timer.unwrap());
            };

            sleep(std::time::Duration::from_millis(0u64.max(current_timestamp() - start))).await
        }

    }

} 

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::*;
    use more_asserts::*;

    fn simple_heartbeat_config() -> HeartbeatConfig {
        HeartbeatConfig {
            heartbeat_variance: 0.0f32,
            heartbeat_interval: 5u32,
            heartbeat_threshold: 0u64
        }
    }

    pub struct DelayingPinger {
        pub delay: u64 
    }

    #[async_trait(? Send)]
    impl Pinging for DelayingPinger {
        async fn ping(&mut self, _peers:Vec<PeerId>) { 
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
            .times(expected_loop_count)
            .return_const(vec![PeerId::random(), PeerId::random()]);

        let mut heartbeat = Heartbeat::new(
            config, DelayingPinger{delay: ping_delay}, mock);
        
        let _ = futures_lite::future::race(
            heartbeat.heartbeat_loop(),
            sleep(std::time::Duration::from_millis((1u64 + (expected_loop_count as u64)) * ping_delay - 1u64))
        ).await;  
    }

    #[async_std::test]
    async fn test_heartbeat_should_interrupt_long_running_heartbeats() {
        let mut config = simple_heartbeat_config();
        config.heartbeat_interval = 5u32;

        let ping_delay = 2 * config.heartbeat_interval as u64;
        let expected_loop_count = 2;

        let mut mock = MockHeartbeatExternalApi::new();
        mock.expect_get_peers()
            .times(expected_loop_count)
            .return_const(vec![PeerId::random(), PeerId::random()]);

        let mut heartbeat = Heartbeat::new(
            config, DelayingPinger{delay: ping_delay}, mock);
        
        let _ = futures_lite::future::race(
            heartbeat.heartbeat_loop(),
            sleep(std::time::Duration::from_millis((expected_loop_count as u64) * ping_delay + 1u64))
        ).await;  
    }
}
