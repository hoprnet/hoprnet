use libp2p_identity::PeerId;
use core_crypto::primitives::{DigestLike, SimpleMac};
use core_crypto::prp::{PRP, PRPParameters};
use core_crypto::routing::{header_length, RoutingInfo};
use core_crypto::types::PublicKey;
use core_types::channels::{AcknowledgementChallenge, Ticket};
use utils_types::traits::{BinarySerializable, PeerIdLike};
use crate::por::POR_SECRET_LENGTH;

pub const INTERMEDIATE_HOPS: usize = 3; // 3 relayers and 1 destination
pub const PAYLOAD_SIZE: usize = 500;

pub const fn packet_length(max_hops: usize, additional_data_relayer_len: usize, additional_data_last_hop_len: usize) -> usize {
    PublicKey::SIZE_COMPRESSED + header_length(max_hops, additional_data_relayer_len, additional_data_last_hop_len) +
        SimpleMac::SIZE + PAYLOAD_SIZE
}

const PADDING_TAG: &[u8] = b"HOPR";

fn add_padding(msg: &[u8]) -> Box<[u8]> {
    assert!(msg.len() <= PAYLOAD_SIZE - PADDING_TAG.len(), "message too long for padding");
    let mut ret = vec![0u8; PAYLOAD_SIZE];
    ret[PAYLOAD_SIZE - msg.len() .. PAYLOAD_SIZE].copy_from_slice(msg);
    ret[PAYLOAD_SIZE - msg.len() - PADDING_TAG .. PADDING_TAG].copy_from_slice(PADDING_TAG);
    ret.into_boxed_slice()
}

fn remove_padding(msg: &[u8]) -> Option<&[u8]> {
    assert_eq!(PAYLOAD_SIZE, msg.len(), "padded message must be PAYLOAD_SIZE long");
    let pos = msg.windows(PADDING_TAG.len()).position(|window| window == PADDING_TAG)?;
    Some(msg.split_at(pos)[1][PADDING_TAG..])
}


fn onion_encrypt(data: &[u8], secrets: &[&[u8]]) -> Box<[u8]> {
    let mut input: Box<[u8]> = data.into();
    for i in secrets.len()..0 {
        let prp = PRP::from_parameters(PRPParameters::new(secrets[i]));
        input = prp.forward(&input).unwrap_or_else(|e| panic!("onion encryption error {}", e))
    }
    input
}

fn encode_packet(secrets: &[&[u8]], alpha: &[u8], msg: &[u8], path: &[PeerId], max_hops: usize,
                 additional_relayer_data_len: usize, additional_data_relayer: &[&[u8]],
                 additional_data_last_hop: Option<&[u8]>) -> Box<[u8]> {
    assert!(msg.len() <= PAYLOAD_SIZE, "message too long to fit into a packet");

    let padded = add_padding(msg);

    let routing_info = RoutingInfo::new(max_hops,
                                        path.iter()
                                            .map(|peer| PublicKey::from_peerid(peer))
                                            .collect(),
                                        secrets, additional_relayer_data_len,
                                        additional_data_relayer, additional_data_last_hop);

    let ct = onion_encrypt(&padded, secrets);

    let mut ret = Vec::<u8>::with_capacity(100); // TODO: update capacity
    ret.extend_from_slice(alpha);
    ret.extend_from_slice(&routing_info.serialize());
    ret.extend_from_slice(&ct);
    ret.into_boxed_slice()
}

enum ForwardedPacket<'a> {
    RelayedPacket {
        _chunk: Box<[u8]>,
        packet: &'a [u8],
        next_hop: &'a [u8],
        additional_relay_data: &'a [u8],
        derived_secret: &'a [u8],
        packet_tag: &'a [u8]
    },
    FinalNodePacket {
        _chunk: Box<[u8]>,
        plaintext: &'a [u8],
        additional_data: &'a [u8],
        derived_secret: &'a [u8],
        packet_tag: &'a [u8]
    }
}

pub struct Packet {
    packet: Box<[u8]>,
    challenge: AcknowledgementChallenge,
    ticket: Ticket
}

impl BinarySerializable for Packet {
    const SIZE: usize = packet_length(INTERMEDIATE_HOPS + 1, POR_SECRET_LENGTH, 0);

    fn deserialize(data: &[u8]) -> utils_types::errors::Result<Self> {
        todo!()
    }

    fn serialize(&self) -> Box<[u8]> {
        todo!()
    }
}