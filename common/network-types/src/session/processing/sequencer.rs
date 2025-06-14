use std::{
    collections::BinaryHeap,
    pin::{Pin, pin},
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

use tracing::instrument;

use crate::session::{errors::SessionError, frames::FrameId};

#[derive(Copy, Clone, Debug)]
pub struct SequencerConfig {
    pub timeout: Duration,
    pub flush_at: usize,
    pub capacity: usize,
}

impl Default for SequencerConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            flush_at: 0,
            capacity: 1024,
        }
    }
}

#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
#[pin_project::pin_project]
pub struct Sequencer<T> {
    buffer: BinaryHeap<std::cmp::Reverse<T>>,
    next_id: FrameId,
    last_emitted: Instant,
    tx_waker: Option<Waker>,
    rx_waker: Option<Waker>,
    is_closed: bool,
    cfg: SequencerConfig,
}

impl<T: Ord> Sequencer<T> {
    pub fn new(cfg: SequencerConfig) -> Self {
        Self {
            buffer: BinaryHeap::with_capacity(cfg.capacity),
            next_id: 1,
            last_emitted: Instant::now(),
            tx_waker: None,
            rx_waker: None,
            is_closed: false,
            cfg,
        }
    }
}

impl<T> futures::Sink<T> for Sequencer<T>
where
    T: Ord + PartialOrd<FrameId>,
{
    type Error = SessionError;

    #[instrument(name = "Sequencer::poll_ready", level = "trace", skip(self, cx))]
    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        tracing::trace!("polling ready");
        if self.is_closed {
            return Poll::Ready(Err(SessionError::ReassemblerClosed));
        }

        tracing::trace!(len = self.buffer.len(), "buffer");

        if self.buffer.len() >= self.cfg.capacity {
            self.rx_waker = Some(cx.waker().clone());

            // Give the stream a chance to yield an element
            if let Some(waker) = self.tx_waker.take() {
                waker.wake();
            }

            tracing::trace!("pending");
            Poll::Pending
        } else {
            tracing::trace!(len = self.cfg.capacity - self.buffer.len(), "ready");
            Poll::Ready(Ok(()))
        }
    }

    #[instrument(name = "Sequencer::start_send", level = "trace", skip(self, item))]
    fn start_send(mut self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        tracing::trace!("starting send");
        if self.is_closed {
            return Err(SessionError::ReassemblerClosed);
        }

        if item.ge(&self.next_id) {
            self.buffer.push(std::cmp::Reverse(item));
            tracing::trace!("pushed new");
            if self.buffer.len() >= self.cfg.flush_at {
                if let Some(waker) = self.tx_waker.take() {
                    waker.wake();
                }
            }
        } else {
            tracing::warn!(next_id = self.next_id, "cannot accept old frame");
        }

        Ok(())
    }

    #[instrument(name = "Sequencer::poll_flush", level = "trace", skip(self))]
    fn poll_flush(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        tracing::trace!("polling flush");
        if self.is_closed {
            return Poll::Ready(Err(SessionError::ReassemblerClosed));
        }

        if let Some(waker) = self.tx_waker.take() {
            waker.wake();
        }

        Poll::Ready(Ok(()))
    }

    #[instrument(name = "Sequencer::poll_close", level = "trace", skip(self))]
    fn poll_close(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        tracing::trace!("polling close");
        if self.is_closed {
            return Poll::Ready(Err(SessionError::ReassemblerClosed));
        }

        self.is_closed = true;
        if let Some(waker) = self.tx_waker.take() {
            waker.wake();
        }
        if let Some(waker) = self.rx_waker.take() {
            waker.wake();
        }

        Poll::Ready(Ok(()))
    }
}

