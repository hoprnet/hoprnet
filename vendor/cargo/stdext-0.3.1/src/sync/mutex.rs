//! Extension traits for `std::sync::Mutex`.

use std::sync::{Mutex, MutexGuard};

/// Extension trait with useful methods for [`std::sync::Mutex`].
///
/// [`std::sync::Mutex`]: https://doc.rust-lang.org/std/sync/struct.Mutex.html
pub trait MutexExt<T> {
    /// Shorthand for `mutex.lock().unwrap()` with a better panic message.
    ///
    /// This method is intended to be used in situations where poisoned locks are
    /// considered an exceptional situation and should always result in panic.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Mutex;
    /// use stdext::prelude::*;
    ///
    /// let lock = Mutex::new(1);
    ///
    /// let n = lock.force_lock();
    /// assert_eq!(*n, 1);
    /// ```
    fn force_lock(&self) -> MutexGuard<T>;
}

impl<T> MutexExt<T> for Mutex<T> {
    fn force_lock(&self) -> MutexGuard<T> {
        self.lock()
            .expect("Unable to obtain lock: Mutex is poisoned")
    }
}
