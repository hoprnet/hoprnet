// Copyright (C) 2018-2022 Stephane Raux. Distributed under the 0BSD license.

//! # Overview
//! - [📦 crates.io](https://crates.io/crates/enum-iterator)
//! - [📖 Documentation](https://docs.rs/enum-iterator)
//! - [⚖ 0BSD license](https://spdx.org/licenses/0BSD.html)
//!
//! Tools to iterate over the values of a type.
//!
//! # Examples
//! ```
//! use enum_iterator::{all, cardinality, first, last, next, previous, reverse_all, Sequence};
//!
//! #[derive(Debug, PartialEq, Sequence)]
//! enum Day { Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday }
//!
//! assert_eq!(cardinality::<Day>(), 7);
//! assert_eq!(all::<Day>().collect::<Vec<_>>(), [
//!     Day::Monday,
//!     Day::Tuesday,
//!     Day::Wednesday,
//!     Day::Thursday,
//!     Day::Friday,
//!     Day::Saturday,
//!     Day::Sunday,
//! ]);
//! assert_eq!(first::<Day>(), Some(Day::Monday));
//! assert_eq!(last::<Day>(), Some(Day::Sunday));
//! assert_eq!(next(&Day::Tuesday), Some(Day::Wednesday));
//! assert_eq!(previous(&Day::Wednesday), Some(Day::Tuesday));
//! assert_eq!(reverse_all::<Day>().collect::<Vec<_>>(), [
//!     Day::Sunday,
//!     Day::Saturday,
//!     Day::Friday,
//!     Day::Thursday,
//!     Day::Wednesday,
//!     Day::Tuesday,
//!     Day::Monday,
//! ]);
//! ```
//!
//! ```
//! use enum_iterator::{cardinality, first, last, Sequence};
//!
//! #[derive(Debug, PartialEq, Sequence)]
//! struct Foo {
//!     a: bool,
//!     b: u8,
//! }
//!
//! assert_eq!(cardinality::<Foo>(), 512);
//! assert_eq!(first::<Foo>(), Some(Foo { a: false, b: 0 }));
//! assert_eq!(last::<Foo>(), Some(Foo { a: true, b: 255 }));
//! ```
//!
//! # Rust version
//! This crate tracks stable Rust. Minor releases may require a newer Rust version. Patch releases
//! must not require a newer Rust version.
//!
//! # Contribute
//! All contributions shall be licensed under the [0BSD license](https://spdx.org/licenses/0BSD.html).

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

use core::{iter::FusedIterator, ops::ControlFlow};

pub use enum_iterator_derive::Sequence;

/// Returns the cardinality (number of values) of `T`
///
/// # Example
/// ```
/// use enum_iterator::{cardinality, Sequence};
///
/// #[derive(Debug, PartialEq, Sequence)]
/// enum Color { Red, Green, Blue }
///
/// assert_eq!(cardinality::<Color>(), 3);
/// ```
pub const fn cardinality<T: Sequence>() -> usize {
    T::CARDINALITY
}

/// Returns an iterator over all values of type `T`.
///
/// Values are yielded in the order defined by [`Sequence::next`], starting with
/// [`Sequence::first`].
///
/// # Example
/// ```
/// use enum_iterator::{all, Sequence};
///
/// #[derive(Debug, PartialEq, Sequence)]
/// enum Color { Red, Green, Blue }
///
/// assert_eq!(
///     all::<Color>().collect::<Vec<_>>(),
///     [Color::Red, Color::Green, Color::Blue],
/// );
/// ```
pub fn all<T: Sequence>() -> All<T> {
    All(T::first())
}

/// Returns an iterator over all values of type `T` in the reverse order of [`all`].
///
/// # Example
/// ```
/// use enum_iterator::{reverse_all, Sequence};
///
/// #[derive(Debug, PartialEq, Sequence)]
/// enum Color { Red, Green, Blue }
///
/// assert_eq!(
///     reverse_all::<Color>().collect::<Vec<_>>(),
///     [Color::Blue, Color::Green, Color::Red],
/// );
/// ```
pub fn reverse_all<T: Sequence>() -> ReverseAll<T> {
    ReverseAll(T::last())
}

