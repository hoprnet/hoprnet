//! Structures used in parsing and navigating OpenAPI specifications.
//!
//! High-level structures include [`Spec`], [`Components`] & [`Schema`].

use std::{collections::BTreeMap, iter::Iterator};

use derive_more::derive::Error;
use http::Method;
use log::debug;
use serde::{Deserialize, Serialize};

mod components;
mod contact;
mod encoding;

mod discriminator;
mod error;
mod example;
mod external_doc;
mod flows;
mod header;
mod info;
mod license;
mod link;
mod media_type;
mod media_type_examples;
mod operation;
mod parameter;
mod path_item;
mod r#ref;
mod request_body;
mod response;
mod schema;
mod security_scheme;
mod server;
mod spec_extensions;
mod tag;

pub use self::{
    components::*,
    contact::*,
    discriminator::*,
    encoding::*,
    error::Error,
    example::*,
    external_doc::*,
    flows::*,
    header::*,
    info::*,
    license::*,
    link::*,
    media_type::*,
    media_type_examples::*,
    operation::*,
    parameter::*,
    path_item::*,
    r#ref::*,
    request_body::*,
    response::*,
    schema::{
        BooleanSchema, Error as SchemaError, ObjectSchema, Schema, Type as SchemaType,
        TypeSet as SchemaTypeSet,
    },
    security_scheme::*,
    server::*,
    tag::*,
};

const OPENAPI_SUPPORTED_VERSION_RANGE: &str = "~3.1";

/// A complete OpenAPI specification.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Spec {
    /// This string MUST be the [semantic version number](https://semver.org/spec/v2.0.0.html)
    /// of the
    /// [OpenAPI Specification version](https://spec.openapis.org/oas/v3.1.0#versions)
    /// that the OpenAPI document uses. The `openapi` field SHOULD be used by tooling
    /// specifications and clients to interpret the OpenAPI document. This is not related to
    /// the API
    /// [`info.version`](https://spec.openapis.org/oas/v3.1.0#infoVersion)
    /// string.
    pub openapi: String,

    /// Provides metadata about the API. The metadata MAY be used by tooling as required.
    pub info: Info,

    /// An array of Server Objects, which provide connectivity information to a target server.
    /// If the `servers` property is not provided, or is an empty array, the default value would
    /// be a
    /// [Server Object](https://spec.openapis.org/oas/v3.1.0#server-object)
    /// with a
    /// [url](https://spec.openapis.org/oas/v3.1.0#serverUrl)
    /// value of `/`.
    // FIXME: Provide a default value as specified in documentation instead of `None`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub servers: Vec<Server>,

    /// Holds the relative paths to the individual endpoints and their operations. The path is
    /// appended to the URL from the
    /// [`Server Object`](https://spec.openapis.org/oas/v3.1.0#server-object)
    /// in order to construct the full URL. The Paths MAY be empty, due to
    /// [ACL constraints](https://spec.openapis.org/oas/v3.1.0#securityFiltering).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paths: Option<BTreeMap<String, PathItem>>,

    /// An element to hold various schemas for the specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Components>,

    // FIXME: Implement
    // /// A declaration of which security mechanisms can be used across the API.
    // /// The list of  values includes alternative security requirement objects that can be used.
    // /// Only one of the security requirement objects need to be satisfied to authorize a request.
    // /// Individual operations can override this definition.
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub security: Option<SecurityRequirement>,
    //
    /// A list of tags used by the specification with additional metadata.
    ///The order of the tags can be used to reflect on their order by the parsing tools.
    /// Not all tags that are used by the
    /// [Operation Object](https://spec.openapis.org/oas/v3.1.0#operation-object)
    /// must be declared. The tags that are not declared MAY be organized randomly or
    /// based on the tools' logic. Each tag name in the list MUST be unique.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,

    /// The incoming webhooks that MAY be received as part of this API and that the API consumer MAY
    /// choose to implement. Closely related to the callbacks feature, this section describes
    /// requests initiated other than by an API call, for example by an out of band registration.
    /// The key name is a unique string to refer to each webhook, while the (optionally referenced)
    /// Path Item Object describes a request that may be initiated by the API provider and the
    /// expected responses. An example is available.
    ///
    /// See <>.
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub webhooks: BTreeMap<String, PathItem>,

    /// Additional external documentation.
    #[serde(skip_serializing_if = "Option::is_none", rename = "externalDocs")]
    pub external_docs: Option<ExternalDoc>,

    /// Specification extensions.
    ///
    /// Only "x-" prefixed keys are collected, and the prefix is stripped.
    ///
    /// See <https://spec.openapis.org/oas/v3.1.0#specification-extensions>.
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

