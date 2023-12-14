use std::time::Duration;

use rand::Rng;

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
    /// Create an instance of a new mixer configuration
    ///
    /// # Arguments
    /// * `min_delay` - the minimum delay invoked by the mixer represented in milliseconds
    /// * `min_delay` - the maximum delay invoked by the mixer represented in milliseconds
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

    /// Get a random delay duration from the specified minimum and maximum delay available
    /// inside the configuration.
    pub fn random_delay(&self) -> Duration {
        let mut rng = rand::thread_rng();
        let random_delay = rng.gen_range(self.min_delay.as_millis()..self.max_delay.as_millis()) as u64;

        Duration::from_millis(random_delay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_lite::stream::StreamExt;
    use more_asserts::*;
    use rust_stream_ext_concurrent::then_concurrent::StreamThenConcurrentExt;

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
        let constant_delay = Duration::from_millis(50);
        let tolerance = Duration::from_millis(3);

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
