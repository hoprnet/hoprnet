use hopr_api::chain::ChainReadChannelOperations;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;

use crate::{HoprProtocolError, ResolvedAcknowledgement, UnacknowledgedTicketProcessor};

#[derive(Debug, Clone, smart_default::SmartDefault)]
pub struct HoprTicketProcessorConfig {
    #[default(std::time::Duration::from_secs(30))]
    pub unack_ticket_timeout: std::time::Duration,
    #[default(10_000_000)]
    pub max_unack_tickets: usize,
    pub channels_dst: Hash,
}

pub struct HoprTicketProcessor<R> {
    unacknowledged_tickets: moka::future::Cache<HalfKeyChallenge, UnacknowledgedTicket>,
    provider: R,
    chain_key: ChainKeypair,
    channels_dst: Hash,
}

impl<R> HoprTicketProcessor<R> {
    pub fn new(provider: R, chain_key: ChainKeypair, cfg: HoprTicketProcessorConfig) -> Self {
        Self {
            unacknowledged_tickets: moka::future::Cache::builder()
                .time_to_live(cfg.unack_ticket_timeout)
                .max_capacity(cfg.max_unack_tickets as u64)
                .build(),
            provider,
            chain_key,
            channels_dst: cfg.channels_dst,
        }
    }
}

#[async_trait::async_trait]
impl<R> UnacknowledgedTicketProcessor for HoprTicketProcessor<R>
where
    R: ChainReadChannelOperations + Send + Sync,
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
        .await
    }
}
