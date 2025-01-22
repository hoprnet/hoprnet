use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use url::Url;

use super::spec_extensions;

/// Allows referencing an external resource for extended documentation.
///
/// See <https://spec.openapis.org/oas/v3.1.0#external-documentation-object>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ExternalDoc {
    /// The URL for the target documentation.
    pub url: Url,

    /// A short description of the target documentation.
    /// [CommonMark syntax](https://spec.commonmark.org) MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Specification extensions.
    ///
    /// Only "x-" prefixed keys are collected, and the prefix is stripped.
    ///
    /// See <https://spec.openapis.org/oas/v3.1.0#specification-extensions>.
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: BTreeMap<String, serde_json::Value>,
}