/// Returns the next value of type `T` or `None` if this was the end.
///
/// Same as [`Sequence::next`].
///
/// # Example
/// ```
/// use enum_iterator::{next, Sequence};
///
/// #[derive(Debug, PartialEq, Sequence)]
/// enum Day { Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday }
///
/// assert_eq!(next(&Day::Friday), Some(Day::Saturday));
/// ```
pub fn next<T: Sequence>(x: &T) -> Option<T> {
    x.next()
}

/// Returns the next value of type `T` or [`first()`](first) if this was the end.
///
/// # Example
/// ```
/// use enum_iterator::{next_cycle, Sequence};
///
/// #[derive(Debug, PartialEq, Sequence)]
/// enum Day { Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday }
///
/// assert_eq!(next_cycle(&Day::Sunday), Some(Day::Monday));
/// ```
pub fn next_cycle<T: Sequence>(x: &T) -> Option<T> {
    next(x).or_else(first)
}

/// Returns the previous value of type `T` or `None` if this was the beginning.
///
/// Same as [`Sequence::previous`].
///
/// # Example
/// ```
/// use enum_iterator::{previous, Sequence};
///
/// #[derive(Debug, PartialEq, Sequence)]
/// enum Day { Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday }
///
/// assert_eq!(previous(&Day::Saturday), Some(Day::Friday));
/// ```
pub fn previous<T: Sequence>(x: &T) -> Option<T> {
    x.previous()
}

/// Returns the previous value of type `T` or [`last()`](last) if this was the beginning.
///
/// # Example
/// ```
/// use enum_iterator::{previous_cycle, Sequence};
///
/// #[derive(Debug, PartialEq, Sequence)]
/// enum Day { Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday }
///
/// assert_eq!(previous_cycle(&Day::Monday), Some(Day::Sunday));
/// ```
pub fn previous_cycle<T: Sequence>(x: &T) -> Option<T> {
    previous(x).or_else(last)
}

/// Returns the first value of type `T`.
///
/// Same as [`Sequence::first`].
///
/// # Example
/// ```
/// use enum_iterator::{first, Sequence};
///
/// #[derive(Debug, PartialEq, Sequence)]
/// enum Day { Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday }
///
/// assert_eq!(first::<Day>(), Some(Day::Monday));
/// ```
pub fn first<T: Sequence>() -> Option<T> {
    T::first()
}

/// Returns the last value of type `T`.
///
/// Same as [`Sequence::last`].
///
/// # Example
/// ```
/// use enum_iterator::{last, Sequence};
///
/// #[derive(Debug, PartialEq, Sequence)]
/// enum Day { Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday }
///
/// assert_eq!(last::<Day>(), Some(Day::Sunday));
/// ```
pub fn last<T: Sequence>() -> Option<T> {
    T::last()
}

/// Iterator over the values of type `T`.
///
/// Returned by [`all`].
#[derive(Clone, Debug)]
pub struct All<T>(Option<T>);

impl<T: Sequence> Iterator for All<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let item = self.0.take()?;
        self.0 = item.next();
        Some(item)
    }
}

impl<T: Sequence> FusedIterator for All<T> {}

/// Iterator over the values of type `T` in reverse order.
///
/// Returned by [`reverse_all`].
#[derive(Clone, Debug)]
pub struct ReverseAll<T>(Option<T>);

impl<T: Sequence> Iterator for ReverseAll<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let item = self.0.take()?;
        self.0 = item.previous();
        Some(item)
    }
}

impl<T: Sequence> FusedIterator for ReverseAll<T> {}

