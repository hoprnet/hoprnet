//! Exponential (Poisson) release mixer — an additive alternative to the uniform `channel`.
//!
//! A dedicated OS thread holds packets in a pool and, on every wake, releases each eligible
//! packet with the memoryless probability `1 - e^(-delta/mean)`, giving exponential holding
//! times and a Poisson-like departure process that resists timing correlation.
//!
//! The thread `block_on`s one merged event stream (`Events`) combining the ingress channel with
//! a self-rescheduling [`futures_timer::Delay`]; merging (not `select!`) lets a new packet
//! interrupt the wait and re-arm the tick with no fuse/cancellation footguns. Ingress and egress
//! are unbounded [`async_channel`]s — lock-free non-blocking `&self` sends, deliberately chosen
//! over `futures::channel::mpsc` for lower overhead on this MPSC hot path.

use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll},
    time::{Duration, Instant},
};

use async_channel::{Receiver as ChanRx, Sender as ChanTx};
use futures::{FutureExt, Stream, StreamExt};
use futures_timer::Delay;

pub use crate::error::SenderError;
use crate::{
    config::MixerConfig,
    pool::{self, Entry, PoissonParams},
};

/// Sender end of the Poisson mixing channel.
pub struct Sender<T> {
    input: ChanTx<(Instant, T)>,
    receiver_active: Arc<AtomicBool>,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            input: self.input.clone(),
            receiver_active: self.receiver_active.clone(),
        }
    }
}

impl<T> Sender<T> {
    /// Send one item into the mixer. Never blocks (the ingress is unbounded); returns
    /// [`SenderError::Closed`] once the receiver is gone.
    pub fn send(&self, item: T) -> Result<(), SenderError> {
        if !self.receiver_active.load(Ordering::Relaxed) {
            return Err(SenderError::Closed);
        }

        self.input.try_send((Instant::now(), item)).map_err(|_| {
            self.receiver_active.store(false, Ordering::Relaxed);
            SenderError::Closed
        })
    }
}

impl<T> futures::sink::Sink<T> for Sender<T> {
    type Error = SenderError;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Unbounded ingress: always ready while the receiver lives.
        if self.receiver_active.load(Ordering::Relaxed) {
            Poll::Ready(Ok(()))
        } else {
            Poll::Ready(Err(SenderError::Closed))
        }
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        self.get_mut().send(item)
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

/// Receiver end of the Poisson mixing channel. Yields already-mixed items and ends
/// with `None` once the last sender drops and the pool has drained.
pub struct Receiver<T> {
    // `async_channel::Receiver` is `!Unpin` (its `Stream` state pins an event listener), so we
    // box-pin it to keep this wrapper `Unpin` for `Stream`/`StreamExt` consumers.
    output: Pin<Box<ChanRx<T>>>,
    receiver_active: Arc<AtomicBool>,
}

impl<T> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().output.as_mut().poll_next(cx)
    }
}

impl<T> Receiver<T> {
    /// Receive a single mixed item.
    pub async fn recv(&mut self) -> Option<T> {
        self.output.next().await
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        // Let senders observe closure immediately; the engine also notices via the
        // closed output channel on its next wake.
        self.receiver_active.store(false, Ordering::Relaxed);
    }
}

/// A single unit of work for the engine loop.
enum Event<T> {
    /// A packet arrived from a sender at the given ingress instant.
    Input(Instant, T),
    /// All senders have dropped; no more input will arrive.
    InputClosed,
    /// The adaptive timer fired.
    Tick,
}

/// The merged event source driving the engine: the ingress channel and one
/// self-rescheduling timer, combined into a single `Stream`. Ingress is polled first so
/// bursts are absorbed promptly; the timer provides progress when ingress is idle.
struct Events<T> {
    // Box-pinned because `async_channel::Receiver` is `!Unpin` (see [`Receiver`]).
    input: Pin<Box<ChanRx<(Instant, T)>>>,
    timer: Delay,
    input_open: bool,
}

