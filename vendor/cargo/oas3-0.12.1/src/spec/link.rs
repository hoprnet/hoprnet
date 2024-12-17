use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{spec_extensions, Server};

/// The Link object represents a possible design-time link for a response.
///
/// The presence of a link does not guarantee the caller's ability to successfully invoke it,
/// rather it provides a known relationship and traversal mechanism between responses and
/// other operations.
///
/// Unlike _dynamic_ links (i.e. links provided *in* the response payload), the OAS linking
/// mechanism does not require link information in the runtime response.
///
/// For computing links, and providing instructions to execute them, a
/// [runtime expression](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#runtimeExpression)
/// is used for accessing values in an operation and using them as parameters while invoking
/// the linked operation.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#link-object>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Link {
    /// A relative or absolute reference to an OAS operation. This field is mutually exclusive
    /// of the `operationId` field, and MUST point to an
    /// [Operation Object](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#operation-object).
    /// Relative `operationRef` values MAY be used to locate an existing
    /// [Operation Object](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#operation-object)
    /// in the OpenAPI definition.
    Ref {
        #[serde(rename = "operationRef")]
        operation_ref: String,

        // FIXME: Implement
        // /// A map representing parameters to pass to an operation as specified with `operationId`
        // /// or identified via `operationRef`. The key is the parameter name to be used, whereas
        // /// the value can be a constant or an expression to be evaluated and passed to the
        // /// linked operation. The parameter name can be qualified using the
        // /// [parameter location](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#parameterIn)
        // /// `[{in}.]{name}` for operations that use the same parameter name in different
        // /// locations (e.g. path.id).
        // parameters: BTreeMap<String, Any | {expression}>,
        //
        #[serde(default)]
        #[serde(skip_serializing_if = "BTreeMap::is_empty")]
        parameters: BTreeMap<String, String>,

        // FIXME: Implement
        // /// A literal value or
        // /// [{expression}](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#runtimeExpression)
        // /// to use as a request body when calling the target operation.
        // #[serde(rename = "requestBody")]
        // request_body: Any | {expression}
        //
        /// A description of the link. [CommonMark syntax](http://spec.commonmark.org/) MAY be
        /// used for rich text representation.
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,

        /// A server object to be used by the target operation.
        #[serde(skip_serializing_if = "Option::is_none")]
        server: Option<Server>,

        /// Specification extensions.
        ///
        /// Only "x-" prefixed keys are collected, and the prefix is stripped.
        ///
        /// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#specification-extensions>.
        #[serde(flatten, with = "spec_extensions")]
        extensions: BTreeMap<String, serde_json::Value>,
    },
    /// The name of an _existing_, resolvable OAS operation, as defined with a unique
    /// `operationId`. This field is mutually exclusive of the `operationRef` field.
    Id {
        #[serde(rename = "operationId")]
        operation_id: String,

        // FIXME: Implement
        // /// A map representing parameters to pass to an operation as specified with `operationId`
        // /// or identified via `operationRef`. The key is the parameter name to be used, whereas
        // /// the value can be a constant or an expression to be evaluated and passed to the
        // /// linked operation. The parameter name can be qualified using the
        // /// [parameter location](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#parameterIn)
        // /// `[{in}.]{name}` for operations that use the same parameter name in different
        // /// locations (e.g. path.id).
        // parameters: BTreeMap<String, Any | {expression}>,
        //
        #[serde(default)]
        #[serde(skip_serializing_if = "BTreeMap::is_empty")]
        parameters: BTreeMap<String, String>,

        // FIXME: Implement
        // /// A literal value or
        // /// [{expression}](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#runtimeExpression)
        // /// to use as a request body when calling the target operation.
        // #[serde(rename = "requestBody")]
        // request_body: Any | {expression}
        /// A description of the link. [CommonMark syntax](http://spec.commonmark.org/) MAY be
        /// used for rich text representation.
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,

        /// A server object to be used by the target operation.
        #[serde(skip_serializing_if = "Option::is_none")]
        server: Option<Server>,

        /// Specification extensions.
        ///
        /// Only "x-" prefixed keys are collected, and the prefix is stripped.
        ///
        /// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#specification-extensions>.
        #[serde(flatten, with = "spec_extensions")]
        extensions: BTreeMap<String, serde_json::Value>,
    },
}
