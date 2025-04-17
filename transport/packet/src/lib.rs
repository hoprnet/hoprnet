//! Transport packet primitives for the HOPR protocol transport layer.

use libp2p_identity::PeerId;

use hopr_internal_types::protocol::Acknowledgement;

pub enum Payload {
    Msg(Box<[u8]>),
    Ack(Acknowledgement),
}

pub struct Header {
    peer: PeerId,
}

pub struct Packet {
    header: Header,
    payload: Payload,
}
