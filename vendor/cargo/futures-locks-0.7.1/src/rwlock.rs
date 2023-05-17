// vim: tw=80

use futures_channel::oneshot;
use futures_task::{Context, Poll};
use std::{
    cell::UnsafeCell,
    clone::Clone,
    collections::VecDeque,
    future::Future,
    ops::{Deref, DerefMut},
    pin::Pin,
    sync,
};
use super::{FutState, TryLockError};
#[cfg(feature = "tokio")] use tokio::task;

/// An RAII guard, much like `std::sync::RwLockReadGuard`.  The wrapped data can
/// be accessed via its `Deref` implementation.
#[derive(Debug)]
pub struct RwLockReadGuard<T: ?Sized> {
    rwlock: RwLock<T>
}

impl<T: ?Sized> Deref for RwLockReadGuard<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {&*self.rwlock.inner.data.get()}
    }
}

impl<T: ?Sized> Drop for RwLockReadGuard<T> {
    fn drop(&mut self) {
        self.rwlock.unlock_reader();
    }
}

/// An RAII guard, much like `std::sync::RwLockWriteGuard`.  The wrapped data
/// can be accessed via its `Deref`  and `DerefMut` implementations.
#[derive(Debug)]
pub struct RwLockWriteGuard<T: ?Sized> {
    rwlock: RwLock<T>
}

impl<T: ?Sized> Deref for RwLockWriteGuard<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {&*self.rwlock.inner.data.get()}
    }
}

impl<T: ?Sized> DerefMut for RwLockWriteGuard<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {&mut *self.rwlock.inner.data.get()}
    }
}

impl<T: ?Sized> Drop for RwLockWriteGuard<T> {
    fn drop(&mut self) {
        self.rwlock.unlock_writer();
    }
}

/// A `Future` representing a pending `RwLock` shared acquisition.
pub struct RwLockReadFut<T: ?Sized> {
    state: FutState,
    rwlock: RwLock<T>,
}

impl<T: ?Sized> RwLockReadFut<T> {
    fn new(state: FutState, rwlock: RwLock<T>) -> Self {
        RwLockReadFut{state, rwlock}
    }
}

impl<T: ?Sized> Drop for RwLockReadFut<T> {
    fn drop(&mut self) {
        match self.state {
            FutState::New => {
                // RwLock hasn't yet been modified; nothing to do
            },
            FutState::Pending(ref mut rx) => {
                rx.close();
                match rx.try_recv() {
                    Ok(Some(())) => {
                        // This future received ownership of the lock, but got
                        // dropped before it was ever polled.  Release the
                        // lock.
                        self.rwlock.unlock_reader()
                    },
                    Ok(None) => {
                        // Dropping the Future before it acquires the lock is
                        // equivalent to cancelling it.
                    },
                    Err(oneshot::Canceled) => {
                        // Never received ownership of the lock
                    }
                }
            },
            FutState::Acquired => {
                // The RwLockReadGuard will take care of releasing the RwLock
            }
        }
    }
}

impl<T: ?Sized> Future for RwLockReadFut<T> {
    type Output = RwLockReadGuard<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let (result, new_state) = match self.state {
            FutState::New => {
                let mut lock_data = self.rwlock.inner.mutex.lock()
                    .expect("sync::Mutex::lock");
                if lock_data.exclusive {
                    let (tx, mut rx) = oneshot::channel::<()>();
                    lock_data.read_waiters.push_back(tx);
                    // Even though we know it isn't ready, we need to poll the
                    // receiver in order to register our task for notification.
                    assert!(Pin::new(&mut rx).poll(cx).is_pending());
                    (Poll::Pending, FutState::Pending(rx))
                } else {
                    lock_data.num_readers += 1;
                    let guard = RwLockReadGuard{rwlock: self.rwlock.clone()};
                    (Poll::Ready(guard), FutState::Acquired)
                }
            },
            FutState::Pending(ref mut rx) => {
                match Pin::new(rx).poll(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(_) => {
                        let state = FutState::Acquired;
                        let result = Poll::Ready(
                            RwLockReadGuard{rwlock: self.rwlock.clone()}
                        );
                        (result, state)
                    }  // LCOV_EXCL_LINE   kcov false negative
                }
            },
            FutState::Acquired => panic!("Double-poll of ready Future")
        };
        self.state = new_state;
        result
    }
}

/// A `Future` representing a pending `RwLock` exclusive acquisition.
pub struct RwLockWriteFut<T: ?Sized> {
    state: FutState,
    rwlock: RwLock<T>,
}

