use std::fmt::{Display, Formatter};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use validator::Validate;
use chain_actions::channels::ChannelActions;
use hopr_db_sql::channels::HoprDbChannelOperations;
use hopr_db_sql::prelude::HoprDbTicketOperations;
use hopr_primitive_types::prelude::{Balance, BalanceType};
use crate::Strategy;
use crate::strategy::SingularStrategy;

fn default_max_stake() -> Balance {
    Balance::new_from_str("200000000000000000000", BalanceType::HOPR)
}

fn default_unrealized_balance_threshold() -> f64 {
    0.9
}

#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct RelativeAutoFundingStrategyConfig {
    #[validate(range(min = 0.0, max = 1.0))]
    #[serde(default = "default_unrealized_balance_threshold")]
    #[default(default_unrealized_balance_threshold())]
    pub unrealized_balance_ratio_threshold: f64,

    #[serde(default = "default_max_stake")]
    #[serde_as(as = "DisplayFromStr")]
    #[default(default_max_stake())]
    pub max_channel_stake: Balance,
}

pub struct RelativeAutoFundingStrategy<A, Db> {
    chain_actions: A,
    db: Db,
    cfg: RelativeAutoFundingStrategyConfig,
}

impl<A, Db> Display for RelativeAutoFundingStrategy<A, Db> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", Strategy::RelativeAutoFunding(self.cfg))
    }
}

impl<A, Db> RelativeAutoFundingStrategy<A, Db>
where A: ChannelActions,  Db: HoprDbChannelOperations + HoprDbTicketOperations + Clone + Send + Sync {
    pub fn new(cfg: RelativeAutoFundingStrategyConfig, db: Db, chain_actions: A,) -> Self {
        Self { chain_actions, db, cfg }
    }
}

impl<A, Db> SingularStrategy for RelativeAutoFundingStrategy<A, Db>
where A: ChannelActions,  Db: HoprDbChannelOperations + HoprDbTicketOperations + Clone + Send + Sync {
    async fn on_tick(&self) -> crate::errors::Result<()> {
        let db = self.db.clone();
        let channels_to_fund = futures::stream::iter(self.db
            .get_outgoing_channels(None)
            .await
            .map_err(hopr_db_sql::api::errors::DbError::from)?)
            .filter_map(|channel| async move {
                match db.get_unrealized_balance(&channel).await {
                    Ok(unrealized_balance) => {
                        let amount_to_fund = BalanceType::HOPR.zero();
                        Some((channel.get_id(), amount_to_fund))
                    }
                    Err(error) => {
                        tracing::error!(%error, %channel, "failed to retrieve unrealized balance");
                        None
                    }
                }
            })
            .collect::<Vec<_>>()
            .await;

        for (channel_id, amount) in channels_to_fund {
            match self.chain_actions.fund_channel(channel_id, amount).await {
                Ok(_) => tracing::info!(%channel_id, %amount, "channel auto-funding submitted"),
                Err(error) => tracing::error!(%channel_id, %amount, %error, "channel auto-funding failed"),
            }
        }

        Ok(())
    }
}



