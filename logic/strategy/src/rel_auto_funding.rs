use crate::strategy::SingularStrategy;
use crate::Strategy;
use async_trait::async_trait;
use chain_actions::channels::ChannelActions;
use futures::StreamExt;
use hopr_db_sql::channels::HoprDbChannelOperations;
use hopr_db_sql::prelude::HoprDbTicketOperations;
use hopr_primitive_types::prelude::{Balance, BalanceType, UnitaryFloatOps};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::fmt::{Display, Formatter};
use validator::{Validate, ValidationError};

fn default_max_stake() -> Balance {
    Balance::new_from_str("200000000000000000000", BalanceType::HOPR)
}

fn default_unrealized_balance_threshold() -> f64 {
    0.9
}

fn default_acceptable_unrealized_balance_ratio() -> f64 {
    0.4
}

fn validate_rel_auto_funding_cfg(cfg: &RelativeAutoFundingStrategyConfig) -> Result<(), ValidationError> {
    if cfg.unrealized_balance_ratio_threshold <= cfg.acceptable_unrealized_balance_ratio {
        return Err(ValidationError::new(
            "unrealized_balance_ratio_threshold <= acceptable_unrealized_balance_ratio",
        ));
    }

    if cfg.max_channel_stake == BalanceType::HOPR.zero() {
        return Err(ValidationError::new("max_channel_stake: 0"));
    }

    Ok(())
}

#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
#[validate(schema(function = "validate_rel_auto_funding_cfg"))]
pub struct RelativeAutoFundingStrategyConfig {
    #[validate(range(min = 0.0, max = 1.0))]
    #[serde(default = "default_unrealized_balance_threshold")]
    #[default(default_unrealized_balance_threshold())]
    pub unrealized_balance_ratio_threshold: f64,

    #[validate(range(min = 0.0, max = 1.0))]
    #[serde(default = "default_acceptable_unrealized_balance_ratio")]
    #[default(default_acceptable_unrealized_balance_ratio())]
    pub acceptable_unrealized_balance_ratio: f64,

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
where
    A: ChannelActions,
    Db: HoprDbChannelOperations + HoprDbTicketOperations + Clone + Send + Sync,
{
    pub fn new(cfg: RelativeAutoFundingStrategyConfig, db: Db, chain_actions: A) -> Self {
        Self { chain_actions, db, cfg }
    }
}

#[async_trait]
impl<A, Db> SingularStrategy for RelativeAutoFundingStrategy<A, Db>
where
    A: ChannelActions + Sync,
    Db: HoprDbChannelOperations + HoprDbTicketOperations + Clone + Send + Sync,
{
    async fn on_tick(&self) -> crate::errors::Result<()> {
        let cfg = self.cfg;
        let channels_to_fund = futures::stream::iter(
            self.db
                .get_outgoing_channels(None)
                .await
                .map_err(hopr_db_sql::api::errors::DbError::from)?,
        )
        .filter_map(|channel| {
            let db = self.db.clone();
            async move {
                match db.get_unrealized_balance(&channel).await {
                    Ok(unrealized_balance) => {
                        let diminished_balance = channel
                            .balance
                            .mul_f64(cfg.unrealized_balance_ratio_threshold)
                            .expect("impossible: unrealized_balance_ratio_threshold not in [0,1]");
                        // The unrealized balance is above the threshold,
                        // so the channel needs funding
                        if unrealized_balance > diminished_balance && channel.balance < cfg.max_channel_stake {
                            // The new stake is equal to the current unrealized balance
                            // divided by the acceptable unrealized balance ratio.
                            // But this must not be more than the maximum channel stake.
                            let new_channel_stake = unrealized_balance
                                .div_f64(cfg.acceptable_unrealized_balance_ratio)
                                .expect("impossible: acceptable_unrealized_balance_ratio not in [0,1]")
                                .amount()
                                .min(cfg.max_channel_stake.amount());

                            let stake_increase = BalanceType::HOPR.balance(new_channel_stake) - channel.balance;
                            if !stake_increase.is_zero() {
                                Some((channel.get_id(), new_channel_stake))
                            } else {
                                tracing::warn!(
                                    channel_id = %channel.get_id(),
                                    stake = %channel.balance,
                                    "channel stake cannot be increased further"
                                );
                                None
                            }
                        } else {
                            tracing::debug!(
                                channel_id = %channel.get_id(),
                                %unrealized_balance,
                                %diminished_balance,
                                stake = %channel.balance,
                                "channel does not need auto-funding or is at max stake");
                            None
                        }
                    }
                    Err(error) => {
                        tracing::error!(%error, %channel, "failed to retrieve unrealized balance");
                        None
                    }
                }
            }
        })
        .collect::<Vec<_>>()
        .await;

        for (channel_id, amount) in channels_to_fund {
            match self
                .chain_actions
                .fund_channel(channel_id, BalanceType::HOPR.balance(amount))
                .await
            {
                Ok(_) => tracing::info!(%channel_id, %amount, "channel auto-funding submitted"),
                Err(error) => tracing::error!(%channel_id, %amount, %error, "channel auto-funding failed"),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // TODO: add tests
}
