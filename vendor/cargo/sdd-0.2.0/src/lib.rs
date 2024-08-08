#![deny(missing_docs, warnings, clippy::all, clippy::pedantic)]
#![doc = include_str!("../README.md")]

mod atomic_owned;
pub use atomic_owned::AtomicOwned;

mod atomic_shared;
pub use atomic_shared::AtomicShared;

mod guard;
pub use guard::Guard;

mod collectible;
pub use collectible::Collectible;

mod owned;
pub use owned::Owned;

mod ptr;
pub use ptr::Ptr;

mod shared;
pub use shared::Shared;

mod tag;
pub use tag::Tag;

mod collector;
mod exit_guard;
mod ref_counted;

/// Suspends the garbage collector of the current thread.
///
/// If returns `false` if there is an active [`Guard`] in the thread. Otherwise, it passes all its
/// retired instances to a free flowing garbage container that can be cleaned up by other threads.
///
/// # Examples
///
/// ```
/// use sdd::{suspend, Guard, Shared};
///
/// assert!(suspend());
///
/// {
///     let shared: Shared<usize> = Shared::new(47);
///     let guard = Guard::new();
///     shared.release(&guard);
///     assert!(!suspend());
/// }
///
/// assert!(suspend());
///
/// let new_shared: Shared<usize> = Shared::new(17);
/// let guard = Guard::new();
/// new_shared.release(&guard);
/// ```
#[inline]
#[must_use]
pub fn suspend() -> bool {
    collector::Collector::pass_garbage()
}

#[cfg(test)]
mod tests;
