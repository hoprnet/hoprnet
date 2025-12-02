use std::sync::{Arc, atomic::AtomicU64};

use futures::StreamExt;
use hopr_api::{chain::ChainReadChannelOperations, db::HoprDbTicketOperations};
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::balance::HoprBalance;
use validator::ValidationError;

use crate::{HoprProtocolError, ResolvedAcknowledgement, TicketTracker, UnacknowledgedTicketProcessor};

const MIN_UNACK_TICKET_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
fn validate_unack_ticket_timeout(timeout: &std::time::Duration) -> Result<(), ValidationError> {
    if timeout < &MIN_UNACK_TICKET_TIMEOUT {
        Err(ValidationError::new("unack_ticket_timeout too low"))
    } else {
        Ok(())
    }
}

const MIN_OUTGOING_INDEX_RETENTION: std::time::Duration = std::time::Duration::from_secs(10);

fn validate_outgoing_index_retention(retention: &std::time::Duration) -> Result<(), ValidationError> {
    if retention < &MIN_OUTGOING_INDEX_RETENTION {
        Err(ValidationError::new("outgoing_index_cache_retention too low"))
    } else {
        Ok(())
    }
}

fn default_outgoing_index_retention() -> std::time::Duration {
    std::time::Duration::from_secs(10)
}

fn default_unack_ticket_timeout() -> std::time::Duration {
    std::time::Duration::from_secs(30)
}

fn default_max_unack_tickets() -> usize {
    10_000_000
}

/// Configuration for the HOPR ticket processor within the packet pipeline.

#[derive(Debug, Clone, Copy, smart_default::SmartDefault, PartialEq, validator::Validate)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(deny_unknown_fields)
)]
pub struct HoprTicketProcessorConfig {
    /// Time after which an unacknowledged ticket is considered stale and is removed from the cache.
    ///
    /// If the counterparty does not send an acknowledgement within this period, the ticket is lost forever.
    ///
    /// Default is 30 seconds.
    #[default(default_unack_ticket_timeout())]
    #[validate(custom(function = "validate_unack_ticket_timeout"))]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_unack_ticket_timeout", with = "humantime_serde")
    )]
    pub unack_ticket_timeout: std::time::Duration,
    /// Maximum number of unacknowledged tickets that can be stored in the cache at any given time.
    ///
    /// When more tickets are received, the oldest ones are discarded and lost forever.
    ///
    /// Default is 10 000 000.
    #[default(default_max_unack_tickets())]
    #[validate(range(min = 100))]
    #[cfg_attr(feature = "serde", serde(default = "default_max_unack_tickets"))]
    pub max_unack_tickets: usize,
    /// Period for which the outgoing ticket index is cached in memory for each channel.
    ///
    /// Default is 10 seconds.
    #[default(default_outgoing_index_retention())]
    #[validate(custom(function = "validate_outgoing_index_retention"))]
    #[cfg_attr(
        feature = "serde",
        serde(default = "default_outgoing_index_retention", with = "humantime_serde")
    )]
    pub outgoing_index_cache_retention: std::time::Duration,
}

/// HOPR-specific implementation of [`UnacknowledgedTicketProcessor`] and [`TicketTracker`].
#[derive(Clone)]
pub struct HoprTicketProcessor<Chain, Db> {
    unacknowledged_tickets:
        moka::future::Cache<OffchainPublicKey, moka::future::Cache<HalfKeyChallenge, UnacknowledgedTicket>>,
    out_ticket_index: moka::future::Cache<(ChannelId, u32), Arc<AtomicU64>>,
    db: Db,
    chain_api: Chain,
    chain_key: ChainKeypair,
    channels_dst: Hash,
    cfg: HoprTicketProcessorConfig,
}

impl<Chain, Db> HoprTicketProcessor<Chain, Db> {
    /// Creates a new instance of the HOPR ticket processor.
    pub fn new(
        chain_api: Chain,
        db: Db,
        chain_key: ChainKeypair,
        channels_dst: Hash,
        cfg: HoprTicketProcessorConfig,
    ) -> Self {
        Self {
            out_ticket_index: moka::future::Cache::builder()
                .time_to_idle(cfg.outgoing_index_cache_retention)
                .max_capacity(10_000)
                .build(),
            unacknowledged_tickets: moka::future::Cache::builder()
                .time_to_idle(cfg.unack_ticket_timeout * 2)
                .max_capacity(100_000)
                .build(),
            chain_api,
            db,
            chain_key,
            channels_dst,
            cfg,
        }
    }
}

impl<Chain, Db> HoprTicketProcessor<Chain, Db>
where
    Db: HoprDbTicketOperations + Clone + Send + 'static,
{
    /// Task that performs periodic synchronization of the outgoing ticket index cache
    /// to the underlying database.
    ///
    /// If this task is not started, the outgoing ticket indices will not survive a node
    /// restart and will result in invalid tickets received by the counterparty.
    pub fn outgoing_index_sync_task(
        &self,
        reg: futures::future::AbortRegistration,
    ) -> impl Future<Output = ()> + use<Db, Chain> {
        let index_save_stream = futures::stream::Abortable::new(
            futures_time::stream::interval(futures_time::time::Duration::from(
                self.cfg.outgoing_index_cache_retention.div_f32(2.0),
            )),
            reg,
        );

        let db = self.db.clone();
        let out_ticket_index = self.out_ticket_index.clone();

        index_save_stream
            .for_each(move |_| {
                let db = db.clone();
                let out_ticket_index = out_ticket_index.clone();
                async move {
                    // This iteration does not alter the popularity estimator of the cache
                    // and therefore still allows the unused entries to expire
                    for (channel_key, out_idx) in out_ticket_index.iter() {
                        if let Err(error) = db
                            .update_outgoing_ticket_index(
                                &channel_key.0,
                                channel_key.1,
                                out_idx.load(std::sync::atomic::Ordering::SeqCst),
                            )
                            .await
                        {
                            tracing::error!(%error, channel_id = %channel_key.0, epoch = channel_key.1, "failed to sync outgoing ticket index to db");
                        }
                    }
                    tracing::trace!("synced outgoing ticket indices to db");
                }
            })
    }
}

