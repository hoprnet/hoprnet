use crate::std::{fmt, marker::PhantomData};

use crate::{
    internal::{Internal, InternalVisitor},
    Error, ValueBag,
};

impl<'v> ValueBag<'v> {
    /// Try get a collection `S` of `u64`s from this value.
    ///
    /// If this value is a sequence then the collection `S` will be extended
    /// with the attempted conversion of each of its elements.
    ///
    /// If this value is not a sequence then this method will return `None`.
    pub fn to_u64_seq<S: Default + Extend<Option<u64>>>(&self) -> Option<S> {
        self.inner.seq::<ExtendPrimitive<S, u64>>().map(|seq| seq.0)
    }

    /// Try get a collection `S` of `i64`s from this value.
    ///
    /// If this value is a sequence then the collection `S` will be extended
    /// with the attempted conversion of each of its elements.
    ///
    /// If this value is not a sequence then this method will return `None`.
    pub fn to_i64_seq<S: Default + Extend<Option<i64>>>(&self) -> Option<S> {
        self.inner.seq::<ExtendPrimitive<S, i64>>().map(|seq| seq.0)
    }

    /// Try get a collection `S` of `u128`s from this value.
    ///
    /// If this value is a sequence then the collection `S` will be extended
    /// with the attempted conversion of each of its elements.
    ///
    /// If this value is not a sequence then this method will return `None`.
    pub fn to_u128_seq<S: Default + Extend<Option<u128>>>(&self) -> Option<S> {
        self.inner
            .seq::<ExtendPrimitive<S, u128>>()
            .map(|seq| seq.0)
    }

    /// Try get a collection `S` of `i128`s from this value.
    ///
    /// If this value is a sequence then the collection `S` will be extended
    /// with the attempted conversion of each of its elements.
    ///
    /// If this value is not a sequence then this method will return `None`.
    pub fn to_i128_seq<S: Default + Extend<Option<i128>>>(&self) -> Option<S> {
        self.inner
            .seq::<ExtendPrimitive<S, i128>>()
            .map(|seq| seq.0)
    }

    /// Try get a collection `S` of `f64`s from this value.
    ///
    /// If this value is a sequence then the collection `S` will be extended
    /// with the attempted conversion of each of its elements.
    ///
    /// If this value is not a sequence then this method will return `None`.
    pub fn to_f64_seq<S: Default + Extend<Option<f64>>>(&self) -> Option<S> {
        self.inner.seq::<ExtendPrimitive<S, f64>>().map(|seq| seq.0)
    }

    /// Get a collection `S` of `f64`s from this value.
    /// 
    /// If this value is a sequence then the collection `S` will be extended
    /// with the conversion of each of its elements. The conversion is the
    /// same as [`ValueBag::as_f64`].
    /// 
    /// If this value is not a sequence then this method will return an
    /// empty collection.
    /// 
    /// This is similar to [`ValueBag::to_f64_seq`], but can be more
    /// convenient when there's no need to distinguish between an empty
    /// collection and a non-collection, or between `f64` and non-`f64` elements.
    pub fn as_f64_seq<S: Default + Extend<f64>>(&self) -> S {
        #[derive(Default)]
        struct ExtendF64<S>(S);

        impl<'a, S: Extend<f64>> ExtendValue<'a> for ExtendF64<S> {
            fn extend<'b>(&mut self, inner: Internal<'b>) {
                self.0.extend(Some(ValueBag { inner }.as_f64()))
            }
        }

        self.inner.seq::<ExtendF64<S>>().map(|seq| seq.0).unwrap_or_default()
    }

    /// Try get a collection `S` of `bool`s from this value.
    ///
    /// If this value is a sequence then the collection `S` will be extended
    /// with the attempted conversion of each of its elements.
    ///
    /// If this value is not a sequence then this method will return `None`.
    pub fn to_bool_seq<S: Default + Extend<Option<bool>>>(&self) -> Option<S> {
        self.inner
            .seq::<ExtendPrimitive<S, bool>>()
            .map(|seq| seq.0)
    }
}

#[cfg(feature = "alloc")]
mod alloc_support {
    use super::*;

