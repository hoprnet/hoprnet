//! ## Multi Strategy
//!
//! Runs multiple sub-strategies concurrently. Each sub-strategy manages its own
//! event subscription and internal timers via the `Strategy::run` method.
//!
//! `MultiStrategy` spawns one async task per sub-strategy. The `on_fail_continue`
//! flag controls whether a sub-strategy failure aborts the whole group:
//! - `true` → OR chain: continue after individual failures
//! - `false` → AND chain: abort all on first failure
//!
//! For details on default parameters see [`MultiStrategyConfig`].
use std::fmt::{Debug, Display, Formatter};

use async_trait::async_trait;
use hopr_lib::api::{
    chain::{
        ChainReadChannelOperations, ChainReadSafeOperations, ChainValues, ChainWriteChannelOperations,
        ChainWriteTicketOperations,
    },
    node::{ActionableEventSource, HasChainApi, HasTicketManagement},
    tickets::TicketManagement,
};
use serde::{Deserialize, Serialize};
#[cfg(all(feature = "telemetry", not(test)))]
use strum::VariantNames;
use tracing::{error, warn};
use validator::{Validate, ValidationError};

#[cfg(feature = "auto-funding")]
use crate::auto_funding::AutoFundingStrategy;
#[cfg(feature = "auto-redeeming")]
use crate::auto_redeeming::AutoRedeemingStrategy;
#[cfg(feature = "closure-finalizer")]
use crate::channel_finalizer::ClosureFinalizerStrategy;
use crate::{Strategy as StrategyEnum, errors::Result};

#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_ENABLED_STRATEGIES: hopr_metrics::MultiGauge =
        hopr_metrics::MultiGauge::new("hopr_strategy_enabled_strategies", "List of enabled strategies", &["strategy"]).unwrap();
}

/// A strategy that runs until cancelled or a fatal error occurs.
///
/// Each implementation subscribes to the node's event stream and/or creates internal
/// timers in [`run`](Strategy::run). The trait is trivially object-safe: `run` takes only
/// `&mut self`, so strategies can be held as `Box<dyn Strategy + Send>`.
#[async_trait]
pub trait Strategy: Display + Send {
    /// Run the strategy. Returns only on cancellation or fatal error.
    async fn run(&mut self) -> Result<()>;
}

/// Combined node interface used exclusively by [`MultiStrategy::build`].
///
/// Individual strategy `build` functions each declare only the subset of traits
/// they actually call. `HoprNode` is the union of all those subsets, so a single
/// concrete HOPR node that satisfies `HoprNode` can be passed to `MultiStrategy`
/// without any per-strategy narrowing at the call site.
///
/// Blanket-implemented for every type that satisfies all constituent sub-traits.
///
/// Test wrappers that implement only a subset of traits still work — they bypass
/// `MultiStrategy::build` and construct the private `*Inner<N>` runner types
/// directly, using only the bounds those types actually require.
pub trait HoprNode:
    ActionableEventSource
    + HasChainApi<
        ChainApi: ChainReadChannelOperations
                      + ChainReadSafeOperations
                      + ChainValues
                      + ChainWriteChannelOperations
                      + ChainWriteTicketOperations
                      + Clone
                      + Send
                      + Sync
                      + 'static,
    > + HasTicketManagement<TicketManager: TicketManagement + Clone + Send + Sync + 'static>
    + Send
    + Sync
    + 'static
{
}

impl<T> HoprNode for T
where
    T: ActionableEventSource + HasChainApi + HasTicketManagement + Send + Sync + 'static,
    T::ChainApi: ChainReadChannelOperations
        + ChainReadSafeOperations
        + ChainValues
        + ChainWriteChannelOperations
        + ChainWriteTicketOperations
        + Clone
        + Send
        + Sync
        + 'static,
    T::TicketManager: TicketManagement + Clone + Send + Sync + 'static,
{
}

#[inline]
fn just_true() -> bool {
    true
}

#[inline]
fn sixty_seconds() -> std::time::Duration {
    std::time::Duration::from_secs(60)
}

#[inline]
fn empty_vector() -> Vec<StrategyEnum> {
    vec![]
}

fn validate_execution_interval(interval: &std::time::Duration) -> std::result::Result<(), ValidationError> {
    if interval < &std::time::Duration::from_secs(10) {
        Err(ValidationError::new(
            "strategy execution interval must be at least 1 second",
        ))
    } else {
        Ok(())
    }
}

