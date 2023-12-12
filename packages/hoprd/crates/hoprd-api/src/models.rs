#![allow(unused_qualifications)]

use validator::Validate;

#[cfg(any(feature = "client", feature = "server"))]
use crate::header;
use crate::models;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AccountGetAddresses200Response {
    /// Blockchain-native account address. Can be funded from external wallets (starts with **0x...**). It **can't be used** internally to send / receive messages, open / close payment channels.
    #[serde(rename = "native")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native: Option<String>,

    /// HOPR account address, also called a PeerId. Used to send / receive messages, open / close payment channels.
    #[serde(rename = "hopr")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hopr: Option<String>,
}

impl AccountGetAddresses200Response {
    #[allow(clippy::new_without_default)]
    pub fn new() -> AccountGetAddresses200Response {
        AccountGetAddresses200Response {
            native: None,
            hopr: None,
        }
    }
}

/// Converts the AccountGetAddresses200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for AccountGetAddresses200Response {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            self.native
                .as_ref()
                .map(|native| vec!["native".to_string(), native.to_string()].join(",")),
            self.hopr
                .as_ref()
                .map(|hopr| vec!["hopr".to_string(), hopr.to_string()].join(",")),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AccountGetAddresses200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AccountGetAddresses200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub native: Vec<String>,
            pub hopr: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing AccountGetAddresses200Response".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "native" => intermediate_rep
                        .native
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "hopr" => intermediate_rep
                        .hopr
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing AccountGetAddresses200Response".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AccountGetAddresses200Response {
            native: intermediate_rep.native.into_iter().next(),
            hopr: intermediate_rep.hopr.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AccountGetAddresses200Response> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<AccountGetAddresses200Response>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<AccountGetAddresses200Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for AccountGetAddresses200Response - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<AccountGetAddresses200Response> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <AccountGetAddresses200Response as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into AccountGetAddresses200Response - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AccountGetBalances200Response {
    /// Amount of NATIVE (ETH) balance in the smallest unit. Used only for gas fees on the blockchain the current release is running on. For example, when you will open or close the payment channel, it will use gas fees to execute this action.
    #[serde(rename = "native")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native: Option<String>,

    /// Amount of HOPR tokens in the smallest unit. Used for funding payment channels.
    #[serde(rename = "hopr")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hopr: Option<String>,

    /// Amount of NATIVE (ETH) balance in the smallest unit. Used only for gas fees on the blockchain the current release is running on. For example, when you will open or close the payment channel, it will use gas fees to execute this action.
    #[serde(rename = "safeNative")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safe_native: Option<String>,

    /// Amount of HOPR tokens in the smallest unit. Used for funding payment channels.
    #[serde(rename = "safeHopr")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safe_hopr: Option<String>,

    /// Amount of HOPR tokens in the smallest unit. Used for funding payment channels.
    #[serde(rename = "safeHoprAllowance")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safe_hopr_allowance: Option<String>,
}

impl AccountGetBalances200Response {
    #[allow(clippy::new_without_default)]
    pub fn new() -> AccountGetBalances200Response {
        AccountGetBalances200Response {
            native: None,
            hopr: None,
            safe_native: None,
            safe_hopr: None,
            safe_hopr_allowance: None,
        }
    }
}

/// Converts the AccountGetBalances200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for AccountGetBalances200Response {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            self.native
                .as_ref()
                .map(|native| vec!["native".to_string(), native.to_string()].join(",")),
            self.hopr
                .as_ref()
                .map(|hopr| vec!["hopr".to_string(), hopr.to_string()].join(",")),
            self.safe_native
                .as_ref()
                .map(|safe_native| vec!["safeNative".to_string(), safe_native.to_string()].join(",")),
            self.safe_hopr
                .as_ref()
                .map(|safe_hopr| vec!["safeHopr".to_string(), safe_hopr.to_string()].join(",")),
            self.safe_hopr_allowance.as_ref().map(|safe_hopr_allowance| {
                vec!["safeHoprAllowance".to_string(), safe_hopr_allowance.to_string()].join(",")
            }),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AccountGetBalances200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AccountGetBalances200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub native: Vec<String>,
            pub hopr: Vec<String>,
            pub safe_native: Vec<String>,
            pub safe_hopr: Vec<String>,
            pub safe_hopr_allowance: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing AccountGetBalances200Response".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "native" => intermediate_rep
                        .native
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "hopr" => intermediate_rep
                        .hopr
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "safeNative" => intermediate_rep
                        .safe_native
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "safeHopr" => intermediate_rep
                        .safe_hopr
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "safeHoprAllowance" => intermediate_rep
                        .safe_hopr_allowance
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing AccountGetBalances200Response".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AccountGetBalances200Response {
            native: intermediate_rep.native.into_iter().next(),
            hopr: intermediate_rep.hopr.into_iter().next(),
            safe_native: intermediate_rep.safe_native.into_iter().next(),
            safe_hopr: intermediate_rep.safe_hopr.into_iter().next(),
            safe_hopr_allowance: intermediate_rep.safe_hopr_allowance.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AccountGetBalances200Response> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<AccountGetBalances200Response>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<AccountGetBalances200Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for AccountGetBalances200Response - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<AccountGetBalances200Response> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <AccountGetBalances200Response as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into AccountGetBalances200Response - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AccountWithdraw200Response {
    /// Withdraw txn hash that can be used to check details of the transaction on ethereum chain.
    #[serde(rename = "receipt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt: Option<String>,
}

impl AccountWithdraw200Response {
    #[allow(clippy::new_without_default)]
    pub fn new() -> AccountWithdraw200Response {
        AccountWithdraw200Response { receipt: None }
    }
}

/// Converts the AccountWithdraw200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for AccountWithdraw200Response {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![self
            .receipt
            .as_ref()
            .map(|receipt| vec!["receipt".to_string(), receipt.to_string()].join(","))];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AccountWithdraw200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AccountWithdraw200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub receipt: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing AccountWithdraw200Response".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "receipt" => intermediate_rep
                        .receipt
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing AccountWithdraw200Response".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AccountWithdraw200Response {
            receipt: intermediate_rep.receipt.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AccountWithdraw200Response> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<AccountWithdraw200Response>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<AccountWithdraw200Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for AccountWithdraw200Response - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<AccountWithdraw200Response> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <AccountWithdraw200Response as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into AccountWithdraw200Response - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AccountWithdraw422Response {
    #[serde(rename = "status")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    #[serde(rename = "error")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl AccountWithdraw422Response {
    #[allow(clippy::new_without_default)]
    pub fn new() -> AccountWithdraw422Response {
        AccountWithdraw422Response {
            status: None,
            error: None,
        }
    }
}

/// Converts the AccountWithdraw422Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for AccountWithdraw422Response {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            self.status
                .as_ref()
                .map(|status| vec!["status".to_string(), status.to_string()].join(",")),
            self.error
                .as_ref()
                .map(|error| vec!["error".to_string(), error.to_string()].join(",")),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AccountWithdraw422Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AccountWithdraw422Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub status: Vec<String>,
            pub error: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing AccountWithdraw422Response".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "status" => intermediate_rep
                        .status
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "error" => intermediate_rep
                        .error
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing AccountWithdraw422Response".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AccountWithdraw422Response {
            status: intermediate_rep.status.into_iter().next(),
            error: intermediate_rep.error.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AccountWithdraw422Response> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<AccountWithdraw422Response>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<AccountWithdraw422Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for AccountWithdraw422Response - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<AccountWithdraw422Response> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <AccountWithdraw422Response as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into AccountWithdraw422Response - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AccountWithdrawRequest {
    #[serde(rename = "currency")]
    pub currency: models::Currency,

    /// Amount to withdraw in the currency's smallest unit.
    #[serde(rename = "amount")]
    pub amount: String,

    /// Blockchain-native account address. Can be funded from external wallets (starts with **0x...**). It **can't be used** internally to send / receive messages, open / close payment channels.
    #[serde(rename = "ethereumAddress")]
    pub ethereum_address: String,
}

impl AccountWithdrawRequest {
    #[allow(clippy::new_without_default)]
    pub fn new(currency: models::Currency, amount: String, ethereum_address: String) -> AccountWithdrawRequest {
        AccountWithdrawRequest {
            currency,
            amount,
            ethereum_address,
        }
    }
}

/// Converts the AccountWithdrawRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for AccountWithdrawRequest {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            // Skipping currency in query parameter serialization
            Some("amount".to_string()),
            Some(self.amount.to_string()),
            Some("ethereumAddress".to_string()),
            Some(self.ethereum_address.to_string()),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AccountWithdrawRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AccountWithdrawRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub currency: Vec<models::Currency>,
            pub amount: Vec<String>,
            pub ethereum_address: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err("Missing value while parsing AccountWithdrawRequest".to_string())
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "currency" => intermediate_rep
                        .currency
                        .push(<models::Currency as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "amount" => intermediate_rep
                        .amount
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "ethereumAddress" => intermediate_rep
                        .ethereum_address
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing AccountWithdrawRequest".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AccountWithdrawRequest {
            currency: intermediate_rep
                .currency
                .into_iter()
                .next()
                .ok_or_else(|| "currency missing in AccountWithdrawRequest".to_string())?,
            amount: intermediate_rep
                .amount
                .into_iter()
                .next()
                .ok_or_else(|| "amount missing in AccountWithdrawRequest".to_string())?,
            ethereum_address: intermediate_rep
                .ethereum_address
                .into_iter()
                .next()
                .ok_or_else(|| "ethereumAddress missing in AccountWithdrawRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AccountWithdrawRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<AccountWithdrawRequest>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<AccountWithdrawRequest>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for AccountWithdrawRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<AccountWithdrawRequest> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <AccountWithdrawRequest as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into AccountWithdrawRequest - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AliasesGetAlias200Response {
    /// HOPR account address, also called a PeerId. Used to send / receive messages, open / close payment channels.
    #[serde(rename = "peerId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_id: Option<String>,
}

impl AliasesGetAlias200Response {
    #[allow(clippy::new_without_default)]
    pub fn new() -> AliasesGetAlias200Response {
        AliasesGetAlias200Response { peer_id: None }
    }
}

/// Converts the AliasesGetAlias200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for AliasesGetAlias200Response {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![self
            .peer_id
            .as_ref()
            .map(|peer_id| vec!["peerId".to_string(), peer_id.to_string()].join(","))];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AliasesGetAlias200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AliasesGetAlias200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub peer_id: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing AliasesGetAlias200Response".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "peerId" => intermediate_rep
                        .peer_id
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing AliasesGetAlias200Response".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AliasesGetAlias200Response {
            peer_id: intermediate_rep.peer_id.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AliasesGetAlias200Response> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<AliasesGetAlias200Response>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<AliasesGetAlias200Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for AliasesGetAlias200Response - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<AliasesGetAlias200Response> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <AliasesGetAlias200Response as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into AliasesGetAlias200Response - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AliasesGetAliases200Response {
    /// HOPR account address, also called a PeerId. Used to send / receive messages, open / close payment channels.
    #[serde(rename = "alice")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alice: Option<String>,

    /// HOPR account address, also called a PeerId. Used to send / receive messages, open / close payment channels.
    #[serde(rename = "bob")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bob: Option<String>,
}

impl AliasesGetAliases200Response {
    #[allow(clippy::new_without_default)]
    pub fn new() -> AliasesGetAliases200Response {
        AliasesGetAliases200Response { alice: None, bob: None }
    }
}

/// Converts the AliasesGetAliases200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for AliasesGetAliases200Response {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            self.alice
                .as_ref()
                .map(|alice| vec!["alice".to_string(), alice.to_string()].join(",")),
            self.bob
                .as_ref()
                .map(|bob| vec!["bob".to_string(), bob.to_string()].join(",")),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AliasesGetAliases200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AliasesGetAliases200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub alice: Vec<String>,
            pub bob: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing AliasesGetAliases200Response".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "alice" => intermediate_rep
                        .alice
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "bob" => intermediate_rep
                        .bob
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing AliasesGetAliases200Response".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AliasesGetAliases200Response {
            alice: intermediate_rep.alice.into_iter().next(),
            bob: intermediate_rep.bob.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AliasesGetAliases200Response> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<AliasesGetAliases200Response>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<AliasesGetAliases200Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for AliasesGetAliases200Response - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<AliasesGetAliases200Response> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <AliasesGetAliases200Response as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into AliasesGetAliases200Response - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AliasesSetAliasRequest {
    /// PeerId that we want to set alias to.
    #[serde(rename = "peerId")]
    pub peer_id: String,

    /// Alias that we want to attach to peerId.
    #[serde(rename = "alias")]
    pub alias: String,
}

