use crate::channel::Parker;
use crate::future::{IntoFuture, Timer};

use futures_core::Stream;

use super::{Buffer, Debounce, Delay, IntoStream, Park, Sample, Throttle, Timeout};

/// Extend `Stream` with time-based operations.
pub trait StreamExt: Stream {
    /// Yield the last item received at the end of each interval.
    ///
    /// If no items have been received during an interval, the stream will not
    /// yield any items. In addition to using a time-based interval, this method can take any
    /// stream as a source. This enables throttling based on alternative event
    /// sources, such as variable-rate timers.
    ///
    /// See also [`throttle()`] and [`debounce()`].
    ///
    /// [`throttle()`]: StreamExt::throttle
    /// [`debounce()`]: `StreamExt::debounce`
    ///
    /// # Data Loss
    ///
    /// This method will discard data between intervals. Though the
    /// discarded items will have their destuctors run, __using this method
    /// incorrectly may lead to unintended data loss__. This method is best used
    /// to reduce the number of _duplicate_ items after the first has been
    /// received, such as repeated mouse clicks or key presses. This method may
    /// lead to unintended data loss when used to discard _unique_ items, such
    /// as network request.
    ///
    /// # Example
    ///
    /// ```
    /// use futures_lite::prelude::*;
    /// use futures_time::prelude::*;
    /// use futures_time::time::{Instant, Duration};
    /// use futures_time::stream;
    ///
    /// fn main() {
    ///    async_io::block_on(async {
    ///        let mut counter = 0;
    ///        stream::interval(Duration::from_millis(100))
    ///            .take(4)
    ///            .sample(Duration::from_millis(200))
    ///            .for_each(|_| counter += 1)
    ///            .await;
    ///
    ///        assert_eq!(counter, 2);
    ///    })
    /// }
    /// ```
    fn sample<I>(self, interval: I) -> Sample<Self, I::IntoStream>
    where
        Self: Sized,
        I: IntoStream,
    {
        Sample::new(self, interval.into_stream())
    }

    /// Group items into vectors which are yielded at every interval.
    ///
    /// In addition to using a time source as a deadline, any stream can be used as a
    /// deadline too. This enables more interesting buffer strategies to be
    /// built on top of this primitive.
    ///
    /// # Future Improvements
    ///
    /// - Lending iterators would allow for internal reusing of the buffer.
    /// Though different from `Iterator::windows`, it could be more efficient.
    /// - Contexts/capabilities would enable custom allocators to be used.
    ///
    /// # Example
    ///
    /// ```
    /// use futures_lite::prelude::*;
    /// use futures_time::prelude::*;
    /// use futures_time::time::{Instant, Duration};
    /// use futures_time::stream;
    ///
    /// fn main() {
    ///     async_io::block_on(async {
    ///         let mut counter = 0;
    ///         stream::interval(Duration::from_millis(5))
    ///             .take(10)
    ///             .buffer(Duration::from_millis(20))
    ///             .for_each(|buf| counter += buf.len())
    ///             .await;
    ///
    ///         assert_eq!(counter, 10);
    ///     })
    /// }
    /// ```
    fn buffer<I>(self, interval: I) -> Buffer<Self, I::IntoStream>
    where
        Self: Sized,
        I: IntoStream,
    {
        Buffer::new(self, interval.into_stream())
    }

    /// Yield the last item received at the end of a window which resets with
    /// each item received.
    ///
    /// Every time an item is yielded by the underlying stream, the window is
    /// reset. Once the window expires, the last item seen will be yielded. This
    /// means that in order to yield an item, no items must be received for the
    /// entire window, or else the window will reset.
    ///
    /// This method is useful to perform actions at the end of bursts of events,
    /// where performing that same action on _every_ event might not be
    /// economical.
    ///
    /// See also [`sample()`] and [`throttle()`].
    ///
    /// [`sample()`]: `StreamExt::sample`
    /// [`throttle()`]: `StreamExt::throttle`
    ///
    /// # Example
    ///
    /// ```
    /// use futures_lite::prelude::*;
    /// use futures_time::prelude::*;
    /// use futures_time::time::{Instant, Duration};
    /// use futures_time::stream;
    ///
    /// fn main() {
    ///     async_io::block_on(async {
    ///         let mut counter = 0;
    ///         stream::interval(Duration::from_millis(10))
    ///             .take(10)
    ///             .debounce(Duration::from_millis(20)) // the window is greater than the interval
    ///             .for_each(|_| counter += 1)
    ///             .await;
    ///
    ///         assert_eq!(counter, 1); // so only the last item is received
    ///     })
    /// }
    /// ```
    fn debounce<D>(self, window: D) -> Debounce<Self, D::IntoFuture>
    where
        Self: Sized,
        D: IntoFuture,
        D::IntoFuture: Timer,
    {
        Debounce::new(self, window.into_future())
    }

