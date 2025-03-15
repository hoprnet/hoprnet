use pin_project_lite::pin_project;

use futures_core::stream::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project! {
    /// Filter out all items after the first for a specified time.
    ///
    /// This `struct` is created by the [`throttle`] method on [`StreamExt`]. See its
    /// documentation for more.
    ///
    /// [`throttle`]: crate::stream::StreamExt::throttle
    /// [`StreamExt`]: crate::stream::StreamExt
    #[derive(Debug)]
    #[must_use = "streams do nothing unless polled or .awaited"]
    pub struct Throttle<S: Stream, I> {
        #[pin]
        stream: S,
        #[pin]
        interval: I,
        state: State,
        budget: usize,
    }
}

impl<S: Stream, I> Throttle<S, I> {
    pub(crate) fn new(stream: S, interval: I) -> Self {
        Self {
            state: State::Streaming(0),
            stream,
            interval,
            budget: 1,
        }
    }
}

#[derive(Debug)]
enum State {
    /// The underlying stream is yielding items.
    Streaming(usize),
    /// All timers have completed and all data has been yielded.
    StreamDone,
    /// The closing `Ready(None)` has been yielded.
    AllDone,
}

impl<S: Stream, I: Stream> Stream for Throttle<S, I> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        let mut slot = None;

        match this.state {
            // The underlying stream is yielding items.
            State::Streaming(count) => {
                // Poll the underlying stream until we get to `Poll::Pending`.
                loop {
                    match this.stream.as_mut().poll_next(cx) {
                        Poll::Ready(Some(value)) => {
                            if count < this.budget {
                                slot = Some(value);
                                *count += 1;
                            }
                        }
                        Poll::Ready(None) => {
                            *this.state = State::StreamDone;
                            break;
                        }
                        Poll::Pending => break,
                    }
                }

                // After the stream, always poll the interval timer.
                let _ = this
                    .interval
                    .as_mut()
                    .poll_next(cx)
                    .map(move |_| match this.state {
                        State::Streaming(count) => *count = 0, // reset the counter
                        State::StreamDone => cx.waker().wake_by_ref(),
                        State::AllDone => {}
                    });
                match slot {
                    Some(item) => Poll::Ready(Some(item)),
                    None => Poll::Pending,
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
            let throttle = Duration::from_millis(300);

            let take = 4;
            let expected = 2;

            let mut counter = 0;
            crate::stream::interval(interval)
                .take(take)
                .throttle(throttle)
                .for_each(|_| counter += 1)
                .await;

            assert_eq!(counter, expected);
        })
    }
}
