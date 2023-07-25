//! Integration between `Value` and `sval`.
//!
//! This module allows any `Value` to implement the `Value` trait,
//! and for any `Value` to be captured as a `Value`.

use crate::{
    fill::Slot,
    internal::{Internal, InternalVisitor},
    std::{any::Any, fmt},
    Error, ValueBag,
};

impl<'v> ValueBag<'v> {
    /// Get a value from a structured type.
    ///
    /// This method will attempt to capture the given value as a well-known primitive
    /// before resorting to using its `Value` implementation.
    pub fn capture_sval2<T>(value: &'v T) -> Self
    where
        T: value_bag_sval2::lib::Value + 'static,
    {
        Self::try_capture(value).unwrap_or(ValueBag {
            inner: Internal::Sval2(value),
        })
    }

    /// Get a value from a structured type without capturing support.
    pub fn from_sval2<T>(value: &'v T) -> Self
    where
        T: value_bag_sval2::lib::Value,
    {
        ValueBag {
            inner: Internal::AnonSval2(value),
        }
    }

    /// Get a value from a structured type without capturing support.
    #[inline]
    pub fn from_dyn_sval2(value: &'v dyn Value) -> Self {
        ValueBag {
            inner: Internal::AnonSval2(value),
        }
    }
}

pub(crate) trait DowncastValue {
    fn as_any(&self) -> &dyn Any;
    fn as_super(&self) -> &dyn Value;
}

impl<T: value_bag_sval2::lib::Value + 'static> DowncastValue for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_super(&self) -> &dyn Value {
        self
    }
}

impl<'s, 'f> Slot<'s, 'f> {
    /// Fill the slot with a structured value.
    ///
    /// The given value doesn't need to satisfy any particular lifetime constraints.
    pub fn fill_sval2<T>(self, value: T) -> Result<(), Error>
    where
        T: value_bag_sval2::lib::Value,
    {
        self.fill(|visitor| visitor.sval2(&value))
    }
}

impl<'v> value_bag_sval2::lib::Value for ValueBag<'v> {
    fn stream<'sval, S: value_bag_sval2::lib::Stream<'sval> + ?Sized>(
        &'sval self,
        s: &mut S,
    ) -> value_bag_sval2::lib::Result {
        use value_bag_sval2::lib_ref::ValueRef as _;

        self.stream_ref(s)
    }
}

