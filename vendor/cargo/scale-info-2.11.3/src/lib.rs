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

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

//! Efficient and space-efficient serialization of Rust types.
//!
//! This library provides structures to easily retrieve compile-time type
//! information at runtime and also to serialize this information in a
//! space-efficient form, aka `PortableForm`.
//!
//! # Registry
//!
//! At the heart of its functionality is the [`Registry`](`crate::Registry`)
//! that acts as a cache for known types in order to efficiently deduplicate
//! types and ensure a space-efficient serialization.
//!
//! # Type Information
//!
//! Information about types is provided via the [`TypeInfo`](`crate::TypeInfo`)
//! trait.
//!
//! This trait should be implemented for all types that are serializable.
//! `scale-info` provides implementations for all commonly used Rust standard
//! types and a derive macro for implementing of custom types.
//!
//! ## Deriving `TypeInfo`
//!
//! Enable the `derive` feature of this crate:
//!
//! ```toml
//! scale-info = { version = "2.0.0", features = ["derive"] }
//! ```
//!
//! ```ignore
//! use scale_info::TypeInfo;
//!
//! #[derive(TypeInfo)]
//! struct MyStruct {
//!     a: u32,
//!     b: MyEnum,
//! }
//!
//! #[derive(TypeInfo)]
//! enum MyEnum {
//!     A(bool),
//!     B { f: Vec<u8> },
//!     C,
//! }
//! ```
//!
//! ### Attributes
//!
//! #### `#[scale_info(bounds(..))]`
//!
//! Replace the auto-generated `where` clause bounds for the derived `TypeInfo` implementation.
//!
//! ```ignore
//! #[derive(TypeInfo)]
//! #[scale_info(bounds(T: TypeInfo + 'static))]
//! struct MyStruct<T> {
//!     a: Vec<T>,
//! }
//! ```
//!
//! The derive macro automatically adds `TypeInfo` bounds for all type parameters, and all field
//! types containing type parameters or associated types.
//!
//! This is naive and sometimes adds unnecessary bounds, since on a syntactical level there is no
//! way to differentiate between a generic type constructor and a type alias with a generic argument
//! e.g.
//!
//! ```ignore
//! trait MyTrait {
//!     type A;
//! }
//!
//! type MyAlias<T> = <T as MyTrait>::A;
//!
//! #[derive(TypeInfo)]
//! struct MyStruct<T> {
//!     a: MyAlias<T>,
//!     b: Vec<T>,
//! }
//! ```
//!
//! So for the above, since a `MyAlias<T>: TypeInfo` bound is required, and we can't distinguish
//! between `MyAlias<T>` and `Vec<T>`, then the `TypeInfo` bound is simply added for all
//! fields which contain any type param. In this case the redundant `Vec<T>: TypeInfo`
//! would be added.
//!
//! This is usually okay, but in some circumstances can cause problems, for example with the
//! [`overflow evaluating the requirement`] error [here](https://github.com/paritytech/scale-info/blob/master/test_suite/tests/ui/pass_custom_bounds_fix_overflow.rs).
//!
//! The `bounds` attribute provides an ["escape hatch"](https://serde.rs/attr-bound.html) to allow
//! the programmer control of the `where` clause on the generated `impl`, to solve this and other
//! issues that can't be foreseen by the auto-generated bounds heuristic.
//!
//! #### `#[scale_info(skip_type_params(..))]`
//!
//! Remove the requirement for the specified type params to implement `TypeInfo`.
//!
//! Consider a simple example of a type parameter which is used for associated types, but the type
//! itself does not carry any type information. Consider this common pattern:
//!
//! ```ignore
//! trait Config {
//!     type Balance;
//! }
//!
//! struct Runtime; // doesn't implement `TypeInfo`
//!
//! impl Config for Runtime {
//!     type Balance = u64;
//! }
//!
//! #[allow(unused)]
//! #[derive(TypeInfo)]
//! #[scale_info(skip_type_params(T))]
//! struct A<T: Config> {
//!     balance: T::Balance,
//!     marker: core::marker::PhantomData<T>,
//! }
//!
//! fn assert_type_info<T: scale_info::TypeInfo + 'static>() {}
//!
//! fn main() {
//!     // without the `skip_type_params` attribute this will fail.
//!     assert_type_info::<A<Runtime>>();
//! }
//! ```
//!
//! By default, the derived `TypeInfo` implementation will add the type of all type parameters to
//! the `TypeParameter` specification e.g.
//!
//! `type_params(vec![TypeParameter::new("T", Some(MetaType::new::<T>()))])`
//!
//! In the example above, this will cause a compiler error because `Runtime` is the concrete tyoe
//! for `T`, which does not satisfy the `TypeInfo` requirement of `MetaType::new::<T>()`.
//!
//! Simply adding a `TypeInfo` derive to `Runtime` is one way of solving this, but that could be
//! misleading (why does it need `TypeInfo` if a value of that type is never encoded?), and can
//! sometimes require adding `TypeInfo` bounds in other impl blocks.
//!
//! The `skip_type_params` attribute solves this, providing an additional "escape hatch" which
//! prevents the given type parameter's type information being required:
//!
//! `type_params(vec![TypeParameter::new("T", None)])`
//!
//! The generated type params do not now require `T` to implement `TypeInfo`, so the auto-generated
//! bound is not added to the generated `TypeInfo` `where` clause.
//!
//! #### Combining `bounds` and `skip_type_params`
//!
//! These two attributes can complement one another, particularly in the case where using `bounds`
//! would still require manually adding a `TypeInfo` bound for the type parameter:
//!
//! ```ignore
//! #[derive(TypeInfo)]
//! #[scale_info(bounds(), skip_type_params(T))]
//! struct A<T> {
//!     marker: core::marker::PhantomData<T>,
//! }
//! ```
//!
//! Without `skip_type_params` in the example above, it would require the `TypeInfo` bounds for `T`
//! to be added manually e.g. `#[scale_info(bounds(T: TypeInfo + 'static))]`. Since the intention of
//! the empty bounds is to **remove** all type bounds, then the addition of `skip_type_params`
//! allows this to compile successfully.
//!
//! ##### Precedence
//!
//! When used independently, both attributes modify the `where` clause of the derived `TypeInfo`
//! impl. When used together, the predicates supplied in the `bounds` attribute replaces *all*
//! auto-generated bounds, and `skip_type_params` will have no effect on the resulting `where`
//! clause.
//!
//! **Note:** When using `bounds` without `skip_type_params`, it is therefore required to manually
//! add a `TypeInfo` bound for any non skipped type parameters. The compiler will let you know.
//!
//! #### `#[scale_info(capture_docs = "default|always|never")]`
//!
//! Docs for types, fields and variants can all be captured by the `docs` feature being enabled.
//! This can be overridden using the `capture_docs` attribute:
//!
//! `#[scale_info(capture_docs = "default")]` will capture docs iff the `docs` feature is enabled.
//! This is the default if `capture_docs` is not specified.
//!
//! `#[scale_info(capture_docs = "always")]` will capture docs for the annotated type even if the
//! `docs` feature is *not* enabled.
//!
//! `#[scale_info(capture_docs = "never")]` will *not* capture docs for the annotated type even if
//! the `docs` is enabled.
//!
//! This is useful e.g. when compiling metadata into a Wasm blob, and it is desirable to keep the
//! binary size as small as possible, so the `docs` feature would be disabled. In case the docs for
//! some types is necessary they could be enabled on a per-type basis with the above attribute.
//!
//! #### `#[scale_info(crate = path::to::crate)]`
//!
//! Specify a path to the scale-info crate instance to use when referring to the APIs from generated
//! code. This is normally only applicable when invoking re-exported scale-info derives from a public
//! macro in a different crate. For example:
//! ```ignore
//! use scale_info_reexport::info::TypeInfo;
//!
//! #[derive(TypeInfo)]
//! #[scale_info(crate = scale_info_reexport::info)]
//! enum TestEnum {
//!     FirstVariant,
//!     SecondVariant,
//! }
//! ```
//!
//! #### `#[scale_info(replace_segment("search", "replace"))]`
//!
//! Specify to rename a segment in the `path` returned by the [`TypeInfo::path`] function.
//! Normally the path is generated by using the `module_path!` macro. This path includes
//! the crate name and all the modules up to the declaration of the type. Sometimes it
//! is useful to replace one of these segments to ensure that for example a renaming
//! of the crate isn't propagated to downstream users. Be aware that if a `crate-name`
//! contains an hypen, the actual segment is `crate_name`. The `replace` name needs
//! to be a valid Rust identifier. The attribute is allowed to be passed multiple
//! times to replace multiple segments.
//!
//! Example:
//! ```ignore
//! use scale_info_reexport::info::TypeInfo;
//!
//! #[derive(TypeInfo)]
//! #[scale_info(replace_segment("something", "better_name"))]
//! #[scale_info(replace_segment("TestEnum", "BetterEnumName"))]
//! enum TestEnum {
//!     FirstVariant,
//!     SecondVariant,
//! }
//! ```
//!
//! # Forms
//!
//! To bridge between compile-time type information and runtime the
//! [`MetaForm`](`crate::form::MetaForm`) is used to easily retrieve all
//! information needed to uniquely identify types.
//!
//! The `MetaForm` and its associated `Registry` can be transformed into the
//! space-efficient form by the [`IntoPortable`](`crate::IntoPortable`) trait; it is
//! used internally by the [`Registry`](`crate::Registry`) in order to convert
//! the expanded types into their space-efficient form.
//!
//! # Symbols and Namespaces
//!
//! To differentiate two types sharing the same name, namespaces are used.
//! Commonly the namespace is equal to the one where the type has been defined
//! in. For Rust prelude types such as [`Option`](`std::option::Option`) and
//! [`Result`](`std::result::Result`) the root namespace (empty namespace) is
//! used.
//!
//! To use this library simply use the [`MetaForm`](`crate::form::MetaForm`)
//! initially with your own data structures; make them generic over the
//! [`Form`](`crate::form::Form`) trait just as has been done in this crate with
//! [`TypeInfo`](`crate::TypeInfo`) in order to get a simple implementation of
//! [`IntoPortable`](`crate::IntoPortable`). Use a single instance of the
//! [`Registry`](`crate::Registry`) for compaction and provide this registry
//! instance upon serialization.
//!
//! A usage example can be found in ink! here:
//! https://github.com/paritytech/ink/blob/master/abi/src/specs.rs

