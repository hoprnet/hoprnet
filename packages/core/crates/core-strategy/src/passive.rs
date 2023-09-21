use crate::config::StrategyConfig;
use crate::strategy::SingularStrategy;
use async_std::sync::RwLock;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_ethereum_misc::transaction_queue::TransactionSender;
use core_network::network::{Network, NetworkExternalActions};
use std::fmt::{Display, Formatter};
use std::sync::Arc;

pub struct PassiveStrategy<Db, Net>
where
    Db: HoprCoreEthereumDbActions,
    Net: NetworkExternalActions,
{
    db: Arc<RwLock<Db>>,
    network: Arc<RwLock<Network<Net>>>,
    tx_sender: TransactionSender,
}

impl<Db, Net> PassiveStrategy<Db, Net>
where
    Db: HoprCoreEthereumDbActions,
    Net: NetworkExternalActions,
{
    pub fn new(
        _cfg: StrategyConfig,
        db: Arc<RwLock<Db>>,
        network: Arc<RwLock<Network<Net>>>,
        tx_sender: TransactionSender,
    ) -> Self {
        Self { db, network, tx_sender }
    }
}

impl<Db, Net> Display for PassiveStrategy<Db, Net>
where
    Db: HoprCoreEthereumDbActions,
    Net: NetworkExternalActions,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "passive")
    }
}

impl<Db, Net> SingularStrategy for PassiveStrategy<Db, Net>
where
    Db: HoprCoreEthereumDbActions,
    Net: NetworkExternalActions,
{
}