impl<'sval> value_bag_sval2::lib_ref::ValueRef<'sval> for ValueBag<'sval> {
    fn stream_ref<S: value_bag_sval2::lib::Stream<'sval> + ?Sized>(
        &self,
        s: &mut S,
    ) -> value_bag_sval2::lib::Result {
        struct Sval2Visitor<'a, S: ?Sized>(&'a mut S);

        impl<'a, 'v, S: value_bag_sval2::lib::Stream<'v> + ?Sized> InternalVisitor<'v>
            for Sval2Visitor<'a, S>
        {
            fn debug(&mut self, v: &dyn fmt::Debug) -> Result<(), Error> {
                value_bag_sval2::fmt::stream_debug(self.0, v).map_err(Error::from_sval2)
            }

            fn display(&mut self, v: &dyn fmt::Display) -> Result<(), Error> {
                value_bag_sval2::fmt::stream_display(self.0, v).map_err(Error::from_sval2)
            }

            fn u64(&mut self, v: u64) -> Result<(), Error> {
                self.0.u64(v).map_err(Error::from_sval2)
            }

            fn i64(&mut self, v: i64) -> Result<(), Error> {
                self.0.i64(v).map_err(Error::from_sval2)
            }

            fn u128(&mut self, v: &u128) -> Result<(), Error> {
                self.0.u128(*v).map_err(Error::from_sval2)
            }

            fn i128(&mut self, v: &i128) -> Result<(), Error> {
                self.0.i128(*v).map_err(Error::from_sval2)
            }

            fn f64(&mut self, v: f64) -> Result<(), Error> {
                self.0.f64(v).map_err(Error::from_sval2)
            }

            fn bool(&mut self, v: bool) -> Result<(), Error> {
                self.0.bool(v).map_err(Error::from_sval2)
            }

            fn char(&mut self, v: char) -> Result<(), Error> {
                let mut buf = [0; 4];
                let v = v.encode_utf8(&mut buf);

                self.0.value_computed(v).map_err(Error::from_sval2)
            }

            fn str(&mut self, v: &str) -> Result<(), Error> {
                self.0.value_computed(v).map_err(Error::from_sval2)
            }

            fn borrowed_str(&mut self, v: &'v str) -> Result<(), Error> {
                self.0.value(v).map_err(Error::from_sval2)
            }

            fn none(&mut self) -> Result<(), Error> {
                self.0.null().map_err(Error::from_sval2)
            }

            #[cfg(feature = "error")]
            fn error(&mut self, v: &(dyn std::error::Error + 'static)) -> Result<(), Error> {
                self.display(&v)
            }

            fn sval2(&mut self, v: &dyn Value) -> Result<(), Error> {
                self.0.value_computed(v).map_err(Error::from_sval2)
            }

            fn borrowed_sval2(&mut self, v: &'v dyn Value) -> Result<(), Error> {
                self.0.value(v).map_err(Error::from_sval2)
            }

            #[cfg(feature = "serde1")]
            fn serde1(
                &mut self,
                v: &dyn crate::internal::serde::v1::Serialize,
            ) -> Result<(), Error> {
                crate::internal::serde::v1::sval2(self.0, v)
            }
        }

        self.internal_visit(&mut Sval2Visitor(s))
            .map_err(Error::into_sval2)?;

        Ok(())
    }
}

pub use value_bag_sval2::dynamic::Value;

pub(in crate::internal) fn fmt(f: &mut fmt::Formatter, v: &dyn Value) -> Result<(), Error> {
    value_bag_sval2::fmt::stream_to_fmt(f, v)?;
    Ok(())
}

#[cfg(feature = "serde1")]
pub(in crate::internal) fn serde1<S>(s: S, v: &dyn Value) -> Result<S::Ok, S::Error>
where
    S: value_bag_serde1::lib::Serializer,
{
    value_bag_sval2::serde1::serialize(s, v)
}

pub(crate) fn internal_visit<'v>(
    v: &dyn Value,
    visitor: &mut dyn InternalVisitor<'v>,
) -> Result<(), Error> {
    let mut visitor = VisitorStream {
        visitor,
        text_buf: Default::default(),
    };

    value_bag_sval2::lib::stream_computed(&mut visitor, v).map_err(Error::from_sval2)?;

    Ok(())
}

pub(crate) fn borrowed_internal_visit<'v>(
    v: &'v dyn Value,
    visitor: &mut dyn InternalVisitor<'v>,
) -> Result<(), Error> {
    let mut visitor = VisitorStream {
        visitor,
        text_buf: Default::default(),
    };

    value_bag_sval2::lib::stream(&mut visitor, v).map_err(Error::from_sval2)?;

    Ok(())
}

struct VisitorStream<'a, 'v> {
    visitor: &'a mut dyn InternalVisitor<'v>,
    text_buf: value_bag_sval2::buffer::TextBuf<'v>,
}

