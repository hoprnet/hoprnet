use std::{
    fmt::{Display, Formatter},
    ops::Sub,
    time::Duration,
};

use async_trait::async_trait;
use futures::StreamExt;
use hopr_lib::{
    ChannelStatusDiscriminants, Utc,
    exports::api::chain::{ChainReadChannelOperations, ChainWriteChannelOperations, ChannelSelector},
};
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

/// Contains configuration of the [`ClosureFinalizerStrategy`].
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
    A: ChainReadChannelOperations + ChainWriteChannelOperations + Send + Sync,
{
    async fn on_tick(&self) -> errors::Result<()> {
        let now = Utc::now();
        let mut outgoing_channels = self
            .hopr_chain_actions
            .stream_channels(
                ChannelSelector::default()
                    .with_source(*self.hopr_chain_actions.me())
                    .with_allowed_states(&[ChannelStatusDiscriminants::PendingToClose])
                    .with_closure_time_range(now.sub(self.cfg.max_closure_overdue)..=now),
            )
            .await
            .map_err(|e| errors::StrategyError::Other(e.into()))?;

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
    use std::{ops::Add, time::SystemTime};

    use futures_time::future::FutureExt;
    use hex_literal::hex;
    use hopr_lib::{
        Address, BytesRepresentable, ChainKeypair, ChannelEntry, ChannelStatus, HoprBalance, Keypair, XDaiBalance,
        exports::api::chain::{ChainEvent, ChainEvents},
    };
    use hopr_utils_chain_connector::{create_trustful_hopr_blokli_connector, testing::BlokliTestStateBuilder};
    use lazy_static::lazy_static;

    use super::*;

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

    #[tokio::test]
    async fn test_should_close_only_non_overdue_pending_to_close_channels_with_elapsed_closure() -> anyhow::Result<()> {
        let max_closure_overdue = Duration::from_secs(600);

        let channel_to_be_closed = ChannelEntry::new(
            *ALICE,
            *DAVE,
            10.into(),
            0.into(),
            ChannelStatus::PendingToClose(SystemTime::now().sub(Duration::from_secs(60))),
            1.into(),
        );

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHARLIE, &*DAVE, &*EUGENE],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([
                // Should leave this channel opened
                ChannelEntry::new(*ALICE, *BOB, 10.into(), 0.into(), ChannelStatus::Open, 0.into()),
                // Should leave this unfinalized, because the channel closure period has not yet elapsed
                ChannelEntry::new(
                    *ALICE,
                    *CHARLIE,
                    10.into(),
                    0.into(),
                    ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(60))),
                    1.into(),
                ),
                // Should finalize closure of this channel
                channel_to_be_closed,
                // Should leave this unfinalized, because the channel closure is long overdue
                ChannelEntry::new(
                    *ALICE,
                    *EUGENE,
                    10.into(),
                    0.into(),
                    ChannelStatus::PendingToClose(SystemTime::now().sub(max_closure_overdue * 2)),
                    1.into(),
                ),
            ])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&ALICE_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect(Duration::from_secs(3)).await?;
        let events = chain_connector.subscribe()?;

        let cfg = ClosureFinalizerStrategyConfig { max_closure_overdue };

        let strat = ClosureFinalizerStrategy::new(cfg, chain_connector);
        strat.on_tick().await?;

        events
            .filter(|event| {
                futures::future::ready(
                    matches!(event, ChainEvent::ChannelClosed(c) if channel_to_be_closed.get_id() == c.get_id()),
                )
            })
            .next()
            .timeout(futures_time::time::Duration::from_secs(2))
            .await?;

        // Cannot do snapshot testing here, since the execution is time-dependent

        Ok(())
    }
}
