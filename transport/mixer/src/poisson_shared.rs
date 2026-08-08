//! Shared-pool Poisson mixer — an opt-in alternative to [`crate::poisson`], behind the
//! `poisson-shared` feature.
//!
//! Where [`crate::poisson`] owns the pool on a dedicated OS thread and relays packets in and out
//! over two `async-channel` queues, this variant keeps the pool behind an `Arc<Mutex<_>>` (like
//! the uniform [`crate::channel`]). Senders lock and push; the sweep and the adaptive timer run
//! on the **consumer's** `poll_next`. Removing both cross-thread hand-offs makes it markedly
//! faster (see `benches/poisson_shared_bench.rs`), with two tradeoffs: the mutex is held across
//! each O(N) sweep, and the mixing runs on the consumer's task rather than an isolated thread
//! (benign, given the cadence-independent release clock).
//!
//! The release logic is shared with [`crate::poisson`] via [`crate::pool::sweep`], so the mixing
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
    pool::{self, Entry},
};

const IDLE_HEARTBEAT: Duration = Duration::from_millis(200);
const MIN_WAKE: Duration = Duration::from_micros(100);

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
    cfg: MixerConfig,
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            sender_count: self.sender_count.clone(),
            receiver_active: self.receiver_active.clone(),
            cfg: self.cfg,
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
        if self.shared.sender_count.fetch_sub(1, Ordering::Relaxed) == 1 {
            // Last sender gone: wake the receiver so it can observe closure and drain.
            wake(&mut self.shared.pool.lock());
        }
    }
}

impl<T> Sender<T> {
    /// Push one item into the shared pool (never blocks beyond the brief lock).
    pub fn send(&self, item: T) -> Result<(), SenderError> {
        if !self.shared.receiver_active.load(Ordering::Relaxed) {
            return Err(SenderError::Closed);
        }
        let mut pool = self.shared.pool.lock();
        pool.entries.push(Entry::new(Instant::now(), item));
        wake(&mut pool);
        Ok(())
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

impl<T: Unpin> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        loop {
            if let Some(item) = this.ready.pop_front() {
                return Poll::Ready(Some(item));
            }

            let now = Instant::now();
            let no_senders = this.shared.sender_count.load(Ordering::Relaxed) == 0;

            // Sweep under the lock; move released items into the local `ready` queue. The lock is
            // released before parking on the timer so senders never wait on the timer poll.
            let sleep_for = {
                let mut pool = this.shared.pool.lock();
                let delta = now.saturating_duration_since(pool.prev_sweep);
                pool.prev_sweep = now;

                let mut scratch = std::mem::take(&mut this.scratch);
                let earliest = pool::sweep(&mut pool.entries, &this.shared.cfg, now, delta, &mut scratch);
                for (_delay, item) in scratch.drain(..) {
                    this.ready.push_back(item);
                }
                this.scratch = scratch;

                if !this.ready.is_empty() {
                    continue; // got items; drop lock and pop
                }
                if pool.entries.is_empty() && no_senders {
                    drop(pool);
                    this.shared.receiver_active.store(false, Ordering::Relaxed);
                    return Poll::Ready(None);
                }

                match pool.waker.as_mut() {
                    Some(w) => w.clone_from(cx.waker()),
                    None => pool.waker = Some(cx.waker().clone()),
                }
                next_wake(earliest, pool.entries.len(), &this.shared.cfg, now)
            };

            this.timer.reset(sleep_for);
            match this.timer.poll_unpin(cx) {
                Poll::Ready(()) => continue, // timer already elapsed; sweep again
                Poll::Pending => return Poll::Pending,
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
        self.shared.receiver_active.store(false, Ordering::Relaxed);
    }
}

/// Same wake policy as [`crate::poisson`]: adaptive interval capped by the soonest jitter window
/// opening, floored at [`MIN_WAKE`]; idle heartbeat when empty.
fn next_wake(earliest_enqueued: Option<Instant>, occupancy: usize, cfg: &MixerConfig, now: Instant) -> Duration {
    match earliest_enqueued {
        None => IDLE_HEARTBEAT,
        Some(enqueued_at) => {
            let interval = cfg.adaptive_interval(occupancy);
            let window_opens = cfg.cap().saturating_sub(cfg.cap_jitter);
            let deadline_wake = (enqueued_at + window_opens).saturating_duration_since(now);
            interval.min(deadline_wake).max(MIN_WAKE)
        }
    }
}

/// Instantiate a shared-pool Poisson mixing channel. No engine thread is spawned.
pub fn poisson_shared_channel<T: Send>(cfg: MixerConfig) -> (Sender<T>, Receiver<T>) {
    let shared = Shared {
        pool: Arc::new(Mutex::new(Pool {
            entries: Vec::with_capacity(cfg.capacity),
            waker: None,
            prev_sweep: Instant::now(),
        })),
        sender_count: Arc::new(AtomicUsize::new(1)),
        receiver_active: Arc::new(AtomicBool::new(true)),
        cfg,
    };
    (
        Sender { shared: shared.clone() },
        Receiver {
            shared,
            timer: Delay::new(IDLE_HEARTBEAT),
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

    const CAP: Duration = Duration::from_millis(
        crate::config::HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS + crate::config::HOPR_MIXER_DEFAULT_DELAY_RANGE_IN_MS,
    );
    const LEEWAY: Duration = Duration::from_millis(500);

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
        let cfg = MixerConfig {
            target_mean_delay: Duration::from_millis(10),
            min_delay: Duration::ZERO,
            delay_range: Duration::from_millis(100),
            ..MixerConfig::default()
        };
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
}
