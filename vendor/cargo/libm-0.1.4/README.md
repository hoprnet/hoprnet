# `libm`

[![Build Status](https://dev.azure.com/rust-lang/libm/_apis/build/status/rust-lang-nursery.libm?branchName=master)](https://dev.azure.com/rust-lang/libm/_build/latest?definitionId=7&branchName=master)

A port of [MUSL]'s libm to Rust.

[MUSL]: https://www.musl-libc.org/

## Goals

The short term goal of this library is to [enable math support (e.g. `sin`, `atan2`) for the
`wasm32-unknown-unknown` target][wasm] (cf. [rust-lang-nursery/compiler-builtins][pr]). The longer
term goal is to enable [math support in the `core` crate][core].

[wasm]: https://github.com/rust-lang-nursery/libm/milestone/1
[pr]: https://github.com/rust-lang-nursery/compiler-builtins/pull/248
[core]: https://github.com/rust-lang-nursery/libm/milestone/2

## Already usable

This crate is [on crates.io] and can be used today in stable `#![no_std]` programs like this:

[on crates.io]: https://crates.io/crates/libm

``` rust
#![no_std]

extern crate libm;

use libm::F32Ext; // adds methods to `f32`

fn foo(x: f32) {
    let y = x.sqrt();
    let z = libm::truncf(x);
}
```

The API documentation can be found [here](https://docs.rs/libm).

## Contributing

Please check [CONTRIBUTING.md](CONTRIBUTING.md)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
