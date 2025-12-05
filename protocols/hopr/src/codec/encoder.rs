use hopr_api::chain::*;
use hopr_crypto_packet::prelude::*;
use hopr_crypto_types::{crypto_traits::Randomizable, prelude::*};
use hopr_internal_types::prelude::*;
use hopr_network_types::prelude::*;
use hopr_primitive_types::prelude::*;

use crate::{
    HoprCodecConfig, OutgoingPacket, PacketEncoder, SurbStore, TicketCreationError, TicketTracker,
    errors::HoprProtocolError,
};

/// Default [encoder](PacketEncoder) implementation for HOPR packets.
pub struct HoprEncoder<Chain, S, T> {
    chain_api: Chain,
    surb_store: S,
    tracker: T,
    chain_key: ChainKeypair,
    channels_dst: Hash,
    cfg: HoprCodecConfig,
}

impl<Chain, S, T> HoprEncoder<Chain, S, T> {
    /// Creates a new instance of the encoder.
    pub fn new(
        chain_key: ChainKeypair,
        chain_api: Chain,
        surb_store: S,
        tracker: T,
        channels_dst: Hash,
        cfg: HoprCodecConfig,
    ) -> Self {
        Self {
            chain_api,
            surb_store,
            tracker,
            chain_key,
            channels_dst,
            cfg,
        }
    }
}

impl<Chain, S, T> HoprEncoder<Chain, S, T>
where
    Chain: ChainKeyOperations + ChainReadChannelOperations + ChainValues + Sync,
    S: SurbStore,
    T: TicketTracker + Sync,
{
    async fn encode_packet_internal<D: AsRef<[u8]> + Send + 'static, Sig: Into<PacketSignals> + Send + 'static>(
        &self,
        next_peer: OffchainPublicKey,
        data: D,
        num_hops: usize,
        signals: Sig,
        routing: PacketRouting<ValidatedPath>,
        pseudonym: HoprPseudonym,
    ) -> Result<OutgoingPacket, HoprProtocolError> {
        let next_peer = self
            .chain_api
            .packet_key_to_chain_key(&next_peer)
            .await
            .map_err(|e| HoprProtocolError::ResolverError(e.into()))?
            .ok_or(HoprProtocolError::KeyNotFound)?;

        // Decide whether to create a multi-hop or a zero-hop ticket
        let next_ticket = if num_hops > 1 {
            let channel = self
                .chain_api
                .channel_by_parties(self.chain_key.as_ref(), &next_peer)
                .await
                .map_err(|e| HoprProtocolError::ResolverError(e.into()))?
                .ok_or_else(|| HoprProtocolError::ChannelNotFound(*self.chain_key.as_ref(), next_peer))?;

            let (outgoing_ticket_win_prob, outgoing_ticket_price) = self
                .chain_api
                .outgoing_ticket_values(self.cfg.outgoing_win_prob, self.cfg.outgoing_ticket_price)
                .await
                .map_err(|e| HoprProtocolError::ResolverError(e.into()))?;

            self.tracker
                .create_multihop_ticket(
                    &channel,
                    num_hops as u8,
                    outgoing_ticket_win_prob,
                    outgoing_ticket_price,
                )
                .await
                .map_err(|e| match e {
                    TicketCreationError::OutOfFunds(id, a) => HoprProtocolError::OutOfFunds(id, a),
                    e => HoprProtocolError::TicketTrackerError(e.into()),
                })?
        } else {
            TicketBuilder::zero_hop().counterparty(next_peer)
        };

        // Construct the outgoing packet
        let chain_key = self.chain_key.clone();
        let mapper = self.chain_api.key_id_mapper_ref().clone();
        let domain_separator = self.channels_dst;
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

        let mut transport_payload = Vec::with_capacity(HoprPacket::SIZE);
        transport_payload.extend_from_slice(out.packet.as_ref());
        transport_payload.extend_from_slice(&out.ticket.into_encoded());

        Ok(OutgoingPacket {
            next_hop: out.next_hop,
            ack_challenge: out.ack_challenge,
            data: transport_payload.into_boxed_slice(),
        })
    }
}

#[async_trait::async_trait]
impl<Chain, S, T> PacketEncoder for HoprEncoder<Chain, S, T>
where
    Chain: ChainKeyOperations + ChainReadChannelOperations + ChainValues + Send + Sync,
    S: SurbStore + Send + Sync,
    T: TicketTracker + Send + Sync,
{
    type Error = HoprProtocolError;

    #[tracing::instrument(skip_all, level = "trace")]
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
                    .chain_api
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

        tracing::trace!(len = data.as_ref().len(), "encoding packet");
        self.encode_packet_internal(next_peer, data, num_hops, signals, routing, pseudonym)
            .await
    }

    #[tracing::instrument(skip_all, level = "trace", fields(destination = destination.to_peerid_str()))]
    async fn encode_acknowledgements(
        &self,
        acks: Vec<VerifiedAcknowledgement>,
        destination: &OffchainPublicKey,
    ) -> Result<OutgoingPacket, Self::Error> {
        tracing::trace!(num_acks = acks.len(), "encoding acknowledgements");

        let mut all_acks = Vec::<u8>::with_capacity(size_of::<u16>() + acks.len() * Acknowledgement::SIZE);
        all_acks.extend((acks.len() as u16).to_be_bytes());
        acks.into_iter().for_each(|ack| all_acks.extend(ack.leak().as_ref()));

        self.encode_packet_internal(
            *destination,
            all_acks,
            0,
            None,
            PacketRouting::NoAck(*destination),
            HoprPseudonym::random(),
        )
        .await
    }
}
