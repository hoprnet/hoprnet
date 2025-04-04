use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::stream::Stream;
use pin_project_lite::pin_project;

pin_project! {
    /// Delay execution of a stream once for the specified duration.
    ///
    /// This `struct` is created by the [`delay`] method on [`StreamExt`]. See its
    /// documentation for more.
    ///
    /// [`delay`]: crate::stream::StreamExt::delay
    /// [`StreamExt`]: crate::stream::StreamExt
    #[derive(Debug)]
    #[must_use = "streams do nothing unless polled or .awaited"]
    pub struct Delay<S, D> {
        #[pin]
        stream: S,
        #[pin]
        deadline: D,
        state: State,
    }
}

#[derive(Debug)]
enum State {
    Timer,
    Streaming,
}

impl<S, D> Delay<S, D> {
    pub(super) fn new(stream: S, deadline: D) -> Self {
        Delay {
            stream,
            deadline,
            state: State::Timer,
        }
    }
}

impl<S, D> Stream for Delay<S, D>
where
    S: Stream,
    D: Future,
{
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        match this.state {
            State::Timer => match this.deadline.poll(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(_) => {
                    *this.state = State::Streaming;
                    this.stream.poll_next(cx)
                }
            },
            State::Streaming => this.stream.poll_next(cx),
        }
    }
}
