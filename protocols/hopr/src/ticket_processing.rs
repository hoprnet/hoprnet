use std::{
    ops::Add,
    sync::{Arc, atomic::AtomicU64},
};

use hopr_api::{
    chain::ChainReadChannelOperations,
    db::{HoprDbTicketOperations, TicketSelector},
};
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::balance::HoprBalance;

use crate::{HoprProtocolError, ResolvedAcknowledgement, TicketTracker, UnacknowledgedTicketProcessor};

#[derive(Debug, Clone, smart_default::SmartDefault)]
pub struct HoprTicketProcessorConfig {
    #[default(std::time::Duration::from_secs(30))]
    pub unack_ticket_timeout: std::time::Duration,
    #[default(10_000_000)]
    pub max_unack_tickets: usize,
    pub channels_dst: Hash,
}

pub struct HoprTicketProcessor<Db, R> {
    unacknowledged_tickets: moka::future::Cache<HalfKeyChallenge, UnacknowledgedTicket>,
    index_tracker: TicketIndexTracker<Db>,
    provider: R,
    chain_key: ChainKeypair,
    channels_dst: Hash,
}

impl<Db, R> HoprTicketProcessor<Db, R> {
    pub fn new(
        provider: R,
        index_tracker: TicketIndexTracker<Db>,
        chain_key: ChainKeypair,
        cfg: HoprTicketProcessorConfig,
    ) -> Self {
        Self {
            unacknowledged_tickets: moka::future::Cache::builder()
                .time_to_live(cfg.unack_ticket_timeout)
                .max_capacity(cfg.max_unack_tickets as u64)
                .build(),
            provider,
            index_tracker,
            chain_key,
            channels_dst: cfg.channels_dst,
        }
    }
}

#[async_trait::async_trait]
impl<Db, R> UnacknowledgedTicketProcessor for HoprTicketProcessor<Db, R>
where
    R: ChainReadChannelOperations + Send + Sync,
    Db: Send + Sync,
{
    type Error = HoprProtocolError;

    async fn insert_unacknowledged_ticket(
        &self,
        challenge: HalfKeyChallenge,
        ticket: UnacknowledgedTicket,
    ) -> Result<(), Self::Error> {
        self.unacknowledged_tickets.insert(challenge, ticket).await;
        Ok(())
    }

    async fn acknowledge_ticket(
        &self,
        peer: OffchainPublicKey,
        ack: Acknowledgement,
    ) -> Result<ResolvedAcknowledgement, Self::Error> {
        let (half_key, challenge) = hopr_parallelize::cpu::spawn_fifo_blocking(move || {
            ack.verify(&peer)
                .and_then(|verified| Ok((*verified.ack_key_share(), verified.ack_key_share().to_challenge()?)))
        })
        .await?;

        let unacknowledged = self
            .unacknowledged_tickets
            .remove(&challenge)
            .await
            .ok_or_else(|| HoprProtocolError::UnacknowledgedTicketNotFound(challenge))?;

        // Issuer's channel must have an epoch matching with the unacknowledged ticket
        let issuer_channel = self
            .provider
            .channel_by_parties(unacknowledged.ticket.verified_issuer(), self.chain_key.as_ref())
            .await
            .map_err(|e| HoprProtocolError::ResolverError(e.into()))?
            .filter(|c| c.channel_epoch.as_u32() == unacknowledged.verified_ticket().channel_epoch)
            .ok_or(HoprProtocolError::ChannelNotFound(
                *unacknowledged.ticket.verified_issuer(),
                *self.chain_key.as_ref(),
            ))?;

        let domain_separator = self.channels_dst;
        let chain_key = self.chain_key.clone();
        let channel_id = *issuer_channel.get_id();
        let res = hopr_parallelize::cpu::spawn_fifo_blocking(move || {
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
                    Ok(ResolvedAcknowledgement::RelayingWin(Box::new(redeemable)))
                }
                Err(CoreTypesError::TicketNotWinning) => {
                    tracing::trace!(%issuer_channel, "found losing ticket");
                    Ok(ResolvedAcknowledgement::RelayingLoss(channel_id))
                }
                Err(error) => {
                    tracing::error!(%error, %issuer_channel, "error when acknowledging ticket");
                    Ok(ResolvedAcknowledgement::RelayingLoss(channel_id))
                }
            }
        })
        .await;

        // Account the newly received winning ticket into unrealized value for that channel
        if let Ok(ResolvedAcknowledgement::RelayingWin(redeemable)) = &res {
            let ticket_value = redeemable.verified_ticket().amount;
            self.index_tracker
                .unrealized_value
                .entry((
                    redeemable.verified_ticket().channel_id,
                    redeemable.verified_ticket().channel_epoch,
                ))
                .and_compute_with(|maybe_value| match maybe_value {
                    None => futures::future::ready(moka::ops::compute::Op::Put(ticket_value)),
                    Some(value) => futures::future::ready(moka::ops::compute::Op::Put(value.value().add(ticket_value))),
                })
                .await;
        }

        res
    }
}

pub struct TicketIndexTracker<Db> {
    db: Arc<Db>,
    ticket_index: moka::future::Cache<(ChannelId, u32), Arc<AtomicU64>>,
    // key is (channel_id, channel_epoch) to ensure calculation of unrealized value does not
    // include tickets from other epochs
    unrealized_value: moka::future::Cache<(ChannelId, u32), HoprBalance>,
}

impl<Db: HoprDbTicketOperations> TicketIndexTracker<Db> {
    pub fn new(db: Db, outgoing_index_cache_retention: std::time::Duration) -> Self {
        Self {
            db: Arc::new(db),
            ticket_index: moka::future::Cache::builder()
                .time_to_idle(outgoing_index_cache_retention)
                .max_capacity(10_000)
                .build(),
            unrealized_value: moka::future::Cache::builder()
                .time_to_idle(std::time::Duration::from_mins(30))
                .max_capacity(10_000)
                .build(),
        }
    }

    pub async fn sync_indices_to_db(&self) {
        // This iteration does not alter the popularity estimator of the cache
        // and therefore still allows the unused entries to expire
        for (channel_key, out_idx) in self.ticket_index.iter() {
            if let Err(error) = self
                .db
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
    }
}

impl<Db> Clone for TicketIndexTracker<Db> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            ticket_index: self.ticket_index.clone(),
            unrealized_value: self.unrealized_value.clone(),
        }
    }
}

#[async_trait::async_trait]
impl<Db: HoprDbTicketOperations + Send + Sync> TicketTracker for TicketIndexTracker<Db> {
    type Error = Arc<Db::Error>;

    async fn next_outgoing_ticket_index(&self, channel_id: &ChannelId, epoch: u32) -> Result<u64, Self::Error> {
        let channel_id = *channel_id;
        let db = self.db.clone();
        self.ticket_index
            .try_get_with((channel_id, epoch), async move {
                db.get_or_create_outgoing_ticket_index(&channel_id, epoch)
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
        let db = self.db.clone();
        let channel_id = *channel_id;
        self.unrealized_value
            .try_get_with((channel_id, epoch), async move {
                db.get_tickets_value(TicketSelector::new(channel_id, epoch))
                    .await
                    .map(|v| v.1)
            })
            .await
    }
}
