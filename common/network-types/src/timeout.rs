use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

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
#[derive(Debug, strum::Display)]
pub enum SinkTimeoutError<E> {
    /// Inner sink could not make progress within the timeout.
    Timeout,
    /// Inner sink returned an error.
    Inner(E),
}
impl<E: std::error::Error> std::error::Error for SinkTimeoutError<E> {}

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
}
