use crate::errors::PacketError::{InvalidPacketState, PacketDecodingError};
use core_crypto::derivation::{derive_ack_key_share, derive_packet_tag};
use core_crypto::primitives::{DigestLike, SimpleMac};
use core_crypto::prp::{PRPParameters, PRP};
use core_crypto::routing::{forward_header, header_length, ForwardedHeader, RoutingInfo};
use core_crypto::shared_keys::GroupEncoding;
use core_crypto::types::{Challenge, HalfKey, HalfKeyChallenge};
use core_types::acknowledgement::Acknowledgement;
use core_types::channels::Ticket;
use libp2p_identity::PeerId;
use core_crypto::ec_groups::{Ed25519SharedKeys};
use core_crypto::types::OffchainPublicKey;
use utils_types::errors::GeneralError::ParseError;
use utils_types::traits::{BinarySerializable, PeerIdLike};

use crate::errors::Result;
use crate::packet::ForwardedMetaPacket::{FinalPacket, RelayedPacket};
use crate::packet::PacketState::{Final, Forwarded, Outgoing};
use crate::por::{pre_verify, ProofOfRelayString, ProofOfRelayValues, POR_SECRET_LENGTH};

/// Number of intermediate hops: 3 relayers and 1 destination
pub const INTERMEDIATE_HOPS: usize = 3;

/// Maximum size of the packet payload
pub const PAYLOAD_SIZE: usize = 500;

/// Length of the packet including header and the payload
pub const PACKET_LENGTH: usize = packet_length(INTERMEDIATE_HOPS + 1, POR_SECRET_LENGTH, 0);

/// Tag used to separate padding from data
const PADDING_TAG: &[u8] = b"HOPR";

/// Determines the total length (header + payload) of the packet given the header information.
pub const fn packet_length(
    max_hops: usize,
    additional_data_relayer_len: usize,
    additional_data_last_hop_len: usize,
) -> usize {
    OffchainPublicKey::SIZE
        + header_length(max_hops, additional_data_relayer_len, additional_data_last_hop_len)
        + SimpleMac::SIZE
        + PAYLOAD_SIZE
}

fn add_padding(msg: &[u8]) -> Box<[u8]> {
    assert!(
        msg.len() <= PAYLOAD_SIZE - PADDING_TAG.len(),
        "message too long for padding"
    );
    let mut ret = vec![0u8; PAYLOAD_SIZE];
    ret[PAYLOAD_SIZE - msg.len()..PAYLOAD_SIZE].copy_from_slice(msg);
    ret[PAYLOAD_SIZE - msg.len() - PADDING_TAG.len()..PAYLOAD_SIZE - msg.len()].copy_from_slice(PADDING_TAG);
    ret.into_boxed_slice()
}

fn remove_padding(msg: &[u8]) -> Option<&[u8]> {
    assert_eq!(PAYLOAD_SIZE, msg.len(), "padded message must be PAYLOAD_SIZE long");
    let pos = msg
        .windows(PADDING_TAG.len())
        .position(|window| window == PADDING_TAG)?;
    Some(&msg.split_at(pos).1[PADDING_TAG.len()..])
}

struct MetaPacket {
    packet: Box<[u8]>,
    header_len: usize,
    alpha_len: usize,
}

#[allow(dead_code)]
enum ForwardedMetaPacket {
    RelayedPacket {
        packet: MetaPacket,
        next_node: OffchainPublicKey,
        additional_info: Box<[u8]>,
        derived_secret: Box<[u8]>,
        packet_tag: Box<[u8]>,
    },
    FinalPacket {
        plain_text: Box<[u8]>,
        additional_data: Box<[u8]>,
        derived_secret: Box<[u8]>,
        packet_tag: Box<[u8]>,
    },
}

