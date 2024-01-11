use hopr_crypto::{
    derivation::{derive_ack_key_share, PacketTag},
    keypairs::{ChainKeypair, OffchainKeypair},
    shared_keys::SphinxSuite,
    types::{Challenge, HalfKey, HalfKeyChallenge, Hash, OffchainPublicKey},
};
use core_packet::{
    errors::{PacketError::PacketDecodingError, Result},
    packet::{CurrentSphinxSuite, ForwardedMetaPacket, MetaPacket, PACKET_LENGTH},
    por::{pre_verify, ProofOfRelayString, ProofOfRelayValues, POR_SECRET_LENGTH},
};
use core_path::path::{Path, TransportPath};
use core_types::channels::Ticket;
use core_types::protocol::INTERMEDIATE_HOPS;
use libp2p_identity::PeerId;
use std::fmt::{Display, Formatter};
use utils_types::traits::{BinarySerializable, PeerIdLike};

/// Indicates the packet type.
#[derive(Debug, Clone)]
pub enum ChainPacketComponents {
    /// Packet is intended for us
    Final {
        packet: Box<[u8]>,
        ticket: Ticket,
        packet_tag: PacketTag,
        ack_key: HalfKey,
        previous_hop: OffchainPublicKey,
        plain_text: Box<[u8]>,
    },
    /// Packet must be forwarded
    Forwarded {
        packet: Box<[u8]>,
        ticket: Ticket,
        ack_challenge: HalfKeyChallenge,
        packet_tag: PacketTag,
        ack_key: HalfKey,
        previous_hop: OffchainPublicKey,
        own_key: HalfKey,
        next_hop: OffchainPublicKey,
        next_challenge: Challenge,
        path_pos: u8,
    },
    /// Packet that is being sent out by us
    Outgoing {
        packet: Box<[u8]>,
        ticket: Ticket,
        next_hop: OffchainPublicKey,
        ack_challenge: HalfKeyChallenge,
    },
}

impl Display for ChainPacketComponents {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Final { .. } => write!(f, "Final"),
            Self::Forwarded { .. } => write!(f, "Forwarded"),
            Self::Outgoing { .. } => write!(f, "Outgoing"),
        }
    }
}

impl ChainPacketComponents {
    /// Size of the packet including header, payload, ticket and ack challenge.
    pub const SIZE: usize = PACKET_LENGTH + Ticket::SIZE;

    /// Constructs new outgoing packet with the given path.
    /// # Arguments
    /// * `msg` packet payload
    /// * `path` complete path for the packet to take
    /// * `private_key` private key of the local node
    /// * `first_ticket` ticket for the first hop on the path
    pub fn into_outgoing(
        msg: &[u8],
        path: &TransportPath,
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

        Ok(Self::Outgoing {
            packet: MetaPacket::<CurrentSphinxSuite>::new(
                shared_keys,
                msg,
                &public_keys_path,
                INTERMEDIATE_HOPS + 1,
                POR_SECRET_LENGTH,
                &por_strings.iter().map(Box::as_ref).collect::<Vec<_>>(),
                None,
            )
            .to_bytes(),
            ticket,
            next_hop: OffchainPublicKey::from_peerid(&path.hops()[0])?,
            ack_challenge: por_values.ack_challenge,
        })
    }

    /// Deserializes the packet and performs the forward-transformation, so the
    /// packet can be further delivered (relayed to the next hop or read).
    pub fn from_incoming(data: &[u8], node_keypair: &OffchainKeypair, sender: &PeerId) -> Result<Self> {
        if data.len() == Self::SIZE {
            let (pre_packet, pre_ticket) = data.split_at(PACKET_LENGTH);
            let previous_hop = OffchainPublicKey::from_peerid(sender)?;

            let mp: MetaPacket<hopr_crypto::ec_groups::X25519Suite> =
                MetaPacket::<CurrentSphinxSuite>::from_bytes(pre_packet)?;

            match mp.forward(node_keypair, INTERMEDIATE_HOPS + 1, POR_SECRET_LENGTH, 0)? {
                ForwardedMetaPacket::RelayedPacket {
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
                    Ok(Self::Forwarded {
                        packet: packet.to_bytes(),
                        ticket,
                        packet_tag,
                        ack_key,
                        previous_hop,
                        path_pos,
                        own_key: verification_output.own_key,
                        next_hop: next_node,
                        next_challenge: verification_output.next_ticket_challenge,
                        ack_challenge: verification_output.ack_challenge,
                    })
                }
                ForwardedMetaPacket::FinalPacket {
                    packet_tag,
                    plain_text,
                    derived_secret,
                    additional_data: _,
                } => {
                    let ack_key = derive_ack_key_share(&derived_secret);

                    let ticket = Ticket::from_bytes(pre_ticket)?;
                    Ok(Self::Final {
                        packet: mp.to_bytes(),
                        ticket,
                        packet_tag,
                        ack_key,
                        previous_hop,
                        plain_text,
                    })
                }
            }
        } else {
            Err(PacketDecodingError("packet has invalid size".into()))
        }
    }
}

