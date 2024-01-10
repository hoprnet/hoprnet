//! Converting standard types into `ValueBag`s.

use super::{Error, ValueBag};

impl<'v> From<()> for ValueBag<'v> {
    #[inline]
    fn from(_: ()) -> Self {
        ValueBag::empty()
    }
}

impl<'v> From<u8> for ValueBag<'v> {
    #[inline]
    fn from(v: u8) -> Self {
        ValueBag::from_u8(v)
    }
}

impl<'v> TryFrom<ValueBag<'v>> for u8 {
    type Error = Error;

    #[inline]
    fn try_from(v: ValueBag<'v>) -> Result<Self, Error> {
        v.to_u64()
            .ok_or_else(|| Error::msg("conversion failed"))?
            .try_into()
            .map_err(|_| Error::msg("conversion failed"))
    }
}

impl<'v> From<u16> for ValueBag<'v> {
    #[inline]
    fn from(v: u16) -> Self {
        ValueBag::from_u16(v)
    }
}

impl<'v> TryFrom<ValueBag<'v>> for u16 {
    type Error = Error;

    #[inline]
    fn try_from(v: ValueBag<'v>) -> Result<Self, Error> {
        v.to_u64()
            .ok_or_else(|| Error::msg("conversion failed"))?
            .try_into()
            .map_err(|_| Error::msg("conversion failed"))
    }
}

impl<'v> From<u32> for ValueBag<'v> {
    #[inline]
    fn from(v: u32) -> Self {
        ValueBag::from_u32(v)
    }
}

impl<'v> TryFrom<ValueBag<'v>> for u32 {
    type Error = Error;

    #[inline]
    fn try_from(v: ValueBag<'v>) -> Result<Self, Error> {
        v.to_u64()
            .ok_or_else(|| Error::msg("conversion failed"))?
            .try_into()
            .map_err(|_| Error::msg("conversion failed"))
    }
}

impl<'v> From<u64> for ValueBag<'v> {
    #[inline]
    fn from(v: u64) -> Self {
        ValueBag::from_u64(v)
    }
}

impl<'v> TryFrom<ValueBag<'v>> for u64 {
    type Error = Error;

    #[inline]
    fn try_from(v: ValueBag<'v>) -> Result<Self, Error> {
        v.to_u64().ok_or_else(|| Error::msg("conversion failed"))
    }
}

impl<'v> From<usize> for ValueBag<'v> {
    #[inline]
    fn from(v: usize) -> Self {
        ValueBag::from_usize(v)
    }
}

impl<'v> TryFrom<ValueBag<'v>> for usize {
    type Error = Error;

    #[inline]
    fn try_from(v: ValueBag<'v>) -> Result<Self, Error> {
        v.to_u64()
            .ok_or_else(|| Error::msg("conversion failed"))?
            .try_into()
            .map_err(|_| Error::msg("conversion failed"))
    }
}

impl<'v> From<i8> for ValueBag<'v> {
    #[inline]
    fn from(v: i8) -> Self {
        ValueBag::from_i8(v)
    }
}

impl<'v> TryFrom<ValueBag<'v>> for i8 {
    type Error = Error;

    #[inline]
    fn try_from(v: ValueBag<'v>) -> Result<Self, Error> {
        v.to_i64()
            .ok_or_else(|| Error::msg("conversion failed"))?
            .try_into()
            .map_err(|_| Error::msg("conversion failed"))
    }
}

impl<'v> From<i16> for ValueBag<'v> {
    #[inline]
    fn from(v: i16) -> Self {
        ValueBag::from_i16(v)
    }
}

impl<'v> TryFrom<ValueBag<'v>> for i16 {
    type Error = Error;

    #[inline]
    fn try_from(v: ValueBag<'v>) -> Result<Self, Error> {
        v.to_i64()
            .ok_or_else(|| Error::msg("conversion failed"))?
            .try_into()
            .map_err(|_| Error::msg("conversion failed"))
    }
}

impl<'v> From<i32> for ValueBag<'v> {
    #[inline]
    fn from(v: i32) -> Self {
        ValueBag::from_i32(v)
    }
}

impl<'v> TryFrom<ValueBag<'v>> for i32 {
    type Error = Error;

    #[inline]
    fn try_from(v: ValueBag<'v>) -> Result<Self, Error> {
        v.to_i64()
            .ok_or_else(|| Error::msg("conversion failed"))?
            .try_into()
            .map_err(|_| Error::msg("conversion failed"))
    }
}

impl<'v> From<i64> for ValueBag<'v> {
    #[inline]
    fn from(v: i64) -> Self {
        ValueBag::from_i64(v)
    }
}

impl<'v> TryFrom<ValueBag<'v>> for i64 {
    type Error = Error;

    #[inline]
    fn try_from(v: ValueBag<'v>) -> Result<Self, Error> {
        v.to_i64().ok_or_else(|| Error::msg("conversion failed"))
    }
}

impl<'v> From<isize> for ValueBag<'v> {
    #[inline]
    fn from(v: isize) -> Self {
        ValueBag::from_isize(v)
    }
}

