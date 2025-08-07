use std::sync::Arc;

use axum::{
    extract::{Json, Path, Query, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use futures::TryFutureExt;
use hopr_crypto_types::types::Hash;
use hopr_lib::{
    Address, AsUnixTimestamp, ChainActionsError, ChannelEntry, ChannelStatus, HoprBalance, ToHex,
    errors::{HoprLibError, HoprStatusError},
};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};

use crate::{ApiError, ApiErrorStatus, BASE_PATH, InternalState, checksum_address_serializer};

#[serde_as]
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "id": "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f",
    "address": "0x188c4462b75e46f0c7262d7f48d182447b93a93c",
    "status": "Open",
    "balance": "10 wxHOPR"
}))]
/// Channel information as seen by the node.
pub(crate) struct NodeChannel {
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f")]
    id: Hash,
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String, example = "0x188c4462b75e46f0c7262d7f48d182447b93a93c")]
    peer_address: Address,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "Open")]
    status: ChannelStatus,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "10 wxHOPR")]
    balance: HoprBalance,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "balance": "10 wxHOPR",
        "channelEpoch": 1,
        "channelId": "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f",
        "closureTime": 0,
        "destination": "0x188c4462b75e46f0c7262d7f48d182447b93a93c",
        "source": "0x07eaf07d6624f741e04f4092a755a9027aaab7f6",
        "status": "Open",
        "ticketIndex": 0
    }))]
#[serde(rename_all = "camelCase")]
/// General information about a channel state.
pub(crate) struct ChannelInfoResponse {
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f")]
    channel_id: Hash,
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String, example = "0x07eaf07d6624f741e04f4092a755a9027aaab7f6")]
    source: Address,
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String, example = "0x188c4462b75e46f0c7262d7f48d182447b93a93c")]
    destination: Address,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "10 wxHOPR")]
    balance: HoprBalance,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "Open")]
    status: ChannelStatus,
    #[schema(example = 0)]
    ticket_index: u32,
    #[schema(example = 1)]
    channel_epoch: u32,
    #[schema(example = 0)]
    closure_time: u64,
}

/// Listing of channels.
#[serde_as]
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "all": [{
            "channelId": "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f",
            "source": "0x07eaf07d6624f741e04f4092a755a9027aaab7f6",
            "destination": "0x188c4462b75e46f0c7262d7f48d182447b93a93c",
            "balance": "10 wxHOPR",
            "status": "Open",
            "ticketIndex": 0,
            "channelEpoch": 1,
            "closureTime": 0
        }],
        "incoming": [],
        "outgoing": [{
            "balance": "10 wxHOPR",
            "id": "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f",
            "peerAddress": "0x188c4462b75e46f0c7262d7f48d182447b93a93c",
            "status": "Open"
        }]
    }))]
pub(crate) struct NodeChannelsResponse {
    /// Channels incoming to this node.
    incoming: Vec<NodeChannel>,
    /// Channels outgoing from this node.
    outgoing: Vec<NodeChannel>,
    /// Complete channel topology as seen by this node.
    all: Vec<ChannelInfoResponse>,
}

async fn query_topology_info(channel: ChannelEntry) -> Result<ChannelInfoResponse, HoprLibError> {
    Ok(ChannelInfoResponse {
        channel_id: channel.get_id(),
        source: channel.source,
        destination: channel.destination,
        balance: channel.balance,
        status: channel.status,
        ticket_index: channel.ticket_index.as_u32(),
        channel_epoch: channel.channel_epoch.as_u32(),
        closure_time: channel
            .closure_time_at()
            .map(|ct| ct.as_unix_timestamp().as_secs())
            .unwrap_or_default(),
    })
}

#[derive(Debug, Default, Copy, Clone, Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
#[into_params(parameter_in = Query)]
#[serde(default, rename_all = "camelCase")]
#[schema(example = json!({
        "includingClosed": true,
        "fullTopology": false
    }))]
/// Parameters for enumerating channels.
pub(crate) struct ChannelsQueryRequest {
    /// Should be the closed channels included?
    #[schema(required = false)]
    #[serde(default)]
    including_closed: bool,
    /// Should all channels (not only the ones concerning this node) be enumerated?
    #[schema(required = false)]
    #[serde(default)]
    full_topology: bool,
}