#[allow(dead_code)] // used in tests
pub fn forward(
    packet: ChainPacketComponents,
    chain_keypair: &ChainKeypair,
    mut next_ticket: Ticket,
    domain_separator: &Hash,
) -> ChainPacketComponents {
    match packet {
        ChainPacketComponents::Forwarded {
            next_challenge,
            packet,
            ack_challenge,
            packet_tag,
            ack_key,
            previous_hop,
            own_key,
            next_hop,
            path_pos,
            ..
        } => {
            next_ticket.challenge = next_challenge.to_ethereum_challenge();
            next_ticket.sign(chain_keypair, domain_separator);
            ChainPacketComponents::Forwarded {
                packet,
                ticket: next_ticket,
                ack_challenge,
                packet_tag,
                ack_key,
                previous_hop,
                own_key,
                next_hop,
                next_challenge,
                path_pos,
            }
        }
        _ => packet,
    }
}

impl ChainPacketComponents {
    #[allow(dead_code)] // used in tests
    pub fn to_bytes(&self) -> Box<[u8]> {
        let (packet, ticket) = match self {
            Self::Final { packet, ticket, .. } => (packet, ticket),
            Self::Forwarded { packet, ticket, .. } => (packet, ticket),
            Self::Outgoing { packet, ticket, .. } => (packet, ticket),
        };

        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(packet.as_ref());
        ret.extend_from_slice(&ticket.to_bytes());
        ret.into_boxed_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::ChainPacketComponents;
    use async_trait::async_trait;
    use hopr_crypto::types::OffchainPublicKey;
    use hopr_crypto::{
        keypairs::{ChainKeypair, Keypair, OffchainKeypair},
        types::{Hash, PublicKey},
    };
    use core_path::channel_graph::ChannelGraph;
    use core_path::path::TransportPath;
    use core_types::channels::{ChannelEntry, ChannelStatus, Ticket};
    use core_types::protocol::PeerAddressResolver;
    use libp2p_identity::PeerId;
    use parameterized::parameterized;
    use utils_types::primitives::Address;
    use utils_types::{
        primitives::{Balance, BalanceType, EthereumChallenge, U256},
        traits::PeerIdLike,
    };

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
            Ticket::new_zero_hop(&next_peer_channel_key.to_address(), private_key, &Hash::default()).unwrap()
        }
    }
    async fn resolve_mock_path(peers: Vec<PeerId>) -> TransportPath {
        let peers_addrs = peers
            .iter()
            .map(|p| (OffchainPublicKey::from_peerid(p).unwrap(), Address::random()))
            .collect::<Vec<_>>();
        let mut cg = ChannelGraph::new(Address::random());
        let mut last_addr = cg.my_address();
        for (_, addr) in peers_addrs.iter() {
            let c = ChannelEntry::new(
                last_addr,
                *addr,
                Balance::new(1000u32.into(), BalanceType::HOPR),
                0u32.into(),
                ChannelStatus::Open,
                0u32.into(),
                0u32.into(),
            );
            cg.update_channel(c);
            last_addr = *addr;
        }

        struct TestResolver(Vec<(OffchainPublicKey, Address)>);

        #[async_trait]
        impl PeerAddressResolver for TestResolver {
            async fn resolve_packet_key(&self, onchain_key: &Address) -> Option<OffchainPublicKey> {
                self.0
                    .iter()
                    .find(|(_, addr)| addr.eq(onchain_key))
                    .map(|(pk, _)| pk.clone())
            }

            async fn resolve_chain_key(&self, offchain_key: &OffchainPublicKey) -> Option<Address> {
                self.0.iter().find(|(pk, _)| pk.eq(offchain_key)).map(|(_, addr)| *addr)
            }
        }

        TransportPath::resolve(peers, &TestResolver(peers_addrs), &cg)
            .await
            .unwrap()
            .0
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
        let path = async_std::task::block_on(resolve_mock_path(
            keypairs.iter().map(|kp| kp.public().to_peerid()).collect(),
        ));

        let mut packet =
            ChainPacketComponents::into_outgoing(test_message, &path, &own_channel_kp, ticket, &Hash::default())
                .expect("failed to construct packet");

        match &packet {
            ChainPacketComponents::Outgoing { .. } => {}
            _ => panic!("invalid packet initial state"),
        }

        for (i, path_element) in keypairs.iter().enumerate() {
            let sender = (i == 0)
                .then_some(own_packet_kp.public().to_peerid())
                .unwrap_or_else(|| keypairs.get(i - 1).map(|kp| kp.public().to_peerid()).unwrap());

            packet = ChainPacketComponents::from_incoming(&packet.to_bytes(), path_element, &sender)
                .unwrap_or_else(|e| panic!("failed to deserialize packet at hop {i}: {e}"));

            match &packet {
                ChainPacketComponents::Final { plain_text, .. } => {
                    assert_eq!(keypairs.len() - 1, i);
                    assert_eq!(&test_message, &plain_text.as_ref());
                }
                ChainPacketComponents::Forwarded { .. } => {
                    let ticket = mock_ticket(
                        &channel_pairs[i + 1].public().0,
                        keypairs.len() - i - 1,
                        &channel_pairs[i],
                    );
                    packet = super::super::forward(packet.clone(), &channel_pairs[i], ticket, &Hash::default());
                }
                ChainPacketComponents::Outgoing { .. } => panic!("invalid packet state"),
            }
        }
    }
}
