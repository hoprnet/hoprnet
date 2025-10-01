use std::{
    fmt::{Display, Formatter},
    ops::Sub,
    time::Duration,
};

use async_trait::async_trait;
use futures::StreamExt;
use hopr_api::chain::{ChainReadChannelOperations, ChainWriteChannelOperations, ChannelSelector};
use hopr_internal_types::prelude::*;
use hopr_platform::time::native::current_time;
use serde::{Deserialize, Serialize};
use serde_with::{DurationSeconds, serde_as};
use tracing::{debug, error, info};
use validator::Validate;

use crate::{Strategy, errors, strategy::SingularStrategy};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_CLOSURE_FINALIZATIONS: hopr_metrics::SimpleCounter = hopr_metrics::SimpleCounter::new(
        "hopr_strategy_closure_auto_finalization_count",
        "Count of channels where closure finalizing was initiated automatically"
    )
    .unwrap();
}

#[inline]
fn default_max_closure_overdue() -> Duration {
    Duration::from_secs(300)
}
/// Contains configuration of the [ClosureFinalizerStrategy].
#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct ClosureFinalizerStrategyConfig {
    /// Do not attempt to finalize closure of channels that have
    /// been overdue for closure for more than this period.
    ///
    /// Default is 300 seconds.
    #[serde_as(as = "DurationSeconds<u64>")]
    #[serde(default = "default_max_closure_overdue")]
    #[default(default_max_closure_overdue())]
    pub max_closure_overdue: Duration,
}

/// Strategy which runs per tick and finalizes `PendingToClose` channels
/// which have elapsed the grace period.
pub struct ClosureFinalizerStrategy<A> {
    cfg: ClosureFinalizerStrategyConfig,
    hopr_chain_actions: A,
}

impl<A> ClosureFinalizerStrategy<A> {
    /// Constructs the strategy.
    pub fn new(cfg: ClosureFinalizerStrategyConfig, hopr_chain_actions: A) -> Self {
        Self {
            hopr_chain_actions,
            cfg,
        }
    }
}

impl<A> Display for ClosureFinalizerStrategy<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::ClosureFinalizer(self.cfg))
    }
}

