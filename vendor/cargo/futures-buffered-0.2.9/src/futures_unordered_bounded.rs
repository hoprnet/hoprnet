use core::{
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{arc_slice::ArcSlice, slot_map::SlotMap};
use futures_core::{FusedStream, Stream};

/// A set of futures which may complete in any order.
///
/// Much like [`futures::stream::FuturesUnordered`](https://docs.rs/futures/0.3.25/futures/stream/struct.FuturesUnordered.html),
/// this is a thread-safe, `Pin` friendly, lifetime friendly, concurrent processing stream.
///
/// The is different to `FuturesUnordered` in that `FuturesUnorderedBounded` has a fixed capacity for processing count.
/// This means it's less flexible, but produces better memory efficiency.
///
/// ## Benchmarks
///
/// ### Speed
///
/// Running 65536 100us timers with 256 concurrent jobs in a single threaded tokio runtime:
///
/// ```text
/// FuturesUnordered         time:   [420.47 ms 422.21 ms 423.99 ms]
/// FuturesUnorderedBounded  time:   [366.02 ms 367.54 ms 369.05 ms]
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
/// FuturesUnordered
///     count:    1024002
///     alloc:    40960144 B
///     dealloc:  40960000 B
///
/// FuturesUnorderedBounded
///     count:    2
///     alloc:    8264 B
///     dealloc:  0 B
/// ```
///
/// ### Conclusion
///
/// As you can see, `FuturesUnorderedBounded` massively reduces you memory overhead while providing a significant performance gain.
/// Perfect for if you want a fixed batch size
///
/// # Example
///
/// Making 1024 total HTTP requests, with a max concurrency of 128
///
/// ```
/// use futures::future::Future;
/// use futures::stream::StreamExt;
/// use futures_buffered::FuturesUnorderedBounded;
/// use hyper::client::conn::http1::{handshake, SendRequest};
/// use hyper::body::Incoming;
/// use hyper::{Request, Response};
/// use hyper_util::rt::TokioIo;
/// use tokio::net::TcpStream;
///
/// # #[cfg(miri)] fn main() {}
/// # #[cfg(not(miri))] #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // create a tcp connection
/// let stream = TcpStream::connect("example.com:80").await?;
///
/// // perform the http handshakes
/// let (mut rs, conn) = handshake(TokioIo::new(stream)).await?;
/// tokio::spawn(conn);
///
/// /// make http request to example.com and read the response
/// fn make_req(rs: &mut SendRequest<String>) -> impl Future<Output = hyper::Result<Response<Incoming>>> {
///     let req = Request::builder()
///         .header("Host", "example.com")
///         .method("GET")
///         .body(String::new())
///         .unwrap();
///     rs.send_request(req)
/// }
///
/// // create a queue that can hold 128 concurrent requests
/// let mut queue = FuturesUnorderedBounded::new(128);
///
/// // start up 128 requests
/// for _ in 0..128 {
///     queue.push(make_req(&mut rs));
/// }
/// // wait for a request to finish and start another to fill its place - up to 1024 total requests
/// for _ in 128..1024 {
///     queue.next().await;
///     queue.push(make_req(&mut rs));
/// }
/// // wait for the tail end to finish
/// for _ in 0..128 {
///     queue.next().await;
/// }
/// # Ok(()) }
/// ```
pub struct FuturesUnorderedBounded<F> {
    pub(crate) tasks: SlotMap<F>,
    pub(crate) shared: ArcSlice,
}

impl<F> Unpin for FuturesUnorderedBounded<F> {}

impl<F> FuturesUnorderedBounded<F> {
    /// Constructs a new, empty [`FuturesUnorderedBounded`] with the given fixed capacity.
    ///
    /// The returned [`FuturesUnorderedBounded`] does not contain any futures.
    /// In this state, [`FuturesUnorderedBounded::poll_next`](Stream::poll_next) will
    /// return [`Poll::Ready(None)`](Poll::Ready).
    pub fn new(cap: usize) -> Self {
        Self {
            tasks: SlotMap::new(cap),
            shared: ArcSlice::new(cap),
        }
    }

    /// Push a future into the set.
    ///
    /// This method adds the given future to the set. This method will not
    /// call [`poll`](core::future::Future::poll) on the submitted future. The caller must
    /// ensure that [`FuturesUnorderedBounded::poll_next`](Stream::poll_next) is called
    /// in order to receive wake-up notifications for the given future.
    ///
    /// # Panics
    /// This method will panic if the buffer is currently full. See [`FuturesUnorderedBounded::try_push`] to get a result instead
    #[track_caller]
    pub fn push(&mut self, fut: F) {
        if self.try_push(fut).is_err() {
            panic!("attempted to push into a full `FuturesUnorderedBounded`");
        }
    }

    /// Push a future into the set.
    ///
    /// This method adds the given future to the set. This method will not
    /// call [`poll`](core::future::Future::poll) on the submitted future. The caller must
    /// ensure that [`FuturesUnorderedBounded::poll_next`](Stream::poll_next) is called
    /// in order to receive wake-up notifications for the given future.
    ///
    /// # Errors
    /// This method will error if the buffer is currently full, returning the future back
    pub fn try_push(&mut self, fut: F) -> Result<(), F> {
        self.try_push_with(fut, core::convert::identity)
    }

    #[inline]
    pub(crate) fn try_push_with<T>(&mut self, t: T, f: impl FnMut(T) -> F) -> Result<(), T> {
        let i = self.tasks.insert_with(t, f)?;
        // safety: i is always within capacity
        unsafe {
            self.shared.push(i);
        }
        Ok(())
    }

    /// Returns `true` if the set contains no futures.
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Returns the number of futures contained in the set.
    ///
    /// This represents the total number of in-flight futures.
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// Returns the number of futures that can be contained in the set.
    pub fn capacity(&self) -> usize {
        self.tasks.capacity()
    }
}

type PollFn<F, O> = fn(Pin<&mut F>, cx: &mut Context<'_>) -> Poll<O>;

impl<F> FuturesUnorderedBounded<F> {
    pub(crate) fn poll_inner_no_remove<O>(
        &mut self,
        cx: &mut Context<'_>,
        poll_fn: PollFn<F, O>,
    ) -> Poll<Option<(usize, O)>> {
        const MAX: usize = 61;

        if self.is_empty() {
            return Poll::Ready(None);
        }

        self.shared.register(cx.waker());

        let mut count = 0;
        loop {
            count += 1;
            // if we are in a pending only loop - let's break out.
            if count > MAX {
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }

            match unsafe { self.shared.pop() } {
                crate::arc_slice::ReadySlot::None => return Poll::Pending,
                crate::arc_slice::ReadySlot::Inconsistent => {
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }
                crate::arc_slice::ReadySlot::Ready((i, waker)) => {
                    if let Some(task) = self.tasks.get(i) {
                        let mut cx = Context::from_waker(&waker);

                        let res = poll_fn(task, &mut cx);

                        if let Poll::Ready(x) = res {
                            return Poll::Ready(Some((i, x)));
                        }
                    }
                }
            }
        }
    }
}

impl<F: Future> FuturesUnorderedBounded<F> {
    pub(crate) fn poll_inner(&mut self, cx: &mut Context<'_>) -> Poll<Option<(usize, F::Output)>> {
        match self.poll_inner_no_remove(cx, F::poll) {
            Poll::Ready(Some((i, x))) => {
                self.tasks.remove(i);
                Poll::Ready(Some((i, x)))
            }
            p => p,
        }
    }
}

impl<F: Future> Stream for FuturesUnorderedBounded<F> {
    type Item = F::Output;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.poll_inner(cx) {
            Poll::Ready(Some((_, x))) => Poll::Ready(Some(x)),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<F> FromIterator<F> for FuturesUnorderedBounded<F> {
    /// Constructs a new, empty [`FuturesUnorderedBounded`] with a fixed capacity that is the length of the iterator.
    ///
    /// # Example
    ///
    /// Making 1024 total HTTP requests, with a max concurrency of 128
    ///
    /// ```
    /// use futures::future::Future;
    /// use futures::stream::StreamExt;
    /// use futures_buffered::FuturesUnorderedBounded;
    /// use hyper::client::conn::http1::{handshake, SendRequest};
    /// use hyper::body::Incoming;
    /// use hyper::{Request, Response};
    /// use hyper_util::rt::TokioIo;
    /// use tokio::net::TcpStream;
    ///
    /// # #[cfg(miri)] fn main() {}
    /// # #[cfg(not(miri))] #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // create a tcp connection
    /// let stream = TcpStream::connect("example.com:80").await?;
    ///
    /// // perform the http handshakes
    /// let (mut rs, conn) = handshake(TokioIo::new(stream)).await?;
    /// tokio::spawn(conn);
    ///
    /// /// make http request to example.com and read the response
    /// fn make_req(rs: &mut SendRequest<String>) -> impl Future<Output = hyper::Result<Response<Incoming>>> {
    ///     let req = Request::builder()
    ///         .header("Host", "example.com")
    ///         .method("GET")
    ///         .body(String::new())
    ///         .unwrap();
    ///     rs.send_request(req)
    /// }
    ///
    /// // create a queue with an initial 128 concurrent requests
    /// let mut queue: FuturesUnorderedBounded<_> = (0..128).map(|_| make_req(&mut rs)).collect();
    ///
    /// // wait for a request to finish and start another to fill its place - up to 1024 total requests
    /// for _ in 128..1024 {
    ///     queue.next().await;
    ///     queue.push(make_req(&mut rs));
    /// }
    /// // wait for the tail end to finish
    /// for _ in 0..128 {
    ///     queue.next().await;
    /// }
    /// # Ok(()) }
    /// ```
    fn from_iter<T: IntoIterator<Item = F>>(iter: T) -> Self {
        // store the futures in our task list
        let tasks = SlotMap::from_iter(iter);

        // determine the actual capacity and create the shared state
        let cap = tasks.len();
        let shared = ArcSlice::new(cap);

        for i in 0..cap {
            // safety: i is always within capacity
            unsafe {
                shared.push(i);
            }
        }

        // create the queue
        Self { tasks, shared }
    }
}

impl<Fut: Future> FusedStream for FuturesUnorderedBounded<Fut> {
    fn is_terminated(&self) -> bool {
        self.is_empty()
    }
}

impl<Fut> fmt::Debug for FuturesUnorderedBounded<Fut> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FuturesUnorderedBounded")
            .field("len", &self.tasks.len())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::{
        cell::Cell,
        future::{poll_fn, ready},
        time::Duration,
    };
    use futures::{channel::oneshot, StreamExt};
    use futures_test::task::noop_context;
    use pin_project_lite::pin_project;
    use std::time::Instant;

    pin_project!(
        struct PollCounter<'c, F> {
            count: &'c Cell<usize>,
            #[pin]
            inner: F,
        }
    );

    impl<F: Future> Future for PollCounter<'_, F> {
        type Output = F::Output;
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            self.count.set(self.count.get() + 1);
            self.project().inner.poll(cx)
        }
    }

    struct Yield {
        done: bool,
    }
    impl Unpin for Yield {}
    impl Future for Yield {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.as_mut().done {
                Poll::Ready(())
            } else {
                cx.waker().wake_by_ref();
                self.as_mut().done = true;
                Poll::Pending
            }
        }
    }

    fn yield_now(count: &Cell<usize>) -> PollCounter<'_, Yield> {
        PollCounter {
            count,
            inner: Yield { done: false },
        }
    }

    #[test]
    fn single() {
        let c = Cell::new(0);

        let mut buffer = FuturesUnorderedBounded::new(10);
        buffer.push(yield_now(&c));
        futures::executor::block_on(buffer.next());

        drop(buffer);
        assert_eq!(c.into_inner(), 2);
    }

    #[test]
    #[should_panic(expected = "attempted to push into a full `FuturesUnorderedBounded`")]
    fn full() {
        let mut buffer = FuturesUnorderedBounded::new(1);
        buffer.push(ready(()));
        buffer.push(ready(()));
    }

    #[test]
    fn len() {
        let mut buffer = FuturesUnorderedBounded::new(1);

        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert_eq!(buffer.capacity(), 1);
        assert_eq!(buffer.size_hint(), (0, Some(0)));
        assert!(buffer.is_terminated());

        buffer.push(ready(()));

        assert_eq!(buffer.len(), 1);
        assert!(!buffer.is_empty());
        assert_eq!(buffer.capacity(), 1);
        assert_eq!(buffer.size_hint(), (1, Some(1)));
        assert!(!buffer.is_terminated());

        futures::executor::block_on(buffer.next());

        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert_eq!(buffer.capacity(), 1);
        assert_eq!(buffer.size_hint(), (0, Some(0)));
        assert!(buffer.is_terminated());
    }

    #[test]
    fn from_iter() {
        let buffer = FuturesUnorderedBounded::from_iter((0..10).map(|_| ready(())));

        assert_eq!(buffer.len(), 10);
        assert_eq!(buffer.capacity(), 10);
        assert_eq!(buffer.size_hint(), (10, Some(10)));
    }

    #[test]
    fn drop_while_waiting() {
        let mut buffer = FuturesUnorderedBounded::new(10);
        let waker = Cell::new(None);
        buffer.push(poll_fn(|cx| {
            waker.set(Some(cx.waker().clone()));
            Poll::<()>::Pending
        }));

        assert_eq!(buffer.poll_next_unpin(&mut noop_context()), Poll::Pending);
        drop(buffer);

        let cx = waker.take().unwrap();
        drop(cx);
    }

    #[test]
    fn multi() {
        fn wait(count: &Cell<usize>) -> PollCounter<'_, Yield> {
            yield_now(count)
        }

        let c = Cell::new(0);

        let mut buffer = FuturesUnorderedBounded::new(10);
        // build up
        for _ in 0..10 {
            buffer.push(wait(&c));
        }
        // poll and insert
        for _ in 0..100 {
            assert!(futures::executor::block_on(buffer.next()).is_some());
            buffer.push(wait(&c));
        }
        // drain down
        for _ in 0..10 {
            assert!(futures::executor::block_on(buffer.next()).is_some());
        }

        let count = c.into_inner();
        assert_eq!(count, 220);
    }

    #[test]
    fn very_slow_task() {
        let c = Cell::new(0);

        let now = Instant::now();

        let mut buffer = FuturesUnorderedBounded::new(10);
        // build up
        for _ in 0..9 {
            buffer.push(yield_now(&c));
        }
        // spawn a slow future among a bunch of fast ones.
        // the test is to make sure this doesn't block the rest getting completed
        buffer.push(yield_now(&c));
        // poll and insert
        for _ in 0..100 {
            assert!(futures::executor::block_on(buffer.next()).is_some());
            buffer.push(yield_now(&c));
        }
        // drain down
        for _ in 0..10 {
            assert!(futures::executor::block_on(buffer.next()).is_some());
        }

        let dur = now.elapsed();
        assert!(dur < Duration::from_millis(2050));

        let count = c.into_inner();
        assert_eq!(count, 220);
    }

    #[cfg(not(miri))]
    #[tokio::test]
    async fn unordered_large() {
        for i in 0..256 {
            let mut queue: FuturesUnorderedBounded<_> = ((0..i).map(|_| async move {
                tokio::time::sleep(Duration::from_nanos(1)).await;
            }))
            .collect();
            for _ in 0..i {
                queue.next().await.unwrap();
            }
        }
    }

    #[test]
    fn correct_fairer_order() {
        const LEN: usize = 256;

        let mut buffer = FuturesUnorderedBounded::new(LEN);
        let mut txs = vec![];
        for _ in 0..LEN {
            let (tx, rx) = oneshot::channel();
            buffer.push(rx);
            txs.push(tx);
        }

        for _ in 0..=(LEN / 61) {
            assert!(buffer.poll_next_unpin(&mut noop_context()).is_pending());
        }

        for (i, tx) in txs.into_iter().enumerate() {
            let _ = tx.send(i);
        }

        for i in 0..LEN {
            let poll = buffer.poll_next_unpin(&mut noop_context());
            assert_eq!(poll, Poll::Ready(Some(Ok(i))));
        }
    }
}
