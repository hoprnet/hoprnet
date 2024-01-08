use crate::{data, tags, Index, Label, Result, Tag, Value};

/**
A consumer of structured data.
*/
pub trait Stream<'sval> {
    /**
    Recurse into a nested value.
    */
    fn value<V: Value + ?Sized>(&mut self, v: &'sval V) -> Result {
        v.stream(self)
    }

    /**
    Recurse into a nested value, borrowed for some arbitrarily short lifetime.
    */
    fn value_computed<V: Value + ?Sized>(&mut self, v: &V) -> Result {
        v.stream(Computed::new_borrowed(self))
    }

    /**
    Stream null, the absence of any other meaningful value.
    */
    fn null(&mut self) -> Result;

    /**
    Stream a boolean.
    */
    fn bool(&mut self, value: bool) -> Result;

    /**
    Start a UTF8 text string.
    */
    fn text_begin(&mut self, num_bytes: Option<usize>) -> Result;

    /**
    Stream a fragment of UTF8 text.
    */
    #[inline]
    fn text_fragment(&mut self, fragment: &'sval str) -> Result {
        self.text_fragment_computed(fragment)
    }

    /**
    Stream a fragment of UTF8 text, borrowed for some arbitrarily short lifetime.
    */
    fn text_fragment_computed(&mut self, fragment: &str) -> Result;

    /**
    Complete a UTF8 text string.
    */
    fn text_end(&mut self) -> Result;

    /**
    Start a bitstring.
    */
    fn binary_begin(&mut self, num_bytes: Option<usize>) -> Result {
        self.seq_begin(num_bytes)
    }

    /**
    Stream a fragment of a bitstring.
    */
    #[inline]
    fn binary_fragment(&mut self, fragment: &'sval [u8]) -> Result {
        self.binary_fragment_computed(fragment)
    }

    /**
    Stream a fragment of a bitstring, borrowed for some arbitrarily short lifetime.
    */
    fn binary_fragment_computed(&mut self, fragment: &[u8]) -> Result {
        for byte in fragment {
            self.seq_value_begin()?;
            self.u8(*byte)?;
            self.seq_value_end()?;
        }

        Ok(())
    }

    /**
    Complete a bitstring.
    */
    #[inline]
    fn binary_end(&mut self) -> Result {
        self.seq_end()
    }

    /**
    Stream an unsigned 8bit integer.
    */
    #[inline]
    fn u8(&mut self, value: u8) -> Result {
        self.u16(value as u16)
    }

    /**
    Stream an unsigned 16bit integer.
    */
    #[inline]
    fn u16(&mut self, value: u16) -> Result {
        self.u32(value as u32)
    }

    /**
    Stream an unsigned 32bit integer.
    */
    #[inline]
    fn u32(&mut self, value: u32) -> Result {
        self.u64(value as u64)
    }

    /**
    Stream an unsigned 64bit integer.
    */
    #[inline]
    fn u64(&mut self, value: u64) -> Result {
        self.u128(value as u128)
    }

    /**
    Stream an unsigned 128bit integer.
    */
    fn u128(&mut self, value: u128) -> Result {
        if let Ok(value) = value.try_into() {
            self.i64(value)
        } else {
            data::stream_u128(value, self)
        }
    }

    /**
    Stream a signed 8bit integer.
    */
    #[inline]
    fn i8(&mut self, value: i8) -> Result {
        self.i16(value as i16)
    }

    /**
    Stream a signed 16bit integer.
    */
    #[inline]
    fn i16(&mut self, value: i16) -> Result {
        self.i32(value as i32)
    }

    /**
    Stream a signed 32bit integer.
    */
    #[inline]
    fn i32(&mut self, value: i32) -> Result {
        self.i64(value as i64)
    }

    /**
    Stream a signed 64bit integer.
    */
    fn i64(&mut self, value: i64) -> Result;

    /**
    Stream a signed 128bit integer.
    */
    fn i128(&mut self, value: i128) -> Result {
        if let Ok(value) = value.try_into() {
            self.i64(value)
        } else {
            data::stream_i128(value, self)
        }
    }

    /**
    Stream a 32bit binary floating point number.
    */
    #[inline]
    fn f32(&mut self, value: f32) -> Result {
        self.f64(value as f64)
    }

    /**
    Stream a 64bit binary floating point number.
    */
    fn f64(&mut self, value: f64) -> Result;

    /**
    Start a homogenous mapping of arbitrary keys to values.
    */
    #[inline]
    fn map_begin(&mut self, num_entries: Option<usize>) -> Result {
        self.seq_begin(num_entries)
    }

    /**
    Start a key in a key-value mapping.
    */
    #[inline]
    fn map_key_begin(&mut self) -> Result {
        self.seq_value_begin()?;
        self.tuple_begin(None, None, None, Some(2))?;
        self.tuple_value_begin(None, &Index::new(0))
    }

    /**
    Complete a key in a key-value mapping.
    */
    #[inline]
    fn map_key_end(&mut self) -> Result {
        self.tuple_value_end(None, &Index::new(0))
    }

    /**
    Start a value in a key-value mapping.
    */
    #[inline]
    fn map_value_begin(&mut self) -> Result {
        self.tuple_value_begin(None, &Index::new(1))
    }

    /**
    Complete a value in a key-value mapping.
    */
    #[inline]
    fn map_value_end(&mut self) -> Result {
        self.tuple_value_end(None, &Index::new(1))?;
        self.tuple_end(None, None, None)?;
        self.seq_value_end()
    }

    /**
    Complete a homogenous mapping of arbitrary keys to values.
    */
    #[inline]
    fn map_end(&mut self) -> Result {
        self.seq_end()
    }

    /**
    Start a homogenous sequence of values.
    */
    fn seq_begin(&mut self, num_entries: Option<usize>) -> Result;

    /**
    Start an individual value in a sequence.
    */
    fn seq_value_begin(&mut self) -> Result;

    /**
    Complete an individual value in a sequence.
    */
    fn seq_value_end(&mut self) -> Result;

    /**
    Complete a homogenous sequence of values.
    */
    fn seq_end(&mut self) -> Result;

    /**
    Start a variant in an enumerated type.
    */
    #[inline]
    fn enum_begin(
        &mut self,
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
    ) -> Result {
        self.tagged_begin(tag, label, index)
    }

    /**
    Complete a variant in an enumerated type.
    */
    #[inline]
    fn enum_end(
        &mut self,
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
    ) -> Result {
        self.tagged_end(tag, label, index)
    }

    /**
    Start a tagged value.

    Tagged values may be used as enum variants.
    */
    #[inline]
    fn tagged_begin(
        &mut self,
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
    ) -> Result {
        let _ = tag;
        let _ = label;
        let _ = index;

        Ok(())
    }

    /**
    Complete a tagged value.
    */
    #[inline]
    fn tagged_end(
        &mut self,
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
    ) -> Result {
        let _ = tag;
        let _ = label;
        let _ = index;

        Ok(())
    }

    /**
    Stream a standalone tag.

    Standalone tags may be used as enum variants.
    */
    fn tag(&mut self, tag: Option<&Tag>, label: Option<&Label>, index: Option<&Index>) -> Result {
        self.tagged_begin(tag, label, index)?;

        // Rust's `Option` is fundamental enough that we handle it specially here
        if let Some(&tags::RUST_OPTION_NONE) = tag {
            self.null()?;
        }
        // If the tag has a label then stream it as its value
        else if let Some(ref label) = label {
            if let Some(label) = label.as_static_str() {
                self.value(label)?;
            } else {
                self.value_computed(label.as_str())?;
            }
        }
        // If the tag doesn't have a label then stream null
        else {
            self.null()?;
        }

        self.tagged_end(tag, label, index)
    }

    /**
    Start a record type.

    Records may be used as enum variants.
    */
    #[inline]
    fn record_begin(
        &mut self,
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
        num_entries: Option<usize>,
    ) -> Result {
        self.tagged_begin(tag, label, index)?;
        self.map_begin(num_entries)
    }

    /**
    Start a field in a record.
    */
    #[inline]
    fn record_value_begin(&mut self, tag: Option<&Tag>, label: &Label) -> Result {
        let _ = tag;

        self.map_key_begin()?;

        if let Some(label) = label.as_static_str() {
            self.value(label)?;
        } else {
            self.value_computed(label.as_str())?;
        }

        self.map_key_end()?;

        self.map_value_begin()
    }

    /**
    Complete a field in a record.
    */
    #[inline]
    fn record_value_end(&mut self, tag: Option<&Tag>, label: &Label) -> Result {
        let _ = tag;
        let _ = label;

        self.map_value_end()
    }

    /**
    Complete a record type.
    */
    #[inline]
    fn record_end(
        &mut self,
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
    ) -> Result {
        self.map_end()?;
        self.tagged_end(tag, label, index)
    }

    /**
    Start a tuple type.

    Tuples may be used as enum variants.
    */
    #[inline]
    fn tuple_begin(
        &mut self,
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
        num_entries: Option<usize>,
    ) -> Result {
        self.tagged_begin(tag, label, index)?;
        self.seq_begin(num_entries)
    }

    /**
    Start a field in a tuple.
    */
    #[inline]
    fn tuple_value_begin(&mut self, tag: Option<&Tag>, index: &Index) -> Result {
        let _ = tag;
        let _ = index;

        self.seq_value_begin()
    }

    /**
    Complete a field in a tuple.
    */
    #[inline]
    fn tuple_value_end(&mut self, tag: Option<&Tag>, index: &Index) -> Result {
        let _ = tag;
        let _ = index;

        self.seq_value_end()
    }

    /**
    Complete a tuple type.
    */
    #[inline]
    fn tuple_end(
        &mut self,
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
    ) -> Result {
        self.seq_end()?;
        self.tagged_end(tag, label, index)
    }

    /**
    Begin a type that may be treated as either a record or a tuple.
    */
    #[inline]
    fn record_tuple_begin(
        &mut self,
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
        num_entries: Option<usize>,
    ) -> Result {
        self.record_begin(tag, label, index, num_entries)
    }

    /**
    Begin a field in a type that may be treated as either a record or a tuple.
    */
    #[inline]
    fn record_tuple_value_begin(
        &mut self,
        tag: Option<&Tag>,
        label: &Label,
        index: &Index,
    ) -> Result {
        let _ = index;

        self.record_value_begin(tag, label)
    }

    /**
    Complete a field in a type that may be treated as either a record or a tuple.
    */
    #[inline]
    fn record_tuple_value_end(
        &mut self,
        tag: Option<&Tag>,
        label: &Label,
        index: &Index,
    ) -> Result {
        let _ = index;

        self.record_value_end(tag, label)
    }

    /**
    Complete a type that may be treated as either a record or a tuple.
    */
    #[inline]
    fn record_tuple_end(
        &mut self,
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
    ) -> Result {
        self.record_end(tag, label, index)
    }
}

