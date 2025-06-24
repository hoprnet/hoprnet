use std::{
    collections::BinaryHeap,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use futures_time::future::Timer;
use tracing::instrument;

use crate::{prelude::errors::SessionError, session::frames::FrameId};

#[must_use = "streams do nothing unless polled"]
#[pin_project::pin_project]
pub struct Sequencer<S: futures::Stream> {
    #[pin]
    inner: S,
    #[pin]
    timer: futures_time::task::Sleep,
    buffer: BinaryHeap<std::cmp::Reverse<S::Item>>,
    next_id: FrameId,
    last_emitted: Instant,
    max_wait: Duration,
    is_closed: bool,
}

impl<S> Sequencer<S>
where
    S: futures::Stream,
    S::Item: Ord + PartialOrd<FrameId>,
{
    fn new(inner: S, max_wait: Duration, capacity: usize) -> Self {
        assert!(capacity > 0, "capacity should be positive");
        Self {
            inner,
            buffer: BinaryHeap::with_capacity(capacity),
            timer: futures_time::task::sleep(max_wait.into()),
            next_id: 1,
            last_emitted: Instant::now(),
            max_wait,
            is_closed: false,
        }
    }
}

impl<S> futures::Stream for Sequencer<S>
where
    S: futures::Stream,
    S::Item: Ord + PartialOrd<FrameId>,
{
    type Item = Result<S::Item, SessionError>;

    #[instrument(name = "Sequencer::poll_next", level = "trace", skip(self, cx), fields(next_frame_id = self.next_id))]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        loop {
            if this.buffer.len() < this.buffer.capacity() && !*this.is_closed {
                // Poll the timer only if the buffer is not empty
                let timer_poll = if !this.buffer.is_empty() {
                    this.timer.as_mut().poll(cx)
                } else {
                    Poll::Pending
                };

                match (this.inner.as_mut().poll_next(cx), timer_poll) {
                    (Poll::Ready(Some(next)), timer) => {
                        if timer.is_ready() {
                            this.timer.as_mut().reset_timer();
                            tracing::trace!("new frame and timer reset");
                        }
                        if next < *this.next_id {
                            tracing::trace!("old frame skipped");
                            continue;
                        }
                        this.buffer.push(std::cmp::Reverse(next));
                        tracing::trace!("new frame");
                    }
                    (Poll::Ready(None), _) => *this.is_closed = true,
                    (Poll::Pending, Poll::Ready(_)) => {
                        this.timer.as_mut().reset_timer();
                        tracing::trace!("timer reset");
                    }
                    (Poll::Pending, Poll::Pending) => return Poll::Pending,
                }
            }

            if let Some(next) = this.buffer.peek().map(|next| &next.0) {
                if next.eq(this.next_id) {
                    *this.next_id += 1;
                    *this.last_emitted = Instant::now();

                    tracing::trace!("emit next frame");
                    return Poll::Ready(this.buffer.pop().map(|item| Ok(item.0)));
                } else if this.last_emitted.elapsed() >= *this.max_wait
                    || *this.is_closed
                    || this.buffer.len() == this.buffer.capacity()
                {
                    let discarded = *this.next_id;
                    *this.next_id += 1;
                    *this.last_emitted = Instant::now();

                    tracing::trace!("discard frame");
                    return Poll::Ready(Some(Err(SessionError::FrameDiscarded(discarded))));
                }
            } else if *this.is_closed {
                return Poll::Ready(None);
            }
        }
    }
}

pub trait SequencerExt: futures::Stream {
    fn sequencer(self, timeout: Duration, capacity: usize) -> Sequencer<Self>
    where
        Self::Item: Ord + PartialOrd<FrameId>,
        Self: Sized,
    {
        Sequencer::new(self, timeout, capacity)
    }
}

impl<T: ?Sized> SequencerExt for T where T: futures::Stream {}

#[cfg(test)]
mod tests {
    use futures::{SinkExt, StreamExt, TryStreamExt, pin_mut};
    use futures_time::future::FutureExt;

