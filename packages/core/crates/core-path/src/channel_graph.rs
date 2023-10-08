use crate::channel_graph::ChannelChange::{CurrentBalance, Epoch, Status, TicketIndex};
use crate::errors::Result;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::channels::{ChannelEntry, ChannelStatus};
use petgraph::graphmap::DiGraphMap;
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use utils_log::{debug, info};
use utils_types::primitives::{Address, Balance};

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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelGraph {
    me: Address,
    graph: DiGraphMap<Address, ChannelEdge>,
}

/// Enumerates possible changes on a channel entry update
#[derive(Clone, Copy, Debug)]
pub enum ChannelChange {
    /// Channel status has changed
    Status { old: ChannelStatus, new: ChannelStatus },

    /// Channel balance has changed
    CurrentBalance { old: Balance, new: Balance },

    /// Channel epoch has changed
    Epoch { old: u32, new: u32 },

    /// Ticket index has changed
    TicketIndex { old: u64, new: u64 },
}

impl Display for ChannelChange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Status { old, new } => {
                write!(f, "Status: {old} -> {new}")
            }

            CurrentBalance { old, new } => {
                write!(f, "Balance: {old} -> {new}")
            }

            Epoch { old, new } => {
                write!(f, "Epoch: {old} -> {new}")
            }

            TicketIndex { old, new } => {
                write!(f, "TicketIndex: {old} -> {new}")
            }
        }
    }
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

    /// Looks up the channel given the source and destination.
    /// Returns `None` if no such edge exists in the graph.
    pub fn get_channel(&self, src: Address, dst: Address) -> Option<&ChannelEntry> {
        self.graph.edge_weight(src, dst).map(|w| &w.channel)
    }

    /// Gets all channels going from (outgoing) the given `Address`
    pub fn channels_from(&self, src: Address) -> impl Iterator<Item = (Address, Address, &ChannelEdge)> {
        self.graph.edges_directed(src, Direction::Outgoing)
    }

    /// Inserts or updates the given channel in the channel graph.
    /// Returns a set of changes if the channel was already present in the graphs or
    /// None if the channel was not previously present in the channel graph.
    pub fn update_channel(&mut self, channel: ChannelEntry) -> Option<Vec<ChannelChange>> {
        if let Some(old_w_value) = self.graph.edge_weight_mut(channel.source, channel.destination) {
            let old_channel = old_w_value.channel;
            old_w_value.channel = channel;

            let mut ret = Vec::new();

            if old_channel.status != channel.status {
                ret.push(Status {
                    old: old_channel.status,
                    new: channel.status,
                });
            }

            if old_channel.balance != channel.balance {
                ret.push(CurrentBalance {
                    old: old_channel.balance,
                    new: channel.balance,
                });
            }

            if old_channel.channel_epoch != channel.channel_epoch {
                ret.push(Epoch {
                    old: old_channel.channel_epoch.as_u32(),
                    new: channel.channel_epoch.as_u32(),
                });
            }

            if old_channel.ticket_index != channel.ticket_index {
                ret.push(TicketIndex {
                    old: old_channel.ticket_index.as_u64(),
                    new: channel.ticket_index.as_u64(),
                })
            }
            debug!(
                "{channel} (own = {}) update changes: {:?}",
                self.is_own_channel(&channel),
                ret
            );

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