macro_rules! impl_stream_forward {
    ({ $($r:tt)* } => $bind:ident => { $($forward:tt)* }) => {
        $($r)* {
            fn value<V: Value + ?Sized>(&mut self, v: &'sval V) -> Result {
                let $bind = self;
                ($($forward)*).value(v)
            }

            fn value_computed<V: Value + ?Sized>(&mut self, v: &V) -> Result {
                let $bind = self;
                ($($forward)*).value_computed(v)
            }

            #[inline]
            fn null(&mut self) -> Result {
                let $bind = self;
                ($($forward)*).null()
            }

            #[inline]
            fn u8(&mut self, value: u8) -> Result {
                let $bind = self;
                ($($forward)*).u8(value)
            }

            #[inline]
            fn u16(&mut self, value: u16) -> Result {
                let $bind = self;
                ($($forward)*).u16(value)
            }

            #[inline]
            fn u32(&mut self, value: u32) -> Result {
                let $bind = self;
                ($($forward)*).u32(value)
            }

            #[inline]
            fn u64(&mut self, value: u64) -> Result {
                let $bind = self;
                ($($forward)*).u64(value)
            }

            #[inline]
            fn u128(&mut self, value: u128) -> Result {
                let $bind = self;
                ($($forward)*).u128(value)
            }

            #[inline]
            fn i8(&mut self, value: i8) -> Result {
                let $bind = self;
                ($($forward)*).i8(value)
            }

            #[inline]
            fn i16(&mut self, value: i16) -> Result {
                let $bind = self;
                ($($forward)*).i16(value)
            }

            #[inline]
            fn i32(&mut self, value: i32) -> Result {
                let $bind = self;
                ($($forward)*).i32(value)
            }

            #[inline]
            fn i64(&mut self, value: i64) -> Result {
                let $bind = self;
                ($($forward)*).i64(value)
            }

            #[inline]
            fn i128(&mut self, value: i128) -> Result {
                let $bind = self;
                ($($forward)*).i128(value)
            }

            #[inline]
            fn f32(&mut self, value: f32) -> Result {
                let $bind = self;
                ($($forward)*).f32(value)
            }

            #[inline]
            fn f64(&mut self, value: f64) -> Result {
                let $bind = self;
                ($($forward)*).f64(value)
            }

            #[inline]
            fn bool(&mut self, value: bool) -> Result {
                let $bind = self;
                ($($forward)*).bool(value)
            }

            #[inline]
            fn text_begin(&mut self, num_bytes: Option<usize>) -> Result {
                let $bind = self;
                ($($forward)*).text_begin(num_bytes)
            }

            #[inline]
            fn text_end(&mut self) -> Result {
                let $bind = self;
                ($($forward)*).text_end()
            }

            #[inline]
            fn text_fragment(&mut self, fragment: &'sval str) -> Result {
                let $bind = self;
                ($($forward)*).text_fragment(fragment)
            }

            #[inline]
            fn text_fragment_computed(&mut self, fragment: &str) -> Result {
                let $bind = self;
                ($($forward)*).text_fragment_computed(fragment)
            }

            #[inline]
            fn binary_begin(&mut self, num_bytes: Option<usize>) -> Result {
                let $bind = self;
                ($($forward)*).binary_begin(num_bytes)
            }

            #[inline]
            fn binary_end(&mut self) -> Result {
                let $bind = self;
                ($($forward)*).binary_end()
            }

            #[inline]
            fn binary_fragment(&mut self, fragment: &'sval [u8]) -> Result {
                let $bind = self;
                ($($forward)*).binary_fragment(fragment)
            }

            #[inline]
            fn binary_fragment_computed(&mut self, fragment: &[u8]) -> Result {
                let $bind = self;
                ($($forward)*).binary_fragment_computed(fragment)
            }

            #[inline]
            fn map_begin(&mut self, num_entries: Option<usize>) -> Result {
                let $bind = self;
                ($($forward)*).map_begin(num_entries)
            }

            #[inline]
            fn map_end(&mut self) -> Result {
                let $bind = self;
                ($($forward)*).map_end()
            }

            #[inline]
            fn map_key_begin(&mut self) -> Result {
                let $bind = self;
                ($($forward)*).map_key_begin()
            }

            #[inline]
            fn map_key_end(&mut self) -> Result {
                let $bind = self;
                ($($forward)*).map_key_end()
            }

            #[inline]
            fn map_value_begin(&mut self) -> Result {
                let $bind = self;
                ($($forward)*).map_value_begin()
            }

            #[inline]
            fn map_value_end(&mut self) -> Result {
                let $bind = self;
                ($($forward)*).map_value_end()
            }

            #[inline]
            fn seq_begin(&mut self, num_entries: Option<usize>) -> Result {
                let $bind = self;
                ($($forward)*).seq_begin(num_entries)
            }

            #[inline]
            fn seq_end(&mut self) -> Result {
                let $bind = self;
                ($($forward)*).seq_end()
            }

            #[inline]
            fn seq_value_begin(&mut self) -> Result {
                let $bind = self;
                ($($forward)*).seq_value_begin()
            }

            #[inline]
            fn seq_value_end(&mut self) -> Result {
                let $bind = self;
                ($($forward)*).seq_value_end()
            }

            #[inline]
            fn tagged_begin(&mut self, tag: Option<&Tag>, label: Option<&Label>, index: Option<&Index>) -> Result {
                let $bind = self;
                ($($forward)*).tagged_begin(tag, label, index)
            }

            #[inline]
            fn tagged_end(&mut self, tag: Option<&Tag>, label: Option<&Label>, index: Option<&Index>) -> Result {
                let $bind = self;
                ($($forward)*).tagged_end(tag, label, index)
            }

            #[inline]
            fn tag(&mut self, tag: Option<&Tag>, label: Option<&Label>, index: Option<&Index>) -> Result {
                let $bind = self;
                ($($forward)*).tag(tag, label, index)
            }

            #[inline]
            fn record_begin(&mut self, tag: Option<&Tag>, label: Option<&Label>, index: Option<&Index>, num_entries: Option<usize>) -> Result {
                let $bind = self;
                ($($forward)*).record_begin(tag, label, index, num_entries)
            }

            #[inline]
            fn record_value_begin(&mut self, tag: Option<&Tag>, label: &Label) -> Result {
                let $bind = self;
                ($($forward)*).record_value_begin(tag, label)
            }

            #[inline]
            fn record_value_end(&mut self, tag: Option<&Tag>, label: &Label) -> Result {
                let $bind = self;
                ($($forward)*).record_value_end(tag, label)
            }

            #[inline]
            fn record_end(&mut self, tag: Option<&Tag>, label: Option<&Label>, index: Option<&Index>) -> Result {
                let $bind = self;
                ($($forward)*).record_end(tag, label, index)
            }

            #[inline]
            fn tuple_begin(&mut self, tag: Option<&Tag>, label: Option<&Label>, index: Option<&Index>, num_entries: Option<usize>) -> Result {
                let $bind = self;
                ($($forward)*).tuple_begin(tag, label, index, num_entries)
            }

            #[inline]
            fn tuple_value_begin(&mut self, tag: Option<&Tag>, index: &Index) -> Result {
                let $bind = self;
                ($($forward)*).tuple_value_begin(tag, index)
            }

            #[inline]
            fn tuple_value_end(&mut self, tag: Option<&Tag>, index: &Index) -> Result {
                let $bind = self;
                ($($forward)*).tuple_value_end(tag, index)
            }

            #[inline]
            fn tuple_end(&mut self, tag: Option<&Tag>, label: Option<&Label>, index: Option<&Index>) -> Result {
                let $bind = self;
                ($($forward)*).tuple_end(tag, label, index)
            }

            #[inline]
            fn record_tuple_begin(&mut self, tag: Option<&Tag>, label: Option<&Label>, index: Option<&Index>, num_entries: Option<usize>) -> Result {
                let $bind = self;
                ($($forward)*).record_tuple_begin(tag, label, index, num_entries)
            }

            #[inline]
            fn record_tuple_value_begin(&mut self, tag: Option<&Tag>, label: &Label, index: &Index) -> Result {
                let $bind = self;
                ($($forward)*).record_tuple_value_begin(tag, label, index)
            }

            #[inline]
            fn record_tuple_value_end(&mut self, tag: Option<&Tag>, label: &Label, index: &Index) -> Result {
                let $bind = self;
                ($($forward)*).record_tuple_value_end(tag, label, index)
            }

            #[inline]
            fn record_tuple_end(&mut self, tag: Option<&Tag>, label: Option<&Label>, index: Option<&Index>) -> Result {
                let $bind = self;
                ($($forward)*).record_tuple_end(tag, label, index)
            }

            #[inline]
            fn enum_begin(&mut self, tag: Option<&Tag>, label: Option<&Label>, index: Option<&Index>) -> Result {
                let $bind = self;
                ($($forward)*).enum_begin(tag, label, index)
            }

            #[inline]
            fn enum_end(&mut self, tag: Option<&Tag>, label: Option<&Label>, index: Option<&Index>) -> Result {
                let $bind = self;
                ($($forward)*).enum_end(tag, label, index)
            }
        }
    };
}

