use std::sync::Arc;

use axum::{
    extract::{Json, Path, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use hopr_crypto_types::types::Hash;
use hopr_lib::{
    HoprBalance, HoprTransportError, ProtocolError, Ticket, TicketStatistics, ToHex,
    errors::{HoprLibError, HoprStatusError},
};
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};

use crate::{ApiError, ApiErrorStatus, BASE_PATH, InternalState};

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
/// Represents a ticket in a channel.
pub(crate) struct ChannelTicket {
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f")]
    channel_id: Hash,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "1.0 wxHOPR")]
    amount: HoprBalance,
    #[schema(example = 0)]
    index: u64,
    #[schema(example = 1)]
    index_offset: u32,
    #[schema(example = "1")]
    win_prob: String,
    #[schema(example = 1)]
    channel_epoch: u32,
    #[schema(
        example = "0xe445fcf4e90d25fe3c9199ccfaff85e23ecce8773304d85e7120f1f38787f2329822470487a37f1b5408c8c0b73e874ee9f7594a632713b6096e616857999891"
    )]
    signature: String,
}

impl From<Ticket> for ChannelTicket {
    fn from(value: Ticket) -> Self {
        Self {
            channel_id: value.channel_id,
            amount: value.amount,
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
        description = "Lists all tickets for the given channel ID.",
        params(
            ("channelId" = String, Path, description = "ID of the channel.", example = "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f")
        ),
        responses(
            (status = 200, description = "Fetched all tickets for the given channel ID", body = [ChannelTicket], example = json!([
                {
                    "amount": "10 wxHOPR",
                    "channelEpoch": 1,
                    "channelId": "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f",
                    "index": 0,
                    "indexOffset": 1,
                    "signature": "0xe445fcf4e90d25fe3c9199ccfaff85e23ecce8773304d85e7120f1f38787f2329822470487a37f1b5408c8c0b73e874ee9f7594a632713b6096e616857999891",
                    "winProb": "1"
                }
            ])),
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
    description = "(deprecated) Returns an empty array.",
    responses(
        (status = 200, description = "Fetched all tickets in all the channels", body = [ChannelTicket], example = json!([
        {
            "amount": "10 wxHOPR",
            "channelEpoch": 1,
            "channelId": "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f",
            "index": 0,
            "indexOffset": 1,
            "signature": "0xe445fcf4e90d25fe3c9199ccfaff85e23ecce8773304d85e7120f1f38787f2329822470487a37f1b5408c8c0b73e874ee9f7594a632713b6096e616857999891",
            "winProb": "1"
        }
        ])),
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

#[serde_as]
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "winningCount": 0,
        "neglectedValue": "0 wxHOPR",
        "redeemedValue": "1000 wxHOPR",
        "rejectedValue": "0 wxHOPR",
        "unredeemedValue": "2000 wxHOPR",
    }))]
#[serde(rename_all = "camelCase")]
/// Received tickets statistics.
pub(crate) struct NodeTicketStatisticsResponse {
    #[schema(example = 0)]
    winning_count: u64,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "20 wxHOPR")]
    unredeemed_value: HoprBalance,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String,example = "100 wxHOPR")]
    redeemed_value: HoprBalance,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String,example = "0 wxHOPR")]
    neglected_value: HoprBalance,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "0 wHOPR")]
    rejected_value: HoprBalance,
}

impl From<TicketStatistics> for NodeTicketStatisticsResponse {
    fn from(value: TicketStatistics) -> Self {
        Self {
            winning_count: value.winning_count as u64,
            unredeemed_value: value.unredeemed_value,
            redeemed_value: value.redeemed_value,
            neglected_value: value.neglected_value,
            rejected_value: value.rejected_value,
        }
    }
}

/// Returns current complete statistics on tickets.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/tickets/statistics"),
        description = "Returns current complete statistics on tickets.",
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

/// Resets the ticket metrics.
#[utoipa::path(
        delete,
        path = const_format::formatcp!("{BASE_PATH}/tickets/statistics"),
        description = "Resets the ticket metrics.",
        responses(
            (status = 204, description = "Ticket statistics reset successfully."),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Tickets"
    )]
pub(super) async fn reset_ticket_statistics(State(state): State<Arc<InternalState>>) -> impl IntoResponse {
    let hopr = state.hopr.clone();
    match hopr.reset_ticket_statistics().await {
        Ok(()) => (StatusCode::NO_CONTENT, "").into_response(),
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
        description = "Starts redeeming of all tickets in all channels.",
        responses(
            (status = 204, description = "Tickets redeemed successfully."),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 412, description = "The node is not ready."),
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
    match hopr.redeem_all_tickets(0, false).await {
        Ok(()) => (StatusCode::NO_CONTENT, "").into_response(),
        Err(HoprLibError::StatusError(HoprStatusError::NotThereYet(..))) => {
            (StatusCode::PRECONDITION_FAILED, ApiErrorStatus::NotReady).into_response()
        }
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
        description = "Starts redeeming all tickets in the given channel.",
        params(
            ("channelId" = String, Path, description = "ID of the channel.", example = "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f")
        ),
        responses(
            (status = 204, description = "Tickets redeemed successfully."),
            (status = 400, description = "Invalid channel id.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "Tickets were not found for that channel. That means that no messages were sent inside this channel yet.", body = ApiError),
            (status = 412, description = "The node is not ready."),
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
        Ok(channel_id) => match hopr.redeem_tickets_in_channel(&channel_id, 0, false).await {
            Ok(count) if count > 0 => (StatusCode::NO_CONTENT, "").into_response(),
            Ok(_) => (StatusCode::NOT_FOUND, ApiErrorStatus::ChannelNotFound).into_response(),
            Err(HoprLibError::StatusError(HoprStatusError::NotThereYet(..))) => {
                (StatusCode::PRECONDITION_FAILED, ApiErrorStatus::NotReady).into_response()
            }
            Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
        },
        Err(_) => (StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidChannelId).into_response(),
    }
}
