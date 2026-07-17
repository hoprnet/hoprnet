//! Runtime-agnostic multi-waker notification primitive.
//!
//! Uses unique IDs per waiter so that dropping a future cleanly
//! removes its waker from the vector — no waker leak on cancellation.
//! No tokio dependency — works with any async runtime.

use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
};

use parking_lot::Mutex;

struct SlotNotifyInner {
    wakers: Vec<(u64, Waker)>,
    next_id: u64,
}

/// Runtime-agnostic multi-waker notification primitive.
///
/// Clone is cheap (clones the inner `Arc`).
#[derive(Clone)]
pub(crate) struct SlotNotify {
    inner: Arc<Mutex<SlotNotifyInner>>,
}

impl SlotNotify {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(SlotNotifyInner {
                wakers: Vec::new(),
                next_id: 0,
            })),
        }
    }

    /// Wake all parked waiters.
    pub(crate) fn notify_waiters(&self) {
        for (_, waker) in self.inner.lock().wakers.drain(..) {
            waker.wake();
        }
    }

    /// Return a future that completes the next time `notify_waiters` is called.
    pub(crate) fn notified(&self) -> SlotNotifyFuture {
        SlotNotifyFuture {
            inner: self.inner.clone(),
            waker_id: 0,
            registered: false,
        }
    }
}

/// Future returned by [`SlotNotify::notified`].
///
/// On cancellation (drop without completion), the registered waker is
/// automatically removed from [`SlotNotify`] so stale entries are never
/// left behind.
pub(crate) struct SlotNotifyFuture {
    inner: Arc<Mutex<SlotNotifyInner>>,
    waker_id: u64,
    registered: bool,
}

impl Drop for SlotNotifyFuture {
    fn drop(&mut self) {
        if self.registered {
            self.inner.lock().wakers.retain(|(id, _)| *id != self.waker_id);
        }
    }
}

impl Future for SlotNotifyFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let this = self.get_mut();
        if this.registered {
            // Already registered — a wake means the waker was called.
            Poll::Ready(())
        } else {
            let mut inner = this.inner.lock();
            this.waker_id = inner.next_id;
            inner.next_id += 1;
            inner.wakers.push((this.waker_id, cx.waker().clone()));
            this.registered = true;
            Poll::Pending
        }
    }
}
