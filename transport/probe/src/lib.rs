//! The crate provides the probing functionality used by the transport layer
//! to identify different attributes of possible transport paths in the network.
//!
//! The goal of probing is to establish a map with weighted properties that will
//! allow the caller to optimize transport, verify transport path properties and
//! lay groundworks for mitigating potential adversarial behavior.
//!
//! There are 2 fundamental types of probing:
//! 1. Immediate hop probing - collects telemetry for direct 0-hop neighbors. Such telemetry
//! can be identified and potentially gamed by an adversary, but it is still useful to identify
//! the basic properties of the most immediate connection to the neighbor, since in the worst
//! case scenario the mitigation strategy can discard unsuitable peers.
//!
//! 2. Multi-hop probing - collects telemetry using a probing mechanism based on looping. A loop
//! is a message sent by this peer to itself through different pre-selected peers. This probing
//! mechanism can be combined together with the cover traffic into a single mechanism improving
//! the network view.

pub mod constants;
pub mod errors;
pub mod heartbeat;
pub mod messaging;
pub mod ping;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, strum::Display)]
pub enum HoprProbeProcess {
    #[strum(to_string = "heartbeat")]
    Heartbeat,
}
