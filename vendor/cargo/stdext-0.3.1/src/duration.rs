//! Extension traits for `std::time::Duration`.

use std::time::Duration;

const SECS_IN_MIN: u64 = 60;
const SECS_IN_HOUR: u64 = 3600;
const SECS_IN_DAY: u64 = 3600 * 24;

/// Extension trait with useful methods for [`std::time::Duration`].
///
/// [`std::time::Duration`]: https://doc.rust-lang.org/std/time/struct.Duration.html
pub trait DurationExt {
    /// Creates a new `Duration` from the specified number of minutes.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use stdext::prelude::*;
    ///
    /// let duration = Duration::from_minutes(1);
    ///
    /// assert_eq!(duration, Duration::from_secs(60));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the total amount of seconds exceeds the `u64` type range.
    fn from_minutes(minutes: u64) -> Duration;

    /// Creates a new `Duration` from the specified number of hours.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use stdext::prelude::*;
    ///
    /// let duration = Duration::from_hours(1);
    ///
    /// assert_eq!(duration, Duration::from_secs(3600));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the total amount of seconds exceeds the `u64` type range.
    fn from_hours(hours: u64) -> Duration;

    /// Creates a new `Duration` from the specified number of hours.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use stdext::prelude::*;
    ///
    /// let duration = Duration::from_days(1);
    ///
    /// assert_eq!(duration, Duration::from_secs(3600 * 24));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the total amount of seconds exceeds the `u64` type range.
    fn from_days(days: u64) -> Duration;

    /// Adds the specified amount of nanoseconds to the `Duration` object.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use stdext::prelude::*;
    ///
    /// let duration = Duration::default().add_nanos(1);
    ///
    /// assert_eq!(duration, Duration::default() + Duration::from_nanos(1));
    /// ```
    fn add_nanos(self, nanos: u64) -> Duration;

    /// Adds the specified amount of microseconds to the `Duration` object.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use stdext::prelude::*;
    ///
    /// let duration = Duration::default().add_micros(1);
    ///
    /// assert_eq!(duration, Duration::default() + Duration::from_micros(1));
    /// ```
    fn add_micros(self, micros: u64) -> Duration;

    /// Adds the specified amount of milliseconds to the `Duration` object.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use stdext::prelude::*;
    ///
    /// let duration = Duration::default().add_millis(1);
    ///
    /// assert_eq!(duration, Duration::default() + Duration::from_millis(1));
    /// ```
    fn add_millis(self, millis: u64) -> Duration;

    /// Adds the specified amount of seconds to the `Duration` object.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use stdext::prelude::*;
    ///
    /// let duration = Duration::default().add_secs(1);
    ///
    /// assert_eq!(duration, Duration::default() + Duration::from_secs(1));
    /// ```
    fn add_secs(self, seconds: u64) -> Duration;

    /// Adds the specified amount of minutes to the `Duration` object.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use stdext::prelude::*;
    ///
    /// let duration = Duration::default().add_minutes(1);
    ///
    /// assert_eq!(duration, Duration::default() + Duration::from_minutes(1));
    /// ```
    fn add_minutes(self, minutes: u64) -> Duration;

    /// Adds the specified amount of hours to the `Duration` object.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use stdext::prelude::*;
    ///
    /// let duration = Duration::default().add_hours(1);
    ///
    /// assert_eq!(duration, Duration::default() + Duration::from_hours(1));
    /// ```
    fn add_hours(self, hours: u64) -> Duration;

    /// Adds the specified amount of days to the `Duration` object.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use stdext::prelude::*;
    ///
    /// let duration = Duration::default().add_days(1);
    ///
    /// assert_eq!(duration, Duration::default() + Duration::from_days(1));
    /// ```
    fn add_days(self, days: u64) -> Duration;

    /// Returns the number of _whole_ minutes contained by this `Duration`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use stdext::prelude::*;
    ///
    /// let duration = Duration::new(61, 730023852);
    /// assert_eq!(duration.as_minutes(), 1);
    /// ```
    fn as_minutes(&self) -> u64;