impl<T> futures::Stream for Sequencer<T>
where
    T: Ord + PartialEq<FrameId>,
{
    type Item = Result<T, SessionError>;

    #[instrument(name = "Sequencer::poll_next", level = "trace", skip(self, cx))]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        tracing::trace!("polling next");
        if self.is_closed && self.buffer.is_empty() {
            tracing::trace!("done");
            return Poll::Ready(None);
        }

        if let Some(current_item) = self.buffer.peek().map(|e| &e.0) {
            let current_to_emit = self.next_id;
            let is_next_ready = current_item.eq(&current_to_emit);

            if is_next_ready || self.last_emitted.elapsed() >= self.cfg.timeout {
                self.last_emitted = Instant::now();
                self.next_id += 1;

                return if is_next_ready {
                    let popped = self.buffer.pop().map(|r| Ok(r.0));
                    if let Some(waker) = self.rx_waker.take() {
                        waker.wake();
                    }
                    tracing::trace!(frame_id = current_to_emit, len = self.buffer.len(), "ready");
                    Poll::Ready(popped)
                } else {
                    tracing::trace!(frame_id = current_to_emit, "discard");
                    Poll::Ready(Some(Err(SessionError::FrameDiscarded(current_to_emit))))
                };
            }

            if self.is_closed && !is_next_ready {
                // If the Sink is closed, but the buffer is not empty,
                // we emit the missing ones as discarded frames until we
                // catch up with the rest of the buffered frames to flush out.
                self.next_id += 1;
                tracing::trace!(frame_id = current_to_emit, "discard");
                return Poll::Ready(Some(Err(SessionError::FrameDiscarded(current_to_emit))));
            }
        }

        // The next Sink operation will wake us up
        self.tx_waker = Some(cx.waker().clone());

        // ... but we need to give it a chance
        if let Some(waker) = self.rx_waker.take() {
            waker.wake();
        }

        tracing::trace!("pending");
        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use futures::{SinkExt, StreamExt, TryStreamExt, pin_mut};
    use futures_time::future::FutureExt;

    use super::*;

    #[test_log::test(tokio::test)]
    async fn sequencer_should_return_entries_in_order() -> anyhow::Result<()> {
        let cfg = SequencerConfig {
            timeout: Duration::from_secs(2),
            ..Default::default()
        };

        let (seq_sink, seq_stream) = Sequencer::<u32>::new(cfg).split();

        let mut expected = vec![4u32, 1, 5, 7, 8, 6, 2, 3];

        let jh = hopr_async_runtime::prelude::spawn(
            futures::stream::iter(expected.clone())
                .then(|e| futures::future::ok(e).delay(futures_time::time::Duration::from_millis(5)))
                .forward(seq_sink),
        );

        let actual: Vec<u32> = seq_stream
            .try_collect()
            .timeout(futures_time::time::Duration::from_secs(5))
            .await??;

        expected.sort();
        assert_eq!(expected, actual);

        let _ = jh.await?;
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn sequencer_should_return_entries_in_order_without_flush() -> anyhow::Result<()> {
        let cfg = SequencerConfig {
            timeout: Duration::from_secs(1),
            flush_at: usize::MAX,
            ..Default::default()
        };

        let (mut seq_sink, seq_stream) = Sequencer::<u32>::new(cfg).split();

        let mut expected = vec![4u32, 1, 5, 7, 8, 6, 2, 3];

        let expected_clone = expected.clone();
        let jh = hopr_async_runtime::prelude::spawn(async move {
            for v in expected_clone {
                seq_sink.feed(v).await?;
            }
            seq_sink.flush().await?;
            seq_sink.close().await
        });

        let actual: Vec<u32> = seq_stream
            .try_collect()
            .timeout(futures_time::time::Duration::from_secs(5))
            .await??;

        expected.sort();
        assert_eq!(expected, actual);

        let _ = jh.await?;
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn sequencer_should_not_allow_emitted_entries() -> anyhow::Result<()> {
        let cfg = SequencerConfig::default();

        let (seq_sink, seq_stream) = Sequencer::<u32>::new(cfg).split();

        pin_mut!(seq_sink);
        pin_mut!(seq_stream);

        seq_sink.send(1u32).await?;
        assert_eq!(Some(1), seq_stream.try_next().await?);

        seq_sink.send(2u32).await?;
        assert_eq!(Some(2), seq_stream.try_next().await?);

        seq_sink.send(2u32).await?;
        seq_sink.send(1u32).await?;

        seq_sink.send(3u32).await?;
        assert_eq!(Some(3), seq_stream.try_next().await?);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn sequencer_should_discard_entry_on_timeout() -> anyhow::Result<()> {
        let cfg = SequencerConfig {
            timeout: Duration::from_millis(25),
            ..Default::default()
        };

        let (mut seq_sink, seq_stream) = Sequencer::<u32>::new(cfg).split();

        let input = vec![2u32, 1, 4, 5, 8, 7, 9, 11, 10];

        let input_clone = input.clone();
        let jh = hopr_async_runtime::prelude::spawn(async move {
            for v in input_clone {
                seq_sink
                    .feed(v)
                    .delay(futures_time::time::Duration::from_millis(5))
                    .await?;
            }
            seq_sink.flush().await?;
            seq_sink.close().await
        });

        pin_mut!(seq_stream);

        assert_eq!(Some(1), seq_stream.try_next().await?);
        assert_eq!(Some(2), seq_stream.try_next().await?);

        let now = Instant::now();
        assert!(matches!(
            seq_stream.try_next().await,
            Err(SessionError::FrameDiscarded(3))
        ));
        assert!(now.elapsed() >= cfg.timeout);

        assert_eq!(Some(4), seq_stream.try_next().await?);
        assert_eq!(Some(5), seq_stream.try_next().await?);

        assert!(matches!(
            seq_stream.try_next().await,
            Err(SessionError::FrameDiscarded(6))
        ));

        assert_eq!(Some(7), seq_stream.try_next().await?);
        assert_eq!(Some(8), seq_stream.try_next().await?);
        assert_eq!(Some(9), seq_stream.try_next().await?);
        assert_eq!(Some(10), seq_stream.try_next().await?);
        assert_eq!(Some(11), seq_stream.try_next().await?);

        assert_eq!(None, seq_stream.try_next().await?);

        let _ = jh.await?;
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn sequencer_should_discard_entry_close() -> anyhow::Result<()> {
        let cfg = SequencerConfig {
            timeout: Duration::from_millis(25),
            ..Default::default()
        };

        let (seq_sink, seq_stream) = Sequencer::<u32>::new(cfg).split();

        let input = vec![2u32, 1, 3, 5, 4, 8, 11];

        hopr_async_runtime::prelude::spawn(futures::stream::iter(input.clone()).map(Ok).forward(seq_sink)).await??;

        pin_mut!(seq_stream);

        assert_eq!(Some(1), seq_stream.try_next().await?);
        assert_eq!(Some(2), seq_stream.try_next().await?);
        assert_eq!(Some(3), seq_stream.try_next().await?);
        assert_eq!(Some(4), seq_stream.try_next().await?);
        assert_eq!(Some(5), seq_stream.try_next().await?);
        assert!(matches!(
            seq_stream.try_next().await,
            Err(SessionError::FrameDiscarded(6))
        ));
        assert!(matches!(
            seq_stream.try_next().await,
            Err(SessionError::FrameDiscarded(7))
        ));
        assert_eq!(Some(8), seq_stream.try_next().await?);
        assert!(matches!(
            seq_stream.try_next().await,
            Err(SessionError::FrameDiscarded(9))
        ));
        assert!(matches!(
            seq_stream.try_next().await,
            Err(SessionError::FrameDiscarded(10))
        ));
        assert_eq!(Some(11), seq_stream.try_next().await?);
        assert_eq!(None, seq_stream.try_next().await?);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn sequencer_should_wait_if_full() -> anyhow::Result<()> {
        let cfg = SequencerConfig {
            timeout: Duration::from_millis(25),
            capacity: 3,
            flush_at: 10,
            ..Default::default()
        };

        let (seq_sink, seq_stream) = Sequencer::<u32>::new(cfg).split();
        pin_mut!(seq_sink);
        pin_mut!(seq_stream);

        seq_sink.send(1u32).await?;
        seq_sink.send(3u32).await?;
        seq_sink.send(2u32).await?;

        // Full capacity is reached here, but SplitSink has
        // an extra slot, so the 4th will wait in the SplitSink's slot.
        assert!(
            seq_sink
                .send(4u32)
                .timeout(futures_time::time::Duration::from_millis(50))
                .await
                .is_err()
        );

        assert_eq!(Some(1), seq_stream.try_next().await?);

        // This inserts 4 into the sequencer and keeps 6 in the slot in the SplitSink
        assert!(
            seq_sink
                .send(6u32)
                .timeout(futures_time::time::Duration::from_millis(50))
                .await
                .is_err()
        );

        assert_eq!(Some(2), seq_stream.try_next().await?);
        assert_eq!(Some(3), seq_stream.try_next().await?);

        // Since two elements were yielded from the Sequencer,
        // there's now space for both: 6 (from the SplitSink's slot) and 5 (new insert)
        seq_sink.send(5u32).await?;
        seq_sink.close().await?;

        assert_eq!(Some(4), seq_stream.try_next().await?);
        assert_eq!(Some(5), seq_stream.try_next().await?);
        assert_eq!(Some(6), seq_stream.try_next().await?);
        assert_eq!(None, seq_stream.try_next().await?);

        Ok(())
    }
}
