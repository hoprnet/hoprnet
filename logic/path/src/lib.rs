//! This Rust crate contains all the path construction and path selection algorithms in the HOPR mixnet.

/// Defines the graph of HOPR payment channels.
pub mod channel_graph;
pub mod errors;
/// Implements different path selectors in the [ChannelGraph](crate::channel_graph::ChannelGraph).
pub mod selectors;

use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::ops::Deref;

use crate::channel_graph::ChannelGraph;
use crate::errors::PathError;
use crate::errors::PathError::{ChannelNotOpened, InvalidPeer, LoopsNotAllowed, MissingChannel, PathNotValid};

/// Represents a type that determines a hop on a [`Path`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, strum::EnumTryAs, strum::EnumIs)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PathAddress {
    Chain(Address),
    Transport(OffchainPublicKey),
}

impl From<Address> for PathAddress {
    fn from(value: Address) -> Self {
        PathAddress::Chain(value)
    }
}

impl From<OffchainPublicKey> for PathAddress {
    fn from(value: OffchainPublicKey) -> Self {
        PathAddress::Transport(value)
    }
}

impl Display for PathAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PathAddress::Chain(addr) => write!(f, "{}", addr.to_hex()),
            PathAddress::Transport(key) => write!(f, "{}", key.to_hex()),
        }
    }
}

/// Base implementation of an abstract path.
pub trait Path<N: Into<PathAddress>>: Clone + Eq + PartialEq + Deref<Target = [N]> + IntoIterator<Item = N> {
    /// Individual hops in the path.
    /// There must be always at least one hop.
    fn hops(&self) -> &[N] {
        self.deref()
    }

    /// Shorthand for the number of hops.
    fn num_hops(&self) -> usize {
        self.hops().len()
    }

    /// Returns the path with the hops in reverse order if it is possible.
    fn invert(self) -> Option<Self>;
}

/// A [`Path`] that is guaranteed to have at least one hop.
pub trait NonEmptyPath<N: Into<PathAddress>>: Path<N> {
    /// Gets the last hop
    fn last_hop(&self) -> &N {
        self.hops().last().expect("non-empty path must have at least one hop")
    }
}

impl<T: Into<PathAddress> + Clone + PartialEq + Eq> Path<T> for Vec<T> {
    fn invert(self) -> Option<Self> {
        Some(self.into_iter().rev().collect())
    }
}

pub type ChannelPath = Vec<Address>;

/// A [`NonEmptyPath`] that can be used to route packets using [`OffchainPublicKeys`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransportPath(Vec<OffchainPublicKey>);

impl TransportPath {
    /// Creates a new instance from the given iterator.
    ///
    /// Fails if the iterator is empty.
    pub fn new<T, I>(path: I) -> errors::Result<Self>
    where
        T: Into<OffchainPublicKey>,
        I: IntoIterator<Item = T>,
    {
        let hops = path.into_iter().map(|t| t.into()).collect::<Vec<_>>();
        if !hops.is_empty() {
            Ok(Self(hops))
        } else {
            Err(PathNotValid)
        }
    }

    /// Creates a direct path just to the `destination`.
    pub fn direct(destination: OffchainPublicKey) -> Self {
        Self(vec![destination])
    }
}

impl Deref for TransportPath {
    type Target = [OffchainPublicKey];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for TransportPath {
    type Item = OffchainPublicKey;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Path<OffchainPublicKey> for TransportPath {
    fn invert(self) -> Option<Self> {
        Some(Self(self.0.into_iter().rev().collect()))
    }
}

impl NonEmptyPath<OffchainPublicKey> for TransportPath {}

/// Represents a [`NonEmptyPath`] that completely specifies a route using [`Addresses`](Address).
///
/// Transport cannot directly use this to deliver packets.
///
/// Note that this is different from [`ChannelPath`], which can be empty and does not contain
/// the address of the destination.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChainPath(Vec<Address>);

impl ChainPath {
    /// Creates a new instance from the given iterator.
    ///
    /// Fails if the iterator is empty.
    pub fn new<T, I>(path: I) -> errors::Result<Self>
    where
        T: Into<Address>,
        I: IntoIterator<Item = T>,
    {
        let hops = path.into_iter().map(|t| t.into()).collect::<Vec<_>>();
        if !hops.is_empty() {
            Ok(Self(hops))
        } else {
            Err(PathNotValid)
        }
    }

    /// Creates a path using the given [`ChannelPath`] (possibly empty) and the given `destination` address.
    pub fn from_channel_path(mut path: ChannelPath, destination: Address) -> Self {
        path.push(destination);
        Self(path)
    }

    /// Creates a direct path just to the `destination`.
    pub fn direct(destination: Address) -> Self {
        Self(vec![destination])
    }

    /// Converts this chain path into the [`ChainPath`] by removing the destination.
    pub fn into_channel_path(mut self) -> ChannelPath {
        self.0.pop();
        self.0
    }
}

impl Deref for ChainPath {
    type Target = [Address];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for ChainPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "chain path [{}]",
            self.0.iter().map(|p| p.to_hex()).collect::<Vec<String>>().join(", ")
        )
    }
}

