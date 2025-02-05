use std::fmt::{Display, Formatter};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use validator::Validate;
use chain_actions::channels::ChannelActions;
use hopr_db_sql::channels::HoprDbChannelOperations;
use hopr_db_sql::prelude::HoprDbTicketOperations;
use crate::Strategy;
use crate::strategy::SingularStrategy;

#[derive(Debug, Clone, Copy, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct RelativeAutoFundingStrategyConfig {
    #[validate(range(min = 0.0, max = 1.0))]
    pub unrealized_balance_ratio_threshold: f64,
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
        let out_channels = futures::stream::iter(self.db
            .get_outgoing_channels(None)
            .await
            .map_err(hopr_db_sql::api::errors::DbError::from)?)
            .then(|channel| async move {
                db.get_unrealized_balance(&channel)
                    .await
                    .map(|unrealized_balance| (channel, unrealized_balance))
                    .map_err(hopr_db_sql::api::errors::DbError::from)
            })
            .collect::<Vec<_>>()
            .await;

        self.db.get_unrealized_balance()

        Ok(())
    }
}



