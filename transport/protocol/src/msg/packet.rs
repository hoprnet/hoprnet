use hopr_db_api::protocol::TransportPacketWithChainData;
use libp2p_identity::PeerId;

use hopr_crypto_packet::errors::Result;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::protocol::Acknowledgement;
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

impl From<TransportPacketWithChainData> for TransportPacket {
    fn from(value: TransportPacketWithChainData) -> Self {
        match value {
            TransportPacketWithChainData::Final {
                packet_tag,
                previous_hop,
                plain_text,
                ack,
            } => TransportPacket::Final {
                packet_tag,
                previous_hop: previous_hop.into(),
                plain_text,
                ack,
            },
            TransportPacketWithChainData::Forwarded {
                packet_tag,
                previous_hop,
                next_hop,
                data,
                ack,
            } => TransportPacket::Forwarded {
                packet_tag,
                previous_hop: previous_hop.into(),
                next_hop: next_hop.into(),
                data,
                ack,
            },
            TransportPacketWithChainData::Outgoing {
                next_hop,
                ack_challenge,
                data,
            } => TransportPacket::Outgoing {
                next_hop: next_hop.into(),
                ack_challenge,
                data,
            },
        }
    }
}

#[async_trait::async_trait]
pub trait PacketConstructing {
    type Input;
    type Packet;

    #[allow(clippy::wrong_self_convention)]
    async fn to_send(&self, data: Self::Input, path: Vec<OffchainPublicKey>) -> Result<Self::Packet>;

    #[allow(clippy::wrong_self_convention)]
    async fn from_recv(
        &self,
        data: Box<[u8]>,
        node_keypair: &OffchainKeypair,
        sender: OffchainPublicKey,
    ) -> Result<Self::Packet>;
}
