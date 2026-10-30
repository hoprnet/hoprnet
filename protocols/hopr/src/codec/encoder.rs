use std::collections::HashMap;

use bytes::{BufMut, BytesMut};
use hopr_api::{
    chain::*,
    types::{
        crypto::{crypto_traits::Randomizable, prelude::*},
        internal::prelude::*,
        primitive::prelude::*,
    },
};
use hopr_crypto_packet::prelude::*;
use hopr_protocol_pix::EntryShareGenerator;

use crate::{HoprCodecConfig, OutgoingPacket, PacketEncoder, SurbStore, errors::HoprProtocolError};

/// Maximum number of acknowledgements that can be packed into a single HOPR packet.
///
/// Currently, the [`HoprPacket::PAYLOAD_SIZE`] minus 16-bit acknowledgement batch size counter
/// divided by [`Acknowledgement::SIZE`].
pub const MAX_ACKNOWLEDGEMENTS_BATCH_SIZE: usize =
    (HoprPacket::PAYLOAD_SIZE - size_of::<u16>()) / Acknowledgement::SIZE;

/// Maximum number of decompressed packet keys kept by [`CachingKeyExpander`].
///
/// A sender only ever expands the keys of the relays on the paths it currently uses, which is a
/// far smaller set than the peers it knows about.
const EXPANDED_KEY_CACHE_CAPACITY: usize = 4_096;

/// A [`KeyExpander`] that memoizes the decompressed form of a packet key.
///
/// Deriving shared secrets for an outgoing packet needs the expanded key of every hop, on both the
/// forward path and each SURB return path - up to twelve per packet. Decompression is a field
/// exponentiation, so paying it per packet would be a measurable share of packet construction,
/// whereas the set of relays a sender uses turns over slowly.
///
/// This memoizes a pure function of public data, so it deliberately avoids an eviction-tracking
/// cache: a plain map behind an `RwLock` costs nothing to construct and a read is a single hash,
/// where a managed cache would charge per-entry bookkeeping on the packet path.
#[derive(Debug, Default)]
pub struct CachingKeyExpander(parking_lot::RwLock<HashMap<OffchainPublicKey, ExpandedOffchainPublicKey>>);

impl KeyExpander for CachingKeyExpander {
    fn expand(&self, key: &OffchainPublicKey) -> hopr_api::types::crypto::errors::Result<ExpandedOffchainPublicKey> {
        if let Some(expanded) = self.0.read().get(key) {
            return Ok(expanded.clone());
        }

        // Two senders racing on the same missing key will both decompress it. That is cheaper
        // than holding the lock across the computation, and is not a correctness concern.
        let expanded = DirectKeyExpander.expand(key)?;

        let mut cache = self.0.write();

        // The set of relays in use turns over slowly, so overflowing the bound means the node's
        // routing has changed wholesale; dropping everything then is rarer and cheaper than
        // tracking recency on every packet.
        if cache.len() >= EXPANDED_KEY_CACHE_CAPACITY {
            cache.clear();
        }
        cache.insert(*key, expanded.clone());

        Ok(expanded)
    }
}

/// Default [encoder](PacketEncoder) implementation for HOPR packets.
pub struct HoprEncoder<Chain, G, S, T> {
    chain_api: Chain,
    surb_store: S,
    ticket_factory: T,
    chain_key: ChainKeypair,
    channels_dst: Hash,
    ssa_generator: G,
    key_expander: CachingKeyExpander,
    cfg: HoprCodecConfig,
}

impl<Chain, G, S, T> HoprEncoder<Chain, G, S, T> {
    /// Creates a new instance of the encoder.
    pub fn new(
        chain_key: ChainKeypair,
        chain_api: Chain,
        surb_store: S,
        ticket_factory: T,
        channels_dst: Hash,
        ssa_generator: G,
        cfg: HoprCodecConfig,
    ) -> Self {
        Self {
            chain_api,
            surb_store,
            ticket_factory,
            chain_key,
            channels_dst,
            ssa_generator,
            key_expander: CachingKeyExpander::default(),
            cfg,
        }
    }
}

