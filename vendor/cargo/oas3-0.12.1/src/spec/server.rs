use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// An object representing a Server.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#server-object>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Server {
    /// A URL to the target host. This URL supports Server Variables and MAY be relative, to
    /// indicate that the host location is relative to the location where the OpenAPI document
    /// is being served. Variable substitutions will be made when a variable is named
    /// in {brackets}.
    pub url: String,

    /// An optional string describing the host designated by the URL. CommonMark syntax MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A map between a variable name and its value. The value is used for substitution in
    /// the server's URL template.
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub variables: BTreeMap<String, ServerVariable>,
}

/// An object representing a Server Variable for server URL template substitution.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#server-variable-object>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ServerVariable {
    /// The default value to use for substitution, and to send, if an alternate value is not
    /// supplied. Unlike the Schema Object's default, this value MUST be provided by the consumer.
    pub default: String,

    /// An enumeration of string values to be used if the substitution options are from a limited
    /// set.
    #[serde(default)]
    #[serde(rename = "enum")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub substitutions_enum: Vec<String>,

    /// An optional description for the server variable. [CommonMark] syntax MAY be used for rich
    /// text representation.
    ///
    /// [CommonMark]: https://spec.commonmark.org/
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
