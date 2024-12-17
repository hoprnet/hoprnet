use std::collections::BTreeMap;

use log::error;
use serde::{Deserialize, Serialize};

use super::{
    Callback, Error, ExternalDoc, ObjectOrReference, Parameter, RequestBody, Response, Server, Spec,
};
use crate::spec::spec_extensions;

/// Describes a single API operation on a path.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#operation-object>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
// #[serde(rename_all = "lowercase")]
pub struct Operation {
    /// A list of tags for API documentation control. Tags can be used for logical grouping of
    /// operations by resources or any other qualifier.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// A short summary of what the operation does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// A verbose explanation of the operation behavior.
    /// [CommonMark syntax](http://spec.commonmark.org/) MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Additional external documentation for this operation.
    #[serde(skip_serializing_if = "Option::is_none", rename = "externalDocs")]
    pub external_docs: Option<ExternalDoc>,

    /// Unique string used to identify the operation. The id MUST be unique among all operations
    /// described in the API. Tools and libraries MAY use the operationId to uniquely identify an
    /// operation, therefore, it is RECOMMENDED to follow common programming naming conventions.
    #[serde(skip_serializing_if = "Option::is_none", rename = "operationId")]
    pub operation_id: Option<String>,

    /// A list of parameters that are applicable for this operation. If a parameter is already
    /// defined at the
    /// [Path Item](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#pathItemParameters),
    /// the new definition will override it but can never remove it. The list MUST NOT
    /// include duplicated parameters. A unique parameter is defined by a combination of a
    /// [name](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#parameterName)
    /// and
    /// [location](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#parameterIn).
    /// The list can use the
    /// [Reference Object](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#reference-object)
    /// to link to parameters that are defined at the
    /// [OpenAPI Object's components/parameters](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#components-parameters).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<ObjectOrReference<Parameter>>,

    /// The request body applicable for this operation. The requestBody is only supported in HTTP methods where the HTTP 1.1 specification RFC7231 has explicitly defined semantics for request bodies. In other cases where the HTTP spec is vague, requestBody SHALL be ignored by consumers.
    #[serde(rename = "requestBody", skip_serializing_if = "Option::is_none")]
    pub request_body: Option<ObjectOrReference<RequestBody>>,

    /// The list of possible responses as they are returned from executing this operation.
    ///
    /// A container for the expected responses of an operation. The container maps a HTTP
    /// response code to the expected response.
    ///
    /// The documentation is not necessarily expected to cover all possible HTTP response codes
    /// because they may not be known in advance. However, documentation is expected to cover
    /// a successful operation response and any known errors.
    ///
    /// The `default` MAY be used as a default response object for all HTTP codes that are not
    /// covered individually by the specification.
    ///
    /// The `Responses Object` MUST contain at least one response code, and it SHOULD be the
    /// response for a successful operation call.
    ///
    /// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#responses-object>.
    pub responses: Option<BTreeMap<String, ObjectOrReference<Response>>>,

    /// A map of possible out-of band callbacks related to the parent operation. The key is
    /// a unique identifier for the Callback Object. Each value in the map is a
    /// [Callback Object](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#callback-object)
    /// that describes a request that may be initiated by the API provider and the
    /// expected responses. The key value used to identify the callback object is
    /// an expression, evaluated at runtime, that identifies a URL to use for the
    /// callback operation.
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub callbacks: BTreeMap<String, Callback>,

    /// Declares this operation to be deprecated. Consumers SHOULD refrain from usage
    /// of the declared operation. Default value is `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,

    // FIXME: Implement
    // /// A declaration of which security mechanisms can be used for this operation. The list of
    // /// values includes alternative security requirement objects that can be used. Only one
    // /// of the security requirement objects need to be satisfied to authorize a request.
    // /// This definition overrides any declared top-level
    // /// [`security`](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#oasSecurity).
    // /// To remove a top-level security declaration, an empty array can be used.
    // pub security: Option<SecurityRequirement>,
    //
    /// An alternative `server` array to service this operation. If an alternative `server`
    /// object is specified at the Path Item Object or Root level, it will be overridden by
    /// this value.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub servers: Vec<Server>,

    /// Specification extensions.
    ///
    /// Only "x-" prefixed keys are collected, and the prefix is stripped.
    ///
    /// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#specification-extensions>.
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

impl Operation {
    pub fn request_body(&self, spec: &Spec) -> Result<RequestBody, Error> {
        self.request_body
            .as_ref()
            .unwrap()
            .resolve(spec)
            .map_err(Error::Ref)
    }

    pub fn responses(&self, spec: &Spec) -> BTreeMap<String, Response> {
        self.responses
            .iter()
            .flatten()
            .filter_map(|(name, oor)| {
                oor.resolve(spec)
                    .map(|obj| (name.clone(), obj))
                    // TODO: find better error solution
                    .map_err(|err| error!("{}", err))
                    .ok()
            })
            .collect()
    }

    pub fn parameters(&self, spec: &Spec) -> Result<Vec<Parameter>, Error> {
        let params = self
            .parameters
            .iter()
            // TODO: find better error solution, maybe vec<result<_>>
            .filter_map(|oor| oor.resolve(spec).map_err(|err| error!("{}", err)).ok())
            .collect();

        Ok(params)
    }

    pub fn parameter(&self, search: &str, spec: &Spec) -> Result<Option<Parameter>, Error> {
        let param = self
            .parameters(spec)?
            .iter()
            .find(|param| param.name == search)
            .cloned();

        Ok(param)
    }
}
