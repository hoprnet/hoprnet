//! Shared-pool Poisson mixer — an opt-in alternative to [`crate::poisson`], behind the
//! `poisson-shared` feature.
//!
//! Where [`crate::poisson`] owns the pool on a dedicated OS thread and relays packets in and out
//! over two `async-channel` queues, this variant keeps the pool behind an `Arc<Mutex<_>>` (like
//! the uniform `channel()`). Senders lock and push; the sweep and the adaptive timer run
//! on the **consumer's** `poll_next`. Removing both cross-thread hand-offs makes it markedly
//! faster (see `benches/poisson_shared_bench.rs`), with two tradeoffs: the mutex is held across
//! each O(N) sweep, and the mixing runs on the consumer's task rather than an isolated thread
//! (benign, given the cadence-independent release clock).
//!
//! The release logic is shared with [`crate::poisson`] via `pool::sweep`, so the mixing
//! behaviour (and its stochastic guarantees) is identical.

use std::{
    collections::VecDeque,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

use futures::{FutureExt, Stream, StreamExt};
use futures_timer::Delay;
use parking_lot::Mutex;

pub use crate::error::SenderError;
use crate::{
    config::MixerConfig,
    pool::{self, Entry, PoissonParams},
};

struct Pool<T> {
    entries: Vec<Entry<T>>,
    waker: Option<Waker>,
    /// Wall-clock instant of the previous sweep; the memoryless clock advances by `now - this`.
    prev_sweep: Instant,
}

struct Shared<T> {
    pool: Arc<Mutex<Pool<T>>>,
    sender_count: Arc<AtomicUsize>,
    receiver_active: Arc<AtomicBool>,
    params: PoissonParams,
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            sender_count: self.sender_count.clone(),
            receiver_active: self.receiver_active.clone(),
            params: self.params,
        }
    }
}

fn wake<T>(pool: &mut Pool<T>) {
    if let Some(w) = pool.waker.take() {
        w.wake();
    }
}

/// Sender end of the shared-pool mixer.
pub struct Sender<T> {
    shared: Shared<T>,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        self.shared.sender_count.fetch_add(1, Ordering::Relaxed);
        Self {
            shared: self.shared.clone(),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        // `AcqRel` so the receiver's `Acquire` load of a zero count synchronizes-with this
        // decrement (and thus with the push that preceded the sender's drop): without it, on a
        // weak memory model the receiver could observe `0`, find the pool empty, and close one
        // packet short.
        if self.shared.sender_count.fetch_sub(1, Ordering::AcqRel) == 1 {
            // Last sender gone: wake the receiver so it can observe closure and drain.
            wake(&mut self.shared.pool.lock());
        }
    }
}

impl<T> Sender<T> {
    /// Push one item into the shared pool (never blocks beyond the brief lock).
    pub fn send(&self, item: T) -> Result<(), SenderError> {
        let mut pool = self.shared.pool.lock();
        // Check the flag *under the lock* so it orders against `Receiver::drop`, which sets it
        // while holding the same lock. An unlocked check would race: a receiver dropping between
        // the check and the push would leave the item in a pool nothing will ever sweep, yet
        // `send` would still return `Ok` and the caller would record a lost packet as delivered.
        if !self.shared.receiver_active.load(Ordering::Relaxed) {
            return Err(SenderError::Closed);
        }
        pool.entries.push(Entry::new(Instant::now(), item));
        wake(&mut pool);
        Ok(())
    }
}

