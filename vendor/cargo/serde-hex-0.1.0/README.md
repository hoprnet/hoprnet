# serde-hex [![Build Status](https://travis-ci.org/forrest-marshall/serde-hex.svg?branch=master)](https://travis-ci.org/forrest-marshall/serde-hex)

Rust crate for easy and versatile serialization/deserialization of hexadecimal values.


## Overview

The `serde-hex` crate is built around the `SerHex` trait, which makes it easy to specify
custom hexadecimal serialization/deserialization with serde.  Once implemented, the `SerHex`
trait is configured via its generic type parameter (e.g.; `SerHex<Compact>` for hexadecimal
values with no leading zeroes, or `SerHex<StrictPfx>` for leading zeroes and the `0x` prefix).
Thanks to Rust's very magical compiler, only the components of the serialization/deserialization
functionality that your configuration actually uses get compiled in.  The most common way to
use `SerHex` is when deriving the `Serialize` and `Deserialize` trait for your types.
Here is a simple example using [`serde_derive`](https://crates.io/crates/serde_derive) and
[`serde_json`](https://crates.io/crates/serde_json):

```rust

use serde_hex::{SerHex,StrictPfx,CompactPfx};

#[derive(Debug,PartialEq,Eq,Serialize,Deserialize)]
struct Foo {
    #[serde(with = "SerHex::<StrictPfx>")]
    bar: [u8;4],
    #[serde(with = "SerHex::<CompactPfx>")]
    bin: u64
}

fn it_works() {
    let foo = Foo { bar: [0,1,2,3], bin: 16 };
    let ser = serde_json::to_string(&foo).unwrap();
    let exp = r#"{"bar":"0x00010203","bin":"0x10"}"#;
    assert_eq!(ser,exp);
}

```

`SerHex` is automatically implemented for all unsigned integer types, and all `Strict` 
variants are implemented for arrays of `[T;1]` through `[T;64]` (where `T: SerHex` of course).
We skip default impls for arrays of length zero, as well as signed integers & floats, 
since there isn't any particularly intuitive way to represent these values.

This crate provides a number of helpful macros for implementing `SerHex` on common
patterns.  If none of the macros suit your needs, a number of utility functions are
also provided to make implementing custom variations as painless as possible.

## Note

Check out the widely used [`hex`](https://crates.io/crates/hex) crate if you are just 
looking for generic hexadecimal conversion traits.  This crate is intended fairly
specifically for customizeable `serde` interop, and the generic traits of the
`hex` crate should be preferred if that isn't what you are looking for.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.


### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

