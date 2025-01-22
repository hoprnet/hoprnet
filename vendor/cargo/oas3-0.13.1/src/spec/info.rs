use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use url::Url;

use super::{spec_extensions, Contact, License};

/// General information about the API.
///
///
/// See <https://spec.openapis.org/oas/v3.1.0#info-object>.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// #[serde(rename_all = "lowercase")]
pub struct Info {
    /// The title of the application.
    pub title: String,

    /// A short description of the application. CommonMark syntax MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// A short description of the application. CommonMark syntax MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A URL to the Terms of Service for the API. MUST be in the format of a URL.
    #[serde(rename = "termsOfService", skip_serializing_if = "Option::is_none")]
    pub terms_of_service: Option<Url>,

    /// The version of the OpenAPI document (which is distinct from the [OpenAPI Specification
    /// version](https://spec.openapis.org/oas/v3.1.0#oasVersion)
    /// or the API implementation version).
    pub version: String,

    /// The contact information for the exposed API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,

    /// The license information for the exposed API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<License>,

    /// Specification extensions.
    ///
    /// Only "x-" prefixed keys are collected, and the prefix is stripped.
    ///
    /// See <https://spec.openapis.org/oas/v3.1.0#specification-extensions>.
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: BTreeMap<String, serde_json::Value>,
}
