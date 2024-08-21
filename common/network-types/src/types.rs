use arrayvec::ArrayVec;
use hopr_crypto_types::prelude::PeerId;

/// Represents routing options in HOPR mixnet.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RoutingOptions {
    /// A fixed intermediate path consisting of at most three hops.
    IntermediatePath(ArrayVec<PeerId, 3>),
    /// Random intermediate path with at least the given number of hops.
    Hops(u8),
}
