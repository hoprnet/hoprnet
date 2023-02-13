use std::time::Duration;

use futures::stream::{Stream};
use futures_lite::stream::StreamExt;
use rand::Rng;

use crate::future_extensions::StreamThenConcurrentExt;


#[cfg(not(wasm))]
use async_std::task::sleep as sleep;

#[cfg(wasm)]
use gloo_timers::future::sleep as sleep;


const DELAY_MIN_MS: u64 = 0;    /// minimum delay in milliseconds
const DELAY_MAX_MS: u64 = 200;  /// maximum delay in milliseconds


#[cfg(feature = "wasm")]
pub mod wasm {
    use super::*;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::stream::JsStream;
    use wasm_bindgen::JsValue;
    use utils_misc::async_iterable::wasm::{to_box_u8_stream,to_jsvalue_stream};
    use js_sys::AsyncIterator;

    #[wasm_bindgen]
    pub struct AsyncIterableHelperMixer {
        stream: Box<dyn Stream<Item = Result<Box<[u8]>, String>> + Unpin>,
    }

    pub fn new(packet_input: AsyncIterator) -> Result<AsyncIterableHelperMixer,String> {
        if DELAY_MIN_MS >= DELAY_MAX_MS {
            panic!("The minimum delay must be smaller than the maximum delay")
        }

        let stream = JsStream::from(packet_input)
            .map(to_box_u8_stream)
            .then_concurrent(|packet| async move {
                let mut rng = rand::thread_rng();
                let random_delay =  rng.gen_range(DELAY_MIN_MS..DELAY_MAX_MS);

                sleep(Duration::from_millis(random_delay)).await;
                packet
            });

        Ok(AsyncIterableHelperMixer {
            stream: Box::new(Box::pin(stream)),
        })
    }

    impl AsyncIterableHelperMixer {
        pub async fn next(&mut self) -> Result<JsValue, JsValue> {
            to_jsvalue_stream(self.stream.as_mut().next().await)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
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
        let mut stream = futures::stream::iter(random_packets(1))
            .then_concurrent(|x|async {
                sleep(TINY_CONSTANT_DELAY * 3).await;
                x
            });

        if let Err(_) = async_std::future::timeout(TINY_CONSTANT_DELAY, stream.next()).await {
            assert!(true, "Timeout expected, it's ok, the packet could not get through the pipeline")
        } else {
            assert!(false, "Timeout expected, but none occurred");
        }
    }

    #[async_std::test]
    async fn test_then_concurrent_proper_execution_results_in_concurrent_processing() {
        let constant_delay = Duration::from_millis(10);
        let tolerance = Duration::from_millis(1);

        let expected = vec!(1,2,3);

        let start = std::time::Instant::now();

        let stream = futures::stream::iter(expected.clone()).then_concurrent(
            |x| async move {
                sleep(constant_delay).await;
                x
            });

        let _ = stream.collect::<Vec<i32>>().await;

        assert_gt!(start.elapsed(), constant_delay);
        assert_lt!(start.elapsed() - constant_delay, tolerance);
    }

    #[async_std::test]
    async fn test_then_concurrent_futures_are_processed_in_the_correct_order() {
        let packet_1 = 10u64;         // 3rd in the output
        let packet_2 = 5u64;         // 1st in the output
        let packet_3 = 7u64;         // 2nd in the output
        let expected_packets = vec!(packet_2, packet_3, packet_1);

        let stream = futures::stream::iter(vec!(packet_1, packet_2, packet_3))
            .then_concurrent(|x| async move {
                sleep(std::time::Duration::from_millis(x)).await;
                x
            });
        let actual_packets = stream.collect::<Vec<u64>>().await;

        assert_eq!(actual_packets, expected_packets);
    }
}
