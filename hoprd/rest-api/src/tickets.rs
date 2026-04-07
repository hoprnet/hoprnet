use std::sync::Arc;

use axum::{
    extract::{Json, Path, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use hopr_lib::{
    HoprBalance, ToHex,
    api::tickets::{ChannelStats, TicketManagement},
    prelude::Hash,
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
    #[schema(example = "1")]
    win_prob: String,
    #[schema(example = 1)]
    channel_epoch: u32,
    #[schema(
        example = "0xe445fcf4e90d25fe3c9199ccfaff85e23ecce8773304d85e7120f1f38787f2329822470487a37f1b5408c8c0b73e874ee9f7594a632713b6096e616857999891"
    )]
    signature: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChannelIdParams {
    channel_id: String,
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

impl From<ChannelStats> for NodeTicketStatisticsResponse {
    fn from(value: ChannelStats) -> Self {
        Self {
            winning_count: value.winning_tickets as u64,
            unredeemed_value: value.unredeemed_value,
            #[allow(deprecated)] // TODO: remove once blokli#237 is merged
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
    if let Ok(ticket_mgt) = hopr.ticket_management() {
        match ticket_mgt.ticket_stats(None).map(NodeTicketStatisticsResponse::from) {
            Ok(stats) => (StatusCode::OK, Json(stats)).into_response(),
            Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
        }
    } else {
        (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::NotReady).into_response()
    }
}

/// Starts redeeming of all tickets in all channels.
///
/// **WARNING:** If there are many unredeemed tickets in all channels, this operation
/// can incur significant transaction costs.
#[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/tickets/redeem"),
        description = "Starts redeeming of all tickets in all channels.",
        responses(
            (status = 202, description = "Ticket redemption started successfully."),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Tickets"
    )]
pub(super) async fn redeem_all_tickets(State(state): State<Arc<InternalState>>) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    hopr_async_runtime::prelude::spawn(async move {
        match hopr.redeem_all_tickets(0).await {
            Ok(_) => {
                tracing::info!("all tickets redeemed on API request");
            }
            Err(error) => {
                tracing::error!(%error, "failed to redeem all tickets on API request");
            }
        }
    });

    (StatusCode::ACCEPTED, "").into_response()
}

/// Starts redeeming all tickets in the given channel.
///
/// **WARNING:** If there are many unredeemed tickets in the given channel, this operation
/// can incur significant transaction costs.
#[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/channels/{{channelId}}/tickets/redeem"),
        description = "Starts redeeming all tickets in the given channel.",
        params(
            ("channelId" = String, Path, description = "ID of the channel.", example = "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f")
        ),
        responses(
            (status = 202, description = "Ticket redemption started successfully."),
            (status = 400, description = "Invalid channel id.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError)
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
        Ok(channel_id) => {
            hopr_async_runtime::prelude::spawn(async move {
                match hopr.redeem_tickets_in_channel(&channel_id, 0).await {
                    Ok(_) => {
                        tracing::info!(%channel_id, "tickets in channel redeemed on API request");
                    }
                    Err(error) => {
                        tracing::error!(%error, %channel_id, "failed to redeem tickets in channel on API request");
                    }
                }
            });

            (StatusCode::ACCEPTED, "").into_response()
        }
        Err(_) => (StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidChannelId).into_response(),
    }
}
