use std::{
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

use futures::{FutureExt, SinkExt, StreamExt};
use hopr_api::types::internal::routing::DestinationRouting;
use hopr_protocol_app::prelude::{ApplicationData, ApplicationDataOut};
use hopr_protocol_start::{KeepAliveFlag, KeepAliveMessage};
/// Bidirectionally copies data between a [`HoprSession`](crate::HoprSession) and an arbitrary
/// async IO stream.
///
/// Re-exported from `hopr-utils`; the published generic version accepts any two
/// `AsyncRead + AsyncWrite` streams. Only available with Tokio.
#[cfg(feature = "runtime-tokio")]
pub use hopr_utils::network_types::utils::{transfer_session, transfer_session_datagram};
use hopr_utils::runtime::AbortHandle;
use tracing::{Instrument, debug, instrument, warn};

use crate::{
    AtomicSurbFlowEstimator, SessionId,
    balancer::{BalancerStateValues, RateController, RateLimitStreamExt},
    errors::TransportSessionError,
    types::HoprStartProtocol,
};

/// Runtime-agnostic multi-waker notification primitive.
///
/// Uses a generation counter to detect notification events: [`notify_waiters`](SlotNotify::notify_waiters)
/// bumps the generation, and [`notified`](SlotNotify::notified) futures compare the generation at
/// creation time against the current one on each `poll()`. This prevents two
/// race conditions a simple waker-vector approach cannot handle:
///
/// 1. **Latent wake.** A [`notified`](SlotNotify::notified) future registers its waker on first `poll()`. If
///    [`notify_waiters`](SlotNotify::notify_waiters) fires between the creation of the future and that first `poll`,
///    the waker-vector is empty and the notification is lost. With a generation counter, `gen_at_creation` already
///    captures the pre-notification value, so the first `poll()` sees the advanced generation and returns `Ready`.
///
/// 2. **Spurious `Ready`.** A second `poll()` of an already-registered future can return `Ready` unconditionally if the
///    notification check is only done on registration. Generation re-check on every `poll()` prevents this.
///
/// Uses unique IDs per waiter so that dropping a future cleanly
/// removes its waker from the vector — no waker leak on cancellation.
/// No tokio dependency — works with any async runtime.
///
/// Clone is cheap (clones the inner `Arc`).
#[derive(Clone)]
pub struct SlotNotify {
    inner: Arc<parking_lot::Mutex<SlotNotifyInner>>,
}

struct SlotNotifyInner {
    wakers: Vec<(u64, Waker)>,
    next_id: u64,
    generation: u64,
}

impl SlotNotify {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(parking_lot::Mutex::new(SlotNotifyInner {
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
    ///
    /// The guard is released before any waker runs. `Waker::wake` executes arbitrary executor code,
    /// and an executor that polls the woken task inline re-enters [`SlotNotifyFuture::poll`] — or its
    /// `Drop` — both of which lock this same non-reentrant mutex. Waking under the guard is therefore
    /// a self-deadlock against exactly the runtimes this type advertises support for; tokio only
    /// enqueues, so the shipped path never hit it. Draining into a local buffer costs one allocation
    /// per notification that actually has waiters to wake, and nothing at all when it does not.
    pub fn notify_waiters(&self) {
        let woken = {
            let mut inner = self.inner.lock();
            inner.generation += 1;
            // `drain` rather than `mem::take`: an empty drain allocates nothing and the inner
            // `Vec` keeps its capacity, which is the common case on a gate that rarely parks.
            inner.wakers.drain(..).collect::<Vec<_>>()
        };
        for (_, waker) in woken {
            waker.wake();
        }
    }

    /// Return a future that completes the next time `notify_waiters` is called.
    ///
    /// The future captures the current generation at creation time. A
    /// concurrent [`notify_waiters`](SlotNotify::notify_waiters) call that fires before the first
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
    inner: Arc<parking_lot::Mutex<SlotNotifyInner>>,
    waker_id: u64,
    registered: bool,
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
                // `will_wake` first: a re-poll of the same task is the common case here — the gate
                // parks callers that are then polled spuriously — and the clone costs a refcount
                // bump, or an allocation for a waker that is not `Arc`-backed. Replacing only when
                // the target actually changed keeps the correctness property that matters, which is
                // that the *latest* waker is the one stored: `will_wake` returning true means
                // waking the stored one wakes the same task.
                if !w.will_wake(cx.waker()) {
                    *w = cx.waker().clone();
                }
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

/// This function will use the given generator to generate an initial seeding key.
/// It will check whether the given cache already contains a value for that key, and if not,
/// calls the generator (with the previous value) to generate a new seeding key and retry.
/// The function either finds a suitable free slot, inserting value generated by `value_fn` and returns the found key,
/// or terminates with `None` when `gen` returns the initial seed again.
pub(crate) fn insert_into_next_slot<F, K, U, V>(
    cache: &moka::sync::Cache<K, V>,
    mut generator: F,
    value_fn: U,
    max_capacity: Option<u64>,
) -> Option<(K, V)>
where
    F: FnMut(Option<K>) -> K,
    K: Copy + std::hash::Hash + Eq + Send + Sync + 'static,
    U: FnOnce(K) -> V,
    V: Clone + Send + Sync + 'static,
{
    cache.run_pending_tasks();

    // Reject when the cache is already at capacity to avoid Moka evicting an
    // existing entry before we can insert the new one.
    if let Some(max) = max_capacity
        && cache.entry_count() >= max
    {
        return None;
    }

    // Wrap the FnOnce so we can "consume" it exactly once,
    // but only when we actually insert into a free slot.
    let value_fn = std::sync::Arc::new(parking_lot::Mutex::new(Some(value_fn)));

    let initial = generator(None);
    let mut next = initial;
    loop {
        let value_fn = value_fn.clone();
        let insertion_result = cache.entry(next).and_compute_with(move |e| {
            if e.is_none() {
                let f = value_fn
                    .lock()
                    .take()
                    .expect("impossible: value_fn was already consumed");

                moka::ops::compute::Op::Put(f(next))
            } else {
                moka::ops::compute::Op::Nop
            }
        });

        // If we inserted successfully, break the loop and return the insertion key
        if let moka::ops::compute::CompResult::Inserted(val) = insertion_result {
            return Some((next, val.into_value()));
        }

        // Otherwise, generate the next key
        next = generator(Some(next));

        // If generated keys made it to full loop, return failure
        if next == initial {
            return None;
        }
    }
}

/// Indicates whether the [keep-alive stream](spawn_keep_alive_stream) should notify the Session counterparty
/// about the SURB target (Entry) or SURB level (Exit).
#[derive(Debug, Clone)]
pub(crate) enum SurbNotificationMode {
    /// No keep-alive messages are sent to the Session counterparty.
    DoNotNotify,
    /// Session initiator notifies the Session recipient about the desired SURB target level.
    Target,
    /// Session recipient notifies the Session initiator about the current SURB level.
    Level(AtomicSurbFlowEstimator),
}

/// How often a stream reports keep-alives it could not send: at most once per this.
const KEEP_ALIVE_FAILURE_REPORT_PERIOD: Duration = Duration::from_secs(1);

/// Rate limit for the report of keep-alives that could not be sent, which at a keep-alive rate of
/// hundreds a second would otherwise be a line per packet for as long as the SURBs stay out.
#[derive(Debug)]
struct KeepAliveFailureReport {
    epoch: Instant,
    /// Milliseconds since `epoch` of the last report, plus one so that `0` is "none yet".
    last_ms_plus_one: AtomicU64,
    /// Failures not reported since the last report.
    suppressed: AtomicU64,
}

impl KeepAliveFailureReport {
    fn new(epoch: Instant) -> Self {
        Self {
            epoch,
            last_ms_plus_one: AtomicU64::new(0),
            suppressed: AtomicU64::new(0),
        }
    }

    /// Notes a failure at `now`. Returns the number of failures swallowed since the last report when it
    /// is time to report this one, `None` when it is not.
    fn note_failure(&self, now: Instant) -> Option<u64> {
        let at = u64::try_from(now.saturating_duration_since(self.epoch).as_millis())
            .unwrap_or(u64::MAX - 1)
            .saturating_add(1);
        let period = u64::try_from(KEEP_ALIVE_FAILURE_REPORT_PERIOD.as_millis()).unwrap_or(u64::MAX);
        let due = self
            .last_ms_plus_one
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |last| {
                (last == 0 || at.saturating_sub(last) >= period).then_some(at)
            })
            .is_ok();
        if due {
            Some(self.suppressed.swap(0, Ordering::Relaxed))
        } else {
            self.suppressed.fetch_add(1, Ordering::Relaxed);
            None
        }
    }
}

/// Spawns a task for a rate-limited stream of Keep-Alive messages to the Session counterparty.
///
/// A keep-alive that fails to send is logged and skipped; the stream goes on. It used to end at the
/// first failure, and the usual failure is that no SURB was available at that moment, which is a
/// condition that passes: one such packet silenced the Exit's keep-alives, its level reports among
/// them, for the rest of the Session's life.
///
/// `controller` is supplied rather than created here because the Exit shares this one stream between
/// two independent producers — its SURB-level notification and the PIX filler — and both the rate and
/// the admission decision below have to be steered from the same handle. The stream is suspended
/// until the controller is given a non-zero rate.
///
/// `admit` is consulted once per released packet and drops it when it answers `false`. It is a
/// predicate rather than a sink wrapper because the sink here must stay `Clone` — `try_for_each_concurrent`
/// clones it per message — and the natural wrapper for a sink that may emit nothing,
/// `SinkExt::with_flat_map`, is not. Filtering after the rate limiter also puts the decision where
/// the Exit wants it: the rate says how often a packet *may* go, and this says whether this
/// particular one should.
#[instrument(level = "debug", skip(sender, routing, notification_mode, cfg, controller, admit))]
pub(crate) fn spawn_keep_alive_stream<S>(
    session_id: SessionId,
    sender: S,
    routing: DestinationRouting,
    notification_mode: SurbNotificationMode,
    cfg: std::sync::Arc<BalancerStateValues>,
    controller: &RateController,
    admit: impl Fn() -> bool + Send + 'static,
) -> AbortHandle
where
    S: futures::Sink<(DestinationRouting, ApplicationDataOut)> + Clone + Send + Sync + Unpin + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    // DropAbortable not needed because the stream only generates items when polled.
    //
    // The message is built *after* the rate limiter releases a slot, not before: the limiter pulls
    // its next item as soon as it has emitted one and holds it for the whole period, so a message
    // built on the way in would carry the level sampled a notify period (15 s by default) before it
    // is sent. Measured after a SURB generation bump: the Entry read the Exit's pre-refill level
    // 15 s late, took it for a deficit, burst to its ceiling and opened its organic gate, and the Exit
    // ended up holding twice its target (three times at production dimensions).
    let (ka_stream, abort_handle) = futures::stream::abortable(
        futures::stream::repeat_with(|| ())
            .rate_limit_with_controller(controller)
            .map(move |_| match &notification_mode {
                SurbNotificationMode::Target => HoprStartProtocol::KeepAlive(KeepAliveMessage {
                    session_id,
                    flags: KeepAliveFlag::BalancerTarget.into(),
                    additional_data: cfg.target_surb_buffer_size.load(std::sync::atomic::Ordering::Relaxed),
                }),
                // Report the level this side acts on: the balancer's accumulator, fed by
                // `produced - consumed - evicted` and clamped to the store capacity on every update,
                // read at the moment the keep-alive goes out. The estimator stays in the enum for the
                // callers.
                SurbNotificationMode::Level(_estimator) => HoprStartProtocol::KeepAlive(KeepAliveMessage {
                    session_id,
                    flags: KeepAliveFlag::BalancerState.into(),
                    additional_data: cfg.buffer_level(),
                }),
                SurbNotificationMode::DoNotNotify => HoprStartProtocol::KeepAlive(KeepAliveMessage {
                    session_id,
                    flags: None.into(),
                    additional_data: 0,
                }),
            })
            .filter(move |_| futures::future::ready(admit())),
    );

    let sender_clone = sender.clone();
    let fwd_routing_clone = routing.clone();
    let failures = Arc::new(KeepAliveFailureReport::new(Instant::now()));

    // This task will automatically terminate once the returned abort handle is used.
    debug!(%session_id, "spawning keep-alive stream");
    let keep_alive_diag = hopr_utils::runtime::diagnostics::ConcurrentDiagnostics::new(
        "session_keep_alive_try_for_each_concurrent",
        module_path!(),
        file!(),
        line!(),
    );
    hopr_utils::runtime::prelude::spawn(hopr_utils::runtime::diagnostics::instrument(
        ka_stream
            .map(move |msg| {
                ApplicationData::try_from(msg)
                    .map(|data| (fwd_routing_clone.clone(), ApplicationDataOut::with_no_packet_info(data)))
                    .map_err(TransportSessionError::from)
            })
            .for_each_concurrent(None, move |msg| {
                let mut sender_clone = sender_clone.clone();
                let keep_alive_diag = keep_alive_diag.clone();
                let failures = failures.clone();
                keep_alive_diag.wrap(|| async move {
                    let sent = match msg {
                        Ok(msg) => sender_clone
                            .send(msg)
                            .await
                            .map_err(TransportSessionError::packet_sending),
                        Err(error) => Err(error),
                    };
                    if let Err(error) = sent {
                        match failures.note_failure(Instant::now()) {
                            Some(suppressed) => {
                                warn!(%session_id, %error, suppressed, "a keep-alive could not be sent, going on");
                            }
                            None => debug!(%session_id, %error, "a keep-alive could not be sent, going on"),
                        }
                    }
                })
            })
            .then(move |()| {
                tracing::debug!(
                    component = "session",
                    %session_id,
                    task = "session keepalive",
                    "background task finished"
                );
                futures::future::ready(())
            })
            .in_current_span(),
        "session_keep_alive",
        module_path!(),
        file!(),
        line!(),
    ));

    abort_handle
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use anyhow::anyhow;
    use futures::FutureExt;
    use hopr_api::types::{crypto_random::Randomizable, internal::prelude::HoprPseudonym};

    use super::*;

    /// Generator that cycles through 0..4, wrapping at 5 back to 0.
    fn cycling_generator(prev: Option<u8>) -> u8 {
        prev.map(|v| (v + 1) % 5).unwrap_or(0)
    }

    /// Tests sequential insertion into an empty cache: each call fills the next slot.
    #[tokio::test]
    async fn test_insert_into_next_slot_sequential() -> anyhow::Result<()> {
        let cache = moka::sync::Cache::new(10);

        for i in 0..5 {
            let (k, v) = insert_into_next_slot(&cache, cycling_generator, |k| format!("foo_{k}"), Some(10u64))
                .ok_or(anyhow!("should insert into slot {i}"))?;
            assert_eq!(k, i);
            assert_eq!(format!("foo_{i}"), v);
            assert_eq!(Some(v), cache.get(&i));
        }

        Ok(())
    }

    /// Tests that insertion returns `None` when all slots are occupied and the generator cycles back.
    #[tokio::test]
    async fn test_insert_into_next_slot_returns_none_when_full() -> anyhow::Result<()> {
        let cache = moka::sync::Cache::new(10);

        for _ in 0..5 {
            insert_into_next_slot(&cache, cycling_generator, |k| format!("foo_{k}"), Some(10u64))
                .ok_or(anyhow!("precondition: should insert"))?;
        }

        assert!(
            insert_into_next_slot(&cache, cycling_generator, |_| "foo".to_string(), Some(10u64)).is_none(),
            "must not find slot when full"
        );

        Ok(())
    }

    /// Tests that a cache with max capacity of 1 rejects a second distinct key.
    #[tokio::test]
    async fn test_insert_into_next_slot_capacity_one_rejects_second_key() -> anyhow::Result<()> {
        let unit_cache = moka::sync::Cache::new(1);

        let (k0, _v0) = insert_into_next_slot(&unit_cache, |prev| prev.map(|v| v + 1).unwrap_or(0), |k| k, Some(1u64))
            .ok_or(anyhow!("first insertion must succeed"))?;
        assert_eq!(k0, 0);

        assert!(
            insert_into_next_slot(&unit_cache, |prev| prev.map(|v| v + 1).unwrap_or(0), |k| k, Some(1u64)).is_none(),
            "second distinct key must be rejected when cache capacity is 1"
        );

        Ok(())
    }

    /// Tests that a rejected insertion does not evict the existing entry.
    #[tokio::test]
    async fn test_insert_into_next_slot_rejected_insertion_does_not_evict() -> anyhow::Result<()> {
        let unit_cache = moka::sync::Cache::new(1);

        let (k0, v0) = insert_into_next_slot(&unit_cache, |prev| prev.map(|v| v + 1).unwrap_or(0), |k| k, Some(1u64))
            .ok_or(anyhow!("first insertion must succeed"))?;

        insert_into_next_slot(&unit_cache, |prev| prev.map(|v| v + 1).unwrap_or(0), |k| k, Some(1u64));

        assert_eq!(
            Some(v0),
            unit_cache.get(&k0),
            "first entry must still be present after rejection"
        );

        Ok(())
    }

    fn noop_waker() -> std::task::Waker {
        futures::task::noop_waker()
    }

    // -------------------------------------------------------------------
    // Generation-counter wake detection
    // -------------------------------------------------------------------

    #[test]
    fn notification_after_first_poll_completes_future() {
        let n = Arc::new(SlotNotify::new());
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
        let n = Arc::new(SlotNotify::new());
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
        let n = Arc::new(SlotNotify::new());
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
        let n = Arc::new(SlotNotify::new());
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
        let n = Arc::new(SlotNotify::new());
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

    /// `notify_waiters` must have released the lock by the time any waker runs.
    ///
    /// `Waker::wake` runs arbitrary executor code, and an executor that polls the woken task inline
    /// re-enters this type: both [`SlotNotifyFuture::poll`] and its `Drop` take the same
    /// `parking_lot::Mutex`, which is not reentrant. Waking under the guard self-deadlocks against
    /// precisely the runtime-agnostic contract this type advertises. Tokio only enqueues, so no
    /// shipped caller ever exercised it and no test could have caught it.
    ///
    /// The waker below stands in for such an executor. Note the failure mode: on a regression this
    /// test does not fail, it *hangs*, and surfaces as a timeout. That is the only observable a
    /// deadlock has.
    #[test]
    fn notify_waiters_releases_the_lock_before_waking() {
        struct ReentrantWaker(SlotNotify);

        impl std::task::Wake for ReentrantWaker {
            fn wake(self: Arc<Self>) {
                // What an inline-polling executor does: re-enter the notifier from inside `wake`.
                // `notified` takes the very lock `notify_waiters` is in the middle of.
                drop(self.0.notified());
            }
        }

        let n = SlotNotify::new();
        let mut fut = n.notified();
        let waker = std::task::Waker::from(Arc::new(ReentrantWaker(n.clone())));
        let mut cx = std::task::Context::from_waker(&waker);

        assert_eq!(
            fut.poll_unpin(&mut cx),
            Poll::Pending,
            "the first poll registers the waker"
        );

        n.notify_waiters();

        assert_eq!(
            fut.poll_unpin(&mut cx),
            Poll::Ready(()),
            "the generation advanced, so the waiter is done"
        );
    }

    /// One `notify_waiters` call must wake *every* parked waiter, not just one.
    ///
    /// `wakers` is a `Vec` and the type calls itself a multi-waker primitive, but every other test
    /// here drives a single future, so the vector never holds more than one entry and a `pop()` in
    /// place of `drain(..)` would pass all of them.
    ///
    /// Asserted on the wakes rather than on the futures completing, which is the part that is easy
    /// to get wrong: the generation counter makes `poll` return `Ready` after any notification
    /// whether or not *that* waiter was woken, so polling every future to completion would pass with
    /// a single-waker regression. What a real executor needs is the wake itself — without it the
    /// future is never polled again and the generation is never read.
    #[test]
    fn one_notification_wakes_every_parked_waiter() {
        struct CountingWaker(std::sync::atomic::AtomicUsize);

        impl std::task::Wake for CountingWaker {
            fn wake(self: Arc<Self>) {
                self.wake_by_ref();
            }

            fn wake_by_ref(self: &Arc<Self>) {
                self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }

        const WAITERS: usize = 4;

        let n = SlotNotify::new();
        let counters: Vec<Arc<CountingWaker>> = (0..WAITERS)
            .map(|_| Arc::new(CountingWaker(std::sync::atomic::AtomicUsize::new(0))))
            .collect();
        let mut futures: Vec<_> = (0..WAITERS).map(|_| n.notified()).collect();

        for (i, (fut, counter)) in futures.iter_mut().zip(&counters).enumerate() {
            let waker = std::task::Waker::from(counter.clone());
            let mut cx = std::task::Context::from_waker(&waker);
            assert_eq!(fut.poll_unpin(&mut cx), Poll::Pending, "waiter {i} must park");
        }
        assert_eq!(
            WAITERS,
            n.inner.lock().wakers.len(),
            "every waiter must have registered"
        );

        n.notify_waiters();

        for (i, counter) in counters.iter().enumerate() {
            assert_eq!(
                1,
                counter.0.load(std::sync::atomic::Ordering::Relaxed),
                "waiter {i} must be woken exactly once by the single notification"
            );
        }
    }

    /// Re-polling with a *different* waker must replace the stored one.
    ///
    /// The `will_wake` guard in `poll` skips the clone when the stored waker would wake the same
    /// task, which is the common case and the point of the guard. Getting its polarity backwards
    /// would keep the first waker forever: the future would then be parked against a task that is no
    /// longer polling it, and nothing would wake the one that is — a hang, not a slowdown. This is
    /// the half of that guard worth pinning; the skipped clone is unobservable by design.
    #[test]
    fn repolling_with_a_new_waker_replaces_the_stored_one() {
        struct CountingWaker(std::sync::atomic::AtomicUsize);

        impl std::task::Wake for CountingWaker {
            fn wake(self: Arc<Self>) {
                self.wake_by_ref();
            }

            fn wake_by_ref(self: &Arc<Self>) {
                self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }

        let n = SlotNotify::new();
        let mut fut = n.notified();

        let first = Arc::new(CountingWaker(std::sync::atomic::AtomicUsize::new(0)));
        let second = Arc::new(CountingWaker(std::sync::atomic::AtomicUsize::new(0)));

        let w1 = std::task::Waker::from(first.clone());
        assert_eq!(
            fut.poll_unpin(&mut std::task::Context::from_waker(&w1)),
            Poll::Pending,
            "the first poll registers"
        );

        // A distinct task, so `will_wake` is false and the replacement must happen.
        let w2 = std::task::Waker::from(second.clone());
        assert_eq!(
            fut.poll_unpin(&mut std::task::Context::from_waker(&w2)),
            Poll::Pending,
            "no notification has happened, so the future stays parked"
        );

        n.notify_waiters();

        assert_eq!(
            0,
            first.0.load(std::sync::atomic::Ordering::Relaxed),
            "the superseded waker must not be the one woken"
        );
        assert_eq!(
            1,
            second.0.load(std::sync::atomic::Ordering::Relaxed),
            "the waker from the most recent poll must be woken exactly once"
        );
    }

    // -------------------------------------------------------------------
    // Async tests (tokio runtime)
    // -------------------------------------------------------------------

    #[tokio::test]
    async fn notified_awaited_completes_on_notify() {
        let n = Arc::new(SlotNotify::new());
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

    /// A sender that refuses its first `fail_next` messages, as one without a SURB does.
    #[derive(Clone)]
    struct FlakySender {
        inner: futures::channel::mpsc::UnboundedSender<(DestinationRouting, ApplicationDataOut)>,
        fail_next: Arc<AtomicU64>,
    }

    impl futures::Sink<(DestinationRouting, ApplicationDataOut)> for FlakySender {
        type Error = std::io::Error;

        fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Pin::new(&mut self.inner).poll_ready(cx).map_err(std::io::Error::other)
        }

        fn start_send(
            mut self: Pin<&mut Self>,
            item: (DestinationRouting, ApplicationDataOut),
        ) -> Result<(), Self::Error> {
            if self
                .fail_next
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |left| left.checked_sub(1))
                .is_ok()
            {
                return Err(std::io::Error::other("no SURB available"));
            }
            Pin::new(&mut self.inner)
                .start_send(item)
                .map_err(std::io::Error::other)
        }

        fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Pin::new(&mut self.inner).poll_flush(cx).map_err(std::io::Error::other)
        }

        fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Pin::new(&mut self.inner).poll_close(cx).map_err(std::io::Error::other)
        }
    }

    /// One keep-alive that finds no SURB must not silence the stream: the next one is sent.
    #[tokio::test]
    async fn a_keep_alive_that_fails_to_send_should_not_end_the_stream() -> anyhow::Result<()> {
        let (tx, mut sent) = futures::channel::mpsc::unbounded();
        let sender = FlakySender {
            inner: tx,
            fail_next: Arc::new(AtomicU64::new(1)),
        };
        let controller = RateController::new(200, Duration::from_secs(1));
        let pseudonym = HoprPseudonym::random();
        let abort = spawn_keep_alive_stream(
            pseudonym,
            sender,
            DestinationRouting::Return(pseudonym.into()),
            SurbNotificationMode::DoNotNotify,
            Arc::new(BalancerStateValues::from(crate::SurbBalancerConfig::default())),
            &controller,
            || true,
        );

        for n in 1..=3 {
            tokio::time::timeout(Duration::from_secs(5), sent.next())
                .await
                .map_err(|_| anyhow!("keep-alive {n} after the failed one never arrived"))?
                .ok_or_else(|| anyhow!("the stream ended after the failed keep-alive"))?;
        }
        abort.abort();
        Ok(())
    }

    /// The level a keep-alive reports is read when the keep-alive goes out, not when the rate
    /// limiter pulled the next item a whole period earlier.
    #[tokio::test]
    async fn a_keep_alive_should_report_the_level_at_the_moment_it_is_sent() -> anyhow::Result<()> {
        let (tx, mut sent) = futures::channel::mpsc::unbounded();
        let controller = RateController::new(2, Duration::from_secs(1));
        let pseudonym = HoprPseudonym::random();
        let state = Arc::new(BalancerStateValues::from(crate::SurbBalancerConfig::default()));
        state.buffer_level.store(100, std::sync::atomic::Ordering::Relaxed);
        let abort = spawn_keep_alive_stream(
            pseudonym,
            tx,
            DestinationRouting::Return(pseudonym.into()),
            SurbNotificationMode::Level(crate::balancer::AtomicSurbFlowEstimator::default()),
            state.clone(),
            &controller,
            || true,
        );
        // The stream has pulled its first slot by now; the level moves before the slot is released.
        tokio::time::sleep(Duration::from_millis(50)).await;
        state.buffer_level.store(7_000, std::sync::atomic::Ordering::Relaxed);

        let (_, data) = tokio::time::timeout(Duration::from_secs(5), sent.next())
            .await
            .map_err(|_| anyhow!("the first keep-alive never arrived"))?
            .ok_or_else(|| anyhow!("the stream ended"))?;
        abort.abort();
        let msg = HoprStartProtocol::decode(data.data.application_tag, &data.data.plain_text)?;
        match msg {
            HoprStartProtocol::KeepAlive(ka) => {
                assert!(ka.flags.contains(KeepAliveFlag::BalancerState));
                assert_eq!(
                    7_000, ka.additional_data,
                    "the level at send time, not the one from a period earlier"
                );
            }
            other => anyhow::bail!("not a keep-alive: {other:?}"),
        }
        Ok(())
    }

    /// At most one report a second per stream, with the number of failures it swallowed since the last.
    #[test]
    fn keep_alive_failures_should_be_reported_at_most_once_a_second() {
        let epoch = Instant::now();
        let report = KeepAliveFailureReport::new(epoch);
        let at = |ms| epoch + Duration::from_millis(ms);

        assert_eq!(Some(0), report.note_failure(at(0)), "the first is reported at once");
        assert_eq!(None, report.note_failure(at(10)));
        assert_eq!(None, report.note_failure(at(999)));
        assert_eq!(
            Some(2),
            report.note_failure(at(1_000)),
            "a second on, with the two it swallowed"
        );
        assert_eq!(None, report.note_failure(at(1_500)));
        assert_eq!(Some(1), report.note_failure(at(2_600)));
    }
}
