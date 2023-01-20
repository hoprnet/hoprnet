# Pharos Changelog

## [Unreleased]

  [Unreleased]: https://github.com/najamelan/pharos/compare/0.5.2...dev


## [0.5.2] - 2021-06-10

  [0.5.2]: https://github.com/najamelan/pharos/compare/0.5.1...0.5.2

### Fixed

  - Remove external_doc for rustdoc 1.54. Thanks to @rajivshah3


## 0.5.1 - 2021-02-18

- Add `ObservableLocal` for observable types that are `!Send`.

## 0.5.0 - 2021-02-17

- **BREAKING CHANGE**: `Observable::observe` is now an async function. This was needed to make it possible to send
  events to a pharos object from different async tasks. So far notifying was async, but observing was not. However
  in order to be able to use a mutex, we need both operations to be one or the other. On wasm, one cannot block the
  thread hence the choice for an async mutex, but in order to lock that we have to be in async context.
  A new helper type `SharedPharos` has been introduced to conveniently use pharos from a shared reference.
- **BREAKING CHANGE**: rename `pharos::Error` to `PharErr`. I want to move away from types that are just called `Error`.
  This allows conveniently exporting the error type at crate level.
- no longer depend on futures-channel appart from the main futures lib. It's annoying if a dependant crate
  want's to patch futures in Cargo.toml.
- move to github actions after travis becomes a paid service.

## 0.4.2 - 2019-11-13

- drop dependency on log.

## 0.4.1 - 2019-11-13

- update dependencies to futures 0.3.1.
- CI testing on stable rust.

## 0.4.0 - 2019-09-28

- **BREAKING CHANGE**: The notify function had a sub optimal implemetation and did not allow notifying observers
  from within `poll_*` functions. It has been replaced with an implementation of Sink on Pharos.
- got rid of dependency on pin_project.
- as Error::kind returns a reference to the error kind, you can now compare `ErrorKind::SomeVariant == err.kind()` without having to write: `&ErrorKind::SomeVariant == err.kind()`.
- updated to futures-preview 0.3.0-alpha.19

## 0.3.2 - 2019-09-23

- check spelling

## 0.3.1 - 2019-09-23

- **BREAKING CHANGE**: Last minute change of heart. I removed two API methods whose only "merit" was
to hide a Box::new from the user.
- fix docs.rs showing readme

## 0.3.0 - 2019-09-23 - yanked

**BREAKING CHANGE**: This is an almost complete rewrite with a much improved API, documentation, ...

- Only have one Observable trait that takes options rather than UboundedObservable.
- Allow filtering events with a predicate.
- Many small improvements.

Please have a look at the readme and the API docs for more.

## 0.2.2 - 2019-08-26

- update to futures 0.3.0-alpha.18
- remove async_await feature (from 1.39 this crate should compile on stable)

## 0.2.1 - 2019-08-02

- remove `await_macro` feature use and convert to new await syntax.
- implement `Default` for `Pharos`.


## 0.2.0 - 2019-07-18

- **BREAKING CHANGE**:  Update dependencies. The Futures library has some changes, which introduce a breaking change. Bounded channels are no longer queue_size + 1. They now are queue_size. This means that `Observable::observe( queue_size: usize )` will now yield a `futures::channel::Receiver` of the exact `queue_size`.
