//! Schema specification for [OpenAPI 3.1](https://spec.openapis.org/oas/v3.1.0)

use std::{collections::BTreeMap, fmt};

use derive_more::derive::{Display, Error};
use serde::{Deserialize, Deserializer, Serialize};

use super::{
    discriminator::Discriminator, spec_extensions, FromRef, ObjectOrReference, Ref, RefError,
    RefType, Spec,
};

/// Schema errors.
#[derive(Debug, Clone, PartialEq, Display, Error)]
pub enum Error {
    /// Missing type field.
    #[display("Missing type field")]
    NoType,

    /// Unknown type.
    #[display("Unknown type: {}", _0)]
    UnknownType(#[error(not(source))] String),

    /// Required property list specified for a non-object schema.
    #[display("Required property list specified for a non-object schema")]
    RequiredSpecifiedOnNonObject,
}

/// Single schema type.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    /// Boolean schema type.
    Boolean,

    /// Integer schema type.
    Integer,

    /// Number schema type.
    Number,

    /// String schema type.
    String,

    /// Array schema type.
    Array,

    /// Object schema type.
    Object,

    /// Null schema type.
    Null,
}

/// Set of schema types.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TypeSet {
    /// Single schema type specified.
    Single(Type),

    /// Multiple possible schema types specified.
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
/// See <https://spec.openapis.org/oas/v3.1.0#schema-object> and
/// <https://json-schema.org/draft/2020-12/draft-bhutton-json-schema-01#name-json-schema-documents>.
///
/// [JSON Schema Specification Wright Draft 00]: https://json-schema.org
/// [JSON Schema Core]: https://tools.ietf.org/html/draft-wright-json-schema-00
/// [JSON Schema Validation]: https://tools.ietf.org/html/draft-wright-json-schema-validation-00
#[derive(Clone, Debug, PartialEq, Default, Deserialize, Serialize)]
pub struct ObjectSchema {
    // #########################################################################
    // Keywords for Applying Subschemas With Logic
    // https://json-schema.org/draft/2020-12/json-schema-core#name-keywords-for-applying-subsch
    // #########################################################################

    //
    /// An instance validates successfully against this keyword if it validates successfully against
    /// all schemas defined by this keyword's value.
    ///
    /// This keyword's value MUST be a non-empty array. Each item of the array MUST be a valid JSON
    /// Schema.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-core#name-allof>.
    #[serde(rename = "allOf", default, skip_serializing_if = "Vec::is_empty")]
    pub all_of: Vec<ObjectOrReference<ObjectSchema>>,

    /// An instance validates successfully against this keyword if it validates successfully against
    /// at least one schema defined by this keyword's value.
    ///
    /// This keyword's value MUST be a non-empty array. Each item of the array MUST be a valid JSON
    /// Schema.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-core#name-anyof>.
    #[serde(rename = "anyOf", default, skip_serializing_if = "Vec::is_empty")]
    pub any_of: Vec<ObjectOrReference<ObjectSchema>>,

    /// An instance validates successfully against this keyword if it validates successfully against
    /// exactly one schema defined by this keyword's value.
    ///
    /// This keyword's value MUST be a non-empty array. Each item of the array MUST be a valid JSON
    /// Schema.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-core#name-oneof>.
    #[serde(rename = "oneOf", default, skip_serializing_if = "Vec::is_empty")]
    pub one_of: Vec<ObjectOrReference<ObjectSchema>>,

    // TODO: missing fields
    // - not

    // #########################################################################
    // TODO: missing concept
    // Keywords for Applying Subschemas Conditionally
    // https://json-schema.org/draft/2020-12/json-schema-core#name-keywords-for-applying-subsche
    // #########################################################################

    // #########################################################################
    // Keywords for Applying Subschemas to Arrays
    // https://json-schema.org/draft/2020-12/json-schema-core#name-keywords-for-applying-subschema
    // #########################################################################

    //
    /// This keyword applies its subschema to all instance array elements.
    ///
    /// Omitting this keyword has the same assertion behavior as an empty schema.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-core#name-items>.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<ObjectOrReference<ObjectSchema>>>,

    // TODO: missing fields
    // - prefixItems
    // - contains

