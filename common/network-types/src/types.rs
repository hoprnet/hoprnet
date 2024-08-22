use hopr_crypto_types::prelude::PeerId;
use std::fmt::{Display, Formatter};

pub use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum IpProtocol {
    TCP,
    UDP,
}

// TODO: move this to hopr-primitive-types
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BoundedU8<const B: u8>(u8);

impl<const B: u8> TryFrom<u8> for BoundedU8<B> {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= B {
            Ok(Self(value))
        } else {
            Err(())
        }
    }
}

impl<const B: u8> From<BoundedU8<B>> for u8 {
    fn from(value: BoundedU8<B>) -> Self {
        value.0
    }
}

impl<const B: u8> From<BoundedU8<B>> for usize {
    fn from(value: BoundedU8<B>) -> Self {
        value.0 as usize
    }
}

impl<const B: u8> Display for BoundedU8<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents routing options in a mixnet with a maximum number of hops.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RoutingOptions {
    /// A fixed intermediate path consisting of at most [`RoutingOptions::MAX_INTERMEDIATE_HOPS`] hops.
    IntermediatePath(Box<ArrayVec<PeerId, { RoutingOptions::MAX_INTERMEDIATE_HOPS }>>),
    /// Random intermediate path with at least the given number of hops,
    /// but at most [`RoutingOptions::MAX_INTERMEDIATE_HOPS`].
    Hops(BoundedU8<{ RoutingOptions::MAX_INTERMEDIATE_HOPS as u8 }>),
}

impl RoutingOptions {
    /// The maximum number of hops this instance can represent.
    pub const MAX_INTERMEDIATE_HOPS: usize = 3;

    /// Inverts the intermediate path if this is an instance of [`RoutingOptions::IntermediatePath`].
    /// Otherwise, does nothing.
    pub fn invert(self) -> RoutingOptions {
        match self {
            RoutingOptions::IntermediatePath(mut v) => {
                v.reverse();
                RoutingOptions::IntermediatePath(v)
            }
            _ => self,
        }
    }
}
