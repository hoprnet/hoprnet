use alloc::vec::Vec;
use core::{
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::FuturesUnorderedBounded;
use futures_core::{FusedStream, Stream};

/// A set of futures which may complete in any order.
///
/// Much like [`futures::stream::FuturesUnordered`](https://docs.rs/futures/0.3.25/futures/stream/struct.FuturesUnordered.html),
/// this is a thread-safe, `Pin` friendly, lifetime friendly, concurrent processing stream.
///
/// The is different to [`FuturesUnorderedBounded`] because it doesn't have a fixed capacity.
/// It still manages to achieve good efficiency however
///
/// ## Benchmarks
///
/// All benchmarks are run with `FuturesUnordered::new()`, no predefined capacity.
///
/// ### Speed
///
/// Running 65536 100us timers with 256 concurrent jobs in a single threaded tokio runtime:
///
/// ```text
/// futures::FuturesUnordered time:   [412.52 ms 414.47 ms 416.41 ms]
/// crate::FuturesUnordered   time:   [412.96 ms 414.69 ms 416.65 ms]
/// FuturesUnorderedBounded   time:   [361.81 ms 362.96 ms 364.13 ms]
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
/// futures::FuturesUnordered
///     count:    1024002
///     alloc:    40960144 B
///     dealloc:  40960000 B
///
/// crate::FuturesUnordered
///     count:    9
///     alloc:    15840 B
///     dealloc:  0 B
/// ```
///
/// ### Conclusion
///
/// As you can see, our `FuturesUnordered` massively reduces you memory overhead while maintaining good performance.
///
/// # Example
///
/// Making 1024 total HTTP requests, with a max concurrency of 128
///
/// ```
/// use futures::future::Future;
/// use futures::stream::StreamExt;
/// use futures_buffered::FuturesUnordered;
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
/// let mut queue = FuturesUnordered::with_capacity(128);
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
pub struct FuturesUnordered<F> {
    rem: usize,
    pub(crate) groups: Vec<FuturesUnorderedBounded<F>>,
    poll_next: usize,
}

pub(crate) const MIN_CAPACITY: usize = 32;

impl<F> Unpin for FuturesUnordered<F> {}

impl<F> Default for FuturesUnordered<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F> FuturesUnordered<F> {
    /// Constructs a new, empty [`FuturesUnordered`].
    ///
    /// The returned [`FuturesUnordered`] does not contain any futures.
    /// In this state, [`FuturesUnordered::poll_next`](Stream::poll_next) will
    /// return [`Poll::Ready(None)`](Poll::Ready).
    pub const fn new() -> Self {
        Self {
            rem: 0,
            groups: Vec::new(),
            poll_next: 0,
        }
    }

    /// Constructs a new, empty [`FuturesUnordered`] with the given fixed capacity.
    ///
    /// The returned [`FuturesUnordered`] does not contain any futures.
    /// In this state, [`FuturesUnordered::poll_next`](Stream::poll_next) will
    /// return [`Poll::Ready(None)`](Poll::Ready).
    pub fn with_capacity(n: usize) -> Self {
        if n > 0 {
            Self {
                rem: 0,
                groups: Vec::from_iter([FuturesUnorderedBounded::new(n)]),
                poll_next: 0,
            }
        } else {
            Self::new()
        }
    }

    /// Push a future into the set.
    ///
    /// This method adds the given future to the set. This method will not
    /// call [`poll`](core::future::Future::poll) on the submitted future. The caller must
    /// ensure that [`FuturesUnordered::poll_next`](Stream::poll_next) is called
    /// in order to receive wake-up notifications for the given future.
    pub fn push(&mut self, fut: F) {
        self.rem += 1;

        let last = match self.groups.last_mut() {
            Some(last) => last,
            None => {
                self.groups.push(FuturesUnorderedBounded::new(MIN_CAPACITY));
                self.groups
                    .last_mut()
                    .expect("group should have at least one entry")
            }
        };
        match last.try_push(fut) {
            Ok(()) => {}
            Err(future) => {
                let mut next = FuturesUnorderedBounded::new(last.capacity() * 2);
                next.push(future);
                self.groups.push(next);
            }
        }
    }

    /// Returns `true` if the set contains no futures.
    pub fn is_empty(&self) -> bool {
        self.rem == 0
    }

    /// Returns the number of futures contained in the set.
    ///
    /// This represents the total number of in-flight futures.
    pub fn len(&self) -> usize {
        self.rem
    }

    /// Returns the number of futures that can be contained in the set.
    pub fn capacity(&self) -> usize {
        match self.groups.as_slice() {
            [] => 0,
            [only] => only.capacity(),
            [.., last] => {
                let spare_cap = last.capacity() - last.len();
                self.rem + spare_cap
            }
        }
    }
}

impl<F: Future> Stream for FuturesUnordered<F> {
    type Item = F::Output;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let Self {
            rem,
            groups,
            poll_next,
        } = &mut *self;
        if groups.is_empty() {
            return Poll::Ready(None);
        }

        for _ in 0..groups.len() {
            if *poll_next >= groups.len() {
                *poll_next = 0;
            }

            let poll = Pin::new(&mut groups[*poll_next]).poll_next(cx);
            match poll {
                Poll::Ready(Some(x)) => {
                    *rem -= 1;
                    return Poll::Ready(Some(x));
                }
                Poll::Ready(None) => {
                    let group = groups.remove(*poll_next);
                    debug_assert!(group.is_empty());

                    if groups.is_empty() {
                        // group should contain at least 1 set
                        groups.push(group);
                        debug_assert_eq!(*rem, 0);
                        return Poll::Ready(None);
                    }

                    // we do not want to drop the last set as it contains
                    // the largest allocation that we want to keep a hold of
                    if *poll_next == groups.len() {
                        groups.push(group);
                        *poll_next = 0;
                    }
                }
                Poll::Pending => {
                    *poll_next += 1;
                }
            }
        }
        Poll::Pending
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.rem, Some(self.rem))
    }
}
impl<F: Future> FusedStream for FuturesUnordered<F> {
    fn is_terminated(&self) -> bool {
        self.is_empty()
    }
}

