use axum::{
    extract::{Json, Path, Query, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use futures::TryFutureExt;
use libp2p_identity::PeerId;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::sync::Arc;
use tracing::warn;

use hopr_crypto_types::types::Hash;
use hopr_lib::{
    errors::{HoprLibError, HoprStatusError},
    Address, AsUnixTimestamp, Balance, BalanceType, ChainActionsError, ChannelEntry, ChannelStatus, Hopr, ToHex,
};

use crate::{
    checksum_address_serializer,
    types::{HoprIdentifier, PeerOrAddress},
    ApiError, ApiErrorStatus, InternalState, BASE_PATH,
};

#[serde_as]
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NodeChannel {
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    id: Hash,
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String)]
    peer_address: Address,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    status: ChannelStatus,
    balance: String,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "balance": "10000000000000000000",
        "channelEpoch": 1,
        "channelId": "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f",
        "closureTime": 0,
        "destinationAddress": "0x188c4462b75e46f0c7262d7f48d182447b93a93c",
        "destinationPeerId": "12D3KooWPWD5P5ZzMRDckgfVaicY5JNoo7JywGotoAv17d7iKx1z",
        "sourceAddress": "0x07eaf07d6624f741e04f4092a755a9027aaab7f6",
        "sourcePeerId": "12D3KooWJmLm8FnBfvYQ5BAZ5qcYBxQFFBzAAEYUBUNJNE8cRsYS",
        "status": "Open",
        "ticketIndex": 0
    }))]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChannelInfoResponse {
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    channel_id: Hash,
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String)]
    source_address: Address,
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String)]
    destination_address: Address,
    source_peer_id: String,
    destination_peer_id: String,
    balance: String,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    status: ChannelStatus,
    ticket_index: u32,
    channel_epoch: u32,
    closure_time: u64,
}

/// Listing of channels.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "all": [{
            "channelId": "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f",
            "sourceAddress": "0x07eaf07d6624f741e04f4092a755a9027aaab7f6",
            "destinationAddress": "0x188c4462b75e46f0c7262d7f48d182447b93a93c",
            "sourcePeerId": "12D3KooWJmLm8FnBfvYQ5BAZ5qcYBxQFFBzAAEYUBUNJNE8cRsYS",
            "destinationPeerId": "12D3KooWPWD5P5ZzMRDckgfVaicY5JNoo7JywGotoAv17d7iKx1z",
            "balance": "10000000000000000000",
            "status": "Open",
            "ticketIndex": 0,
            "channelEpoch": 1,
            "closureTime": 0
        }],
        "incoming": [],
        "outgoing": [{
            "balance": "10000000000000000010",
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

async fn query_topology_info(channel: &ChannelEntry, node: &Hopr) -> Result<ChannelInfoResponse, HoprLibError> {
    Ok(ChannelInfoResponse {
        channel_id: channel.get_id(),
        source_address: channel.source,
        destination_address: channel.destination,
        source_peer_id: node
            .chain_key_to_peerid(&channel.source)
            .await?
            .map(|v| PeerId::to_string(&v))
            .unwrap_or_else(|| {
                warn!(address = %channel.source, "failed to map address to PeerId", );
                "<FAILED_TO_MAP_THE_PEERID>".into()
            }),
        destination_peer_id: node
            .chain_key_to_peerid(&channel.destination)
            .await?
            .map(|v| PeerId::to_string(&v))
            .unwrap_or_else(|| {
                warn!(address = %channel.destination, "failed to map address to PeerId", );
                "<FAILED_TO_MAP_THE_PEERID>".into()
            }),
        balance: channel.balance.amount().to_string(),
        status: channel.status,
        ticket_index: channel.ticket_index.as_u32(),
        channel_epoch: channel.channel_epoch.as_u32(),
        closure_time: channel
            .closure_time_at()
            .map(|ct| ct.as_unix_timestamp().as_secs())
            .unwrap_or_default(),
    })
}

/// Parameters for enumerating channels.
#[derive(Debug, Default, Copy, Clone, Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
#[into_params(parameter_in = Query)]
#[serde(default, rename_all = "camelCase")]
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
        let hopr_clone = hopr.clone();
        let topology = hopr
            .all_channels()
            .and_then(|channels| async move {
                futures::future::try_join_all(channels.iter().map(|c| query_topology_info(c, hopr_clone.as_ref())))
                    .await
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
                            balance: c.balance.amount().to_string(),
                        })
                        .collect(),
                    outgoing: outgoing
                        .into_iter()
                        .filter(|c| query.including_closed || c.status != ChannelStatus::Closed)
                        .map(|c| NodeChannel {
                            id: c.get_id(),
                            peer_address: c.destination,
                            status: c.status,
                            balance: c.balance.amount().to_string(),
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
        "amount": "10",
        "destination": "0xa8194d36e322592d4c707b70dbe96121f5c74c64"
    }))]
pub(crate) struct OpenChannelBodyRequest {
    /// On-chain address of the counterparty.
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    destination: PeerOrAddress,
    /// Initial amount of stake in HOPR tokens.
    amount: String,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "channelId": "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f",
        "transactionReceipt": "0x5181ac24759b8e01b3c932e4636c3852f386d17517a8dfc640a5ba6f2258f29c"
    }))]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenChannelResponse {
    /// ID of the new channel.
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    channel_id: Hash,
    /// Receipt of the channel open transaction.
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    transaction_receipt: Hash,
}

