use std::mem;
use std::pin::Pin;

use pin_project_lite::pin_project;

use core::task::{Context, Poll};
use futures_core::stream::Stream;

pin_project! {
    /// Buffer items and flushes them at each interval.
    ///
    /// This `struct` is created by the [`buffer`] method on [`StreamExt`]. See its
    /// documentation for more.
    ///
    /// [`buffer`]: crate::stream::StreamExt::buffer
    /// [`StreamExt`]: crate::stream::StreamExt
    #[derive(Debug)]
    #[must_use = "streams do nothing unless polled or .awaited"]
    pub struct Buffer<S: Stream, I> {
        #[pin]
        stream: S,
        #[pin]
        interval: I,
        slot: Vec<S::Item>,
        state: State,
    }
}

impl<S: Stream, I> Buffer<S, I> {
    pub(crate) fn new(stream: S, interval: I) -> Self {
        Self {
            stream,
            interval,
            slot: vec![],
            state: State::Streaming,
        }
    }
}

#[derive(Debug)]
enum State {
    /// The underlying stream is yielding items.
    Streaming,
    /// The underlying stream is done yielding items.
    StreamDone,
    /// All timers have completed and all data has been yielded.
    TimerDone,
    /// The closing `Ready(None)` has been yielded.
    AllDone,
}

impl<S: Stream, I: Stream> Stream for Buffer<S, I> {
    type Item = Vec<S::Item>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        match this.state {
            // The underlying stream is yielding items.
            State::Streaming => {
                // Poll the underlying stream until we get to `Poll::Pending`.
                loop {
                    match this.stream.as_mut().poll_next(cx) {
                        Poll::Ready(Some(value)) => this.slot.push(value),
                        Poll::Ready(None) => {
                            *this.state = State::StreamDone;
                            break;
                        }
                        Poll::Pending => break,
                    }
                }

                // After the stream, always poll the interval timer.
                this.interval.as_mut().poll_next(cx).map(move |_| {
                    if let State::StreamDone = this.state {
                        *this.state = State::TimerDone;
                        cx.waker().wake_by_ref();
                    }
                    Some(mem::take(&mut *this.slot))
                })
            }

            // The underlying stream is done yielding items.
            State::StreamDone => this.interval.as_mut().poll_next(cx).map(|_| {
                cx.waker().wake_by_ref();
                *this.state = State::TimerDone;
                Some(mem::take(&mut *this.slot))
            }),

            // All timers have completed and all data has been yielded.
            State::TimerDone => {
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
    fn buffer_all_values() {
        async_io::block_on(async {
            let interval = Duration::from_millis(5);
            let buffer = Duration::from_millis(20);

            let mut counter = 0;
            crate::stream::interval(interval)
                .take(10)
                .buffer(buffer)
                .for_each(|buf| counter += buf.len())
                .await;

            assert_eq!(counter, 10);
        })
    }

    #[test]
    fn no_debounces_hit() {
        async_io::block_on(async {
            let interval = Duration::from_millis(20);
            let buffer = Duration::from_millis(10);

            let mut counter = 0;
            crate::stream::interval(interval)
                .take(10)
                .buffer(buffer)
                .for_each(|buf| counter += buf.len())
                .await;

            assert_eq!(counter, 10);
        })
    }
}
