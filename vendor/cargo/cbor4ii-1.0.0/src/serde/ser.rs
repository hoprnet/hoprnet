use core::fmt;
use serde::Serialize;
use crate::core::types;
use crate::core::enc::{ self, Encode };
use crate::serde::error::EncodeError;


pub struct Serializer<W> {
    writer: W
}

impl<W> Serializer<W> {
    pub fn new(writer: W) -> Serializer<W> {
        Serializer { writer }
    }

    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<'a, W: enc::Write> serde::Serializer for &'a mut Serializer<W> {
    type Ok = ();
    type Error = EncodeError<W::Error>;

    type SerializeSeq = Collect<'a, W>;
    type SerializeTuple = BoundedCollect<'a, W>;
    type SerializeTupleStruct = BoundedCollect<'a, W>;
    type SerializeTupleVariant = BoundedCollect<'a, W>;
    type SerializeMap = Collect<'a, W>;
    type SerializeStruct = BoundedCollect<'a, W>;
    type SerializeStructVariant = BoundedCollect<'a, W>;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let mut buf = [0; 4];
        self.serialize_str(v.encode_utf8(&mut buf))
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        types::Bytes(v).encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        types::Null.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_some<T: Serialize + ?Sized>(self, value: &T)
        -> Result<Self::Ok, Self::Error>
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        types::Array::bounded(0, &mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str)
        -> Result<Self::Ok, Self::Error>
    {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T
    ) -> Result<Self::Ok, Self::Error> {
        types::Map::bounded(1, &mut self.writer)?;
        variant.encode(&mut self.writer)?;
        value.serialize(self)
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>)
        -> Result<Self::SerializeSeq, Self::Error>
    {
        if let Some(len) = len {
            types::Array::bounded(len, &mut self.writer)?;
        } else {
            types::Array::unbounded(&mut self.writer)?;
        }
        Ok(Collect {
            bounded: len.is_some(),
            ser: self
        })
    }

    #[inline]
    fn serialize_tuple(self, len: usize)
        -> Result<Self::SerializeTuple, Self::Error>
    {
        types::Array::bounded(len, &mut self.writer)?;
        Ok(BoundedCollect { ser: self })
    }

    #[inline]
    fn serialize_tuple_struct(self, _name: &'static str, len: usize)
        -> Result<Self::SerializeTupleStruct, Self::Error>
    {
        self.serialize_tuple(len)
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        types::Map::bounded(1, &mut self.writer)?;
        variant.encode(&mut self.writer)?;
        types::Array::bounded(len, &mut self.writer)?;
        Ok(BoundedCollect { ser: self })
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>)
        -> Result<Self::SerializeMap, Self::Error>
    {
        if let Some(len) = len {
            types::Map::bounded(len, &mut self.writer)?;
        } else {
            types::Map::unbounded(&mut self.writer)?;
        }
        Ok(Collect {
            bounded: len.is_some(),
            ser: self
        })
    }

    #[inline]
    fn serialize_struct(self, _name: &'static str, len: usize)
        -> Result<Self::SerializeStruct, Self::Error>
    {
        types::Map::bounded(len, &mut self.writer)?;
        Ok(BoundedCollect { ser: self })
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        types::Map::bounded(1, &mut self.writer)?;
        variant.encode(&mut self.writer)?;
        types::Map::bounded(len, &mut self.writer)?;
        Ok(BoundedCollect { ser: self })
    }

    #[inline]
    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        v.encode(&mut self.writer)?;
        Ok(())
    }

    #[inline]
    fn collect_str<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: fmt::Display,
    {
        collect_str(&mut self.writer, &value)
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

pub struct Collect<'a, W> {
    bounded: bool,
    ser: &'a mut Serializer<W>
}

pub struct BoundedCollect<'a, W> {
    ser: &'a mut Serializer<W>
}

impl<W: enc::Write> serde::ser::SerializeSeq for Collect<'_, W> {
    type Ok = ();
    type Error = EncodeError<W::Error>;

    #[inline]
    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T)
        -> Result<(), Self::Error>
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        if !self.bounded {
            types::Array::end(&mut self.ser.writer)?;
        }

        Ok(())
    }
}

