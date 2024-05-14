# Changelog

This is the changelog for [parameterized](https://github.com/foresterre/parameterized). Parameterized is a Rust library
which provides an attribute macro, to be used for parameterized testing.

If you found an issue, have a suggestion or want to provide feedback or insights, please feel free to open an issue on
the [issue tracker](https://github.com/foresterre/parameterized/issues), or open a topic in
the [discussions section](https://github.com/foresterre/parameterized/discussions).

## [2.0.0] - 2024-03-05

### Added

* Added support for custom test macros (e.g., `#[tokio::test]` instead of the default `#[test]`) by specifying the 
  `#[parameterized_macro(...)]` attribute.
* `#[parameterized]` macro now parses and regenerates optional `const`, `async`, `unsafe` and return type items for `fn`
  definitions.

### Changed

* MSRV is now `1.63`

[2.0.0]: https://github.com/foresterre/parameterized/releases/tag/v2.0.0

## [1.1.0] - 2023-10-23

### Added

* Add option to use return type in tests

[1.1.0]: https://github.com/foresterre/parameterized/releases/tag/v1.1.0

## [1.0.1] - 2022-11-09

### Changed

* Updated MSRV to Rust `1.38`
* Updated parameterized-macro to `1.0.1`

[1.0.1]: https://github.com/foresterre/parameterized/releases/tag/v1.0.1

## [1.0.0] - 2022-05-02

### Changed

* Updated MSRV to Rust `1.36`

[1.0.0]: https://github.com/foresterre/parameterized/releases/tag/v1.0.0

## [0.3.1] - 2021-01-07

### Fixed

* Fix issue where parameterized-macro used the public API of the syn::export module which the docs described as
  a non-public implementation details module, not covered by semver

[0.3.1]: https://github.com/foresterre/parameterized/releases/tag/v0.3.1

## [0.3.0] - 2020-12-30

This release consists of internal changes. There are no changes for users.
An alternative syntax which was added after 0.2.0, but was never released,
has been moved into its own project named [Yare](https://github.com/foresterre/yare).

[0.3.0]: https://github.com/foresterre/parameterized/releases/tag/v0.3.0

## [0.2.0] - 2020-06-23

This release consists of internal changes. There are no changes for users.

[0.2.0]: https://github.com/foresterre/parameterized/releases/tag/v0.2.0

## [0.1.0] - 2019-11-09

First version published on crates.io

[0.1.0]: https://github.com/foresterre/parameterized/releases/tag/v0.1.0
