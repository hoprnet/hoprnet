//! This Rust crate contains all the path construction and path selection algorithms in the HOPR mixnet.

/// Defines the graph of HOPR payment channels.
pub mod channel_graph;
pub mod errors;
/// Implements different path selectors in the [ChannelGraph](crate::channel_graph::ChannelGraph).
pub mod selectors;

use hopr_crypto_types::types::OffchainPublicKey;
use hopr_internal_types::channels::ChannelStatus;
use hopr_primitive_types::prelude::{Address, ToHex};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, RandomState};
use std::ops::Deref;

use crate::channel_graph::ChannelGraph;
use crate::errors::PathError;
use crate::errors::PathError::{ChannelNotOpened, InvalidPeer, LoopsNotAllowed, MissingChannel, PathNotValid};

/// Base implementation of an abstract path.
///
/// Must contain always at least a single entry.
pub trait Path<N>: Clone + Eq + PartialEq + Deref<Target = [N]>
where
    N: Copy + Eq + PartialEq + Hash,
{
    /// Individual hops in the path.
    /// There must be always at least one hop.
    fn hops(&self) -> &[N] {
        self.deref()
    }

    /// Shorthand for the number of hops.
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

    /// Returns the path with the hops in reverse order if it is possible.
    fn invert(self) -> Option<Self>;
}

impl<T: Copy + Eq + PartialEq + Hash> Path<T> for Vec<T> {
    fn invert(self) -> Option<Self> {
        Some(self.into_iter().rev().collect())
    }
}

pub type ChannelPath = Vec<Address>;

pub type TransportPath = Vec<OffchainPublicKey>;

#[async_trait::async_trait]
pub trait TransportKeyResolver {
    async fn resolve_transport_key(&self, address: &Address) -> Result<Option<OffchainPublicKey>, PathError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullPath(Vec<Address>);

impl FullPath {
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

    pub fn from_channel_path(mut path: ChannelPath, destination: Address) -> Self {
        path.push(destination);
        Self(path)
    }

    pub fn direct(destination: Address) -> Self {
        Self(vec![destination])
    }

    /// Turns this transport path into a [`ValidatedPath`], checking
    /// that all addresses and channels on the path exist and resolving the corresponding [`OffchainPublicKeys`](OffchainPublicKey).
    pub async fn validate<R: TransportKeyResolver>(
        self,
        cg: &ChannelGraph,
        resolver: &R,
    ) -> errors::Result<ValidatedPath> {
        let mut ticket_receiver;
        let mut ticket_issuer = cg.my_address();

        let mut keys = Vec::with_capacity(self.0.len());

        for (i, hop) in self.0.iter().enumerate() {
            ticket_receiver = *hop;

            // Check for loops
            if ticket_issuer == ticket_receiver {
                return Err(LoopsNotAllowed(ticket_receiver.to_hex()));
            }

            // Check if the channel is opened, if not the last hop
            if i < self.0.len() - 1 {
                let channel = cg
                    .get_channel(&ticket_issuer, &ticket_receiver)
                    .ok_or(MissingChannel(ticket_issuer.to_hex(), ticket_receiver.to_hex()))?;

                if channel.status != ChannelStatus::Open {
                    return Err(ChannelNotOpened(ticket_issuer.to_hex(), ticket_receiver.to_hex()));
                }
            }

            ticket_issuer = ticket_receiver;

            keys.push(
                resolver
                    .resolve_transport_key(&hop)
                    .await?
                    .ok_or(InvalidPeer(hop.to_hex()))?,
            );
        }

        Ok(ValidatedPath(keys, self))
    }
}

impl Deref for FullPath {
    type Target = [Address];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for FullPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "full path [{}]",
            self.0.iter().map(|p| p.to_hex()).collect::<Vec<String>>().join(", ")
        )
    }
}

impl From<FullPath> for ChannelPath {
    fn from(value: FullPath) -> Self {
        let len = value.0.len();
        value.0.into_iter().take(len - 1).collect()
    }
}

impl Path<Address> for FullPath {
    fn invert(self) -> Option<Self> {
        Some(Self(self.0.into_iter().rev().collect()))
    }
}

/// Represents a [`TransportPath`] that has been resolved and [validated](TransportPath::validate).
///
/// Such a path can be directly used to deliver packets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedPath(Vec<OffchainPublicKey>, FullPath);

impl ValidatedPath {
    pub fn full_path(&self) -> &FullPath {
        &self.1
    }

    pub fn length(&self) -> usize {
        debug_assert_eq!(self.0.len(), self.1 .0.len(), "validated path must have equal lengths");
        self.0.len()
    }
}

impl Deref for ValidatedPath {
    type Target = [OffchainPublicKey];

    fn deref(&self) -> &Self::Target {
        &self.0
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
            "validated full path [{}]",
            self.1 .0.iter().map(|p| p.to_hex()).collect::<Vec<String>>().join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    //use super::*;
}