//! This Rust crate contains all the path construction and path selection algorithms in the HOPR mixnet.

/// Defines the graph of HOPR payment channels.
pub mod channel_graph;
pub mod errors;
/// Defines the two most important types: [TransportPath](crate::path::TransportPath) and [ChannelPath](crate::path::ChannelPath).
mod path;
/// Implements different path selectors in the [ChannelGraph](crate::channel_graph::ChannelGraph).
pub mod selectors;

use std::collections::HashSet;
use std::fmt::Display;
use std::hash::{Hash, RandomState};
use libp2p_identity::PeerId;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_primitive_types::prelude::{Address, GeneralError};

/// Base implementation of an abstract path.
///
/// Must contain always at least a single entry.
pub trait Path<N>: Clone + Eq + PartialEq
where
    N: Copy + Eq + PartialEq + Hash,
{
    /// Creates the instance from an iterable list of hops.
    ///
    /// Returns an error if the size of the list is invalid for the given path type.
    fn from_hops<T: IntoIterator<Item = N>>(hops: T) -> errors::Result<Self>;

    /// Individual hops in the path.
    /// There must be always at least one hop.
    fn hops(&self) -> &[N];

    /// Shorthand for number of hops.
    fn length(&self) -> usize {
        self.hops().len()
    }

    /// Gets the last hop
    fn last_hop(&self) -> Option<&N> {
        self.hops().last()
    }

    /// Checks if all the hops in this path are to distinct addresses.
    ///
    /// Returns `true` if there are duplicate Addresses on this path.
    /// Note that the duplicate Addresses can never be adjacent.
    fn contains_cycle(&self) -> bool {
        let set = HashSet::<&N, RandomState>::from_iter(self.hops().iter());
        set.len() != self.hops().len()
    }

    /// Returns the path with the hops in reverse order.
    fn invert(self) -> Self {
        Self::from_hops(self.hops().iter().rev().cloned())
            .expect("reversing path must not fail")
    }
}

impl Path<Address> for Vec<Address> {
    fn from_hops<T: IntoIterator<Item = Address>>(hops: T) -> errors::Result<Self> {
        Ok(hops.into_iter().collect())
    }

    fn hops(&self) -> &[Address] {
        &self
    }
}

pub type ChannelPath = Vec<Address>;

#[derive(Clone, PartialEq, Eq)]
pub struct TransportPath {
    path: Vec<OffchainPublicKey>,
    peers: Vec<PeerId>,
    channels: Option<ChannelPath>
}

impl Path<PeerId> for TransportPath {
    fn from_hops<T: IntoIterator<Item=PeerId>>(hops: T) -> errors::Result<Self> {
        let peers: Vec<PeerId> = hops.into_iter().collect();
        if !peers.is_empty() {
            let path = peers.iter()
                .map(|p| OffchainPublicKey::try_from(*p))
                .collect::<Result<Vec<_>, GeneralError>>()?;
            Ok(Self {
                path,
                peers,
                channels: None
            })
        } else {
            Err(errors::PathError::PathNotValid)
        }
    }

    fn hops(&self) -> &[PeerId] {
        &self.peers
    }
}

impl TransportPath {

}
