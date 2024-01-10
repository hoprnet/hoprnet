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

This crate's performance is comparable with [`faster-hex`], but the latter only
provides specialized implementations for `x86`/`x86-64`.

This crate is 10 to 50 times faster than [`hex`] in encoding and decoding, and
100+ times faster than `libstd` in formatting.

The following benchmarks were ran on an AMD Ryzen 9 5900X (AVX2), compiled with
`rustc 1.75.0-nightly (aa1a71e9e 2023-10-26)` on `x86_64-unknown-linux-gnu`.

You can run these benchmarks with `cargo bench --features std` on a nightly
compiler.

```log
test decode::const_hex::bench1_32              ... bench:          19 ns/iter (+/- 0)
test decode::const_hex::bench2_256             ... bench:          28 ns/iter (+/- 1)
test decode::const_hex::bench3_2048            ... bench:         133 ns/iter (+/- 4)
test decode::const_hex::bench4_16384           ... bench:         948 ns/iter (+/- 11)
test decode::faster_hex::bench1_32             ... bench:          23 ns/iter (+/- 0)
test decode::faster_hex::bench2_256            ... bench:          46 ns/iter (+/- 0)
test decode::faster_hex::bench3_2048           ... bench:         148 ns/iter (+/- 7)
test decode::faster_hex::bench4_16384          ... bench:       1,063 ns/iter (+/- 22)
test decode::hex::bench1_32                    ... bench:          63 ns/iter (+/- 11)
test decode::hex::bench2_256                   ... bench:         440 ns/iter (+/- 12)
test decode::hex::bench3_2048                  ... bench:       3,346 ns/iter (+/- 574)
test decode::hex::bench4_16384                 ... bench:      43,589 ns/iter (+/- 2,809)

test decode_to_slice::const_hex::bench1_32     ... bench:          11 ns/iter (+/- 1)
test decode_to_slice::const_hex::bench2_256    ... bench:          18 ns/iter (+/- 2)
test decode_to_slice::const_hex::bench3_2048   ... bench:         123 ns/iter (+/- 5)
test decode_to_slice::const_hex::bench4_16384  ... bench:         940 ns/iter (+/- 41)
test decode_to_slice::faster_hex::bench1_32    ... bench:          11 ns/iter (+/- 0)
test decode_to_slice::faster_hex::bench2_256   ... bench:          19 ns/iter (+/- 0)
test decode_to_slice::faster_hex::bench3_2048  ... bench:         130 ns/iter (+/- 7)
test decode_to_slice::faster_hex::bench4_16384 ... bench:       1,011 ns/iter (+/- 33)
test decode_to_slice::hex::bench1_32           ... bench:          22 ns/iter (+/- 2)
test decode_to_slice::hex::bench2_256          ... bench:         171 ns/iter (+/- 8)
test decode_to_slice::hex::bench3_2048         ... bench:       1,650 ns/iter (+/- 221)
test decode_to_slice::hex::bench4_16384        ... bench:      15,614 ns/iter (+/- 736)

test encode::const_hex::bench1_32              ... bench:          13 ns/iter (+/- 1)
test encode::const_hex::bench2_256             ... bench:          26 ns/iter (+/- 0)
test encode::const_hex::bench3_2048            ... bench:         136 ns/iter (+/- 1)
test encode::const_hex::bench4_16384           ... bench:         863 ns/iter (+/- 39)
test encode::faster_hex::bench1_32             ... bench:          19 ns/iter (+/- 0)
test encode::faster_hex::bench2_256            ... bench:          34 ns/iter (+/- 0)
test encode::faster_hex::bench3_2048           ... bench:         135 ns/iter (+/- 1)
test encode::faster_hex::bench4_16384          ... bench:         825 ns/iter (+/- 8)
test encode::hex::bench1_32                    ... bench:         148 ns/iter (+/- 7)
test encode::hex::bench2_256                   ... bench:       1,037 ns/iter (+/- 17)
test encode::hex::bench3_2048                  ... bench:       8,186 ns/iter (+/- 295)
test encode::hex::bench4_16384                 ... bench:      65,257 ns/iter (+/- 15,832)

test encode_to_slice::const_hex::bench1_32     ... bench:           1 ns/iter (+/- 0)
test encode_to_slice::const_hex::bench2_256    ... bench:           8 ns/iter (+/- 0)
test encode_to_slice::const_hex::bench3_2048   ... bench:          79 ns/iter (+/- 1)
test encode_to_slice::const_hex::bench4_16384  ... bench:         577 ns/iter (+/- 11)
test encode_to_slice::faster_hex::bench1_32    ... bench:           5 ns/iter (+/- 0)
test encode_to_slice::faster_hex::bench2_256   ... bench:           9 ns/iter (+/- 0)
test encode_to_slice::faster_hex::bench3_2048  ... bench:          68 ns/iter (+/- 3)
test encode_to_slice::faster_hex::bench4_16384 ... bench:         533 ns/iter (+/- 11)
test encode_to_slice::hex::bench1_32           ... bench:          10 ns/iter (+/- 0)
test encode_to_slice::hex::bench2_256          ... bench:         129 ns/iter (+/- 10)
test encode_to_slice::hex::bench3_2048         ... bench:         844 ns/iter (+/- 75)
test encode_to_slice::hex::bench4_16384        ... bench:       6,711 ns/iter (+/- 461)

test format::const_hex::bench1_32              ... bench:          10 ns/iter (+/- 0)
test format::const_hex::bench2_256             ... bench:          40 ns/iter (+/- 0)
test format::const_hex::bench3_2048            ... bench:         176 ns/iter (+/- 1)
test format::const_hex::bench4_16384           ... bench:       1,399 ns/iter (+/- 12)
test format::std::bench1_32                    ... bench:         503 ns/iter (+/- 4)
test format::std::bench2_256                   ... bench:       3,915 ns/iter (+/- 29)
test format::std::bench3_2048                  ... bench:      30,012 ns/iter (+/- 396)
test format::std::bench4_16384                 ... bench:     242,046 ns/iter (+/- 1,357)
```

## Acknowledgements

- [`hex`] for the initial encoding/decoding implementations
- [`faster-hex`] for the `x86`/`x86-64` check and decode implementations
- [dtolnay]/[itoa] for the initial crate/library API layout

[`hex`]: https://crates.io/crates/hex
[`faster-hex`]: https://crates.io/crates/faster-hex
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
