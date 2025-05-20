use axum::{
    extract::{Json, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::sync::Arc;

use hopr_lib::{
    errors::{HoprLibError, HoprStatusError},
    Address, Balance, BalanceType, U256,
};

use crate::{ApiError, ApiErrorStatus, InternalState, BASE_PATH};

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "hopr": "12D3KooWJmLm8FnBfvYQ5BAZ5qcYBxQFFBzAAEYUBUNJNE8cRsYS",
    "native": "0x07eaf07d6624f741e04f4092a755a9027aaab7f6"
}))]
#[serde(rename_all = "camelCase")]
/// Contains the node's HOPR and native addresses.
pub(crate) struct AccountAddressesResponse {
    #[schema(example = "0x07eaf07d6624f741e04f4092a755a9027aaab7f6")]
    native: String,
    #[schema(example = "12D3KooWJmLm8FnBfvYQ5BAZ5qcYBxQFFBzAAEYUBUNJNE8cRsYS")]
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
        native: state.hopr.me_onchain().to_checksum(),
        hopr: state.hopr.me_peer_id().to_string(),
    };

    (StatusCode::OK, Json(addresses)).into_response()
}

#[derive(Debug, Default, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "hopr": "2000000000000000000000",
        "native": "9999563581204904000",
        "safeHopr": "2000000000000000000000",
        "safeHoprAllowance": "115792089237316195423570985008687907853269984665640564039457584007913129639935",
        "safeNative": "10000000000000000000"
    }))]
#[serde(rename_all = "camelCase")]
/// Contains all node's and safe's related balances.
pub(crate) struct AccountBalancesResponse {
    #[schema(example = "10000000000000000000")]
    safe_native: String,
    #[schema(example = "9999563581204904000")]
    native: String,
    #[schema(example = "2000000000000000000000")]
    safe_hopr: String,
    #[schema(example = "2000000000000000000000")]
    hopr: String,
    #[schema(example = "115792089237316195423570985008687907853269984665640564039457584007913129639935")]
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

fn deserialize_u256_value_from_str<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let s: &str = serde::de::Deserialize::deserialize(deserializer)?;
    let v: u128 = s.parse().map_err(serde::de::Error::custom)?;
    Ok(U256::from(v))
}

// #[deprecated(
//     since = "3.2.0",
//     note = "The `BalanceType` enum deserialization using all capitals is deprecated and will be removed in hoprd v3.0 REST API"
// )]
fn deserialize_balance_type<'de, D>(deserializer: D) -> Result<BalanceType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let buf = <String as serde::Deserialize>::deserialize(deserializer)?;

    match buf.as_str() {
        "Native" | "NATIVE" => Ok(BalanceType::Native),
        "HOPR" => Ok(BalanceType::HOPR),
        _ => Err(serde::de::Error::custom("Unsupported balance type")),
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[schema(example = json!({
        "address": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe",
        "amount": "20000",
        "currency": "HOPR"
    }))]
#[serde(rename_all = "camelCase")]
/// Request body for the withdraw endpoint.
pub(crate) struct WithdrawBodyRequest {
    // #[serde_as(as = "DisplayFromStr")]
    #[serde(deserialize_with = "deserialize_balance_type")]
    #[schema(value_type = String, example = "HOPR")]
    currency: BalanceType,
    #[serde(deserialize_with = "deserialize_u256_value_from_str")]
    #[schema(value_type = String, example= "20000")]
    amount: U256,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example= "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe")]
    address: Address,
}

#[derive(Debug, Default, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "receipt": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe",
    }))]
#[serde(rename_all = "camelCase")]
/// Response body for the withdraw endpoint.
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
    match state
        .hopr
        .withdraw(req_data.address, Balance::new(req_data.amount, req_data.currency))
        .await
    {
        Ok(receipt) => (
            StatusCode::OK,
            Json(WithdrawResponse {
                receipt: receipt.to_string(),
            }),
        )
            .into_response(),
        Err(HoprLibError::StatusError(HoprStatusError::NotThereYet(_, _))) => {
            (StatusCode::PRECONDITION_FAILED, ApiErrorStatus::NotReady).into_response()
        }

        Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use hopr_lib::Address;

    use crate::account::WithdrawBodyRequest;

    #[test]
    fn withdraw_input_data_should_be_convertible_from_string() -> anyhow::Result<()> {
        let expected = WithdrawBodyRequest {
            currency: "HOPR".parse().unwrap(),
            amount: 20000.into(),
            address: Address::from_str("0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe")?,
        };

        let actual: WithdrawBodyRequest = serde_json::from_str(
            r#"{
            "currency": "HOPR",
            "amount": "20000",
            "address": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe"}"#,
        )?;

        assert_eq!(actual, expected);

        Ok(())
    }
}
