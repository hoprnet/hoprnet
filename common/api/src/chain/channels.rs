use std::{
    error::Error,
    ops::{Bound, RangeBounds},
};

use futures::{future::BoxFuture, stream::BoxStream};
pub use hopr_internal_types::prelude::{ChannelDirection, ChannelEntry, ChannelId, ChannelStatusDiscriminants};
use hopr_internal_types::prelude::{ChannelStatus, generate_channel_id};
use hopr_primitive_types::prelude::Address;
pub use hopr_primitive_types::prelude::HoprBalance;
pub type DateTime = chrono::DateTime<chrono::Utc>;
pub use chrono::Utc;
use strum::IntoDiscriminant;

use crate::chain::ChainReceipt;

/// Selector for channels.
///
/// See [`ChainReadChannelOperations::stream_channels`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelSelector {
    /// Filter by source address.
    pub source: Option<Address>,
    /// Filter by destination address
    pub destination: Option<Address>,
    /// Filter by possible channel states.
    pub allowed_states: Vec<ChannelStatusDiscriminants>,
    /// Range of closure times if `PendingToClose` was specified in `allowed_states`,
    /// otherwise has no effect.
    pub closure_time_range: (Bound<DateTime>, Bound<DateTime>),
}

impl Default for ChannelSelector {
    fn default() -> Self {
        Self {
            source: None,
            destination: None,
            allowed_states: vec![],
            closure_time_range: (Bound::Unbounded, Bound::Unbounded),
        }
    }
}

impl ChannelSelector {
    /// Sets the `source` bound on channel.
    #[must_use]
    pub fn with_source<A: Into<Address>>(mut self, address: A) -> Self {
        self.source = Some(address.into());
        self
    }

    /// Sets the `destination` bound on channel.
    #[must_use]
    pub fn with_destination<A: Into<Address>>(mut self, address: A) -> Self {
        self.destination = Some(address.into());
        self
    }

    /// Sets the allowed channel states.
    #[must_use]
    pub fn with_allowed_states(mut self, allowed_states: &[ChannelStatusDiscriminants]) -> Self {
        self.allowed_states.extend_from_slice(allowed_states);
        self
    }

    /// Sets the channel closure range.
    ///
    /// This has effect only if `PendingToClose` is set in the allowed states.
    #[must_use]
    pub fn with_closure_time_range<T: RangeBounds<DateTime>>(mut self, range: T) -> Self {
        self.closure_time_range = (range.start_bound().cloned(), range.end_bound().cloned());
        self
    }

    pub fn satisfies(&self, entry: &ChannelEntry) -> bool {
        if let Some(source) = &self.source {
            if entry.source != *source {
                return false;
            }
        }

        if let Some(dst) = &self.destination {
            if entry.destination != *dst {
                return false;
            }
        }

        if !self.allowed_states.is_empty() && !self.allowed_states.contains(&entry.status.discriminant()) {
            return false;
        }

        if let ChannelStatus::PendingToClose(time) = &entry.status {
            let time = DateTime::from(*time);
            if !self.closure_time_range.contains(&time) {
                return false;
            }
        }

        true
    }
}

/// On-chain read operations regarding channels.
#[async_trait::async_trait]
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait ChainReadChannelOperations {
    type Error: Error + Send + Sync + 'static;

    /// Returns on-chain [`Address`] of the current node.
    fn me(&self) -> &Address;

    /// Returns a single channel given `src` and `dst`.
    async fn channel_by_parties(&self, src: &Address, dst: &Address) -> Result<Option<ChannelEntry>, Self::Error> {
        self.channel_by_id(&generate_channel_id(src, dst)).await
    }

    /// Returns a single channel given `channel_id`.
    async fn channel_by_id(&self, channel_id: &ChannelId) -> Result<Option<ChannelEntry>, Self::Error>;

    /// Returns a stream of channels given the [`ChannelSelector`].
    async fn stream_channels<'a>(
        &'a self,
        selector: ChannelSelector,
    ) -> Result<BoxStream<'a, ChannelEntry>, Self::Error>;
}

/// On-chain write operations regarding channels.
#[async_trait::async_trait]
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait ChainWriteChannelOperations {
    type Error: Error + Send + Sync + 'static;
    /// Opens a channel with `dst` and `amount`.
    async fn open_channel<'a>(
        &'a self,
        dst: &'a Address,
        amount: HoprBalance,
    ) -> Result<BoxFuture<'a, Result<(ChannelId, ChainReceipt), Self::Error>>, Self::Error>;

    /// Funds an existing channel.
    async fn fund_channel<'a>(
        &'a self,
        channel_id: &'a ChannelId,
        amount: HoprBalance,
    ) -> Result<BoxFuture<'a, Result<ChainReceipt, Self::Error>>, Self::Error>;

    /// Closes an existing channel.
    async fn close_channel<'a>(
        &'a self,
        channel_id: &'a ChannelId,
    ) -> Result<BoxFuture<'a, Result<ChainReceipt, Self::Error>>, Self::Error>;
}
