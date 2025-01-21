use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{
    schema::ObjectSchema, spec_extensions, Callback, Example, Header, Link, ObjectOrReference,
    Parameter, PathItem, RequestBody, Response, SecurityScheme,
};

/// Holds a set of reusable objects for different aspects of the OAS.
///
/// All objects defined within the components object will have no effect on the API unless
/// they are explicitly referenced from properties outside the components object.
///
/// See <https://spec.openapis.org/oas/v3.1.0#components-object>.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Components {
    /// An object to hold reusable [Schema Objects](ObjectSchema).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub schemas: BTreeMap<String, ObjectOrReference<ObjectSchema>>,

    /// An object to hold reusable [Response Objects](Response).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub responses: BTreeMap<String, ObjectOrReference<Response>>,

    /// An object to hold reusable [Parameter Objects](Parameter).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub parameters: BTreeMap<String, ObjectOrReference<Parameter>>,

    /// An object to hold reusable [Example Objects](Example).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub examples: BTreeMap<String, ObjectOrReference<Example>>,

    /// An object to hold reusable [Request Body Objects](RequestBody).
    #[serde(
        rename = "requestBodies",
        default,
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub request_bodies: BTreeMap<String, ObjectOrReference<RequestBody>>,

    /// An object to hold reusable [Header Objects](Header).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, ObjectOrReference<Header>>,

    /// An object to hold reusable [Path Item Objects](PathItem).
    #[serde(
        rename = "pathItems",
        default,
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub path_items: BTreeMap<String, ObjectOrReference<PathItem>>,

    /// An object to hold reusable [Security Scheme Objects](SecurityScheme).
    #[serde(
        rename = "securitySchemes",
        default,
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub security_schemes: BTreeMap<String, ObjectOrReference<SecurityScheme>>,

    /// An object to hold reusable [Link Objects](crate::spec::Link).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub links: BTreeMap<String, ObjectOrReference<Link>>,

    /// An object to hold reusable [Callback Objects](crate::spec::Callback).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub callbacks: BTreeMap<String, ObjectOrReference<Callback>>,

    /// Specification extensions.
    ///
    /// Only "x-" prefixed keys are collected, and the prefix is stripped.
    ///
    /// See <https://spec.openapis.org/oas/v3.1.0#specification-extensions>.
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: BTreeMap<String, serde_json::Value>,
}
