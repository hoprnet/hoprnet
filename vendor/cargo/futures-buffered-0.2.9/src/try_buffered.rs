use core::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::{FuturesOrderedBounded, TryStream};
use crate::{FuturesUnorderedBounded, TryFuture};
use futures_core::ready;
use futures_core::Stream;
use pin_project_lite::pin_project;

impl<T: ?Sized + TryStream> BufferedTryStreamExt for T {}

/// An extension trait for `Stream`s that provides a variety of convenient
/// combinator functions.
pub trait BufferedTryStreamExt: TryStream {
    /// An adaptor for creating a buffered list of pending futures.
    ///
    /// If this stream's item can be converted into a future, then this adaptor
    /// will buffer up to at most `n` futures and then return the outputs in the
    /// same order as the underlying stream. No more than `n` futures will be
    /// buffered at any point in time, and less than `n` may also be buffered
    /// depending on the state of each future.
    ///
    /// The returned stream will be a stream of each future's output.
    fn try_buffered_ordered(self, n: usize) -> TryBufferedOrdered<Self>
    where
        Self::Ok: TryFuture<Err = Self::Err>,
        Self: Sized,
    {
        TryBufferedOrdered {
            stream: Some(self),
            in_progress_queue: FuturesOrderedBounded::new(n),
        }
    }

    /// An adaptor for creating a buffered list of pending futures (unordered).
    ///
    /// If this stream's item can be converted into a future, then this adaptor
    /// will buffer up to `n` futures and then return the outputs in the order
    /// in which they complete. No more than `n` futures will be buffered at
    /// any point in time, and less than `n` may also be buffered depending on
    /// the state of each future.
    ///
    /// The returned stream will be a stream of each future's output.
    fn try_buffered_unordered(self, n: usize) -> TryBufferUnordered<Self>
    where
        Self::Ok: TryFuture<Err = Self::Err>,
        Self: Sized,
    {
        TryBufferUnordered {
            stream: Some(self),
            in_progress_queue: FuturesUnorderedBounded::new(n),
        }
    }
}

pin_project! {
    /// Stream for the [`try_buffered_ordered`](BufferedTryStreamExt::try_buffered_ordered) method.
    #[must_use = "streams do nothing unless polled"]
    pub struct TryBufferedOrdered<St>
    where
        St: TryStream,
        St::Ok: TryFuture,
    {
        #[pin]
        stream: Option<St>,
        in_progress_queue: FuturesOrderedBounded<St::Ok>,
    }
}

impl<St> Stream for TryBufferedOrdered<St>
where
    St: TryStream,
    St::Ok: TryFuture<Err = St::Err>,
{
    type Item = Result<<St::Ok as TryFuture>::Ok, St::Err>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        // First up, try to spawn off as many futures as possible by filling up
        // our queue of futures.
        let ordered = this.in_progress_queue;
        while ordered.in_progress_queue.tasks.len() < ordered.in_progress_queue.tasks.capacity() {
            if let Some(s) = this.stream.as_mut().as_pin_mut() {
                match s.poll_next(cx)? {
                    Poll::Ready(Some(fut)) => {
                        ordered.push_back(fut);
                        continue;
                    }
                    Poll::Ready(None) => this.stream.as_mut().set(None),
                    Poll::Pending => {}
                }
            }
            break;
        }

        // Attempt to pull the next value from the in_progress_queue
        let res = Pin::new(ordered).poll_next(cx);
        if let Some(val) = ready!(res) {
            return Poll::Ready(Some(val));
        }

        // If more values are still coming from the stream, we're not done yet
        if this.stream.is_none() {
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.stream {
            Some(s) => {
                let queue_len = self.in_progress_queue.len();
                let (lower, upper) = s.size_hint();
                let lower = lower.saturating_add(queue_len);
                let upper = match upper {
                    Some(x) => x.checked_add(queue_len),
                    None => None,
                };
                (lower, upper)
            }
            _ => (0, Some(0)),
        }
    }
}

pin_project!(
    /// Stream for the [`try_buffered_unordered`](BufferedTryStreamExt::try_buffered_unordered) method.
    #[must_use = "streams do nothing unless polled"]
    pub struct TryBufferUnordered<S: TryStream> {
        #[pin]
        stream: Option<S>,
        in_progress_queue: FuturesUnorderedBounded<S::Ok>,
    }
);