impl<Chain, G, S, T> HoprEncoder<Chain, G, S, T>
where
    Chain: ChainKeyOperations + ChainReadChannelOperations + ChainReadTicketOperations + ChainValues + Sync,
    G: EntryShareGenerator<HoprPixSpec>,
    S: SurbStore,
    T: hopr_api::tickets::TicketFactory + Sync,
{
    fn encode_packet_internal<D: AsRef<[u8]> + Send + 'static, Sig: Into<PacketSignals> + Send + 'static>(
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
            .map_err(HoprProtocolError::resolver)?
            .ok_or(HoprProtocolError::KeyNotFound)?;

        // Decide whether to create a multi-hop or a zero-hop ticket
        let next_ticket = if num_hops > 1 {
            let channel = self
                .chain_api
                .channel_by_parties(self.chain_key.as_ref(), &next_peer)
                .map_err(HoprProtocolError::resolver)?
                .ok_or_else(|| HoprProtocolError::ChannelNotFound(*self.chain_key.as_ref(), next_peer))?;

            let (outgoing_ticket_win_prob, outgoing_ticket_price) = self
                .chain_api
                .outgoing_ticket_values(self.cfg.outgoing_win_prob, self.cfg.outgoing_ticket_price)
                .map_err(HoprProtocolError::resolver)?;

            self.ticket_factory
                .new_multihop_ticket(
                    &channel,
                    (num_hops as u8).try_into().expect("cannot fail due to num_hops > 1"),
                    outgoing_ticket_win_prob,
                    outgoing_ticket_price,
                )
                .map_err(HoprProtocolError::ticket_factory)?
        } else {
            TicketBuilder::zero_hop().counterparty(next_peer)
        };

        // Construct the outgoing packet
        let (packet, openers) = HoprPacket::into_outgoing(
            data.as_ref(),
            &pseudonym,
            routing,
            &self.chain_key,
            next_ticket,
            self.chain_api.key_id_mapper_ref(),
            &self.key_expander,
            &self.channels_dst,
            &self.ssa_generator,
            signals,
        )?;

        // Store the reply openers under the given SenderId
        // This is a no-op for reply packets
        let mut minted_surbs = Vec::with_capacity(openers.len());
        openers.into_iter().for_each(|(surb_id, opener)| {
            minted_surbs.push(surb_id);
            self.surb_store
                .insert_reply_opener(HoprSenderId::from_pseudonym_and_id(&pseudonym, surb_id), opener);
        });

        let out = packet.try_as_outgoing().ok_or(HoprProtocolError::InvalidState(
            "cannot send out packet that is not outgoing",
        ))?;

        let mut transport_payload = BytesMut::with_capacity(HoprPacket::SIZE);
        transport_payload.put_slice(out.packet.as_ref());
        transport_payload.put_slice(&out.ticket.into_encoded());

        Ok(OutgoingPacket {
            next_hop: out.next_hop,
            ack_challenge: out.ack_challenge,
            encrypted_pix_share: out.encrypted_pix_share,
            data: transport_payload.freeze(),
            minted_surbs,
        })
    }
}

impl<Chain, G, S, T> PacketEncoder for HoprEncoder<Chain, G, S, T>
where
    Chain: ChainKeyOperations + ChainReadChannelOperations + ChainReadTicketOperations + ChainValues + Send + Sync,
    G: EntryShareGenerator<HoprPixSpec>,
    S: SurbStore + Send + Sync,
    T: hopr_api::tickets::TicketFactory + Send + Sync,
{
    type Error = HoprProtocolError;

    #[tracing::instrument(skip_all, level = "trace")]
    fn encode_packet<D: AsRef<[u8]> + Send + 'static, Sig: Into<PacketSignals> + Send + 'static>(
        &self,
        data: D,
        routing: ResolvedTransportRouting<HoprSurb>,
        signals: Sig,
        generation: Option<u8>,
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
                    // Stamp the SURB-batch generation captured when these return paths were resolved,
                    // so the replying side drops SURBs left over from a superseded return path. It is
                    // captured with the plan (not read here) so a concurrent re-plan/bump cannot
                    // label this already-chosen batch with a newer generation; falling back to the
                    // store's current value only when a caller did not supply one.
                    generation: generation.unwrap_or_else(|| self.surb_store.current_generation(&pseudonym)),
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
                    PacketRouting::Surb(sender_id, surb),
                )
            }
        };

        tracing::trace!(len = data.as_ref().len(), "encoding packet");
        self.encode_packet_internal(next_peer, data, num_hops, signals, routing, pseudonym)
    }

    #[tracing::instrument(skip_all, level = "trace", fields(destination = destination.to_peerid_str()))]
    fn encode_acknowledgements(
        &self,
        acks: &[VerifiedAcknowledgement],
        destination: &OffchainPublicKey,
    ) -> Result<OutgoingPacket, Self::Error> {
        tracing::trace!(num_acks = acks.len(), "encoding acknowledgements");

        let mut all_acks = Vec::<u8>::with_capacity(size_of::<u16>() + acks.len() * Acknowledgement::SIZE);
        all_acks.extend((acks.len() as u16).to_be_bytes());
        acks.iter().for_each(|ack| all_acks.extend(ack.leak().as_ref()));

        self.encode_packet_internal(
            *destination,
            all_acks,
            0,
            None,
            PacketRouting::NoAck(*destination),
            HoprPseudonym::random(),
        )
    }
}
