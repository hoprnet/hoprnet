use crate::{future::IntoFuture, task::SleepUntil};

use std::ops::{Add, AddAssign, Sub, SubAssign};

use super::Duration;

/// A measurement of a monotonically nondecreasing clock. Opaque and useful only
/// with Duration.
///
/// This type wraps `std::time::Duration` so we can implement traits on it
/// without coherence issues, just like if we were implementing this in the
/// stdlib.
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Copy)]
pub struct Instant(pub(crate) std::time::Instant);

impl Instant {
    /// Returns an instant corresponding to "now".
    ///
    /// # Examples
    ///
    /// ```
    /// use futures_time::time::Instant;
    ///
    /// let now = Instant::now();
    /// ```
    #[must_use]
    pub fn now() -> Self {
        std::time::Instant::now().into()
    }
}

impl Add<Duration> for Instant {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        (self.0 + rhs.0).into()
    }
}

impl AddAssign<Duration> for Instant {
    fn add_assign(&mut self, rhs: Duration) {
        *self = (self.0 + rhs.0).into()
    }
}

impl Sub<Duration> for Instant {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        (self.0 - rhs.0).into()
    }
}

impl SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = (self.0 - rhs.0).into()
    }
}

impl std::ops::Deref for Instant {
    type Target = std::time::Instant;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Instant {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<std::time::Instant> for Instant {
    fn from(inner: std::time::Instant) -> Self {
        Self(inner)
    }
}

impl Into<std::time::Instant> for Instant {
    fn into(self) -> std::time::Instant {
        self.0
    }
}

impl IntoFuture for Instant {
    type Output = Instant;

    type IntoFuture = SleepUntil;

    fn into_future(self) -> Self::IntoFuture {
        crate::task::sleep_until(self)
    }
}
