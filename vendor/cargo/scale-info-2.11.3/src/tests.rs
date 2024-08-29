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

use crate::{
    build::*,
    prelude::{
        borrow::Cow,
        boxed::Box,
        collections::{BTreeMap, BTreeSet, BinaryHeap},
        string::String,
        vec,
    },
    *,
};
use core::marker::PhantomData;
use scale::Compact;
use std::num::NonZeroU32;

fn assert_type<T, E>(expected: E)
where
    T: TypeInfo + ?Sized,
    E: Into<Type>,
{
    assert_eq!(T::type_info(), expected.into());
}

macro_rules! assert_type {
    ( $ty:ty, $expected:expr ) => {{
        assert_type::<$ty, _>($expected)
    }};
}

#[test]
fn primitives() {
    assert_type!(bool, TypeDefPrimitive::Bool);
    assert_type!(&str, TypeDefPrimitive::Str);
    assert_type!(i8, TypeDefPrimitive::I8);

    assert_type!([bool], TypeDefSequence::new(meta_type::<bool>()));
}

#[test]
fn prelude_items() {
    assert_type!(String, TypeDefPrimitive::Str);

    assert_type!(
        Option<u128>,
        Type::builder()
            .path(Path::prelude("Option"))
            .type_params(named_type_params![(T, u128)])
            .variant(
                Variants::new()
                    .variant("None", |v| v.index(0))
                    .variant("Some", |v| {
                        v.index(1)
                            .fields(Fields::unnamed().field(|f| f.ty::<u128>()))
                    })
            )
    );
    assert_type!(
        Result<bool, String>,
        Type::builder()
            .path(Path::prelude("Result"))
            .type_params(named_type_params![(T, bool), (E, String)])
            .variant(
                Variants::new()
                    .variant(
                        "Ok", |v| v
                            .index(0)
                            .fields(Fields::unnamed().field(|f| f.ty::<bool>()))
                    )
                    .variant(
                        "Err", |v| v
                            .index(1)
                            .fields(Fields::unnamed().field(|f| f.ty::<String>()))
                    )
            )
    );
    assert_type!(
        Cow<u128>,
        Type::builder()
            .path(Path::prelude("Cow"))
            .type_params(named_type_params![(T, u128)])
            .composite(Fields::unnamed().field(|f| f.ty::<u128>()))
    );

    assert_type!(
        NonZeroU32,
        Type::builder()
            .path(Path::prelude("NonZeroU32"))
            .composite(Fields::unnamed().field(|f| f.ty::<u32>()))
    )
}

#[test]
fn phantom_data() {
    assert_type!(
        PhantomData<i32>,
        Type::builder()
            .path(Path::prelude("PhantomData"))
            .docs(&["PhantomData placeholder, this type should be filtered out"])
            .composite(Fields::unit())
    )
}

#[test]
fn collections() {
    assert_type!(
        BTreeMap<String, u32>,
        Type::builder()
            .path(Path::prelude("BTreeMap"))
            .type_params(named_type_params![(K, String), (V, u32)])
            .composite(Fields::unnamed().field(|f| f.ty::<[(String, u32)]>()))
    );

    assert_type!(
        BTreeSet<String>,
        Type::builder()
            .path(Path::prelude("BTreeSet"))
            .type_params(named_type_params![(T, String)])
            .composite(Fields::unnamed().field(|f| f.ty::<[String]>()))
    );

    assert_type!(
        BinaryHeap<String>,
        Type::builder()
            .path(Path::prelude("BinaryHeap"))
            .type_params(named_type_params![(T, String)])
            .composite(Fields::unnamed().field(|f| f.ty::<[String]>()))
    );

    assert_type!(
        std::collections::VecDeque<String>,
        TypeDefSequence::new(meta_type::<String>())
    );
}

#[cfg(feature = "bit-vec")]
#[test]
fn bitvec() {
    use bitvec::{
        order::{Lsb0, Msb0},
        vec::BitVec,
    };

    assert_type!(
        BitVec<u8,Lsb0>,
        TypeDefBitSequence::new::<u8,Lsb0>()
    );

    assert_type!(
        BitVec<u16,Msb0>,
        TypeDefBitSequence::new::<u16,Msb0>()
    );

    assert_type!(
        BitVec<u32,Msb0>,
        TypeDefBitSequence::new::<u32,Msb0>()
    );
}

#[test]
fn scale_compact_types() {
    assert_type!(Compact<i32>, TypeDefCompact::new(meta_type::<i32>()))
}

#[test]
fn tuple_primitives() {
    // unit
    assert_type!((), TypeDefTuple::new(tuple_meta_type!()));

    // tuple with one element
    assert_type!((bool,), TypeDefTuple::new(tuple_meta_type!(bool)));

    // tuple with multiple elements
    assert_type!(
        (bool, String),
        TypeDefTuple::new(tuple_meta_type!(bool, String))
    );

    // nested tuple
    assert_type!(
        ((i8, i16), (u32, u64)),
        TypeDefTuple::new(vec![meta_type::<(i8, i16)>(), meta_type::<(u32, u64)>(),])
    );
}

