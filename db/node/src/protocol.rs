use std::ops::{Mul, Sub};

use async_trait::async_trait;
use hopr_api::{
    chain::{ChainKeyOperations, ChainReadChannelOperations, ChainValues},
    db::*,
};
use hopr_crypto_packet::{errors::PacketError, prelude::*};
use hopr_crypto_types::{crypto_traits::Randomizable, prelude::*};
use hopr_internal_types::prelude::*;
use hopr_network_types::prelude::{ResolvedTransportRouting, SurbMatcher};
use hopr_parallelize::cpu::spawn_fifo_blocking;
use hopr_path::{Path, ValidatedPath};
use hopr_primitive_types::prelude::*;
use tracing::{instrument, trace, warn};

use crate::{cache::SurbRingBuffer, db::HoprNodeDb, errors::NodeDbError};


impl HoprNodeDb {
    
}

#[async_trait]
impl HoprDbProtocolOperations for HoprNodeDb {
    type Error = NodeDbError;

    #[instrument(level = "trace", skip(self, ack, resolver), err(Debug), ret)]
    async fn handle_acknowledgement<R>(&self, ack: VerifiedAcknowledgement, resolver: &R) -> Result<(), NodeDbError>
    where
        R: ChainReadChannelOperations + ChainValues + Send + Sync,
    {

    }

    #[tracing::instrument(level = "trace", skip(self, matcher), err)]
    async fn find_surb(&self, matcher: SurbMatcher) -> Result<FoundSurb, NodeDbError> {
        let pseudonym = matcher.pseudonym();
        let surbs_for_pseudonym = self
            .caches
            .surbs_per_pseudonym
            .get(&pseudonym)
            .await
            .ok_or(NodeDbError::NoSurbAvailable("pseudonym not found".into()))?;

        match matcher {
            SurbMatcher::Pseudonym(_) => Ok(surbs_for_pseudonym.pop_one().map(|popped_surb| FoundSurb {
                sender_id: HoprSenderId::from_pseudonym_and_id(&pseudonym, popped_surb.id),
                surb: popped_surb.surb,
                remaining: popped_surb.remaining,
            })?),
            // The following code intentionally only checks the first SURB in the ring buffer
            // and does not search the entire RB.
            // This is because the exact match use-case is suited only for situations
            // when there is a single SURB in the RB.
            SurbMatcher::Exact(id) => Ok(surbs_for_pseudonym
                .pop_one_if_has_id(&id.surb_id())
                .map(|popped_surb| FoundSurb {
                    sender_id: HoprSenderId::from_pseudonym_and_id(&pseudonym, popped_surb.id),
                    surb: popped_surb.surb,
                    remaining: popped_surb.remaining, // = likely 0
                })?),
        }
    }

    #[inline]
    fn get_surb_config(&self) -> SurbCacheConfig {
        SurbCacheConfig {
            rb_capacity: self.cfg.surb_ring_buffer_size,
            distress_threshold: self.cfg.surb_distress_threshold,
        }
    }

    #[tracing::instrument(level = "trace", skip(self, data, resolver))]
    async fn to_send_no_ack<R>(
        &self,
        data: Box<[u8]>,
        destination: OffchainPublicKey,
        resolver: &R,
    ) -> Result<OutgoingPacket, NodeDbError>
    where
        R: ChainKeyOperations + ChainValues + Send + Sync,
    {

    }

    #[tracing::instrument(level = "trace", skip(self, data, routing, resolver))]
    async fn to_send<R>(
        &self,
        data: Box<[u8]>,
        routing: ResolvedTransportRouting,
        outgoing_ticket_win_prob: Option<WinningProbability>,
        outgoing_ticket_price: Option<HoprBalance>,
        signals: PacketSignals,
        resolver: &R,
    ) -> Result<OutgoingPacket, NodeDbError>
    where
        R: ChainReadChannelOperations + ChainKeyOperations + ChainValues + Send + Sync,
    {

    }

    #[tracing::instrument(level = "trace", skip_all, fields(sender = %sender), err)]
    async fn from_recv<R>(
        &self,
        data: Box<[u8]>,
        pkt_keypair: &OffchainKeypair,
        sender: OffchainPublicKey,
        outgoing_ticket_win_prob: Option<WinningProbability>,
        outgoing_ticket_price: Option<HoprBalance>,
        resolver: &R,
    ) -> Result<IncomingPacket, IncomingPacketError<NodeDbError>>
    where
        R: ChainReadChannelOperations + ChainKeyOperations + ChainValues + Send + Sync,
    {
        
    }
}



impl HoprNodeDb {
   
}
