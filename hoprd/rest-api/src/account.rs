use std::{str::FromStr, sync::Arc};

use axum::{
    extract::{Json, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use hopr_lib::{
    Address, HoprBalance, WxHOPR, XDai, XDaiBalance,
    errors::{HoprLibError, HoprStatusError},
};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};

use crate::{ApiError, ApiErrorStatus, BASE_PATH, InternalState};

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "native": "0x07eaf07d6624f741e04f4092a755a9027aaab7f6"
}))]
#[serde(rename_all = "camelCase")]
/// Contains the node's native addresses.
pub(crate) struct AccountAddressesResponse {
    #[schema(example = "0x07eaf07d6624f741e04f4092a755a9027aaab7f6")]
    native: String,
}

/// Get node's native addresses.
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
        native: state.hopr.me_onchain().to_checksum(),
    };

    (StatusCode::OK, Json(addresses)).into_response()
}

#[serde_as]
#[derive(Debug, Default, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "hopr": "1000 wxHOPR",
        "native": "0.1 xDai",
        "safeHopr": "1000 wxHOPR",
        "safeHoprAllowance": "10000 wxHOPR",
        "safeNative": "0.1 xDai"
    }))]
#[serde(rename_all = "camelCase")]
/// Contains all node's and safe's related balances.
pub(crate) struct AccountBalancesResponse {
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "0.1 xDai")]
    safe_native: XDaiBalance,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "0.1 xDai")]
    native: XDaiBalance,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "2000 wxHOPR")]
    safe_hopr: HoprBalance,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "2000 wxHOPR")]
    hopr: HoprBalance,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "10000 wxHOPR")]
    safe_hopr_allowance: HoprBalance,
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

    match hopr.get_balance::<XDai>().await {
        Ok(v) => account_balances.native = v,
        Err(e) => return (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }

    match hopr.get_balance::<WxHOPR>().await {
        Ok(v) => account_balances.hopr = v,
        Err(e) => return (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }

    match hopr.get_safe_balance::<XDai>().await {
        Ok(v) => account_balances.safe_native = v,
        Err(e) => return (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }

    match hopr.get_safe_balance::<WxHOPR>().await {
        Ok(v) => account_balances.safe_hopr = v,
        Err(e) => return (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }

    match hopr.safe_allowance().await {
        Ok(v) => account_balances.safe_hopr_allowance = v,
        Err(e) => return (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }

    (StatusCode::OK, Json(account_balances)).into_response()
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[schema(example = json!({
        "address": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe",
        "amount": "20000 wxHOPR",
    }))]
#[serde(rename_all = "camelCase")]
/// Request body for the withdrawal endpoint.
pub(crate) struct WithdrawBodyRequest {
    #[schema(value_type = String, example= "20000 wxHOPR")]
    amount: String,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example= "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe")]
    address: Address,
}

#[derive(Debug, Default, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "receipt": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe",
    }))]
#[serde(rename_all = "camelCase")]
/// Response body for the withdrawal endpoint.
pub(crate) struct WithdrawResponse {
    #[schema(example = "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe")]
    receipt: String,
}

/// Withdraw funds from this node to the ethereum wallet address.
///
/// Both Native or HOPR can be withdrawn using this method.
#[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/account/withdraw"),
        description = "Withdraw funds from this node to the ethereum wallet address",
        request_body(
            content = WithdrawBodyRequest,
            content_type = "application/json",
            description = "Request body for the withdraw endpoint",
        ),
        responses(
            (status = 200, description = "The node's funds have been withdrawn", body = WithdrawResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 412, description = "The node is not ready."),
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
    let res = if let Ok(native) = XDaiBalance::from_str(&req_data.amount) {
        state.hopr.withdraw_native(req_data.address, native).await
    } else if let Ok(hopr) = HoprBalance::from_str(&req_data.amount) {
        state.hopr.withdraw_tokens(req_data.address, hopr).await
    } else {
        Err(HoprLibError::GeneralError("invalid currency".into()))
    };

    match res {
        Ok(receipt) => (
            StatusCode::OK,
            Json(WithdrawResponse {
                receipt: receipt.to_string(),
            }),
        )
            .into_response(),
        Err(HoprLibError::StatusError(HoprStatusError::NotThereYet(..))) => {
            (StatusCode::PRECONDITION_FAILED, ApiErrorStatus::NotReady).into_response()
        }

        Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }
}
