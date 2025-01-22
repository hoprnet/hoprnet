# const-hex

[![github](https://img.shields.io/badge/github-danipopes/const--hex-8da0cb?style=for-the-badge&labelColor=555555&logo=github)](https://github.com/danipopes/const-hex)
[![crates.io](https://img.shields.io/crates/v/const-hex.svg?style=for-the-badge&color=fc8d62&logo=rust)](https://crates.io/crates/const-hex)
[![docs.rs](https://img.shields.io/badge/docs.rs-const--hex-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs)](https://docs.rs/const-hex)
[![build status](https://img.shields.io/github/actions/workflow/status/danipopes/const-hex/ci.yml?branch=master&style=for-the-badge)](https://github.com/danipopes/const-hex/actions?query=branch%3Amaster)

This crate provides a fast conversion of byte arrays to hexadecimal strings,
both at compile time, and at run time.

It aims to be a drop-in replacement for the [`hex`] crate, as well as extending
the API with [const-eval], a [const-generics formatting buffer][buffer], similar
to [`itoa`]'s, and more.

_Version requirement: rustc 1.64+_

[const-eval]: https://docs.rs/const-hex/latest/const_hex/fn.const_encode.html
[buffer]: https://docs.rs/const-hex/latest/const_hex/struct.Buffer.html
[`itoa`]: https://docs.rs/itoa/latest/itoa/struct.Buffer.html

## Performance

This crate's performance is comparable with [`faster-hex`], but the latter only
provides specialized implementations for `x86`/`x86-64`.

This crate is 10 to 50 times faster than [`hex`] in encoding and decoding, and
100+ times faster than `libstd` in formatting.

The following benchmarks were ran on an AMD Ryzen 9 7950X, compiled with
`1.80.0-nightly (bdbbb6c6a 2024-05-26)` on `x86_64-unknown-linux-gnu`.

You can run these benchmarks with `cargo bench --features std` on a nightly
compiler.

```log
test check::const_hex::bench1_32b             ... bench:           9.79 ns/iter (+/- 2.22)
test check::const_hex::bench2_256b            ... bench:          20.22 ns/iter (+/- 6.41)
test check::const_hex::bench3_2k              ... bench:         124.57 ns/iter (+/- 9.68)
test check::const_hex::bench4_16k             ... bench:         927.53 ns/iter (+/- 81.94)
test check::const_hex::bench5_128k            ... bench:       7,300.16 ns/iter (+/- 153.57)
test check::const_hex::bench6_1m              ... bench:      59,886.62 ns/iter (+/- 1,312.15)
test check::faster_hex::bench1_32b            ... bench:           2.93 ns/iter (+/- 0.20)
test check::faster_hex::bench2_256b           ... bench:          15.54 ns/iter (+/- 0.77)
test check::faster_hex::bench3_2k             ... bench:         121.17 ns/iter (+/- 0.70)
test check::faster_hex::bench4_16k            ... bench:         945.17 ns/iter (+/- 12.48)
test check::faster_hex::bench5_128k           ... bench:       7,632.07 ns/iter (+/- 172.68)
test check::faster_hex::bench6_1m             ... bench:      61,427.03 ns/iter (+/- 732.21)
test check::naive::bench1_32b                 ... bench:          26.15 ns/iter (+/- 1.04)
test check::naive::bench2_256b                ... bench:         225.69 ns/iter (+/- 9.02)
test check::naive::bench3_2k                  ... bench:       1,952.05 ns/iter (+/- 20.46)
test check::naive::bench4_16k                 ... bench:      20,329.63 ns/iter (+/- 455.31)
test check::naive::bench5_128k                ... bench:     503,460.10 ns/iter (+/- 8,930.00)
test check::naive::bench6_1m                  ... bench:   4,252,331.00 ns/iter (+/- 191,519.79)

test decode::const_hex::bench1_32b            ... bench:          19.49 ns/iter (+/- 0.82)
test decode::const_hex::bench2_256b           ... bench:          41.47 ns/iter (+/- 1.45)
test decode::const_hex::bench3_2k             ... bench:         236.69 ns/iter (+/- 2.75)
test decode::const_hex::bench4_16k            ... bench:       1,682.25 ns/iter (+/- 18.07)
test decode::const_hex::bench5_128k           ... bench:      13,096.94 ns/iter (+/- 138.05)
test decode::const_hex::bench6_1m             ... bench:     105,360.67 ns/iter (+/- 2,771.85)
test decode::faster_hex::bench1_32b           ... bench:          16.49 ns/iter (+/- 0.12)
test decode::faster_hex::bench2_256b          ... bench:          54.92 ns/iter (+/- 1.74)
test decode::faster_hex::bench3_2k            ... bench:         246.11 ns/iter (+/- 3.25)
test decode::faster_hex::bench4_16k           ... bench:       1,839.88 ns/iter (+/- 22.36)
test decode::faster_hex::bench5_128k          ... bench:      14,376.52 ns/iter (+/- 203.97)
test decode::faster_hex::bench6_1m            ... bench:     116,345.50 ns/iter (+/- 1,377.06)
test decode::hex::bench1_32b                  ... bench:         101.38 ns/iter (+/- 3.41)
test decode::hex::bench2_256b                 ... bench:         655.85 ns/iter (+/- 17.69)
test decode::hex::bench3_2k                   ... bench:       4,830.90 ns/iter (+/- 73.58)
test decode::hex::bench4_16k                  ... bench:      37,976.46 ns/iter (+/- 443.65)
test decode::hex::bench5_128k                 ... bench:     734,983.30 ns/iter (+/- 4,297.49)
test decode::hex::bench6_1m                   ... bench:   5,901,860.30 ns/iter (+/- 22,806.40)

test decode_to_slice::const_hex::bench1_32b   ... bench:          11.03 ns/iter (+/- 0.50)
test decode_to_slice::const_hex::bench2_256b  ... bench:          28.93 ns/iter (+/- 1.38)
test decode_to_slice::const_hex::bench3_2k    ... bench:         210.49 ns/iter (+/- 5.65)
test decode_to_slice::const_hex::bench4_16k   ... bench:       1,670.55 ns/iter (+/- 17.87)
test decode_to_slice::const_hex::bench5_128k  ... bench:      13,094.65 ns/iter (+/- 115.89)
test decode_to_slice::const_hex::bench6_1m    ... bench:     105,036.12 ns/iter (+/- 1,722.56)
test decode_to_slice::faster_hex::bench1_32b  ... bench:           6.26 ns/iter (+/- 0.10)
test decode_to_slice::faster_hex::bench2_256b ... bench:          28.73 ns/iter (+/- 0.55)
test decode_to_slice::faster_hex::bench3_2k   ... bench:         213.70 ns/iter (+/- 2.83)
test decode_to_slice::faster_hex::bench4_16k  ... bench:       1,718.21 ns/iter (+/- 23.97)
test decode_to_slice::faster_hex::bench5_128k ... bench:      13,530.96 ns/iter (+/- 97.41)
test decode_to_slice::faster_hex::bench6_1m   ... bench:     107,708.79 ns/iter (+/- 1,425.58)
test decode_to_slice::hex::bench1_32b         ... bench:          39.07 ns/iter (+/- 1.85)
test decode_to_slice::hex::bench2_256b        ... bench:         311.73 ns/iter (+/- 10.92)
test decode_to_slice::hex::bench3_2k          ... bench:       2,515.69 ns/iter (+/- 63.09)
test decode_to_slice::hex::bench4_16k         ... bench:      20,899.92 ns/iter (+/- 442.95)
test decode_to_slice::hex::bench5_128k        ... bench:     634,859.00 ns/iter (+/- 6,609.09)
test decode_to_slice::hex::bench6_1m          ... bench:   5,338,354.20 ns/iter (+/- 51,493.30)

test encode::const_hex::bench1_32b            ... bench:           6.92 ns/iter (+/- 0.11)
test encode::const_hex::bench2_256b           ... bench:          11.41 ns/iter (+/- 0.14)
test encode::const_hex::bench3_2k             ... bench:          73.59 ns/iter (+/- 1.52)
test encode::const_hex::bench4_16k            ... bench:         461.00 ns/iter (+/- 5.17)
test encode::const_hex::bench5_128k           ... bench:       3,527.28 ns/iter (+/- 46.86)
test encode::const_hex::bench6_1m             ... bench:      29,402.78 ns/iter (+/- 1,032.91)
test encode::faster_hex::bench1_32b           ... bench:          17.30 ns/iter (+/- 0.30)
test encode::faster_hex::bench2_256b          ... bench:          39.07 ns/iter (+/- 0.73)
test encode::faster_hex::bench3_2k            ... bench:         102.93 ns/iter (+/- 1.92)
test encode::faster_hex::bench4_16k           ... bench:         651.65 ns/iter (+/- 1.55)
test encode::faster_hex::bench5_128k          ... bench:       5,074.50 ns/iter (+/- 22.97)
test encode::faster_hex::bench6_1m            ... bench:      46,227.11 ns/iter (+/- 945.52)
test encode::hex::bench1_32b                  ... bench:         100.46 ns/iter (+/- 0.93)
test encode::hex::bench2_256b                 ... bench:         717.75 ns/iter (+/- 5.91)
test encode::hex::bench3_2k                   ... bench:       5,660.67 ns/iter (+/- 246.73)
test encode::hex::bench4_16k                  ... bench:      44,981.79 ns/iter (+/- 340.51)
test encode::hex::bench5_128k                 ... bench:     359,401.72 ns/iter (+/- 1,689.97)
test encode::hex::bench6_1m                   ... bench:   2,966,947.20 ns/iter (+/- 165,738.51)

test encode_to_slice::const_hex::bench1_32b   ... bench:           1.55 ns/iter (+/- 0.04)
test encode_to_slice::const_hex::bench2_256b  ... bench:           6.73 ns/iter (+/- 0.08)
test encode_to_slice::const_hex::bench3_2k    ... bench:          54.23 ns/iter (+/- 2.06)
test encode_to_slice::const_hex::bench4_16k   ... bench:         471.90 ns/iter (+/- 21.13)
test encode_to_slice::const_hex::bench5_128k  ... bench:       3,730.44 ns/iter (+/- 113.80)
test encode_to_slice::const_hex::bench6_1m    ... bench:      29,247.93 ns/iter (+/- 926.38)
test encode_to_slice::faster_hex::bench1_32b  ... bench:           3.95 ns/iter (+/- 0.26)
test encode_to_slice::faster_hex::bench2_256b ... bench:           7.49 ns/iter (+/- 0.47)
test encode_to_slice::faster_hex::bench3_2k   ... bench:          48.56 ns/iter (+/- 1.95)
test encode_to_slice::faster_hex::bench4_16k  ... bench:         424.65 ns/iter (+/- 11.39)
test encode_to_slice::faster_hex::bench5_128k ... bench:       3,317.47 ns/iter (+/- 103.44)
test encode_to_slice::faster_hex::bench6_1m   ... bench:      26,079.18 ns/iter (+/- 889.52)
test encode_to_slice::hex::bench1_32b         ... bench:          11.99 ns/iter (+/- 0.30)
test encode_to_slice::hex::bench2_256b        ... bench:         119.07 ns/iter (+/- 2.85)
test encode_to_slice::hex::bench3_2k          ... bench:         999.68 ns/iter (+/- 26.35)
test encode_to_slice::hex::bench4_16k         ... bench:       8,049.82 ns/iter (+/- 105.51)
test encode_to_slice::hex::bench5_128k        ... bench:      65,186.25 ns/iter (+/- 758.98)
test encode_to_slice::hex::bench6_1m          ... bench:     511,447.00 ns/iter (+/- 4,866.41)

test format::const_hex::bench1_32b            ... bench:           9.84 ns/iter (+/- 0.21)
test format::const_hex::bench2_256b           ... bench:          17.90 ns/iter (+/- 0.55)
test format::const_hex::bench3_2k             ... bench:         119.47 ns/iter (+/- 3.87)
test format::const_hex::bench4_16k            ... bench:       1,161.94 ns/iter (+/- 20.11)
test format::const_hex::bench5_128k           ... bench:       9,580.23 ns/iter (+/- 188.10)
test format::const_hex::bench6_1m             ... bench:      84,316.47 ns/iter (+/- 1,407.10)
test format::std::bench1_32b                  ... bench:         371.37 ns/iter (+/- 3.63)
test format::std::bench2_256b                 ... bench:       2,987.01 ns/iter (+/- 41.26)
test format::std::bench3_2k                   ... bench:      23,989.24 ns/iter (+/- 373.84)
test format::std::bench4_16k                  ... bench:     192,881.92 ns/iter (+/- 9,266.64)
test format::std::bench5_128k                 ... bench:   1,554,062.80 ns/iter (+/- 10,998.30)
test format::std::bench6_1m                   ... bench:  12,378,548.00 ns/iter (+/- 247,626.30)
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
