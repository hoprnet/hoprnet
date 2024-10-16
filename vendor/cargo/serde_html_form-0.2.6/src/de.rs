//! Deserialization support for the `application/x-www-form-urlencoded` format.

use std::io::Read;

use form_urlencoded::{parse, Parse as UrlEncodedParse};
use indexmap::map::{self, IndexMap};
use serde::{
    de::{self, value::MapDeserializer},
    forward_to_deserialize_any,
};

#[doc(inline)]
pub use serde::de::value::Error;

mod part;
mod val_or_vec;

use self::{part::Part, val_or_vec::ValOrVec};

/// Deserializes a `application/x-www-form-urlencoded` value from a `&[u8]`.
///
/// ```
/// let meal = vec![
///     ("bread".to_owned(), "baguette".to_owned()),
///     ("cheese".to_owned(), "comté".to_owned()),
///     ("meat".to_owned(), "ham".to_owned()),
///     ("fat".to_owned(), "butter".to_owned()),
/// ];
///
/// assert_eq!(
///     serde_html_form::from_bytes::<Vec<(String, String)>>(
///         b"bread=baguette&cheese=comt%C3%A9&meat=ham&fat=butter"
///     ),
///     Ok(meal)
/// );
/// ```
pub fn from_bytes<'de, T>(input: &'de [u8]) -> Result<T, Error>
where
    T: de::Deserialize<'de>,
{
    T::deserialize(Deserializer::new(parse(input)))
}

/// Deserializes a `application/x-www-form-urlencoded` value from a `&str`.
///
/// ```
/// let meal = vec![
///     ("bread".to_owned(), "baguette".to_owned()),
///     ("cheese".to_owned(), "comté".to_owned()),
///     ("meat".to_owned(), "ham".to_owned()),
///     ("fat".to_owned(), "butter".to_owned()),
/// ];
///
/// assert_eq!(
///     serde_html_form::from_str::<Vec<(String, String)>>(
///         "bread=baguette&cheese=comt%C3%A9&meat=ham&fat=butter"
///     ),
///     Ok(meal)
/// );
/// ```
pub fn from_str<'de, T>(input: &'de str) -> Result<T, Error>
where
    T: de::Deserialize<'de>,
{
    from_bytes(input.as_bytes())
}

/// Convenience function that reads all bytes from `reader` and deserializes
/// them with `from_bytes`.
pub fn from_reader<T, R>(mut reader: R) -> Result<T, Error>
where
    T: de::DeserializeOwned,
    R: Read,
{
    let mut buf = vec![];
    reader
        .read_to_end(&mut buf)
        .map_err(|e| de::Error::custom(format_args!("could not read input: {}", e)))?;
    from_bytes(&buf)
}

/// A deserializer for the `application/x-www-form-urlencoded` format.
///
/// * Supported top-level outputs are structs, maps and sequences of pairs,
///   with or without a given length.
///
/// * Main `deserialize` methods defers to `deserialize_map`.
///
/// * Everything else but `deserialize_seq` and `deserialize_seq_fixed_size`
///   defers to `deserialize`.
pub struct Deserializer<'de> {
    inner: UrlEncodedParse<'de>,
}

impl<'de> Deserializer<'de> {
    /// Returns a new `Deserializer`.
    pub fn new(parse: UrlEncodedParse<'de>) -> Self {
        Deserializer { inner: parse }
    }
}

impl<'de> de::Deserializer<'de> for Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(MapDeserializer::new(group_entries(self.inner).into_iter()))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(MapDeserializer::new(PartIterator(self.inner)))
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let deserializer = MapDeserializer::new(PartIterator(self.inner));
        deserializer.end()?;
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(self, _name: &str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    forward_to_deserialize_any! {
        bool
        u8
        u16
        u32
        u64
        i8
        i16
        i32
        i64
        f32
        f64
        char
        str
        string
        option
        bytes
        byte_buf
        unit_struct
        tuple_struct
        identifier
        tuple
        enum
        ignored_any
    }
}

struct PartIterator<'de>(UrlEncodedParse<'de>);

impl<'de> Iterator for PartIterator<'de> {
    type Item = (Part<'de>, Part<'de>);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, v)| (Part(k), Part(v)))
    }
}

fn group_entries(parse: UrlEncodedParse<'_>) -> IndexMap<Part<'_>, ValOrVec<Part<'_>>> {
    use map::Entry::*;

    let mut res = IndexMap::new();

    for (key, value) in parse {
        match res.entry(Part(key)) {
            Vacant(v) => {
                v.insert(ValOrVec::Val(Part(value)));
            }
            Occupied(mut o) => {
                o.get_mut().push(Part(value));
            }
        }
    }

    res
}

#[cfg(test)]
mod tests;
