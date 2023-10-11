pub mod legacy;

use crate::channel_graph::ChannelGraph;
use crate::errors::PathError::ChannelNotOpened;
use crate::errors::{PathError::MissingChannel, Result};
use crate::path::{BasePath, ChannelPath};
use core_types::channels::{ChannelEntry, ChannelStatus};
use core_types::protocol::INTERMEDIATE_HOPS;
use std::ops::Add;
use utils_types::primitives::{Address, U256};

/// Computes weights of edges corresponding to `ChannelEntry`.
pub trait EdgeWeighting<W>
where
    W: Default + Add<W, Output = W>,
{
    /// Edge weighting function.
    fn calculate_weight(channel: &ChannelEntry) -> W;

    /// Calculates the total weight of the given outgoing channel path.
    fn total_path_weight(graph: &ChannelGraph, path: ChannelPath) -> Result<W> {
        let mut initial_addr = graph.my_address();
        let mut weight = W::default();

        for hop in path.hops() {
            let w = graph
                .get_channel(initial_addr, *hop)
                .ok_or(MissingChannel(initial_addr.to_string(), hop.to_string()))?;

            if w.status != ChannelStatus::Open {
                return Err(ChannelNotOpened(initial_addr.to_string(), hop.to_string()));
            }

            weight = weight.add(Self::calculate_weight(w));
            initial_addr = *hop;
        }

        Ok(weight)
    }
}

/// Trait for implementing custom path selection algorithm from the channel graph.
pub trait PathSelector<CW, W = U256>
where
    CW: EdgeWeighting<W>,
    W: Default + Add<W, Output = W>,
{
    /// Select path of maximum `max_hops` from `source` to `destination` in the given channel graph.
    /// NOTE: the resulting path does not contain `source` but does contain `destination`.
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
