//! Async, fast synchronization primitives for task wakeup.
//!
//! `diatomic-waker` is similar to [`atomic-waker`][atomic-waker] in that it
//! enables concurrent updates and notifications to a wrapped `Waker`. Unlike
//! the latter, however, it does not use spinlocks[^spinlocks] and is faster, in
//! particular when the consumer is notified periodically rather than just once.
//! It can in particular be used as a very fast, single-consumer [eventcount] to
//! turn a non-blocking data structure into an asynchronous one (see MPSC
//! channel receiver example).
//!
//! The API distinguishes between the entity that registers wakers ([`WakeSink`]
//! or [`WakeSinkRef`]) and the possibly many entities that notify the waker
//! ([`WakeSource`]s or [`WakeSourceRef`]s).
//!
//! Most users will prefer to use [`WakeSink`] and [`WakeSource`], which readily
//! store a shared [`DiatomicWaker`] within an `Arc`. You may otherwise elect to
//! allocate a [`DiatomicWaker`] yourself, but will then need to use the
//! lifetime-bounded [`WakeSinkRef`] and [`WakeSourceRef`], or ensure by other
//! means that waker registration is not performed concurrently.
//!
//! [atomic-waker]: https://docs.rs/atomic-waker/latest/atomic_waker/
//! [eventcount]:
//!     https://www.1024cores.net/home/lock-free-algorithms/eventcounts
//! [^spinlocks]: The implementation of [AtomicWaker][atomic-waker] yields to the
//!     runtime on contention, which is in effect an executor-mediated spinlock.
//!
//! # Features flags
//!
//! By default, this crate enables the `alloc` feature to provide the owned
//! [`WakeSink`] and [`WakeSource`]. It can be made `no-std`-compatible by
//! specifying `default-features = false`.
//!
//!
//! # Examples
//!
//! A multi-producer, single-consumer channel of capacity 1 for sending
//! `NonZeroUsize` values, with an asynchronous receiver:
//!
//! ```
//! use std::num::NonZeroUsize;
//! use std::sync::atomic::{AtomicUsize, Ordering};
//! use std::sync::Arc;
//!
//! use diatomic_waker::{WakeSink, WakeSource};
//!
//! // The sending side of the channel.
//! #[derive(Clone)]
//! struct Sender {
//!     wake_src: WakeSource,
//!     value: Arc<AtomicUsize>,
//! }
//!
//! // The receiving side of the channel.
//! struct Receiver {
//!     wake_sink: WakeSink,
//!     value: Arc<AtomicUsize>,
//! }
//!
//! // Creates an empty channel.
//! fn channel() -> (Sender, Receiver) {
//!     let value = Arc::new(AtomicUsize::new(0));
//!     let wake_sink = WakeSink::new();
//!     let wake_src = wake_sink.source();
//!
//!     (
//!         Sender {
//!             wake_src,
//!             value: value.clone(),
//!         },
//!         Receiver { wake_sink, value },
//!     )
//! }
//!
//! impl Sender {
//!     // Sends a value if the channel is empty.
//!     fn try_send(&self, value: NonZeroUsize) -> bool {
//!         let success = self
//!             .value
//!             .compare_exchange(0, value.get(), Ordering::Relaxed, Ordering::Relaxed)
//!             .is_ok();
//!         if success {
//!             self.wake_src.notify()
//!         };
//!
//!         success
//!     }
//! }
//!
//! impl Receiver {
//!     // Receives a value asynchronously.
//!     async fn recv(&mut self) -> NonZeroUsize {
//!         // Wait until the predicate returns `Some(value)`, i.e. when the atomic
//!         // value becomes non-zero.
//!         self.wake_sink
//!             .wait_until(|| NonZeroUsize::new(self.value.swap(0, Ordering::Relaxed)))
//!             .await
//!     }
//! }
//! ```
//!
//!
//! In some case, it may be necessary to use the lower-level
//! [`register`](WakeSink::register) and [`unregister`](WakeSink::unregister)
//! methods rather than the [`wait_until`](WakeSink::wait_until) convenience
//! method.
//!
//! This is how the behavior of the above `recv` method could be
//! reproduced with a hand-coded future:
//!
//! ```
//! use std::future::Future;
//! # use std::num::NonZeroUsize;
//! use std::pin::Pin;
//! # use std::sync::atomic::{AtomicUsize, Ordering};
//! # use std::sync::Arc;
//! use std::task::{Context, Poll};
//! # use diatomic_waker::WakeSink;
//!
//! # struct Receiver {
//! #     wake_sink: WakeSink,
//! #     value: Arc<AtomicUsize>,
//! # }
//! struct Recv<'a> {
//!     receiver: &'a mut Receiver,
//! }
//!
//! impl Future for Recv<'_> {
//!     type Output = NonZeroUsize;
//!
//!     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<NonZeroUsize> {
//!         // Avoid waker registration if a value is readily available.
//!         let value = NonZeroUsize::new(self.receiver.value.swap(0, Ordering::Relaxed));
//!         if let Some(value) = value {
//!             return Poll::Ready(value);
//!         }
//!
//!         // Register the waker to be polled again once a value is available.
//!         self.receiver.wake_sink.register(cx.waker());
//!
//!         // Check again after registering the waker to prevent a race condition.
//!         let value = NonZeroUsize::new(self.receiver.value.swap(0, Ordering::Relaxed));
//!         if let Some(value) = value {
//!             // Avoid a spurious wake-up.
//!             self.receiver.wake_sink.unregister();
//!
//!             return Poll::Ready(value);
//!         }
//!
//!         Poll::Pending
//!     }
//! }
//! ```
#![warn(missing_docs, missing_debug_implementations, unreachable_pub)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg_hide))]
#![cfg_attr(docsrs, doc(cfg_hide(diatomic_waker_loom)))]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
mod arc_waker;
mod borrowed_waker;
mod loom_exports;
#[deprecated(
    since = "0.2.0",
    note = "items from this module are now available in the root module"
)]
pub mod primitives;
mod waker;