impl MetaPacket {
    pub fn new(
        shared_keys: Ed25519SharedKeys,
        msg: &[u8],
        path: &[&PeerId],
        max_hops: usize,
        additional_relayer_data_len: usize,
        additional_data_relayer: &[&[u8]],
        additional_data_last_hop: Option<&[u8]>,
    ) -> MetaPacket {
        assert!(msg.len() <= PAYLOAD_SIZE, "message too long to fit into a packet");

        let mut padded = add_padding(msg);
        let secrets = shared_keys.secrets.iter().map(|s| s.as_slice()).collect::<Vec<_>>();
        let routing_info = RoutingInfo::new(
            max_hops,
            &path
                .iter()
                .map(|peer| OffchainPublicKey::from_peerid(peer).unwrap_or_else(|e| panic!("invalid peer id given: {e}")))
                .collect::<Vec<_>>(),
            &secrets,
            additional_relayer_data_len,
            additional_data_relayer,
            additional_data_last_hop,
        );

        // Encrypt packet payload using the derived shared secrets
        for secret in secrets.iter().rev() {
            let prp = PRP::from_parameters(PRPParameters::new(secret));
            prp.forward_inplace(&mut padded)
                .unwrap_or_else(|e| panic!("onion encryption error {e}"))
        }

        MetaPacket::new_from_parts(
            &shared_keys.alpha.encode(),
            &routing_info.routing_information,
            &routing_info.mac,
            &padded,
        )
    }

    fn new_from_parts(alpha: &[u8], routing_info: &[u8], mac: &[u8], payload: &[u8]) -> Self {
        assert!(alpha.len() >= 32);
        assert!(routing_info.len() > 0);
        assert_eq!(SimpleMac::SIZE, mac.len());
        assert_eq!(PAYLOAD_SIZE, payload.len());

        let mut packet = Vec::with_capacity(Self::size(alpha.len(),routing_info.len()));
        packet.extend_from_slice(alpha);
        packet.extend_from_slice(routing_info);
        packet.extend_from_slice(mac);
        packet.extend_from_slice(payload);

        Self {
            packet: packet.into_boxed_slice(),
            header_len: routing_info.len(),
            alpha_len: alpha.len()
        }
    }
    
    pub fn alpha(&self) -> &[u8] {
        &self.packet[..self.alpha_len]
    }

    pub fn routing_info(&self) -> &[u8] {
        &self.packet[self.alpha_len..self.alpha_len + self.header_len]
    }

    pub fn mac(&self) -> &[u8] {
        &self.packet[self.alpha_len + self.header_len
            ..self.alpha_len + self.header_len + SimpleMac::SIZE]
    }

    pub fn payload(&self) -> &[u8] {
        &self.packet[self.alpha_len + self.header_len + SimpleMac::SIZE
            ..self.alpha_len + self.header_len + SimpleMac::SIZE + PAYLOAD_SIZE]
    }

    pub const fn size(alpha_len: usize, header_len: usize) -> usize {
        alpha_len + header_len + SimpleMac::SIZE + PAYLOAD_SIZE
    }

    pub fn from_bytes(packet: &[u8], alpha_len: usize, header_len: usize) -> utils_types::errors::Result<MetaPacket> {
        if packet.len() == Self::size(alpha_len, header_len) {
            Ok(Self {
                packet: packet.into(),
                header_len,
                alpha_len
            })
        } else {
            Err(ParseError)
        }
    }

    pub fn to_bytes(&self) -> &[u8] {
        &self.packet
    }

    pub fn forward(
        &self,
        node_private_key: &[u8],
        node_public_key: &OffchainPublicKey,
        max_hops: usize,
        additional_data_relayer_len: usize,
        additional_data_last_hop_len: usize,
    ) -> Result<ForwardedMetaPacket> {
        let (alpha, secret) = Ed25519SharedKeys::forward_transform(
            OffchainPublicKey::from_bytes(self.alpha())?,
            node_private_key.try_into().expect("invalid packet private key"),
            node_public_key
        )?;

        let mut routing_info_cpy: Vec<u8> = self.routing_info().into();
        let fwd_header = forward_header(
            &secret,
            &mut routing_info_cpy,
            self.mac(),
            max_hops,
            additional_data_relayer_len,
            additional_data_last_hop_len,
        )?;

        let prp = PRP::from_parameters(PRPParameters::new(&secret));
        let decrypted = prp.inverse(self.payload())?;

        Ok(match fwd_header {
            ForwardedHeader::RelayNode {
                header,
                mac,
                next_node,
                additional_info,
            } => RelayedPacket {
                packet: MetaPacket::new_from_parts(&alpha.encode(), &header, &mac, &decrypted),
                packet_tag: derive_packet_tag(&secret)?,
                derived_secret: secret.into(),
                next_node,
                additional_info,
            },
            ForwardedHeader::FinalNode { additional_data } => FinalPacket {
                packet_tag: derive_packet_tag(&secret)?,
                derived_secret: secret.into(),
                plain_text: remove_padding(&decrypted)
                    .ok_or(PacketDecodingError("couldn't remove padding".into()))?
                    .into(),
                additional_data,
            },
        })
    }
}

