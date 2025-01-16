//! Schema specification for [OpenAPI 3.0.1](https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md)

use std::{collections::BTreeMap, fmt};

use derive_more::derive::{Display, Error};
use serde::{Deserialize, Deserializer, Serialize};

use super::{spec_extensions, FromRef, ObjectOrReference, Ref, RefError, RefType, Spec};

/// Schema Errors
#[derive(Debug, Clone, PartialEq, Display, Error)]
pub enum Error {
    #[display("Missing type property")]
    NoType,

    #[display("Unknown type: {}", _0)]
    UnknownType(#[error(not(source))] String),

    #[display("Required fields specified on a non-object schema")]
    RequiredSpecifiedOnNonObject,
}

/// Single schema type.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    Boolean,
    Integer,
    Number,
    String,
    Array,
    Object,
    Null,
}

/// Set of schema types.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TypeSet {
    Single(Type),
    Multiple(Vec<Type>),
}

impl TypeSet {
    /// Returns `true` if this type-set contains the given type.
    pub fn contains(&self, type_: Type) -> bool {
        match self {
            TypeSet::Single(single_type) => *single_type == type_,
            TypeSet::Multiple(type_set) => type_set.contains(&type_),
        }
    }

    /// Returns `true` if this type-set is `object` or `[object, 'null']`.
    pub fn is_object_or_nullable_object(&self) -> bool {
        match self {
            TypeSet::Single(Type::Object) => true,
            TypeSet::Multiple(set) if set == &[Type::Object] => true,
            TypeSet::Multiple(set) if set == &[Type::Object, Type::Null] => true,
            TypeSet::Multiple(set) if set == &[Type::Null, Type::Object] => true,
            _ => false,
        }
    }

    /// Returns `true` if this type-set is `array` or `[array, 'null']`.
    pub fn is_array_or_nullable_array(&self) -> bool {
        match self {
            TypeSet::Single(Type::Array) => true,
            TypeSet::Multiple(set) if set == &[Type::Array] => true,
            TypeSet::Multiple(set) if set == &[Type::Array, Type::Null] => true,
            TypeSet::Multiple(set) if set == &[Type::Null, Type::Array] => true,
            _ => false,
        }
    }
}

/// A schema object allows the definition of input and output data types.
///
/// These types can be objects, but also primitives and arrays. This object is an extended subset of
/// the [JSON Schema Specification Wright Draft 00]. For more information about the properties, see
/// [JSON Schema Core] and [JSON Schema Validation]. Unless stated otherwise, the property
/// definitions follow the JSON Schema.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#schema-object> and
/// <https://json-schema.org/draft/2020-12/draft-bhutton-json-schema-01#name-json-schema-documents>.
///
/// [JSON Schema Specification Wright Draft 00]: https://json-schema.org
/// [JSON Schema Core]: https://tools.ietf.org/html/draft-wright-json-schema-00
/// [JSON Schema Validation]: https://tools.ietf.org/html/draft-wright-json-schema-validation-00
#[derive(Clone, Debug, PartialEq, Default, Deserialize, Serialize)]
pub struct ObjectSchema {
    //
    // display metadata
    //
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    //
    // type
    //
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<TypeSet>,

    //
    // structure
    //
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<ObjectOrReference<ObjectSchema>>>,

    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, ObjectOrReference<ObjectSchema>>,

