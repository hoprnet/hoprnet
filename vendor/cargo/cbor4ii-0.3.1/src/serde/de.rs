use crate::alloc::borrow::Cow;
use serde::de::{ self, Visitor };
use crate::core::{ major, marker, types, error };
use crate::core::dec::{ self, Decode };
use crate::util::ScopeGuard;
use crate::serde::error::DecodeError;


pub struct Deserializer<R> {
    reader: R
}

impl<R> Deserializer<R> {
    pub fn new(reader: R) -> Deserializer<R> {
        Deserializer { reader }
    }

    pub fn into_inner(self) -> R {
        self.reader
    }
}

impl<'de, R: dec::Read<'de>> Deserializer<R> {
    #[inline]
    fn try_step(&mut self, name: error::StaticStr) -> Result<ScopeGuard<'_, Self>, dec::Error<R::Error>> {
        if self.reader.step_in() {
            Ok(ScopeGuard(self, |de| de.reader.step_out()))
        } else {
            Err(dec::Error::depth_overflow(name))
        }
    }
}

macro_rules! deserialize_type {
    ( @ $t:ty , $name:ident , $visit:ident ) => {
        #[inline]
        fn $name<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
        {
            let value = <$t>::decode(&mut self.reader)?;
            visitor.$visit(value)
        }
    };
    ( $( $t:ty , $name:ident , $visit:ident );* $( ; )? ) => {
        $(
            deserialize_type!(@ $t, $name, $visit);
        )*
    };
}

