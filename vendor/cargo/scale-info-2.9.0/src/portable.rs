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

//! The registry stores type definitions in a space-efficient manner.
//!
//! This is done by deduplicating common types in order to reuse their
//! definitions which otherwise can grow arbitrarily large. A type is uniquely
//! identified by its type identifier that is therefore used to refer to types
//! and their definitions.
//!
//! Types with the same name are uniquely identifiable by introducing
//! namespaces. The normal Rust namespace of a type is used, except for the Rust
//! prelude types that live in the so-called root namespace which is empty.

use crate::{
    form::PortableForm,
    interner::Interner,
    prelude::{
        collections::BTreeMap,
        fmt::Debug,
        mem,
        vec::Vec,
    },
    Path,
    Registry,
    Type,
    TypeDef,
    TypeDefPrimitive,
};
use scale::Encode;

/// A read-only registry containing types in their portable form for serialization.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(all(feature = "serde", feature = "decode"), derive(serde::Deserialize))]
#[cfg_attr(any(feature = "std", feature = "decode"), derive(scale::Decode))]
#[derive(Clone, Debug, PartialEq, Eq, Encode)]
pub struct PortableRegistry {
    /// The types contained by the [`PortableRegistry`].
    pub types: Vec<PortableType>,
}

impl From<Registry> for PortableRegistry {
    fn from(registry: Registry) -> Self {
        PortableRegistry {
            types: registry
                .types()
                .map(|(k, v)| {
                    PortableType {
                        id: k.id,
                        ty: v.clone(),
                    }
                })
                .collect::<Vec<_>>(),
        }
    }
}

impl PortableRegistry {
    /// Returns the type definition for the given identifier, `None` if no type found for that ID.
    pub fn resolve(&self, id: u32) -> Option<&Type<PortableForm>> {
        self.types.get(id as usize).map(|ty| &ty.ty)
    }

    /// Returns all types with their associated identifiers.
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn types(&self) -> &[PortableType] {
        &self.types
    }