#[cfg(feature = "alloc")]
pub use arc_waker::{WakeSink, WakeSource};
pub use borrowed_waker::{WakeSinkRef, WakeSourceRef};
pub use waker::{DiatomicWaker, WaitUntil};

/// Tests.
#[cfg(all(test, not(diatomic_waker_loom)))]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;
    use std::time::Duration;

    use pollster::block_on;

    use super::*;

    #[test]
    fn waker_wait_until() {
        let mut sink = WakeSink::new();
        let source = sink.source();
        static FLAG: AtomicBool = AtomicBool::new(false);

        let t1 = thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(10));
            source.notify(); // force a spurious notification
            std::thread::sleep(Duration::from_millis(10));
            FLAG.store(true, Ordering::Relaxed);
            source.notify();
        });

        let t2 = thread::spawn(move || {
            block_on(sink.wait_until(|| {
                if FLAG.load(Ordering::Relaxed) {
                    Some(())
                } else {
                    None
                }
            }));

            assert!(FLAG.load(Ordering::Relaxed));
        });

        t1.join().unwrap();
        t2.join().unwrap();
    }

    #[test]
    fn waker_ref_wait_until() {
        let mut w = DiatomicWaker::new();
        let mut sink = w.sink_ref();
        let source = sink.source_ref();
        static FLAG: AtomicBool = AtomicBool::new(false);

        thread::scope(|s| {
            s.spawn(move || {
                std::thread::sleep(Duration::from_millis(10));
                source.notify(); // force a spurious notification
                std::thread::sleep(Duration::from_millis(10));
                FLAG.store(true, Ordering::Relaxed);
                source.notify();
            });

            s.spawn(move || {
                block_on(sink.wait_until(|| {
                    if FLAG.load(Ordering::Relaxed) {
                        Some(())
                    } else {
                        None
                    }
                }));

                assert!(FLAG.load(Ordering::Relaxed));
            });
        });
    }
}

