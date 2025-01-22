use std::collections::BTreeMap;

use http::Method;
use serde::{Deserialize, Serialize};

use super::{spec_extensions, ObjectOrReference, Operation, Parameter, Server};

/// Describes the operations available on a single path.
///
/// A Path Item MAY be empty, due to [ACL constraints]. The path itself is still exposed to the
/// documentation viewer but they will not know which operations and parameters are available.
///
/// [ACL constraints]: https://spec.openapis.org/oas/v3.1.0#securityFiltering
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct PathItem {
    /// An external definition of this path item.
    ///
    /// The referenced structure MUST be in the format of a [Path Item Object]. If there are
    /// conflicts between the referenced definition and this Path Item's definition, the behavior is
    /// undefined.
    ///
    /// [Path Item Object]: https://spec.openapis.org/oas/v3.1.0#pathItemObject
    // FIXME: Should this ref be moved to an enum?
    #[serde(skip_serializing_if = "Option::is_none", rename = "$ref")]
    pub reference: Option<String>,

    /// An optional, string summary, intended to apply to all operations in this path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// An optional, string description, intended to apply to all operations in this path.
    ///
    /// [CommonMark syntax](https://spec.commonmark.org) MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A definition of a GET operation on this path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,

    /// A definition of a PUT operation on this path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,

    /// A definition of a POST operation on this path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,

    /// A definition of a DELETE operation on this path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,

    /// A definition of a OPTIONS operation on this path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Operation>,

    /// A definition of a HEAD operation on this path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<Operation>,

    /// A definition of a PATCH operation on this path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<Operation>,

    /// A definition of a TRACE operation on this path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Operation>,

    /// An alternative `server` array to service all operations in this path.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub servers: Vec<Server>,

    /// A list of parameters that are applicable for all the operations described under this path.
    ///
    /// These parameters can be overridden at the operation level, but cannot be removed there. The
    /// list MUST NOT include duplicated parameters. A unique parameter is defined by a combination
    /// of a [name] and [location] The list can use the [Reference Object] to link to parameters
    /// that are defined at the [OpenAPI Object's components/parameters].
    ///
    /// [name]: https://spec.openapis.org/oas/v3.1.0#parameterName
    /// [location]: https://spec.openapis.org/oas/v3.1.0#parameterIn
    /// [Reference Object]: https://spec.openapis.org/oas/v3.1.0#reference-object
    /// [OpenAPI Object's components/parameters]: https://spec.openapis.org/oas/v3.1.0#componentsParameters
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<ObjectOrReference<Parameter>>,

    /// Specification extensions.
    ///
    /// Only "x-" prefixed keys are collected, and the prefix is stripped.
    ///
    /// See <https://spec.openapis.org/oas/v3.1.0#specification-extensions>.
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

impl PathItem {
    /// Returns iterator over this path's provided operations, keyed by method.
    pub fn methods(&self) -> impl IntoIterator<Item = (Method, &Operation)> {
        let mut methods = vec![];

        macro_rules! push_method {
            ($field:ident, $method:ident) => {{
                if let Some(ref op) = self.$field {
                    methods.push((Method::$method, op))
                }
            }};
        }

        push_method!(get, GET);
        push_method!(put, PUT);
        push_method!(post, POST);
        push_method!(delete, DELETE);
        push_method!(options, OPTIONS);
        push_method!(head, HEAD);
        push_method!(patch, PATCH);
        push_method!(trace, TRACE);
        push_method!(trace, TRACE);

        methods
    }
}
