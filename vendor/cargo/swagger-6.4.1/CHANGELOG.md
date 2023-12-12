# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed

### Added

### Fixed

## [6.4.1] - 2023-09-05
### Fixed
- Fixed documentation build breakage on stable

## [6.4.0] - 2023-09-04
### Added
- Support `serde_valid` 0.16.

## [6.3.0] - 2023-06-30
### Added
- Support validation via `serde_valid` for ByteArray and Nullable

### Fixed
- Fix clippy warnings for Rust 1.70

## [6.2.1] - 2022-07-13
### Fixed
- Fixed build when using `--features uds`

## [6.2.0] - 2022-06-25
### Added
- Use a trait for connection on CompositeService to allow users to define
  their own supported connection types

## [6.1.1] - 2022-02-01
### Fixed
- Remove private, unused dependency on `chrono`

## [6.1.0] - 2021-06-17
### Added
- Add support for Unix Domain Socket services as part of a Composite Service

## [6.0.0] - 2021-03-18
### Breaking Changes
- Remove implicit conversions for ApiError from `&str`, `String` and `serde_json::Error`
- Support Hyper 0.14 and match features
  - Add new feature `tcp` to signal HTTP support
  - Add new feature `tls` to signal HTTP(S) support
  - Add new feature `client` to signal Client support
  - Add new feature `server` to signal Server support
- Upgrade base64 to 0.13
- Update `mime_multipart` requirement from 0.5 to 0.6
- Upgrade bytes to 1.0
- Upgrade tokio to 1.0

## [5.1.0] - 2021-03-04
### Added
- Support conversion for AnyOf and OneOf
- Expose CompositeMakeServiceEntry

## [5.0.2] - 2021-01-13
### Fixed
- Fix off by one error declaring OneOf and AnyOf with more than 10 arguments

## [5.0.1] - 2021-01-12
### Fixed
- Fix interop with serde 1.0.119

## [5.0.0] - 2020-10-06

## Breaking Changes
- Support Hyper 0.13
- Remove headers module

### Added
- Add Clone implementation for AllowAllAuthenticator
- Add support functions for multipart/related
- Add ToString/FromStr implementations for ByteArray
- Add support for OneOf/AnyOf

### Fixed
- Fix Composite support

## [4.0.2] - 2020-03-12
### Changed
- Fixed panic where X-Span-ID headers contained non-ASCII characters.

## [4.0.1] - 2020-03-06
### Changed
- Use hyper-tls 3 on Mac/Windows/iOS which is compatible with Hyper 0.12, instead of version 4, which isn't.

## [4.0.0] - 2020-02-20
### Changed
- Use hyper-openssl on Linux, instead of hyper-tls
- Use a builder pattern to created client connectors
- Allow HTTPS connectors to be built which don't pin the server CA certificate
- Allow HTTPS to work on Mac/Windows/iOS
- Enforce that HTTPS is used if we are using a HTTPS connector.
- Return Results, rather than unwrapping errors on connector creation
- openssl 0.10

## [3.2.0] - 2020-02-12
### Added
- Added macro `request_parser_joiner` to easily create a composite RequestParser that will match a request against multiple RequestParsersa
- Add DropContextService as a client middleware too
### Changed
- Make context macros self-contained
- Update swagger-rs to Rust 2018 Edition

## [3.1.0] - 2019-05-07
### Added
- Add `swagger::client::MakeService` trait

## [3.0.0] - 2019-03-08
### Changed
- Hyper 0.12 support.

  This creates large scale changes with corresponding renaming (e.g. `NewService` -> `MakeService`) and other fall out.

  Hyper Services don't have Request as a type parameters -  instead the body of the request / response are type parameters.

  As such context for requests, such as authorization data is kept in a `ContextualPayload` struct, instead of a tuple.

### Removed
- `AddContext` structs which we were previously deprecated are now removed.

## [2.0.2] - 2018-12-13
### Added
- Allow ContextWrapper to be cloned.

## [2.0.1] - 2018-11-12
### Changed
* Make compatible with clippy on stable (1.30.1)

## [2.0.0] - 2018-09-28

### Changed
- Added the `AddContextNewService` and `AddContextService` structs, and deprecated the old `AddContext` struct. One or other of the new structs should be a drop-in replacement for the `AddContext`, depending on whether it was being used as a `NewService` or `Service`.
- modified the `new_context_type` macro to only implement `Push`, `Pop` and `Has` for types explicitly passed to the macro. This is a breaking change, which should only require minor changes such as adding type annotations if the macro was used as recommended in the docs.

## [1.0.2] - 2018-07-23
### Added
- Added (non-HTTPS) support for Windows/MacOS/iOS

## [1.0.1] - 2018-05-24
### Added
- `SwaggerService` trait used by swagger-codegen middlewares.

## [1.0.0] - 2018-04-30
No changes. We now think we've got enough to declare this crate stable.

## [0.12.1] - 2018-04-27
### Added
- `RequestParser` trait for retrieving Swagger related info in middlewares.

### Changed
- Fixed `DropContext` to remove trait bounds on the type of context it can drop

## [0.12.0] - 2018-04-26
### Added
- `DropContext` to pass a raw (context-less) `hyper::Request` to a service.

