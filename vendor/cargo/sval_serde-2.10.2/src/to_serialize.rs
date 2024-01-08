use core::mem;

use serde::ser::{
    Error as _, Serialize as _, SerializeMap as _, SerializeSeq as _, SerializeStruct as _,
    SerializeStructVariant as _, SerializeTuple as _, SerializeTupleStruct as _,
    SerializeTupleVariant as _,
};

use sval_buffer::{BinaryBuf, TextBuf, ValueBuf};

/**
Serialize an [`sval::Value`] into a [`serde::Serializer`].
*/
pub fn serialize<S: serde::Serializer>(
    serializer: S,
    value: impl sval::Value,
) -> Result<S::Ok, S::Error> {
    ToSerialize::new(value).serialize(serializer)
}

/**
Adapt an [`sval::Value`] into a [`serde::Serialize`].
*/
#[repr(transparent)]
pub struct ToSerialize<V: ?Sized>(V);

impl<V: sval::Value> ToSerialize<V> {
    /**
    Adapt an [`sval::Value`] into a [`serde::Serialize`].
    */
    pub const fn new(value: V) -> ToSerialize<V> {
        ToSerialize(value)
    }
}

impl<V: sval::Value + ?Sized> ToSerialize<V> {
    /**
    Adapt a reference to an [`sval::Value`] into a [`serde::Serialize`].
    */
    pub const fn new_borrowed<'a>(value: &'a V) -> &'a ToSerialize<V> {
        // SAFETY: `&'a V` and `&'a ToSerialize<V>` have the same ABI
        unsafe { &*(value as *const _ as *const ToSerialize<V>) }
    }
}

impl<V: sval::Value> serde::Serialize for ToSerialize<V> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut stream = Serializer {
            buffered: None,
            state: State::Any(Some(Any {
                serializer,
                is_option: false,
                struct_label: None,
                variant_label: None,
                variant_index: None,
            })),
        };

        let _ = self.0.stream(&mut stream);

        stream.finish()
    }
}

struct Serializer<'sval, S: serde::Serializer> {
    buffered: Option<Buffered<'sval>>,
    state: State<S>,
}

