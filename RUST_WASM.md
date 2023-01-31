# Documentation of our usage of Rust & WASM

This file documents how we're using our Rust toolchain to build WASM compatible crates that are used in
our monorepo.

It is also meant as place that can contain various tips that make development of WASM-compatible creates easier,
including tips related to `wasm-bindgen`.

## Structure

All our packages contain the `crates` directory, that is meant to contain the Rust crates.
E.g. the `utils` package looks as follows:

```text
utils
    ├── crates
    │   ├── Makefile
    │   ├── utils-metrics
    │   │   ├── Cargo.toml
    │   │   ├── README.md
    │   │   ├── src
    │   │   │   ├── lib.rs
    │   │   │   └── metrics.rs
    │   │   └── tests
    │   │       └── wasm.rs
    │   └── utils-misc
    │       ├── Cargo.toml
    │       ├── src
    │       │   ├── lib.rs
    │       │   └── utils.rs
    │       └── tests
    │           └── node.rs

```

Each crate is prefixed using the name of the package, therefore the crate names are

- `utils-metrics`
- `utils-misc`

In addition, all Rust crates across all packages must be enumerated in `Cargo.toml` in the root of the Monorepo:

```toml
[workspace]

members = [
    "packages/real/crates/real-base",
    "packages/hoprd/crates/hoprd-misc",
    "packages/utils/crates/utils-misc",
    "packages/utils/crates/utils-metrics"
]

# ... etc.
```

The `Makefile` which is present in `crates` directory of each package takes care of building and testing the
crates for that respective package. The crates in a package must be enumerated on the first line of the respective `Makefile`.
e.g. in `utils` crate:

```makefile
# Append here when new Rust WASM crate is added (the list is space separated)
PACKAGES = utils-misc utils-metrics
##########################################

# ... etc.
```

The `Makefile` supports basic actions:

- `clean`
- `all`
- `install`
- `test`

### Testing

The unit tests are meant to be in the same module file, for each Rust module.
They are pure-Rust and are easy to debug with IDE.

The integration tests, that run in WASM runtime are currently not possible to be debugged
and are located in the `test` directory of a crate.

## Adding a new crate

To add a new Rust WASM module (crate) into an existing package:

1. `cd packages/<package>/crates`
2. `wasm-pack new my-crate --template https://github.com/hoprnet/hopr-wasm-template`, this will create a new Rust crate `my-crate` from a template.
3. remove the cloned Git repo: `rm -rf my-crate/.git`
4. add `my-crate` to `PACKAGES` space separated list in `Makefile`
5. add `my-crate` to `workspace.members` in `Cargo.toml` in the root of the Monorepo
6. run `make all && make install` for the first time
7. commit all changes: `git add my-crate && git commit -a`

You can use the following pattern in TS to export Rust types or functions:

```typescript
// Load `my-crate` crate
import { set_panic_hook as my_crate_panic_hook } from '../lib/my_crate.js'
my_crate_panic_hook()
export { foo } from '../lib/my_crate.js'
```

Note, that the panic hook needs to be installed for each crate separately (each crate run in separate execution environment).

Triggering the build of the entire monorepo (`make deps build`) should now also build
and integrate your new Rust crate. Similarly `make test` in the root of the monorepo
will trigger your Rust unit tests and WASM integration tests.

## Guidelines and tips

We distinguish between 3 different Rust types and expressions with respect to WASM:

- Rust-specific (WASM-incompatible)
- WASM-compatible
- WASM-specific

### To make something visible to WASM and Rust

Use `#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]` attribute on it (on a function, type or `impl` block).
The attribute makes sure, that when the `wasm` feature is enabled in Cargo, it will be available to BOTH WASM and non-WASM (pure Rust),
therefore you MUST use only types which are compatible with BOTH. Avoid using WASM-incompatible types and WASM-specific types in your
types and functions.

### Something available only to pure Rust

Do not use any attribute. Types NOT compatible with WASM can be used freely, unless the `#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]`
is specified.
Please note that:

- trait implementations, lifetimes & generic types are NOT supported by `wasm-bindgen` and therefore they can be used only in pure Rust.

