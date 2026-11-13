//! The crate provides the probing functionality used by the transport layer
//! to identify different attributes of possible transport paths in the network.
//!
//! The goal of probing is to establish a map with weighted properties that will
//! allow the caller to optimize transport, verify transport path properties and
//! lay groundworks for mitigating potential adversarial behavior.
//!
//! There are 2 fundamental types of probing:
//! 1. **Immediate hop probing** - collects telemetry for direct 0-hop neighbors. Such telemetry can be identified and
//!    potentially gamed by an adversary, but it is still useful to identify the basic properties of the most immediate
//!    connection to the neighbor, since in the worst case scenario the mitigation strategy can discard unsuitable
//!    peers.
//!
//! 2. **Multi-hop probing** - collects telemetry using a probing mechanism based on looping. A loop is a message sent
//!    by this peer to itself through different pre-selected peers. This probing mechanism can be combined together with
//!    the cover traffic into a single mechanism improving the network view.
//!
//!
//! Probing is fully configurable mechanism and **MAY** be used as a dispersion for the `Cover Traffic`.

pub mod config;
pub mod content;
pub mod errors;
pub mod ping;
pub mod probe;
pub mod types;

pub use hopr_api::{
    ct::ProbingTrafficGeneration,
    graph::{EdgeTransportTelemetry, NetworkGraphError},
};

pub use crate::{
    config::ProbeConfig,
    content::Message as TrafficReturnedObservation,
    probe::{Probe, ProbeClassifierState, ProbeDispatch},
    types::{NeighborTelemetry, PathTelemetry},
};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, strum::Display)]
pub enum HoprProbeProcess {
    #[strum(to_string = "probe emission")]
    Emit,
    /// Records loopback probe timeouts on the graph.
    ///
    /// The tag cache's eviction listener hands expired probes here rather than recording them
    /// inline, so each can wait out the probe's local-drop grace before it counts. A probe a drop
    /// site reported in that window is suppressed, and a batch that expires together is recorded
    /// after one shared wait — which is why a genuine timeout reaches the graph a few seconds after
    /// its tag expired.
    #[strum(to_string = "probe path verdicts")]
    PathVerdicts,
}
