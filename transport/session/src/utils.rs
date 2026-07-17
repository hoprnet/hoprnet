use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
    time::Duration,
};

use futures::{FutureExt, SinkExt, StreamExt, TryStreamExt};
use hopr_api::types::internal::routing::DestinationRouting;
use hopr_protocol_app::prelude::{ApplicationData, ApplicationDataOut};
use hopr_protocol_start::{KeepAliveFlag, KeepAliveMessage};
use hopr_utils::runtime::AbortHandle;
use tracing::{Instrument, debug, error, instrument};

use crate::{
    AtomicSurbFlowEstimator, SessionId,
    balancer::{BalancerStateValues, RateController, RateLimitStreamExt, SurbFlowEstimator},
    errors::TransportSessionError,
    types::HoprStartProtocol,
};

/// Runtime-agnostic multi-waker notification primitive.
///
/// Uses a generation counter to detect notification events: [`notify_waiters`]
/// bumps the generation, and [`notified`] futures compare the generation at
/// creation time against the current one on each `poll()`. This prevents two
/// race conditions a simple waker-vector approach cannot handle:
///
/// 1. **Latent wake.** A [`notified`] future registers its waker on first `poll()`. If [`notify_waiters`] fires between
///    the creation of the future and that first `poll`, the waker-vector is empty and the notification is lost. With a
///    generation counter, `gen_at_creation` already captures the pre-notification value, so the first `poll()` sees the
///    advanced generation and returns `Ready`.
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

/// Convenience function to copy data in both directions between a [`Session`](crate::HoprSession) and arbitrary
/// async IO stream.
/// This function is only available with Tokio and will panic with other runtimes.
///
/// The `abort_stream` will terminate the transfer from the `stream` side, i.e.:
/// 1. Initiates graceful shutdown of `stream`
/// 2. Once done, initiates a graceful shutdown of `session`
/// 3. The function terminates, returning the number of bytes transferred in both directions.
#[cfg(feature = "runtime-tokio")]
pub async fn transfer_session<S>(
    session: &mut crate::HoprSession,
    stream: &mut S,
    max_buffer: usize,
    abort_stream: Option<futures::future::AbortRegistration>,
) -> std::io::Result<(usize, usize)>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    // We can use equally sized buffer for both directions
    tracing::debug!(
        session_id = ?session.id(),
        egress_buffer = max_buffer,
        ingress_buffer = max_buffer,
        "session buffers"
    );

    if let Some(abort_stream) = abort_stream {
        // We only allow aborting from the "stream" side, not from the "session side"
        // This is useful for UDP-like streams on the "stream" side, which cannot be terminated
        // by a signal from outside (e.g.: for TCP sockets such signal is socket closure).
        let (_, dummy) = futures::future::AbortHandle::new_pair();
        hopr_utils::network_types::utils::copy_duplex_abortable(
            session,
            stream,
            (max_buffer, max_buffer),
            (dummy, abort_stream),
        )
        .await
        .map(|(a, b)| (a as usize, b as usize))
    } else {
        hopr_utils::network_types::utils::copy_duplex(session, stream, (max_buffer, max_buffer))
            .await
            .map(|(a, b)| (a as usize, b as usize))
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

/// Spawns a task for a rate-limited stream of Keep-Alive messages to the Session counterparty.
#[instrument(level = "debug", skip(sender, routing, notification_mode, cfg))]
pub(crate) fn spawn_keep_alive_stream<S>(
    session_id: SessionId,
    sender: S,
    routing: DestinationRouting,
    notification_mode: SurbNotificationMode,
    cfg: std::sync::Arc<BalancerStateValues>,
) -> (RateController, AbortHandle)
where
    S: futures::Sink<(DestinationRouting, ApplicationDataOut)> + Clone + Send + Sync + Unpin + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    // The stream is suspended until the caller sets a rate via the Controller
    let controller = RateController::new(0, Duration::from_secs(1));

    // DropAbortable not needed because the stream only generates items when polled
    let (ka_stream, abort_handle) = futures::stream::abortable(
        futures::stream::repeat_with(move || match &notification_mode {
            SurbNotificationMode::Target => HoprStartProtocol::KeepAlive(KeepAliveMessage {
                session_id,
                flags: KeepAliveFlag::BalancerTarget.into(),
                additional_data: cfg.target_surb_buffer_size.load(std::sync::atomic::Ordering::Relaxed),
            }),
            SurbNotificationMode::Level(estimator) => HoprStartProtocol::KeepAlive(KeepAliveMessage {
                session_id,
                flags: KeepAliveFlag::BalancerState.into(),
                additional_data: estimator.saturating_diff(),
            }),
            SurbNotificationMode::DoNotNotify => HoprStartProtocol::KeepAlive(KeepAliveMessage {
                session_id,
                flags: None.into(),
                additional_data: 0,
            }),
        })
        .rate_limit_with_controller(&controller),
    );

    let sender_clone = sender.clone();
    let fwd_routing_clone = routing.clone();

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
            })
            .map_err(TransportSessionError::from)
            .try_for_each_concurrent(None, move |msg| {
                let mut sender_clone = sender_clone.clone();
                let keep_alive_diag = keep_alive_diag.clone();
                keep_alive_diag.wrap(|| async move {
                    sender_clone
                        .send(msg)
                        .await
                        .map_err(TransportSessionError::packet_sending)
                })
            })
            .then(move |res| {
                match res {
                    Ok(_) => tracing::debug!(
                        component = "session",
                        %session_id,
                        task = "session keepalive",
                        "background task finished"
                    ),
                    Err(error) => error!(%session_id, %error, "keep-alive stream failed"),
                }
                futures::future::ready(())
            })
            .in_current_span(),
        "session_keep_alive",
        module_path!(),
        file!(),
        line!(),
    ));

    (controller, abort_handle)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use anyhow::anyhow;
    use futures::FutureExt;

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
}