impl<'sval, S: serde::Serializer> sval::Stream<'sval> for Serializer<'sval, S> {
    fn value<V: sval::Value + ?Sized>(&mut self, value: &'sval V) -> sval::Result {
        self.buffer_or_value(|buf| buf.value(value), || ToSerialize(value))
    }

    fn value_computed<V: sval::Value + ?Sized>(&mut self, value: &V) -> sval::Result {
        self.buffer_or_value(|buf| buf.value_computed(value), || ToSerialize(value))
    }

    fn null(&mut self) -> sval::Result {
        self.buffer_or_value(|buf| buf.null(), || None::<()>)
    }

    fn bool(&mut self, value: bool) -> sval::Result {
        self.buffer_or_value(|buf| buf.bool(value), || value)
    }

    fn u8(&mut self, value: u8) -> sval::Result {
        self.buffer_or_value(|buf| buf.u8(value), || value)
    }

    fn u16(&mut self, value: u16) -> sval::Result {
        self.buffer_or_value(|buf| buf.u16(value), || value)
    }

    fn u32(&mut self, value: u32) -> sval::Result {
        self.buffer_or_value(|buf| buf.u32(value), || value)
    }

    fn u64(&mut self, value: u64) -> sval::Result {
        self.buffer_or_value(|buf| buf.u64(value), || value)
    }

    fn u128(&mut self, value: u128) -> sval::Result {
        self.buffer_or_value(|buf| buf.u128(value), || value)
    }

    fn i8(&mut self, value: i8) -> sval::Result {
        self.buffer_or_value(|buf| buf.i8(value), || value)
    }

    fn i16(&mut self, value: i16) -> sval::Result {
        self.buffer_or_value(|buf| buf.i16(value), || value)
    }

    fn i32(&mut self, value: i32) -> sval::Result {
        self.buffer_or_value(|buf| buf.i32(value), || value)
    }

    fn i64(&mut self, value: i64) -> sval::Result {
        self.buffer_or_value(|buf| buf.i64(value), || value)
    }

    fn i128(&mut self, value: i128) -> sval::Result {
        self.buffer_or_value(|buf| buf.i128(value), || value)
    }

    fn f32(&mut self, value: f32) -> sval::Result {
        self.buffer_or_value(|buf| buf.f32(value), || value)
    }

    fn f64(&mut self, value: f64) -> sval::Result {
        self.buffer_or_value(|buf| buf.f64(value), || value)
    }

    fn text_begin(&mut self, size_hint: Option<usize>) -> sval::Result {
        self.buffer_or_serialize_with(
            |buf| buf.text_begin(size_hint),
            |serializer| serializer.put_buffer(Buffered::Text(TextBuf::new())),
        )
    }

    fn text_fragment(&mut self, fragment: &'sval str) -> sval::Result {
        self.buffer_or_serialize_with(
            |buf| buf.text_fragment(fragment),
            |serializer| serializer.with_text(|text| text.push_fragment(fragment)),
        )
    }

    fn text_fragment_computed(&mut self, fragment: &str) -> sval::Result {
        self.buffer_or_serialize_with(
            |buf| buf.text_fragment_computed(fragment),
            |serializer| serializer.with_text(|text| text.push_fragment_computed(fragment)),
        )
    }

    fn text_end(&mut self) -> sval::Result {
        self.buffer_or_serialize_with(
            |buf| buf.text_end(),
            |serializer| {
                let buf = serializer.take_text()?;

                serializer.state.serialize_value(buf.as_str())
            },
        )
    }

    fn binary_begin(&mut self, size_hint: Option<usize>) -> sval::Result {
        self.buffer_or_serialize_with(
            |buf| buf.binary_begin(size_hint),
            |serializer| serializer.put_buffer(Buffered::Binary(BinaryBuf::new())),
        )
    }

    fn binary_fragment(&mut self, fragment: &'sval [u8]) -> sval::Result {
        self.buffer_or_serialize_with(
            |buf| buf.binary_fragment(fragment),
            |serializer| serializer.with_binary(|binary| binary.push_fragment(fragment)),
        )
    }

    fn binary_fragment_computed(&mut self, fragment: &[u8]) -> sval::Result {
        self.buffer_or_serialize_with(
            |buf| buf.binary_fragment_computed(fragment),
            |serializer| serializer.with_binary(|binary| binary.push_fragment_computed(fragment)),
        )
    }

    fn binary_end(&mut self) -> sval::Result {
        self.buffer_or_serialize_with(
            |buf| buf.binary_end(),
            |serializer| {
                let buf = serializer.take_binary()?;

                serializer.state.serialize_value(Bytes(buf.as_slice()))
            },
        )
    }

    fn map_begin(&mut self, num_entries_hint: Option<usize>) -> sval::Result {
        self.buffer_or_transition_any_with(
            |buf| buf.map_begin(num_entries_hint),
            |serializer| {
                Ok(State::Map(Some(Map {
                    serializer: serializer.serializer.serialize_map(num_entries_hint)?,
                    is_key: true,
                })))
            },
        )
    }

    fn map_key_begin(&mut self) -> sval::Result {
        self.buffer_or_serialize_with(
            |buf| buf.map_key_begin(),
            |serializer| {
                serializer.with_map(|serializer| {
                    serializer.is_key = true;

                    Ok(())
                })
            },
        )
    }

    fn map_key_end(&mut self) -> sval::Result {
        self.buffer_or_serialize_with(|buf| buf.map_key_end(), |_| Ok(()))
    }

    fn map_value_begin(&mut self) -> sval::Result {
        self.buffer_or_serialize_with(
            |buf| buf.map_value_begin(),
            |serializer| {
                serializer.with_map(|serializer| {
                    serializer.is_key = false;

                    Ok(())
                })
            },
        )
    }

    fn map_value_end(&mut self) -> sval::Result {
        self.buffer_or_serialize_with(|buf| buf.map_value_end(), |_| Ok(()))
    }

    fn map_end(&mut self) -> sval::Result {
        self.buffer_or_transition_done_with(
            |buf| buf.map_end(),
            |serializer| serializer.take_map()?.serializer.end(),
        )
    }

    fn seq_begin(&mut self, num_entries_hint: Option<usize>) -> sval::Result {
        self.buffer_or_transition_any_with(
            |buf| buf.seq_begin(num_entries_hint),
            |serializer| {
                Ok(State::Seq(Some(Seq {
                    serializer: serializer.serializer.serialize_seq(num_entries_hint)?,
                })))
            },
        )
    }

    fn seq_value_begin(&mut self) -> sval::Result {
        self.buffer_or_serialize_with(|buf| buf.seq_value_begin(), |_| Ok(()))
    }

    fn seq_value_end(&mut self) -> sval::Result {
        self.buffer_or_serialize_with(|buf| buf.seq_value_begin(), |_| Ok(()))
    }

    fn seq_end(&mut self) -> sval::Result {
        self.buffer_or_transition_done_with(
            |buf| buf.seq_end(),
            |serializer| serializer.take_seq()?.serializer.end(),
        )
    }

    fn enum_begin(
        &mut self,
        tag: Option<&sval::Tag>,
        label: Option<&sval::Label>,
        index: Option<&sval::Index>,
    ) -> sval::Result {
        self.buffer_or_transition_any_with(
            |buf| buf.enum_begin(tag, label, index),
            |mut serializer| {
                serializer.struct_label = label.and_then(|label| label.as_static_str());

                Ok(State::Any(Some(serializer)))
            },
        )
    }

    fn enum_end(
        &mut self,
        tag: Option<&sval::Tag>,
        label: Option<&sval::Label>,
        index: Option<&sval::Index>,
    ) -> sval::Result {
        self.buffer_or_transition_done_with(
            |buf| buf.enum_end(tag, label, index),
            |serializer| serializer.finish(),
        )
    }

    fn tagged_begin(
        &mut self,
        tag: Option<&sval::Tag>,
        label: Option<&sval::Label>,
        index: Option<&sval::Index>,
    ) -> sval::Result {
        self.buffer_or_transition_any_with(
            |buf| buf.tagged_begin(tag, label, index),
            |mut serializer| {
                match tag {
                    Some(&sval::tags::RUST_OPTION_SOME) => {
                        serializer.is_option = true;
                    }
                    _ => {
                        if serializer.struct_label.is_none() {
                            serializer.struct_label = label.and_then(|label| label.as_static_str());
                        } else {
                            serializer.variant_label =
                                label.and_then(|label| label.as_static_str());
                            serializer.variant_index = index.and_then(|index| index.to_u32());
                        }
                    }
                }

                Ok(State::Any(Some(serializer)))
            },
        )
    }

    fn tagged_end(
        &mut self,
        tag: Option<&sval::Tag>,
        label: Option<&sval::Label>,
        index: Option<&sval::Index>,
    ) -> sval::Result {
        self.buffer_or_transition_done_with(
            |buf| buf.tagged_end(tag, label, index),
            |serializer| serializer.finish(),
        )
    }

    fn tag(
        &mut self,
        tag: Option<&sval::Tag>,
        label: Option<&sval::Label>,
        index: Option<&sval::Index>,
    ) -> sval::Result {
        match tag {
            Some(&sval::tags::RUST_OPTION_NONE) => {
                self.buffer_or_value(|buf| buf.tag(tag, label, index), || None::<()>)
            }
            Some(&sval::tags::RUST_UNIT) => {
                self.buffer_or_value(|buf| buf.tag(tag, label, index), || ())
            }
            _ => self.buffer_or_transition_any_with(
                |buf| buf.tag(tag, label, index),
                |serializer| {
                    let unit_label = label
                        .and_then(|label| label.as_static_str())
                        .ok_or_else(|| S::Error::custom("missing unit label"))?;

                    let r = if let Some(struct_label) = serializer.struct_label {
                        let variant_index = index
                            .and_then(|index| index.to_u32())
                            .ok_or_else(|| S::Error::custom("missing variant index"))?;

                        serializer.serializer.serialize_unit_variant(
                            struct_label,
                            variant_index,
                            unit_label,
                        )
                    } else {
                        serializer.serializer.serialize_unit_struct(unit_label)
                    };

                    Ok(State::Done(Some(r)))
                },
            ),
        }
    }

    fn record_begin(
        &mut self,
        tag: Option<&sval::Tag>,
        label: Option<&sval::Label>,
        index: Option<&sval::Index>,
        num_entries: Option<usize>,
    ) -> sval::Result {
        self.buffer_or_transition_any_with(
            |buf| buf.record_begin(tag, label, index, num_entries),
            |serializer| {
                let record_label = label.and_then(|label| label.as_static_str());

                let num_entries =
                    num_entries.ok_or_else(|| S::Error::custom("missing struct field count"))?;

                match (serializer.struct_label, record_label) {
                    (Some(struct_label), Some(variant_label)) => {
                        let variant_index = index
                            .and_then(|index| index.to_u32())
                            .ok_or_else(|| S::Error::custom("missing variant index"))?;

                        Ok(State::Record(Some(Record::Variant(VariantRecord {
                            serializer: serializer.serializer.serialize_struct_variant(
                                struct_label,
                                variant_index,
                                variant_label,
                                num_entries,
                            )?,
                            label: None,
                        }))))
                    }
                    (None, Some(struct_label)) | (Some(struct_label), None) => {
                        Ok(State::Record(Some(Record::Struct(StructRecord {
                            serializer: serializer
                                .serializer
                                .serialize_struct(struct_label, num_entries)?,
                            label: None,
                        }))))
                    }
                    (None, None) => Ok(State::Record(Some(Record::Anonymous(AnonymousRecord {
                        serializer: serializer.serializer.serialize_map(Some(num_entries))?,
                    })))),
                }
            },
        )
    }

    fn record_value_begin(&mut self, tag: Option<&sval::Tag>, label: &sval::Label) -> sval::Result {
        self.buffer_or_serialize_with(
            |buf| buf.record_value_begin(tag, label),
            |serializer| {
                serializer.with_record(|serializer| match serializer {
                    Record::Anonymous(serializer) => {
                        serializer.serializer.serialize_key(label.as_str())?;

                        Ok(())
                    }
                    Record::Struct(serializer) => {
                        serializer.label = label.as_static_str();

                        Ok(())
                    }
                    Record::Variant(serializer) => {
                        serializer.label = label.as_static_str();

                        Ok(())
                    }
                })
            },
        )
    }

    fn record_value_end(&mut self, tag: Option<&sval::Tag>, label: &sval::Label) -> sval::Result {
        self.buffer_or_serialize_with(
            |buf| buf.record_value_end(tag, label),
            |serializer| {
                serializer.with_record(|serializer| match serializer {
                    Record::Anonymous(_) => Ok(()),
                    Record::Struct(serializer) => {
                        serializer.label = None;

                        Ok(())
                    }
                    Record::Variant(serializer) => {
                        serializer.label = None;

                        Ok(())
                    }
                })
            },
        )
    }

    fn record_end(
        &mut self,
        tag: Option<&sval::Tag>,
        label: Option<&sval::Label>,
        index: Option<&sval::Index>,
    ) -> sval::Result {
        self.buffer_or_transition_done_with(
            |buf| buf.record_end(tag, label, index),
            |serializer| match serializer.take_record()? {
                Record::Anonymous(serializer) => serializer.serializer.end(),
                Record::Struct(serializer) => serializer.serializer.end(),
                Record::Variant(serializer) => serializer.serializer.end(),
            },
        )
    }

    fn tuple_begin(
        &mut self,
        tag: Option<&sval::Tag>,
        label: Option<&sval::Label>,
        index: Option<&sval::Index>,
        num_entries: Option<usize>,
    ) -> sval::Result {
        self.buffer_or_transition_any_with(
            |buf| buf.tuple_begin(tag, label, index, num_entries),
            |serializer| {
                let tuple_label = label.and_then(|label| label.as_static_str());

                let num_entries =
                    num_entries.ok_or_else(|| S::Error::custom("missing tuple field count"))?;

                match (serializer.struct_label, tuple_label) {
                    (Some(struct_label), Some(variant_label)) => {
                        let variant_index = index
                            .and_then(|index| index.to_u32())
                            .ok_or_else(|| S::Error::custom("missing variant index"))?;

                        Ok(State::Tuple(Some(Tuple::Variant(VariantTuple {
                            serializer: serializer.serializer.serialize_tuple_variant(
                                struct_label,
                                variant_index,
                                variant_label,
                                num_entries,
                            )?,
                        }))))
                    }
                    (None, Some(struct_label)) | (Some(struct_label), None) => {
                        Ok(State::Tuple(Some(Tuple::Struct(StructTuple {
                            serializer: serializer
                                .serializer
                                .serialize_tuple_struct(struct_label, num_entries)?,
                        }))))
                    }
                    (None, None) => Ok(State::Tuple(Some(Tuple::Anonymous(AnonymousTuple {
                        serializer: serializer.serializer.serialize_tuple(num_entries)?,
                    })))),
                }
            },
        )
    }

    fn tuple_value_begin(&mut self, tag: Option<&sval::Tag>, index: &sval::Index) -> sval::Result {
        self.buffer_or_serialize_with(|buf| buf.tuple_value_begin(tag, index), |_| Ok(()))
    }

    fn tuple_value_end(&mut self, tag: Option<&sval::Tag>, index: &sval::Index) -> sval::Result {
        self.buffer_or_serialize_with(|buf| buf.tuple_value_end(tag, index), |_| Ok(()))
    }

    fn tuple_end(
        &mut self,
        tag: Option<&sval::Tag>,
        label: Option<&sval::Label>,
        index: Option<&sval::Index>,
    ) -> sval::Result {
        self.buffer_or_transition_done_with(
            |buf| buf.tuple_end(tag, label, index),
            |serializer| match serializer.take_tuple()? {
                Tuple::Anonymous(serializer) => serializer.serializer.end(),
                Tuple::Struct(serializer) => serializer.serializer.end(),
                Tuple::Variant(serializer) => serializer.serializer.end(),
            },
        )
    }
}

