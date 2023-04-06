use core_crypto::primitives::{DigestLike, SimpleMac};
use core_crypto::prp::{PRP, PRPParameters};
use core_crypto::routing::header_length;
use core_crypto::types::PublicKey;
use core_types::channels::{AcknowledgementChallenge, Ticket};
use utils_types::traits::BinarySerializable;
use crate::por::POR_SECRET_LENGTH;

pub const INTERMEDIATE_HOPS: usize = 3; // 3 relayers and 1 destination
pub const PAYLOAD_SIZE: usize = 500;

pub const fn packet_length(max_hops: usize, additional_data_relayer_len: usize, additional_data_last_hop_len: usize) -> usize {
    PublicKey::SIZE_COMPRESSED + header_length(max_hops, additional_data_relayer_len, additional_data_last_hop_len) +
        SimpleMac::SIZE + PAYLOAD_SIZE
}

fn onion_encrypt(data: &[u8], secrets: &[&[u8]]) -> Box<[u8]> {
    let mut input: Box<[u8]> = data.into();
    for i in secrets.len()..0 {
        let prp = PRP::from_parameters(PRPParameters::new(secrets[i]));
        input = prp.forward(&input).unwrap_or_else(|e| panic!("onion encryption error {}", e))
    }
    input
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