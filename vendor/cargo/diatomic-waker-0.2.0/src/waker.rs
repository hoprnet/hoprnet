use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::Ordering;
use core::task::{Context, Poll, Waker};

use crate::loom_exports::cell::UnsafeCell;
use crate::loom_exports::sync::atomic::AtomicUsize;
use crate::WakeSinkRef;

// The state of the waker is tracked by the following bit flags:
//
// * INDEX [I]: slot index of the current waker, if any (0 or 1),
// * UPDATE [U]: an updated waker has been registered in the redundant slot at
//   index 1 - INDEX,
// * REGISTERED [R]: a waker is registered and awaits a notification
// * LOCKED [L]: a notifier has taken the notifier lock and is in the process of
//   sending a notification,
// * NOTIFICATION [N]: a notifier has failed to take the lock when a waker was
//   registered and has requested the notifier holding the lock to send a
//   notification on its behalf (implies REGISTERED and LOCKED).
//
// The waker stored in the slot at INDEX ("current" waker) is shared between the
// sink (entity which registers wakers) and the source that holds the notifier
// lock (if any). For this reason, this waker may only be accessed by shared
// reference. The waker at 1 - INDEX is exclusively owned by the sink, which is
// free to mutate it.

// Summary of valid states:
//
// |  N   L   R   U   I  |
// |---------------------|
// |  0  any any any any |
// |  1   1   1  any any |

// [I] Index of the current waker (0 or 1).
const INDEX: usize = 0b00001;
// [U] Indicates that an updated waker is available at 1 - INDEX.
const UPDATE: usize = 0b00010;
// [R] Indicates that a waker has been registered.
const REGISTERED: usize = 0b00100;
// [L] Indicates that a notifier holds the notifier lock to the waker at INDEX.
const LOCKED: usize = 0b01000;
// [N] Indicates that a notifier has failed to acquire the lock and has
// requested the notifier holding the lock to notify on its behalf.
const NOTIFICATION: usize = 0b10000;

/// A primitive that can send or await notifications.
///
/// It is almost always preferable to use the [`WakeSink`](crate::WakeSink) and
/// [`WakeSource`](crate::WakeSource) which offer more convenience at the cost
/// of an allocation in an `Arc`.
///
/// If allocation is not possible or desirable, the
/// [`sink_ref`](DiatomicWaker::sink_ref) method can be used to create a
/// [`WakeSinkRef`] handle and one or more
/// [`WakeSourceRef`](crate::borrowed_waker::WakeSourceRef)s, the non-owned
/// counterparts to `WakeSink` and `WakeSource`.
///
/// Finally, `DiatomicWaker` exposes `unsafe` methods that can be used to create
/// custom synchronization primitives.
#[derive(Debug)]
pub struct DiatomicWaker {
    /// A bit field for `INDEX`, `UPDATE`, `REGISTERED`, `LOCKED` and `NOTIFICATION`.
    state: AtomicUsize,
    /// Redundant slots for the waker.
    waker: [UnsafeCell<Option<Waker>>; 2],
}

impl DiatomicWaker {
    /// Creates a new `DiatomicWaker`.
    #[cfg(not(diatomic_waker_loom))]
    pub const fn new() -> Self {
        Self {
            state: AtomicUsize::new(0),
            waker: [UnsafeCell::new(None), UnsafeCell::new(None)],
        }
    }

    #[cfg(diatomic_waker_loom)]
    pub fn new() -> Self {
        Self {
            state: AtomicUsize::new(0),
            waker: [UnsafeCell::new(None), UnsafeCell::new(None)],
        }
    }

