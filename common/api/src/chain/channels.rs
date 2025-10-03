use std::error::Error;

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
}

/// On-chain write operations regarding channels.
#[async_trait::async_trait]
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
    ) -> Result<BoxFuture<'a, Result<(ChannelStatus, ChainReceipt), Self::Error>>, Self::Error>;
}
