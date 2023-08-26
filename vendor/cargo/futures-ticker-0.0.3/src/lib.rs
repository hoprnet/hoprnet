//! futures-ticker - Asynchronous, recurring delivery of a timer event.

use futures::{stream::Stream, task::Context};
use futures_timer::Delay;
use std::{future::Future, pin::Pin, task::Poll, time::Duration};

#[cfg(target_family = "wasm")]
use instant::Instant;
#[cfg(not(target_family = "wasm"))]
use std::time::Instant;

/// Yields the current time in regular intervals.
///
/// Tickers are an asynchronous notification mechanism which deliver
/// the "current" time in regular intervals (a "tick"). In case any
/// ticks were missed, they will be skipped, and only the nearest
/// upcoming tick is delivered.
#[derive(Debug)]
pub struct Ticker {
    interval: Duration,
    next: Instant,
    schedule: Delay,
}

impl Ticker {
    /// Constructs a ticker that goes off once per `interval`. It
    /// is scheduled to deliver the first tick at `interval` from now.
    pub fn new(interval: Duration) -> Ticker {
        Ticker::new_with_next(interval, interval)
    }

    /// Constructs a ticker that goes off once per `interval`. The first
    /// tick is scheduled to arrive after `first_in` elapses.
    pub fn new_with_next(interval: Duration, first_in: Duration) -> Ticker {
        let first = Instant::now() + first_in;
        Ticker {
            interval,
            next: first,
            schedule: Delay::new(first_in),
        }
    }

    /// Returns the next Instant at which the Ticker will be ready.
    pub fn next_tick(&self) -> Instant {
        self.next_tick_from(Instant::now())
    }

    /// Answers the hypothetical question, "at Instant `now`, when
    /// would the next tick go off?"
    ///
    /// This function is useful mainly for tests. Use
    /// [`Ticker::next_tick`] for real use cases instead.
    pub fn next_tick_from(&self, now: Instant) -> Instant {
        if self.next > now {
            return self.next;
        }
        let raw_next = self.next + self.interval;
        if raw_next > now {
            return raw_next;
        }
        if self.interval.as_nanos() == 0 {
            // Silly special case: If somebody specifies "now", the
            // ticker is always ready to return a result.
            return now;
        }
        // If the "next" tick would be in the past, let's schedule it
        // to go off in the future at a multiple of the interval,
        // instead:
        let missed_times = 1 + ((now - raw_next).as_nanos() / self.interval.as_nanos()) as u32;
        self.next + self.interval * (missed_times + 1)
    }
}

impl Stream for Ticker {
    type Item = Instant;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let schedule = Pin::new(&mut self.schedule);
        match schedule.poll(cx) {
            Poll::Pending => Poll::Pending,

            Poll::Ready(_) => {
                let now = Instant::now();
                let next = self.next_tick_from(now);
                self.next = next;
                self.schedule.reset(
                    next.checked_duration_since(now)
                        .unwrap_or_else(|| Duration::from_nanos(0)),
                );
                Poll::Ready(Some(now))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (1, None)
    }
}
