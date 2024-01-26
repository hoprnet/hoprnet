//! Provides a simple spinlock based on the abstractions provided by the [`lock_api`] crate.
//!
//! [`lock_api`]: https://docs.rs/lock_api/
//!
//! ## Usage Example
//!
//! ```rust
//! use spinning_top::Spinlock;
//!
//! fn main() {
//!     let data = String::from("Hello");
//!     // Wrap some data in a spinlock
//!     let spinlock = Spinlock::new(data);
//!
//!     // Lock the spinlock to get a mutex guard for the data
//!     let mut locked_data = spinlock.lock();
//!     // The guard implements the `Deref` trait, so we can use it like a `&String`
//!     assert_eq!(locked_data.as_str(), "Hello");
//!     // It also implements `DerefMut` so mutation is possible too. This is safe
//!     // because the spinlock ensures mutual exclusion
//!     locked_data.make_ascii_uppercase();
//!     assert_eq!(locked_data.as_str(), "HELLO");
//!
//!     // the guard automatically frees the lock at the end of the scope
//! }
//! ```

#![no_std]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

/// The spinlock implemenation is based on the abstractions provided by the `lock_api` crate.
pub use lock_api;

pub use spinlock::{const_spinlock, MappedSpinlockGuard, RawSpinlock, Spinlock, SpinlockGuard};

mod spinlock;
