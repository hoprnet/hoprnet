use std::fmt::{Display, Formatter};

use hopr_crypto_sphinx::{derivation::derive_ack_key_share, shared_keys::SphinxSuite};
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

use crate::{
    errors::{PacketError::PacketDecodingError, Result},
    packet::{ForwardedMetaPacket, MetaPacket},
    por::{pre_verify, ProofOfRelayString, ProofOfRelayValues, POR_SECRET_LENGTH},
    CurrentSphinxSuite,
};

/// Indicates the packet type.
#[allow(clippy::large_enum_variant)] // TODO: see if some parts can be boxed
#[derive(Debug, Clone)]
pub enum ChainPacketComponents {
    /// Packet is intended for us
    Final {
        packet_tag: PacketTag,
        ack_key: HalfKey,
        previous_hop: OffchainPublicKey,
        plain_text: Box<[u8]>,
    },
    /// Packet must be forwarded
    Forwarded {
        packet: MetaPacket<CurrentSphinxSuite>,
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
        packet: MetaPacket<CurrentSphinxSuite>,
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
    /// Size of the packet including header, padded payload, ticket and ack challenge.
    pub const SIZE: usize = MetaPacket::<CurrentSphinxSuite>::PACKET_LEN + Ticket::SIZE;

    /// Constructs a new outgoing packet with the given path.
    /// # Arguments
    /// * `msg` packet payload
    /// * `public_keys_path` public keys of a complete path for the packet to take
    /// * `private_key` private key of the local node
    /// * `ticket` ticket for the first hop on the path
    /// * `domain_separator`
    pub fn into_outgoing(
        msg: &[u8],
        public_keys_path: &[OffchainPublicKey],
        chain_keypair: &ChainKeypair,
        ticket: TicketBuilder,
        domain_separator: &Hash,
    ) -> Result<Self> {
        let shared_keys = CurrentSphinxSuite::new_shared_keys(public_keys_path)?;
        let por_values = ProofOfRelayValues::new(&shared_keys.secrets[0], shared_keys.secrets.get(1));
        let por_strings = ProofOfRelayString::from_shared_secrets(&shared_keys.secrets);

        // Update the ticket with the challenge
        let ticket = ticket
            .challenge(por_values.ticket_challenge.to_ethereum_challenge())
            .build_signed(chain_keypair, domain_separator)?
            .leak();

        Ok(Self::Outgoing {
            packet: MetaPacket::<CurrentSphinxSuite>::new(
                shared_keys,
                msg,
                public_keys_path,
                INTERMEDIATE_HOPS + 1,
                POR_SECRET_LENGTH,
                &por_strings.iter().map(|s| s.as_ref()).collect::<Vec<_>>(),
                None,
            ),
            ticket,
            next_hop: public_keys_path[0],
            ack_challenge: por_values.ack_challenge,
        })
    }

    /// Deserializes the packet and performs the forward-transformation, so the
    /// packet can be further delivered (relayed to the next hop or read).
    pub fn from_incoming(data: &[u8], node_keypair: &OffchainKeypair, previous_hop: OffchainPublicKey) -> Result<Self> {
        if data.len() == Self::SIZE {
            let (pre_packet, pre_ticket) = data.split_at(MetaPacket::<CurrentSphinxSuite>::PACKET_LEN);

            let mp: MetaPacket<CurrentSphinxSuite> = MetaPacket::try_from(pre_packet)?;

            match mp.into_forwarded(node_keypair, INTERMEDIATE_HOPS + 1, POR_SECRET_LENGTH, 0)? {
                ForwardedMetaPacket::Relayed {
                    packet,
                    derived_secret,
                    additional_info,
                    packet_tag,
                    next_node,
                    path_pos,
                    ..
                } => {
                    let ack_key = derive_ack_key_share(&derived_secret);

                    let ticket = Ticket::try_from(pre_ticket)?;
                    let verification_output = pre_verify(&derived_secret, &additional_info, &ticket.challenge)?;
                    Ok(Self::Forwarded {
                        packet,
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
                ForwardedMetaPacket::Final {
                    packet_tag,
                    plain_text,
                    derived_secret,
                    additional_data: _,
                } => {
                    let ack_key = derive_ack_key_share(&derived_secret);
                    // This ticket is not parsed nor verified on the final hop
                    Ok(Self::Final {
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
    next_ticket: TicketBuilder,
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
            let ticket = next_ticket
                .challenge(next_challenge.to_ethereum_challenge())
                .build_signed(chain_keypair, domain_separator)
                .expect("ticket should create")
                .leak();
            ChainPacketComponents::Forwarded {
                packet,
                ticket,
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

#[cfg(test)]
mod tests {
    use super::ChainPacketComponents;
    use async_trait::async_trait;
    use hex_literal::hex;
    use hopr_crypto_types::prelude::*;
    use hopr_internal_types::prelude::*;
    use hopr_path::channel_graph::ChannelGraph;
    use hopr_path::path::TransportPath;
    use hopr_primitive_types::prelude::*;
    use libp2p_identity::PeerId;
    use parameterized::parameterized;

    lazy_static::lazy_static! {
        static ref PEERS_ONCHAIN: [ChainKeypair; 4] = [
            hex!("a7c486ceccf5ab53bd428888ab1543dc2667abd2d5e80aae918da8d4b503a426"),
            hex!("9a82976f7182c05126313bead5617c623b93d11f9f9691c87b1a26f869d569ed"),
            hex!("ca4bdfd54a8467b5283a0216288fdca7091122479ccf3cfb147dfa59d13f3486"),
            hex!("e306ebfb0d01d0da0952c9a567d758093a80622c6cb55052bf5f1a6ebd8d7b5c")
        ].map(|privkey| ChainKeypair::from_secret(&privkey).expect("lazy static keypair should be valid"));

        static ref PEERS_OFFCHAIN: [OffchainKeypair; 4] = [
            hex!("5eb212d4d6aa5948c4f71574d45dad43afef6d330edb873fca69d0e1b197e906"),
            hex!("e995db483ada5174666c46bafbf3628005aca449c94ebdc0c9239c3f65d61ae0"),
            hex!("9dec751c00f49e50fceff7114823f726a0425a68a8dc6af0e4287badfea8f4a4"),
            hex!("9a82976f7182c05126313bead5617c623b93d11f9f9691c87b1a26f869d569ed")
        ].map(|privkey| OffchainKeypair::from_secret(&privkey).expect("lazy static keypair should be valid"));
    }

    impl ChainPacketComponents {
        pub fn to_bytes(&self) -> Box<[u8]> {
            let dummy_ticket = hex!("67f0ca18102feec505e5bfedcc25963e9c64a6f8a250adcad7d2830dd607585700000000000000000000000000000000000000000000000000000000000000003891bf6fd4a78e868fc7ad477c09b16fc70dd01ea67e18264d17e3d04f6d8576de2e6472b0072e510df6e9fa1dfcc2727cc7633edfeb9ec13860d9ead29bee71d68de3736c2f7a9f42de76ccd57a5f5847bc7349");
            let (packet, ticket) = match self {
                Self::Final { plain_text, .. } => (plain_text.clone(), dummy_ticket.as_ref().into()),
                Self::Forwarded { packet, ticket, .. } => (
                    Vec::from(packet.as_ref()).into_boxed_slice(),
                    ticket.clone().into_boxed(),
                ),
                Self::Outgoing { packet, ticket, .. } => (
                    Vec::from(packet.as_ref()).into_boxed_slice(),
                    ticket.clone().into_boxed(),
                ),
            };

            let mut ret = Vec::with_capacity(Self::SIZE);
            ret.extend_from_slice(packet.as_ref());
            ret.extend_from_slice(&ticket);
            ret.into_boxed_slice()
        }
    }

    fn mock_ticket(
        next_peer_channel_key: &PublicKey,
        path_len: usize,
        private_key: &ChainKeypair,
    ) -> anyhow::Result<TicketBuilder> {
        assert!(path_len > 0);
        let price_per_packet: U256 = 10000000000000000u128.into();

        if path_len > 1 {
            Ok(TicketBuilder::default()
                .direction(&private_key.public().to_address(), &next_peer_channel_key.to_address())
                .amount(price_per_packet.div_f64(1.0)? * U256::from(path_len as u64 - 1))
                .index(1)
                .index_offset(1)
                .win_prob(1.0)
                .channel_epoch(1)
                .challenge(Default::default()))
        } else {
            Ok(TicketBuilder::zero_hop()
                .direction(&private_key.public().to_address(), &next_peer_channel_key.to_address()))
        }
    }
    async fn resolve_mock_path(
        me: Address,
        peers_offchain: Vec<PeerId>,
        peers_onchain: Vec<Address>,
    ) -> anyhow::Result<TransportPath> {
        let peers_addrs = peers_offchain
            .iter()
            .zip(peers_onchain)
            .map(|(peer_id, addr)| {
                (
                    OffchainPublicKey::try_from(peer_id).expect("PeerId should be convertible to offchain public key"),
                    addr,
                )
            })
            .collect::<Vec<_>>();
        let mut cg = ChannelGraph::new(me, Default::default());
        let mut last_addr = cg.my_address();
        for (_, addr) in peers_addrs.iter() {
            let c = ChannelEntry::new(
                last_addr,
                *addr,
                Balance::new(1000_u32, BalanceType::HOPR),
                0u32.into(),
                ChannelStatus::Open,
                0u32.into(),
            );
            cg.update_channel(c);
            last_addr = *addr;
        }

        struct TestResolver(Vec<(OffchainPublicKey, Address)>);

        #[async_trait]
        impl hopr_db_api::resolver::HoprDbResolverOperations for TestResolver {
            async fn resolve_packet_key(
                &self,
                onchain_key: &Address,
            ) -> hopr_db_api::errors::Result<Option<OffchainPublicKey>> {
                Ok(self.0.iter().find(|(_, addr)| addr.eq(onchain_key)).map(|(pk, _)| *pk))
            }

            async fn resolve_chain_key(
                &self,
                offchain_key: &OffchainPublicKey,
            ) -> hopr_db_api::errors::Result<Option<Address>> {
                Ok(self.0.iter().find(|(pk, _)| pk.eq(offchain_key)).map(|(_, addr)| *addr))
            }
        }

        Ok(TransportPath::resolve(peers_offchain, &TestResolver(peers_addrs), &cg)
            .await
            .map_err(|_e| hopr_db_api::errors::DbError::General("failed to generate a transport path".into()))?
            .0)
    }

    #[parameterized(amount = { 4, 3, 2 })]
    fn test_packet_create_and_transform(amount: usize) -> anyhow::Result<()> {
        let mut keypairs_offchain = Vec::from_iter(PEERS_OFFCHAIN[0..amount].iter());
        let mut keypairs_onchain = Vec::from_iter(PEERS_ONCHAIN[0..amount].iter());

        let own_channel_kp = keypairs_onchain
            .drain(..1)
            .last()
            .expect("should have at least one onchain keypair");
        let own_packet_kp = keypairs_offchain
            .drain(..1)
            .last()
            .expect("should have at least one offchain keypair");

        // Create ticket for the first peer on the path
        let ticket = mock_ticket(
            &keypairs_onchain[0].public().0,
            keypairs_offchain.len(),
            &own_channel_kp,
        )?;

        let test_message = b"some testing message";
        let path = async_std::task::block_on(resolve_mock_path(
            own_channel_kp.public().to_address(),
            keypairs_offchain.iter().map(|kp| kp.public().into()).collect(),
            keypairs_onchain.iter().map(|kp| kp.public().to_address()).collect(),
        ))?;

        let path = hopr_path::path::Path::hops(&path)
            .iter()
            .map(|v| OffchainPublicKey::try_from(v))
            .collect::<hopr_primitive_types::errors::Result<Vec<_>>>()?;

        let mut packet =
            ChainPacketComponents::into_outgoing(test_message, &path, &own_channel_kp, ticket, &Hash::default())?;

        match &packet {
            ChainPacketComponents::Outgoing { .. } => {}
            _ => panic!("invalid packet initial state"),
        }

        for (i, path_element) in keypairs_offchain.iter().enumerate() {
            let sender = (i == 0)
                .then_some(own_packet_kp)
                .unwrap_or_else(|| keypairs_offchain.get(i - 1).expect("should have previous keypair"))
                .public()
                .clone();

            packet = ChainPacketComponents::from_incoming(&packet.to_bytes(), path_element, sender)
                .unwrap_or_else(|e| panic!("failed to deserialize packet at hop {i}: {e}"));

            match &packet {
                ChainPacketComponents::Final { plain_text, .. } => {
                    assert_eq!(keypairs_offchain.len() - 1, i);
                    assert_eq!(&test_message, &plain_text.as_ref());
                }
                ChainPacketComponents::Forwarded { .. } => {
                    let ticket = mock_ticket(
                        &keypairs_onchain[i + 1].public().0,
                        keypairs_offchain.len() - i - 1,
                        &keypairs_onchain[i],
                    )?;
                    packet = super::super::forward(packet.clone(), &keypairs_onchain[i], ticket, &Hash::default());
                }
                ChainPacketComponents::Outgoing { .. } => panic!("invalid packet state"),
            }
        }

        Ok(())
    }
}
