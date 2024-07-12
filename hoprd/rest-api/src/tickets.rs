use axum::{
    extract::{Json, Path, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use hopr_crypto_types::types::Hash;
use hopr_lib::{errors::HoprLibError, HoprTransportError, ProtocolError, Ticket, TicketStatistics, ToHex};
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use std::sync::Arc;

use crate::{ApiErrorStatus, InternalState, BASE_PATH};

#[serde_as]
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "amount": "100",
        "channelEpoch": 1,
        "channelId": "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f",
        "index": 0,
        "indexOffset": 1,
        "signature": "0xe445fcf4e90d25fe3c9199ccfaff85e23ecce8773304d85e7120f1f38787f2329822470487a37f1b5408c8c0b73e874ee9f7594a632713b6096e616857999891",
        "winProb": "1"
    }))]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChannelTicket {
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    channel_id: Hash,
    amount: String,
    index: u64,
    index_offset: u32,
    win_prob: String,
    channel_epoch: u32,
    signature: String,
}

impl From<Ticket> for ChannelTicket {
    fn from(value: Ticket) -> Self {
        Self {
            channel_id: value.channel_id,
            amount: value.amount.amount().to_string(),
            index: value.index,
            index_offset: value.index_offset,
            win_prob: value.win_prob().to_string(),
            channel_epoch: value.channel_epoch,
            signature: value.signature.expect("impossible to have an unsigned ticket").to_hex(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChannelIdParams {
    channel_id: String,
}

/// Lists all tickets for the given channel  ID.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/channels/{{channelId}}/tickets"),
        params(
            ("channelId" = String, Path, description = "ID of the channel.")
        ),
        responses(
            (status = 200, description = "Fetched all tickets for the given channel ID", body = [ChannelTicket]),
            (status = 400, description = "Invalid channel id.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "Channel not found.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Channels"
    )]
pub(super) async fn show_channel_tickets(
    Path(ChannelIdParams { channel_id }): Path<ChannelIdParams>,
    State(state): State<Arc<InternalState>>,
) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    match Hash::from_hex(channel_id.as_str()) {
        Ok(channel_id) => match hopr.tickets_in_channel(&channel_id).await {
            Ok(Some(_tickets)) => {
                let tickets: Vec<ChannelTicket> = vec![];
                (StatusCode::OK, Json(tickets)).into_response()
            }
            Ok(None) => (StatusCode::NOT_FOUND, ApiErrorStatus::TicketsNotFound).into_response(),
            Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
        },
        Err(_) => (StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidChannelId).into_response(),
    }
}

/// Endpoint is deprecated and will be removed in the future. Returns an empty array.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/tickets"),
        responses(
            (status = 200, description = "Fetched all tickets in all the channels", body = [ChannelTicket]),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Tickets"
    )]
pub(super) async fn show_all_tickets() -> impl IntoResponse {
    let tickets: Vec<ChannelTicket> = vec![];
    (StatusCode::OK, Json(tickets)).into_response()
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "winning_count": 0,
        "neglectedValue": "0",
        "redeemedValue": "100",
        "rejectedValue": "0",
        "unredeemedValue": "200",
    }))]
#[serde(rename_all = "camelCase")]
pub(crate) struct NodeTicketStatisticsResponse {
    winning_count: u64,
    unredeemed_value: String,
    redeemed_value: String,
    neglected_value: String,
    rejected_value: String,
}

impl From<TicketStatistics> for NodeTicketStatisticsResponse {
    fn from(value: TicketStatistics) -> Self {
        Self {
            winning_count: value.winning_count as u64,
            unredeemed_value: value.unredeemed_value.amount().to_string(),
            redeemed_value: value.redeemed_value.amount().to_string(),
            neglected_value: value.neglected_value.amount().to_string(),
            rejected_value: value.rejected_value.amount().to_string(),
        }
    }
}

/// Returns current complete statistics on tickets.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/tickets/statistics"),
        responses(
            (status = 200, description = "Tickets statistics fetched successfully. Check schema for description of every field in the statistics.", body = NodeTicketStatisticsResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Tickets"
    )]
pub(super) async fn show_ticket_statistics(State(state): State<Arc<InternalState>>) -> impl IntoResponse {
    let hopr = state.hopr.clone();
    match hopr.ticket_statistics().await.map(NodeTicketStatisticsResponse::from) {
        Ok(stats) => (StatusCode::OK, Json(stats)).into_response(),
        Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }
}

/// Starts redeeming of all tickets in all channels.
///
/// **WARNING:** this should almost **never** be used as it can issue a large
/// number of on-chain transactions. The tickets should almost always be aggregated first.
#[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/tickets/redeem"),
        responses(
            (status = 204, description = "Tickets redeemed successfully."),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Tickets"
    )]
pub(super) async fn redeem_all_tickets(State(state): State<Arc<InternalState>>) -> impl IntoResponse {
    let hopr = state.hopr.clone();
    match hopr.redeem_all_tickets(false).await {
        Ok(()) => (StatusCode::NO_CONTENT, "").into_response(),
        Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }
}

/// Starts redeeming all tickets in the given channel.
///
/// **WARNING:** this should almost **never** be used as it can issue a large
/// number of on-chain transactions. The tickets should almost always be aggregated first.
#[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/channels/{{channelId}}/tickets/redeem"),
        params(
            ("channelId" = String, Path, description = "ID of the channel.")
        ),
        responses(
            (status = 204, description = "Tickets redeemed successfully."),
            (status = 400, description = "Invalid channel id.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "Tickets were not found for that channel. That means that no messages were sent inside this channel yet.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Channels"
    )]
pub(super) async fn redeem_tickets_in_channel(
    Path(ChannelIdParams { channel_id }): Path<ChannelIdParams>,
    State(state): State<Arc<InternalState>>,
) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    match Hash::from_hex(channel_id.as_str()) {
        Ok(channel_id) => match hopr.redeem_tickets_in_channel(&channel_id, false).await {
            Ok(count) if count > 0 => (StatusCode::NO_CONTENT, "").into_response(),
            Ok(_) => (StatusCode::NOT_FOUND, ApiErrorStatus::ChannelNotFound).into_response(),
            Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
        },
        Err(_) => (StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidChannelId).into_response(),
    }
}

/// Starts aggregation of tickets in the given channel.
#[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/channels/{{channelId}}/tickets/aggregate"),
        params(
            ("channelId" = String, Path, description = "ID of the channel.")
        ),
        responses(
            (status = 204, description = "Tickets successfully aggregated"),
            (status = 400, description = "Invalid channel id.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "Tickets were not found for that channel. That means that no messages were sent inside this channel yet.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Channels"
    )]
pub(super) async fn aggregate_tickets_in_channel(
    Path(ChannelIdParams { channel_id }): Path<ChannelIdParams>,
    State(state): State<Arc<InternalState>>,
) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    match Hash::from_hex(channel_id.as_str()) {
        Ok(channel_id) => match hopr.aggregate_tickets(&channel_id).await {
            Ok(_) => (StatusCode::NO_CONTENT, "").into_response(),
            Err(HoprLibError::TransportError(HoprTransportError::Protocol(ProtocolError::ChannelNotFound))) => {
                (StatusCode::NOT_FOUND, ApiErrorStatus::ChannelNotFound).into_response()
            }
            Err(HoprLibError::TransportError(HoprTransportError::Protocol(ProtocolError::ChannelClosed))) => {
                (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::ChannelNotOpen).into_response()
            }
            Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
        },
        Err(_) => (StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidChannelId).into_response(),
    }
}