impl<'v> TryFrom<ValueBag<'v>> for isize {
    type Error = Error;

    #[inline]
    fn try_from(v: ValueBag<'v>) -> Result<Self, Error> {
        v.to_i64()
            .ok_or_else(|| Error::msg("conversion failed"))?
            .try_into()
            .map_err(|_| Error::msg("conversion failed"))
    }
}

impl<'v> From<f32> for ValueBag<'v> {
    #[inline]
    fn from(v: f32) -> Self {
        ValueBag::from_f32(v)
    }
}

impl<'v> From<f64> for ValueBag<'v> {
    #[inline]
    fn from(v: f64) -> Self {
        ValueBag::from_f64(v)
    }
}

impl<'v> TryFrom<ValueBag<'v>> for f64 {
    type Error = Error;

    #[inline]
    fn try_from(v: ValueBag<'v>) -> Result<Self, Error> {
        v.to_f64()
            .ok_or_else(|| Error::msg("conversion failed"))?
            .try_into()
            .map_err(|_| Error::msg("conversion failed"))
    }
}

impl<'v> From<bool> for ValueBag<'v> {
    #[inline]
    fn from(v: bool) -> Self {
        ValueBag::from_bool(v)
    }
}

impl<'v> TryFrom<ValueBag<'v>> for bool {
    type Error = Error;

    #[inline]
    fn try_from(v: ValueBag<'v>) -> Result<Self, Error> {
        v.to_bool().ok_or_else(|| Error::msg("conversion failed"))
    }
}

impl<'v> From<char> for ValueBag<'v> {
    #[inline]
    fn from(v: char) -> Self {
        ValueBag::from_char(v)
    }
}

impl<'v> TryFrom<ValueBag<'v>> for char {
    type Error = Error;

    #[inline]
    fn try_from(v: ValueBag<'v>) -> Result<Self, Error> {
        v.to_char().ok_or_else(|| Error::msg("conversion failed"))
    }
}

impl<'v> From<&'v str> for ValueBag<'v> {
    #[inline]
    fn from(v: &'v str) -> Self {
        ValueBag::from_str(v)
    }
}

impl<'v> TryFrom<ValueBag<'v>> for &'v str {
    type Error = Error;

    #[inline]
    fn try_from(v: ValueBag<'v>) -> Result<Self, Error> {
        v.to_borrowed_str()
            .ok_or_else(|| Error::msg("conversion failed"))
    }
}

impl<'v> From<&'v ()> for ValueBag<'v> {
    #[inline]
    fn from(_: &'v ()) -> Self {
        ValueBag::empty()
    }
}

impl<'v> From<&'v u8> for ValueBag<'v> {
    #[inline]
    fn from(v: &'v u8) -> Self {
        ValueBag::from_u8(*v)
    }
}

impl<'v> From<&'v u16> for ValueBag<'v> {
    #[inline]
    fn from(v: &'v u16) -> Self {
        ValueBag::from_u16(*v)
    }
}

impl<'v> From<&'v u32> for ValueBag<'v> {
    #[inline]
    fn from(v: &'v u32) -> Self {
        ValueBag::from_u32(*v)
    }
}

impl<'v> From<&'v u64> for ValueBag<'v> {
    #[inline]
    fn from(v: &'v u64) -> Self {
        ValueBag::from_u64(*v)
    }
}

impl<'v> From<&'v u128> for ValueBag<'v> {
    #[inline]
    fn from(v: &'v u128) -> Self {
        ValueBag::from_u128_ref(v)
    }
}

#[cfg(feature = "inline-i128")]
impl<'v> From<u128> for ValueBag<'v> {
    #[inline]
    fn from(v: u128) -> Self {
        ValueBag::from_u128(v)
    }
}

impl<'v> TryFrom<ValueBag<'v>> for u128 {
    type Error = Error;

    #[inline]
    fn try_from(v: ValueBag<'v>) -> Result<Self, Error> {
        v.to_u128().ok_or_else(|| Error::msg("conversion failed"))
    }
}

impl<'v> From<&'v usize> for ValueBag<'v> {
    #[inline]
    fn from(v: &'v usize) -> Self {
        ValueBag::from_usize(*v)
    }
}

impl<'v> From<&'v i8> for ValueBag<'v> {
    #[inline]
    fn from(v: &'v i8) -> Self {
        ValueBag::from_i8(*v)
    }
}

impl<'v> From<&'v i16> for ValueBag<'v> {
    #[inline]
    fn from(v: &'v i16) -> Self {
        ValueBag::from_i16(*v)
    }
}

impl<'v> From<&'v i32> for ValueBag<'v> {
    #[inline]
    fn from(v: &'v i32) -> Self {
        ValueBag::from_i32(*v)
    }
}

impl<'v> From<&'v i64> for ValueBag<'v> {
    #[inline]
    fn from(v: &'v i64) -> Self {
        ValueBag::from_i64(*v)
    }
}