/// Lists channels opened to/from this node. Alternatively, it can print all
/// the channels in the network as this node sees them.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/channels"),
        description = "List channels opened to/from this node. Alternatively, it can print all the channels in the network as this node sees them.",
        params(ChannelsQueryRequest),
        responses(
            (status = 200, description = "Channels fetched successfully", body = NodeChannelsResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Channels",
    )]
pub(super) async fn list_channels(
    Query(query): Query<ChannelsQueryRequest>,
    State(state): State<Arc<InternalState>>,
) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    if query.full_topology {
        let topology = hopr
            .all_channels()
            .and_then(|channels| async move {
                futures::future::try_join_all(channels.into_iter().map(query_topology_info)).await
            })
            .await;

        match topology {
            Ok(all) => (
                StatusCode::OK,
                Json(NodeChannelsResponse {
                    incoming: vec![],
                    outgoing: vec![],
                    all,
                }),
            )
                .into_response(),
            Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
        }
    } else {
        let channels = hopr
            .channels_to(&hopr.me_onchain())
            .and_then(|incoming| async {
                let outgoing = hopr.channels_from(&hopr.me_onchain()).await?;
                Ok((incoming, outgoing))
            })
            .await;

        match channels {
            Ok((incoming, outgoing)) => {
                let channel_info = NodeChannelsResponse {
                    incoming: incoming
                        .into_iter()
                        .filter(|c| query.including_closed || c.status != ChannelStatus::Closed)
                        .map(|c| NodeChannel {
                            id: c.get_id(),
                            peer_address: c.source,
                            status: c.status,
                            balance: c.balance,
                        })
                        .collect(),
                    outgoing: outgoing
                        .into_iter()
                        .filter(|c| query.including_closed || c.status != ChannelStatus::Closed)
                        .map(|c| NodeChannel {
                            id: c.get_id(),
                            peer_address: c.destination,
                            status: c.status,
                            balance: c.balance,
                        })
                        .collect(),
                    all: vec![],
                };

                (StatusCode::OK, Json(channel_info)).into_response()
            }
            Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
        "amount": "10 wxHOPR",
        "destination": "0xa8194d36e322592d4c707b70dbe96121f5c74c64"
    }))]
/// Request body for opening a channel.
pub(crate) struct OpenChannelBodyRequest {
    /// On-chain address of the counterparty.
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "0xa8194d36e322592d4c707b70dbe96121f5c74c64")]
    destination: Address,
    /// Initial amount of stake in HOPR tokens.
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "10 wxHOPR")]
    amount: HoprBalance,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "channelId": "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f",
        "transactionReceipt": "0x5181ac24759b8e01b3c932e4636c3852f386d17517a8dfc640a5ba6f2258f29c"
    }))]
#[serde(rename_all = "camelCase")]
/// Response body for opening a channel.
pub(crate) struct OpenChannelResponse {
    /// ID of the new channel.
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f")]
    channel_id: Hash,
    /// Receipt of the channel open transaction.
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "0x5181ac24759b8e01b3c932e4636c3852f386d17517a8dfc640a5ba6f2258f29c")]
    transaction_receipt: Hash,
}

/// Opens a channel to the given on-chain address with the given initial stake of HOPR tokens.
#[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/channels"),
        description = "Opens a channel to the given on-chain address with the given initial stake of HOPR tokens.",
        request_body(
            content = OpenChannelBodyRequest,
            description = "Open channel request specification: on-chain address of the counterparty and the initial HOPR token stake.",
            content_type = "application/json"),
        responses(
            (status = 201, description = "Channel successfully opened.", body = OpenChannelResponse),
            (status = 400, description = "Invalid counterparty address or stake amount.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 403, description = "Failed to open the channel because of insufficient HOPR balance or allowance.", body = ApiError),
            (status = 409, description = "Failed to open the channel because the channel between these nodes already exists.", body = ApiError),
            (status = 412, description = "The node is not ready."),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Channels",
    )]