#[async_trait]
impl<A> SingularStrategy for ClosureFinalizerStrategy<A>
where
    A: ChainReadChannelOperations + ChainWriteChannelOperations +  Send + Sync,
{
    async fn on_tick(&self) -> errors::Result<()> {
        let ts_limit = current_time().sub(self.cfg.max_closure_overdue);

        let mut outgoing_channels = self
            .hopr_chain_actions
            .stream_channels(ChannelSelector {
                direction: vec![ChannelDirection::Outgoing],
                allowed_states: vec![ChannelStatusDiscriminants::PendingToClose],
                ..Default::default()
            })
            .await
            .map_err(|e| errors::StrategyError::Other(e.into()))?
            .filter_map(|channel| {
                futures::future::ready(
                    if matches!(channel.status, ChannelStatus::PendingToClose(ct) if ct > ts_limit)
                        && channel.closure_time_passed(current_time())
                    {
                        Some(channel)
                    } else {
                        None
                    },
                )
            });

        while let Some(channel) = outgoing_channels.next().await {
            info!(%channel, "channel closure finalizer: finalizing closure");
            match self.hopr_chain_actions.close_channel(&channel.get_id()).await {
                Ok(_) => {
                    // Currently, we're not interested in awaiting the Close transactions to confirmation
                    debug!(%channel, "channel closure finalizer: finalizing closure");
                    #[cfg(all(feature = "prometheus", not(test)))]
                    METRIC_COUNT_CLOSURE_FINALIZATIONS.increment();
                }
                Err(e) => error!(%channel, error = %e, "channel closure finalizer: failed to finalize closure"),
            }
        }


        debug!("channel closure finalizer: initiated closure finalization done");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{ops::Add, time::SystemTime};
    use futures::{FutureExt, future::ok, stream::BoxStream, future::BoxFuture};
    use hex_literal::hex;
    use hopr_api::chain::ChainReceipt;
    use hopr_crypto_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use lazy_static::lazy_static;
    use crate::errors::StrategyError;

    lazy_static! {
        static ref ALICE_KP: ChainKeypair = ChainKeypair::from_secret(&hex!(
            "492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"
        ))
        .expect("lazy static keypair should be valid");
        static ref ALICE: Address = ALICE_KP.public().to_address();
        static ref BOB: Address = hex!("3798fa65d6326d3813a0d33489ac35377f4496ef").into();
        static ref CHARLIE: Address = hex!("250eefb2586ab0873befe90b905126810960ee7c").into();
        static ref DAVE: Address = hex!("68499f50ff68d523385dc60686069935d17d762a").into();
        static ref EUGENE: Address = hex!("0c1da65d269f89b05e3775bf8fcd21a138e8cbeb").into();
    }

    // Due to async-trait and lifetimes, we cannot use mockall
    struct MockChainActions(Vec<ChannelEntry>, Hash);

    #[async_trait::async_trait]
    impl ChainReadChannelOperations for MockChainActions {
        type Error = StrategyError;

        async fn channel_by_parties(&self, _: &Address, _: &Address) -> Result<Option<ChannelEntry>, Self::Error> {
            unimplemented!()
        }

        async fn channel_by_id(&self, _: &ChannelId) -> Result<Option<ChannelEntry>, Self::Error> {
            unimplemented!()
        }

        async fn stream_channels<'a>(&'a self, _: ChannelSelector) -> Result<BoxStream<'a, ChannelEntry>, Self::Error> {
            // TODO: validate the selector
            Ok(futures::stream::iter(self.0.iter().cloned()).boxed())
        }
    }

    #[async_trait::async_trait]
    impl ChainWriteChannelOperations for MockChainActions {
        type Error = StrategyError;

        async fn open_channel<'a>(&'a self, _: &'a Address, _: HoprBalance) -> Result<BoxFuture<'a, Result<(ChannelId, ChainReceipt), Self::Error>>, Self::Error> {
            unimplemented!()
        }

        async fn fund_channel<'a>(&'a self, _: &'a ChannelId, _: HoprBalance) -> Result<BoxFuture<'a, Result<ChainReceipt, Self::Error>>, Self::Error> {
            unimplemented!()
        }

        async fn close_channel<'a>(&'a self, channel_id: &'a ChannelId) -> Result<BoxFuture<'a, Result<(ChannelStatus, ChainReceipt), Self::Error>>, Self::Error> {
            assert_eq!(self.1, *channel_id);
            Ok(ok((ChannelStatus::Closed, ChainReceipt::default())).boxed())
        }
    }



    #[tokio::test]
    async fn test_should_close_only_non_overdue_pending_to_close_channels_with_elapsed_closure() -> anyhow::Result<()> {
        let max_closure_overdue = Duration::from_secs(600);

        // Should leave this channel opened
        let c_open = ChannelEntry::new(*ALICE, *BOB, 10.into(), 0.into(), ChannelStatus::Open, 0.into());

        // Should leave this unfinalized, because the channel closure period has not yet elapsed
        let c_pending = ChannelEntry::new(
            *ALICE,
            *CHARLIE,
            10.into(),
            0.into(),
            ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(60))),
            0.into(),
        );

        // Should finalize closure of this channel
        let c_pending_elapsed = ChannelEntry::new(
            *ALICE,
            *DAVE,
            10.into(),
            0.into(),
            ChannelStatus::PendingToClose(SystemTime::now().sub(Duration::from_secs(60))),
            0.into(),
        );

        // Should leave this unfinalized, because the channel closure is long overdue
        let c_pending_overdue = ChannelEntry::new(
            *ALICE,
            *EUGENE,
            10.into(),
            0.into(),
            ChannelStatus::PendingToClose(SystemTime::now().sub(max_closure_overdue * 2)),
            0.into(),
        );

        let actions = MockChainActions(vec![c_open, c_pending, c_pending_elapsed, c_pending_overdue], c_pending_elapsed.get_id());

        let cfg = ClosureFinalizerStrategyConfig { max_closure_overdue };

        let strat = ClosureFinalizerStrategy::new(cfg, actions);
        strat.on_tick().await?;

        Ok(())
    }
}
