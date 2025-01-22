use std::str::FromStr;

use derive_more::derive::{Display, Error};
use log::trace;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::Spec;

static RE_REF: Lazy<Regex> = Lazy::new(|| {
    Regex::new("^(?P<source>[^#]*)#/components/(?P<type>[^/]+)/(?P<name>.+)$").unwrap()
});

/// Container for a type of OpenAPI object, or a reference to one.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum ObjectOrReference<T> {
    /// Object reference.
    Ref {
        /// Path, file reference, or URL pointing to object.
        #[serde(rename = "$ref")]
        ref_path: String,
    },

    /// Inline object.
    Object(T),
}

impl<T> ObjectOrReference<T>
where
    T: FromRef,
{
    /// Resolves the object (if needed) from the given `spec` and returns it.
    pub fn resolve(&self, spec: &Spec) -> Result<T, RefError> {
        match self {
            Self::Object(component) => Ok(component.clone()),
            Self::Ref { ref_path } => T::from_ref(spec, ref_path),
        }
    }
}

/// Object reference error.
#[derive(Clone, Debug, PartialEq, Display, Error)]
pub enum RefError {
    /// Referenced object has unknown type.
    #[display("Invalid type: {}", _0)]
    UnknownType(#[error(not(source))] String),

    /// Referenced object was not of expected type.
    #[display("Mismatched type: cannot reference a {} as a {}", _0, _1)]
    MismatchedType(RefType, RefType),

    /// Reference path points outside the given spec file.
    #[display("Unresolvable path: {}", _0)]
    Unresolvable(#[error(not(source))] String), // TODO: use some kind of path structure
}

/// Component type of a reference.
#[derive(Debug, Clone, Copy, PartialEq, Display)]
pub enum RefType {
    /// Schema component type.
    Schema,

    /// Response component type.
    Response,

    /// Parameter component type.
    Parameter,

    /// Example component type.
    Example,

    /// Request body component type.
    RequestBody,

    /// Header component type.
    Header,

    /// Security scheme component type.
    SecurityScheme,

    /// Link component type.
    Link,

    /// Callback component type.
    Callback,
}

impl FromStr for RefType {
    type Err = RefError;

    fn from_str(typ: &str) -> Result<Self, Self::Err> {
        Ok(match typ {
            "schemas" => Self::Schema,
            "responses" => Self::Response,
            "parameters" => Self::Parameter,
            "examples" => Self::Example,
            "requestBodies" => Self::RequestBody,
            "headers" => Self::Header,
            "securitySchemes" => Self::SecurityScheme,
            "links" => Self::Link,
            "callbacks" => Self::Callback,
            typ => return Err(RefError::UnknownType(typ.to_owned())),
        })
    }
}

/// Parsed reference path.
#[derive(Debug, Clone)]
pub struct Ref {
    /// Source file of the object being references.
    pub source: String,

    /// Type of object being referenced.
    pub kind: RefType,

    /// Name of object being referenced.
    pub name: String,
}

impl FromStr for Ref {
    type Err = RefError;

    fn from_str(path: &str) -> Result<Self, Self::Err> {
        let parts = RE_REF.captures(path).unwrap();

        trace!("creating Ref: {}/{}", &parts["type"], &parts["name"]);

        Ok(Self {
            source: parts["source"].to_owned(),
            kind: parts["type"].parse()?,
            name: parts["name"].to_owned(),
        })
    }
}

/// Find an object from a reference path (`$ref`).
///
/// Implemented for object types which can be shared via a spec's `components` object.
pub trait FromRef: Clone {
    /// Finds an object in `spec` using the given `path`.
    fn from_ref(spec: &Spec, path: &str) -> Result<Self, RefError>;
}
