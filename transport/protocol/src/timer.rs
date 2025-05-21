use std::time::Duration;

use futures::{
    future::{select, Either},
    pin_mut, FutureExt,
};
use hopr_async_runtime::prelude::sleep;
use hopr_platform::time::native::current_time;
use hopr_primitive_types::prelude::AsUnixTimestamp;
use tracing::{trace, warn};

/// Construct an infinitely running background loop producing ticks with a given period
/// with the maximum tick duration at most the period.
pub async fn execute_on_tick<F>(cycle: Duration, action: impl Fn() -> F, operation: String)
where
    F: std::future::Future<Output = ()> + Send,
{
    loop {
        let start = current_time().as_unix_timestamp();

        let timeout = sleep(cycle).fuse();
        let todo = (action)().fuse();

        pin_mut!(timeout, todo);

        match select(timeout, todo).await {
            Either::Left(_) => warn!(operation, "Timer tick interrupted by timeout"),
            Either::Right(_) => {
                trace!(operation, "Timer tick finished");

                let action_duration = current_time().as_unix_timestamp().saturating_sub(start);
                if let Some(remaining) = cycle.checked_sub(action_duration) {
                    trace!(
                        remaining_time_in_ms = remaining.as_millis(),
                        "Universal timer sleeping for",
                    );
                    sleep(remaining).await
                }
            }
        };
    }
}
