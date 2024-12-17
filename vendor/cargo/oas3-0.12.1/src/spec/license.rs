use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use url::Url;

use super::spec_extensions;

/// License information for the exposed API.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#license-object>.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct License {
    /// The license name used for the API.
    pub name: String,

    /// An SPDX license expression for the API. The identifier field is mutually exclusive of the url field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,

    /// A URL to the license used for the API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,

    /// Specification extensions.
    ///
    /// Only "x-" prefixed keys are collected, and the prefix is stripped.
    ///
    /// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#specification-extensions>.
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: BTreeMap<String, serde_json::Value>,
}
