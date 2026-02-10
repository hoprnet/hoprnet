use std::sync::Arc;

use axum::{
    extract::{Json, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use hopr_lib::HoprBalance;
use serde_with::{DisplayFromStr, serde_as};

use crate::{ApiError, ApiErrorStatus, BASE_PATH, InternalState};

#[serde_as]
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "price": "0.03 wxHOPR"
}))]
#[serde(rename_all = "camelCase")]
/// Contains the ticket price in HOPR tokens.
pub(crate) struct TicketPriceResponse {
    /// Price of the ticket in HOPR tokens.
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "0.03 wxHOPR")]
    price: HoprBalance,
}

/// Gets the current ticket price.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/network/price"),
        description = "Get the current ticket price",
        responses(
            (status = 200, description = "Current ticket price", body = TicketPriceResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Network"
    )]
pub(super) async fn price(State(state): State<Arc<InternalState>>) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    match hopr.get_ticket_price().await {
        Ok(price) => (StatusCode::OK, Json(TicketPriceResponse { price })).into_response(),
        Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "probability": 0.5
}))]
#[serde(rename_all = "camelCase")]
/// Contains the winning probability of a ticket.
pub(crate) struct TicketProbabilityResponse {
    #[schema(example = 0.5)]
    /// Winning probability of a ticket.
    probability: f64,
}

/// Gets the current minimum incoming ticket winning probability defined by the network.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/network/probability"),
        description = "Get the current minimum incoming ticket winning probability defined by the network",
        responses(
            (status = 200, description = "Minimum incoming ticket winning probability defined by the network", body = TicketProbabilityResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Network"
    )]
pub(super) async fn probability(State(state): State<Arc<InternalState>>) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    match hopr.get_minimum_incoming_ticket_win_probability().await {
        Ok(p) => (
            StatusCode::OK,
            Json(TicketProbabilityResponse { probability: p.into() }),
        )
            .into_response(),
        Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }
}
