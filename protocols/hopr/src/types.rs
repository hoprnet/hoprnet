use hopr_crypto_packet::prelude::*;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;

/// Packet that is being sent out by us.
pub struct OutgoingPacket {
    /// Offchain public key of the next hop.
    pub next_hop: OffchainPublicKey,
    /// Challenge to be solved from the acknowledgement of the next hop.
    pub ack_challenge: HalfKeyChallenge,
    /// Encoded HOPR packet.
    pub data: Box<[u8]>,
}

impl std::fmt::Debug for OutgoingPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutgoingPacket")
            .field("next_hop", &self.next_hop)
            .field("ack_challenge", &self.ack_challenge)
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
}

/// An incoming packet with a payload intended for us.
pub struct IncomingFinalPacket {
    /// Packet tag.
    pub packet_tag: PacketTag,
    /// Offchain public key of the previous hop.
    pub previous_hop: OffchainPublicKey,
    /// Sender pseudonym.
    pub sender: HoprPseudonym,
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
    pub data: Box<[u8]>,
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

/// Contains a SURB found in the SURB ring buffer via  [`SurbStore::find_surb`].
#[derive(Debug)]
pub struct FoundSurb {
    /// Complete sender ID of the SURB.
    pub sender_id: HoprSenderId,
    /// The SURB itself.
    pub surb: HoprSurb,
    /// Number of SURBs remaining in the ring buffer with the same pseudonym.
    pub remaining: usize,
}

/// Determines the result of how an acknowledgement was resolved.
#[derive(Debug, strum::EnumTryAs)]
pub enum ResolvedAcknowledgement {
    /// The acknowledgement resulted in a winning ticket.
    RelayingWin(Box<RedeemableTicket>),
    /// The acknowledgement resulted in a losing ticket.
    RelayingLoss(ChannelId),
}
