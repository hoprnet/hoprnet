use std::future::{Future, pending};
use std::time::Duration;
use std::pin::Pin;

use futures::stream::{FuturesUnordered, StreamExt};
use rand::Rng;

#[cfg(not(wasm))]
use async_std::task::sleep as sleep;

#[cfg(wasm)]
use gloo_timers::future::sleep as sleep;


type Packet = Box<[u8]>;

const DELAY_MIN_MS: u64 = 0;    /// minimum delay in milliseconds
const DELAY_MAX_MS: u64 = 200;  /// maximum delay in milliseconds


/// Mixer
/// API definition
/// - gets a random int to set a threshold for packet release
/// - uses a heap to establish an internal data structure
///
/// - push(packet) - push a packet into the mixer queue
/// - next() // create an iterable that returns a packet promise
/// - end() // clear mixer timeouts
///
/// mixer value comparator function
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct StochasticUniformMixer {
    queue: FuturesUnordered<Pin<Box<dyn Future<Output = Packet>>>>,
    delay_min_ms: u64,
    delay_max_ms: u64,
}


#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl StochasticUniformMixer {
    pub fn new() -> StochasticUniformMixer {
        StochasticUniformMixer::new_with_delay_spec(DELAY_MIN_MS, DELAY_MAX_MS)
    }

    pub fn new_with_delay_spec(delay_min_ms: u64, delay_max_ms: u64) -> StochasticUniformMixer {
        if delay_min_ms >= delay_max_ms {
            panic!("The minimum delay must be smaller than the maximum delay")
        }

        // push a dummy packet that is never going to be awaited resulting in a perpetually
        // open stream waiting for other futures to finish
        let futures: FuturesUnordered<Pin<Box<dyn Future<Output = Packet>>>> = FuturesUnordered::new();
        futures.push(Box::pin(async move {
            let () = pending().await;
            unreachable!();
        }));

        StochasticUniformMixer {
            queue: futures,
            delay_min_ms,
            delay_max_ms,
        }
    }
}

impl StochasticUniformMixer {
    /// Ingests a single packet along with the current timestamp
    ///
    /// # Arguments
    ///
    /// * `packet` - A string slice that holds the name of the person
    pub fn push(&mut self, packet: Packet) {    //
        let mut rng = rand::thread_rng();
        let random_delay =  rng.gen_range(self.delay_min_ms..self.delay_max_ms);

        self.queue.push(Box::pin(async move {
            sleep(Duration::from_millis(random_delay)).await;
            packet
        }));
    }

    pub async fn pop(&mut self) -> Packet {
        if let Some(packet) = self.queue.next().await {
            packet
        } else {
            panic!("There should have been a packet to dispatch")
        }
    }

    pub fn length(&self) -> usize {
        // accounting for the dummy packet
        self.queue.len() - 1
    }
}

// TODO: never used, should be exported?
// // move to misc-utils crate
// #[cfg(not(wasm))]
// fn current_timestamp() -> u64 {
//     use std::time::{SystemTime, UNIX_EPOCH};
//     match SystemTime::now().duration_since(UNIX_EPOCH) {
//         Ok(d) => d.as_millis() as u64,
//         Err(_) => 1,
//     }
// }
//
// #[cfg(wasm)]
// fn get_current_timestamp() -> u64 {
//     (js_sys::Date::now() / 1000.0) as u64
// }


#[cfg(feature = "wasm")]
pub mod wasm {
    use super::*;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;
    // use js_sys::AsyncIterator;

    #[wasm_bindgen]
    impl StochasticUniformMixer {
        pub fn push_data(&mut self, packet: JsValue) {
            self.push(Box::from_iter(js_sys::Uint8Array::new(&packet).to_vec()));
        }

        // TODO: create an async iterator implementation
        // pub async fn pop_data(&mut self) -> js_sys::Promise {
        //     let value = self.pop().await;
        //     wasm_bindgen_futures::future_to_promise(async move {
        //         Ok(serde_wasm_bindgen::to_value(value.as_ref())?)
        //     })
        // }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    const TINY_CONSTANT_DELAY: u64 = 20;        // ms

    fn random_packet() -> Packet {
        let mut rng = rand::thread_rng();

        Box::new([rng.gen()])
    }

    #[test]
    fn test_mixer_without_any_packets_should_declare_none() {
        let mixer = StochasticUniformMixer::new();

        assert_eq!(0, mixer.length())
    }

    #[test]
    #[should_panic]
    fn test_mixer_with_incorrect_delay_ordering_in_definition_should_panic() {
        std::panic::set_hook(Box::new(|_| {}));       // remove stack trace on expected panic

        let smaller: u64 = 5;
        let greater: u64 = 10;

        StochasticUniformMixer::new_with_delay_spec(greater, smaller);
    }

    #[test]
    fn test_mixer_should_register_packets() {
        let mut mixer = StochasticUniformMixer::new();

        mixer.push(random_packet());

        assert_eq!(1, mixer.length());
    }

    #[async_std::test]
    async fn test_mixer_should_return_no_value_if_it_is_empty() {
        let mut mixer = StochasticUniformMixer::new_with_delay_spec(TINY_CONSTANT_DELAY-1, TINY_CONSTANT_DELAY);

        if let Err(_) = async_std::future::timeout(Duration::from_millis(TINY_CONSTANT_DELAY), mixer.pop()).await {
            assert!(true, "Timeout expected as no packet has been pushed")
        }
    }

    #[async_std::test]
    async fn test_mixer_should_return_exactly_the_same_packet_as_obtained() {
        let mut mixer = StochasticUniformMixer::new();

        let expected = random_packet();
        mixer.push(expected.clone());
        let actual = mixer.pop().await;

        assert_eq!(expected, actual)
    }

    #[async_std::test]
    async fn test_mixer_should_return_the_scheduled_packet() {
        let mut mixer = StochasticUniformMixer::new_with_delay_spec(TINY_CONSTANT_DELAY-1, TINY_CONSTANT_DELAY);

        mixer.push(random_packet());
        let actual = mixer.pop().await;

        assert!(actual.len() > 0);
    }

    #[async_std::test]
    async fn test_mixer_pop_operation_will_not_last_longer_than_the_latest_possible_interval() {
        let mut mixer = StochasticUniformMixer::new_with_delay_spec(TINY_CONSTANT_DELAY-1, TINY_CONSTANT_DELAY);

        let tolerance_ms = 5;
        let packet_count = 10;

        let timer = std::time::Instant::now();

        for _ in 1..packet_count {
            mixer.push(random_packet());
        }
        for _ in 1..packet_count {
            mixer.pop().await;
        }

        assert!(timer.elapsed().as_millis() <= (TINY_CONSTANT_DELAY + tolerance_ms) as u128);
    }

    #[async_std::test]
    async fn test_mixer_should_be_resumable() {
        let mut mixer = StochasticUniformMixer::new_with_delay_spec(TINY_CONSTANT_DELAY-1, TINY_CONSTANT_DELAY);

        mixer.push(random_packet());
        mixer.pop().await;

        if let Err(_) = async_std::future::timeout(Duration::from_millis(TINY_CONSTANT_DELAY), mixer.pop()).await {
            assert!(true, "Timeout expected as no packets can be fetched")
        }

        let expected = random_packet();
        mixer.push(expected.clone());
        let actual = mixer.pop().await;

        assert_eq!(expected, actual);
    }
}
