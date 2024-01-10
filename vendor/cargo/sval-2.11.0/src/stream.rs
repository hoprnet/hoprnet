use crate::{data, tags, Index, Label, Result, Tag, Value};

/**
A consumer of structured data.
*/
pub trait Stream<'sval> {
    /**
    Recurse into a nested value.
    */
    fn value<V: Value + ?Sized>(&mut self, v: &'sval V) -> Result {
        default_stream::value(self, v)
    }

    /**
    Recurse into a nested value, borrowed for some arbitrarily short lifetime.
    */
    fn value_computed<V: Value + ?Sized>(&mut self, v: &V) -> Result {
        default_stream::value_computed(self, v)
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
        default_stream::text_fragment(self, fragment)
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
        default_stream::binary_begin(self, num_bytes)
    }

    /**
    Stream a fragment of a bitstring.
    */
    #[inline]
    fn binary_fragment(&mut self, fragment: &'sval [u8]) -> Result {
        default_stream::binary_fragment(self, fragment)
    }

    /**
    Stream a fragment of a bitstring, borrowed for some arbitrarily short lifetime.
    */
    fn binary_fragment_computed(&mut self, fragment: &[u8]) -> Result {
        default_stream::binary_fragment_computed(self, fragment)
    }

    /**
    Complete a bitstring.
    */
    #[inline]
    fn binary_end(&mut self) -> Result {
        default_stream::binary_end(self)
    }

    /**
    Stream an unsigned 8bit integer.
    */
    #[inline]
    fn u8(&mut self, value: u8) -> Result {
        default_stream::u8(self, value)
    }

    /**
    Stream an unsigned 16bit integer.
    */
    #[inline]
    fn u16(&mut self, value: u16) -> Result {
        default_stream::u16(self, value)
    }

    /**
    Stream an unsigned 32bit integer.
    */
    #[inline]
    fn u32(&mut self, value: u32) -> Result {
        default_stream::u32(self, value)
    }

    /**
    Stream an unsigned 64bit integer.
    */
    #[inline]
    fn u64(&mut self, value: u64) -> Result {
        default_stream::u64(self, value)
    }

    /**
    Stream an unsigned 128bit integer.
    */
    fn u128(&mut self, value: u128) -> Result {
        default_stream::u128(self, value)
    }

    /**
    Stream a signed 8bit integer.
    */
    #[inline]
    fn i8(&mut self, value: i8) -> Result {
        default_stream::i8(self, value)
    }

    /**
    Stream a signed 16bit integer.
    */
    #[inline]
    fn i16(&mut self, value: i16) -> Result {
        default_stream::i16(self, value)
    }

    /**
    Stream a signed 32bit integer.
    */
    #[inline]
    fn i32(&mut self, value: i32) -> Result {
        default_stream::i32(self, value)
    }

    /**
    Stream a signed 64bit integer.
    */
    fn i64(&mut self, value: i64) -> Result;

    /**
    Stream a signed 128bit integer.
    */
    fn i128(&mut self, value: i128) -> Result {
        default_stream::i128(self, value)
    }

    /**
    Stream a 32bit binary floating point number.
    */
    #[inline]
    fn f32(&mut self, value: f32) -> Result {
        default_stream::f32(self, value)
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
        default_stream::map_begin(self, num_entries)
    }

    /**
    Start a key in a key-value mapping.
    */
    #[inline]
    fn map_key_begin(&mut self) -> Result {
        default_stream::map_key_begin(self)
    }

    /**
    Complete a key in a key-value mapping.
    */
    #[inline]
    fn map_key_end(&mut self) -> Result {
        default_stream::map_key_end(self)
    }

    /**
    Start a value in a key-value mapping.
    */
    #[inline]
    fn map_value_begin(&mut self) -> Result {
        default_stream::map_value_begin(self)
    }

    /**
    Complete a value in a key-value mapping.
    */
    #[inline]
    fn map_value_end(&mut self) -> Result {
        default_stream::map_value_end(self)
    }

    /**
    Complete a homogenous mapping of arbitrary keys to values.
    */
    #[inline]
    fn map_end(&mut self) -> Result {
        default_stream::map_end(self)
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
        default_stream::enum_begin(self, tag, label, index)
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
        default_stream::enum_end(self, tag, label, index)
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
        default_stream::tagged_begin(self, tag, label, index)
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
        default_stream::tagged_end(self, tag, label, index)
    }

    /**
    Stream a standalone tag.

    Standalone tags may be used as enum variants.
    */
    fn tag(&mut self, tag: Option<&Tag>, label: Option<&Label>, index: Option<&Index>) -> Result {
        default_stream::tag(self, tag, label, index)
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
        default_stream::record_begin(self, tag, label, index, num_entries)
    }

    /**
    Start a field in a record.
    */
    #[inline]
    fn record_value_begin(&mut self, tag: Option<&Tag>, label: &Label) -> Result {
        default_stream::record_value_begin(self, tag, label)
    }

    /**
    Complete a field in a record.
    */
    #[inline]
    fn record_value_end(&mut self, tag: Option<&Tag>, label: &Label) -> Result {
        default_stream::record_value_end(self, tag, label)
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
        default_stream::record_end(self, tag, label, index)
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
        default_stream::tuple_begin(self, tag, label, index, num_entries)
    }

    /**
    Start a field in a tuple.
    */
    #[inline]
    fn tuple_value_begin(&mut self, tag: Option<&Tag>, index: &Index) -> Result {
        default_stream::tuple_value_begin(self, tag, index)
    }

    /**
    Complete a field in a tuple.
    */
    #[inline]
    fn tuple_value_end(&mut self, tag: Option<&Tag>, index: &Index) -> Result {
        default_stream::tuple_value_end(self, tag, index)
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
        default_stream::tuple_end(self, tag, label, index)
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
        default_stream::record_tuple_begin(self, tag, label, index, num_entries)
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
        default_stream::record_tuple_value_begin(self, tag, label, index)
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
        default_stream::record_tuple_value_end(self, tag, label, index)
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
        default_stream::record_tuple_end(self, tag, label, index)
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
    fn value<V: Value + ?Sized>(&mut self, v: &'b V) -> Result {
        default_stream::value(self, v)
    }

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

pub mod default_stream {
    /*!
    Default method implementations for [`Stream`]s.
    */

    use super::*;

    /**
    Recurse into a nested value.
    */
    pub fn value<'sval>(
        stream: &mut (impl Stream<'sval> + ?Sized),
        v: &'sval (impl Value + ?Sized),
    ) -> Result {
        v.stream(stream)
    }

    /**
    Recurse into a nested value, borrowed for some arbitrarily short lifetime.
    */
    pub fn value_computed<'a, 'b>(
        stream: &mut (impl Stream<'a> + ?Sized),
        v: &'b (impl Value + ?Sized),
    ) -> Result {
        v.stream(Computed::new_borrowed(stream))
    }

    /**
    Stream a fragment of UTF8 text.
    */
    pub fn text_fragment<'sval>(
        stream: &mut (impl Stream<'sval> + ?Sized),
        fragment: &'sval str,
    ) -> Result {
        stream.text_fragment_computed(fragment)
    }

    /**
    Start a bitstring.
    */
    pub fn binary_begin<'sval>(
        stream: &mut (impl Stream<'sval> + ?Sized),
        num_bytes: Option<usize>,
    ) -> Result {
        stream.seq_begin(num_bytes)
    }

    /**
    Stream a fragment of a bitstring.
    */
    pub fn binary_fragment<'sval>(
        stream: &mut (impl Stream<'sval> + ?Sized),
        fragment: &'sval [u8],
    ) -> Result {
        stream.binary_fragment_computed(fragment)
    }

    /**
    Stream a fragment of a bitstring, borrowed for some arbitrarily short lifetime.
    */
    pub fn binary_fragment_computed<'sval>(
        stream: &mut (impl Stream<'sval> + ?Sized),
        fragment: &[u8],
    ) -> Result {
        for byte in fragment {
            stream.seq_value_begin()?;
            stream.u8(*byte)?;
            stream.seq_value_end()?;
        }

        Ok(())
    }

    /**
    Complete a bitstring.
    */
    pub fn binary_end<'sval>(stream: &mut (impl Stream<'sval> + ?Sized)) -> Result {
        stream.seq_end()
    }

    /**
    Stream an unsigned 8bit integer.
    */
    pub fn u8<'sval>(stream: &mut (impl Stream<'sval> + ?Sized), value: u8) -> Result {
        stream.u16(value as u16)
    }

    /**
    Stream an unsigned 16bit integer.
    */
    pub fn u16<'sval>(stream: &mut (impl Stream<'sval> + ?Sized), value: u16) -> Result {
        stream.u32(value as u32)
    }

    /**
    Stream an unsigned 32bit integer.
    */
    pub fn u32<'sval>(stream: &mut (impl Stream<'sval> + ?Sized), value: u32) -> Result {
        stream.u64(value as u64)
    }

    /**
    Stream an unsigned 64bit integer.
    */
    pub fn u64<'sval>(stream: &mut (impl Stream<'sval> + ?Sized), value: u64) -> Result {
        stream.u128(value as u128)
    }

    /**
    Stream an unsigned 128bit integer.
    */
    pub fn u128<'sval>(stream: &mut (impl Stream<'sval> + ?Sized), value: u128) -> Result {
        if let Ok(value) = value.try_into() {
            stream.i64(value)
        } else {
            data::stream_u128(value, stream)
        }
    }

    /**
    Stream a signed 8bit integer.
    */
    pub fn i8<'sval>(stream: &mut (impl Stream<'sval> + ?Sized), value: i8) -> Result {
        stream.i16(value as i16)
    }

    /**
    Stream a signed 16bit integer.
    */
    pub fn i16<'sval>(stream: &mut (impl Stream<'sval> + ?Sized), value: i16) -> Result {
        stream.i32(value as i32)
    }

    /**
    Stream a signed 32bit integer.
    */
    pub fn i32<'sval>(stream: &mut (impl Stream<'sval> + ?Sized), value: i32) -> Result {
        stream.i64(value as i64)
    }

    /**
    Stream a signed 128bit integer.
    */
    pub fn i128<'sval>(stream: &mut (impl Stream<'sval> + ?Sized), value: i128) -> Result {
        if let Ok(value) = value.try_into() {
            stream.i64(value)
        } else {
            data::stream_i128(value, stream)
        }
    }

    /**
    Stream a 32bit binary floating point number.
    */
    pub fn f32<'sval>(stream: &mut (impl Stream<'sval> + ?Sized), value: f32) -> Result {
        stream.f64(value as f64)
    }

    /**
    Start a homogenous mapping of arbitrary keys to values.
    */
    pub fn map_begin<'sval>(
        stream: &mut (impl Stream<'sval> + ?Sized),
        num_entries: Option<usize>,
    ) -> Result {
        stream.seq_begin(num_entries)
    }

    /**
    Start a key in a key-value mapping.
    */
    pub fn map_key_begin<'sval>(stream: &mut (impl Stream<'sval> + ?Sized)) -> Result {
        stream.seq_value_begin()?;
        stream.tuple_begin(None, None, None, Some(2))?;
        stream.tuple_value_begin(None, &Index::new(0))
    }

    /**
    Complete a key in a key-value mapping.
    */
    pub fn map_key_end<'sval>(stream: &mut (impl Stream<'sval> + ?Sized)) -> Result {
        stream.tuple_value_end(None, &Index::new(0))
    }

    /**
    Start a value in a key-value mapping.
    */
    pub fn map_value_begin<'sval>(stream: &mut (impl Stream<'sval> + ?Sized)) -> Result {
        stream.tuple_value_begin(None, &Index::new(1))
    }

    /**
    Complete a value in a key-value mapping.
    */
    pub fn map_value_end<'sval>(stream: &mut (impl Stream<'sval> + ?Sized)) -> Result {
        stream.tuple_value_end(None, &Index::new(1))?;
        stream.tuple_end(None, None, None)?;
        stream.seq_value_end()
    }

    /**
    Complete a homogenous mapping of arbitrary keys to values.
    */
    pub fn map_end<'sval>(stream: &mut (impl Stream<'sval> + ?Sized)) -> Result {
        stream.seq_end()
    }

    /**
    Start a variant in an enumerated type.
    */
    pub fn enum_begin<'sval>(
        stream: &mut (impl Stream<'sval> + ?Sized),
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
    ) -> Result {
        stream.tagged_begin(tag, label, index)
    }

    /**
    Complete a variant in an enumerated type.
    */
    pub fn enum_end<'sval>(
        stream: &mut (impl Stream<'sval> + ?Sized),
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
    ) -> Result {
        stream.tagged_end(tag, label, index)
    }

    /**
    Start a tagged value.

    Tagged values may be used as enum variants.
    */
    pub fn tagged_begin<'sval>(
        stream: &mut (impl Stream<'sval> + ?Sized),
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
    ) -> Result {
        let _ = stream;
        let _ = tag;
        let _ = label;
        let _ = index;

        Ok(())
    }

    /**
    Complete a tagged value.
    */
    pub fn tagged_end<'sval>(
        stream: &mut (impl Stream<'sval> + ?Sized),
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
    ) -> Result {
        let _ = stream;
        let _ = tag;
        let _ = label;
        let _ = index;

        Ok(())
    }

    /**
    Stream a standalone tag.

    Standalone tags may be used as enum variants.
    */
    pub fn tag<'sval>(
        stream: &mut (impl Stream<'sval> + ?Sized),
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
    ) -> Result {
        stream.tagged_begin(tag, label, index)?;

        // Rust's `Option` is fundamental enough that we handle it specially here
        if let Some(&tags::RUST_OPTION_NONE) = tag {
            stream.null()?;
        }
        // If the tag has a label then stream it as its value
        else if let Some(ref label) = label {
            if let Some(label) = label.as_static_str() {
                stream.value(label)?;
            } else {
                stream.value_computed(label.as_str())?;
            }
        }
        // If the tag doesn't have a label then stream null
        else {
            stream.null()?;
        }

        stream.tagged_end(tag, label, index)
    }

    /**
    Start a record type.

    Records may be used as enum variants.
    */
    pub fn record_begin<'sval>(
        stream: &mut (impl Stream<'sval> + ?Sized),
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
        num_entries: Option<usize>,
    ) -> Result {
        stream.tagged_begin(tag, label, index)?;
        stream.map_begin(num_entries)
    }

    /**
    Start a field in a record.
    */
    pub fn record_value_begin<'sval>(
        stream: &mut (impl Stream<'sval> + ?Sized),
        tag: Option<&Tag>,
        label: &Label,
    ) -> Result {
        let _ = tag;

        stream.map_key_begin()?;

        if let Some(label) = label.as_static_str() {
            stream.value(label)?;
        } else {
            stream.value_computed(label.as_str())?;
        }

        stream.map_key_end()?;

        stream.map_value_begin()
    }

    /**
    Complete a field in a record.
    */
    pub fn record_value_end<'sval>(
        stream: &mut (impl Stream<'sval> + ?Sized),
        tag: Option<&Tag>,
        label: &Label,
    ) -> Result {
        let _ = tag;
        let _ = label;

        stream.map_value_end()
    }

    /**
    Complete a record type.
    */
    pub fn record_end<'sval>(
        stream: &mut (impl Stream<'sval> + ?Sized),
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
    ) -> Result {
        stream.map_end()?;
        stream.tagged_end(tag, label, index)
    }

    /**
    Start a tuple type.

    Tuples may be used as enum variants.
    */
    pub fn tuple_begin<'sval>(
        stream: &mut (impl Stream<'sval> + ?Sized),
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
        num_entries: Option<usize>,
    ) -> Result {
        stream.tagged_begin(tag, label, index)?;
        stream.seq_begin(num_entries)
    }

    /**
    Start a field in a tuple.
    */
    pub fn tuple_value_begin<'sval>(
        stream: &mut (impl Stream<'sval> + ?Sized),
        tag: Option<&Tag>,
        index: &Index,
    ) -> Result {
        let _ = tag;
        let _ = index;

        stream.seq_value_begin()
    }

    /**
    Complete a field in a tuple.
    */
    pub fn tuple_value_end<'sval>(
        stream: &mut (impl Stream<'sval> + ?Sized),
        tag: Option<&Tag>,
        index: &Index,
    ) -> Result {
        let _ = tag;
        let _ = index;

        stream.seq_value_end()
    }

    /**
    Complete a tuple type.
    */
    pub fn tuple_end<'sval>(
        stream: &mut (impl Stream<'sval> + ?Sized),
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
    ) -> Result {
        stream.seq_end()?;
        stream.tagged_end(tag, label, index)
    }

    /**
    Begin a type that may be treated as either a record or a tuple.
    */
    pub fn record_tuple_begin<'sval>(
        stream: &mut (impl Stream<'sval> + ?Sized),
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
        num_entries: Option<usize>,
    ) -> Result {
        stream.record_begin(tag, label, index, num_entries)
    }

    /**
    Begin a field in a type that may be treated as either a record or a tuple.
    */
    pub fn record_tuple_value_begin<'sval>(
        stream: &mut (impl Stream<'sval> + ?Sized),
        tag: Option<&Tag>,
        label: &Label,
        index: &Index,
    ) -> Result {
        let _ = index;

        stream.record_value_begin(tag, label)
    }

    /**
    Complete a field in a type that may be treated as either a record or a tuple.
    */
    pub fn record_tuple_value_end<'sval>(
        stream: &mut (impl Stream<'sval> + ?Sized),
        tag: Option<&Tag>,
        label: &Label,
        index: &Index,
    ) -> Result {
        let _ = index;

        stream.record_value_end(tag, label)
    }

    /**
    Complete a type that may be treated as either a record or a tuple.
    */
    pub fn record_tuple_end<'sval>(
        stream: &mut (impl Stream<'sval> + ?Sized),
        tag: Option<&Tag>,
        label: Option<&Label>,
        index: Option<&Index>,
    ) -> Result {
        stream.record_end(tag, label, index)
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
