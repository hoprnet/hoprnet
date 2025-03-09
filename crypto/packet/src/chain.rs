use hopr_crypto_sphinx::surb::{create_surb, LocalSURBEntry, SURB};
use hopr_crypto_sphinx::{derivation::derive_ack_key_share, shared_keys::SphinxSuite};
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use std::fmt::{Display, Formatter};

use crate::errors::PacketError;
use crate::errors::PacketError::PacketConstructionError;
use crate::packet::{KeyIdMapper, MetaPacketRouting};
use crate::{
    errors::{PacketError::PacketDecodingError, Result},
    packet::{ForwardedMetaPacket, MetaPacket},
    por::{pre_verify, ProofOfRelayString, ProofOfRelayValues},
    CurrentSphinxSuite, HoprPseudonym, HoprSphinxHeaderSpec, HoprSurb,
};
use hopr_crypto_sphinx::surb::SphinxRecipientMessage;

/// Indicates the packet type.
#[allow(clippy::large_enum_variant)] // TODO: see if some parts can be boxed
#[derive(Clone)]
pub enum ChainPacketComponents {
    /// The packet is intended for us
    Final {
        packet_tag: PacketTag,
        ack_key: HalfKey,
        previous_hop: OffchainPublicKey,
        plain_text: Option<Box<[u8]>>,
        surbs: Vec<(HoprPseudonym, SURB<CurrentSphinxSuite, HoprSphinxHeaderSpec>)>,
    },
    /// The packet must be forwarded
    Forwarded {
        packet: MetaPacket<CurrentSphinxSuite, HoprSphinxHeaderSpec>,
        ticket: Ticket,
        ack_challenge: HalfKeyChallenge,
        packet_tag: PacketTag,
        ack_key: HalfKey,
        previous_hop: OffchainPublicKey,
        own_key: HalfKey,
        next_hop: OffchainPublicKey,
        next_challenge: EthereumChallenge,
        path_pos: u8,
    },
    /// The packet that is being sent out by us
    Outgoing {
        packet: MetaPacket<CurrentSphinxSuite, HoprSphinxHeaderSpec>,
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

/// Determines options on how HOPR packet can be routed to its destination.
#[derive(Clone)]
pub enum PacketRouting<'a> {
    /// The packet is routed directly via the given path.
    ForwardPath(&'a [OffchainPublicKey]),
    /// The packet is routed via an existing SURB.
    Surb(HoprSurb),
}

impl ChainPacketComponents {
    /// The size of the packet including header, padded payload, ticket, and ack challenge.
    pub const SIZE: usize = MetaPacket::<CurrentSphinxSuite, HoprSphinxHeaderSpec>::PACKET_LEN + Ticket::SIZE;

    fn into_raw_msg<M: KeyIdMapper<CurrentSphinxSuite, HoprSphinxHeaderSpec>>(
        msg: &[u8],
        msg_type: SphinxRecipientMessage<HoprPseudonym>,
        routing: PacketRouting,
        chain_keypair: &ChainKeypair,
        ticket: TicketBuilder,
        mapper: &M,
        domain_separator: &Hash,
    ) -> Result<Self> {
        match routing {
            PacketRouting::ForwardPath(forward_path) => {
                let shared_keys = CurrentSphinxSuite::new_shared_keys(forward_path)?;
                let por_values = ProofOfRelayValues::new(&shared_keys.secrets[0], shared_keys.secrets.get(1))?;
                let por_strings = ProofOfRelayString::from_shared_secrets(&shared_keys.secrets)?;

                // Update the ticket with the challenge
                let ticket = ticket
                    .challenge(por_values.ticket_challenge())
                    .build_signed(chain_keypair, domain_separator)?
                    .leak();

                Ok(Self::Outgoing {
                    packet: MetaPacket::<CurrentSphinxSuite, HoprSphinxHeaderSpec>::new(
                        msg,
                        MetaPacketRouting::ForwardPath {
                            shared_keys,
                            forward_path,
                            additional_data_relayer: &por_strings,
                            additional_data_last_hop: msg_type.into(),
                        },
                        mapper,
                    )?,
                    ticket,
                    next_hop: forward_path[0],
                    ack_challenge: por_values.acknowledgement_challenge(),
                })
            }
            PacketRouting::Surb(surb) => {
                // Update the ticket with the challenge
                let ticket = ticket
                    .challenge(surb.additional_data_receiver.ticket_challenge())
                    .build_signed(chain_keypair, domain_separator)?
                    .leak();

                Ok(Self::Outgoing {
                    ticket,
                    next_hop: mapper.map_id_to_public(&surb.first_relayer).ok_or_else(|| {
                        PacketConstructionError(format!(
                            "failed to map key id {} to public key",
                            surb.first_relayer.to_hex()
                        ))
                    })?,
                    ack_challenge: surb.additional_data_receiver.acknowledgement_challenge(),
                    packet: MetaPacket::<CurrentSphinxSuite, HoprSphinxHeaderSpec>::new(
                        msg,
                        MetaPacketRouting::Surb(surb),
                        mapper,
                    )?,
                })
            }
        }
    }

    /// Constructs a new outgoing packet with the given path.
    /// # Arguments
    /// * `msg` packet payload
    /// * `routing` routing to the destination
    /// * `chain_keypair` private key of the local node
    /// * `ticket` ticket builder for the first hop on the path
    /// * `domain_separator` channels contract domain separator
    pub fn into_outgoing<M: KeyIdMapper<CurrentSphinxSuite, HoprSphinxHeaderSpec>>(
        msg: &[u8],
        routing: PacketRouting,
        surb: Option<(HoprPseudonym, &[OffchainPublicKey])>,
        chain_keypair: &ChainKeypair,
        ticket: TicketBuilder,
        mapper: &M,
        domain_separator: &Hash,
    ) -> Result<(Self, Option<LocalSURBEntry>)> {
        if let Some((pseudonym, return_path)) = surb {
            if msg.len() + HoprSurb::SIZE > PAYLOAD_SIZE {
                return Err(PacketError::PacketConstructionError(
                    "message too long to fit with a surb into the packet".into(),
                ));
            }

            let shared_keys = CurrentSphinxSuite::new_shared_keys(return_path)?;
            let por_strings = ProofOfRelayString::from_shared_secrets(&shared_keys.secrets)?;
            let por_values = ProofOfRelayValues::new(&shared_keys.secrets[0], shared_keys.secrets.get(1))?;

            let (surb, local) = create_surb::<CurrentSphinxSuite, HoprSphinxHeaderSpec>(
                shared_keys,
                &return_path
                    .iter()
                    .map(|k| {
                        mapper
                            .map_key_to_id(k)
                            .ok_or_else(|| PacketConstructionError(format!("failed to map key {} to id", k.to_hex())))
                    })
                    .collect::<Result<Vec<_>>>()?,
                &por_strings,
                SphinxRecipientMessage::DataOnly.into(),
                por_values,
            )?;
            let mut composed_msg = Vec::with_capacity(msg.len() + HoprSurb::SIZE);
            composed_msg.extend(surb.into_boxed());
            composed_msg.extend_from_slice(msg);

            Self::into_raw_msg(
                &composed_msg,
                SphinxRecipientMessage::DataWithSurb(pseudonym),
                routing,
                chain_keypair,
                ticket,
                mapper,
                domain_separator,
            )
            .map(|p| (p, Some(local)))
        } else {
            Self::into_raw_msg(
                msg,
                SphinxRecipientMessage::DataOnly,
                routing,
                chain_keypair,
                ticket,
                mapper,
                domain_separator,
            )
            .map(|p| (p, None))
        }
    }

    pub fn into_outgoing_surbs<M: KeyIdMapper<CurrentSphinxSuite, HoprSphinxHeaderSpec>>(
        pseudonym: HoprPseudonym,
        return_paths: &[&[OffchainPublicKey]],
        routing: PacketRouting,
        chain_keypair: &ChainKeypair,
        ticket: TicketBuilder,
        mapper: &M,
        domain_separator: &Hash,
    ) -> Result<(Self, Vec<LocalSURBEntry>)> {
        if return_paths.is_empty() || return_paths.len() * HoprSurb::SIZE > PAYLOAD_SIZE {
            return Err(PacketError::PacketConstructionError(
                "too many SURBs to fit in the packet".into(),
            ));
        }

        let mut composed_msg = Vec::with_capacity(return_paths.len() * HoprSurb::SIZE);
        let mut local_surbs = Vec::with_capacity(return_paths.len());

        for return_path in return_paths {
            let shared_keys = CurrentSphinxSuite::new_shared_keys(return_path)?;
            let por_strings = ProofOfRelayString::from_shared_secrets(&shared_keys.secrets)?;
            let por_values = ProofOfRelayValues::new(&shared_keys.secrets[0], shared_keys.secrets.get(1))?;

            let (surb, local) = create_surb::<CurrentSphinxSuite, HoprSphinxHeaderSpec>(
                shared_keys,
                &return_path
                    .iter()
                    .map(|k| {
                        mapper
                            .map_key_to_id(k)
                            .ok_or_else(|| PacketConstructionError(format!("failed to map key {} to id", k.to_hex())))
                    })
                    .collect::<Result<Vec<_>>>()?,
                &por_strings,
                SphinxRecipientMessage::DataOnly.into(),
                por_values,
            )?;

            composed_msg.extend(surb.into_boxed());
            local_surbs.push(local);
        }

        Self::into_raw_msg(
            &composed_msg,
            SphinxRecipientMessage::SurbsOnly(return_paths.len() as u8, pseudonym),
            routing,
            chain_keypair,
            ticket,
            mapper,
            domain_separator,
        )
        .map(|p| (p, local_surbs))
    }

    /// Deserializes the packet and performs the forward-transformation, so the
    /// packet can be further delivered (relayed to the next hop or read).
    pub fn from_incoming<M, F>(
        data: &[u8],
        node_keypair: &OffchainKeypair,
        previous_hop: OffchainPublicKey,
        mapper: &M,
        local_surbs: F,
    ) -> Result<Self>
    where
        M: KeyIdMapper<CurrentSphinxSuite, HoprSphinxHeaderSpec>,
        F: Fn(&HoprPseudonym) -> Option<LocalSURBEntry>,
    {
        if data.len() == Self::SIZE {
            let (pre_packet, pre_ticket) =
                data.split_at(MetaPacket::<CurrentSphinxSuite, HoprSphinxHeaderSpec>::PACKET_LEN);

            let mp: MetaPacket<CurrentSphinxSuite, HoprSphinxHeaderSpec> = MetaPacket::try_from(pre_packet)?;

            match mp.into_forwarded(node_keypair, mapper, local_surbs)? {
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
                    let verification_output = pre_verify(&derived_secret, additional_info.as_ref(), &ticket.challenge)?;
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
                    additional_data,
                } => {
                    let ack_key = derive_ack_key_share(&derived_secret);
                    let (surbs, plain_text) = match additional_data {
                        SphinxRecipientMessage::DataOnly | SphinxRecipientMessage::ReplyOnly(_) => {
                            (Vec::new(), Some(plain_text))
                        }
                        d @ SphinxRecipientMessage::DataWithSurb(p) | d @ SphinxRecipientMessage::SurbsOnly(_, p) => {
                            let chunks = plain_text.chunks_exact(HoprSurb::SIZE);
                            let num_surbs = d.num_surbs();
                            let data = matches!(d, SphinxRecipientMessage::DataWithSurb(_))
                                .then(|| Box::from(chunks.remainder()));

                            let surbs = chunks
                                .map(|c| {
                                    HoprSurb::try_from(c)
                                        .map(|s| (p, s))
                                        .map_err(|_| PacketDecodingError("packet has invalid surb".into()))
                                })
                                .collect::<Result<Vec<_>>>()?;

                            if num_surbs != surbs.len() as u8 {
                                return Err(PacketDecodingError("packet has invalid number of surbs".into()));
                            }

                            (surbs, data)
                        }
                    };

                    // This ticket is not parsed nor verified on the final hop
                    Ok(Self::Final {
                        packet_tag,
                        ack_key,
                        previous_hop,
                        plain_text,
                        surbs,
                    })
                }
            }
        } else {
            Err(PacketDecodingError("packet has invalid size".into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ChainPacketComponents, PacketRouting};

    use async_trait::async_trait;
    use bimap::BiHashMap;
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

    fn forward(
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
                    .challenge(next_challenge)
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

    impl ChainPacketComponents {
        pub fn to_bytes(&self) -> Box<[u8]> {
            let dummy_ticket = hex!("67f0ca18102feec505e5bfedcc25963e9c64a6f8a250adcad7d2830dd607585700000000000000000000000000000000000000000000000000000000000000003891bf6fd4a78e868fc7ad477c09b16fc70dd01ea67e18264d17e3d04f6d8576de2e6472b0072e510df6e9fa1dfcc2727cc7633edfeb9ec13860d9ead29bee71d68de3736c2f7a9f42de76ccd57a5f5847bc7349");
            let (packet, ticket) = match self {
                Self::Final { plain_text, .. } => (plain_text.clone().unwrap(), dummy_ticket.as_ref().into()),
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
        let mut cg = ChannelGraph::new(me);
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
        let mapper = keypairs_offchain
            .iter()
            .enumerate()
            .map(|(i, k)| (KeyIdent::from(i as u32), k.public().clone()))
            .collect::<BiHashMap<_, _>>();

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

        let (mut packet, surb) = ChainPacketComponents::into_outgoing(
            test_message,
            PacketRouting::ForwardPath(&path),
            None,
            &own_channel_kp,
            ticket,
            &mapper,
            &Hash::default(),
        )?;

        assert!(surb.is_none());

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

            packet = ChainPacketComponents::from_incoming(&packet.to_bytes(), path_element, sender, &mapper, |_| None)
                .unwrap_or_else(|e| panic!("failed to deserialize packet at hop {i}: {e}"));

            match &packet {
                ChainPacketComponents::Final { plain_text, .. } => {
                    assert_eq!(keypairs_offchain.len() - 1, i);
                    assert_eq!(&test_message, &plain_text.clone().unwrap().as_ref());
                }
                ChainPacketComponents::Forwarded { .. } => {
                    let ticket = mock_ticket(
                        &keypairs_onchain[i + 1].public().0,
                        keypairs_offchain.len() - i - 1,
                        &keypairs_onchain[i],
                    )?;
                    packet = forward(packet.clone(), &keypairs_onchain[i], ticket, &Hash::default());
                }
                ChainPacketComponents::Outgoing { .. } => panic!("invalid packet state"),
            }
        }

        Ok(())
    }
}
