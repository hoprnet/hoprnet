# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.4.5 (2020-12-26)
### Changed
- Use `u128` to impl `mulx` ([#111])

[#111]: https://github.com/RustCrypto/universal-hashes/pull/111

## 0.4.4 (2020-12-26)
### Added
- `Debug` impl using `opaque-debug` ([#105])
- `mulx` feature ([#107])

[#105]: https://github.com/RustCrypto/universal-hashes/pull/105
[#107]: https://github.com/RustCrypto/universal-hashes/pull/107

## 0.4.3 (2020-12-08)
### Added
- CLMUL detection ([#92])

[#92]: https://github.com/RustCrypto/universal-hashes/pull/92

## 0.4.2 (2020-11-25)
### Added
- `KEY_SIZE` constant ([#82])

### Changed
- Bump `cfg-if` from v0.1 to v1.0.0 ([#86])

[#86]: https://github.com/RustCrypto/universal-hashes/pull/86
[#82]: https://github.com/RustCrypto/universal-hashes/pull/82

## 0.4.1 (2020-09-26)
### Changed
- Performance improvements ([#75])

[#75]: https://github.com/RustCrypto/universal-hashes/pull/75

## 0.4.0 (2020-06-06)
### Changed
- Bump `universal-hash` dependency to v0.4; MSRV 1.41 ([#52], [#57])
- Rename `result` methods to to `finalize` ([#56])

[#57]: https://github.com/RustCrypto/universal-hashes/pull/57
[#56]: https://github.com/RustCrypto/universal-hashes/pull/56
[#52]: https://github.com/RustCrypto/universal-hashes/pull/52

## 0.3.3 (2019-12-21)
### Changed
- Match ideal assembly implementation on x86/x86_64 ([#43], [#44])

[#43]: https://github.com/RustCrypto/universal-hashes/pull/43
[#44]: https://github.com/RustCrypto/universal-hashes/pull/44

## 0.3.2 (2019-12-05)
### Added
- Constant-time 32-bit software implementation ([#39])

### Changed
- Use `cfg-if` crate to reduce duplication ([#40])

[#39]: https://github.com/RustCrypto/universal-hashes/pull/39
[#40]: https://github.com/RustCrypto/universal-hashes/pull/40

## 0.3.1 (2019-11-14)
### Changed
- Upgrade to `zeroize` 1.0 ([#33])

[#33]: https://github.com/RustCrypto/universal-hashes/pull/33

## 0.3.0 (2019-10-05)
### Removed
- Remove `pub` from `field` module ([#28])

[#28]: https://github.com/RustCrypto/universal-hashes/pull/28

## 0.2.0 (2019-10-04)
### Changed
- Upgrade to `universal-hash` crate v0.3 ([#22])

[#22]: https://github.com/RustCrypto/universal-hashes/pull/22

## 0.1.1 (2019-10-01)
### Changed
- Upgrade to `zeroize` v1.0.0-pre ([#19])

[#19]: https://github.com/RustCrypto/universal-hashes/pull/19

## 0.1.0 (2019-09-19)
### Added
- Constant time software implementation ([#7])

### Changed
- Update to Rust 2018 edition ([#3])
- Use `UniversalHash` trait ([#6])
- Removed generics/traits from `field::Element` API ([#12])

### Removed
- `insecure-soft` cargo feature ([#7])

[#3]: https://github.com/RustCrypto/universal-hashes/pull/3
[#6]: https://github.com/RustCrypto/universal-hashes/pull/6
[#7]: https://github.com/RustCrypto/universal-hashes/pull/7
[#12]: https://github.com/RustCrypto/universal-hashes/pull/12

## 0.0.1 (2019-08-26)

- Initial release
