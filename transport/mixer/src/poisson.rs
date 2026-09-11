//! Shared-pool virtual-clock timing-wheel mixer.
//!
//! The pool lives behind an `Arc<Mutex<_>>` and the sweep + wake timer run inline on the
//! consumer's `poll_next` — no dedicated thread, no cross-thread channel hand-off. The pool is a
//! flat `Vec` (see the internal `pool` module for why not a heap); the tradeoff for the dropped
//! thread isolation is holding the mutex across each sweep and mixing on the consumer task rather
//! than an isolated thread.

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
    pool::{self, Entry, PoissonParams, VirtualClock},
};

struct Pool<T> {
    entries: Vec<Entry<T>>,
    waker: Option<Waker>,
    clock: VirtualClock,
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
        let now = Instant::now();
        // Reborrow through the guard's `DerefMut` once, up front: splitting `pool.entries` and
        // `pool.clock` into two disjoint `&mut` requires a single concrete `&mut Pool<T>` — two
        // separate `&mut pool.field` expressions each re-invoke `DerefMut` on the guard itself and
        // the borrow checker can't see they're disjoint through that indirection.
        let pool: &mut Pool<T> = &mut pool;
        pool::enqueue(&mut pool.entries, &mut pool.clock, &self.shared.params, now, now, item);
        wake(pool);
        Ok(())
    }
}

impl<T> futures::sink::Sink<T> for Sender<T> {
    type Error = SenderError;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Unbounded pool: always ready while the receiver lives.
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
// Removing it would need `unsafe` pin-projection, not worth it here.
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

            // Sweep outcome, decided under the lock and acted on after it drops.
            enum Next {
                Released,
                Closed,
                Sleep(Duration),
            }

            // Sweep under the lock, writing released items straight into the reused `scratch`
            // (a field disjoint from the pool guard). Draining `scratch` into `ready` and parking
            // the timer both happen *after* the guard drops, so senders never wait on that work.
            let next = {
                let mut pool = this.shared.pool.lock();
                // See the matching comment in `Sender::send`: split-borrowing `entries` and
                // `clock` needs one concrete `&mut Pool<T>`, not two separate derefs of the guard.
                let pool: &mut Pool<T> = &mut pool;

                let earliest_v_release = pool::sweep(
                    &mut pool.entries,
                    &mut pool.clock,
                    &this.shared.params,
                    now,
                    &mut this.scratch,
                );

                #[cfg(all(feature = "telemetry", not(test)))]
                {
                    let len = pool.entries.len();
                    crate::metrics::METRIC_QUEUE_SIZE.set(len as f64);
                    crate::metrics::record_anonymity_set(len);
                }

                if !this.scratch.is_empty() {
                    Next::Released
                } else if pool.entries.is_empty() && no_senders {
                    Next::Closed
                } else {
                    match pool.waker.as_mut() {
                        Some(w) => w.clone_from(cx.waker()),
                        None => pool.waker = Some(cx.waker().clone()),
                    }
                    Next::Sleep(pool::next_wake(earliest_v_release, &pool.clock, &this.shared.params))
                }
            };

            match next {
                Next::Released => {
                    for (delay, item) in this.scratch.drain(..) {
                        tracing::trace!(delay_ms = delay.as_millis() as u64, "mixer released packet");
                        #[cfg(all(feature = "telemetry", not(test)))]
                        {
                            crate::metrics::record_packet_delay(
                                delay.as_millis() as f64,
                                this.shared.params.metric_delay_window,
                            );
                            crate::metrics::record_window_miss(delay > this.shared.params.max_delay());
                        }
                        #[cfg(not(all(feature = "telemetry", not(test))))]
                        let _ = delay;
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

/// Instantiate a shared-pool timing-wheel mixing channel. No engine thread is spawned.
pub fn poisson_channel<T: Send>(cfg: MixerConfig) -> (Sender<T>, Receiver<T>) {
    let params = PoissonParams::from_mixer(&cfg);
    let shared = Shared {
        pool: Arc::new(Mutex::new(Pool {
            entries: Vec::with_capacity(params.capacity()),
            waker: None,
            clock: VirtualClock::new(Instant::now()),
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

    const CAP: Duration = Duration::from_millis(crate::config::HOPR_MIXER_DEFAULT_MAX_DELAY_IN_MS);
    const LEEWAY: Duration = Duration::from_millis(500);

    #[tokio::test]
    async fn should_pass_an_element() -> anyhow::Result<()> {
        let (tx, mut rx) = poisson_channel(MixerConfig::default());
        tx.send(1)?;
        assert_eq!(timeout(CAP + LEEWAY, rx.recv()).await?, Some(1));
        Ok(())
    }

    #[tokio::test]
    async fn should_deliver_all_and_close() -> anyhow::Result<()> {
        const N: u32 = 2000;
        let (tx, mut rx) = poisson_channel::<u32>(MixerConfig::default());
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
    async fn should_mix_under_load() -> anyhow::Result<()> {
        const N: usize = 3000;
        let cfg = MixerConfig::new_poisson(Duration::from_millis(50), 0.01);
        let (tx, mut rx) = poisson_channel::<(u32, Instant)>(cfg);

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
    async fn send_should_fail_after_receiver_dropped() -> anyhow::Result<()> {
        let (tx, rx) = poisson_channel::<u32>(MixerConfig::default());
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
    async fn should_stay_open_until_last_sender_clone_drops() -> anyhow::Result<()> {
        // Exercises the `sender_count` AcqRel/Acquire handshake: the channel must stay open while
        // any clone lives and close (yield `None`) only once the last one drops.
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
        got.sort_unstable();
        assert_eq!(got, vec![1, 2]);
        assert!(
            rx.next().await.is_none(),
            "channel should close only after both sender clones drop"
        );
        Ok(())
    }

    #[tokio::test]
    async fn passthrough_should_preserve_order() -> anyhow::Result<()> {
        const ITERATIONS: usize = 32;
        let (tx, rx) = poisson_channel(MixerConfig::new_poisson(Duration::ZERO, 0.01));

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
    async fn sender_should_deliver_through_the_sink_api() -> anyhow::Result<()> {
        // The transport wires the sender as a `Sink`, so exercise that path (not just the
        // inherent `send`): `SinkExt::send` drives poll_ready -> start_send -> poll_flush.
        use futures::SinkExt;
        let (mut tx, mut rx) = poisson_channel::<u32>(MixerConfig::default());
        SinkExt::send(&mut tx, 7).await?;
        assert_eq!(timeout(CAP + LEEWAY, rx.recv()).await?, Some(7));
        Ok(())
    }
}
