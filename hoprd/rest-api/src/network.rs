use axum::{
    extract::{Json, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;

use crate::{ApiErrorStatus, InternalState, BASE_PATH};

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TicketPriceResponse {
    /// Price of the ticket in HOPR tokens.
    price: String,
}

/// Obtains the current ticket price.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/network/price"),
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
        Ok(Some(price)) => (
            StatusCode::OK,
            Json(TicketPriceResponse {
                price: price.to_string(),
            }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiErrorStatus::UnknownFailure("The ticket price is not available".into()),
        )
            .into_response(),
        Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TicketProbabilityResponse {
    /// Winning probability of a ticket.
    probability: f64,
}

/// Obtains the current ticket winning probability.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/network/probability"),
        responses(
            (status = 200, description = "Winning ticket probablity", body = TicketProbabilityResponse),
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

    match hopr.get_ticket_probability().await {
        Ok(probability) => (
            StatusCode::OK,
            Json(TicketProbabilityResponse {
                probability: probability,
            }),
        )
            .into_response(),
        Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }
}
