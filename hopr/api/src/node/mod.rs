//! High-level HOPR node API trait definitions.
//!
//! This module defines the external API interface for interacting with a running HOPR node.
//! The [`HoprNodeApi`] trait provides the complete set of operations available to external
//! consumers, abstracting over the underlying implementation details.

pub mod state;

use std::time::Duration;

use futures::{Sink, Stream};
use hopr_crypto_types::prelude::Hash;
use hopr_internal_types::{
    prelude::{AccountEntry, AcknowledgedTicket, ChannelEntry, VerifiedTicket},
    tickets::{RedeemableTicket, WinningProbability},
};
use hopr_primitive_types::prelude::{Address, Balance, Currency, HoprBalance, XDaiBalance};
use multiaddr::Multiaddr;
pub use multiaddr::PeerId;

pub use crate::chain::ChainInfo;
use crate::{
    chain::ChannelId,
    db::{ChannelTicketStatistics, TicketSelector},
    graph::Observable,
    network::Health,
};

/// Result of opening a channel on-chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenChannelResult {
    /// Transaction hash of the channel open operation.
    pub tx_hash: Hash,
    /// The ID of the opened channel.
    pub channel_id: ChannelId,
}

/// Result of closing a channel on-chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseChannelResult {
    /// Transaction hash of the channel close operation.
    pub tx_hash: Hash,
}

/// Configuration for the Safe module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeModuleConfig {
    /// Address of the Safe contract.
    pub safe_address: Address,
    /// Address of the module contract.
    pub module_address: Address,
}

/// High-level network operations.
#[async_trait::async_trait]
pub trait HoprNodeNetworkOperations {
    /// Error type for node operations.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Observable type returned by peer information queries.
    type PeerObservable: Observable + Send;

    // === Identity ===

    /// Returns the PeerId of this node used in the transport layer.
    fn me_peer_id(&self) -> PeerId;

    /// Returns all public nodes announced on the network.
    async fn get_public_nodes(&self) -> Result<Vec<(PeerId, Address, Vec<Multiaddr>)>, Self::Error>;

    /// Returns the current network health status.
    async fn network_health(&self) -> Health;

    /// Returns all currently connected peers.
    async fn network_connected_peers(&self) -> Result<Vec<PeerId>, Self::Error>;

    /// Returns observations for a specific peer.
    fn network_peer_info(&self, peer: &PeerId) -> Option<Self::PeerObservable>;

    /// Returns all network peers with quality above the minimum score.
    async fn all_network_peers(
        &self,
        minimum_score: f64,
    ) -> Result<Vec<(Option<Address>, PeerId, Self::PeerObservable)>, Self::Error>;

    // === Transport ===

    /// Returns the multiaddresses this node is announcing.
    fn local_multiaddresses(&self) -> Vec<Multiaddr>;

    /// Returns the multiaddresses this node is listening on.
    async fn listening_multiaddresses(&self) -> Vec<Multiaddr>;

    /// Returns the observed multiaddresses for a peer.
    async fn network_observed_multiaddresses(&self, peer: &PeerId) -> Vec<Multiaddr>;

    /// Returns the multiaddresses announced on-chain for a peer.
    async fn multiaddresses_announced_on_chain(&self, peer: &PeerId) -> Result<Vec<Multiaddr>, Self::Error>;

    // === Peers ===

    /// Pings a peer and returns the round-trip time along with observable data.
    async fn ping(&self, peer: &PeerId) -> Result<(Duration, Self::PeerObservable), Self::Error>;
}

/// High-level chain operations.
#[async_trait::async_trait]
pub trait HoprNodeChainOperations {
    /// Error type for node operations.
    type Error: std::error::Error + Send + Sync + 'static;

    // === Identity ===

    /// Returns the on-chain address of this node.
    fn me_onchain(&self) -> Address;

    // === Configuration ===

    /// Returns the Safe module configuration.
    fn get_safe_config(&self) -> SafeModuleConfig;

    // === Balance ===

    /// Returns the balance of the node's on-chain account.
    async fn get_balance<C: Currency + Send>(&self) -> Result<Balance<C>, Self::Error>;

    /// Returns the balance of the node's Safe.
    async fn get_safe_balance<C: Currency + Send>(&self) -> Result<Balance<C>, Self::Error>;

    /// Returns the current Safe allowance for this node.
    async fn safe_allowance(&self) -> Result<HoprBalance, Self::Error>;

    // === Chain Information ===

    /// Returns information about the chain environment.
    async fn chain_info(&self) -> Result<ChainInfo, Self::Error>;

    /// Returns the minimum ticket price on the network.
    async fn get_ticket_price(&self) -> Result<HoprBalance, Self::Error>;