impl_stream_forward!({ impl<'sval, 'a, S: ?Sized> Stream<'sval> for &'a mut S where S: Stream<'sval> } => x => { **x });

#[cfg(feature = "alloc")]
mod alloc_support {
    use super::*;

    use crate::std::boxed::Box;

    impl_stream_forward!({ impl<'sval, 'a, S: ?Sized> Stream<'sval> for Box<S> where S: Stream<'sval> } => x => { **x });
}

/**
A `Stream` that accepts values for any lifetime.

This is the result of calling [`Stream::computed`].
*/
#[repr(transparent)]
struct Computed<S: ?Sized>(S);

impl<S: ?Sized> Computed<S> {
    #[inline]
    fn new_borrowed<'a>(stream: &'a mut S) -> &'a mut Computed<S> {
        // SAFETY: `&'a mut S` and `&'a mut Computed<S>` have the same ABI
        unsafe { &mut *(stream as *mut _ as *mut Computed<S>) }
    }
}

impl<'a, 'b, S: Stream<'a> + ?Sized> Stream<'b> for Computed<S> {
    #[inline]
    fn value_computed<V: Value + ?Sized>(&mut self, v: &V) -> Result {
        self.0.value_computed(v)
    }

    #[inline]
    fn text_fragment(&mut self, fragment: &'b str) -> Result {
        self.0.text_fragment_computed(fragment)
    }

