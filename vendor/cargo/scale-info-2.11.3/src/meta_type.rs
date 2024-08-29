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
    any::TypeId,
    cmp::Ordering,
    fmt::{Debug, Error as FmtError, Formatter},
    hash::{Hash, Hasher},
};

use crate::{Type, TypeInfo};

/// A metatype abstraction.
///
/// Allows to store compile-time type information at runtime.
/// This again allows to derive type ID and type definition from it.
///
/// This needs a conversion to another representation of types
/// in order to be serializable.
#[derive(Clone, Copy)]
pub struct MetaType {
    /// Function pointer to get type information.
    fn_type_info: fn() -> Type,
    // The standard type ID (ab)used in order to provide
    // cheap implementations of the standard traits
    // such as `PartialEq`, `PartialOrd`, `Debug` and `Hash`.
    type_id: TypeId,
}

impl PartialEq for MetaType {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}

impl Eq for MetaType {}

impl PartialOrd for MetaType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MetaType {
    fn cmp(&self, other: &Self) -> Ordering {
        self.type_id.cmp(&other.type_id)
    }
}

impl Hash for MetaType {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.type_id.hash(state)
    }
}

impl Debug for MetaType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        self.type_id.fmt(f)
    }
}

impl MetaType {
    /// Creates a new meta type from the given compile-time known type.
    pub fn new<T>() -> Self
    where
        T: TypeInfo + ?Sized + 'static,
    {
        Self {
            fn_type_info: <T as TypeInfo>::type_info,
            type_id: TypeId::of::<T::Identity>(),
        }
    }

    /// Returns the meta type information.
    pub fn type_info(&self) -> Type {
        (self.fn_type_info)()
    }

    /// Returns the type identifier provided by `core::any`.
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// Returns true if this represents a type of [`core::marker::PhantomData`].
    pub(crate) fn is_phantom(&self) -> bool {
        self == &MetaType::new::<crate::impls::PhantomIdentity>()
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for MetaType {
    fn schema_name() -> String {
        "MetaType".into()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        // since MetaType does not really get serialized, we don't care about its actual schema
        gen.subschema_for::<u64>()
    }
}