    /// Returns the minimum winning probability for incoming tickets.
    async fn get_minimum_incoming_ticket_win_probability(&self) -> Result<WinningProbability, Self::Error>;

    /// Returns the channel closure notice period.
    async fn get_channel_closure_notice_period(&self) -> Result<Duration, Self::Error>;

    // === Accounts ===

    /// Returns all accounts announced on-chain.
    async fn accounts_announced_on_chain(&self) -> Result<Vec<AccountEntry>, Self::Error>;

    /// Resolves a PeerId to its on-chain address.
    async fn peerid_to_chain_key(&self, peer_id: &PeerId) -> Result<Option<Address>, Self::Error>;

    /// Resolves an on-chain address to its PeerId.
    async fn chain_key_to_peerid(&self, address: &Address) -> Result<Option<PeerId>, Self::Error>;

    // === Channels ===

    /// Returns a channel by its ID.
    async fn channel_from_hash(&self, channel_id: &Hash) -> Result<Option<ChannelEntry>, Self::Error>;

    /// Returns a channel between two addresses.
    async fn channel(&self, src: &Address, dest: &Address) -> Result<Option<ChannelEntry>, Self::Error>;

    /// Returns all channels from the given source address.
    async fn channels_from(&self, src: &Address) -> Result<Vec<ChannelEntry>, Self::Error>;

    /// Returns all channels to the given destination address.
    async fn channels_to(&self, dest: &Address) -> Result<Vec<ChannelEntry>, Self::Error>;

    /// Returns all channels.
    async fn all_channels(&self) -> Result<Vec<ChannelEntry>, Self::Error>;

    /// Opens a channel to the destination with the given amount.
    async fn open_channel(&self, destination: &Address, amount: HoprBalance) -> Result<OpenChannelResult, Self::Error>;

    /// Funds an existing channel with additional balance.
    async fn fund_channel(&self, channel_id: &ChannelId, amount: HoprBalance) -> Result<Hash, Self::Error>;

    /// Closes a channel by its ID.
    async fn close_channel_by_id(&self, channel_id: &ChannelId) -> Result<CloseChannelResult, Self::Error>;

    // === Tickets ===

    /// Returns all tickets in a specific channel.
    async fn tickets_in_channel(&self, channel_id: &ChannelId) -> Result<Option<Vec<RedeemableTicket>>, Self::Error>;

    /// Returns all tickets held by this node.
    async fn all_tickets(&self) -> Result<Vec<VerifiedTicket>, Self::Error>;

    /// Returns statistics for all tickets.
    async fn ticket_statistics(&self) -> Result<ChannelTicketStatistics, Self::Error>;

    /// Resets ticket statistics to zero.
    async fn reset_ticket_statistics(&self) -> Result<(), Self::Error>;

    /// Redeems all tickets with value above the minimum.
    async fn redeem_all_tickets<B: Into<HoprBalance> + Send>(&self, min_value: B) -> Result<(), Self::Error>;

    /// Redeems tickets from a specific counterparty.
    async fn redeem_tickets_with_counterparty<B: Into<HoprBalance> + Send>(
        &self,
        counterparty: &Address,
        min_value: B,
    ) -> Result<(), Self::Error>;

    /// Redeems tickets in a specific channel.
    async fn redeem_tickets_in_channel<B: Into<HoprBalance> + Send>(
        &self,
        channel_id: &Hash,
        min_value: B,
    ) -> Result<(), Self::Error>;

    /// Redeems a specific ticket.
    async fn redeem_ticket(&self, ack_ticket: AcknowledgedTicket) -> Result<(), Self::Error>;

    /// Sink type for submitting ticket redemption requests.
    type RedemptionSink: Sink<TicketSelector, Error = Self::Error> + Clone + Send;

    /// Returns a stream of newly received winning tickets.
    fn subscribe_winning_tickets(&self) -> impl Stream<Item = VerifiedTicket> + Send + 'static;

    /// Returns a sink for submitting ticket redemption requests.
    fn redemption_requests(&self) -> Result<Self::RedemptionSink, Self::Error>;

    // === Withdrawals ===

    /// Withdraws HOPR tokens to the specified recipient.
    async fn withdraw_tokens(&self, recipient: Address, amount: HoprBalance) -> Result<Hash, Self::Error>;

    /// Withdraws native currency to the specified recipient.
    async fn withdraw_native(&self, recipient: Address, amount: XDaiBalance) -> Result<Hash, Self::Error>;

    // === Tickets ===
}

pub trait HoprNodeOperations {
    fn status(&self) -> state::HoprState;
}