impl Spec {
    /// Validates spec version field.
    pub fn validate_version(&self) -> Result<semver::Version, Error> {
        let spec_version = &self.openapi;
        let sem_ver = semver::Version::parse(spec_version)?;
        let required_version = semver::VersionReq::parse(OPENAPI_SUPPORTED_VERSION_RANGE).unwrap();

        if required_version.matches(&sem_ver) {
            Ok(sem_ver)
        } else {
            Err(Error::UnsupportedSpecFileVersion(sem_ver))
        }
    }

    /// Returns a reference to the operation with given `operation_id`, or `None` if not found.
    pub fn operation_by_id(&self, operation_id: &str) -> Option<&Operation> {
        self.operations()
            .find(|(_, _, op)| {
                op.operation_id
                    .as_deref()
                    .map_or(false, |id| id == operation_id)
            })
            .map(|(_, _, op)| op)
    }

    /// Returns a reference to the operation with given `method` and `path`, or `None` if not found.
    pub fn operation(&self, method: &http::Method, path: &str) -> Option<&Operation> {
        let resource = self.paths.as_ref()?.get(path)?;

        match *method {
            Method::GET => resource.get.as_ref(),
            Method::POST => resource.post.as_ref(),
            Method::PUT => resource.put.as_ref(),
            Method::PATCH => resource.patch.as_ref(),
            Method::DELETE => resource.delete.as_ref(),
            Method::HEAD => resource.head.as_ref(),
            Method::OPTIONS => resource.options.as_ref(),
            Method::TRACE => resource.trace.as_ref(),
            _ => None,
        }
    }

    /// Returns an iterator over all the operations defined in this spec.
    pub fn operations(&self) -> impl Iterator<Item = (String, Method, &Operation)> {
        let paths = &self.paths;

        debug!(
            "num paths: {}",
            paths.as_ref().map_or(0, |paths| paths.len())
        );

        let ops = paths
            .iter()
            .flatten()
            .flat_map(|(path, item)| {
                debug!(
                    "path: {}, methods: {}",
                    path,
                    item.methods().into_iter().count()
                );

                item.methods()
                    .into_iter()
                    .map(move |(method, op)| (path.to_owned(), method, op))
            })
            .collect::<Vec<_>>();

        debug!("num ops: {}", ops.len());

        ops.into_iter()
    }

    /// Returns a reference to the primary (first) server definition.
    pub fn primary_server(&self) -> Option<&Server> {
        self.servers.first()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn spec_extensions_deserialize() {
        let spec = indoc::indoc! {"
            openapi: '3.1.0'
            info:
              title: test
              version: v1
            components: {}
            x-bar: true
            qux: true
        "};

        let spec = serde_yml::from_str::<Spec>(spec).unwrap();
        assert!(spec.components.is_some());
        assert!(!spec.extensions.contains_key("x-bar"));
        assert!(!spec.extensions.contains_key("qux"));
        assert_eq!(spec.extensions.get("bar").unwrap(), true);
    }

    #[test]
    fn spec_extensions_serialize() {
        let spec = indoc::indoc! {"
            openapi: '3.1.0'
            info:
              title: test
              version: v1
            components: {}
            x-bar: true
        "};

        let parsed_spec = serde_yml::from_str::<Spec>(spec).unwrap();
        let round_trip_spec = serde_yml::to_string(&parsed_spec).unwrap();

        assert_eq!(spec, round_trip_spec);
    }
}