/// Configuration options for the `MultiStrategy` group.
#[derive(Debug, Clone, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MultiStrategyConfig {
    /// If `false`, the first sub-strategy failure stops the entire group.
    /// If `true`, failures are logged and execution continues.
    ///
    /// Default is `true`.
    #[default = true]
    #[serde(default = "just_true")]
    pub on_fail_continue: bool,

    /// Indicate whether the `MultiStrategy` can contain another `MultiStrategy`.
    ///
    /// Default is `true`.
    #[default = true]
    #[serde(default = "just_true")]
    pub allow_recursive: bool,

    /// Execution interval for periodic scans within each sub-strategy.
    ///
    /// Default is 60 seconds, minimum is 10 seconds.
    #[default(sixty_seconds())]
    #[serde(default = "sixty_seconds", with = "humantime_serde")]
    #[validate(custom(function = "validate_execution_interval"))]
    pub execution_interval: std::time::Duration,

    /// Configuration of individual sub-strategies.
    ///
    /// Default is empty, which makes the `MultiStrategy` behave as passive.
    #[default(_code = "vec![]")]
    #[serde(default = "empty_vector")]
    pub strategies: Vec<StrategyEnum>,
}

/// Runs a group of sub-strategies concurrently, each in its own async task.
pub struct MultiStrategy {
    strategies: Vec<Box<dyn Strategy + Send>>,
    cfg: MultiStrategyConfig,
}

impl MultiStrategy {
    /// Builds a `MultiStrategy` from config and a node reference.
    ///
    /// Each sub-strategy receives a clone of `node` and starts its own event
    /// subscription inside `run()`. The generic `N` is erased at construction time.
    pub fn build<N: HoprNode>(cfg: MultiStrategyConfig, node: std::sync::Arc<N>) -> Box<dyn Strategy + Send> {
        let mut strategies = Vec::<Box<dyn Strategy + Send>>::new();

        #[cfg(all(feature = "telemetry", not(test)))]
        StrategyEnum::VARIANTS
            .iter()
            .for_each(|s| METRIC_ENABLED_STRATEGIES.set(&[*s], 0_f64));

        for strategy in cfg.strategies.iter() {
            match strategy {
                #[cfg(feature = "auto-redeeming")]
                StrategyEnum::AutoRedeeming(sub_cfg) => strategies.push(
                    AutoRedeemingStrategy::new(*sub_cfg, cfg.execution_interval).build(std::sync::Arc::clone(&node)),
                ),
                #[cfg(feature = "auto-funding")]
                StrategyEnum::AutoFunding(sub_cfg) => strategies.push(
                    AutoFundingStrategy::new(*sub_cfg, cfg.execution_interval).build(std::sync::Arc::clone(&node)),
                ),
                #[cfg(feature = "closure-finalizer")]
                StrategyEnum::ClosureFinalizer(sub_cfg) => strategies.push(
                    ClosureFinalizerStrategy::new(*sub_cfg, cfg.execution_interval).build(std::sync::Arc::clone(&node)),
                ),
                StrategyEnum::Multi(sub_cfg) => {
                    if cfg.allow_recursive {
                        let mut cfg_clone = sub_cfg.clone();
                        cfg_clone.allow_recursive = false;
                        strategies.push(Self::build(cfg_clone, std::sync::Arc::clone(&node)))
                    } else {
                        error!("recursive multi-strategy not allowed and skipped")
                    }
                }
                StrategyEnum::Passive => {} // passive = empty sub-strategy list
            }

            #[cfg(all(feature = "telemetry", not(test)))]
            METRIC_ENABLED_STRATEGIES.set(&[&strategy.to_string()], 1_f64);
        }

        Box::new(Self { strategies, cfg })
    }
}

impl Debug for MultiStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", StrategyEnum::Multi(self.cfg.clone()))
    }
}

impl Display for MultiStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", StrategyEnum::Multi(self.cfg.clone()))
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

        let on_fail_continue = self.cfg.on_fail_continue;
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

    #[tokio::test]
    async fn test_multi_strategy_logical_or_flow() -> anyhow::Result<()> {
        let cfg = MultiStrategyConfig {
            on_fail_continue: true,
            allow_recursive: true,
            execution_interval: std::time::Duration::from_secs(60),
            strategies: Vec::new(),
        };

        let mut ms = MultiStrategy {
            strategies: vec![Box::new(FailStrategy), Box::new(OkStrategy)],
            cfg,
        };
        // With on_fail_continue=true, even if FailStrategy errors, the multi-strategy succeeds.
        ms.run().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_multi_strategy_logical_and_flow() {
        let cfg = MultiStrategyConfig {
            on_fail_continue: false,
            allow_recursive: true,
            execution_interval: std::time::Duration::from_secs(60),
            strategies: Vec::new(),
        };

        let mut ms = MultiStrategy {
            strategies: vec![Box::new(FailStrategy), Box::new(OkStrategy)],
            cfg,
        };
        ms.run().await.expect_err("multi-strategy should fail");
    }
}