    #[inline]
    fn binary_fragment(&mut self, fragment: &'b [u8]) -> Result {
        self.0.binary_fragment_computed(fragment)
    }

    #[inline]
    fn null(&mut self) -> Result {
        self.0.null()
    }

    #[inline]
    fn u8(&mut self, v: u8) -> Result {
        self.0.u8(v)
    }

    #[inline]
    fn u16(&mut self, v: u16) -> Result {
        self.0.u16(v)
    }

    #[inline]
    fn u32(&mut self, v: u32) -> Result {
        self.0.u32(v)
    }

    #[inline]
    fn u64(&mut self, v: u64) -> Result {
        self.0.u64(v)
    }

    #[inline]
    fn u128(&mut self, v: u128) -> Result {
        self.0.u128(v)
    }

    #[inline]
    fn i8(&mut self, v: i8) -> Result {
        self.0.i8(v)
    }

    #[inline]
    fn i16(&mut self, v: i16) -> Result {
        self.0.i16(v)
    }

    #[inline]
    fn i32(&mut self, v: i32) -> Result {
        self.0.i32(v)
    }

    #[inline]
    fn i64(&mut self, v: i64) -> Result {
        self.0.i64(v)
    }

    #[inline]
    fn i128(&mut self, v: i128) -> Result {
        self.0.i128(v)
    }