    /// Retains only the portable types needed to express the provided ids.
    ///
    /// The type IDs retained are returned as key to the [`BTreeMap`].
    /// The value of the map represents the new ID of that type.
    ///
    /// # Note
    ///
    /// A given type ID can be defined by nesting type IDs, such as the case
    /// of a [`TypeDef::Composite`] and others. To retain a valid [`PortableRegistry`]
    /// all the types needed to express an ID are included. Therefore, the number of
    /// elements defined by the result equals or exceeds the number of provided IDs.
    pub fn retain<F>(&mut self, mut filter: F) -> BTreeMap<u32, u32>
    where
        F: FnMut(u32) -> bool,
    {
        let mut retained_mappings = BTreeMap::new();
        let mut new_types = crate::prelude::vec![];

        fn retain_type(
            id: u32,
            types: &mut [PortableType],
            new_types: &mut Vec<PortableType>,
            retained_mappings: &mut BTreeMap<u32, u32>,
        ) -> u32 {
            // Type already retained; just return the new ID for it:
            if let Some(id) = retained_mappings.get(&id) {
                return *id
            }

            // Zero-allocation default implementation that is used as
            // a placeholder and never accessed.
            let placeholder = PortableType {
                id: 0,
                ty: Type {
                    type_def: TypeDef::Primitive(TypeDefPrimitive::Bool),
                    path: Path::default(),
                    type_params: crate::prelude::vec![],
                    docs: crate::prelude::vec![],
                },
            };
            // Take the type out of the registry that we'll be retaining:
            let mut ty = mem::replace(&mut types[id as usize], placeholder);

            // Make sure any type params are also retained:
            for param in ty.ty.type_params.iter_mut() {
                let Some(ty) = &param.ty else { continue };
                let new_id = retain_type(ty.id, types, new_types, retained_mappings);
                param.ty = Some(new_id).map(Into::into);
            }

            // make sure any types inside this type are also retained and update the IDs:
            match &mut ty.ty.type_def {
                TypeDef::Composite(composite) => {
                    for field in composite.fields.iter_mut() {
                        let new_id =
                            retain_type(field.ty.id, types, new_types, retained_mappings);
                        field.ty = new_id.into();
                    }
                }
                TypeDef::Variant(variant) => {
                    for var in variant.variants.iter_mut() {
                        for field in var.fields.iter_mut() {
                            let new_id = retain_type(
                                field.ty.id,
                                types,
                                new_types,
                                retained_mappings,
                            );
                            field.ty = new_id.into();
                        }
                    }
                }
                TypeDef::Sequence(sequence) => {
                    let new_id = retain_type(
                        sequence.type_param.id,
                        types,
                        new_types,
                        retained_mappings,
                    );
                    sequence.type_param = new_id.into();
                }
                TypeDef::Array(array) => {
                    let new_id = retain_type(
                        array.type_param.id,
                        types,
                        new_types,
                        retained_mappings,
                    );
                    array.type_param = new_id.into();
                }
                TypeDef::Tuple(tuple) => {
                    for ty in tuple.fields.iter_mut() {
                        let new_id =
                            retain_type(ty.id, types, new_types, retained_mappings);
                        *ty = new_id.into();
                    }
                }
                TypeDef::Primitive(_) => (),
                TypeDef::Compact(compact) => {
                    let new_id = retain_type(
                        compact.type_param.id,
                        types,
                        new_types,
                        retained_mappings,
                    );
                    compact.type_param = new_id.into();
                }
                TypeDef::BitSequence(bit_seq) => {
                    let bit_store_id = retain_type(
                        bit_seq.bit_store_type.id,
                        types,
                        new_types,
                        retained_mappings,
                    );
                    let bit_order_id = retain_type(
                        bit_seq.bit_order_type.id,
                        types,
                        new_types,
                        retained_mappings,
                    );

                    bit_seq.bit_store_type = bit_store_id.into();
                    bit_seq.bit_order_type = bit_order_id.into();
                }
            }

            // Retain this type, having updated any inner IDs:
            let new_id = new_types.len() as u32;
            new_types.push(ty);
            retained_mappings.insert(id, new_id);
            new_id
        }

        for id in 0..self.types.len() as u32 {
            // We don't care about the type; move on:
            if !filter(id) {
                continue
            }

            retain_type(id, &mut self.types, &mut new_types, &mut retained_mappings);
        }

        self.types = new_types;
        retained_mappings
    }
}

/// Represent a type in it's portable form.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(all(feature = "serde", feature = "decode"), derive(serde::Deserialize))]
#[cfg_attr(any(feature = "std", feature = "decode"), derive(scale::Decode))]
#[derive(Clone, Debug, PartialEq, Eq, Encode)]
pub struct PortableType {
    /// The ID of the portable type.
    #[codec(compact)]
    pub id: u32,
    /// The portable form of the type.
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub ty: Type<PortableForm>,
}

impl PortableType {
    /// Construct a custom `PortableType`.
    pub fn new(id: u32, ty: Type<PortableForm>) -> Self {
        Self { id, ty }
    }

    /// Returns the index of the [`PortableType`].
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the type of the [`PortableType`].
    #[deprecated(
        since = "2.5.0",
        note = "Prefer to access the fields directly; this getter will be removed in the next major version"
    )]
    pub fn ty(&self) -> &Type<PortableForm> {
        &self.ty
    }
}

/// Construct a [`PortableRegistry`].
///
/// Guarantees that the resulting [`PortableRegistry`] has the list of types in the correct order,
/// since downstream libs assume that a `u32` type id corresponds to the index of the type
/// definition type table.
#[derive(Debug, Default)]
pub struct PortableRegistryBuilder {
    types: Interner<Type<PortableForm>>,
}

impl PortableRegistryBuilder {
    /// Create a new [`PortableRegistryBuilder`].
    pub fn new() -> Self {
        Default::default()
    }