impl<St> Stream for TryBufferUnordered<St>
where
    St: TryStream,
    St::Ok: TryFuture<Err = St::Err>,
{
    type Item = Result<<St::Ok as TryFuture>::Ok, St::Err>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        // First up, try to spawn off as many futures as possible by filling up
        // our queue of futures.
        let unordered = this.in_progress_queue;
        while unordered.tasks.len() < unordered.tasks.capacity() {
            if let Some(s) = this.stream.as_mut().as_pin_mut() {
                match s.poll_next(cx)? {
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
        match &self.stream {
            Some(s) => {
                let queue_len = self.in_progress_queue.len();
                let (lower, upper) = s.size_hint();
                let lower = lower.saturating_add(queue_len);
                let upper = match upper {
                    Some(x) => x.checked_add(queue_len),
                    None => None,
                };
                (lower, upper)
            }
            _ => (0, Some(0)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::task::Poll;
    use futures::{
        channel::oneshot::{self, Canceled},
        stream, TryFutureExt, TryStreamExt,
    };
    use futures_test::task::noop_context;

    fn _else(_: Canceled) -> Result<i32, i32> {
        Ok(0)
    }

    #[test]
    fn buffered_ordered() {
        let (send_one, recv_one) = oneshot::channel();
        let (send_two, recv_two) = oneshot::channel();

        let stream_of_futures = stream::iter(vec![
            Ok(recv_one.unwrap_or_else(_else)),
            Err(0),
            Ok(recv_two.unwrap_or_else(_else)),
        ]);
        let mut buffered = stream_of_futures.try_buffered_ordered(10);
        let mut cx = noop_context();

        // sized properly
        assert_eq!(buffered.size_hint(), (3, Some(3)));

        // stream errors upfront
        assert_eq!(
            buffered.try_poll_next_unpin(&mut cx),
            Poll::Ready(Some(Err(0)))
        );

        // make sure it returns pending
        assert_eq!(buffered.try_poll_next_unpin(&mut cx), Poll::Pending);

        // returns in a fixed order
        send_two.send(Ok(2)).unwrap();
        assert_eq!(buffered.try_poll_next_unpin(&mut cx), Poll::Pending);

        send_one.send(Err(1)).unwrap();
        assert_eq!(
            buffered.try_poll_next_unpin(&mut cx),
            Poll::Ready(Some(Err(1)))
        );
        assert_eq!(
            buffered.try_poll_next_unpin(&mut cx),
            Poll::Ready(Some(Ok(2)))
        );

        // completes properly
        assert_eq!(buffered.try_poll_next_unpin(&mut cx), Poll::Ready(None));
    }

    #[test]
    fn buffered_unordered() {
        let (send_one, recv_one) = oneshot::channel();
        let (send_two, recv_two) = oneshot::channel();

        let stream_of_futures = stream::iter(vec![
            Ok(recv_one.unwrap_or_else(_else)),
            Err(0),
            Ok(recv_two.unwrap_or_else(_else)),
        ]);
        let mut buffered = stream_of_futures.try_buffered_unordered(10);
        let mut cx = noop_context();

        // sized properly
        assert_eq!(buffered.size_hint(), (3, Some(3)));

        // stream errors upfront
        assert_eq!(
            buffered.try_poll_next_unpin(&mut cx),
            Poll::Ready(Some(Err(0)))
        );

        // make sure it returns pending
        assert_eq!(buffered.try_poll_next_unpin(&mut cx), Poll::Pending);

        // returns in any order
        send_two.send(Ok(2)).unwrap();
        assert_eq!(
            buffered.try_poll_next_unpin(&mut cx),
            Poll::Ready(Some(Ok(2)))
        );

        send_one.send(Ok(1)).unwrap();
        assert_eq!(
            buffered.try_poll_next_unpin(&mut cx),
            Poll::Ready(Some(Ok(1)))
        );

        // completes properly
        assert_eq!(buffered.try_poll_next_unpin(&mut cx), Poll::Ready(None));
    }
}
