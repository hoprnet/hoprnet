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

use std::{ops::Sub, str::FromStr, time::Duration};

use futures::{StreamExt, pin_mut};
use futures_concurrency::stream::Merge;
use hopr_api::chain::ChainEvent;
use hopr_internal_types::{
    channels::ChannelStatus,
    prelude::{AcknowledgedTicket, ChannelChange},
};
use hopr_primitive_types::prelude::*;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, VariantNames};

use crate::{
    Strategy::AutoRedeeming,
    auto_funding::AutoFundingStrategyConfig,
    auto_redeeming::AutoRedeemingStrategyConfig,
    channel_finalizer::ClosureFinalizerStrategyConfig,
    strategy::{MultiStrategyConfig, SingularStrategy},
};

pub mod auto_funding;
pub mod auto_redeeming;
pub mod channel_finalizer;
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
        execution_interval: Duration::from_secs(60),
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

enum StrategyEvent {
    Tick,
    ChainEvent(ChainEvent),
    AckTicket(AcknowledgedTicket),
}

/// Streams [`ChainEvents`](ChainEvent), [`AcknowledgedTickets`](AcknowledgedTicket) and `tick` at regular time
/// intervals as events into the given `strategy`.
pub fn stream_events_to_strategy_with_tick<C, T, S>(
    strategy: std::sync::Arc<S>,
    chain_events: C,
    ticket_events: T,
    tick: std::time::Duration,
    me: Address,
) -> hopr_async_runtime::AbortHandle
where
    C: futures::stream::Stream<Item = ChainEvent> + Send + 'static,
    T: futures::stream::Stream<Item = AcknowledgedTicket> + Send + 'static,
    S: SingularStrategy + Send + Sync + 'static,
{
    let tick_stream = futures_time::stream::interval(tick.into()).map(|_| StrategyEvent::Tick);
    let chain_stream = chain_events.map(StrategyEvent::ChainEvent).fuse();
    let ticket_stream = ticket_events.map(StrategyEvent::AckTicket).fuse();

    let (stream, abort_handle) = futures::stream::abortable((tick_stream, chain_stream, ticket_stream).merge());
    hopr_async_runtime::prelude::spawn(async move {
        pin_mut!(stream);
        while let Some(event) = stream.next().await {
            match event {
                StrategyEvent::Tick => {
                    if let Err(error) = strategy.on_tick().await {
                        tracing::error!(%error, "error while notifying tick to strategy");
                    }
                }
                StrategyEvent::ChainEvent(chain_event) => {
                    // TODO: rework strategies so that they can react directly to `ChainEvent`s and avoid the following
                    // conversion to `ChannelChange`
                    match chain_event {
                        ChainEvent::ChannelOpened(channel) => {
                            if let Some(dir) = channel.direction(&me) {
                                let _ = strategy
                                    .on_own_channel_changed(
                                        &channel,
                                        dir,
                                        ChannelChange::Status {
                                            left: ChannelStatus::Closed,
                                            right: ChannelStatus::Open,
                                        },
                                    )
                                    .await;
                            }
                        }
                        ChainEvent::ChannelClosureInitiated(channel) => {
                            if let Some(dir) = channel.direction(&me) {
                                let _ = strategy
                                    .on_own_channel_changed(
                                        &channel,
                                        dir,
                                        ChannelChange::Status {
                                            left: ChannelStatus::Open,
                                            right: channel.status,
                                        },
                                    )
                                    .await;
                            }
                        }
                        ChainEvent::ChannelClosed(channel) => {
                            if let Some(dir) = channel.direction(&me) {
                                let _ = strategy
                                    .on_own_channel_changed(
                                        &channel,
                                        dir,
                                        ChannelChange::Status {
                                            left: ChannelStatus::PendingToClose(
                                                std::time::SystemTime::now().sub(Duration::from_secs(30)),
                                            ),
                                            right: ChannelStatus::Closed,
                                        },
                                    )
                                    .await;
                            }
                        }
                        ChainEvent::ChannelBalanceIncreased(channel, diff) => {
                            if let Some(dir) = channel.direction(&me) {
                                let _ = strategy
                                    .on_own_channel_changed(
                                        &channel,
                                        dir,
                                        ChannelChange::Balance {
                                            left: channel.balance - diff,
                                            right: channel.balance,
                                        },
                                    )
                                    .await;
                            }
                        }
                        ChainEvent::ChannelBalanceDecreased(channel, diff) => {
                            if let Some(dir) = channel.direction(&me) {
                                let _ = strategy
                                    .on_own_channel_changed(
                                        &channel,
                                        dir,
                                        ChannelChange::Balance {
                                            left: channel.balance + diff,
                                            right: channel.balance,
                                        },
                                    )
                                    .await;
                            }
                        }
                        ChainEvent::TicketRedeemed(channel, _) => {
                            if let Some(dir) = channel.direction(&me) {
                                let _ = strategy
                                    .on_own_channel_changed(
                                        &channel,
                                        dir,
                                        ChannelChange::TicketIndex {
                                            left: channel.ticket_index.as_u64() - 1,
                                            right: channel.ticket_index.as_u64(),
                                        },
                                    )
                                    .await;
                            }
                        }
                        _ => {}
                    }
                }
                StrategyEvent::AckTicket(ack_ticket) => {
                    if let Err(error) = strategy.on_acknowledged_winning_ticket(&ack_ticket).await {
                        tracing::error!(%error, "error while notifying new winning ticket to strategy");
                    }
                }
            }
        }
    });

    abort_handle
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
