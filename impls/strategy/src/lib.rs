//! This crate contains all the Strategies for HOPRd.
//! Strategies are vital for (partial) automation of ticket and HOPR channel operations
//! during node runtime.
//!
//! - [passive strategy](crate::strategy::MultiStrategy)
//! - auto funding strategy (`auto_funding` module, feature `strategy-auto-funding`)
//! - auto redeeming strategy (`auto_redeeming` module, feature `strategy-auto-redeeming`)
//! - closure finalizer (`channel_finalizer` module, feature `strategy-closure-finalizer`)
//! - channel lifecycle strategy (feature `strategy-channel-lifecycle`)
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
//!   allow_recursive: true
//!   execution_interval: 60
//!   strategies:
//!     - !AutoFunding
//!       funding_amount: 20
//! ```

/// Shared serde default helpers used across multiple strategy configs.
#[cfg(feature = "strategy-auto-redeeming")]
pub(crate) fn just_true() -> bool {
    true
}
#[cfg(feature = "strategy-auto-redeeming")]
pub(crate) fn just_false() -> bool {
    false
}

#[cfg(feature = "strategy-auto-funding")]
pub mod auto_funding;
#[cfg(feature = "strategy-auto-redeeming")]
pub mod auto_redeeming;
#[cfg(feature = "strategy-closure-finalizer")]
pub mod channel_finalizer;
#[cfg(feature = "strategy-channel-lifecycle")]
pub mod channel_lifecycle;
pub mod errors;
#[cfg(feature = "strategy-pix")]
pub mod non_anonymous_pix;
#[cfg(feature = "strategy-pix")]
pub mod pix_recovery_store;
pub mod strategy;
