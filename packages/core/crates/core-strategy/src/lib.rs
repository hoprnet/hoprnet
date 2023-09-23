use crate::aggregating::AggregatingStrategyConfig;
use crate::auto_funding::AutoFundingStrategyConfig;
use crate::auto_redeeming::AutoRedeemingStrategyConfig;
use crate::promiscuous::PromiscuousStrategyConfig;
use crate::strategy::MultiStrategyConfig;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, EnumVariantNames};

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

impl Default for Strategy {
    fn default() -> Self {
        Self::Multi(Default::default())
    }
}

pub type StrategyConfig = MultiStrategyConfig;
