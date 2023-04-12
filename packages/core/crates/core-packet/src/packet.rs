use libp2p_identity::PeerId;
use core_crypto::derivation::{derive_ack_key_share, derive_packet_tag};
use core_crypto::primitives::{DigestLike, SimpleMac};
use core_crypto::prp::{PRP, PRPParameters};
use core_crypto::routing::{forward_header, ForwardedHeader, header_length, RoutingInfo};
use core_crypto::shared_keys::SharedKeys;
use core_crypto::types::{Challenge, CurvePoint, HalfKey, HalfKeyChallenge, PublicKey};
use core_types::acknowledgment::{Acknowledgement, AcknowledgementChallenge};
use core_types::channels::Ticket;
use utils_types::traits::{BinarySerializable, PeerIdLike};
use crate::errors::PacketError::PacketDecodingError;

use crate::por::{POR_SECRET_LENGTH, pre_verify, ProofOfRelayString, ProofOfRelayValues};
use crate::errors::Result;
use crate::packet::ForwardedPacket::{FinalNodePacket, RelayedPacket};
use crate::packet::PacketState::{Final, Forwarded, Outgoing};

/// Number of intermediate hops: 3 relayers and 1 destination
pub const INTERMEDIATE_HOPS: usize = 3;

/// Maximum size of the packet payload
pub const PAYLOAD_SIZE: usize = 500;

/// Length of the packet including header and the payload
pub const PACKET_LENGTH: usize = packet_length(INTERMEDIATE_HOPS + 1, POR_SECRET_LENGTH, 0);

/// Tag used to separate padding from data
const PADDING_TAG: &[u8] = b"HOPR";

pub const fn packet_length(max_hops: usize, additional_data_relayer_len: usize, additional_data_last_hop_len: usize) -> usize {
    PublicKey::SIZE_COMPRESSED + header_length(max_hops, additional_data_relayer_len, additional_data_last_hop_len) +
        SimpleMac::SIZE + PAYLOAD_SIZE
}

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

enum ForwardedPacket {
    RelayedPacket {
        packet: Box<[u8]>,
        next_node: PublicKey,
        additional_info: Box<[u8]>,
        derived_secret: Box<[u8]>,
        packet_tag: Box<[u8]>
    },
    FinalNodePacket {
        plain_text: Box<[u8]>,
        additional_data: Box<[u8]>,
        derived_secret: Box<[u8]>,
        packet_tag: Box<[u8]>
    }
}

fn forward_packet(private_key: &[u8], packet: &[u8], additional_data_relayer_len: usize,
                      additional_data_last_hop_len: usize, max_hops: usize) -> Result<ForwardedPacket> {
    let header_len = header_length(max_hops, additional_data_relayer_len, additional_data_last_hop_len);
    let decoded = PacketHeader::deserialize(packet, header_len);

    let shared_keys = SharedKeys::forward_transform(decoded.alpha, private_key)?;
    let secret = shared_keys.secret(0);

    let mut routing_info_mut: Vec<u8> = decoded.routing_info.into();
    let fwd_header = forward_header(secret, &mut routing_info_mut, decoded.mac, max_hops,
                                additional_data_relayer_len, additional_data_last_hop_len)?;

    let prp = PRP::from_parameters(PRPParameters::new(secret));
    let plain_text = prp.inverse(decoded.cipher_text)?;

    match fwd_header {
        ForwardedHeader::RelayNode { header, mac, next_node, additional_info } => {
            let packet = PacketHeader {
                alpha: &shared_keys.alpha(),
                routing_info: &header,
                mac: &mac,
                cipher_text: &plain_text
            };

            Ok(RelayedPacket {
                packet: packet.serialize(),
                packet_tag: derive_packet_tag(secret)?,
                derived_secret: secret.into(),
                next_node, additional_info
            })
        }
        ForwardedHeader::FinalNode { additional_data } => {
            Ok(FinalNodePacket {
                packet_tag: derive_packet_tag(secret)?,
                derived_secret: secret.into(),
                plain_text: remove_padding(&plain_text).ok_or(PacketDecodingError)?.into(),
                additional_data
            })
        }
    }
}