/// Loom tests.
#[cfg(all(test, diatomic_waker_loom))]
mod tests {
    use super::*;

    use core::task::Waker;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;
    use std::task::{Context, Poll};

    use loom::model::Builder;
    use loom::sync::atomic::{AtomicU32, AtomicUsize};
    use loom::thread;

    use waker_fn::waker_fn;

    /// A waker factory that registers notifications from the newest waker only.
    #[derive(Clone, Default)]
    struct MultiWaker {
        state: Arc<AtomicU32>,
    }
    impl MultiWaker {
        /// Clears the notification flag and returns the former notification
        /// status.
        ///
        /// This operation has Acquire semantic when a notification is indeed
        /// present, and Relaxed otherwise. It is therefore appropriate to
        /// simulate a scheduler receiving a notification as it ensures that all
        /// memory operations preceding the notification of a task are visible.
        fn take_notification(&self) -> bool {
            // Clear the notification flag.
            let mut state = self.state.load(Ordering::Relaxed);
            loop {
                // This is basically a `fetch_or` but with an atomic memory
                // ordering that depends on the LSB.
                let notified_stated = state | 1;
                let unnotified_stated = state & !1;
                match self.state.compare_exchange_weak(
                    notified_stated,
                    unnotified_stated,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return true,
                    Err(s) => {
                        state = s;
                        if state == unnotified_stated {
                            return false;
                        }
                    }
                }
            }
        }

        /// Clears the notification flag and creates a new waker.
        fn new_waker(&self) -> Waker {
            // Increase the epoch and clear the notification flag.
            let mut state = self.state.load(Ordering::Relaxed);
            let mut epoch;
            loop {
                // Increase the epoch by 2.
                epoch = (state & !1) + 2;
                match self.state.compare_exchange_weak(
                    state,
                    epoch,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(s) => state = s,
                }
            }

            // Create a waker that only notifies if it is the newest waker.
            let waker_state = self.state.clone();
            waker_fn(move || {
                let mut state = waker_state.load(Ordering::Relaxed);
                loop {
                    let new_state = if state & !1 == epoch {
                        epoch | 1
                    } else {
                        break;
                    };
                    match waker_state.compare_exchange(
                        state,
                        new_state,
                        Ordering::Release,
                        Ordering::Relaxed,
                    ) {
                        Ok(_) => break,
                        Err(s) => state = s,
                    }
                }
            })
        }
    }

    // A simple counter that can be used to simulate the availability of a
    // certain number of tokens. In order to model the weakest possible
    // predicate from the viewpoint of atomic memory ordering, only Relaxed
    // atomic operations are used.
    #[derive(Clone, Default)]
    struct Counter {
        count: Arc<AtomicUsize>,
    }
    impl Counter {
        fn increment(&self) {
            self.count.fetch_add(1, Ordering::Relaxed);
        }
        fn try_decrement(&self) -> bool {
            let mut count = self.count.load(Ordering::Relaxed);
            loop {
                if count == 0 {
                    return false;
                }
                match self.count.compare_exchange(
                    count,
                    count - 1,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return true,
                    Err(c) => count = c,
                }
            }
        }
    }

