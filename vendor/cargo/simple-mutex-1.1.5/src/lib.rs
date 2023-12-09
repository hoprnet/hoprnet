//! A simple mutex.
//!
//! More efficient than [`std::sync::Mutex`] and simpler than
//! [`parking_lot::Mutex`](https://docs.rs/parking_lot).
//!
//! The locking mechanism uses eventual fairness to ensure locking will be fair on average without
//! sacrificing performance. This is done by forcing a fair lock whenever a lock operation is
//! starved for longer than 0.5 milliseconds.
//!
//! # Examples
//!
//! ```
//! use simple_mutex::Mutex;
//! use std::sync::Arc;
//! use std::thread;
//!
//! let m = Arc::new(Mutex::new(0));
//! let mut threads = vec![];
//!
//! for _ in 0..10 {
//!     let m = m.clone();
//!     threads.push(thread::spawn(move || {
//!         *m.lock() += 1;
//!     }));
//! }
//!
//! for t in threads {
//!     t.join().unwrap();
//! }
//! assert_eq!(*m.lock(), 10);
//! ```

#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]

use std::cell::UnsafeCell;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{self, AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use event_listener::Event;

/// A simple mutex.
pub struct Mutex<T> {
    /// Current state of the mutex.
    ///
    /// The least significant bit is set to 1 if the mutex is locked.
    /// The other bits hold the number of starved lock operations.
    state: AtomicUsize,

    /// Lock operations waiting for the mutex to be released.
    lock_ops: Event,

    /// The value inside the mutex.
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    /// Creates a new mutex.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_mutex::Mutex;
    ///
    /// let mutex = Mutex::new(0);
    /// ```
    pub fn new(data: T) -> Mutex<T> {
        Mutex {
            state: AtomicUsize::new(0),
            lock_ops: Event::new(),
            data: UnsafeCell::new(data),
        }
    }

    /// Acquires the mutex.
    ///
    /// Returns a guard that releases the mutex when dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_mutex::Mutex;
    ///
    /// let mutex = Mutex::new(10);
    /// let guard = mutex.lock();
    /// assert_eq!(*guard, 10);
    /// ```
    #[inline]
    pub fn lock(&self) -> MutexGuard<'_, T> {
        if let Some(guard) = self.try_lock() {
            return guard;
        }
        self.lock_slow()
    }

    /// Slow path for acquiring the mutex.
    #[cold]
    fn lock_slow(&self) -> MutexGuard<'_, T> {
        for step in 0..10 {
            // Try locking if nobody is being starved.
            match self.state.compare_and_swap(0, 1, Ordering::Acquire) {
                // Lock acquired!
                0 => return MutexGuard(self),

                // Lock is held and nobody is starved.
                1 => {}

                // Somebody is starved.
                _ => break,
            }

            // Back off before trying again.
            if step <= 3 {
                for _ in 0..1 << step {
                    atomic::spin_loop_hint();
                }
            } else {
                thread::yield_now();
            }
        }

        // Get the current time.
        let start = Instant::now();

        loop {
            // Start listening for events.
            let listener = self.lock_ops.listen();

            // Try locking if nobody is being starved.
            match self.state.compare_and_swap(0, 1, Ordering::Acquire) {
                // Lock acquired!
                0 => return MutexGuard(self),

                // Lock is held and nobody is starved.
                1 => {}

                // Somebody is starved.
                _ => break,
            }

            // Wait for a notification.
            listener.wait();

            // Try locking if nobody is being starved.
            match self.state.compare_and_swap(0, 1, Ordering::Acquire) {
                // Lock acquired!
                0 => return MutexGuard(self),

                // Lock is held and nobody is starved.
                1 => {}

                // Somebody is starved.
                _ => {
                    // Notify the first listener in line because we probably received a
                    // notification that was meant for a starved thread.
                    self.lock_ops.notify(1);
                    break;
                }
            }

            // If waiting for too long, fall back to a fairer locking strategy that will prevent
            // newer lock operations from starving us forever.
            if start.elapsed() > Duration::from_micros(500) {
                break;
            }
        }

        // Increment the number of starved lock operations.
        self.state.fetch_add(2, Ordering::Release);

        // Decrement the counter when exiting this function.
        let _call = CallOnDrop(|| {
            self.state.fetch_sub(2, Ordering::Release);
        });

        loop {
            // Start listening for events.
            let listener = self.lock_ops.listen();

            // Try locking if nobody else is being starved.
            match self.state.compare_and_swap(2, 2 | 1, Ordering::Acquire) {
                // Lock acquired!
                2 => return MutexGuard(self),

                // Lock is held by someone.
                s if s % 2 == 1 => {}

                // Lock is available.
                _ => {
                    // Be fair: notify the first listener and then go wait in line.
                    self.lock_ops.notify(1);
                }
            }

            // Wait for a notification.
            listener.wait();

            // Try acquiring the lock without waiting for others.
            if self.state.fetch_or(1, Ordering::Acquire) % 2 == 0 {
                return MutexGuard(self);
            }
        }
    }

    /// Attempts to acquire the mutex.
    ///
    /// If the mutex could not be acquired at this time, then [`None`] is returned. Otherwise, a
    /// guard is returned that releases the mutex when dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_mutex::Mutex;
    ///
    /// let mutex = Mutex::new(10);
    /// if let Some(guard) = mutex.try_lock() {
    ///     assert_eq!(*guard, 10);
    /// }
    /// # ;
    /// ```
    #[inline]
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if self.state.compare_and_swap(0, 1, Ordering::Acquire) == 0 {
            Some(MutexGuard(self))
        } else {
            None
        }
    }

    /// Consumes the mutex, returning the underlying data.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_mutex::Mutex;
    ///
    /// let mutex = Mutex::new(10);
    /// assert_eq!(mutex.into_inner(), 10);
    /// ```
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the mutex mutably, no actual locking takes place -- the mutable
    /// borrow statically guarantees the mutex is not already acquired.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_mutex::Mutex;
    ///
    /// let mut mutex = Mutex::new(0);
    /// *mutex.get_mut() = 10;
    /// assert_eq!(*mutex.lock(), 10);
    /// ```
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }
}

