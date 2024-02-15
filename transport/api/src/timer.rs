use futures::future::{select, Either};
use futures::pin_mut;
use futures::FutureExt;
use tracing::{trace, warn};
use std::time::Duration;

use async_std::task::sleep;

use hopr_platform::time::native::current_time;
use hopr_primitive_types::prelude::AsUnixTimestamp;

/// Construct an infinitely running background loop producing ticks with a given period
/// with the maximum tick duration at most the period.
pub async fn execute_on_tick<F>(cycle: Duration, action: impl Fn() -> F)
where
    F: std::future::Future<Output = ()> + Send,
{
    loop {
        let start = current_time().as_unix_timestamp();

        let timeout = sleep(cycle).fuse();
        let todo = (action)().fuse();

        pin_mut!(timeout, todo);

        match select(timeout, todo).await {
            Either::Left(_) => warn!("Timer tick interrupted by timeout"),
            Either::Right(_) => {
                trace!("Timer tick finished");

                let action_duration = current_time().as_unix_timestamp().saturating_sub(start);
                if let Some(remaining) = cycle.checked_sub(action_duration) {
                    trace!("Universal timer sleeping for: {}ms", remaining.as_millis());
                    sleep(remaining).await
                }
            }
        };
    }
}
