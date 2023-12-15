use crate::{std::marker::PhantomData, Error};

#[cfg(feature = "alloc")]
use crate::{
    std::{boxed::Box, mem, vec::Vec},
    BinaryBuf, TextBuf,
};

use sval_ref::ValueRef as _;

#[cfg(feature = "alloc")]
pub use alloc_support::*;

/**
Buffer arbitrary values into a tree-like structure.

This type requires the `alloc` or `std` features, otherwise most methods
will fail.
*/
#[derive(Debug)]
pub struct ValueBuf<'sval> {
    #[cfg(feature = "alloc")]
    parts: Vec<ValuePart<'sval>>,
    #[cfg(feature = "alloc")]
    stack: Vec<usize>,
    #[cfg(feature = "alloc")]
    is_in_text_or_binary: bool,
    err: Option<Error>,
    _marker: PhantomData<&'sval ()>,
}

/**
An immutable buffered value.

This type is more compact than `ValueBuf`.
*/
#[derive(Debug, Clone)]
pub struct Value<'sval> {
    #[cfg(feature = "alloc")]
    parts: Box<[ValuePart<'sval>]>,
    _marker: PhantomData<&'sval ()>,
}

impl<'sval> Default for ValueBuf<'sval> {
    fn default() -> Self {
        ValueBuf::new()
    }
}

impl<'sval> ValueBuf<'sval> {
    /**
    Create a new empty value buffer.
    */
    pub fn new() -> Self {
        ValueBuf {
            #[cfg(feature = "alloc")]
            parts: Vec::new(),
            #[cfg(feature = "alloc")]
            stack: Vec::new(),
            #[cfg(feature = "alloc")]
            is_in_text_or_binary: false,
            err: None,
            _marker: PhantomData,
        }
    }

    /**
    Buffer a value.

    This method will fail if the `alloc` feature is not enabled.
    */
    pub fn collect(v: &'sval (impl sval::Value + ?Sized)) -> Result<Self, Error> {
        let mut buf = ValueBuf::new();

        match v.stream(&mut buf) {
            Ok(()) => Ok(buf),
            Err(_) => Err(buf.into_err()),
        }
    }

    /**
    Whether or not the contents of the value buffer are complete.
    */
    pub fn is_complete(&self) -> bool {
        #[cfg(feature = "alloc")]
        {
            self.stack.len() == 0 && self.parts.len() > 0 && !self.is_in_text_or_binary
        }
        #[cfg(not(feature = "alloc"))]
        {
            true
        }
    }

    /**
    Clear this buffer so it can be re-used for future values.
    */
    pub fn clear(&mut self) {
        #[cfg(feature = "alloc")]
        {
            let ValueBuf {
                parts,
                stack,
                is_in_text_or_binary,
                err,
                _marker,
            } = self;

            parts.clear();
            stack.clear();
            *is_in_text_or_binary = false;
            *err = None;
        }
        #[cfg(not(feature = "alloc"))]
        {
            let ValueBuf { err, _marker } = self;

            *err = None;
        }
    }

    /**
    Get an independent immutable value from this buffer.
    */
    pub fn to_value(&self) -> Value<'sval> {
        Value {
            #[cfg(feature = "alloc")]
            parts: self.parts.clone().into_boxed_slice(),
            _marker: PhantomData,
        }
    }

    /**
    Convert this buffer into an immutable value.
    */
    pub fn into_value(self) -> Value<'sval> {
        Value {
            #[cfg(feature = "alloc")]
            parts: self.parts.into_boxed_slice(),
            _marker: PhantomData,
        }
    }

    /**
    Fully buffer any borrowed data, returning a buffer that doesn't borrow anything.

    This method will fail if the `alloc` feature is not enabled.
    */
    pub fn into_owned(self) -> Result<ValueBuf<'static>, Error> {
        #[cfg(feature = "alloc")]
        {
            let ValueBuf {
                mut parts,
                mut stack,
                mut is_in_text_or_binary,
                mut err,
                _marker,
            } = self;

            // Re-assign all parts within the value in-place without re-allocating for them
            // This will take care of converted any actually borrowed data into owned
            for part in &mut parts {
                crate::assert_static(part.into_owned_in_place());
            }

            // SAFETY: `parts` no longer contains any data borrowed for `'sval`
            let mut parts =
                unsafe { mem::transmute::<Vec<ValuePart<'sval>>, Vec<ValuePart<'static>>>(parts) };

            crate::assert_static(&mut parts);
            crate::assert_static(&mut stack);
            crate::assert_static(&mut is_in_text_or_binary);
            crate::assert_static(&mut err);

            Ok(ValueBuf {
                parts,
                stack,
                is_in_text_or_binary,
                err,
                _marker: PhantomData,
            })
        }
        #[cfg(not(feature = "alloc"))]
        {
            Err(Error::no_alloc("buffered value"))
        }
    }

    #[cfg(feature = "alloc")]
    fn try_catch(
        &mut self,
        f: impl FnOnce(&mut ValueBuf<'sval>) -> Result<(), Error>,
    ) -> sval::Result {
        match f(self) {
            Ok(()) => Ok(()),
            Err(e) => self.fail(e),
        }
    }

    fn fail(&mut self, err: Error) -> sval::Result {
        self.err = Some(err);
        sval::error()
    }

    fn into_err(self) -> Error {
        self.err
            .unwrap_or_else(|| Error::invalid_value("the value itself failed to stream"))
    }
}

impl ValueBuf<'static> {
    /**
    Fully buffer a value, including any internal borrowed data.

    This method will fail if the `alloc` feature is not enabled.
    */
    pub fn collect_owned(v: impl sval::Value) -> Result<Self, Error> {
        let mut buf = ValueBuf::new();

        // Buffering the value as computed means any borrowed data will
        // have to be converted into owned anyways
        match sval::stream_computed(&mut buf, v) {
            Ok(()) => Ok(buf),
            Err(_) => Err(buf.into_err()),
        }
    }
}

impl<'sval> Value<'sval> {
    /**
    Buffer a value.

    This method will fail if the `alloc` feature is not enabled.
    */
    pub fn collect(v: &'sval (impl sval::Value + ?Sized)) -> Result<Self, Error> {
        ValueBuf::collect(v).map(|buf| buf.into_value())
    }

