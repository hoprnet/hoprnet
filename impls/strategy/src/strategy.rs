//! ## Multi Strategy
//!
//! Runs multiple sub-strategies concurrently. Each sub-strategy manages its own
//! event subscription and internal timers via the `Strategy::run` method.
//!
//! `MultiStrategy` is a pure combinator: it accepts any `Box<dyn Strategy + Send>` —
//! including strategies defined outside this crate — and runs them all concurrently.
//! The `on_fail_continue` flag controls whether a sub-strategy failure aborts the whole group:
//! - `true` → OR chain: continue after individual failures
//! - `false` → AND chain: abort all on first failure
use std::fmt::{Debug, Display, Formatter};

use async_trait::async_trait;
use tracing::warn;

use crate::errors::Result;

/// A strategy that runs until cancelled or a fatal error occurs.
///
/// Each implementation subscribes to the node's event stream and/or creates internal
/// timers in [`run`](Strategy::run). The trait is trivially object-safe: `run` takes only
/// `&mut self`, so strategies can be held as `Box<dyn Strategy + Send>`.
///
/// Any type implementing this trait can be composed into a [`MultiStrategy`] without
/// any changes to this crate.
#[async_trait]
pub trait Strategy: Display + Send {
    /// Run the strategy. Returns only on cancellation or fatal error.
    async fn run(&mut self) -> Result<()>;
}

/// Runs a group of sub-strategies concurrently, each in its own async task.
///
/// `MultiStrategy` is strategy-kind-agnostic: it only knows about
/// `Box<dyn Strategy + Send>`. Any type implementing [`Strategy`] — including
/// ones defined outside this crate — can be composed here.
pub struct MultiStrategy {
    strategies: Vec<Box<dyn Strategy + Send>>,
    on_fail_continue: bool,
}

impl MultiStrategy {
    /// Creates a new `MultiStrategy` from pre-built strategy objects.
    ///
    /// Strategies are passed in already constructed; `MultiStrategy` does not know or
    /// care about the concrete types. Pass an empty `strategies` vec to get a passive
    /// strategy that blocks forever.
    pub fn new(strategies: Vec<Box<dyn Strategy + Send>>, on_fail_continue: bool) -> Self {
        Self {
            strategies,
            on_fail_continue,
        }
    }
}

impl Debug for MultiStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MultiStrategy({} sub-strategies)", self.strategies.len())
    }
}

impl Display for MultiStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let names: Vec<String> = self.strategies.iter().map(|s| s.to_string()).collect();
        if names.is_empty() {
            write!(f, "multi_strategy(passive)")
        } else {
            write!(f, "multi_strategy({})", names.join(", "))
        }
    }
}

#[async_trait]
impl Strategy for MultiStrategy {
    async fn run(&mut self) -> Result<()> {
        use hopr_async_runtime::prelude::spawn;

        let strategies = std::mem::take(&mut self.strategies);

        if strategies.is_empty() {
            // Passive strategy: block forever until cancelled.
            futures::future::pending::<()>().await;
            return Ok(());
        }

        let on_fail_continue = self.on_fail_continue;
        let tasks: Vec<_> = strategies
            .into_iter()
            .map(|mut s| spawn(async move { s.run().await }))
            .collect();

        let results = futures::future::join_all(tasks).await;

        let mut last_error = None;
        for result in results {
            let task_result = result.map_err(|e| crate::errors::StrategyError::Other(e.into()))?;
            if let Err(e) = task_result {
                if !on_fail_continue {
                    return Err(e);
                }
                warn!(%e, "sub-strategy failed, continuing per on_fail_continue=true");
                last_error = Some(e);
            }
        }

        if let Some(e) = last_error {
            warn!(%e, "multi-strategy: some sub-strategies failed");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::{Display, Formatter};

    use super::*;
    use crate::errors::StrategyError;

    struct OkStrategy;
    impl Display for OkStrategy {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "ok")
        }
    }
    #[async_trait]
    impl Strategy for OkStrategy {
        async fn run(&mut self) -> Result<()> {
            Ok(())
        }
    }

    struct FailStrategy;
    impl Display for FailStrategy {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "fail")
        }
    }
    #[async_trait]
    impl Strategy for FailStrategy {
        async fn run(&mut self) -> Result<()> {
            Err(StrategyError::Other(anyhow::anyhow!("error")))
        }
    }

    /// An externally-defined strategy — simulates a plugin or application-defined strategy.
    struct ExternalStrategy {
        ran: bool,
    }
    impl Display for ExternalStrategy {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "external")
        }
    }
    #[async_trait]
    impl Strategy for ExternalStrategy {
        async fn run(&mut self) -> Result<()> {
            self.ran = true;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_multi_strategy_logical_or_flow() -> anyhow::Result<()> {
        let mut ms = MultiStrategy::new(vec![Box::new(FailStrategy), Box::new(OkStrategy)], true);
        // With on_fail_continue=true, even if FailStrategy errors, the multi-strategy succeeds.
        ms.run().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_multi_strategy_logical_and_flow() {
        let mut ms = MultiStrategy::new(vec![Box::new(FailStrategy), Box::new(OkStrategy)], false);
        ms.run().await.expect_err("multi-strategy should fail");
    }

    #[tokio::test]
    async fn test_multi_strategy_accepts_external_strategy() -> anyhow::Result<()> {
        // Demonstrates that any impl Strategy can be composed without modifying hopr-strategy.
        let mut ms = MultiStrategy::new(
            vec![Box::new(OkStrategy), Box::new(ExternalStrategy { ran: false })],
            true,
        );
        ms.run().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_multi_strategy_empty_is_passive() {
        // An empty MultiStrategy blocks forever — verify it does not complete immediately.
        let mut ms = MultiStrategy::new(vec![], true);
        let result =
            futures_time::future::FutureExt::timeout(ms.run(), futures_time::time::Duration::from_millis(50)).await;
        assert!(result.is_err(), "empty MultiStrategy should block (timeout expected)");
    }

    #[test]
    fn test_multi_strategy_display() {
        let ms = MultiStrategy::new(vec![Box::new(OkStrategy), Box::new(FailStrategy)], true);
        assert_eq!(ms.to_string(), "multi_strategy(ok, fail)");
    }

    #[test]
    fn test_multi_strategy_display_passive() {
        let ms = MultiStrategy::new(vec![], false);
        assert_eq!(ms.to_string(), "multi_strategy(passive)");
    }
}