pub(super) async fn open_channel(
    State(state): State<Arc<InternalState>>,
    Json(open_req): Json<OpenChannelBodyRequest>,
) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    match hopr.open_channel(&open_req.destination, open_req.amount).await {
        Ok(channel_details) => (
            StatusCode::CREATED,
            Json(OpenChannelResponse {
                channel_id: channel_details.channel_id,
                transaction_receipt: channel_details.tx_hash,
            }),
        )
            .into_response(),
        Err(HoprLibError::ChainError(ChainActionsError::BalanceTooLow)) => {
            (StatusCode::FORBIDDEN, ApiErrorStatus::NotEnoughBalance).into_response()
        }
        Err(HoprLibError::ChainError(ChainActionsError::NotEnoughAllowance)) => {
            (StatusCode::FORBIDDEN, ApiErrorStatus::NotEnoughAllowance).into_response()
        }
        Err(HoprLibError::ChainError(ChainActionsError::ChannelAlreadyExists)) => {
            (StatusCode::CONFLICT, ApiErrorStatus::ChannelAlreadyOpen).into_response()
        }
        Err(HoprLibError::StatusError(HoprStatusError::NotThereYet(..))) => {
            (StatusCode::PRECONDITION_FAILED, ApiErrorStatus::NotReady).into_response()
        }
        Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "channelId": "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f"
}))]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChannelIdParams {
    channel_id: String,
}

/// Returns information about the given channel.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/channels/{{channelId}}"),
        description = "Returns information about the given channel.",
        params(
            ("channelId" = String, Path, description = "ID of the channel.", example = "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f")
        ),
        responses(
            (status = 200, description = "Channel fetched successfully", body = ChannelInfoResponse),
            (status = 400, description = "Invalid channel id.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "Channel not found.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Channels",
    )]
pub(super) async fn show_channel(
    Path(ChannelIdParams { channel_id }): Path<ChannelIdParams>,
    State(state): State<Arc<InternalState>>,
) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    match Hash::from_hex(channel_id.as_str()) {
        Ok(channel_id) => match hopr.channel_from_hash(&channel_id).await {
            Ok(Some(channel)) => {
                let info = query_topology_info(channel).await;
                match info {
                    Ok(info) => (StatusCode::OK, Json(info)).into_response(),
                    Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
                }
            }
            Ok(None) => (StatusCode::NOT_FOUND, ApiErrorStatus::ChannelNotFound).into_response(),
            Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
        },
        Err(_) => (StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidChannelId).into_response(),
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "receipt": "0xd77da7c1821249e663dead1464d185c03223d9663a06bc1d46ed0ad449a07118",
        "channelStatus": "PendingToClose"
    }))]
#[serde(rename_all = "camelCase")]
/// Status of the channel after a close operation.
pub(crate) struct CloseChannelResponse {
    /// Receipt for the channel close transaction.
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "0xd77da7c1821249e663dead1464d185c03223d9663a06bc1d46ed0ad449a07118")]
    receipt: Hash,
    /// New status of the channel. Will be one of `Closed` or `PendingToClose`.
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "PendingToClose")]
    channel_status: ChannelStatus,
}

/// Closes the given channel.
///
/// If the channel is currently `Open`, it will transition it to `PendingToClose`.
/// If the channels are in `PendingToClose` and the channel closure period has elapsed,
/// it will transition it to `Closed`.
#[utoipa::path(
        delete,
        path = const_format::formatcp!("{BASE_PATH}/channels/{{channelId}}"),
        description = "Closes the given channel.",
        params(
            ("channelId" = String, Path, description = "ID of the channel.", example = "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f")
        ),
        responses(
            (status = 200, description = "Channel closed successfully", body = CloseChannelResponse),
            (status = 400, description = "Invalid channel id.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "Channel not found.", body = ApiError),
            (status = 412, description = "The node is not ready."),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Channels",
    )]
pub(super) async fn close_channel(
    Path(ChannelIdParams { channel_id }): Path<ChannelIdParams>,
    State(state): State<Arc<InternalState>>,
) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    match Hash::from_hex(channel_id.as_str()) {
        Ok(channel_id) => match hopr.close_channel_by_id(channel_id, false).await {
            Ok(receipt) => (
                StatusCode::OK,
                Json(CloseChannelResponse {
                    channel_status: receipt.status,
                    receipt: receipt.tx_hash,
                }),
            )
                .into_response(),
            Err(HoprLibError::ChainError(ChainActionsError::ChannelDoesNotExist)) => {
                (StatusCode::NOT_FOUND, ApiErrorStatus::ChannelNotFound).into_response()
            }
            Err(HoprLibError::ChainError(ChainActionsError::InvalidArguments(_))) => {
                (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::UnsupportedFeature).into_response()
            }
            Err(HoprLibError::StatusError(HoprStatusError::NotThereYet(..))) => {
                (StatusCode::PRECONDITION_FAILED, ApiErrorStatus::NotReady).into_response()
            }
            Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
        },
        Err(_) => (StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidChannelId).into_response(),
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "hash": "0x188c4462b75e46f0c7262d7f48d182447b93a93c",
}))]
#[serde(rename_all = "camelCase")]
/// Response body for funding a channel.
pub(crate) struct FundChannelResponse {
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    hash: Hash,
}

