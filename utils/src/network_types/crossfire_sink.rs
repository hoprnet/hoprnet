use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::StreamExt;

type PendingFut<T> = Pin<Box<dyn Future<Output = Result<(), crossfire::SendError<T>>> + Send + 'static>>;

/// Error type for [`CrossfireSink`].
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum CrossfireSinkError {
    /// The channel receiver has been dropped.
    #[error("channel receiver disconnected")]
    Disconnected,
}

/// A [`futures::Sink`] backed by a bounded crossfire MPSC channel sender.
///
/// Enforces strict capacity: unlike `futures::channel::mpsc::Sender`, cloning this sink
/// does **not** add extra slots to the channel. When the channel is full, `poll_ready`
/// returns `Poll::Pending` and registers a waker, allowing combinators such as
/// [`crate::network_types::timeout::TimeoutSink`] to arm their timers correctly.
///
/// Construct via [`bounded_sink_channel`].
pub struct CrossfireSink<T: 'static> {
    tx: crossfire::MAsyncTx<crossfire::mpsc::Array<T>>,
    /// Item buffered by `start_send`, awaiting forwarding to the channel.
    buffered: Option<T>,
    /// In-flight send future created when `try_send` returned `Full` during `poll_ready`.
    ///
    /// This is what makes `poll_ready` return `Pending` when the channel is saturated,
    /// which is necessary for `TimeoutSink::poll_ready` to arm its deadline timer.
    pending: Option<PendingFut<T>>,
}

// CrossfireSink does not use structural pinning — no field is accessed exclusively through
// Pin<&mut CrossfireSink<T>>. Moving a CrossfireSink after pinning is safe.
impl<T: 'static> Unpin for CrossfireSink<T> {}

// SAFETY: All &self methods (try_send, clone) only access `tx: MAsyncTx<Array<T>>` which
// is Sync. The `pending` and `buffered` fields are accessed exclusively via &mut self
// (Sink trait methods), making concurrent &self access free of data races.
unsafe impl<T: Send + 'static> Sync for CrossfireSink<T> {}

impl<T: 'static> std::fmt::Debug for CrossfireSink<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CrossfireSink")
            .field("buffered", &self.buffered.is_some())
            .field("pending", &self.pending.is_some())
            .finish_non_exhaustive()
    }
}

impl<T: Send + 'static> Clone for CrossfireSink<T> {
    fn clone(&self) -> Self {
        // Each clone starts idle — no buffered item and no in-flight future.
        // Cloning does not expand channel capacity.
        Self {
            tx: self.tx.clone(),
            buffered: None,
            pending: None,
        }
    }
}

impl<T: 'static> CrossfireSink<T> {
    /// Attempts to send `item` without waiting for channel space.
    ///
    /// Returns `Err(TrySendError::Full(item))` when the channel is at capacity,
    /// or `Err(TrySendError::Disconnected(item))` when all receivers have been dropped.
    /// Never inflates channel capacity — capacity is strict regardless of clone count.
    pub fn try_send(&self, item: T) -> Result<(), crossfire::TrySendError<T>> {
        self.tx.try_send(item)
    }
}

impl<T: Send + Unpin + 'static> futures::Sink<T> for CrossfireSink<T> {
    type Error = CrossfireSinkError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.get_mut();
        loop {
            if let Some(mut fut) = this.pending.take() {
                // Poll the in-flight send future. While it is Pending the channel is full,
                // so we return Pending here — this is the waker registration point for
                // TimeoutSink.
                match fut.as_mut().poll(cx) {
                    Poll::Ready(Ok(())) => return Poll::Ready(Ok(())),
                    Poll::Ready(Err(_)) => return Poll::Ready(Err(CrossfireSinkError::Disconnected)),
                    Poll::Pending => {
                        this.pending = Some(fut);
                        return Poll::Pending;
                    }
                }
            } else if let Some(item) = this.buffered.take() {
                // An item was stored by `start_send`. Try to place it in the channel.
                match this.tx.try_send(item) {
                    Ok(()) => return Poll::Ready(Ok(())),
                    Err(crossfire::TrySendError::Full(item)) => {
                        // Channel is full. Create an owned send future so the waker from the
                        // next poll_ready call is registered with the crossfire scheduler.
                        let tx = this.tx.clone();
                        this.pending = Some(Box::pin(async move { tx.send(item).await }));
                        // Loop once more to immediately poll the new future with the current
                        // waker, so we don't miss a wake-up that arrived between try_send
                        // and now.
                    }
                    Err(crossfire::TrySendError::Disconnected(_)) => {
                        return Poll::Ready(Err(CrossfireSinkError::Disconnected));
                    }
                }
            } else {
                return Poll::Ready(Ok(()));
            }
        }
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        let this = self.get_mut();
        debug_assert!(
            this.buffered.is_none() && this.pending.is_none(),
            "start_send called without a preceding successful poll_ready"
        );
        this.buffered = Some(item);
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Drive buffered/pending to completion.
        self.poll_ready(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Flush the remaining buffered item. The channel closes automatically when
        // all CrossfireSink clones are dropped.
        self.poll_flush(cx)
    }
}

