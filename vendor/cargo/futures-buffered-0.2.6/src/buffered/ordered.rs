use crate::FuturesOrderedBounded;
use core::{
    future::Future,
    pin::Pin,
    task::{ready, Context, Poll},
};
use futures_core::Stream;
use pin_project_lite::pin_project;

pin_project! {
    /// Stream for the [`buffered_ordered`](crate::BufferedStreamExt::buffered_ordered) method.
    #[must_use = "streams do nothing unless polled"]
    pub struct BufferedOrdered<St>
    where
        St: Stream,
        St::Item: Future,
    {
        #[pin]
        pub(crate) stream: Option<St>,
        pub(crate) in_progress_queue: FuturesOrderedBounded<St::Item>,
    }
}

impl<St> Stream for BufferedOrdered<St>
where
    St: Stream,
    St::Item: Future,
{
    type Item = <St::Item as Future>::Output;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        // First up, try to spawn off as many futures as possible by filling up
        // our queue of futures.
        let ordered = this.in_progress_queue;
        while ordered.in_progress_queue.tasks.len() < ordered.in_progress_queue.tasks.capacity() {
            if let Some(s) = this.stream.as_mut().as_pin_mut() {
                match s.poll_next(cx) {
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

#[cfg(test)]
mod tests {
    use crate::BufferedStreamExt;

    use super::*;
    use futures::{channel::oneshot, stream, StreamExt};
    use futures_test::task::noop_context;

    #[test]
    fn buffered_ordered() {
        let (send_one, recv_one) = oneshot::channel();
        let (send_two, recv_two) = oneshot::channel();

        let stream_of_futures = stream::iter(vec![recv_one, recv_two]);
        let mut buffered = stream_of_futures.buffered_ordered(10);
        let mut cx = noop_context();

        // sized properly
        assert_eq!(buffered.size_hint(), (2, Some(2)));

        // make sure it returns pending
        assert_eq!(buffered.poll_next_unpin(&mut cx), Poll::Pending);

        // returns in a fixed order
        send_two.send(2i32).unwrap();
        assert_eq!(buffered.poll_next_unpin(&mut cx), Poll::Pending);

        send_one.send(1i32).unwrap();
        assert_eq!(
            buffered.poll_next_unpin(&mut cx),
            Poll::Ready(Some(Ok(1i32)))
        );
        assert_eq!(
            buffered.poll_next_unpin(&mut cx),
            Poll::Ready(Some(Ok(2i32)))
        );

        // completes properly
        assert_eq!(buffered.poll_next_unpin(&mut cx), Poll::Ready(None));
    }
}
