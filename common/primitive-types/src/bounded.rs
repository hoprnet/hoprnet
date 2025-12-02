use std::fmt::{Display, Formatter};

use crate::prelude::GeneralError;

/// Unsigned integer (`usize`) that has an explicit upper bound.
/// Trying to convert an integer that's above this bound will fail.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BoundedSize<const B: usize>(usize);

impl<const B: usize> BoundedSize<B> {
    /// Maximum value - evaluates to `B`.
    pub const MAX: Self = Self(B);
    /// Minimum value - evaluates to 0.
    pub const MIN: Self = Self(0);
}

impl<const B: usize> TryFrom<u8> for BoundedSize<B> {
    type Error = GeneralError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        (value as usize).try_into()
    }
}

impl<const B: usize> TryFrom<u16> for BoundedSize<B> {
    type Error = GeneralError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        (value as usize).try_into()
    }
}

impl<const B: usize> TryFrom<u32> for BoundedSize<B> {
    type Error = GeneralError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        (value as usize).try_into()
    }
}

impl<const B: usize> TryFrom<u64> for BoundedSize<B> {
    type Error = GeneralError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        (value as usize).try_into()
    }
}

impl<const B: usize> TryFrom<usize> for BoundedSize<B> {
    type Error = GeneralError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value <= B {
            Ok(Self(value))
        } else {
            Err(GeneralError::InvalidInput)
        }
    }
}

impl<const B: usize> TryFrom<i8> for BoundedSize<B> {
    type Error = GeneralError;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        Self::try_from(value as isize)
    }
}

impl<const B: usize> TryFrom<i16> for BoundedSize<B> {
    type Error = GeneralError;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Self::try_from(value as isize)
    }
}

impl<const B: usize> TryFrom<i32> for BoundedSize<B> {
    type Error = GeneralError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::try_from(value as isize)
    }
}

impl<const B: usize> TryFrom<i64> for BoundedSize<B> {
    type Error = GeneralError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::try_from(value as isize)
    }
}

impl<const B: usize> TryFrom<isize> for BoundedSize<B> {
    type Error = GeneralError;

    fn try_from(value: isize) -> Result<Self, Self::Error> {
        if value >= 0 {
            Self::try_from(value as usize)
        } else {
            Err(GeneralError::InvalidInput)
        }
    }
}

impl<const B: usize> Display for BoundedSize<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<const B: usize> From<BoundedSize<B>> for u8 {
    fn from(value: BoundedSize<B>) -> Self {
        value.0 as u8
    }
}

impl<const B: usize> From<BoundedSize<B>> for u16 {
    fn from(value: BoundedSize<B>) -> Self {
        value.0 as u16
    }
}

impl<const B: usize> From<BoundedSize<B>> for u32 {
    fn from(value: BoundedSize<B>) -> Self {
        value.0 as u32
    }
}

impl<const B: usize> From<BoundedSize<B>> for u64 {
    fn from(value: BoundedSize<B>) -> Self {
        value.0 as u64
    }
}

impl<const B: usize> From<BoundedSize<B>> for usize {
    fn from(value: BoundedSize<B>) -> Self {
        value.0
    }
}

/// Wrapper for [`Vec`] that has an explicit upper bound on the number of elements.
/// The Structure remains heap-allocated to avoid blowing up the size of types where it is used.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BoundedVec<T, const N: usize>(Vec<T>);

impl<T, const N: usize> Default for BoundedVec<T, N> {
    fn default() -> Self {
        Self(vec![])
    }
}

impl<T, const N: usize> TryFrom<Vec<T>> for BoundedVec<T, N> {
    type Error = GeneralError;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        if value.len() <= N {
            Ok(Self(value))
        } else {
            Err(GeneralError::InvalidInput)
        }
    }
}

impl<T, const N: usize> IntoIterator for BoundedVec<T, N> {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T, const N: usize> FromIterator<T> for BoundedVec<T, N> {
    fn from_iter<V: IntoIterator<Item = T>>(iter: V) -> Self {
        Self(iter.into_iter().take(N).collect())
    }
}

impl<T, const N: usize> From<[T; N]> for BoundedVec<T, N> {
    fn from(value: [T; N]) -> Self {
        Self(Vec::from(value))
    }
}

impl<T, const N: usize> From<BoundedVec<T, N>> for Vec<T> {
    fn from(value: BoundedVec<T, N>) -> Self {
        value.0
    }
}

impl<T, const N: usize> AsRef<[T]> for BoundedVec<T, N> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<T: Default + Copy, const N: usize> From<BoundedVec<T, N>> for [T; N] {
    fn from(value: BoundedVec<T, N>) -> Self {
        let mut out = [T::default(); N];
        value.0.into_iter().enumerate().for_each(|(i, e)| out[i] = e);
        out
    }
}

#[cfg(test)]
mod tests {
    use crate::bounded::{BoundedSize, BoundedVec};

    #[test]
    fn bounded_size_should_not_allow_bigger_numbers() {
        let min_bounded_size: usize = BoundedSize::<10>::MIN.into();
        assert_eq!(0usize, min_bounded_size);
        let max_bounded_size: usize = BoundedSize::<10>::MAX.into();
        assert_eq!(10usize, max_bounded_size);

        assert!(BoundedSize::<10>::try_from(5).is_ok_and(|b| u8::from(b) == 5));
        assert!(BoundedSize::<10>::try_from(11).is_err());
    }

    #[test]
    fn bounded_vec_should_not_fit_more_than_allowed() {
        assert!(BoundedVec::<i32, 3>::try_from(vec![]).is_ok_and(|b| Vec::from(b).is_empty()));
        assert!(BoundedVec::<i32, 3>::try_from(vec![1, 2]).is_ok_and(|b| Vec::from(b) == vec![1, 2]));
        assert!(BoundedVec::<i32, 3>::try_from(vec![1, 2, 3]).is_ok_and(|b| Vec::from(b) == vec![1, 2, 3]));
        assert!(BoundedVec::<i32, 3>::try_from(vec![1, 2, 3, 4]).is_err());

        assert_eq!(
            vec![1, 2, 3],
            Vec::from(BoundedVec::<i32, 3>::from_iter(vec![1, 2, 3, 4]))
        );
    }
}
