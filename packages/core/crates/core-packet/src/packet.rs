use crate::errors::PacketError::{InvalidPacketState, PacketDecodingError};
use core_crypto::{
    derivation::{derive_ack_key_share, derive_packet_tag, PacketTag},
    keypairs::{ChainKeypair, Keypair, OffchainKeypair},
    primitives::{DigestLike, SimpleMac},
    prp::{PRPParameters, PRP},
    routing::{forward_header, header_length, ForwardedHeader, RoutingInfo},
    shared_keys::{Alpha, GroupElement, SharedKeys, SharedSecret, SphinxSuite},
    types::{Challenge, HalfKey, HalfKeyChallenge, Hash, OffchainPublicKey},
};
use core_path::path::Path;
use core_types::{acknowledgement::Acknowledgement, channels::Ticket};
use libp2p_identity::PeerId;
use std::fmt::{Display, Formatter};
use typenum::Unsigned;
use utils_types::{
    errors::GeneralError::ParseError,
    traits::{BinarySerializable, PeerIdLike},
};

use crate::{
    errors::Result,
    packet::{
        ForwardedMetaPacket::{FinalPacket, RelayedPacket},
        PacketState::{Final, Forwarded, Outgoing},
    },
    por::{pre_verify, ProofOfRelayString, ProofOfRelayValues, POR_SECRET_LENGTH},
};

/// Currently used ciphersuite for Sphinx
type CurrentSphinxSuite = core_crypto::ec_groups::X25519Suite;

/// Number of intermediate hops: 3 relayers and 1 destination
pub const INTERMEDIATE_HOPS: usize = 3;

/// Maximum size of the packet payload
pub const PAYLOAD_SIZE: usize = 500;

/// Length of the packet including header and the payload
pub const PACKET_LENGTH: usize = packet_length::<CurrentSphinxSuite>(INTERMEDIATE_HOPS + 1, POR_SECRET_LENGTH, 0);

/// Tag used to separate padding from data
const PADDING_TAG: &[u8] = b"HOPR";

