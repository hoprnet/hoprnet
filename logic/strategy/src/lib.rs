//! This crate contains all the Strategies for HOPRd.
//! Strategies are vital for (partial) automation of ticket and HOPR channel operations
//! during node runtime.
//!
//! - [passive strategy](crate::strategy::MultiStrategy)
//! - [auto funding strategy](crate::auto_funding)
//! - [auto redeeming strategy](crate::auto_redeeming)
//! - [multiple strategy chains](crate::strategy)
//!
//! HOPRd can be configured to use any of the above strategies.
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

use std::str::FromStr;

use hopr_primitive_types::prelude::*;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, VariantNames};

use crate::{
    Strategy::AutoRedeeming, auto_funding::AutoFundingStrategyConfig, auto_redeeming::AutoRedeemingStrategyConfig,
    channel_finalizer::ClosureFinalizerStrategyConfig, strategy::MultiStrategyConfig,
};

pub mod auto_funding;
pub mod auto_redeeming;
mod channel_finalizer;
pub mod errors;
pub mod strategy;

/// Lists all possible strategies with their respective configurations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, EnumString, VariantNames)]
#[strum(serialize_all = "snake_case")]
pub enum Strategy {
    AutoRedeeming(AutoRedeemingStrategyConfig),
    AutoFunding(AutoFundingStrategyConfig),
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
        execution_interval: 60,
        strategies: vec![
            // AutoFunding(AutoFundingStrategyConfig {
            // min_stake_threshold: Balance::new_from_str("1000000000000000000", BalanceType::HOPR),
            // funding_amount: Balance::new_from_str("10000000000000000000", BalanceType::HOPR),
            // }),
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

#[cfg(test)]
pub(crate) mod tests {
    use futures::{FutureExt, Stream, StreamExt, future::BoxFuture, stream::BoxStream};
    use hopr_api::{
        chain::{
            ChainReadChannelOperations, ChainReceipt, ChainWriteChannelOperations, ChainWriteTicketOperations,
            ChannelSelector,
        },
        db::TicketSelector,
    };
    use hopr_internal_types::{
        channels::{ChannelEntry, ChannelId, ChannelStatus},
        prelude::AcknowledgedTicket,
    };
    use hopr_primitive_types::{balance::HoprBalance, prelude::Address};

    use crate::errors::StrategyError;

    // Mock helper needs to be created, because ChainWriteTicketOperations and ChainReadChannelOperations
    // cannot be mocked directly due to impossible lifetimes.
    #[mockall::automock]
    pub trait TestActions {
        fn me(&self) -> &Address;
        fn fund_channel(&self, channel_id: &ChannelId, amount: HoprBalance) -> Result<ChainReceipt, StrategyError>;
        fn close_channel(&self, channel_id: &ChannelId) -> Result<(ChannelStatus, ChainReceipt), StrategyError>;
        fn redeem_with_selector(&self, selector: TicketSelector) -> Vec<ChainReceipt>;
        fn stream_channels(&self, selector: ChannelSelector) -> impl Stream<Item = ChannelEntry> + Send;
        fn channel_by_id(&self, channel_id: &ChannelId) -> Option<ChannelEntry>;
    }

    pub struct MockChainActions<T>(pub std::sync::Arc<T>);

    impl<T> Clone for MockChainActions<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    #[async_trait::async_trait]
    impl<T: TestActions + Send + Sync> ChainReadChannelOperations for MockChainActions<T> {
        type Error = StrategyError;

        fn me(&self) -> &Address {
            self.0.me()
        }

        async fn channel_by_parties(&self, _: &Address, _: &Address) -> Result<Option<ChannelEntry>, Self::Error> {
            unimplemented!()
        }

        async fn channel_by_id(&self, channel_id: &ChannelId) -> Result<Option<ChannelEntry>, Self::Error> {
            Ok(self.0.channel_by_id(channel_id))
        }

        async fn stream_channels<'a>(
            &'a self,
            selector: ChannelSelector,
        ) -> Result<BoxStream<'a, ChannelEntry>, Self::Error> {
            Ok(self.0.stream_channels(selector).boxed())
        }
    }

    #[async_trait::async_trait]
    impl<T: TestActions + Send + Sync> ChainWriteChannelOperations for MockChainActions<T> {
        type Error = StrategyError;

        async fn open_channel<'a>(
            &'a self,
            _: &'a Address,
            _: HoprBalance,
        ) -> Result<BoxFuture<'a, Result<(ChannelId, ChainReceipt), Self::Error>>, Self::Error> {
            unimplemented!()
        }

        async fn fund_channel<'a>(
            &'a self,
            channel_id: &'a ChannelId,
            amount: HoprBalance,
        ) -> Result<BoxFuture<'a, Result<ChainReceipt, Self::Error>>, Self::Error> {
            Ok(futures::future::ready(self.0.fund_channel(channel_id, amount)).boxed())
        }

        async fn close_channel<'a>(
            &'a self,
            channel_id: &'a ChannelId,
        ) -> Result<BoxFuture<'a, Result<(ChannelStatus, ChainReceipt), Self::Error>>, Self::Error> {
            Ok(futures::future::ready(self.0.close_channel(channel_id)).boxed())
        }
    }

    #[async_trait::async_trait]
    impl<T: TestActions + Send + Sync> ChainWriteTicketOperations for MockChainActions<T> {
        type Error = StrategyError;

        async fn redeem_ticket(
            &self,
            _: AcknowledgedTicket,
        ) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, Self::Error> {
            unimplemented!()
        }

        async fn redeem_tickets_via_selector(
            &self,
            selector: TicketSelector,
        ) -> Result<Vec<BoxFuture<'_, Result<ChainReceipt, Self::Error>>>, Self::Error> {
            let receipts = self.0.redeem_with_selector(selector);
            Ok(receipts
                .into_iter()
                .map(|r| futures::future::ready(Ok(r)).boxed())
                .collect())
        }
    }
}
