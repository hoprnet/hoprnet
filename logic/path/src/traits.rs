use serde::{Deserialize, Serialize};
use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::fmt::Display;
use std::hash::Hash;

use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

/// Base implementation of an abstract path.
/// Must contain always at least a single entry.
pub trait Path<N>: Display + Clone + Eq + PartialEq
where
    N: Copy + Eq + PartialEq + Hash,
{
    /// Individual hops in the path.
    /// There must be always at least one hop.
    fn hops(&self) -> &[N];

    /// Shorthand for number of hops.
    fn length(&self) -> usize {
        self.hops().len()
    }

    /// Gets the last hop
    fn last_hop(&self) -> &N {
        // Path must contain at least one hop
        self.hops().last().expect("path is invalid")
    }

    /// Checks if all the hops in this path are to distinct addresses.
    ///
    /// Returns `true` if there are duplicate Addresses on this path.
    /// Note that the duplicate Addresses can never be adjacent.
    fn contains_cycle(&self) -> bool {
        let set = HashSet::<&N, RandomState>::from_iter(self.hops().iter());
        set.len() != self.hops().len()
    }
}

/// Structure that adds additional data `Q` to a [ChannelEntry], which
/// can be used to compute edge weights and traverse a [ChannelQualityGraph].
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChannelEdge<Q> {
    /// Underlying channel
    pub channel: ChannelEntry,
    /// Network quality of this channel at the transport level (if any).
    /// This value is currently present only for *own channels*.
    pub quality: Option<Q>,
}

/// Abstraction over a directed graph of [channels](ChannelEntry) with an
/// additional independent transport information (e.g. quality) `Q` on each [edge](ChannelEdge).
///
/// The nodes of the graph are represented by [Addresses](Address), the edges
/// hold additional information [ChannelEntry] and `Q` (via [ChannelEdge]).
///
/// The underlying idea is that the [ChannelEntry] and quality information `Q`
/// can be updated into the graph independently of each other.
pub trait ChannelQualityGraph<Q> {
    /// Looks up a channel given the source and destination.
    ///
    /// Returns `None` if no such edge exists in the graph.
    fn get_channel(&self, source: Address, destination: Address) -> Option<&ChannelEdge<Q>>;

    /// Gets all `Open` outgoing channels going to (if `direction` is `Incoming`) or
    /// from (if `direction` is `Outgoing`) the given `target` [Address]
    fn opened_channels_at<'a>(
        &'a self,
        target: Address,
        direction: ChannelDirection,
    ) -> impl Iterator<Item = (Address, Address, &'a ChannelEdge<Q>)>
    where
        Q: 'a;

    /// Checks whether there's any path via Open channels that connects `source` and `destination`
    /// This does not need to be necessarily a multi-hop path.
    fn has_path(&self, source: Address, destination: Address) -> bool;

    /// Checks whether the given channel from `source` to `destination` is in the graph already.
    fn contains_channel(&self, source: Address, destination: Address) -> bool;

    /// Inserts or updates the given channel in the channel graph with the optional
    /// quality information.
    ///
    /// Returns a set of changes if the channel was already present in the graphs or
    /// None if the channel was not previously present in the channel graph.
    fn upsert_channel(&mut self, channel: ChannelEntry, quality: Option<Q>) -> Option<Vec<ChannelChange>>;

    /// Updates the quality value of network connection between `source` and `destination`, independently
    /// of the [ChannelEntry] on that edge.
    fn set_channel_quality(&mut self, source: Address, destination: Address, quality: Q);
}
