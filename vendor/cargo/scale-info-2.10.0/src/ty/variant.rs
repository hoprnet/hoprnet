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

use crate::prelude::vec::Vec;

use crate::{
    form::{Form, MetaForm, PortableForm},
    Field, IntoPortable, Registry,
};
use derive_more::From;
use scale::Encode;
#[cfg(feature = "serde")]
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// A Enum type (consisting of variants).
///
/// # Examples
///
/// ## A Rust enum, aka tagged union.
///
/// ```
/// enum MyEnum {
///     RustAllowsForClikeVariants,
///     AndAlsoForTupleStructs(i32, bool),
///     OrStructs {
///         with: i32,
///         named: bool,
///         fields: [u8; 32],
///     },
///     ItIsntPossibleToSetADiscriminantThough,
/// }
/// ```
///
/// ## A C-like enum type.
///
/// ```
/// enum Days {
///     Monday,
///     Tuesday,
///     Wednesday,
///     Thursday = 42, // Allows setting the discriminant explicitly
///     Friday,
///     Saturday,
///     Sunday,
/// }
/// ```
///
/// ## An empty enum (for marker purposes)
///
/// ```
/// enum JustAMarker {}
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "T::Type: Serialize, T::String: Serialize",
        deserialize = "T::Type: DeserializeOwned, T::String: DeserializeOwned",
    ))
)]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[cfg_attr(any(feature = "std", feature = "decode"), derive(scale::Decode))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, From, Encode)]
pub struct TypeDefVariant<T: Form = MetaForm> {
    /// The variants of a variant type
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Vec::is_empty", default)
    )]
    pub variants: Vec<Variant<T>>,
}

impl IntoPortable for TypeDefVariant {
    type Output = TypeDefVariant<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        TypeDefVariant {
            variants: registry.map_into_portable(self.variants),
        }
    }
}

impl<T> TypeDefVariant<T>
where
    T: Form,
{
    /// Create a new `TypeDefVariant` with the given variants
    pub fn new<I>(variants: I) -> Self
    where
        I: IntoIterator<Item = Variant<T>>,
    {
        Self {
            variants: variants.into_iter().collect(),
        }
    }
}

impl<T> TypeDefVariant<T>
where
    T: Form,
{
    /// Returns the variants of a variant type
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn variants(&self) -> &[Variant<T>] {
        &self.variants
    }
}

/// A struct enum variant with either named (struct) or unnamed (tuple struct)
/// fields.
///
/// # Example
///
/// ```
/// enum Operation {
///     Zero,
/// //  ^^^^ this is a unit struct enum variant
///     Add(i32, i32),
/// //  ^^^^^^^^^^^^^ this is a tuple-struct enum variant
///     Minus { source: i32 }
/// //  ^^^^^^^^^^^^^^^^^^^^^ this is a struct enum variant
/// }
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "T::Type: Serialize, T::String: Serialize",
        deserialize = "T::Type: DeserializeOwned, T::String: DeserializeOwned",
    ))
)]
#[cfg_attr(any(feature = "std", feature = "decode"), derive(scale::Decode))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Encode)]
pub struct Variant<T: Form = MetaForm> {
    /// The name of the variant.
    pub name: T::String,
    /// The fields of the variant.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Vec::is_empty", default)
    )]
    pub fields: Vec<Field<T>>,
    /// Index of the variant, used in `parity-scale-codec`.
    ///
    /// The value of this will be, in order of precedence:
    ///     1. The explicit index defined by a `#[codec(index = N)]` attribute.
    ///     2. The implicit index from the position of the variant in the `enum` definition.
    pub index: u8,
    /// Documentation
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Vec::is_empty", default)
    )]
    pub docs: Vec<T::String>,
}

impl IntoPortable for Variant {
    type Output = Variant<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        Variant {
            name: self.name.into(),
            fields: registry.map_into_portable(self.fields),
            index: self.index,
            docs: self.docs.into_iter().map(Into::into).collect(),
        }
    }
}

impl<T> Variant<T>
where
    T: Form,
{
    /// Creates a new variant.
    pub fn new(name: T::String, fields: Vec<Field<T>>, index: u8, docs: Vec<T::String>) -> Self {
        Self {
            name,
            fields,
            index,
            docs,
        }
    }
}

impl<T> Variant<T>
where
    T: Form,
{
    /// Returns the name of the variant.
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn name(&self) -> &T::String {
        &self.name
    }

    /// Returns the fields of the struct variant.
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn fields(&self) -> &[Field<T>] {
        &self.fields
    }

    /// Returns the index of the variant.
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn index(&self) -> u8 {
        self.index
    }

    /// Returns the documentation of the variant.
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn docs(&self) -> &[T::String] {
        &self.docs
    }
}
