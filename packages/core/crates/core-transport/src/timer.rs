use futures::future::{select, Either};
use futures::pin_mut;
use futures::FutureExt;
use std::time::Duration;
use utils_log::{debug, warn};

use async_std::task::sleep;
use platform::time::native::current_timestamp;



fn get_timestamp() -> Duration {
    current_timestamp()
}

/// Represents a periodically timed ticks in a loop with the given period.
/// Could be later extended so it supports multiple different periods and multiple actions.
pub async fn execute_on_tick<F>(cycle: Duration , action: impl Fn() -> F)
where
    F: std::future::Future<Output = ()> + Send,
{
    loop {
        let start = get_timestamp();

        let timeout = sleep(cycle).fuse();
        let todo = (action)().fuse();

        pin_mut!(timeout, todo);

        match select(timeout, todo).await {
            Either::Left(_) => warn!("Timer tick interrupted by timeout"),
            Either::Right(_) => debug!("Timer tick finished"),
        };

        let action_duration = get_timestamp().saturating_sub(start);
        if let Some(remaining) = action_duration.checked_sub(cycle) {
            debug!("Universal timer sleeping for: {}ms", remaining.as_millis());
            sleep(remaining).await
        }
    }
}