#[async_trait::async_trait]
impl<Chain, Db> UnacknowledgedTicketProcessor for HoprTicketProcessor<Chain, Db>
where
    Chain: ChainReadChannelOperations + Send + Sync,
    Db: Send + Sync,
{
    type Error = HoprProtocolError;

    async fn insert_unacknowledged_ticket(
        &self,
        next_hop: &OffchainPublicKey,
        challenge: HalfKeyChallenge,
        ticket: UnacknowledgedTicket,
    ) -> Result<(), Self::Error> {
        tracing::trace!(%ticket, "received unacknowledged ticket");
        self.unacknowledged_tickets
            .get_with_by_ref(next_hop, async {
                moka::future::Cache::builder()
                    .time_to_live(self.cfg.unack_ticket_timeout)
                    .max_capacity(self.cfg.max_unack_tickets as u64)
                    .build()
            })
            .await
            .insert(challenge, ticket)
            .await;

        Ok(())
    }

    async fn acknowledge_ticket(
        &self,
        peer: OffchainPublicKey,
        ack: Acknowledgement,
    ) -> Result<Option<ResolvedAcknowledgement>, Self::Error> {
        // Check if we're even expecting an acknowledgement from this peer
        let Some(awaiting_ack_from_peer) = self.unacknowledged_tickets.get(&peer).await else {
            tracing::trace!(%peer, "not awaiting any acknowledgement from peer");
            return Ok(None);
        };

        // If we do, verify the acknowledgement signature and extract the challenge
        let (half_key, challenge) = hopr_parallelize::cpu::spawn_fifo_blocking(move || {
            ack.verify(&peer)
                .and_then(|verified| Ok((*verified.ack_key_share(), verified.ack_key_share().to_challenge()?)))
        })
        .await?;

        // Check if we have a ticket to be acknowledged by this peer
        let unacknowledged = awaiting_ack_from_peer
            .remove(&challenge)
            .await
            .ok_or_else(|| HoprProtocolError::UnacknowledgedTicketNotFound(challenge))?;

        // Issuer's channel must have an epoch matching with the unacknowledged ticket
        let issuer_channel = self
            .chain_api
            .channel_by_parties(unacknowledged.ticket.verified_issuer(), self.chain_key.as_ref())
            .await
            .map_err(|e| HoprProtocolError::ResolverError(e.into()))?
            .filter(|c| c.channel_epoch == unacknowledged.verified_ticket().channel_epoch)
            .ok_or(HoprProtocolError::ChannelNotFound(
                *unacknowledged.ticket.verified_issuer(),
                *self.chain_key.as_ref(),
            ))?;

        let domain_separator = self.channels_dst;
        let chain_key = self.chain_key.clone();
        let channel_id = *issuer_channel.get_id();
        hopr_parallelize::cpu::spawn_fifo_blocking(move || {
            // This explicitly checks whether the acknowledgement
            // solves the challenge on the ticket. It must be done before we
            // check that the ticket is winning, which is a lengthy operation
            // and should not be done for bogus unacknowledged tickets
            let ack_ticket = unacknowledged.acknowledge(&half_key)?;

            // This operation checks if the ticket is winning, and if it is, it
            // turns it into a redeemable ticket.
            match ack_ticket.into_redeemable(&chain_key, &domain_separator) {
                Ok(redeemable) => {
                    tracing::debug!(%issuer_channel, "found winning ticket");
                    Ok(Some(ResolvedAcknowledgement::RelayingWin(Box::new(redeemable))))
                }
                Err(CoreTypesError::TicketNotWinning) => {
                    tracing::trace!(%issuer_channel, "found losing ticket");
                    Ok(Some(ResolvedAcknowledgement::RelayingLoss(channel_id)))
                }
                Err(error) => {
                    tracing::error!(%error, %issuer_channel, "error when acknowledging ticket");
                    Ok(Some(ResolvedAcknowledgement::RelayingLoss(channel_id)))
                }
            }
        })
        .await
    }
}

#[async_trait::async_trait]
impl<Chain, Db> TicketTracker for HoprTicketProcessor<Chain, Db>
where
    Chain: Send + Sync,
    Db: HoprDbTicketOperations + Clone + Send + Sync + 'static,
{
    type Error = Arc<Db::Error>;

    async fn next_outgoing_ticket_index(&self, channel_id: &ChannelId, epoch: u32) -> Result<u64, Self::Error> {
        let channel_id = *channel_id;
        self.out_ticket_index
            .try_get_with((channel_id, epoch), async {
                self.db
                    .get_or_create_outgoing_ticket_index(&channel_id, epoch)
                    .await
                    .map(|maybe_idx| Arc::new(AtomicU64::new(maybe_idx.unwrap_or_default())))
            })
            .await
            .map(|idx| idx.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
    }

    async fn incoming_channel_unrealized_balance(
        &self,
        channel_id: &ChannelId,
        epoch: u32,
    ) -> Result<HoprBalance, Self::Error> {
        // This value cannot be cached here and must be cached in the DB
        // because the cache invalidation logic can be only done from within the DB.
        self.db.get_tickets_value(channel_id, epoch).await.map_err(Into::into)
    }
}