    /**
    Fully buffer this value, including any internal borrowed data.

    This method will fail if the `alloc` feature is not enabled.
    */
    pub fn into_owned(self) -> Result<Value<'static>, Error> {
        #[cfg(feature = "alloc")]
        {
            let Value { mut parts, _marker } = self;

            // Re-assign all parts within the value in-place without re-allocating for them
            // This will take care of converted any actually borrowed data into owned
            for part in &mut *parts {
                crate::assert_static(part.into_owned_in_place());
            }

            // SAFETY: `parts` no longer contains any data borrowed for `'sval`
            let mut parts = unsafe {
                mem::transmute::<Box<[ValuePart<'sval>]>, Box<[ValuePart<'static>]>>(parts)
            };
            crate::assert_static(&mut parts);

            Ok(Value {
                parts,
                _marker: PhantomData,
            })
        }
        #[cfg(not(feature = "alloc"))]
        {
            Err(Error::no_alloc("buffered value"))
        }
    }
}

impl Value<'static> {
    /**
    Fully buffer a value, including any internal borrowed data.

    This method will fail if the `alloc` feature is not enabled.
    */
    pub fn collect_owned(v: impl sval::Value) -> Result<Self, Error> {
        ValueBuf::collect_owned(v).map(|buf| buf.into_value())
    }
}

impl<'a> sval::Value for ValueBuf<'a> {
    fn stream<'sval, S: sval::Stream<'sval> + ?Sized>(&'sval self, stream: &mut S) -> sval::Result {
        self.stream_ref(stream)
    }
}

impl<'sval> sval_ref::ValueRef<'sval> for ValueBuf<'sval> {
    fn stream_ref<S: sval::Stream<'sval> + ?Sized>(&self, stream: &mut S) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            stream_ref(&self.parts, stream)
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = stream;
            sval::error()
        }
    }
}

impl<'a> sval::Value for Value<'a> {
    fn stream<'sval, S: sval::Stream<'sval> + ?Sized>(&'sval self, stream: &mut S) -> sval::Result {
        self.stream_ref(stream)
    }
}

impl<'sval> sval_ref::ValueRef<'sval> for Value<'sval> {
    fn stream_ref<S: sval::Stream<'sval> + ?Sized>(&self, stream: &mut S) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            stream_ref(&self.parts, stream)
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = stream;
            sval::error()
        }
    }
}

#[cfg(feature = "alloc")]
fn stream_ref<'a, 'sval, S: sval::Stream<'sval> + ?Sized>(
    parts: &'a [ValuePart<'sval>],
    stream: &mut S,
) -> sval::Result {
    // If the buffer is empty then stream null
    if parts.len() == 0 {
        return stream.null();
    }

    ValueSlice::new(parts).stream_ref(stream)
}

