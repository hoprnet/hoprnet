use futures::{Future, stream::BoxStream};
use hopr_crypto_packet::{HoprSphinxHeaderSpec, HoprSphinxSuite, KeyIdMapper};
use hopr_crypto_types::types::{Hash, OffchainPublicKey};
use hopr_internal_types::prelude::{
    AccountEntry, ChannelDirection, ChannelEntry, ChannelId, ChannelStatus, RedeemableTicket,
};
use hopr_primitive_types::{
    balance::HoprBalance,
    prelude::{Address, Balance, Currency},
};
use multiaddr::Multiaddr;

/// Receipt of an on-chain operation.
pub type ChainReceipt = Hash;

/// Selector for channels.
///
/// See [`ChainReadChannelOperations::stream_channels`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChannelSelector {
    /// Filter by counterparty address.
    pub counterparty: Option<Address>,
    /// Filter by direction.
    pub direction: Option<ChannelDirection>,
    /// Filter by possible channel states.
    pub allowed_states: Vec<ChannelStatus>,
}

/// On-chain read operations regarding channels.
#[async_trait::async_trait]
pub trait ChainReadChannelOperations {
    type Error;

    /// Returns a single channel given `src` and `dst`.
    async fn channel_by_parties(&self, src: &Address, dst: &Address) -> Result<Option<ChannelEntry>, Self::Error>;

    /// Returns a single channel given `channel_id`.
    async fn channel_by_id(&self, channel_id: &ChannelId) -> Result<Option<ChannelEntry>, Self::Error>;

    /// Returns a stream of channels given the [`ChannelSelector`].
    async fn stream_channels<'a>(
        &'a self,
        selector: ChannelSelector,
    ) -> Result<BoxStream<'a, Result<ChannelEntry, Self::Error>>, Self::Error>;
}

/// On-chain write operations regarding channels.
#[async_trait::async_trait]
pub trait ChainWriteChannelOperations {
    type Error;
    /// Opens a channel with `dst` and `amount`.
    async fn open_channel(
        &self,
        dst: &Address,
        amount: HoprBalance,
    ) -> Result<impl Future<Output = Result<ChainReceipt, Self::Error>>, Self::Error>;

    /// Funds an existing channel.
    async fn fund_channel(
        &self,
        channel_id: &ChannelId,
        amount: HoprBalance,
    ) -> Result<impl Future<Output = Result<ChainReceipt, Self::Error>>, Self::Error>;

    /// Closes an existing channel.
    async fn close_channel(
        &self,
        channel_id: &ChannelId,
    ) -> Result<impl Future<Output = Result<ChainReceipt, Self::Error>>, Self::Error>;
}

/// On-chain write operations regarding on-chain node accounts.
#[async_trait::async_trait]
pub trait ChainWriteAccountOperations {
    type Error;

    /// Announces transport key and list of multiaddresses.
    async fn announce(
        &self,
        multiaddrs: &[Multiaddr],
        key: &OffchainPublicKey,
    ) -> Result<impl Future<Output = Result<ChainReceipt, Self::Error>>, Self::Error>;

    /// Withdraws native or token currency.
    async fn withdraw<C: Currency>(
        &self,
        balance: Balance<C>,
    ) -> Result<impl Future<Output = Result<ChainReceipt, Self::Error>>, Self::Error>;

    /// Registers Safe address with the current node.
    async fn register_safe(
        &self,
        safe_address: Address,
    ) -> Result<impl Future<Output = Result<ChainReceipt, Self::Error>>, Self::Error>;
}

/// Selector for on-chain node accounts.
///
/// See [`ChainReadAccountOperations::stream_accounts`].
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct AccountSelector {
    pub public_only: bool,
}

/// Chain operations that read on-chain node accounts.
#[async_trait::async_trait]
pub trait ChainReadAccountOperations {
    type Error;

    /// Returns on-chain node accounts with the given [`AccountSelector`].
    async fn stream_accounts(
        &self,
        selector: AccountSelector,
    ) -> Result<BoxStream<'_, Result<AccountEntry, Self::Error>>, Self::Error>;
}

/// Operations for offchain keys.
///
/// This typically translates to optimized (and cached) versions of [`ChainReadChannelOperations`].
#[async_trait::async_trait]
pub trait ChainKeyOperations {
    type Error;
    /// Translates [`Address`] into [`OffchainPublicKey`].
    async fn chain_key_to_packet_key(&self, chain: &Address) -> Result<Option<OffchainPublicKey>, Self::Error>;
    /// Translates [`OffchainPublicKey`] into [`Address`].
    async fn packet_key_to_chain_key(&self, packet: &OffchainPublicKey) -> Result<Option<Address>, Self::Error>;
    /// Returns [mapper](KeyIdMapper) for offchain key IDs.
    fn key_id_mapper(&self) -> &impl KeyIdMapper<HoprSphinxSuite, HoprSphinxHeaderSpec>;
}

/// On-chain operations with tickets.
#[async_trait::async_trait]
pub trait ChainTicketOperations {
    type Error;

    /// Redeems a single ticket on-chain.
    async fn redeem_ticket(
        &self,
        ticket: RedeemableTicket,
    ) -> Result<impl Future<Output = Result<ChainReceipt, Self::Error>>, Self::Error>;
}