/// Trait to iterate over the values of a type.
///
/// # Derivation
///
/// `Sequence` can be derived for `enum` and `struct` types. Specifically, it can be derived
/// for:
/// - Enumerations whose variants meet one of the following criteria:
///   - The variant does not have fields.
///   - The variant has fields meeting all these conditions:
///     - Every field has a type that implements `Sequence`.
///     - Every field but the last one has a type that implements `Clone`.
/// - Enumerations without variants.
/// - Structures whose fields meet all these conditions:
///     - Every field has a type that implements `Sequence`.
///     - Every field but the last one has a type that implements `Clone`.
/// - Unit structures (i.e. without fields).
///
/// The cardinality (number of values) of the type must not exceed `usize::MAX`.
///
/// # Laws
///
/// `T: Sequence` implies the following assertions:
/// - `T::first().and_then(|x| x.previous()).is_none()`
/// - `T::last().and_then(|x| x.next()).is_none()`
/// - `T::first().is_none()` ⇔ `T::last().is_none()`
/// - `std::iter::successors(T::first(), T::next)` must eventually yield `T::last()`.
///
/// # Examples
/// ## C-like enumeration
///
/// ```
/// use enum_iterator::{all, cardinality, Sequence};
///
/// #[derive(Clone, Copy, Debug, PartialEq, Sequence)]
/// enum Direction { North, South, West, East }
///
/// assert_eq!(cardinality::<Direction>(), 4);
/// assert_eq!(all::<Direction>().collect::<Vec<_>>(), [
///     Direction::North,
///     Direction::South,
///     Direction::West,
///     Direction::East,
/// ]);
/// ```
///
/// ## Enumeration with data
///
/// ```
/// use enum_iterator::{all, cardinality, Sequence};
///
/// #[derive(Clone, Copy, Debug, PartialEq, Sequence)]
/// enum Direction { North, South, West, East }
///
/// #[derive(Clone, Copy, Debug, PartialEq, Sequence)]
/// enum Greeting {
///     Hi,
///     Bye,
/// }
///
/// #[derive(Clone, Copy, Debug, PartialEq, Sequence)]
/// enum Action {
///     Move(Direction),
///     Jump,
///     Talk { greeting: Greeting, loud: bool },
/// }
///
/// assert_eq!(cardinality::<Action>(), 4 + 1 + 2 * 2);
/// assert_eq!(all::<Action>().collect::<Vec<_>>(), [
///     Action::Move(Direction::North),
///     Action::Move(Direction::South),
///     Action::Move(Direction::West),
///     Action::Move(Direction::East),
///     Action::Jump,
///     Action::Talk { greeting: Greeting::Hi, loud: false },
///     Action::Talk { greeting: Greeting::Hi, loud: true },
///     Action::Talk { greeting: Greeting::Bye, loud: false },
///     Action::Talk { greeting: Greeting::Bye, loud: true },
/// ]);
/// ```
///
/// ## Structure
///
/// ```
/// use enum_iterator::{all, cardinality, Sequence};
///
/// #[derive(Clone, Copy, Debug, PartialEq, Sequence)]
/// enum Side {
///     Left,
///     Right,
/// }
///
/// #[derive(Clone, Copy, Debug, PartialEq, Sequence)]
/// enum LimbKind {
///     Arm,
///     Leg,
/// }
///
/// #[derive(Debug, PartialEq, Sequence)]
/// struct Limb {
///     kind: LimbKind,
///     side: Side,
/// }
///
/// assert_eq!(cardinality::<Limb>(), 4);
/// assert_eq!(all::<Limb>().collect::<Vec<_>>(), [
///     Limb { kind: LimbKind::Arm, side: Side::Left },
///     Limb { kind: LimbKind::Arm, side: Side::Right },
///     Limb { kind: LimbKind::Leg, side: Side::Left },
///     Limb { kind: LimbKind::Leg, side: Side::Right },
/// ]);
/// ```
pub trait Sequence: Sized {
    /// Number of values of type `Self`.
    ///
    /// # Example
    /// ```
    /// use enum_iterator::Sequence;
    ///
    /// #[derive(Sequence)]
    /// enum Day { Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday }
    ///
    /// assert_eq!(Day::CARDINALITY, 7);
    /// ```
    const CARDINALITY: usize;

