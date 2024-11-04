# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.9.1 (2023-08-17)
### Fixed
- `Scalar` conversion to `SecretKey` ([#137])

[#137]: https://github.com/RustCrypto/nacl-compat/pull/137

## 0.9.0 (2023-07-22)

COMPATIBILITY NOTE: previous versions of this crate provided a `ChaChaBox`
construction which is incompatible with the one in libsodium. This release
makes breaking changes to that construction which provide compatibility with
the libsodium implementation. No backwards compatibility is provided for the
construction in previous releases of this crate. We apologize for the
inconvenience.

### Added
- `SEALBYTES` constant ([#81])
- `serde` impl for `SecretKey` ([#101])
- Generic implementation of `CryptoBox` ([#113])

### Changed
- Use `crypto_secretbox` crate for symmetric cryptography ([#82], [#109])
- Replaced `x25519-dalek` with `curve25519-dalek` v4; MSRV 1.60 ([#88], [#130])
- Use `MontgomeryPoint` as `PublicKey`'s inner type ([#91])
- Make `seal` and `unseal` inherent methods ([#118])
  - `seal` => `PublicKey::seal`
  - `seal_open` => `SecretKey::unseal`
- Consistent constructors for `PublicKey`/`SecretKey` ([#119])

### Removed
- `rand_core` reexport ([#116])

[#81]: https://github.com/RustCrypto/nacl-compat/pull/81
[#82]: https://github.com/RustCrypto/nacl-compat/pull/82
[#88]: https://github.com/RustCrypto/nacl-compat/pull/88
[#91]: https://github.com/RustCrypto/nacl-compat/pull/91
[#101]: https://github.com/RustCrypto/nacl-compat/pull/101
[#109]: https://github.com/RustCrypto/nacl-compat/pull/109
[#113]: https://github.com/RustCrypto/nacl-compat/pull/113
[#116]: https://github.com/RustCrypto/nacl-compat/pull/116
[#118]: https://github.com/RustCrypto/nacl-compat/pull/118
[#119]: https://github.com/RustCrypto/nacl-compat/pull/119
[#130]: https://github.com/RustCrypto/nacl-compat/pull/130

## 0.8.2 (2022-10-29)
### Added
- `seal` and `seal_open` functions ([#66])

[#66]: https://github.com/RustCrypto/nacl-compat/pull/66

## 0.8.1 (2022-08-15)
### Changed
- Revert "select `x25519-dalek` backend automatically" ([#63])

[#63]: https://github.com/RustCrypto/nacl-compat/pull/63

## 0.8.0 (2022-08-10) [YANKED]

This release was yanked due to issues with automatically selecting the
`x25519-dalek` backend. See [#55].

### Changed
- Unpin `zeroize` version ([#41])
- Upgrade to Rust 2021 edition; MSRV 1.56 ([#42])
- Bump `chacha20` to v0.9 ([#50])
- Bump `chacha20poly1305` to v0.10 ([#50])
- Use `serdect` crate ([#51])
- Select `x25519-dalek` backend automatically ([#55])

### Removed
- `Box` type alias: use `SalsaBox` instead ([#53])
- `u32_backend` and `u64_backend` features: now selected automatically ([#55])

[#41]: https://github.com/RustCrypto/nacl-compat/pull/41
[#42]: https://github.com/RustCrypto/nacl-compat/pull/42
[#50]: https://github.com/RustCrypto/nacl-compat/pull/50
[#51]: https://github.com/RustCrypto/nacl-compat/pull/51
[#53]: https://github.com/RustCrypto/nacl-compat/pull/53
[#55]: https://github.com/RustCrypto/nacl-compat/pull/55

## 0.7.2 (2022-03-21)
### Fixed
- Building on docs.rs ([#30])

[#30]: https://github.com/RustCrypto/nacl-compat/pull/30

## 0.7.1 (2022-01-12)
### Added
- `SecretKey::as_bytes` ([#24])
- Optional `serde` support ([#27])

### Changed
- Deprecate `SecretKey::to_bytes` ([#24])

[#24]: https://github.com/RustCrypto/nacl-compat/pull/24
[#27]: https://github.com/RustCrypto/nacl-compat/pull/27

## 0.7.0 (2021-08-30)
### Changed
- MSRV 1.51 ([#8])
- Bump `chacha20poly1305` to v0.9 ([#8])
- Bump `xsalsa20poly1305` to v0.8 ([#8])

[#8]: https://github.com/RustCrypto/nacl-compat/pull/8

## 0.6.1 (2021-07-20)
### Changed
- Pin `zeroize` dependency to v1.3

## 0.6.0 (2021-04-29)
### Changed
- Bump `chacha20poly1305` crate dependency to v0.8
- Bump `xsalsa20poly1305` crate dependency to v0.7
- Bump `rand_core` crate dependency to v0.6

### SECURITY
- Fix XChaCha20Poly1305 key derivation

## 0.5.0 (2020-10-16)
### Added
- `ChaChaBox`

### Changed
- Replace `block-cipher`/`stream-cipher` with `cipher` crate
- Bump `xsalsa20poly1305` dependency to v0.6

## 0.4.0 (2020-09-17)
### Added
- Optional `std` feature; disabled by default

### Changed
- Upgrade `xsalsa20poly1305` to v0.5

## 0.3.0 (2020-08-18)
### Changed
- Bump `x25519-dalek` dependency to 1.0

## 0.2.0 (2020-06-06)
### Changed
- Bump `aead` crate dependency to v0.3; MSRV 1.41+
- Bump `xsalsa20poly1305` dependency to v0.4

## 0.1.0 (2020-02-25)
- Initial release
