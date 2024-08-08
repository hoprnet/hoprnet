use axum::{
    extract::{Json, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::sync::Arc;

use hopr_lib::{Address, Balance, BalanceType};

use crate::{ApiErrorStatus, InternalState, BASE_PATH};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
        "hopr": "12D3KooWJmLm8FnBfvYQ5BAZ5qcYBxQFFBzAAEYUBUNJNE8cRsYS",
        "native": "0x07eaf07d6624f741e04f4092a755a9027aaab7f6"
    }))]
#[serde(rename_all = "camelCase")]
pub(crate) struct AccountAddressesResponse {
    native: String,
    hopr: String,
}

/// Get node's HOPR and native addresses.
///
/// HOPR address is represented by the P2P PeerId and can be used by other node owner to interact with this node.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/account/addresses"),
        responses(
            (status = 200, description = "The node's public addresses", body = AccountAddressesResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Account",
    )]
pub(super) async fn addresses(State(state): State<Arc<InternalState>>) -> impl IntoResponse {
    let addresses = AccountAddressesResponse {
        native: state.hopr.me_onchain().to_string(),
        hopr: state.hopr.me_peer_id().to_string(),
    };

    (StatusCode::OK, Json(addresses)).into_response()
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
        "hopr": "2000000000000000000000",
        "native": "9999563581204904000",
        "safeHopr": "2000000000000000000000",
        "safeHoprAllowance": "115792089237316195423570985008687907853269984665640564039457584007913129639935",
        "safeNative": "10000000000000000000"
    }))]
#[serde(rename_all = "camelCase")]
pub(crate) struct AccountBalancesResponse {
    safe_native: String,
    native: String,
    safe_hopr: String,
    hopr: String,
    safe_hopr_allowance: String,
}

/// Get node's and associated Safe's HOPR and native balances as the allowance for HOPR
/// tokens to be drawn by HoprChannels from Safe.
///
/// HOPR tokens from the Safe balance are used to fund the payment channels between this
/// node and other nodes on the network.
/// NATIVE balance of the Node is used to pay for the gas fees for the blockchain.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/account/balances"),
        responses(
            (status = 200, description = "The node's HOPR and Safe balances", body = AccountBalancesResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Account",
    )]
pub(super) async fn balances(State(state): State<Arc<InternalState>>) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    let mut account_balances = AccountBalancesResponse::default();

    match hopr.get_balance(BalanceType::Native).await {
        Ok(v) => account_balances.native = v.to_value_string(),
        Err(e) => return (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }

    match hopr.get_balance(BalanceType::HOPR).await {
        Ok(v) => account_balances.hopr = v.to_value_string(),
        Err(e) => return (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }

    match hopr.get_safe_balance(BalanceType::Native).await {
        Ok(v) => account_balances.safe_native = v.to_value_string(),
        Err(e) => return (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }

    match hopr.get_safe_balance(BalanceType::HOPR).await {
        Ok(v) => account_balances.safe_hopr = v.to_value_string(),
        Err(e) => return (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }

    match hopr.safe_allowance().await {
        Ok(v) => account_balances.safe_hopr_allowance = v.to_value_string(),
        Err(e) => return (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }

    (StatusCode::OK, Json(account_balances)).into_response()
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
        "address": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe",
        "amount": "20000",
        "currency": "HOPR"
    }))]
#[serde(rename_all = "camelCase")]
pub(crate) struct WithdrawBodyRequest {
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    currency: BalanceType,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    amount: String,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    address: Address,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
        "receipt": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe",
    }))]
#[serde(rename_all = "camelCase")]
pub(crate) struct WithdrawResponse {
    receipt: String,
}

/// Withdraw funds from this node to the ethereum wallet address.
///
/// Both NATIVE or HOPR can be withdrawn using this method.
#[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/account/withdraw"),
        request_body(
            content = WithdrawBodyRequest,
            content_type = "application/json"),
        responses(
            (status = 200, description = "The node's funds have been withdrawn", body = WithdrawResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Account",
    )]
pub(super) async fn withdraw(
    State(state): State<Arc<InternalState>>,
    Json(req_data): Json<WithdrawBodyRequest>,
) -> impl IntoResponse {
    match state
        .hopr
        .withdraw(
            req_data.address,
            Balance::new_from_str(&req_data.amount, req_data.currency),
        )
        .await
    {
        Ok(receipt) => (
            StatusCode::OK,
            Json(WithdrawResponse {
                receipt: receipt.to_string(),
            }),
        )
            .into_response(),
        Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }
}