impl<'de, 'a, R: dec::Read<'de>> serde::Deserializer<'de> for &'a mut Deserializer<R> {
    type Error = DecodeError<R::Error>;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        let name = &"any";

        let mut de = self.try_step(name)?;
        let de = &mut *de;

        let byte = dec::peek_one(name, &mut de.reader)?;
        match dec::if_major(byte) {
            major::UNSIGNED => de.deserialize_u64(visitor),
            major::NEGATIVE => de.deserialize_i64(visitor),
            major::BYTES => de.deserialize_byte_buf(visitor),
            major::STRING => de.deserialize_string(visitor),
            major::ARRAY => de.deserialize_seq(visitor),
            major::MAP => de.deserialize_map(visitor),
            // NOTE: that this does not support untagged enum.
            // see https://github.com/serde-rs/serde/issues/1682
            major::TAG => match byte {
                0xc2 => de.deserialize_u128(visitor),
                0xc3 => de.deserialize_i128(visitor),
                _ => Err(dec::Error::unsupported(name, byte).into())
            },
            major::SIMPLE => match byte {
                marker::FALSE => {
                    de.reader.advance(1);
                    visitor.visit_bool(false)
                },
                marker::TRUE => {
                    de.reader.advance(1);
                    visitor.visit_bool(true)
                },
                marker::NULL | marker::UNDEFINED => {
                    de.reader.advance(1);
                    visitor.visit_none()
                },
                #[cfg(feature = "half-f16")]
                marker::F16 => {
                    de.reader.advance(1);
                    let v = half::f16::decode(&mut de.reader)?;
                    visitor.visit_f32(v.into())
                },
                marker::F32 => de.deserialize_f32(visitor),
                marker::F64 => de.deserialize_f64(visitor),
                _ => Err(dec::Error::unsupported(name, byte).into())
            },
            _ => Err(dec::Error::unsupported(name, byte).into())
        }
    }

    deserialize_type!(
        bool,       deserialize_bool,       visit_bool;

        i8,         deserialize_i8,         visit_i8;
        i16,        deserialize_i16,        visit_i16;
        i32,        deserialize_i32,        visit_i32;
        i64,        deserialize_i64,        visit_i64;
        i128,       deserialize_i128,       visit_i128;

        u8,         deserialize_u8,         visit_u8;
        u16,        deserialize_u16,        visit_u16;
        u32,        deserialize_u32,        visit_u32;
        u64,        deserialize_u64,        visit_u64;
        u128,       deserialize_u128,       visit_u128;

        f32,        deserialize_f32,        visit_f32;
        f64,        deserialize_f64,        visit_f64;
    );

    #[inline]
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de>
    {
        // Treat it as a String.
        // This is a bit wasteful when encountering strings of more than one character,
        // but we are optimistic this is a cold path.
        self.deserialize_str(visitor)
    }

    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de>
    {
        match <types::Bytes<Cow<[u8]>>>::decode(&mut self.reader)?.0 {
            Cow::Borrowed(buf) => visitor.visit_borrowed_bytes(buf),
            Cow::Owned(buf) => visitor.visit_byte_buf(buf)
        }
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de>
    {
        self.deserialize_bytes(visitor)
    }

    #[inline]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de>
    {
        match <Cow<str>>::decode(&mut self.reader)? {
            Cow::Borrowed(buf) => visitor.visit_borrowed_str(buf),
            Cow::Owned(buf) => visitor.visit_string(buf)
        }
    }

    #[inline]
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de>
    {
        self.deserialize_str(visitor)
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de>
    {
        let name = &"option";

        let byte = dec::peek_one(name, &mut self.reader)?;
        if byte != marker::NULL && byte != marker::UNDEFINED {
            let mut de = self.try_step(name)?;
            visitor.visit_some(&mut *de)
        } else {
            self.reader.advance(1);
            visitor.visit_none()
        }
    }

    #[inline]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de>
    {
        let name = &"unit";

        let byte = dec::pull_one(name, &mut self.reader)?;
        // 0 length array
        if byte == (major::ARRAY << 5) {
            visitor.visit_unit()
        } else {
            Err(dec::Error::mismatch(name, byte).into())
        }
    }

    #[inline]
    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V)
        -> Result<V::Value, Self::Error>
    where V: Visitor<'de>
    {
        self.deserialize_unit(visitor)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V)
        -> Result<V::Value, Self::Error>
    where V: Visitor<'de>
    {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: Visitor<'de>
    {
        let name = &"seq";
        let mut de = self.try_step(name)?;
        let seq = Accessor::array(name, &mut de)?;
        visitor.visit_seq(seq)
    }

    #[inline]
    fn deserialize_tuple<V>(
        self,
        len: usize,
        visitor: V
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        let name = &"tuple";
        let mut de = self.try_step(name)?;
        let seq = Accessor::tuple(name, &mut de, len)?;
        visitor.visit_seq(seq)
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        self.deserialize_tuple(len, visitor)
    }

    #[inline]
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        let name = &"map";
        let mut de = self.try_step(name)?;
        let map = Accessor::map(name, &mut de)?;
        visitor.visit_map(map)
    }

    #[inline]
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V
    ) -> Result<V::Value, Self::Error>
    where V: Visitor<'de>
    {
        self.deserialize_map(visitor)
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        let name = &"enum";
        let mut de = self.try_step(name)?;
        let accessor = EnumAccessor::enum_(name, &mut de)?;
        visitor.visit_enum(accessor)
    }

    #[inline]
    fn deserialize_identifier<V>(self, visitor: V)
        -> Result<V::Value, Self::Error>
    where V: Visitor<'de>
    {
        self.deserialize_str(visitor)
    }

    #[inline]
    fn deserialize_ignored_any<V>(self, visitor: V)
        -> Result<V::Value, Self::Error>
    where V: Visitor<'de>
    {
        let _ignore = dec::IgnoredAny::decode(&mut self.reader)?;
        visitor.visit_unit()
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

struct Accessor<'a, R> {
    de: &'a mut Deserializer<R>,
    len: Option<usize>
}