    /// Returns a sink with a lifetime bound to this `DiatomicWaker`.
    ///
    /// This mutably borrows the waker, thus ensuring that at most one
    /// associated sink can be active at a time.
    pub fn sink_ref(&mut self) -> WakeSinkRef<'_> {
        WakeSinkRef { inner: self }
    }

    /// Sends a notification if a waker is registered.
    ///
    /// This automatically unregisters any waker that may have been previously
    /// registered.
    pub fn notify(&self) {
        // Transitions: see `try_lock` and `try_unlock`.

        let mut state = if let Ok(s) = try_lock(&self.state) {
            s
        } else {
            return;
        };

        loop {
            let idx = state & INDEX;

            // Safety: the notifier lock has been acquired, which guarantees
            // exclusive access to the waker at `INDEX`.
            unsafe {
                self.wake_by_ref(idx);
            }

            if let Err(s) = try_unlock(&self.state, state) {
                state = s;
            } else {
                return;
            }

            // One more loop iteration is necessary because the waker was
            // registered again and another notifier has failed to send a
            // notification while the notifier lock was taken.
        }
    }

    /// Registers a new waker.
    ///
    /// Registration is lazy: the waker is cloned only if it differs from the
    /// last registered waker (note that the last registered waker is cached
    /// even if it was unregistered).
    ///
    /// # Safety
    ///
    /// The `register`, `unregister` and `wait_until` methods cannot be used
    /// concurrently from multiple threads.
    pub unsafe fn register(&self, waker: &Waker) {
        // Transitions if the new waker is the same as the one currently stored.
        //
        // |  N  L  R  U  I  |  N  L  R  U  I  |
        // |-----------------|-----------------|
        // |  n  l  r  u  i  |  n  l  1  u  i  |
        //
        //
        // Transitions if the new waker needs to be stored:
        //
        // Step 1 (only necessary if the state initially indicates R=U=1):
        //
        // |  N  L  R  U  I  |  N  L  R  U  I  |
        // |-----------------|-----------------|
        // |  n  l  r  u  i  |  0  l  0  u  i  |
        //
        // Step 2:
        //
        // |  N  L  R  U  I  |  N  L  R  U  I  |
        // |-----------------|-----------------|
        // |  n  l  r  u  i  |  n  l  1  1  i  |

        // Ordering: Acquire ordering is necessary to synchronize with the
        // Release unlocking operation in `notify`, which ensures that all calls
        // to the waker in the redundant slot have completed.
        let state = self.state.load(Ordering::Acquire);

        // Compute the index of the waker that was most recently updated. Note
        // that the value of `recent_idx` as computed below remains correct even
        // if the state is stale since only this thread can store new wakers.
        let mut idx = state & INDEX;
        let recent_idx = if state & UPDATE == 0 {
            idx
        } else {
            INDEX - idx
        };

        // Safety: it is safe to call `will_wake` since the registering thread
        // is the only one allowed to mutate the wakers so there can be no
        // concurrent mutable access to the waker.
        let is_up_to_date = self.will_wake(recent_idx, waker);

        // Fast path in case the waker is up to date.
        if is_up_to_date {
            // Set the `REGISTERED` flag. Ideally, the `NOTIFICATION` flag would
            // be cleared at the same time to avoid a spurious wake-up, but it
            // probably isn't worth the overhead of a CAS loop because having
            // this flag set when calling `register` is very unlikely: it would
            // mean that since the last call to `register`:
            // 1) a notifier has been holding the lock continuously,
            // 2) another notifier has tried and failed to take the lock, and
            // 3) `unregister` was never called.
            //
            // Ordering: Acquire ordering synchronizes with the Release and
            // AcqRel RMWs in `try_lock` (called by `notify`) and ensures that
            // either the predicate set before the call to `notify` will be
            // visible after the call to `register`, or the registered waker
            // will be visible during the call to `notify` (or both). Note that
            // Release ordering is not necessary since the waker has not changed
            // and this RMW takes part in a release sequence headed by the
            // initial registration of the waker.
            self.state.fetch_or(REGISTERED, Ordering::Acquire);

            return;
        }

        // The waker needs to be stored in the redundant slot.
        //
        // It is necessary to make sure that either the `UPDATE` or the
        // `REGISTERED` flag is cleared to prevent concurrent access by a notifier
        // to the redundant waker slot while the waker is updated.
        //
        // Note that only the thread registering the waker can set `REGISTERED`
        // and `UPDATE` so even if the state is stale, observing `REGISTERED` or
        // `UPDATE` as cleared guarantees that such flag is and will remain
        // cleared until this thread sets them.
        if state & (UPDATE | REGISTERED) == (UPDATE | REGISTERED) {
            // Clear the `REGISTERED` and `NOTIFICATION` flags.
            //
            // Ordering: Acquire ordering is necessary to synchronize with the
            // Release unlocking operation in `notify`, which ensures that all
            // calls to the waker in the redundant slot have completed.
            let state = self
                .state
                .fetch_and(!(REGISTERED | NOTIFICATION), Ordering::Acquire);

            // It is possible that `UPDATE` was cleared and `INDEX` was switched
            // by a notifier after the initial load of the state, so the waker
            // index needs to be updated.
            idx = state & INDEX;
        }

        // Always store the new waker in the redundant slot to avoid racing with
        // a notifier.
        let redundant_idx = 1 - idx;

        // Store the new waker.
        //
        // Safety: it is safe to store the new waker in the redundant slot
        // because the `REGISTERED` flag and/or the `UPDATE` flag are/is cleared
        // so the notifier will not attempt to switch the waker.
        self.set_waker(redundant_idx, waker.clone());

        // Make the waker visible.
        //
        // Ordering: Acquire ordering synchronizes with the Release and AcqRel
        // RMWs in `try_lock` (called by `notify`) and ensures that either the
        // predicate set before the call to `notify` will be visible after the
        // call to `register`, or the registered waker will be visible during
        // the call to `notify` (or both). Since the waker has been modified
        // above, Release ordering is also necessary to synchronize with the
        // AcqRel RMW in `try_lock` (success case) and ensure that the
        // modification to the waker is fully visible when notifying.
        self.state.fetch_or(UPDATE | REGISTERED, Ordering::AcqRel);
    }

    /// Unregisters the waker.
    ///
    /// After the waker is unregistered, subsequent calls to `notify` will be
    /// ignored.
    ///
    /// Note that the previously-registered waker (if any) remains cached.
    ///
    /// # Safety
    ///
    /// The `register`, `unregister` and `wait_until` methods cannot be used
    /// concurrently from multiple threads.
    pub unsafe fn unregister(&self) {
        // Transitions:
        //
        // |  N  L  R  U  I  |  N  L  R  U  I  |
        // |-----------------|-----------------|
        // |  n  l  r  u  i  |  0  l  0  u  i  |

        // Modify the state. Note that the waker is not dropped: caching it can
        // avoid a waker drop/cloning cycle (typically, 2 RMWs) in the frequent
        // case when the next waker to be registered will be the same as the one
        // being unregistered.
        //
        // Ordering: no waker was modified so Relaxed ordering is sufficient.
        self.state
            .fetch_and(!(REGISTERED | NOTIFICATION), Ordering::Relaxed);
    }

    /// Returns a future that can be `await`ed until the provided predicate
    /// returns a value.
    ///
    /// The predicate is checked each time a notification is received.
    ///
    /// # Safety
    ///
    /// The `register`, `unregister` and `wait_until` methods cannot be used
    /// concurrently from multiple threads.
    pub unsafe fn wait_until<P, T>(&self, predicate: P) -> WaitUntil<'_, P, T>
    where
        P: FnMut() -> Option<T>,
    {
        WaitUntil::new(self, predicate)
    }

    /// Sets the waker at index `idx`.
    ///
    /// # Safety
    ///
    /// The caller must have exclusive access to the waker at index `idx`.
    unsafe fn set_waker(&self, idx: usize, new: Waker) {
        self.waker[idx].with_mut(|waker| (*waker) = Some(new));
    }

    /// Notify the waker at index `idx`.
    ///
    /// # Safety
    ///
    /// The waker at index `idx` cannot be modified concurrently.
    unsafe fn wake_by_ref(&self, idx: usize) {
        self.waker[idx].with(|waker| {
            if let Some(waker) = &*waker {
                waker.wake_by_ref();
            }
        });
    }

    /// Check whether the waker at index `idx` will wake the same task as the
    /// provided waker.
    ///
    /// # Safety
    ///
    /// The waker at index `idx` cannot be modified concurrently.
    unsafe fn will_wake(&self, idx: usize, other: &Waker) -> bool {
        self.waker[idx].with(|waker| match &*waker {
            Some(waker) => waker.will_wake(other),
            None => false,
        })
    }
}