impl<T: fmt::Debug> fmt::Debug for Mutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct Locked;
        impl fmt::Debug for Locked {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("<locked>")
            }
        }

        match self.try_lock() {
            None => f.debug_struct("Mutex").field("data", &Locked).finish(),
            Some(guard) => f.debug_struct("Mutex").field("data", &&*guard).finish(),
        }
    }
}

impl<T> From<T> for Mutex<T> {
    fn from(val: T) -> Mutex<T> {
        Mutex::new(val)
    }
}

impl<T: Default> Default for Mutex<T> {
    fn default() -> Mutex<T> {
        Mutex::new(Default::default())
    }
}

/// A guard that releases the mutex when dropped.
pub struct MutexGuard<'a, T>(&'a Mutex<T>);

unsafe impl<T: Send> Send for MutexGuard<'_, T> {}
unsafe impl<T: Sync> Sync for MutexGuard<'_, T> {}

impl<'a, T> MutexGuard<'a, T> {
    /// Returns a reference to the mutex a guard came from.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_mutex::{Mutex, MutexGuard};
    ///
    /// let mutex = Mutex::new(10i32);
    /// let guard = mutex.lock();
    /// dbg!(MutexGuard::source(&guard));
    /// ```
    pub fn source(guard: &MutexGuard<'a, T>) -> &'a Mutex<T> {
        guard.0
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        // Remove the last bit and notify a waiting lock operation.
        self.0.state.fetch_sub(1, Ordering::Release);

        if cfg!(any(target_arch = "x86", target_arch = "x86_64")) {
            // On x86 architectures, `fetch_sub()` has the effect of a `SeqCst` fence so we don't need
            // to emit another one here.
            self.0.lock_ops.notify_relaxed(1);
        } else {
            self.0.lock_ops.notify(1);
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: fmt::Display> fmt::Display for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.0.data.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.0.data.get() }
    }
}

/// Calls a function when dropped.
struct CallOnDrop<F: Fn()>(F);

impl<F: Fn()> Drop for CallOnDrop<F> {
    fn drop(&mut self) {
        (self.0)();
    }
}
