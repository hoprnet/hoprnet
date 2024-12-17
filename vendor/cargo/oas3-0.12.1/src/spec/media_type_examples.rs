use std::collections::BTreeMap;

use log::error;
use serde::{Deserialize, Serialize};

use super::{Example, ObjectOrReference, Spec};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum MediaTypeExamples {
    /// Example of the media type. The example object SHOULD be in the correct format as
    /// specified by the media type. The `example` field is mutually exclusive of the
    /// `examples` field. Furthermore, if referencing a `schema` which contains an example,
    /// the `example` value SHALL override the example provided by the schema.
    Example { example: serde_json::Value },

    /// Examples of the media type. Each example object SHOULD match the media type and
    /// specified schema if present. The `examples` field is mutually exclusive of
    /// the `example` field. Furthermore, if referencing a `schema` which contains an
    /// example, the `examples` value SHALL override the example provided by the schema.
    Examples {
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
    pub fn is_empty(&self) -> bool {
        match self {
            MediaTypeExamples::Example { .. } => false,
            MediaTypeExamples::Examples { examples } => examples.is_empty(),
        }
    }

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
