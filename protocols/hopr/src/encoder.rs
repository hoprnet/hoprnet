use hopr_api::chain::*;
use hopr_crypto_packet::prelude::*;
use hopr_crypto_types::{crypto_traits::Randomizable, prelude::*};
use hopr_internal_types::prelude::*;
use hopr_network_types::prelude::*;
use hopr_path::Path;
use hopr_primitive_types::prelude::*;

use crate::{OutgoingPacket, PacketEncoder, SurbStore, TicketTracker, errors::HoprProtocolError};

#[derive(Clone, Debug, smart_default::SmartDefault)]
pub struct HoprEncoderConfig {
    pub outgoing_ticket_price: Option<HoprBalance>,
    #[default(Some(WinningProbability::ALWAYS))]
    pub outgoing_win_prob: Option<WinningProbability>,
    pub channels_dst: Hash,
}

pub struct HoprEncoder<R, S, T> {
    provider: R,
    surb_store: S,
    tracker: T,
    chain_key: ChainKeypair,
    cfg: HoprEncoderConfig,
}

impl<R, S, T> HoprEncoder<R, S, T> {
    pub fn new(provider: R, surb_store: S, tracker: T, chain_key: ChainKeypair, cfg: HoprEncoderConfig) -> Self {
        Self {
            provider,
            surb_store,
            tracker,
            chain_key,
            cfg,
        }
    }
}

#[async_trait::async_trait]
impl<R, S, T> PacketEncoder for HoprEncoder<R, S, T>
where
    R: ChainKeyOperations + ChainReadChannelOperations + ChainValues + Send + Sync,
    S: SurbStore + Send + Sync,
    T: TicketTracker + Send + Sync,
{
    type Error = HoprProtocolError;

    async fn encode_packet<D: AsRef<[u8]> + Send + 'static, Sig: Into<PacketSignals> + Send + 'static>(
        &self,
        data: D,
        routing: ResolvedTransportRouting,
        signals: Sig,
    ) -> Result<OutgoingPacket, Self::Error> {
        // Get necessary packet routing values
        let (next_peer, num_hops, pseudonym, routing) = match routing {
            ResolvedTransportRouting::Forward {
                pseudonym,
                forward_path,
                return_paths,
            } => (
                forward_path[0],
                forward_path.num_hops(),
                pseudonym,
                PacketRouting::ForwardPath {
                    forward_path,
                    return_paths,
                },
            ),
            ResolvedTransportRouting::Return(sender_id, surb) => {
                let next = self
                    .provider
                    .key_id_mapper_ref()
                    .map_id_to_public(&surb.first_relayer)
                    .ok_or(HoprProtocolError::KeyNotFound)?;

                (
                    next,
                    surb.additional_data_receiver.proof_of_relay_values().chain_length() as usize,
                    sender_id.pseudonym(),
                    PacketRouting::Surb(sender_id.surb_id(), surb),
                )
            }
        };

        let next_peer = self
            .provider
            .packet_key_to_chain_key(&next_peer)
            .await
            .map_err(|e| HoprProtocolError::ResolverError(e.into()))?
            .ok_or(HoprProtocolError::KeyNotFound)?;

        // Decide whether to create a multi-hop or a zero-hop ticket
        let next_ticket = if num_hops > 1 {
            let channel = self
                .provider
                .channel_by_parties(self.chain_key.as_ref(), &next_peer)
                .await
                .map_err(|e| HoprProtocolError::ResolverError(e.into()))?
                .ok_or_else(|| HoprProtocolError::ChannelNotFound(*self.chain_key.as_ref(), next_peer))?;

            let (outgoing_ticket_win_prob, outgoing_ticket_price) = self
                .provider
                .outgoing_ticket_values(self.cfg.outgoing_win_prob, self.cfg.outgoing_ticket_price)
                .await
                .map_err(|e| HoprProtocolError::ResolverError(e.into()))?;

            self.tracker.create_multihop_ticket(
                &channel,
                num_hops as u8,
                outgoing_ticket_win_prob,
                outgoing_ticket_price,
            )?
        } else {
            TicketBuilder::zero_hop().direction(self.chain_key.as_ref(), &next_peer)
        };

        // Construct the outgoing packet
        let chain_key = self.chain_key.clone();
        let mapper = self.provider.key_id_mapper_ref().clone();
        let domain_separator = self.cfg.channels_dst;
        let (packet, openers) = hopr_parallelize::cpu::spawn_fifo_blocking(move || {
            HoprPacket::into_outgoing(
                data.as_ref(),
                &pseudonym,
                routing,
                &chain_key,
                next_ticket,
                &mapper,
                &domain_separator,
                signals,
            )
        })
        .await?;

        // Store the reply openers under the given SenderId
        // This is a no-op for reply packets
        openers.into_iter().for_each(|(surb_id, opener)| {
            self.surb_store
                .insert_reply_opener(HoprSenderId::from_pseudonym_and_id(&pseudonym, surb_id), opener);
        });

        let out = packet.try_as_outgoing().ok_or(HoprProtocolError::InvalidState(
            "cannot send out packet that is not outgoing",
        ))?;

        // self.unacked_tickets
        //    .insert(out.ack_challenge, PendingAcknowledgement::WaitingAsSender)
        //    .await;

        let mut transport_payload = Vec::with_capacity(HoprPacket::SIZE);
        transport_payload.extend_from_slice(out.packet.as_ref());
        transport_payload.extend_from_slice(&out.ticket.into_encoded());

        Ok(OutgoingPacket {
            next_hop: out.next_hop,
            ack_challenge: out.ack_challenge,
            data: transport_payload.into_boxed_slice(),
        })
    }

    async fn encode_acknowledgement(
        &self,
        ack: VerifiedAcknowledgement,
        peer: &OffchainPublicKey,
    ) -> Result<OutgoingPacket, Self::Error> {
        let chain_key = self
            .provider
            .packet_key_to_chain_key(peer)
            .await
            .map_err(|e| HoprProtocolError::ResolverError(e.into()))?
            .ok_or(HoprProtocolError::KeyNotFound)?;

        self.encode_packet(
            ack.leak(),
            ResolvedTransportRouting::Forward {
                pseudonym: HoprPseudonym::random(),
                forward_path: ValidatedPath::direct(*peer, chain_key),
                return_paths: Vec::with_capacity(0),
            },
            None,
        )
        .await
    }
}
