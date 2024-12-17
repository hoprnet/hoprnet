use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use url::Url;

/// Allows configuration of the supported OAuth Flows.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#oauth-flows-object>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Flows {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implicit: Option<ImplicitFlow>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<PasswordFlow>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_credentials: Option<ClientCredentialsFlow>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_code: Option<AuthorizationCodeFlow>,
}

/// Configuration details for a implicit OAuth Flow.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#oauth-flow-object>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImplicitFlow {
    pub authorization_url: Url,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<Url>,

    pub scopes: BTreeMap<String, String>,
}

/// Configuration details for a password OAuth Flow.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#oauth-flow-object>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PasswordFlow {
    token_url: Url,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<Url>,

    pub scopes: BTreeMap<String, String>,
}

/// Configuration details for a client credentials OAuth Flow.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#oauth-flow-object>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClientCredentialsFlow {
    token_url: Url,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<Url>,

    pub scopes: BTreeMap<String, String>,
}

/// Configuration details for a authorization code OAuth Flow.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#oauth-flow-object>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationCodeFlow {
    pub authorization_url: Url,
    pub token_url: Url,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<Url>,

    pub scopes: BTreeMap<String, String>,
}

// TODO: Implement
/// Map of possible out-of band callbacks related to the parent operation.
///
/// Each value in the map is a Path Item Object that describes a set of requests that may be
/// initiated by the API provider and the expected responses. The key value used to identify the
/// callback object is an expression, evaluated at runtime, that identifies a URL to use for the
/// callback operation.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#callback-object>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct Callback(
    /// A Path Item Object used to define a callback request and expected responses.
    serde_json::Value, // TODO: Add "Specification Extensions" https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#specificationExtensions}
);

// FIXME: Implement
// /// Allows configuration of the supported OAuth Flows.
// /// https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#oauthFlowsObject
// #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
// pub struct OAuthFlows {
// }
