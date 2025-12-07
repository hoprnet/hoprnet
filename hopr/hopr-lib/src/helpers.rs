use std::time::Duration;

use hopr_api::chain::ChainReadAccountOperations;
use hopr_primitive_types::prelude::{Address, XDai, XDaiBalance};

use crate::errors::HoprLibError;

/// Waits until the given address is funded.
///
/// This is done by querying the RPC provider for balance with backoff until `max_delay` argument.
pub async fn wait_for_funds<R: ChainReadAccountOperations>(
    min_balance: XDaiBalance,
    suggested_balance: XDaiBalance,
    max_delay: Duration,
    account: Address,
    resolver: &R,
) -> Result<(), HoprLibError> {
    tracing::info!(
        suggested_minimum_balance = %suggested_balance,
        "node about to start, checking for funds",
    );

    let multiplier = 1.05;
    let mut current_delay = Duration::from_secs(2).min(max_delay);

    while current_delay <= max_delay {
        match resolver.balance::<XDai, _>(account).await {
            Ok(current_balance) => {
                tracing::info!(balance = %current_balance, "balance status");
                if current_balance.ge(&min_balance) {
                    tracing::info!("node is funded");
                    return Ok(());
                } else {
                    tracing::warn!("still unfunded, trying again soon");
                }
            }
            Err(error) => tracing::error!(%error, "failed to fetch balance from the chain"),
        }

        hopr_async_runtime::prelude::sleep(current_delay).await;
        current_delay = current_delay.mul_f64(multiplier);
    }

    Err(HoprLibError::GeneralError(format!(
        "failed to fund the node within {} seconds",
        max_delay.as_secs()
    )))
}