impl AliasesSetAliasRequest {
    #[allow(clippy::new_without_default)]
    pub fn new(peer_id: String, alias: String) -> AliasesSetAliasRequest {
        AliasesSetAliasRequest { peer_id, alias }
    }
}

/// Converts the AliasesSetAliasRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for AliasesSetAliasRequest {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            Some("peerId".to_string()),
            Some(self.peer_id.to_string()),
            Some("alias".to_string()),
            Some(self.alias.to_string()),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AliasesSetAliasRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AliasesSetAliasRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub peer_id: Vec<String>,
            pub alias: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err("Missing value while parsing AliasesSetAliasRequest".to_string())
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "peerId" => intermediate_rep
                        .peer_id
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "alias" => intermediate_rep
                        .alias
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing AliasesSetAliasRequest".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AliasesSetAliasRequest {
            peer_id: intermediate_rep
                .peer_id
                .into_iter()
                .next()
                .ok_or_else(|| "peerId missing in AliasesSetAliasRequest".to_string())?,
            alias: intermediate_rep
                .alias
                .into_iter()
                .next()
                .ok_or_else(|| "alias missing in AliasesSetAliasRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AliasesSetAliasRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<AliasesSetAliasRequest>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<AliasesSetAliasRequest>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for AliasesSetAliasRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<AliasesSetAliasRequest> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <AliasesSetAliasRequest as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into AliasesSetAliasRequest - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Channel {
    /// Channel can be either incomming or outgoing. Incomming means that other node can send messages using this node as relay. Outgoing means that this node can use other node to send message as realy.
    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    /// The unique identifier of a unidirectional HOPR channel.
    #[serde(rename = "id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,

    /// HOPR account address, also called a PeerId. Used to send / receive messages, open / close payment channels.
    #[serde(rename = "peerId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_id: Option<String>,

    #[serde(rename = "status")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<models::ChannelStatus>,

    /// Amount of HOPR tokens in the smallest unit. Used for funding payment channels.
    #[serde(rename = "balance")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance: Option<String>,
}

impl Channel {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Channel {
        Channel {
            r#type: None,
            id: None,
            peer_id: None,
            status: None,
            balance: None,
        }
    }
}

