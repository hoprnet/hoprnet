use hopr_db_api::protocol::TransportPacketWithChainData;
use libp2p_identity::PeerId;

use hopr_crypto_types::prelude::*;
use hopr_internal_types::protocol::Acknowledgement;

use crate::errors::ProtocolError;

pub enum IncomingPacket {
    /// Packet is intended for us
    Final {
        packet_tag: PacketTag,
        previous_hop: PeerId,
        plain_text: Box<[u8]>,
        ack: Acknowledgement,
    },
    /// Packet must be forwarded
    Forwarded {
        packet_tag: PacketTag,
        previous_hop: PeerId,
        next_hop: PeerId,
        data: Box<[u8]>,
        ack: Acknowledgement,
    },
}

impl TryFrom<TransportPacketWithChainData> for IncomingPacket {
    type Error = ProtocolError;

    fn try_from(value: TransportPacketWithChainData) -> std::result::Result<Self, ProtocolError> {
        match value {
            TransportPacketWithChainData::Final {
                packet_tag,
                previous_hop,
                plain_text,
                ack,
            } => Ok(IncomingPacket::Final {
                packet_tag,
                previous_hop: previous_hop.into(),
                plain_text,
                ack,
            }),
            TransportPacketWithChainData::Forwarded {
                packet_tag,
                previous_hop,
                next_hop,
                data,
                ack,
            } => Ok(IncomingPacket::Forwarded {
                packet_tag,
                previous_hop: previous_hop.into(),
                next_hop: next_hop.into(),
                data,
                ack,
            }),
            TransportPacketWithChainData::Outgoing { .. } => Err(ProtocolError::Logic(
                "Outgoing packet received when processing incoming packets".to_string(),
            )),
        }
    }
}

/// Packet that is being sent out by us
pub struct OutgoingPacket {
    pub next_hop: PeerId,
    pub ack_challenge: HalfKeyChallenge,
    pub data: Box<[u8]>,
}

impl TryFrom<TransportPacketWithChainData> for OutgoingPacket {
    type Error = ProtocolError;

    fn try_from(value: TransportPacketWithChainData) -> std::result::Result<Self, Self::Error> {
        match value {
            TransportPacketWithChainData::Final { .. } | TransportPacketWithChainData::Forwarded { .. } => Err(
                ProtocolError::Logic("Incoming packet received when processing outgoing packets".to_string()),
            ),
            TransportPacketWithChainData::Outgoing {
                next_hop,
                ack_challenge,
                data,
            } => Ok(OutgoingPacket {
                next_hop: next_hop.into(),
                ack_challenge,
                data,
            }),
        }
    }
}

pub enum TransportPacket {
    /// Packet is intended for us
    Final {
        packet_tag: PacketTag,
        previous_hop: PeerId,
        plain_text: Box<[u8]>,
        ack: Acknowledgement,
    },
    /// Packet must be forwarded
    Forwarded {
        packet_tag: PacketTag,
        previous_hop: PeerId,
        next_hop: PeerId,
        data: Box<[u8]>,
        ack: Acknowledgement,
    },
    /// Packet that is being sent out by us
    Outgoing {
        next_hop: PeerId,
        ack_challenge: HalfKeyChallenge,
        data: Box<[u8]>,
    },
}

impl From<IncomingPacket> for TransportPacket {
    fn from(value: IncomingPacket) -> Self {
        match value {
            IncomingPacket::Final {
                packet_tag,
                previous_hop,
                plain_text,
                ack,
            } => TransportPacket::Final {
                packet_tag,
                previous_hop,
                plain_text,
                ack,
            },
            IncomingPacket::Forwarded {
                packet_tag,
                previous_hop,
                next_hop,
                data,
                ack,
            } => TransportPacket::Forwarded {
                packet_tag,
                previous_hop,
                next_hop,
                data,
                ack,
            },
        }
    }
}

impl From<OutgoingPacket> for TransportPacket {
    fn from(value: OutgoingPacket) -> Self {
        TransportPacket::Outgoing {
            next_hop: value.next_hop,
            ack_challenge: value.ack_challenge,
            data: value.data,
        }
    }
}
