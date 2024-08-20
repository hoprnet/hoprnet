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
    prelude::{collections::BTreeMap, fmt::Debug, mem, vec::Vec},
    Path, Registry, Type, TypeDef, TypeDefPrimitive,
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
                .map(|(k, v)| PortableType {
                    id: k.id,
                    ty: v.clone(),
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

        fn placeholder_type() -> PortableType {
            PortableType {
                id: u32::MAX,
                ty: Type {
                    type_def: TypeDef::Primitive(TypeDefPrimitive::Bool),
                    path: Path::default(),
                    type_params: crate::prelude::vec![],
                    docs: crate::prelude::vec![],
                },
            }
        }

        fn retain_type(
            id: u32,
            types: &mut [PortableType],
            new_types: &mut Vec<PortableType>,
            retained_mappings: &mut BTreeMap<u32, u32>,
        ) -> u32 {
            // Type already retained; just return the new ID for it:
            if let Some(id) = retained_mappings.get(&id) {
                return *id;
            }

            // First, save a spot for this type in our new registry. We do this straight away
            // so that we can add the type ID to the retained mappings _before_ recursing into
            // it below. This means that if a type contains itself, we'll bail above when we
            // see the same type again.
            let new_id = new_types.len() as u32;
            new_types.push(placeholder_type());
            retained_mappings.insert(id, new_id);

            // Now, take the actual type we'll be retaining out of the old registry,
            // swapping it with a placeholder type to avoid any allocations. Because of
            // the above, nothing should ever try to access this placeholder type anyway.
            let mut ty = mem::replace(&mut types[id as usize], placeholder_type());
            ty.id = new_id;

            // Now we recursively retain any type parameters in the type we're retaining.
            // Update their IDs to point to the new locations of the retained types.
            for param in ty.ty.type_params.iter_mut() {
                let Some(param_ty) = &param.ty else { continue };
                let new_id = retain_type(param_ty.id, types, new_types, retained_mappings);
                param.ty = Some(Into::into(new_id));
            }

            // Also recurse into any types inside this type to retain them too. Update their IDs
            // to point to the new locations of the retained types.
            match &mut ty.ty.type_def {
                TypeDef::Composite(composite) => {
                    for field in composite.fields.iter_mut() {
                        let new_id = retain_type(field.ty.id, types, new_types, retained_mappings);
                        field.ty = new_id.into();
                    }
                }
                TypeDef::Variant(variant) => {
                    for var in variant.variants.iter_mut() {
                        for field in var.fields.iter_mut() {
                            let new_id =
                                retain_type(field.ty.id, types, new_types, retained_mappings);
                            field.ty = new_id.into();
                        }
                    }
                }
                TypeDef::Sequence(sequence) => {
                    let new_id =
                        retain_type(sequence.type_param.id, types, new_types, retained_mappings);
                    sequence.type_param = new_id.into();
                }
                TypeDef::Array(array) => {
                    let new_id =
                        retain_type(array.type_param.id, types, new_types, retained_mappings);
                    array.type_param = new_id.into();
                }
                TypeDef::Tuple(tuple) => {
                    for ty in tuple.fields.iter_mut() {
                        let new_id = retain_type(ty.id, types, new_types, retained_mappings);
                        *ty = new_id.into();
                    }
                }
                TypeDef::Primitive(_) => (),
                TypeDef::Compact(compact) => {
                    let new_id =
                        retain_type(compact.type_param.id, types, new_types, retained_mappings);
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

            // Now we've updated the IDs etc of this type, we put it into the new registry
            // and override our placeholder type that was saving its space for us.
            new_types[new_id as usize] = ty;
            new_id
        }

        for id in 0..self.types.len() as u32 {
            // We don't care about the type; move on:
            if !filter(id) {
                continue;
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
            .map(|(i, ty)| PortableType {
                id: i as u32,
                ty: ty.clone(),
            })
            .collect();
        PortableRegistry { types }
    }
}

#[cfg(test)]
mod tests {
    use scale::Compact;

    use super::*;
    use crate::ty::TypeDefPrimitive;
    use crate::{build::*, prelude::vec, *};

    fn ty<T: TypeInfo + 'static>() -> MetaType {
        MetaType::new::<T>()
    }

    fn make_registry(tys: impl IntoIterator<Item = MetaType>) -> (Vec<u32>, PortableRegistry) {
        // Register our types, recording the corresponding IDs.
        let mut types = Registry::new();
        let mut ids = vec![];
        for ty in tys.into_iter() {
            let id = types.register_type(&ty);
            ids.push(id.id);
        }

        let registry = types.into();
        (ids, registry)
    }

    #[test]
    fn retain_seq_type() {
        let (ids, mut registry) = make_registry([ty::<bool>(), ty::<Vec<u32>>(), ty::<String>()]);

        assert_eq!(registry.types.len(), 4);

        // Retain only the vec.
        let vec_id = ids[1];
        let retained_ids = registry.retain(|id| id == vec_id);

        assert_eq!(retained_ids.len(), 2);
        assert_eq!(registry.types.len(), 2);

        // Check that vec was retained and has correct ID.
        let new_vec_id = *retained_ids
            .get(&vec_id)
            .expect("vec should have been retained");
        let registry_ty = registry
            .types
            .get(new_vec_id as usize)
            .expect("vec should exist");

        assert_eq!(registry_ty.id, new_vec_id);

        // Check that vec type info is as expected.
        let seq = match &registry_ty.ty.type_def {
            TypeDef::Sequence(s) => s,
            def => panic!("Expected a sequence type, got {def:?}"),
        };

        let vec_param = registry
            .resolve(seq.type_param.id)
            .expect("vec param should be exist");
        assert!(matches!(
            vec_param.type_def,
            TypeDef::Primitive(TypeDefPrimitive::U32)
        ));
    }

    #[test]
    fn retain_array_type() {
        let (ids, mut registry) = make_registry([ty::<bool>(), ty::<[u32; 16]>(), ty::<String>()]);

        assert_eq!(registry.types.len(), 4);

        // Retain only the array.
        let arr_id = ids[1];
        let retained_ids = registry.retain(|id| id == arr_id);

        assert_eq!(retained_ids.len(), 2);
        assert_eq!(registry.types.len(), 2);

        // Check that array was retained and has correct ID.
        let new_arr_id = *retained_ids
            .get(&arr_id)
            .expect("array should have been retained");
        let registry_ty = registry
            .types
            .get(new_arr_id as usize)
            .expect("array should exist");

        assert_eq!(registry_ty.id, new_arr_id);

        // Check that array type info is as expected.
        let arr = match &registry_ty.ty.type_def {
            TypeDef::Array(a) => a,
            def => panic!("Expected an array type, got {def:?}"),
        };

        let array_param = registry
            .resolve(arr.type_param.id)
            .expect("array param should be exist");
        assert!(matches!(
            array_param.type_def,
            TypeDef::Primitive(TypeDefPrimitive::U32)
        ));
    }

    #[test]
    fn retain_tuple_type() {
        let (ids, mut registry) =
            make_registry([ty::<bool>(), ty::<(u32, [u8; 32], bool)>(), ty::<String>()]);

        assert_eq!(registry.types.len(), 6);

        // Retain only the tuple.
        let tuple_id = ids[1];
        let retained_ids = registry.retain(|id| id == tuple_id);

        // We only actually ditch the String when retaining:
        assert_eq!(retained_ids.len(), 5);
        assert_eq!(registry.types.len(), 5);

        // Check that tuple was retained and has correct ID.
        let new_tuple_id = *retained_ids
            .get(&tuple_id)
            .expect("tuple should have been retained");
        let registry_ty = registry
            .types
            .get(new_tuple_id as usize)
            .expect("tuple should exist");

        assert_eq!(registry_ty.id, new_tuple_id);

        // Check that tuple type info is as expected.
        let tup = match &registry_ty.ty.type_def {
            TypeDef::Tuple(t) => t,
            def => panic!("Expected an tuple type, got {def:?}"),
        };

        // Check that tuple fields are as expected.
        assert!(matches!(
            registry.resolve(tup.fields[0].id).unwrap().type_def,
            TypeDef::Primitive(TypeDefPrimitive::U32)
        ));
        assert!(matches!(
            registry.resolve(tup.fields[1].id).unwrap().type_def,
            TypeDef::Array(_)
        ));
        assert!(matches!(
            registry.resolve(tup.fields[2].id).unwrap().type_def,
            TypeDef::Primitive(TypeDefPrimitive::Bool)
        ));
    }

    #[test]
    fn retain_composite_type() {
        #[derive(scale_info_derive::TypeInfo)]
        #[allow(dead_code)]
        struct Foo {
            a: u32,
            b: [u8; 32],
            c: bool,
        }

        let (ids, mut registry) = make_registry([ty::<bool>(), ty::<Foo>(), ty::<String>()]);

        assert_eq!(registry.types.len(), 6);

        // Retain only the struct.
        let struct_id = ids[1];
        let retained_ids = registry.retain(|id| id == struct_id);

        // We only actually ditch the String when retaining:
        assert_eq!(retained_ids.len(), 5);
        assert_eq!(registry.types.len(), 5);

        // Check that struct was retained and has correct ID.
        let struct_id = *retained_ids
            .get(&struct_id)
            .expect("struct should have been retained");
        let registry_ty = registry
            .types
            .get(struct_id as usize)
            .expect("struct should exist");

        assert_eq!(registry_ty.id, struct_id);

        // Check that struct type info is as expected.
        let struc = match &registry_ty.ty.type_def {
            TypeDef::Composite(s) => s,
            def => panic!("Expected an struct type, got {def:?}"),
        };

        // Check that struct fields are as expected.
        assert_eq!(struc.fields.len(), 3);
        assert_eq!(struc.fields[0].name, Some("a".to_owned()));
        assert!(matches!(
            registry.resolve(struc.fields[0].ty.id).unwrap().type_def,
            TypeDef::Primitive(TypeDefPrimitive::U32)
        ));
        assert_eq!(struc.fields[1].name, Some("b".to_owned()));
        assert!(matches!(
            registry.resolve(struc.fields[1].ty.id).unwrap().type_def,
            TypeDef::Array(_)
        ));
        assert_eq!(struc.fields[2].name, Some("c".to_owned()));
        assert!(matches!(
            registry.resolve(struc.fields[2].ty.id).unwrap().type_def,
            TypeDef::Primitive(TypeDefPrimitive::Bool)
        ));
    }

    #[test]
    fn retain_variant_type() {
        #[derive(scale_info_derive::TypeInfo)]
        #[allow(dead_code)]
        enum Foo {
            A(u32),
            B(bool),
        }

        let (ids, mut registry) = make_registry([ty::<bool>(), ty::<Foo>(), ty::<String>()]);

        assert_eq!(registry.types.len(), 4);

        // Retain only the variant.
        let variant_id = ids[1];
        let retained_ids = registry.retain(|id| id == variant_id);

        // We only actually ditch the String when retaining:
        assert_eq!(retained_ids.len(), 3);
        assert_eq!(registry.types.len(), 3);

        // Check that variant was retained and has correct ID.
        let variant_id = *retained_ids
            .get(&variant_id)
            .expect("variant should have been retained");
        let registry_ty = registry
            .types
            .get(variant_id as usize)
            .expect("variant should exist");

        assert_eq!(registry_ty.id, variant_id);

        // Check that variant type info is as expected.
        let var = match &registry_ty.ty.type_def {
            TypeDef::Variant(v) => v,
            def => panic!("Expected a variant type, got {def:?}"),
        };

        assert_eq!(var.variants.len(), 2);
        assert_eq!(var.variants[0].name, "A".to_owned());
        assert_eq!(var.variants[0].fields.len(), 1);
        assert!(matches!(
            registry
                .resolve(var.variants[0].fields[0].ty.id)
                .unwrap()
                .type_def,
            TypeDef::Primitive(TypeDefPrimitive::U32)
        ));

        assert_eq!(var.variants[1].name, "B".to_owned());
        assert_eq!(var.variants[1].fields.len(), 1);
        assert!(matches!(
            registry
                .resolve(var.variants[1].fields[0].ty.id)
                .unwrap()
                .type_def,
            TypeDef::Primitive(TypeDefPrimitive::Bool)
        ));
    }

    #[test]
    fn retain_compact_type() {
        let (ids, mut registry) =
            make_registry([ty::<bool>(), ty::<String>(), ty::<Compact<u32>>()]);

        assert_eq!(registry.types.len(), 4);

        // Retain only the compact.
        let compact_id = ids[2];
        let retained_ids = registry.retain(|id| id == compact_id);

        assert_eq!(retained_ids.len(), 2);
        assert_eq!(registry.types.len(), 2);

        // Check that compact was retained and has correct ID.
        let compact_id = *retained_ids
            .get(&compact_id)
            .expect("compact should have been retained");
        let registry_ty = registry
            .types
            .get(compact_id as usize)
            .expect("compact should exist");

        assert_eq!(registry_ty.id, compact_id);

        // Check that compact type info is as expected.
        let compact = match &registry_ty.ty.type_def {
            TypeDef::Compact(c) => c,
            def => panic!("Expected a compact type, got {def:?}"),
        };

        // And the compact param should be a u32.
        assert!(matches!(
            registry.resolve(compact.type_param.id).unwrap().type_def,
            TypeDef::Primitive(TypeDefPrimitive::U32)
        ));
    }

    #[test]
    fn retain_bitsequence_type() {
        // Use a more verbose method to build the registry to avoid
        // needing to pull in BitVec as a dev dep:
        let mut builder = PortableRegistryBuilder::new();

        // Register a couple of primitives:
        let bool_type = Type::new(Path::default(), vec![], TypeDefPrimitive::Bool, vec![]);
        let _bool_type_id = builder.register_type(bool_type);

        let u32_type = Type::new(Path::default(), vec![], TypeDefPrimitive::U32, vec![]);
        let u32_type_id = builder.register_type(u32_type);

        let u64_type = Type::new(Path::default(), vec![], TypeDefPrimitive::U64, vec![]);
        let u64_type_id = builder.register_type(u64_type);

        // Register a bit sequence:
        let bit_seq_type = Type::new(
            Path::default(),
            vec![],
            TypeDefBitSequence::new_portable(u32_type_id.into(), u64_type_id.into()),
            vec![],
        );
        let bit_seq_type_id = builder.register_type(bit_seq_type);

        // Now we have a registry with the above types in:
        let mut registry = builder.finish();

        assert_eq!(registry.types.len(), 4);

        // Retain only the bitseq.
        let retained_ids = registry.retain(|id| id == bit_seq_type_id);

        // One bitsequence entry + 2 params for it:
        assert_eq!(retained_ids.len(), 3);
        assert_eq!(registry.types.len(), 3);

        // Check that bitseq was retained and has correct ID.
        let bitseq_id = *retained_ids
            .get(&bit_seq_type_id)
            .expect("bitseq should have been retained");
        let registry_ty = registry
            .types
            .get(bitseq_id as usize)
            .expect("bitseq should exist");

        assert_eq!(registry_ty.id, bitseq_id);

        // Check that bitseq type info is as expected.
        let bitseq = match &registry_ty.ty.type_def {
            TypeDef::BitSequence(b) => b,
            def => panic!("Expected a bit sequence type, got {def:?}"),
        };
        assert!(matches!(
            registry.resolve(bitseq.bit_store_type.id).unwrap().type_def,
            TypeDef::Primitive(TypeDefPrimitive::U32)
        ));
        assert!(matches!(
            registry.resolve(bitseq.bit_order_type.id).unwrap().type_def,
            TypeDef::Primitive(TypeDefPrimitive::U64)
        ));
    }

    #[test]
    fn retain_recursive_type() {
        #[derive(scale_info_derive::TypeInfo)]
        #[allow(dead_code)]
        enum Recursive {
            Value(Box<Recursive>),
            Empty,
        }

        let (ids, mut registry) = make_registry([ty::<bool>(), ty::<Recursive>(), ty::<String>()]);

        assert_eq!(registry.types.len(), 3);

        // Retain only the recursive type.
        let variant_id = ids[1];
        let retained_ids = registry.retain(|id| id == variant_id);

        assert_eq!(retained_ids.len(), 1);
        assert_eq!(registry.types.len(), 1);

        // Check that variant was retained and has correct ID.
        let variant_id = *retained_ids
            .get(&variant_id)
            .expect("variant should have been retained");
        let registry_ty = registry
            .types
            .get(variant_id as usize)
            .expect("variant should exist");

        assert_eq!(registry_ty.id, variant_id);

        // Check that variant type info is as expected.
        let var = match &registry_ty.ty.type_def {
            TypeDef::Variant(v) => v,
            def => panic!("Expected a variant type, got {def:?}"),
        };

        assert_eq!(var.variants.len(), 2);
        assert_eq!(var.variants[0].name, "Value".to_owned());
        assert_eq!(var.variants[0].fields.len(), 1);
        assert!(matches!(
            registry
                .resolve(var.variants[0].fields[0].ty.id)
                .unwrap()
                .type_def,
            TypeDef::Variant(_)
        ));

        assert_eq!(var.variants[1].name, "Empty".to_owned());
        assert_eq!(var.variants[1].fields.len(), 0);
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
}
