use std::future::{Future, ready};
use std::time::Duration;
use std::pin::Pin;
use std::task::{Waker, Context, Poll};

use futures::future::Either;
use futures::stream::{FuturesUnordered, StreamExt};
use rand::Rng;

// NOTE: futures_timer::Delay did not work in tokio runtime
#[cfg(not(wasm))]
use tokio::time::sleep as sleep;

#[cfg(wasm)]
use gloo_timers::future::sleep as sleep;



type Packet = Box<[u8]>;

const DELAY_MIN_MS: u64 = 0;    /// minimum delay in milliseconds
const DELAY_MAX_MS: u64 = 200;  /// maximum delay in milliseconds

struct MixerWakingFuture<'a> {
    mixer: &'a mut StochasticUniformMixer,
}

impl Future for MixerWakingFuture<'_> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.mixer.waker = Some(cx.waker().clone());

        if self.mixer.length() > 0 {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

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
    waker: Option<Waker>
}


#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl StochasticUniformMixer {
    pub fn new() -> StochasticUniformMixer {
        StochasticUniformMixer::new_with_delay_spec(DELAY_MIN_MS, DELAY_MAX_MS)
    }

    pub fn new_with_delay_spec(delay_min_ms: u64, delay_max_ms: u64) -> StochasticUniformMixer {
        StochasticUniformMixer {
            queue: FuturesUnordered::new(),
            delay_min_ms,
            delay_max_ms,
            waker: None
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

        if let Some(waker) = &self.waker {
            waker.wake_by_ref();
        }
    }

    pub async fn pop(&mut self) -> Packet {
        let timeout_til_next_packet = if self.queue.len() > 0 {
            Either::Left(ready(()))
        } else {
            let mixer_waking = MixerWakingFuture{mixer: self};
            Either::Right(async move {
                mixer_waking.await;
                ()
            })
        };
        timeout_til_next_packet.await;

        if let Some(packet) = self.queue.next().await {
            packet
        } else {
            panic!("There should have been a packet to dispatch")
        }
    }

    pub fn length(&self) -> usize {
        self.queue.len()
    }
}

// TOTO: never used, should be exported?
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
    // TODO: add JS API here
    // Use this module to specify everything that is WASM-specific (e.g. uses wasm-bindgen types, js_sys, ...etc.)

    use super::*;
    use wasm_bindgen::prelude::*;
    // use wasm_bindgen::JsValue;

    #[wasm_bindgen]
    impl StochasticUniformMixer {
        pub fn push_data(&mut self, packet: Packet) {
            self.push(packet);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn random_packet() -> Packet {
        let mut rng = rand::thread_rng();

        Box::new([rng.gen()])
    }

    const TINY_CONSTANT_DELAY: u64 = 20;        // ms

    #[test]
    fn test_mixer_without_any_packets_should_declare_none() {
        let mixer = StochasticUniformMixer::new();

        assert_eq!(0, mixer.length())
    }

    #[test]
    fn test_mixer_should_register_packets() {
        let mut mixer = StochasticUniformMixer::new();

        mixer.push(random_packet());

        assert_eq!(1, mixer.length());
    }

    #[tokio::test]
    async fn test_mixer_should_return_no_value_if_it_is_empty() {
        let mut mixer = StochasticUniformMixer::new_with_delay_spec(TINY_CONSTANT_DELAY-1, TINY_CONSTANT_DELAY);

        if let Err(_) = tokio::time::timeout(Duration::from_millis(TINY_CONSTANT_DELAY), mixer.pop()).await {
            assert!(true, "Timeout expected as no packet has been pushed")
        }
    }

    // #[tokio::test]
    // async fn test_mixer_should_wait_for_packet_push() {
    //     todo!()
    // }

    #[tokio::test]
    async fn test_mixer_should_return_exactly_the_same_packet_as_obtained() {
        let mut mixer = StochasticUniformMixer::new();

        let expected = random_packet();
        mixer.push(expected.clone());
        let actual = mixer.pop().await;

        assert_eq!(expected, actual)
    }

    #[tokio::test]
    async fn test_mixer_should_return_the_scheduled_packet() {
        let mut mixer = StochasticUniformMixer::new_with_delay_spec(TINY_CONSTANT_DELAY-1, TINY_CONSTANT_DELAY);

        mixer.push(random_packet());
        let actual = mixer.pop().await;

        assert!(actual.len() > 0);
    }

    #[tokio::test]
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
}
