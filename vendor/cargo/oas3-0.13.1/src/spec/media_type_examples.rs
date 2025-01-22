use std::collections::BTreeMap;

use log::error;
use serde::{Deserialize, Serialize};

use super::{Example, ObjectOrReference, Spec};

/// Examples for a media type.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MediaTypeExamples {
    /// Example of the media type.
    ///
    /// The example object SHOULD be in the correct format as specified by the media type. The
    /// `example` field is mutually exclusive of the `examples` field. Furthermore, if referencing a
    /// `schema` which contains an example, the `example` value SHALL override the example provided
    /// by the schema.
    Example {
        /// Example of the media type.
        example: serde_json::Value,
    },

    /// Examples of the media type.
    ///
    /// Each example object SHOULD match the media type and specified schema if present. The
    /// `examples` field is mutually exclusive of the `example` field. Furthermore, if referencing a
    /// `schema` which contains an example, the `examples` value SHALL override the example provided
    /// by the schema.
    Examples {
        /// Examples of the media type.
        examples: BTreeMap<String, ObjectOrReference<Example>>,
    },
}

impl Default for MediaTypeExamples {
    fn default() -> Self {
        MediaTypeExamples::Examples {
            examples: BTreeMap::new(),
        }
    }
}

impl MediaTypeExamples {
    /// Returns true if no examples are provided.
    pub fn is_empty(&self) -> bool {
        match self {
            MediaTypeExamples::Example { .. } => false,
            MediaTypeExamples::Examples { examples } => examples.is_empty(),
        }
    }

    /// Resolves references and returns a map of provided examples keyed by name.
    ///
    /// If the `example` field is provided, then the map contains one item, keyed as "default".
    pub fn resolve_all(&self, spec: &Spec) -> BTreeMap<String, Example> {
        match self {
            Self::Example { example } => {
                let example = Example {
                    description: None,
                    summary: None,
                    value: Some(example.clone()),
                    extensions: BTreeMap::default(),
                };

                let mut map = BTreeMap::new();
                map.insert("default".to_owned(), example);

                map
            }

            Self::Examples { examples } => examples
                .iter()
                .filter_map(|(name, oor)| {
                    oor.resolve(spec)
                        .map(|obj| (name.clone(), obj))
                        .map_err(|err| error!("{}", err))
                        .ok()
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn is_empty() {
        assert!(MediaTypeExamples::default().is_empty());

        assert!(!MediaTypeExamples::Example {
            example: serde_json::Value::Null
        }
        .is_empty());
    }

    #[test]
    fn deserialize() {
        assert_eq!(
            serde_yml::from_str::<MediaTypeExamples>(indoc::indoc! {"
                    examples: {}
                "})
            .unwrap(),
            MediaTypeExamples::default(),
        );

        assert_eq!(
            serde_yml::from_str::<MediaTypeExamples>(indoc::indoc! {"
                    example: null
                "})
            .unwrap(),
            MediaTypeExamples::Example {
                example: json!(null)
            },
        );
    }

    #[test]
    fn serialize() {
        let mt_examples = MediaTypeExamples::default();
        assert_eq!(
            serde_yml::to_string(&mt_examples).unwrap(),
            indoc::indoc! {"
                examples: {}
            "},
        );

        let mt_examples = MediaTypeExamples::Example {
            example: json!(null),
        };
        assert_eq!(
            serde_yml::to_string(&mt_examples).unwrap(),
            indoc::indoc! {"
                example: null
            "},
        );
    }

    #[test]
    fn single_example() {
        let spec = serde_yml::from_str::<Spec>(indoc::indoc! {"
openapi: 3.1.0
info:
  title: Mascots
  version: 0.0.0
paths: {}
        "})
        .unwrap();

        let mt_examples = MediaTypeExamples::Example {
            example: json!("ferris"),
        };

        let examples = mt_examples.resolve_all(&spec);

        // no example with this name defined
        assert_eq!(examples.get("Go"), None);

        // single examples are put in "default" key of returned map
        assert_eq!(
            examples.get("default").unwrap().value.as_ref().unwrap(),
            &json!("ferris"),
        );
    }

    #[test]
    fn resolve_references() {
        let spec = serde_yml::from_str::<Spec>(indoc::indoc! {"
openapi: 3.1.0
info:
  title: Mascots
  version: 0.0.0
paths: {}
components:
    examples:
        RustMascot:
            value: ferris
        "})
        .unwrap();

        let mt_examples = MediaTypeExamples::Examples {
            examples: BTreeMap::from([
                (
                    "Rust".to_owned(),
                    ObjectOrReference::Ref {
                        ref_path: "#/components/examples/RustMascot".to_owned(),
                    },
                ),
                (
                    "Go".to_owned(),
                    ObjectOrReference::Ref {
                        ref_path: "#/components/examples/GoMascot".to_owned(),
                    },
                ),
            ]),
        };

        let examples = mt_examples.resolve_all(&spec);

        // reference points to non-existent component
        assert_eq!(examples.get("Go"), None);

        // reference points to valid example component
        assert_eq!(
            examples.get("Rust").unwrap().value.as_ref().unwrap(),
            &json!("ferris"),
        );
    }
}
