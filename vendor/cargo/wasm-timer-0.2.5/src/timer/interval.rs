use pin_utils::unsafe_pinned;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::prelude::*;

use crate::timer::delay;
use crate::{Delay, Instant, TimerHandle};

/// A stream representing notifications at fixed interval
///
/// Intervals are created through the `Interval::new` or
/// `Interval::new_at` methods indicating when a first notification
/// should be triggered and when it will be repeated.
///
/// Note that intervals are not intended for high resolution timers, but rather
/// they will likely fire some granularity after the exact instant that they're
/// otherwise indicated to fire at.
#[derive(Debug)]
pub struct Interval {
    delay: Delay,
    interval: Duration,
}

impl Interval {
    unsafe_pinned!(delay: Delay);

    /// Creates a new interval which will fire at `dur` time into the future,
    /// and will repeat every `dur` interval after
    ///
    /// The returned object will be bound to the default timer for this thread.
    /// The default timer will be spun up in a helper thread on first use.
    pub fn new(dur: Duration) -> Interval {
        Interval::new_at(Instant::now() + dur, dur)
    }

    /// Creates a new interval which will fire at the time specified by `at`,
    /// and then will repeat every `dur` interval after
    ///
    /// The returned object will be bound to the default timer for this thread.
    /// The default timer will be spun up in a helper thread on first use.
    pub fn new_at(at: Instant, dur: Duration) -> Interval {
        Interval {
            delay: Delay::new_at(at),
            interval: dur,
        }
    }

    /// Creates a new interval which will fire at the time specified by `at`,
    /// and then will repeat every `dur` interval after
    ///
    /// The returned object will be bound to the timer specified by `handle`.
    pub fn new_handle(at: Instant, dur: Duration, handle: TimerHandle) -> Interval {
        Interval {
            delay: Delay::new_handle(at, handle),
            interval: dur,
        }
    }
}

impl Stream for Interval {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if Pin::new(&mut *self).delay().poll(cx).is_pending() {
            return Poll::Pending;
        }
        let next = next_interval(delay::fires_at(&self.delay), Instant::now(), self.interval);
        self.delay.reset_at(next);
        Poll::Ready(Some(()))
    }
}

/// Converts Duration object to raw nanoseconds if possible
///
/// This is useful to divide intervals.
///
/// While technically for large duration it's impossible to represent any
/// duration as nanoseconds, the largest duration we can represent is about
/// 427_000 years. Large enough for any interval we would use or calculate in
/// tokio.
fn duration_to_nanos(dur: Duration) -> Option<u64> {
    dur.as_secs()
        .checked_mul(1_000_000_000)
        .and_then(|v| v.checked_add(dur.subsec_nanos() as u64))
}

fn next_interval(prev: Instant, now: Instant, interval: Duration) -> Instant {
    let new = prev + interval;
    if new > now {
        return new;
    } else {
        let spent_ns =
            duration_to_nanos(now.duration_since(prev)).expect("interval should be expired");
        let interval_ns =
            duration_to_nanos(interval).expect("interval is less that 427 thousand years");
        let mult = spent_ns / interval_ns + 1;
        assert!(
            mult < (1 << 32),
            "can't skip more than 4 billion intervals of {:?} \
             (trying to skip {})",
            interval,
            mult
        );
        return prev + interval * (mult as u32);
    }
}

#[cfg(test)]
mod test {
    use super::next_interval;
    use std::time::{Duration, Instant};

    struct Timeline(Instant);

    impl Timeline {
        fn new() -> Timeline {
            Timeline(Instant::now())
        }
        fn at(&self, millis: u64) -> Instant {
            self.0 + Duration::from_millis(millis)
        }
        fn at_ns(&self, sec: u64, nanos: u32) -> Instant {
            self.0 + Duration::new(sec, nanos)
        }
    }

    fn dur(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    // The math around Instant/Duration isn't 100% precise due to rounding
    // errors, see #249 for more info
    fn almost_eq(a: Instant, b: Instant) -> bool {
        if a == b {
            true
        } else if a > b {
            a - b < Duration::from_millis(1)
        } else {
            b - a < Duration::from_millis(1)
        }
    }

    #[test]
    fn norm_next() {
        let tm = Timeline::new();
        assert!(almost_eq(
            next_interval(tm.at(1), tm.at(2), dur(10)),
            tm.at(11)
        ));
        assert!(almost_eq(
            next_interval(tm.at(7777), tm.at(7788), dur(100)),
            tm.at(7877)
        ));
        assert!(almost_eq(
            next_interval(tm.at(1), tm.at(1000), dur(2100)),
            tm.at(2101)
        ));
    }

    #[test]
    fn fast_forward() {
        let tm = Timeline::new();
        assert!(almost_eq(
            next_interval(tm.at(1), tm.at(1000), dur(10)),
            tm.at(1001)
        ));
        assert!(almost_eq(
            next_interval(tm.at(7777), tm.at(8888), dur(100)),
            tm.at(8977)
        ));
        assert!(almost_eq(
            next_interval(tm.at(1), tm.at(10000), dur(2100)),
            tm.at(10501)
        ));
    }

    /// TODO: this test actually should be successful, but since we can't
    ///       multiply Duration on anything larger than u32 easily we decided
    ///       to allow it to fail for now
    #[test]
    #[should_panic(expected = "can't skip more than 4 billion intervals")]
    fn large_skip() {
        let tm = Timeline::new();
        assert_eq!(
            next_interval(tm.at_ns(0, 1), tm.at_ns(25, 0), Duration::new(0, 2)),
            tm.at_ns(25, 1)
        );
    }
}