    /// Returns value following `*self` or `None` if this was the end.
    ///
    /// Values are yielded in the following order. Comparisons between values are based on their
    /// relative order as yielded by `next`; an element yielded after another is considered greater.
    ///
    /// - For primitive types, in increasing order (same as `Ord`).
    /// - For arrays and tuples, in lexicographic order of the sequence of their elements.
    /// - When derived for an enumeration, in variant definition order.
    /// - When derived for a structure, in lexicographic order of the sequence of its fields taken
    ///   in definition order.
    ///
    /// The order described above is the same as `Ord` if any custom `Sequence` implementation
    /// follows `Ord` and any enumeration has its variants defined in increasing order of
    /// discriminant.
    ///
    /// # Example
    /// ```
    /// use enum_iterator::Sequence;
    ///
    /// #[derive(Debug, PartialEq, Sequence)]
    /// enum Day { Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday }
    ///
    /// assert_eq!(Day::Tuesday.next(), Some(Day::Wednesday));
    /// ```
    fn next(&self) -> Option<Self>;

    /// Returns value preceding `*self` or `None` if this was the beginning.
    ///
    /// # Example
    /// ```
    /// use enum_iterator::Sequence;
    ///
    /// #[derive(Debug, PartialEq, Sequence)]
    /// enum Day { Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday }
    ///
    /// assert_eq!(Day::Wednesday.previous(), Some(Day::Tuesday));
    /// ```
    fn previous(&self) -> Option<Self>;

    /// Returns the first value of type `Self`.
    ///
    /// # Example
    /// ```
    /// use enum_iterator::Sequence;
    ///
    /// #[derive(Debug, PartialEq, Sequence)]
    /// enum Day { Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday }
    ///
    /// assert_eq!(Day::first(), Some(Day::Monday));
    /// ```
    fn first() -> Option<Self>;

    /// Returns the last value of type `Self`.
    ///
    /// # Example
    /// ```
    /// use enum_iterator::Sequence;
    ///
    /// #[derive(Debug, PartialEq, Sequence)]
    /// enum Day { Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday }
    ///
    /// assert_eq!(Day::last(), Some(Day::Sunday));
    /// ```
    fn last() -> Option<Self>;
}

impl Sequence for bool {
    const CARDINALITY: usize = 2;

    fn next(&self) -> Option<Self> {
        (!*self).then_some(true)
    }

    fn previous(&self) -> Option<Self> {
        (*self).then_some(false)
    }

    fn first() -> Option<Self> {
        Some(false)
    }

    fn last() -> Option<Self> {
        Some(true)
    }
}

macro_rules! impl_sequence_for_int {
    ($ty:ty) => {
        impl Sequence for $ty {
            const CARDINALITY: usize = 1 << <$ty>::BITS;

            fn next(&self) -> Option<Self> {
                self.checked_add(1)
            }

            fn previous(&self) -> Option<Self> {
                self.checked_sub(1)
            }

            fn first() -> Option<Self> {
                Some(Self::MIN)
            }

            fn last() -> Option<Self> {
                Some(Self::MAX)
            }
        }
    };
}

impl_sequence_for_int!(i8);
impl_sequence_for_int!(u8);
impl_sequence_for_int!(i16);
impl_sequence_for_int!(u16);

impl Sequence for () {
    const CARDINALITY: usize = 1;

    fn next(&self) -> Option<Self> {
        None
    }

    fn previous(&self) -> Option<Self> {
        None
    }

    fn first() -> Option<Self> {
        Some(())
    }

    fn last() -> Option<Self> {
        Some(())
    }
}

impl Sequence for core::convert::Infallible {
    const CARDINALITY: usize = 0;

    fn next(&self) -> Option<Self> {
        None
    }