/// Creates a strict-capacity bounded MPSC channel returning a [`CrossfireSink`] sender
/// and a [`futures::Stream`] receiver.
///
/// The channel holds at most `capacity` items at any time, independent of how many times
/// the sender is cloned. This contrasts with `futures::channel::mpsc::channel(N)`, where
/// each live `Sender` clone silently adds one extra slot.
pub fn bounded_sink_channel<T: Send + Unpin + 'static>(
    capacity: usize,
) -> (CrossfireSink<T>, futures::stream::BoxStream<'static, T>) {
    let (tx, rx) = crossfire::mpsc::bounded_async::<T>(capacity);
    let sink = CrossfireSink {
        tx,
        buffered: None,
        pending: None,
    };
    (sink, rx.into_stream().boxed())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use anyhow::Context as _;
    use futures::SinkExt;

    use super::*;
    use crate::network_types::timeout::{SinkTimeoutError, TimeoutSinkExt};

    #[test_log::test(tokio::test)]
    async fn capacity_should_be_buffer_not_buffer_plus_clones() -> anyhow::Result<()> {
        let capacity = 4usize;
        let (tx, _rx) = bounded_sink_channel::<u32>(capacity);

        // Clone the sender many times — this inflates capacity in futures::mpsc but not here.
        let _clones: Vec<_> = (0..100).map(|_| tx.clone()).collect();

        // Fill exactly `capacity` items via a separate clone so `tx` stays idle.
        let mut filler = tx.clone();
        for i in 0..capacity as u32 {
            filler.send(i).await.context("send within capacity")?;
        }

        // A send beyond capacity must block. We race it against a short sleep: if the
        // sleep wins, the send correctly blocked; if the send wins first, capacity grew.
        let mut overflow = tx.clone();
        tokio::select! {
            result = overflow.send(999u32) => {
                anyhow::bail!(
                    "send on full channel should block regardless of clone count, got: {result:?}"
                );
            }
            _ = tokio::time::sleep(Duration::from_millis(20)) => {
                // Correct: send blocked because capacity is strictly 4, not 4 + clones.
            }
        }

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn sink_should_return_error_when_receiver_dropped() -> anyhow::Result<()> {
        let (mut tx, rx) = bounded_sink_channel::<u32>(4);
        drop(rx);

        let result = tx.send(42u32).await;
        assert!(
            matches!(result, Err(CrossfireSinkError::Disconnected)),
            "expected Disconnected, got {result:?}"
        );

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn with_timeout_should_fire_when_channel_is_full() -> anyhow::Result<()> {
        let capacity = 2usize;
        let (mut tx, _rx) = bounded_sink_channel::<u32>(capacity);

        // Fill to capacity; _rx is held but not polled, so the channel stays full.
        for i in 0..capacity as u32 {
            tx.send(i).await.context("send within capacity")?;
        }

        // TimeoutSink::poll_ready fires only when inner poll_ready returns Pending.
        // CrossfireSink::poll_ready returns Pending only when it holds a buffered item
        // that cannot be sent (channel full).
        //
        // `feed` calls poll_ready then start_send but does NOT flush — so the first
        // feed stores 999 in CrossfireSink's buffer without blocking (poll_ready sees
        // an empty sink → Ready). The second feed's poll_ready finds the buffered item,
        // attempts try_send → Full → Pending, allowing TimeoutSink to arm its timer.
        let mut timed_tx = (&mut tx).with_timeout(Duration::from_millis(10));
        timed_tx.feed(999u32).await.context("first feed should succeed")?;
        let result = timed_tx.feed(998u32).await;
        assert!(
            matches!(result, Err(SinkTimeoutError::Timeout)),
            "expected Timeout when channel is full and sink has buffered item, got {result:?}"
        );

        Ok(())
    }
}
