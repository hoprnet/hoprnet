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

use crate::prelude::{vec, vec::Vec};

use crate::{
    build::TypeBuilder,
    form::{Form, MetaForm, PortableForm},
    IntoPortable, MetaType, Registry, TypeInfo,
};
use derive_more::From;
use scale::Encode;
#[cfg(feature = "serde")]
use serde::{de::DeserializeOwned, Deserialize, Serialize};

mod composite;
mod fields;
mod path;
mod variant;

pub use self::{composite::*, fields::*, path::*, variant::*};

/// A [`Type`] definition with optional metadata.
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
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, From, Debug, Encode)]
pub struct Type<T: Form = MetaForm> {
    /// The unique path to the type. Can be empty for built-in types
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Path::is_empty", default)
    )]
    pub path: Path<T>,
    /// The generic type parameters of the type in use. Empty for non generic types
    #[cfg_attr(
        feature = "serde",
        serde(rename = "params", skip_serializing_if = "Vec::is_empty", default)
    )]
    pub type_params: Vec<TypeParameter<T>>,
    /// The actual type definition
    #[cfg_attr(feature = "serde", serde(rename = "def"))]
    pub type_def: TypeDef<T>,
    /// Documentation
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Vec::is_empty", default)
    )]
    pub docs: Vec<T::String>,
}

impl IntoPortable for Type {
    type Output = Type<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        Type {
            path: self.path.into_portable(registry),
            type_params: registry.map_into_portable(self.type_params),
            type_def: self.type_def.into_portable(registry),
            docs: self.docs.into_iter().map(Into::into).collect(),
        }
    }
}

macro_rules! impl_from_type_def_for_type {
    ( $( $t:ty  ), + $(,)?) => { $(
        impl<F: Form> From<$t> for Type<F> {
            fn from(item: $t) -> Self {
                Self::new(Path::voldemort(), Vec::new(), item, Vec::new())
            }
        }
    )* }
}

impl_from_type_def_for_type!(
    TypeDefPrimitive,
    TypeDefArray<F>,
    TypeDefSequence<F>,
    TypeDefTuple<F>,
    TypeDefCompact<F>,
    TypeDefBitSequence<F>,
);

impl Type {
    /// Create a [`TypeBuilder`](`crate::build::TypeBuilder`) the public API for constructing a
    /// [`Type`] of [`MetaForm`].
    pub fn builder() -> TypeBuilder {
        TypeBuilder::default()
    }

    /// Create a [`TypeBuilder`](`crate::build::TypeBuilder`) the public API for constructing a
    /// [`Type`] of [`PortableForm`] for use at runtime.
    pub fn builder_portable() -> TypeBuilder<PortableForm> {
        TypeBuilder::default()
    }
}

impl<F> Type<F>
where
    F: Form,
{
    /// Create a [`Type`].
    pub fn new<I, D>(path: Path<F>, type_params: I, type_def: D, docs: Vec<F::String>) -> Type<F>
    where
        I: IntoIterator<Item = TypeParameter<F>>,
        D: Into<TypeDef<F>>,
    {
        Self {
            path,
            type_params: type_params.into_iter().collect(),
            type_def: type_def.into(),
            docs,
        }
    }
}

impl<T> Type<T>
where
    T: Form,
{
    /// Returns the path of the type
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn path(&self) -> &Path<T> {
        &self.path
    }

    /// Returns the generic type parameters of the type
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn type_params(&self) -> &[TypeParameter<T>] {
        &self.type_params
    }

    /// Returns the definition of the type
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn type_def(&self) -> &TypeDef<T> {
        &self.type_def
    }

    /// Returns the documentation of the type
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn docs(&self) -> &[T::String] {
        &self.docs
    }
}

/// A generic type parameter.
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
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, From, Debug, Encode)]
pub struct TypeParameter<T: Form = MetaForm> {
    /// The name of the generic type parameter e.g. "T".
    pub name: T::String,
    /// The concrete type for the type parameter.
    ///
    /// `None` if the type parameter is skipped.
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub ty: Option<T::Type>,
}

