use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use futures_core::Stream;
use pin_project_lite::pin_project;

use crate::FuturesUnorderedBounded;

pin_project!(
    /// Stream for the [`buffered_unordered`](crate::BufferedStreamExt::buffered_unordered)
    /// method.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures::channel::oneshot;
    /// use futures::stream::{self, StreamExt};
    /// use futures_buffered::BufferedStreamExt;
    ///
    /// let (send_one, recv_one) = oneshot::channel();
    /// let (send_two, recv_two) = oneshot::channel();
    ///
    /// let stream_of_futures = stream::iter(vec![recv_one, recv_two]);
    /// let mut buffered = stream_of_futures.buffered_unordered(10);
    ///
    /// send_two.send(2i32)?;
    /// assert_eq!(buffered.next().await, Some(Ok(2i32)));
    ///
    /// send_one.send(1i32)?;
    /// assert_eq!(buffered.next().await, Some(Ok(1i32)));
    ///
    /// assert_eq!(buffered.next().await, None);
    /// # Ok::<(), i32>(()) }).unwrap();
    /// ```
    ///
    /// ## Benchmarks
    ///
    /// ### Speed
    ///
    /// Running 65536 100us timers with 256 concurrent jobs in a single threaded tokio runtime:
    ///
    /// ```text
    /// futures::stream::BufferUnordered    time:   [420.33 ms 422.57 ms 424.83 ms]
    /// futures_buffered::BufferUnordered   time:   [363.39 ms 365.59 ms 367.78 ms]
    /// ```
    ///
    /// ### Memory usage
    ///
    /// Running 512000 `Ready<i32>` futures with 256 concurrent jobs.
    ///
    /// - count: the number of times alloc/dealloc was called
    /// - alloc: the number of cumulative bytes allocated
    /// - dealloc: the number of cumulative bytes deallocated
    ///
    /// ```text
    /// futures::stream::BufferUnordered
    ///     count:    1024002
    ///     alloc:    40960144 B
    ///     dealloc:  40960000 B
    ///
    /// futures_buffered::BufferUnordered
    ///     count:    2
    ///     alloc:    8264 B
    ///     dealloc:  0 B
    /// ```
    #[must_use = "streams do nothing unless polled"]
    pub struct BufferUnordered<S: Stream> {
        #[pin]
        pub(crate) stream: Option<S>,
        pub(crate) in_progress_queue: FuturesUnorderedBounded<S::Item>,
    }
);

impl<St> Stream for BufferUnordered<St>
where
    St: Stream,
    St::Item: Future,
{
    type Item = <St::Item as Future>::Output;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        // First up, try to spawn off as many futures as possible by filling up
        // our queue of futures.
        let unordered = this.in_progress_queue;
        while unordered.tasks.len() < unordered.tasks.capacity() {
            if let Some(s) = this.stream.as_mut().as_pin_mut() {
                match s.poll_next(cx) {
                    Poll::Ready(Some(fut)) => {
                        unordered.push(fut);
                        continue;
                    }
                    Poll::Ready(None) => this.stream.as_mut().set(None),
                    Poll::Pending => {}
                }
            }
            break;
        }

        // Attempt to pull the next value from the in_progress_queue
        match Pin::new(unordered).poll_next(cx) {
            x @ (Poll::Pending | Poll::Ready(Some(_))) => return x,
            Poll::Ready(None) => {}
        }

        // If more values are still coming from the stream, we're not done yet
        if this.stream.as_pin_mut().is_none() {
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let queue_len = self.in_progress_queue.len();
        let (lower, upper) = self
            .stream
            .as_ref()
            .map(|s| s.size_hint())
            .unwrap_or((0, Some(0)));
        let lower = lower.saturating_add(queue_len);
        let upper = match upper {
            Some(x) => x.checked_add(queue_len),
            None => None,
        };
        (lower, upper)
    }
}

#[cfg(test)]
mod tests {
    use crate::BufferedStreamExt;

    use super::*;
    use futures::{channel::oneshot, stream, StreamExt};
    use futures_test::task::noop_context;
    use rand::{thread_rng, Rng};
    use tokio::task::JoinSet;

    #[test]
    fn buffered_unordered() {
        let (send_one, recv_one) = oneshot::channel();
        let (send_two, recv_two) = oneshot::channel();

        let stream_of_futures = stream::iter(vec![recv_one, recv_two]);
        let mut buffered = stream_of_futures.buffered_unordered(10);
        let mut cx = noop_context();

        // sized properly
        assert_eq!(buffered.size_hint(), (2, Some(2)));

        // make sure it returns pending
        assert_eq!(buffered.poll_next_unpin(&mut cx), Poll::Pending);

        // returns in any order
        send_two.send(2i32).unwrap();
        assert_eq!(
            buffered.poll_next_unpin(&mut cx),
            Poll::Ready(Some(Ok(2i32)))
        );

        send_one.send(1i32).unwrap();
        assert_eq!(
            buffered.poll_next_unpin(&mut cx),
            Poll::Ready(Some(Ok(1i32)))
        );

        // completes properly
        assert_eq!(buffered.poll_next_unpin(&mut cx), Poll::Ready(None));
    }

    #[cfg(not(miri))]
    // #[tokio::test(flavor = "multi_thread")]
    #[tokio::test(start_paused = true)]
    async fn high_concurrency() {
        let now = tokio::time::Instant::now();
        let dur = std::time::Duration::from_millis(10);
        let n = 1024 * 16;
        let c = 32;

        let estimated = dur.as_secs_f64() * 10.5 * (n as f64) / (c as f64) * 4.0;
        dbg!(estimated);

        let mut js = JoinSet::new();

        for _ in 0..32 {
            js.spawn(async move {
                let x = futures::stream::repeat_with(|| {
                    let n = thread_rng().gen_range(1..=20);
                    let fut = async move {
                        for _ in 0..4 {
                            tokio::time::sleep(n * dur).await;
                        }
                    };
                    tokio::time::timeout(dur * (5 * n), fut)
                });
                let x = x.take(n as usize).buffered_unordered(c as usize);
                x.for_each(|res| async { res.unwrap() }).await;
            });
        }

        while js.join_next().await.is_some() {}

        let elapsed = now.elapsed().as_secs_f64();
        dbg!(elapsed);
    }
}