impl Default for DiatomicWaker {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Send for DiatomicWaker {}
unsafe impl Sync for DiatomicWaker {}

/// Attempts to acquire the notifier lock and returns the current state upon
/// success.
///
/// Acquisition of the lock will fail in the following cases:
///
/// * the `REGISTERED` flag is cleared, meaning that there is no need to wake
///   and therefore no need to lock,
/// * the lock is already taken, in which case the `NOTIFICATION` flag will be
///   set if the `REGISTERED` flag is set.
///
/// If acquisition of the lock succeeds, the `REGISTERED` flag is cleared. If
/// additionally the `UPDATE` flag was set, it is cleared and `INDEX` is
/// flipped.
///
///  Transition table:
///
/// |  N  L  R  U  I  |  N  L  R  U  I  |
/// |-----------------|-----------------|
/// |  0  0  0  u  i  |  0  0  0  u  i  | (failure)
/// |  0  0  1  0  i  |  0  1  0  0  i  | (success)
/// |  0  0  1  1  i  |  0  1  0  0 !i  | (success)
/// |  0  1  0  u  i  |  0  1  0  u  i  | (failure)
/// |  n  1  1  u  i  |  1  1  1  u  i  | (failure)
///
fn try_lock(state: &AtomicUsize) -> Result<usize, ()> {
    let mut old_state = state.load(Ordering::Relaxed);

    loop {
        if old_state & (LOCKED | REGISTERED) == REGISTERED {
            // Success path.

            // If `UPDATE` is set, clear `UPDATE` and flip `INDEX` with the xor
            // mask.
            let update_bit = old_state & UPDATE;
            let xor_mask = update_bit | (update_bit >> 1);

            // Set `LOCKED` and clear `REGISTERED` with the xor mask.
            let xor_mask = xor_mask | LOCKED | REGISTERED;

            let new_state = old_state ^ xor_mask;

            // Ordering: Acquire is necessary to synchronize with the Release
            // ordering in `register` so that the new waker, if any, is visible.
            // Release ordering synchronizes with the Acquire and AcqRel RMWs in
            // `register` and ensures that either the predicate set before the
            // call to `notify` will be visible after the call to `register`, or
            // the registered waker will be visible during the call to `notify`
            // (or both).
            match state.compare_exchange_weak(
                old_state,
                new_state,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(new_state),
                Err(s) => old_state = s,
            }
        } else {
            // Failure path.

            // Set the `NOTIFICATION` bit if `REGISTERED` was set.
            let registered_bit = old_state & REGISTERED;
            let new_state = old_state | (registered_bit << 2);

            // Ordering: Release ordering synchronizes with the Acquire and
            // AcqRel RMWs in `register` and ensures that either the predicate
            // set before the call to `notify` will be visible after the call to
            // `register`, or the registered waker will be visible during the
            // call to `notify` (or both).
            match state.compare_exchange_weak(
                old_state,
                new_state,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Err(()),
                Err(s) => old_state = s,
            }
        };
    }
}

/// Attempts to release the notifier lock and returns the current state upon
/// failure.
///
/// Release of the lock will fail if the `NOTIFICATION` flag is set because it
/// means that, after the lock was taken, the registering thread has requested
/// to be notified again and another notifier has subsequently requested that
/// such notification be sent on its behalf; if additionally the `UPDATE` flag
/// was set (i.e. a new waker is available), it is cleared and `INDEX` is
/// flipped.
///
/// Transition table:
///
/// |  N  L  R  U  I  |  N  L  R  U  I  |
/// |-----------------|-----------------|
/// |  0  1  r  u  i  |  0  0  r  u  i  | (success)
/// |  1  1  1  0  i  |  0  1  0  0  i  | (failure)
/// |  1  1  1  1  i  |  0  1  0  0 !i  | (failure)
///
fn try_unlock(state: &AtomicUsize, mut old_state: usize) -> Result<(), usize> {
    loop {
        if old_state & NOTIFICATION == 0 {
            // Success path.

            let new_state = old_state & !LOCKED;

            // Ordering: Release is necessary to synchronize with the Acquire
            // ordering in `register` and ensure that the waker call has
            // completed before a new waker is stored.
            match state.compare_exchange_weak(
                old_state,
                new_state,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(()),
                Err(s) => old_state = s,
            }
        } else {
            // Failure path.

            // If `UPDATE` is set, clear `UPDATE` and flip `INDEX` with the xor mask.
            let update_bit = old_state & UPDATE;
            let xor_mask = update_bit | (update_bit >> 1);

            // Clear `NOTIFICATION` and `REGISTERED` with the xor mask.
            let xor_mask = xor_mask | NOTIFICATION | REGISTERED;

            let new_state = old_state ^ xor_mask;

            // Ordering: Release is necessary to synchronize with the Acquire
            // ordering in `register` and ensure that the call to
            // `Waker::wake_by_ref` has completed before a new waker is stored.
            // Acquire ordering is in turn necessary to ensure that any newly
            // registered waker is visible.
            match state.compare_exchange_weak(
                old_state,
                new_state,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Err(new_state),
                Err(s) => old_state = s,
            }
        };
    }
}

/// A future that can be `await`ed until a predicate is satisfied.
#[derive(Debug)]
pub struct WaitUntil<'a, P, T>
where
    P: FnMut() -> Option<T>,
{
    predicate: P,
    wake: &'a DiatomicWaker,
}

impl<'a, P, T> WaitUntil<'a, P, T>
where
    P: FnMut() -> Option<T>,
{
    /// Creates a future associated to the specified wake that can be `await`ed
    /// until the specified predicate is satisfied.
    fn new(wake: &'a DiatomicWaker, predicate: P) -> Self {
        Self { predicate, wake }
    }
}

impl<P: FnMut() -> Option<T>, T> Unpin for WaitUntil<'_, P, T> {}

impl<'a, P, T> Future for WaitUntil<'a, P, T>
where
    P: FnMut() -> Option<T>,
{
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        // Safety: the safety of this method is contingent on the safety of the
        // `register` and `unregister` methods. Since a `WaitUntil` future can
        // only be created from the unsafe `wait_until` method, however, the
        // user must uphold the contract that `register`, `unregister` and
        // `wait_until` cannot be used concurrently from multiple threads.
        unsafe {
            if let Some(value) = (self.predicate)() {
                return Poll::Ready(value);
            }
            self.wake.register(cx.waker());

            if let Some(value) = (self.predicate)() {
                self.wake.unregister();
                return Poll::Ready(value);
            }
        }

        Poll::Pending
    }
}
