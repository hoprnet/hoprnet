use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::channel::Parker;

use futures_core::{ready, Stream};
use pin_project_lite::pin_project;

pin_project! {
    /// Suspend or resume execution of a future.
    ///
    /// This `struct` is created by the [`park`] method on [`FutureExt`]. See its
    /// documentation for more.
    ///
    /// [`park`]: crate::future::FutureExt::park
    /// [`FutureExt`]: crate::future::FutureExt
    #[must_use = "futures do nothing unless polled or .awaited"]
    pub struct Park<F, I>
    where
        F: Future,
        I: Stream<Item = Parker>
    {
        #[pin]
        future: F,
        #[pin]
        interval: I,
        state: State,
    }
}

/// The internal state
#[derive(Debug)]
enum State {
    /// Actively polling the future.
    Active,
    /// The future has been paused, so we wait for a signal from the channel.
    Suspended,
    /// The channel has been dropped, no more need to check it!
    NoChannel,
    /// The future has completed.
    Completed,
}

impl<F, I> Park<F, I>
where
    F: Future,
    I: Stream<Item = Parker>,
{
    pub(super) fn new(future: F, interval: I) -> Self {
        Self {
            future,
            interval,
            state: State::Suspended,
        }
    }
}

impl<F, I> Future for Park<F, I>
where
    F: Future,
    I: Stream<Item = Parker>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        loop {
            match this.state {
                State::Suspended => match ready!(this.interval.as_mut().poll_next(cx)) {
                    Some(Parker::Park) => return Poll::Pending,
                    Some(Parker::Unpark) => *this.state = State::Active,
                    None => *this.state = State::NoChannel,
                },
                State::Active => {
                    if let Poll::Ready(Some(Parker::Park)) = this.interval.as_mut().poll_next(cx) {
                        *this.state = State::Suspended;
                        return Poll::Pending;
                    }
                    let value = ready!(this.future.as_mut().poll(cx));
                    *this.state = State::Completed;
                    return Poll::Ready(value);
                }
                State::NoChannel => {
                    let value = ready!(this.future.as_mut().poll(cx));
                    *this.state = State::Completed;
                    return Poll::Ready(value);
                }
                State::Completed => panic!("future polled after completing"),
            }
        }
    }
}

// NOTE(yosh): we should probably test this, but I'm too tired today lol.
