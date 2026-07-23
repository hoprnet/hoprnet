use std::time::Duration;

use hopr_api::{
    chain::ChainValues,
    types::primitive::{
        balance::{Balance, Currency},
        prelude::Address,
    },
};

/// Polls the on-chain balance of `address` for currency `C` with exponential backoff,
/// returning `Ok` as soon as it reaches `min_balance`, or `Err` once the per-iteration
/// backoff grows past `max_delay`.
pub(crate) async fn wait_for_balance<C: Currency, R: ChainValues>(
    min_balance: Balance<C>,
    max_delay: Duration,
    address: Address,
    resolver: &R,
) -> Result<(), ()> {
    let multiplier = 1.05;
    let mut current_delay = Duration::from_secs(2).min(max_delay);

    while current_delay <= max_delay {
        match resolver.balance::<C, _>(address).await {
            Ok(current_balance) => {
                tracing::info!(%address, balance = %current_balance, "balance status");
                if current_balance.ge(&min_balance) {
                    return Ok(());
                } else {
                    tracing::warn!(%address, "still underfunded, trying again soon");
                }
            }
            Err(error) => tracing::error!(%address, %error, "failed to fetch balance from the chain"),
        }

        hopr_utils::runtime::prelude::sleep(current_delay).await;
        current_delay = current_delay.mul_f64(multiplier);
    }

    Err(())
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
            prelude::{Address, HoprBalance},
        },
    };

    use super::wait_for_balance;

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
            unimplemented!("not used by wait_for_balance")
        }

        async fn minimum_incoming_ticket_win_prob(&self) -> Result<WinningProbability, Self::Error> {
            unimplemented!("not used by wait_for_balance")
        }

        async fn minimum_ticket_price(&self) -> Result<HoprBalance, Self::Error> {
            unimplemented!("not used by wait_for_balance")
        }

        async fn key_binding_fee(&self) -> Result<HoprBalance, Self::Error> {
            unimplemented!("not used by wait_for_balance")
        }

        async fn channel_closure_notice_period(&self) -> Result<Duration, Self::Error> {
            unimplemented!("not used by wait_for_balance")
        }

        async fn chain_info(&self) -> Result<ChainInfo, Self::Error> {
            unimplemented!("not used by wait_for_balance")
        }

        async fn redemption_stats<A: Into<Address> + Send>(
            &self,
            _safe_addr: A,
        ) -> Result<RedemptionStats, Self::Error> {
            unimplemented!("not used by wait_for_balance")
        }

        async fn typical_resolution_time(&self) -> Result<Duration, Self::Error> {
            unimplemented!("not used by wait_for_balance")
        }
    }

    #[tokio::test(start_paused = true)]
    async fn returns_immediately_when_safe_is_already_funded() {
        let chain = MockChain {
            initial_base: 100,
            funded_base: 100,
            ..Default::default()
        };

        let result = wait_for_balance(
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

        let result = wait_for_balance(
            HoprBalance::new_base(50),
            Duration::from_secs(200),
            Address::default(),
            &chain,
        )
        .await;

        assert_eq!(result, Err(()), "expected the wait to time out, got {result:?}");
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

        let result = wait_for_balance(
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

        let result = wait_for_balance(
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
