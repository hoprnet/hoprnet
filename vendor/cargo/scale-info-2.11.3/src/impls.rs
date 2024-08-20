// Copyright 2019-2022 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::prelude::{
    borrow::{Cow, ToOwned},
    boxed::Box,
    collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque},
    fmt,
    marker::PhantomData,
    ops::{Range, RangeInclusive},
    rc::Rc,
    string::String,
    sync::Arc,
    time::Duration,
    vec::Vec,
};

use crate::{
    build::*, MetaType, Path, Type, TypeDefArray, TypeDefCompact, TypeDefPrimitive,
    TypeDefSequence, TypeDefTuple, TypeInfo,
};
use core::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
    NonZeroU32, NonZeroU64, NonZeroU8,
};

macro_rules! impl_metadata_for_primitives {
    ( $( $t:ty => $ident_kind:expr, )* ) => { $(
        impl TypeInfo for $t {
            type Identity = Self;

            fn type_info() -> Type {
                $ident_kind.into()
            }
        }
    )* }
}

impl_metadata_for_primitives!(
    bool => TypeDefPrimitive::Bool,
    char => TypeDefPrimitive::Char,
    u8 => TypeDefPrimitive::U8,
    u16 => TypeDefPrimitive::U16,
    u32 => TypeDefPrimitive::U32,
    u64 => TypeDefPrimitive::U64,
    u128 => TypeDefPrimitive::U128,
    i8 => TypeDefPrimitive::I8,
    i16 => TypeDefPrimitive::I16,
    i32 => TypeDefPrimitive::I32,
    i64 => TypeDefPrimitive::I64,
    i128 => TypeDefPrimitive::I128,
);

impl<T: TypeInfo + 'static, const N: usize> TypeInfo for [T; N] {
    type Identity = Self;

    fn type_info() -> Type {
        TypeDefArray::new(N as u32, MetaType::new::<T>()).into()
    }
}

macro_rules! impl_metadata_for_tuple {
    ( $($ty:ident),* ) => {
        impl<$($ty),*> TypeInfo for ($($ty,)*)
        where
            $(
                $ty: TypeInfo+ 'static,
            )*
        {
            type Identity = Self;

            fn type_info() -> Type {
                TypeDefTuple::new(tuple_meta_type!($($ty),*)).into()
            }
        }
    }
}

impl_metadata_for_tuple!();
impl_metadata_for_tuple!(A);
impl_metadata_for_tuple!(A, B);
impl_metadata_for_tuple!(A, B, C);
impl_metadata_for_tuple!(A, B, C, D);
impl_metadata_for_tuple!(A, B, C, D, E);
impl_metadata_for_tuple!(A, B, C, D, E, F);
impl_metadata_for_tuple!(A, B, C, D, E, F, G);
impl_metadata_for_tuple!(A, B, C, D, E, F, G, H);
impl_metadata_for_tuple!(A, B, C, D, E, F, G, H, I);
impl_metadata_for_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_metadata_for_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_metadata_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_metadata_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_metadata_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_metadata_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_metadata_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_metadata_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_metadata_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_metadata_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_metadata_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);

macro_rules! impl_for_non_zero {
    ( $( $t: ty: $inner: ty ),* $(,)? ) => {
        $(
            impl TypeInfo for $t {
                type Identity = Self;
                fn type_info() -> Type {
                    Type::builder()
                        .path(Path::prelude(stringify!($t)))
                        .composite(Fields::unnamed().field(|f| f.ty::<$inner>()))
                }
            }
        )*
    };
}

impl_for_non_zero!(
    NonZeroI8: i8,
    NonZeroI16: i16,
    NonZeroI32: i32,
    NonZeroI64: i64,
    NonZeroI128: i128,
    NonZeroU8: u8,
    NonZeroU16: u16,
    NonZeroU32: u32,
    NonZeroU64: u64,
    NonZeroU128: u128
);

impl TypeInfo for Duration {
    type Identity = Self;

    fn type_info() -> Type {
        Type::builder().path(Path::prelude("Duration")).composite(
            Fields::unnamed()
                .field(|f| {
                    // Seconds
                    f.ty::<u64>().type_name("u64")
                })
                .field(|f| {
                    // Nanoseconds
                    f.ty::<u32>().type_name("u32")
                }),
        )
    }
}

impl<T> TypeInfo for Vec<T>
where
    T: TypeInfo + 'static,
{
    type Identity = [T];

    fn type_info() -> Type {
        Self::Identity::type_info()
    }
}

impl<T> TypeInfo for VecDeque<T>
where
    T: TypeInfo + 'static,
{
    type Identity = [T];

    fn type_info() -> Type {
        Self::Identity::type_info()
    }
}

impl<T> TypeInfo for Option<T>
where
    T: TypeInfo + 'static,
{
    type Identity = Self;

    fn type_info() -> Type {
        Type::builder()
            .path(Path::prelude("Option"))
            .type_params(type_params![T])
            .variant(
                Variants::new()
                    .variant("None", |v| v.index(0))
                    .variant("Some", |v| {
                        v.index(1).fields(Fields::unnamed().field(|f| f.ty::<T>()))
                    }),
            )
    }
}

impl<T, E> TypeInfo for Result<T, E>
where
    T: TypeInfo + 'static,
    E: TypeInfo + 'static,
{
    type Identity = Self;

    fn type_info() -> Type {
        Type::builder()
            .path(Path::prelude("Result"))
            .type_params(type_params!(T, E))
            .variant(
                Variants::new()
                    .variant("Ok", |v| {
                        v.index(0).fields(Fields::unnamed().field(|f| f.ty::<T>()))
                    })
                    .variant("Err", |v| {
                        v.index(1).fields(Fields::unnamed().field(|f| f.ty::<E>()))
                    }),
            )
    }
}

