use std::sync::{Arc, OnceLock};

use hopr_crypto_types::types::Hash;
use hopr_db_sql::api::tickets::{AggregationPrerequisites, HoprDbTicketOperations};
use hopr_transport_p2p::swarm::{TicketAggregationRequestType, TicketAggregationResponseType};
use hopr_transport_ticket_aggregation::{TicketAggregationActions, TicketAggregationError, TicketAggregatorTrait};

#[derive(Debug, Clone)]
pub struct TicketAggregatorProxy<Db>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug,
{
    #[allow(dead_code)]
    db: Db,
    #[allow(dead_code)]
    maybe_writer: Arc<OnceLock<TicketAggregationActions<TicketAggregationResponseType, TicketAggregationRequestType>>>,
    #[allow(dead_code)]
    agg_timeout: std::time::Duration,
}

impl<Db> TicketAggregatorProxy<Db>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug,
{
    pub fn new(
        db: Db,
        maybe_writer: Arc<
            OnceLock<TicketAggregationActions<TicketAggregationResponseType, TicketAggregationRequestType>>,
        >,
        agg_timeout: std::time::Duration,
    ) -> Self {
        Self {
            db,
            maybe_writer,
            agg_timeout,
        }
    }
}

#[async_trait::async_trait]
impl<Db> TicketAggregatorTrait for TicketAggregatorProxy<Db>
where
    Db: HoprDbTicketOperations + Send + Sync + Clone + std::fmt::Debug,
{
    async fn aggregate_tickets(
        &self,
        _channel: &Hash,
        _prerequisites: AggregationPrerequisites,
    ) -> hopr_transport_ticket_aggregation::Result<()> {
        // if let Some(writer) = self.maybe_writer.clone().get() {
        //     AwaitingAggregator::new(self.db.clone(), writer.clone(), self.agg_timeout)
        //         .aggregate_tickets(channel, prerequisites)
        //         .await
        // } else {
        //     Err(TicketAggregationError::TransportError(
        //         "Ticket aggregation writer not available, the object was not yet initialized".to_string(),
        //     ))
        // }

        Err(TicketAggregationError::TransportError(
            "Ticket aggregation not supported as a session protocol yet".to_string(),
        ))
    }
}
