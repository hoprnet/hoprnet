use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use rand::Rng;

use utils_log::debug;

#[cfg(all(feature = "prometheus", not(test)))]
use {lazy_static::lazy_static, utils_metrics::metrics::SimpleGauge};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static! {
    static ref METRIC_QUEUE_SIZE: SimpleGauge =
        SimpleGauge::new("core_gauge_mixer_queue_size", "Current mixer queue size").unwrap();
    static ref METRIC_AVERAGE_DELAY: SimpleGauge = SimpleGauge::new(
        "core_gauge_mixer_average_packet_delay",
        "Average mixer packet delay averaged over a packet window"
    )
    .unwrap();
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct MixerConfig {
    min_delay: Duration,
    max_delay: Duration,
    pub metric_delay_window: u64,
}

impl Default for MixerConfig {
    fn default() -> Self {
        Self {
            min_delay: Duration::from_millis(0u64),
            max_delay: Duration::from_millis(200u64),
            metric_delay_window: 10u64,
        }
    }
}

impl MixerConfig {
    /// Get a random delay duration from the specified minimum and maximum delay available
    /// inside the configuration.
    fn random_delay(&self) -> Duration {
        let mut rng = rand::thread_rng();
        let random_delay = rng.gen_range(self.min_delay.as_millis()..self.max_delay.as_millis()) as u64;

        Duration::from_millis(random_delay)
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl MixerConfig {
    /// Create an instance of a new mixer configuration
    ///
    /// # Arguments
    /// * `min_delay` - the minimum delay invoked by the mixer represented in milliseconds
    /// * `min_delay` - the maximum delay invoked by the mixer represented in milliseconds
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
    pub fn new(min_delay: u64, max_delay: u64) -> Self {
        if min_delay >= max_delay {
            panic!("The minimum delay must be smaller than the maximum delay")
        }

        Self {
            min_delay: Duration::from_millis(min_delay),
            max_delay: Duration::from_millis(max_delay),
            metric_delay_window: 10u64,
        }
    }
}

type SleepFn = fn(Duration) -> Box<dyn Future<Output = ()>>;

/// Mixer implementation using Async instead of a real queue to provide the packet mixing functionality
#[derive(Copy, Clone)]
pub struct Mixer {
    pub cfg: MixerConfig,
    sleep: SleepFn,
}

impl Mixer {
    /// Creates new mixer with standard sleep function.
    /// This constructor is not suitable for WASM context and will panic.
    pub fn new(cfg: MixerConfig) -> Self {
        Self {
            sleep: |d| Box::new(async_std::task::sleep(d)),
            cfg,
        }
    }

    /// Creates new Mixer with gloo-timers for sleep method.
    /// This constructor is suitable for WASM context.
    #[cfg(feature = "wasm")]
    pub fn new_with_gloo_timers(cfg: MixerConfig) -> Self {
        Self {
            sleep: |d| Box::new(gloo_timers::future::sleep(d)),
            cfg,
        }
    }

    /// Async mixing operation on the packet.
    ///
    /// # Arguments
    /// * data: Data ontent to be time mixed
    ///
    /// # Returns
    /// The same data as ingested on input postponed by a random delay
    pub async fn mix<T>(&self, data: T) -> T {
        let random_delay = self.cfg.random_delay();
        debug!("Mixer created a random packet delay of {}ms", random_delay.as_millis());

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_QUEUE_SIZE.increment(1.0f64);

        Pin::from((self.sleep)(random_delay)).await;

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            METRIC_QUEUE_SIZE.decrement(1.0f64);

            let weight = 1.0f64 / self.cfg.metric_delay_window as f64;
            METRIC_AVERAGE_DELAY
                .set((weight * random_delay.as_millis() as f64) + ((1.0f64 - weight) * METRIC_AVERAGE_DELAY.get()));
        }

        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::future_extensions::StreamThenConcurrentExt;
    use futures_lite::stream::StreamExt;
    use more_asserts::*;

    type Packet = Box<[u8]>;

    const TINY_CONSTANT_DELAY: Duration = Duration::from_millis(10);

    fn random_packets(count: usize) -> Vec<Packet> {
        let mut rng = rand::thread_rng();
        let mut packets: Vec<Packet> = Vec::new();

        for _ in 0..count {
            packets.push(Box::new([rng.gen()]))
        }

        packets
    }

    #[async_std::test]
    async fn test_then_concurrent_empty_stream_should_not_produce_a_value_if_none_is_ready() {
        let mut stream = futures::stream::iter(random_packets(1)).then_concurrent(|x| async {
            async_std::task::sleep(TINY_CONSTANT_DELAY * 3).await;
            x
        });

        if let Err(_) = async_std::future::timeout(TINY_CONSTANT_DELAY, stream.next()).await {
            assert!(true, "Timeout expected, the packet should not get through the pipeline")
        } else {
            assert!(false, "Timeout expected, but none occurred");
        }
    }

    #[async_std::test]
    async fn test_then_concurrent_proper_execution_results_in_concurrent_processing() {
        let constant_delay = Duration::from_millis(10);
        let tolerance = Duration::from_millis(1);

        let expected = vec![1, 2, 3];

        let start = std::time::Instant::now();

        let stream = futures::stream::iter(expected.clone()).then_concurrent(|x| async move {
            async_std::task::sleep(constant_delay).await;
            x
        });

        let _ = stream.collect::<Vec<i32>>().await;

        let elapsed = start.elapsed();
        assert_gt!(elapsed, constant_delay);
        assert_lt!(elapsed - constant_delay, tolerance);
    }

    #[async_std::test]
    async fn test_then_concurrent_futures_are_processed_in_the_correct_order() {
        let packet_1 = 10u64; // 3rd in the output
        let packet_2 = 5u64; // 1st in the output
        let packet_3 = 7u64; // 2nd in the output
        let expected_packets = vec![packet_2, packet_3, packet_1];

        let stream = futures::stream::iter(vec![packet_1, packet_2, packet_3]).then_concurrent(|x| async move {
            async_std::task::sleep(std::time::Duration::from_millis(x)).await;
            x
        });
        let actual_packets = stream.collect::<Vec<u64>>().await;

        assert_eq!(actual_packets, expected_packets);
    }
}