impl<F> FromIterator<F> for FuturesUnordered<F> {
    /// Constructs a new, empty [`FuturesUnordered`] with a fixed capacity that is the length of the iterator.
    ///
    /// # Example
    ///
    /// Making 1024 total HTTP requests, with a max concurrency of 128
    ///
    /// ```
    /// use futures::future::Future;
    /// use futures::stream::StreamExt;
    /// use futures_buffered::FuturesUnordered;
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
    /// let mut queue: FuturesUnordered<_> = (0..128).map(|_| make_req(&mut rs)).collect();
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
        let iter = iter.into_iter();
        let mut this =
            FuturesUnordered::with_capacity(usize::max(iter.size_hint().0, MIN_CAPACITY));
        for fut in iter {
            this.push(fut);
        }
        this
    }
}

impl<Fut> fmt::Debug for FuturesUnordered<Fut> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FuturesUnordered")
            .field("queues", &self.groups)
            .field("len", &self.rem)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::{cell::Cell, future::ready, time::Duration};
    use futures::StreamExt;
    use pin_project_lite::pin_project;
    use std::{thread, time::Instant};

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

    struct Sleep {
        until: Instant,
    }
    impl Unpin for Sleep {}
    impl Future for Sleep {
        type Output = ();

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let until = self.until;
            if until > Instant::now() {
                let waker = cx.waker().clone();
                thread::spawn(move || {
                    thread::sleep(until.duration_since(Instant::now()));
                    waker.wake();
                });
                Poll::Pending
            } else {
                Poll::Ready(())
            }
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

        let mut buffer = FuturesUnordered::new();
        buffer.push(yield_now(&c));
        futures::executor::block_on(buffer.next());

        drop(buffer);
        assert_eq!(c.into_inner(), 2);
    }

    #[test]
    fn len() {
        let mut buffer = FuturesUnordered::with_capacity(1);

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

        buffer.push(ready(()));

        assert_eq!(buffer.len(), 2);
        assert!(!buffer.is_empty());
        assert_eq!(buffer.capacity(), 3);
        assert_eq!(buffer.size_hint(), (2, Some(2)));
        assert!(!buffer.is_terminated());

        futures::executor::block_on(buffer.next());
        futures::executor::block_on(buffer.next());

        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert_eq!(buffer.capacity(), 2);
        assert_eq!(buffer.size_hint(), (0, Some(0)));
        assert!(buffer.is_terminated());
    }

    #[test]
    fn from_iter() {
        let buffer = FuturesUnordered::from_iter((0..10).map(|_| ready(())));

        assert_eq!(buffer.len(), 10);
        assert_eq!(buffer.capacity(), 32);
        assert_eq!(buffer.size_hint(), (10, Some(10)));
    }

    #[test]
    fn multi() {
        fn wait(count: &Cell<usize>) -> PollCounter<'_, Yield> {
            yield_now(count)
        }

        let c = Cell::new(0);

        let mut buffer = FuturesUnordered::with_capacity(1);
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

        let mut buffer = FuturesUnordered::with_capacity(1);
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
}
