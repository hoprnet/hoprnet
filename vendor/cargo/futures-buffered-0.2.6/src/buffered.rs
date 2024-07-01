use crate::FuturesOrderedBounded;
use crate::FuturesUnorderedBounded;
use core::future::Future;
use futures_core::Stream;

mod for_each;
mod ordered;
mod unordered;

pub use for_each::ForEachConcurrent;
pub use ordered::BufferedOrdered;
pub use unordered::BufferUnordered;

impl<T: ?Sized + Stream> BufferedStreamExt for T {}

/// An extension trait for `Stream`s that provides a variety of convenient
/// combinator functions.
pub trait BufferedStreamExt: Stream {
    /// An adaptor for creating a buffered list of pending futures.
    ///
    /// If this stream's item can be converted into a future, then this adaptor
    /// will buffer up to at most `n` futures and then return the outputs in the
    /// same order as the underlying stream. No more than `n` futures will be
    /// buffered at any point in time, and less than `n` may also be buffered
    /// depending on the state of each future.
    ///
    /// The returned stream will be a stream of each future's output.
    fn buffered_ordered(self, n: usize) -> BufferedOrdered<Self>
    where
        Self::Item: Future,
        Self: Sized,
    {
        BufferedOrdered {
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
    /// See [`BufferUnordered`] for performance details
    fn buffered_unordered(self, n: usize) -> BufferUnordered<Self>
    where
        Self::Item: Future,
        Self: Sized,
    {
        BufferUnordered {
            stream: Some(self),
            in_progress_queue: FuturesUnorderedBounded::new(n),
        }
    }

    /// Runs this stream to completion, executing the provided asynchronous
    /// closure for each element on the stream concurrently as elements become
    /// available.
    ///
    /// This is similar to [`StreamExt::for_each`](futures_util::StreamExt::for_each), but the futures
    /// produced by the closure are run concurrently (but not in parallel--
    /// this combinator does not introduce any threads).
    ///
    /// The closure provided will be called for each item this stream produces,
    /// yielding a future. That future will then be executed to completion
    /// concurrently with the other futures produced by the closure.
    ///
    /// The first argument is an optional limit on the number of concurrent
    /// futures. If this limit is not `None`, no more than `limit` futures
    /// will be run concurrently. The `limit` argument is of type
    /// `Into<Option<usize>>`, and so can be provided as either `None`,
    /// `Some(10)`, or just `10`. Note: a limit of zero is interpreted as
    /// no limit at all, and will have the same result as passing in `None`.
    ///
    /// This method is only available when the `std` or `alloc` feature of this
    /// library is activated, and it is activated by default.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures::channel::oneshot;
    /// use futures::stream::{self, StreamExt};
    ///
    /// let (tx1, rx1) = oneshot::channel();
    /// let (tx2, rx2) = oneshot::channel();
    /// let (tx3, rx3) = oneshot::channel();
    ///
    /// let fut = stream::iter(vec![rx1, rx2, rx3]).for_each_concurrent(
    ///     /* limit */ 2,
    ///     |rx| async move {
    ///         rx.await.unwrap();
    ///     }
    /// );
    /// tx1.send(()).unwrap();
    /// tx2.send(()).unwrap();
    /// tx3.send(()).unwrap();
    /// fut.await;
    /// # })
    /// ```
    fn for_each_concurrent<Fut, F>(self, limit: usize, f: F) -> ForEachConcurrent<Self, Fut, F>
    where
        F: FnMut(Self::Item) -> Fut,
        Fut: Future<Output = ()>,
        Self: Sized,
    {
        ForEachConcurrent::new(self, limit, f)
    }
}
