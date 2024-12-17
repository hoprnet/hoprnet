use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{FromRef, MediaType, Ref, RefError, RefType, Spec};

/// Describes a single request body.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#request-body-object>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct RequestBody {
    /// A brief description of the request body. This could contain examples of use.
    /// [CommonMark syntax](http://spec.commonmark.org/) MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The content of the request body. The key is a media type or
    /// [media type range](https://tools.ietf.org/html/rfc7231#appendix-D) and the
    /// value describes it. For requests that match multiple keys, only the most specific key
    /// is applicable. e.g. text/plain overrides text/*
    pub content: BTreeMap<String, MediaType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

impl FromRef for RequestBody {
    fn from_ref(spec: &Spec, path: &str) -> Result<Self, RefError>
    where
        Self: Sized,
    {
        let refpath = path.parse::<Ref>()?;

        match refpath.kind {
            RefType::RequestBody => spec
                .components
                .as_ref()
                .and_then(|cs| cs.request_bodies.get(&refpath.name))
                .ok_or_else(|| RefError::Unresolvable(path.to_owned()))
                .and_then(|oor| oor.resolve(spec)),

            typ => Err(RefError::MismatchedType(typ, RefType::RequestBody)),
        }
    }
}
