use libp2p_identity::PeerId;
use core_crypto::primitives::{DigestLike, SimpleMac};
use core_crypto::prp::{PRP, PRPParameters};
use core_crypto::routing::{header_length, RoutingInfo};
use core_crypto::shared_keys::SharedKeys;
use core_crypto::types::{CurvePoint, HalfKeyChallenge, PublicKey};
use core_types::channels::{AcknowledgementChallenge, Ticket};
use utils_types::traits::{BinarySerializable, PeerIdLike};

use crate::por::{POR_SECRET_LENGTH, ProofOfRelayString, ProofOfRelayValues};
use crate::errors::Result;
use crate::errors::PacketError::PacketDecodingError;

pub const INTERMEDIATE_HOPS: usize = 3; // 3 relayers and 1 destination
pub const PAYLOAD_SIZE: usize = 500;

pub const fn packet_length(max_hops: usize, additional_data_relayer_len: usize, additional_data_last_hop_len: usize) -> usize {
    PublicKey::SIZE_COMPRESSED + header_length(max_hops, additional_data_relayer_len, additional_data_last_hop_len) +
        SimpleMac::SIZE + PAYLOAD_SIZE
}

pub const PACKET_LENGTH: usize = packet_length(INTERMEDIATE_HOPS + 1, POR_SECRET_LENGTH, 0);

const PADDING_TAG: &[u8] = b"HOPR";

fn add_padding(msg: &[u8]) -> Box<[u8]> {
    assert!(msg.len() <= PAYLOAD_SIZE - PADDING_TAG.len(), "message too long for padding");
    let mut ret = vec![0u8; PAYLOAD_SIZE];
    ret[PAYLOAD_SIZE - msg.len() .. PAYLOAD_SIZE].copy_from_slice(msg);
    ret[PAYLOAD_SIZE - msg.len() - PADDING_TAG.len() .. PAYLOAD_SIZE - msg.len()]
        .copy_from_slice(PADDING_TAG);
    ret.into_boxed_slice()
}

fn remove_padding(msg: &[u8]) -> Option<&[u8]> {
    assert_eq!(PAYLOAD_SIZE, msg.len(), "padded message must be PAYLOAD_SIZE long");
    let pos = msg.windows(PADDING_TAG.len()).position(|window| window == PADDING_TAG)?;
    Some(&msg.split_at(pos).1[PADDING_TAG.len() ..])
}


fn onion_encrypt(data: &[u8], secrets: &[&[u8]]) -> Box<[u8]> {
    let mut input: Box<[u8]> = data.into();
    for i in secrets.len()..0 {
        let prp = PRP::from_parameters(PRPParameters::new(secrets[i]));
        input = prp.forward(&input).unwrap_or_else(|e| panic!("onion encryption error {}", e))
    }
    input
}

struct PacketHeader<'a> {
    alpha: &'a [u8],
    routing_info: &'a [u8],
    mac: &'a [u8],
    cipher_text: &'a [u8]
}

fn encode_packet(shared_keys: SharedKeys, msg: &[u8], path: &[&PeerId], max_hops: usize,
                 additional_relayer_data_len: usize, additional_data_relayer: &[&[u8]],
                 additional_data_last_hop: Option<&[u8]>) -> Box<[u8]> {
    assert!(msg.len() <= PAYLOAD_SIZE, "message too long to fit into a packet");

    let padded = add_padding(msg);
    let secrets = shared_keys.secrets();
    let routing_info = RoutingInfo::new(max_hops,
                                        &path.iter()
                                            .map(|peer| PublicKey::from_peerid(peer)
                                                .unwrap_or_else(|e| panic!("invalid peer id given: {}", e)))
                                            .collect::<Vec<_>>(),
                                        &secrets, additional_relayer_data_len,
                                        additional_data_relayer, additional_data_last_hop);

    PacketHeader {
        alpha: &shared_keys.alpha(),
        routing_info: &routing_info.serialize(),
        mac: &routing_info.mac,
        cipher_text: &onion_encrypt(&padded, &secrets)
    }.serialize()
}