### Something available only to WASM

Put it strictly inside the `wasm` submodule. The code in that module is built only when the `wasm` feature is enabled in Cargo,
and does not interfere with your pure Rust code (unless you intentionally `use` that module).
You can use WASM-specific types freely in this submodule.

## Avoid WASM-specific imports outside `wasm` submodule

Any WASM imports (such as from `wasm_bindgen`, `js_sys`,...etc.) ARE WASM-specific (obviously), and must be used strictly inside
the `wasm` submodule. This makes sure our code is buildable with the `wasm` Cargo feature turned off.

The following example shows our some guidelines how to properly implement WASM and non-WASM types and their definitions:

```rust
// This type will be made available to both WASM (when the "wasm" feature is turned on)
// and non-WASM (pure Rust)
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct MyStruct {
    foo: u32,       // private members do not need to have WASM-compatible types
    pub bar: u32    // public members MUST have WASM-compatible types, if struct used with #[cfg_attr(feature = "wasm"...
    // NOTE: you can use #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(skip))]
    // on a public attribute that shall not be available to WASM
}

// This function won't be available in WASM, even when the "wasm" feature is turned on
// You can use WASM-incompatible types in its arguments and return types freely.
pub fn foo() -> u32 { 42 }

// This function must not use WASM-specific types. Only WASM-compatible types must be used,
// because the attribute makes it available to both WASM and non-WASM (pure Rust)
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub fn bar() -> u32 { 0 }

impl MyStruct {
   // Here, specify methods with types that are strictly NOT WASM-compatible
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl MyStruct {
   // Here, specify methods with types that are strictly WASM-compatible, but not WASM-specific.
}

// NOTE: That several `impl` blocks of the same type can co-exist, if there are different attributes on them!

// Trait implementations for types can NEVER be made available for WASM (not currently supported by wasm-bindgen)
impl ToString for ChannelStatus {
    fn to_string(&self) -> String {
        todo!()
    }
}

/// Unit tests of pure Rust code (must not contain anything WASM-specific)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foo() {
        assert_eq!(42, foo());
    }
}

/// Module for WASM-specific Rust code
#[cfg(feature = "wasm")]
pub mod wasm {

    // Use this module to specify everything that is WASM-specific (e.g. uses wasm-bindgen types, js_sys, ...etc.)

    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;

    #[wasm_bindgen]
    pub fn foo(_val: JsValue) -> i32 {
        super::foo()
    }

    #[wasm_bindgen]
    impl MyStruct {
        // Specify methods of MyStruct which use WASM-specific
    }
}
```

### Types between WASM and Rust

The following sections list examples of Rust types with respect to WASM.
For a complete guidance, please refer to [wasm-bindgen docs](https://rustwasm.github.io/docs/wasm-bindgen/reference/types.html).

#### WASM-incompatible types & expressions

- fixed size arrays
- `Result<X,Y>` where one of `X` and `Y` is WASM-incompatible
- `Vec<X>` if `X` is not WASM-compatible
- any reference types used on return values
- trait implementations, lifetime specifiers & generic types

#### WASM-compatible types & expressions

Below are example of types are compatible with both pure Rust and WASM.
As mentioned in the previous text, these can be used anywhere
below the `#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]` attribute.

- `Box<[X]>` on public members, arguments and return types which are fixed length byte arrays. (`X` is Rust builtin number type)
- `Vec<X>` on public members, arguments and return types that can change at some point. (`X` is Rust builtin number type)
- `&[X]` on arguments only. (`X` is Rust builtin number type)
- `String` on public members and return types.
- `&str` on arguments only.
- any builtin Rust numeric types except `usize`, can be used on members, arguments and return types.
- WASM-incompatible types used as private members

#### WASM-specific types & expressions

The following are examples of WASM-specific types or expressions and MUST be used ONLY within the `wasm` submodule.

- `#[wasm_bindgen]`
- `JsValue`
- `Result<X, JsValue>`
- `JsString`, `JsFunction`, `Uint8Array`... and anything from `js_sys` crate
