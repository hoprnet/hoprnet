use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use async_io::Timer;
use pin_project_lite::pin_project;

use crate::time::Instant;

/// Sleeps until the specified instant.
pub fn sleep_until(deadline: Instant) -> SleepUntil {
    SleepUntil {
        timer: Timer::at(deadline.into()),
        completed: false,
    }
}

pin_project! {
    /// Sleeps until the specified instant.
    #[must_use = "futures do nothing unless polled or .awaited"]
    pub struct SleepUntil {
        #[pin]
        timer: Timer,
        completed: bool,
    }
}

impl Future for SleepUntil {
    type Output = Instant;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        assert!(!self.completed, "future polled after completing");
        let this = self.project();
        match this.timer.poll(cx) {
            Poll::Ready(instant) => {
                *this.completed = true;
                Poll::Ready(instant.into())
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
