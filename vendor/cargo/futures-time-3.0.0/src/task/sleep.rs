use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use async_io::Timer as AsyncTimer;
use pin_project_lite::pin_project;

use crate::future::Timer;
use crate::time::{Duration, Instant};

/// Sleeps for the specified amount of time.
///
/// This future can be `push_deadline` to be moved
pub fn sleep(dur: Duration) -> Sleep {
    Sleep {
        dur,
        timer: AsyncTimer::after(dur.into()),
        completed: false,
    }
}

pin_project! {
    /// Sleeps for the specified amount of time.
    #[must_use = "futures do nothing unless polled or .awaited"]
    pub struct Sleep {
        #[pin]
        timer: AsyncTimer,
        completed: bool,
        dur: Duration,
    }
}

impl Future for Sleep {
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

impl Timer for Sleep {
    /// Resets the timer to be `Instant::now()` + `Duration` into the future.
    fn reset_timer(self: std::pin::Pin<&mut Self>) {
        let mut this = self.project();
        this.timer.set_after(**this.dur);
        *this.completed = false;
    }
}