impl IntoPortable for TypeParameter {
    type Output = TypeParameter<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        TypeParameter {
            name: self.name.into(),
            ty: self.ty.map(|ty| registry.register_type(&ty)),
        }
    }
}

impl TypeParameter<MetaForm> {
    /// Create a new [`TypeParameter`].
    pub fn new(name: <MetaForm as Form>::String, ty: Option<<MetaForm as Form>::Type>) -> Self {
        Self { name, ty }
    }
}

impl TypeParameter<PortableForm> {
    /// Create a new [`TypeParameter`] in [`PortableForm`].
    pub fn new_portable(
        name: <PortableForm as Form>::String,
        ty: Option<<PortableForm as Form>::Type>,
    ) -> Self {
        Self { name, ty }
    }
}

impl<T> TypeParameter<T>
where
    T: Form,
{
    /// Get the type of the parameter.
    ///
    /// `None` if the parameter is skipped.
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn ty(&self) -> Option<&T::Type> {
        self.ty.as_ref()
    }

    /// Get the name of the parameter.
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn name(&self) -> &T::String {
        &self.name
    }
}

/// The possible types a SCALE encodable Rust value could have.
///
/// # Note
///
/// In order to preserve backwards compatibility, variant indices are explicitly specified instead
/// of depending on the default implicit ordering.
///
/// When adding a new variant, it must be added at the end with an incremented index.
///
/// When removing an existing variant, the rest of variant indices remain the same, and the removed
/// index should not be reused.
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
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Encode)]
pub enum TypeDef<T: Form = MetaForm> {
    /// A composite type (e.g. a struct or a tuple)
    #[codec(index = 0)]
    Composite(TypeDefComposite<T>),
    /// A variant type (e.g. an enum)
    #[codec(index = 1)]
    Variant(TypeDefVariant<T>),
    /// A sequence type with runtime known length.
    #[codec(index = 2)]
    Sequence(TypeDefSequence<T>),
    /// An array type with compile-time known length.
    #[codec(index = 3)]
    Array(TypeDefArray<T>),
    /// A tuple type.
    #[codec(index = 4)]
    Tuple(TypeDefTuple<T>),
    /// A Rust primitive type.
    #[codec(index = 5)]
    Primitive(TypeDefPrimitive),
    /// A type using the [`Compact`] encoding
    #[codec(index = 6)]
    Compact(TypeDefCompact<T>),
    /// A type representing a sequence of bits.
    #[codec(index = 7)]
    BitSequence(TypeDefBitSequence<T>),
}

macro_rules! impl_from_type_defs {
    ( $($from:ty => $variant:ident, )* ) => { $(
        impl<F: Form> From<$from> for TypeDef<F> {
            fn from(x: $from) -> Self {
                Self::$variant(x)
            }
        }
    )* }
}

impl_from_type_defs!(
    TypeDefComposite<F> => Composite,
    TypeDefVariant<F> => Variant,
    TypeDefSequence<F> => Sequence,
    TypeDefArray<F> => Array,
    TypeDefTuple<F> => Tuple,
    TypeDefPrimitive => Primitive,
    TypeDefCompact<F> => Compact,
    TypeDefBitSequence<F> => BitSequence,
);

impl IntoPortable for TypeDef {
    type Output = TypeDef<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        match self {
            TypeDef::Composite(composite) => composite.into_portable(registry).into(),
            TypeDef::Variant(variant) => variant.into_portable(registry).into(),
            TypeDef::Sequence(sequence) => sequence.into_portable(registry).into(),
            TypeDef::Array(array) => array.into_portable(registry).into(),
            TypeDef::Tuple(tuple) => tuple.into_portable(registry).into(),
            TypeDef::Primitive(primitive) => primitive.into(),
            TypeDef::Compact(compact) => compact.into_portable(registry).into(),
            TypeDef::BitSequence(bitseq) => bitseq.into_portable(registry).into(),
        }
    }
}