    /// Delay the yielding of items from the stream until the given deadline.
    ///
    /// The underlying stream will not be polled until the deadline has expired. In addition
    /// to using a time source as a deadline, any future can be used as a
    /// deadline too. When used in combination with a multi-consumer channel,
    /// this method can be used to synchronize the start of multiple streams and futures.
    ///
    /// # Example
    ///
    /// ```
    /// use futures_lite::prelude::*;
    /// use futures_time::prelude::*;
    /// use futures_time::time::{Instant, Duration};
    /// use futures_lite::stream;
    ///
    /// fn main() {
    ///     async_io::block_on(async {
    ///         let now = Instant::now();
    ///         let delay = Duration::from_millis(100);
    ///         let _ = stream::once("meow").delay(delay).next().await;
    ///         assert!(now.elapsed() >= *delay);
    ///     });
    /// }
    /// ```
    fn delay<D>(self, deadline: D) -> Delay<Self, D::IntoFuture>
    where
        Self: Sized,
        D: IntoFuture,
    {
        Delay::new(self, deadline.into_future())
    }

    /// Suspend or resume execution of a stream.
    ///
    /// When this method is called the execution of the stream will be put into
    /// a suspended state until the channel returns `Parker::Unpark` or the
    /// channel's senders are dropped. The underlying stream will not be polled
    /// while the it is paused.
    fn park<I>(self, interval: I) -> Park<Self, I::IntoStream>
    where
        Self: Sized,
        I: IntoStream<Item = Parker>,
    {
        Park::new(self, interval.into_stream())
    }

    /// Yield an item, then ignore subsequent items for a duration.
    ///
    /// In addition to using a time-based interval, this method can take any
    /// stream as a source. This enables throttling based on alternative event
    /// sources, such as variable-rate timers.
    ///
    /// See also [`sample()`] and [`debounce()`].
    ///
    /// [`sample()`]: `StreamExt::sample`
    /// [`debounce()`]: `StreamExt::debounce`
    ///
    /// # Data Loss
    ///
    /// This method will discard data between intervals. Though the
    /// discarded items will have their destuctors run, __using this method
    /// incorrectly may lead to unintended data loss__. This method is best used
    /// to reduce the number of _duplicate_ items after the first has been
    /// received, such as repeated mouse clicks or key presses. This method may
    /// lead to unintended data loss when used to discard _unique_ items, such
    /// as network request.
    ///
    /// # Examples
    ///
    /// ```
    /// use futures_lite::prelude::*;
    /// use futures_time::prelude::*;
    /// use futures_time::time::Duration;
    /// use futures_time::stream;
    ///
    /// fn main() {
    ///     async_io::block_on(async {
    ///         let mut counter = 0;
    ///         stream::interval(Duration::from_millis(100))  // Yield an item every 100ms
    ///             .take(4)                                  // Stop after 4 items
    ///             .throttle(Duration::from_millis(300))     // Only let an item through every 300ms
    ///             .for_each(|_| counter += 1)               // Increment a counter for each item
    ///             .await;
    ///
    ///         assert_eq!(counter, 2);
    ///     })
    /// }
    /// ```
    fn throttle<I>(self, interval: I) -> Throttle<Self, I::IntoStream>
    where
        Self: Sized,
        I: IntoStream,
    {
        Throttle::new(self, interval.into_stream())
    }

    /// Return an error if a stream does not yield an item within a given time
    /// span.
    ///
    /// Typically timeouts are, as the name implies, based on _time_. However
    /// this method can time out based on any future. This can be useful in
    /// combination with channels, as it allows (long-lived) streams to be
    /// cancelled based on some external event.
    ///
    /// When a timeout is returned, the stream will be dropped and destructors
    /// will be run.
    ///
    /// # Example
    ///
    /// ```
    /// use futures_lite::prelude::*;
    /// use futures_time::prelude::*;
    /// use futures_time::time::{Instant, Duration};
    /// use futures_lite::stream;
    /// use std::io;
    ///
    /// fn main() {
    ///     async_io::block_on(async {
    ///         let res = stream::once("meow")
    ///             .delay(Duration::from_millis(100))  // longer delay
    ///             .timeout(Duration::from_millis(50)) // shorter timeout
    ///             .next()
    ///             .await;
    ///         assert_eq!(res.unwrap().unwrap_err().kind(), io::ErrorKind::TimedOut); // error
    ///
    ///         let res = stream::once("meow")
    ///             .delay(Duration::from_millis(50))    // shorter delay
    ///             .timeout(Duration::from_millis(100)) // longer timeout
    ///             .next()
    ///             .await;
    ///         assert_eq!(res.unwrap().unwrap(), "meow"); // success
    ///     });
    /// }
    /// ```
    fn timeout<D>(self, deadline: D) -> Timeout<Self, D::IntoFuture>
    where
        Self: Sized,
        D: IntoFuture,
        D::IntoFuture: Timer,
    {
        Timeout::new(self, deadline.into_future())
    }
}

impl<S> StreamExt for S where S: Stream {}