impl<T> Events<T> {
    /// Re-arm the timer to fire after `after` from now.
    fn rearm(&mut self, after: Duration) {
        self.timer.reset(after);
    }

    /// Pull one already-queued ingress item without waiting, returning `None` when the
    /// queue is momentarily empty (or closed). Used to absorb a whole burst into the pool
    /// in one pass, so the swept occupancy reflects the true concurrent buffering rather
    /// than one-item-at-a-time.
    fn try_next_input(&mut self) -> Option<(Instant, T)> {
        if !self.input_open {
            return None;
        }
        // `Err` covers both momentarily-empty and closed; closure is observed authoritatively by
        // the `Stream` path (which yields `Event::InputClosed`), so treat both as "nothing now".
        self.input.try_recv().ok()
    }
}

impl<T> Stream for Events<T> {
    type Item = Event<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if this.input_open {
            match this.input.as_mut().poll_next(cx) {
                Poll::Ready(Some((ts, item))) => return Poll::Ready(Some(Event::Input(ts, item))),
                Poll::Ready(None) => {
                    this.input_open = false;
                    return Poll::Ready(Some(Event::InputClosed));
                }
                Poll::Pending => {}
            }
        }

        match this.timer.poll_unpin(cx) {
            Poll::Ready(()) => Poll::Ready(Some(Event::Tick)),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// The engine loop, run to completion by [`futures::executor::block_on`] on the dedicated
/// thread. Sweeps the pool on every event and re-arms the adaptive timer afterwards.
async fn run_engine<T>(
    params: PoissonParams,
    input: ChanRx<(Instant, T)>,
    output: ChanTx<T>,
    receiver_active: Arc<AtomicBool>,
) {
    let mut pool: Vec<Entry<T>> = Vec::with_capacity(params.capacity());
    // Reused across sweeps to avoid a per-sweep allocation.
    let mut released: Vec<(Duration, T)> = Vec::new();
    let mut prev_sweep = Instant::now();
    let mut events = Events {
        input: Box::pin(input),
        timer: Delay::new(pool::IDLE_HEARTBEAT),
        input_open: true,
    };

    while let Some(event) = events.next().await {
        if let Event::Input(ts, item) = event {
            pool.push(Entry::new(ts, item));
        }
        // Absorb the rest of any concurrently-queued burst before sweeping, so occupancy
        // reflects true buffering (otherwise the low-watermark flush would release each
        // item singly, in order, and never mix).
        while let Some((ts, item)) = events.try_next_input() {
            pool.push(Entry::new(ts, item));
        }

        let now = Instant::now();
        let delta = now.saturating_duration_since(prev_sweep);
        prev_sweep = now;
        let earliest = pool::sweep(&mut pool, &params, now, delta, &mut released);
        for (realized_delay, item) in released.drain(..) {
            #[cfg(all(feature = "telemetry", not(test)))]
            crate::metrics::record_average_delay(realized_delay.as_millis() as f64, params.metric_delay_window);
            #[cfg(not(all(feature = "telemetry", not(test))))]
            let _ = realized_delay;

            if output.try_send(item).is_err() {
                receiver_active.store(false, Ordering::Relaxed);
                return;
            }
        }

        #[cfg(all(feature = "telemetry", not(test)))]
        crate::metrics::METRIC_QUEUE_SIZE.set(pool.len() as f64);

        // Terminate once no more input can arrive and the pool has drained, or once the
        // receiver has gone away.
        if !events.input_open && pool.is_empty() {
            break;
        }
        if output.is_closed() {
            break;
        }

        let wake = pool::next_wake(earliest, pool.len(), &params, now);
        events.rearm(wake);
    }

    receiver_active.store(false, Ordering::Relaxed);
}

/// Instantiate a Poisson mixing channel, spawning the dedicated engine thread.
///
/// Drop-in alternative to the uniform `channel()`: same `Sender`/`Receiver` shapes, but the
/// release process is exponential (memoryless) rather than uniform.
pub fn poisson_channel<T: Send + 'static>(cfg: MixerConfig) -> (Sender<T>, Receiver<T>) {
    #[cfg(all(feature = "telemetry", not(test)))]
    {
        lazy_static::initialize(&crate::metrics::METRIC_QUEUE_SIZE);
        lazy_static::initialize(&crate::metrics::METRIC_MIXER_AVERAGE_DELAY);
    }

    let params = PoissonParams::from_mixer(&cfg);
    let (input_tx, input_rx) = async_channel::unbounded::<(Instant, T)>();
    let (output_tx, output_rx) = async_channel::unbounded::<T>();
    let receiver_active = Arc::new(AtomicBool::new(true));

    let engine_flag = receiver_active.clone();
    std::thread::Builder::new()
        .name("hopr-mixer-poisson".into())
        .spawn(move || {
            futures::executor::block_on(run_engine(params, input_rx, output_tx, engine_flag));
        })
        .expect("failed to spawn the mixer engine thread");

    (
        Sender {
            input: input_tx,
            receiver_active: receiver_active.clone(),
        },
        Receiver {
            output: Box::pin(output_rx),
            receiver_active,
        },
    )
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;
    use rstest::rstest;
    use tokio::time::timeout;

    use super::*;
    use crate::config::{MixerType, PoissonConfig, PoissonDelay};

    const CAP: Duration = Duration::from_millis(crate::config::HOPR_MIXER_DEFAULT_MAX_CAP_IN_MS);
    const LEEWAY: Duration = Duration::from_millis(500);

    /// Config selecting the dedicated-thread engine with the given delay anchor and percentile.
    fn poisson_cfg(delay: PoissonDelay, cap_percentile: f64) -> MixerConfig {
        MixerConfig {
            mixer_type: MixerType::Poisson(PoissonConfig {
                delay,
                cap_percentile,
                ..PoissonConfig::default()
            }),
            ..MixerConfig::default()
        }
    }

    /// Drain `rx` in a spawned task, mapping each item at the instant it is received (so realized
    /// delays are measured at delivery), returning the mapped values in delivery order.
    fn spawn_drain<T, R>(mut rx: Receiver<T>, f: impl Fn(T) -> R + Send + 'static) -> tokio::task::JoinHandle<Vec<R>>
    where
        T: Send + 'static,
        R: Send + 'static,
    {
        tokio::spawn(async move {
            let mut out = Vec::new();
            while let Some(item) = rx.next().await {
                out.push(f(item));
            }
            out
        })
    }

    #[tokio::test]
    async fn mixer_should_pass_an_element() -> anyhow::Result<()> {
        let (tx, mut rx) = poisson_channel(MixerConfig::default());
        tx.send(1)?;
        assert_eq!(timeout(CAP + LEEWAY, rx.recv()).await?, Some(1));
        Ok(())
    }

    #[tokio::test]
    async fn max_delay_should_be_bounded_by_the_cap() -> anyhow::Result<()> {
        // Set the mean far above the cap: without the hard cap most packets would wait ~50 ms
        // (the exponential mean), so asserting the *maximum realized* delay stays within the cap
        // proves the cap actually bounds latency — a tight upper bound the old timeout-based
        // check (500 ms leeway on a 20 ms cap) could never provide.
        const N: usize = 3000;
        let cap = Duration::from_millis(20);
        // mean >> cap, so the hard cap (not the coin) must do the bounding.
        let cfg = poisson_cfg(PoissonDelay::Cap(cap), 0.33);
        let (tx, mut rx) = poisson_channel::<Instant>(cfg);

        let receiver = tokio::spawn(async move {
            let mut max_delay_ms = 0.0f64;
            let mut count = 0usize;
            while let Some(sent_at) = rx.next().await {
                max_delay_ms = max_delay_ms.max(sent_at.elapsed().as_secs_f64() * 1000.0);
                count += 1;
            }
            (max_delay_ms, count)
        });
        for _ in 0..N {
            tx.send(Instant::now())?;
        }
        drop(tx);

        let (observed_max, count) = tokio::time::timeout(Duration::from_secs(10), receiver).await??;
        assert_eq!(count, N, "every packet must be delivered");

        let cap_ms = cap.as_secs_f64() * 1000.0;
        // Generous scheduling slack, but far below the ~50 ms the uncapped exponential would
        // routinely produce, so a regression that stopped enforcing the cap fails here.
        assert!(
            observed_max < cap_ms + 80.0,
            "max realized delay {observed_max:.2} ms must stay within the {cap_ms:.0} ms cap plus slack"
        );
        Ok(())
    }

    #[tokio::test]
    async fn mixer_should_produce_mixed_output() -> anyhow::Result<()> {
        // Sending a burst well above the low watermark exercises the exponential clock and the
        // per-wake shuffle, so the output order should differ from the input. A mean well below
        // the cap keeps the whole burst dwelling together long enough to mix.
        const ITERATIONS: usize = 40;
        let (tx, rx) = poisson_channel(poisson_cfg(PoissonDelay::Mean(Duration::from_millis(20)), 0.99));

        let input = (0..ITERATIONS).collect::<Vec<_>>();
        for i in input.iter() {
            tx.send(*i)?;
        }

        let mixed = timeout(4 * CAP + LEEWAY, rx.take(ITERATIONS).collect::<Vec<_>>()).await?;
        assert_eq!(mixed.len(), ITERATIONS, "every item must be delivered");
        assert_ne!(input, mixed, "output order should be mixed");
        Ok(())
    }

    #[tokio::test]
    async fn mixer_should_hold_and_reorder_under_load() -> anyhow::Result<()> {
        // End-to-end guard: a high-occupancy stream must be genuinely held (non-trivial mean
        // delay) and reordered — the property the engine exists for, and a regression guard
        // against the "release everything instantly" failure mode.
        const N: usize = 3000;
        let cfg = poisson_cfg(PoissonDelay::Mean(Duration::from_millis(10)), 0.999);
        let (tx, mut rx) = poisson_channel::<(u32, Instant)>(cfg);

        let receiver = tokio::spawn(async move {
            let mut delays_ms = Vec::with_capacity(N);
            let mut max_seq: i64 = -1;
            let mut out_of_order = 0usize;
            while let Some((seq, sent_at)) = rx.next().await {
                delays_ms.push(sent_at.elapsed().as_secs_f64() * 1000.0);
                if (seq as i64) < max_seq {
                    out_of_order += 1;
                }
                max_seq = max_seq.max(seq as i64);
            }
            (delays_ms, out_of_order)
        });

        // Burst-send so occupancy is well above the min-mix threshold (the coin regime).
        for seq in 0..N as u32 {
            tx.send((seq, Instant::now()))?;
        }
        drop(tx);

        let (delays_ms, out_of_order) = tokio::time::timeout(Duration::from_secs(10), receiver).await??;
        assert_eq!(delays_ms.len(), N, "every packet must be delivered");

        let observed_mean = delays_ms.iter().sum::<f64>() / N as f64;
        assert!(
            observed_mean > 3.0,
            "packets must be genuinely held (observed mean {observed_mean:.2} ms)"
        );
        assert!(observed_mean < 60.0, "mean {observed_mean:.2} ms unexpectedly large");

        let reordered = out_of_order as f64 / N as f64;
        assert!(
            reordered > 0.10,
            "output must be substantially mixed (reordered {reordered:.2})"
        );
        Ok(())
    }

    #[tokio::test]
    async fn burst_after_idle_should_still_be_mixed() -> anyhow::Result<()> {
        // Regression: after an idle gap the previous-sweep instant goes stale, so the sweep's
        // `delta` is large. A burst arriving then must still be held and mixed — not dumped with
        // ~zero delay because the large `delta` gave every fresh packet a ~1 release probability.
        let cfg = poisson_cfg(PoissonDelay::Mean(Duration::from_millis(10)), 0.999);
        let (tx, mut rx) = poisson_channel::<(u32, Instant)>(cfg);

        // Idle less than the 200 ms heartbeat, so the burst hits a stale `prev_sweep`.
        tokio::time::sleep(Duration::from_millis(150)).await;

        const N: usize = 2000;
        let receiver = tokio::spawn(async move {
            let mut delays_ms = Vec::with_capacity(N);
            while let Some((_seq, sent_at)) = rx.next().await {
                delays_ms.push(sent_at.elapsed().as_secs_f64() * 1000.0);
            }
            delays_ms
        });
        for seq in 0..N as u32 {
            tx.send((seq, Instant::now()))?;
        }
        drop(tx);

        let delays_ms = tokio::time::timeout(Duration::from_secs(10), receiver).await??;
        assert_eq!(delays_ms.len(), N);
        let mean = delays_ms.iter().sum::<f64>() / N as f64;
        assert!(
            mean > 3.0,
            "a burst after an idle gap must still be held/mixed (observed mean {mean:.2} ms)"
        );
        Ok(())
    }

    #[tokio::test]
    async fn poisson_channel_passthrough_should_preserve_order() -> anyhow::Result<()> {
        const ITERATIONS: usize = 40;
        let (tx, rx) = poisson_channel(poisson_cfg(PoissonDelay::Cap(Duration::ZERO), 0.99));

        let input = (0..ITERATIONS).collect::<Vec<_>>();
        for i in input.iter() {
            tx.send(*i)?;
            tokio::time::sleep(Duration::from_micros(50)).await;
        }

        let output = timeout(2 * CAP + LEEWAY, rx.take(ITERATIONS).collect::<Vec<_>>()).await?;
        assert_eq!(input, output, "pass-through must preserve FIFO order");
        Ok(())
    }

    #[tokio::test]
    async fn receiver_should_drain_buffered_items_after_last_sender_drops() -> anyhow::Result<()> {
        const ITERATIONS: usize = 16;
        let (tx, mut rx) = poisson_channel::<u32>(MixerConfig::default());

        for i in 0..ITERATIONS as u32 {
            tx.send(i)?;
        }
        drop(tx);

        let mut received = 0usize;
        while let Some(_item) = timeout(CAP + LEEWAY, rx.next()).await? {
            received += 1;
        }
        assert_eq!(received, ITERATIONS, "all buffered items must drain before close");
        Ok(())
    }

    #[tokio::test]
    async fn sender_clones_should_share_engine() -> anyhow::Result<()> {
        let (tx_a, mut rx) = poisson_channel::<u32>(MixerConfig::default());
        let tx_b = tx_a.clone();

        tx_a.send(1)?;
        tx_b.send(2)?;
        drop(tx_a);
        drop(tx_b);

        let mut got = vec![
            timeout(CAP + LEEWAY, rx.next()).await?.expect("first item"),
            timeout(CAP + LEEWAY, rx.next()).await?.expect("second item"),
        ];
        got.sort();
        assert_eq!(got, vec![1, 2]);
        assert!(
            rx.next().await.is_none(),
            "channel should close after both senders drop"
        );
        Ok(())
    }

    #[tokio::test]
    async fn send_should_fail_after_receiver_dropped() -> anyhow::Result<()> {
        let (tx, rx) = poisson_channel::<u32>(MixerConfig::default());
        drop(rx);

        // The receiver's Drop flips the shared flag synchronously.
        let result = tx.send(1);
        assert!(
            matches!(result, Err(SenderError::Closed)),
            "send after receiver drop should return Closed, got {result:?}"
        );
        Ok(())
    }

    #[rstest]
    #[case::exactly_once(4_000)]
    #[case::unbounded_flood(20_000)]
    #[tokio::test]
    async fn all_sent_ids_should_be_delivered_exactly_once(#[case] n: u32) -> anyhow::Result<()> {
        // No loss, no duplication, no fabrication — including a flood the unbounded ingress must
        // accept without blocking.
        let (tx, rx) = poisson_channel::<u32>(MixerConfig::default());
        let drained = spawn_drain(rx, |x| x);

        for i in 0..n {
            tx.send(i).expect("unbounded ingress must always accept");
        }
        drop(tx);

        let mut got = tokio::time::timeout(Duration::from_secs(10), drained).await??;
        got.sort_unstable();
        assert_eq!(
            got,
            (0..n).collect::<Vec<_>>(),
            "every sent id must be delivered exactly once"
        );
        Ok(())
    }

    #[tokio::test]
    async fn concurrent_sender_clones_should_all_be_delivered() -> anyhow::Result<()> {
        // Many concurrent sender clones share one engine; the union of their sends must arrive
        // exactly once. Exercises concurrent lock-free ingress.
        const CLONES: u32 = 8;
        const PER: u32 = 500;
        let (tx, rx) = poisson_channel::<u32>(MixerConfig::default());
        let receiver = spawn_drain(rx, |x| x);

        let mut tasks = Vec::new();
        for c in 0..CLONES {
            let tx_clone = tx.clone();
            tasks.push(tokio::spawn(async move {
                for j in 0..PER {
                    tx_clone.send(c * PER + j).expect("send must succeed");
                }
                // `tx_clone` drops here.
            }));
        }
        drop(tx); // channel stays open until every clone task completes and drops its clone

        for t in tasks {
            t.await?;
        }

        let mut got = tokio::time::timeout(Duration::from_secs(10), receiver).await??;
        got.sort_unstable();
        assert_eq!(
            got,
            (0..CLONES * PER).collect::<Vec<_>>(),
            "all clones' items must be delivered exactly once"
        );
        Ok(())
    }

    #[tokio::test]
    async fn per_clone_delivery_should_be_fair() -> anyhow::Result<()> {
        // The engine treats all packets identically regardless of which sender clone produced
        // them, so no clone should be starved or systematically delayed more than another.
        const CLONES: usize = 6;
        const PER: usize = 800;
        let cfg = poisson_cfg(PoissonDelay::Mean(Duration::from_millis(10)), 0.999);
        let (tx, mut rx) = poisson_channel::<(usize, Instant)>(cfg);

        let receiver = tokio::spawn(async move {
            // Per-clone (count, summed delay ms).
            let mut stats = vec![(0usize, 0.0f64); CLONES];
            while let Some((clone_id, sent_at)) = rx.next().await {
                let e = &mut stats[clone_id];
                e.0 += 1;
                e.1 += sent_at.elapsed().as_secs_f64() * 1000.0;
            }
            stats
        });

        let mut tasks = Vec::new();
        for clone_id in 0..CLONES {
            let tx_clone = tx.clone();
            tasks.push(tokio::spawn(async move {
                for _ in 0..PER {
                    tx_clone.send((clone_id, Instant::now())).expect("send must succeed");
                }
            }));
        }
        drop(tx);
        for t in tasks {
            t.await?;
        }

        let stats = tokio::time::timeout(Duration::from_secs(15), receiver).await??;

        // No starvation: every clone's packets all arrive.
        for (clone_id, (count, _)) in stats.iter().enumerate() {
            assert_eq!(*count, PER, "clone {clone_id} delivered {count}/{PER}");
        }

        // Fair latency: per-clone mean delays cluster together (identical treatment).
        let means: Vec<f64> = stats.iter().map(|(c, sum)| sum / *c as f64).collect();
        let min_mean = means.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_mean = means.iter().cloned().fold(0.0_f64, f64::max);
        assert!(
            max_mean / min_mean < 4.0,
            "per-clone mean delays should be similar (min {min_mean:.2} ms, max {max_mean:.2} ms, means {means:?})"
        );
        Ok(())
    }

    #[tokio::test]
    async fn sender_should_not_be_blocked_by_the_engine() -> anyhow::Result<()> {
        let (tx, _rx) = poisson_channel::<u32>(MixerConfig::default());
        let start = std::time::Instant::now();
        tx.send(0)?;
        assert!(
            start.elapsed() < Duration::from_millis(50),
            "send must be a lock-free hand-off, took {:?}",
            start.elapsed()
        );
        Ok(())
    }
}
