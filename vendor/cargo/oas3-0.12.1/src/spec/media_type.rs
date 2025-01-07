use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{Encoding, Error, Example, MediaTypeExamples, ObjectOrReference, ObjectSchema, Spec};

/// Each Media Type Object provides schema and examples for the media type identified by its key.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#media-type-object>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct MediaType {
    /// The schema defining the type used for the request body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<ObjectOrReference<ObjectSchema>>,

    /// Example of the media type.
    // TODO: figure out how to make this not an Option
    // #[serde(flatten, default, skip_serializing_if = "MediaTypeExamples::is_empty")]
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub examples: Option<MediaTypeExamples>,

    /// A map between a property name and its encoding information. The key, being the
    /// property name, MUST exist in the schema as a property. The encoding object SHALL
    /// only apply to `requestBody` objects when the media type is `multipart`
    /// or `application/x-www-form-urlencoded`.
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub encoding: BTreeMap<String, Encoding>,
}

impl MediaType {
    pub fn schema(&self, spec: &Spec) -> Result<ObjectSchema, Error> {
        self.schema
            .as_ref()
            .unwrap()
            .resolve(spec)
            .map_err(Error::Ref)
    }

    pub fn examples(&self, spec: &Spec) -> BTreeMap<String, Example> {
        self.examples
            .as_ref()
            .map(|examples| examples.resolve_all(spec))
            .unwrap_or_default()
    }
}