impl<'a> PacketHeader<'a> {
    pub fn deserialize(packet: &'a [u8], header_len: usize) -> PacketHeader<'a> {
        Self {
            alpha: &packet[..CurvePoint::SIZE_COMPRESSED],
            routing_info: &packet[CurvePoint::SIZE_COMPRESSED .. CurvePoint::SIZE_COMPRESSED + header_len],
            mac: &packet[CurvePoint::SIZE_COMPRESSED + header_len .. CurvePoint::SIZE_COMPRESSED + header_len
                + SimpleMac::SIZE],
            cipher_text: &packet[CurvePoint::SIZE_COMPRESSED + header_len + SimpleMac::SIZE ..
                CurvePoint::SIZE_COMPRESSED + header_len + SimpleMac::SIZE + PAYLOAD_SIZE ]
        }
    }

    pub fn serialize(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(self.alpha.len() + self.routing_info.len());
        ret.extend_from_slice(&self.alpha);
        ret.extend_from_slice(&self.routing_info);
        ret.extend_from_slice(&self.cipher_text);
        ret.into_boxed_slice()
    }
}

/*enum ForwardedPacket {
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
}*/

pub enum ForwardedPacket {

}

pub fn forward_packet(private_key: &[u8], packet: &[u8], additional_data_relayer_len: usize,
                      additional_data_last_hop_len: usize, max_hops: usize) -> Result<ForwardedPacket> {
    let header_len = header_length(max_hops, additional_data_relayer_len, additional_data_last_hop_len);
    let decoded = PacketHeader::deserialize(packet, header_len);

    SharedKeys::forward_transform(decoded.alpha, private_key);
    todo!()
}


#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Packet {
    packet: Box<[u8]>,
    challenge: AcknowledgementChallenge,
    ack_challenge: HalfKeyChallenge,
    ready_to_forward: bool,
    ticket: Ticket,
}

impl Packet {
    pub const SIZE: usize = PACKET_LENGTH + AcknowledgementChallenge::SIZE + Ticket::SIZE;

    pub fn new(msg: &[u8], path: &[&PeerId], private_key: &[u8], ticket: Ticket) -> Result<Self> {
        assert!(!path.is_empty(), "path must not be empty");

        let shared_keys = SharedKeys::new(path)?;
        let secrets = shared_keys.secrets();

        let porv = ProofOfRelayValues::new(&secrets[0], secrets.get(1).map(|s| *s));

        let mut por_strings = Vec::with_capacity(path.len() - 1);
        for i in 0..path.len() - 1 {
            let has_other = i + 2 < path.len();
            por_strings.push(ProofOfRelayString::new(secrets[i + 1], has_other.then_some(secrets[i + 2])).serialize())
        }

        Ok(Self {
            challenge: AcknowledgementChallenge::new(porv.ack_challenge.clone(), private_key),
            ack_challenge: porv.ack_challenge,
            packet: encode_packet(shared_keys, msg, path, INTERMEDIATE_HOPS + 1, POR_SECRET_LENGTH, &por_strings
                .iter()
                .map(|pors| pors.as_ref())
                .collect::<Vec<_>>(), None),
            ready_to_forward: true,
            ticket
        })
    }

    /*pub fn deserialize(pre: &[u8], private_key: &[u8], sender: &PeerId) -> Result<Self> {
        if pre.len() == Self::SIZE {
            let (packet, r0) = pre.split_at(PACKET_LENGTH);
            let (pre_challenge, pre_ticket) = r0.split_at(AcknowledgementChallenge::SIZE);



            Ok(())
        } else {
            Err(PacketDecodingError)
        }
    }*/

}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Packet {
    pub fn serialize(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.packet);
        ret.extend_from_slice(&self.challenge.serialize());
        ret.extend_from_slice(&self.ticket.serialize());
        ret.into_boxed_slice()
    }
}

#[cfg(test)]
mod tests {
    use crate::packet::{add_padding, PADDING_TAG, remove_padding};

    #[test]
    fn test_padding() {
        let data = b"test";
        let padded = add_padding(data);

        let mut expected = vec![0u8; 492];
        expected.extend_from_slice(PADDING_TAG);
        expected.extend_from_slice(data);
        assert_eq!(&expected, padded.as_ref());

        let unpadded = remove_padding(&padded);
        assert!(unpadded.is_some());
        assert_eq!(data, &unpadded.unwrap());
    }
}