    fn previous(&self) -> Option<Self> {
        None
    }

    fn first() -> Option<Self> {
        None
    }

    fn last() -> Option<Self> {
        None
    }
}

impl<T: Sequence> Sequence for Option<T> {
    const CARDINALITY: usize = T::CARDINALITY + 1;

    fn next(&self) -> Option<Self> {
        match self {
            None => Some(T::first()),
            Some(x) => x.next().map(Some),
        }
    }

    fn previous(&self) -> Option<Self> {
        self.as_ref().map(T::previous)
    }

    fn first() -> Option<Self> {
        Some(None)
    }

    fn last() -> Option<Self> {
        Some(T::last())
    }
}

impl<const N: usize, T: Sequence + Clone> Sequence for [T; N] {
    const CARDINALITY: usize = {
        let tc = T::CARDINALITY;
        let mut c = 1;
        let mut i = 0;
        loop {
            if i == N {
                break c;
            }
            c *= tc;
            i += 1;
        }
    };

    fn next(&self) -> Option<Self> {
        advance_for_array(self, T::first)
    }

    fn previous(&self) -> Option<Self> {
        advance_for_array(self, T::last)
    }

    fn first() -> Option<Self> {
        if N == 0 {
            Some(core::array::from_fn(|_| unreachable!()))
        } else {
            let x = T::first()?;
            Some(core::array::from_fn(|_| x.clone()))
        }
    }

    fn last() -> Option<Self> {
        if N == 0 {
            Some(core::array::from_fn(|_| unreachable!()))
        } else {
            let x = T::last()?;
            Some(core::array::from_fn(|_| x.clone()))
        }
    }
}

fn advance_for_array<const N: usize, T, R>(a: &[T; N], reset: R) -> Option<[T; N]>
where
    T: Sequence + Clone,
    R: Fn() -> Option<T>,
{
    let mut a = a.clone();
    let keep = a.iter_mut().rev().try_fold((), |_, x| match x.next() {
        Some(new_x) => {
            *x = new_x;
            ControlFlow::Break(true)
        }
        None => match reset() {
            Some(new_x) => {
                *x = new_x;
                ControlFlow::Continue(())
            }
            None => ControlFlow::Break(false),
        },
    });
    Some(a).filter(|_| matches!(keep, ControlFlow::Break(true)))
}

macro_rules! impl_seq_advance_for_tuple {
    (
        $this:ident,
        $advance:ident,
        $reset:ident,
        $carry:ident
        @ $($values:expr,)*
        @
        @ $($placeholders:pat,)*
    ) => {
        Some(($($values,)*)).filter(|_| !$carry)
    };
    (
        $this:ident,
        $advance:ident,
        $reset:ident,
        $carry:ident
        @ $($values:expr,)*
        @ $ty:ident, $($types:ident,)*
        @ $($placeholders:pat,)*
    ) => {{
        let (.., item, $($placeholders,)*) = $this;
        let (x, new_carry) = if $carry {
            match Sequence::$advance(item) {
                Some(x) => (x, false),
                None => (Sequence::$reset()?, true),
            }
        } else {
            (item.clone(), false)
        };
        impl_seq_advance_for_tuple!(
            $this,
            $advance,
            $reset,
            new_carry
            @ x, $($values,)*
            @ $($types,)*
            @ _, $($placeholders,)*
        )
    }};
    ($this:ident, $advance:ident, $reset:ident @ $($types:ident,)*) => {{
        let (.., item) = $this;
        let (x, carry) = match Sequence::$advance(item) {
            Some(x) => (x, false),
            None => (Sequence::$reset()?, true),
        };
        impl_seq_advance_for_tuple!($this, $advance, $reset, carry @ x, @ $($types,)* @ _,)
    }};
}