## [0.11.0] - 2018-04-11
### Added
- `Has<T>`, `Pop<T>` and `Push<T>` traits for specifying requirements on context types in hyper services, and providing methods for manipulating them
- `new_context_type!` macro for defining structs that can be used to build concrete context types that implement `Has`, `Pop` and `Push`
- `make_context!` and `make_context_ty!` for conveniently creating contexts at value and type level

### Removed
- Old `Context` struct

### Changed
- Renamed `NoAuthentication` to `MiddlewareWrapper` and moved it to its own module.

## [0.10.0] - 2018-03-16
### Added
- Structs for combining multiple hyper services

## [0.9.0] - 2018-01-25
### Added
- Connector functions for instantiating easy-mode clients
- The ability to pass in a `slog::Logger` with Context

## [0.8.1] - 2017-12-20
### Changed
- Fix build error and clippy warning.

## [0.8.0] - 2017-12-15
### Added
- Asynchronous HTTP client/server support

### Removed
- Synchronous HTTP client/server support - if you're still using synchronous swagger-codegen, stay at 0.7.0

### Changed
- `AllowAllMiddleware` (an Iron middleware) has been replaced by `AllowAllAuthenticator` (a Hyper Service wrapper)

## [0.7.0] - 2017-10-02
### Added
- `ContextWrapper` - wraps an `Api` with a `Context`

## [0.6.0] - 2017-09-25
### Changed
- Authorization struct now has new field `issuer`.

## [0.5.0] - 2017-09-18
- Start of changelog.

[Unreleased]: https://github.com/Metaswitch/swagger-rs/compare/6.4.1...HEAD
[6.4.1]: https://github.com/Metaswitch/swagger-rs/compare/6.4.0...6.4.1
[6.4.0]: https://github.com/Metaswitch/swagger-rs/compare/6.3.0...6.4.0
[6.3.0]: https://github.com/Metaswitch/swagger-rs/compare/6.2.1...6.3.0
[6.2.1]: https://github.com/Metaswitch/swagger-rs/compare/6.2.0...6.2.1
[6.2.0]: https://github.com/Metaswitch/swagger-rs/compare/6.1.1...6.2.0
[6.1.1]: https://github.com/Metaswitch/swagger-rs/compare/6.1.0...6.1.1
[6.1.0]: https://github.com/Metaswitch/swagger-rs/compare/6.0.0...6.1.0
[6.0.0]: https://github.com/Metaswitch/swagger-rs/compare/5.1.0...6.0.0
[5.1.0]: https://github.com/Metaswitch/swagger-rs/compare/5.0.2...5.1.0
[5.0.2]: https://github.com/Metaswitch/swagger-rs/compare/5.0.1...5.0.2
[5.0.1]: https://github.com/Metaswitch/swagger-rs/compare/5.0.0...5.0.1
[5.0.0]: https://github.com/Metaswitch/swagger-rs/compare/4.0.2...5.0.0
[4.0.2]: https://github.com/Metaswitch/swagger-rs/compare/4.0.1...4.0.2
[4.0.1]: https://github.com/Metaswitch/swagger-rs/compare/4.0.0...4.0.1
[4.0.0]: https://github.com/Metaswitch/swagger-rs/compare/3.2.0...4.0.0
[3.2.0]: https://github.com/Metaswitch/swagger-rs/compare/3.1.0...3.2.0
[3.1.0]: https://github.com/Metaswitch/swagger-rs/compare/3.0.0...3.1.0
[3.0.0]: https://github.com/Metaswitch/swagger-rs/compare/2.0.2...3.0.0
[2.0.2]: https://github.com/Metaswitch/swagger-rs/compare/2.0.1...2.0.2
[2.0.1]: https://github.com/Metaswitch/swagger-rs/compare/2.0.0...2.0.1
[2.0.0]: https://github.com/Metaswitch/swagger-rs/compare/1.0.2...2.0.0
[1.0.2]: https://github.com/Metaswitch/swagger-rs/compare/1.0.1...1.0.2
[1.0.1]: https://github.com/Metaswitch/swagger-rs/compare/1.0.0...1.0.1
[1.0.0]: https://github.com/Metaswitch/swagger-rs/compare/0.12.1...1.0.0
[0.12.1]: https://github.com/Metaswitch/swagger-rs/compare/0.12.0...0.12.1
[0.12.0]: https://github.com/Metaswitch/swagger-rs/compare/0.11.0...0.12.0
[0.11.0]: https://github.com/Metaswitch/swagger-rs/compare/0.10.0...0.11.0
[0.10.0]: https://github.com/Metaswitch/swagger-rs/compare/0.9.0...0.10.0
[0.9.0]: https://github.com/Metaswitch/swagger-rs/compare/0.8.1...0.9.0
[0.8.1]: https://github.com/Metaswitch/swagger-rs/compare/0.8.0...0.8.1
[0.8.0]: https://github.com/Metaswitch/swagger-rs/compare/0.7.0...0.8.0
[0.7.0]: https://github.com/Metaswitch/swagger-rs/compare/0.6.0...0.7.0
[0.6.0]: https://github.com/Metaswitch/swagger-rs/compare/0.5.0...0.6.0
[0.5.0]: https://github.com/Metaswitch/swagger-rs/compare/0.4.0...0.5.0
