use crate::strategy::SingularStrategy;
use crate::Strategy;
use async_std::sync::RwLock;
use async_trait::async_trait;
use core_ethereum_actions::channels::fund_channel;
use core_ethereum_actions::transaction_queue::TransactionSender;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::channels::ChannelEntry;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use utils_log::info;
use utils_types::primitives::{Balance, BalanceType};
use validator::Validate;

/// Configuration for `AutoFundingStrategy`
#[derive(Debug, Clone, PartialEq, Eq, Validate, Serialize, Deserialize)]
pub struct AutoFundingStrategyConfig {
    /// Minimum stake that a channel's balance must not go below.
    /// Default is 0 HOPR, which never triggers funding.
    pub min_stake_threshold: Balance,
}

impl Default for AutoFundingStrategyConfig {
    fn default() -> Self {
        Self {
            min_stake_threshold: Balance::zero(BalanceType::HOPR),
        }
    }
}

/// The `AutoFundingStrategys` automatically funds channel that
/// dropped it's staked balance below the configured threshold.
pub struct AutoFundingStrategy<Db: HoprCoreEthereumDbActions> {
    tx_sender: TransactionSender,
    db: Arc<RwLock<Db>>,
    cfg: AutoFundingStrategyConfig,
}

impl<Db: HoprCoreEthereumDbActions> AutoFundingStrategy<Db> {
    pub fn new(cfg: AutoFundingStrategyConfig, db: Arc<RwLock<Db>>, tx_sender: TransactionSender) -> Self {
        Self { cfg, tx_sender, db }
    }
}

impl<Db: HoprCoreEthereumDbActions> Display for AutoFundingStrategy<Db> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::AutoFunding(Default::default()))
    }
}

#[async_trait(? Send)]
impl<Db: HoprCoreEthereumDbActions> SingularStrategy for AutoFundingStrategy<Db> {
    async fn on_channel_state_changed(&self, channel: &ChannelEntry) -> crate::errors::Result<()> {
        if channel.balance.lt(&self.cfg.min_stake_threshold) {
            info!(
                "{self} strategy: stake on {channel} is below treshhold {} < {}",
                channel.balance, self.cfg.min_stake_threshold
            );
            let to_stake = self.cfg.min_stake_threshold.sub(&channel.balance);
            let _ = fund_channel(
                self.db.clone(),
                self.tx_sender.clone(),
                channel.get_id(),
                to_stake.clone(),
            )
            .await?;
            info!("{self} strategy: issued re-staking of {channel} with {to_stake}");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