macro_rules! impl_sequence_for_tuple {
    ($($types:ident,)* @ $last:ident) => {
        impl<$($types,)* $last> Sequence for ($($types,)* $last,)
        where
            $($types: Sequence + Clone,)*
            $last: Sequence,
        {
            const CARDINALITY: usize =
                $(<$types as Sequence>::CARDINALITY *)* <$last as Sequence>::CARDINALITY;

            fn next(&self) -> Option<Self> {
                impl_seq_advance_for_tuple!(self, next, first @ $($types,)*)
            }

            fn previous(&self) -> Option<Self> {
                impl_seq_advance_for_tuple!(self, previous, last @ $($types,)*)
            }

            fn first() -> Option<Self> {
                Some((
                    $(<$types as Sequence>::first()?,)*
                    <$last as Sequence>::first()?,
                ))
            }

            fn last() -> Option<Self> {
                Some((
                    $(<$types as Sequence>::last()?,)*
                    <$last as Sequence>::last()?,
                ))
            }
        }
    };
}

macro_rules! impl_sequence_for_tuples {
    ($($types:ident,)*) => {
        impl_sequence_for_tuples!(@ $($types,)*);
    };
    ($($types:ident,)* @ $head:ident, $($tail:ident,)*) => {
        impl_sequence_for_tuple!($($types,)* @ $head);
        impl_sequence_for_tuples!($($types,)* $head, @ $($tail,)*);
    };
    ($($types:ident,)* @) => {};
}

impl_sequence_for_tuples!(
    T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20,
    T21, T22, T23, T24, T25, T26, T27, T28, T29, T30, T31,
);

#[cfg(test)]
mod tests {
    use crate::{all, cardinality, reverse_all, Sequence};
    use core::convert::Infallible;

    fn cardinality_equals_item_count<T: Sequence>() {
        assert_eq!(cardinality::<T>(), all::<T>().count());
    }

    #[test]
    fn cardinality_equals_item_count_for_bool() {
        cardinality_equals_item_count::<bool>();
    }

    #[test]
    fn all_bool_values_are_yielded() {
        assert!(all::<bool>().eq([false, true]));
    }

    #[test]
    fn all_bool_values_are_yielded_in_reverse() {
        assert!(reverse_all::<bool>().eq([true, false]));
    }

    #[test]
    fn cardinality_equals_item_count_for_i8() {
        cardinality_equals_item_count::<i8>();
    }

    #[test]
    fn all_i8_values_are_yielded() {
        assert!(all::<i8>().eq(i8::MIN..=i8::MAX));
    }

    #[test]
    fn all_i8_values_are_yielded_in_reverse() {
        assert!(reverse_all::<i8>().eq((i8::MIN..=i8::MAX).rev()));
    }

    #[test]
    fn cardinality_equals_item_count_for_u8() {
        cardinality_equals_item_count::<u8>();
    }

    #[test]
    fn all_u8_values_are_yielded() {
        assert!(all::<u8>().eq(u8::MIN..=u8::MAX));
    }

    #[test]
    fn all_u8_values_are_yielded_in_reverse() {
        assert!(reverse_all::<u8>().eq((u8::MIN..=u8::MAX).rev()));
    }

    #[test]
    fn cardinality_equals_item_count_for_i16() {
        cardinality_equals_item_count::<i16>();
    }

    #[test]
    fn all_i16_values_are_yielded() {
        assert!(all::<i16>().eq(i16::MIN..=i16::MAX));
    }

    #[test]
    fn all_i16_values_are_yielded_in_reverse() {
        assert!(reverse_all::<i16>().eq((i16::MIN..=i16::MAX).rev()));
    }

    #[test]
    fn cardinality_equals_item_count_for_u16() {
        cardinality_equals_item_count::<u16>();
    }

    #[test]
    fn all_u16_values_are_yielded() {
        assert!(all::<u16>().eq(u16::MIN..=u16::MAX));
    }

    #[test]
    fn all_u16_values_are_yielded_in_reverse() {
        assert!(reverse_all::<u16>().eq((u16::MIN..=u16::MAX).rev()));
    }