/// Determines the total length (header + payload) of the packet given the header information.
pub const fn packet_length<S: SphinxSuite>(
    max_hops: usize,
    additional_data_relayer_len: usize,
    additional_data_last_hop_len: usize,
) -> usize {
    <S::P as Keypair>::Public::SIZE
        + header_length::<S>(max_hops, additional_data_relayer_len, additional_data_last_hop_len)
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

struct MetaPacket<S: SphinxSuite> {
    packet: Box<[u8]>,
    alpha: Alpha<<S::G as GroupElement<S::E>>::AlphaLen>,
    header_len: usize,
}

#[allow(dead_code)]
enum ForwardedMetaPacket<S: SphinxSuite> {
    RelayedPacket {
        packet: MetaPacket<S>,
        next_node: <S::P as Keypair>::Public,
        path_pos: u8,
        additional_info: Box<[u8]>,
        derived_secret: SharedSecret,
        packet_tag: PacketTag,
    },
    FinalPacket {
        plain_text: Box<[u8]>,
        additional_data: Box<[u8]>,
        derived_secret: SharedSecret,
        packet_tag: PacketTag,
    },
}

impl<S: SphinxSuite> MetaPacket<S> {
    pub fn new(
        shared_keys: SharedKeys<S::E, S::G>,
        msg: &[u8],
        path: &[<S::P as Keypair>::Public],
        max_hops: usize,
        additional_relayer_data_len: usize,
        additional_data_relayer: &[&[u8]],
        additional_data_last_hop: Option<&[u8]>,
    ) -> Self {
        assert!(msg.len() <= PAYLOAD_SIZE, "message too long to fit into a packet");

        let mut padded = add_padding(msg);
        let routing_info = RoutingInfo::new::<S>(
            max_hops,
            path,
            &shared_keys.secrets,
            additional_relayer_data_len,
            additional_data_relayer,
            additional_data_last_hop,
        );

        // Encrypt packet payload using the derived shared secrets
        for secret in shared_keys.secrets.iter().rev() {
            let prp = PRP::from_parameters(PRPParameters::new(secret));
            prp.forward_inplace(&mut padded)
                .unwrap_or_else(|e| panic!("onion encryption error {e}"))
        }

        Self::new_from_parts(
            shared_keys.alpha,
            &routing_info.routing_information,
            &routing_info.mac,
            &padded,
        )
    }

    fn new_from_parts(
        alpha: Alpha<<S::G as GroupElement<S::E>>::AlphaLen>,
        routing_info: &[u8],
        mac: &[u8],
        payload: &[u8],
    ) -> Self {
        assert!(!routing_info.is_empty(), "routing info must not be empty");
        assert_eq!(SimpleMac::SIZE, mac.len(), "mac has incorrect length");
        assert_eq!(PAYLOAD_SIZE, payload.len(), "payload has incorrect length");

        let mut packet = Vec::with_capacity(Self::size(routing_info.len()));
        packet.extend_from_slice(&alpha);
        packet.extend_from_slice(routing_info);
        packet.extend_from_slice(mac);
        packet.extend_from_slice(payload);

        Self {
            packet: packet.into_boxed_slice(),
            header_len: routing_info.len(),
            alpha,
        }
    }

    pub fn routing_info(&self) -> &[u8] {
        let base = <S::G as GroupElement<S::E>>::AlphaLen::USIZE;
        &self.packet[base..base + self.header_len]
    }

    pub fn mac(&self) -> &[u8] {
        let base = <S::G as GroupElement<S::E>>::AlphaLen::USIZE + self.header_len;
        &self.packet[base..base + SimpleMac::SIZE]
    }

    pub fn payload(&self) -> &[u8] {
        let base = <S::G as GroupElement<S::E>>::AlphaLen::USIZE + self.header_len + SimpleMac::SIZE;
        &self.packet[base..base + PAYLOAD_SIZE]
    }

    pub const fn size(header_len: usize) -> usize {
        <S::G as GroupElement<S::E>>::AlphaLen::USIZE + header_len + SimpleMac::SIZE + PAYLOAD_SIZE
    }

    pub fn from_bytes(packet: &[u8], header_len: usize) -> utils_types::errors::Result<Self> {
        if packet.len() == Self::size(header_len) {
            let mut ret = Self {
                packet: packet.into(),
                header_len,
                alpha: Default::default(),
            };
            ret.alpha
                .copy_from_slice(&packet[..<S::G as GroupElement<S::E>>::AlphaLen::USIZE]);
            Ok(ret)
        } else {
            Err(ParseError)
        }
    }

    pub fn to_bytes(&self) -> &[u8] {
        &self.packet
    }

    pub fn forward(
        &self,
        node_keypair: &S::P,
        max_hops: usize,
        additional_data_relayer_len: usize,
        additional_data_last_hop_len: usize,
    ) -> Result<ForwardedMetaPacket<S>> {
        let (alpha, secret) = SharedKeys::<S::E, S::G>::forward_transform(
            &self.alpha,
            &(node_keypair.into()),
            &(node_keypair.public().into()),
        )?;

        let mut routing_info_cpy: Vec<u8> = self.routing_info().into();
        let fwd_header = forward_header::<S>(
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
                path_pos,
                next_node,
                additional_info,
            } => RelayedPacket {
                packet: Self::new_from_parts(alpha, &header, &mac, &decrypted),
                packet_tag: derive_packet_tag(&secret),
                derived_secret: secret,
                next_node: <S::P as Keypair>::Public::from_bytes(&next_node)
                    .map_err(|_| PacketDecodingError("couldn't parse next node id".into()))?,
                path_pos,
                additional_info,
            },
            ForwardedHeader::FinalNode { additional_data } => FinalPacket {
                packet_tag: derive_packet_tag(&secret),
                derived_secret: secret,
                plain_text: remove_padding(&decrypted)
                    .ok_or(PacketDecodingError(format!(
                        "couldn't remove padding: {}",
                        hex::encode(decrypted.as_ref())
                    )))?
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
        packet_tag: PacketTag,
        ack_key: HalfKey,
        previous_hop: OffchainPublicKey,
        plain_text: Box<[u8]>,
    },
    /// Packet must be forwarded
    Forwarded {
        ack_challenge: HalfKeyChallenge,
        packet_tag: PacketTag,
        ack_key: HalfKey,
        previous_hop: OffchainPublicKey,
        own_key: HalfKey,
        own_share: HalfKeyChallenge,
        next_hop: OffchainPublicKey,
        next_challenge: Challenge,
        path_pos: u8,
    },
    /// Packet that is being sent out by us
    Outgoing {
        next_hop: OffchainPublicKey,
        ack_challenge: HalfKeyChallenge,
    },
}

impl Display for PacketState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Final { .. } => write!(f, "Final"),
            Forwarded { .. } => write!(f, "Forwarded"),
            Outgoing { .. } => write!(f, "Outgoing"),
        }
    }
}