impl<'a, 'v> value_bag_sval2::lib::Stream<'v> for VisitorStream<'a, 'v> {
    fn null(&mut self) -> value_bag_sval2::lib::Result {
        self.visitor.none().map_err(Error::into_sval2)
    }

    fn bool(&mut self, v: bool) -> value_bag_sval2::lib::Result {
        self.visitor.bool(v).map_err(Error::into_sval2)
    }

    fn i64(&mut self, v: i64) -> value_bag_sval2::lib::Result {
        self.visitor.i64(v).map_err(Error::into_sval2)
    }

    fn u64(&mut self, v: u64) -> value_bag_sval2::lib::Result {
        self.visitor.u64(v).map_err(Error::into_sval2)
    }

    fn i128(&mut self, v: i128) -> value_bag_sval2::lib::Result {
        self.visitor.i128(&v).map_err(Error::into_sval2)
    }

    fn u128(&mut self, v: u128) -> value_bag_sval2::lib::Result {
        self.visitor.u128(&v).map_err(Error::into_sval2)
    }

    fn f64(&mut self, v: f64) -> value_bag_sval2::lib::Result {
        self.visitor.f64(v).map_err(Error::into_sval2)
    }

    fn text_begin(&mut self, _: Option<usize>) -> value_bag_sval2::lib::Result {
        self.text_buf.clear();
        Ok(())
    }

    fn text_fragment_computed(&mut self, f: &str) -> value_bag_sval2::lib::Result {
        self.text_buf
            .push_fragment_computed(f)
            .map_err(|_| value_bag_sval2::lib::Error::new())
    }

    fn text_fragment(&mut self, f: &'v str) -> value_bag_sval2::lib::Result {
        self.text_buf
            .push_fragment(f)
            .map_err(|_| value_bag_sval2::lib::Error::new())
    }

    fn text_end(&mut self) -> value_bag_sval2::lib::Result {
        if let Some(v) = self.text_buf.as_borrowed_str() {
            self.visitor.borrowed_str(v).map_err(Error::into_sval2)
        } else {
            self.visitor
                .str(self.text_buf.as_str())
                .map_err(Error::into_sval2)
        }
    }

    fn seq_begin(&mut self, _: Option<usize>) -> value_bag_sval2::lib::Result {
        value_bag_sval2::lib::error()
    }

    fn seq_value_begin(&mut self) -> value_bag_sval2::lib::Result {
        value_bag_sval2::lib::error()
    }

    fn seq_value_end(&mut self) -> value_bag_sval2::lib::Result {
        value_bag_sval2::lib::error()
    }

    fn seq_end(&mut self) -> value_bag_sval2::lib::Result {
        value_bag_sval2::lib::error()
    }
}

impl Error {
    pub(in crate::internal) fn from_sval2(_: value_bag_sval2::lib::Error) -> Self {
        Error::msg("`sval` serialization failed")
    }

    pub(in crate::internal) fn into_sval2(self) -> value_bag_sval2::lib::Error {
        value_bag_sval2::lib::Error::new()
    }
}

#[cfg(feature = "owned")]
pub(crate) mod owned {
    impl value_bag_sval2::lib::Value for crate::OwnedValueBag {
        fn stream<'sval, S: value_bag_sval2::lib::Stream<'sval> + ?Sized>(
            &'sval self,
            s: &mut S,
        ) -> value_bag_sval2::lib::Result {
            value_bag_sval2::lib_ref::ValueRef::stream_ref(&self.by_ref(), s)
        }
    }

    pub(crate) type OwnedValue = value_bag_sval2::buffer::Value<'static>;

