use core::task::Waker;

use crate::{DiatomicWaker, WaitUntil};

/// A non-owned object that can await notifications from one or several
/// [`WakeSourceRef`](WakeSourceRef)s.
///
/// See the [crate-level documentation](crate) for usage.
#[derive(Debug)]
pub struct WakeSinkRef<'a> {
    /// The shared data.
    pub(crate) inner: &'a DiatomicWaker,
}

impl<'a> WakeSinkRef<'a> {
    /// Creates a new `WakeSourceRef` associated to this sink with the same
    /// lifetime.
    #[inline]
    pub fn source_ref(&self) -> WakeSourceRef<'a> {
        WakeSourceRef { inner: self.inner }
    }

    /// Registers a new waker.
    ///
    /// Registration is lazy: the waker is cloned only if it differs from the
    /// last registered waker (note that the last registered waker is cached
    /// even if it was unregistered).
    #[inline]
    pub fn register(&mut self, waker: &Waker) {
        // Safety: `DiatomicWaker::register`, `DiatomicWaker::unregister` and
        // `DiatomicWaker::wait_until` cannot be used concurrently from multiple
        // thread since `WakeSinkRef` does not implement `Clone` and the
        // wrappers of the above methods require exclusive ownership to
        // `WakeSinkRef`.
        unsafe { self.inner.register(waker) };
    }

    /// Unregisters the waker.
    ///
    /// After the waker is unregistered, subsequent calls to
    /// `WakeSourceRef::notify` will be ignored.
    ///
    /// Note that the previously-registered waker (if any) remains cached.
    #[inline]
    pub fn unregister(&mut self) {
        // Safety: `DiatomicWaker::register`, `DiatomicWaker::unregister` and
        // `DiatomicWaker::wait_until` cannot be used concurrently from multiple
        // thread since `WakeSinkRef` does not implement `Clone` and the
        // wrappers of the above methods require exclusive ownership to
        // `WakeSinkRef`.
        unsafe { self.inner.unregister() };
    }

    /// Returns a future that can be `await`ed until the provided predicate
    /// returns a value.
    ///
    /// The predicate is checked each time a notification is received.
    #[inline]
    pub fn wait_until<P, T>(&mut self, predicate: P) -> WaitUntil<'_, P, T>
    where
        P: FnMut() -> Option<T> + Unpin,
    {
        // Safety: `DiatomicWaker::register`, `DiatomicWaker::unregister` and
        // `DiatomicWaker::wait_until` cannot be used concurrently from multiple
        // thread since `WakeSinkRef` does not implement `Clone` and the
        // wrappers of the above methods require exclusive ownership to
        // `WakeSinkRef`.
        unsafe { self.inner.wait_until(predicate) }
    }
}

/// A non-owned object that can send notifications to a
/// [`WakeSinkRef`](WakeSinkRef).
///
/// See the [crate-level documentation](crate) for usage.
#[derive(Clone, Debug)]
pub struct WakeSourceRef<'a> {
    /// The shared data.
    pub(crate) inner: &'a DiatomicWaker,
}

impl WakeSourceRef<'_> {
    /// Notifies the sink if a waker is registered.
    ///
    /// This automatically unregisters any waker that may have been previously
    /// registered.
    #[inline]
    pub fn notify(&self) {
        self.inner.notify();
    }
}