/// Opens a channel to the given on-chain address with the given initial stake of HOPR tokens.
#[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/channels"),
        request_body(
            content = OpenChannelBodyRequest,
            description = "Open channel request specification: on-chain address of the counterparty and the initial HOPR token stake.",
            content_type = "application/json"),
        responses(
            (status = 201, description = "Channel successfully opened.", body = OpenChannelResponse),
            (status = 400, description = "Invalid counterparty address or stake amount.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 403, description = "Failed to open the channel because of insufficient HOPR balance or allowance.", body = ApiError),
            (status = 409, description = "Failed to open the channel because the channel between this nodes already exists.", body = ApiError),
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

    let address = match HoprIdentifier::new_with(open_req.destination, hopr.peer_resolver()).await {
        Ok(destination) => destination.address,
        Err(e) => return (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
    };

    match hopr
        .open_channel(&address, &Balance::new_from_str(&open_req.amount, BalanceType::HOPR))
        .await
    {
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
        Err(HoprLibError::StatusError(HoprStatusError::NotThereYet(_, _))) => {
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
        params(
            ("channelId" = String, Path, description = "ID of the channel.")
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
                let info = query_topology_info(&channel, hopr.as_ref()).await;
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
pub(crate) struct CloseChannelResponse {
    /// Receipt for the channel close transaction.
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    receipt: Hash,
    /// New status of the channel. Will be one of `Closed` or `PendingToClose`.
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    channel_status: ChannelStatus,
}

/// Closes the given channel.
///
/// If the channel is currently `Open`, it will transition it to `PendingToClose`.
/// If the channels is in `PendingToClose` and the channel closure period has elapsed,
/// it will transition it to `Closed`.
#[utoipa::path(
        delete,
        path = const_format::formatcp!("{BASE_PATH}/channels/{{channelId}}"),
        params(
            ("channelId" = String, Path, description = "ID of the channel.")
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
            Err(HoprLibError::StatusError(HoprStatusError::NotThereYet(_, _))) => {
                (StatusCode::PRECONDITION_FAILED, ApiErrorStatus::NotReady).into_response()
            }
            Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
        },
        Err(_) => (StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidChannelId).into_response(),
    }
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
        "direction": "outgoing",
        "status": "Open"
    }))]
pub(crate) struct CloseMultipleBodyRequest {
    /// Direction of the channels to close.
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    direction: hopr_lib::ChannelDirection,

    /// Status of the channels to close.
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    status: hopr_lib::ChannelStatus,
}

/// Closes multiple channels in a single call.
#[utoipa::path(
        delete,
        path = const_format::formatcp!("{BASE_PATH}/channels"),
        request_body(
            content = CloseMultipleBodyRequest,
            description = "Specifies the direction and status of the channels to close.",
            content_type = "application/json",
        ),
        responses(
            (status = 200, description = "Channels closed successfully", body = String),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 412, description = "The node is not ready."),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Channels",
    )]

pub(super) async fn close_multiple_channels(
    State(state): State<Arc<InternalState>>,
    Json(req_body): Json<CloseMultipleBodyRequest>,
) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    let direction = req_body.direction;
    let status = req_body.status;

    match hopr.close_multiple_channels(direction, status, false).await {
        Ok(_) => (
            StatusCode::OK,
            Json(format!("All {} {} channels closed successfully", direction, status)),
        )
            .into_response(),
        Err(HoprLibError::StatusError(HoprStatusError::NotThereYet(_, _))) => {
            (StatusCode::PRECONDITION_FAILED, ApiErrorStatus::NotReady).into_response()
        }
        Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }
}

/// Specifies the amount of HOPR tokens to fund a channel with.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
        "amount": "10000000000000000000"
    }))]
pub(crate) struct FundBodyRequest {
    /// Amount of HOPR tokens to fund the channel with.
    amount: String,
}

/// Funds the given channel with the given amount of HOPR tokens.
#[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/channels/{{channelId}}/fund"),
        params(
            ("channelId" = String, Path, description = "ID of the channel.")
        ),
        request_body(
            content = FundBodyRequest,
            description = "Specifies the amount of HOPR tokens to fund a channel with.",
            content_type = "application/json",
        ),
        responses(
            (status = 200, description = "Channel funded successfully", body = String),
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

    let amount = Balance::new_from_str(&fund_req.amount, BalanceType::HOPR);

    match Hash::from_hex(channel_id.as_str()) {
        Ok(channel_id) => match hopr.fund_channel(&channel_id, &amount).await {
            Ok(hash) => (StatusCode::OK, Json(hash.to_hex())).into_response(),
            Err(HoprLibError::ChainError(ChainActionsError::ChannelDoesNotExist)) => {
                (StatusCode::NOT_FOUND, ApiErrorStatus::ChannelNotFound).into_response()
            }
            Err(HoprLibError::ChainError(ChainActionsError::NotEnoughAllowance)) => {
                (StatusCode::FORBIDDEN, ApiErrorStatus::NotEnoughAllowance).into_response()
            }
            Err(HoprLibError::ChainError(ChainActionsError::BalanceTooLow)) => {
                (StatusCode::FORBIDDEN, ApiErrorStatus::NotEnoughBalance).into_response()
            }
            Err(HoprLibError::StatusError(HoprStatusError::NotThereYet(_, _))) => {
                (StatusCode::PRECONDITION_FAILED, ApiErrorStatus::NotReady).into_response()
            }
            Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
        },
        Err(_) => (StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidChannelId).into_response(),
    }
}
