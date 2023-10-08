use crate::channel_graph::ChannelChange::{CurrentBalance, Epoch, Status, TicketIndex};
use crate::errors::PathError::MissingChannel;
use crate::errors::{PathError, Result};
use crate::path::ChannelPath;
use core_crypto::random::random_float;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::channels::{ChannelEntry, ChannelStatus};
use core_types::protocol::INTERMEDIATE_HOPS;
use petgraph::graphmap::DiGraphMap;
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Add;
use utils_log::{debug, info};
use utils_types::primitives::{Address, Balance, U256};

pub const DEFAULT_INITIAL_QUALITY: f64 = 0.0_f64;

/// Structure that adds additional data to a `ChannelEntry`, which
/// can be used to compute edge weights.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeightedChannel {
    channel: ChannelEntry,
    quality: Option<f64>,
}

const PATH_RANDOMNESS: f64 = 0.1;

type EdgeWeight = U256;

impl WeightedChannel {
    /// Calculates edge weight based on the channel information
    pub fn calculate_weight(&self) -> EdgeWeight {
        let r = random_float() * PATH_RANDOMNESS;
        let base = self.channel.balance.value().addn(1);
        // (stake + 1) * (1 + r)
        base.add(base.multiply_f64(r).unwrap())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WeightedChannelPath(Vec<Address>, EdgeWeight);

impl PartialOrd for WeightedChannelPath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.1.partial_cmp(&other.1)
    }
}

impl Ord for WeightedChannelPath {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

/// Trait for implementing custom path selection algorithm from the channel graph.
pub trait PathSelector {
    /// Select path of maximum `max_hops` from `source` to `destination` in the given channel graph.
    /// Fails if no such path can be found.
    fn select_path(
        &self,
        graph: &DiGraphMap<Address, WeightedChannel>,
        source: Address,
        destination: Address,
        max_hops: usize,
    ) -> Result<ChannelPath>;
}

/// Simple path selector using depth-first search of the channel graph.
#[derive(Clone, Debug)]
pub struct DFSPathSelector {
    /// Maximum number of iterations before a path selection fails
    /// Default is 100
    pub max_iterations: usize,
    /// Peer quality threshold for a channel to be taken into account.
    /// Default is 0.5
    pub quality_threshold: f64,
}

impl Default for DFSPathSelector {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            quality_threshold: 0.5_f64,
        }
    }
}

impl DFSPathSelector {
    fn filter_channel(
        &self,
        channel: &WeightedChannel,
        destination: &Address,
        current: &Vec<Address>,
        dead_ends: &HashSet<Address>,
    ) -> bool {
        !destination.eq(&channel.channel.destination) &&               // last hop does not need channel
        channel.quality.unwrap_or(0_f64) > self.quality_threshold &&  // quality threshold
        !current.contains(&channel.channel.destination) &&     // must not be in the path already (no loops allowed)
        !dead_ends.contains(&channel.channel.destination) // must not be in the dead end list
    }
}

impl PathSelector for DFSPathSelector {
    fn select_path(
        &self,
        graph: &DiGraphMap<Address, WeightedChannel>,
        source: Address,
        destination: Address,
        max_hops: usize,
    ) -> Result<ChannelPath> {
        let mut queue = BinaryHeap::new();
        let mut dead_ends = HashSet::new();

        graph
            .edges_directed(source, Direction::Outgoing)
            .filter_map(|channel| {
                let w = channel.weight();
                self.filter_channel(w, &destination, &vec![], &dead_ends)
                    .then(|| WeightedChannelPath(vec![w.channel.destination], w.calculate_weight()))
            })
            .for_each(|wcp| queue.push(wcp));

        let mut iters = 0;
        while !queue.is_empty() && iters < self.max_iterations {
            let current_path = queue.peek().unwrap();
            let current_path_len = current_path.0.len();

            if current_path_len >= max_hops && current_path_len <= INTERMEDIATE_HOPS {
                return Ok(ChannelPath::new_valid(queue.pop().unwrap().0));
            }

            let last_peer = *current_path.0.last().unwrap();
            let new_channels = graph
                .edges_directed(last_peer, Direction::Outgoing)
                .filter(|channel| self.filter_channel(channel.weight(), &destination, &current_path.0, &dead_ends))
                .collect::<Vec<_>>();

            if !new_channels.is_empty() {
                let current_path_clone = current_path.clone();
                for new_channel in new_channels {
                    let mut next_path_variant = current_path_clone.clone();
                    next_path_variant.0.push(new_channel.weight().channel.destination);
                    next_path_variant.1 += new_channel.weight().calculate_weight();
                    queue.push(next_path_variant);
                }
            } else {
                queue.pop();
                dead_ends.insert(last_peer);
            }

            iters += 1;
        }

        Err(PathError::PathNotFound(
            max_hops,
            source.to_string(),
            destination.to_string(),
        ))
    }
}

/// Implements a HOPR payment channel graph cached in-memory.
/// This structure is useful for tracking channel state changes and
/// packet path finding.
/// The structure is updated only from the Indexer and therefore contains only
/// the channels that were *seen* on-chain.
/// Using this structure is generally faster than querying the DB and therefore
/// is preferred for per-packet path-finding computations.
pub struct ChannelGraph {
    me: Address,
    graph: DiGraphMap<Address, WeightedChannel>,
    path_selector: Box<dyn PathSelector>,
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
    pub fn new(me: Address, path_selector: Box<dyn PathSelector>) -> Self {
        Self {
            me,
            graph: DiGraphMap::default(),
            path_selector,
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
            let weighted = WeightedChannel { channel, quality: None };

            self.graph.add_edge(channel.source, channel.destination, weighted);
            info!("new {channel}");

            None
        }
    }

    pub fn update_channel_quality(&mut self, src: Address, dst: Address, quality: f64) {
        if let Some(channel) = self.graph.edge_weight_mut(src, dst) {
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

    /// Calculates the total weight of the given outgoing channel path
    pub fn path_weight(&self, path: ChannelPath) -> Result<EdgeWeight> {
        let mut initial_addr = self.me;
        let mut weight = EdgeWeight::zero();

        for hop in path.hops() {
            let w = self
                .graph
                .edge_weight(initial_addr, *hop)
                .ok_or(MissingChannel(initial_addr.to_string(), hop.to_string()))?;
            weight = weight.add(w.calculate_weight());
            initial_addr = *hop;
        }

        Ok(weight)
    }

    /// Constructs a new valid packet `Path` from self and the given destination.
    /// This method uses `INTERMEDIATE_HOPS` as the maximum number of hops.
    pub fn find_auto_path(&self, destination: Address) -> Result<ChannelPath> {
        self.find_path(self.me, destination, INTERMEDIATE_HOPS)
    }

    /// Constructs a new valid packet `Path` from the given source and destination.
    pub fn find_path(&self, source: Address, destination: Address, max_hops: usize) -> Result<ChannelPath> {
        self.path_selector
            .select_path(&self.graph, source, destination, max_hops)
    }
}
