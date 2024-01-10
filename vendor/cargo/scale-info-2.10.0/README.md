# scale-info &middot; [![build][a1]][a2] [![Latest Version][b1]][b2]

[a1]: https://github.com/paritytech/scale-info/workflows/Rust/badge.svg
[a2]: https://github.com/paritytech/scale-info/actions?query=workflow%3ARust+branch%3Amaster
[b1]: https://img.shields.io/crates/v/scale-info.svg
[b2]: https://crates.io/crates/scale-info

A library to describe Rust types, geared towards providing info about the structure of [SCALE
](https://github.com/paritytech/parity-scale-codec) encodable types.

The definitions provide third party tools (e.g. a UI client) with information about how they
are able to decode types agnostic of language.

At its core is the `TypeInfo` trait:

```rust
pub trait TypeInfo {
    type Identity: ?Sized + 'static;
    fn type_info() -> Type;
}
```

Types implementing this trait build up and return a `Type` struct:

```rust
pub struct Type<T: Form = MetaForm> {
    /// The unique path to the type. Can be empty for built-in types
    path: Path<T>,
    /// The generic type parameters of the type in use. Empty for non generic types
    type_params: Vec<T::Type>,
    /// The actual type definition
    type_def: TypeDef<T>,
}
```
Types are defined as one of the following variants:
```rust
pub enum TypeDef<T: Form = MetaForm> {
    /// A composite type (e.g. a struct or a tuple)
    Composite(TypeDefComposite<T>),
    /// A variant type (e.g. an enum)
    Variant(TypeDefVariant<T>),
    /// A sequence type with runtime known length.
    Sequence(TypeDefSequence<T>),
    /// An array type with compile-time known length.
    Array(TypeDefArray<T>),
    /// A tuple type.
    Tuple(TypeDefTuple<T>),
    /// A Rust primitive type.
    Primitive(TypeDefPrimitive),
}
```

## Built-in Type Definitions

The following "built-in" types have predefined `TypeInfo` definitions:

- **Primitives:** `bool`, `char`, `str`, `u8`, `u16`, `u32`, `u64`, `u128`, `i8`, `i16`, `i32`, `i64`, `i128`.

- **Sequence:** Variable size sequence of elements of `T`, where `T` implements `TypeInfo`. e.g. `[T]`, `&[T]`, `&mut
 [T]`, `Vec<T>`

- **Array:** Fixed size `[T: $n]` for any `T` which implements `TypeInfo`, where `$n` is one of the
 predefined sizes.

- **Tuple:** Tuples consisting of up to 10 fields with types implementing `TypeInfo`.

## User-defined Types

There are two kinds of user-defined types: `Composite` and `Variant`.

Both make use of the `Path` and `Field` types in their definition:

### Fields

A fundamental building block to represent user defined types is the `Field` struct which defines the `Type` of a
field together with its optional name. Builders for the user defined types enforce the invariant that either all
fields have a name (e.g. structs) or all fields are unnamed (e.g. tuples).

### Path

The path of a type is a unique sequence of identifiers. Rust types typically construct a path from
the namespace and the identifier e.g. `foo::bar::Baz` is converted to the path `["foo", "bar", 
"Baz"]`.

### Composite

[Composite data types](https://en.wikipedia.org/wiki/Composite_data_type) are composed of a set of `Fields`.

**Structs** are represented by a set of *named* fields, enforced during construction:

```rust
struct Foo<T> {
    bar: T,
    data: u64,
}

impl<T> TypeInfo for Foo<T>
where
    T: TypeInfo + 'static,
{
    type Identity = Self;

    fn type_info() -> Type {
        Type::builder()
            .path(Path::new("Foo", module_path!()))
            .type_params(vec![MetaType::new::<T>()])
            .composite(Fields::named()
                .field(|f| f.ty::<T>().name("bar").type_name("T"))
                .field(|f| f.ty::<u64>().name("data").type_name("u64"))
            )
    }
}
```

**Tuples** are represented by a set of *unnamed* fields, enforced during construction:

```rust
struct Foo(u32, bool);

impl TypeInfo for Foo {
    type Identity = Self;

    fn type_info() -> Type {
        Type::builder()
            .path(Path::new("Foo", module_path!()))
            .composite(Fields::unnamed()
                .field(|f| f.ty::<u32>().type_name("u32"))
                .field(|f| f.ty::<bool>().type_name("bool"))
            )
    }
}
```

### Variant

[Variant types](https://en.wikipedia.org/wiki/Tagged_union) aka enums or tagged unions are
composed of a set of variants. Variants can have unnamed fields, named fields or no fields at all:

```rust
enum Foo<T>{
    A(T),
    B { f: u32 },
    C,
}

impl<T> TypeInfo for Foo<T>
where
    T: TypeInfo + 'static,
{
    type Identity = Self;

    fn type_info() -> Type {
        Type::builder()
            .path(Path::new("Foo", module_path!()))
            .type_params(vec![MetaType::new::<T>()])
            .variant(
                Variants::new()
                   .variant("A", |v| v.fields(Fields::unnamed().field(|f| f.ty::<T>())))
                   .variant("B", |v| v.fields(Fields::named().field(|f| f.ty::<u32>().name("f").type_name("u32"))))
                   .variant_unit("C")
            )
    }
}
```

If no variants contain fields then the discriminant can be set explicitly, enforced by the
builder during construction:

```rust
enum Foo {
    A,
    B,
    C = 33,
}

impl TypeInfo for Foo {
    type Identity = Self;

    fn type_info() -> Type {
        Type::builder()
            .path(Path::new("Foo", module_path!()))
            .variant(
                Variants::new()
                    .variant("A", |v| v.index(1))
                    .variant("B", |v| v.index(2))
                    .variant("C", |v| v.index(33))
            )
    }
}
```

## The Registry

Information about types is provided within the so-called type registry (`Registry`).
Type definitions are registered there and are associated with unique IDs that the outside
can refer to, providing a lightweight way to decrease overhead instead of using type
identifiers.

All concrete `TypeInfo` structures have two forms:

- One meta form (`MetaType`) that acts as a bridge to other forms
- A portable form suitable for serialization.

The `IntoPortable` trait must also be implemented in order prepare a type
definition for serialization using an instance of the type registry.

After transformation all type definitions are stored in the type registry.
Note that the type registry should be serialized as part of the metadata structure where the
registered types are utilized to allow consumers to resolve the types.

## Encoding

The type registry can be encoded as:

- JSON (with the "serde" feature enabled).
- SCALE itself (using `parity-scale-codec`).

## Features

The following optional `cargo` features are available:

- **serde** includes support for json serialization/deserialization of the type registry. See example [here](https://github.com/paritytech/scale-info/blob/master/test_suite/tests/json.rs).
- **derive** reexports the [`scale-info-derive`](https://crates.io/crates/scale-info-derive) crate.

## Known issues

When deriving `TypeInfo` for a type with generic compact fields e.g.

```rust
#[derive(Encode, TypeInfo)]
struct Foo<S> { #[codec(compact)] a: S }
```

You may experience the following error when using this generic type without the correct bounds:

```sh
error[E0275]: overflow evaluating the requirement `_::_parity_scale_codec::Compact<_>: Decode`
```

See https://github.com/paritytech/scale-info/issues/65 for more information.

## Resources

- See usage for describing types for [`ink!`](https://github.com/paritytech/ink/blob/master/crates/metadata/src/specs.rs) smart contracts metadata.
- [Original design draft (*outdated*)](https://hackmd.io/0wWm0ueBSF26m2pBG5NaeQ?view)