impl<'de, 'a, R: dec::Read<'de>> Accessor<'a, R> {
    #[inline]
    pub fn array(_name: error::StaticStr, de: &'a mut Deserializer<R>)
        -> Result<Accessor<'a, R>, dec::Error<R::Error>>
    {
        let len = types::Array::len(&mut de.reader)?;
        Ok(Accessor {
            de,
            len,
        })
    }

    #[inline]
    pub fn tuple(name: error::StaticStr, de: &'a mut Deserializer<R>, len: usize)
        -> Result<Accessor<'a, R>, dec::Error<R::Error>>
    {
        let array_len = types::Array::len(&mut de.reader)?;

        if array_len == Some(len) {
            Ok(Accessor {
                de,
                len: array_len,
            })
        } else {
            Err(dec::Error::require_length(name, array_len))
        }
    }

    #[inline]
    pub fn map(_name: error::StaticStr, de: &'a mut Deserializer<R>)
        -> Result<Accessor<'a, R>, dec::Error<R::Error>>
    {
        let len = types::Map::len(&mut de.reader)?;
        Ok(Accessor {
            de,
            len,
        })
    }
}

impl<'de, 'a, R> de::SeqAccess<'de> for Accessor<'a, R>
where
    R: dec::Read<'de>
{
    type Error = DecodeError<R::Error>;

    #[inline]
    fn next_element_seed<T>(&mut self, seed: T)
        -> Result<Option<T::Value>, Self::Error>
    where T: de::DeserializeSeed<'de>
    {
        if let Some(len) = self.len.as_mut() {
            if *len > 0 {
                *len -= 1;
                Ok(Some(seed.deserialize(&mut *self.de)?))
            } else {
                Ok(None)
            }
        } else if dec::is_break(&mut self.de.reader)? {
            Ok(None)
        } else {
            Ok(Some(seed.deserialize(&mut *self.de)?))
        }
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        self.len
    }
}

impl<'de, 'a, R: dec::Read<'de>> de::MapAccess<'de> for Accessor<'a, R> {
    type Error = DecodeError<R::Error>;

    #[inline]
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where K: de::DeserializeSeed<'de>
    {
        if let Some(len) = self.len.as_mut() {
            if *len > 0 {
                *len -= 1;
                Ok(Some(seed.deserialize(&mut *self.de)?))
            } else {
                Ok(None)
            }
        } else if dec::is_break(&mut self.de.reader)? {
            Ok(None)
        } else {
            Ok(Some(seed.deserialize(&mut *self.de)?))
        }
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where V: de::DeserializeSeed<'de>
    {
        seed.deserialize(&mut *self.de)
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        self.len
    }
}

struct EnumAccessor<'a, R> {
    de: &'a mut Deserializer<R>,
}

impl<'de, 'a, R: dec::Read<'de>> EnumAccessor<'a, R> {
    #[inline]
    pub fn enum_(name: error::StaticStr, de: &'a mut Deserializer<R>)
        -> Result<EnumAccessor<'a, R>, dec::Error<R::Error>>
    {
        let byte = dec::peek_one(name, &mut de.reader)?;
        match dec::if_major(byte) {
            // string
            major::STRING => Ok(EnumAccessor { de }),
            // 1 length map
            major::MAP if byte == (major::MAP << 5) | 1 => {
                de.reader.advance(1);
                Ok(EnumAccessor { de })
            },
            _ => Err(dec::Error::mismatch(name, byte))
        }
    }
}

impl<'de, 'a, R> de::EnumAccess<'de> for EnumAccessor<'a, R>
where
    R: dec::Read<'de>
{
    type Error = DecodeError<R::Error>;
    type Variant = EnumAccessor<'a, R>;

    #[inline]
    fn variant_seed<V>(self, seed: V)
        -> Result<(V::Value, Self::Variant), Self::Error>
    where V: de::DeserializeSeed<'de>
    {
        let variant = seed.deserialize(&mut *self.de)?;
        Ok((variant, self))
    }
}

impl<'de, 'a, R> de::VariantAccess<'de> for EnumAccessor<'a, R>
where
    R: dec::Read<'de>
{
    type Error = DecodeError<R::Error>;

    #[inline]
    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where T: de::DeserializeSeed<'de>
    {
        seed.deserialize(&mut *self.de)
    }

    #[inline]
    fn tuple_variant<V>(self, len: usize, visitor: V)
        -> Result<V::Value, Self::Error>
    where V: Visitor<'de>
    {
        use serde::Deserializer;

        self.de.deserialize_tuple(len, visitor)
    }

    #[inline]
    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V
    ) -> Result<V::Value, Self::Error>
    where V: Visitor<'de>
    {
        use serde::Deserializer;

        self.de.deserialize_map(visitor)
    }
}
