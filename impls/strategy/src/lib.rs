//! This crate contains all the Strategies for HOPRd.
//! Strategies are vital for (partial) automation of ticket and HOPR channel operations
//! during node runtime.
//!
//! - [passive strategy](crate::strategy::MultiStrategy)
//! - [auto funding strategy](crate::auto_funding) (feature `auto-funding`)
//! - [auto redeeming strategy](crate::auto_redeeming) (feature `auto-redeeming`)
//! - [closure finalizer](crate::channel_finalizer) (feature `closure-finalizer`)
//! - [multiple strategy chains](crate::strategy)
//!
//! Individual strategies are gated behind Cargo features.  Enable the `hopr` feature
//! to get the default HOPR network strategy set (currently `auto-redeeming`).
//!
//! ## Configuring strategies in HOPRd
//!
//! There are two ways of configuring strategies in HOPRd: via CLI and via a YAML config file.
//!
//! The configuration through CLI allows only fairly primitive single-strategy setting, through the `defaultStrategy`
//! parameter. It can be set to any of the above strategies, however, the strategy parameters are not further
//! configurable via the CLI and will always have their default values.
//! In addition, if the ` disableTicketAutoRedeem ` CLI argument is `false`, the default Auto Redeem strategy is added
//! to the strategy configured via the `defaultStrategy` argument (they execute together as Multi strategy).
//!
//! For more complex strategy configurations, the YAML configuration method is recommended via the `strategy` YAML
//! section. In this case, the top-most strategy is always assumed to be Multi strategy:
//!
//! ```yaml
//! strategy:
//!   on_fail_continue: true
//!   allow_recursive: true
//!   execution_interval: 60
//!   strategies:
//!     - !AutoFunding
//!       funding_amount: 20
//! ```

#[cfg(feature = "auto-redeeming")]
use std::str::FromStr;
use std::time::Duration;

#[cfg(feature = "auto-redeeming")]
use hopr_lib::HoprBalance;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, VariantNames};

#[cfg(feature = "auto-redeeming")]
use crate::Strategy::AutoRedeeming;
#[cfg(feature = "auto-funding")]
use crate::auto_funding::AutoFundingStrategyConfig;
#[cfg(feature = "auto-redeeming")]
use crate::auto_redeeming::AutoRedeemingStrategyConfig;
#[cfg(feature = "closure-finalizer")]
use crate::channel_finalizer::ClosureFinalizerStrategyConfig;
use crate::strategy::MultiStrategyConfig;

#[cfg(feature = "auto-funding")]
pub mod auto_funding;
#[cfg(feature = "auto-redeeming")]
pub mod auto_redeeming;
#[cfg(feature = "closure-finalizer")]
pub mod channel_finalizer;
pub mod errors;
pub mod strategy;

/// Lists all possible strategies with their respective configurations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, EnumString, VariantNames)]
#[strum(serialize_all = "snake_case")]
pub enum Strategy {
    #[cfg(feature = "auto-redeeming")]
    AutoRedeeming(AutoRedeemingStrategyConfig),
    #[cfg(feature = "auto-funding")]
    AutoFunding(AutoFundingStrategyConfig),
    #[cfg(feature = "closure-finalizer")]
    ClosureFinalizer(ClosureFinalizerStrategyConfig),
    Multi(MultiStrategyConfig),
    Passive,
}

/// Default HOPR node strategies (in order).
///
/// ## Auto-redeem Strategy
/// - redeem single tickets on channel close if worth at least 1 wxHOPR
pub fn hopr_default_strategies() -> MultiStrategyConfig {
    MultiStrategyConfig {
        on_fail_continue: true,
        allow_recursive: false,
        execution_interval: Duration::from_secs(60),
        strategies: vec![
            // AutoFunding(AutoFundingStrategyConfig {
            // min_stake_threshold: Balance::new_from_str("1000000000000000000", BalanceType::HOPR),
            // funding_amount: Balance::new_from_str("10000000000000000000", BalanceType::HOPR),
            // }),
            #[cfg(feature = "auto-redeeming")]
            AutoRedeeming(AutoRedeemingStrategyConfig {
                redeem_all_on_close: true,
                minimum_redeem_ticket_value: HoprBalance::from_str("1 wxHOPR").unwrap(),
                redeem_on_winning: true,
            }),
        ],
    }
}

impl Default for Strategy {
    fn default() -> Self {
        Self::Multi(hopr_default_strategies())
    }
}

/// An alias for the strategy configuration type.
pub type StrategyConfig = MultiStrategyConfig;
