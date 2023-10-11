use crate::errors::Result;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::channels::{ChannelChange, ChannelEntry, ChannelStatus};
use petgraph::algo::has_path_connecting;
use petgraph::graphmap::DiGraphMap;
use petgraph::visit::{EdgeFiltered, EdgeRef};
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use utils_log::info;
use utils_types::primitives::Address;

/// Structure that adds additional data to a `ChannelEntry`, which
/// can be used to compute edge weights and traverse the `ChannelGraph`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChannelEdge {
    /// Underlying channel
    pub channel: ChannelEntry,
    /// Network quality of this channel at the transport level (if any).
    /// This value is currently present only for *own channels*.
    pub quality: Option<f64>,
}

/// Implements a HOPR payment channel graph cached in-memory.
/// This structure is useful for tracking channel state changes and
/// packet path finding.
/// The structure is updated only from the Indexer and therefore contains only
/// the channels that were *seen* on-chain. The network qualities are also
/// added to the graph on the fly.
/// Using this structure is much faster than querying the DB and therefore
/// is preferred for per-packet path-finding computations.
/// Per default, the graph does not track channels in `Closed` state, and therefore
/// cannot detect channel re-openings.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelGraph {
    me: Address,
    graph: DiGraphMap<Address, ChannelEdge>,
}

impl ChannelGraph {
    /// Maximum number of intermediate hops the automatic path finding algorithm will look for.
    pub const INTERMEDIATE_HOPS: usize = 3;

    /// Creates a new instance with the given self `Address`.
    pub fn new(me: Address) -> Self {
        Self {
            me,
            graph: DiGraphMap::default(),
        }
    }

    /// Checks if the channel is incoming to or outgoing from this node
    pub fn is_own_channel(&self, channel: &ChannelEntry) -> bool {
        channel.destination == self.me || channel.source == self.me
    }

    /// Convenience method to get this node's own address
    pub fn my_address(&self) -> Address {
        self.me
    }

    /// Looks up an `Open` or `PendingToClose' channel given the source and destination.
    /// Returns `None` if no such edge exists in the graph.
    pub fn get_channel(&self, source: Address, destination: Address) -> Option<&ChannelEntry> {
        self.graph.edge_weight(source, destination).map(|w| &w.channel)
    }

    /// Gets all `Open` outgoing channels going from the given `Address`
    pub fn open_channels_from(&self, source: Address) -> impl Iterator<Item = (Address, Address, &ChannelEdge)> {
        self.graph
            .edges_directed(source, Direction::Outgoing)
            .filter(|c| c.weight().channel.status == ChannelStatus::Open)
    }

    /// Checks whether there's any path via Open channels that connects `source` and `destination`
    /// This does not need to be necessarily a multi-hop path.
    pub fn has_path(&self, source: Address, destination: Address) -> bool {
        let only_open_graph = EdgeFiltered::from_fn(&self.graph, |e| e.weight().channel.status == ChannelStatus::Open);
        has_path_connecting(&only_open_graph, source, destination, None)
    }

    /// Inserts or updates the given channel in the channel graph.
    /// Returns a set of changes if the channel was already present in the graphs or
    /// None if the channel was not previously present in the channel graph.
    pub fn update_channel(&mut self, channel: ChannelEntry) -> Option<Vec<ChannelChange>> {
        // Remove the edge since we don't allow Closed channels
        if channel.status == ChannelStatus::Closed {
            let ret = if let Some(old_value) = self.graph.remove_edge(channel.source, channel.destination) {
                Some(ChannelChange::diff_channels(&old_value.channel, &channel))
            } else {
                None
            };

            info!("removed {channel}");
            return ret;
        }

        if let Some(old_value) = self.graph.edge_weight_mut(channel.source, channel.destination) {
            let old_channel = old_value.channel;
            old_value.channel = channel;

            let ret = ChannelChange::diff_channels(&old_channel, &channel);
            info!("updated {channel}: {} changes", ret.len());
            Some(ret)
        } else {
            let weighted = ChannelEdge { channel, quality: None };
            self.graph.add_edge(channel.source, channel.destination, weighted);
            info!("new {channel}");

            None
        }
    }

    /// Updates the quality value of network connection between `source` and `destination`
    pub fn update_channel_quality(&mut self, source: Address, destination: Address, quality: f64) {
        if let Some(channel) = self.graph.edge_weight_mut(source, destination) {
            channel.quality = Some(quality);
        }
    }

    /// Synchronizes the channel entries in this graph with the database.
    /// The synchronization is one-way from DB to the graph, not vice versa.
    pub async fn sync_channels<Db: HoprCoreEthereumDbActions>(&mut self, db: &Db) -> Result<()> {
        db.get_channels().await?.into_iter().for_each(|c| {
            self.update_channel(c);
        });
        Ok(())
    }

    /// Checks whether the given channel is in the graph already.
    pub fn contains_channel(&self, channel: &ChannelEntry) -> bool {
        self.graph.contains_edge(channel.source, channel.destination)
    }
}

#[cfg(test)]
mod tests {}

#[cfg(feature = "wasm")]
pub mod wasm {
    use async_std::sync::RwLock;
    use std::sync::Arc;
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    #[derive(Clone)]
    pub struct ChannelGraph {
        w: Arc<RwLock<super::ChannelGraph>>,
    }

    impl ChannelGraph {
        pub fn new(w: Arc<RwLock<super::ChannelGraph>>) -> Self {
            Self { w }
        }

        pub fn as_ref_counted(&self) -> Arc<RwLock<super::ChannelGraph>> {
            self.w.clone()
        }
    }
}
