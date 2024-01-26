// This implementation is based on:
// https://github.com/Amanieu/parking_lot/tree/fa294cd677936bf365afa0497039953b10c722f5/lock_api
// and
// https://github.com/mvdnes/spin-rs/tree/7516c8037d3d15712ba4d8499ab075e97a19d778

use core::{
    hint,
    sync::atomic::{AtomicBool, Ordering},
};
use lock_api::{GuardSend, RawMutex};

/// Provides mutual exclusion based on spinning on an `AtomicBool`.
///
/// It's recommended to use this type either combination with [`lock_api::Mutex`] or
/// through the [`Spinlock`] type.
///
/// ## Example
///
/// ```rust
/// use lock_api::RawMutex;
/// let lock = spinning_top::RawSpinlock::INIT;
/// assert_eq!(lock.try_lock(), true); // lock it
/// assert_eq!(lock.try_lock(), false); // can't be locked a second time
/// unsafe { lock.unlock(); } // unlock it
/// assert_eq!(lock.try_lock(), true); // now it can be locked again
#[derive(Debug)]
pub struct RawSpinlock {
    /// Whether the spinlock is locked.
    locked: AtomicBool,
}

impl RawSpinlock {
    // Can fail to lock even if the spinlock is not locked. May be more efficient than `try_lock`
    // when called in a loop.
    fn try_lock_weak(&self) -> bool {
        // The Orderings are the same as try_lock, and are still correct here.
        self.locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }
}

unsafe impl RawMutex for RawSpinlock {
    const INIT: RawSpinlock = RawSpinlock {
        locked: AtomicBool::new(false),
    };

    // A spinlock guard can be sent to another thread and unlocked there
    type GuardMarker = GuardSend;

    fn lock(&self) {
        while !self.try_lock_weak() {
            // Wait until the lock looks unlocked before retrying
            // Code from https://github.com/mvdnes/spin-rs/commit/d3e60d19adbde8c8e9d3199c7c51e51ee5a20bf6
            while self.is_locked() {
                // Tell the CPU that we're inside a busy-wait loop
                hint::spin_loop();
            }
        }
    }

    fn try_lock(&self) -> bool {
        // Code taken from:
        // https://github.com/Amanieu/parking_lot/blob/fa294cd677936bf365afa0497039953b10c722f5/lock_api/src/lib.rs#L49-L53
        //
        // The reason for using a strong compare_exchange is explained here:
        // https://github.com/Amanieu/parking_lot/pull/207#issuecomment-575869107
        //
        // The second Ordering argument specfies the ordering when the compare_exchange
        // fails. Since we don't access any critical data if we fail to acquire the lock,
        // we can use a Relaxed ordering in this case.
        self.locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    unsafe fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }

    fn is_locked(&self) -> bool {
        // Relaxed is sufficient because this operation does not provide synchronization, only atomicity.
        self.locked.load(Ordering::Relaxed)
    }
}

/// A mutual exclusion (Mutex) type based on busy-waiting.
///
/// Calling `lock` (or `try_lock`) on this type returns a [`SpinlockGuard`], which
/// automatically frees the lock when it goes out of scope.
///
/// ## Example
///
/// ```rust
/// use spinning_top::Spinlock;
///
/// fn main() {
///     // Wrap some data in a spinlock
///     let data = String::from("Hello");
///     let spinlock = Spinlock::new(data);
///     make_uppercase(&spinlock); // only pass a shared reference
///
///     // We have ownership of the spinlock, so we can extract the data without locking
///     // Note: this consumes the spinlock
///     let data = spinlock.into_inner();
///     assert_eq!(data.as_str(), "HELLO");
/// }
///
/// fn make_uppercase(spinlock: &Spinlock<String>) {
///     // Lock the spinlock to get a mutable reference to the data
///     let mut locked_data = spinlock.lock();
///     assert_eq!(locked_data.as_str(), "Hello");
///     locked_data.make_ascii_uppercase();
///
///     // the lock is automatically freed at the end of the scope
/// }
/// ```
///
/// ## Usage in statics
///
/// `Spinlock::new` is a `const` function. This makes the `Spinlock` type
/// usable in statics:
///
/// ```rust,ignore
/// use spinning_top::Spinlock;
///
/// static DATA: Spinlock<u32> = Spinlock::new(0);
///
/// fn main() {
///     let mut data = DATA.lock();
///     *data += 1;
///     assert_eq!(*data, 1);
/// }
/// ```
pub type Spinlock<T> = lock_api::Mutex<RawSpinlock, T>;