impl<W: enc::Write> serde::ser::SerializeTuple for BoundedCollect<'_, W> {
    type Ok = ();
    type Error = EncodeError<W::Error>;

    #[inline]
    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T)
        -> Result<(), Self::Error>
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W: enc::Write> serde::ser::SerializeTupleStruct for BoundedCollect<'_, W> {
    type Ok = ();
    type Error = EncodeError<W::Error>;

    #[inline]
    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T)
        -> Result<(), Self::Error>
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W: enc::Write> serde::ser::SerializeTupleVariant for BoundedCollect<'_, W> {
    type Ok = ();
    type Error = EncodeError<W::Error>;

    #[inline]
    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T)
        -> Result<(), Self::Error>
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W: enc::Write> serde::ser::SerializeMap for Collect<'_, W> {
    type Ok = ();
    type Error = EncodeError<W::Error>;

    #[inline]
    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T)
        -> Result<(), Self::Error>
    {
        key.serialize(&mut *self.ser)
    }

    #[inline]
    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T)
        -> Result<(), Self::Error>
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        if !self.bounded {
            types::Map::end(&mut self.ser.writer)?;
        }

        Ok(())
    }
}

impl<W: enc::Write> serde::ser::SerializeStruct for BoundedCollect<'_, W> {
    type Ok = ();
    type Error = EncodeError<W::Error>;

    #[inline]
    fn serialize_field<T: Serialize + ?Sized>(&mut self, key: &'static str, value: &T)
        -> Result<(), Self::Error>
    {
        key.serialize(&mut *self.ser)?;
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W: enc::Write> serde::ser::SerializeStructVariant for BoundedCollect<'_, W> {
    type Ok = ();
    type Error = EncodeError<W::Error>;

    #[inline]
    fn serialize_field<T: Serialize + ?Sized>(&mut self, key: &'static str, value: &T)
        -> Result<(), Self::Error>
    {
        key.serialize(&mut *self.ser)?;
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

fn collect_str<W: enc::Write>(writer: &mut W, value: &dyn fmt::Display)
    -> Result<(), EncodeError<W::Error>>
{
    use core::fmt::Write;
    use serde::ser::Error;

    let mut writer = FmtWriter::new(writer);

    if let Err(err) = write!(&mut writer, "{}", value) {
        if !writer.is_error() {
            return Err(EncodeError::custom(err));
        }
    }

    writer.flush()?;
    Ok(())
}

struct FmtWriter<'a, W: enc::Write> {
    inner: &'a mut W,
    buf: [u8; 255],
    pos: u8,
    state: State<enc::Error<W::Error>>,
}

enum State<E> {
    Short,
    Segment,
    Error(E)
}

impl<W: enc::Write> fmt::Write for FmtWriter<'_, W> {
    #[inline]
    fn write_str(&mut self, input: &str) -> fmt::Result {
        macro_rules! try_ {
            ( $e:expr ) => {
                match $e {
                    Ok(v) => v,
                    Err(err) => {
                        self.state = State::Error(err);
                        return Err(fmt::Error);
                    }
                }
            }
        }

        debug_assert!(usize::from(u8::MAX) >= self.buf.len());

        match self.state {
            State::Short => if usize::from(self.pos) + input.len() > self.buf.len() {
                self.state = State::Segment;
                try_!(types::UncheckedStr::unbounded(self.inner));
            },
            State::Segment => (),
            State::Error(_) => return Err(fmt::Error)
        }

        loop {
            if let Some(buf) = self.buf.get_mut(self.pos.into()..)
                .and_then(|buf| buf.get_mut(..input.len()))
            {
                use core::convert::TryInto;

                buf.copy_from_slice(input.as_bytes());
                let len: u8 = input.len().try_into().unwrap(); // checked by if
                self.pos += len;
                break
            }

            if self.pos > 0 {
                try_!(types::UncheckedStr(&self.buf[..self.pos.into()]).encode(self.inner));
                self.pos = 0;
            }

            if input.len() > self.buf.len() {
                try_!(input.encode(self.inner));
                break
            }
        }

        Ok(())
    }
}

impl<W: enc::Write> FmtWriter<'_, W> {
    #[inline]
    fn new(inner: &mut W) -> FmtWriter<'_, W> {
        FmtWriter {
            inner,
            buf: [0; 255],
            pos: 0,
            state: State::Short,
        }
    }

    #[inline]
    fn is_error(&self) -> bool {
        matches!(self.state, State::Error(_))
    }

    #[inline]
    fn flush(self) -> Result<(), enc::Error<W::Error>> {
        match self.state {
            State::Short | State::Segment => {
                if matches!(self.state, State::Short) || self.pos != 0 {
                    types::UncheckedStr(&self.buf[..self.pos.into()]).encode(self.inner)?;
                }

                if matches!(self.state, State::Segment) {
                    types::UncheckedStr::end(self.inner)?;
                }

                Ok(())
            },
            State::Error(err) => Err(err),
        }
    }
}