/// Indicates if the packet is supposed to be forwarded to the next hop or if it is intended for us.
#[derive(Debug)]
pub enum PacketState {
    /// Packet is intended for us
    Final {
        packet_tag: Box<[u8]>,
        ack_key: HalfKey,
        previous_hop: OffchainPublicKey,
        plain_text: Box<[u8]>,
    },
    /// Packet must be forwarded
    Forwarded {
        ack_challenge: HalfKeyChallenge,
        packet_tag: Box<[u8]>,
        ack_key: HalfKey,
        previous_hop: OffchainPublicKey,
        own_key: HalfKey,
        own_share: HalfKeyChallenge,
        next_hop: OffchainPublicKey,
        next_challenge: Challenge,
    },
    /// Packet that is being sent out by us
    Outgoing {
        next_hop: OffchainPublicKey,
        ack_challenge: HalfKeyChallenge,
    },
}

/// Represents a HOPR packet
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct Packet {
    packet: MetaPacket,
    pub ticket: Ticket,
    state: PacketState,
}

impl Packet {
    /// Size of the packet including header, payload, ticket and ack challenge.
    pub const SIZE: usize = PACKET_LENGTH + Ticket::SIZE;

    /// Constructs new outgoing packet with the given path.
    /// # Arguments
    /// * `msg` packet payload
    /// * `path` complete path for the packet to take
    /// * `private_key` private key of the local node
    /// * `first_ticket` ticket for the first hop on the path
    pub fn new(msg: &[u8], path: &[&PeerId], channel_private_key: &[u8], mut ticket: Ticket) -> Result<Self> {
        assert!(!path.is_empty(), "path must not be empty");

        let shared_keys = Ed25519SharedKeys::new(path)?;

        let por_values = ProofOfRelayValues::new(shared_keys.secrets[0], shared_keys.secrets.get(1).cloned());

        let por_strings = (1..path.len())
            .map(|i| ProofOfRelayString::new(shared_keys.secrets[i], shared_keys.secrets.get(i + 1).cloned()).to_bytes())
            .collect::<Vec<_>>();

        // Update the ticket with the challenge
        ticket.challenge = por_values.ticket_challenge.to_ethereum_challenge();
        ticket.sign(channel_private_key);

        Ok(Self {
            packet: MetaPacket::new(
                shared_keys,
                msg,
                path,
                INTERMEDIATE_HOPS + 1,
                POR_SECRET_LENGTH,
                &por_strings.iter().map(Box::as_ref).collect::<Vec<_>>(),
                None,
            ),
            ticket,
            state: Outgoing {
                next_hop: OffchainPublicKey::from_peerid(&path[0])?,
                ack_challenge: por_values.ack_challenge,
            },
        })
    }

