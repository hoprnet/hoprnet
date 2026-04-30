use std::sync::Arc;

use axum::{
    extract::{Json, Path, Query, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use futures::{StreamExt, TryFutureExt};
use hopr_lib::{
    api::{
        chain::{ChainReadChannelOperations, ChannelSelector},
        node::{HasChainApi, IncentiveChannelOperations},
        types::{
            crypto::prelude::Hash,
            internal::prelude::{ChannelEntry, ChannelStatus},
            primitive::prelude::{Address, AsUnixTimestamp, HoprBalance},
        },
    },
    errors::HoprLibError,
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
    ticket_index: u64,
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

fn channel_to_topology_info(channel: &ChannelEntry) -> ChannelInfoResponse {
    ChannelInfoResponse {
        channel_id: *channel.get_id(),
        source: channel.source,
        destination: channel.destination,
        balance: channel.balance,
        status: channel.status,
        ticket_index: channel.ticket_index,
        channel_epoch: channel.channel_epoch,
        closure_time: channel
            .closure_time_at()
            .map(|ct| ct.as_unix_timestamp().as_secs())
            .unwrap_or_default(),
    }
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
pub(super) async fn list_channels<
    H: HasChainApi<ChainError = hopr_lib::errors::HoprLibError> + Send + Sync + 'static,
>(
    Query(query): Query<ChannelsQueryRequest>,
    State(state): State<Arc<InternalState<H>>>,
) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    if query.full_topology {
        let topology = hopr
            .chain_api()
            .stream_channels(ChannelSelector::default())
            .map(|stream| stream.collect::<Vec<_>>());

        match topology {
            Ok(stream) => {
                let all = stream.await.iter().map(channel_to_topology_info).collect::<Vec<_>>();
                (
                    StatusCode::OK,
                    Json(NodeChannelsResponse {
                        incoming: vec![],
                        outgoing: vec![],
                        all,
                    }),
                )
                    .into_response()
            }
            Err(e) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ApiErrorStatus::from(HoprLibError::chain(e)),
            )
                .into_response(),
        }
    } else {
        let me = hopr.identity().node_address;
        let channels = hopr
            .channels_to(me)
            .and_then(|incoming| async {
                let outgoing = hopr.channels_from(me).await?;
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
                            id: *c.get_id(),
                            peer_address: c.source,
                            status: c.status,
                            balance: c.balance,
                        })
                        .collect(),
                    outgoing: outgoing
                        .into_iter()
                        .filter(|c| query.including_closed || c.status != ChannelStatus::Closed)
                        .map(|c| NodeChannel {
                            id: *c.get_id(),
                            peer_address: c.destination,
                            status: c.status,
                            balance: c.balance,
                        })
                        .collect(),
                    all: vec![],
                };

                (StatusCode::OK, Json(channel_info)).into_response()
            }
            Err(e) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ApiErrorStatus::UnknownFailure(e.to_string()),
            )
                .into_response(),
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
pub(super) async fn open_channel<
    H: HasChainApi<ChainError = hopr_lib::errors::HoprLibError> + Send + Sync + 'static,
>(
    State(state): State<Arc<InternalState<H>>>,
    Json(open_req): Json<OpenChannelBodyRequest>,
) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    match hopr.open_channel(open_req.destination, open_req.amount).await {
        Ok(channel_details) => (
            StatusCode::CREATED,
            Json(OpenChannelResponse {
                channel_id: channel_details.output().copied().unwrap_or_default(),
                transaction_receipt: *channel_details.tx_hash(),
            }),
        )
            .into_response(),
        Err(hopr_lib::api::node::EitherErr::Right(HoprLibError::NotReady(..))) => {
            (StatusCode::PRECONDITION_FAILED, ApiErrorStatus::NotReady).into_response()
        }
        Err(e) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiErrorStatus::UnknownFailure(e.to_string()),
        )
            .into_response(),
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AddressParams {
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    address: Address,
}

/// Direction of a channel relative to this node.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ChannelDirection {
    /// Channel this node is the source of (this node → counterparty).
    #[default]
    Outgoing,
    /// Channel this node is the destination of (counterparty → this node).
    Incoming,
}

/// Query parameters selecting which channel (incoming or outgoing) to address.
#[derive(Debug, Clone, Default, Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
#[into_params(parameter_in = Query)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct ChannelDirectionQuery {
    /// Direction of the channel relative to this node. Defaults to `outgoing`.
    #[schema(required = false, example = "outgoing")]
    #[serde(default)]
    direction: ChannelDirection,
}