impl<T> TypeInfo for Cow<'static, T>
where
    T: ToOwned + TypeInfo + ?Sized + 'static,
{
    type Identity = Self;

    fn type_info() -> Type {
        Type::builder()
            .path(Path::prelude("Cow"))
            .type_params(type_params!(T))
            .composite(Fields::unnamed().field(|f| f.ty::<T>()))
    }
}

impl<K, V> TypeInfo for BTreeMap<K, V>
where
    K: TypeInfo + 'static,
    V: TypeInfo + 'static,
{
    type Identity = Self;

    fn type_info() -> Type {
        Type::builder()
            .path(Path::prelude("BTreeMap"))
            .type_params(type_params![K, V])
            .composite(Fields::unnamed().field(|f| f.ty::<[(K, V)]>()))
    }
}

impl<T> TypeInfo for BTreeSet<T>
where
    T: TypeInfo + 'static,
{
    type Identity = Self;

    fn type_info() -> Type {
        Type::builder()
            .path(Path::prelude("BTreeSet"))
            .type_params(type_params![T])
            .composite(Fields::unnamed().field(|f| f.ty::<[T]>()))
    }
}

impl<T> TypeInfo for BinaryHeap<T>
where
    T: TypeInfo + 'static,
{
    type Identity = Self;

    fn type_info() -> Type {
        Type::builder()
            .path(Path::prelude("BinaryHeap"))
            .type_params(type_params![T])
            .composite(Fields::unnamed().field(|f| f.ty::<[T]>()))
    }
}

impl<T> TypeInfo for Box<T>
where
    T: TypeInfo + ?Sized + 'static,
{
    type Identity = T;

    fn type_info() -> Type {
        Self::Identity::type_info()
    }
}

impl<T> TypeInfo for Rc<T>
where
    T: TypeInfo + ?Sized + 'static,
{
    type Identity = T;

    fn type_info() -> Type {
        Self::Identity::type_info()
    }
}

impl<T> TypeInfo for Arc<T>
where
    T: TypeInfo + ?Sized + 'static,
{
    type Identity = T;

    fn type_info() -> Type {
        Self::Identity::type_info()
    }
}

impl<T> TypeInfo for &T
where
    T: TypeInfo + ?Sized + 'static,
{
    type Identity = T;

    fn type_info() -> Type {
        Self::Identity::type_info()
    }
}

impl<T> TypeInfo for &mut T
where
    T: TypeInfo + ?Sized + 'static,
{
    type Identity = T;

    fn type_info() -> Type {
        Self::Identity::type_info()
    }
}

impl<T> TypeInfo for [T]
where
    T: TypeInfo + 'static,
{
    type Identity = Self;

    fn type_info() -> Type {
        TypeDefSequence::of::<T>().into()
    }
}

impl TypeInfo for str {
    type Identity = Self;

    fn type_info() -> Type {
        TypeDefPrimitive::Str.into()
    }
}

impl TypeInfo for String {
    type Identity = str;

    fn type_info() -> Type {
        Self::Identity::type_info()
    }
}

pub(crate) type PhantomIdentity = PhantomData<()>;

impl<T> TypeInfo for PhantomData<T> {
    type Identity = PhantomIdentity;

    fn type_info() -> Type {
        // Fields of this type should be filtered out and never appear in the type graph.
        Type::builder()
            .path(Path::prelude("PhantomData"))
            .docs(&["PhantomData placeholder, this type should be filtered out"])
            .composite(Fields::unit())
    }
}

impl<T> TypeInfo for scale::Compact<T>
where
    T: TypeInfo + 'static,
{
    type Identity = Self;
    fn type_info() -> Type {
        TypeDefCompact::new(MetaType::new::<T>()).into()
    }
}

impl<Idx> TypeInfo for Range<Idx>
where
    Idx: TypeInfo + 'static + PartialOrd + fmt::Debug,
{
    type Identity = Self;
    fn type_info() -> Type {
        Type::builder()
            .path(Path::prelude("Range"))
            .type_params(type_params![Idx])
            .composite(
                Fields::named()
                    .field(|f| f.name("start").ty::<Idx>().type_name("Idx"))
                    .field(|f| f.name("end").ty::<Idx>().type_name("Idx")),
            )
    }
}

impl<Idx> TypeInfo for RangeInclusive<Idx>
where
    Idx: TypeInfo + 'static + PartialOrd + fmt::Debug,
{
    type Identity = Self;
    fn type_info() -> Type {
        Type::builder()
            .path(Path::prelude("RangeInclusive"))
            .type_params(type_params![Idx])
            .composite(
                Fields::named()
                    .field(|f| f.name("start").ty::<Idx>().type_name("Idx"))
                    .field(|f| f.name("end").ty::<Idx>().type_name("Idx")),
            )
    }
}

#[cfg(feature = "bit-vec")]
mod bit_vec {
    use super::*;

    impl<T, O> TypeInfo for bitvec::vec::BitVec<T, O>
    where
        T: bitvec::store::BitStore + TypeInfo + 'static,
        O: bitvec::order::BitOrder + TypeInfo + 'static,
    {
        type Identity = Self;

        fn type_info() -> Type {
            crate::TypeDefBitSequence::new::<T, O>().into()
        }
    }

    impl TypeInfo for bitvec::order::Lsb0 {
        type Identity = Self;

        fn type_info() -> Type {
            Type::builder()
                .path(Path::new("Lsb0", "bitvec::order"))
                .composite(Fields::unit())
        }
    }

    impl TypeInfo for bitvec::order::Msb0 {
        type Identity = Self;

        fn type_info() -> Type {
            Type::builder()
                .path(Path::new("Msb0", "bitvec::order"))
                .composite(Fields::unit())
        }
    }
}