#[test]
fn tuple_phantom_data_erased() {
    // nested tuple
    assert_type!(
        (u64, PhantomData<u8>),
        TypeDefTuple::new(vec![meta_type::<u64>(),])
    );
}

#[test]
fn array_primitives() {
    // array
    assert_type!([bool; 3], TypeDefArray::new(3, meta_type::<bool>()));
    // nested
    assert_type!([[i32; 5]; 5], TypeDefArray::new(5, meta_type::<[i32; 5]>()));
    // sequence
    assert_type!([bool], TypeDefSequence::new(meta_type::<bool>()));
    // vec
    assert_type!(Vec<bool>, TypeDefSequence::new(meta_type::<bool>()));
}

#[test]
fn struct_with_generics() {
    #[allow(unused)]
    struct MyStruct<T> {
        data: T,
    }

    impl<T> TypeInfo for MyStruct<T>
    where
        T: TypeInfo + 'static,
    {
        type Identity = Self;

        fn type_info() -> Type {
            Type::builder()
                .path(Path::new("MyStruct", module_path!()))
                .type_params(type_params!(T))
                .composite(Fields::named().field(|f| f.ty::<T>().name("data").type_name("T")))
        }
    }

    // Normal struct
    let struct_bool_type_info = Type::builder()
        .path(Path::from_segments(vec!["scale_info", "tests", "MyStruct"]).unwrap())
        .type_params(named_type_params![(T, bool)])
        .composite(Fields::named().field(|f| f.ty::<bool>().name("data").type_name("T")));

    assert_type!(MyStruct<bool>, struct_bool_type_info);

    // With "`Self` typed" fields
    type SelfTyped = MyStruct<Box<MyStruct<bool>>>;
    let expected_type = Type::builder()
        .path(Path::new("MyStruct", "scale_info::tests"))
        .type_params(named_type_params![(T, Box<MyStruct<bool>>)])
        .composite(
            Fields::named().field(|f| f.ty::<Box<MyStruct<bool>>>().name("data").type_name("T")),
        );
    assert_type!(SelfTyped, expected_type);
}

#[test]
fn basic_struct_with_phantoms() {
    #[allow(unused)]
    struct SomeStruct<T> {
        a: u8,
        marker: PhantomData<T>,
    }

    impl<T> TypeInfo for SomeStruct<T>
    where
        T: TypeInfo + 'static,
    {
        type Identity = Self;

        fn type_info() -> Type {
            Type::builder()
                .path(Path::new("SomeStruct", module_path!()))
                .type_params(type_params!(T))
                .composite(Fields::named().field(|f| f.ty::<u8>().name("a").type_name("u8")))
        }
    }

    let struct_bool_type_info = Type::builder()
        .path(Path::from_segments(vec!["scale_info", "tests", "SomeStruct"]).unwrap())
        .type_params(named_type_params![(T, bool)])
        .composite(Fields::named().field(|f| f.ty::<u8>().name("a").type_name("u8")));

    assert_type!(SomeStruct<bool>, struct_bool_type_info);
}

#[test]
fn basic_enum_with_index() {
    use scale::Encode;

    #[allow(unused)]
    #[derive(Encode)]
    enum IndexedRustEnum {
        #[codec(index = 3)]
        A(bool),
        #[codec(index = 0)]
        B {
            b: u8,
        },
        C(u16, u32),
        D,
    }
    impl TypeInfo for IndexedRustEnum {
        type Identity = Self;

        fn type_info() -> Type {
            Type::builder()
                .path(Path::new("IndexedRustEnum", module_path!()))
                .variant(
                    Variants::new()
                        .variant("A", |v| {
                            v.index(3).fields(
                                Fields::unnamed().field(|f| f.ty::<bool>().type_name("bool")),
                            )
                        })
                        .variant("B", |v| {
                            v.index(0).fields(
                                Fields::named().field(|f| f.ty::<u8>().name("b").type_name("u8")),
                            )
                        })
                        .variant("C", |v| {
                            v.index(2).fields(
                                Fields::unnamed()
                                    .field(|f| f.ty::<u16>().type_name("u16"))
                                    .field(|f| f.ty::<u32>().type_name("u32")),
                            )
                        })
                        .variant_unit("D", 3),
                )
        }
    }

    let ty = Type::builder()
        .path(Path::new("IndexedRustEnum", module_path!()))
        .variant(
            Variants::new()
                .variant("A", |v| {
                    v.index(3)
                        .fields(Fields::unnamed().field(|f| f.ty::<bool>().type_name("bool")))
                })
                .variant("B", |v| {
                    v.index(0)
                        .fields(Fields::named().field(|f| f.ty::<u8>().name("b").type_name("u8")))
                })
                .variant("C", |v| {
                    v.index(2).fields(
                        Fields::unnamed()
                            .field(|f| f.ty::<u16>().type_name("u16"))
                            .field(|f| f.ty::<u32>().type_name("u32")),
                    )
                })
                .variant_unit("D", 3),
        );

    assert_type!(IndexedRustEnum, ty);
}
