#![allow(
    missing_docs,
    trivial_casts,
    unused_variables,
    unused_mut,
    unused_imports,
    unused_extern_crates,
    non_camel_case_types
)]
#![allow(unused_imports, unused_attributes)]
#![allow(clippy::derive_partial_eq_without_eq, clippy::disallowed_names)]

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::task::{Context, Poll};
use swagger::{ApiError, ContextWrapper};

#[cfg(feature = "server")]
pub use {
    hyper, swagger
};

type ServiceError = Box<dyn Error + Send + Sync + 'static>;

pub const BASE_PATH: &str = "/api/v3";
pub const API_VERSION: &str = "3.0.0";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum AccountGetAddressResponse {
    /// Addresses fetched successfully.
    AddressesFetchedSuccessfully(models::AccountGetAddresses200Response),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum AccountGetAddressesResponse {
    /// Addresses fetched successfully.
    AddressesFetchedSuccessfully(models::AccountGetAddresses200Response),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum AccountGetBalancesResponse {
    /// Balances fetched successfuly.
    BalancesFetchedSuccessfuly(models::AccountGetBalances200Response),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum AccountWithdrawResponse {
    /// Withdraw successful. Receipt from this response can be used to check details of the transaction on ethereum chain.
    WithdrawSuccessful(models::AccountWithdraw200Response),
    /// Incorrect data in request body. Make sure to provide valid currency ('NATIVE' | 'HOPR') or amount.
    IncorrectDataInRequestBody(models::RequestStatus),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Withdraw amount exeeds current balance or unknown error. You can check current balance using /account/balance endpoint.
    WithdrawAmountExeedsCurrentBalanceOrUnknownError(models::AccountWithdraw422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum AliasesGetAliasResponse {
    /// HOPR address was found for the provided alias.
    HOPRAddressWasFoundForTheProvidedAlias(models::AliasesGetAlias200Response),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// This alias was not assigned to any PeerId before. You can get the list of all PeerId's and thier corresponding aliases using /aliases endpoint.
    ThisAliasWasNotAssignedToAnyPeerIdBefore(models::RequestStatus),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum AliasesGetAliasesResponse {
    /// Returns List of Aliases and corresponding peerIds.
    ReturnsListOfAliasesAndCorrespondingPeerIds(models::AliasesGetAliases200Response),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum AliasesRemoveAliasResponse {
    /// Alias removed succesfully.
    AliasRemovedSuccesfully,
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum AliasesSetAliasResponse {
    /// Alias set succesfully
    AliasSetSuccesfully,
    /// Invalid peerId. The format or length of the peerId is incorrect.
    InvalidPeerId(models::RequestStatus),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum ChannelsAggregateTicketsResponse {
    /// Tickets successfully aggregated
    TicketsSuccessfullyAggregated,
    /// Invalid channel id.
    InvalidChannelId(models::RequestStatus),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// The specified resource was not found
    TheSpecifiedResourceWasNotFound,
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum ChannelsCloseChannelResponse {
    /// Channel closed succesfully.
    ChannelClosedSuccesfully(models::ChannelsCloseChannel200Response),
    /// Invalid channel id.
    InvalidChannelId(models::RequestStatus),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum ChannelsFundChannelResponse {
    /// Channel funded successfully.
    ChannelFundedSuccessfully(models::ChannelsFundChannel200Response),
    /// Invalid channel id.
    InvalidChannelId(models::RequestStatus),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// The specified resource was not found
    TheSpecifiedResourceWasNotFound,
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum ChannelsGetChannelResponse {
    /// Channel fetched succesfully.
    ChannelFetchedSuccesfully(models::ChannelTopology),
    /// Invalid channel id.
    InvalidChannelId(models::RequestStatus),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// The specified resource was not found
    TheSpecifiedResourceWasNotFound,
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum ChannelsGetChannelsResponse {
    /// Channels fetched successfully.
    ChannelsFetchedSuccessfully(models::ChannelsGetChannels200Response),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum ChannelsGetTicketsResponse {
    /// Tickets fetched successfully.
    TicketsFetchedSuccessfully(Vec<models::Ticket>),
    /// Invalid peerId.
    InvalidPeerId(models::RequestStatus),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Tickets were not found for that channel. That means that no messages were sent inside this channel yet.
    TicketsWereNotFoundForThatChannel(models::RequestStatus),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum ChannelsOpenChannelResponse {
    /// Channel succesfully opened.
    ChannelSuccesfullyOpened(models::ChannelsOpenChannel201Response),
    /// Problem with inputs.
    ProblemWithInputs(models::RequestStatus),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// Failed to open the channel because of insufficient HOPR balance or allowance.
    FailedToOpenTheChannelBecauseOfInsufficientHOPRBalanceOrAllowance(models::ChannelsOpenChannel403Response),
    /// Failed to open the channel because the channel between this nodes already exists.
    FailedToOpenTheChannelBecauseTheChannelBetweenThisNodesAlreadyExists(models::ChannelsOpenChannel409Response),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum ChannelsRedeemTicketsResponse {
    /// Tickets redeemed successfully.
    TicketsRedeemedSuccessfully,
    /// Invalid channel id.
    InvalidChannelId(models::RequestStatus),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Tickets were not found for that channel. That means that no messages were sent inside this channel yet.
    TicketsWereNotFoundForThatChannel(models::RequestStatus),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum CheckNodeHealthyResponse {
    /// The node is ready
    TheNodeIsReady(serde_json::Value),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum CheckNodeReadyResponse {
    /// The node is ready
    TheNodeIsReady(serde_json::Value),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum CheckNodeStartedResponse {
    /// The node is started
    TheNodeIsStarted(serde_json::Value),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum MessagesDeleteMessagesResponse {
    /// Messages successfully deleted.
    MessagesSuccessfullyDeleted,
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum MessagesGetSizeResponse {
    /// Returns the message inbox size filtered by the given tag.
    ReturnsTheMessageInboxSizeFilteredByTheGivenTag(models::MessagesGetSize200Response),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum MessagesPopAllMessageResponse {
    /// Returns list of messages.
    ReturnsListOfMessages(models::MessagesPopAllMessage200Response),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum MessagesPopMessageResponse {
    /// Returns a message.
    ReturnsAMessage(models::ReceivedMessage),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// The specified resource was not found
    TheSpecifiedResourceWasNotFound,
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum MessagesSendMessageResponse {
    /// The message was sent successfully. NOTE: This does not imply successful delivery.
    TheMessageWasSentSuccessfully(String),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum MessagesWebsocketResponse {
    /// Switching protocols
    SwitchingProtocols,
    /// Incoming data
    IncomingData(String),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Not found
    NotFound,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum NodeGetEntryNodesResponse {
    /// Entry node information fetched successfuly.
    EntryNodeInformationFetchedSuccessfuly(
        std::collections::HashMap<String, models::NodeGetEntryNodes200ResponseValue>,
    ),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum NodeGetInfoResponse {
    /// Node information fetched successfuly.
    NodeInformationFetchedSuccessfuly(models::NodeGetInfo200Response),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum NodeGetMetricsResponse {
    /// Returns the encoded serialized metrics.
    ReturnsTheEncodedSerializedMetrics(String),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum NodeGetPeersResponse {
    /// Peers information fetched successfuly.
    PeersInformationFetchedSuccessfuly(models::NodeGetPeers200Response),
    /// Invalid input. One of the parameters passed is in an incorrect format.
    InvalidInput(models::RequestStatus),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum NodeGetVersionResponse {
    /// Returns the release version of the running node.
    ReturnsTheReleaseVersionOfTheRunningNode(String),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum PeerInfoGetPeerInfoResponse {
    /// Peer information fetched successfully.
    PeerInformationFetchedSuccessfully(models::PeerInfoGetPeerInfo200Response),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum PeersPingPeerResponse {
    /// Ping successful.
    PingSuccessful(models::PeersPingPeer200Response),
    /// Invalid peerId.
    InvalidPeerId(models::RequestStatus),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// An error occured (see error details) or timeout - node with specified PeerId didn't respond in time.
    AnErrorOccured(models::RequestStatus),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum SettingsGetSettingsResponse {
    /// Settings fetched succesfully.
    SettingsFetchedSuccesfully(models::Settings),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum SettingsSetSettingResponse {
    /// Setting set succesfully
    SettingSetSuccesfully,
    /// Invalid input. Either setting with that name doesn't exist or the value is incorrect.
    InvalidInput(models::RequestStatus),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum TicketsGetStatisticsResponse {
    /// Tickets statistics fetched successfully. Check schema for description of every field in the statistics.
    TicketsStatisticsFetchedSuccessfully(models::TicketsGetStatistics200Response),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum TicketsGetTicketsResponse {
    /// Tickets fetched successfully.
    TicketsFetchedSuccessfully(Vec<models::Ticket>),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum TicketsRedeemTicketsResponse {
    /// Tickets redeemed succesfully.
    TicketsRedeemedSuccesfully,
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum TokensCreateResponse {
    /// Token succesfully created.
    TokenSuccesfullyCreated(models::TokensCreate201Response),
    /// Problem with inputs.
    ProblemWithInputs(models::RequestStatus),
    /// Missing capability to access endpoint
    MissingCapabilityToAccessEndpoint,
    /// Unknown failure.
    UnknownFailure(models::TokensCreate422Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum TokensDeleteResponse {
    /// Token successfully deleted.
    TokenSuccessfullyDeleted,
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// The specified resource was not found
    TheSpecifiedResourceWasNotFound,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum TokensGetTokenResponse {
    /// Token information.
    TokenInformation(models::Token),
    /// authentication failed
    AuthenticationFailed(models::Error),
    /// authorization failed
    AuthorizationFailed(models::Error),
    /// The specified resource was not found
    TheSpecifiedResourceWasNotFound,
}

/// API
#[async_trait]
#[allow(clippy::too_many_arguments, clippy::ptr_arg)]
pub trait Api<C: Send + Sync> {
    fn poll_ready(&self, _cx: &mut Context) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>> {
        Poll::Ready(Ok(()))
    }

    async fn account_get_address(&self, context: &C) -> Result<AccountGetAddressResponse, ApiError>;

    async fn account_get_addresses(&self, context: &C) -> Result<AccountGetAddressesResponse, ApiError>;

    async fn account_get_balances(&self, context: &C) -> Result<AccountGetBalancesResponse, ApiError>;

    async fn account_withdraw(
        &self,
        account_withdraw_request: Option<models::AccountWithdrawRequest>,
        context: &C,
    ) -> Result<AccountWithdrawResponse, ApiError>;

    async fn aliases_get_alias(&self, alias: String, context: &C) -> Result<AliasesGetAliasResponse, ApiError>;

    async fn aliases_get_aliases(&self, context: &C) -> Result<AliasesGetAliasesResponse, ApiError>;

    async fn aliases_remove_alias(&self, alias: String, context: &C) -> Result<AliasesRemoveAliasResponse, ApiError>;

    async fn aliases_set_alias(
        &self,
        aliases_set_alias_request: Option<models::AliasesSetAliasRequest>,
        context: &C,
    ) -> Result<AliasesSetAliasResponse, ApiError>;

    async fn channels_aggregate_tickets(
        &self,
        channelid: String,
        context: &C,
    ) -> Result<ChannelsAggregateTicketsResponse, ApiError>;

    async fn channels_close_channel(
        &self,
        channelid: String,
        context: &C,
    ) -> Result<ChannelsCloseChannelResponse, ApiError>;

    async fn channels_fund_channel(
        &self,
        channelid: String,
        channels_fund_channel_request: Option<models::ChannelsFundChannelRequest>,
        context: &C,
    ) -> Result<ChannelsFundChannelResponse, ApiError>;

    async fn channels_get_channel(
        &self,
        channelid: serde_json::Value,
        context: &C,
    ) -> Result<ChannelsGetChannelResponse, ApiError>;

    async fn channels_get_channels(
        &self,
        including_closed: Option<String>,
        full_topology: Option<String>,
        context: &C,
    ) -> Result<ChannelsGetChannelsResponse, ApiError>;

    async fn channels_get_tickets(
        &self,
        channelid: String,
        context: &C,
    ) -> Result<ChannelsGetTicketsResponse, ApiError>;

    async fn channels_open_channel(
        &self,
        channels_open_channel_request: Option<models::ChannelsOpenChannelRequest>,
        context: &C,
    ) -> Result<ChannelsOpenChannelResponse, ApiError>;

    async fn channels_redeem_tickets(
        &self,
        channelid: String,
        context: &C,
    ) -> Result<ChannelsRedeemTicketsResponse, ApiError>;

    async fn check_node_healthy(&self, context: &C) -> Result<CheckNodeHealthyResponse, ApiError>;

    async fn check_node_ready(&self, context: &C) -> Result<CheckNodeReadyResponse, ApiError>;

    async fn check_node_started(&self, context: &C) -> Result<CheckNodeStartedResponse, ApiError>;

    async fn messages_delete_messages(&self, tag: i32, context: &C)
        -> Result<MessagesDeleteMessagesResponse, ApiError>;

    async fn messages_get_size(&self, tag: i32, context: &C) -> Result<MessagesGetSizeResponse, ApiError>;

    async fn messages_pop_all_message(
        &self,
        messages_pop_all_message_request: Option<models::MessagesPopAllMessageRequest>,
        context: &C,
    ) -> Result<MessagesPopAllMessageResponse, ApiError>;

    async fn messages_pop_message(
        &self,
        messages_pop_all_message_request: Option<models::MessagesPopAllMessageRequest>,
        context: &C,
    ) -> Result<MessagesPopMessageResponse, ApiError>;

    async fn messages_send_message(
        &self,
        messages_send_message_request: Option<models::MessagesSendMessageRequest>,
        context: &C,
    ) -> Result<MessagesSendMessageResponse, ApiError>;

    async fn messages_websocket(&self, context: &C) -> Result<MessagesWebsocketResponse, ApiError>;

    async fn node_get_entry_nodes(&self, context: &C) -> Result<NodeGetEntryNodesResponse, ApiError>;

    async fn node_get_info(&self, context: &C) -> Result<NodeGetInfoResponse, ApiError>;

    async fn node_get_metrics(&self, context: &C) -> Result<NodeGetMetricsResponse, ApiError>;

    async fn node_get_peers(&self, quality: Option<f64>, context: &C) -> Result<NodeGetPeersResponse, ApiError>;

    async fn node_get_version(&self, context: &C) -> Result<NodeGetVersionResponse, ApiError>;

    async fn peer_info_get_peer_info(
        &self,
        peerid: String,
        context: &C,
    ) -> Result<PeerInfoGetPeerInfoResponse, ApiError>;

    async fn peers_ping_peer(&self, peerid: String, context: &C) -> Result<PeersPingPeerResponse, ApiError>;

    async fn settings_get_settings(&self, context: &C) -> Result<SettingsGetSettingsResponse, ApiError>;

    async fn settings_set_setting(
        &self,
        setting: String,
        settings_set_setting_request: Option<models::SettingsSetSettingRequest>,
        context: &C,
    ) -> Result<SettingsSetSettingResponse, ApiError>;

    async fn tickets_get_statistics(&self, context: &C) -> Result<TicketsGetStatisticsResponse, ApiError>;

    async fn tickets_get_tickets(&self, context: &C) -> Result<TicketsGetTicketsResponse, ApiError>;

    async fn tickets_redeem_tickets(&self, context: &C) -> Result<TicketsRedeemTicketsResponse, ApiError>;

    async fn tokens_create(
        &self,
        tokens_create_request: Option<models::TokensCreateRequest>,
        context: &C,
    ) -> Result<TokensCreateResponse, ApiError>;

    async fn tokens_delete(&self, id: String, context: &C) -> Result<TokensDeleteResponse, ApiError>;

    async fn tokens_get_token(&self, context: &C) -> Result<TokensGetTokenResponse, ApiError>;
}

/// API where `Context` isn't passed on every API call
#[async_trait]
#[allow(clippy::too_many_arguments, clippy::ptr_arg)]
pub trait ApiNoContext<C: Send + Sync> {
    fn poll_ready(&self, _cx: &mut Context) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>>;

    fn context(&self) -> &C;

    async fn account_get_address(&self) -> Result<AccountGetAddressResponse, ApiError>;

    async fn account_get_addresses(&self) -> Result<AccountGetAddressesResponse, ApiError>;

    async fn account_get_balances(&self) -> Result<AccountGetBalancesResponse, ApiError>;

    async fn account_withdraw(
        &self,
        account_withdraw_request: Option<models::AccountWithdrawRequest>,
    ) -> Result<AccountWithdrawResponse, ApiError>;

    async fn aliases_get_alias(&self, alias: String) -> Result<AliasesGetAliasResponse, ApiError>;

    async fn aliases_get_aliases(&self) -> Result<AliasesGetAliasesResponse, ApiError>;

    async fn aliases_remove_alias(&self, alias: String) -> Result<AliasesRemoveAliasResponse, ApiError>;

    async fn aliases_set_alias(
        &self,
        aliases_set_alias_request: Option<models::AliasesSetAliasRequest>,
    ) -> Result<AliasesSetAliasResponse, ApiError>;

    async fn channels_aggregate_tickets(&self, channelid: String)
        -> Result<ChannelsAggregateTicketsResponse, ApiError>;

    async fn channels_close_channel(&self, channelid: String) -> Result<ChannelsCloseChannelResponse, ApiError>;

    async fn channels_fund_channel(
        &self,
        channelid: String,
        channels_fund_channel_request: Option<models::ChannelsFundChannelRequest>,
    ) -> Result<ChannelsFundChannelResponse, ApiError>;

    async fn channels_get_channel(&self, channelid: serde_json::Value) -> Result<ChannelsGetChannelResponse, ApiError>;

    async fn channels_get_channels(
        &self,
        including_closed: Option<String>,
        full_topology: Option<String>,
    ) -> Result<ChannelsGetChannelsResponse, ApiError>;

    async fn channels_get_tickets(&self, channelid: String) -> Result<ChannelsGetTicketsResponse, ApiError>;

    async fn channels_open_channel(
        &self,
        channels_open_channel_request: Option<models::ChannelsOpenChannelRequest>,
    ) -> Result<ChannelsOpenChannelResponse, ApiError>;

    async fn channels_redeem_tickets(&self, channelid: String) -> Result<ChannelsRedeemTicketsResponse, ApiError>;

    async fn check_node_healthy(&self) -> Result<CheckNodeHealthyResponse, ApiError>;

    async fn check_node_ready(&self) -> Result<CheckNodeReadyResponse, ApiError>;

    async fn check_node_started(&self) -> Result<CheckNodeStartedResponse, ApiError>;

    async fn messages_delete_messages(&self, tag: i32) -> Result<MessagesDeleteMessagesResponse, ApiError>;

    async fn messages_get_size(&self, tag: i32) -> Result<MessagesGetSizeResponse, ApiError>;

    async fn messages_pop_all_message(
        &self,
        messages_pop_all_message_request: Option<models::MessagesPopAllMessageRequest>,
    ) -> Result<MessagesPopAllMessageResponse, ApiError>;

    async fn messages_pop_message(
        &self,
        messages_pop_all_message_request: Option<models::MessagesPopAllMessageRequest>,
    ) -> Result<MessagesPopMessageResponse, ApiError>;

    async fn messages_send_message(
        &self,
        messages_send_message_request: Option<models::MessagesSendMessageRequest>,
    ) -> Result<MessagesSendMessageResponse, ApiError>;

    async fn messages_websocket(&self) -> Result<MessagesWebsocketResponse, ApiError>;

    async fn node_get_entry_nodes(&self) -> Result<NodeGetEntryNodesResponse, ApiError>;

    async fn node_get_info(&self) -> Result<NodeGetInfoResponse, ApiError>;

    async fn node_get_metrics(&self) -> Result<NodeGetMetricsResponse, ApiError>;

    async fn node_get_peers(&self, quality: Option<f64>) -> Result<NodeGetPeersResponse, ApiError>;

    async fn node_get_version(&self) -> Result<NodeGetVersionResponse, ApiError>;

    async fn peer_info_get_peer_info(&self, peerid: String) -> Result<PeerInfoGetPeerInfoResponse, ApiError>;

    async fn peers_ping_peer(&self, peerid: String) -> Result<PeersPingPeerResponse, ApiError>;

    async fn settings_get_settings(&self) -> Result<SettingsGetSettingsResponse, ApiError>;

    async fn settings_set_setting(
        &self,
        setting: String,
        settings_set_setting_request: Option<models::SettingsSetSettingRequest>,
    ) -> Result<SettingsSetSettingResponse, ApiError>;

    async fn tickets_get_statistics(&self) -> Result<TicketsGetStatisticsResponse, ApiError>;

    async fn tickets_get_tickets(&self) -> Result<TicketsGetTicketsResponse, ApiError>;

    async fn tickets_redeem_tickets(&self) -> Result<TicketsRedeemTicketsResponse, ApiError>;

    async fn tokens_create(
        &self,
        tokens_create_request: Option<models::TokensCreateRequest>,
    ) -> Result<TokensCreateResponse, ApiError>;

    async fn tokens_delete(&self, id: String) -> Result<TokensDeleteResponse, ApiError>;

    async fn tokens_get_token(&self) -> Result<TokensGetTokenResponse, ApiError>;
}

/// Trait to extend an API to make it easy to bind it to a context.
pub trait ContextWrapperExt<C: Send + Sync>
where
    Self: Sized,
{
    /// Binds this API to a context.
    fn with_context(self, context: C) -> ContextWrapper<Self, C>;
}

impl<T: Api<C> + Send + Sync, C: Clone + Send + Sync> ContextWrapperExt<C> for T {
    fn with_context(self: T, context: C) -> ContextWrapper<T, C> {
        ContextWrapper::<T, C>::new(self, context)
    }
}

#[async_trait]
impl<T: Api<C> + Send + Sync, C: Clone + Send + Sync> ApiNoContext<C> for ContextWrapper<T, C> {
    fn poll_ready(&self, cx: &mut Context) -> Poll<Result<(), ServiceError>> {
        self.api().poll_ready(cx)
    }

    fn context(&self) -> &C {
        ContextWrapper::context(self)
    }

    async fn account_get_address(&self) -> Result<AccountGetAddressResponse, ApiError> {
        let context = self.context().clone();
        self.api().account_get_address(&context).await
    }

    async fn account_get_addresses(&self) -> Result<AccountGetAddressesResponse, ApiError> {
        let context = self.context().clone();
        self.api().account_get_addresses(&context).await
    }

    async fn account_get_balances(&self) -> Result<AccountGetBalancesResponse, ApiError> {
        let context = self.context().clone();
        self.api().account_get_balances(&context).await
    }

    async fn account_withdraw(
        &self,
        account_withdraw_request: Option<models::AccountWithdrawRequest>,
    ) -> Result<AccountWithdrawResponse, ApiError> {
        let context = self.context().clone();
        self.api().account_withdraw(account_withdraw_request, &context).await
    }

    async fn aliases_get_alias(&self, alias: String) -> Result<AliasesGetAliasResponse, ApiError> {
        let context = self.context().clone();
        self.api().aliases_get_alias(alias, &context).await
    }

    async fn aliases_get_aliases(&self) -> Result<AliasesGetAliasesResponse, ApiError> {
        let context = self.context().clone();
        self.api().aliases_get_aliases(&context).await
    }

    async fn aliases_remove_alias(&self, alias: String) -> Result<AliasesRemoveAliasResponse, ApiError> {
        let context = self.context().clone();
        self.api().aliases_remove_alias(alias, &context).await
    }

    async fn aliases_set_alias(
        &self,
        aliases_set_alias_request: Option<models::AliasesSetAliasRequest>,
    ) -> Result<AliasesSetAliasResponse, ApiError> {
        let context = self.context().clone();
        self.api().aliases_set_alias(aliases_set_alias_request, &context).await
    }

    async fn channels_aggregate_tickets(
        &self,
        channelid: String,
    ) -> Result<ChannelsAggregateTicketsResponse, ApiError> {
        let context = self.context().clone();
        self.api().channels_aggregate_tickets(channelid, &context).await
    }

    async fn channels_close_channel(&self, channelid: String) -> Result<ChannelsCloseChannelResponse, ApiError> {
        let context = self.context().clone();
        self.api().channels_close_channel(channelid, &context).await
    }

    async fn channels_fund_channel(
        &self,
        channelid: String,
        channels_fund_channel_request: Option<models::ChannelsFundChannelRequest>,
    ) -> Result<ChannelsFundChannelResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .channels_fund_channel(channelid, channels_fund_channel_request, &context)
            .await
    }

    async fn channels_get_channel(&self, channelid: serde_json::Value) -> Result<ChannelsGetChannelResponse, ApiError> {
        let context = self.context().clone();
        self.api().channels_get_channel(channelid, &context).await
    }

    async fn channels_get_channels(
        &self,
        including_closed: Option<String>,
        full_topology: Option<String>,
    ) -> Result<ChannelsGetChannelsResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .channels_get_channels(including_closed, full_topology, &context)
            .await
    }

    async fn channels_get_tickets(&self, channelid: String) -> Result<ChannelsGetTicketsResponse, ApiError> {
        let context = self.context().clone();
        self.api().channels_get_tickets(channelid, &context).await
    }

    async fn channels_open_channel(
        &self,
        channels_open_channel_request: Option<models::ChannelsOpenChannelRequest>,
    ) -> Result<ChannelsOpenChannelResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .channels_open_channel(channels_open_channel_request, &context)
            .await
    }

    async fn channels_redeem_tickets(&self, channelid: String) -> Result<ChannelsRedeemTicketsResponse, ApiError> {
        let context = self.context().clone();
        self.api().channels_redeem_tickets(channelid, &context).await
    }

    async fn check_node_healthy(&self) -> Result<CheckNodeHealthyResponse, ApiError> {
        let context = self.context().clone();
        self.api().check_node_healthy(&context).await
    }

    async fn check_node_ready(&self) -> Result<CheckNodeReadyResponse, ApiError> {
        let context = self.context().clone();
        self.api().check_node_ready(&context).await
    }

    async fn check_node_started(&self) -> Result<CheckNodeStartedResponse, ApiError> {
        let context = self.context().clone();
        self.api().check_node_started(&context).await
    }

    async fn messages_delete_messages(&self, tag: i32) -> Result<MessagesDeleteMessagesResponse, ApiError> {
        let context = self.context().clone();
        self.api().messages_delete_messages(tag, &context).await
    }

    async fn messages_get_size(&self, tag: i32) -> Result<MessagesGetSizeResponse, ApiError> {
        let context = self.context().clone();
        self.api().messages_get_size(tag, &context).await
    }

    async fn messages_pop_all_message(
        &self,
        messages_pop_all_message_request: Option<models::MessagesPopAllMessageRequest>,
    ) -> Result<MessagesPopAllMessageResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .messages_pop_all_message(messages_pop_all_message_request, &context)
            .await
    }

    async fn messages_pop_message(
        &self,
        messages_pop_all_message_request: Option<models::MessagesPopAllMessageRequest>,
    ) -> Result<MessagesPopMessageResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .messages_pop_message(messages_pop_all_message_request, &context)
            .await
    }

    async fn messages_send_message(
        &self,
        messages_send_message_request: Option<models::MessagesSendMessageRequest>,
    ) -> Result<MessagesSendMessageResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .messages_send_message(messages_send_message_request, &context)
            .await
    }

    async fn messages_websocket(&self) -> Result<MessagesWebsocketResponse, ApiError> {
        let context = self.context().clone();
        self.api().messages_websocket(&context).await
    }

    async fn node_get_entry_nodes(&self) -> Result<NodeGetEntryNodesResponse, ApiError> {
        let context = self.context().clone();
        self.api().node_get_entry_nodes(&context).await
    }

    async fn node_get_info(&self) -> Result<NodeGetInfoResponse, ApiError> {
        let context = self.context().clone();
        self.api().node_get_info(&context).await
    }

    async fn node_get_metrics(&self) -> Result<NodeGetMetricsResponse, ApiError> {
        let context = self.context().clone();
        self.api().node_get_metrics(&context).await
    }

    async fn node_get_peers(&self, quality: Option<f64>) -> Result<NodeGetPeersResponse, ApiError> {
        let context = self.context().clone();
        self.api().node_get_peers(quality, &context).await
    }

    async fn node_get_version(&self) -> Result<NodeGetVersionResponse, ApiError> {
        let context = self.context().clone();
        self.api().node_get_version(&context).await
    }

    async fn peer_info_get_peer_info(&self, peerid: String) -> Result<PeerInfoGetPeerInfoResponse, ApiError> {
        let context = self.context().clone();
        self.api().peer_info_get_peer_info(peerid, &context).await
    }

    async fn peers_ping_peer(&self, peerid: String) -> Result<PeersPingPeerResponse, ApiError> {
        let context = self.context().clone();
        self.api().peers_ping_peer(peerid, &context).await
    }

    async fn settings_get_settings(&self) -> Result<SettingsGetSettingsResponse, ApiError> {
        let context = self.context().clone();
        self.api().settings_get_settings(&context).await
    }

    async fn settings_set_setting(
        &self,
        setting: String,
        settings_set_setting_request: Option<models::SettingsSetSettingRequest>,
    ) -> Result<SettingsSetSettingResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .settings_set_setting(setting, settings_set_setting_request, &context)
            .await
    }

    async fn tickets_get_statistics(&self) -> Result<TicketsGetStatisticsResponse, ApiError> {
        let context = self.context().clone();
        self.api().tickets_get_statistics(&context).await
    }

    async fn tickets_get_tickets(&self) -> Result<TicketsGetTicketsResponse, ApiError> {
        let context = self.context().clone();
        self.api().tickets_get_tickets(&context).await
    }

    async fn tickets_redeem_tickets(&self) -> Result<TicketsRedeemTicketsResponse, ApiError> {
        let context = self.context().clone();
        self.api().tickets_redeem_tickets(&context).await
    }

    async fn tokens_create(
        &self,
        tokens_create_request: Option<models::TokensCreateRequest>,
    ) -> Result<TokensCreateResponse, ApiError> {
        let context = self.context().clone();
        self.api().tokens_create(tokens_create_request, &context).await
    }

    async fn tokens_delete(&self, id: String) -> Result<TokensDeleteResponse, ApiError> {
        let context = self.context().clone();
        self.api().tokens_delete(id, &context).await
    }

    async fn tokens_get_token(&self) -> Result<TokensGetTokenResponse, ApiError> {
        let context = self.context().clone();
        self.api().tokens_get_token(&context).await
    }
}

#[cfg(feature = "client")]
pub mod client;

// Re-export Client as a top-level name
#[cfg(feature = "client")]
pub use client::Client;

#[cfg(feature = "server")]
pub mod server;

// Re-export router() as a top-level name
#[cfg(feature = "server")]
pub use self::server::Service;

#[cfg(feature = "server")]
pub mod context;

pub mod models;

#[cfg(any(feature = "client", feature = "server"))]
pub(crate) mod header;
