use crate::FuturesUnorderedBounded;
use alloc::collections::binary_heap::{BinaryHeap, PeekMut};
use core::cmp::Ordering;
use core::fmt;
use core::iter::FromIterator;
use core::num::Wrapping;
use core::pin::Pin;
use futures_core::future::Future;
use futures_core::ready;
use futures_core::stream::Stream;
use futures_core::{
    task::{Context, Poll},
    FusedStream,
};
use pin_project_lite::pin_project;

pin_project! {
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    #[derive(Debug)]
    pub(crate) struct OrderWrapper<T> {
        #[pin]
        pub data: T, // A future or a future's output
        pub index: usize,
    }
}

impl<T> PartialEq for OrderWrapper<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Eq for OrderWrapper<T> {}

impl<T> PartialOrd for OrderWrapper<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for OrderWrapper<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap is a max heap, so compare backwards here.
        other.index.cmp(&self.index)
    }
}

impl<T> Future for OrderWrapper<T>
where
    T: Future,
{
    type Output = OrderWrapper<T::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let index = self.index;
        self.project().data.poll(cx).map(|output| OrderWrapper {
            data: output,
            index,
        })
    }
}

/// An unbounded queue of futures.
///
/// This "combinator" is similar to `FuturesUnordered`, but it imposes an order
/// on top of the set of futures. While futures in the set will race to
/// completion in parallel, results will only be returned in the order their
/// originating futures were added to the queue.
///
/// Futures are pushed into this queue and their realized values are yielded in
/// order. This structure is optimized to manage a large number of futures.
/// Futures managed by `FuturesOrderedBounded` will only be polled when they generate
/// notifications. This reduces the required amount of work needed to coordinate
/// large numbers of futures.
///
/// When a `FuturesOrderedBounded` is first created, it does not contain any futures.
/// Calling `poll` in this state will result in `Poll::Ready(None))` to be
/// returned. Futures are submitted to the queue using `push`; however, the
/// future will **not** be polled at this point. `FuturesOrderedBounded` will only
/// poll managed futures when `FuturesOrderedBounded::poll` is called. As such, it
/// is important to call `poll` after pushing new futures.
///
/// If `FuturesOrderedBounded::poll` returns `Poll::Ready(None)` this means that
/// the queue is currently not managing any futures. A future may be submitted
/// to the queue at a later time. At that point, a call to
/// `FuturesOrderedBounded::poll` will either return the future's resolved value
/// **or** `Poll::Pending` if the future has not yet completed. When
/// multiple futures are submitted to the queue, `FuturesOrderedBounded::poll` will
/// return `Poll::Pending` until the first future completes, even if
/// some of the later futures have already completed.
///
/// Note that you can create a ready-made `FuturesOrderedBounded` via the
/// [`collect`](Iterator::collect) method, or you can start with an empty queue
/// with the `FuturesOrderedBounded::new` constructor.
#[must_use = "streams do nothing unless polled"]
pub struct FuturesOrderedBounded<T: Future> {
    pub(crate) in_progress_queue: FuturesUnorderedBounded<OrderWrapper<T>>,
    queued_outputs: BinaryHeap<OrderWrapper<T::Output>>,
    pub(crate) next_incoming_index: Wrapping<usize>,
    next_outgoing_index: Wrapping<usize>,
}

impl<T: Future> Unpin for FuturesOrderedBounded<T> {}

impl<Fut: Future> FuturesOrderedBounded<Fut> {
    /// Constructs a new, empty `FuturesOrderedBounded`
    ///
    /// The returned `FuturesOrderedBounded` does not contain any futures and, in this
    /// state, `FuturesOrderedBounded::poll_next` will return `Poll::Ready(None)`.
    pub fn new(capacity: usize) -> Self {
        Self {
            in_progress_queue: FuturesUnorderedBounded::new(capacity),
            queued_outputs: BinaryHeap::with_capacity(capacity - 1),
            next_incoming_index: Wrapping(0),
            next_outgoing_index: Wrapping(0),
        }
    }

    /// Returns the number of futures contained in the queue.
    ///
    /// This represents the total number of in-flight futures, both
    /// those currently processing and those that have completed but
    /// which are waiting for earlier futures to complete.
    pub fn len(&self) -> usize {
        self.in_progress_queue.len() + self.queued_outputs.len()
    }

    /// Returns `true` if the queue contains no futures
    pub fn is_empty(&self) -> bool {
        self.in_progress_queue.is_empty() && self.queued_outputs.is_empty()
    }

    /// Pushes a future to the back of the queue.
    ///
    /// This function submits the given future to the internal set for managing.
    /// This function will not call `poll` on the submitted future. The caller
    /// must ensure that `FuturesOrderedBounded::poll` is called in order to receive
    /// task notifications.
    ///
    /// # Errors
    /// This method will error if the buffer is currently full, returning the future back
    pub fn try_push_back(&mut self, future: Fut) -> Result<(), Fut> {
        self.in_progress_queue.try_push_with(future, |future| {
            let wrapped = OrderWrapper {
                data: future,
                index: self.next_incoming_index.0,
            };
            self.next_incoming_index += 1;
            wrapped
        })
    }

    /// Pushes a future to the front of the queue.
    ///
    /// This function submits the given future to the internal set for managing.
    /// This function will not call `poll` on the submitted future. The caller
    /// must ensure that `FuturesOrderedBounded::poll` is called in order to receive
    /// task notifications. This future will be the next future to be returned
    /// complete.
    ///
    /// # Errors
    /// This method will error if the buffer is currently full, returning the future back
    pub fn try_push_front(&mut self, future: Fut) -> Result<(), Fut> {
        self.in_progress_queue.try_push_with(future, |future| {
            self.next_outgoing_index -= 1;
            OrderWrapper {
                data: future,
                index: self.next_outgoing_index.0,
            }
        })
    }

