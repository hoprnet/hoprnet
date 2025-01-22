use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{spec_extensions, Server};

/// The Link object represents a possible design-time link for a response.
///
/// The presence of a link does not guarantee the caller's ability to successfully invoke it, rather
/// it provides a known relationship and traversal mechanism between responses and other operations.
///
/// Unlike _dynamic_ links (i.e. links provided *in* the response payload), the OAS linking
/// mechanism does not require link information in the runtime response.
///
/// For computing links, and providing instructions to execute them, a [runtime expression] is used
/// for accessing values in an operation and using them as parameters while invoking the linked
/// operation.
///
/// The `operationRef` and `operationId` fields are mutually exclusive and so this structure is
/// modelled as an enum.
///
/// See <https://spec.openapis.org/oas/v3.1.0#link-object>.
///
/// [runtime expression]: https://spec.openapis.org/oas/v3.1.0#runtime-expressions
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Link {
    /// A relative or absolute reference to an OAS operation.
    Ref {
        /// A relative or absolute reference to an OAS operation.
        ///
        /// This field is mutually exclusive of the `operationId` field, and MUST point to an
        /// [Operation Object]. Relative `operationRef` values MAY be used to locate an existing
        /// [Operation Object] in the OpenAPI definition.
        ///
        /// [Operation Object]: https://spec.openapis.org/oas/v3.1.0#operation-object
        #[serde(rename = "operationRef")]
        operation_ref: String,

        /// A map representing parameters to pass to an operation.
        ///
        /// The key is the parameter name to be used, whereas the value can be a constant or an
        /// expression to be evaluated and passed to the linked operation. The parameter name can be
        /// qualified using the [parameter location] `[{in}.]{name}` for operations that use the
        /// same parameter name in different locations (e.g. path.id).
        ///
        /// [parameter location]: https://spec.openapis.org/oas/v3.1.0#parameterIn
        //
        // FIXME: Implement
        // parameters: BTreeMap<String, Any | {expression}>,
        //
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        parameters: BTreeMap<String, String>,

        // FIXME: Implement
        // /// A literal value or
        // /// [{expression}](https://spec.openapis.org/oas/v3.1.0#runtimeExpression)
        // /// to use as a request body when calling the target operation.
        // #[serde(rename = "requestBody")]
        // request_body: Any | {expression}
        //
        /// A description of the link.
        ///
        /// [CommonMark syntax](https://spec.commonmark.org) MAY be used for rich text
        /// representation.
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,

        /// A server object to be used by the target operation.
        #[serde(skip_serializing_if = "Option::is_none")]
        server: Option<Server>,

        /// Specification extensions.
        ///
        /// Only "x-" prefixed keys are collected, and the prefix is stripped.
        ///
        /// See <https://spec.openapis.org/oas/v3.1.0#specification-extensions>.
        #[serde(flatten, with = "spec_extensions")]
        extensions: BTreeMap<String, serde_json::Value>,
    },

    /// The name of an _existing_, resolvable OAS operation, as defined with a unique `operationId`.
    Id {
        /// The name of an _existing_, resolvable OAS operation, as defined with a unique
        /// `operationId`.
        #[serde(rename = "operationId")]
        operation_id: String,

        /// A map representing parameters to pass to an operation.
        ///
        /// The key is the parameter name to be used, whereas the value can be a constant or an
        /// expression to be evaluated and passed to the linked operation. The parameter name can be
        /// qualified using the [parameter location] `[{in}.]{name}` for operations that use the
        /// same parameter name in different locations (e.g. path.id).
        ///
        /// [parameter location]: https://spec.openapis.org/oas/v3.1.0#parameterIn
        //
        // FIXME: Implement
        // parameters: BTreeMap<String, Any | {expression}>,
        //
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        parameters: BTreeMap<String, String>,

        // FIXME: Implement
        // /// A literal value or
        // /// [{expression}](https://spec.openapis.org/oas/v3.1.0#runtimeExpression)
        // /// to use as a request body when calling the target operation.
        // #[serde(rename = "requestBody")]
        // request_body: Any | {expression}
        //
        /// A description of the link.
        ///
        /// [CommonMark syntax](https://spec.commonmark.org) MAY be used for rich text
        /// representation.
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,

        /// A server object to be used by the target operation.
        #[serde(skip_serializing_if = "Option::is_none")]
        server: Option<Server>,

        /// Specification extensions.
        ///
        /// Only "x-" prefixed keys are collected, and the prefix is stripped.
        ///
        /// See <https://spec.openapis.org/oas/v3.1.0#specification-extensions>.
        #[serde(flatten, with = "spec_extensions")]
        extensions: BTreeMap<String, serde_json::Value>,
    },
}