    /// Register a type, returning the assigned ID.
    ///
    /// If the type is already registered it will return the existing ID.
    pub fn register_type(&mut self, ty: Type<PortableForm>) -> u32 {
        self.types.intern_or_get(ty).1.into_untracked().id
    }

    /// Returns the type id that would be assigned to a newly registered type.
    pub fn next_type_id(&self) -> u32 {
        self.types.elements().len() as u32
    }

    /// Returns a reference to the type registered at the given ID (if any).
    pub fn get(&self, id: u32) -> Option<&Type<PortableForm>> {
        self.types.elements().get(id as usize)
    }

    /// Finalize and return a valid [`PortableRegistry`] instance.
    pub fn finish(&self) -> PortableRegistry {
        let types = self
            .types
            .elements()
            .iter()
            .enumerate()
            .map(|(i, ty)| {
                PortableType {
                    id: i as u32,
                    ty: ty.clone(),
                }
            })
            .collect();
        PortableRegistry { types }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build::*,
        prelude::vec,
        *,
    };

    // Type IDs generated by `build_registry`.
    const U32_TY_ID: u32 = 0;
    const U64_TY_ID: u32 = 1;
    const VEC_U32_TY_ID: u32 = 2;
    const ARRAY_U32_TY_ID: u32 = 3;
    const TUPLE_TY_ID: u32 = 4;
    const COMPACT_TY_ID: u32 = 5;
    const BIT_SEQ_TY_ID: u32 = 6;
    const COMPOSITE_TY_ID: u32 = 7;
    const VARIANT_TY_ID: u32 = 8;

    fn build_registry() -> PortableRegistry {
        let mut builder = PortableRegistryBuilder::new();
        // Primitives
        let u32_type = Type::new(Path::default(), vec![], TypeDefPrimitive::U32, vec![]);
        let u32_type_id = builder.register_type(u32_type);
        assert_eq!(U32_TY_ID, u32_type_id);

        let u64_type = Type::new(Path::default(), vec![], TypeDefPrimitive::U64, vec![]);
        let u64_type_id = builder.register_type(u64_type);
        assert_eq!(U64_TY_ID, u64_type_id);

        // Sequence
        let vec_u32_type = Type::new(
            Path::default(),
            vec![],
            TypeDefSequence::new(u32_type_id.into()),
            vec![],
        );
        let vec_u32_type_id = builder.register_type(vec_u32_type);
        assert_eq!(VEC_U32_TY_ID, vec_u32_type_id);

        // Array
        let array_u32_type = Type::new(
            Path::default(),
            vec![],
            TypeDefArray::new(3, u32_type_id.into()),
            vec![],
        );
        let array_u32_type_id = builder.register_type(array_u32_type);
        assert_eq!(ARRAY_U32_TY_ID, array_u32_type_id);

        // Tuple
        let tuple_type = Type::new(
            Path::default(),
            vec![],
            TypeDefTuple::new_portable(vec![u32_type_id.into(), u64_type_id.into()]),
            vec![],
        );
        let tuple_type_id = builder.register_type(tuple_type);
        assert_eq!(TUPLE_TY_ID, tuple_type_id);

        // Compact
        let compact_type = Type::new(
            Path::default(),
            vec![],
            TypeDefCompact::new(tuple_type_id.into()),
            vec![],
        );
        let compact_type_id = builder.register_type(compact_type);
        assert_eq!(COMPACT_TY_ID, compact_type_id);

        // BitSequence
        let bit_seq_type = Type::new(
            Path::default(),
            vec![],
            TypeDefBitSequence::new_portable(u32_type_id.into(), u64_type_id.into()),
            vec![],
        );
        let bit_seq_type_id = builder.register_type(bit_seq_type);
        assert_eq!(BIT_SEQ_TY_ID, bit_seq_type_id);

        // Composite
        let composite_type = Type::builder_portable()
            .path(Path::from_segments_unchecked(["MyStruct".into()]))
            .composite(
                Fields::named()
                    .field_portable(|f| f.name("primitive".into()).ty(u32_type_id))
                    .field_portable(|f| f.name("vec_of_u32".into()).ty(vec_u32_type_id)),
            );
        let composite_type_id = builder.register_type(composite_type);
        assert_eq!(COMPOSITE_TY_ID, composite_type_id);

        // Variant
        let enum_type = Type::builder_portable()
            .path(Path::from_segments_unchecked(["MyEnum".into()]))
            .variant(
                Variants::new()
                    .variant("A".into(), |v| {
                        v.index(0).fields(
                            Fields::<PortableForm>::named()
                                .field_portable(|f| {
                                    f.name("primitive".into()).ty(u32_type_id)
                                })
                                .field_portable(|f| {
                                    f.name("vec_of_u32".into()).ty(vec_u32_type_id)
                                }),
                        )
                    })
                    .variant_unit("B".into(), 1),
            );
        let enum_type_id = builder.register_type(enum_type);
        assert_eq!(VARIANT_TY_ID, enum_type_id);

        builder.finish()
    }