    /// Pushes a future to the back of the queue.
    ///
    /// This function submits the given future to the internal set for managing.
    /// This function will not call `poll` on the submitted future. The caller
    /// must ensure that `FuturesOrderedBounded::poll` is called in order to receive
    /// task notifications.
    ///
    /// # Panics
    /// This method will panic if the buffer is currently full. See [`FuturesOrderedBounded::try_push_back`] to get a result instead
    #[track_caller]
    pub fn push_back(&mut self, future: Fut) {
        if self.try_push_back(future).is_err() {
            panic!("attempted to push into a full `FuturesOrderedBounded`")
        }
    }

    /// Pushes a future to the front of the queue.
    ///
    /// This function submits the given future to the internal set for managing.
    /// This function will not call `poll` on the submitted future. The caller
    /// must ensure that `FuturesOrderedBounded::poll` is called in order to receive
    /// task notifications. This future will be the next future to be returned
    /// complete.
    ///
    /// # Panics
    /// This method will panic if the buffer is currently full. See [`FuturesOrderedBounded::try_push_front`] to get a result instead
    #[track_caller]
    pub fn push_front(&mut self, future: Fut) {
        if self.try_push_front(future).is_err() {
            panic!("attempted to push into a full `FuturesOrderedBounded`")
        }
    }
}

impl<Fut: Future> Stream for FuturesOrderedBounded<Fut> {
    type Item = Fut::Output;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = &mut *self;

        const MSB: usize = !(usize::MAX >> 1);
        // house keeping if the indices gets too high
        if this.next_outgoing_index.0 & MSB == MSB {
            let mut ready_queue = core::mem::take(&mut this.queued_outputs).into_vec();
            for entry in &mut ready_queue {
                entry.index ^= MSB;
            }
            this.queued_outputs = ready_queue.into();

            for task in this.in_progress_queue.tasks.iter_mut() {
                *task.project().index ^= MSB;
            }

            this.next_outgoing_index.0 ^= MSB;
            this.next_incoming_index.0 ^= MSB;
        }

        // Check to see if we've already received the next value
        if let Some(next_output) = this.queued_outputs.peek_mut() {
            if next_output.index == this.next_outgoing_index.0 {
                this.next_outgoing_index += 1;
                return Poll::Ready(Some(PeekMut::pop(next_output).data));
            }
        }

        loop {
            match ready!(Pin::new(&mut this.in_progress_queue).poll_next(cx)) {
                Some(output) => {
                    if output.index == this.next_outgoing_index.0 {
                        this.next_outgoing_index += 1;
                        return Poll::Ready(Some(output.data));
                    } else {
                        this.queued_outputs.push(output)
                    }
                }
                None => return Poll::Ready(None),
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<Fut: Future> fmt::Debug for FuturesOrderedBounded<Fut> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FuturesOrderedBounded {{ ... }}")
    }
}

impl<Fut: Future> FromIterator<Fut> for FuturesOrderedBounded<Fut> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Fut>,
    {
        let mut index = Wrapping(0);
        let in_progress_queue = FuturesUnorderedBounded::from_iter(iter.into_iter().map(|data| {
            let next_index = index + Wrapping(1);
            OrderWrapper {
                data,
                index: core::mem::replace(&mut index, next_index).0,
            }
        }));
        Self {
            in_progress_queue,
            queued_outputs: BinaryHeap::new(),
            next_incoming_index: index,
            next_outgoing_index: Wrapping(0),
        }
    }
}

impl<Fut: Future> FusedStream for FuturesOrderedBounded<Fut> {
    fn is_terminated(&self) -> bool {
        self.in_progress_queue.is_terminated() && self.queued_outputs.is_empty()
    }
}

impl<Fut: Future> Extend<Fut> for FuturesOrderedBounded<Fut> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Fut>,
    {
        for item in iter {
            self.push_back(item);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::FuturesOrderedBounded;
    use core::{future::ready, task::Poll};
    use futures::{Stream, StreamExt};
    use futures_test::task::noop_context;

    #[test]
    fn ordered() {
        let mut buffer = FuturesOrderedBounded::new(10);

        for i in 0..10 {
            buffer.push_back(ready(i));
        }

        for i in 0..10 {
            assert_eq!(
                buffer.poll_next_unpin(&mut noop_context()),
                Poll::Ready(Some(i))
            );
        }
    }

    #[test]
    fn ordered_front() {
        let mut buffer = FuturesOrderedBounded::new(10);

        for i in 0..10 {
            buffer.push_front(ready(i));
        }

        for i in (0..10).rev() {
            assert_eq!(
                buffer.poll_next_unpin(&mut noop_context()),
                Poll::Ready(Some(i))
            );
        }
    }

    #[test]
    #[should_panic(expected = "attempted to push into a full `FuturesOrderedBounded`")]
    fn full_back() {
        let mut buffer = FuturesOrderedBounded::new(1);
        buffer.push_back(ready(()));
        buffer.push_back(ready(()));
    }

    #[test]
    #[should_panic(expected = "attempted to push into a full `FuturesOrderedBounded`")]
    fn full_front() {
        let mut buffer = FuturesOrderedBounded::new(1);
        buffer.push_front(ready(()));
        buffer.push_front(ready(()));
    }

    #[test]
    fn from_iter() {
        let buffer = FuturesOrderedBounded::from_iter((0..10).map(|_| ready(())));

        assert_eq!(buffer.len(), 10);
        assert_eq!(buffer.size_hint(), (10, Some(10)));
    }
}
