use std::{error::Error, time::Duration};

use futures::{future::BoxFuture, stream::BoxStream};
use hopr_internal_types::prelude::ChannelStatus;
pub use hopr_internal_types::prelude::{ChannelDirection, ChannelEntry, ChannelId, ChannelStatusDiscriminants};
use hopr_primitive_types::prelude::Address;
pub use hopr_primitive_types::prelude::HoprBalance;

use crate::chain::ChainReceipt;

/// Selector for channels.
///
/// See [`ChainReadChannelOperations::stream_channels`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChannelSelector {
    /// Filter by counterparty address.
    pub counterparty: Option<Address>,
    /// Filter by direction.
    pub direction: Vec<ChannelDirection>,
    /// Filter by possible channel states.
    pub allowed_states: Vec<ChannelStatusDiscriminants>,
}

impl ChannelSelector {
    pub fn any() -> Self {
        Self {
            counterparty: None,
            direction: vec![ChannelDirection::Incoming, ChannelDirection::Outgoing],
            allowed_states: vec![
                ChannelStatusDiscriminants::Open,
                ChannelStatusDiscriminants::Closed,
                ChannelStatusDiscriminants::PendingToClose,
            ],
        }
    }
}

/// On-chain read operations regarding channels.
#[async_trait::async_trait]
pub trait ChainReadChannelOperations {
    type Error: Error + Send + Sync + 'static;

    /// Returns a single channel given `src` and `dst`.
    async fn channel_by_parties(&self, src: &Address, dst: &Address) -> Result<Option<ChannelEntry>, Self::Error>;

    /// Returns a single channel given `channel_id`.
    async fn channel_by_id(&self, channel_id: &ChannelId) -> Result<Option<ChannelEntry>, Self::Error>;

    /// Returns a stream of channels given the [`ChannelSelector`].
    async fn stream_channels<'a>(
        &'a self,
        selector: ChannelSelector,
    ) -> Result<BoxStream<'a, ChannelEntry>, Self::Error>;

    /// Gets the grace period for channel closure finalization.
    async fn channel_closure_notice_period(&self) -> Result<Duration, Self::Error>;
}

/// On-chain write operations regarding channels.
#[async_trait::async_trait]
pub trait ChainWriteChannelOperations {
    type Error: Error + Send + Sync + 'static;
    /// Opens a channel with `dst` and `amount`.
    async fn open_channel(
        &self,
        dst: &Address,
        amount: HoprBalance,
    ) -> Result<BoxFuture<'_, Result<(ChannelId, ChainReceipt), Self::Error>>, Self::Error>;

    /// Funds an existing channel.
    async fn fund_channel(
        &self,
        channel_id: &ChannelId,
        amount: HoprBalance,
    ) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, Self::Error>;

    /// Closes an existing channel.
    async fn close_channel(
        &self,
        channel_id: &ChannelId,
        direction: ChannelDirection,
    ) -> Result<BoxFuture<'_, Result<(ChannelStatus, ChainReceipt), Self::Error>>, Self::Error>;
}
