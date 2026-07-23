use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use hopr_api::{
    chain::ChainValues,
    types::primitive::prelude::{Address, XDai, XDaiBalance},
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

#[derive(Clone)]
pub struct BroadcastSenderSink<T>(pub async_broadcast::Sender<T>);

impl<T: Clone> futures::Sink<T> for BroadcastSenderSink<T> {
    type Error = async_broadcast::TrySendError<T>;

    fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        self.0.try_broadcast(item).map(|_| ())
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.0.close();
        Poll::Ready(Ok(()))
    }
}
