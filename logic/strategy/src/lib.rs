//! This crate contains all the Strategies for HOPRd.
//! Strategies are vital for (partial) automation of ticket and HOPR channel operations
//! during node runtime.
//!
//! - [passive strategy](crate::strategy::MultiStrategy)
//! - [promiscuous strategy](crate::promiscuous)
//! - [auto funding strategy](crate::auto_funding)
//! - [auto redeeming strategy](crate::auto_redeeming)
//! - [aggregating strategy](crate::aggregating)
//! - [multiple strategy chains](crate::strategy)
//!
//! HOPRd can be configured to use any of the above strategies.
//!
//! ## Configuring strategies in HOPRd
//!
//! There are two ways of configuring strategies in HOPRd: via CLI and via a YAML config file.
//!
//! The configuration through CLI allows only fairly primitive single-strategy setting, through the `defaultStrategy`
//! parameter. It can be set to any of the above strategies, however the strategy parameters are not further
//! configurable via the CLI and will always have their default values.
//! In addition, if `disableTicketAutoRedeem` CLI argument is `false`, the default Auto Redeem strategy is added to the
//! strategy configured via the `defaultStrategy` argument (they execute together as Multi strategy).
//!
//! For more complex strategy configurations, the YAML configuration method is recommended via the `strategy` YAML section.
//! In this case, the top-most strategy is always assumed to be Multi strategy:
//!
//! ```yaml
//! strategy:
//!   on_fail_continue: true
//!   allow_recursive: true
//!   strategies:
//!     - !Promiscuous
//!       max_channels: 50
//!       new_channel_stake: 20
//!     - !AutoFunding
//!       funding_amount: 20
//!     - !Aggregating:
//!       aggregation_threshold: 1000
//! ```

use hopr_primitive_types::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use strum::{Display, EnumString, EnumVariantNames};

use crate::aggregating::AggregatingStrategyConfig;
use crate::auto_funding::AutoFundingStrategyConfig;
use crate::auto_redeeming::AutoRedeemingStrategyConfig;
use crate::channel_finalizer::ClosureFinalizerStrategyConfig;
use crate::promiscuous::PromiscuousStrategyConfig;
use crate::strategy::MultiStrategyConfig;
use crate::Strategy::{Aggregating, AutoFunding, AutoRedeeming};

pub mod aggregating;
pub mod auto_funding;
pub mod auto_redeeming;
mod channel_finalizer;
pub mod errors;
pub mod promiscuous;
pub mod strategy;

/// Enumerates all possible strategies with their respective configurations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, EnumString, EnumVariantNames)]
#[strum(serialize_all = "snake_case")]
pub enum Strategy {
    Promiscuous(PromiscuousStrategyConfig),
    Aggregating(AggregatingStrategyConfig),
    AutoRedeeming(AutoRedeemingStrategyConfig),
    AutoFunding(AutoFundingStrategyConfig),
    ClosureFinalizer(ClosureFinalizerStrategyConfig),
    Multi(MultiStrategyConfig),
    Passive,
}

/// Default HOPR node strategies (in order).
///
/// ## Aggregation strategy
///  - aggregate every 100 tickets on all channels
///  - or when unredeemed value in the channel is more than 90% of channel's current balance
///  - aggregate unredeemed tickets when channel transitions to `PendingToClose`
/// ## Auto-redeem Strategy
/// - redeem only aggregated tickets
/// - redeem single tickets on channel close if worth at least 2 HOPR
/// ## Auto-funding Strategy
/// - funding amount: 10 HOPR
/// - lower limit: 1 HOPR
/// - the strategy will fund channels which fall below the lower limit with the funding amount
pub fn hopr_default_strategies() -> MultiStrategyConfig {
    MultiStrategyConfig {
        on_fail_continue: true,
        allow_recursive: false,
        strategies: vec![
            AutoFunding(AutoFundingStrategyConfig {
                min_stake_threshold: Balance::new_from_str("1000000000000000000", BalanceType::HOPR),
                funding_amount: Balance::new_from_str("10000000000000000000", BalanceType::HOPR),
            }),
            Aggregating(AggregatingStrategyConfig {
                aggregation_threshold: Some(100),
                unrealized_balance_ratio: Some(0.9),
                aggregation_timeout: Duration::from_secs(60),
                aggregate_on_channel_close: true,
            }),
            AutoRedeeming(AutoRedeemingStrategyConfig {
                redeem_only_aggregated: true,
                on_close_redeem_single_tickets_value_min: Balance::new_from_str(
                    "2000000000000000000",
                    BalanceType::HOPR,
                ),
            }),
        ],
    }
}

impl Default for Strategy {
    fn default() -> Self {
        Self::Multi(hopr_default_strategies())
    }
}

/// An alias for strategy configuration type.
pub type StrategyConfig = MultiStrategyConfig;