impl<T> futures::sink::Sink<T> for Sender<T> {
    type Error = SenderError;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // The pool is unbounded, so a send never needs to wait: always ready while the receiver
        // lives, matching the dedicated-thread engine's sender.
        if self.shared.receiver_active.load(Ordering::Relaxed) {
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

/// Receiver end of the shared-pool mixer. Runs the sweep + adaptive timer inline.
pub struct Receiver<T> {
    shared: Shared<T>,
    timer: Delay,
    /// Items released by the last sweep, yielded one per `poll_next`.
    ready: VecDeque<T>,
    /// Reused scratch for `sweep` output.
    scratch: Vec<(Duration, T)>,
}

// `T: Unpin` is required so `poll_next` can `get_mut()` the receiver: its buffered fields
// (`VecDeque<T>`/`Vec<(_, T)>`) only implement `Unpin` when `T` does under the current toolchain.
// The dedicated-thread `poisson::Receiver` avoids the bound because it box-pins its channel; here
// removing it would need `unsafe` pin-projection, not worth it for this opt-in engine.
impl<T: Unpin> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        loop {
            if let Some(item) = this.ready.pop_front() {
                return Poll::Ready(Some(item));
            }

            let now = Instant::now();
            // `Acquire` pairs with the sender's `AcqRel` decrement so observing `0` guarantees
            // every pushed entry is visible below.
            let no_senders = this.shared.sender_count.load(Ordering::Acquire) == 0;

            // Outcome of the sweep, decided under the lock but acted on after releasing it.
            enum Next {
                /// Items were released into `scratch`; move them out and pop.
                Released,
                /// No input can arrive and the pool is drained.
                Closed,
                /// Nothing ready; park the timer for this long.
                Sleep(Duration),
            }

            // Sweep under the lock, writing released items straight into the reused `scratch`
            // (a field disjoint from the pool guard). Draining `scratch` into `ready` and parking
            // the timer both happen *after* the guard drops, so senders never wait on that work.
            let next = {
                let mut pool = this.shared.pool.lock();
                let delta = now.saturating_duration_since(pool.prev_sweep);
                pool.prev_sweep = now;

                let earliest = pool::sweep(&mut pool.entries, &this.shared.params, now, delta, &mut this.scratch);

                #[cfg(all(feature = "telemetry", not(test)))]
                crate::metrics::METRIC_QUEUE_SIZE.set(pool.entries.len() as f64);

                if !this.scratch.is_empty() {
                    Next::Released
                } else if pool.entries.is_empty() && no_senders {
                    Next::Closed
                } else {
                    match pool.waker.as_mut() {
                        Some(w) => w.clone_from(cx.waker()),
                        None => pool.waker = Some(cx.waker().clone()),
                    }
                    Next::Sleep(pool::next_wake(earliest, pool.entries.len(), &this.shared.params, now))
                }
            };

            match next {
                Next::Released => {
                    for (_delay, item) in this.scratch.drain(..) {
                        // Feed realized delay into the same EMA gauge the dedicated engine uses,
                        // so switching to the shared engine doesn't flatline the dashboards.
                        #[cfg(all(feature = "telemetry", not(test)))]
                        crate::metrics::record_average_delay(
                            _delay.as_millis() as f64,
                            this.shared.params.metric_delay_window,
                        );
                        this.ready.push_back(item);
                    }
                    continue;
                }
                Next::Closed => {
                    this.shared.receiver_active.store(false, Ordering::Relaxed);
                    return Poll::Ready(None);
                }
                Next::Sleep(sleep_for) => {
                    this.timer.reset(sleep_for);
                    match this.timer.poll_unpin(cx) {
                        Poll::Ready(()) => continue, // timer already elapsed; sweep again
                        Poll::Pending => return Poll::Pending,
                    }
                }
            }
        }
    }
}

impl<T: Unpin> Receiver<T> {
    /// Receive a single mixed item.
    pub async fn recv(&mut self) -> Option<T> {
        self.next().await
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        // Set the flag under the pool lock so it orders against `Sender::send`'s re-check: a
        // sender either sees the pool still open (and its push will be swept before this drop
        // completes) or sees `false` and returns `Closed` — never a silently orphaned push.
        let _guard = self.shared.pool.lock();
        self.shared.receiver_active.store(false, Ordering::Relaxed);
    }
}