    /// Returns the number of _whole_ hours contained by this `Duration`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use stdext::prelude::*;
    ///
    /// let duration = Duration::new(3601, 730023852);
    /// assert_eq!(duration.as_hours(), 1);
    /// ```
    fn as_hours(&self) -> u64;

    /// Returns the number of _whole_ hours contained by this `Duration`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use stdext::prelude::*;
    ///
    /// let duration = Duration::new(61, 730023852).add_days(1);
    /// assert_eq!(duration.as_days(), 1);
    /// ```
    fn as_days(&self) -> u64;
}

impl DurationExt for Duration {
    fn from_minutes(minutes: u64) -> Self {
        let seconds = minutes * SECS_IN_MIN;
        Self::from_secs(seconds)
    }

    fn from_hours(hours: u64) -> Self {
        let seconds: u64 = hours * SECS_IN_HOUR;
        Self::from_secs(seconds)
    }

    fn from_days(days: u64) -> Self {
        let seconds = days * SECS_IN_DAY;
        Self::from_secs(seconds)
    }

    fn add_nanos(self, nanos: u64) -> Self {
        self + Self::from_nanos(nanos)
    }

    fn add_micros(self, micros: u64) -> Self {
        self + Self::from_micros(micros)
    }

    fn add_millis(self, millis: u64) -> Self {
        self + Self::from_millis(millis)
    }

    fn add_secs(self, seconds: u64) -> Self {
        self + Self::from_secs(seconds)
    }

    fn add_minutes(self, minutes: u64) -> Self {
        self + Self::from_minutes(minutes)
    }

    fn add_hours(self, hours: u64) -> Self {
        self + Self::from_hours(hours)
    }

    fn add_days(self, days: u64) -> Self {
        self + Self::from_days(days)
    }

    fn as_minutes(&self) -> u64 {
        self.as_secs() / SECS_IN_MIN
    }

    fn as_hours(&self) -> u64 {
        self.as_secs() / SECS_IN_HOUR
    }

    fn as_days(&self) -> u64 {
        self.as_secs() / SECS_IN_DAY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_methods() {
        // Check `from_minutes`
        let test_vector = vec![0, 1, u64::max_value() / SECS_IN_MIN];
        for minutes in test_vector {
            let seconds = minutes * SECS_IN_MIN;
            assert_eq!(
                Duration::from_minutes(minutes),
                Duration::from_secs(seconds)
            );
        }

        // Check `from_hours`
        let test_vector = vec![0, 1, u64::max_value() / SECS_IN_HOUR];
        for hours in test_vector {
            let seconds = hours * SECS_IN_HOUR;
            assert_eq!(Duration::from_hours(hours), Duration::from_secs(seconds));
        }

        // Check `from_days`
        let test_vector = vec![0, 1, u64::max_value() / SECS_IN_DAY];
        for days in test_vector {
            let seconds = days * SECS_IN_DAY;
            assert_eq!(Duration::from_days(days), Duration::from_secs(seconds));
        }
    }

    #[test]
    fn add_methods() {
        let duration = Duration::default()
            .add_nanos(1)
            .add_micros(1)
            .add_millis(1)
            .add_secs(1)
            .add_minutes(1)
            .add_hours(1)
            .add_days(1);

        let expected_duration = Duration::new(
            SECS_IN_DAY + SECS_IN_HOUR + SECS_IN_MIN + 1,
            1_000_000 + 1_000 + 1,
        );

        assert_eq!(duration, expected_duration);
    }

    #[test]
    fn as_methods() {
        let test_vector = vec![0, SECS_IN_MIN, SECS_IN_HOUR, SECS_IN_DAY];

        for seconds in test_vector {
            for seconds in &[seconds, seconds + 1, seconds * 2, seconds * 2 + 1] {
                let duration = Duration::from_secs(*seconds);

                assert_eq!(duration.as_minutes(), duration.as_secs() / SECS_IN_MIN);
                assert_eq!(duration.as_hours(), duration.as_secs() / SECS_IN_HOUR);
                assert_eq!(duration.as_days(), duration.as_secs() / SECS_IN_DAY);
            }
        }
    }
}
