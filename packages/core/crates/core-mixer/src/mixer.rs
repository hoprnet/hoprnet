use std::cmp::Ordering;

use priority_queue::PriorityQueue;
use rand::Rng;


type Packet = u8;      // [u8; MAX_PACKET_SIZE];

const MIXER_BUFFER_CAPACITY: usize = 1024;
const DELAY_MIN_VALUE: u64 = 0;    /// minimum delay in milliseconds
const DELAY_MAX_VALUE: u64 = 200;  /// maximum delay in milliseconds

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct MixerTimestamp {
    value: u64
}

/// This reverses the timestamp logic
///
/// The higher the timestamp, the later in time the scheduling is required, therefore the smaller
/// the priority.
impl Ord for MixerTimestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.value < other.value {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}

impl PartialOrd for MixerTimestamp {
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
   queue: PriorityQueue<Packet, MixerTimestamp>,
   min_packet_count: usize,
}


#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl StochasticUniformMixer {
    pub fn new(packet_release_threshold: usize) -> StochasticUniformMixer {
        StochasticUniformMixer {
            queue: PriorityQueue::with_capacity_and_default_hasher(MIXER_BUFFER_CAPACITY),
            min_packet_count: packet_release_threshold
        }
    }
}


impl StochasticUniformMixer {
    pub fn pop(&mut self) -> Option<Packet> {
        if self.queue.len() > self.min_packet_count  {
            if let Some(top) = self.queue.peek() {
                let (value, _priority) = top;
                return Some(*value);
            }
        }

        None
    }

    /// Ingests a single packet along with the current timestamp
    ///
    /// # Arguments
    ///
    /// * `packet` - A string slice that holds the name of the person
    /// * `timestamp` - The current timestamp (note: necessary due to WASM interface)
    pub fn push(&mut self, packet: Packet, timestamp: u64) {    //
        let mut rng = rand::thread_rng();
        let random_delay =  rng.gen_range(DELAY_MIN_VALUE..DELAY_MAX_VALUE);
        if let Some(_) = self.queue.push(packet, MixerTimestamp{value: timestamp + random_delay}) {
            panic!("Duplicate packet detected")
        }
    }

    pub fn length(&self) -> usize {
        self.queue.len()
    }
}

/// Module for WASM-specific Rust code
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
            let timestamp_now = (js_sys::Date::now() / 1000.0) as u64;
            self.push(packet, timestamp_now);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    const NO_PACKET_THRESHOLD: usize = 0;

    #[test]
    fn test_mixer_timestamp_derives_equality() {
        let left = MixerTimestamp{ value: 1 };
        let right = MixerTimestamp{ value: 1 };

        assert_eq!(left, right)
    }

    #[test]
    fn test_mixer_timestamp_derives_reverse_instead_of_direct_ordering() {
        let smaller = MixerTimestamp{ value: 1 };
        let greater = MixerTimestamp{ value: 2 };

        assert!(smaller > greater);
        assert!(greater < smaller);
    }

    fn random_packet() -> Packet {
        let mut rng = rand::thread_rng();

        let value: Packet = rng.gen();
        value
    }

    #[test]
    fn test_mixer_without_any_packets_should_declare_none() {
        let mixer = StochasticUniformMixer::new(NO_PACKET_THRESHOLD);

        assert_eq!(0, mixer.length())
    }

    #[test]
    fn test_mixer_should_register_packets() {
        let mut mixer = StochasticUniformMixer::new(NO_PACKET_THRESHOLD);

        mixer.push(random_packet(), 0);
        assert_eq!(1, mixer.length());
    }

    #[test]
    #[should_panic]
    fn test_mixer_should_panic_on_adding_the_same_packet() {
        std::panic::set_hook(Box::new(|_| {}));

        let mut mixer = StochasticUniformMixer::new(NO_PACKET_THRESHOLD);

        let random_generated_packet = random_packet();
        mixer.push(random_generated_packet, 0);
        mixer.push(random_generated_packet, 1);
    }

    #[test]
    fn test_mixer_should_return_no_value_if_it_is_empty() {
        let mut mixer = StochasticUniformMixer::new(NO_PACKET_THRESHOLD);

        let actual = mixer.pop();
        assert!(actual.is_none())
    }

    #[test]
    fn test_mixer_should_return_no_value_unless_it_contains_at_least_the_threshold_amount_of_values() {
        let packet_threshold = 1;
        let mut mixer = StochasticUniformMixer::new(packet_threshold);

        mixer.push(random_packet(), 0);
        let actual = mixer.pop();

        assert!(actual.is_none())
    }

    #[test]
    fn test_mixer_should_return_value_if_more_than_threshold_values_are_present() {
        let mut mixer = StochasticUniformMixer::new(NO_PACKET_THRESHOLD);

        mixer.push(random_packet(), 0);
        let actual = mixer.pop();

        assert!(actual.is_some())
    }

    #[test]
    fn test_mixer_should_the_value_with_an_earlier_timestamp_first() {
        let at_least_two_packets = 1;
        let mut mixer = StochasticUniformMixer::new(at_least_two_packets);

        let (packet_with_earlier_timestamp, earlier_timestamp) = (random_packet(), DELAY_MIN_VALUE);
        let (packet_with_latter_timestamp, latter_timestamp) = (random_packet(), (DELAY_MAX_VALUE * 2));

        mixer.push(packet_with_latter_timestamp, latter_timestamp);
        mixer.push(packet_with_earlier_timestamp, earlier_timestamp);

        let first = mixer.pop();

        assert!(first.is_some());
        assert_eq!(first.unwrap(), packet_with_earlier_timestamp);
    }
}



// This function must not use WASM-specific types. Only WASM-compatible types must be used,
// because the attribute makes it available to both WASM and non-WASM (pure Rust)
// #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
// pub fn bar() -> u32 { 0 }

// impl MyStruct {
//     // Here, specify methods with types that are strictly NOT WASM-compatible.
// }
//
// #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
// impl MyStruct {
//     // Here, specify methods with types that are strictly WASM-compatible, but not WASM-specific.
// }

// // Trait implementations for types can NEVER be made available for WASM
// impl std::string::ToString for MyStruct {
//     fn to_string(&self) -> String {
//         format!("{}", self.foo)
//     }
// }



