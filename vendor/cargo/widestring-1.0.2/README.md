# widestring
[![Crates.io](https://img.shields.io/crates/v/widestring.svg)](https://crates.io/crates/widestring/) [![Documentation](https://docs.rs/widestring/badge.svg)](https://docs.rs/widestring/) ![Crates.io](https://img.shields.io/crates/l/widestring) [![Build status](https://github.com/starkat99/widestring-rs/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/starkat99/widestring-rs/actions/workflows/rust.yml)

A wide string Rust library for converting to and from wide strings, such as
those often used in Windows API or other FFI libaries. Both `u16` and `u32` string types are
provided, including support for UTF-16 and UTF-32, malformed encoding, C-style strings, etc.

Macros for converting string literals to UTF-16 and UTF-32 strings at compile time are also 
included.

*Requires Rust 1.58 or greater.* If you need support for older versions of Rust, use 0.x versions of
this crate.

## Documentation

- [Crate API Reference](https://docs.rs/widestring/)
- [Latest Changes](CHANGELOG.md)

### Optional Features

- **`alloc`** - Enabled by default. Enable use of the [`alloc`](https://doc.rust-lang.org/alloc/)
  crate when not using the `std` library.

  This enables the owned string types and aliases.

- **`std`** - Enabled by default. Enable features that depend on the Rust `std` library, including
  everything in the `alloc` feature.

## License

This library is distributed under the terms of either of:

* [MIT License](LICENSES/MIT.txt)
  ([http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* [Apache License, Version 2.0](LICENSES/Apache-2.0.txt)
  ([http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.

This project is [REUSE-compliant](https://reuse.software/spec/). Copyrights are retained by their
contributors. Some files may include explicit copyright notices and/or license
[SPDX identifiers](https://spdx.dev/ids/). For full authorship information, see the version control
history.

### Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
