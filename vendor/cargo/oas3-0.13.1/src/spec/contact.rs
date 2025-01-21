use std::collections::BTreeMap;

use derive_more::derive::{Display, Error};
use serde::{Deserialize, Serialize};
use url::Url;

use super::spec_extensions;

/// Error raised when contact info contains an email field which is not a valid email.
#[derive(Debug, Display, Error)]
#[display("Email address is not valid")]
#[non_exhaustive]
pub struct InvalidEmail;

/// Contact information for the exposed API.
///
/// See <https://spec.openapis.org/oas/v3.1.0#contact-object>.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Contact {
    /// Identifying name of the contact person/organization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// URL pointing to the contact information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,

    /// Email address of the contact person/organization.
    ///
    /// Use [`validate_email()`](Self::validate_email) after deserializing to check that email
    /// address provided is valid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Specification extensions.
    ///
    /// Only "x-" prefixed keys are collected, and the prefix is stripped.
    ///
    /// See <https://spec.openapis.org/oas/v3.1.0#specification-extensions>.
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

impl Contact {
    /// Validates email address field.
    pub fn validate_email(&self) -> Result<(), InvalidEmail> {
        let Some(email) = &self.email else {
            return Ok(());
        };

        if email.contains("@") {
            Ok(())
        } else {
            Err(InvalidEmail)
        }
    }
}
