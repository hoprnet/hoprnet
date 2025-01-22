// Copyright (C) 2019-2023 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#![deny(missing_docs)]

//! A crate providing a replacement #[[macro@test]] attribute that
//! initializes logging and/or tracing infrastructure before running
//! tests.

/// A procedural macro for the `test` attribute.
///
/// The attribute can be used to define a test that has the `env_logger`
/// and/or `tracing` crates initialized (depending on the features used).
///
/// # Example
///
/// Specify the attribute on a per-test basis:
/// ```rust
/// # // doctests seemingly run in a slightly different environment where
/// # // `super`, which is what our macro makes use of, is not available.
/// # // By having a fake module here we work around that problem.
/// # #[cfg(feature = "log")]
/// # mod fordoctest {
/// # use logging::info;
/// # // Note that no test would actually run, regardless of `no_run`,
/// # // because we do not invoke the function.
/// #[test_log::test]
/// fn it_works() {
///   info!("Checking whether it still works...");
///   assert_eq!(2 + 2, 4);
///   info!("Looks good!");
/// }
/// # }
/// ```
///
/// It can be very convenient to convert over all tests by overriding
/// the `#[test]` attribute on a per-module basis:
/// ```rust,no_run
/// # mod fordoctest {
/// use test_log::test;
///
/// #[test]
/// fn it_still_works() {
///   // ...
/// }
/// # }
/// ```
///
/// You can also wrap another attribute. For example, suppose you use
/// [`#[tokio::test]`](https://docs.rs/tokio/1.4.0/tokio/attr.test.html)
/// to run async tests:
/// ```
/// # mod fordoctest {
/// use test_log::test;
///
/// #[test(tokio::test)]
/// async fn it_still_works() {
///   // ...
/// }
/// # }
/// ```
pub use test_log_macros::test;

#[cfg(feature = "trace")]
#[doc(hidden)]
pub use tracing_subscriber;

#[cfg(feature = "log")]
#[doc(hidden)]
pub use env_logger;