impl<'sval> sval::Stream<'sval> for ValueBuf<'sval> {
    fn null(&mut self) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_kind(ValueKind::Null);

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn bool(&mut self, value: bool) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_kind(ValueKind::Bool(value));

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = value;
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn text_begin(&mut self, _: Option<usize>) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_kind(ValueKind::Text(TextBuf::new()));
            self.is_in_text_or_binary = true;

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn text_fragment(&mut self, fragment: &'sval str) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.try_catch(|buf| match buf.current_mut().kind {
                ValueKind::Text(ref mut text) => text.push_fragment(fragment),
                _ => Err(Error::outside_container("text")),
            })
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = fragment;
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn text_fragment_computed(&mut self, fragment: &str) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.try_catch(|buf| match buf.current_mut().kind {
                ValueKind::Text(ref mut text) => text.push_fragment_computed(fragment),
                _ => Err(Error::outside_container("text")),
            })
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = fragment;
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn text_end(&mut self) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.is_in_text_or_binary = false;

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn binary_begin(&mut self, _: Option<usize>) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_kind(ValueKind::Binary(BinaryBuf::new()));
            self.is_in_text_or_binary = true;

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn binary_fragment(&mut self, fragment: &'sval [u8]) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.try_catch(|buf| match buf.current_mut().kind {
                ValueKind::Binary(ref mut binary) => binary.push_fragment(fragment),
                _ => Err(Error::outside_container("binary")),
            })
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = fragment;
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn binary_fragment_computed(&mut self, fragment: &[u8]) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.try_catch(|buf| match buf.current_mut().kind {
                ValueKind::Binary(ref mut binary) => binary.push_fragment_computed(fragment),
                _ => Err(Error::outside_container("binary")),
            })
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = fragment;
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn binary_end(&mut self) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.is_in_text_or_binary = false;
            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn u8(&mut self, value: u8) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_kind(ValueKind::U8(value));

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = value;
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn u16(&mut self, value: u16) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_kind(ValueKind::U16(value));

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = value;
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn u32(&mut self, value: u32) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_kind(ValueKind::U32(value));

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = value;
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn u64(&mut self, value: u64) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_kind(ValueKind::U64(value));

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = value;
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn u128(&mut self, value: u128) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_kind(ValueKind::U128(value));

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = value;
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn i8(&mut self, value: i8) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_kind(ValueKind::I8(value));

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = value;
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn i16(&mut self, value: i16) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_kind(ValueKind::I16(value));

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = value;
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn i32(&mut self, value: i32) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_kind(ValueKind::I32(value));

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = value;
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn i64(&mut self, value: i64) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_kind(ValueKind::I64(value));

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = value;
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn i128(&mut self, value: i128) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_kind(ValueKind::I128(value));

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = value;
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn f32(&mut self, value: f32) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_kind(ValueKind::F32(value));

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = value;
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn f64(&mut self, value: f64) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_kind(ValueKind::F64(value));

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = value;
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn map_begin(&mut self, num_entries_hint: Option<usize>) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_begin(ValueKind::Map {
                len: 0,
                num_entries_hint,
            });

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = num_entries_hint;
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn map_key_begin(&mut self) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_begin(ValueKind::MapKey { len: 0 });

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn map_key_end(&mut self) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.try_catch(|buf| buf.push_end())
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn map_value_begin(&mut self) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_begin(ValueKind::MapValue { len: 0 });

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn map_value_end(&mut self) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.try_catch(|buf| buf.push_end())
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn map_end(&mut self) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.try_catch(|buf| buf.push_end())
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn seq_begin(&mut self, num_entries_hint: Option<usize>) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_begin(ValueKind::Seq {
                len: 0,
                num_entries_hint,
            });

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = num_entries_hint;
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn seq_value_begin(&mut self) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_begin(ValueKind::SeqValue { len: 0 });

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn seq_value_end(&mut self) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.try_catch(|buf| buf.push_end())
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn seq_end(&mut self) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.try_catch(|buf| buf.push_end())
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn enum_begin(
        &mut self,
        tag: Option<&sval::Tag>,
        label: Option<&sval::Label>,
        index: Option<&sval::Index>,
    ) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_begin(ValueKind::Enum {
                len: 0,
                tag: tag.cloned(),
                index: index.cloned(),
                label: label.map(|label| label.to_owned()),
            });

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = (tag, label, index);
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn enum_end(
        &mut self,
        _: Option<&sval::Tag>,
        _: Option<&sval::Label>,
        _: Option<&sval::Index>,
    ) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.try_catch(|buf| buf.push_end())
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn tagged_begin(
        &mut self,
        tag: Option<&sval::Tag>,
        label: Option<&sval::Label>,
        index: Option<&sval::Index>,
    ) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_begin(ValueKind::Tagged {
                len: 0,
                tag: tag.cloned(),
                index: index.cloned(),
                label: label.map(|label| label.to_owned()),
            });

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = (tag, label, index);
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn tagged_end(
        &mut self,
        _: Option<&sval::Tag>,
        _: Option<&sval::Label>,
        _: Option<&sval::Index>,
    ) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.try_catch(|buf| buf.push_end())
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn tag(
        &mut self,
        tag: Option<&sval::Tag>,
        label: Option<&sval::Label>,
        index: Option<&sval::Index>,
    ) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_kind(ValueKind::Tag {
                tag: tag.cloned(),
                index: index.cloned(),
                label: label.map(|label| label.to_owned()),
            });

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = (tag, label, index);
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn record_begin(
        &mut self,
        tag: Option<&sval::Tag>,
        label: Option<&sval::Label>,
        index: Option<&sval::Index>,
        num_entries: Option<usize>,
    ) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_begin(ValueKind::Record {
                len: 0,
                tag: tag.cloned(),
                index: index.cloned(),
                label: label.map(|label| label.to_owned()),
                num_entries,
            });

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = (tag, label, index, num_entries);
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn record_value_begin(&mut self, tag: Option<&sval::Tag>, label: &sval::Label) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_begin(ValueKind::RecordValue {
                len: 0,
                tag: tag.cloned(),
                label: label.to_owned(),
            });

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = tag;
            let _ = label;
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn record_value_end(&mut self, _: Option<&sval::Tag>, _: &sval::Label) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.try_catch(|buf| buf.push_end())
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn record_end(
        &mut self,
        _: Option<&sval::Tag>,
        _: Option<&sval::Label>,
        _: Option<&sval::Index>,
    ) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.try_catch(|buf| buf.push_end())
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn tuple_begin(
        &mut self,
        tag: Option<&sval::Tag>,
        label: Option<&sval::Label>,
        index: Option<&sval::Index>,
        num_entries: Option<usize>,
    ) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_begin(ValueKind::Tuple {
                len: 0,
                tag: tag.cloned(),
                index: index.cloned(),
                label: label.map(|label| label.to_owned()),
                num_entries,
            });

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = (tag, label, index, num_entries);
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn tuple_value_begin(&mut self, tag: Option<&sval::Tag>, index: &sval::Index) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_begin(ValueKind::TupleValue {
                len: 0,
                tag: tag.cloned(),
                index: index.clone(),
            });

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = tag;
            let _ = index;
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn tuple_value_end(&mut self, _: Option<&sval::Tag>, _: &sval::Index) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.try_catch(|buf| buf.push_end())
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn tuple_end(
        &mut self,
        _: Option<&sval::Tag>,
        _: Option<&sval::Label>,
        _: Option<&sval::Index>,
    ) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.try_catch(|buf| buf.push_end())
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn record_tuple_begin(
        &mut self,
        tag: Option<&sval::Tag>,
        label: Option<&sval::Label>,
        index: Option<&sval::Index>,
        num_entries: Option<usize>,
    ) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_begin(ValueKind::RecordTuple {
                len: 0,
                tag: tag.cloned(),
                index: index.cloned(),
                label: label.map(|label| label.to_owned()),
                num_entries,
            });

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = (tag, label, index, num_entries);
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn record_tuple_value_begin(
        &mut self,
        tag: Option<&sval::Tag>,
        label: &sval::Label,
        index: &sval::Index,
    ) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.push_begin(ValueKind::RecordTupleValue {
                len: 0,
                tag: tag.cloned(),
                label: label.to_owned(),
                index: index.clone(),
            });

            Ok(())
        }
        #[cfg(not(feature = "alloc"))]
        {
            let _ = tag;
            let _ = label;
            let _ = index;
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn record_tuple_value_end(
        &mut self,
        _: Option<&sval::Tag>,
        _: &sval::Label,
        _: &sval::Index,
    ) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.try_catch(|buf| buf.push_end())
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.fail(Error::no_alloc("buffered value"))
        }
    }

    fn record_tuple_end(
        &mut self,
        _: Option<&sval::Tag>,
        _: Option<&sval::Label>,
        _: Option<&sval::Index>,
    ) -> sval::Result {
        #[cfg(feature = "alloc")]
        {
            self.try_catch(|buf| buf.push_end())
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.fail(Error::no_alloc("buffered value"))
        }
    }
}

#[cfg(feature = "alloc")]
mod alloc_support {
    use super::*;

    use crate::{
        std::{mem, ops::Range},
        BinaryBuf, TextBuf,
    };

