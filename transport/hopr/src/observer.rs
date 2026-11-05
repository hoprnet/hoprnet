//! Events the transport reports to an optional observer, such as the node's periodic state dump.
//!
//! The transport only reports what happened. It does not know who listens or what is done with the
//! events, so it needs no dependency on the code that summarizes them.
use std::sync::Arc;

use hopr_api::{OffchainPublicKey, graph::ForwardAndReturnPath, types::internal::NodeId};
pub use hopr_transport_probe::ProbeOutcome;

/// Whether a path carries packets to a destination, or replies back from it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathDirection {
    Forward,
    Return,
}

/// How many candidates one search phase returned, and how many of them were usable.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PhaseCounts {
    pub returned: u64,
    pub eligible: u64,
}

/// One thing the transport observed.
#[derive(Clone, Debug)]
pub enum TransportEvent {
    /// A path resolve finished, including one served from the cache.
    PathResolved {
        direction: PathDirection,
        destination: NodeId,
        succeeded: bool,
    },
    /// The selector searched the graph. `phase2` is `None` when the second phase did not run.
    PathSearched {
        direction: PathDirection,
        destination: NodeId,
        phase1: PhaseCounts,
        phase2: Option<PhaseCounts>,
        retained: u64,
    },
    /// SURBs minted for, or returned from, one destination over one pair of paths.
    Surb {
        destination: OffchainPublicKey,
        paths: ForwardAndReturnPath,
        minted: u64,
        returned: u64,
    },
    /// A neighbor or loopback probe finished.
    Probe(ProbeOutcome),
}

/// Receives [`TransportEvent`]s. It is called on the packet path, so it must be cheap and must not
/// block.
pub type TransportObserver = Arc<dyn Fn(TransportEvent) + Send + Sync>;
