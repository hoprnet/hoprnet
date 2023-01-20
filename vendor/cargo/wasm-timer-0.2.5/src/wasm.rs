// Copyright 2019 Pierre Krieger
// Copyright (c) 2019 Tokio Contributors
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

#![cfg(all(target_arch = "wasm32", target_os = "unknown"))]

use std::cmp::{Eq, PartialEq, Ord, PartialOrd, Ordering};
use std::ops::{Add, Sub, AddAssign, SubAssign};
use std::time::Duration;

#[derive(Debug, Copy, Clone)]
pub struct Instant {
    /// Unit is milliseconds.
    inner: f64,
}

impl PartialEq for Instant {
    fn eq(&self, other: &Instant) -> bool {
        // Note that this will most likely only compare equal if we clone an `Instant`,
        // but that's ok.
        self.inner == other.inner
    }
}

impl Eq for Instant {}

impl PartialOrd for Instant {
    fn partial_cmp(&self, other: &Instant) -> Option<Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl Ord for Instant {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.partial_cmp(&other.inner).unwrap()
    }
}

impl Instant {
    pub fn now() -> Instant {
        let val = web_sys::window()
            .expect("not in a browser")
            .performance()
            .expect("performance object not available")
            .now();
        Instant { inner: val }
    }

    pub fn duration_since(&self, earlier: Instant) -> Duration {
        *self - earlier
    }

    pub fn elapsed(&self) -> Duration {
        Instant::now() - *self
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, other: Duration) -> Instant {
        let new_val = self.inner + other.as_millis() as f64;
        Instant { inner: new_val as f64 }
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;

    fn sub(self, other: Duration) -> Instant {
        let new_val = self.inner - other.as_millis() as f64;
        Instant { inner: new_val as f64 }
    }
}

impl Sub<Instant> for Instant {
    type Output = Duration;

    fn sub(self, other: Instant) -> Duration {
        let ms = self.inner - other.inner;
        assert!(ms >= 0.0);
        Duration::from_millis(ms as u64)
    }
}

pub const UNIX_EPOCH: SystemTime = SystemTime { inner: 0.0 };

#[derive(Debug, Copy, Clone)]
pub struct SystemTime {
    /// Unit is milliseconds.
    inner: f64,
}

impl PartialEq for SystemTime {
    fn eq(&self, other: &SystemTime) -> bool {
        // Note that this will most likely only compare equal if we clone an `SystemTime`,
        // but that's ok.
        self.inner == other.inner
    }
}

impl Eq for SystemTime {}

impl PartialOrd for SystemTime {
    fn partial_cmp(&self, other: &SystemTime) -> Option<Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl Ord for SystemTime {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.partial_cmp(&other.inner).unwrap()
    }
}

impl SystemTime {
    pub const UNIX_EPOCH: SystemTime = SystemTime { inner: 0.0 };

    pub fn now() -> SystemTime {
        let val = js_sys::Date::now();
        SystemTime { inner: val }
    }

    pub fn duration_since(&self, earlier: SystemTime) -> Result<Duration, ()> {
        let dur_ms = self.inner - earlier.inner;
        if dur_ms < 0.0 {
            return Err(())
        }
        Ok(Duration::from_millis(dur_ms as u64))
    }

    pub fn elapsed(&self) -> Result<Duration, ()> {
        self.duration_since(SystemTime::now())
    }

    pub fn checked_add(&self, duration: Duration) -> Option<SystemTime> {
        Some(*self + duration)
    }

    pub fn checked_sub(&self, duration: Duration) -> Option<SystemTime> {
        Some(*self - duration)
    }
}

impl Add<Duration> for SystemTime {
    type Output = SystemTime;

    fn add(self, other: Duration) -> SystemTime {
        let new_val = self.inner + other.as_millis() as f64;
        SystemTime { inner: new_val as f64 }
    }
}

impl Sub<Duration> for SystemTime {
    type Output = SystemTime;

    fn sub(self, other: Duration) -> SystemTime {
        let new_val = self.inner - other.as_millis() as f64;
        SystemTime { inner: new_val as f64 }
    }
}

impl AddAssign<Duration> for SystemTime {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl SubAssign<Duration> for SystemTime {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}