    #[test]
    fn cardinality_equals_item_count_for_unit() {
        cardinality_equals_item_count::<()>();
    }

    #[test]
    fn all_unit_values_are_yielded() {
        assert!(all::<()>().eq([()]));
    }

    #[test]
    fn all_unit_values_are_yielded_in_reverse() {
        assert!(reverse_all::<()>().eq([()]));
    }

    #[test]
    fn cardinality_equals_item_count_for_infallible() {
        cardinality_equals_item_count::<Infallible>();
    }

    #[test]
    fn all_infallible_values_are_yielded() {
        assert!(all::<Infallible>().next().is_none());
    }

    #[test]
    fn all_infallible_values_are_yielded_in_reverse() {
        assert!(reverse_all::<Infallible>().next().is_none());
    }

    #[test]
    fn cardinality_equals_item_count_for_tuple_with_infallible() {
        cardinality_equals_item_count::<(bool, Infallible)>();
    }

    #[test]
    fn all_tuple_with_infallible_values_are_yielded() {
        assert!(all::<(bool, Infallible)>().next().is_none());
    }

    #[test]
    fn all_tuple_with_infallible_values_are_yielded_in_reverse() {
        assert!(reverse_all::<(bool, Infallible)>().next().is_none());
    }

    #[test]
    fn cardinality_equals_item_count_for_singleton() {
        cardinality_equals_item_count::<(u8,)>();
    }

    #[test]
    fn cardinality_equals_item_count_for_pair() {
        cardinality_equals_item_count::<(u8, bool)>();
    }

    #[test]
    fn cardinality_equals_item_count_for_triple() {
        cardinality_equals_item_count::<(bool, u8, bool)>();
    }

    #[test]
    fn cardinality_equals_item_count_for_option() {
        cardinality_equals_item_count::<Option<u8>>();
    }

    #[test]
    fn all_bool_option_items_are_yielded() {
        assert!(all::<Option<bool>>().eq([None, Some(false), Some(true)]));
    }

    #[test]
    fn tuple_fields_vary_from_right_to_left() {
        assert!(all::<(Option<bool>, bool)>().eq([
            (None, false),
            (None, true),
            (Some(false), false),
            (Some(false), true),
            (Some(true), false),
            (Some(true), true),
        ]));
    }

    #[test]
    fn cardinality_of_empty_array_is_one() {
        assert_eq!(cardinality::<[u8; 0]>(), 1);
    }

    #[test]
    fn cardinality_equals_item_count_for_empty_array() {
        cardinality_equals_item_count::<[u8; 0]>();
    }

    #[test]
    fn cardinality_equals_item_count_for_array() {
        cardinality_equals_item_count::<[u8; 3]>();
    }

    #[test]
    fn array_items_vary_from_right_to_left() {
        assert!(all::<[Option<bool>; 2]>().eq([
            [None, None],
            [None, Some(false)],
            [None, Some(true)],
            [Some(false), None],
            [Some(false), Some(false)],
            [Some(false), Some(true)],
            [Some(true), None],
            [Some(true), Some(false)],
            [Some(true), Some(true)],
        ]));
    }

    #[test]
    fn all_empty_array_items_are_yielded() {
        assert!(all::<[bool; 0]>().eq([[]]));
    }

    #[test]
    fn cardinality_of_empty_infallible_array_is_one() {
        assert_eq!(cardinality::<[Infallible; 0]>(), 1);
    }

    #[test]
    fn cardinality_of_non_empty_infallible_array_is_zero() {
        assert_eq!(cardinality::<[Infallible; 1]>(), 0);
    }

    #[test]
    fn all_empty_infallible_array_items_are_yielded() {
        assert!(all::<[Infallible; 0]>().eq([[]]));
    }

    #[test]
    fn all_non_empty_infallible_array_items_are_yielded() {
        assert!(all::<[Infallible; 1]>().next().is_none());
    }
}
