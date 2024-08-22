use hopr_crypto_types::prelude::PeerId;

use hopr_primitive_types::bounded::{BoundedSize, BoundedVec};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum IpProtocol {
    TCP,
    UDP,
}

/// Represents routing options in a mixnet with a maximum number of hops.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RoutingOptions {
    /// A fixed intermediate path consisting of at most [`RoutingOptions::MAX_INTERMEDIATE_HOPS`] hops.
    IntermediatePath(BoundedVec<PeerId, { RoutingOptions::MAX_INTERMEDIATE_HOPS }>),
    /// Random intermediate path with at least the given number of hops,
    /// but at most [`RoutingOptions::MAX_INTERMEDIATE_HOPS`].
    Hops(BoundedSize<{ RoutingOptions::MAX_INTERMEDIATE_HOPS }>),
}

impl RoutingOptions {
    /// The maximum number of hops this instance can represent.
    pub const MAX_INTERMEDIATE_HOPS: usize = 3;

    /// Inverts the intermediate path if this is an instance of [`RoutingOptions::IntermediatePath`].
    /// Otherwise, does nothing.
    pub fn invert(self) -> RoutingOptions {
        match self {
            RoutingOptions::IntermediatePath(v) => RoutingOptions::IntermediatePath(v.into_iter().rev().collect()),
            _ => self,
        }
    }
}