/// Indicates if the packet is supposed to be forwarded to the next hop or if it is intended for us.
pub enum PacketState {
    /// Packet is intended for us
    Final {
        packet_tag: Box<[u8]>,
        ack_key: HalfKey,
        previous_hop: PublicKey,
        plain_text: Box<[u8]>,
        old_challenge: Option<AcknowledgementChallenge>,
    },
    /// Packet must be forwarded
    Forwarded {
        ack_challenge: HalfKeyChallenge,
        packet_tag: Box<[u8]>,
        ack_key: HalfKey,
        previous_hop: PublicKey,
        own_key: HalfKey,
        own_share: HalfKeyChallenge,
        next_hop: PublicKey,
        next_challenge: Challenge,
        old_challenge: Option<AcknowledgementChallenge>,
    },
    /// Packet that is being sent out by us
    Outgoing {
        ack_challenge: HalfKeyChallenge,
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Packet {
    packet: Box<[u8]>,
    challenge: AcknowledgementChallenge,
    ticket: Ticket,
    state: PacketState
}

impl Packet {
    pub const SIZE: usize = PACKET_LENGTH + AcknowledgementChallenge::SIZE + Ticket::SIZE;

    pub fn new(msg: &[u8], path: &[&PeerId], private_key: &[u8], created_ticket: Ticket) -> Result<Self> {
        assert!(!path.is_empty(), "path must not be empty");

        let shared_keys = SharedKeys::new(path)?;

        let porv = ProofOfRelayValues::new(shared_keys.secret(0),
                                           (shared_keys.count_shared_keys() > 1).then_some(shared_keys.secret(1))
        );

        let mut por_strings = Vec::with_capacity(path.len() - 1);
        for i in 0..path.len() - 1 {
            let has_other = i + 2 < path.len();
            por_strings.push(ProofOfRelayString::new(shared_keys.secret(i + 1),
                                                     has_other.then_some(shared_keys.secret(i + 2)))
                .serialize())
        }

        let mut ticket = created_ticket;
        ticket.challenge = porv.ticket_challenge.to_ethereum_challenge();

        Ok(Self {
            challenge: AcknowledgementChallenge::new(porv.ack_challenge.clone(), private_key),
            packet: encode_packet(shared_keys, msg, path, INTERMEDIATE_HOPS + 1, POR_SECRET_LENGTH, &por_strings
                .iter()
                .map(|pors| pors.as_ref())
                .collect::<Vec<_>>(), None),
            ticket,
            state: Outgoing {
                ack_challenge: porv.ack_challenge
            }
        })
    }

    pub fn deserialize(pre: &[u8], private_key: &[u8], sender: &PeerId) -> Result<Self> {
        if pre.len() == Self::SIZE {
            let (packet, r0) = pre.split_at(PACKET_LENGTH);
            let (pre_challenge, pre_ticket) = r0.split_at(AcknowledgementChallenge::SIZE);
            let previous_hop = PublicKey::from_peerid(sender)?;

            match forward_packet(private_key, packet, POR_SECRET_LENGTH, 0, INTERMEDIATE_HOPS + 1)? {
                RelayedPacket { derived_secret, additional_info, packet_tag, next_node, .. } => {
                    let ack_key = derive_ack_key_share(&derived_secret);
                    let mut challenge = AcknowledgementChallenge::deserialize(pre_challenge)?;
                    challenge.validate(ack_key.to_challenge(),
                                       &previous_hop).then(|| ()).ok_or(PacketDecodingError)?;

                    let ticket = Ticket::deserialize(pre_ticket)?;
                    let verification_output = pre_verify(&derived_secret, &additional_info, &ticket.challenge)?;
                    Ok(Self {
                        packet: packet.into(),
                        challenge,
                        ticket,
                        state: Forwarded {
                            packet_tag,
                            ack_key,
                            previous_hop,
                            own_key: verification_output.own_key,
                            own_share: verification_output.own_share,
                            next_hop: next_node,
                            next_challenge: verification_output.next_ticket_challenge,
                            ack_challenge: verification_output.ack_challenge,
                            old_challenge: None,
                        }
                    })
                }
                FinalNodePacket { packet_tag, plain_text, derived_secret, .. } => {
                    let ack_key = derive_ack_key_share(&derived_secret);
                    let mut challenge = AcknowledgementChallenge::deserialize(pre_challenge)?;
                    challenge.validate(ack_key.to_challenge(),
                                       &previous_hop).then(|| ()).ok_or(PacketDecodingError)?;

                    let ticket = Ticket::deserialize(pre_ticket)?;
                    Ok(Self {
                        packet: packet.into(),
                        challenge,
                        ticket,
                        state: Final {
                            packet_tag,
                            ack_key,
                            previous_hop,
                            plain_text,
                            old_challenge: None,
                        }
                    })
                }
            }
        } else {
            Err(PacketDecodingError)
        }
    }

    pub fn create_acknowledgement(&self, private_key: &[u8]) -> Option<Acknowledgement> {
        match &self.state {
            Final { ack_key, old_challenge, .. } |
            Forwarded { ack_key, old_challenge, .. } => {
              Some(Acknowledgement::new(self.old_challenge.unwrap_or(self.challenge.clone())
                                        , ack_key.clone(), private_key))
            },
            Outgoing { .. } => None
        }
    }

    pub fn state(&self) -> &PacketState {
        &self.state
    }

    pub fn ticket(&self) -> &Ticket {
        &self.ticket
    }
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
    use libp2p_identity::{Keypair, PeerId};
    use core_crypto::types::PublicKey;
    use core_types::channels::Ticket;
    use utils_types::traits::PeerIdLike;
    use crate::packet::{add_padding, INTERMEDIATE_HOPS, Packet, PacketState, PADDING_TAG, remove_padding};

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

    fn mock_ticket(next_peer: &PeerId, path_len: usize, private_key: &[u8]) -> Ticket {

    }

    #[test]
    fn test_packet_create_and_transform() {
        const AMOUNT: usize = INTERMEDIATE_HOPS + 1;
        let mut keypairs = (0 .. AMOUNT)
            .map(|_| Keypair::generate_secp256k1())
            .map(|kp| (kp.into_secp256k1().unwrap().secret().to_bytes(), kp.public().to_peer_id()))
            .collect::<Vec<_>>();

        let (private_key, public_key) = keypairs.drain(1)
            .map(|kp| (kp.0, PublicKey::from_peerid(&kp.1).unwrap()))
            .last()
            .unwrap();

        let path = &keypairs
            .iter()
            .map(|kp| &kp.1)
            .collect::<Vec<_>>();

        // Create ticket for the first peer on the path
        let ticket = mock_ticket(&keypairs[0].1, keypairs.len(), &private_key);

        let test_message = b"some testing message";
        let mut packet = Packet::new(&test_message, path, &private_key, ticket)
            .expect("failed to construct packet");

        for (i, (node_private, node_id)) in keypairs.iter().enumerate() {
            let sender = if i == 0 { &public_key.to_peerid() } else { &path[i - 1] };
            packet = Packet::deserialize(&packet.serialize(), node_private, sender)
                .expect(format!("failed to deserialize packet at hop {}", i).into());

            match packet.state() {
                PacketState::Final { plain_text, .. } => {
                    assert_eq!(path.len() - 1, i);
                    assert_eq!(test_message, plain_text);
                }
                PacketState::Forwarded { .. } => {}
                PacketState::Outgoing { .. } => {}
            }
        }
    }
}