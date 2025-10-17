use hopr_crypto_packet::prelude::*;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;

/// Packet that is being sent out by us
pub struct OutgoingPacket {
    pub next_hop: OffchainPublicKey,
    pub ack_challenge: HalfKeyChallenge,
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

pub struct IncomingFinalPacket {
    pub packet_tag: PacketTag,
    pub previous_hop: OffchainPublicKey,
    pub sender: HoprPseudonym,
    pub plain_text: Box<[u8]>,
    pub ack_key: HalfKey,
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

pub struct IncomingForwardedPacket {
    pub packet_tag: PacketTag,
    pub previous_hop: OffchainPublicKey,
    pub next_hop: OffchainPublicKey,
    pub data: Box<[u8]>,
    /// Challenge to be solved from the acknowledgement of the next hop.
    pub ack_challenge: HalfKeyChallenge,
    /// Ticket to be acknowledged by the next hop.
    pub ticket: UnacknowledgedTicket,
    /// Acknowledgement payload to be sent to the previous hop
    pub ack_key: HalfKey,
}

impl std::fmt::Debug for IncomingForwardedPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IncomingForwardedPacket")
            .field("packet_tag", &self.packet_tag)
            .field("previous_hop", &self.previous_hop)
            .field("next_hop", &self.next_hop)
            .field("ack_key", &self.ack_key)
            .finish_non_exhaustive()
    }
}

pub struct IncomingAcknowledgementPacket {
    pub packet_tag: PacketTag,
    pub previous_hop: OffchainPublicKey,
    pub ack: Acknowledgement,
}

impl std::fmt::Debug for IncomingAcknowledgementPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IncomingAcknowledgementPacket")
            .field("packet_tag", &self.packet_tag)
            .field("previous_hop", &self.previous_hop)
            .field("ack", &self.ack)
            .finish()
    }
}

#[derive(Debug)]
pub enum IncomingPacket {
    /// Packet is intended for us
    Final(Box<IncomingFinalPacket>),
    /// Packet must be forwarded
    Forwarded(Box<IncomingForwardedPacket>),
    /// The packet contains an acknowledgement of a delivered packet.
    Acknowledgement(Box<IncomingAcknowledgementPacket>),
}

impl IncomingPacket {
    pub fn packet_tag(&self) -> &PacketTag {
        match self {
            IncomingPacket::Final(f) => &f.packet_tag,
            IncomingPacket::Forwarded(f) => &f.packet_tag,
            IncomingPacket::Acknowledgement(f) => &f.packet_tag,
        }
    }

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
