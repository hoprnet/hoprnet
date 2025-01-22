use std::{
    collections::{BTreeMap, HashMap},
    fmt,
};

use serde::{de, Deserializer, Serializer};

/// Deserializes fields of a map beginning with `x-`.
pub(crate) fn deserialize<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<String, serde_json::Value>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ExtraFieldsVisitor;

    impl<'de> de::Visitor<'de> for ExtraFieldsVisitor {
        type Value = BTreeMap<String, serde_json::Value>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("extensions")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: de::MapAccess<'de>,
        {
            let mut map = HashMap::<serde_yml::Value, serde_json::Value>::new();

            while let Some((key, value)) = access.next_entry()? {
                map.insert(key, value);
            }

            Ok(map
                .into_iter()
                .filter_map(|(key, value)| {
                    key.as_str()?
                        .strip_prefix("x-")
                        .map(|key| (key.to_owned(), value))
                })
                .collect())
        }
    }

    deserializer.deserialize_map(ExtraFieldsVisitor)
}

/// Serializes fields of a map prefixed with `x-`.
pub(crate) fn serialize<S>(
    extensions: &BTreeMap<String, serde_json::Value>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.collect_map(
        extensions
            .iter()
            .map(|(key, value)| (format!("x-{key}"), value)),
    )
}
