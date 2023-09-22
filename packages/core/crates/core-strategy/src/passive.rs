use crate::config::StrategyConfig;
use crate::strategy::SingularStrategy;
use async_std::sync::RwLock;
use core_ethereum_actions::transaction_queue::TransactionSender;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_network::network::{Network, NetworkExternalActions};
use core_protocol::ticket_aggregation::processor::BasicTicketAggregationActions;
use core_types::channels::Ticket;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

#[allow(dead_code)]
pub struct PassiveStrategy<Db, Net>
where
    Db: HoprCoreEthereumDbActions,
    Net: NetworkExternalActions,
{
    db: Arc<RwLock<Db>>,
    network: Arc<RwLock<Network<Net>>>,
    tx_sender: TransactionSender,
    ticket_aggregator: BasicTicketAggregationActions<Result<Ticket, String>>,
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
        ticket_aggregator: BasicTicketAggregationActions<Result<Ticket, String>>,
    ) -> Self {
        Self {
            db,
            network,
            tx_sender,
            ticket_aggregator,
        }
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
