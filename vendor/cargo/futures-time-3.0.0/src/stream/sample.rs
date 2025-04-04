use pin_project_lite::pin_project;

use futures_core::stream::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project! {
    /// Yield the last value received, if any, at each interval.
    ///
    /// If no value was emitted during the last interval, no value is emitted
    /// and we skip to the next interval.
    ///
    /// This `struct` is created by the [`sample`] method on [`StreamExt`]. See its
    /// documentation for more.
    ///
    /// [`sample`]: crate::stream::StreamExt::sample
    /// [`StreamExt`]: crate::stream::StreamExt
    #[derive(Debug)]
    #[must_use = "streams do nothing unless polled or .awaited"]
    pub struct Sample<S: Stream, I> {
        #[pin]
        stream: S,
        #[pin]
        interval: I,
        state: State,
        slot: Option<S::Item>,
    }
}

impl<S: Stream, I> Sample<S, I> {
    pub(crate) fn new(stream: S, interval: I) -> Self {
        Self {
            state: State::Streaming,
            stream,
            interval,
            slot: None,
        }
    }
}

#[derive(Debug)]
enum State {
    /// The underlying stream is yielding items.
    Streaming,
    /// All timers have completed and all data has been yielded.
    StreamDone,
    /// The closing `Ready(None)` has been yielded.
    AllDone,
}

impl<S: Stream, I: Stream> Stream for Sample<S, I> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        match this.state {
            // The underlying stream is yielding items.
            State::Streaming => {
                // Poll the underlying stream until we get to `Poll::Pending`.
                loop {
                    match this.stream.as_mut().poll_next(cx) {
                        Poll::Ready(Some(value)) => {
                            let _ = this.slot.insert(value);
                        }
                        Poll::Ready(None) => {
                            *this.state = State::StreamDone;
                            break;
                        }
                        Poll::Pending => break,
                    }
                }

                // After the stream, always poll the interval timer.
                match this.interval.as_mut().poll_next(cx) {
                    Poll::Ready(_) => {
                        if let State::StreamDone = this.state {
                            cx.waker().wake_by_ref();
                        }
                        match this.slot.take() {
                            Some(item) => Poll::Ready(Some(item)),
                            None => Poll::Pending,
                        }
                    }
                    Poll::Pending => Poll::Pending,
                }
            }

            // All streams have completed and all data has been yielded.
            State::StreamDone => {
                *this.state = State::AllDone;
                Poll::Ready(None)
            }

            // The closing `Ready(None)` has been yielded.
            State::AllDone => panic!("stream polled after completion"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use crate::time::Duration;
    use futures_lite::prelude::*;

    #[test]
    fn smoke() {
        async_io::block_on(async {
            let interval = Duration::from_millis(100);
            let throttle = Duration::from_millis(200);

            let take = 4;
            let expected = 2;

            let mut counter = 0;
            crate::stream::interval(interval)
                .take(take)
                .sample(throttle)
                .for_each(|_| counter += 1)
                .await;

            assert_eq!(counter, expected);
        })
    }
}
