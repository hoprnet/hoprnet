//! Schema specification for [OpenAPI 3.1](https://spec.openapis.org/oas/v3.1.0)

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A discriminator object can be used to aid in serialization, deserialization, and validation when
/// payloads may be one of a number of different schemas.
///
/// The discriminator is a specific object in a schema which is used to inform the consumer of the
/// document of an alternative schema based on the value associated with it.
///
/// See <https://spec.openapis.org/oas/v3.1.0#discriminator-object>.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Discriminator {
    /// Name of the property in the payload that will hold the discriminator value.
    pub property_name: String,

    /// Object to hold mappings between payload values and schema names or references.
    ///
    /// When using the discriminator, inline schemas will not be considered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapping: Option<BTreeMap<String, String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminator_property_name_parsed_correctly() {
        let spec = "propertyName: testName";
        let discriminator = serde_yml::from_str::<Discriminator>(spec).unwrap();
        assert_eq!("testName", discriminator.property_name);
        assert!(discriminator.mapping.is_none());
    }

    #[test]
    fn discriminator_mapping_parsed_correctly() {
        let spec = indoc::indoc! {"
            propertyName: petType
            mapping:
              dog: '#/components/schemas/Dog'
              cat: '#/components/schemas/Cat'
              monster: 'https://gigantic-server.com/schemas/Monster/schema.json'
        "};
        let discriminator = serde_yml::from_str::<Discriminator>(spec).unwrap();

        assert_eq!("petType", discriminator.property_name);
        let mapping = discriminator.mapping.unwrap();

        assert_eq!("#/components/schemas/Dog", mapping.get("dog").unwrap());
        assert_eq!("#/components/schemas/Cat", mapping.get("cat").unwrap());
        assert_eq!(
            "https://gigantic-server.com/schemas/Monster/schema.json",
            mapping.get("monster").unwrap()
        );
    }
}
