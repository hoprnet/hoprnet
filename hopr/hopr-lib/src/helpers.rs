use std::time::Duration;

use hopr_api::{
    chain::ChainValues,
    types::primitive::prelude::{Address, HoprBalance, WxHOPR, XDai, XDaiBalance},
};

use crate::errors::HoprLibError;

/// Waits until the given address is funded.
///
/// This is done by querying the RPC provider for balance with backoff until `max_delay` argument.
pub(crate) async fn wait_for_funds<R: ChainValues>(
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

        hopr_utils::runtime::prelude::sleep(current_delay).await;
        current_delay = current_delay.mul_f64(multiplier);
    }

    Err(HoprLibError::InsufficientFunds(format!(
        "failed to fund the node within {} seconds",
        max_delay.as_secs()
    )))
}

/// Waits until the given safe holds at least `min_balance` of wxHOPR.
pub(crate) async fn wait_for_safe_hopr_balance<R: ChainValues>(
    min_balance: HoprBalance,
    max_delay: Duration,
    safe: Address,
    resolver: &R,
) -> Result<(), HoprLibError> {
    tracing::info!(
        %safe,
        required_wxhopr = %min_balance,
        "safe is underfunded with wxHOPR, waiting for it to be funded before announcing the node",
    );

    let multiplier = 1.05;
    let mut current_delay = Duration::from_secs(2).min(max_delay);

    while current_delay <= max_delay {
        match resolver.balance::<WxHOPR, _>(safe).await {
            Ok(current_balance) => {
                tracing::info!(%safe, balance = %current_balance, "safe wxHOPR balance status");
                if current_balance.ge(&min_balance) {
                    tracing::info!(%safe, "safe is funded with enough wxHOPR to announce the node");
                    return Ok(());
                } else {
                    tracing::warn!(%safe, "safe still underfunded with wxHOPR, trying again soon");
                }
            }
            Err(error) => tracing::error!(%safe, %error, "failed to fetch the safe wxHOPR balance from the chain"),
        }

        hopr_utils::runtime::prelude::sleep(current_delay).await;
        current_delay = current_delay.mul_f64(multiplier);
    }

    Err(HoprLibError::InsufficientFunds(format!(
        "the safe {safe} does not hold enough wxHOPR (needs at least {min_balance}) to announce the node on chain; \
         fund the safe with wxHOPR and restart the node"
    )))
}