/// A primitive Rust type.
///
/// # Note
///
/// Explicit codec indices specified to ensure backwards compatibility. See [`TypeDef`].
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[cfg_attr(any(feature = "std", feature = "decode"), derive(scale::Decode))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Debug)]
pub enum TypeDefPrimitive {
    /// `bool` type
    #[codec(index = 0)]
    Bool,
    /// `char` type
    #[codec(index = 1)]
    Char,
    /// `str` type
    #[codec(index = 2)]
    Str,
    /// `u8`
    #[codec(index = 3)]
    U8,
    /// `u16`
    #[codec(index = 4)]
    U16,
    /// `u32`
    #[codec(index = 5)]
    U32,
    /// `u64`
    #[codec(index = 6)]
    U64,
    /// `u128`
    #[codec(index = 7)]
    U128,
    /// 256 bits unsigned int (no rust equivalent)
    #[codec(index = 8)]
    U256,
    /// `i8`
    #[codec(index = 9)]
    I8,
    /// `i16`
    #[codec(index = 10)]
    I16,
    /// `i32`
    #[codec(index = 11)]
    I32,
    /// `i64`
    #[codec(index = 12)]
    I64,
    /// `i128`
    #[codec(index = 13)]
    I128,
    /// 256 bits signed int (no rust equivalent)
    #[codec(index = 14)]
    I256,
}

/// An array type.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(any(feature = "std", feature = "decode"), derive(scale::Decode))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Debug)]
pub struct TypeDefArray<T: Form = MetaForm> {
    /// The length of the array type.
    pub len: u32,
    /// The element type of the array type.
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub type_param: T::Type,
}

impl IntoPortable for TypeDefArray {
    type Output = TypeDefArray<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        TypeDefArray {
            len: self.len,
            type_param: registry.register_type(&self.type_param),
        }
    }
}

#[allow(clippy::len_without_is_empty)]
impl<T> TypeDefArray<T>
where
    T: Form,
{
    /// Creates a new array type.
    pub fn new(len: u32, type_param: <T as Form>::Type) -> Self {
        Self { len, type_param }
    }

    /// Returns the length of the array type.
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn len(&self) -> u32 {
        self.len
    }

    /// Returns the element type of the array type.
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn type_param(&self) -> &T::Type {
        &self.type_param
    }
}

/// A type to refer to tuple types.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "T::Type: Serialize, T::String: Serialize",
        deserialize = "T::Type: DeserializeOwned, T::String: DeserializeOwned",
    ))
)]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(any(feature = "std", feature = "decode"), derive(scale::Decode))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Debug)]
pub struct TypeDefTuple<T: Form = MetaForm> {
    /// The types of the tuple fields.
    pub fields: Vec<T::Type>,
}

impl IntoPortable for TypeDefTuple {
    type Output = TypeDefTuple<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        TypeDefTuple {
            fields: registry.register_types(self.fields),
        }
    }
}

impl TypeDefTuple {
    /// Creates a new tuple type definition from the given types.
    pub fn new<T>(type_params: T) -> Self
    where
        T: IntoIterator<Item = MetaType>,
    {
        Self {
            fields: type_params
                .into_iter()
                .filter(|ty| !ty.is_phantom())
                .collect(),
        }
    }

    /// Creates a new unit tuple to represent the unit type, `()`.
    pub fn unit() -> Self {
        Self::new(vec![])
    }
}

impl TypeDefTuple<PortableForm> {
    /// Creates a new custom type definition from the given types.
    pub fn new_portable<I>(type_params: I) -> Self
    where
        I: IntoIterator<Item = <PortableForm as Form>::Type>,
    {
        Self {
            fields: type_params.into_iter().collect(),
        }
    }
}

impl<T> TypeDefTuple<T>
where
    T: Form,
{
    /// Returns the types of the tuple fields.
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn fields(&self) -> &[T::Type] {
        &self.fields
    }
}

/// A type to refer to a sequence of elements of the same type.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(any(feature = "std", feature = "decode"), derive(scale::Decode))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Debug)]
pub struct TypeDefSequence<T: Form = MetaForm> {
    /// The element type of the sequence type.
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub type_param: T::Type,
}

impl IntoPortable for TypeDefSequence {
    type Output = TypeDefSequence<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        TypeDefSequence {
            type_param: registry.register_type(&self.type_param),
        }
    }
}