/// Filters a channel lookup result to find a non-closed channel.
///
/// Returns `ChannelNotFound` when the channel is absent or closed,
/// and `UnknownFailure` for lookup errors.
fn filter_open_channel<E: std::error::Error>(
    result: Result<Option<ChannelEntry>, E>,
) -> Result<ChannelEntry, ApiErrorStatus> {
    match result {
        Ok(Some(ch)) if ch.status != ChannelStatus::Closed => Ok(ch),
        Ok(_) => Err(ApiErrorStatus::ChannelNotFound),
        Err(e) => Err(ApiErrorStatus::UnknownFailure(e.to_string())),
    }
}

/// Resolves the channel with the given counterparty in the specified direction.
fn resolve_channel<H: HasChainApi<ChainError = hopr_lib::errors::HoprLibError> + Send + Sync>(
    hopr: &H,
    address: &Address,
    direction: ChannelDirection,
) -> Result<ChannelEntry, ApiErrorStatus> {
    let me = hopr.identity().node_address;
    let lookup = match direction {
        ChannelDirection::Outgoing => hopr.channel(me, *address),
        ChannelDirection::Incoming => hopr.channel(*address, me),
    };
    filter_open_channel(lookup)
}

/// Returns information about the channel with the given counterparty address in the given direction.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/channels/{{address}}"),
        description = "Returns information about the channel with the given counterparty address. Use the `direction` query parameter to choose between the outgoing (this node → counterparty, default) and incoming (counterparty → this node) channel.",
        params(
            ("address" = String, Path, description = "On-chain address of the counterparty.", example = "0x188c4462b75e46f0c7262d7f48d182447b93a93c"),
            ChannelDirectionQuery
        ),
        responses(
            (status = 200, description = "Channel fetched successfully", body = ChannelInfoResponse),
            (status = 400, description = "Invalid counterparty address or direction.", body = ApiError),
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
pub(super) async fn show_channel<
    H: HasChainApi<ChainError = hopr_lib::errors::HoprLibError> + Send + Sync + 'static,
>(
    Path(AddressParams { address }): Path<AddressParams>,
    Query(ChannelDirectionQuery { direction }): Query<ChannelDirectionQuery>,
    State(state): State<Arc<InternalState<H>>>,
) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    match hopr_async_runtime::prelude::spawn_blocking(move || resolve_channel(&*hopr, &address, direction)).await {
        Ok(Ok(channel)) => (StatusCode::OK, Json(channel_to_topology_info(&channel))).into_response(),
        Ok(Err(status)) => match status {
            ApiErrorStatus::ChannelNotFound => (StatusCode::NOT_FOUND, status).into_response(),
            _ => (StatusCode::UNPROCESSABLE_ENTITY, status).into_response(),
        },
        Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
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
    // /// New status of the channel. Will be one of `Closed` or `PendingToClose`.
    // #[serde_as(as = "DisplayFromStr")]
    // #[schema(value_type = String, example = "PendingToClose")]
    // channel_status: ChannelStatus,
}

/// Closes the channel with the given counterparty in the given direction.
///
/// If the channel is currently `Open`, it will transition it to `PendingToClose`.
/// If the channel is in `PendingToClose` and the channel closure period has elapsed,
/// it will transition it to `Closed`.
#[utoipa::path(
        delete,
        path = const_format::formatcp!("{BASE_PATH}/channels/{{address}}"),
        description = "Closes the channel with the given counterparty. Use the `direction` query parameter to choose between the outgoing (this node → counterparty, default) and incoming (counterparty → this node) channel.",
        params(
            ("address" = String, Path, description = "On-chain address of the counterparty.", example = "0x188c4462b75e46f0c7262d7f48d182447b93a93c"),
            ChannelDirectionQuery
        ),
        responses(
            (status = 200, description = "Channel closed successfully", body = CloseChannelResponse),
            (status = 400, description = "Invalid counterparty address or direction.", body = ApiError),
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
pub(super) async fn close_channel<
    H: HasChainApi<ChainError = hopr_lib::errors::HoprLibError> + Send + Sync + 'static,
>(
    Path(AddressParams { address }): Path<AddressParams>,
    Query(ChannelDirectionQuery { direction }): Query<ChannelDirectionQuery>,
    State(state): State<Arc<InternalState<H>>>,
) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    let channel_id = match resolve_channel(&*hopr, &address, direction) {
        Ok(ch) => *ch.get_id(),
        Err(status) => {
            let code = match status {
                ApiErrorStatus::ChannelNotFound => StatusCode::NOT_FOUND,
                _ => StatusCode::UNPROCESSABLE_ENTITY,
            };
            return (code, status).into_response();
        }
    };

    match hopr.close_channel_by_id(&channel_id).await {
        Ok(output) => (
            StatusCode::OK,
            Json(CloseChannelResponse {
                receipt: *output.tx_hash(),
            }),
        )
            .into_response(),
        Err(hopr_lib::api::node::EitherErr::Right(HoprLibError::NotReady(..))) => {
            (StatusCode::PRECONDITION_FAILED, ApiErrorStatus::NotReady).into_response()
        }
        Err(e) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiErrorStatus::UnknownFailure(e.to_string()),
        )
            .into_response(),
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

/// Funds the outgoing channel to the given counterparty with the given amount of HOPR tokens.
///
/// Funding applies only to the outgoing channel (this node → counterparty), since only
/// the channel source can add stake.
#[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/channels/{{address}}/fund"),
        description = "Funds the outgoing channel to the given counterparty with the given amount of HOPR tokens.",
        params(
            ("address" = String, Path, description = "On-chain address of the counterparty.", example = "0x188c4462b75e46f0c7262d7f48d182447b93a93c")
        ),
        request_body(
            content = FundBodyRequest,
            description = "Specifies the amount of HOPR tokens to fund a channel with.",
            content_type = "application/json",
        ),
        responses(
            (status = 200, description = "Channel funded successfully", body = FundChannelResponse),
            (status = 400, description = "Invalid counterparty address or request body.", body = ApiError),
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
pub(super) async fn fund_channel<
    H: HasChainApi<ChainError = hopr_lib::errors::HoprLibError> + Send + Sync + 'static,
>(
    Path(AddressParams { address }): Path<AddressParams>,
    State(state): State<Arc<InternalState<H>>>,
    Json(fund_req): Json<FundBodyRequest>,
) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    let channel_id = match resolve_channel(&*hopr, &address, ChannelDirection::Outgoing) {
        Ok(ch) => *ch.get_id(),
        Err(status) => {
            let code = match status {
                ApiErrorStatus::ChannelNotFound => StatusCode::NOT_FOUND,
                _ => StatusCode::UNPROCESSABLE_ENTITY,
            };
            return (code, status).into_response();
        }
    };

    match hopr.fund_channel(&channel_id, fund_req.amount).await {
        Ok(output) => (
            StatusCode::OK,
            Json(FundChannelResponse {
                hash: *output.tx_hash(),
            }),
        )
            .into_response(),
        Err(hopr_lib::api::node::EitherErr::Right(HoprLibError::NotReady(..))) => {
            (StatusCode::PRECONDITION_FAILED, ApiErrorStatus::NotReady).into_response()
        }
        Err(e) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiErrorStatus::UnknownFailure(e.to_string()),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use hopr_lib::errors::HoprLibError;

    use super::*;

    fn test_channel(status: ChannelStatus) -> ChannelEntry {
        let src: Address = "0x07eaf07d6624f741e04f4092a755a9027aaab7f6".parse().unwrap();
        let dst: Address = "0x188c4462b75e46f0c7262d7f48d182447b93a93c".parse().unwrap();
        ChannelEntry::builder()
            .between(src, dst)
            .balance("10 wxHOPR".parse().unwrap())
            .status(status)
            .build()
            .unwrap()
    }

    #[test]
    fn channel_to_topology_info_should_convert_open_channel() {
        let ch = test_channel(ChannelStatus::Open);
        let info = channel_to_topology_info(&ch);

        assert_eq!(info.channel_id, *ch.get_id());
        assert_eq!(info.source, ch.source);
        assert_eq!(info.destination, ch.destination);
        assert_eq!(info.balance, ch.balance);
        assert_eq!(info.status, ch.status);
        assert_eq!(info.channel_epoch, 1);
        assert_eq!(info.ticket_index, 0);
        assert_eq!(info.closure_time, 0);
    }

    #[test]
    fn channel_to_topology_info_should_serialize_correctly() {
        let ch = test_channel(ChannelStatus::Open);
        let info = channel_to_topology_info(&ch);
        let json = serde_json::to_value(&info).unwrap();

        assert_eq!(json["status"], "Open");
        assert_eq!(json["balance"], "10 wxHOPR");
        assert!(json.get("channelId").is_some());
        assert!(json.get("source").is_some());
        assert!(json.get("destination").is_some());
    }

    #[test]
    fn filter_open_channel_should_return_open_channel() {
        let ch = test_channel(ChannelStatus::Open);
        let result = filter_open_channel::<HoprLibError>(Ok(Some(ch)));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().get_id(), ch.get_id());
    }

    #[test]
    fn filter_open_channel_should_return_pending_to_close() {
        let ch = test_channel(ChannelStatus::PendingToClose(std::time::SystemTime::now()));
        assert!(filter_open_channel::<HoprLibError>(Ok(Some(ch))).is_ok());
    }

    #[test]
    fn filter_open_channel_should_reject_closed_channel() {
        let ch = test_channel(ChannelStatus::Closed);
        let result = filter_open_channel::<HoprLibError>(Ok(Some(ch)));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ApiErrorStatus::ChannelNotFound);
    }

    #[test]
    fn filter_open_channel_should_reject_none() {
        let result = filter_open_channel::<HoprLibError>(Ok(None));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ApiErrorStatus::ChannelNotFound);
    }

    #[test]
    fn filter_open_channel_should_map_error_to_unknown_failure() {
        let err = HoprLibError::GeneralError("test".into());
        let result = filter_open_channel(Err(err));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiErrorStatus::UnknownFailure(_)));
    }

    #[test]
    fn node_channel_should_serialize_correctly() {
        let ch = test_channel(ChannelStatus::Open);
        let node_ch = NodeChannel {
            id: *ch.get_id(),
            peer_address: ch.destination,
            status: ch.status,
            balance: ch.balance,
        };
        let json = serde_json::to_value(&node_ch).unwrap();
        assert_eq!(json["status"], "Open");
        assert_eq!(json["balance"], "10 wxHOPR");
    }

    #[test]
    fn channels_query_request_should_default_to_false() {
        let req: ChannelsQueryRequest = serde_json::from_str("{}").unwrap();
        assert!(!req.including_closed);
        assert!(!req.full_topology);
    }

    #[test]
    fn channels_query_request_should_deserialize_flags() {
        let req: ChannelsQueryRequest =
            serde_json::from_str(r#"{"includingClosed": true, "fullTopology": true}"#).unwrap();
        assert!(req.including_closed);
        assert!(req.full_topology);
    }

    #[test]
    fn open_channel_body_request_should_deserialize() {
        let json = serde_json::json!({
            "amount": "10 wxHOPR",
            "destination": "0xa8194d36e322592d4c707b70dbe96121f5c74c64"
        });
        let req: OpenChannelBodyRequest = serde_json::from_value(json).unwrap();
        assert_eq!(req.amount, "10 wxHOPR".parse().unwrap());
    }

    #[test]
    fn fund_body_request_should_deserialize() {
        let json = serde_json::json!({ "amount": "5 wxHOPR" });
        let req: FundBodyRequest = serde_json::from_value(json).unwrap();
        assert_eq!(req.amount, "5 wxHOPR".parse().unwrap());
    }

    #[test]
    fn channel_direction_should_default_to_outgoing() {
        assert_eq!(ChannelDirection::default(), ChannelDirection::Outgoing);
    }

    #[test]
    fn channel_direction_query_should_default_to_outgoing_when_missing() {
        let q: ChannelDirectionQuery = serde_json::from_str("{}").unwrap();
        assert_eq!(q.direction, ChannelDirection::Outgoing);
    }

    #[test]
    fn channel_direction_should_parse_outgoing() {
        let d: ChannelDirection = serde_json::from_str(r#""outgoing""#).unwrap();
        assert_eq!(d, ChannelDirection::Outgoing);
    }

    #[test]
    fn channel_direction_should_parse_incoming() {
        let d: ChannelDirection = serde_json::from_str(r#""incoming""#).unwrap();
        assert_eq!(d, ChannelDirection::Incoming);
    }

    #[test]
    fn channel_direction_should_reject_unknown_value() {
        let res: Result<ChannelDirection, _> = serde_json::from_str(r#""both""#);
        assert!(res.is_err());
    }

    #[test]
    fn address_params_should_deserialize() {
        let json = serde_json::json!({ "address": "0x188c4462b75e46f0c7262d7f48d182447b93a93c" });
        let params: AddressParams = serde_json::from_value(json).unwrap();
        assert_eq!(
            params.address,
            "0x188c4462b75e46f0c7262d7f48d182447b93a93c".parse().unwrap()
        );
    }

    #[test]
    fn channel_info_response_should_serialize_all_fields() {
        let ch = test_channel(ChannelStatus::Open);
        let info = channel_to_topology_info(&ch);
        let json = serde_json::to_value(&info).unwrap();

        assert_eq!(json["channelId"], ch.get_id().to_string());
        assert_eq!(
            json["source"].as_str().unwrap().to_lowercase(),
            ch.source.to_string().to_lowercase()
        );
        assert_eq!(
            json["destination"].as_str().unwrap().to_lowercase(),
            ch.destination.to_string().to_lowercase()
        );
        assert_eq!(json["balance"], "10 wxHOPR");
        assert_eq!(json["status"], "Open");
        assert_eq!(json["ticketIndex"], 0);
        assert_eq!(json["channelEpoch"], 1);
        assert_eq!(json["closureTime"], 0);
    }

    #[test]
    fn channel_to_topology_info_should_compute_closure_time_for_pending_to_close() {
        let ch = test_channel(ChannelStatus::PendingToClose(
            std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000),
        ));
        let info = channel_to_topology_info(&ch);
        assert_eq!(info.closure_time, 1_700_000_000);
    }

    #[test]
    fn node_channels_response_should_serialize_all_three_groups() {
        let ch = test_channel(ChannelStatus::Open);
        let node_ch = NodeChannel {
            id: *ch.get_id(),
            peer_address: ch.destination,
            status: ch.status,
            balance: ch.balance,
        };
        let resp = NodeChannelsResponse {
            incoming: vec![node_ch.clone()],
            outgoing: vec![node_ch.clone()],
            all: vec![channel_to_topology_info(&ch)],
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["incoming"].as_array().unwrap().len(), 1);
        assert_eq!(json["outgoing"].as_array().unwrap().len(), 1);
        assert_eq!(json["all"].as_array().unwrap().len(), 1);
        assert_eq!(json["incoming"][0]["status"], "Open");
        assert_eq!(json["all"][0]["channelId"], ch.get_id().to_string());
    }

    #[test]
    fn open_channel_body_request_should_include_destination() {
        let json = serde_json::json!({
            "amount": "10 wxHOPR",
            "destination": "0xa8194d36e322592d4c707b70dbe96121f5c74c64"
        });
        let req: OpenChannelBodyRequest = serde_json::from_value(json).unwrap();
        assert_eq!(
            req.destination,
            "0xa8194d36e322592d4c707b70dbe96121f5c74c64".parse().unwrap()
        );
        assert_eq!(req.amount, "10 wxHOPR".parse().unwrap());
    }

    #[test]
    fn open_channel_response_should_serialize_correctly() {
        let resp = OpenChannelResponse {
            channel_id: Hash::default(),
            transaction_receipt: Hash::default(),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert!(json.get("channelId").is_some());
        assert!(json.get("transactionReceipt").is_some());
    }

    #[test]
    fn close_channel_response_should_serialize_correctly() {
        let resp = CloseChannelResponse {
            receipt: Hash::default(),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert!(json.get("receipt").is_some());
    }

    #[test]
    fn fund_channel_response_should_serialize_correctly() {
        let resp = FundChannelResponse { hash: Hash::default() };
        let json = serde_json::to_value(&resp).unwrap();
        assert!(json.get("hash").is_some());
    }

    #[test]
    fn address_params_should_reject_invalid_address() {
        let json = serde_json::json!({ "address": "not-an-address" });
        assert!(serde_json::from_value::<AddressParams>(json).is_err());
    }

    // ── Endpoint-level tests ───────────────────────────────────────────────

    use std::sync::Arc;

    use anyhow::Context;
    use axum::{Router, body::Body, http::Request, routing::get};
    use tower::ServiceExt;

    use crate::testing::MockChainNode;

    fn channels_router(node: MockChainNode) -> Router {
        let state = Arc::new(crate::InternalState {
            hoprd_cfg: serde_json::Value::Null,
            auth: Arc::new(crate::config::Auth::Token("test".into())),
            hopr: Arc::new(node),
            open_listeners: Arc::new(hopr_utils_session::ListenerJoinHandles::default()),
            default_listen_host: "127.0.0.1:0".parse().unwrap(),
        });

        Router::new()
            .route("/channels", get(list_channels::<MockChainNode>))
            .with_state(state)
    }

    #[tokio::test]
    async fn list_channels_should_return_empty_when_no_channels() -> anyhow::Result<()> {
        let node = MockChainNode::random();

        let resp = channels_router(node)
            .oneshot(Request::get("/channels").body(Body::empty())?)
            .await?;

        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await?;
        let json: serde_json::Value = serde_json::from_slice(&body)?;

        // StubChain::stream_channels returns empty stream, so channels_to/channels_from
        // both return empty Vecs
        assert_eq!(
            json["incoming"]
                .as_array()
                .context("incoming should be an array")?
                .len(),
            0
        );
        assert_eq!(
            json["outgoing"]
                .as_array()
                .context("outgoing should be an array")?
                .len(),
            0
        );

        Ok(())
    }
}