    #[test]
    fn retain_recursive_seq() {
        let mut registry = build_registry();

        let ids_result = registry.retain(|id| id == VEC_U32_TY_ID);
        assert_eq!(ids_result.len(), 2);
        assert_eq!(ids_result.get(&U32_TY_ID), Some(&0));
        assert_eq!(ids_result.get(&VEC_U32_TY_ID), Some(&1));

        assert_eq!(registry.types.len(), 2);
        let expected_ty =
            Type::new(Path::default(), vec![], TypeDefPrimitive::U32, vec![]);
        assert_eq!(registry.resolve(0).unwrap(), &expected_ty);
        let expected_ty = Type::new(
            Path::default(),
            vec![],
            TypeDefSequence::new(0.into()),
            vec![],
        );
        assert_eq!(registry.resolve(1).unwrap(), &expected_ty);
    }

    #[test]
    fn retain_recursive_array() {
        let mut registry = build_registry();

        let ids_result = registry.retain(|id| id == ARRAY_U32_TY_ID);
        assert_eq!(ids_result.len(), 2);
        assert_eq!(ids_result.get(&U32_TY_ID), Some(&0));
        assert_eq!(ids_result.get(&ARRAY_U32_TY_ID), Some(&1));

        assert_eq!(registry.types.len(), 2);
        let expected_ty =
            Type::new(Path::default(), vec![], TypeDefPrimitive::U32, vec![]);
        assert_eq!(registry.resolve(0).unwrap(), &expected_ty);
        let expected_ty = Type::new(
            Path::default(),
            vec![],
            TypeDefArray::new(3, 0.into()),
            vec![],
        );
        assert_eq!(registry.resolve(1).unwrap(), &expected_ty);
    }

    #[test]
    fn retain_recursive_tuple() {
        let mut registry = build_registry();

        let ids_result = registry.retain(|id| id == TUPLE_TY_ID);
        assert_eq!(ids_result.len(), 3);
        assert_eq!(ids_result.get(&U32_TY_ID), Some(&0));
        assert_eq!(ids_result.get(&U64_TY_ID), Some(&1));
        assert_eq!(ids_result.get(&TUPLE_TY_ID), Some(&2));

        assert_eq!(registry.types.len(), 3);
        let expected_ty =
            Type::new(Path::default(), vec![], TypeDefPrimitive::U32, vec![]);
        assert_eq!(registry.resolve(0).unwrap(), &expected_ty);
        let expected_ty =
            Type::new(Path::default(), vec![], TypeDefPrimitive::U64, vec![]);
        assert_eq!(registry.resolve(1).unwrap(), &expected_ty);
        let expected_ty = Type::new(
            Path::default(),
            vec![],
            TypeDefTuple::new_portable(vec![0.into(), 1.into()]),
            vec![],
        );
        assert_eq!(registry.resolve(2).unwrap(), &expected_ty);
    }

