use std::cmp::Ordering;

use priority_queue::PriorityQueue;
use rand::Rng;


/// Mixer
/// API definition
/// - gets a random int to set a threshold for packet release
/// - uses a heap to establish an internal data structure
///
/// - push(packet) - push a packet into the mixer queue
/// - next() // create an iterable that returns a packet promise
/// - end() // clear mixer timeouts

/// mixer value comparator function

type Packet = u8;      // [u8; MAX_PACKET_SIZE];

const MIXER_BUFFER_CAPACITY: usize = 1024;

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct MixerTimestamp {
    value: u32
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



#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Mixer {
   queue: PriorityQueue<Packet, MixerTimestamp>,
   min_packet_count: usize,
}


#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Mixer {
    pub fn new(packet_release_threshold: usize) -> Mixer  {
        Mixer {
            queue: PriorityQueue::with_capacity_and_default_hasher(MIXER_BUFFER_CAPACITY),
            min_packet_count: packet_release_threshold
        }
    }
}


impl Mixer {
    pub fn pop(&mut self) -> Option<Packet> {
        if self.queue.len() > self.min_packet_count  {
            if let Some(top) = self.queue.peek() {
                let (value, _priority) = top;
                return Some(*value);
            }
        }

        None
    }

    pub fn push(&mut self, packet: Packet) {
        let random_priority = 5;
        if let Some(_) = self.queue.push(packet, MixerTimestamp{value: random_priority}) {
            panic!("Duplicate packet detected")
        }
    }

    pub fn length(&self) -> usize {
        self.queue.len()
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
        let left = MixerTimestamp{ value: 1 };
        let right = MixerTimestamp{ value: 2 };

        assert!(left > right)
    }

    fn random_packet() -> Packet {
        let mut rng = rand::thread_rng();

        let value: Packet = rng.gen();
        value
    }

    #[test]
    fn test_mixer_without_any_packets_should_declare_none() {
        let mixer = Mixer::new(NO_PACKET_THRESHOLD);

        assert_eq!(0, mixer.length())
    }

    #[test]
    fn test_mixer_should_register_packets() {
        let mut mixer = Mixer::new(NO_PACKET_THRESHOLD);

        mixer.push(random_packet());
        assert_eq!(1, mixer.length());
    }

    #[test]
    #[should_panic]
    fn test_mixer_should_panic_on_adding_the_same_packet() {
        std::panic::set_hook(Box::new(|_| {}));

        let mut mixer = Mixer::new(NO_PACKET_THRESHOLD);

        let random_generated_packet = random_packet();
        mixer.push(random_generated_packet);
        mixer.push(random_generated_packet);
    }

    #[test]
    fn test_mixer_should_return_no_value_if_it_is_empty() {
        let mut mixer = Mixer::new(NO_PACKET_THRESHOLD);

        let actual = mixer.pop();
        assert!(actual.is_none())
    }

    #[test]
    fn test_mixer_should_return_no_value_unless_it_contains_at_least_the_threshold_amount_of_values() {
        let packet_threshold = 1;
        let mut mixer = Mixer::new(packet_threshold);

        mixer.push(random_packet());
        let actual = mixer.pop();

        assert!(actual.is_none())
    }

    #[test]
    fn test_mixer_should_return_value_if_more_than_threshold_values_are_present() {
        let mut mixer = Mixer::new(NO_PACKET_THRESHOLD);

        mixer.push(random_packet());
        let actual = mixer.pop();

        assert!(actual.is_some())
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



/// Module for WASM-specific Rust code
#[cfg(feature = "wasm")]
pub mod wasm {

     // Use this module to specify everything that is WASM-specific (e.g. uses wasm-bindgen types, js_sys, ...etc.)

    // use super::*;
    // use wasm_bindgen::prelude::*;
    // use wasm_bindgen::JsValue;

    // #[wasm_bindgen]
    // pub fn foo(_val: JsValue) -> u32 {
    //     super::foo()
    // }
    //
    // #[wasm_bindgen]
    // impl MyStruct {
    //     // Specify methods of MyStruct which use WASM-specific
    // }

}

