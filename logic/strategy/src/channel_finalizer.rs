use async_lock::RwLock;
use async_trait::async_trait;
use chain_actions::channels::ChannelActions;
use chain_actions::ChainActions;
use chain_db::traits::HoprCoreEthereumDbActions;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use hopr_internal_types::prelude::*;
use hopr_platform::time::native::current_time;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::Sub;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};
use validator::Validate;

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::SimpleCounter;

use crate::strategy::SingularStrategy;
use crate::{errors, Strategy};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_CLOSURE_FINALIZATIONS: SimpleCounter = SimpleCounter::new(
        "hopr_strategy_closure_auto_finalization_count",
        "Count of channels where closure finalizing was initiated automatically"
    )
    .unwrap();
}

/// Contains configuration of the [ClosureFinalizerStrategy].
#[derive(Debug, Clone, Copy, PartialEq, Eq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct ClosureFinalizerStrategyConfig {
    /// Do not attempt to finalize closure of channels that have
    /// been overdue for closure for more than this period.
    ///
    /// Default is 3600 seconds.
    #[default(Duration::from_secs(3600))]
    pub max_closure_overdue: Duration,
}

/// Strategy which runs per tick and finalizes `PendingToClose` channels
/// which have elapsed the grace period.
pub struct ClosureFinalizerStrategy<Db: HoprCoreEthereumDbActions + Clone + Send + Sync> {
    db: Arc<RwLock<Db>>,
    cfg: ClosureFinalizerStrategyConfig,
    chain_actions: ChainActions<Db>,
}

impl<Db: HoprCoreEthereumDbActions + Clone + Send + Sync> ClosureFinalizerStrategy<Db> {
    /// Constructs the strategy.
    pub fn new(cfg: ClosureFinalizerStrategyConfig, db: Arc<RwLock<Db>>, chain_actions: ChainActions<Db>) -> Self {
        Self { db, chain_actions, cfg }
    }
}

impl<Db: HoprCoreEthereumDbActions + Clone + Send + Sync> Display for ClosureFinalizerStrategy<Db> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::ClosureFinalizer(self.cfg))
    }
}

#[async_trait]
impl<Db: HoprCoreEthereumDbActions + Clone + Send + Sync> SingularStrategy for ClosureFinalizerStrategy<Db> {
    async fn on_tick(&self) -> errors::Result<()> {
        let ts_limit = current_time().sub(self.cfg.max_closure_overdue);

        let outgoing_channels = self.db.read().await.get_outgoing_channels().await?;

        let to_close = outgoing_channels
            .iter()
            .filter(|channel| {
                matches!(channel.status, ChannelStatus::PendingToClose(ct) if ct > ts_limit)
                    && channel.closure_time_passed(current_time())
            })
            .map(|channel| async {
                let channel_cpy = *channel;
                info!("channel closure finalizer: finalizing closure of {channel_cpy}");
                match self
                    .chain_actions
                    .close_channel(channel_cpy.destination, ChannelDirection::Outgoing, false)
                    .await
                {
                    Ok(_) => {
                        // Currently, we're not interested in awaiting the Close transactions to confirmation
                        debug!("channel closure finalizer: finalizing closure of {channel_cpy}");
                    }
                    Err(e) => error!("channel closure finalizer: failed to finalize closure of {channel_cpy}: {e}"),
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
    #[async_std::test]
    async fn test_should_close_only_non_overdue_pending_to_close_channels_with_elapsed_closure() {}
}