    #[test]
    fn retain_recursive_compact() {
        let mut registry = build_registry();

        let ids_result = registry.retain(|id| id == COMPACT_TY_ID);
        assert_eq!(ids_result.len(), 4);
        assert_eq!(ids_result.get(&U32_TY_ID), Some(&0));
        assert_eq!(ids_result.get(&U64_TY_ID), Some(&1));
        assert_eq!(ids_result.get(&TUPLE_TY_ID), Some(&2));
        assert_eq!(ids_result.get(&COMPACT_TY_ID), Some(&3));

        assert_eq!(registry.types.len(), 4);
        let expected_ty =
            Type::new(Path::default(), vec![], TypeDefPrimitive::U32, vec![]);
        assert_eq!(registry.resolve(0).unwrap(), &expected_ty);
        let expected_ty =
            Type::new(Path::default(), vec![], TypeDefPrimitive::U64, vec![]);
        assert_eq!(registry.resolve(1).unwrap(), &expected_ty);
        let expected_ty = Type::new(
            Path::default(),
            vec![],
            TypeDefTuple::new_portable(vec![0.into(), 1.into()]),
            vec![],
        );
        assert_eq!(registry.resolve(2).unwrap(), &expected_ty);
        let expected_ty = Type::new(
            Path::default(),
            vec![],
            TypeDefCompact::new(2.into()),
            vec![],
        );
        assert_eq!(registry.resolve(3).unwrap(), &expected_ty);
    }

    #[test]
    fn retain_recursive_bit_seq() {
        let mut registry = build_registry();

        let ids_result = registry.retain(|id| id == BIT_SEQ_TY_ID);
        assert_eq!(ids_result.len(), 3);
        assert_eq!(ids_result.get(&U32_TY_ID), Some(&0));
        assert_eq!(ids_result.get(&U64_TY_ID), Some(&1));
        assert_eq!(ids_result.get(&BIT_SEQ_TY_ID), Some(&2));

        assert_eq!(registry.types.len(), 3);
        let expected_ty =
            Type::new(Path::default(), vec![], TypeDefPrimitive::U32, vec![]);
        assert_eq!(registry.resolve(0).unwrap(), &expected_ty);
        let expected_ty =
            Type::new(Path::default(), vec![], TypeDefPrimitive::U64, vec![]);
        assert_eq!(registry.resolve(1).unwrap(), &expected_ty);
        let expected_ty = Type::new(
            Path::default(),
            vec![],
            TypeDefBitSequence::new_portable(0.into(), 1.into()),
            vec![],
        );
        assert_eq!(registry.resolve(2).unwrap(), &expected_ty);
    }

    #[test]
    fn retain_recursive_composite() {
        let mut registry = build_registry();

        let ids_result = registry.retain(|id| id == COMPOSITE_TY_ID);
        assert_eq!(ids_result.len(), 3);
        assert_eq!(ids_result.get(&U32_TY_ID), Some(&0));
        assert_eq!(ids_result.get(&VEC_U32_TY_ID), Some(&1));
        assert_eq!(ids_result.get(&COMPOSITE_TY_ID), Some(&2));

        assert_eq!(registry.types.len(), 3);
        let expected_ty =
            Type::new(Path::default(), vec![], TypeDefPrimitive::U32, vec![]);
        assert_eq!(registry.resolve(0).unwrap(), &expected_ty);
        let expected_ty = Type::new(
            Path::default(),
            vec![],
            TypeDefSequence::new(0.into()),
            vec![],
        );
        assert_eq!(registry.resolve(1).unwrap(), &expected_ty);
        let expected_ty = Type::builder_portable()
            .path(Path::from_segments_unchecked(["MyStruct".into()]))
            .composite(
                Fields::named()
                    .field_portable(|f| f.name("primitive".into()).ty(0))
                    .field_portable(|f| f.name("vec_of_u32".into()).ty(1)),
            );
        assert_eq!(registry.resolve(2).unwrap(), &expected_ty);
    }

