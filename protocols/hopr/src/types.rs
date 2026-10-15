use bytes::Bytes;
use hopr_api::types::{crypto::prelude::*, internal::prelude::*};
use hopr_crypto_packet::prelude::*;
use hopr_protocol_pix::TaggedEncryptedPartialSsaShare;

/// Packet that is being sent out by us.
pub struct OutgoingPacket {
    /// Offchain public key of the next hop.
    pub next_hop: OffchainPublicKey,
    /// Challenge to be solved from the acknowledgement of the next hop.
    pub ack_challenge: HalfKeyChallenge,
    /// Optional encrypted partial SSA share for PIX protocol.
    pub encrypted_pix_share: Option<TaggedEncryptedPartialSsaShare<HoprPixSpec>>,
    /// Encoded HOPR packet.
    pub data: Bytes,
    /// SURBs minted onto this packet, in the order of the return paths that produced them.
    ///
    /// Surfaced so a layer above can pair each SURB with the return path it encodes — the ids exist
    /// only inside packet construction, and the routing that produced them only outside it, so this
    /// is the single point where the two can be associated. Empty for packets carrying no SURBs.
    pub minted_surbs: Vec<HoprSurbId>,
}

impl std::fmt::Debug for OutgoingPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutgoingPacket")
            .field("next_hop", &self.next_hop)
            .field("ack_challenge", &self.ack_challenge)
            .field("encrypted_pix_share", &self.encrypted_pix_share)
            .finish_non_exhaustive()
    }
}

/// Contains some miscellaneous information about a received packet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct AuxiliaryPacketInfo {
    /// Packet signals that the packet carried.
    ///
    /// Zero if no signal flags were specified.
    pub packet_signals: PacketSignals,
    /// Number of SURBs that the packet carried.
    pub num_surbs: usize,
    /// How many of those SURBs the store dropped on arrival because the per-pseudonym buffer was
    /// already full: the SURBs that left the store, or never entered it, to make room. A share-less
    /// newcomer the store turned away counts as well.
    ///
    /// This is the only point at which an overflow is visible per packet. The store gives up
    /// share-less SURBs first, and refuses a share-less newcomer before it evicts a share-bearing
    /// SURB, so what an eviction costs depends on what the buffer holds. A share-less SURB is only a
    /// lost return path. A share-bearing one carries a partial SSA share that reaches the
    /// reconstructor only when the SURB is *used*, so evicting it destroys the share. That only
    /// happens once the buffer holds nothing but shares, which makes this count an upper bound on the
    /// shares lost. The exact figure is [`SurbInsertOutcome::evicted_shares`]. It is deliberately not
    /// carried here: it is the store's own business, reported in its eviction summary (the
    /// `ring_share` value of the `cache` label of `hopr_surb_store_evictions_count`, and a warning per
    /// report interval). The redundancy budget that absorbs those losses is a fixed surplus per
    /// polynomial, and emission is windowed, so on deployed dimensions a burst of roughly
    /// `surplus × SHARE_EMISSION_WINDOW` evicted share-bearing SURBs is enough to put polynomials
    /// below their threshold — and a cycle short of recovery is worth nothing at all. See
    /// [`SurbStoreConfig::rb_capacity`](crate::SurbStoreConfig::rb_capacity) and
    /// `hopr_protocol_pix::SHARE_EMISSION_WINDOW`.
    ///
    /// The Exit's Session books this count into its SURB flow estimate — an evicted SURB has left the
    /// buffer just as a spent one has — so the meaning above must not change: it is the total of
    /// SURBs that left or never entered the store, not the number of lost shares. See
    /// `counterparty_buffer_capacity` in `hopr-transport-session`.
    pub num_evicted_surbs: usize,
}

/// An incoming packet with a payload intended for us.
pub struct IncomingFinalPacket {
    /// Packet tag.
    pub packet_tag: PacketTag,
    /// Offchain public key of the previous hop.
    pub previous_hop: OffchainPublicKey,
    /// Sender pseudonym.
    pub sender: HoprPseudonym,
    /// SURB this packet was a reply on, when it was one.
    ///
    /// `None` for a packet that was not sent using one of our SURBs. Surfaced because decoding
    /// already resolves the sender id to find the reply opener, so the id is known here and
    /// nowhere later.
    pub replied_on_surb: Option<HoprSurbId>,
    /// Plain text payload of the packet.
    pub plain_text: Box<[u8]>,
    /// Acknowledgement to be sent to the previous hop.
    pub ack_key: HalfKey,
    /// Miscellaneous information about the packet.
    pub info: AuxiliaryPacketInfo,
}

impl std::fmt::Debug for IncomingFinalPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IncomingFinalPacket")
            .field("packet_tag", &self.packet_tag)
            .field("previous_hop", &self.previous_hop)
            .field("sender", &self.sender)
            .field("ack_key", &self.ack_key)
            .field("info", &self.info)
            .finish_non_exhaustive()
    }
}