    #[inline]
    fn f32(&mut self, v: f32) -> Result {
        self.0.f32(v)
    }

    #[inline]
    fn f64(&mut self, v: f64) -> Result {
        self.0.f64(v)
    }

    #[inline]
    fn bool(&mut self, v: bool) -> Result {
        self.0.bool(v)
    }

    #[inline]
    fn text_begin(&mut self, num_bytes: Option<usize>) -> Result {
        self.0.text_begin(num_bytes)
    }

    #[inline]
    fn text_fragment_computed(&mut self, fragment: &str) -> Result {
        self.0.text_fragment_computed(fragment)
    }

    #[inline]
    fn text_end(&mut self) -> Result {
        self.0.text_end()
    }

    #[inline]
    fn binary_begin(&mut self, num_bytes: Option<usize>) -> Result {
        self.0.binary_begin(num_bytes)
    }

    #[inline]
    fn binary_fragment_computed(&mut self, fragment: &[u8]) -> Result {
        self.0.binary_fragment_computed(fragment)
    }

    #[inline]
    fn binary_end(&mut self) -> Result {
        self.0.binary_end()
    }

    #[inline]
    fn map_begin(&mut self, num_entries: Option<usize>) -> Result {
        self.0.map_begin(num_entries)
    }

    #[inline]
    fn map_key_begin(&mut self) -> Result {
        self.0.map_key_begin()
    }

