//! Highly inspired by [and_then_concurrent](https://docs.rs/and-then-concurrent/latest/and_then_concurrent/)
//!
//! `ThenConcurrent` extension adds the `then_concurrent` method to any `futures::stream::Stream`
//! object allowing a concurrent execution of futures over the stream items.

use futures::stream::{FuturesUnordered, Stream};
use pin_project::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// Stream for the [`Stream::then_concurrent()`] method.
#[pin_project(project = ThenConcurrentProj)]
pub struct ThenConcurrent<St, Fut: Future, F> {
    #[pin]
    stream: St,
    #[pin]
    futures: FuturesUnordered<Fut>,
    fun: F,
}

impl<St, Fut, F, T> Stream for ThenConcurrent<St, Fut, F>
where
    St: Stream,
    Fut: Future<Output = T>,
    F: FnMut(St::Item) -> Fut,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let ThenConcurrentProj {
            mut stream,
            mut futures,
            fun,
        } = self.project();

        // Eagerly fetch all ready items from the stream
        loop {
            match stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(n)) => {
                    futures.push(fun(n));
                }
                Poll::Ready(None) => {
                    if futures.is_empty() {
                        return Poll::Ready(None);
                    }
                    break;
                }
                Poll::Pending => {
                    if futures.is_empty() {
                        return Poll::Pending;
                    }
                    break;
                }
            }
        }

        futures.as_mut().poll_next(cx)
    }
}

/// Extension to `futures::stream::Stream`
pub trait StreamThenConcurrentExt: Stream {
    /// Chain a computation when a stream value is ready, passing `Ok` values to the closure `f`.
    ///
    /// This function is similar to [`futures::stream::StreamExt::then`], but the
    /// stream is polled concurrently with the futures returned by `f`. An unbounded number of
    /// futures corresponding to past stream values is kept via `FuturesUnordered`.
    fn then_concurrent<Fut, F>(self, f: F) -> ThenConcurrent<Self, Fut, F>
    where
        Self: Sized,
        Fut: Future,
        F: FnMut(Self::Item) -> Fut;
}

impl<S: Stream> StreamThenConcurrentExt for S {
    fn then_concurrent<Fut, F>(self, f: F) -> ThenConcurrent<Self, Fut, F>
    where
        Self: Sized,
        Fut: Future,
        F: FnMut(Self::Item) -> Fut,
    {
        ThenConcurrent {
            stream: self,
            futures: FuturesUnordered::new(),
            fun: f,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{channel::mpsc::unbounded, StreamExt};

    #[async_std::test]
    async fn no_items() {
        let stream = futures::stream::iter::<Vec<u64>>(vec![]).then_concurrent(|_| async move {
            panic!("must not be called");
        });

        assert_eq!(stream.collect::<Vec<_>>().await, vec![]);
    }

    #[async_std::test]
    async fn paused_stream() {
        let (mut tx, rx) = unbounded::<u64>();

        let mut stream = rx.then_concurrent(|x| async move {
            if x == 0 {
                x
            } else {
                async_std::task::sleep(std::time::Duration::from_millis(x)).await;
                x
            }
        });

        // we need to poll the stream such that FuturesUnordered gets empty
        let first_item = stream.next();

        tx.start_send(0).unwrap();

        assert_eq!(first_item.await, Some(0));

        let second_item = stream.next();

        // item produces a delay
        tx.start_send(5).unwrap();

        assert_eq!(second_item.await, Some(5));
    }

    #[async_std::test]
    async fn fast_items() {
        let item_1 = 0u64; // 3rd in the output
        let item_2 = 0u64; // 1st in the output
        let item_3 = 7u64; // 2nd in the output

        let stream = futures::stream::iter(vec![item_1, item_2, item_3]).then_concurrent(|x| async move {
            if x == 0 {
                x
            } else {
                async_std::task::sleep(std::time::Duration::from_millis(x)).await;
                x
            }
        });
        let actual_packets = stream.collect::<Vec<u64>>().await;

        assert_eq!(actual_packets, vec![0, 0, 7]);
    }

    #[async_std::test]
    async fn reorder_items() {
        let item_1 = 10u64; // 3rd in the output
        let item_2 = 5u64; // 1st in the output
        let item_3 = 7u64; // 2nd in the output

        let stream = futures::stream::iter(vec![item_1, item_2, item_3]).then_concurrent(|x| async move {
            async_std::task::sleep(std::time::Duration::from_millis(x)).await;
            x
        });
        let actual_packets = stream.collect::<Vec<u64>>().await;

        assert_eq!(actual_packets, vec![5, 7, 10]);
    }
}