impl<S: serde::Serializer> State<S> {
    fn serialize_value<T: serde::Serialize>(&mut self, v: T) -> sval::Result {
        let mut r = || match self {
            State::Any(serializer) => {
                let serializer = serializer
                    .take()
                    .ok_or_else(|| S::Error::custom("missing serializer"))?;

                let r = match serializer {
                    Any {
                        is_option: true,
                        serializer,
                        ..
                    } => serializer.serialize_some(&v),
                    Any {
                        struct_label: Some(struct_label),
                        variant_label: None,
                        variant_index: None,
                        is_option: false,
                        serializer,
                    } => serializer.serialize_newtype_struct(struct_label, &v),
                    Any {
                        struct_label: Some(struct_label),
                        variant_label: Some(variant_label),
                        variant_index: Some(variant_index),
                        is_option: false,
                        serializer,
                    } => serializer.serialize_newtype_variant(
                        struct_label,
                        variant_index,
                        variant_label,
                        &v,
                    ),
                    Any { serializer, .. } => v.serialize(serializer),
                };

                match r {
                    Ok(r) => Ok(Some(r)),
                    Err(e) => Err(e),
                }
            }
            State::Map(serializer) => {
                let serializer = serializer
                    .as_mut()
                    .ok_or_else(|| S::Error::custom("missing serializer"))?;

                if serializer.is_key {
                    serializer.serializer.serialize_key(&v)?;

                    Ok(None)
                } else {
                    serializer.serializer.serialize_value(&v)?;

                    Ok(None)
                }
            }
            State::Seq(serializer) => {
                serializer
                    .as_mut()
                    .ok_or_else(|| S::Error::custom("missing serializer"))?
                    .serializer
                    .serialize_element(&v)?;

                Ok(None)
            }
            State::Record(serializer) => {
                let serializer = serializer
                    .as_mut()
                    .ok_or_else(|| S::Error::custom("missing serializer"))?;

                match serializer {
                    Record::Anonymous(serializer) => {
                        serializer.serializer.serialize_value(&v)?;
                    }
                    Record::Struct(serializer) => serializer.serializer.serialize_field(
                        serializer
                            .label
                            .ok_or_else(|| S::Error::custom("missing field label"))?,
                        &v,
                    )?,
                    Record::Variant(serializer) => serializer.serializer.serialize_field(
                        serializer
                            .label
                            .ok_or_else(|| S::Error::custom("missing field label"))?,
                        &v,
                    )?,
                }

                Ok(None)
            }
            State::Tuple(serializer) => {
                let serializer = serializer
                    .as_mut()
                    .ok_or_else(|| S::Error::custom("missing serializer"))?;

                match serializer {
                    Tuple::Anonymous(serializer) => serializer.serializer.serialize_element(&v)?,
                    Tuple::Struct(serializer) => serializer.serializer.serialize_field(&v)?,
                    Tuple::Variant(serializer) => serializer.serializer.serialize_field(&v)?,
                }

                Ok(None)
            }
            State::Done(_) => Err(S::Error::custom("already completed")),
        };

        match r() {
            Ok(Some(r)) => {
                *self = State::Done(Some(Ok(r)));
                Ok(())
            }
            Ok(None) => Ok(()),
            Err(e) => {
                *self = State::Done(Some(Err(e)));
                Err(sval::Error::new())
            }
        }
    }
}