    /// Deserializes the packet and performs the forward-transformation, so the
    /// packet can be further delivered (relayed to the next hop or read).
    pub fn from_bytes(data: &[u8], node_private_key: &[u8], node_public_key: &OffchainPublicKey, sender: &PeerId) -> Result<Self> {
        if data.len() == Self::SIZE {
            let (pre_packet, pre_ticket) = data.split_at(PACKET_LENGTH);
            let previous_hop = OffchainPublicKey::from_peerid(sender)?;

            let header_len = header_length(INTERMEDIATE_HOPS + 1, POR_SECRET_LENGTH, 0);
            let mp = MetaPacket::from_bytes(pre_packet, OffchainPublicKey::SIZE, header_len)?;

            match mp.forward(node_private_key, node_public_key, INTERMEDIATE_HOPS + 1, POR_SECRET_LENGTH, 0)? {
                RelayedPacket {
                    packet,
                    derived_secret,
                    additional_info,
                    packet_tag,
                    next_node,
                    ..
                } => {
                    let ack_key = derive_ack_key_share(&derived_secret);

                    let ticket = Ticket::from_bytes(pre_ticket)?;
                    let verification_output = pre_verify(&derived_secret, &additional_info, &ticket.challenge)?;
                    Ok(Self {
                        packet,
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
                        },
                    })
                }
                FinalPacket {
                    packet_tag,
                    plain_text,
                    derived_secret,
                    additional_data: _,
                } => {
                    let ack_key = derive_ack_key_share(&derived_secret);

                    let ticket = Ticket::from_bytes(pre_ticket)?;
                    Ok(Self {
                        packet: mp,
                        ticket,
                        state: Final {
                            packet_tag,
                            ack_key,
                            previous_hop,
                            plain_text,
                        },
                    })
                }
            }
        } else {
            Err(PacketDecodingError("packet has invalid size".into()))
        }
    }

    /// State of this packet
    pub fn state(&self) -> &PacketState {
        &self.state
    }

    /// Forwards the packet to the next hop.
    /// Requires private key of the local node and prepared ticket for the next recipient.
    /// Panics if the packet is not meant to be forwarded.
    pub fn forward(&mut self, node_private_key: &[u8], mut next_ticket: Ticket) -> Result<()> {
        match &mut self.state {
            Forwarded {
                next_challenge,
                ..
            } => {
                next_ticket.challenge = next_challenge.to_ethereum_challenge();
                next_ticket.sign(node_private_key);
                self.ticket = next_ticket;
                Ok(())
            }
            _ => Err(InvalidPacketState),
        }
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Packet {
    pub fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(self.packet.to_bytes());
        ret.extend_from_slice(&self.ticket.to_bytes());
        ret.into_boxed_slice()
    }

    /// Creates an acknowledgement for this packet.
    /// Returns None if this packet is sent by us.
    pub fn create_acknowledgement(&self, node_private_key: &[u8]) -> Option<Acknowledgement> {
        match &self.state {
            Final {
                ack_key, ..
            }
            | Forwarded {
                ack_key, ..
            } => Some(Acknowledgement::new(
                ack_key.clone(),
                node_private_key,
            )),
            Outgoing { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::packet::{
        add_padding, remove_padding, ForwardedMetaPacket, MetaPacket, Packet, PacketState, INTERMEDIATE_HOPS,
        PADDING_TAG,
    };
    use crate::por::{ProofOfRelayString, POR_SECRET_LENGTH};
    use core_crypto::types::{OffchainPublicKey, PublicKey, SecretKey};
    use core_types::channels::Ticket;
    use libp2p_identity::PeerId;
    use parameterized::parameterized;
    use core_crypto::ec_groups::Ed25519SharedKeys;
    use utils_types::primitives::{Balance, BalanceType, U256};
    use utils_types::traits::{BinarySerializable, PeerIdLike};

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

    fn generate_keypairs(amount: usize) -> (Vec<SecretKey>, Vec<PeerId>) {
        // TODO: make this compatible with generating random ed25519 PeerIds
        (0..amount)
            .map(|_| OffchainPublicKey::random_keypair())
            .map(|(secret, public)| (secret, public.to_peerid()))
            .unzip()
    }

    #[parameterized(amount = { 4, 3, 2 })]
    fn test_meta_packet(amount: usize) {
        let (secrets, path) = generate_keypairs(amount);
        let shared_keys = Ed25519SharedKeys::new(&path.iter().collect::<Vec<_>>()).unwrap();
        let por_strings = (1..path.len())
            .map(|i| ProofOfRelayString::new(shared_keys.secrets[i], shared_keys.secrets.get(i + 1).cloned()).to_bytes())
            .collect::<Vec<_>>();

        let msg = b"some random test message";

        let mut mp = MetaPacket::new(
            shared_keys,
            msg,
            &path.iter().collect::<Vec<_>>(),
            INTERMEDIATE_HOPS + 1,
            POR_SECRET_LENGTH,
            &por_strings.iter().map(|pors| pors.as_ref()).collect::<Vec<_>>(),
            None,
        );

        for (i, secret) in secrets.iter().enumerate() {
            let pub_key = OffchainPublicKey::from_peerid(&path[i]).unwrap();
            let fwd = mp
                .forward(secret, &pub_key, INTERMEDIATE_HOPS + 1, POR_SECRET_LENGTH, 0)
                .expect(&format!("failed to unwrap at {i}"));
            match fwd {
                ForwardedMetaPacket::RelayedPacket { packet, .. } => {
                    assert!(i < path.len() - 1);
                    mp = packet;
                }
                ForwardedMetaPacket::FinalPacket {
                    plain_text,
                    additional_data,
                    ..
                } => {
                    assert_eq!(path.len() - 1, i);
                    assert_eq!(msg, plain_text.as_ref());
                    assert!(additional_data.is_empty());
                }
            }
        }
    }

    fn mock_ticket(next_peer_channel_key: &PublicKey, path_len: usize, private_key: &[u8]) -> Ticket {
        assert!(path_len > 0);
        const PRICE_PER_PACKET: u128 = 10000000000000000u128;
        const INVERSE_TICKET_WIN_PROB: u128 = 1;

        if path_len > 1 {
            Ticket::new(
                next_peer_channel_key.to_address(),
                U256::zero(),
                U256::zero(),
                Balance::new(
                    (PRICE_PER_PACKET * INVERSE_TICKET_WIN_PROB * (path_len - 1) as u128).into(),
                    BalanceType::HOPR,
                ),
                U256::one(),
                U256::zero(),
                private_key,
            )
        } else {
            Ticket::new_zero_hop(next_peer_channel_key.clone(), private_key)
        }
    }

    #[parameterized(amount = { 4, 3, 2 })]
    fn test_packet_create_and_transform(amount: usize) {
        let (mut node_private_keys, mut path) = generate_keypairs(amount);

        // Generate random channel public keys
        let channel_pub_keys = (0..amount).map(|_| PublicKey::random()).collect::<Vec<_>>();

        let private_key = node_private_keys.drain(..1).last().unwrap();
        let public_key = path.drain(..1).last().unwrap();

        // Create ticket for the first peer on the path
        let ticket = mock_ticket(&channel_pub_keys[0], path.len(), &private_key);

        let test_message = b"some testing message";
        let mut packet = Packet::new(test_message, &path.iter().collect::<Vec<_>>(), &private_key, ticket)
            .expect("failed to construct packet");

        match &packet.state() {
            PacketState::Outgoing { .. } => {}
            _ => panic!("invalid packet initial state"),
        }

        for (i, (node_private, node_id)) in node_private_keys.iter().zip(path.iter()).enumerate() {
            let sender = (i == 0)
                .then_some(&public_key)
                .unwrap_or_else(|| path.get(i - 1).unwrap());

            let pub_key = OffchainPublicKey::from_peerid(node_id).unwrap();
            packet = Packet::from_bytes(&packet.to_bytes(), node_private, &pub_key, &sender)
                .unwrap_or_else(|e| panic!("failed to deserialize packet at hop {i}: {e}"));

            match packet.state() {
                PacketState::Final { plain_text, .. } => {
                    assert_eq!(path.len() - 1, i);
                    assert_eq!(&test_message, &plain_text.as_ref());
                }
                PacketState::Forwarded { .. } => {
                    let ticket = mock_ticket(&channel_pub_keys[i], path.len() - i - 1, node_private);
                    packet.forward(node_private, ticket).unwrap();
                }
                PacketState::Outgoing { .. } => panic!("invalid packet state"),
            }
        }
    }
}

