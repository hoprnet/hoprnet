#![deny(missing_docs)]

//! This crate augments the `frunk` crate with extra functionallity to allow transmogrification
//! between enums with similar structures.
//!
//! This crate specifically defines the data structures used in the generic representation of an
//! enum, the `frunk-enum-derive` crate adds a proc-macro to auto-generate the per-type code needed
//! to take advantage of this core function.

use frunk::labelled::Transmogrifier;
use frunk::{HCons, HNil};
use std::marker::PhantomData;

/// A building block for a generic enum.  The "additive-type" mirror for `frunk::HCons`.  This is
/// normally used as:
///
/// ```ignore
/// HEither<A,
///         HEither<B,
///                 HEither<C,
///                         Void>>>
/// ```
pub enum HEither<H, T> {
    /// The first variant.
    Head(H),
    /// The second/other variant.
    Tail(T),
}

/// A generic representation of an enum variant.  This holds the ident of the variant at both type
/// and value levels.
pub struct Variant<K, T> {
    /// A text representation of the variant ident
    pub key: &'static str,
    /// The value of the contents of the variant
    pub value: T,
    /// A type-level representation of the variant ident
    pub name_type_holder: PhantomData<K>,
}

/// A macro to ease the creation of `Variant`s.  See `frunk::field!` for usage.
#[macro_export]
macro_rules! variant {
    // No name provided and type is a tuple
    (($($repeated: ty),*), $value: expr) => {
        $crate::variant!( ($($repeated),*), $value, concat!( $(stringify!($repeated)),* ) )
    };
    // No name provided and type is a tuple, but with trailing commas
    (($($repeated: ty,)*), $value: expr) => {
        $crate::variant!( ($($repeated),*), $value )
    };
    // We are provided any type, with no stable name
    ($name_type: ty, $value: expr) => {
        $crate::variant!( $name_type, $value, stringify!($name_type) )
    };
    // We are provided any type, with a stable name
    ($name_type: ty, $value: expr, $name: expr) => {
        $crate::Variant::<$name_type,_> {
            key: $name,
            value: $value,
            name_type_holder: std::marker::PhantomData,
        }
    }
}

impl<TargetHead, TargetTail, SourceHead, SourceTail, HeadIndices, TailIndices, Key>
    Transmogrifier<HEither<Variant<Key, TargetHead>, TargetTail>, HCons<HeadIndices, TailIndices>>
    for HEither<Variant<Key, SourceHead>, SourceTail>
where
    SourceHead: Transmogrifier<TargetHead, HeadIndices>,
    SourceTail: Transmogrifier<TargetTail, TailIndices>,
{
    #[inline(always)]
    fn transmogrify(self) -> HEither<Variant<Key, TargetHead>, TargetTail> {
        match self {
            HEither::Head(Variant {
                value: h, key: k, ..
            }) => HEither::Head(variant!(Key, h.transmogrify(), k)),
            HEither::Tail(t) => HEither::Tail(t.transmogrify()),
        }
    }
}

/// An uninhabited type.
pub enum Void {}

impl Transmogrifier<Void, HNil> for Void {
    fn transmogrify(self) -> Void {
        match self {}
    }
}
