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

#[cfg(test)]
mod tests {
    use std::{
        sync::atomic::{AtomicUsize, Ordering},
        time::Duration,
    };

    use hopr_api::{
        chain::{ChainInfo, ChainValues, DomainSeparators, RedemptionStats, WinningProbability},
        types::primitive::{
            balance::{Balance, Currency},
            prelude::Address,
        },
    };

    use super::{HoprBalance, wait_for_safe_hopr_balance};
    use crate::errors::HoprLibError;

    #[derive(Debug, thiserror::Error)]
    #[error("mock chain error")]
    struct MockError;

    /// Mock [`ChainValues`] resolver that scripts a sequence of `balance()` outcomes.
    ///
    /// The first `fail_first` calls return an error (simulating a flaky RPC), after which
    /// the reported balance is `initial_base` until `bump_after` successful reads have
    /// happened, at which point it jumps to `funded_base` (simulating the safe being topped up).
    #[derive(Default)]
    struct MockChain {
        calls: AtomicUsize,
        fail_first: usize,
        bump_after: usize,
        initial_base: u64,
        funded_base: u64,
    }

    impl MockChain {
        /// Number of times `balance()` was invoked.
        fn call_count(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    #[async_trait::async_trait]
    impl ChainValues for MockChain {
        type Error = MockError;

        async fn balance<C: Currency, A: Into<Address> + Send>(&self, _address: A) -> Result<Balance<C>, Self::Error> {
            let n = self.calls.fetch_add(1, Ordering::SeqCst);
            if n < self.fail_first {
                return Err(MockError);
            }
            let base = if n >= self.bump_after {
                self.funded_base
            } else {
                self.initial_base
            };
            Ok(Balance::<C>::new_base(base))
        }

        async fn domain_separators(&self) -> Result<DomainSeparators, Self::Error> {
            unimplemented!("not used by wait_for_safe_hopr_balance")
        }

        async fn minimum_incoming_ticket_win_prob(&self) -> Result<WinningProbability, Self::Error> {
            unimplemented!("not used by wait_for_safe_hopr_balance")
        }

        async fn minimum_ticket_price(&self) -> Result<HoprBalance, Self::Error> {
            unimplemented!("not used by wait_for_safe_hopr_balance")
        }

        async fn key_binding_fee(&self) -> Result<HoprBalance, Self::Error> {
            unimplemented!("not used by wait_for_safe_hopr_balance")
        }

        async fn channel_closure_notice_period(&self) -> Result<Duration, Self::Error> {
            unimplemented!("not used by wait_for_safe_hopr_balance")
        }

        async fn chain_info(&self) -> Result<ChainInfo, Self::Error> {
            unimplemented!("not used by wait_for_safe_hopr_balance")
        }

        async fn redemption_stats<A: Into<Address> + Send>(
            &self,
            _safe_addr: A,
        ) -> Result<RedemptionStats, Self::Error> {
            unimplemented!("not used by wait_for_safe_hopr_balance")
        }

        async fn typical_resolution_time(&self) -> Result<Duration, Self::Error> {
            unimplemented!("not used by wait_for_safe_hopr_balance")
        }
    }

    #[tokio::test(start_paused = true)]
    async fn returns_immediately_when_safe_is_already_funded() {
        let chain = MockChain {
            initial_base: 100,
            funded_base: 100,
            ..Default::default()
        };

        let result = wait_for_safe_hopr_balance(
            HoprBalance::new_base(50),
            Duration::from_secs(200),
            Address::default(),
            &chain,
        )
        .await;

        assert!(result.is_ok());
        // A single read is enough since the safe already holds the required balance.
        assert_eq!(chain.call_count(), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn errors_when_safe_stays_underfunded_until_deadline() {
        let chain = MockChain {
            initial_base: 10,
            funded_base: 10,
            ..Default::default()
        };

        let result = wait_for_safe_hopr_balance(
            HoprBalance::new_base(50),
            Duration::from_secs(200),
            Address::default(),
            &chain,
        )
        .await;

        assert!(
            matches!(result, Err(HoprLibError::InsufficientFunds(_))),
            "expected InsufficientFunds, got {result:?}"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn succeeds_after_the_safe_gets_topped_up() {
        // Underfunded for the first two reads, then funded.
        let chain = MockChain {
            initial_base: 10,
            funded_base: 100,
            bump_after: 2,
            ..Default::default()
        };

        let result = wait_for_safe_hopr_balance(
            HoprBalance::new_base(50),
            Duration::from_secs(200),
            Address::default(),
            &chain,
        )
        .await;

        assert!(result.is_ok());
        // Two underfunded reads plus the successful third one.
        assert_eq!(chain.call_count(), chain.bump_after + 1);
    }

    #[tokio::test(start_paused = true)]
    async fn tolerates_transient_fetch_errors_then_succeeds() {
        // First read errors out, the safe is funded from the second read onwards.
        let chain = MockChain {
            initial_base: 100,
            funded_base: 100,
            fail_first: 1,
            ..Default::default()
        };

        let result = wait_for_safe_hopr_balance(
            HoprBalance::new_base(50),
            Duration::from_secs(200),
            Address::default(),
            &chain,
        )
        .await;

        assert!(result.is_ok());
        // One failed read followed by a successful one.
        assert_eq!(chain.call_count(), chain.fail_first + 1);
    }
}
