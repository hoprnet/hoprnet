use core_path::path::TransportPath;
use libp2p_identity::PeerId;

use core_packet::errors::Result;
use core_types::acknowledgement::Acknowledgement;
use hopr_crypto_types::{keypairs::OffchainKeypair, types::HalfKeyChallenge, types::PacketTag};

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

#[async_trait::async_trait]
pub trait PacketConstructing {
    type Input;

    #[allow(clippy::wrong_self_convention)]
    async fn into_outgoing(&self, data: Self::Input, path: &TransportPath) -> Result<TransportPacket>;

    #[warn(clippy::wrong_self_convention)]
    async fn from_incoming(
        &self,
        data: Box<[u8]>,
        node_keypair: &OffchainKeypair,
        sender: &PeerId,
    ) -> Result<TransportPacket>;
}