/// Instantiate a shared-pool Poisson mixing channel. No engine thread is spawned.
pub fn poisson_shared_channel<T: Send>(cfg: MixerConfig) -> (Sender<T>, Receiver<T>) {
    let params = PoissonParams::from_mixer(&cfg);
    let shared = Shared {
        pool: Arc::new(Mutex::new(Pool {
            entries: Vec::with_capacity(params.capacity()),
            waker: None,
            prev_sweep: Instant::now(),
        })),
        sender_count: Arc::new(AtomicUsize::new(1)),
        receiver_active: Arc::new(AtomicBool::new(true)),
        params,
    };
    (
        Sender { shared: shared.clone() },
        Receiver {
            shared,
            timer: Delay::new(pool::IDLE_HEARTBEAT),
            ready: VecDeque::new(),
            scratch: Vec::new(),
        },
    )
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;
    use tokio::time::timeout;

    use super::*;
    use crate::config::{MixerType, PoissonConfig};

    const CAP: Duration = Duration::from_millis(
        crate::config::HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS + crate::config::HOPR_MIXER_DEFAULT_DELAY_RANGE_IN_MS,
    );
    const LEEWAY: Duration = Duration::from_millis(500);

    /// Config selecting the shared-pool engine with an explicit mean and cap bounds.
    fn shared_cfg(min_delay: Duration, delay_range: Duration, target_mean_delay: Duration) -> MixerConfig {
        MixerConfig {
            min_delay,
            delay_range,
            mixer_type: MixerType::PoissonShared(PoissonConfig {
                target_mean_delay,
                ..PoissonConfig::default()
            }),
            ..MixerConfig::default()
        }
    }

    #[tokio::test]
    async fn shared_should_pass_an_element() -> anyhow::Result<()> {
        let (tx, mut rx) = poisson_shared_channel(MixerConfig::default());
        tx.send(1)?;
        assert_eq!(timeout(CAP + LEEWAY, rx.recv()).await?, Some(1));
        Ok(())
    }

    #[tokio::test]
    async fn shared_should_deliver_all_and_close() -> anyhow::Result<()> {
        const N: u32 = 2000;
        let (tx, mut rx) = poisson_shared_channel::<u32>(MixerConfig::default());
        for i in 0..N {
            tx.send(i)?;
        }
        drop(tx);

        let mut got = Vec::with_capacity(N as usize);
        while let Some(item) = timeout(CAP + LEEWAY, rx.next()).await? {
            got.push(item);
        }
        got.sort_unstable();
        assert_eq!(
            got,
            (0..N).collect::<Vec<_>>(),
            "all items delivered exactly once before close"
        );
        Ok(())
    }

    #[tokio::test]
    async fn shared_should_mix_under_load() -> anyhow::Result<()> {
        const N: usize = 3000;
        let cfg = shared_cfg(Duration::ZERO, Duration::from_millis(100), Duration::from_millis(10));
        let (tx, mut rx) = poisson_shared_channel::<(u32, Instant)>(cfg);

        let recv = tokio::spawn(async move {
            let mut delays = Vec::with_capacity(N);
            let mut max_seq: i64 = -1;
            let mut ooo = 0usize;
            while let Some((seq, sent)) = rx.next().await {
                delays.push(sent.elapsed().as_secs_f64() * 1000.0);
                if (seq as i64) < max_seq {
                    ooo += 1;
                }
                max_seq = max_seq.max(seq as i64);
            }
            (delays, ooo)
        });
        for seq in 0..N as u32 {
            tx.send((seq, Instant::now()))?;
        }
        drop(tx);

        let (delays, ooo) = tokio::time::timeout(Duration::from_secs(10), recv).await??;
        assert_eq!(delays.len(), N);
        let mean = delays.iter().sum::<f64>() / N as f64;
        assert!(mean > 3.0, "packets must be genuinely held (mean {mean:.2} ms)");
        assert!(ooo as f64 / N as f64 > 0.10, "output must be substantially mixed");
        Ok(())
    }

    #[tokio::test]
    async fn shared_send_should_fail_after_receiver_dropped() -> anyhow::Result<()> {
        let (tx, rx) = poisson_shared_channel::<u32>(MixerConfig::default());
        drop(rx);

        // `Receiver::drop` flips the flag under the pool lock, so this re-checks authoritatively.
        let result = tx.send(1);
        assert!(
            matches!(result, Err(SenderError::Closed)),
            "send after receiver drop should return Closed, got {result:?}"
        );
        Ok(())
    }

    #[tokio::test]
    async fn shared_should_stay_open_until_last_sender_clone_drops() -> anyhow::Result<()> {
        // Exercises the `sender_count` AcqRel/Acquire handshake: the channel must stay open while
        // any clone lives and close (yield `None`) only once the last one drops.
        let (tx_a, mut rx) = poisson_shared_channel::<u32>(MixerConfig::default());
        let tx_b = tx_a.clone();

        tx_a.send(1)?;
        tx_b.send(2)?;
        drop(tx_a);
        drop(tx_b);

        let mut got = vec![
            timeout(CAP + LEEWAY, rx.next()).await?.expect("first item"),
            timeout(CAP + LEEWAY, rx.next()).await?.expect("second item"),
        ];
        got.sort_unstable();
        assert_eq!(got, vec![1, 2]);
        assert!(
            rx.next().await.is_none(),
            "channel should close only after both sender clones drop"
        );
        Ok(())
    }

    #[tokio::test]
    async fn shared_passthrough_should_preserve_order() -> anyhow::Result<()> {
        const ITERATIONS: usize = 32;
        let (tx, rx) = poisson_shared_channel(MixerConfig {
            min_delay: Duration::ZERO,
            delay_range: Duration::ZERO,
            ..MixerConfig::default()
        });

        let input = (0..ITERATIONS as u32).collect::<Vec<_>>();
        for i in input.iter() {
            tx.send(*i)?;
            tokio::time::sleep(Duration::from_micros(50)).await;
        }

        let output = timeout(2 * CAP + LEEWAY, rx.take(ITERATIONS).collect::<Vec<_>>()).await?;
        assert_eq!(input, output, "pass-through must preserve FIFO order");
        Ok(())
    }

    #[tokio::test]
    async fn shared_sender_should_deliver_through_the_sink_api() -> anyhow::Result<()> {
        // The transport wires the sender as a `Sink`, so exercise that path (not just the
        // inherent `send`): `SinkExt::send` drives poll_ready -> start_send -> poll_flush.
        use futures::SinkExt;
        let (mut tx, mut rx) = poisson_shared_channel::<u32>(MixerConfig::default());
        SinkExt::send(&mut tx, 7).await?;
        assert_eq!(timeout(CAP + LEEWAY, rx.recv()).await?, Some(7));
        Ok(())
    }
}