impl<T: ?Sized> RwLockWriteFut<T> {
    fn new(state: FutState, rwlock: RwLock<T>) -> Self {
        RwLockWriteFut{state, rwlock}
    }
}

impl<T: ?Sized> Drop for RwLockWriteFut<T> {
    fn drop(&mut self) {
        match self.state {
            FutState::New => {
                // RwLock hasn't yet been modified; nothing to do
            },
            FutState::Pending(ref mut rx) => {
                rx.close();
                match rx.try_recv() {
                    Ok(Some(())) => {
                        // This future received ownership of the lock, but got
                        // dropped before it was ever polled.  Release the
                        // lock.
                        self.rwlock.unlock_writer()
                    },
                    Ok(None) => {
                        // Dropping the Future before it acquires the lock is
                        // equivalent to cancelling it.
                    },
                    Err(oneshot::Canceled) => {
                        // Never received ownership of the lock
                    }
                }
            },
            FutState::Acquired => {
                // The RwLockWriteGuard will take care of releasing the RwLock
            }
        }
    }
}

impl<T: ?Sized> Future for RwLockWriteFut<T> {
    type Output = RwLockWriteGuard<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let (result, new_state) = match self.state {
            FutState::New => {
                let mut lock_data = self.rwlock.inner.mutex.lock()
                    .expect("sync::Mutex::lock");
                if lock_data.exclusive || lock_data.num_readers > 0 {
                    let (tx, mut rx) = oneshot::channel::<()>();
                    lock_data.write_waiters.push_back(tx);
                    // Even though we know it isn't ready, we need to poll the
                    // receiver in order to register our task for notification.
                    assert!(Pin::new(&mut rx).poll(cx).is_pending());
                    (Poll::Pending, FutState::Pending(rx))
                } else {
                    lock_data.exclusive = true;
                    let guard = RwLockWriteGuard{rwlock: self.rwlock.clone()};
                    (Poll::Ready(guard), FutState::Acquired)
                }
            },
            FutState::Pending(ref mut rx) => {
                match Pin::new(rx).poll(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(_) => {
                        let state = FutState::Acquired;
                        let result = Poll::Ready(
                            RwLockWriteGuard{rwlock: self.rwlock.clone()}
                        );
                        (result, state)
                    }  // LCOV_EXCL_LINE   kcov false negative
                }
            },
            FutState::Acquired => panic!("Double-poll of ready Future")
        };
        self.state = new_state;
        result
    }
}

#[derive(Debug, Default)]
struct RwLockData {
    /// True iff the `RwLock` is currently exclusively owned
    exclusive: bool,

    /// The number of tasks that currently have shared ownership of the RwLock
    num_readers: u32,

    // FIFO queue of waiting readers
    read_waiters: VecDeque<oneshot::Sender<()>>,

    // FIFO queue of waiting writers
    write_waiters: VecDeque<oneshot::Sender<()>>,
}

#[derive(Debug, Default)]
struct Inner<T: ?Sized> {
    mutex: sync::Mutex<RwLockData>,
    data: UnsafeCell<T>,
}

/// A Futures-aware RwLock.
///
/// `std::sync::RwLock` cannot be used in an asynchronous environment like
/// Tokio, because an acquisition can block an entire reactor.  This class can
/// be used instead.  It functions much like `std::sync::RwLock`.  Unlike that
/// class, it also has a builtin `Arc`, making it accessible from multiple
/// threads.  It's also safe to `clone`.  Also unlike `std::sync::RwLock`, this
/// class does not detect lock poisoning.
#[derive(Debug, Default)]
pub struct RwLock<T: ?Sized> {
    inner: sync::Arc<Inner<T>>,
}

impl<T: ?Sized> Clone for RwLock<T> {
    fn clone(&self) -> RwLock<T> {
        RwLock { inner: self.inner.clone()}
    }
}

impl<T> RwLock<T> {
    /// Create a new `RwLock` in the unlocked state.
    pub fn new(t: T) -> RwLock<T> {
        let lock_data = RwLockData {
            exclusive: false,
            num_readers: 0,
            read_waiters: VecDeque::new(),
            write_waiters: VecDeque::new(),
        };  // LCOV_EXCL_LINE   kcov false negative
        let inner = Inner {
            mutex: sync::Mutex::new(lock_data),
            data: UnsafeCell::new(t)
        };  // LCOV_EXCL_LINE   kcov false negative
        RwLock { inner: sync::Arc::new(inner)}
    }

