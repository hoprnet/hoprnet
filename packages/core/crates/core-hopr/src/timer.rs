use std::time::Duration;
use futures::future::{Either, select};
use futures::pin_mut;
use futures::FutureExt;
use utils_log::{debug, warn};

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::sleep;
#[cfg(any(not(feature = "wasm"), test))]
use utils_misc::time::native::current_timestamp;

#[cfg(all(feature = "wasm", not(test)))]
use gloo_timers::future::sleep;
#[cfg(all(feature = "wasm", not(test)))]
use utils_misc::time::wasm::current_timestamp;

/// Represents a periodic timer that ticks in a loop with the given period.
/// Could be later extended so it supports multiple different periods and multiple actions.
#[derive(Debug, Clone)]
pub struct UniversalTimer {
    period: Duration
}

fn get_timestamp() -> Duration {
    Duration::from_millis(current_timestamp())
}

impl UniversalTimer {
    pub fn new(period: Duration) -> Self {
        Self { period }
    }

    pub async fn timer_loop<F>(&mut self, action: impl Fn() -> F)
    where F: std::future::Future<Output = ()> {
        loop {
            let start = get_timestamp();

            let timeout = sleep(self.period).fuse();
            let todo = (action)().fuse();

            pin_mut!(timeout, todo);

            match select(timeout, todo).await {
                Either::Left(_) => warn!("Timer tick interrupted by timeout"),
                Either::Right(_) => debug!("Timer tick finished"),
            };

            let action_duration = get_timestamp().saturating_sub(start);
            if action_duration < self.period {
                let remaining = self.period - action_duration;
                debug!("Universal timer sleeping for: {}ms", remaining.as_millis());
                sleep(remaining).await
            }
        }
    }
}