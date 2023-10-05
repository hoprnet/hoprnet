use crate::channel_graph::ChannelChange::{CurrentBalance, Epoch, Status};
use crate::errors::Result;
use crate::path::Path;
use core_types::channels::{ChannelEntry, ChannelStatus};
use core_types::protocol::INTERMEDIATE_HOPS;
use petgraph::graphmap::DiGraphMap;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use utils_log::{debug, info};
use utils_types::primitives::{Address, Balance};

/// Internal structure that adds additional data to a `ChannelEntry` that
/// can be used to compute edge weights.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
struct WeightedChannel {
    channel: ChannelEntry,
    weight: f64
}

/// Implements a HOPR payment channel graph cached in-memory.
/// This structure is useful for tracking channel state changes and
/// packet path finding.
/// The structure is updated only from the Indexer and therefore contains only
/// the channels that were *seen* on-chain.
/// Using this structure is generally faster than querying the DB and therefore
/// is preferred for per-packet path-finding computations.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelGraph {
    me: Address,
    graph: DiGraphMap<Address, WeightedChannel>,
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

    /// Inserts or updates the given channel in the channel graph.
    /// Returns a set of changes if the channel was already present in the graphs or
    /// None if the channel was not previously present in the channel graph.
    pub fn update_channel(&mut self, channel: ChannelEntry) -> Option<Vec<ChannelChange>> {
        let weighted = WeightedChannel {
            channel, weight: 1_f64 // TODO: compute weight properly
        };

        if let Some(old_w_value) = self.graph.add_edge(channel.source, channel.destination, weighted) {
            let mut ret = Vec::new();
            let old_channel = old_w_value.channel;

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
            debug!("channel update changes: {:?}", ret);

            info!("updated {channel}: {} changes", ret.len());
            Some(ret)
        } else {
            info!("new {channel}");
            None
        }
    }

    /// Checks whether the given channel is in the graph already.
    pub fn contains_channel(&self, channel: &ChannelEntry) -> bool {
        self.graph.contains_edge(channel.source, channel.destination)
    }

    /// Constructs a new valid packet `Path` from self and the given destination.
    /// This method uses `INTERMEDIATE_HOPS` as the maximum number of hops.
    pub fn find_auto_path(&self, destination: Address) -> Result<Path> {
        self.find_path(self.me, destination, INTERMEDIATE_HOPS)
    }

    /// Constructs a new valid packet `Path` from the given source and destination.
    pub fn find_path(&self, _source: Address, _destination: Address, _max_hops: usize) -> Result<Path> {
        unimplemented!()
    }
}