    // #########################################################################
    // Keywords for Applying Subschemas to Objects
    // https://json-schema.org/draft/2020-12/json-schema-core#name-keywords-for-applying-subschemas
    // #########################################################################

    //
    /// Validation succeeds if, for each name that appears in both the instance and as a name within
    /// this keyword's value, the child instance for that name successfully validates against the
    /// corresponding schema.
    ///
    /// Omitting this keyword has the same assertion behavior as an empty object.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-core#name-properties>.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, ObjectOrReference<ObjectSchema>>,

    /// Schema for additional object properties.
    ///
    /// Inline or referenced item MUST be of a [Schema Object] or a boolean.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-core#name-additionalproperties>,
    /// <https://json-schema.org/draft/2020-12/json-schema-core#name-json-schema-documents>, and
    /// <https://json-schema.org/draft/2020-12/json-schema-core#name-boolean-json-schemas>.
    ///
    /// [Schema Object]: https://spec.openapis.org/oas/v3.1.0#schema-object
    #[serde(
        rename = "additionalProperties",
        skip_serializing_if = "Option::is_none"
    )]
    pub additional_properties: Option<Schema>,

    // TODO: missing fields
    // - patternProperties
    // - propertyNames

    // #########################################################################
    // TODO: missing concept
    // A Vocabulary for Unevaluated Locations
    // https://json-schema.org/draft/2020-12/json-schema-core#name-a-vocabulary-for-unevaluate
    // #########################################################################

    // #########################################################################
    // Validation Keywords for Any Instance Type
    // https://json-schema.org/draft/2020-12/json-schema-validation#name-validation-keywords-for-any
    // #########################################################################

    //
    /// Schema type.
    ///
    /// String values MUST be one of the six primitive types ("null", "boolean", "object", "array",
    /// "number", or "string"), or "integer" which matches any number with a zero fractional part.
    ///
    /// If the value of "type" is a string, then an instance validates successfully if its type
    /// matches the type represented by the value of the string. If the value of "type" is an array,
    /// then an instance validates successfully if its type matches any of the types indicated by
    /// the strings in the array.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-validation#name-type>.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<TypeSet>,

    /// An instance validates successfully against this if its value is equal to one of the elements
    /// in this array.
    ///
    /// Elements in the array might be of any type, including null.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-validation#name-enum>.
    #[serde(rename = "enum", default, skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<serde_json::Value>,

    /// Functionally equivalent to an "enum" with a single value.
    ///
    /// The value of this keyword MAY be of any type, including null.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-validation#name-const>.
    #[serde(rename = "const", skip_serializing_if = "Option::is_none")]
    pub const_value: Option<serde_json::Value>,

    // #########################################################################
    // Validation Keywords for Numeric Instances (number and integer)
    // https://json-schema.org/draft/2020-12/json-schema-validation#name-validation-keywords-for-num
    // #########################################################################

    //
    /// A numeric instance is valid only if division by this keyword's value results in an integer.
    ///
    /// The value of "multipleOf" MUST be a number, strictly greater than 0.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-validation#name-multipleof>.
    #[serde(rename = "multipleOf", skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<serde_json::Number>,

    /// If the instance is a number, then this keyword validates only if the instance is less than
    /// or exactly equal to "maximum".
    ///
    /// The value of "maximum" MUST be a number, representing an inclusive upper limit for a numeric
    /// instance.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-validation#name-maximum>.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<serde_json::Number>,

    /// If the instance is a number, then the instance is valid only if it has a value strictly less
    /// than (not equal to) "exclusiveMaximum".
    ///
    /// The value of "exclusiveMaximum" MUST be a number, representing an exclusive upper limit for
    /// a numeric instance.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-validation#name-exclusivemaximum>.
    #[serde(rename = "exclusiveMaximum", skip_serializing_if = "Option::is_none")]
    pub exclusive_maximum: Option<serde_json::Number>,

    /// If the instance is a number, then this keyword validates only if the instance is greater
    /// than or exactly equal to "minimum".
    ///
    /// The value of "minimum" MUST be a number, representing an inclusive lower limit for a numeric
    /// instance.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-validation#name-minimum>.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<serde_json::Number>,

    /// If the instance is a number, then the instance is valid only if it has a value strictly
    /// greater than (not equal to) "exclusiveMinimum".
    ///
    /// The value of "exclusiveMinimum" MUST be a number, representing an exclusive lower limit for
    /// a numeric instance.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-validation#name-exclusiveminimum>.
    #[serde(rename = "exclusiveMinimum", skip_serializing_if = "Option::is_none")]
    pub exclusive_minimum: Option<serde_json::Number>,

    // #########################################################################
    // Validation Keywords for Strings
    // https://json-schema.org/draft/2020-12/json-schema-validation#name-validation-keywords-for-str
    // #########################################################################

    //
    /// A string instance is valid against this keyword if its length is less than, or equal to, the
    /// value of this keyword.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-validation#name-maxlength>.
    #[serde(rename = "maxLength", skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u64>,

    /// A string instance is valid against this keyword if its length is greater than, or equal to,
    /// the value of this keyword.
    ///
    /// Omitting this keyword has the same behavior as a value of 0.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-validation#name-minlength>.
    #[serde(rename = "minLength", skip_serializing_if = "Option::is_none")]
    pub min_length: Option<u64>,

    /// A string instance is considered valid if the regular expression matches the instance
    /// successfully.
    ///
    /// Recall: regular expressions are not implicitly anchored.
    ///
    /// This string SHOULD be a valid regular expression, according to the ECMA-262 regular
    /// expression dialect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    // #########################################################################
    // Validation Keywords for Arrays
    // https://json-schema.org/draft/2020-12/json-schema-validation#name-validation-keywords-for-arr
    // #########################################################################

    //
    /// An array instance is valid against "maxItems" if its size is less than, or equal to, the
    /// value of this keyword.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-validation#name-maxitems>.
    #[serde(rename = "maxItems", skip_serializing_if = "Option::is_none")]
    pub max_items: Option<u64>,

    /// An array instance is valid against "minItems" if its size is greater than, or equal to, the
    /// value of this keyword.
    ///
    /// Omitting this keyword has the same behavior as a value of 0.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-validation#name-minitems>.
    #[serde(rename = "minItems", skip_serializing_if = "Option::is_none")]
    pub min_items: Option<u64>,

    /// True if elements of the array instance must be unique.
    ///
    /// If this keyword has boolean value false, the instance validates successfully. If it has
    /// boolean value true, the instance validates successfully if all of its elements are unique.
    ///
    /// Omitting this keyword has the same behavior as a value of false.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-validation#name-uniqueitems>.
    #[serde(rename = "uniqueItems", skip_serializing_if = "Option::is_none")]
    pub unique_items: Option<bool>,

    // TODO: missing fields
    // - maxContains
    // - minContains

    // #########################################################################
    // Validation Keywords for Objects
    // https://json-schema.org/draft/2020-12/json-schema-validation#name-validation-keywords-for-obj
    // #########################################################################

    //
    /// An object instance is valid against "maxProperties" if its number of properties is less
    /// than, or equal to, the value of this keyword.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-validation#name-maxproperties>.
    #[serde(rename = "maxProperties", skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<u64>,

    /// An object instance is valid against "minProperties" if its number of properties is greater
    /// than, or equal to, the value of this keyword.
    ///
    /// Omitting this keyword has the same behavior as a value of 0.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-validation#name-minproperties>.
    #[serde(rename = "minProperties", skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<u64>,

    /// An object instance is valid against this keyword if every item in the array is the name of a
    /// property in the instance.
    ///
    /// Omitting this keyword has the same behavior as an empty array.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-validation#name-required>.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,

    // TODO: missing fields
    // - dependentRequired

    // #########################################################################
    // Vocabularies for Semantic Content With "format"
    // https://json-schema.org/draft/2020-12/json-schema-validation#name-vocabularies-for-semantic-c
    // #########################################################################

    //
    /// The "format" annotation keyword is defined to allow schema authors to convey semantic
    /// information for a fixed subset of values which are accurately described by authoritative
    /// resources, be they RFCs or other external specifications.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-validation#name-vocabularies-for-semantic-c>.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    // #########################################################################
    // A Vocabulary for Basic Meta-Data Annotations
    // https://json-schema.org/draft/2020-12/json-schema-validation#name-a-vocabulary-for-basic-meta
    // #########################################################################

    //
    /// Schema title.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-validation#name-title-and-description>.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Explains the purpose of the instance described by this schema.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-validation#name-title-and-description>.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Default value associated with a particular schema.
    ///
    /// It is RECOMMENDED that a default value be valid against the associated schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,

    /// If true, indicates that applications SHOULD refrain from usage of the declared property.
    ///
    /// It MAY mean the property is going to be removed in the future. A root schema containing
    /// "deprecated" with a value of true indicates that the entire resource being described MAY be
    /// removed in the future.
    ///
    /// The "deprecated" keyword applies to each instance location to which the schema object
    /// containing the keyword successfully applies. This can result in scenarios where every array
    /// item or object property is deprecated even though the containing array or object is not.
    ///
    /// Omitting this keyword has the same behavior as a value of false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,

    /// If "readOnly" has a value of boolean true, it indicates that the value of the instance is
    /// managed exclusively by the owning authority, and attempts by an application to modify the
    /// value of this property are expected to be ignored or rejected by that owning authority.
    ///
    /// An instance document that is marked as "readOnly" for the entire document MAY be ignored if
    /// sent to the owning authority, or MAY result in an error, at the authority's discretion.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-validation#name-readonly-and-writeonly>.
    #[serde(rename = "readOnly", skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,

    /// If "writeOnly" has a value of boolean true, it indicates that the value is never present
    /// when the instance is retrieved from the owning authority.
    ///
    /// It can be present when sent to the owning authority to update or create the document (or the
    /// resource it represents), but it will not be included in any updated or newly created version
    /// of the instance.
    ///
    /// An instance document that is marked as "writeOnly" for the entire document MAY be returned
    /// as a blank document of some sort, or MAY produce an error upon retrieval, or have the
    /// retrieval request ignored, at the authority's discretion.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-validation#name-readonly-and-writeonly>.
    #[serde(rename = "writeOnly", skip_serializing_if = "Option::is_none")]
    pub write_only: Option<bool>,

    /// This keyword can be used to provide sample JSON values associated with a particular schema,
    /// for the purpose of illustrating usage.
    ///
    /// It is RECOMMENDED that these values be valid against the associated schema.
    ///
    /// See <https://json-schema.org/draft/2020-12/json-schema-validation#name-examples>.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<serde_json::Value>,

    // #########################################################################
    // OpenAPI Fixed Fields
    // https://spec.openapis.org/oas/v3.1.0#fixed-fields-20
    // #########################################################################

    //
    /// Discriminator for object selection based on propertyName
    ///
    /// See <https://spec.openapis.org/oas/v3.1.0#discriminator-object>
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<Discriminator>,

    /// A free-form property to include an example of an instance for this schema.
    ///
    /// To represent examples that cannot be naturally represented in JSON or YAML, a string value
    /// can be used to contain the example with escaping where necessary.
    ///
    /// # Deprecated
    ///
    /// The `example` property has been deprecated in favor of the JSON Schema `examples` keyword.
    /// Use of `example` is discouraged, and later versions of this specification may remove it.
    #[serde(
        default,
        deserialize_with = "distinguish_missing_and_null",
        skip_serializing_if = "Option::is_none"
    )]
    pub example: Option<serde_json::Value>,

    // #########################################################################
    // OpenAPI Other
    // #########################################################################

    //
    /// Specification extensions.
    ///
    /// Only "x-" prefixed keys are collected, and the prefix is stripped.
    ///
    /// See <https://spec.openapis.org/oas/v3.1.0#specification-extensions>.
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

    #[test]
    fn discriminator_example_is_parsed_correctly() {
        let spec = indoc::indoc! {"
          oneOf:
            - $ref: '#/components/schemas/Cat'
            - $ref: '#/components/schemas/Dog'
            - $ref: '#/components/schemas/Lizard'
            - $ref: 'https://gigantic-server.com/schemas/Monster/schema.json'
          discriminator:
            propertyName: petType
            mapping:
              dog: '#/components/schemas/Dog'
              monster: 'https://gigantic-server.com/schemas/Monster/schema.json'
        "};
        let schema = serde_yml::from_str::<ObjectSchema>(spec).unwrap();

        assert!(schema.discriminator.is_some());
        assert_eq!(2, schema.discriminator.unwrap().mapping.unwrap().len());
    }
}
