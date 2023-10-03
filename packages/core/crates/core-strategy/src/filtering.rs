use std::fmt::{Display, Formatter};
use std::sync::Arc;
use async_std::sync::RwLock;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use validator::Validate;
use core_ethereum_actions::errors::CoreEthereumActionsError::ChannelDoesNotExist;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_path::channel_graph::ChannelChange;
use core_types::acknowledgement::AcknowledgedTicket;
use core_types::channels::ChannelEntry;
use utils_log::error;
use crate::errors::StrategyError::Filtered;
use crate::filtering::FilterTarget::{All, OnAckTicket, OnChannelChange};
use crate::Strategy;
use crate::strategy::SingularStrategy;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum FilterTarget {
    /// Will act on all event types.
    All,
    /// Will act on `on_acknowledged_ticket` event.
    OnAckTicket,
    /// Will act on `on_channel_state_change` event.
    OnChannelChange,
}

#[derive(Debug, Clone, PartialEq, Validate, Serialize, Deserialize)]
pub struct FilteringStrategyConfig {
    pub targets: Vec<FilterTarget>,
    pub require_all: bool,

    /// Upper bound for unrealized balance
    pub unrealized_balance_ratio: Option<f64>,
    pub unredeemed_tickets_count: Option<usize>,
}

impl Default for FilteringStrategyConfig {
    fn default() -> Self {
        Self {
            targets: vec![All],
            require_all: false,
            unrealized_balance_ratio: None,
            unredeemed_tickets_count: Some(100),
        }
    }
}

/// Strategy meant just for filtering events based on different criteria for
/// other strategies that follow in a `MultiStrategy` chain.
/// The strategy itself does nothing, except succeeding or failing with `Filtered` error
/// depending if the given criteria is met.
pub struct FilteringStrategy<Db: HoprCoreEthereumDbActions> {
    db: Arc<RwLock<Db>>,
    cfg: FilteringStrategyConfig,
}

impl<Db: HoprCoreEthereumDbActions> Display for FilteringStrategy<Db> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::Filtering(Default::default()))
    }
}

impl<Db: HoprCoreEthereumDbActions> FilteringStrategy<Db> {
    pub fn new(db: Arc<RwLock<Db>>, cfg: FilteringStrategyConfig) -> Self {
        Self { db, cfg }
    }

    async fn check_unredeemed_tickets_count(&self, channel: &ChannelEntry) -> bool {
        todo!()
    }

    async fn check_unrealized_balance(&self, channel: &ChannelEntry) -> bool {
        todo!()
    }

    async fn filter_channel(&self, channel: &ChannelEntry) -> crate::errors::Result<()> {

        let result = if self.cfg.require_all {
             self.check_unredeemed_tickets_count(channel).await &&
             self.check_unrealized_balance(channel).await
        } else {
            self.check_unredeemed_tickets_count(channel).await ||
            self.check_unrealized_balance(channel).await
        };

        if result {
            Ok(())
        } else {
            Err(Filtered)
        }
    }
}

#[async_trait(? Send)]
impl<Db: HoprCoreEthereumDbActions> SingularStrategy for FilteringStrategy<Db> {
    async fn on_acknowledged_ticket(&self, ack: &AcknowledgedTicket) -> crate::errors::Result<()> {
        if self.cfg.targets.iter().any(|t| All.eq(t) || OnAckTicket.eq(t)) {
            let maybe_channel = self.db.read().await.get_channel(&ack.ticket.channel_id)
                .await?;

            if let Some(channel) =  maybe_channel {
                self.filter_channel(&channel).await
            } else {
                error!("{self} strategy: encountered {ack} in non-existing channel.");
                Err(ChannelDoesNotExist.into())
            }
        } else {
            Ok(())
        }
    }

    async fn on_channel_state_changed(&self, channel: &ChannelEntry, _change: ChannelChange) -> crate::errors::Result<()> {
        if self.cfg.targets.iter().any(|t| All.eq(t) || OnChannelChange.eq(t)) {
            self.filter_channel(channel).await
        } else {
            Ok(())
        }
    }
}