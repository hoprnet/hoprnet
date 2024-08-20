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
    build::FieldBuilder,
    form::{Form, MetaForm, PortableForm},
    prelude::vec::Vec,
    IntoPortable, Registry,
};
use scale::Encode;
#[cfg(feature = "serde")]
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// A field of a struct-like data type.
///
/// Name is optional so it can represent both named and unnamed fields.
///
/// This can be a named field of a struct type or an enum struct variant, or an
/// unnamed field of a tuple struct.
///
/// # Type name
///
/// The `type_name` field contains a string which is the name of the type of the
/// field as it appears in the source code. The exact contents and format of the
/// type name are not specified, but in practice will be the name of any valid
/// type for a field e.g.
///
///   - Concrete types e.g `"u32"`, `"bool"`, `"Foo"` etc.
///   - Type parameters e.g `"T"`, `"U"`
///   - Generic types e.g `"Vec<u32>"`, `"Vec<T>"`
///   - Associated types e.g. `"T::MyType"`, `"<T as MyTrait>::MyType"`
///   - Type aliases e.g. `"MyTypeAlias"`, `"MyTypeAlias<T>"`
///   - Other built in Rust types e.g. arrays, references etc.
///
/// Note that the type name doesn't correspond to the underlying type of the
/// field, unless using a concrete type directly. Any given type may be referred
/// to by multiple field type names, when using generic type parameters and type
/// aliases.
///
/// This is intended for informational and diagnostic purposes only. Although it
/// is possible to infer certain properties e.g. whether a type name is a type
/// alias, there are no guarantees provided, and the type name representation
/// may change.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "T::Type: Serialize, T::String: Serialize",
        deserialize = "T::Type: DeserializeOwned, T::String: DeserializeOwned",
    ))
)]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(any(feature = "std", feature = "decode"), derive(scale::Decode))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Encode)]
pub struct Field<T: Form = MetaForm> {
    /// The name of the field. None for unnamed fields.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub name: Option<T::String>,
    /// The type of the field.
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub ty: T::Type,
    /// The name of the type of the field as it appears in the source code.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub type_name: Option<T::String>,
    /// Documentation
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Vec::is_empty", default)
    )]
    pub docs: Vec<T::String>,
}

impl IntoPortable for Field {
    type Output = Field<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        Field {
            name: self.name.map(Into::into),
            ty: registry.register_type(&self.ty),
            type_name: self.type_name.map(Into::into),
            docs: self.docs.into_iter().map(Into::into).collect(),
        }
    }
}

impl<T> Field<T>
where
    T: Form,
{
    /// Returns a new [`FieldBuilder`] for constructing a field.
    pub fn builder() -> FieldBuilder<T> {
        FieldBuilder::new()
    }

    /// Creates a new field.
    ///
    /// Use this constructor if you want to instantiate from a given meta type.
    pub fn new(
        name: Option<T::String>,
        ty: T::Type,
        type_name: Option<T::String>,
        docs: Vec<T::String>,
    ) -> Self {
        Self {
            name,
            ty,
            type_name,
            docs,
        }
    }
}

impl<T> Field<T>
where
    T: Form,
{
    /// Returns the name of the field. None for unnamed fields.
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn name(&self) -> Option<&T::String> {
        self.name.as_ref()
    }

    /// Returns the type of the field.
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn ty(&self) -> &T::Type {
        &self.ty
    }

    /// Returns a string which is the name of the type of the field as it
    /// appears in the source code. The exact contents and format of the type
    /// name are not specified, but in practice will be the name of any valid
    /// type for a field. This is intended for informational and diagnostic
    /// purposes only.
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn type_name(&self) -> Option<&T::String> {
        self.type_name.as_ref()
    }

    /// Returns the documentation of the field.
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn docs(&self) -> &[T::String] {
        &self.docs
    }
}