    /// Consumes the `RwLock` and returns the wrapped data.  If the `RwLock`
    /// still has multiple references (not necessarily locked), returns a copy
    /// of `self` instead.
    pub fn try_unwrap(self) -> Result<T, RwLock<T>> {
        match sync::Arc::try_unwrap(self.inner) {
            Ok(inner) => Ok({
                // `unsafe` is no longer needed as of somewhere around 1.25.0.
                // https://github.com/rust-lang/rust/issues/35067
                #[allow(unused_unsafe)]
                unsafe { inner.data.into_inner() }
            }),
            Err(arc) => Err(RwLock {inner: arc})
        }
    }
}

impl<T: ?Sized> RwLock<T> {
    /// Returns a reference to the underlying data, if there are no other
    /// clones of the `RwLock`.
    ///
    /// Since this call borrows the `RwLock` mutably, no actual locking takes
    /// place -- the mutable borrow statically guarantees no locks exist.
    /// However, if the `RwLock` has already been cloned, then `None` will be
    /// returned instead.
    ///
    /// # Examples
    ///
    /// ```
    /// # use futures_locks::*;
    /// # fn main() {
    /// let mut lock = RwLock::<u32>::new(0);
    /// *lock.get_mut().unwrap() += 5;
    /// assert_eq!(lock.try_unwrap().unwrap(), 5);
    /// # }
    /// ```
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if let Some(inner) = sync::Arc::get_mut(&mut self.inner) {
            let lock_data = inner.mutex.get_mut().unwrap();
            let data = unsafe { inner.data.get().as_mut() }.unwrap();
            debug_assert!(!lock_data.exclusive);
            debug_assert_eq!(lock_data.num_readers, 0);
            Some(data)
        } else {
            None
        }
    }

    /// Acquire the `RwLock` nonexclusively, read-only, blocking the task in the
    /// meantime.
    ///
    /// When the returned `Future` is ready, then this task will have read-only
    /// access to the protected data.
    ///
    /// # Examples
    /// ```
    /// # use futures_locks::*;
    /// # use futures::executor::block_on;
    /// # use futures::{Future, FutureExt};
    /// # fn main() {
    /// let rwlock = RwLock::<u32>::new(42);
    /// let fut = rwlock.read().map(|mut guard| { *guard });
    /// assert_eq!(block_on(fut), 42);
    /// # }
    ///
    /// ```
    pub fn read(&self) -> RwLockReadFut<T> {
        RwLockReadFut::new(FutState::New, self.clone())
    }

    /// Acquire the `RwLock` exclusively, read-write, blocking the task in the
    /// meantime.
    ///
    /// When the returned `Future` is ready, then this task will have read-write
    /// access to the protected data.
    ///
    /// # Examples
    /// ```
    /// # use futures_locks::*;
    /// # use futures::executor::block_on;
    /// # use futures::{Future, FutureExt};
    /// # fn main() {
    /// let rwlock = RwLock::<u32>::new(42);
    /// let fut = rwlock.write().map(|mut guard| { *guard = 5;});
    /// block_on(fut);
    /// assert_eq!(rwlock.try_unwrap().unwrap(), 5);
    /// # }
    ///
    /// ```
    pub fn write(&self) -> RwLockWriteFut<T> {
        RwLockWriteFut::new(FutState::New, self.clone())
    }

    /// Attempts to acquire the `RwLock` nonexclusively.
    ///
    /// If the operation would block, returns `Err` instead.  Otherwise, returns
    /// a guard (not a `Future`).
    ///
    /// # Examples
    /// ```
    /// # use futures_locks::*;
    /// # fn main() {
    /// let mut lock = RwLock::<u32>::new(5);
    /// let r = match lock.try_read() {
    ///     Ok(guard) => *guard,
    ///     Err(_) => panic!("Better luck next time!")
    /// };
    /// assert_eq!(5, r);
    /// # }
    /// ```
    pub fn try_read(&self) -> Result<RwLockReadGuard<T>, TryLockError> {
        let mut lock_data = self.inner.mutex.lock().expect("sync::Mutex::lock");
        if lock_data.exclusive {
            Err(TryLockError)
        } else {
            lock_data.num_readers += 1;
            Ok(RwLockReadGuard{rwlock: self.clone()})
        }
    }

    /// Attempts to acquire the `RwLock` exclusively.
    ///
    /// If the operation would block, returns `Err` instead.  Otherwise, returns
    /// a guard (not a `Future`).
    ///
    /// # Examples
    /// ```
    /// # use futures_locks::*;
    /// # fn main() {
    /// let mut lock = RwLock::<u32>::new(5);
    /// match lock.try_write() {
    ///     Ok(mut guard) => *guard += 5,
    ///     Err(_) => panic!("Better luck next time!")
    /// }
    /// assert_eq!(10, lock.try_unwrap().unwrap());
    /// # }
    /// ```
    pub fn try_write(&self) -> Result<RwLockWriteGuard<T>, TryLockError> {
        let mut lock_data = self.inner.mutex.lock().expect("sync::Mutex::lock");
        if lock_data.exclusive || lock_data.num_readers > 0 {
            Err(TryLockError)
        } else {
            lock_data.exclusive = true;
            Ok(RwLockWriteGuard{rwlock: self.clone()})
        }
    }

    /// Release a shared lock of an `RwLock`.
    fn unlock_reader(&self) {
        let mut lock_data = self.inner.mutex.lock().expect("sync::Mutex::lock");
        assert!(lock_data.num_readers > 0);
        assert!(!lock_data.exclusive);
        assert_eq!(lock_data.read_waiters.len(), 0);
        lock_data.num_readers -= 1;
        if lock_data.num_readers == 0 {
            while let Some(tx) = lock_data.write_waiters.pop_front() {
                if tx.send(()).is_ok() {
                    lock_data.exclusive = true;
                    return
                }
            }
        }
    }

    /// Release an exclusive lock of an `RwLock`.
    fn unlock_writer(&self) {
        let mut lock_data = self.inner.mutex.lock().expect("sync::Mutex::lock");
        assert!(lock_data.num_readers == 0);
        assert!(lock_data.exclusive);

        // First try to wake up any writers
        while let Some(tx) = lock_data.write_waiters.pop_front() {
            if tx.send(()).is_ok() {
                return;
            }
        }

        // If there are no writers, try to wake up readers
        lock_data.exclusive = false;
        lock_data.num_readers += lock_data.read_waiters.len() as u32;
        for tx in lock_data.read_waiters.drain(..) {
            // Ignore errors, which are due to a reader's future getting
            // dropped before it was ready
            let _ = tx.send(());
        }
    }
}