/// Converts the Channel value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Channel {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            self.r#type
                .as_ref()
                .map(|r#type| vec!["type".to_string(), r#type.to_string()].join(",")),
            // Skipping id in query parameter serialization
            self.peer_id
                .as_ref()
                .map(|peer_id| vec!["peerId".to_string(), peer_id.to_string()].join(",")),
            // Skipping status in query parameter serialization
            self.balance
                .as_ref()
                .map(|balance| vec!["balance".to_string(), balance.to_string()].join(",")),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Channel value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Channel {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub r#type: Vec<String>,
            pub id: Vec<serde_json::Value>,
            pub peer_id: Vec<String>,
            pub status: Vec<models::ChannelStatus>,
            pub balance: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing Channel".to_string()),
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "type" => intermediate_rep
                        .r#type
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "id" => intermediate_rep
                        .id
                        .push(<serde_json::Value as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "peerId" => intermediate_rep
                        .peer_id
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "status" => intermediate_rep
                        .status
                        .push(<models::ChannelStatus as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "balance" => intermediate_rep
                        .balance
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing Channel".to_string()),
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Channel {
            r#type: intermediate_rep.r#type.into_iter().next(),
            id: intermediate_rep.id.into_iter().next(),
            peer_id: intermediate_rep.peer_id.into_iter().next(),
            status: intermediate_rep.status.into_iter().next(),
            balance: intermediate_rep.balance.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Channel> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Channel>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<Channel>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Channel - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Channel> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <Channel as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into Channel - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

/// The unique identifier of a unidirectional HOPR channel.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ChannelId(serde_json::Value);

impl std::convert::From<serde_json::Value> for ChannelId {
    fn from(x: serde_json::Value) -> Self {
        ChannelId(x)
    }
}

impl std::convert::From<ChannelId> for serde_json::Value {
    fn from(x: ChannelId) -> Self {
        x.0
    }
}

impl std::ops::Deref for ChannelId {
    type Target = serde_json::Value;
    fn deref(&self) -> &serde_json::Value {
        &self.0
    }
}

impl std::ops::DerefMut for ChannelId {
    fn deref_mut(&mut self) -> &mut serde_json::Value {
        &mut self.0
    }
}

/// Status of the channel can be: Open, PendingToClose, or Closed.
/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum ChannelStatus {
    #[serde(rename = "Open")]
    Open,
    #[serde(rename = "PendingToClose")]
    PendingToClose,
    #[serde(rename = "Closed")]
    Closed,
}

impl std::fmt::Display for ChannelStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ChannelStatus::Open => write!(f, "Open"),
            ChannelStatus::PendingToClose => write!(f, "PendingToClose"),
            ChannelStatus::Closed => write!(f, "Closed"),
        }
    }
}

impl std::str::FromStr for ChannelStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Open" => std::result::Result::Ok(ChannelStatus::Open),
            "PendingToClose" => std::result::Result::Ok(ChannelStatus::PendingToClose),
            "Closed" => std::result::Result::Ok(ChannelStatus::Closed),
            _ => std::result::Result::Err(format!("Value not valid: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ChannelTopology {
    /// The unique identifier of a unidirectional HOPR channel.
    #[serde(rename = "channelId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<serde_json::Value>,

    /// HOPR account address, also called a PeerId. Used to send / receive messages, open / close payment channels.
    #[serde(rename = "sourcePeerId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_peer_id: Option<String>,

    /// HOPR account address, also called a PeerId. Used to send / receive messages, open / close payment channels.
    #[serde(rename = "destinationPeerId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_peer_id: Option<String>,

    /// Blockchain-native account address. Can be funded from external wallets (starts with **0x...**). It **can't be used** internally to send / receive messages, open / close payment channels.
    #[serde(rename = "sourceAddress")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_address: Option<String>,

    /// Blockchain-native account address. Can be funded from external wallets (starts with **0x...**). It **can't be used** internally to send / receive messages, open / close payment channels.
    #[serde(rename = "destinationAddress")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_address: Option<String>,

    /// Amount of HOPR tokens in the smallest unit. Used for funding payment channels.
    #[serde(rename = "balance")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance: Option<String>,

    #[serde(rename = "status")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<models::ChannelStatus>,

    /// Each ticket is labeled by an ongoing serial number named ticket index i and its current value is stored in the smart contract.
    #[serde(rename = "ticketIndex")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket_index: Option<String>,

    /// Payment channels might run through multiple open and close sequences, this epoch tracks the sequence.
    #[serde(rename = "channelEpoch")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_epoch: Option<String>,

    /// Time when the channel can be closed
    #[serde(rename = "closureTime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closure_time: Option<String>,
}

impl ChannelTopology {
    #[allow(clippy::new_without_default)]
    pub fn new() -> ChannelTopology {
        ChannelTopology {
            channel_id: None,
            source_peer_id: None,
            destination_peer_id: None,
            source_address: None,
            destination_address: None,
            balance: None,
            status: None,
            ticket_index: None,
            channel_epoch: None,
            closure_time: None,
        }
    }
}

/// Converts the ChannelTopology value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ChannelTopology {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            // Skipping channelId in query parameter serialization
            self.source_peer_id
                .as_ref()
                .map(|source_peer_id| vec!["sourcePeerId".to_string(), source_peer_id.to_string()].join(",")),
            self.destination_peer_id.as_ref().map(|destination_peer_id| {
                vec!["destinationPeerId".to_string(), destination_peer_id.to_string()].join(",")
            }),
            self.source_address
                .as_ref()
                .map(|source_address| vec!["sourceAddress".to_string(), source_address.to_string()].join(",")),
            self.destination_address.as_ref().map(|destination_address| {
                vec!["destinationAddress".to_string(), destination_address.to_string()].join(",")
            }),
            self.balance
                .as_ref()
                .map(|balance| vec!["balance".to_string(), balance.to_string()].join(",")),
            // Skipping status in query parameter serialization
            self.ticket_index
                .as_ref()
                .map(|ticket_index| vec!["ticketIndex".to_string(), ticket_index.to_string()].join(",")),
            self.channel_epoch
                .as_ref()
                .map(|channel_epoch| vec!["channelEpoch".to_string(), channel_epoch.to_string()].join(",")),
            self.closure_time
                .as_ref()
                .map(|closure_time| vec!["closureTime".to_string(), closure_time.to_string()].join(",")),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ChannelTopology value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ChannelTopology {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub channel_id: Vec<serde_json::Value>,
            pub source_peer_id: Vec<String>,
            pub destination_peer_id: Vec<String>,
            pub source_address: Vec<String>,
            pub destination_address: Vec<String>,
            pub balance: Vec<String>,
            pub status: Vec<models::ChannelStatus>,
            pub ticket_index: Vec<String>,
            pub channel_epoch: Vec<String>,
            pub closure_time: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing ChannelTopology".to_string()),
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "channelId" => intermediate_rep
                        .channel_id
                        .push(<serde_json::Value as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "sourcePeerId" => intermediate_rep
                        .source_peer_id
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "destinationPeerId" => intermediate_rep
                        .destination_peer_id
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "sourceAddress" => intermediate_rep
                        .source_address
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "destinationAddress" => intermediate_rep
                        .destination_address
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "balance" => intermediate_rep
                        .balance
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "status" => intermediate_rep
                        .status
                        .push(<models::ChannelStatus as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "ticketIndex" => intermediate_rep
                        .ticket_index
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "channelEpoch" => intermediate_rep
                        .channel_epoch
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "closureTime" => intermediate_rep
                        .closure_time
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing ChannelTopology".to_string()),
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ChannelTopology {
            channel_id: intermediate_rep.channel_id.into_iter().next(),
            source_peer_id: intermediate_rep.source_peer_id.into_iter().next(),
            destination_peer_id: intermediate_rep.destination_peer_id.into_iter().next(),
            source_address: intermediate_rep.source_address.into_iter().next(),
            destination_address: intermediate_rep.destination_address.into_iter().next(),
            balance: intermediate_rep.balance.into_iter().next(),
            status: intermediate_rep.status.into_iter().next(),
            ticket_index: intermediate_rep.ticket_index.into_iter().next(),
            channel_epoch: intermediate_rep.channel_epoch.into_iter().next(),
            closure_time: intermediate_rep.closure_time.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ChannelTopology> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ChannelTopology>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<ChannelTopology>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ChannelTopology - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ChannelTopology> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <ChannelTopology as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into ChannelTopology - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ChannelsCloseChannel200Response {
    /// Receipt of the closing transaction
    #[serde(rename = "receipt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt: Option<String>,

    /// Current status of the channel
    #[serde(rename = "channelStatus")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_status: Option<String>,
}

impl ChannelsCloseChannel200Response {
    #[allow(clippy::new_without_default)]
    pub fn new() -> ChannelsCloseChannel200Response {
        ChannelsCloseChannel200Response {
            receipt: None,
            channel_status: None,
        }
    }
}

/// Converts the ChannelsCloseChannel200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ChannelsCloseChannel200Response {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            self.receipt
                .as_ref()
                .map(|receipt| vec!["receipt".to_string(), receipt.to_string()].join(",")),
            self.channel_status
                .as_ref()
                .map(|channel_status| vec!["channelStatus".to_string(), channel_status.to_string()].join(",")),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ChannelsCloseChannel200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ChannelsCloseChannel200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub receipt: Vec<String>,
            pub channel_status: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ChannelsCloseChannel200Response".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "receipt" => intermediate_rep
                        .receipt
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "channelStatus" => intermediate_rep
                        .channel_status
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ChannelsCloseChannel200Response".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ChannelsCloseChannel200Response {
            receipt: intermediate_rep.receipt.into_iter().next(),
            channel_status: intermediate_rep.channel_status.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ChannelsCloseChannel200Response> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ChannelsCloseChannel200Response>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ChannelsCloseChannel200Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ChannelsCloseChannel200Response - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ChannelsCloseChannel200Response> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ChannelsCloseChannel200Response as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ChannelsCloseChannel200Response - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ChannelsFundChannel200Response {
    /// Receipt of the funding transaction
    #[serde(rename = "receipt")]
    pub receipt: String,
}

impl ChannelsFundChannel200Response {
    #[allow(clippy::new_without_default)]
    pub fn new(receipt: String) -> ChannelsFundChannel200Response {
        ChannelsFundChannel200Response { receipt }
    }
}

/// Converts the ChannelsFundChannel200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ChannelsFundChannel200Response {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![Some("receipt".to_string()), Some(self.receipt.to_string())];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ChannelsFundChannel200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ChannelsFundChannel200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub receipt: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ChannelsFundChannel200Response".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "receipt" => intermediate_rep
                        .receipt
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ChannelsFundChannel200Response".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ChannelsFundChannel200Response {
            receipt: intermediate_rep
                .receipt
                .into_iter()
                .next()
                .ok_or_else(|| "receipt missing in ChannelsFundChannel200Response".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ChannelsFundChannel200Response> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ChannelsFundChannel200Response>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ChannelsFundChannel200Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ChannelsFundChannel200Response - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ChannelsFundChannel200Response> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ChannelsFundChannel200Response as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ChannelsFundChannel200Response - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ChannelsFundChannelRequest {
    /// Amount of weiHOPR tokens to fund the channel. It will be used to pay for sending messages through channel
    #[serde(rename = "amount")]
    pub amount: String,
}

impl ChannelsFundChannelRequest {
    #[allow(clippy::new_without_default)]
    pub fn new(amount: String) -> ChannelsFundChannelRequest {
        ChannelsFundChannelRequest { amount }
    }
}

/// Converts the ChannelsFundChannelRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ChannelsFundChannelRequest {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![Some("amount".to_string()), Some(self.amount.to_string())];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ChannelsFundChannelRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ChannelsFundChannelRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub amount: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ChannelsFundChannelRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "amount" => intermediate_rep
                        .amount
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ChannelsFundChannelRequest".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ChannelsFundChannelRequest {
            amount: intermediate_rep
                .amount
                .into_iter()
                .next()
                .ok_or_else(|| "amount missing in ChannelsFundChannelRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ChannelsFundChannelRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ChannelsFundChannelRequest>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ChannelsFundChannelRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ChannelsFundChannelRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ChannelsFundChannelRequest> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ChannelsFundChannelRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ChannelsFundChannelRequest - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ChannelsGetChannels200Response {
    /// Incomming channels are the ones that were opened by a different node and this node acts as relay.
    #[serde(rename = "incoming")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incoming: Option<Vec<models::Channel>>,

    /// Outgoing channels are the ones that were opened by this node and is using other node as relay.
    #[serde(rename = "outgoing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outgoing: Option<Vec<models::Channel>>,

    /// All the channels indexed by the node in the current network.
    #[serde(rename = "all")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all: Option<Vec<models::ChannelTopology>>,
}

impl ChannelsGetChannels200Response {
    #[allow(clippy::new_without_default)]
    pub fn new() -> ChannelsGetChannels200Response {
        ChannelsGetChannels200Response {
            incoming: None,
            outgoing: None,
            all: None,
        }
    }
}

/// Converts the ChannelsGetChannels200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ChannelsGetChannels200Response {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            // Skipping incoming in query parameter serialization

            // Skipping outgoing in query parameter serialization

            // Skipping all in query parameter serialization

        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ChannelsGetChannels200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ChannelsGetChannels200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub incoming: Vec<Vec<models::Channel>>,
            pub outgoing: Vec<Vec<models::Channel>>,
            pub all: Vec<Vec<models::ChannelTopology>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ChannelsGetChannels200Response".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    "incoming" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ChannelsGetChannels200Response"
                                .to_string(),
                        )
                    }
                    "outgoing" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ChannelsGetChannels200Response"
                                .to_string(),
                        )
                    }
                    "all" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ChannelsGetChannels200Response"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ChannelsGetChannels200Response".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ChannelsGetChannels200Response {
            incoming: intermediate_rep.incoming.into_iter().next(),
            outgoing: intermediate_rep.outgoing.into_iter().next(),
            all: intermediate_rep.all.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ChannelsGetChannels200Response> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ChannelsGetChannels200Response>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ChannelsGetChannels200Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ChannelsGetChannels200Response - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ChannelsGetChannels200Response> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ChannelsGetChannels200Response as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ChannelsGetChannels200Response - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ChannelsOpenChannel201Response {
    /// The unique identifier of a unidirectional HOPR channel.
    #[serde(rename = "channelId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<serde_json::Value>,

    /// Receipt identifier for an Ethereum transaction.
    #[serde(rename = "transactionReceipt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_receipt: Option<String>,
}

impl ChannelsOpenChannel201Response {
    #[allow(clippy::new_without_default)]
    pub fn new() -> ChannelsOpenChannel201Response {
        ChannelsOpenChannel201Response {
            channel_id: None,
            transaction_receipt: None,
        }
    }
}

/// Converts the ChannelsOpenChannel201Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ChannelsOpenChannel201Response {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            // Skipping channelId in query parameter serialization
            self.transaction_receipt.as_ref().map(|transaction_receipt| {
                vec!["transactionReceipt".to_string(), transaction_receipt.to_string()].join(",")
            }),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ChannelsOpenChannel201Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ChannelsOpenChannel201Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub channel_id: Vec<serde_json::Value>,
            pub transaction_receipt: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ChannelsOpenChannel201Response".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "channelId" => intermediate_rep
                        .channel_id
                        .push(<serde_json::Value as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "transactionReceipt" => intermediate_rep
                        .transaction_receipt
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ChannelsOpenChannel201Response".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ChannelsOpenChannel201Response {
            channel_id: intermediate_rep.channel_id.into_iter().next(),
            transaction_receipt: intermediate_rep.transaction_receipt.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ChannelsOpenChannel201Response> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ChannelsOpenChannel201Response>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ChannelsOpenChannel201Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ChannelsOpenChannel201Response - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ChannelsOpenChannel201Response> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ChannelsOpenChannel201Response as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ChannelsOpenChannel201Response - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ChannelsOpenChannel403Response {
    /// Insufficient balance to open channel. Amount passed in request body exeeds current balance.
    #[serde(rename = "status")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

impl ChannelsOpenChannel403Response {
    #[allow(clippy::new_without_default)]
    pub fn new() -> ChannelsOpenChannel403Response {
        ChannelsOpenChannel403Response { status: None }
    }
}

/// Converts the ChannelsOpenChannel403Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ChannelsOpenChannel403Response {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![self
            .status
            .as_ref()
            .map(|status| vec!["status".to_string(), status.to_string()].join(","))];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ChannelsOpenChannel403Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ChannelsOpenChannel403Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub status: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ChannelsOpenChannel403Response".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "status" => intermediate_rep
                        .status
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ChannelsOpenChannel403Response".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ChannelsOpenChannel403Response {
            status: intermediate_rep.status.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ChannelsOpenChannel403Response> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ChannelsOpenChannel403Response>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ChannelsOpenChannel403Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ChannelsOpenChannel403Response - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ChannelsOpenChannel403Response> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ChannelsOpenChannel403Response as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ChannelsOpenChannel403Response - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ChannelsOpenChannel409Response {
    /// Channel already open. Cannot open more than one channel between two nodes.
    #[serde(rename = "status")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

impl ChannelsOpenChannel409Response {
    #[allow(clippy::new_without_default)]
    pub fn new() -> ChannelsOpenChannel409Response {
        ChannelsOpenChannel409Response { status: None }
    }
}

/// Converts the ChannelsOpenChannel409Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ChannelsOpenChannel409Response {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![self
            .status
            .as_ref()
            .map(|status| vec!["status".to_string(), status.to_string()].join(","))];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ChannelsOpenChannel409Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ChannelsOpenChannel409Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub status: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ChannelsOpenChannel409Response".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "status" => intermediate_rep
                        .status
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ChannelsOpenChannel409Response".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ChannelsOpenChannel409Response {
            status: intermediate_rep.status.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ChannelsOpenChannel409Response> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ChannelsOpenChannel409Response>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ChannelsOpenChannel409Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ChannelsOpenChannel409Response - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ChannelsOpenChannel409Response> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ChannelsOpenChannel409Response as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ChannelsOpenChannel409Response - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ChannelsOpenChannelRequest {
    /// Peer address that we want to transact with using this channel.
    #[serde(rename = "peerAddress")]
    pub peer_address: String,

    /// Amount of HOPR tokens to fund the channel. It will be used to pay for sending messages through channel
    #[serde(rename = "amount")]
    pub amount: String,
}

impl ChannelsOpenChannelRequest {
    #[allow(clippy::new_without_default)]
    pub fn new(peer_address: String, amount: String) -> ChannelsOpenChannelRequest {
        ChannelsOpenChannelRequest { peer_address, amount }
    }
}

/// Converts the ChannelsOpenChannelRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ChannelsOpenChannelRequest {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            Some("peerAddress".to_string()),
            Some(self.peer_address.to_string()),
            Some("amount".to_string()),
            Some(self.amount.to_string()),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ChannelsOpenChannelRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ChannelsOpenChannelRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub peer_address: Vec<String>,
            pub amount: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ChannelsOpenChannelRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "peerAddress" => intermediate_rep
                        .peer_address
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "amount" => intermediate_rep
                        .amount
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ChannelsOpenChannelRequest".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ChannelsOpenChannelRequest {
            peer_address: intermediate_rep
                .peer_address
                .into_iter()
                .next()
                .ok_or_else(|| "peerAddress missing in ChannelsOpenChannelRequest".to_string())?,
            amount: intermediate_rep
                .amount
                .into_iter()
                .next()
                .ok_or_else(|| "amount missing in ChannelsOpenChannelRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ChannelsOpenChannelRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ChannelsOpenChannelRequest>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ChannelsOpenChannelRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ChannelsOpenChannelRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ChannelsOpenChannelRequest> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ChannelsOpenChannelRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ChannelsOpenChannelRequest - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

/// Supported currencies, NATIVE used for the interacting with blockchain or HOPR used to fund channels.
/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum Currency {
    #[serde(rename = "NATIVE")]
    Native,
    #[serde(rename = "HOPR")]
    Hopr,
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Currency::Native => write!(f, "NATIVE"),
            Currency::Hopr => write!(f, "HOPR"),
        }
    }
}

impl std::str::FromStr for Currency {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "NATIVE" => std::result::Result::Ok(Currency::Native),
            "HOPR" => std::result::Result::Ok(Currency::Hopr),
            _ => std::result::Result::Err(format!("Value not valid: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Error {
    #[serde(rename = "status")]
    pub status: String,

    #[serde(rename = "error")]
    pub error: String,
}

impl Error {
    #[allow(clippy::new_without_default)]
    pub fn new(status: String, error: String) -> Error {
        Error { status, error }
    }
}

/// Converts the Error value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Error {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            Some("status".to_string()),
            Some(self.status.to_string()),
            Some("error".to_string()),
            Some(self.error.to_string()),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Error value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Error {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub status: Vec<String>,
            pub error: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing Error".to_string()),
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "status" => intermediate_rep
                        .status
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "error" => intermediate_rep
                        .error
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing Error".to_string()),
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Error {
            status: intermediate_rep
                .status
                .into_iter()
                .next()
                .ok_or_else(|| "status missing in Error".to_string())?,
            error: intermediate_rep
                .error
                .into_iter()
                .next()
                .ok_or_else(|| "error missing in Error".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Error> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Error>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<Error>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Error - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Error> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <Error as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into Error - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

/// HOPR account address, also called a PeerId. Used to send / receive messages, open / close payment channels.
#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct HoprAddress(String);

impl std::convert::From<String> for HoprAddress {
    fn from(x: String) -> Self {
        HoprAddress(x)
    }
}

impl std::string::ToString for HoprAddress {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl std::str::FromStr for HoprAddress {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(HoprAddress(x.to_string()))
    }
}

impl std::convert::From<HoprAddress> for String {
    fn from(x: HoprAddress) -> Self {
        x.0
    }
}

impl std::ops::Deref for HoprAddress {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for HoprAddress {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

/// Amount of HOPR tokens in the smallest unit. Used for funding payment channels.
#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct HoprBalance(String);

impl std::convert::From<String> for HoprBalance {
    fn from(x: String) -> Self {
        HoprBalance(x)
    }
}

impl std::string::ToString for HoprBalance {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl std::str::FromStr for HoprBalance {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(HoprBalance(x.to_string()))
    }
}

impl std::convert::From<HoprBalance> for String {
    fn from(x: HoprBalance) -> Self {
        x.0
    }
}

impl std::ops::Deref for HoprBalance {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for HoprBalance {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct MessageBody(String);

impl std::convert::From<String> for MessageBody {
    fn from(x: String) -> Self {
        MessageBody(x)
    }
}

impl std::string::ToString for MessageBody {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl std::str::FromStr for MessageBody {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(MessageBody(x.to_string()))
    }
}

impl std::convert::From<MessageBody> for String {
    fn from(x: MessageBody) -> Self {
        x.0
    }
}

impl std::ops::Deref for MessageBody {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for MessageBody {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

/// The message tag which can be used to filter messages between apps.
#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct MessageTag(i32);

impl std::convert::From<i32> for MessageTag {
    fn from(x: i32) -> Self {
        MessageTag(x)
    }
}

impl std::convert::From<MessageTag> for i32 {
    fn from(x: MessageTag) -> Self {
        x.0
    }
}

impl std::ops::Deref for MessageTag {
    type Target = i32;
    fn deref(&self) -> &i32 {
        &self.0
    }
}

impl std::ops::DerefMut for MessageTag {
    fn deref_mut(&mut self) -> &mut i32 {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct MessagesGetSize200Response {
    #[serde(rename = "size")]
    #[validate(range(min = 0))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>,
}

impl MessagesGetSize200Response {
    #[allow(clippy::new_without_default)]
    pub fn new() -> MessagesGetSize200Response {
        MessagesGetSize200Response { size: None }
    }
}

/// Converts the MessagesGetSize200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for MessagesGetSize200Response {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![self
            .size
            .as_ref()
            .map(|size| vec!["size".to_string(), size.to_string()].join(","))];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a MessagesGetSize200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for MessagesGetSize200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub size: Vec<u32>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing MessagesGetSize200Response".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "size" => intermediate_rep
                        .size
                        .push(<u32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing MessagesGetSize200Response".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(MessagesGetSize200Response {
            size: intermediate_rep.size.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<MessagesGetSize200Response> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<MessagesGetSize200Response>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<MessagesGetSize200Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for MessagesGetSize200Response - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<MessagesGetSize200Response> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <MessagesGetSize200Response as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into MessagesGetSize200Response - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct MessagesPopAllMessage200Response {
    #[serde(rename = "messages")]
    pub messages: Vec<models::ReceivedMessage>,
}

impl MessagesPopAllMessage200Response {
    #[allow(clippy::new_without_default)]
    pub fn new(messages: Vec<models::ReceivedMessage>) -> MessagesPopAllMessage200Response {
        MessagesPopAllMessage200Response { messages }
    }
}

/// Converts the MessagesPopAllMessage200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for MessagesPopAllMessage200Response {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            // Skipping messages in query parameter serialization

        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a MessagesPopAllMessage200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for MessagesPopAllMessage200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub messages: Vec<Vec<models::ReceivedMessage>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing MessagesPopAllMessage200Response".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    "messages" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in MessagesPopAllMessage200Response"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing MessagesPopAllMessage200Response".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(MessagesPopAllMessage200Response {
            messages: intermediate_rep
                .messages
                .into_iter()
                .next()
                .ok_or_else(|| "messages missing in MessagesPopAllMessage200Response".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<MessagesPopAllMessage200Response> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<MessagesPopAllMessage200Response>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<MessagesPopAllMessage200Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for MessagesPopAllMessage200Response - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<MessagesPopAllMessage200Response> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <MessagesPopAllMessage200Response as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into MessagesPopAllMessage200Response - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct MessagesPopAllMessageRequest {
    /// The message tag which can be used to filter messages between apps.
    #[serde(rename = "tag")]
    #[validate(range(min = 0, max = 65536))]
    pub tag: u32,
}

impl MessagesPopAllMessageRequest {
    #[allow(clippy::new_without_default)]
    pub fn new(tag: u32) -> MessagesPopAllMessageRequest {
        MessagesPopAllMessageRequest { tag }
    }
}

/// Converts the MessagesPopAllMessageRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for MessagesPopAllMessageRequest {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![Some("tag".to_string()), Some(self.tag.to_string())];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a MessagesPopAllMessageRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for MessagesPopAllMessageRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub tag: Vec<u32>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing MessagesPopAllMessageRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "tag" => intermediate_rep
                        .tag
                        .push(<u32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing MessagesPopAllMessageRequest".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(MessagesPopAllMessageRequest {
            tag: intermediate_rep
                .tag
                .into_iter()
                .next()
                .ok_or_else(|| "tag missing in MessagesPopAllMessageRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<MessagesPopAllMessageRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<MessagesPopAllMessageRequest>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<MessagesPopAllMessageRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for MessagesPopAllMessageRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<MessagesPopAllMessageRequest> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <MessagesPopAllMessageRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into MessagesPopAllMessageRequest - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct MessagesSendMessageRequest {
    /// The message tag which can be used to filter messages between apps.
    #[serde(rename = "tag")]
    #[validate(range(min = 0, max = 65536))]
    pub tag: u32,

    #[serde(rename = "body")]
    pub body: String,

    /// The recipient HOPR peer id, to which the message is sent.
    #[serde(rename = "peerId")]
    pub peer_id: String,

    /// The path is ordered list of peer ids through which the message should be sent. If no path is provided, a path which covers the nodes minimum required hops will be determined automatically.
    #[serde(rename = "path")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<Vec<String>>,

    /// Number of required intermediate nodes. This parameter is ignored if path is set.
    #[serde(rename = "hops")]
    #[validate(range(min = 1, max = 3))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hops: Option<u8>,
}

impl MessagesSendMessageRequest {
    #[allow(clippy::new_without_default)]
    pub fn new(tag: u32, body: String, peer_id: String) -> MessagesSendMessageRequest {
        MessagesSendMessageRequest {
            tag,
            body,
            peer_id,
            path: None,
            hops: None,
        }
    }
}

/// Converts the MessagesSendMessageRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for MessagesSendMessageRequest {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            Some("tag".to_string()),
            Some(self.tag.to_string()),
            Some("body".to_string()),
            Some(self.body.to_string()),
            Some("peerId".to_string()),
            Some(self.peer_id.to_string()),
            self.path.as_ref().map(|path| {
                vec![
                    "path".to_string(),
                    path.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","),
                ]
                .join(",")
            }),
            self.hops
                .as_ref()
                .map(|hops| vec!["hops".to_string(), hops.to_string()].join(",")),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a MessagesSendMessageRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for MessagesSendMessageRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub tag: Vec<u32>,
            pub body: Vec<String>,
            pub peer_id: Vec<String>,
            pub path: Vec<Vec<String>>,
            pub hops: Vec<u8>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing MessagesSendMessageRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "tag" => intermediate_rep
                        .tag
                        .push(<u32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "body" => intermediate_rep
                        .body
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "peerId" => intermediate_rep
                        .peer_id
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    "path" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in MessagesSendMessageRequest"
                                .to_string(),
                        )
                    }
                    #[allow(clippy::redundant_clone)]
                    "hops" => intermediate_rep
                        .hops
                        .push(<u8 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing MessagesSendMessageRequest".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(MessagesSendMessageRequest {
            tag: intermediate_rep
                .tag
                .into_iter()
                .next()
                .ok_or_else(|| "tag missing in MessagesSendMessageRequest".to_string())?,
            body: intermediate_rep
                .body
                .into_iter()
                .next()
                .ok_or_else(|| "body missing in MessagesSendMessageRequest".to_string())?,
            peer_id: intermediate_rep
                .peer_id
                .into_iter()
                .next()
                .ok_or_else(|| "peerId missing in MessagesSendMessageRequest".to_string())?,
            path: intermediate_rep.path.into_iter().next(),
            hops: intermediate_rep.hops.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<MessagesSendMessageRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<MessagesSendMessageRequest>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<MessagesSendMessageRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for MessagesSendMessageRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<MessagesSendMessageRequest> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <MessagesSendMessageRequest as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into MessagesSendMessageRequest - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

/// A multi address is a composable and future-proof network address, usually announced by Public HOPR nodes.
#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct MultiAddress(String);

impl std::convert::From<String> for MultiAddress {
    fn from(x: String) -> Self {
        MultiAddress(x)
    }
}

impl std::string::ToString for MultiAddress {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl std::str::FromStr for MultiAddress {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(MultiAddress(x.to_string()))
    }
}

impl std::convert::From<MultiAddress> for String {
    fn from(x: MultiAddress) -> Self {
        x.0
    }
}

impl std::ops::Deref for MultiAddress {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for MultiAddress {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

/// Blockchain-native account address. Can be funded from external wallets (starts with **0x...**). It **can't be used** internally to send / receive messages, open / close payment channels.
#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NativeAddress(String);

impl std::convert::From<String> for NativeAddress {
    fn from(x: String) -> Self {
        NativeAddress(x)
    }
}

impl std::string::ToString for NativeAddress {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl std::str::FromStr for NativeAddress {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(NativeAddress(x.to_string()))
    }
}

impl std::convert::From<NativeAddress> for String {
    fn from(x: NativeAddress) -> Self {
        x.0
    }
}

impl std::ops::Deref for NativeAddress {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for NativeAddress {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

/// Amount of NATIVE (ETH) balance in the smallest unit. Used only for gas fees on the blockchain the current release is running on. For example, when you will open or close the payment channel, it will use gas fees to execute this action.
#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NativeBalance(String);

impl std::convert::From<String> for NativeBalance {
    fn from(x: String) -> Self {
        NativeBalance(x)
    }
}

impl std::string::ToString for NativeBalance {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl std::str::FromStr for NativeBalance {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(NativeBalance(x.to_string()))
    }
}

impl std::convert::From<NativeBalance> for String {
    fn from(x: NativeBalance) -> Self {
        x.0
    }
}

impl std::ops::Deref for NativeBalance {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for NativeBalance {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NodeGetEntryNodes200ResponseValue {
    /// Known Multiaddrs of the node
    #[serde(rename = "multiaddrs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiaddrs: Option<Vec<String>>,

    /// true if peer is allowed to access network, otherwise false
    #[serde(rename = "isEligible")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_eligible: Option<bool>,
}

impl NodeGetEntryNodes200ResponseValue {
    #[allow(clippy::new_without_default)]
    pub fn new() -> NodeGetEntryNodes200ResponseValue {
        NodeGetEntryNodes200ResponseValue {
            multiaddrs: None,
            is_eligible: None,
        }
    }
}

/// Converts the NodeGetEntryNodes200ResponseValue value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for NodeGetEntryNodes200ResponseValue {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            self.multiaddrs.as_ref().map(|multiaddrs| {
                vec![
                    "multiaddrs".to_string(),
                    multiaddrs.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","),
                ]
                .join(",")
            }),
            self.is_eligible
                .as_ref()
                .map(|is_eligible| vec!["isEligible".to_string(), is_eligible.to_string()].join(",")),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a NodeGetEntryNodes200ResponseValue value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NodeGetEntryNodes200ResponseValue {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub multiaddrs: Vec<Vec<String>>,
            pub is_eligible: Vec<bool>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing NodeGetEntryNodes200ResponseValue".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    "multiaddrs" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in NodeGetEntryNodes200ResponseValue"
                                .to_string(),
                        )
                    }
                    #[allow(clippy::redundant_clone)]
                    "isEligible" => intermediate_rep
                        .is_eligible
                        .push(<bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing NodeGetEntryNodes200ResponseValue".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(NodeGetEntryNodes200ResponseValue {
            multiaddrs: intermediate_rep.multiaddrs.into_iter().next(),
            is_eligible: intermediate_rep.is_eligible.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<NodeGetEntryNodes200ResponseValue> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<NodeGetEntryNodes200ResponseValue>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<NodeGetEntryNodes200ResponseValue>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for NodeGetEntryNodes200ResponseValue - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<NodeGetEntryNodes200ResponseValue> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <NodeGetEntryNodes200ResponseValue as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into NodeGetEntryNodes200ResponseValue - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NodeGetInfo200Response {
    /// Name of the network the node is running on.
    #[serde(rename = "network")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,

    #[serde(rename = "announcedAddress")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub announced_address: Option<Vec<String>>,

    #[serde(rename = "listeningAddress")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listening_address: Option<Vec<String>>,

    /// Name of the Hopr network this node connects to.
    #[serde(rename = "chain")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,

    /// Contract address of the Hopr token on the ethereum chain.
    #[serde(rename = "hoprToken")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hopr_token: Option<String>,

    /// Contract address of the HoprChannels smart contract on ethereum chain. This smart contract is used to open payment channels between nodes on blockchain.
    #[serde(rename = "hoprChannels")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hopr_channels: Option<String>,

    /// Contract address of the contract that allows to control the number of nodes in the network
    #[serde(rename = "hoprNetworkRegistryAddress")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hopr_network_registry_address: Option<String>,

    /// Contract address of the contract that register node and safe pairs
    #[serde(rename = "hoprNodeSafeRegistryAddress")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hopr_node_safe_registry_address: Option<String>,

    /// Contract address of the Safe module for managing the current hopr node
    #[serde(rename = "nodeManagementModule")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_management_module: Option<String>,

    /// Contract address of the safe that holds asset for the current node
    #[serde(rename = "nodeSafe")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_safe: Option<String>,

    /// Indicates how good is the connectivity of this node to the HOPR network: either RED, ORANGE, YELLOW or GREEN
    #[serde(rename = "connectivityStatus")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connectivity_status: Option<String>,

    /// Determines whether the staking account associated with this node is eligible for accessing the HOPR network. Always true if network registry is disabled.
    #[serde(rename = "isEligible")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_eligible: Option<bool>,

    /// Time (in minutes) that this node needs in order to clean up before closing the channel. When requesting to close the channel each node needs some time to make sure that channel can be closed securely and cleanly. After this channelClosurePeriod passes the second request for closing channel will close the channel.
    #[serde(rename = "channelClosurePeriod")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_closure_period: Option<f64>,
}

impl NodeGetInfo200Response {
    #[allow(clippy::new_without_default)]
    pub fn new() -> NodeGetInfo200Response {
        NodeGetInfo200Response {
            network: None,
            announced_address: None,
            listening_address: None,
            chain: None,
            hopr_token: None,
            hopr_channels: None,
            hopr_network_registry_address: None,
            hopr_node_safe_registry_address: None,
            node_management_module: None,
            node_safe: None,
            connectivity_status: None,
            is_eligible: None,
            channel_closure_period: None,
        }
    }
}

/// Converts the NodeGetInfo200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for NodeGetInfo200Response {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            self.network
                .as_ref()
                .map(|network| vec!["network".to_string(), network.to_string()].join(",")),
            self.announced_address.as_ref().map(|announced_address| {
                vec![
                    "announcedAddress".to_string(),
                    announced_address
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                ]
                .join(",")
            }),
            self.listening_address.as_ref().map(|listening_address| {
                vec![
                    "listeningAddress".to_string(),
                    listening_address
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                ]
                .join(",")
            }),
            self.chain
                .as_ref()
                .map(|chain| vec!["chain".to_string(), chain.to_string()].join(",")),
            self.hopr_token
                .as_ref()
                .map(|hopr_token| vec!["hoprToken".to_string(), hopr_token.to_string()].join(",")),
            self.hopr_channels
                .as_ref()
                .map(|hopr_channels| vec!["hoprChannels".to_string(), hopr_channels.to_string()].join(",")),
            self.hopr_network_registry_address
                .as_ref()
                .map(|hopr_network_registry_address| {
                    vec![
                        "hoprNetworkRegistryAddress".to_string(),
                        hopr_network_registry_address.to_string(),
                    ]
                    .join(",")
                }),
            self.hopr_node_safe_registry_address
                .as_ref()
                .map(|hopr_node_safe_registry_address| {
                    vec![
                        "hoprNodeSafeRegistryAddress".to_string(),
                        hopr_node_safe_registry_address.to_string(),
                    ]
                    .join(",")
                }),
            self.node_management_module.as_ref().map(|node_management_module| {
                vec!["nodeManagementModule".to_string(), node_management_module.to_string()].join(",")
            }),
            self.node_safe
                .as_ref()
                .map(|node_safe| vec!["nodeSafe".to_string(), node_safe.to_string()].join(",")),
            self.connectivity_status.as_ref().map(|connectivity_status| {
                vec!["connectivityStatus".to_string(), connectivity_status.to_string()].join(",")
            }),
            self.is_eligible
                .as_ref()
                .map(|is_eligible| vec!["isEligible".to_string(), is_eligible.to_string()].join(",")),
            self.channel_closure_period.as_ref().map(|channel_closure_period| {
                vec!["channelClosurePeriod".to_string(), channel_closure_period.to_string()].join(",")
            }),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a NodeGetInfo200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NodeGetInfo200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub network: Vec<String>,
            pub announced_address: Vec<Vec<String>>,
            pub listening_address: Vec<Vec<String>>,
            pub chain: Vec<String>,
            pub hopr_token: Vec<String>,
            pub hopr_channels: Vec<String>,
            pub hopr_network_registry_address: Vec<String>,
            pub hopr_node_safe_registry_address: Vec<String>,
            pub node_management_module: Vec<String>,
            pub node_safe: Vec<String>,
            pub connectivity_status: Vec<String>,
            pub is_eligible: Vec<bool>,
            pub channel_closure_period: Vec<f64>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err("Missing value while parsing NodeGetInfo200Response".to_string())
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "network" => intermediate_rep
                        .network
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    "announcedAddress" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in NodeGetInfo200Response".to_string(),
                        )
                    }
                    "listeningAddress" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in NodeGetInfo200Response".to_string(),
                        )
                    }
                    #[allow(clippy::redundant_clone)]
                    "chain" => intermediate_rep
                        .chain
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "hoprToken" => intermediate_rep
                        .hopr_token
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "hoprChannels" => intermediate_rep
                        .hopr_channels
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "hoprNetworkRegistryAddress" => intermediate_rep
                        .hopr_network_registry_address
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "hoprNodeSafeRegistryAddress" => intermediate_rep
                        .hopr_node_safe_registry_address
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "nodeManagementModule" => intermediate_rep
                        .node_management_module
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "nodeSafe" => intermediate_rep
                        .node_safe
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "connectivityStatus" => intermediate_rep
                        .connectivity_status
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "isEligible" => intermediate_rep
                        .is_eligible
                        .push(<bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "channelClosurePeriod" => intermediate_rep
                        .channel_closure_period
                        .push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing NodeGetInfo200Response".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(NodeGetInfo200Response {
            network: intermediate_rep.network.into_iter().next(),
            announced_address: intermediate_rep.announced_address.into_iter().next(),
            listening_address: intermediate_rep.listening_address.into_iter().next(),
            chain: intermediate_rep.chain.into_iter().next(),
            hopr_token: intermediate_rep.hopr_token.into_iter().next(),
            hopr_channels: intermediate_rep.hopr_channels.into_iter().next(),
            hopr_network_registry_address: intermediate_rep.hopr_network_registry_address.into_iter().next(),
            hopr_node_safe_registry_address: intermediate_rep.hopr_node_safe_registry_address.into_iter().next(),
            node_management_module: intermediate_rep.node_management_module.into_iter().next(),
            node_safe: intermediate_rep.node_safe.into_iter().next(),
            connectivity_status: intermediate_rep.connectivity_status.into_iter().next(),
            is_eligible: intermediate_rep.is_eligible.into_iter().next(),
            channel_closure_period: intermediate_rep.channel_closure_period.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<NodeGetInfo200Response> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<NodeGetInfo200Response>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<NodeGetInfo200Response>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for NodeGetInfo200Response - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<NodeGetInfo200Response> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <NodeGetInfo200Response as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into NodeGetInfo200Response - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NodeGetPeers200Response {
    #[serde(rename = "connected")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connected: Option<Vec<models::NodeGetPeers200ResponseConnectedInner>>,

    #[serde(rename = "announced")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub announced: Option<Vec<models::NodeGetPeers200ResponseConnectedInner>>,
}

impl NodeGetPeers200Response {
    #[allow(clippy::new_without_default)]
    pub fn new() -> NodeGetPeers200Response {
        NodeGetPeers200Response {
            connected: None,
            announced: None,
        }
    }
}

/// Converts the NodeGetPeers200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for NodeGetPeers200Response {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            // Skipping connected in query parameter serialization

            // Skipping announced in query parameter serialization

        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a NodeGetPeers200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NodeGetPeers200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub connected: Vec<Vec<models::NodeGetPeers200ResponseConnectedInner>>,
            pub announced: Vec<Vec<models::NodeGetPeers200ResponseConnectedInner>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err("Missing value while parsing NodeGetPeers200Response".to_string())
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    "connected" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in NodeGetPeers200Response".to_string(),
                        )
                    }
                    "announced" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in NodeGetPeers200Response".to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing NodeGetPeers200Response".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(NodeGetPeers200Response {
            connected: intermediate_rep.connected.into_iter().next(),
            announced: intermediate_rep.announced.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<NodeGetPeers200Response> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<NodeGetPeers200Response>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<NodeGetPeers200Response>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for NodeGetPeers200Response - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<NodeGetPeers200Response> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <NodeGetPeers200Response as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into NodeGetPeers200Response - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NodeGetPeers200ResponseConnectedInner {
    /// HOPR account address, also called a PeerId. Used to send / receive messages, open / close payment channels.
    #[serde(rename = "peerId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_id: Option<String>,

    /// Blockchain-native account address. Can be funded from external wallets (starts with **0x...**). It **can't be used** internally to send / receive messages, open / close payment channels.
    #[serde(rename = "peerAddress")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_address: Option<String>,

    /// A multi address is a composable and future-proof network address, usually announced by Public HOPR nodes.
    #[serde(rename = "multiAddr")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_addr: Option<String>,

    #[serde(rename = "heartbeats")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeats: Option<models::NodeGetPeers200ResponseConnectedInnerHeartbeats>,

    /// Timestamp on when the node was last seen (in milliseconds)
    #[serde(rename = "lastSeen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen: Option<f64>,

    /// Latency recorded the last time a node was measured when seen (in milliseconds)
    #[serde(rename = "lastSeenLatency")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen_latency: Option<f64>,

    /// A float between 0 (completely unreliable) and 1 (completely reliable) estimating the quality of service of a peer's network connection
    #[serde(rename = "quality")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<f64>,

    #[serde(rename = "backoff")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backoff: Option<f64>,

    /// True if the node is new (no heartbeats sent yet).
    #[serde(rename = "isNew")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_new: Option<bool>,

    /// HOPR protocol version as determined from the successful ping in the Major.Minor.Patch format or \"unknown\"
    #[serde(rename = "reportedVersion")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reported_version: Option<String>,
}

impl NodeGetPeers200ResponseConnectedInner {
    #[allow(clippy::new_without_default)]
    pub fn new() -> NodeGetPeers200ResponseConnectedInner {
        NodeGetPeers200ResponseConnectedInner {
            peer_id: None,
            peer_address: None,
            multi_addr: None,
            heartbeats: None,
            last_seen: None,
            last_seen_latency: None,
            quality: None,
            backoff: None,
            is_new: None,
            reported_version: None,
        }
    }
}

/// Converts the NodeGetPeers200ResponseConnectedInner value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for NodeGetPeers200ResponseConnectedInner {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            self.peer_id
                .as_ref()
                .map(|peer_id| vec!["peerId".to_string(), peer_id.to_string()].join(",")),
            self.peer_address
                .as_ref()
                .map(|peer_address| vec!["peerAddress".to_string(), peer_address.to_string()].join(",")),
            self.multi_addr
                .as_ref()
                .map(|multi_addr| vec!["multiAddr".to_string(), multi_addr.to_string()].join(",")),
            // Skipping heartbeats in query parameter serialization
            self.last_seen
                .as_ref()
                .map(|last_seen| vec!["lastSeen".to_string(), last_seen.to_string()].join(",")),
            self.last_seen_latency
                .as_ref()
                .map(|last_seen_latency| vec!["lastSeenLatency".to_string(), last_seen_latency.to_string()].join(",")),
            self.quality
                .as_ref()
                .map(|quality| vec!["quality".to_string(), quality.to_string()].join(",")),
            self.backoff
                .as_ref()
                .map(|backoff| vec!["backoff".to_string(), backoff.to_string()].join(",")),
            self.is_new
                .as_ref()
                .map(|is_new| vec!["isNew".to_string(), is_new.to_string()].join(",")),
            self.reported_version
                .as_ref()
                .map(|reported_version| vec!["reportedVersion".to_string(), reported_version.to_string()].join(",")),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a NodeGetPeers200ResponseConnectedInner value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NodeGetPeers200ResponseConnectedInner {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub peer_id: Vec<String>,
            pub peer_address: Vec<String>,
            pub multi_addr: Vec<String>,
            pub heartbeats: Vec<models::NodeGetPeers200ResponseConnectedInnerHeartbeats>,
            pub last_seen: Vec<f64>,
            pub last_seen_latency: Vec<f64>,
            pub quality: Vec<f64>,
            pub backoff: Vec<f64>,
            pub is_new: Vec<bool>,
            pub reported_version: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing NodeGetPeers200ResponseConnectedInner".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "peerId" => intermediate_rep
                        .peer_id
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "peerAddress" => intermediate_rep
                        .peer_address
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "multiAddr" => intermediate_rep
                        .multi_addr
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "heartbeats" => intermediate_rep.heartbeats.push(
                        <models::NodeGetPeers200ResponseConnectedInnerHeartbeats as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    #[allow(clippy::redundant_clone)]
                    "lastSeen" => intermediate_rep
                        .last_seen
                        .push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "lastSeenLatency" => intermediate_rep
                        .last_seen_latency
                        .push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "quality" => intermediate_rep
                        .quality
                        .push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "backoff" => intermediate_rep
                        .backoff
                        .push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "isNew" => intermediate_rep
                        .is_new
                        .push(<bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "reportedVersion" => intermediate_rep
                        .reported_version
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing NodeGetPeers200ResponseConnectedInner".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(NodeGetPeers200ResponseConnectedInner {
            peer_id: intermediate_rep.peer_id.into_iter().next(),
            peer_address: intermediate_rep.peer_address.into_iter().next(),
            multi_addr: intermediate_rep.multi_addr.into_iter().next(),
            heartbeats: intermediate_rep.heartbeats.into_iter().next(),
            last_seen: intermediate_rep.last_seen.into_iter().next(),
            last_seen_latency: intermediate_rep.last_seen_latency.into_iter().next(),
            quality: intermediate_rep.quality.into_iter().next(),
            backoff: intermediate_rep.backoff.into_iter().next(),
            is_new: intermediate_rep.is_new.into_iter().next(),
            reported_version: intermediate_rep.reported_version.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<NodeGetPeers200ResponseConnectedInner> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<NodeGetPeers200ResponseConnectedInner>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<NodeGetPeers200ResponseConnectedInner>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for NodeGetPeers200ResponseConnectedInner - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<NodeGetPeers200ResponseConnectedInner>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <NodeGetPeers200ResponseConnectedInner as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into NodeGetPeers200ResponseConnectedInner - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NodeGetPeers200ResponseConnectedInnerHeartbeats {
    /// Heartbeats sent to the node
    #[serde(rename = "sent")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sent: Option<f64>,

    /// Successful heartbeats sent to the node
    #[serde(rename = "success")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success: Option<f64>,
}

impl NodeGetPeers200ResponseConnectedInnerHeartbeats {
    #[allow(clippy::new_without_default)]
    pub fn new() -> NodeGetPeers200ResponseConnectedInnerHeartbeats {
        NodeGetPeers200ResponseConnectedInnerHeartbeats {
            sent: None,
            success: None,
        }
    }
}

/// Converts the NodeGetPeers200ResponseConnectedInnerHeartbeats value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for NodeGetPeers200ResponseConnectedInnerHeartbeats {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            self.sent
                .as_ref()
                .map(|sent| vec!["sent".to_string(), sent.to_string()].join(",")),
            self.success
                .as_ref()
                .map(|success| vec!["success".to_string(), success.to_string()].join(",")),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a NodeGetPeers200ResponseConnectedInnerHeartbeats value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NodeGetPeers200ResponseConnectedInnerHeartbeats {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub sent: Vec<f64>,
            pub success: Vec<f64>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing NodeGetPeers200ResponseConnectedInnerHeartbeats".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "sent" => intermediate_rep
                        .sent
                        .push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "success" => intermediate_rep
                        .success
                        .push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing NodeGetPeers200ResponseConnectedInnerHeartbeats".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(NodeGetPeers200ResponseConnectedInnerHeartbeats {
            sent: intermediate_rep.sent.into_iter().next(),
            success: intermediate_rep.success.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<NodeGetPeers200ResponseConnectedInnerHeartbeats> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<NodeGetPeers200ResponseConnectedInnerHeartbeats>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<NodeGetPeers200ResponseConnectedInnerHeartbeats>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for NodeGetPeers200ResponseConnectedInnerHeartbeats - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<NodeGetPeers200ResponseConnectedInnerHeartbeats>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <NodeGetPeers200ResponseConnectedInnerHeartbeats as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into NodeGetPeers200ResponseConnectedInnerHeartbeats - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct PeerInfoGetPeerInfo200Response {
    #[serde(rename = "announced")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub announced: Option<Vec<models::MultiAddress>>,

    #[serde(rename = "observed")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed: Option<Vec<models::MultiAddress>>,
}

impl PeerInfoGetPeerInfo200Response {
    #[allow(clippy::new_without_default)]
    pub fn new() -> PeerInfoGetPeerInfo200Response {
        PeerInfoGetPeerInfo200Response {
            announced: None,
            observed: None,
        }
    }
}

/// Converts the PeerInfoGetPeerInfo200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for PeerInfoGetPeerInfo200Response {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            self.announced.as_ref().map(|announced| {
                vec![
                    "announced".to_string(),
                    announced.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","),
                ]
                .join(",")
            }),
            self.observed.as_ref().map(|observed| {
                vec![
                    "observed".to_string(),
                    observed.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","),
                ]
                .join(",")
            }),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a PeerInfoGetPeerInfo200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for PeerInfoGetPeerInfo200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub announced: Vec<Vec<models::MultiAddress>>,
            pub observed: Vec<Vec<models::MultiAddress>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing PeerInfoGetPeerInfo200Response".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    "announced" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in PeerInfoGetPeerInfo200Response"
                                .to_string(),
                        )
                    }
                    "observed" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in PeerInfoGetPeerInfo200Response"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing PeerInfoGetPeerInfo200Response".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(PeerInfoGetPeerInfo200Response {
            announced: intermediate_rep.announced.into_iter().next(),
            observed: intermediate_rep.observed.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<PeerInfoGetPeerInfo200Response> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<PeerInfoGetPeerInfo200Response>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<PeerInfoGetPeerInfo200Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for PeerInfoGetPeerInfo200Response - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<PeerInfoGetPeerInfo200Response> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <PeerInfoGetPeerInfo200Response as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into PeerInfoGetPeerInfo200Response - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct PeersPingPeer200Response {
    /// Number of milliseconds it took to get the response from the pinged node.
    #[serde(rename = "latency")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency: Option<f64>,

    /// HOPR protocol version as determined from the successful ping in the Major.Minor.Patch format or \"unknown\"
    #[serde(rename = "reportedVersion")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reported_version: Option<String>,
}

impl PeersPingPeer200Response {
    #[allow(clippy::new_without_default)]
    pub fn new() -> PeersPingPeer200Response {
        PeersPingPeer200Response {
            latency: None,
            reported_version: None,
        }
    }
}

/// Converts the PeersPingPeer200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for PeersPingPeer200Response {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            self.latency
                .as_ref()
                .map(|latency| vec!["latency".to_string(), latency.to_string()].join(",")),
            self.reported_version
                .as_ref()
                .map(|reported_version| vec!["reportedVersion".to_string(), reported_version.to_string()].join(",")),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a PeersPingPeer200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for PeersPingPeer200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub latency: Vec<f64>,
            pub reported_version: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err("Missing value while parsing PeersPingPeer200Response".to_string())
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "latency" => intermediate_rep
                        .latency
                        .push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "reportedVersion" => intermediate_rep
                        .reported_version
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing PeersPingPeer200Response".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(PeersPingPeer200Response {
            latency: intermediate_rep.latency.into_iter().next(),
            reported_version: intermediate_rep.reported_version.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<PeersPingPeer200Response> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<PeersPingPeer200Response>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<PeersPingPeer200Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for PeersPingPeer200Response - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<PeersPingPeer200Response> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <PeersPingPeer200Response as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into PeersPingPeer200Response - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ReceivedMessage {
    /// The message tag which can be used to filter messages between apps.
    #[serde(rename = "tag")]
    #[validate(range(min = 0, max = 65536))]
    pub tag: u32,

    #[serde(rename = "body")]
    pub body: String,

    /// Timestamp when the message was received in seconds since epoch.
    #[serde(rename = "receivedAt")]
    pub received_at: i32,
}

impl ReceivedMessage {
    #[allow(clippy::new_without_default)]
    pub fn new(tag: u32, body: String, received_at: i32) -> ReceivedMessage {
        ReceivedMessage { tag, body, received_at }
    }
}

/// Converts the ReceivedMessage value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ReceivedMessage {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            Some("tag".to_string()),
            Some(self.tag.to_string()),
            Some("body".to_string()),
            Some(self.body.to_string()),
            Some("receivedAt".to_string()),
            Some(self.received_at.to_string()),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ReceivedMessage value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ReceivedMessage {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub tag: Vec<u32>,
            pub body: Vec<String>,
            pub received_at: Vec<i32>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing ReceivedMessage".to_string()),
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "tag" => intermediate_rep
                        .tag
                        .push(<u32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "body" => intermediate_rep
                        .body
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "receivedAt" => intermediate_rep
                        .received_at
                        .push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing ReceivedMessage".to_string()),
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ReceivedMessage {
            tag: intermediate_rep
                .tag
                .into_iter()
                .next()
                .ok_or_else(|| "tag missing in ReceivedMessage".to_string())?,
            body: intermediate_rep
                .body
                .into_iter()
                .next()
                .ok_or_else(|| "body missing in ReceivedMessage".to_string())?,
            received_at: intermediate_rep
                .received_at
                .into_iter()
                .next()
                .ok_or_else(|| "receivedAt missing in ReceivedMessage".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ReceivedMessage> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ReceivedMessage>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<ReceivedMessage>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ReceivedMessage - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ReceivedMessage> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <ReceivedMessage as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into ReceivedMessage - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct RequestStatus {
    /// Status declaring success/failure of the request.
    #[serde(rename = "status")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

impl RequestStatus {
    #[allow(clippy::new_without_default)]
    pub fn new() -> RequestStatus {
        RequestStatus { status: None }
    }
}

/// Converts the RequestStatus value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for RequestStatus {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![self
            .status
            .as_ref()
            .map(|status| vec!["status".to_string(), status.to_string()].join(","))];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a RequestStatus value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for RequestStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub status: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing RequestStatus".to_string()),
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "status" => intermediate_rep
                        .status
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing RequestStatus".to_string()),
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(RequestStatus {
            status: intermediate_rep.status.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<RequestStatus> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<RequestStatus>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<RequestStatus>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for RequestStatus - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<RequestStatus> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <RequestStatus as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into RequestStatus - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

/// Various settings that affects how this node is interacting with the network.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Settings {
    /// Prepends your address to all messages so that receiver of the message can know that you sent that message.
    #[serde(rename = "includeRecipient")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_recipient: Option<bool>,

    /// By default, hoprd runs in **passive** mode, this means that your node will not attempt to open or close any channels automatically. When you set your strategy to **promiscuous** mode, your node will attempt to open channels to a _randomly_ selected group of nodes which you have a healthy connection to. At the same time, your node will also attempt to close channels that are running low on balance or are unhealthy.
    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "strategy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
}

impl Settings {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Settings {
        Settings {
            include_recipient: None,
            strategy: None,
        }
    }
}

/// Converts the Settings value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Settings {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            self.include_recipient
                .as_ref()
                .map(|include_recipient| vec!["includeRecipient".to_string(), include_recipient.to_string()].join(",")),
            self.strategy
                .as_ref()
                .map(|strategy| vec!["strategy".to_string(), strategy.to_string()].join(",")),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Settings value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Settings {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub include_recipient: Vec<bool>,
            pub strategy: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing Settings".to_string()),
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "includeRecipient" => intermediate_rep
                        .include_recipient
                        .push(<bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "strategy" => intermediate_rep
                        .strategy
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing Settings".to_string()),
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Settings {
            include_recipient: intermediate_rep.include_recipient.into_iter().next(),
            strategy: intermediate_rep.strategy.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Settings> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Settings>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<Settings>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Settings - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Settings> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <Settings as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into Settings - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct SettingsSetSettingRequest {
    #[serde(rename = "settingValue")]
    pub setting_value: serde_json::Value,
}

impl SettingsSetSettingRequest {
    #[allow(clippy::new_without_default)]
    pub fn new(setting_value: serde_json::Value) -> SettingsSetSettingRequest {
        SettingsSetSettingRequest { setting_value }
    }
}

/// Converts the SettingsSetSettingRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for SettingsSetSettingRequest {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            // Skipping settingValue in query parameter serialization

        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a SettingsSetSettingRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for SettingsSetSettingRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub setting_value: Vec<serde_json::Value>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing SettingsSetSettingRequest".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "settingValue" => intermediate_rep
                        .setting_value
                        .push(<serde_json::Value as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing SettingsSetSettingRequest".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(SettingsSetSettingRequest {
            setting_value: intermediate_rep
                .setting_value
                .into_iter()
                .next()
                .ok_or_else(|| "settingValue missing in SettingsSetSettingRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<SettingsSetSettingRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<SettingsSetSettingRequest>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<SettingsSetSettingRequest>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for SettingsSetSettingRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<SettingsSetSettingRequest> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <SettingsSetSettingRequest as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into SettingsSetSettingRequest - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

/// Signature from requested message.
#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Signature(String);

impl std::convert::From<String> for Signature {
    fn from(x: String) -> Self {
        Signature(x)
    }
}

impl std::string::ToString for Signature {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl std::str::FromStr for Signature {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(Signature(x.to_string()))
    }
}

impl std::convert::From<Signature> for String {
    fn from(x: Signature) -> Self {
        x.0
    }
}

impl std::ops::Deref for Signature {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for Signature {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Ticket {
    /// The unique identifier of a unidirectional HOPR channel.
    #[serde(rename = "channelId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<serde_json::Value>,

    /// The ticket's value in HOPR. Only relevant if ticket is a win.
    #[serde(rename = "amount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,

    /// Each ticket is labeled by an ongoing serial number named ticket index i and its current value is stored in the smart contract.
    #[serde(rename = "index")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<String>,

    /// Offset by which the on-chain stored ticket index gets increased when redeeming the ticket. Used to aggregate tickets.
    #[serde(rename = "indexOffset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_offset: Option<String>,

    /// Payment channels might run through multiple open and close sequences, this epoch tracks the sequence.
    #[serde(rename = "channelEpoch")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_epoch: Option<String>,

    /// The ticket's winning probability, going from 0.0 to 1.0 where 0.0 ~= 0% winning probability and 1.0 equals 100% winning probability.
    #[serde(rename = "winProb")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub win_prob: Option<String>,

    /// Signature from requested message.
    #[serde(rename = "signature")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl Ticket {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Ticket {
        Ticket {
            channel_id: None,
            amount: None,
            index: None,
            index_offset: None,
            channel_epoch: None,
            win_prob: None,
            signature: None,
        }
    }
}

/// Converts the Ticket value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Ticket {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            // Skipping channelId in query parameter serialization
            self.amount
                .as_ref()
                .map(|amount| vec!["amount".to_string(), amount.to_string()].join(",")),
            self.index
                .as_ref()
                .map(|index| vec!["index".to_string(), index.to_string()].join(",")),
            self.index_offset
                .as_ref()
                .map(|index_offset| vec!["indexOffset".to_string(), index_offset.to_string()].join(",")),
            self.channel_epoch
                .as_ref()
                .map(|channel_epoch| vec!["channelEpoch".to_string(), channel_epoch.to_string()].join(",")),
            self.win_prob
                .as_ref()
                .map(|win_prob| vec!["winProb".to_string(), win_prob.to_string()].join(",")),
            self.signature
                .as_ref()
                .map(|signature| vec!["signature".to_string(), signature.to_string()].join(",")),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Ticket value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Ticket {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub channel_id: Vec<serde_json::Value>,
            pub amount: Vec<String>,
            pub index: Vec<String>,
            pub index_offset: Vec<String>,
            pub channel_epoch: Vec<String>,
            pub win_prob: Vec<String>,
            pub signature: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing Ticket".to_string()),
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "channelId" => intermediate_rep
                        .channel_id
                        .push(<serde_json::Value as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "amount" => intermediate_rep
                        .amount
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "index" => intermediate_rep
                        .index
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "indexOffset" => intermediate_rep
                        .index_offset
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "channelEpoch" => intermediate_rep
                        .channel_epoch
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "winProb" => intermediate_rep
                        .win_prob
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "signature" => intermediate_rep
                        .signature
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing Ticket".to_string()),
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Ticket {
            channel_id: intermediate_rep.channel_id.into_iter().next(),
            amount: intermediate_rep.amount.into_iter().next(),
            index: intermediate_rep.index.into_iter().next(),
            index_offset: intermediate_rep.index_offset.into_iter().next(),
            channel_epoch: intermediate_rep.channel_epoch.into_iter().next(),
            win_prob: intermediate_rep.win_prob.into_iter().next(),
            signature: intermediate_rep.signature.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Ticket> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Ticket>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<Ticket>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Ticket - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Ticket> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <Ticket as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into Ticket - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TicketsGetStatistics200Response {
    /// Number of tickets that wait to be redeemed as for Hopr tokens.
    #[serde(rename = "unredeemed")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unredeemed: Option<f64>,

    /// Total value of all unredeemed tickets in Hopr tokens.
    #[serde(rename = "unredeemedValue")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unredeemed_value: Option<String>,

    /// Number of tickets already redeemed on this node.
    #[serde(rename = "redeemed")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redeemed: Option<f64>,

    /// Total value of all redeemed tickets in Hopr tokens.
    #[serde(rename = "redeemedValue")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redeemed_value: Option<String>,

    /// Number of tickets that didn't win any Hopr tokens. To better understand how tickets work read about probabilistic payments (https://docs.hoprnet.org/core/probabilistic-payments)
    #[serde(rename = "losingTickets")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub losing_tickets: Option<f64>,

    /// Proportion of number of winning tickets vs loosing tickets, 1 means 100% of tickets won and 0 means that all tickets were losing ones.
    #[serde(rename = "winProportion")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub win_proportion: Option<f64>,

    /// Number of tickets that were not redeemed in time before channel was closed. Those cannot be redeemed anymore.
    #[serde(rename = "neglected")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub neglected: Option<f64>,

    /// Total value of all neglected tickets in Hopr tokens.
    #[serde(rename = "neglectedValue")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub neglected_value: Option<String>,

    /// Number of tickets that were rejected by the network by not passing validation. In other words tickets that look suspicious and are not eligible for redeeming.
    #[serde(rename = "rejected")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected: Option<f64>,

    /// Total value of rejected tickets in Hopr tokens
    #[serde(rename = "rejectedValue")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_value: Option<String>,
}

impl TicketsGetStatistics200Response {
    #[allow(clippy::new_without_default)]
    pub fn new() -> TicketsGetStatistics200Response {
        TicketsGetStatistics200Response {
            unredeemed: None,
            unredeemed_value: None,
            redeemed: None,
            redeemed_value: None,
            losing_tickets: None,
            win_proportion: None,
            neglected: None,
            neglected_value: None,
            rejected: None,
            rejected_value: None,
        }
    }
}

/// Converts the TicketsGetStatistics200Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TicketsGetStatistics200Response {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            self.unredeemed
                .as_ref()
                .map(|unredeemed| vec!["unredeemed".to_string(), unredeemed.to_string()].join(",")),
            self.unredeemed_value
                .as_ref()
                .map(|unredeemed_value| vec!["unredeemedValue".to_string(), unredeemed_value.to_string()].join(",")),
            self.redeemed
                .as_ref()
                .map(|redeemed| vec!["redeemed".to_string(), redeemed.to_string()].join(",")),
            self.redeemed_value
                .as_ref()
                .map(|redeemed_value| vec!["redeemedValue".to_string(), redeemed_value.to_string()].join(",")),
            self.losing_tickets
                .as_ref()
                .map(|losing_tickets| vec!["losingTickets".to_string(), losing_tickets.to_string()].join(",")),
            self.win_proportion
                .as_ref()
                .map(|win_proportion| vec!["winProportion".to_string(), win_proportion.to_string()].join(",")),
            self.neglected
                .as_ref()
                .map(|neglected| vec!["neglected".to_string(), neglected.to_string()].join(",")),
            self.neglected_value
                .as_ref()
                .map(|neglected_value| vec!["neglectedValue".to_string(), neglected_value.to_string()].join(",")),
            self.rejected
                .as_ref()
                .map(|rejected| vec!["rejected".to_string(), rejected.to_string()].join(",")),
            self.rejected_value
                .as_ref()
                .map(|rejected_value| vec!["rejectedValue".to_string(), rejected_value.to_string()].join(",")),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TicketsGetStatistics200Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TicketsGetStatistics200Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub unredeemed: Vec<f64>,
            pub unredeemed_value: Vec<String>,
            pub redeemed: Vec<f64>,
            pub redeemed_value: Vec<String>,
            pub losing_tickets: Vec<f64>,
            pub win_proportion: Vec<f64>,
            pub neglected: Vec<f64>,
            pub neglected_value: Vec<String>,
            pub rejected: Vec<f64>,
            pub rejected_value: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing TicketsGetStatistics200Response".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "unredeemed" => intermediate_rep
                        .unredeemed
                        .push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "unredeemedValue" => intermediate_rep
                        .unredeemed_value
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "redeemed" => intermediate_rep
                        .redeemed
                        .push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "redeemedValue" => intermediate_rep
                        .redeemed_value
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "losingTickets" => intermediate_rep
                        .losing_tickets
                        .push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "winProportion" => intermediate_rep
                        .win_proportion
                        .push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "neglected" => intermediate_rep
                        .neglected
                        .push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "neglectedValue" => intermediate_rep
                        .neglected_value
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "rejected" => intermediate_rep
                        .rejected
                        .push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "rejectedValue" => intermediate_rep
                        .rejected_value
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing TicketsGetStatistics200Response".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TicketsGetStatistics200Response {
            unredeemed: intermediate_rep.unredeemed.into_iter().next(),
            unredeemed_value: intermediate_rep.unredeemed_value.into_iter().next(),
            redeemed: intermediate_rep.redeemed.into_iter().next(),
            redeemed_value: intermediate_rep.redeemed_value.into_iter().next(),
            losing_tickets: intermediate_rep.losing_tickets.into_iter().next(),
            win_proportion: intermediate_rep.win_proportion.into_iter().next(),
            neglected: intermediate_rep.neglected.into_iter().next(),
            neglected_value: intermediate_rep.neglected_value.into_iter().next(),
            rejected: intermediate_rep.rejected.into_iter().next(),
            rejected_value: intermediate_rep.rejected_value.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TicketsGetStatistics200Response> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<TicketsGetStatistics200Response>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<TicketsGetStatistics200Response>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for TicketsGetStatistics200Response - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TicketsGetStatistics200Response> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <TicketsGetStatistics200Response as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into TicketsGetStatistics200Response - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Token {
    /// Unique ID of the token
    #[serde(rename = "id")]
    pub id: String,

    /// Some description for the token
    #[serde(rename = "description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Seconds since epoch until the token is valid
    #[serde(rename = "valid_until")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<i32>,

    /// Array of capabilities associated with the token
    #[serde(rename = "capabilities")]
    // #[validate()]
    pub capabilities: Vec<models::TokenCapability>,
}

impl Token {
    #[allow(clippy::new_without_default)]
    pub fn new(id: String, capabilities: Vec<models::TokenCapability>) -> Token {
        Token {
            id,
            description: None,
            valid_until: None,
            capabilities,
        }
    }
}

/// Converts the Token value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Token {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            Some("id".to_string()),
            Some(self.id.to_string()),
            self.description
                .as_ref()
                .map(|description| vec!["description".to_string(), description.to_string()].join(",")),
            self.valid_until
                .as_ref()
                .map(|valid_until| vec!["valid_until".to_string(), valid_until.to_string()].join(",")),
            // Skipping capabilities in query parameter serialization
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Token value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Token {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub id: Vec<String>,
            pub description: Vec<String>,
            pub valid_until: Vec<i32>,
            pub capabilities: Vec<Vec<models::TokenCapability>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing Token".to_string()),
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "id" => intermediate_rep
                        .id
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "description" => intermediate_rep
                        .description
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "valid_until" => intermediate_rep
                        .valid_until
                        .push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    "capabilities" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in Token".to_string(),
                        )
                    }
                    _ => return std::result::Result::Err("Unexpected key while parsing Token".to_string()),
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Token {
            id: intermediate_rep
                .id
                .into_iter()
                .next()
                .ok_or_else(|| "id missing in Token".to_string())?,
            description: intermediate_rep.description.into_iter().next(),
            valid_until: intermediate_rep.valid_until.into_iter().next(),
            capabilities: intermediate_rep
                .capabilities
                .into_iter()
                .next()
                .ok_or_else(|| "capabilities missing in Token".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Token> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Token>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<Token>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Token - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Token> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <Token as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into Token - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TokenCapability {
    /// Short reference of the operation this capability is tied to.
    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "endpoint")]
    pub endpoint: String,

    #[serde(rename = "limits")]
    // #[validate()]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<Vec<models::TokenCapabilityLimit>>,
}

impl TokenCapability {
    #[allow(clippy::new_without_default)]
    pub fn new(endpoint: String) -> TokenCapability {
        TokenCapability { endpoint, limits: None }
    }
}

/// Converts the TokenCapability value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TokenCapability {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            Some("endpoint".to_string()),
            Some(self.endpoint.to_string()),
            // Skipping limits in query parameter serialization
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TokenCapability value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TokenCapability {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub endpoint: Vec<String>,
            pub limits: Vec<Vec<models::TokenCapabilityLimit>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TokenCapability".to_string()),
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "endpoint" => intermediate_rep
                        .endpoint
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    "limits" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in TokenCapability".to_string(),
                        )
                    }
                    _ => return std::result::Result::Err("Unexpected key while parsing TokenCapability".to_string()),
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TokenCapability {
            endpoint: intermediate_rep
                .endpoint
                .into_iter()
                .next()
                .ok_or_else(|| "endpoint missing in TokenCapability".to_string())?,
            limits: intermediate_rep.limits.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TokenCapability> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<TokenCapability>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TokenCapability>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for TokenCapability - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TokenCapability> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <TokenCapability as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into TokenCapability - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TokenCapabilityLimit {
    /// Limit type
    #[serde(rename = "type")]
    pub r#type: String,

    #[serde(rename = "conditions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<models::TokenCapabilityLimitConditions>,
}

impl TokenCapabilityLimit {
    #[allow(clippy::new_without_default)]
    pub fn new(r#type: String) -> TokenCapabilityLimit {
        TokenCapabilityLimit {
            r#type,
            conditions: None,
        }
    }
}

/// Converts the TokenCapabilityLimit value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TokenCapabilityLimit {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            Some("type".to_string()),
            Some(self.r#type.to_string()),
            // Skipping conditions in query parameter serialization
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TokenCapabilityLimit value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TokenCapabilityLimit {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub r#type: Vec<String>,
            pub conditions: Vec<models::TokenCapabilityLimitConditions>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err("Missing value while parsing TokenCapabilityLimit".to_string())
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "type" => intermediate_rep
                        .r#type
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "conditions" => intermediate_rep.conditions.push(
                        <models::TokenCapabilityLimitConditions as std::str::FromStr>::from_str(val)
                            .map_err(|x| x.to_string())?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing TokenCapabilityLimit".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TokenCapabilityLimit {
            r#type: intermediate_rep
                .r#type
                .into_iter()
                .next()
                .ok_or_else(|| "type missing in TokenCapabilityLimit".to_string())?,
            conditions: intermediate_rep.conditions.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TokenCapabilityLimit> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<TokenCapabilityLimit>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TokenCapabilityLimit>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for TokenCapabilityLimit - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TokenCapabilityLimit> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <TokenCapabilityLimit as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into TokenCapabilityLimit - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

/// Limit conditions, if any
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TokenCapabilityLimitConditions {
    /// Upper ceiling. Applies to limit type calls.
    #[serde(rename = "max")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<i32>,
}

impl TokenCapabilityLimitConditions {
    #[allow(clippy::new_without_default)]
    pub fn new() -> TokenCapabilityLimitConditions {
        TokenCapabilityLimitConditions { max: None }
    }
}

/// Converts the TokenCapabilityLimitConditions value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TokenCapabilityLimitConditions {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![self
            .max
            .as_ref()
            .map(|max| vec!["max".to_string(), max.to_string()].join(","))];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TokenCapabilityLimitConditions value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TokenCapabilityLimitConditions {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub max: Vec<i32>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing TokenCapabilityLimitConditions".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "max" => intermediate_rep
                        .max
                        .push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing TokenCapabilityLimitConditions".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TokenCapabilityLimitConditions {
            max: intermediate_rep.max.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TokenCapabilityLimitConditions> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<TokenCapabilityLimitConditions>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<TokenCapabilityLimitConditions>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for TokenCapabilityLimitConditions - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TokenCapabilityLimitConditions> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <TokenCapabilityLimitConditions as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into TokenCapabilityLimitConditions - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TokensCreate201Response {
    /// The generated token which must be used when authenticating for API calls.
    #[serde(rename = "token")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

impl TokensCreate201Response {
    #[allow(clippy::new_without_default)]
    pub fn new() -> TokensCreate201Response {
        TokensCreate201Response { token: None }
    }
}

/// Converts the TokensCreate201Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TokensCreate201Response {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![self
            .token
            .as_ref()
            .map(|token| vec!["token".to_string(), token.to_string()].join(","))];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TokensCreate201Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TokensCreate201Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub token: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err("Missing value while parsing TokensCreate201Response".to_string())
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "token" => intermediate_rep
                        .token
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing TokensCreate201Response".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TokensCreate201Response {
            token: intermediate_rep.token.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TokensCreate201Response> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<TokensCreate201Response>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TokensCreate201Response>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for TokensCreate201Response - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TokensCreate201Response> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <TokensCreate201Response as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into TokensCreate201Response - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TokensCreate422Response {
    #[serde(rename = "status")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    #[serde(rename = "error")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl TokensCreate422Response {
    #[allow(clippy::new_without_default)]
    pub fn new() -> TokensCreate422Response {
        TokensCreate422Response {
            status: None,
            error: None,
        }
    }
}

/// Converts the TokensCreate422Response value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TokensCreate422Response {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            self.status
                .as_ref()
                .map(|status| vec!["status".to_string(), status.to_string()].join(",")),
            self.error
                .as_ref()
                .map(|error| vec!["error".to_string(), error.to_string()].join(",")),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TokensCreate422Response value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TokensCreate422Response {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub status: Vec<String>,
            pub error: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err("Missing value while parsing TokensCreate422Response".to_string())
                }
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "status" => intermediate_rep
                        .status
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "error" => intermediate_rep
                        .error
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing TokensCreate422Response".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TokensCreate422Response {
            status: intermediate_rep.status.into_iter().next(),
            error: intermediate_rep.error.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TokensCreate422Response> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<TokensCreate422Response>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TokensCreate422Response>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for TokensCreate422Response - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TokensCreate422Response> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <TokensCreate422Response as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into TokensCreate422Response - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TokensCreateRequest {
    /// Capabilities attached to the created token.
    #[serde(rename = "capabilities")]
    // #[validate()]
    pub capabilities: Vec<models::TokenCapability>,

    /// Lifetime of the token in seconds since creation. Defaults to unlimited lifetime.
    #[serde(rename = "lifetime")]
    #[validate(range(min = 1))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifetime: Option<u32>,

    /// Description associated with the token.
    #[serde(rename = "description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl TokensCreateRequest {
    #[allow(clippy::new_without_default)]
    pub fn new(capabilities: Vec<models::TokenCapability>) -> TokensCreateRequest {
        TokensCreateRequest {
            capabilities,
            lifetime: None,
            description: None,
        }
    }
}

/// Converts the TokensCreateRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TokensCreateRequest {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            // Skipping capabilities in query parameter serialization
            self.lifetime
                .as_ref()
                .map(|lifetime| vec!["lifetime".to_string(), lifetime.to_string()].join(",")),
            self.description
                .as_ref()
                .map(|description| vec!["description".to_string(), description.to_string()].join(",")),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TokensCreateRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TokensCreateRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub capabilities: Vec<Vec<models::TokenCapability>>,
            pub lifetime: Vec<u32>,
            pub description: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TokensCreateRequest".to_string()),
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    "capabilities" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in TokensCreateRequest".to_string(),
                        )
                    }
                    #[allow(clippy::redundant_clone)]
                    "lifetime" => intermediate_rep
                        .lifetime
                        .push(<u32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "description" => intermediate_rep
                        .description
                        .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => {
                        return std::result::Result::Err("Unexpected key while parsing TokensCreateRequest".to_string())
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TokensCreateRequest {
            capabilities: intermediate_rep
                .capabilities
                .into_iter()
                .next()
                .ok_or_else(|| "capabilities missing in TokensCreateRequest".to_string())?,
            lifetime: intermediate_rep.lifetime.into_iter().next(),
            description: intermediate_rep.description.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TokensCreateRequest> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<TokensCreateRequest>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TokensCreateRequest>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for TokensCreateRequest - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TokensCreateRequest> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <TokensCreateRequest as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into TokensCreateRequest - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => {
                std::result::Result::Err(format!("Unable to convert header: {:?} to string: {}", hdr_value, e))
            }
        }
    }
}

/// Receipt identifier for an Ethereum transaction.
#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TransactionReceipt(String);

impl std::convert::From<String> for TransactionReceipt {
    fn from(x: String) -> Self {
        TransactionReceipt(x)
    }
}

impl std::string::ToString for TransactionReceipt {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl std::str::FromStr for TransactionReceipt {
    type Err = std::string::ParseError;
    fn from_str(x: &str) -> std::result::Result<Self, Self::Err> {
        std::result::Result::Ok(TransactionReceipt(x.to_string()))
    }
}

impl std::convert::From<TransactionReceipt> for String {
    fn from(x: TransactionReceipt) -> Self {
        x.0
    }
}

impl std::ops::Deref for TransactionReceipt {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl std::ops::DerefMut for TransactionReceipt {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}
