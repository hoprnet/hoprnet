use std::{
    fmt::{Display, Formatter},
    ops::Sub,
    time::Duration,
};

use async_trait::async_trait;
use futures::{StreamExt, stream::FuturesUnordered};
use hopr_chain_actions::channels::ChannelActions;
use hopr_db_sql::{accounts::HoprDbAccountOperations, channels::HoprDbChannelOperations};
use hopr_internal_types::prelude::*;
#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::SimpleCounter;
use hopr_platform::time::native::current_time;
use serde::{Deserialize, Serialize};
use serde_with::{DurationSeconds, serde_as};
use tracing::{debug, error, info};
use validator::Validate;

use crate::{Strategy, errors, strategy::SingularStrategy};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_CLOSURE_FINALIZATIONS: SimpleCounter = SimpleCounter::new(
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
pub struct ClosureFinalizerStrategy<Db, A>
where
    Db: HoprDbChannelOperations + Clone + Send + Sync,
    A: ChannelActions,
{
    db: Db,
    cfg: ClosureFinalizerStrategyConfig,
    hopr_chain_actions: A,
}

impl<Db, A> ClosureFinalizerStrategy<Db, A>
where
    Db: HoprDbChannelOperations + Clone + Send + Sync,
    A: ChannelActions,
{
    /// Constructs the strategy.
    pub fn new(cfg: ClosureFinalizerStrategyConfig, db: Db, hopr_chain_actions: A) -> Self {
        Self {
            db,
            hopr_chain_actions,
            cfg,
        }
    }
}

impl<Db, A> Display for ClosureFinalizerStrategy<Db, A>
where
    Db: HoprDbChannelOperations + Clone + Send + Sync,
    A: ChannelActions,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::ClosureFinalizer(self.cfg))
    }
}

#[async_trait]
impl<Db, A> SingularStrategy for ClosureFinalizerStrategy<Db, A>
where
    Db: HoprDbChannelOperations + HoprDbAccountOperations + Clone + Send + Sync,
    A: ChannelActions + Send + Sync,
{
    async fn on_tick(&self) -> errors::Result<()> {
        let ts_limit = current_time().sub(self.cfg.max_closure_overdue);

        let outgoing_channels = self
            .db
            .get_outgoing_channels(None)
            .await
            .map_err(hopr_db_sql::api::errors::DbError::from)?;

        let to_close = outgoing_channels
            .iter()
            .filter(|channel| {
                matches!(channel.status, ChannelStatus::PendingToClose(ct) if ct > ts_limit)
                    && channel.closure_time_passed(current_time())
            })
            .map(|channel| async {
                let channel_cpy = *channel;
                info!(channel = %channel_cpy, "channel closure finalizer: finalizing closure");
                match self
                    .hopr_chain_actions
                    .close_channel(channel_cpy.destination, ChannelDirection::Outgoing, false)
                    .await
                {
                    Ok(_) => {
                        // Currently, we're not interested in awaiting the Close transactions to confirmation
                        debug!("channel closure finalizer: finalizing closure of {channel_cpy}");
                    }
                    Err(e) => error!(%channel_cpy, error = %e, "channel closure finalizer: failed to finalize closure"),
                }
            })
            .collect::<FuturesUnordered<_>>()
            .count()
            .await;

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_COUNT_CLOSURE_FINALIZATIONS.increment_by(to_close as u64);

        debug!("channel closure finalizer: initiated closure finalization of {to_close} channels");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{ops::Add, time::SystemTime};

    use futures::{FutureExt, future::ok};
    use hex_literal::hex;
    use hopr_chain_actions::action_queue::{ActionConfirmation, PendingAction};
    use hopr_chain_types::{actions::Action, chain_events::ChainEventType};
    use hopr_crypto_random::random_bytes;
    use hopr_crypto_types::prelude::*;
    use hopr_db_sql::{HoprDbGeneralModelOperations, db::HoprDb};
    use hopr_primitive_types::prelude::*;
    use lazy_static::lazy_static;
    use mockall::mock;

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

    mock! {
        ChannelAct { }
        #[async_trait]
        impl ChannelActions for ChannelAct {
            async fn open_channel(&self, destination: Address, amount: HoprBalance) -> hopr_chain_actions::errors::Result<PendingAction>;
            async fn fund_channel(&self, channel_id: Hash, amount: HoprBalance) -> hopr_chain_actions::errors::Result<PendingAction>;
            async fn close_channel(
                &self,
                counterparty: Address,
                direction: ChannelDirection,
                redeem_before_close: bool,
            ) -> hopr_chain_actions::errors::Result<PendingAction>;
        }
    }

    fn mock_action_confirmation_closure(channel: ChannelEntry) -> ActionConfirmation {
        let random_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());
        ActionConfirmation {
            tx_hash: random_hash,
            event: Some(ChainEventType::ChannelClosureInitiated(channel)),
            action: Action::CloseChannel(channel, ChannelDirection::Outgoing),
        }
    }

    #[tokio::test]
    async fn test_should_close_only_non_overdue_pending_to_close_channels_with_elapsed_closure() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await?;

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

        let db_clone = db.clone();
        db.begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    db_clone.upsert_channel(Some(tx), c_open).await?;
                    db_clone.upsert_channel(Some(tx), c_pending).await?;
                    db_clone.upsert_channel(Some(tx), c_pending_elapsed).await?;
                    db_clone.upsert_channel(Some(tx), c_pending_overdue).await
                })
            })
            .await?;

        let mut actions = MockChannelAct::new();
        actions
            .expect_close_channel()
            .once()
            .withf(|addr, dir, _| DAVE.eq(addr) && ChannelDirection::Outgoing.eq(dir))
            .return_once(move |_, _, _| Ok(ok(mock_action_confirmation_closure(c_pending_elapsed)).boxed()));

        let cfg = ClosureFinalizerStrategyConfig { max_closure_overdue };

        let strat = ClosureFinalizerStrategy::new(cfg, db.clone(), actions);
        strat.on_tick().await?;

        Ok(())
    }
}