/// Takes a number of types and returns a vector that contains their respective
/// [`MetaType`](`crate::MetaType`) instances.
///
/// This is useful for places that require inputs of iterators over [`MetaType`](`crate::MetaType`)
/// instances and provide a way out of code bloat in these scenarios.
///
/// # Example
///
/// ```
/// # use scale_info::tuple_meta_type;
/// assert_eq!(
///     tuple_meta_type!(i32, [u8; 32], String),
///     {
///         use scale_info::MetaType;
///         let mut vec = Vec::new();
///         vec.push(MetaType::new::<i32>());
///         vec.push(MetaType::new::<[u8; 32]>());
///         vec.push(MetaType::new::<String>());
///         vec
///     }
/// );
/// ```
#[macro_export]
macro_rules! tuple_meta_type {
    ( $($ty:ty),* ) => {
        {
            $crate::prelude::vec![
                $(
                    $crate::MetaType::new::<$ty>(),
                )*
            ]
        }
    }
}

/// Construct a vector of `TypeParameter`s from pairs of the name and the concrete type.
///
/// # Example
///
/// ```
/// # use scale_info::{named_type_params, MetaType, TypeParameter};
/// assert_eq!(
///     named_type_params![(T, u8), (U, u32)],
///     vec! [
///         TypeParameter::new("T", Some(MetaType::new::<u8>())),
///         TypeParameter::new("U", Some(MetaType::new::<u32>())),
///     ]
/// );
/// ```
#[macro_export]
macro_rules! named_type_params {
    ( $(($tp:ty, $ty:ty)),* ) => {
        {
            $crate::prelude::vec![
                $(
                    $crate::TypeParameter::<$crate::form::MetaForm>::new(
                        ::core::stringify!($tp),
                        Some($crate::MetaType::new::<$ty>())
                    ),
                )*
            ]
        }
    }
}

