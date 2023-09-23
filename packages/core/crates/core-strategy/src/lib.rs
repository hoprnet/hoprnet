use std::time::Duration;
use crate::aggregating::AggregatingStrategyConfig;
use crate::auto_funding::AutoFundingStrategyConfig;
use crate::auto_redeeming::AutoRedeemingStrategyConfig;
use crate::promiscuous::PromiscuousStrategyConfig;
use crate::strategy::MultiStrategyConfig;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, EnumVariantNames};
use utils_types::primitives::{Balance, BalanceType};
use crate::Strategy::{Aggregating, AutoFunding};

pub mod strategy;

pub mod aggregating;
pub mod auto_funding;
pub mod auto_redeeming;
pub mod decision;
pub mod errors;
pub mod promiscuous;

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
///  - redeem the newly aggregated ticket right away
/// Auto-redeem Strategy
/// - disabled (because it would redeem single tickets)
/// Auto-funding Strategy
/// - funding amount: 10 HOPR
/// - lower limit: 1 HOPR
/// - the strategy will fund channels which fall below the lower limit with the funding amount
pub fn hopr_default_strategies() -> MultiStrategyConfig {
    MultiStrategyConfig {
        on_fail_continue: true,
        allow_recursive: false,
        strategies: vec![
            Aggregating(AggregatingStrategyConfig {
                aggregation_threshold: 100,
                aggregation_timeout: Duration::from_secs(60),
                redeem_after_aggregation: true,
            }),
            AutoFunding(AutoFundingStrategyConfig {
                min_stake_threshold: Balance::from_str("1000000000000000000", BalanceType::HOPR),
                funding_amount: Balance::from_str("10000000000000000000", BalanceType::HOPR)
            })
        ]
    }
}

impl Default for Strategy {
    fn default() -> Self {
        Self::Multi(hopr_default_strategies())
    }
}

pub type StrategyConfig = MultiStrategyConfig;

