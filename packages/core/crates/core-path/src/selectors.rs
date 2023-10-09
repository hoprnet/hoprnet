use crate::channel_graph::{ChannelEdge, ChannelGraph};
use crate::errors::PathError::MissingChannel;
use crate::errors::{PathError, Result};
use crate::path::ChannelPath;
use core_types::protocol::INTERMEDIATE_HOPS;
use petgraph::visit::EdgeRef;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::ops::Add;
use utils_types::primitives::{Address, U256};

/// Trait for implementing custom path selection algorithm from the channel graph.
pub trait PathSelector {
    /// Select path of maximum `max_hops` from `source` to `destination` in the given channel graph.
    /// Fails if no such path can be found.
    fn select_path(
        &self,
        graph: &ChannelGraph,
        source: Address,
        destination: Address,
        max_hops: usize,
    ) -> Result<ChannelPath>;

    /// Constructs a new valid packet `Path` from self and the given destination.
    /// This method uses `INTERMEDIATE_HOPS` as the maximum number of hops.
    fn select_auto_path(&self, graph: &ChannelGraph, destination: Address) -> Result<ChannelPath> {
        self.select_path(graph, graph.my_address(), destination, INTERMEDIATE_HOPS)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WeightedChannelPath(Vec<Address>, U256);

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

/// Calculates the total weight of the given outgoing channel path.
pub fn path_weight(graph: &ChannelGraph, path: ChannelPath) -> Result<U256> {
    let mut initial_addr = graph.my_address();
    let mut weight = U256::zero();

    for hop in path.hops() {
        let w = graph
            .get_channel(initial_addr, *hop)
            .ok_or(MissingChannel(initial_addr.to_string(), hop.to_string()))?;
        weight = weight.add(w.get_weight());
        initial_addr = *hop;
    }

    Ok(weight)
}

/// Simple path selector using depth-first search of the channel graph.
#[derive(Clone, Debug)]
pub struct DfsPathSelector {
    /// Maximum number of iterations before a path selection fails
    /// Default is 100
    pub max_iterations: usize,
    /// Peer quality threshold for a channel to be taken into account.
    /// Default is 0.5
    pub quality_threshold: f64,
}

impl Default for DfsPathSelector {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            quality_threshold: 0.5_f64,
        }
    }
}

impl DfsPathSelector {
    fn filter_channel(
        &self,
        channel: &ChannelEdge,
        destination: &Address,
        current: &[Address],
        dead_ends: &HashSet<Address>,
    ) -> bool {
        !destination.eq(&channel.channel.destination) &&               // last hop does not need channel
            channel.quality.unwrap_or(0_f64) > self.quality_threshold &&  // quality threshold
            !current.contains(&channel.channel.destination) &&     // must not be in the path already (no loops allowed)
            !dead_ends.contains(&channel.channel.destination) // must not be in the dead end list
    }
}

impl PathSelector for DfsPathSelector {
    fn select_path(
        &self,
        graph: &ChannelGraph,
        source: Address,
        destination: Address,
        max_hops: usize,
    ) -> Result<ChannelPath> {
        let mut queue = BinaryHeap::new();
        let mut dead_ends = HashSet::new();

        graph
            .open_channels_from(source)
            .filter_map(|channel| {
                let w = channel.weight();
                self.filter_channel(w, &destination, &[], &dead_ends)
                    .then(|| WeightedChannelPath(vec![w.channel.destination], w.channel.get_weight()))
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
                .open_channels_from(last_peer)
                .filter(|channel| self.filter_channel(channel.weight(), &destination, &current_path.0, &dead_ends))
                .collect::<Vec<_>>();

            if !new_channels.is_empty() {
                let current_path_clone = current_path.clone();
                for new_channel in new_channels {
                    let mut next_path_variant = current_path_clone.clone();
                    next_path_variant.0.push(new_channel.weight().channel.destination);
                    next_path_variant.1 += new_channel.weight().channel.get_weight();
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

#[cfg(test)]
mod tests {

}