/// Specifies the amount of HOPR tokens to fund a channel with.
#[serde_as]
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
        "amount": "10 wxHOPR"
    }))]
pub(crate) struct FundBodyRequest {
    /// Amount of HOPR tokens to fund the channel with.
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "10 wxHOPR")]
    amount: HoprBalance,
}

/// Funds the given channel with the given amount of HOPR tokens.
#[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/channels/{{channelId}}/fund"),
        description = "Funds the given channel with the given amount of HOPR tokens.",
        params(
            ("channelId" = String, Path, description = "ID of the channel.", example = "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f")
        ),
        request_body(
            content = FundBodyRequest,
            description = "Specifies the amount of HOPR tokens to fund a channel with.",
            content_type = "application/json",
        ),
        responses(
            (status = 200, description = "Channel funded successfully", body = FundChannelResponse),
            (status = 400, description = "Invalid channel id.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 403, description = "Failed to fund the channel because of insufficient HOPR balance or allowance.", body = ApiError),
            (status = 404, description = "Channel not found.", body = ApiError),
            (status = 412, description = "The node is not ready."),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Channels",
    )]
pub(super) async fn fund_channel(
    Path(ChannelIdParams { channel_id }): Path<ChannelIdParams>,
    State(state): State<Arc<InternalState>>,
    Json(fund_req): Json<FundBodyRequest>,
) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    match Hash::from_hex(channel_id.as_str()) {
        Ok(channel_id) => match hopr.fund_channel(&channel_id, fund_req.amount).await {
            Ok(hash) => (StatusCode::OK, Json(FundChannelResponse { hash })).into_response(),
            Err(HoprLibError::ChainError(ChainActionsError::ChannelDoesNotExist)) => {
                (StatusCode::NOT_FOUND, ApiErrorStatus::ChannelNotFound).into_response()
            }
            Err(HoprLibError::ChainError(ChainActionsError::NotEnoughAllowance)) => {
                (StatusCode::FORBIDDEN, ApiErrorStatus::NotEnoughAllowance).into_response()
            }
            Err(HoprLibError::ChainError(ChainActionsError::BalanceTooLow)) => {
                (StatusCode::FORBIDDEN, ApiErrorStatus::NotEnoughBalance).into_response()
            }
            Err(HoprLibError::StatusError(HoprStatusError::NotThereYet(..))) => {
                (StatusCode::PRECONDITION_FAILED, ApiErrorStatus::NotReady).into_response()
            }
            Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
        },
        Err(_) => (StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidChannelId).into_response(),
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "channelsIds": ["0x188c4462b75e46f0c7262d7f48d182447b93a93c"],
}))]
#[serde(rename_all = "camelCase")]
/// Response body for funding a channel.
pub(crate) struct CorruptedChannelsResponse {
    #[schema(value_type = String)]
    channels_ids: Vec<Hash>,
}

#[utoipa::path(
    get,
    path = const_format::formatcp!("{BASE_PATH}/channels/corrupted"),
    description = "List corrupted channels due to incorrect indexing.",
    responses(
        (status = 200, description = "Corrupted channels retrieved", body = Vec<String>),
        (status = 401, description = "Invalid authorization token.", body = ApiError),
        (status = 422, description = "Unknown failure", body = ApiError)
    ),
    security(
        ("api_token" = []),
        ("bearer_token" = [])
    ),
    tag = "Channels",
)]
pub(super) async fn corrupted_channels(State(state): State<Arc<InternalState>>) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    let corrupted = match hopr.corrupted_channels().await {
        Ok(corrupted) => corrupted,
        Err(e) => return (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    };

    let channels_ids: Vec<Hash> = corrupted.into_iter().map(|c| c.channel().get_id()).collect();

    (StatusCode::OK, Json(CorruptedChannelsResponse { channels_ids })).into_response()
}