/// A RAII guard that frees the spinlock when it goes out of scope.
///
/// Allows access to the locked data through the [`core::ops::Deref`] and [`core::ops::DerefMut`] operations.
///
/// ## Example
///
/// ```rust
/// use spinning_top::{Spinlock, SpinlockGuard};
///
/// let spinlock = Spinlock::new(Vec::new());
///
/// // begin a new scope
/// {
///     // lock the spinlock to create a `SpinlockGuard`
///     let mut guard: SpinlockGuard<_> = spinlock.lock();
///
///     // guard can be used like a `&mut Vec` since it implements `DerefMut`
///     guard.push(1);
///     guard.push(2);
///     assert_eq!(guard.len(), 2);
/// } // guard is dropped -> frees the spinlock again
///
/// // spinlock is unlocked again
/// assert!(spinlock.try_lock().is_some());
/// ```
pub type SpinlockGuard<'a, T> = lock_api::MutexGuard<'a, RawSpinlock, T>;

/// A RAII guard returned by `SpinlockGuard::map`.
///
/// ## Example
/// ```rust
/// use spinning_top::{MappedSpinlockGuard, Spinlock, SpinlockGuard};
///
/// let spinlock = Spinlock::new(Some(3));
///
/// // Begin a new scope.
/// {
///     // Lock the spinlock to create a `SpinlockGuard`.
///     let mut guard: SpinlockGuard<_> = spinlock.lock();
///
///     // Map the internal value of `gurad`. `guard` is moved.
///     let mut mapped: MappedSpinlockGuard<'_, _> =
///         SpinlockGuard::map(guard, |g| g.as_mut().unwrap());
///     assert_eq!(*mapped, 3);
///
///     *mapped = 5;
///     assert_eq!(*mapped, 5);
/// } // `mapped` is dropped -> frees the spinlock again.
///
/// // The operation is reflected to the original lock.
/// assert_eq!(*spinlock.lock(), Some(5));
/// ```
pub type MappedSpinlockGuard<'a, T> = lock_api::MappedMutexGuard<'a, RawSpinlock, T>;

/// Create an unlocked `Spinlock` in a `const` context.
///
/// ## Example
///
/// ```rust
/// use spinning_top::{const_spinlock, Spinlock};
///
/// static SPINLOCK: Spinlock<i32> = const_spinlock(42);
/// ```
pub const fn const_spinlock<T>(val: T) -> Spinlock<T> {
    Spinlock::const_new(<RawSpinlock as lock_api::RawMutex>::INIT, val)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_lock() {
        let spinlock = Spinlock::new(42);
        let data = spinlock.try_lock();
        assert!(data.is_some());
        assert_eq!(*data.unwrap(), 42);
    }

    #[test]
    fn mutual_exclusion() {
        let spinlock = Spinlock::new(1);
        let data = spinlock.try_lock();
        assert!(data.is_some());
        assert!(spinlock.try_lock().is_none());
        assert!(spinlock.try_lock().is_none()); // still None
        core::mem::drop(data);
        assert!(spinlock.try_lock().is_some());
    }

    #[test]
    fn three_locks() {
        let spinlock1 = Spinlock::new(1);
        let spinlock2 = Spinlock::new(2);
        let spinlock3 = Spinlock::new(3);
        let data1 = spinlock1.try_lock();
        let data2 = spinlock2.try_lock();
        let data3 = spinlock3.try_lock();
        assert!(data1.is_some());
        assert!(data2.is_some());
        assert!(data3.is_some());
        assert!(spinlock1.try_lock().is_none());
        assert!(spinlock1.try_lock().is_none()); // still None
        assert!(spinlock2.try_lock().is_none());
        assert!(spinlock3.try_lock().is_none());
        core::mem::drop(data3);
        assert!(spinlock3.try_lock().is_some());
    }

    #[test]
    fn mapped_lock() {
        let spinlock = Spinlock::new([1, 2, 3]);
        let data = spinlock.lock();
        let mut mapped = SpinlockGuard::map(data, |d| &mut d[0]);
        assert_eq!(*mapped, 1);
        *mapped = 4;
        assert_eq!(*mapped, 4);
        core::mem::drop(mapped);
        assert!(!spinlock.is_locked());
        assert_eq!(*spinlock.lock(), [4, 2, 3]);
    }
}
