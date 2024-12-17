use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{
    spec_extensions, Example, FromRef, MediaType, ObjectOrReference, ObjectSchema, Ref, RefError,
    RefType, Spec,
};

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ParameterIn {
    /// Used together with [path templating], where the parameter value is actually part of the
    /// operation's URL.
    ///
    /// This does not include the host or base path of the API. For example, in `/items/{itemId}`,
    /// the path parameter is `itemId`.
    ///
    /// [path templating]: https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#path-templating
    Path,

    /// Parameters that are appended to the URL. For example, in `/items?id=###`, the query
    /// parameter is `id`.
    Query,

    /// Custom headers that are expected as part of the request.
    ///
    /// Note that [RFC 7230] states header names are case insensitive.
    ///
    /// [RFC 7230]: https://datatracker.ietf.org/doc/html/rfc7230#section-3.2
    Header,

    /// Used to pass a specific cookie value to the API.
    Cookie,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ParameterStyle {
    /// Path-style parameters defined by [RFC 6570].
    ///
    /// Applies to: `primitive, array, object` in `path`.
    ///
    /// [RFC 6570]: https://datatracker.ietf.org/doc/html/rfc6570
    Matrix,

    /// Label style parameters defined by [RFC 6570].
    ///
    /// Applies to: `primitive, array, object` in `path`.
    ///
    /// [RFC 6570]: https://datatracker.ietf.org/doc/html/rfc6570
    Label,

    /// Form style parameters defined by [RFC 6570]. This option replaces collectionFormat with a csv (when explode is false) or multi (when explode is true) value from OpenAPI 2.0..
    ///
    /// Applies to: `primitive, array, object` in `query, cookie`.
    ///
    /// [RFC 6570]: https://datatracker.ietf.org/doc/html/rfc6570
    Form,

    /// Simple style parameters defined by [RFC 6570]. This option replaces collectionFormat with a csv value from OpenAPI 2.0..
    ///
    /// Applies to: `array` in `path, header`.
    ///
    /// [RFC 6570]: https://datatracker.ietf.org/doc/html/rfc6570
    Simple,

    /// Space separated array or object values. This option replaces collectionFormat equal to ssv from OpenAPI 2.0..
    ///
    /// Applies to: `array, object` in `query`.
    SpaceDelimited,

    /// Pipe separated array or object values. This option replaces collectionFormat equal to pipes from OpenAPI 2.0..
    ///
    /// Applies to: `array, object` in `query`.
    PipeDelimited,

    /// Provides a simple way of rendering nested objects using form parameters..
    ///
    /// Applies to: `object` in `query`.
    DeepObject,
}

/// Describes a single operation parameter.
///
/// A unique parameter is defined by a combination of a `name` and location (`in`).
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#parameter-object>.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Parameter {
    /// The name of the parameter.
    pub name: String,

    /// The location of the parameter.
    ///
    /// Given by the `in` field.
    #[serde(rename = "in")]
    pub location: ParameterIn,

    /// A brief description of the parameter.
    ///
    /// This could contain examples of use. CommonMark syntax MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Determines whether this parameter is mandatory.
    ///
    /// If the parameter location is "path", this property is REQUIRED and its value MUST be true.
    /// Otherwise, the property MAY be included and its default value is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,

    /// Specifies that a parameter is deprecated and SHOULD be transitioned out of usage.
    ///
    /// Default value is false.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,

    /// Sets the ability to pass empty-valued parameters.
    ///
    /// This is valid only for query parameters and allows sending a parameter with an empty value.
    /// Default value is false. If style is used, and if behavior is n/a (cannot be serialized), the
    /// value of `allowEmptyValue` SHALL be ignored. Use of this property is NOT RECOMMENDED, as it
    /// is likely to be removed in a later revision.
    #[serde(
        rename = "allowEmptyValue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub allow_empty_value: Option<bool>,

    /// Describes how the parameter value will be serialized depending on the type of the parameter
    /// value.
    ///
    /// Default values (based on value of in): for `query` - `form`; for `path` - `simple`; for
    /// `header` - `simple`; for cookie - `form`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ParameterStyle>,

    /// True if array/object parameter values generate separate parameters for each value of the
    /// array or key-value pair of the map.
    ///
    /// For other types of parameters this property has no effect. When `style` is `form`, the
    /// default value is true. For all other styles, the default value is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explode: Option<bool>,

    /// Determines whether the parameter value SHOULD allow reserved characters to be included
    /// without percent-encoding.
    ///
    /// Reserved characters as defined by [RFC 3986 §2.2]: `:/?#[]@!$&'()*+,;=`. This property only
    /// applies to parameters with an `in` value of `query`. The default value is false.
    ///
    /// [RFC 3986 §2.2]: https://datatracker.ietf.org/doc/html/rfc3986#section-2.2
    #[serde(
        rename = "allowReserved",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub allow_reserved: Option<bool>,

    /// The schema defining the type used for the parameter.
    ///
    /// A parameter MUST contain either a schema property, or a content property, but not both.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<ObjectOrReference<ObjectSchema>>,

    /// Example of the parameter's potential value.
    ///
    /// The example SHOULD match the specified schema and encoding properties if present. The
    /// `example` field is mutually exclusive of the `examples` field. Furthermore, if referencing a
    /// `schema` that contains an example, the `example` value SHALL override the example provided
    /// by the schema. To represent examples of media types that cannot naturally be represented in
    /// JSON or YAML, a string value can contain the example with escaping where necessary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,

    /// Examples of the parameter's potential value.
    ///
    /// Each example SHOULD contain a value in the correct format as specified in the parameter
    /// encoding. The `examples` field is mutually exclusive of the `example` field. Furthermore, if
    /// referencing a `schema` that contains an example, the `examples` value SHALL override the
    /// example provided by the schema.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub examples: BTreeMap<String, ObjectOrReference<Example>>,

    /// A map containing the representations for the parameter.
    ///
    /// A parameter MUST contain either a schema property, or a content property, but not both.
    ///
    /// The key is the media type and the value describes it. The map MUST only contain one entry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<BTreeMap<String, MediaType>>,

    /// Specification extensions.
    ///
    /// Only "x-" prefixed keys are collected, and the prefix is stripped.
    ///
    /// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#specification-extensions>.
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

impl FromRef for Parameter {
    fn from_ref(spec: &Spec, path: &str) -> Result<Self, RefError>
    where
        Self: Sized,
    {
        let refpath = path.parse::<Ref>()?;

        match refpath.kind {
            RefType::Parameter => spec
                .components
                .as_ref()
                .and_then(|cs| cs.parameters.get(&refpath.name))
                .ok_or_else(|| RefError::Unresolvable(path.to_owned()))
                .and_then(|oor| oor.resolve(spec)),

            typ => Err(RefError::MismatchedType(typ, RefType::Parameter)),
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    #[test]
    fn deserialization() {
        let spec = indoc! {"
            name: foo
            in: query
            description: bar
            required: false
            schema:
                type: string
        "};

        let parameter = serde_yml::from_str::<Parameter>(spec).unwrap();
        assert_eq!(parameter.name, "foo");
    }
}