    #[inline]
    fn map_key_end(&mut self) -> Result {
        self.0.map_key_end()
    }

    #[inline]
    fn map_value_begin(&mut self) -> Result {
        self.0.map_value_begin()
    }

    #[inline]
    fn map_value_end(&mut self) -> Result {
        self.0.map_value_end()
    }

    #[inline]
    fn map_end(&mut self) -> Result {
        self.0.map_end()
    }

    #[inline]
    fn seq_begin(&mut self, num_entries: Option<usize>) -> Result {
        self.0.seq_begin(num_entries)
    }

    #[inline]
    fn seq_value_begin(&mut self) -> Result {
        self.0.seq_value_begin()
    }

    #[inline]
    fn seq_value_end(&mut self) -> Result {
        self.0.seq_value_end()
    }

    #[inline]
    fn seq_end(&mut self) -> Result {
        self.0.seq_end()
    }

    #[inline]
    fn tagged_begin(
        &mut self,
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
    ) -> Result {
        self.0.tagged_begin(tag, label, index)
    }

    #[inline]
    fn tagged_end(
        &mut self,
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
    ) -> Result {
        self.0.tagged_end(tag, label, index)
    }

    #[inline]
    fn tag(&mut self, tag: Option<&Tag>, label: Option<&Label>, index: Option<&Index>) -> Result {
        self.0.tag(tag, label, index)
    }

