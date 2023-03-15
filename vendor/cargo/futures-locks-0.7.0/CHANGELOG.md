# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## [0.7.0] - 2021-11-18
### Added
### Changed
- Updated to Tokio 1.0.  Crates needing Tokio 0.2 must stay on the 0.6 branch.
  ([#42](https://github.com/asomers/futures-locks/pull/42))

- `Mutex::try_lock`, `RwLock::try_read`, and `RwLock::try_write`, now return a
  dedicated error type instead of `()`, and it implements `std::error::Error`.
  ([#41](https://github.com/asomers/futures-locks/pull/41))

### Fixed

- Fixed a soundness issue with `RwLock`: it shouldn't implement `Sync` unless
  its inner type also does.
  ([#45](https://github.com/asomers/futures-locks/pull/45))

### Removed

## [0.6.0] - 2020-09-07
### Added
### Changed
- Updated to `std::future`.  `futures-locks` no longer works with
  `futures-0.1`.  For `futures-0.1`-based applications, continue to use the 0.5
  branch.  Most methods have similar interfaces.  However, the `IntoFuture`
  trait no longer exists in the `std::future` world.  And `Future<T>` doesn't
  implement `From<T>` (though I don't know why it couldn't).  So `Mutex::with`,
  `RwLock::with_read`, and `RwLock::with_write` now take closures that return
  `impl Future`, instead of `impl IntoFuture`.  Closure arguments that used to
  return something like `Result<T, E>` should now return
  `futures::future::ready<Result<T, E>>` instead.

### Fixed
### Removed

## [0.5.0] - 2019-11-03
### Added
- Derived `Default` for `Mutex` and `RwLock`
  ([#22](https://github.com/asomers/futures-locks/pull/22))

### Changed
- Minimum compiler version has increased to 1.32.0
  ([#28](https://github.com/asomers/futures-locks/pull/28))

### Fixed
- Fixed panics after an unready Future gets dropped
  ([#24](https://github.com/asomers/futures-locks/pull/24))
 
## [0.4.0] - 2019-08-24
### Added
- Added `Mutex::ptr_eq()`
  ([#20](https://github.com/asomers/futures-locks/pull/20))
- Added `MutexWeak`
  ([#17](https://github.com/asomers/futures-locks/pull/17)) 

## [0.3.2] - 2019-01-30
### Changed
- Better documentation
  ([#15](https://github.com/asomers/futures-locks/pull/15))

## [0.3.1] - 2018-12-26
### Changed
- Minimum compiler version has increased to 1.31.0
  ([#10](https://github.com/asomers/futures-locks/pull/10))
- `futures-locks` only depends on tokio crates `tokio-current-thread` and
  `tokio-executor` when built with the `tokio` feature.
  ([#10](https://github.com/asomers/futures-locks/pull/10))

## [0.3.0] - 2018-06-28
### Added
- Added `Mutex::with_local`, `RwLock::with_read_local`, and
  `RwLock::with_write_local` for non-`Send` Futures.
  ([#7](https://github.com/asomers/futures-locks/pull/7))

### Changed
- `Mutex::with`, `RwLock::with_read`, and `RwLock::with_write` now require
  their Futures to be `Send`.  They also can return errors.
  ([#7](https://github.com/asomers/futures-locks/pull/7))
- Methods enabled with the tokio feature now return a Future type equivalent
  to the one returned by the provided closure.
  ([#6](https://github.com/asomers/futures-locks/pull/6))

### Fixed
- `Mutex::with`, `RwLock::with_read`, and `RwLock::with_write` now work with
  all Tokio Runtimes.
  ([#7](https://github.com/asomers/futures-locks/pull/7))

## [0.2.1] - 2018-06-18
### Changed
- Tokio support is now enabled by default.

## [0.2.0] - 2018-06-18
### Added
- Added methods for running critical code in its own task.  These methods
  require Tokio.
  ([#3](https://github.com/asomers/futures-locks/issues/3))
- Implemented `Future` for all future types, even when the wrapped type is
  unsized.

## [0.1.1] - 2018-06-07
### Added

### Changed
- Futures should do nothing until polled.
  ([#2](https://github.com/asomers/futures-locks/issues/2))
- Fixed potential deadlocks when dropping Futures without first polling them.
  ([#1](https://github.com/asomers/futures-locks/issues/1))

### Fixed

### Removed
