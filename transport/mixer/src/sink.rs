use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use futures::{FutureExt, Sink};
use futures_timer::Delay;
use tracing::trace;

#[cfg(all(feature = "telemetry", not(test)))]
use crate::channel::{METRIC_MIXER_AVERAGE_DELAY, METRIC_QUEUE_SIZE};
use crate::{config::MixerConfig, data::DelayedData};

/// A [`Sink`] adapter that applies random delays to items before forwarding them to an inner sink.
///
/// Items pushed via `start_send` are held in an internal heap until their randomly-assigned
/// release time, then forwarded to the wrapped sink. `poll_flush` drains due items and parks
/// on a timer for the next pending item, so the owning task wakes up automatically when items
/// become ready — no separate forwarding task is required.
///
/// Cloning creates a fresh, empty sink that shares only the inner sink clone and configuration;
/// each clone maintains an independent delay heap.
pub struct MixerSink<S, T> {
    inner: S,
    heap: BinaryHeap<Reverse<DelayedData<T>>>,
    timer: Delay,
    cfg: MixerConfig,
}

impl<S, T> MixerSink<S, T> {
    pub fn new(inner: S, cfg: MixerConfig) -> Self {
        let mut heap = BinaryHeap::new();
        heap.reserve(cfg.capacity);
        Self {
            inner,
            heap,
            timer: Delay::new(Duration::ZERO),
            cfg,
        }
    }
}

impl<S: Clone, T> Clone for MixerSink<S, T> {
    fn clone(&self) -> Self {
        Self::new(self.inner.clone(), self.cfg)
    }
}