fn try_catch<'sval, T, S: serde::Serializer>(
    serializer: &mut Serializer<'sval, S>,
    f: impl FnOnce(&mut Serializer<'sval, S>) -> Result<T, S::Error>,
) -> sval::Result<T> {
    match f(serializer) {
        Ok(v) => Ok(v),
        Err(e) => {
            serializer.state = State::Done(Some(Err(e)));

            sval::error()
        }
    }
}

impl<'sval, S: serde::Serializer> Serializer<'sval, S> {
    fn buffer_or_serialize_with(
        &mut self,
        buffer: impl FnOnce(&mut sval_buffer::ValueBuf<'sval>) -> sval::Result,
        stream: impl FnOnce(&mut Self) -> sval::Result,
    ) -> sval::Result {
        match self {
            Serializer {
                buffered: Some(Buffered::Value(ref mut buf)),
                ..
            } => buffer(buf),
            serializer => stream(serializer),
        }
    }

    fn buffer_or_value<T: serde::Serialize>(
        &mut self,
        buffer: impl FnOnce(&mut sval_buffer::ValueBuf<'sval>) -> sval::Result,
        value: impl FnOnce() -> T,
    ) -> sval::Result {
        self.buffer_or_serialize_with(buffer, |stream| stream.state.serialize_value(value()))
    }

    fn put_buffer(&mut self, buf: Buffered<'sval>) -> sval::Result {
        try_catch(self, |serializer| match serializer.buffered {
            None => {
                serializer.buffered = Some(buf);

                Ok(())
            }
            Some(_) => Err(S::Error::custom("a buffer is already active")),
        })
    }

    fn buffer_or_transition_any_with(
        &mut self,
        mut buffer: impl FnMut(&mut sval_buffer::ValueBuf<'sval>) -> sval::Result,
        transition: impl FnOnce(Any<S>) -> Result<State<S>, S::Error>,
    ) -> sval::Result {
        let buf = try_catch(self, |serializer| {
            match serializer {
                Serializer {
                    buffered: Some(Buffered::Value(ref mut buf)),
                    ..
                } => {
                    buffer(buf).map_err(|_| S::Error::custom("failed to buffer a value"))?;

                    return Ok(None);
                }
                Serializer {
                    buffered: None,
                    state: State::Any(any),
                } => {
                    if let Ok(state) = transition(
                        any.take()
                            .ok_or_else(|| S::Error::custom("missing serializer"))?,
                    ) {
                        serializer.state = state;

                        return Ok(None);
                    }
                }
                _ => return Err(S::Error::custom("invalid serializer state")),
            }

            let mut value = ValueBuf::new();
            buffer(&mut value).map_err(|_| S::Error::custom("failed to buffer a value"))?;

            Ok(Some(Buffered::Value(value)))
        })?;

        self.buffered = buf;

        Ok(())
    }

    fn buffer_or_transition_done_with(
        &mut self,
        buffer: impl FnOnce(&mut sval_buffer::ValueBuf<'sval>) -> sval::Result,
        transition: impl FnOnce(&mut Serializer<S>) -> Result<S::Ok, S::Error>,
    ) -> sval::Result {
        let r = try_catch(self, |serializer| match serializer {
            Serializer {
                buffered: Some(Buffered::Value(ref mut buf)),
                ..
            } => {
                buffer(buf).map_err(|_| S::Error::custom("failed to buffer a value"))?;

                if buf.is_complete() {
                    // Errors handled internally by `serialize_value`
                    let _ = serializer.state.serialize_value(ToSerialize(&*buf));
                    serializer.buffered = None;
                }

                return Ok(None);
            }
            Serializer { buffered: None, .. } => Ok(Some(transition(serializer)?)),
            _ => return Err(S::Error::custom("invalid serializer state")),
        })?;

        if let Some(r) = r {
            self.state = State::Done(Some(Ok(r)));
        }

        Ok(())
    }

    fn with_map(&mut self, f: impl FnOnce(&mut Map<S>) -> Result<(), S::Error>) -> sval::Result {
        try_catch(self, |serializer| match serializer {
            Serializer {
                buffered: None,
                state: State::Map(Some(map)),
            } => f(map),
            _ => Err(S::Error::custom("invalid serializer state")),
        })
    }

    fn take_map(&mut self) -> Result<Map<S>, S::Error> {
        match self {
            Serializer {
                buffered: None,
                state: State::Map(map),
            } => map
                .take()
                .ok_or_else(|| S::Error::custom("invalid serializer state")),
            _ => Err(S::Error::custom("invalid serializer state")),
        }
    }

    fn take_seq(&mut self) -> Result<Seq<S>, S::Error> {
        match self {
            Serializer {
                buffered: None,
                state: State::Seq(seq),
            } => seq
                .take()
                .ok_or_else(|| S::Error::custom("invalid serializer state")),
            _ => Err(S::Error::custom("invalid serializer state")),
        }
    }

    fn with_record(
        &mut self,
        f: impl FnOnce(&mut Record<S>) -> Result<(), S::Error>,
    ) -> sval::Result {
        try_catch(self, |serializer| match serializer {
            Serializer {
                buffered: None,
                state: State::Record(Some(s)),
            } => f(s),
            _ => Err(S::Error::custom("invalid serializer state")),
        })
    }

    fn take_record(&mut self) -> Result<Record<S>, S::Error> {
        match self {
            Serializer {
                buffered: None,
                state: State::Record(s),
            } => s
                .take()
                .ok_or_else(|| S::Error::custom("invalid serializer state")),
            _ => Err(S::Error::custom("invalid serializer state")),
        }
    }

    fn take_tuple(&mut self) -> Result<Tuple<S>, S::Error> {
        match self {
            Serializer {
                buffered: None,
                state: State::Tuple(s),
            } => s
                .take()
                .ok_or_else(|| S::Error::custom("invalid serializer state")),
            _ => Err(S::Error::custom("invalid serializer state")),
        }
    }

    fn with_text(
        &mut self,
        f: impl FnOnce(&mut TextBuf<'sval>) -> Result<(), sval_buffer::Error>,
    ) -> sval::Result {
        try_catch(self, |serializer| match serializer.buffered {
            Some(Buffered::Text(ref mut buf)) => f(buf).map_err(|e| S::Error::custom(e)),
            _ => Err(S::Error::custom("no active text buffer")),
        })
    }

    fn take_text(&mut self) -> sval::Result<TextBuf<'sval>> {
        try_catch(self, |serializer| match serializer.buffered {
            Some(Buffered::Text(ref mut buf)) => {
                let buf = mem::take(buf);
                serializer.buffered = None;

                Ok(buf)
            }
            _ => Err(S::Error::custom("no active text buffer")),
        })
    }

    fn with_binary(
        &mut self,
        f: impl FnOnce(&mut BinaryBuf<'sval>) -> Result<(), sval_buffer::Error>,
    ) -> sval::Result {
        try_catch(self, |serializer| match serializer.buffered {
            Some(Buffered::Binary(ref mut buf)) => f(buf).map_err(|e| S::Error::custom(e)),
            _ => Err(S::Error::custom("no active binary buffer")),
        })
    }

    fn take_binary(&mut self) -> sval::Result<BinaryBuf<'sval>> {
        try_catch(self, |serializer| match serializer.buffered {
            Some(Buffered::Binary(ref mut buf)) => {
                let buf = mem::take(buf);
                serializer.buffered = None;

                Ok(buf)
            }
            _ => Err(S::Error::custom("no active binary buffer")),
        })
    }

    fn finish(&mut self) -> Result<S::Ok, S::Error> {
        if let State::Done(ref mut r) = self.state {
            r.take()
                .unwrap_or_else(|| Err(S::Error::custom("incomplete serializer")))
        } else {
            Err(S::Error::custom("incomplete serializer"))
        }
    }
}

enum Buffered<'sval> {
    Text(TextBuf<'sval>),
    Binary(BinaryBuf<'sval>),
    Value(ValueBuf<'sval>),
}

enum State<S: serde::Serializer> {
    Any(Option<Any<S>>),
    Map(Option<Map<S>>),
    Seq(Option<Seq<S>>),
    Record(Option<Record<S>>),
    Tuple(Option<Tuple<S>>),
    Done(Option<Result<S::Ok, S::Error>>),
}

struct Any<S: serde::Serializer> {
    serializer: S,
    is_option: bool,
    struct_label: Option<&'static str>,
    variant_label: Option<&'static str>,
    variant_index: Option<u32>,
}

struct Map<S: serde::Serializer> {
    serializer: S::SerializeMap,
    is_key: bool,
}

struct Seq<S: serde::Serializer> {
    serializer: S::SerializeSeq,
}

struct AnonymousRecord<S: serde::Serializer> {
    serializer: S::SerializeMap,
}

struct StructRecord<S: serde::Serializer> {
    serializer: S::SerializeStruct,
    label: Option<&'static str>,
}
struct VariantRecord<S: serde::Serializer> {
    serializer: S::SerializeStructVariant,
    label: Option<&'static str>,
}

enum Record<S: serde::Serializer> {
    Anonymous(AnonymousRecord<S>),
    Struct(StructRecord<S>),
    Variant(VariantRecord<S>),
}

struct AnonymousTuple<S: serde::Serializer> {
    serializer: S::SerializeTuple,
}
struct StructTuple<S: serde::Serializer> {
    serializer: S::SerializeTupleStruct,
}
struct VariantTuple<S: serde::Serializer> {
    serializer: S::SerializeTupleVariant,
}

enum Tuple<S: serde::Serializer> {
    Anonymous(AnonymousTuple<S>),
    Struct(StructTuple<S>),
    Variant(VariantTuple<S>),
}

struct Bytes<'sval>(&'sval [u8]);

impl<'sval> serde::Serialize for Bytes<'sval> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.0)
    }
}
