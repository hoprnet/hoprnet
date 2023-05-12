use serde::de::{self, Deserialize, Deserializer, Visitor};

/// Gets the serialization names for structs and enums.
///
/// # Example
///
/// ```rust
/// use serde_aux::prelude::*;
///
/// #[derive(serde::Deserialize, Debug)]
/// struct AnotherStruct {
///     #[serde(rename = "a3")]
///     aaa: Option<i64>,
///     #[serde(rename = "b3")]
///     bbb: i128,
///     #[serde(rename = "c3")]
///     ccc: u128,
/// }
/// let fields = serde_introspect::<AnotherStruct>();
/// assert_eq!(fields[0], "a3");
/// assert_eq!(fields[1], "b3");
/// assert_eq!(fields[2], "c3");
///
/// #[derive(serde::Deserialize, Debug)]
/// enum SomeEnum {
///       #[serde(rename = "a")]
///       EnumA,
///       #[serde(rename = "b")]
///       EnumB
/// }
/// let variants = serde_introspect::<SomeEnum>();
/// assert_eq!(variants[0], "a");
/// assert_eq!(variants[1], "b");
/// ```
pub fn serde_introspect<'de, T>() -> &'static [&'static str]
where
    T: Deserialize<'de>,
{
    struct StructFieldsDeserializer<'a> {
        fields: &'a mut Option<&'static [&'static str]>,
    }

    impl<'de, 'a> Deserializer<'de> for StructFieldsDeserializer<'a> {
        type Error = serde::de::value::Error;

        fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(de::Error::custom("I'm just here for the fields"))
        }

        fn deserialize_struct<V>(
            self,
            _name: &'static str,
            fields: &'static [&'static str],
            _visitor: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            *self.fields = Some(fields); // get the names of the deserialized fields
            Err(de::Error::custom("I'm just here for the fields"))
        }

        serde::forward_to_deserialize_any! {
            bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
            byte_buf option unit unit_struct newtype_struct seq tuple
            tuple_struct map enum identifier ignored_any
        }
    }

    struct EnumVariantsDeserializer<'a> {
        variants: &'a mut Option<&'static [&'static str]>,
    }

    impl<'de, 'a> Deserializer<'de> for EnumVariantsDeserializer<'a> {
        type Error = serde::de::value::Error;

        fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(de::Error::custom("I'm just here for the fields"))
        }

        fn deserialize_enum<V>(
            self,
            _name: &'static str,
            variants: &'static [&'static str],
            _visitor: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            *self.variants = Some(variants);
            Err(de::Error::custom("I'm just here for the fields"))
        }

        serde::forward_to_deserialize_any! {
            bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
            byte_buf option unit unit_struct newtype_struct seq tuple
            tuple_struct map struct identifier ignored_any
        }
    }

    let mut serialized_names = None;
    let _ = T::deserialize(EnumVariantsDeserializer {
        variants: &mut serialized_names,
    });
    let _ = T::deserialize(StructFieldsDeserializer {
        fields: &mut serialized_names,
    });
    serialized_names.unwrap_or_default()
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]

    use crate::prelude::serde_introspect;

    #[test]
    fn serde_introspect_given_struct_introspect_serialization_names() {
        #[derive(serde::Deserialize, Debug)]
        enum SomeEnum {
            #[serde(rename = "a")]
            EnumA,
            #[serde(rename = "b")]
            EnumB,
        }
        #[derive(serde::Deserialize, Debug)]
        struct AnotherStruct {
            #[serde(rename = "a3")]
            aaa: Option<i64>,
            #[serde(rename = "b3")]
            bbb: i128,
            #[serde(rename = "c3")]
            ccc: u128,
            #[serde(rename = "d3")]
            ddd: SomeEnum,
        }
        let names = serde_introspect::<AnotherStruct>();
        assert_eq!(names[0], "a3");
        assert_eq!(names[1], "b3");
        assert_eq!(names[2], "c3");
        assert_eq!(names[3], "d3");
    }

    #[test]
    fn serde_introspect_enum_struct_introspect_serialization_names() {
        #[derive(serde::Deserialize, Debug)]
        enum SomeEnum {
            #[serde(rename = "a")]
            EnumA,
            #[serde(rename = "b")]
            EnumB,
        }

        let names = serde_introspect::<SomeEnum>();
        assert_eq!(names[0], "a");
        assert_eq!(names[1], "b");
    }
}