impl<S, T> Sink<T> for MixerSink<S, T>
where
    S: Sink<T> + Unpin,
    T: Unpin,
{
    type Error = S::Error;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        let this = Pin::into_inner(self);
        let random_delay = this.cfg.random_delay();

        trace!(delay_ms = random_delay.as_millis(), "mixer: delaying item");

        this.heap
            .push(Reverse(DelayedData::from((Instant::now() + random_delay, item))));

        #[cfg(all(feature = "telemetry", not(test)))]
        {
            METRIC_QUEUE_SIZE.increment(1.0f64);

            let weight = 1.0f64 / this.cfg.metric_delay_window as f64;
            METRIC_MIXER_AVERAGE_DELAY.set(
                (weight * random_delay.as_millis() as f64) + ((1.0f64 - weight) * METRIC_MIXER_AVERAGE_DELAY.get()),
            );
        }

        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = Pin::into_inner(self);

        loop {
            let now = Instant::now();

            while this.heap.peek().is_some_and(|x| x.0.release_at <= now) {
                let item = this.heap.pop().unwrap().0.item;

                match Pin::new(&mut this.inner).poll_ready(cx) {
                    Poll::Ready(Ok(())) => {}
                    Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                    Poll::Pending => {
                        this.heap.push(Reverse(DelayedData::from((now, item))));
                        break;
                    }
                }

                if let Err(e) = Pin::new(&mut this.inner).start_send(item) {
                    return Poll::Ready(Err(e));
                }

                #[cfg(all(feature = "telemetry", not(test)))]
                METRIC_QUEUE_SIZE.decrement(1.0f64);
            }

            futures::ready!(Pin::new(&mut this.inner).poll_flush(cx))?;

            if this.heap.is_empty() {
                return Poll::Ready(Ok(()));
            }

            let next_release = this.heap.peek().unwrap().0.release_at;
            let sleep_for = next_release.saturating_duration_since(now);

            if sleep_for.is_zero() {
                // Reachable only because inner.poll_ready returned Pending earlier in
                // this poll (we pushed the item back with release_at = now). The inner
                // sink already registered its waker via that poll_ready call, and any
                // items it had buffered were flushed by the poll_flush call above —
                // delegate to the lower sink's wake-up signal and yield.
                return Poll::Pending;
            }

            this.timer.reset(sleep_for);
            // Park until the timer fires; cx.waker() is registered inside poll_unpin.
            futures::ready!(this.timer.poll_unpin(cx));
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        futures::ready!(self.as_mut().poll_flush(cx))?;
        Pin::new(&mut Pin::into_inner(self).inner).poll_close(cx)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::{SinkExt, StreamExt, channel::mpsc};
    use tokio::time::timeout;

    use super::*;

    const LEEWAY: Duration = Duration::from_millis(200);

    #[tokio::test]
    async fn items_are_forwarded_after_delay() {
        let (tx, mut rx) = mpsc::channel::<u32>(100);
        let cfg = MixerConfig {
            min_delay: Duration::from_millis(50),
            delay_range: Duration::from_millis(10),
            capacity: 16,
            metric_delay_window: 100,
        };

        let mut sink = MixerSink::new(tx, cfg);

        sink.send(42u32).await.unwrap();

        let item = timeout(Duration::from_millis(60) + LEEWAY, rx.next())
            .await
            .expect("should receive within leeway")
            .unwrap();

        assert_eq!(item, 42);
    }

    #[tokio::test]
    async fn all_items_are_forwarded_with_zero_delay() {
        let (tx, mut rx) = mpsc::channel::<u32>(100);
        let cfg = MixerConfig {
            min_delay: Duration::from_millis(0),
            delay_range: Duration::from_millis(0),
            capacity: 16,
            metric_delay_window: 100,
        };

        let mut sink = MixerSink::new(tx, cfg);

        for i in 0u32..5 {
            sink.start_send_unpin(i).unwrap();
        }
        sink.flush().await.unwrap();

        let mut received = Vec::new();
        while let Ok(Some(item)) = timeout(Duration::from_millis(10), rx.next()).await {
            received.push(item);
        }

        assert_eq!(received.len(), 5, "all items should be forwarded");
    }

    #[tokio::test]
    async fn clone_starts_with_empty_heap() {
        let (tx, mut rx) = mpsc::channel::<u32>(100);
        let cfg = MixerConfig {
            min_delay: Duration::from_millis(50),
            delay_range: Duration::from_millis(10),
            capacity: 16,
            metric_delay_window: 100,
        };

        let mut sink = MixerSink::new(tx, cfg);
        sink.start_send_unpin(1u32).unwrap();

        let mut cloned = sink.clone();
        cloned.flush().await.unwrap();

        sink.flush().await.unwrap();

        let item = timeout(Duration::from_millis(60) + LEEWAY, rx.next())
            .await
            .expect("original item should arrive")
            .unwrap();
        assert_eq!(item, 1);

        assert!(timeout(Duration::from_millis(10), rx.next()).await.is_err());
    }

    /// Regression test for the `poll_flush` busy-spin bug.
    ///
    /// When `inner.poll_ready` returns `Poll::Pending` (inner sink is full) and
    /// `inner.poll_flush` returns `Poll::Ready(Ok(()))` (channels are no-op flush),
    /// the outer loop must return `Poll::Pending` instead of spinning.
    ///
    /// If this test hangs, the busy-spin bug has been re-introduced: the executor
    /// thread is pegged and the timeout future can never fire.
    #[tokio::test]
    async fn poll_flush_returns_pending_not_spin_when_inner_full() {
        use std::convert::Infallible;
        use futures::Sink;

        struct AlwaysFullNoOpFlushSink;

        impl Sink<u32> for AlwaysFullNoOpFlushSink {
            type Error = Infallible;

            fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Infallible>> {
                Poll::Pending // always full — simulates a saturated mpsc channel
            }

            fn start_send(self: Pin<&mut Self>, _: u32) -> Result<(), Infallible> {
                unreachable!("start_send must not be called when poll_ready returns Pending")
            }

            fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Infallible>> {
                Poll::Ready(Ok(())) // no-op flush — identical to futures::channel::mpsc
            }

            fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Infallible>> {
                Poll::Ready(Ok(()))
            }
        }

        let cfg = MixerConfig {
            min_delay: Duration::from_millis(0),
            delay_range: Duration::from_millis(0),
            capacity: 16,
            metric_delay_window: 100,
        };
        let mut sink = MixerSink::new(AlwaysFullNoOpFlushSink, cfg);
        sink.start_send_unpin(42u32).unwrap();

        // flush() must return Poll::Pending (inner not ready), allowing the timeout to fire.
        // If the spin bug is present, poll_flush loops forever on the single-threaded
        // executor and the timeout can never be polled — the test hangs.
        let result = timeout(Duration::from_millis(50), sink.flush()).await;
        assert!(
            result.is_err(),
            "flush should have returned Pending (inner is full) and triggered the timeout, \
             but it completed — inner should not have become ready"
        );
    }
}
