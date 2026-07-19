//! ## Multi Strategy
//!
//! Runs multiple sub-strategies concurrently. Each sub-strategy manages its own
//! event subscription and internal timers via the `Strategy::run` method.
//!
//! `MultiStrategy` is a pure combinator: it accepts any `Box<dyn Strategy + Send>` —
//! including strategies defined outside this crate — and runs them all concurrently.
//! Sub-strategies are fully isolated: a failure in one is logged and does not affect
//! the others.
use std::fmt::{Debug, Display, Formatter};

use async_trait::async_trait;

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
}

impl MultiStrategy {
    /// Creates a new `MultiStrategy` from pre-built strategy objects.
    ///
    /// Strategies are passed in already constructed; `MultiStrategy` does not know or
    /// care about the concrete types. Pass an empty `strategies` vec to get a passive
    /// strategy that blocks forever.
    pub fn new(strategies: Vec<Box<dyn Strategy + Send>>) -> Self {
        Self { strategies }
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
        let strategies = std::mem::take(&mut self.strategies);

        if strategies.is_empty() {
            // Passive strategy: block forever until cancelled.
            futures::future::pending::<()>().await;
            return Ok(());
        }

        #[cfg(not(feature = "runtime-tokio"))]
        {
            let _ = strategies;
            return Err(crate::errors::StrategyError::Other(anyhow::anyhow!(
                "MultiStrategy with sub-strategies requires the `runtime-tokio` feature"
            )));
        }

        #[cfg(feature = "runtime-tokio")]
        {
            use futures::StreamExt as _;
            use hopr_utils::runtime::prelude::{AbortHandle, abortable, spawn};

            // Spawn each sub-strategy as an abortable task.
            // Keeping all AbortHandles in a RAII guard ensures every sub-task is cancelled
            // when MultiStrategy is dropped (graceful shutdown).
            let mut join_handles = Vec::new();
            let mut abort_handles: Vec<AbortHandle> = Vec::new();
            for mut s in strategies {
                let proc = hopr_utils::runtime::diagnostics::instrument(
                    async move { s.run().await },
                    "multi_strategy_sub_task",
                    module_path!(),
                    file!(),
                    line!(),
                );
                let (proc, abort_handle) = abortable(proc);
                join_handles.push(spawn(proc));
                abort_handles.push(abort_handle);
            }

            struct AbortGuard(Vec<AbortHandle>);
            impl Drop for AbortGuard {
                fn drop(&mut self) {
                    for h in &self.0 {
                        h.abort();
                    }
                }
            }
            let _guard = AbortGuard(abort_handles);

            // Process completions as they arrive. Sub-strategies are fully isolated:
            // a failure in one is logged but does not affect the others.
            let mut pending: futures::stream::FuturesUnordered<_> = join_handles.into_iter().collect();

            while let Some(join_result) = pending.next().await {
                let strategy_result = match join_result {
                    Err(e) => Err(crate::errors::StrategyError::Other(e.into())),
                    Ok(Ok(result)) => result,
                    Ok(Err(_aborted)) => continue, // aborted by the guard — expected during shutdown
                };

                if let Err(e) = strategy_result {
                    tracing::warn!(%e, "sub-strategy failed");
                }
            }

            Ok(())
        }
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
    async fn test_multi_strategy_sub_failure_does_not_propagate() -> anyhow::Result<()> {
        // A failing sub-strategy is isolated: the MultiStrategy still returns Ok.
        let mut ms = MultiStrategy::new(vec![Box::new(FailStrategy), Box::new(OkStrategy)]);
        ms.run().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_multi_strategy_accepts_external_strategy() -> anyhow::Result<()> {
        // Demonstrates that any impl Strategy can be composed without modifying hopr-strategy.
        let mut ms = MultiStrategy::new(vec![Box::new(OkStrategy), Box::new(ExternalStrategy { ran: false })]);
        ms.run().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_multi_strategy_empty_is_passive() {
        // An empty MultiStrategy blocks forever — verify it does not complete immediately.
        let mut ms = MultiStrategy::new(vec![]);
        let result =
            futures_time::future::FutureExt::timeout(ms.run(), futures_time::time::Duration::from_millis(50)).await;
        assert!(result.is_err(), "empty MultiStrategy should block (timeout expected)");
    }

    #[test]
    fn test_multi_strategy_display() {
        let ms = MultiStrategy::new(vec![Box::new(OkStrategy), Box::new(FailStrategy)]);
        assert_eq!(ms.to_string(), "multi_strategy(ok, fail)");
    }

    #[test]
    fn test_multi_strategy_display_passive() {
        let ms = MultiStrategy::new(vec![]);
        assert_eq!(ms.to_string(), "multi_strategy(passive)");
    }
}