    #[test]
    fn retain_recursive_variant() {
        let mut registry = build_registry();

        let ids_result = registry.retain(|id| id == VARIANT_TY_ID);
        assert_eq!(ids_result.len(), 3);
        assert_eq!(ids_result.get(&U32_TY_ID), Some(&0));
        assert_eq!(ids_result.get(&VEC_U32_TY_ID), Some(&1));
        assert_eq!(ids_result.get(&VARIANT_TY_ID), Some(&2));

        assert_eq!(registry.types.len(), 3);
        let expected_ty =
            Type::new(Path::default(), vec![], TypeDefPrimitive::U32, vec![]);
        assert_eq!(registry.resolve(0).unwrap(), &expected_ty);
        let expected_ty = Type::new(
            Path::default(),
            vec![],
            TypeDefSequence::new(0.into()),
            vec![],
        );
        assert_eq!(registry.resolve(1).unwrap(), &expected_ty);
        let expected_ty = Type::builder_portable()
            .path(Path::from_segments_unchecked(["MyEnum".into()]))
            .variant(
                Variants::new()
                    .variant("A".into(), |v| {
                        v.index(0).fields(
                            Fields::<PortableForm>::named()
                                .field_portable(|f| f.name("primitive".into()).ty(0))
                                .field_portable(|f| f.name("vec_of_u32".into()).ty(1)),
                        )
                    })
                    .variant_unit("B".into(), 1),
            );
        assert_eq!(registry.resolve(2).unwrap(), &expected_ty);
    }

    #[test]
    fn type_ids_are_sequential() {
        let mut registry = Registry::new();
        registry.register_type(&MetaType::new::<u32>());
        registry.register_type(&MetaType::new::<bool>());
        registry.register_type(&MetaType::new::<Option<(u32, bool)>>());

        let readonly: PortableRegistry = registry.into();

        assert_eq!(4, readonly.types.len());

        for (expected, ty) in readonly.types.iter().enumerate() {
            assert_eq!(expected as u32, ty.id);
        }
    }

    #[test]
    fn construct_portable_registry() {
        let mut builder = PortableRegistryBuilder::new();
        let u32_type = Type::new(Path::default(), vec![], TypeDefPrimitive::U32, vec![]);
        let u32_type_id = builder.register_type(u32_type.clone());

        let vec_u32_type = Type::new(
            Path::default(),
            vec![],
            TypeDefSequence::new(u32_type_id.into()),
            vec![],
        );
        let vec_u32_type_id = builder.register_type(vec_u32_type.clone());

        let self_referential_type_id = builder.next_type_id();

        let composite_type = Type::builder_portable()
            .path(Path::from_segments_unchecked(["MyStruct".into()]))
            .composite(
                Fields::named()
                    .field_portable(|f| f.name("primitive".into()).ty(u32_type_id))
                    .field_portable(|f| f.name("vec_of_u32".into()).ty(vec_u32_type_id))
                    .field_portable(|f| {
                        f.name("self_referential".into())
                            .ty(self_referential_type_id)
                    }),
            );
        let composite_type_id = builder.register_type(composite_type.clone());

        assert_eq!(self_referential_type_id, composite_type_id);

        assert_eq!(builder.get(u32_type_id).unwrap(), &u32_type);
        assert_eq!(builder.get(vec_u32_type_id).unwrap(), &vec_u32_type);
        assert_eq!(builder.get(composite_type_id).unwrap(), &composite_type);

        let registry = builder.finish();

        assert_eq!(Some(&u32_type), registry.resolve(u32_type_id));
        assert_eq!(Some(&vec_u32_type), registry.resolve(vec_u32_type_id));
        assert_eq!(Some(&composite_type), registry.resolve(composite_type_id));
    }

