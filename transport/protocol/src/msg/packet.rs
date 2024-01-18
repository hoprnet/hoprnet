use core_path::path::TransportPath;
use libp2p_identity::PeerId;

use core_packet::errors::Result;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::acknowledgement::Acknowledgement;

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
#[allow(clippy::wrong_self_convention)] // TODO: The `from_*` and `into_*` should take self, not a reference
pub trait PacketConstructing {
    type Input;

    #[allow(clippy::wrong_self_convention)]
    async fn into_outgoing(&self, data: Self::Input, path: &TransportPath) -> Result<TransportPacket>;

    #[allow(clippy::wrong_self_convention)]
    async fn from_incoming(
        &self,
        data: Box<[u8]>,
        node_keypair: &OffchainKeypair,
        sender: &PeerId,
    ) -> Result<TransportPacket>;
}