/// Incoming packet that must be forwarded.
pub struct IncomingForwardedPacket {
    /// Packet tag.
    pub packet_tag: PacketTag,
    /// Offchain public key of the previous hop.
    pub previous_hop: OffchainPublicKey,
    /// Offchain public key of the next hop.
    pub next_hop: OffchainPublicKey,
    /// Data to be forwarded to the next hop.
    pub data: Bytes,
    /// Challenge to be solved from the acknowledgement received from the next hop.
    pub ack_challenge: HalfKeyChallenge,
    /// Ticket to be acknowledged by solving the `ack_challenge`.
    pub received_ticket: UnacknowledgedTicket,
    /// Acknowledgement payload to be sent to the previous hop
    pub ack_key_prev_hop: HalfKey,
}

impl std::fmt::Debug for IncomingForwardedPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IncomingForwardedPacket")
            .field("packet_tag", &self.packet_tag)
            .field("previous_hop", &self.previous_hop)
            .field("next_hop", &self.next_hop)
            .field("received_ticket", &self.received_ticket)
            .field("ack_challenge", &self.ack_challenge)
            .field("ack_key_prev_hop", &self.ack_key_prev_hop)
            .finish_non_exhaustive()
    }
}

/// Incoming packet that contains acknowledgements of delivered packets.
pub struct IncomingAcknowledgementPacket {
    /// Packet tag.
    pub packet_tag: PacketTag,
    /// Offchain public key of the previous hop which sent the acknowledgements.
    pub previous_hop: OffchainPublicKey,
    /// Unverified acknowledgements.
    pub received_acks: Vec<Acknowledgement>,
}

impl std::fmt::Debug for IncomingAcknowledgementPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IncomingAcknowledgementPacket")
            .field("packet_tag", &self.packet_tag)
            .field("previous_hop", &self.previous_hop)
            .field("received_acks", &self.received_acks)
            .finish()
    }
}

/// Incoming HOPR packet.
#[derive(Debug, strum::EnumTryAs)]
pub enum IncomingPacket {
    /// Packet is intended for us
    Final(Box<IncomingFinalPacket>),
    /// Packet must be forwarded
    Forwarded(Box<IncomingForwardedPacket>),
    /// The packet contains acknowledgements of delivered packets.
    Acknowledgement(Box<IncomingAcknowledgementPacket>),
}

impl IncomingPacket {
    /// Tag identifying the packet.
    pub fn packet_tag(&self) -> &PacketTag {
        match self {
            IncomingPacket::Final(f) => &f.packet_tag,
            IncomingPacket::Forwarded(f) => &f.packet_tag,
            IncomingPacket::Acknowledgement(f) => &f.packet_tag,
        }
    }

    /// Previous hop that sent us the packet.
    pub fn previous_hop(&self) -> &OffchainPublicKey {
        match self {
            IncomingPacket::Final(f) => &f.previous_hop,
            IncomingPacket::Forwarded(f) => &f.previous_hop,
            IncomingPacket::Acknowledgement(f) => &f.previous_hop,
        }
    }
}

/// Contains a SURB found in the SURB ring buffer via `SurbStore::find_surb`.
#[derive(Debug)]
pub struct FoundSurb {
    /// Complete sender ID of the SURB.
    pub sender_id: HoprSenderId,
    /// The SURB itself.
    pub surb: HoprSurb,
    /// Number of SURBs remaining in the ring buffer with the same pseudonym.
    pub remaining: usize,
}

/// What storing SURBs via `SurbStore::insert_surbs` did to the ring buffer.
///
/// The `evicted` count exists because an overflow is otherwise entirely silent: the buffer drops a
/// SURB to make room — a share-less one whenever there is one to drop — and the caller sees only that
/// the insert "succeeded". That count is the only local evidence that the sender is producing faster
/// than this side can hold. It does not say what the overflow cost, though. Under PIX a share reaches
/// the reconstructor only when its SURB is *used*, so an evicted share-bearing SURB takes its share
/// with it permanently, whereas a share-less one is only a lost return path. `evicted_shares` is the
/// exact number of shares lost to overflow, which is what tells routine share-less churn from a loss
/// that matters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SurbInsertOutcome {
    /// Number of SURBs held for the pseudonym after the insert.
    pub retained: usize,
    /// Number of SURBs that left, or never entered, the buffer to make room during this insert:
    /// share-less ones first, oldest first, and the oldest share-bearing one only once none remain. A
    /// share-less SURB turned away because the buffer held nothing but shares counts as well.
    pub evicted: usize,
    /// Of [`evicted`](Self::evicted), how many carried a PIX share: the exact number of shares this
    /// insert lost to overflow, each of them for good. It is zero for as long as the buffer still has
    /// share-less SURBs to give up, and for every refused share-less newcomer.
    ///
    /// `evicted - evicted_shares` is the number of share-less SURBs dropped or refused, which cost a
    /// return path and nothing more.
    pub evicted_shares: usize,
}

/// Determines the result of how an acknowledgement was resolved.
#[derive(Debug, strum::EnumTryAs)]
pub enum ResolvedAcknowledgement {
    /// The acknowledgement resulted in a winning ticket.
    RelayingWin(Box<RedeemableTicket>),
    /// The acknowledgement resulted in a losing ticket.
    RelayingLoss(ChannelId),
}