impl<T: 'static + ?Sized> RwLock<T> {
    /// Acquires a `RwLock` nonexclusively and performs a computation on its
    /// guarded value in a separate task.  Returns a `Future` containing the
    /// result of the computation.
    ///
    /// When using Tokio, this method will often hold the `RwLock` for less time
    /// than chaining a computation to [`read`](#method.read).  The reason is
    /// that Tokio polls all tasks promptly upon notification.  However, Tokio
    /// does not guarantee that it will poll all futures promptly when their
    /// owning task gets notified.  So it's best to hold `RwLock`s within their
    /// own tasks, lest their continuations get blocked by slow stacked
    /// combinators.
    ///
    /// # Examples
    ///
    /// ```
    /// # use futures_locks::*;
    /// # use futures::{Future, future::ready};
    /// # use tokio::runtime::Runtime;
    /// # fn main() {
    /// let rwlock = RwLock::<u32>::new(5);
    /// let mut rt = Runtime::new().unwrap();
    /// let r = rt.block_on(async {
    ///     rwlock.with_read(|mut guard| {
    ///         ready(*guard)
    ///     }).await
    /// });
    /// assert_eq!(r, 5);
    /// # }
    /// ```
    #[cfg(any(feature = "tokio", all(docsrs, rustdoc)))]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
    pub fn with_read<B, F, R>(&self, f: F)
        -> impl Future<Output = R>
        where F: FnOnce(RwLockReadGuard<T>) -> B + Send + 'static,
              B: Future<Output = R> + Send + 'static,
              R: Send + 'static,
              T: Send
    {
        let jh = tokio::spawn({
            let fut = self.read();
            async move { f(fut.await).await }
        });

        async move { jh.await.unwrap() }
    }

    /// Like [`with_read`](#method.with_read) but for Futures that aren't
    /// `Send`.  Spawns a new task on a single-threaded Runtime to complete the
    /// Future.
    ///
    /// # Examples
    ///
    /// ```
    /// # use futures_locks::*;
    /// # use futures::{Future, future::ready};
    /// # use std::rc::Rc;
    /// # use tokio::runtime::Runtime;
    /// # fn main() {
    /// // Note: Rc is not `Send`
    /// let rwlock = RwLock::<Rc<u32>>::new(Rc::new(5));
    /// let mut rt = Runtime::new().unwrap();
    /// let r = rt.block_on(async {
    ///     rwlock.with_read_local(|mut guard| {
    ///         ready(**guard)
    ///     }).await
    /// });
    /// assert_eq!(r, 5);
    /// # }
    /// ```
    #[cfg(any(feature = "tokio", all(docsrs, rustdoc)))]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
    pub fn with_read_local<B, F, R>(&self, f: F)
        -> impl Future<Output = R>
        where F: FnOnce(RwLockReadGuard<T>) -> B + 'static + Unpin,
              B: Future<Output = R> + 'static,
              R: 'static
    {
        let local = task::LocalSet::new();
        let jh = local.spawn_local({
            let fut = self.read();
            async move { f(fut.await).await }
        });

        async move {
            local.await;
            jh.await.unwrap()
        }
    }

    /// Acquires a `RwLock` exclusively and performs a computation on its
    /// guarded value in a separate task.  Returns a `Future` containing the
    /// result of the computation.
    ///
    /// When using Tokio, this method will often hold the `RwLock` for less time
    /// than chaining a computation to [`write`](#method.write).  The reason is
    /// that Tokio polls all tasks promptly upon notification.  However, Tokio
    /// does not guarantee that it will poll all futures promptly when their
    /// owning task gets notified.  So it's best to hold `RwLock`s within their
    /// own tasks, lest their continuations get blocked by slow stacked
    /// combinators.
    ///
    /// # Examples
    ///
    /// ```
    /// # use futures::{Future, future::ready};
    /// # use futures_locks::*;
    /// # use tokio::runtime::Runtime;
    /// # fn main() {
    /// let rwlock = RwLock::<u32>::new(0);
    /// let mut rt = Runtime::new().unwrap();
    /// let r = rt.block_on(async {
    ///     rwlock.with_write(|mut guard| {
    ///         *guard += 5;
    ///         ready(())
    ///     }).await
    /// });
    /// assert_eq!(rwlock.try_unwrap().unwrap(), 5);
    /// # }
    /// ```
    #[cfg(any(feature = "tokio", all(docsrs, rustdoc)))]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
    pub fn with_write<B, F, R>(&self, f: F)
        -> impl Future<Output = R>
        where F: FnOnce(RwLockWriteGuard<T>) -> B + Send + 'static,
              B: Future<Output = R> + Send + 'static,
              R: Send + 'static,
              T: Send
    {
        let jh = tokio::spawn({
            let fut = self.write();
            async move { f(fut.await).await }
        });

        async move { jh.await.unwrap() }
    }

    /// Like [`with_write`](#method.with_write) but for Futures that aren't
    /// `Send`.  Spawns a new task on a single-threaded Runtime to complete the
    /// Future.
    ///
    /// # Examples
    ///
    /// ```
    /// # use futures::{Future, future::ready};
    /// # use futures_locks::*;
    /// # use std::rc::Rc;
    /// # use tokio::runtime::Runtime;
    /// # fn main() {
    /// // Note: Rc is not `Send`
    /// let rwlock = RwLock::<Rc<u32>>::new(Rc::new(0));
    /// let mut rt = Runtime::new().unwrap();
    /// let r = rt.block_on(async {
    ///     rwlock.with_write_local(|mut guard| {
    ///         *Rc::get_mut(&mut *guard).unwrap() += 5;
    ///         ready(())
    ///     }).await
    /// });
    /// assert_eq!(*rwlock.try_unwrap().unwrap(), 5);
    /// # }
    /// ```
    #[cfg(any(feature = "tokio", all(docsrs, rustdoc)))]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
    pub fn with_write_local<B, F, R>(&self, f: F)
        -> impl Future<Output = R>
        where F: FnOnce(RwLockWriteGuard<T>) -> B + 'static + Unpin,
              B: Future<Output = R> + 'static,
              R: 'static
    {
        let local = task::LocalSet::new();
        let jh = local.spawn_local({
            let fut = self.write();
            async move { f(fut.await).await }
        });

        async move {
            local.await;
            jh.await.unwrap()
        }
    }
}

// Clippy doesn't like the Arc within Inner.  But the access rules of the RwLock
// make it safe to send.  std::sync::RwLock has the same Send impl
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: ?Sized + Send> Send for RwLock<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for RwLock<T> {}

// LCOV_EXCL_START
#[cfg(test)]
mod t {
    use super::*;

    /// Pet Kcov
    #[test]
    fn debug() {
        let m = RwLock::<u32>::new(0);
        format!("{:?}", &m);
    }

    #[test]
    fn test_default() {
        let lock = RwLock::default();
        let value: u32 = lock.try_unwrap().unwrap();
        let expected = u32::default();

        assert_eq!(expected, value);
    }
}
// LCOV_EXCL_STOP
