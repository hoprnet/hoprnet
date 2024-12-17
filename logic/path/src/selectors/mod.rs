pub mod dfs;

use hopr_internal_types::prelude::*;
use hopr_primitive_types::primitives::{Address, U256};
use std::ops::Add;

use crate::channel_graph::{ChannelEdge, ChannelGraph};
use crate::errors::Result;
use crate::path::ChannelPath;

/// Computes weights of edges corresponding to [`ChannelEdge`].
pub trait EdgeWeighting<W>
where
    W: Default + Add<W, Output = W>,
{
    /// Edge weighting function.
    fn calculate_weight(channel: &ChannelEdge) -> W;
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
        min_hops: usize,
        max_hops: usize,
    ) -> Result<ChannelPath>;

    /// Constructs a new valid packet `Path` from self and the given destination.
    /// This method uses `INTERMEDIATE_HOPS` as the maximum number of hops.
    fn select_auto_path(&self, graph: &ChannelGraph, destination: Address) -> Result<ChannelPath> {
        self.select_path(graph, graph.my_address(), destination, 1usize, INTERMEDIATE_HOPS)
    }
}