/// Represents a HOPR packet.
/// Packet also defines the conversion between peer ids, off-chain public keys and group elements from Sphinx suite.
pub struct Packet {
    packet: MetaPacket<CurrentSphinxSuite>,
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
    pub fn new(
        msg: &[u8],
        path: &Path,
        chain_keypair: &ChainKeypair,
        mut ticket: Ticket,
        domain_separator: &Hash,
    ) -> Result<Self> {
        let public_keys_path: Vec<OffchainPublicKey> = path.try_into()?;

        let shared_keys = CurrentSphinxSuite::new_shared_keys(&public_keys_path)?;
        let por_values = ProofOfRelayValues::new(&shared_keys.secrets[0], shared_keys.secrets.get(1));
        let por_strings = ProofOfRelayString::from_shared_secrets(&shared_keys.secrets);

        // Update the ticket with the challenge
        ticket.challenge = por_values.ticket_challenge.to_ethereum_challenge();
        ticket.sign(chain_keypair, domain_separator);

        Ok(Self {
            packet: MetaPacket::new(
                shared_keys,
                msg,
                &public_keys_path,
                INTERMEDIATE_HOPS + 1,
                POR_SECRET_LENGTH,
                &por_strings.iter().map(Box::as_ref).collect::<Vec<_>>(),
                None,
            ),
            ticket,
            state: Outgoing {
                next_hop: OffchainPublicKey::from_peerid(&path.hops()[0])?,
                ack_challenge: por_values.ack_challenge,
            },
        })
    }

