use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use futures_core::{FusedFuture, Stream};
use pin_project_lite::pin_project;

use crate::FuturesUnorderedBounded;

pin_project! {
    /// Future for the [`for_each_concurrent`](super::StreamExt::for_each_concurrent)
    /// method.
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct ForEachConcurrent<St, Fut, F> {
        #[pin]
        stream: Option<St>,
        f: F,
        futures: FuturesUnorderedBounded<Fut>,
    }
}

impl<St, Fut, F> ForEachConcurrent<St, Fut, F>
where
    St: Stream,
    F: FnMut(St::Item) -> Fut,
    Fut: Future<Output = ()>,
{
    pub(super) fn new(stream: St, limit: usize, f: F) -> Self {
        Self {
            stream: Some(stream),
            f,
            futures: FuturesUnorderedBounded::new(limit),
        }
    }
}

impl<St, Fut, F> FusedFuture for ForEachConcurrent<St, Fut, F>
where
    St: Stream,
    F: FnMut(St::Item) -> Fut,
    Fut: Future<Output = ()>,
{
    fn is_terminated(&self) -> bool {
        self.stream.is_none() && self.futures.is_empty()
    }
}

impl<St, Fut, F> Future for ForEachConcurrent<St, Fut, F>
where
    St: Stream,
    F: FnMut(St::Item) -> Fut,
    Fut: Future<Output = ()>,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let mut this = self.project();

        loop {
            let mut should_poll_stream = false;

            let unordered = &mut *this.futures;

            // if there's capacity for more futures, try and poll the stream
            if unordered.tasks.len() < unordered.tasks.capacity() {
                if let Some(s) = this.stream.as_mut().as_pin_mut() {
                    match s.poll_next(cx) {
                        Poll::Ready(Some(elem)) => {
                            should_poll_stream = true;
                            unordered.push((this.f)(elem));
                        }
                        Poll::Ready(None) => this.stream.as_mut().set(None),
                        Poll::Pending => {}
                    }
                }
            }

            // Attempt to pull the next value from the in_progress_queue
            match Pin::new(unordered).poll_next(cx) {
                Poll::Pending => {}
                Poll::Ready(None) => {
                    // If the stream is finished, then we are done here
                    if this.stream.as_mut().as_pin_mut().is_none() {
                        break Poll::Ready(());
                    }
                }
                Poll::Ready(Some(())) => continue,
            }

            if !should_poll_stream {
                break Poll::Pending;
            }
        }
    }
}
