use crate::strategy::SingularStrategy;
use async_std::sync::{Mutex, RwLock};
use core_ethereum_actions::errors::CoreEthereumActionsError::ChannelDoesNotExist;
use core_ethereum_actions::redeem::redeem_tickets_in_channel;
use core_ethereum_actions::transaction_queue::TransactionSender;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_protocol::ticket_aggregation::processor::BasicTicketAggregationActions;
use core_types::acknowledgement::{AcknowledgedTicket, AcknowledgedTicketStatus};
use core_types::channels::Ticket;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::time::Duration;
use utils_log::{debug, error, info};
use validator::Validate;

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::spawn_local;
use async_trait::async_trait;

#[cfg(all(feature = "wasm", not(test)))]
use wasm_bindgen_futures::spawn_local;

/// Configuration object for the `AggregatingStrategy`
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Validate, Serialize, Deserialize)]
pub struct AggregatingStrategyConfig {
    /// Number of acknowledged tickets in a channel that has to be exceeded to trigger
    /// the ticket aggregation in that channel.
    /// Default is 100.
    pub aggregation_threshold: u32,

    /// Maximum time to wait for the ticket aggregation to complete.
    /// This does not affect the runtime of the strategy `on_acknowledged_ticket` event processing.
    /// Default is 1 minute.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub aggregation_timeout: Duration,

    /// Indicates whether to also perform ticket redemption after the ticket aggregation
    /// in a channel was successful.
    /// Default is `true`.
    pub redeem_after_aggregation: bool,
}

impl Default for AggregatingStrategyConfig {
    fn default() -> Self {
        Self {
            aggregation_threshold: 100,
            aggregation_timeout: Duration::from_secs(60),
            redeem_after_aggregation: true,
        }
    }
}

/// Represents a strategy that starts aggregating tickets in a certain
/// channel, once the amount of acknowledged tickets in that channel goes
/// above the given threshold.
/// Optionally, the strategy can also redeem the aggregated ticket, if the aggregation
/// was successful.
pub struct AggregatingStrategy<Db: HoprCoreEthereumDbActions> {
    db: Arc<RwLock<Db>>,
    tx_sender: TransactionSender,
    ticket_aggregator: Arc<Mutex<BasicTicketAggregationActions<Result<Ticket, String>>>>,
    cfg: AggregatingStrategyConfig,
}

impl<Db: HoprCoreEthereumDbActions> Display for AggregatingStrategy<Db> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "aggregating")
    }
}

impl<Db: HoprCoreEthereumDbActions> AggregatingStrategy<Db> {
    pub fn new(
        cfg: AggregatingStrategyConfig,
        db: Arc<RwLock<Db>>,
        tx_sender: TransactionSender,
        ticket_aggregator: BasicTicketAggregationActions<Result<Ticket, String>>,
    ) -> Self {
        Self {
            cfg,
            db,
            tx_sender,
            ticket_aggregator: Arc::new(Mutex::new(ticket_aggregator)),
        }
    }
}

#[async_trait(? Send)]
impl<Db: HoprCoreEthereumDbActions + 'static> SingularStrategy for AggregatingStrategy<Db> {
    async fn on_acknowledged_ticket(&self, ack: &AcknowledgedTicket) -> crate::errors::Result<()> {
        let channel_id = ack.ticket.channel_id;
        let maybe_channel = self.db.read().await.get_channel(&channel_id).await?;

        if let Some(channel) = maybe_channel {
            let acks_in_channel = self
                .db
                .read()
                .await
                .get_acknowledged_tickets(Some(channel.clone()))
                .await?
                .iter()
                .filter(|ack| ack.status == AcknowledgedTicketStatus::Untouched)
                .count() as u32;

            if acks_in_channel >= self.cfg.aggregation_threshold {
                info!("{self} strategy: channel has {acks_in_channel} acknowledged tickets, going to aggregate them");

                let db_clone = self.db.clone();
                let cfg_clone = self.cfg.clone();
                let aggregator_clone = self.ticket_aggregator.clone();
                let tx_sender_clone = self.tx_sender.clone();
                let strat_name = self.to_string();

                spawn_local(async move {
                    let agg = aggregator_clone.lock().await.aggregate_tickets(&channel_id);
                    match agg {
                        Ok(mut awaiter) => match awaiter.consume_and_wait(cfg_clone.aggregation_timeout).await {
                            Ok(_) => {
                                if cfg_clone.redeem_after_aggregation {
                                    if let Err(e) = redeem_tickets_in_channel(db_clone, &channel, tx_sender_clone).await
                                    {
                                        error!("{strat_name} strategy: failed to send tickets for redemption: {e}");
                                    } else {
                                        info!("{strat_name} strategy: successfully aggregated tickets & sent for redeeming in {channel}");
                                    }
                                } else {
                                    info!("{strat_name} strategy: successfully aggregated tickets in {channel}");
                                }
                            }
                            Err(e) => error!("{strat_name} strategy: could not aggregate tickets: {e}"),
                        },
                        Err(e) => error!("{strat_name} strategy: could initiate ticket aggregation: {e}"),
                    }
                });
            } else {
                debug!(
                    "{self} strategy: {channel} has {acks_in_channel} < {} ack tickets, not aggregating yet",
                    self.cfg.aggregation_threshold
                );
            }
            Ok(())
        } else {
            error!("{self} strategy: encountered {ack} in a non-existing channel!");
            Err(ChannelDoesNotExist.into())
        }
    }
}

#[cfg(test)]
mod tests {}
