use std::fmt::{Display, Formatter};
use petgraph::graphmap::DiGraphMap;
use core_types::channels::{ChannelEntry, ChannelStatus};
use core_types::protocol::INTERMEDIATE_HOPS;
use serde::{Deserialize, Serialize};
use utils_log::info;
use utils_types::primitives::{Address, Balance};
use crate::channel_graph::ChannelChange::{CurrentBalance, Status};
use crate::path::Path;
use crate::errors::Result;

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
    graph: DiGraphMap<Address, ChannelEntry>
}

/// Enumerates changes on a channel entry update
#[derive(Clone, Copy, Debug)]
pub enum ChannelChange {
    /// Channel status has changed
    Status { old: ChannelStatus, new: ChannelStatus },

    /// Channel balance has changed
    CurrentBalance { old: Balance, new: Balance }
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
        }
    }
}

impl ChannelGraph {
    /// Maximum number of intermediate hops the automatic path finding algorithm will look for.
    pub const INTERMEDIATE_HOPS: usize = 3;

    /// Creates a new instance with the given self `Address`.
    pub fn new(me: Address) -> Self {
        Self { me, graph: DiGraphMap::default() }
    }

    /// Inserts or updates the given channel in the channel graph.
    /// Returns a set of changes if the channel was already present in the graphs or
    /// None if the channel was not previously present in the channel graph.
    pub fn update_channel(&mut self, channel: ChannelEntry) -> Option<Vec<ChannelChange>> {
        if let Some(old_value) = self.graph.add_edge(channel.source, channel.destination, channel) {
            let mut ret = Vec::new();

            if old_value.status != channel.status {
                ret.push(Status { old: old_value.status, new: channel.status });
            }

            if old_value.balance != channel.balance {
                ret.push(CurrentBalance { old: old_value.balance, new: channel.balance })
            }

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