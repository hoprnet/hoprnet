//! HOPR-specific strategy wiring.
//!
//! This module owns:
//! - [`hopr_default_strategies`] — the default HOPRd strategy configuration
//! - [`build_strategies`] — maps a [`MultiStrategyConfig`] to a runnable [`MultiStrategy`]
//!
//! Separating dispatch here (rather than inside `hopr-strategy`) keeps the library crate
//! free of application-specific concerns and allows external strategies to be composed
//! alongside the built-in ones.

use std::{sync::Arc, time::Duration};

use hopr_lib::{
    HoprBalance,
    api::{
        chain::{
            ChainReadChannelOperations, ChainReadSafeOperations, ChainValues, ChainWriteChannelOperations,
            ChainWriteTicketOperations,
        },
        node::{ActionableEventSource, HasChainApi, HasTicketManagement},
        tickets::TicketManagement,
    },
};
use hopr_strategy::{
    StrategyKind,
    strategy::{MultiStrategy, MultiStrategyConfig, Strategy},
};
#[cfg(all(feature = "telemetry", not(test)))]
use strum::VariantNames;
use tracing::error;

#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_ENABLED_STRATEGIES: hopr_metrics::MultiGauge =
        hopr_metrics::MultiGauge::new(
            "hopr_strategy_enabled_strategies",
            "List of enabled strategies",
            &["strategy"],
        )
        .unwrap();
}

/// Default HOPRd strategy configuration.
///
/// ## Strategies included
/// - `AutoRedeeming`: redeems single tickets on channel close if worth at least 1 wxHOPR
pub fn hopr_default_strategies() -> MultiStrategyConfig {
    use std::str::FromStr;

    use hopr_strategy::auto_redeeming::AutoRedeemingStrategyConfig;

    MultiStrategyConfig {
        on_fail_continue: true,
        allow_recursive: false,
        execution_interval: Duration::from_secs(60),
        strategies: vec![StrategyKind::AutoRedeeming(AutoRedeemingStrategyConfig {
            redeem_all_on_close: true,
            minimum_redeem_ticket_value: HoprBalance::from_str("1 wxHOPR").unwrap(),
            redeem_on_winning: true,
        })],
    }
}

/// Builds a [`MultiStrategy`] from a [`MultiStrategyConfig`] and a node reference.
///
/// Maps each [`StrategyKind`] variant to its concrete builder, wires in the node,
/// and returns a single `Box<dyn Strategy + Send>` that runs all sub-strategies
/// concurrently.
///
/// External strategies can be composed by building this result first, then wrapping
/// it with additional strategies in a new `MultiStrategy::new(...)` call at the
/// call site.
pub fn build_strategies<N>(cfg: &MultiStrategyConfig, node: Arc<N>) -> Box<dyn Strategy + Send>
where
    N: ActionableEventSource
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
        + 'static,
{
    let mut strategies = Vec::<Box<dyn Strategy + Send>>::new();

    #[cfg(all(feature = "telemetry", not(test)))]
    StrategyKind::VARIANTS
        .iter()
        .for_each(|s| METRIC_ENABLED_STRATEGIES.set(&[*s], 0_f64));

    for strategy in cfg.strategies.iter() {
        match strategy {
            StrategyKind::AutoRedeeming(sub_cfg) => strategies.push(
                hopr_strategy::auto_redeeming::AutoRedeemingStrategy::new(*sub_cfg, cfg.execution_interval)
                    .build(Arc::clone(&node)),
            ),
            StrategyKind::AutoFunding(sub_cfg) => strategies.push(
                hopr_strategy::auto_funding::AutoFundingStrategy::new(*sub_cfg, cfg.execution_interval)
                    .build(Arc::clone(&node)),
            ),
            StrategyKind::ClosureFinalizer(sub_cfg) => strategies.push(
                hopr_strategy::channel_finalizer::ClosureFinalizerStrategy::new(*sub_cfg, cfg.execution_interval)
                    .build(Arc::clone(&node)),
            ),
            StrategyKind::Multi(sub_cfg) => {
                if cfg.allow_recursive {
                    let mut sub = sub_cfg.clone();
                    sub.allow_recursive = false;
                    strategies.push(build_strategies(&sub, Arc::clone(&node)));
                } else {
                    error!("recursive multi-strategy not allowed and skipped");
                }
            }
            StrategyKind::Passive => {} // passive = empty sub-strategy list
        }

        #[cfg(all(feature = "telemetry", not(test)))]
        METRIC_ENABLED_STRATEGIES.set(&[&strategy.to_string()], 1_f64);
    }

    Box::new(MultiStrategy::new(strategies, cfg.on_fail_continue))
}