impl<'v> From<&'v i128> for ValueBag<'v> {
    #[inline]
    fn from(v: &'v i128) -> Self {
        ValueBag::from_i128_ref(v)
    }
}

#[cfg(feature = "inline-i128")]
impl<'v> From<i128> for ValueBag<'v> {
    #[inline]
    fn from(v: i128) -> Self {
        ValueBag::from_i128(v)
    }
}

impl<'v> TryFrom<ValueBag<'v>> for i128 {
    type Error = Error;

    #[inline]
    fn try_from(v: ValueBag<'v>) -> Result<Self, Error> {
        v.to_i128().ok_or_else(|| Error::msg("conversion failed"))
    }
}

impl<'v> From<&'v isize> for ValueBag<'v> {
    #[inline]
    fn from(v: &'v isize) -> Self {
        ValueBag::from_isize(*v)
    }
}

impl<'v> From<&'v f32> for ValueBag<'v> {
    #[inline]
    fn from(v: &'v f32) -> Self {
        ValueBag::from_f32(*v)
    }
}

impl<'v> From<&'v f64> for ValueBag<'v> {
    #[inline]
    fn from(v: &'v f64) -> Self {
        ValueBag::from_f64(*v)
    }
}

impl<'v> From<&'v bool> for ValueBag<'v> {
    #[inline]
    fn from(v: &'v bool) -> Self {
        ValueBag::from_bool(*v)
    }
}

impl<'v> From<&'v char> for ValueBag<'v> {
    #[inline]
    fn from(v: &'v char) -> Self {
        ValueBag::from_char(*v)
    }
}

impl<'v, 'u> From<&'v &'u str> for ValueBag<'v>
where
    'u: 'v,
{
    #[inline]
    fn from(v: &'v &'u str) -> Self {
        ValueBag::from_str(*v)
    }
}

#[cfg(feature = "alloc")]
mod alloc_support {
    use super::*;

    use crate::std::{borrow::Cow, string::String};

    impl<'v> From<&'v String> for ValueBag<'v> {
        #[inline]
        fn from(v: &'v String) -> Self {
            ValueBag::from_str(v)
        }
    }

    impl<'v> TryFrom<ValueBag<'v>> for Cow<'v, str> {
        type Error = Error;

        #[inline]
        fn try_from(v: ValueBag<'v>) -> Result<Self, Error> {
            v.to_str().ok_or_else(|| Error::msg("conversion failed"))
        }
    }

    impl<'v> TryFrom<ValueBag<'v>> for String {
        type Error = Error;

        #[inline]
        fn try_from(v: ValueBag<'v>) -> Result<Self, Error> {
            Ok(v.to_str()
                .ok_or_else(|| Error::msg("conversion failed"))?
                .into_owned())
        }
    }
}

#[cfg(feature = "owned")]
mod owned_support {
    use super::*;

    use crate::OwnedValueBag;

    impl<'v> From<&'v OwnedValueBag> for ValueBag<'v> {
        #[inline]
        fn from(v: &'v OwnedValueBag) -> ValueBag<'v> {
            v.by_ref()
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    use crate::{
        std::{borrow::ToOwned, string::ToString},
        test::{IntoValueBag, TestToken},
    };

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_into_display() {
        assert_eq!(42u64.into_value_bag().by_ref().to_string(), "42");
        assert_eq!(42i64.into_value_bag().by_ref().to_string(), "42");
        assert_eq!(42.01f64.into_value_bag().by_ref().to_string(), "42.01");
        assert_eq!(true.into_value_bag().by_ref().to_string(), "true");
        assert_eq!('a'.into_value_bag().by_ref().to_string(), "a");
        assert_eq!(
            "a loong string".into_value_bag().by_ref().to_string(),
            "a loong string"
        );
        assert_eq!(().into_value_bag().by_ref().to_string(), "None");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_into_structured() {
        assert_eq!(
            42u64.into_value_bag().by_ref().to_test_token(),
            TestToken::U64(42)
        );
        assert_eq!(
            42i64.into_value_bag().by_ref().to_test_token(),
            TestToken::I64(42)
        );
        assert_eq!(
            42.01f64.into_value_bag().by_ref().to_test_token(),
            TestToken::F64(42.01)
        );
        assert_eq!(
            true.into_value_bag().by_ref().to_test_token(),
            TestToken::Bool(true)
        );
        assert_eq!(
            'a'.into_value_bag().by_ref().to_test_token(),
            TestToken::Char('a')
        );
        assert_eq!(
            "a loong string".into_value_bag().by_ref().to_test_token(),
            TestToken::Str("a loong string".to_owned())
        );
        assert_eq!(
            ().into_value_bag().by_ref().to_test_token(),
            TestToken::None
        );

        #[cfg(feature = "inline-i128")]
        {
            assert_eq!(
                42u128.into_value_bag().by_ref().to_test_token(),
                TestToken::U128(42)
            );
            assert_eq!(
                42i128.into_value_bag().by_ref().to_test_token(),
                TestToken::I128(42)
            );
        }
    }
}
