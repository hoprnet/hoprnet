use std::sync::Arc;

use axum::{
    extract::{Json, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use hopr_lib::{
    Address, ChannelStatus, HoprBalance, IncentiveChannelOperations, IncentiveRedeemOperations,
    api::{node::HasChainApi, tickets::ChannelStats},
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
pub(super) async fn show_ticket_statistics<H: crate::HoprNode>(State(state): State<Arc<InternalState<H>>>) -> impl IntoResponse {
    let hopr = state.hopr.clone();
    match hopr.ticket_statistics() {
        Ok(stats) => (StatusCode::OK, Json(NodeTicketStatisticsResponse::from(stats))).into_response(),
        Err(e) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiErrorStatus::UnknownFailure(e.to_string()),
        )
            .into_response(),
    }
}

#[serde_as]
#[derive(Debug, Default, Clone, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "address": "0x188c4462b75e46f0c7262d7f48d182447b93a93c"
}))]
#[serde(rename_all = "camelCase")]
/// Request body for ticket redemption with optional fields.
pub(crate) struct RedeemTicketsRequest {
    /// On-chain address of the counterparty whose incoming channel tickets to redeem.
    /// If omitted, tickets in all channels are redeemed.
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[schema(value_type = Option<String>, example = "0x188c4462b75e46f0c7262d7f48d182447b93a93c")]
    address: Option<Address>,
}

/// Starts redeeming tickets.
///
/// When an `address` is specified, only tickets in the incoming channel from that
/// counterparty are redeemed. When omitted, tickets in all channels are redeemed.
///
/// **WARNING:** Redeeming many tickets can incur significant transaction costs.
#[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/tickets/redeem"),
        description = "Starts redeeming tickets. When a counterparty address is specified, only tickets from that counterparty are redeemed.",
        request_body(
            content = RedeemTicketsRequest,
            description = "Optional counterparty address to scope ticket redemption.",
            content_type = "application/json",
        ),
        responses(
            (status = 202, description = "Ticket redemption started successfully."),
            (status = 400, description = "Invalid request body or malformed JSON.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "Channel with counterparty not found.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError),
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Tickets"
    )]
pub(super) async fn redeem_tickets<H: crate::HoprNode>(
    State(state): State<Arc<InternalState<H>>>,
    Json(req): Json<RedeemTicketsRequest>,
) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    match req.address {
        Some(address) => {
            // Resolve the incoming channel from the counterparty (counterparty → me).
            let me = hopr.identity().node_address;
            let channel_id = match hopr.channel(address, me) {
                Ok(Some(ch)) if ch.status != ChannelStatus::Closed => *ch.get_id(),
                Ok(_) => return (StatusCode::NOT_FOUND, ApiErrorStatus::ChannelNotFound).into_response(),
                Err(e) => {
                    return (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        ApiErrorStatus::UnknownFailure(e.to_string()),
                    )
                        .into_response();
                }
            };

            hopr_async_runtime::prelude::spawn(async move {
                match hopr.redeem_tickets_with_counterparty(address, 0).await {
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
        None => {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticket_statistics_response_should_serialize_correctly() {
        let stats = NodeTicketStatisticsResponse {
            winning_count: 5,
            unredeemed_value: "20 wxHOPR".parse().unwrap(),
            neglected_value: "0 wxHOPR".parse().unwrap(),
            rejected_value: "0 wxHOPR".parse().unwrap(),
        };

        let json = serde_json::to_value(&stats).unwrap();
        assert_eq!(json["winningCount"], 5);
        assert_eq!(json["unredeemedValue"], "20 wxHOPR");
        assert_eq!(json["neglectedValue"], "0 wxHOPR");
        assert_eq!(json["rejectedValue"], "0 wxHOPR");
    }

    #[test]
    fn redeem_tickets_request_should_deserialize_with_address() {
        let json = serde_json::json!({
            "address": "0x188c4462b75e46f0c7262d7f48d182447b93a93c"
        });

        let req: RedeemTicketsRequest = serde_json::from_value(json).unwrap();
        assert!(req.address.is_some());
    }

    #[test]
    fn redeem_tickets_request_should_deserialize_without_address() {
        let json = serde_json::json!({});
        let req: RedeemTicketsRequest = serde_json::from_value(json).unwrap();
        assert!(req.address.is_none());
    }

    #[test]
    fn channel_stats_should_convert_to_response() {
        let stats = ChannelStats {
            winning_tickets: 10,
            unredeemed_value: "100 wxHOPR".parse().unwrap(),
            neglected_value: "5 wxHOPR".parse().unwrap(),
            rejected_value: "1 wxHOPR".parse().unwrap(),
        };

        let response = NodeTicketStatisticsResponse::from(stats);
        assert_eq!(response.winning_count, 10);
        assert_eq!(response.unredeemed_value, "100 wxHOPR".parse().unwrap());
        assert_eq!(response.neglected_value, "5 wxHOPR".parse().unwrap());
        assert_eq!(response.rejected_value, "1 wxHOPR".parse().unwrap());
    }

    #[test]
    fn redeem_tickets_request_default_should_have_no_address() {
        let req = RedeemTicketsRequest::default();
        assert!(req.address.is_none());
    }

    #[test]
    fn redeem_tickets_request_should_reject_invalid_address() {
        let json = serde_json::json!({ "address": "not-an-address" });
        assert!(serde_json::from_value::<RedeemTicketsRequest>(json).is_err());
    }

    #[test]
    fn channel_ticket_should_serialize_correctly() {
        let ticket = ChannelTicket {
            channel_id: Hash::default(),
            amount: "1.0 wxHOPR".parse().unwrap(),
            index: 7,
            win_prob: "1".to_string(),
            channel_epoch: 2,
            signature: "0xdeadbeef".to_string(),
        };
        let json = serde_json::to_value(&ticket).unwrap();
        assert_eq!(json["amount"], "1 wxHOPR");
        assert_eq!(json["index"], 7);
        assert_eq!(json["winProb"], "1");
        assert_eq!(json["channelEpoch"], 2);
        assert_eq!(json["signature"], "0xdeadbeef");
        assert!(json.get("channelId").is_some());
    }
}