    use super::*;

    #[test_log::test(tokio::test)]
    async fn sequencer_should_return_entries_in_order() -> anyhow::Result<()> {
        let mut expected = vec![4u32, 1, 5, 7, 8, 6, 2, 3];

        let actual: Vec<u32> = futures::stream::iter(expected.clone())
            .sequencer(Duration::from_secs(5), 4096)
            .try_collect()
            .timeout(futures_time::time::Duration::from_secs(5))
            .await??;

        expected.sort();
        assert_eq!(expected, actual);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn sequencer_should_not_allow_emitted_entries() -> anyhow::Result<()> {
        let (seq_sink, seq_stream) = futures::channel::mpsc::unbounded();

        let seq_stream = seq_stream.sequencer(Duration::from_secs(1), 4096);

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
        let timeout = Duration::from_millis(25);
        let (mut seq_sink, seq_stream) = futures::channel::mpsc::unbounded();

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

        let seq_stream = seq_stream.sequencer(timeout, 4096);

        pin_mut!(seq_stream);

        assert_eq!(Some(1), seq_stream.try_next().await?);
        assert_eq!(Some(2), seq_stream.try_next().await?);

        let now = Instant::now();
        assert!(matches!(
            seq_stream.try_next().await,
            Err(SessionError::FrameDiscarded(3))
        ));
        assert!(now.elapsed() >= timeout);

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
        let (seq_sink, seq_stream) = futures::channel::mpsc::unbounded();

        let input = vec![2u32, 1, 3, 5, 4, 8, 11];

        hopr_async_runtime::prelude::spawn(futures::stream::iter(input.clone()).map(Ok).forward(seq_sink)).await??;

        let seq_stream = seq_stream.sequencer(Duration::from_millis(25), 4096);

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
    async fn sequencer_should_discard_entry_when_inner_stream_pending() -> anyhow::Result<()> {
        let sent = vec![4u32, 1, 7, 8, 6, 2, 3];
        let (tx, rx) = futures::channel::mpsc::unbounded();

        pin_mut!(tx);
        tx.send_all(&mut futures::stream::iter(sent.clone()).map(Ok)).await?;

        let rx = rx.sequencer(Duration::from_millis(10), 4096);
        pin_mut!(rx);

        assert!(matches!(rx.next().await, Some(Ok(1))));
        assert!(matches!(rx.next().await, Some(Ok(2))));
        assert!(matches!(rx.next().await, Some(Ok(3))));
        assert!(matches!(rx.next().await, Some(Ok(4))));
        assert!(matches!(rx.next().await, Some(Err(SessionError::FrameDiscarded(5)))));
        assert!(matches!(rx.next().await, Some(Ok(6))));
        assert!(matches!(rx.next().await, Some(Ok(7))));
        assert!(matches!(rx.next().await, Some(Ok(8))));

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn sequencer_should_discard_entry_when_capacity_is_reached() -> anyhow::Result<()> {
        let sent = vec![4u32, 5, 7, 8, 2, 6, 3];
        let (tx, rx) = futures::channel::mpsc::unbounded();

        pin_mut!(tx);
        tx.send_all(&mut futures::stream::iter(sent.clone()).map(Ok)).await?;

        let rx = rx.sequencer(Duration::from_millis(10), 4);
        pin_mut!(rx);

        assert!(matches!(rx.next().await, Some(Err(SessionError::FrameDiscarded(1)))));
        assert!(matches!(rx.next().await, Some(Err(SessionError::FrameDiscarded(2)))));
        assert!(matches!(rx.next().await, Some(Err(SessionError::FrameDiscarded(3)))));
        assert!(matches!(rx.next().await, Some(Ok(4))));
        assert!(matches!(rx.next().await, Some(Ok(5))));
        assert!(matches!(rx.next().await, Some(Ok(6))));
        assert!(matches!(rx.next().await, Some(Ok(7))));
        assert!(matches!(rx.next().await, Some(Ok(8))));

        Ok(())
    }
}
