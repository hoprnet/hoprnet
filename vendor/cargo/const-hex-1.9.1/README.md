# const-hex

[![github](https://img.shields.io/badge/github-danipopes/const--hex-8da0cb?style=for-the-badge&labelColor=555555&logo=github)](https://github.com/danipopes/const-hex)
[![crates.io](https://img.shields.io/crates/v/const-hex.svg?style=for-the-badge&color=fc8d62&logo=rust)](https://crates.io/crates/const-hex)
[![docs.rs](https://img.shields.io/badge/docs.rs-const--hex-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs)](https://docs.rs/const-hex)
[![build status](https://img.shields.io/github/actions/workflow/status/danipopes/const-hex/ci.yml?branch=master&style=for-the-badge)](https://github.com/danipopes/const-hex/actions?query=branch%3Amaster)

This crate provides a fast conversion of byte arrays to hexadecimal strings,
both at compile time, and at run time.

Extends the [`hex`] crate's implementation with [const-eval], a
[const-generics formatting buffer][buffer], similar to [`itoa`]'s, and more.

_Version requirement: rustc 1.64+_

[const-eval]: https://docs.rs/const-hex/latest/const_hex/fn.const_encode.html
[buffer]: https://docs.rs/const-hex/latest/const_hex/struct.Buffer.html
[`itoa`]: https://docs.rs/itoa/latest/itoa/struct.Buffer.html

## Performance

This crate is 5 to 20 times faster than [`hex`] in encoding and decoding, and
30+ times faster than `libstd` in formatting.

The following benchmarks were ran on an AMD Ryzen 9 5900X, compiled with
`rustc 1.74.0-nightly (203c57dbe 2023-09-17)` on `x86_64-unknown-linux-gnu`.

You can run these benchmarks with `cargo bench --features std` on a
nightly compiler.

```log
test decode::const_hex::bench1_32             ... bench:          23 ns/iter (+/- 1)
test decode::const_hex::bench2_256            ... bench:          98 ns/iter (+/- 4)
test decode::const_hex::bench3_2048           ... bench:         550 ns/iter (+/- 117)
test decode::const_hex::bench4_16384          ... bench:       4,104 ns/iter (+/- 35)
test decode::hex::bench1_32                   ... bench:         104 ns/iter (+/- 6)
test decode::hex::bench2_256                  ... bench:         828 ns/iter (+/- 14)
test decode::hex::bench3_2048                 ... bench:       6,229 ns/iter (+/- 113)
test decode::hex::bench4_16384                ... bench:      70,582 ns/iter (+/- 1,774)

test decode_to_slice::const_hex::bench1_32    ... bench:          12 ns/iter (+/- 1)
test decode_to_slice::const_hex::bench2_256   ... bench:          89 ns/iter (+/- 0)
test decode_to_slice::const_hex::bench3_2048  ... bench:         678 ns/iter (+/- 15)
test decode_to_slice::const_hex::bench4_16384 ... bench:       5,348 ns/iter (+/- 20)
test decode_to_slice::hex::bench1_32          ... bench:          57 ns/iter (+/- 0)
test decode_to_slice::hex::bench2_256         ... bench:         515 ns/iter (+/- 43)
test decode_to_slice::hex::bench3_2048        ... bench:       4,021 ns/iter (+/- 173)
test decode_to_slice::hex::bench4_16384       ... bench:      54,224 ns/iter (+/- 2,023)

test encode::const_hex::bench1_32             ... bench:          12 ns/iter (+/- 0)
test encode::const_hex::bench2_256            ... bench:          25 ns/iter (+/- 1)
test encode::const_hex::bench3_2048           ... bench:         129 ns/iter (+/- 1)
test encode::const_hex::bench4_16384          ... bench:         854 ns/iter (+/- 8)
test encode::hex::bench1_32                   ... bench:         134 ns/iter (+/- 1)
test encode::hex::bench2_256                  ... bench:         925 ns/iter (+/- 5)
test encode::hex::bench3_2048                 ... bench:       7,349 ns/iter (+/- 50)
test encode::hex::bench4_16384                ... bench:      58,211 ns/iter (+/- 2,474)

test encode_to_slice::const_hex::bench1_32    ... bench:           1 ns/iter (+/- 0)
test encode_to_slice::const_hex::bench2_256   ... bench:           8 ns/iter (+/- 0)
test encode_to_slice::const_hex::bench3_2048  ... bench:          70 ns/iter (+/- 0)
test encode_to_slice::const_hex::bench4_16384 ... bench:         586 ns/iter (+/- 10)
test encode_to_slice::hex::bench1_32          ... bench:          14 ns/iter (+/- 0)
test encode_to_slice::hex::bench2_256         ... bench:         130 ns/iter (+/- 3)
test encode_to_slice::hex::bench3_2048        ... bench:         794 ns/iter (+/- 1)
test encode_to_slice::hex::bench4_16384       ... bench:       6,401 ns/iter (+/- 9)

test format::const_hex::bench1_32             ... bench:          11 ns/iter (+/- 0)
test format::const_hex::bench2_256            ... bench:          27 ns/iter (+/- 0)
test format::const_hex::bench3_2048           ... bench:         173 ns/iter (+/- 0)
test format::const_hex::bench4_16384          ... bench:       1,409 ns/iter (+/- 7)
test format::std::bench1_32                   ... bench:         510 ns/iter (+/- 5)
test format::std::bench2_256                  ... bench:       3,836 ns/iter (+/- 347)
test format::std::bench3_2048                 ... bench:      30,298 ns/iter (+/- 253)
test format::std::bench4_16384                ... bench:     247,471 ns/iter (+/- 7,873)
```

## Acknowledgements

- [`hex`] for the initial encoding/decoding implementations
- [dtolnay]/[itoa] for the initial crate/library API layout

[`hex`]: https://crates.io/crates/hex
[dtolnay]: https://github.com/dtolnay
[itoa]: https://github.com/dtolnay/itoa

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in these crates by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
</sub>