    #[inline]
    fn record_begin(
        &mut self,
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
        num_entries: Option<usize>,
    ) -> Result {
        self.0.record_begin(tag, label, index, num_entries)
    }

    #[inline]
    fn record_value_begin(&mut self, tag: Option<&Tag>, label: &Label) -> Result {
        self.0.record_value_begin(tag, label)
    }

    #[inline]
    fn record_value_end(&mut self, tag: Option<&Tag>, label: &Label) -> Result {
        self.0.record_value_end(tag, label)
    }

    #[inline]
    fn record_end(
        &mut self,
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
    ) -> Result {
        self.0.record_end(tag, label, index)
    }

    #[inline]
    fn tuple_begin(
        &mut self,
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
        num_entries: Option<usize>,
    ) -> Result {
        self.0.tuple_begin(tag, label, index, num_entries)
    }

    #[inline]
    fn tuple_value_begin(&mut self, tag: Option<&Tag>, index: &Index) -> Result {
        self.0.tuple_value_begin(tag, index)
    }

    #[inline]
    fn tuple_value_end(&mut self, tag: Option<&Tag>, index: &Index) -> Result {
        self.0.tuple_value_end(tag, index)
    }

    #[inline]
    fn tuple_end(
        &mut self,
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
    ) -> Result {
        self.0.tuple_end(tag, label, index)
    }

    #[inline]
    fn record_tuple_begin(
        &mut self,
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
        num_entries: Option<usize>,
    ) -> Result {
        self.0.record_tuple_begin(tag, label, index, num_entries)
    }

    #[inline]
    fn record_tuple_value_begin(
        &mut self,
        tag: Option<&Tag>,
        label: &Label,
        index: &Index,
    ) -> Result {
        self.0.record_tuple_value_begin(tag, label, index)
    }

    #[inline]
    fn record_tuple_value_end(
        &mut self,
        tag: Option<&Tag>,
        label: &Label,
        index: &Index,
    ) -> Result {
        self.0.record_tuple_value_end(tag, label, index)
    }

    #[inline]
    fn record_tuple_end(
        &mut self,
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
    ) -> Result {
        self.0.record_tuple_end(tag, label, index)
    }

    #[inline]
    fn enum_begin(
        &mut self,
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
    ) -> Result {
        self.0.enum_begin(tag, label, index)
    }

    #[inline]
    fn enum_end(
        &mut self,
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
    ) -> Result {
        self.0.enum_end(tag, label, index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_computed() {
        struct ComputedValue(usize);

        impl Value for ComputedValue {
            fn stream<'sval, S: Stream<'sval> + ?Sized>(&'sval self, stream: &mut S) -> Result {
                if self.0 == 0 {
                    stream.bool(true)
                } else {
                    stream.value_computed(&ComputedValue(self.0 - 1))
                }
            }
        }

        assert_eq!(true, ComputedValue(5).to_bool().unwrap());
    }
}
