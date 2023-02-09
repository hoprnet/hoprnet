use std::cmp::Ordering;

use priority_queue::PriorityQueue;      // TODO: replace with https://doc.rust-lang.org/src/alloc/collections/binary_heap.rs.html#268
use rand::Rng;

type Packet = Box<[u8]>;

const MIXER_BUFFER_CAPACITY: usize = 1024;
const DELAY_MIN_MS: u64 = 0;    /// minimum delay in milliseconds
const DELAY_MAX_MS: u64 = 200;  /// maximum delay in milliseconds

#[cfg(not(wasm))]
async fn sleep(time: std::time::Duration) -> futures_timer::Delay {
    futures_timer::Delay::new(time)
}

#[cfg(not(wasm))]
type Delay = futures_timer::Delay;

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct ScheduledPacket {
    timestamp: u64,
    // TODO: Add packet here
}

/// This reverses the timestamp logic
///
/// The higher the timestamp, the later in time the scheduling is required, therefore the smaller
/// the priority.
impl Ord for ScheduledPacket {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.timestamp < other.timestamp {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}

impl PartialOrd for ScheduledPacket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
    queue: PriorityQueue<Packet, ScheduledPacket>,
    delay_min_ms: u64,
    delay_max_ms: u64,
    timer: futures_timer::Delay,
}


#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl StochasticUniformMixer {
    pub fn new() -> StochasticUniformMixer {
        StochasticUniformMixer::new_with_delay_spec(DELAY_MIN_MS, DELAY_MAX_MS)
    }

    pub fn new_with_delay_spec(delay_min_ms: u64, delay_max_ms: u64) -> StochasticUniformMixer {
        StochasticUniformMixer {
            queue: PriorityQueue::with_capacity_and_default_hasher(MIXER_BUFFER_CAPACITY),
            delay_min_ms,
            delay_max_ms,
            timer: Delay::new(std::time::Duration::from_secs(0))
        }
    }
}

impl StochasticUniformMixer {
    /// Ingests a single packet along with the current timestamp
    ///
    /// # Arguments
    ///
    /// * `packet` - A string slice that holds the name of the person
    /// !!! The same packet cannot be duplicated!
    pub fn push(&mut self, packet: Packet) {    //
        let mut rng = rand::thread_rng();
        let random_delay =  rng.gen_range(self.delay_min_ms..self.delay_max_ms);
        if let Some(_) = self.queue.push(packet, ScheduledPacket { timestamp: current_timestamp() + random_delay}) {
            panic!("Duplicate packet detected")
        }

        let current_ts = current_timestamp();
        if let Some((_, priority)) = self.queue.peek() {
            let remaining_time = if priority.timestamp > current_ts { priority.timestamp - current_ts } else { 0u64 };
            self.timer.reset(std::time::Duration::from_millis(remaining_time));
        }
    }

    pub fn pop(&mut self) -> Option<Packet> {
        if let Some((_, priority)) = self.queue.peek() {
            if priority.timestamp <= current_timestamp() {
                if let Some((value, _ )) = self.queue.pop() {
                    return Some(value);
                } else {
                    panic!("Expected to pop out a value")
                }
            }
        }

        None
    }

    pub async fn pop_async(&mut self) -> Packet {
        loop {
            let bar = futures::future::pending::<()>();
            match futures::future::select(&mut self.timer, bar).await {
                futures::future::Either::Left(_) => {
                    if self.queue.is_empty() {
                        self.timer.reset(std::time::Duration::from_millis(1000));
                    } else {
                        break;
                    };
                },
                futures::future::Either::Right(_) => unreachable!("A failed case for long wait time"),
            }
        }

        if let Some((value, _ )) = self.queue.pop() {
            return value;
        } else {
            panic!("There should have been a value")
        }
    }

    pub fn peek(&self) -> Option<(&Packet,&ScheduledPacket)> {
        self.queue.peek()
    }

    pub fn length(&self) -> usize {
        self.queue.len()
    }
}

// #[cfg(wasm)]
// Some wrapper with specific definitions
// use Timer = gloo_timer::Timer;


// move to misc-utils crate
#[cfg(not(wasm))]
fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_millis() as u64,
        Err(_) => 1,
    }
}

#[cfg(wasm)]
fn get_current_timestamp() -> u64 {
    (js_sys::Date::now() / 1000.0) as u64
}


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

    #[test]
    fn test_scheduled_packet_derives_equality() {
        let left = ScheduledPacket { timestamp: 1 };
        let right = ScheduledPacket { timestamp: 1 };

        assert_eq!(left, right)
    }

    #[test]
    fn test_mixer_timestamp_derives_reverse_instead_of_direct_ordering_for_priority_queue_logic() {
        let smaller = ScheduledPacket { timestamp: 1 };
        let greater = ScheduledPacket { timestamp: 2 };

        assert!(smaller > greater);
        assert!(greater < smaller);
    }

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

    #[test]
    fn test_mixer_should_return_no_value_if_it_is_empty() {
        let mut mixer = StochasticUniformMixer::new();

        let actual = mixer.pop();
        assert!(actual.is_none())
    }

    #[tokio::test]
    async fn test_mixer_should_return_exactly_the_same_packet_as_obtained() {
        let mut mixer = StochasticUniformMixer::new();

        let expected = random_packet();

        mixer.push(expected.clone());
        sleep(std::time::Duration::from_millis(2)).await;
        let actual = mixer.pop_async().await;

        assert_eq!(expected, actual)
    }

    #[tokio::test]
    async fn test_mixer_should_return_the_scheduled_packet() {
        let mut mixer = StochasticUniformMixer::new();

        mixer.push(random_packet());
        let actual = mixer.pop_async().await;

        assert!(actual.len() > 0);
    }

    #[tokio::test]
    async fn test_mixer_should_return_the_value_with_an_earlier_timestamp_first() {
        let mut mixer = StochasticUniformMixer::new();

        mixer.push(random_packet());
        mixer.push(random_packet());

        let first_timestamp = mixer.peek().unwrap().1.timestamp.clone();
        let _x =  mixer.pop_async().await;
        let second_timestamp = mixer.peek().unwrap().1.timestamp.clone();

        assert!(first_timestamp < second_timestamp);
    }

    #[tokio::test]
    async fn test_mixer_pop_operation_with_will_not_last_longer_than_the_latest_possible_interval() {
        let base_delay: u64 = 15;

        let mut mixer = StochasticUniformMixer::new_with_delay_spec(base_delay, base_delay+1);

        let first = random_packet();
        let second = random_packet();

        let before = current_timestamp();

        mixer.push(first);
        sleep(std::time::Duration::from_millis(base_delay/2)).await;
        mixer.push(second);

        let _ = mixer.pop_async().await;
        let _ = mixer.pop_async().await;

        let after = current_timestamp();

        // assert_eq!(before, after);
        assert!((after - before) < (base_delay + base_delay / 2));
    }
}


