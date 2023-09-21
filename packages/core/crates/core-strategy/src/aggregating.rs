use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::time::Duration;
use async_std::sync::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};
use validator::Validate;
use core_ethereum_actions::errors::CoreEthereumActionsError::ChannelDoesNotExist;
use core_ethereum_actions::redeem::redeem_tickets_in_channel;
use core_ethereum_actions::transaction_queue::TransactionSender;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_protocol::ticket_aggregation::processor::BasicTicketAggregationActions;
use core_types::acknowledgement::{AcknowledgedTicket, AcknowledgedTicketStatus};
use core_types::channels::Ticket;
use utils_log::{error, info};
use crate::strategy::SingularStrategy;

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::spawn_local;
use async_trait::async_trait;

#[cfg(all(feature = "wasm", not(test)))]
use wasm_bindgen_futures::spawn_local;

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Validate, Serialize, Deserialize)]
pub struct AggregatingStrategyConfig {
    pub aggregation_threshold: u32,
    #[serde_as(as = "DurationSeconds<u64>")]
    pub aggregation_timeout: Duration,
    pub redeem: bool,
}

impl Default for AggregatingStrategyConfig {
    fn default() -> Self {
        Self {
            aggregation_threshold: 100,
            aggregation_timeout: Duration::from_secs(60),
            redeem: true
        }
    }
}

pub struct AggregatingStrategy<Db: HoprCoreEthereumDbActions> {
    db: Arc<RwLock<Db>>,
    tx_sender: TransactionSender,
    ticket_aggregator: Arc<Mutex<BasicTicketAggregationActions<Result<Ticket, String>>>>,
    cfg: AggregatingStrategyConfig
}

impl<Db: HoprCoreEthereumDbActions> Display for AggregatingStrategy<Db> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "aggregating")
    }
}

impl<Db: HoprCoreEthereumDbActions> AggregatingStrategy<Db> {
    pub fn new(cfg: AggregatingStrategyConfig, db: Arc<RwLock<Db>>, tx_sender: TransactionSender, ticket_aggregator: BasicTicketAggregationActions<Result<Ticket, String>>) -> Self {
        Self { cfg, db, tx_sender, ticket_aggregator: Arc::new(Mutex::new(ticket_aggregator))}
    }
}

#[async_trait(? Send)]
impl<Db: HoprCoreEthereumDbActions + 'static> SingularStrategy for AggregatingStrategy<Db> {
    async fn on_acknowledged_ticket(&self, ack: &AcknowledgedTicket) -> crate::errors::Result<()> {
        let channel_id = ack.ticket.channel_id;
        let maybe_channel = self.db.read().await.get_channel(&channel_id).await?;

        if let Some(channel) = maybe_channel {
            let acks_in_channel = self.db.read()
                .await
                .get_acknowledged_tickets(Some(channel.clone()))
                .await?
                .iter().filter(|ack| ack.status == AcknowledgedTicketStatus::Untouched)
                .count() as u32;

            if acks_in_channel >= self.cfg.aggregation_threshold {
                info!("channel has {acks_in_channel} acknowledged tickets, strategy will aggregate them");

                let db_clone = self.db.clone();
                let cfg_clone = self.cfg.clone();
                let aggregator_clone = self.ticket_aggregator.clone();
                let tx_sender_clone = self.tx_sender.clone();

                spawn_local(async move {
                    let agg = aggregator_clone.lock().await.aggregate_tickets(&channel_id);
                    match agg {
                        Ok(mut awaiter) => {
                            if let Err(e) = awaiter.consume_and_wait(cfg_clone.aggregation_timeout).await {
                                error!("strategy could not aggregate tickets: {e}");
                            } else if cfg_clone.redeem {
                                let _ = redeem_tickets_in_channel(db_clone, &channel, tx_sender_clone).await;
                            }
                        }
                        Err(e) => error!("strategy could not aggregate tickets: {e}")
                    }
                });
            }
            Ok(())
        } else {
            Err(ChannelDoesNotExist.into())
        }
    }
}