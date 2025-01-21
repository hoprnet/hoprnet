use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{FromRef, MediaType, Ref, RefError, RefType, Spec};

/// Describes a single request body.
///
/// See <https://spec.openapis.org/oas/v3.1.0#request-body-object>.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct RequestBody {
    /// A brief description of the request body.
    ///
    /// This could contain examples of use.
    ///
    /// [CommonMark syntax](https://spec.commonmark.org) MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The content of the request body.
    ///
    /// The key is a media type or [media type range] and the value describes it. For requests that
    /// match multiple keys, only the most specific key is applicable. E.g., `text/plain` overrides
    /// `text/*`.
    ///
    /// [media type range]: https://tools.ietf.org/html/rfc7231#appendix-D
    pub content: BTreeMap<String, MediaType>,

    /// Determines if the request body is required in the request.
    ///
    /// Defaults to false.
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