    /// Test whether notifications may be lost.
    ///
    /// Make a certain amount of tokens available and notify the sink each time
    /// a token is made available. Optionally, it is possible to:
    /// - request that `max_spurious_wake` threads will simulate a spurious
    ///   wake-up,
    /// - change the waker each time it is polled.
    ///
    /// A default preemption bound will be applied if none was specified through
    /// an environment variable.
    fn loom_notify(
        token_count: usize,
        max_spurious_wake: usize,
        change_waker: bool,
        preemption_bound: usize,
    ) {
        // Only set the preemption bound if it wasn't already specified via a environment variable.
        let mut builder = Builder::new();
        if builder.preemption_bound.is_none() {
            builder.preemption_bound = Some(preemption_bound);
        }

        builder.check(move || {
            let token_counter = Counter::default();
            let mut wake_sink = WakeSink::new();

            for src_id in 0..token_count {
                thread::spawn({
                    let token_counter = token_counter.clone();
                    let wake_src = wake_sink.source();

                    move || {
                        if src_id < max_spurious_wake {
                            wake_src.notify();
                        }
                        token_counter.increment();
                        wake_src.notify();
                    }
                });
            }

            let multi_waker = MultiWaker::default();
            let mut waker = multi_waker.new_waker();
            let mut satisfied_predicates_count = 0;

            // Iterate until all tokens are "received".
            //
            // Note: the loop does not have any assertion. This is by design:
            // missed notifications will be caught by Loom with a `Model
            // exceeded maximum number of branches` error because the spin loop
            // will then spin forever.
            while satisfied_predicates_count < token_count {
                let mut wait_until = wake_sink.wait_until(|| {
                    if token_counter.try_decrement() {
                        Some(())
                    } else {
                        None
                    }
                });

                // Poll the predicate until it is satisfied.
                loop {
                    let mut cx = Context::from_waker(&waker);
                    let poll_state = Pin::new(&mut wait_until).poll(&mut cx);

                    if poll_state == Poll::Ready(()) {
                        satisfied_predicates_count += 1;
                        break;
                    }

                    // Simulate the scheduler by spinning until the next
                    // notification.
                    while !multi_waker.take_notification() {
                        thread::yield_now();
                    }

                    if change_waker {
                        waker = multi_waker.new_waker();
                    }
                }
            }
        });
    }

    #[test]
    fn loom_notify_two_tokens() {
        const DEFAULT_PREEMPTION_BOUND: usize = 4;

        loom_notify(2, 0, false, DEFAULT_PREEMPTION_BOUND);
    }

    #[test]
    fn loom_notify_two_tokens_one_spurious() {
        const DEFAULT_PREEMPTION_BOUND: usize = 4;

        loom_notify(2, 1, false, DEFAULT_PREEMPTION_BOUND);
    }

    #[test]
    fn loom_notify_two_tokens_change_waker() {
        const DEFAULT_PREEMPTION_BOUND: usize = 3;

        loom_notify(2, 0, true, DEFAULT_PREEMPTION_BOUND);
    }

    #[test]
    fn loom_notify_two_tokens_one_spurious_change_waker() {
        const DEFAULT_PREEMPTION_BOUND: usize = 3;

        loom_notify(2, 1, true, DEFAULT_PREEMPTION_BOUND);
    }

    #[test]
    fn loom_notify_three_tokens() {
        const DEFAULT_PREEMPTION_BOUND: usize = 2;

        loom_notify(3, 0, false, DEFAULT_PREEMPTION_BOUND);
    }

    #[test]
    /// Test whether concurrent read and write access to the waker is possible.
    ///
    /// 3 different wakers are registered to force a waker slot to be re-used.
    fn loom_waker_slot_reuse() {
        // This tests require a high preemption bound to catch typical atomic
        // memory ordering mistakes.
        const DEFAULT_PREEMPTION_BOUND: usize = 5;

        // Only set the preemption bound if it wasn't already specified via a
        // environment variable.
        let mut builder = Builder::new();
        if builder.preemption_bound.is_none() {
            builder.preemption_bound = Some(DEFAULT_PREEMPTION_BOUND);
        }

        builder.check(move || {
            let mut wake_sink = WakeSink::new();

            thread::spawn({
                let wake_src = wake_sink.source();

                move || {
                    wake_src.notify();
                }
            });
            thread::spawn({
                let wake_src = wake_sink.source();

                move || {
                    wake_src.notify();
                    wake_src.notify();
                }
            });

            let multi_waker = MultiWaker::default();
            for _ in 0..3 {
                let waker = multi_waker.new_waker();
                wake_sink.register(&waker);
            }
        });
    }
}
