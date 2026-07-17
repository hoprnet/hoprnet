//! Runtime-agnostic multi-waker notification primitive.
//!
//! Uses a generation counter to detect notification events: [`notify_waiters`]
//! bumps the generation, and [`notified`] futures compare the generation at
//! creation time against the current one on each `poll()`. This prevents two
//! race conditions a simple waker-vector approach cannot handle:
//!
//! 1. **Latent wake.** A [`notified`] future registers its waker on first `poll()`. If [`notify_waiters`] fires between
//!    the creation of the future and that first `poll`, the waker-vector is empty and the notification is lost. With a
//!    generation counter, `gen_at_creation` already captures the pre-notification value, so the first `poll()` sees the
//!    advanced generation and returns `Ready`.
//!
//! 2. **Spurious `Ready`.** A second `poll()` of an already-registered future returned `Ready` unconditionally under
//!    the old design. Now it checks the generation again — if it did not advance, this is a spurious wake and we
//!    re-register the waker and stay `Pending`.
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
    /// Monotonic counter bumped by every [`notify_waiters`] call.
    generation: u64,
}

/// Runtime-agnostic multi-waker notification primitive.
///
/// Clone is cheap (clones the inner `Arc`).
#[derive(Clone)]
pub struct SlotNotify {
    inner: Arc<Mutex<SlotNotifyInner>>,
}

impl SlotNotify {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(SlotNotifyInner {
                wakers: Vec::new(),
                next_id: 0,
                generation: 0,
            })),
        }
    }

    /// Wake all parked waiters.
    ///
    /// Bumps the generation counter **before** draining wakers so that
    /// futures created between this call and their first `poll()` see the
    /// advanced generation and return `Ready` immediately.
    pub fn notify_waiters(&self) {
        let mut inner = self.inner.lock();
        inner.generation += 1;
        for (_, waker) in inner.wakers.drain(..) {
            waker.wake();
        }
    }

    /// Return a future that completes the next time `notify_waiters` is called.
    ///
    /// The future captures the current generation at creation time. A
    /// concurrent [`notify_waiters`] call that fires before the first
    /// `poll()` will have bumped the generation, so the future still
    /// completes — no notification is lost.
    pub fn notified(&self) -> SlotNotifyFuture {
        let generation = self.inner.lock().generation;
        SlotNotifyFuture {
            inner: self.inner.clone(),
            waker_id: 0,
            registered: false,
            gen_at_creation: generation,
        }
    }
}

/// Future returned by [`SlotNotify::notified`].
///
/// On cancellation (drop without completion), the registered waker is
/// automatically removed from [`SlotNotify`] so stale entries are never
/// left behind.
pub struct SlotNotifyFuture {
    inner: Arc<Mutex<SlotNotifyInner>>,
    waker_id: u64,
    registered: bool,
    /// Generation captured at future creation time.
    gen_at_creation: u64,
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
        let mut inner = this.inner.lock();

        // If the generation advanced past our creation snapshot, a
        // notification happened — we are done.
        if inner.generation != this.gen_at_creation {
            return Poll::Ready(());
        }

        if this.registered {
            // Already registered but generation hasn't advanced — this is a
            // spurious wake (or a replaced waker). Update the stored waker
            // and stay Pending.
            if let Some((_, w)) = inner.wakers.iter_mut().find(|(id, _)| *id == this.waker_id) {
                *w = cx.waker().clone();
            }
            return Poll::Pending;
        }

        // First poll: register.
        this.waker_id = inner.next_id;
        inner.next_id += 1;
        inner.wakers.push((this.waker_id, cx.waker().clone()));
        this.registered = true;
        Poll::Pending
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::FutureExt;

    use super::*;

    fn noop_waker() -> std::task::Waker {
        futures::task::noop_waker()
    }

    // -------------------------------------------------------------------
    // Generation-counter wake detection
    // -------------------------------------------------------------------

    #[test]
    fn notification_after_first_poll_completes_future() {
        let n = SlotNotify::new();
        let mut fut = n.notified();

        let waker = noop_waker();
        let mut cx = std::task::Context::from_waker(&waker);

        // First poll registers and returns Pending.
        assert_eq!(fut.poll_unpin(&mut cx), Poll::Pending);

        n.notify_waiters();

        // Second poll sees the advanced generation -> Ready.
        assert_eq!(fut.poll_unpin(&mut cx), Poll::Ready(()));
    }

    #[test]
    fn notify_after_creation_before_first_poll_completes() {
        let n = SlotNotify::new();
        let mut fut = n.notified();

        // Notify AFTER creation but BEFORE first poll.
        n.notify_waiters();

        let waker = noop_waker();
        let mut cx = std::task::Context::from_waker(&waker);

        // First poll should see the advanced generation -> Ready.
        assert_eq!(fut.poll_unpin(&mut cx), Poll::Ready(()));
    }

    #[test]
    fn spurious_repoll_stays_pending_without_notification() {
        let n = SlotNotify::new();
        let mut fut = n.notified();

        let waker = noop_waker();
        let mut cx = std::task::Context::from_waker(&waker);

        // First poll returns Pending.
        assert_eq!(fut.poll_unpin(&mut cx), Poll::Pending);

        // Second poll without any notify -> spurious wake, stays Pending.
        assert_eq!(fut.poll_unpin(&mut cx), Poll::Pending);
    }

    #[test]
    fn multiple_notifications_bump_generation() {
        let n = SlotNotify::new();
        let mut fut = n.notified();

        let waker = noop_waker();
        let mut cx = std::task::Context::from_waker(&waker);

        assert_eq!(fut.poll_unpin(&mut cx), Poll::Pending);

        n.notify_waiters();
        assert_eq!(fut.poll_unpin(&mut cx), Poll::Ready(()));

        // A new future should also work for the next notification cycle.
        let mut fut2 = n.notified();
        assert_eq!(fut2.poll_unpin(&mut cx), Poll::Pending);

        n.notify_waiters();
        assert_eq!(fut2.poll_unpin(&mut cx), Poll::Ready(()));
    }

    #[test]
    fn drop_uncompleted_future_removes_waker() {
        let n = SlotNotify::new();
        {
            let mut fut = n.notified();

            let waker = noop_waker();
            let mut cx = std::task::Context::from_waker(&waker);

            assert_eq!(fut.poll_unpin(&mut cx), Poll::Pending);
            // fut drops here -> waker should be removed.
        }
        // After the future is dropped, the inner waker list should be empty.
        assert!(n.inner.lock().wakers.is_empty());
    }

    // -------------------------------------------------------------------
    // Async tests (tokio runtime)
    // -------------------------------------------------------------------

    #[tokio::test]
    async fn notified_awaited_completes_on_notify() {
        let n = SlotNotify::new();
        let n2 = n.clone();

        let handle = tokio::spawn(async move {
            n2.notified().await;
            42u32
        });

        tokio::time::sleep(Duration::from_millis(10)).await;
        n.notify_waiters();

        let result = tokio::time::timeout(Duration::from_secs(1), handle)
            .await
            .expect("timeout")
            .expect("task panicked");
        assert_eq!(result, 42);
    }
}
