use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Stream, StreamExt};

/// Represents a sink that will time out after a given duration if an item
/// cannot be sent.
#[pin_project::pin_project]
pub struct TimeoutSink<S> {
    #[pin]
    inner: S,
    #[pin]
    timer: Option<futures_time::task::Sleep>,
    timeout: std::time::Duration,
}

/// Error type for [`TimeoutSink`].
#[derive(Debug, thiserror::Error, strum::EnumTryAs)]
pub enum SinkTimeoutError<E> {
    /// Inner sink could not make progress within the timeout.
    #[error("sink timed out")]
    Timeout,
    /// Inner sink returned an error.
    #[error("inner sink error: {0}")]
    Inner(E),
}

impl<I, S: futures::Sink<I>> futures::Sink<I> for TimeoutSink<S> {
    type Error = SinkTimeoutError<S::Error>;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut this = self.project();

        // First, see if we can make progress on the inner sink.
        match this.inner.poll_ready(cx) {
            Poll::Ready(res) => {
                // The inner sink is ready, so we can clear the timer.
                this.timer.set(None);
                Poll::Ready(res.map_err(SinkTimeoutError::Inner))
            }
            Poll::Pending => {
                if this.timer.is_none() {
                    // If no timer is present, create one with the given timeout.
                    this.timer
                        .set(Some(futures_time::task::sleep(futures_time::time::Duration::from(
                            *this.timeout,
                        ))));
                }

                // If a timer is present, poll it as well
                if let Some(timer) = this.timer.as_mut().as_pin_mut() {
                    futures::ready!(timer.poll(cx));
                    this.timer.set(None);
                    // The timer has expired, so we won't poll the inner sink again
                    // and return an error.
                    Poll::Ready(Err(SinkTimeoutError::Timeout))
                } else {
                    // Cannot happen as the timer is always set at this point
                    unreachable!();
                }
            }
        }
    }

    fn start_send(self: Pin<&mut Self>, item: I) -> Result<(), Self::Error> {
        self.project().inner.start_send(item).map_err(SinkTimeoutError::Inner)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_flush(cx).map_err(SinkTimeoutError::Inner)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_close(cx).map_err(SinkTimeoutError::Inner)
    }
}

impl<S: Clone> Clone for TimeoutSink<S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            timer: None,
            timeout: self.timeout,
        }
    }
}

/// [`futures::Sink`] adaptor that adds timeout.
pub trait TimeoutSinkExt<I>: futures::Sink<I> {
    /// Attaches a timeout onto this [`futures::Sink`]'s `poll_ready` function.
    ///
    /// The returned `Sink` will return an error if `poll_ready` does not
    /// return within the given `timeout`.
    fn with_timeout(self, timeout: std::time::Duration) -> TimeoutSink<Self>
    where
        Self: Sized,
    {
        TimeoutSink {
            inner: self,
            timer: None,
            timeout,
        }
    }
}

impl<T: ?Sized, I> TimeoutSinkExt<I> for T where T: futures::Sink<I> {}

#[pin_project::pin_project]
pub struct ForwardWithTimeout<St, Si, Item> {
    #[pin]
    sink: Option<Si>,
    #[pin]
    stream: futures::stream::Fuse<St>,
    buffered_item: Option<Item>,
}

impl<St: futures::Stream, Si, Item> ForwardWithTimeout<St, Si, Item> {
    pub(crate) fn new(stream: St, sink: Si) -> Self {
        Self {
            sink: Some(sink),
            stream: stream.fuse(),
            buffered_item: None,
        }
    }
}

impl<St, Si, Item, E> futures::future::FusedFuture for ForwardWithTimeout<St, Si, Item>
where
    Si: futures::Sink<Item, Error = SinkTimeoutError<E>>,
    St: Stream<Item = Result<Item, E>>,
{
    fn is_terminated(&self) -> bool {
        self.sink.is_none()
    }
}

