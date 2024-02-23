pub mod legacy;

use crate::channel_graph::ChannelGraph;
use crate::errors::PathError::ChannelNotOpened;
use crate::errors::{PathError::MissingChannel, Result};
use crate::path::ChannelPath;
use crate::traits::{ChannelEdge, ChannelQualityGraph, Path};
use hopr_internal_types::prelude::*;
use hopr_primitive_types::primitives::{Address, U256};
use std::ops::Add;

/// Computes weights `W` of edges corresponding to [`ChannelEdge<Q>`].
pub trait EdgeWeighting<W, Q>
where
    W: Default + Add<W, Output = W>,
{
    /// Edge weighting function.
    fn calculate_weight(edge: &ChannelEdge<Q>) -> W;

    /// Calculates the total weight of the given outgoing channel path.
    fn total_path_weight<G>(graph: &G, start: Address, path: ChannelPath) -> Result<W>
    where
        G: ChannelQualityGraph<Q>,
    {
        let mut initial_addr = start;
        let mut weight = W::default();

        for hop in path.hops() {
            let w = graph
                .get_channel(initial_addr, *hop)
                .ok_or(MissingChannel(initial_addr.to_string(), hop.to_string()))?;

            if w.channel.status != ChannelStatus::Open {
                return Err(ChannelNotOpened(initial_addr.to_string(), hop.to_string()));
            }

            weight = weight.add(Self::calculate_weight(w));
            initial_addr = *hop;
        }

        Ok(weight)
    }
}

/// Trait for implementing custom path selection algorithm from the channel graph.
pub trait PathSelector<CW, Q = f64, W = U256>
where
    CW: EdgeWeighting<W, Q>,
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