    /**
    Buffer a value.
    */
    pub fn stream_to_value<'sval>(
        v: &'sval (impl sval::Value + ?Sized),
    ) -> Result<ValueBuf<'sval>, Error> {
        ValueBuf::collect(v)
    }

    /**
    Buffer an owned value.
    */
    pub fn stream_to_value_owned(v: impl sval::Value) -> Result<ValueBuf<'static>, Error> {
        ValueBuf::collect_owned(v)
    }

    #[repr(transparent)]
    pub(super) struct ValueSlice<'sval>([ValuePart<'sval>]);

    #[derive(Debug, Clone, PartialEq)]
    pub(super) struct ValuePart<'sval> {
        pub(super) kind: ValueKind<'sval>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub(super) enum ValueKind<'sval> {
        Null,
        Bool(bool),
        U8(u8),
        U16(u16),
        U32(u32),
        U64(u64),
        U128(u128),
        I8(i8),
        I16(i16),
        I32(i32),
        I64(i64),
        I128(i128),
        F32(f32),
        F64(f64),
        Text(TextBuf<'sval>),
        Binary(BinaryBuf<'sval>),
        Map {
            len: usize,
            num_entries_hint: Option<usize>,
        },
        MapKey {
            len: usize,
        },
        MapValue {
            len: usize,
        },
        Seq {
            len: usize,
            num_entries_hint: Option<usize>,
        },
        SeqValue {
            len: usize,
        },
        Tag {
            tag: Option<sval::Tag>,
            label: Option<sval::Label<'static>>,
            index: Option<sval::Index>,
        },
        Enum {
            len: usize,
            tag: Option<sval::Tag>,
            label: Option<sval::Label<'static>>,
            index: Option<sval::Index>,
        },
        Tagged {
            len: usize,
            tag: Option<sval::Tag>,
            label: Option<sval::Label<'static>>,
            index: Option<sval::Index>,
        },
        Record {
            len: usize,
            tag: Option<sval::Tag>,
            label: Option<sval::Label<'static>>,
            index: Option<sval::Index>,
            num_entries: Option<usize>,
        },
        RecordValue {
            len: usize,
            tag: Option<sval::Tag>,
            label: sval::Label<'static>,
        },
        Tuple {
            len: usize,
            tag: Option<sval::Tag>,
            label: Option<sval::Label<'static>>,
            index: Option<sval::Index>,
            num_entries: Option<usize>,
        },
        TupleValue {
            len: usize,
            tag: Option<sval::Tag>,
            index: sval::Index,
        },
        RecordTuple {
            len: usize,
            tag: Option<sval::Tag>,
            label: Option<sval::Label<'static>>,
            index: Option<sval::Index>,
            num_entries: Option<usize>,
        },
        RecordTupleValue {
            len: usize,
            tag: Option<sval::Tag>,
            label: sval::Label<'static>,
            index: sval::Index,
        },
    }

    impl<'sval> ValueBuf<'sval> {
        pub(super) fn push_kind(&mut self, kind: ValueKind<'sval>) {
            self.parts.push(ValuePart { kind });
        }

        pub(super) fn push_begin(&mut self, kind: ValueKind<'sval>) {
            self.stack.push(self.parts.len());
            self.parts.push(ValuePart { kind });
        }

        pub(super) fn push_end(&mut self) -> Result<(), Error> {
            let index = self
                .stack
                .pop()
                .ok_or_else(|| Error::invalid_value("unbalanced calls to `begin` and `end`"))?;

            let len = self.parts.len() - index - 1;

            *match &mut self.parts[index].kind {
                ValueKind::Map { len, .. } => len,
                ValueKind::MapKey { len } => len,
                ValueKind::MapValue { len } => len,
                ValueKind::Seq { len, .. } => len,
                ValueKind::SeqValue { len } => len,
                ValueKind::Enum { len, .. } => len,
                ValueKind::Tagged { len, .. } => len,
                ValueKind::Record { len, .. } => len,
                ValueKind::RecordValue { len, .. } => len,
                ValueKind::Tuple { len, .. } => len,
                ValueKind::TupleValue { len, .. } => len,
                ValueKind::RecordTuple { len, .. } => len,
                ValueKind::RecordTupleValue { len, .. } => len,
                ValueKind::Null
                | ValueKind::Bool(_)
                | ValueKind::U8(_)
                | ValueKind::U16(_)
                | ValueKind::U32(_)
                | ValueKind::U64(_)
                | ValueKind::U128(_)
                | ValueKind::I8(_)
                | ValueKind::I16(_)
                | ValueKind::I32(_)
                | ValueKind::I64(_)
                | ValueKind::I128(_)
                | ValueKind::F32(_)
                | ValueKind::F64(_)
                | ValueKind::Text(_)
                | ValueKind::Binary(_)
                | ValueKind::Tag { .. } => {
                    return Err(Error::invalid_value("can't end at this index"))
                }
            } = len;

            Ok(())
        }

        pub(super) fn current_mut(&mut self) -> &mut ValuePart<'sval> {
            self.parts.last_mut().expect("missing current")
        }
    }

    impl<'sval> ValueSlice<'sval> {
        pub(super) fn new<'a>(parts: &'a [ValuePart<'sval>]) -> &'a ValueSlice<'sval> {
            unsafe { mem::transmute::<&'a [ValuePart<'sval>], &'a ValueSlice<'sval>>(parts) }
        }

        fn get(&self, i: usize) -> Option<&ValuePart<'sval>> {
            self.0.get(i)
        }

        pub(super) fn slice<'a>(&'a self, range: Range<usize>) -> &'a ValueSlice<'sval> {
            match self.0.get(range.clone()) {
                Some(_) => (),
                None => {
                    panic!("{:?} is out of range for {:?}", range, &self.0);
                }
            }

            // SAFETY: `&[ValuePart]` and `&ValueSlice` have the same ABI
            unsafe {
                mem::transmute::<&'a [ValuePart<'sval>], &'a ValueSlice<'sval>>(&self.0[range])
            }
        }
    }

    impl<'sval> ValuePart<'sval> {
        pub(super) fn into_owned_in_place(&mut self) -> &mut ValuePart<'static> {
            let ValuePart { kind } = self;

            match kind {
                ValueKind::Text(ref mut text) => crate::assert_static(text.into_owned_in_place()),
                ValueKind::Binary(ref mut binary) => {
                    crate::assert_static(binary.into_owned_in_place())
                }
                ValueKind::Null => (),
                ValueKind::Bool(v) => crate::assert_static(v),
                ValueKind::U8(v) => crate::assert_static(v),
                ValueKind::U16(v) => crate::assert_static(v),
                ValueKind::U32(v) => crate::assert_static(v),
                ValueKind::U64(v) => crate::assert_static(v),
                ValueKind::U128(v) => crate::assert_static(v),
                ValueKind::I8(v) => crate::assert_static(v),
                ValueKind::I16(v) => crate::assert_static(v),
                ValueKind::I32(v) => crate::assert_static(v),
                ValueKind::I64(v) => crate::assert_static(v),
                ValueKind::I128(v) => crate::assert_static(v),
                ValueKind::F32(v) => crate::assert_static(v),
                ValueKind::F64(v) => crate::assert_static(v),
                ValueKind::Map {
                    len,
                    num_entries_hint,
                } => {
                    crate::assert_static(len);
                    crate::assert_static(num_entries_hint)
                }
                ValueKind::MapKey { len } => crate::assert_static(len),
                ValueKind::MapValue { len } => crate::assert_static(len),
                ValueKind::Seq {
                    len,
                    num_entries_hint,
                } => {
                    crate::assert_static(len);
                    crate::assert_static(num_entries_hint)
                }
                ValueKind::SeqValue { len } => crate::assert_static(len),
                ValueKind::Tag { tag, label, index } => {
                    crate::assert_static(tag);
                    crate::assert_static(label);
                    crate::assert_static(index)
                }
                ValueKind::Enum {
                    len,
                    tag,
                    label,
                    index,
                } => {
                    crate::assert_static(len);
                    crate::assert_static(tag);
                    crate::assert_static(label);
                    crate::assert_static(index)
                }
                ValueKind::Tagged {
                    len,
                    tag,
                    label,
                    index,
                } => {
                    crate::assert_static(len);
                    crate::assert_static(tag);
                    crate::assert_static(label);
                    crate::assert_static(index)
                }
                ValueKind::Record {
                    len,
                    tag,
                    label,
                    index,
                    num_entries,
                } => {
                    crate::assert_static(len);
                    crate::assert_static(tag);
                    crate::assert_static(label);
                    crate::assert_static(index);
                    crate::assert_static(num_entries)
                }
                ValueKind::RecordValue { len, tag, label } => {
                    crate::assert_static(len);
                    crate::assert_static(tag);
                    crate::assert_static(label)
                }
                ValueKind::Tuple {
                    len,
                    tag,
                    label,
                    index,
                    num_entries,
                } => {
                    crate::assert_static(len);
                    crate::assert_static(tag);
                    crate::assert_static(label);
                    crate::assert_static(index);
                    crate::assert_static(num_entries)
                }
                ValueKind::TupleValue { len, tag, index } => {
                    crate::assert_static(len);
                    crate::assert_static(tag);
                    crate::assert_static(index)
                }
                ValueKind::RecordTuple {
                    len,
                    tag,
                    label,
                    index,
                    num_entries,
                } => {
                    crate::assert_static(len);
                    crate::assert_static(tag);
                    crate::assert_static(label);
                    crate::assert_static(index);
                    crate::assert_static(num_entries)
                }
                ValueKind::RecordTupleValue {
                    len,
                    tag,
                    label,
                    index,
                } => {
                    crate::assert_static(len);
                    crate::assert_static(tag);
                    crate::assert_static(label);
                    crate::assert_static(index)
                }
            }

            // SAFETY: `self` no longer contains any data borrowed for `'sval`
            unsafe { mem::transmute::<&mut ValuePart<'sval>, &mut ValuePart<'static>>(self) }
        }
    }

    impl<'a> sval::Value for ValueSlice<'a> {
        fn stream<'sval, S: sval::Stream<'sval> + ?Sized>(
            &'sval self,
            stream: &mut S,
        ) -> sval::Result {
            self.stream_ref(stream)
        }
    }

    impl<'sval> sval_ref::ValueRef<'sval> for ValueSlice<'sval> {
        fn stream_ref<'a, S: sval::Stream<'sval> + ?Sized>(
            &'a self,
            stream: &mut S,
        ) -> sval::Result {
            let mut i = 0;

            fn stream_value<'a, 'sval, S: sval::Stream<'sval> + ?Sized>(
                stream: &mut S,
                i: &mut usize,
                len: usize,
                value: &ValueSlice<'sval>,
                f: impl FnOnce(&mut S, &ValueSlice<'sval>) -> sval::Result,
            ) -> sval::Result {
                let value = value.slice({
                    let start = *i + 1;
                    let end = start + len;

                    start..end
                });

                f(stream, value)?;

                *i += len;

                Ok(())
            }

            while let Some(part) = self.get(i) {
                let part: &ValuePart<'sval> = part;
                match &part.kind {
                    ValueKind::Null => stream.null()?,
                    ValueKind::Bool(v) => stream.bool(*v)?,
                    ValueKind::U8(v) => stream.u8(*v)?,
                    ValueKind::U16(v) => stream.u16(*v)?,
                    ValueKind::U32(v) => stream.u32(*v)?,
                    ValueKind::U64(v) => stream.u64(*v)?,
                    ValueKind::U128(v) => stream.u128(*v)?,
                    ValueKind::I8(v) => stream.i8(*v)?,
                    ValueKind::I16(v) => stream.i16(*v)?,
                    ValueKind::I32(v) => stream.i32(*v)?,
                    ValueKind::I64(v) => stream.i64(*v)?,
                    ValueKind::I128(v) => stream.i128(*v)?,
                    ValueKind::F32(v) => stream.f32(*v)?,
                    ValueKind::F64(v) => stream.f64(*v)?,
                    ValueKind::Text(v) => sval_ref::stream_ref(stream, v)?,
                    ValueKind::Binary(v) => sval_ref::stream_ref(stream, v)?,
                    ValueKind::Map {
                        len,
                        num_entries_hint,
                    } => {
                        stream_value(stream, &mut i, *len, self, |stream, body| {
                            stream.map_begin(*num_entries_hint)?;
                            sval_ref::stream_ref(stream, body)?;
                            stream.map_end()
                        })?;
                    }
                    ValueKind::MapKey { len } => {
                        stream_value(stream, &mut i, *len, self, |stream, body| {
                            stream.map_key_begin()?;
                            sval_ref::stream_ref(stream, body)?;
                            stream.map_key_end()
                        })?;
                    }
                    ValueKind::MapValue { len } => {
                        stream_value(stream, &mut i, *len, self, |stream, body| {
                            stream.map_value_begin()?;
                            sval_ref::stream_ref(stream, body)?;
                            stream.map_value_end()
                        })?;
                    }
                    ValueKind::Seq {
                        len,
                        num_entries_hint,
                    } => {
                        stream_value(stream, &mut i, *len, self, |stream, body| {
                            stream.seq_begin(*num_entries_hint)?;
                            sval_ref::stream_ref(stream, body)?;
                            stream.seq_end()
                        })?;
                    }
                    ValueKind::SeqValue { len } => {
                        stream_value(stream, &mut i, *len, self, |stream, body| {
                            stream.seq_value_begin()?;
                            sval_ref::stream_ref(stream, body)?;
                            stream.seq_value_end()
                        })?;
                    }
                    ValueKind::Tag { tag, label, index } => {
                        stream.tag(tag.as_ref(), label.as_ref(), index.as_ref())?;
                    }
                    ValueKind::Enum {
                        len,
                        tag,
                        label,
                        index,
                    } => {
                        stream_value(stream, &mut i, *len, self, |stream, body| {
                            stream.enum_begin(tag.as_ref(), label.as_ref(), index.as_ref())?;
                            sval_ref::stream_ref(stream, body)?;
                            stream.enum_end(tag.as_ref(), label.as_ref(), index.as_ref())
                        })?;
                    }
                    ValueKind::Tagged {
                        len,
                        tag,
                        label,
                        index,
                    } => {
                        stream_value(stream, &mut i, *len, self, |stream, body| {
                            stream.tagged_begin(tag.as_ref(), label.as_ref(), index.as_ref())?;
                            sval_ref::stream_ref(stream, body)?;
                            stream.tagged_end(tag.as_ref(), label.as_ref(), index.as_ref())
                        })?;
                    }
                    ValueKind::Record {
                        len,
                        tag,
                        label,
                        index,
                        num_entries,
                    } => {
                        stream_value(stream, &mut i, *len, self, |stream, body| {
                            stream.record_begin(
                                tag.as_ref(),
                                label.as_ref(),
                                index.as_ref(),
                                *num_entries,
                            )?;
                            sval_ref::stream_ref(stream, body)?;
                            stream.record_end(tag.as_ref(), label.as_ref(), index.as_ref())
                        })?;
                    }
                    ValueKind::RecordValue { len, tag, label } => {
                        stream_value(stream, &mut i, *len, self, |stream, body| {
                            stream.record_value_begin(tag.as_ref(), label)?;
                            sval_ref::stream_ref(stream, body)?;
                            stream.record_value_end(tag.as_ref(), label)
                        })?;
                    }
                    ValueKind::Tuple {
                        len,
                        tag,
                        label,
                        index,
                        num_entries,
                    } => {
                        stream_value(stream, &mut i, *len, self, |stream, body| {
                            stream.tuple_begin(
                                tag.as_ref(),
                                label.as_ref(),
                                index.as_ref(),
                                *num_entries,
                            )?;
                            sval_ref::stream_ref(stream, body)?;
                            stream.tuple_end(tag.as_ref(), label.as_ref(), index.as_ref())
                        })?;
                    }
                    ValueKind::TupleValue { len, tag, index } => {
                        stream_value(stream, &mut i, *len, self, |stream, body| {
                            stream.tuple_value_begin(tag.as_ref(), index)?;
                            sval_ref::stream_ref(stream, body)?;
                            stream.tuple_value_end(tag.as_ref(), index)
                        })?;
                    }
                    ValueKind::RecordTuple {
                        len,
                        tag,
                        label,
                        index,
                        num_entries,
                    } => {
                        stream_value(stream, &mut i, *len, self, |stream, body| {
                            stream.record_tuple_begin(
                                tag.as_ref(),
                                label.as_ref(),
                                index.as_ref(),
                                *num_entries,
                            )?;
                            sval_ref::stream_ref(stream, body)?;
                            stream.record_tuple_end(tag.as_ref(), label.as_ref(), index.as_ref())
                        })?;
                    }
                    ValueKind::RecordTupleValue {
                        len,
                        tag,
                        label,
                        index,
                    } => {
                        stream_value(stream, &mut i, *len, self, |stream, body| {
                            stream.record_tuple_value_begin(tag.as_ref(), label, index)?;
                            sval_ref::stream_ref(stream, body)?;
                            stream.record_tuple_value_end(tag.as_ref(), label, index)
                        })?;
                    }
                }

                i += 1;
            }

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::std::{string::String, vec};

        use sval::Stream as _;
        use sval_derive_macros::*;

        #[test]
        fn is_send_sync() {
            fn assert<T: Send + Sync>() {}

            assert::<ValueBuf>();
            assert::<Value>();
        }

        #[test]
        fn empty_is_complete() {
            assert!(!ValueBuf::new().is_complete());
        }

        #[test]
        fn primitive_is_complete() {
            assert!(ValueBuf::collect(&42).unwrap().is_complete());
        }

        #[test]
        fn text_is_complete() {
            let mut buf = ValueBuf::new();

            buf.text_begin(None).unwrap();

            assert!(!buf.is_complete());

            buf.text_end().unwrap();

            assert!(buf.is_complete());
        }

        #[test]
        fn binary_is_complete() {
            let mut buf = ValueBuf::new();

            buf.binary_begin(None).unwrap();

            assert!(!buf.is_complete());

            buf.binary_end().unwrap();

            assert!(buf.is_complete());
        }

        #[test]
        fn map_is_complete() {
            let mut buf = ValueBuf::new();

            buf.map_begin(None).unwrap();

            assert!(!buf.is_complete());

            buf.map_end().unwrap();

            assert!(buf.is_complete());
        }

        #[test]
        fn seq_is_complete() {
            let mut buf = ValueBuf::new();

            buf.seq_begin(None).unwrap();

            assert!(!buf.is_complete());

            buf.seq_end().unwrap();

            assert!(buf.is_complete());
        }

        #[test]
        fn empty() {
            use sval_test::{assert_tokens, Token::*};

            assert_tokens(&ValueBuf::new(), &[Null]);
        }

        #[test]
        fn buffer_primitive() {
            for (value, expected) in [
                (
                    ValueBuf::collect(&true).unwrap(),
                    vec![ValuePart {
                        kind: ValueKind::Bool(true),
                    }],
                ),
                (
                    ValueBuf::collect(&1i8).unwrap(),
                    vec![ValuePart {
                        kind: ValueKind::I8(1),
                    }],
                ),
                (
                    ValueBuf::collect(&2i16).unwrap(),
                    vec![ValuePart {
                        kind: ValueKind::I16(2),
                    }],
                ),
                (
                    ValueBuf::collect(&3i32).unwrap(),
                    vec![ValuePart {
                        kind: ValueKind::I32(3),
                    }],
                ),
                (
                    ValueBuf::collect(&4i64).unwrap(),
                    vec![ValuePart {
                        kind: ValueKind::I64(4),
                    }],
                ),
                (
                    ValueBuf::collect(&5i128).unwrap(),
                    vec![ValuePart {
                        kind: ValueKind::I128(5),
                    }],
                ),
                (
                    ValueBuf::collect(&1u8).unwrap(),
                    vec![ValuePart {
                        kind: ValueKind::U8(1),
                    }],
                ),
                (
                    ValueBuf::collect(&2u16).unwrap(),
                    vec![ValuePart {
                        kind: ValueKind::U16(2),
                    }],
                ),
                (
                    ValueBuf::collect(&3u32).unwrap(),
                    vec![ValuePart {
                        kind: ValueKind::U32(3),
                    }],
                ),
                (
                    ValueBuf::collect(&4u64).unwrap(),
                    vec![ValuePart {
                        kind: ValueKind::U64(4),
                    }],
                ),
                (
                    ValueBuf::collect(&5u128).unwrap(),
                    vec![ValuePart {
                        kind: ValueKind::U128(5),
                    }],
                ),
                (
                    ValueBuf::collect(&3.14f32).unwrap(),
                    vec![ValuePart {
                        kind: ValueKind::F32(3.14),
                    }],
                ),
                (
                    ValueBuf::collect(&3.1415f64).unwrap(),
                    vec![ValuePart {
                        kind: ValueKind::F64(3.1415),
                    }],
                ),
                (
                    ValueBuf::collect("abc").unwrap(),
                    vec![ValuePart {
                        kind: ValueKind::Text(TextBuf::from("abc")),
                    }],
                ),
                (
                    ValueBuf::collect(sval::BinarySlice::new(b"abc")).unwrap(),
                    vec![ValuePart {
                        kind: ValueKind::Binary(BinaryBuf::from(b"abc")),
                    }],
                ),
            ] {
                assert_eq!(expected, value.parts, "{:?}", value);
            }
        }

        #[test]
        fn buffer_option() {
            let expected = vec![ValuePart {
                kind: ValueKind::Tag {
                    tag: Some(sval::tags::RUST_OPTION_NONE),
                    label: Some(sval::Label::new("None")),
                    index: Some(sval::Index::new(0)),
                },
            }];

            assert_eq!(expected, ValueBuf::collect(&None::<i32>).unwrap().parts);

            let expected = vec![
                ValuePart {
                    kind: ValueKind::Tagged {
                        len: 1,
                        tag: Some(sval::tags::RUST_OPTION_SOME),
                        label: Some(sval::Label::new("Some")),
                        index: Some(sval::Index::new(1)),
                    },
                },
                ValuePart {
                    kind: ValueKind::I32(42),
                },
            ];

            assert_eq!(expected, ValueBuf::collect(&Some(42i32)).unwrap().parts);
        }

        #[test]
        fn buffer_map() {
            let mut value = ValueBuf::new();

            value.map_begin(Some(2)).unwrap();

            value.map_key_begin().unwrap();
            value.i32(0).unwrap();
            value.map_key_end().unwrap();

            value.map_value_begin().unwrap();
            value.bool(false).unwrap();
            value.map_value_end().unwrap();

            value.map_key_begin().unwrap();
            value.i32(1).unwrap();
            value.map_key_end().unwrap();

            value.map_value_begin().unwrap();
            value.bool(true).unwrap();
            value.map_value_end().unwrap();

            value.map_end().unwrap();

            let expected = vec![
                ValuePart {
                    kind: ValueKind::Map {
                        len: 8,
                        num_entries_hint: Some(2),
                    },
                },
                ValuePart {
                    kind: ValueKind::MapKey { len: 1 },
                },
                ValuePart {
                    kind: ValueKind::I32(0),
                },
                ValuePart {
                    kind: ValueKind::MapValue { len: 1 },
                },
                ValuePart {
                    kind: ValueKind::Bool(false),
                },
                ValuePart {
                    kind: ValueKind::MapKey { len: 1 },
                },
                ValuePart {
                    kind: ValueKind::I32(1),
                },
                ValuePart {
                    kind: ValueKind::MapValue { len: 1 },
                },
                ValuePart {
                    kind: ValueKind::Bool(true),
                },
            ];

            assert_eq!(expected, value.parts);
        }

        #[test]
        fn buffer_seq() {
            let mut value = ValueBuf::new();

            value.seq_begin(Some(2)).unwrap();

            value.seq_value_begin().unwrap();
            value.bool(false).unwrap();
            value.seq_value_end().unwrap();

            value.seq_value_begin().unwrap();
            value.bool(true).unwrap();
            value.seq_value_end().unwrap();

            value.seq_end().unwrap();

            let expected = vec![
                ValuePart {
                    kind: ValueKind::Seq {
                        len: 4,
                        num_entries_hint: Some(2),
                    },
                },
                ValuePart {
                    kind: ValueKind::SeqValue { len: 1 },
                },
                ValuePart {
                    kind: ValueKind::Bool(false),
                },
                ValuePart {
                    kind: ValueKind::SeqValue { len: 1 },
                },
                ValuePart {
                    kind: ValueKind::Bool(true),
                },
            ];

            assert_eq!(expected, value.parts);
        }

        #[test]
        fn buffer_record() {
            let mut value = ValueBuf::new();

            value
                .record_begin(
                    Some(&sval::Tag::new("test")),
                    Some(&sval::Label::new("A")),
                    Some(&sval::Index::new(1)),
                    Some(2),
                )
                .unwrap();

            value
                .record_value_begin(None, &sval::Label::new("a"))
                .unwrap();
            value.bool(false).unwrap();
            value
                .record_value_end(None, &sval::Label::new("a"))
                .unwrap();

            value
                .record_value_begin(None, &sval::Label::new("b"))
                .unwrap();
            value.bool(true).unwrap();
            value
                .record_value_end(None, &sval::Label::new("b"))
                .unwrap();

            value
                .record_end(
                    Some(&sval::Tag::new("test")),
                    Some(&sval::Label::new("A")),
                    Some(&sval::Index::new(1)),
                )
                .unwrap();

            let expected = vec![
                ValuePart {
                    kind: ValueKind::Record {
                        len: 4,
                        tag: Some(sval::Tag::new("test")),
                        label: Some(sval::Label::new("A")),
                        index: Some(sval::Index::new(1)),
                        num_entries: Some(2),
                    },
                },
                ValuePart {
                    kind: ValueKind::RecordValue {
                        len: 1,
                        tag: None,
                        label: sval::Label::new("a"),
                    },
                },
                ValuePart {
                    kind: ValueKind::Bool(false),
                },
                ValuePart {
                    kind: ValueKind::RecordValue {
                        len: 1,
                        tag: None,
                        label: sval::Label::new("b"),
                    },
                },
                ValuePart {
                    kind: ValueKind::Bool(true),
                },
            ];

            assert_eq!(expected, value.parts);
        }

        #[test]
        fn buffer_tuple() {
            let mut value = ValueBuf::new();

            value
                .tuple_begin(
                    Some(&sval::Tag::new("test")),
                    Some(&sval::Label::new("A")),
                    Some(&sval::Index::new(1)),
                    Some(2),
                )
                .unwrap();

            value.tuple_value_begin(None, &sval::Index::new(0)).unwrap();
            value.bool(false).unwrap();
            value.tuple_value_end(None, &sval::Index::new(0)).unwrap();

            value.tuple_value_begin(None, &sval::Index::new(1)).unwrap();
            value.bool(true).unwrap();
            value.tuple_value_end(None, &sval::Index::new(1)).unwrap();

            value
                .tuple_end(
                    Some(&sval::Tag::new("test")),
                    Some(&sval::Label::new("A")),
                    Some(&sval::Index::new(1)),
                )
                .unwrap();

            let expected = vec![
                ValuePart {
                    kind: ValueKind::Tuple {
                        len: 4,
                        tag: Some(sval::Tag::new("test")),
                        label: Some(sval::Label::new("A")),
                        index: Some(sval::Index::new(1)),
                        num_entries: Some(2),
                    },
                },
                ValuePart {
                    kind: ValueKind::TupleValue {
                        len: 1,
                        tag: None,
                        index: sval::Index::new(0),
                    },
                },
                ValuePart {
                    kind: ValueKind::Bool(false),
                },
                ValuePart {
                    kind: ValueKind::TupleValue {
                        len: 1,
                        tag: None,
                        index: sval::Index::new(1),
                    },
                },
                ValuePart {
                    kind: ValueKind::Bool(true),
                },
            ];

            assert_eq!(expected, value.parts);
        }

        #[test]
        fn buffer_record_tuple() {
            let mut value = ValueBuf::new();

            value
                .record_tuple_begin(
                    Some(&sval::Tag::new("test")),
                    Some(&sval::Label::new("A")),
                    Some(&sval::Index::new(1)),
                    Some(2),
                )
                .unwrap();

            value
                .record_tuple_value_begin(None, &sval::Label::new("a"), &sval::Index::new(0))
                .unwrap();
            value.bool(false).unwrap();
            value
                .record_tuple_value_end(None, &sval::Label::new("a"), &sval::Index::new(0))
                .unwrap();

            value
                .record_tuple_value_begin(None, &sval::Label::new("b"), &sval::Index::new(1))
                .unwrap();
            value.bool(true).unwrap();
            value
                .record_tuple_value_end(None, &sval::Label::new("b"), &sval::Index::new(1))
                .unwrap();

            value
                .record_tuple_end(
                    Some(&sval::Tag::new("test")),
                    Some(&sval::Label::new("A")),
                    Some(&sval::Index::new(1)),
                )
                .unwrap();

            let expected = vec![
                ValuePart {
                    kind: ValueKind::RecordTuple {
                        len: 4,
                        tag: Some(sval::Tag::new("test")),
                        label: Some(sval::Label::new("A")),
                        index: Some(sval::Index::new(1)),
                        num_entries: Some(2),
                    },
                },
                ValuePart {
                    kind: ValueKind::RecordTupleValue {
                        len: 1,
                        tag: None,
                        label: sval::Label::new("a"),
                        index: sval::Index::new(0),
                    },
                },
                ValuePart {
                    kind: ValueKind::Bool(false),
                },
                ValuePart {
                    kind: ValueKind::RecordTupleValue {
                        len: 1,
                        tag: None,
                        label: sval::Label::new("b"),
                        index: sval::Index::new(1),
                    },
                },
                ValuePart {
                    kind: ValueKind::Bool(true),
                },
            ];

            assert_eq!(expected, value.parts);
        }

        #[test]
        fn buffer_enum() {
            let mut value = ValueBuf::new();

            value
                .enum_begin(
                    Some(&sval::Tag::new("test")),
                    Some(&sval::Label::new("A")),
                    Some(&sval::Index::new(1)),
                )
                .unwrap();

            value
                .tag(
                    None,
                    Some(&sval::Label::new("B")),
                    Some(&sval::Index::new(0)),
                )
                .unwrap();

            value
                .enum_end(
                    Some(&sval::Tag::new("test")),
                    Some(&sval::Label::new("A")),
                    Some(&sval::Index::new(1)),
                )
                .unwrap();

            let expected = vec![
                ValuePart {
                    kind: ValueKind::Enum {
                        len: 1,
                        tag: Some(sval::Tag::new("test")),
                        label: Some(sval::Label::new("A")),
                        index: Some(sval::Index::new(1)),
                    },
                },
                ValuePart {
                    kind: ValueKind::Tag {
                        tag: None,
                        label: Some(sval::Label::new("B")),
                        index: Some(sval::Index::new(0)),
                    },
                },
            ];

            assert_eq!(expected, value.parts);
        }

        #[test]
        fn buffer_roundtrip() {
            for value_1 in [
                ValueBuf::collect(&42i32).unwrap(),
                ValueBuf::collect(&vec![
                    vec![],
                    vec![vec![1, 2, 3], vec![4]],
                    vec![vec![5, 6], vec![7, 8, 9]],
                ])
                .unwrap(),
                ValueBuf::collect(&{
                    #[derive(Value)]
                    struct Record {
                        a: i32,
                        b: bool,
                    }

                    Record { a: 42, b: true }
                })
                .unwrap(),
                ValueBuf::collect(&{
                    #[derive(Value)]
                    struct Tuple(i32, bool);

                    Tuple(42, true)
                })
                .unwrap(),
                ValueBuf::collect(&{
                    #[derive(Value)]
                    enum Enum {
                        A,
                    }

                    Enum::A
                })
                .unwrap(),
            ] {
                let value_2 = ValueBuf::collect(&value_1).unwrap();

                assert_eq!(value_1.parts, value_2.parts, "{:?}", value_1);
            }
        }

        #[test]
        fn buffer_reuse() {
            let mut buf = ValueBuf::new();

            buf.i32(42).unwrap();

            assert_eq!(Value::collect(&42i32).unwrap().parts, buf.to_value().parts);

            buf.clear();

            buf.bool(true).unwrap();

            assert_eq!(Value::collect(&true).unwrap().parts, buf.to_value().parts);
        }

        #[test]
        fn buffer_invalid() {
            struct Kaboom;

            impl sval::Value for Kaboom {
                fn stream<'sval, S: sval::Stream<'sval> + ?Sized>(
                    &'sval self,
                    _: &mut S,
                ) -> sval::Result {
                    sval::error()
                }
            }

            // Ensure we don't panic
            let _ = Value::collect(&Kaboom);
            let _ = Value::collect_owned(&Kaboom);
        }

        #[test]
        fn into_owned() {
            let short_lived = String::from("abc");

            let buf = ValueBuf::collect(&short_lived).unwrap();
            let borrowed_ptr = buf.parts.as_ptr() as *const ();

            let owned = buf.into_owned().unwrap();
            let owned_ptr = owned.parts.as_ptr() as *const ();
            drop(short_lived);

            assert!(core::ptr::eq(borrowed_ptr, owned_ptr));

            match owned.parts[0].kind {
                ValueKind::Text(ref text) => {
                    assert!(text.as_borrowed_str().is_none());
                    assert_eq!("abc", text.as_str());
                }
                _ => unreachable!(),
            }
        }

        #[test]
        fn collect_owned() {
            let short_lived = String::from("abc");

            let buf = ValueBuf::collect_owned(&short_lived).unwrap();
            drop(short_lived);

            match buf.parts[0].kind {
                ValueKind::Text(ref text) => {
                    assert!(text.as_borrowed_str().is_none());
                    assert_eq!("abc", text.as_str());
                }
                _ => unreachable!(),
            }
        }
    }
}

#[cfg(test)]
#[cfg(not(feature = "alloc"))]
mod no_std_tests {
    use super::*;

    use sval::Stream as _;

    #[test]
    fn buffer_reuse() {
        let mut buf = ValueBuf::new();

        buf.i32(42).unwrap_err();

        assert!(buf.err.is_some());

        buf.clear();

        assert!(buf.err.is_none());
    }
}
