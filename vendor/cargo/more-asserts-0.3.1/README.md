# [More Asserts](https://crates.io/crates/more-asserts) (for Rust).

[![Docs](https://docs.rs/more-asserts/badge.svg)](https://docs.rs/more-asserts)
[![Latest Version](https://img.shields.io/crates/v/arcstr.svg)](https://crates.io/crates/more-asserts)
![Minimum Rust Version](https://img.shields.io/badge/MSRV%201.46.0-blue.svg)

Small library providing assertion macros similar to the `{debug_,}assert_{eq,ne}` macros in the stdlib.

## Usage

```rust
use more_asserts as ma;

#[derive(Debug, PartialEq, PartialOrd)]
enum Example { Foo, Bar }

ma::assert_le!(3, 4);
ma::assert_ge!(
    10, 10,
    "You can pass a message too (just like `assert_eq!`)",
);
ma::debug_assert_lt!(
    1.3, 4.5,
    "Format syntax is supported ({}).",
    "also like `assert_eq!`"
);

ma::assert_gt!(
    Example::Bar, Example::Foo,
    "It works on anything that implements PartialOrd and Debug!",
);
```

## License

Public domain as explained by either [the Unlicense](./UNLICENSE) or [CC0](./LICENSE-CC0). If neither of these are acceptable, you can also choose use it under the MIT/Apache2 combo typical of Rust code.

*Yes, this is a lot. It was originally just "public domain as explained by CC0" (since CC0 was the public domain-alike license preferred by lawyers), but then [CC0 fell out of favor](https://lists.fedoraproject.org/archives/list/legal@lists.fedoraproject.org/thread/RRYM3CLYJYW64VSQIXY6IF3TCDZGS6LM/), so I added the others as alternatives: Unlicense as a (hopefully) better public domain analogue, and MIT/Apache2 since (I know some folks dislike the Unlicense and) Rust will implode if those ever become a problem.*