impl From<ChainPath> for ChannelPath {
    fn from(value: ChainPath) -> Self {
        let len = value.0.len();
        value.0.into_iter().take(len - 1).collect()
    }
}

impl IntoIterator for ChainPath {
    type Item = Address;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Path<Address> for ChainPath {
    fn invert(self) -> Option<Self> {
        Some(Self(self.0.into_iter().rev().collect()))
    }
}

impl NonEmptyPath<Address> for ChainPath {}

/// Allows resolution of [`OffchainPublicKeys`] for a given [`Address`] or vice versa.
#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait PathAddressResolver {
    async fn resolve_transport_address(&self, address: &Address) -> Result<Option<OffchainPublicKey>, PathError>;
    async fn resolve_chain_address(&self, key: &OffchainPublicKey) -> Result<Option<Address>, PathError>;
}

/// Represents [`NonEmptyPath`] that has been resolved and validated.
///
/// Such a path can be directly used to deliver packets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedPath(TransportPath, ChainPath);

impl ValidatedPath {
    /// Shortcut to create a direct path to a destination with the given addresses.
    pub fn direct(dst_key: OffchainPublicKey, dst_address: Address) -> Self {
        Self(TransportPath(vec![dst_key]), ChainPath(vec![dst_address]))
    }

    /// Turns the given path into a [`ValidatedPath`].
    ///
    /// This makes sure that all addresses and channels on the path exist
    /// and do resolve to the corresponding [`OffchainPublicKeys`](OffchainPublicKey) or
    /// [`Addresses`](Address).
    pub async fn new<N, P, R>(path: P, cg: &ChannelGraph, resolver: &R) -> errors::Result<ValidatedPath>
    where
        N: Into<PathAddress> + Copy,
        P: NonEmptyPath<N>,
        R: PathAddressResolver,
    {
        let mut ticket_issuer = cg.my_address();
        let mut keys = Vec::with_capacity(path.num_hops());
        let mut addrs = Vec::with_capacity(path.num_hops());

        let num_hops = path.num_hops();
        for (i, hop) in path.into_iter().enumerate() {
            // Resolve the counterpart address
            // and get the chain Address to validate against the channel graph
            let ticket_receiver = match &hop.into() {
                PathAddress::Chain(addr) => {
                    let key = resolver
                        .resolve_transport_address(addr)
                        .await?
                        .ok_or(InvalidPeer(addr.to_string()))?;
                    keys.push(key);
                    addrs.push(*addr);
                    *addr
                }
                PathAddress::Transport(key) => {
                    let addr = resolver
                        .resolve_chain_address(key)
                        .await?
                        .ok_or(InvalidPeer(key.to_string()))?;
                    addrs.push(addr);
                    keys.push(*key);
                    addr
                }
            };

            // Check for loops
            if ticket_issuer == ticket_receiver {
                return Err(LoopsNotAllowed(ticket_receiver.to_hex()));
            }

            // Check if the channel is opened, if not the last hop
            if i < num_hops - 1 {
                let channel = cg
                    .get_channel(&ticket_issuer, &ticket_receiver)
                    .ok_or(MissingChannel(ticket_issuer.to_hex(), ticket_receiver.to_hex()))?;

                if channel.status != ChannelStatus::Open {
                    return Err(ChannelNotOpened(ticket_issuer.to_hex(), ticket_receiver.to_hex()));
                }
            }

            ticket_issuer = ticket_receiver;
        }

        Ok(ValidatedPath(TransportPath(keys), ChainPath(addrs)))
    }

    /// Valid chain path.
    pub fn chain_path(&self) -> &ChainPath {
        &self.1
    }

    /// Valid transport path.
    pub fn transport_path(&self) -> &TransportPath {
        &self.0
    }
}

impl Deref for ValidatedPath {
    type Target = [OffchainPublicKey];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for ValidatedPath {
    type Item = OffchainPublicKey;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Path<OffchainPublicKey> for ValidatedPath {
    /// Returns always `None`.
    ///
    /// A validated path cannot be inverted, as the inverted path could be invalid.
    fn invert(self) -> Option<Self> {
        None
    }
}

impl Display for ValidatedPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "validated path [{}]",
            self.1 .0.iter().map(|p| p.to_hex()).collect::<Vec<String>>().join(", ")
        )
    }
}

impl NonEmptyPath<OffchainPublicKey> for ValidatedPath {}

#[cfg(test)]
mod tests {
    //use super::*;
}
