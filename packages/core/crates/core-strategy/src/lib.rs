use serde::{Deserialize, Serialize};
use std::time::Duration;
use strum::{Display, EnumString, EnumVariantNames};
use utils_types::primitives::{Balance, BalanceType};

use crate::aggregating::AggregatingStrategyConfig;
use crate::auto_funding::AutoFundingStrategyConfig;
use crate::auto_redeeming::AutoRedeemingStrategyConfig;
use crate::promiscuous::PromiscuousStrategyConfig;
use crate::strategy::MultiStrategyConfig;
use crate::Strategy::{Aggregating, AutoFunding, AutoRedeeming};

pub mod strategy;

pub mod aggregating;
pub mod auto_funding;
pub mod auto_redeeming;
pub mod decision;
pub mod errors;
pub mod promiscuous;

/// Enumerates all possible strategies with their respective configurations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, EnumString, EnumVariantNames)]
pub enum Strategy {
    #[strum(serialize = "promiscuous")]
    Promiscuous(PromiscuousStrategyConfig),

    #[strum(serialize = "aggregating")]
    Aggregating(AggregatingStrategyConfig),

    #[strum(serialize = "auto_redeeming")]
    AutoRedeeming(AutoRedeemingStrategyConfig),

    #[strum(serialize = "auto_funding")]
    AutoFunding(AutoFundingStrategyConfig),

    #[strum(serialize = "multi")]
    Multi(MultiStrategyConfig),

    #[strum(serialize = "passive")]
    Passive,
}

/// Default HOPR node strategies:
///
/// Aggregation strategy:
///  - aggregate every 100 tickets on all channels
///  - or when unredeemed value in the channel is more than 90% of channel's current balance
///  - aggregate unredeemed tickets when channel transitions to `PendingToClose`
/// Auto-redeem Strategy
/// - redeem only aggregated tickets
/// Auto-funding Strategy
/// - funding amount: 10 HOPR
/// - lower limit: 1 HOPR
/// - the strategy will fund channels which fall below the lower limit with the funding amount
pub fn hopr_default_strategies() -> MultiStrategyConfig {
    MultiStrategyConfig {
        on_fail_continue: true,
        allow_recursive: false,
        finalize_channel_closure: false,
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
            }),
        ],
    }
}

impl Default for Strategy {
    fn default() -> Self {
        Self::Multi(hopr_default_strategies())
    }
}

pub type StrategyConfig = MultiStrategyConfig;
