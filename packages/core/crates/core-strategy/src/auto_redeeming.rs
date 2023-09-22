use std::fmt::{Display, Formatter};
use std::sync::Arc;
use async_std::sync::RwLock;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use validator::Validate;
use core_ethereum_actions::redeem::redeem_ticket;
use core_ethereum_actions::transaction_queue::TransactionSender;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::acknowledgement::{AcknowledgedTicket};
use utils_log::{info};
use crate::strategy::SingularStrategy;

/// Configuration object for the `AutoRedeemingStrategy`
#[derive(Clone, Debug, Default, Serialize, Deserialize, Validate, PartialEq, Eq)]
pub struct AutoRedeemingStrategyConfig ;

/// The `AutoRedeemingStrategy` automatically sends an acknowledged ticket
/// for redemption once encountered.
/// The strategy does not await the result of the redemption.
pub struct AutoRedeemingStrategy<Db: HoprCoreEthereumDbActions> {
    db: Arc<RwLock<Db>>,
    tx_sender: TransactionSender,
    #[allow(dead_code)]
    cfg: AutoRedeemingStrategyConfig
}

impl<Db: HoprCoreEthereumDbActions> Display for AutoRedeemingStrategy<Db> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "auto_redeeming")
    }
}

impl<Db: HoprCoreEthereumDbActions> AutoRedeemingStrategy<Db> {
    pub fn new(cfg: AutoRedeemingStrategyConfig, db: Arc<RwLock<Db>>, tx_sender: TransactionSender) -> Self {
        Self { cfg, db, tx_sender }
    }
}

#[async_trait(? Send)]
impl<Db: HoprCoreEthereumDbActions + 'static> SingularStrategy for AutoRedeemingStrategy<Db> {
    async fn on_acknowledged_ticket(&self, ack: &AcknowledgedTicket) -> crate::errors::Result<()> {
        info!("{self} strategy: auto-redeeming {ack}");
        let _ = redeem_ticket(self.db.clone(), ack.clone(), self.tx_sender.clone()).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

}