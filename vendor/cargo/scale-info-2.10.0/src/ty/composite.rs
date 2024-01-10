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

/// A composite type, consisting of either named (struct) or unnamed (tuple
/// struct) fields
///
/// # Examples
///
/// ## A Rust struct with named fields.
///
/// ```
/// struct Person {
///     name: String,
///     age_in_years: u8,
///     friends: Vec<Person>,
/// }
/// ```
///
/// ## A tuple struct with unnamed fields.
///
/// ```
/// struct Color(u8, u8, u8);
/// ```
///
/// ## A so-called unit struct
///
/// ```
/// struct JustAMarker;
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
pub struct TypeDefComposite<T: Form = MetaForm> {
    /// The fields of the composite type.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Vec::is_empty", default)
    )]
    pub fields: Vec<Field<T>>,
}

impl IntoPortable for TypeDefComposite {
    type Output = TypeDefComposite<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        TypeDefComposite {
            fields: registry.map_into_portable(self.fields),
        }
    }
}

impl<T> TypeDefComposite<T>
where
    T: Form,
{
    /// Creates a new struct definition with named fields.
    pub fn new<I>(fields: I) -> Self
    where
        I: IntoIterator<Item = Field<T>>,
    {
        Self {
            fields: fields.into_iter().collect(),
        }
    }
}

impl<T> TypeDefComposite<T>
where
    T: Form,
{
    /// Returns the fields of the composite type.
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn fields(&self) -> &[Field<T>] {
        &self.fields
    }
}
