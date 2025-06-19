pub mod dfs;

use std::ops::Add;

use async_trait::async_trait;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::primitives::Address;

use crate::{ChannelPath, channel_graph::ChannelEdge, errors::Result};

/// Computes weights of edges corresponding to [`ChannelEdge`].
pub trait EdgeWeighting<W>
where
    W: Default + Add<W, Output = W>,
{
    /// Edge weighting function.
    fn calculate_weight(channel: &ChannelEdge) -> W;
}

/// Trait for implementing a custom path selection algorithm from the channel graph.
#[async_trait]
pub trait PathSelector {
    /// Select a path of maximum `max_hops` from `source` to `destination` in the given channel graph.
    /// NOTE: the resulting path does not contain `source` but does contain `destination`.
    /// Fails if no such path can be found.
    async fn select_path(
        &self,
        source: Address,
        destination: Address,
        min_hops: usize,
        max_hops: usize,
    ) -> Result<ChannelPath>;

    /// Constructs a new valid packet `Path` from source to the given destination.
    /// This method uses `INTERMEDIATE_HOPS` as the maximum number of hops.
    async fn select_auto_path(&self, source: Address, destination: Address) -> Result<ChannelPath> {
        self.select_path(source, destination, 1usize, INTERMEDIATE_HOPS).await
    }
}
