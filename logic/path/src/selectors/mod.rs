use hopr_internal_types::prelude::*;
use hopr_primitive_types::primitives::Address;

use crate::{
    ChannelPath,
    errors::{PathError, Result},
};

/// Trait for implementing a custom path selection algorithm from the channel graph.
#[async_trait::async_trait]
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
    /// This method uses `INTERMEDIATE_HOPS` as the maximum number of hops and 1 hop as a minimum.
    async fn select_auto_path(&self, source: Address, destination: Address) -> Result<ChannelPath> {
        self.select_path(source, destination, 1usize, INTERMEDIATE_HOPS).await
    }
}

/// A path selector that does not resolve any path, always returns [`PathError::PathNotFound`].
#[derive(Debug, Clone, Copy, Default)]
pub struct NoPathSelector;

#[async_trait::async_trait]
impl PathSelector for NoPathSelector {
    async fn select_path(
        &self,
        source: Address,
        destination: Address,
        min_hops: usize,
        _max_hops: usize,
    ) -> Result<ChannelPath> {
        Err(PathError::PathNotFound(
            min_hops,
            source.to_string(),
            destination.to_string(),
        ))
    }
}
