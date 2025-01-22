use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::spec_extensions;

/// Adds metadata to a single tag that is used by the [Operation Object].
///
/// It is not mandatory to have a Tag Object per tag defined in the Operation Object instances.
///
/// See <https://spec.openapis.org/oas/v3.1.0#tag-object>.
///
/// [Operation Object]: https://spec.openapis.org/oas/v3.1.0#operation-object
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Tag {
    /// The name of the tag.
    pub name: String,

    /// A short description for the tag.
    /// [CommonMark syntax](http://spec.commonmark.org/) MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    //
    // /// Additional external documentation for this tag.
    // #[serde(default)]
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub external_docs: Vec<ExternalDoc>,
    //
    /// Specification extensions.
    ///
    /// Only "x-" prefixed keys are collected, and the prefix is stripped.
    ///
    /// See <https://spec.openapis.org/oas/v3.1.0#specification-extensions>.
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: BTreeMap<String, serde_json::Value>,
}
