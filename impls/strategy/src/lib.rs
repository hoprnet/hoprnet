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
//! Individual strategies are gated behind Cargo features.
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

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, VariantNames};

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
///
/// This is a pure serde config type — it is used for YAML deserialization and
/// carries no runtime behaviour. The runtime combinator is [`strategy::MultiStrategy`],
/// which accepts any `Box<dyn strategy::Strategy + Send>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, EnumString, VariantNames)]
#[strum(serialize_all = "snake_case")]
pub enum StrategyKind {
    #[cfg(feature = "auto-redeeming")]
    AutoRedeeming(AutoRedeemingStrategyConfig),
    #[cfg(feature = "auto-funding")]
    AutoFunding(AutoFundingStrategyConfig),
    #[cfg(feature = "closure-finalizer")]
    ClosureFinalizer(ClosureFinalizerStrategyConfig),
    Multi(MultiStrategyConfig),
    Passive,
}

/// An alias for the strategy configuration type.
pub type StrategyConfig = MultiStrategyConfig;

#[cfg(test)]
mod tests {
    use super::*;

    /// Ensures that every StrategyKind variant serializes and deserializes correctly.
    #[test]
    fn test_strategy_kind_serde_roundtrip() {
        let variants: Vec<StrategyKind> = vec![
            #[cfg(feature = "auto-redeeming")]
            StrategyKind::AutoRedeeming(Default::default()),
            #[cfg(feature = "auto-funding")]
            StrategyKind::AutoFunding(Default::default()),
            #[cfg(feature = "closure-finalizer")]
            StrategyKind::ClosureFinalizer(Default::default()),
            StrategyKind::Multi(Default::default()),
            StrategyKind::Passive,
        ];

        for variant in variants {
            let serialized = serde_json::to_string(&variant).expect("serialization failed");
            let deserialized: StrategyKind = serde_json::from_str(&serialized).expect("deserialization failed");
            assert_eq!(variant, deserialized, "roundtrip failed for {variant}");
        }
    }
}
