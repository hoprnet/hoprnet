use serde::de::{DeserializeOwned, Error};
use serde::{Deserialize, Deserializer};

/// Deserializes a struct without checking for the fields case sensititivity.
///
/// # **Notes**
///
/// - The following deserializer is incompatible with serde's one. If you wish
/// to use `serde(rename)`, there is a high risk it won't work. Please see
/// <https://github.com/vityafx/serde-aux/issues/8> for further information.
///
/// # Example:
///
/// ```rust
/// use serde_aux::prelude::*;
///
/// #[derive(serde::Deserialize, Debug)]
/// struct AnotherStruct {
///     aaa: String,
/// }
/// #[derive(serde::Deserialize, Debug)]
/// struct MyStruct {
///     #[serde(deserialize_with = "deserialize_struct_case_insensitive")]
///     another_struct: AnotherStruct,
/// }
///
/// let s = r#"{ "another_struct": { "AaA": "Test example" } }"#;
/// let a: MyStruct = serde_json::from_str(s).unwrap();
/// assert_eq!(a.another_struct.aaa, "Test example");
/// ```
pub fn deserialize_struct_case_insensitive<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: DeserializeOwned,
    D: Deserializer<'de>,
{
    use serde_json::Value;

    use std::collections::BTreeMap as Map;

    let map = Map::<String, Value>::deserialize(deserializer)?;
    let lower = map
        .into_iter()
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();
    T::deserialize(Value::Object(lower)).map_err(Error::custom)
}

/// This contains both serialization and ser/deserialization of a enum into and from numbers.
/// The [reference implementation](https://serde.rs/enum-number.html) does not work if your
/// enum has negative values. This `enum_number` handles this also.
///
/// # Example
///
/// ```rust
/// serde_aux::enum_number_declare!(TestEnum {
///     Up = 1,
///     None = 0,
///     Down = -1,
/// });
///
/// let s = r#"1"#;
/// let a: TestEnum = serde_json::from_str(s).unwrap();
/// assert_eq!(a, TestEnum::Up);
///
/// let s = r#"0"#;
/// let a: TestEnum = serde_json::from_str(s).unwrap();
/// assert_eq!(a, TestEnum::None);
///
/// let s = r#"-1"#;
/// let a: TestEnum = serde_json::from_str(s).unwrap();
/// assert_eq!(a, TestEnum::Down);
///
/// let s = r#"5"#;
/// assert!(serde_json::from_str::<TestEnum>(s).is_err());
/// ```
#[macro_export]
macro_rules! enum_number_declare {
    ($name:ident { $($variant:ident = $value:expr, )* }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
        pub enum $name {
            $($variant = $value,)*
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: serde::Deserializer<'de>
            {
                use std::fmt;
                struct Visitor;

                impl<'de> serde::de::Visitor<'de> for Visitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("integer")
                    }

                    fn visit_i64<E>(self, value: i64) -> Result<$name, E>
                        where E: serde::de::Error
                    {
                        // Rust does not come with a simple way of converting a
                        // number to an enum, so use a big `match`.
                        match value {
                            $( $value => Ok($name::$variant), )*
                            _ => Err(E::custom(
                                format!("unknown {} value: {}",
                                stringify!($name), value))),
                        }
                    }

                    fn visit_u64<E>(self, value: u64) -> Result<$name, E>
                        where E: serde::de::Error
                    {
                        self.visit_i64(value as i64)
                    }
                }

                // Deserialize the enum from a i64.
                deserializer.deserialize_i64(Visitor)
            }
        }
    }
}