impl<St, Si, Item, E> Future for ForwardWithTimeout<St, Si, Item>
where
    Si: futures::Sink<Item, Error = SinkTimeoutError<E>>,
    St: Stream<Item = Result<Item, E>>,
{
    type Output = Result<(), E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        let mut si = this
            .sink
            .as_mut()
            .as_pin_mut()
            .expect("polled `Forward` after completion");

        loop {
            // If we've got an item buffered already, we need to try to write it to the
            // sink before we can do anything else
            if this.buffered_item.is_some() {
                match futures::ready!(si.as_mut().poll_ready(cx)) {
                    Ok(_) => {
                        si.as_mut()
                            .start_send(this.buffered_item.take().unwrap())
                            .map_err(|e| e.try_as_inner().unwrap())?;
                    }
                    Err(SinkTimeoutError::Timeout) => {
                        // If there was a timeout, drop the buffered item and continue
                        // polling the stream for the next one.
                        *this.buffered_item = None;
                        continue;
                    }
                    Err(SinkTimeoutError::Inner(e)) => return Poll::Ready(Err(e)),
                }
            }

            match this.stream.as_mut().poll_next(cx)? {
                Poll::Ready(Some(item)) => {
                    *this.buffered_item = Some(item);
                }
                Poll::Ready(None) => {
                    futures::ready!(si.poll_close(cx)).map_err(|e| e.try_as_inner().unwrap())?;
                    this.sink.set(None);
                    return Poll::Ready(Ok(()));
                }
                Poll::Pending => {
                    futures::ready!(si.poll_flush(cx)).map_err(|e| e.try_as_inner().unwrap())?;
                    return Poll::Pending;
                }
            }
        }
    }
}

/// [`futures::TryStream`] extension that allows forwarding items to a sink with a timeout while
/// discarding timed out items.
pub trait TimeoutStreamExt: futures::TryStream {
    /// Specialization of [`StreamExt::forward`] for Sinks using the [`SinkTimeoutError`].
    ///
    /// If the `sink` returns [`SinkTimeoutError::Timeout`], the current item from this
    /// stream is discarded and the forwarding process continues with the next item
    /// until the stream is depleted.
    ///
    /// This is in contrast to [`StreamExt::forward`] which would terminate with [`SinkTimeoutError::Timeout`].
    ///
    /// Errors other than [`SinkTimeoutError::Timeout`] cause the forwarding to terminate
    /// with that error (as in the original behavior of [`StreamExt::forward`]).
    fn forward_to_timeout<S>(self, sink: S) -> ForwardWithTimeout<Self, S, Self::Ok>
    where
        S: futures::Sink<Self::Ok, Error = SinkTimeoutError<Self::Error>>,
        Self: Sized,
    {
        ForwardWithTimeout::new(self, sink)
    }
}

impl<T: ?Sized> TimeoutStreamExt for T where T: futures::TryStream {}

#[cfg(test)]
mod tests {
    use futures::SinkExt;

    use super::*;

    #[derive(Default)]
    struct FixedSink<const N: usize, I>(Vec<I>);

    impl<const N: usize, I> futures::Sink<I> for FixedSink<N, I> {
        type Error = std::convert::Infallible;

        fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            if self.0.len() < N {
                Poll::Ready(Ok(()))
            } else {
                Poll::Pending
            }
        }

        fn start_send(self: Pin<&mut Self>, item: I) -> Result<(), Self::Error> {
            // SAFETY: We're not moving any pinned data, just mutating the Vec in place
            let this = unsafe { self.get_unchecked_mut() };
            this.0.push(item);
            Ok(())
        }

        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
    }

    #[test_log::test(tokio::test)]
    async fn test_timeout_sink() -> anyhow::Result<()> {
        let mut sink = FixedSink::<1, i32>::default();

        {
            let mut timed_sink = (&mut sink).with_timeout(std::time::Duration::from_millis(10));

            timed_sink.send(10).await?;
            assert!(matches!(timed_sink.send(20).await, Err(SinkTimeoutError::Timeout)));
        }

        assert_eq!(1, sink.0.len());
        sink.0.remove(0);

        {
            let mut timed_sink = (&mut sink).with_timeout(std::time::Duration::from_millis(10));

            timed_sink.send(10).await?;
            assert!(matches!(timed_sink.send(20).await, Err(SinkTimeoutError::Timeout)));
        }

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_forward_with_timeout() -> anyhow::Result<()> {
        let stream = futures::stream::iter([1, 2, 3, 4, 5]).map(Ok);

        let mut sink = FixedSink::<2, i32>::default();

        let start = std::time::Instant::now();
        stream
            .forward_to_timeout((&mut sink).with_timeout(std::time::Duration::from_millis(10)))
            .await?;
        assert!(
            start.elapsed() > std::time::Duration::from_millis(29),
            "should've taken at least 30ms"
        );

        assert_eq!(2, sink.0.len());
        assert_eq!(1, sink.0[0]);
        assert_eq!(2, sink.0[1]);

        Ok(())
    }
}