/// Construct a vector of [`TypeParameter`] instances with the name of the type parameter,
/// together with its concrete [`MetaType`].
#[macro_export]
macro_rules! type_params {
    ( $($ty:ty),* ) => {
        $crate::named_type_params!{ $( ($ty, $ty) ),* }
    }
}

pub mod prelude;

pub mod build;
pub mod form;
mod impls;
pub mod interner;
mod meta_type;
mod portable;
mod registry;
mod ty;
mod utils;

#[doc(hidden)]
pub use scale;

pub use self::{
    meta_type::MetaType,
    portable::{PortableRegistry, PortableRegistryBuilder, PortableType},
    registry::{IntoPortable, Registry},
    ty::*,
};

#[cfg(feature = "derive")]
pub use scale_info_derive::TypeInfo;

/// Implementors return their meta type information.
pub trait TypeInfo {
    /// The type identifying for which type info is provided.
    ///
    /// # Note
    ///
    /// This is used to uniquely identify a type via [`core::any::TypeId::of`]. In most cases it
    /// will just be `Self`, but can be used to unify different types which have the same encoded
    /// representation e.g. reference types `Box<T>`, `&T` and `&mut T`.
    type Identity: ?Sized + 'static;

    /// Returns the static type identifier for `Self`.
    fn type_info() -> Type;
}

/// Convenience trait for implementors, combining `TypeInfo` and `'static` bounds.
///
/// # Note
///
/// Currently because of the `'static` constraint on [`std::any::TypeId::of`] (see [`MetaType`]),
/// `TypeInfo` constraints must also be accompanied by a `'static` bound. This trait is useful to
/// implementors so only a single constraint is required.
pub trait StaticTypeInfo: TypeInfo + 'static {}

impl<T> StaticTypeInfo for T where T: TypeInfo + 'static {}

/// Returns the runtime bridge to the types compile-time type information.
pub fn meta_type<T>() -> MetaType
where
    T: ?Sized + TypeInfo + 'static,
{
    MetaType::new::<T>()
}

#[cfg(test)]
mod tests;
