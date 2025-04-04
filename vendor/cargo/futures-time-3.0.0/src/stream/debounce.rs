use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::ready;
use futures_core::stream::Stream;
use pin_project_lite::pin_project;

use crate::future::Timer;

pin_project! {
    /// Debounce the stream.
    ///
    /// This `struct` is created by the [`debounce`] method on [`StreamExt`]. See its
    /// documentation for more.
    ///
    /// [`debounce`]: crate::stream::StreamExt::debounce
    /// [`StreamExt`]: crate::stream::StreamExt
    #[derive(Debug)]
    #[must_use = "streams do nothing unless polled or .awaited"]
    pub struct Debounce<S: Stream, D> {
        #[pin]
        stream: S,
        #[pin]
        deadline: D,
        slot: Option<S::Item>,
        state: State,
    }
}

/// Internal state.
#[derive(Debug)]
enum State {
    /// We're actively streaming and may have data.
    Streaming,
    /// The stream has ended, but we need to send the final `Ready(Some(Item))`
    /// and `Ready(None)` messages.
    FinalItem,
    /// The stream has ended, but we need to send the final `Ready(None)` message.
    SendingNone,
    /// The stream has completed.
    Finished,
}

impl<S: Stream, D> Debounce<S, D> {
    pub(crate) fn new(stream: S, deadline: D) -> Self {
        Self {
            stream,
            deadline,
            slot: None,
            state: State::Streaming,
        }
    }
}

impl<S, D> Stream for Debounce<S, D>
where
    S: Stream,
    D: Timer,
{
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        // See if we need to get more data from the stream.
        if let State::Streaming = this.state {
            match this.stream.poll_next(cx) {
                Poll::Ready(Some(item)) => {
                    *this.slot = Some(item);
                    this.deadline.as_mut().reset_timer();
                }
                Poll::Ready(None) => match *this.slot {
                    Some(_) => *this.state = State::FinalItem,
                    None => *this.state = State::SendingNone,
                },
                _ => {}
            };
        }

        // Handle the timer.
        match this.state {
            State::Streaming => match this.slot.is_some() {
                true => {
                    ready!(this.deadline.as_mut().poll(cx));
                    Poll::Ready(this.slot.take())
                }
                false => Poll::Pending,
            },

            State::FinalItem => {
                let _ = futures_core::ready!(this.deadline.as_mut().poll(cx));
                *this.state = State::SendingNone;
                cx.waker().wake_by_ref();
                Poll::Ready(this.slot.take())
            }

            State::SendingNone => {
                *this.state = State::Finished;
                Poll::Ready(None)
            }
            State::Finished => panic!("stream polled after completion"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use crate::time::Duration;
    use futures_lite::prelude::*;

    #[test]
    fn all_values_debounce() {
        async_io::block_on(async {
            let interval = Duration::from_millis(10);
            let debounce = Duration::from_millis(20);

            let mut counter = 0;
            crate::stream::interval(interval)
                .take(10)
                .debounce(debounce)
                .for_each(|_| counter += 1)
                .await;

            assert_eq!(counter, 1);
        })
    }

    #[test]
    fn no_debounces_hit() {
        async_io::block_on(async {
            let interval = Duration::from_millis(40);
            let debounce = Duration::from_millis(10);

            let mut counter = 0;
            crate::stream::interval(interval)
                .take(10)
                .debounce(debounce)
                .for_each(|_| counter += 1)
                .await;

            assert_eq!(counter, 10);
        })
    }
}