    #[test]
    fn retain_ids() {
        let mut builder = PortableRegistryBuilder::new();
        let u32_type = Type::new(Path::default(), vec![], TypeDefPrimitive::U32, vec![]);
        let _u32_type_id = builder.register_type(u32_type);

        let u64_type = Type::new(Path::default(), vec![], TypeDefPrimitive::U64, vec![]);
        let u64_type_id = builder.register_type(u64_type.clone());

        let mut registry = builder.finish();
        assert_eq!(registry.types.len(), 2);

        let ids_result = registry.retain(|id| id == u64_type_id);
        assert_eq!(ids_result.len(), 1);
        assert_eq!(ids_result.get(&u64_type_id), Some(&0));

        assert_eq!(registry.types.len(), 1);
        assert_eq!(registry.resolve(0).unwrap(), &u64_type);
    }

    #[test]
    fn retain_recursive_ids() {
        let mut builder = PortableRegistryBuilder::new();
        let u32_type = Type::new(Path::default(), vec![], TypeDefPrimitive::U32, vec![]);
        let u32_type_id = builder.register_type(u32_type.clone());

        let u64_type = Type::new(Path::default(), vec![], TypeDefPrimitive::U64, vec![]);
        let _u64_type_id = builder.register_type(u64_type);

        let vec_u32_type = Type::new(
            Path::default(),
            vec![],
            TypeDefSequence::new(u32_type_id.into()),
            vec![],
        );
        let vec_u32_type_id = builder.register_type(vec_u32_type);

        let composite_type = Type::builder_portable()
            .path(Path::from_segments_unchecked(["MyStruct".into()]))
            .composite(
                Fields::named()
                    .field_portable(|f| f.name("primitive".into()).ty(u32_type_id))
                    .field_portable(|f| f.name("vec_of_u32".into()).ty(vec_u32_type_id)),
            );
        let composite_type_id = builder.register_type(composite_type);

        let composite_type_second = Type::builder_portable()
            .path(Path::from_segments_unchecked(["MyStructSecond".into()]))
            .composite(
                Fields::named()
                    .field_portable(|f| f.name("vec_of_u32".into()).ty(vec_u32_type_id))
                    .field_portable(|f| f.name("second".into()).ty(composite_type_id)),
            );
        let composite_type_second_id = builder.register_type(composite_type_second);

        let mut registry = builder.finish();
        assert_eq!(registry.types.len(), 5);

        let ids_result = registry.retain(|id| id == composite_type_second_id);
        assert_eq!(ids_result.len(), 4);
        assert_eq!(ids_result.get(&u32_type_id), Some(&0));
        assert_eq!(ids_result.get(&vec_u32_type_id), Some(&1));
        assert_eq!(ids_result.get(&composite_type_id), Some(&2));
        assert_eq!(ids_result.get(&composite_type_second_id), Some(&3));

        assert_eq!(registry.types.len(), 4);

        // New type IDs are generated in DFS manner.
        assert_eq!(registry.resolve(0).unwrap(), &u32_type);

        let expected_type = Type::new(
            Path::default(),
            vec![],
            TypeDefSequence::new(0.into()),
            vec![],
        );
        assert_eq!(registry.resolve(1).unwrap(), &expected_type);

        let expected_type = Type::builder_portable()
            .path(Path::from_segments_unchecked(["MyStruct".into()]))
            .composite(
                Fields::named()
                    .field_portable(|f| f.name("primitive".into()).ty(0))
                    .field_portable(|f| f.name("vec_of_u32".into()).ty(1)),
            );
        assert_eq!(registry.resolve(2).unwrap(), &expected_type);

        let expected_type = Type::builder_portable()
            .path(Path::from_segments_unchecked(["MyStructSecond".into()]))
            .composite(
                Fields::named()
                    .field_portable(|f| f.name("vec_of_u32".into()).ty(1))
                    .field_portable(|f| f.name("second".into()).ty(2)),
            );
        assert_eq!(registry.resolve(3).unwrap(), &expected_type);
    }
}