    /// Schema for additional object properties.
    ///
    /// Inline or referenced item MUST be of a [Schema Object] or a boolean.
    ///
    /// See <https://json-schema.org/draft/2020-12/draft-bhutton-json-schema-01#name-additionalproperties>,
    /// <https://json-schema.org/draft/2020-12/draft-bhutton-json-schema-01#name-json-schema-documents>,
    /// and <https://json-schema.org/draft/2020-12/draft-bhutton-json-schema-01#name-boolean-json-schemas>.
    ///
    /// [Schema Object]: https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#schema-object
    #[serde(
        rename = "additionalProperties",
        skip_serializing_if = "Option::is_none"
    )]
    pub additional_properties: Option<Schema>,

    //
    // additional metadata
    //
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,

    #[serde(
        default,
        deserialize_with = "distinguish_missing_and_null",
        skip_serializing_if = "Option::is_none"
    )]
    pub example: Option<serde_json::Value>,

    //
    // validation requirements
    //
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// An instance validates successfully against this if its value is equal to one of the elements
    /// in this array.
    #[serde(rename = "enum", default, skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<String>,

    /// Functionally equivalent to an "enum" with a single value.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-validation#name-const>.
    #[serde(rename = "const", skip_serializing_if = "Option::is_none")]
    pub const_value: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    #[serde(rename = "multipleOf", skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<serde_json::Number>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<serde_json::Number>,

    #[serde(rename = "exclusiveMaximum", skip_serializing_if = "Option::is_none")]
    pub exclusive_maximum: Option<serde_json::Number>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<serde_json::Number>,

    #[serde(rename = "exclusiveMinimum", skip_serializing_if = "Option::is_none")]
    pub exclusive_minimum: Option<serde_json::Number>,

    #[serde(rename = "minLength", skip_serializing_if = "Option::is_none")]
    pub min_length: Option<u64>,

    #[serde(rename = "maxLength", skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u64>,

    #[serde(rename = "minItems", skip_serializing_if = "Option::is_none")]
    pub min_items: Option<u64>,

    #[serde(rename = "maxItems", skip_serializing_if = "Option::is_none")]
    pub max_items: Option<u64>,

    #[serde(rename = "uniqueItems", skip_serializing_if = "Option::is_none")]
    pub unique_items: Option<bool>,

    #[serde(rename = "maxProperties", skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<u64>,

    #[serde(rename = "minProperties", skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<u64>,

    #[serde(rename = "readOnly", skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,

    #[serde(rename = "writeOnly", skip_serializing_if = "Option::is_none")]
    pub write_only: Option<bool>,

    //
    // composition
    //
    #[serde(rename = "allOf", default, skip_serializing_if = "Vec::is_empty")]
    pub all_of: Vec<ObjectOrReference<ObjectSchema>>,

    #[serde(rename = "oneOf", default, skip_serializing_if = "Vec::is_empty")]
    pub one_of: Vec<ObjectOrReference<ObjectSchema>>,

    #[serde(rename = "anyOf", default, skip_serializing_if = "Vec::is_empty")]
    pub any_of: Vec<ObjectOrReference<ObjectSchema>>,

    /// Specification extensions.
    ///
    /// Only "x-" prefixed keys are collected, and the prefix is stripped.
    ///
    /// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#specification-extensions>.
    #[serde(flatten, with = "spec_extensions")]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

impl ObjectSchema {
    /// Returns true if [`Null`](Type::Null) appears in set of schema types, or None if unspecified.
    pub fn is_nullable(&self) -> Option<bool> {
        Some(match self.schema_type.as_ref()? {
            TypeSet::Single(type_) => *type_ == Type::Null,
            TypeSet::Multiple(set) => set.contains(&Type::Null),
        })
    }
}

impl FromRef for ObjectSchema {
    fn from_ref(spec: &Spec, path: &str) -> Result<Self, RefError> {
        let refpath = path.parse::<Ref>()?;

        match refpath.kind {
            RefType::Schema => spec
                .components
                .as_ref()
                .and_then(|cs| cs.schemas.get(&refpath.name))
                .ok_or_else(|| RefError::Unresolvable(path.to_owned()))
                .and_then(|oor| oor.resolve(spec)),

            typ => Err(RefError::MismatchedType(typ, RefType::Schema)),
        }
    }
}

/// A boolean JSON schema.
///
/// See <https://json-schema.org/draft/2020-12/draft-bhutton-json-schema-01#name-boolean-json-schemas>.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct BooleanSchema(pub bool);

/// A JSON schema document.
///
/// See <https://json-schema.org/draft/2020-12/draft-bhutton-json-schema-01#name-json-schema-documents>.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Schema {
    /// A boolean JSON schema.
    Boolean(BooleanSchema),

    /// An object JSON schema.
    Object(Box<ObjectOrReference<ObjectSchema>>),
}

/// Considers any value that is present as `Some`, including `null`.
fn distinguish_missing_and_null<'de, T, D>(de: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de> + fmt::Debug,
    D: Deserializer<'de>,
{
    T::deserialize(de).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_set_contains() {
        let spec = "type: integer";
        let schema = serde_yml::from_str::<ObjectSchema>(spec).unwrap();
        let schema_type = schema.schema_type.unwrap();
        assert!(schema_type.contains(Type::Integer));

        let spec = "type: [integer, 'null']";
        let schema = serde_yml::from_str::<ObjectSchema>(spec).unwrap();
        let schema_type = schema.schema_type.unwrap();
        assert!(schema_type.contains(Type::Integer));

        let spec = "type: [object, 'null']";
        let schema = serde_yml::from_str::<ObjectSchema>(spec).unwrap();
        let schema_type = schema.schema_type.unwrap();
        assert!(schema_type.contains(Type::Object));
        assert!(schema_type.is_object_or_nullable_object());

        let spec = "type: [array]";
        let schema = serde_yml::from_str::<ObjectSchema>(spec).unwrap();
        let schema_type = schema.schema_type.unwrap();
        assert!(schema_type.contains(Type::Array));
        assert!(schema_type.is_array_or_nullable_array());
    }

    #[test]
    fn example_can_be_explicit_null() {
        let spec = indoc::indoc! {"
            type: [string, 'null']
        "};
        let schema = serde_yml::from_str::<ObjectSchema>(spec).unwrap();
        assert_eq!(schema.example, None);

        let spec = indoc::indoc! {"
            type: [string, 'null']
            example: null
        "};
        let schema = serde_yml::from_str::<ObjectSchema>(spec).unwrap();
        assert_eq!(schema.example, Some(serde_json::Value::Null));
    }
}