    use crate::std::borrow::Cow;

    impl<'v> ValueBag<'v> {
        /// Try get a collection `S` of strings from this value.
        ///
        /// If this value is a sequence then the collection `S` will be extended
        /// with the attempted conversion of each of its elements.
        ///
        /// If this value is not a sequence then this method will return `None`.
        #[inline]
        pub fn to_str_seq<S: Default + Extend<Option<Cow<'v, str>>>>(&self) -> Option<S> {
            #[derive(Default)]
            struct ExtendStr<'a, S>(S, PhantomData<Cow<'a, str>>);

            impl<'a, S: Extend<Option<Cow<'a, str>>>> ExtendValue<'a> for ExtendStr<'a, S> {
                fn extend<'b>(&mut self, inner: Internal<'b>) {
                    self.0.extend(Some(
                        ValueBag { inner }
                            .to_str()
                            .map(|s| Cow::Owned(s.into_owned())),
                    ))
                }

                fn extend_borrowed(&mut self, inner: Internal<'a>) {
                    self.0.extend(Some(ValueBag { inner }.to_str()))
                }
            }

            self.inner.seq::<ExtendStr<'v, S>>().map(|seq| seq.0)
        }
    }
}

#[derive(Default)]
struct ExtendPrimitive<S, T>(S, PhantomData<T>);

impl<'a, S: Extend<Option<T>>, T: for<'b> TryFrom<ValueBag<'b>>> ExtendValue<'a>
    for ExtendPrimitive<S, T>
{
    fn extend<'b>(&mut self, inner: Internal<'b>) {
        self.0.extend(Some(ValueBag { inner }.try_into().ok()))
    }
}

pub(crate) trait ExtendValue<'v> {
    fn extend<'a>(&mut self, v: Internal<'a>);

    fn extend_borrowed(&mut self, v: Internal<'v>) {
        self.extend(v);
    }
}

impl<'v> Internal<'v> {
    #[inline]
    fn seq<S: Default + ExtendValue<'v>>(&self) -> Option<S> {
        struct SeqVisitor<S>(Option<S>);

        impl<'v, S: Default + ExtendValue<'v>> InternalVisitor<'v> for SeqVisitor<S> {
            #[inline]
            fn debug(&mut self, _: &dyn fmt::Debug) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn display(&mut self, _: &dyn fmt::Display) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn u64(&mut self, _: u64) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn i64(&mut self, _: i64) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn u128(&mut self, _: &u128) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn i128(&mut self, _: &i128) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn f64(&mut self, _: f64) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn bool(&mut self, _: bool) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn char(&mut self, _: char) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn str(&mut self, _: &str) -> Result<(), Error> {
                Ok(())
            }

            #[inline]
            fn none(&mut self) -> Result<(), Error> {
                Ok(())
            }

            #[cfg(feature = "error")]
            #[inline]
            fn error(&mut self, _: &dyn crate::internal::error::Error) -> Result<(), Error> {
                Ok(())
            }

            #[cfg(feature = "sval2")]
            #[inline]
            fn sval2(&mut self, v: &dyn crate::internal::sval::v2::Value) -> Result<(), Error> {
                self.0 = crate::internal::sval::v2::seq::extend(v);

                Ok(())
            }

            #[cfg(feature = "sval2")]
            #[inline]
            fn borrowed_sval2(
                &mut self,
                v: &'v dyn crate::internal::sval::v2::Value,
            ) -> Result<(), Error> {
                self.0 = crate::internal::sval::v2::seq::extend_borrowed(v);

                Ok(())
            }

            #[cfg(feature = "serde1")]
            #[inline]
            fn serde1(
                &mut self,
                v: &dyn crate::internal::serde::v1::Serialize,
            ) -> Result<(), Error> {
                self.0 = crate::internal::serde::v1::seq::extend(v);

                Ok(())
            }

            fn poisoned(&mut self, _: &'static str) -> Result<(), Error> {
                Ok(())
            }
        }

        let mut visitor = SeqVisitor(None);
        let _ = self.internal_visit(&mut visitor);

        visitor.0
    }
}