impl TypeDefSequence {
    /// Creates a new sequence type.
    ///
    /// Use this constructor if you want to instantiate from a given
    /// compile-time type.
    pub fn of<T>() -> Self
    where
        T: TypeInfo + 'static,
    {
        Self::new(MetaType::new::<T>())
    }
}

impl<T> TypeDefSequence<T>
where
    T: Form,
{
    /// Creates a new sequence type.
    ///
    /// Use this constructor if you want to instantiate from a given meta type.
    pub fn new(type_param: <T as Form>::Type) -> Self {
        Self { type_param }
    }

    /// Returns the element type of the sequence type.
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn type_param(&self) -> &T::Type {
        &self.type_param
    }
}

/// A type wrapped in [`Compact`].
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(any(feature = "std", feature = "decode"), derive(scale::Decode))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Debug)]
pub struct TypeDefCompact<T: Form = MetaForm> {
    /// The type wrapped in [`Compact`], i.e. the `T` in `Compact<T>`.
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub type_param: T::Type,
}

impl IntoPortable for TypeDefCompact {
    type Output = TypeDefCompact<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        TypeDefCompact {
            type_param: registry.register_type(&self.type_param),
        }
    }
}

impl<T> TypeDefCompact<T>
where
    T: Form,
{
    /// Creates a new type wrapped in [`Compact`].
    pub fn new(type_param: <T as Form>::Type) -> Self {
        Self { type_param }
    }

    /// Returns the [`Compact`] wrapped type, i.e. the `T` in `Compact<T>`.
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn type_param(&self) -> &T::Type {
        &self.type_param
    }
}

/// Type describing a [`bitvec::vec::BitVec`].
///
/// # Note
///
/// This can only be constructed for `TypeInfo` in the `MetaForm` with the `bit-vec` feature
/// enabled, but can be decoded or deserialized into the `PortableForm` without this feature.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(any(feature = "std", feature = "decode"), derive(scale::Decode))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Debug)]
pub struct TypeDefBitSequence<T: Form = MetaForm> {
    /// The type implementing [`bitvec::store::BitStore`].
    pub bit_store_type: T::Type,
    /// The type implementing [`bitvec::order::BitOrder`].
    pub bit_order_type: T::Type,
}

impl IntoPortable for TypeDefBitSequence {
    type Output = TypeDefBitSequence<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        TypeDefBitSequence {
            bit_store_type: registry.register_type(&self.bit_store_type),
            bit_order_type: registry.register_type(&self.bit_order_type),
        }
    }
}

impl<T> TypeDefBitSequence<T>
where
    T: Form,
{
    /// Returns the type of the bit ordering of the [`::bitvec::vec::BitVec`].
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn bit_order_type(&self) -> &T::Type {
        &self.bit_order_type
    }

    /// Returns underlying type used to store the [`::bitvec::vec::BitVec`].
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn bit_store_type(&self) -> &T::Type {
        &self.bit_store_type
    }
}

impl TypeDefBitSequence {
    /// Creates a new [`TypeDefBitSequence`] for the supplied bit order and bit store types.
    ///
    /// With the `bit-vec` feature enabled, the expected usage is to provide either
    /// `bitvec::order::Lsb0` or `bitvec::order::Msb0` as the order type, and then something
    /// like u8, u8, or u32 as the store type. Without the `bit-vec` feature enabled, it's
    /// recommended that your types have identical `TypeInfo` to those.
    pub fn new<Store, Order>() -> Self
    where
        Store: TypeInfo + 'static,
        Order: TypeInfo + 'static,
    {
        Self {
            bit_store_type: MetaType::new::<Store>(),
            bit_order_type: MetaType::new::<Order>(),
        }
    }
}

impl TypeDefBitSequence<PortableForm> {
    /// Creates a new [`TypeDefBitSequence`] for the supplied bit order and bit store types.
    pub fn new_portable(
        bit_store_type: <PortableForm as Form>::Type,
        bit_order_type: <PortableForm as Form>::Type,
    ) -> Self {
        Self {
            bit_store_type,
            bit_order_type,
        }
    }
}