    pub(crate) fn buffer(v: impl value_bag_sval2::lib::Value) -> OwnedValue {
        OwnedValue::collect_owned(v).unwrap()
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    use super::*;
    use crate::test::*;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval2_capture() {
        assert_eq!(
            ValueBag::capture_sval2(&42u64).to_test_token(),
            TestToken::U64(42)
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval2_capture_cast() {
        assert_eq!(
            42u64,
            ValueBag::capture_sval2(&42u64)
                .to_u64()
                .expect("invalid value")
        );

        assert_eq!(
            "a string",
            ValueBag::capture_sval2(&"a string")
                .to_borrowed_str()
                .expect("invalid value")
        );

        #[cfg(feature = "std")]
        assert_eq!(
            "a string",
            ValueBag::capture_sval2(&"a string")
                .to_str()
                .expect("invalid value")
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval2_capture_cast_borrowed_str() {
        struct Number<'a>(&'a str);

        impl<'a> value_bag_sval2::lib::Value for Number<'a> {
            fn stream<'sval, S: value_bag_sval2::lib::Stream<'sval> + ?Sized>(
                &'sval self,
                stream: &mut S,
            ) -> value_bag_sval2::lib::Result {
                stream.tagged_begin(Some(&value_bag_sval2::lib::tags::NUMBER), None, None)?;
                stream.value(self.0)?;
                stream.tagged_end(Some(&value_bag_sval2::lib::tags::NUMBER), None, None)
            }
        }

        assert_eq!(
            "123.456e789",
            ValueBag::capture_sval2(&Number("123.456e789"))
                .to_borrowed_str()
                .expect("invalid value")
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval2_from_cast() {
        assert_eq!(
            42u64,
            ValueBag::from_sval2(&42u64)
                .to_u64()
                .expect("invalid value")
        );

        #[cfg(feature = "std")]
        assert_eq!(
            "a string",
            ValueBag::from_sval2(&"a string")
                .to_str()
                .expect("invalid value")
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval2_downcast() {
        #[derive(Debug, PartialEq, Eq)]
        struct Timestamp(usize);

        impl value_bag_sval2::lib::Value for Timestamp {
            fn stream<'sval, S: value_bag_sval2::lib::Stream<'sval> + ?Sized>(
                &'sval self,
                stream: &mut S,
            ) -> value_bag_sval2::lib::Result {
                stream.u64(self.0 as u64)
            }
        }

        let ts = Timestamp(42);

        assert_eq!(
            &ts,
            ValueBag::capture_sval2(&ts)
                .downcast_ref::<Timestamp>()
                .expect("invalid value")
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval2_structured() {
        let value = ValueBag::from(42u64);

        value_bag_sval2::test::assert_tokens(&value, &[value_bag_sval2::test::Token::U64(42)]);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval2_debug() {
        struct TestSval;

        impl value_bag_sval2::lib::Value for TestSval {
            fn stream<'sval, S: value_bag_sval2::lib::Stream<'sval> + ?Sized>(
                &'sval self,
                stream: &mut S,
            ) -> value_bag_sval2::lib::Result {
                stream.u64(42)
            }
        }

        assert_eq!(
            format!("{:04?}", 42u64),
            format!("{:04?}", ValueBag::capture_sval2(&TestSval)),
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn sval2_visit() {
        ValueBag::from_sval2(&42u64)
            .visit(TestVisit::default())
            .expect("failed to visit value");
        ValueBag::from_sval2(&-42i64)
            .visit(TestVisit::default())
            .expect("failed to visit value");
        ValueBag::from_sval2(&11f64)
            .visit(TestVisit::default())
            .expect("failed to visit value");
        ValueBag::from_sval2(&true)
            .visit(TestVisit::default())
            .expect("failed to visit value");
        ValueBag::from_sval2(&"some borrowed string")
            .visit(TestVisit::default())
            .expect("failed to visit value");
        ValueBag::from_sval2(&'n')
            .visit(TestVisit {
                str: "n",
                ..Default::default()
            })
            .expect("failed to visit value");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg(feature = "serde1")]
    fn sval2_serde1() {
        use value_bag_serde1::test::{assert_ser_tokens, Token};

        struct TestSval;

        impl value_bag_sval2::lib::Value for TestSval {
            fn stream<'sval, S: value_bag_sval2::lib::Stream<'sval> + ?Sized>(
                &'sval self,
                stream: &mut S,
            ) -> value_bag_sval2::lib::Result {
                stream.u64(42)
            }
        }

        assert_ser_tokens(&ValueBag::capture_sval2(&TestSval), &[Token::U64(42)]);
    }

    #[cfg(feature = "std")]
    mod std_support {
        use super::*;

        use crate::std::borrow::ToOwned;

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn sval2_cast() {
            assert_eq!(
                "a string",
                ValueBag::capture_sval2(&"a string".to_owned())
                    .by_ref()
                    .to_str()
                    .expect("invalid value")
            );
        }
    }
}
