//! Highly inspired by [and_then_concurrent](https://docs.rs/and-then-concurrent/latest/and_then_concurrent/)
//!
//! `ThenConcurrent` extension adds the `then_concurrent` method to any `futures::stream::Stream`
//! object allowing a concurrent execution of futures over the stream items.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::stream::{FuturesUnordered, Stream};
use pin_project::pin_project;

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

        match stream.as_mut().poll_next(cx) {
            Poll::Ready(Some(n)) => {
                futures.push(fun(n));
            }
            Poll::Pending => {
                if futures.is_empty() {
                    return Poll::Pending;
                }
            }
            _ => (),
        }

        let x = futures.as_mut().poll_next(cx);
        if let Poll::Pending = x {
            match stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(n)) => {
                    futures.push(fun(n));
                }
                _ => (),
            }
        }
        x
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