    /// Deserializes the packet and performs the forward-transformation, so the
    /// packet can be further delivered (relayed to the next hop or read).
    pub fn from_bytes(data: &[u8], node_keypair: &OffchainKeypair, sender: &PeerId) -> Result<Self> {
        if data.len() == Self::SIZE {
            let (pre_packet, pre_ticket) = data.split_at(PACKET_LENGTH);
            let previous_hop = OffchainPublicKey::from_peerid(sender)?;

            let header_len = header_length::<CurrentSphinxSuite>(INTERMEDIATE_HOPS + 1, POR_SECRET_LENGTH, 0);
            let mp = MetaPacket::<CurrentSphinxSuite>::from_bytes(pre_packet, header_len)?;

            match mp.forward(node_keypair, INTERMEDIATE_HOPS + 1, POR_SECRET_LENGTH, 0)? {
                RelayedPacket {
                    packet,
                    derived_secret,
                    additional_info,
                    packet_tag,
                    next_node,
                    path_pos,
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
                            path_pos,
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
    pub fn forward(
        &mut self,
        chain_keypair: &ChainKeypair,
        mut next_ticket: Ticket,
        domain_separator: &Hash,
    ) -> Result<()> {
        match &mut self.state {
            Forwarded { next_challenge, .. } => {
                next_ticket.challenge = next_challenge.to_ethereum_challenge();
                next_ticket.sign(chain_keypair, domain_separator);
                self.ticket = next_ticket;
                Ok(())
            }
            _ => Err(InvalidPacketState),
        }
    }
}

impl Packet {
    pub fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(self.packet.to_bytes());
        ret.extend_from_slice(&self.ticket.to_bytes());
        ret.into_boxed_slice()
    }

    /// Creates an acknowledgement for this packet.
    /// Returns None if this packet is sent by us.
    pub fn create_acknowledgement(&self, node_keypair: &OffchainKeypair) -> Option<Acknowledgement> {
        match &self.state {
            Final { ack_key, .. } | Forwarded { ack_key, .. } => {
                Some(Acknowledgement::new(ack_key.clone(), node_keypair))
            }
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
    use core_crypto::{
        ec_groups::{Ed25519Suite, Secp256k1Suite, X25519Suite},
        keypairs::{ChainKeypair, Keypair, OffchainKeypair},
        shared_keys::SphinxSuite,
        types::{Hash, PublicKey},
    };
    use core_path::path::Path;
    use core_types::channels::Ticket;
    use parameterized::parameterized;
    use utils_types::{
        primitives::{Balance, BalanceType, EthereumChallenge, U256},
        traits::PeerIdLike,
    };

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

    fn generic_test_meta_packet<S: SphinxSuite>(keypairs: Vec<S::P>) {
        let pubkeys = keypairs.iter().map(|kp| kp.public().clone()).collect::<Vec<_>>();

        let shared_keys = S::new_shared_keys(&pubkeys).unwrap();
        let por_strings = ProofOfRelayString::from_shared_secrets(&shared_keys.secrets);

        assert_eq!(shared_keys.secrets.len() - 1, por_strings.len());

        let msg = b"some random test message";

        let mut mp = MetaPacket::<S>::new(
            shared_keys,
            msg,
            &pubkeys,
            INTERMEDIATE_HOPS + 1,
            POR_SECRET_LENGTH,
            &por_strings.iter().map(Box::as_ref).collect::<Vec<_>>(),
            None,
        );

        for (i, pair) in keypairs.iter().enumerate() {
            let fwd = mp
                .forward(pair, INTERMEDIATE_HOPS + 1, POR_SECRET_LENGTH, 0)
                .expect(&format!("failed to unwrap at {i}"));

            match fwd {
                ForwardedMetaPacket::RelayedPacket { packet, .. } => {
                    assert!(i < keypairs.len() - 1);
                    mp = packet;
                }
                ForwardedMetaPacket::FinalPacket {
                    plain_text,
                    additional_data,
                    ..
                } => {
                    assert_eq!(keypairs.len() - 1, i);
                    assert_eq!(msg, plain_text.as_ref());
                    assert!(additional_data.is_empty());
                }
            }
        }
    }

    #[parameterized(amount = { 4, 3, 2 })]
    fn test_ed25519_meta_packet(amount: usize) {
        generic_test_meta_packet::<Ed25519Suite>((0..amount).map(|_| OffchainKeypair::random()).collect());
    }

    #[parameterized(amount = { 4, 3, 2 })]
    fn test_x25519_meta_packet(amount: usize) {
        generic_test_meta_packet::<X25519Suite>((0..amount).map(|_| OffchainKeypair::random()).collect())
    }

    #[parameterized(amount = { 4, 3, 2 })]
    fn test_secp256k1_meta_packet(amount: usize) {
        generic_test_meta_packet::<Secp256k1Suite>((0..amount).map(|_| ChainKeypair::random()).collect())
    }

    fn mock_ticket(next_peer_channel_key: &PublicKey, path_len: usize, private_key: &ChainKeypair) -> Ticket {
        assert!(path_len > 0);
        let price_per_packet: U256 = 10000000000000000u128.into();
        let ticket_win_prob = 1.0f64;

        if path_len > 1 {
            Ticket::new(
                &next_peer_channel_key.to_address(),
                &Balance::new(
                    price_per_packet.divide_f64(ticket_win_prob).unwrap() * U256::from(path_len as u64 - 1),
                    BalanceType::HOPR,
                ),
                1u64.into(),
                1u64.into(),
                ticket_win_prob,
                1u64.into(),
                EthereumChallenge::default(),
                private_key,
                &Hash::default(),
            )
            .unwrap()
        } else {
            Ticket::new_zero_hop(&next_peer_channel_key.to_address(), private_key, &Hash::default())
        }
    }

    #[parameterized(amount = { 4, 3, 2 })]
    fn test_packet_create_and_transform(amount: usize) {
        // Generate random path with node private keys
        let mut keypairs = (0..amount).map(|_| OffchainKeypair::random()).collect::<Vec<_>>();

        // Generate random channel public keys
        let mut channel_pairs = (0..amount).map(|_| ChainKeypair::random()).collect::<Vec<_>>();

        let own_channel_kp = channel_pairs.drain(..1).last().unwrap();
        let own_packet_kp = keypairs.drain(..1).last().unwrap();

        // Create ticket for the first peer on the path
        let ticket = mock_ticket(&channel_pairs[0].public().0, keypairs.len(), &own_channel_kp);

        let test_message = b"some testing message";
        let path = Path::new_valid(keypairs.iter().map(|kp| kp.public().to_peerid()).collect());
        let mut packet = Packet::new(test_message, &path, &own_channel_kp, ticket, &Hash::default())
            .expect("failed to construct packet");

        match &packet.state() {
            PacketState::Outgoing { .. } => {}
            _ => panic!("invalid packet initial state"),
        }

        for (i, path_element) in keypairs.iter().enumerate() {
            let sender = (i == 0)
                .then_some(own_packet_kp.public().to_peerid())
                .unwrap_or_else(|| keypairs.get(i - 1).map(|kp| kp.public().to_peerid()).unwrap());

            packet = Packet::from_bytes(&packet.to_bytes(), path_element, &sender)
                .unwrap_or_else(|e| panic!("failed to deserialize packet at hop {i}: {e}"));

            match packet.state() {
                PacketState::Final { plain_text, .. } => {
                    assert_eq!(keypairs.len() - 1, i);
                    assert_eq!(&test_message, &plain_text.as_ref());
                }
                PacketState::Forwarded { .. } => {
                    let ticket = mock_ticket(
                        &channel_pairs[i + 1].public().0,
                        keypairs.len() - i - 1,
                        &channel_pairs[i],
                    );
                    packet.forward(&channel_pairs[i], ticket, &Hash::default()).unwrap();
                }
                PacketState::Outgoing { .. } => panic!("invalid packet state"),
            }
        }
    }
}
