//! Extension traits for `std::sync::RwLock`.

use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Extension trait with useful methods for [`std::sync::RwLock`].
///
/// [`std::sync::RwLock`]: https://doc.rust-lang.org/std/sync/struct.RwLock.html
pub trait RwLockExt<T> {
    /// Shorthand for `lock.read().unwrap()` with a better panic message.
    ///
    /// This method is intended to be used in situations where poisoned locks are
    /// considered an exceptional situation and should always result in panic.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, RwLock};
    /// use stdext::prelude::*;
    ///
    /// let lock = Arc::new(RwLock::new(1));
    ///
    /// let n = lock.force_read();
    /// assert_eq!(*n, 1);
    /// ```
    fn force_read(&self) -> RwLockReadGuard<T>;

    /// Shorthand for `lock.write().unwrap()` with a better panic message.
    ///
    /// This method is intended to be used in situations where poisoned locks are
    /// considered an exceptional situation and should always result in panic.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, RwLock};
    /// use stdext::prelude::*;
    ///
    /// let lock = Arc::new(RwLock::new(1));
    ///
    /// {
    ///     let mut n = lock.force_write();
    ///     *n = 2;
    /// }
    ///
    /// let n = lock.force_read();
    /// assert_eq!(*n, 2);
    /// ```
    fn force_write(&self) -> RwLockWriteGuard<T>;
}

impl<T> RwLockExt<T> for RwLock<T> {
    fn force_read(&self) -> RwLockReadGuard<T> {
        self.read()
            .expect("Unable to obtain read lock: RwLock is poisoned")
    }
    fn force_write(&self) -> RwLockWriteGuard<T> {
        self.write()
            .expect("Unable to obtain write lock: RwLock is poisoned")
    }
}
