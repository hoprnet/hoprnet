use hopr_crypto_sphinx::surb::{create_surb, ReplyOpener, SURB};
use hopr_crypto_sphinx::{derivation::derive_ack_key_share, shared_keys::SphinxSuite};
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use std::fmt::{Display, Formatter};

use crate::errors::PacketError::PacketConstructionError;
use crate::packet::{KeyIdMapper, MetaPacketRouting};
use crate::{
    errors::{PacketError::PacketDecodingError, Result},
    packet::{ForwardedMetaPacket, MetaPacket},
    por::{pre_verify, ProofOfRelayString, ProofOfRelayValues},
    HoprPseudonym, HoprSphinxHeaderSpec, HoprSphinxSuite, HoprSurb,
};
use hopr_crypto_sphinx::surb::SphinxRecipientMessage;

/// Indicates the packet type.
#[allow(clippy::large_enum_variant)] // TODO: see if some parts can be boxed
#[derive(Clone)]
pub enum HoprPacket {
    /// The packet is intended for us
    Final {
        packet_tag: PacketTag,
        ack_key: HalfKey,
        previous_hop: OffchainPublicKey,
        plain_text: Box<[u8]>,
        surbs: Vec<(HoprPseudonym, SURB<HoprSphinxSuite, HoprSphinxHeaderSpec>)>,
    },
    /// The packet must be forwarded
    Forwarded {
        packet: MetaPacket<HoprSphinxSuite, HoprSphinxHeaderSpec>,
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
        packet: MetaPacket<HoprSphinxSuite, HoprSphinxHeaderSpec>,
        ticket: Ticket,
        next_hop: OffchainPublicKey,
        ack_challenge: HalfKeyChallenge,
    },
}

impl Display for HoprPacket {
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

impl HoprPacket {
    /// The size of the packet including header, padded payload, ticket, and ack challenge.
    pub const SIZE: usize = MetaPacket::<HoprSphinxSuite, HoprSphinxHeaderSpec>::PACKET_LEN + Ticket::SIZE;

    fn into_raw_msg<M: KeyIdMapper<HoprSphinxSuite, HoprSphinxHeaderSpec>>(
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
                if forward_path.is_empty() {
                    return Err(PacketConstructionError(
                        "packet cannot be routed to an empty path".into(),
                    ));
                }

                let shared_keys = HoprSphinxSuite::new_shared_keys(forward_path)?;
                let por_values = ProofOfRelayValues::new(
                    &shared_keys.secrets[0],
                    shared_keys.secrets.get(1),
                    shared_keys.secrets.len() as u8,
                )?;
                let por_strings = ProofOfRelayString::from_shared_secrets(&shared_keys.secrets)?;

                // Update the ticket with the challenge
                let ticket = ticket
                    .challenge(por_values.ticket_challenge())
                    .build_signed(chain_keypair, domain_separator)?
                    .leak();

                Ok(Self::Outgoing {
                    packet: MetaPacket::<HoprSphinxSuite, HoprSphinxHeaderSpec>::new(
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
                    packet: MetaPacket::<HoprSphinxSuite, HoprSphinxHeaderSpec>::new(
                        msg,
                        MetaPacketRouting::Surb(surb),
                        mapper,
                    )?,
                })
            }
        }
    }

    fn create_surb_for_path<M: KeyIdMapper<HoprSphinxSuite, HoprSphinxHeaderSpec>>(
        return_path: &[OffchainPublicKey],
        pseudonym: HoprPseudonym,
        mapper: &M,
    ) -> Result<(HoprSurb, ReplyOpener)> {
        if return_path.is_empty() {
            return Err(PacketConstructionError("return path cannot be empty".into()));
        }

        let shared_keys = HoprSphinxSuite::new_shared_keys(return_path)?;
        let por_strings = ProofOfRelayString::from_shared_secrets(&shared_keys.secrets)?;
        let por_values = ProofOfRelayValues::new(
            &shared_keys.secrets[0],
            shared_keys.secrets.get(1),
            shared_keys.secrets.len() as u8,
        )?;

        Ok(create_surb::<HoprSphinxSuite, HoprSphinxHeaderSpec>(
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
            SphinxRecipientMessage::ReplyOnly(pseudonym).into(),
            por_values,
        )?)
    }

    /// Calculates how many SURBs can be fitted into a packet that
    /// also carries a message of the given length.
    pub const fn max_surbs_with_message(msg_len: usize) -> usize {
        (PAYLOAD_SIZE - msg_len) / HoprSurb::SIZE
    }

    /// Constructs a new outgoing packet with the given path.
    ///
    /// # Arguments
    /// * `msg` packet payload
    /// * `routing` routing to the destination
    /// * `surb` optional SURBs with same pseudonym
    /// * `chain_keypair` private key of the local node
    /// * `ticket` ticket builder for the first hop on the path
    /// * `domain_separator` channels contract domain separator
    ///
    /// **NOTE**
    /// For the given pseudonym, the [`ReplyOpener`] order matters.
    pub fn into_outgoing<M: KeyIdMapper<HoprSphinxSuite, HoprSphinxHeaderSpec>>(
        msg: &[u8],
        routing: PacketRouting,
        surb: Option<(HoprPseudonym, &[&[OffchainPublicKey]])>,
        chain_keypair: &ChainKeypair,
        ticket: TicketBuilder,
        mapper: &M,
        domain_separator: &Hash,
    ) -> Result<(Self, Vec<ReplyOpener>)> {
        if let Some((pseudonym, return_paths)) = surb {
            if return_paths.is_empty() {
                return Err(PacketConstructionError("return path cannot be empty".into()));
            }

            if msg.len() + return_paths.len() * HoprSurb::SIZE > PAYLOAD_SIZE {
                return Err(PacketConstructionError(format!(
                    "message too long to fit with {} surbs into the packet",
                    return_paths.len()
                )));
            }

            let mut composed_msg = Vec::with_capacity(msg.len() + return_paths.len() * HoprSurb::SIZE);
            let mut openers = Vec::with_capacity(return_paths.len());
            for return_path in return_paths {
                let (surb, opener) = Self::create_surb_for_path(return_path, pseudonym, mapper)?;

                composed_msg.extend(surb.into_boxed());
                openers.push(opener);
            }

            composed_msg.extend(msg);

            Self::into_raw_msg(
                &composed_msg,
                SphinxRecipientMessage::DataAndSurbs {
                    num_surbs: return_paths.len() as u16,
                    pseudonym,
                    remainder_data: msg.len() as u16,
                },
                routing,
                chain_keypair,
                ticket,
                mapper,
                domain_separator,
            )
            .map(|p| (p, openers))
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
            .map(|p| (p, Vec::with_capacity(0)))
        }
    }

    /// Deserializes the packet and performs the forward-transformation, so the
    /// packet can be further delivered (relayed to the next hop or read).
    pub fn from_incoming<M, F>(
        data: &[u8],
        node_keypair: &OffchainKeypair,
        previous_hop: OffchainPublicKey,
        mapper: &M,
        reply_openers: F,
    ) -> Result<Self>
    where
        M: KeyIdMapper<HoprSphinxSuite, HoprSphinxHeaderSpec>,
        F: FnMut(&HoprPseudonym) -> Option<ReplyOpener>,
    {
        if data.len() == Self::SIZE {
            let (pre_packet, pre_ticket) =
                data.split_at(MetaPacket::<HoprSphinxSuite, HoprSphinxHeaderSpec>::PACKET_LEN);

            let mp: MetaPacket<HoprSphinxSuite, HoprSphinxHeaderSpec> = MetaPacket::try_from(pre_packet)?;

            match mp.into_forwarded(node_keypair, mapper, reply_openers)? {
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
                            (Vec::with_capacity(0), plain_text)
                        }
                        SphinxRecipientMessage::DataAndSurbs { num_surbs, pseudonym, remainder_data } => {
                            let chunks = plain_text.chunks_exact(HoprSurb::SIZE);
                            if chunks.len() != num_surbs as usize {
                                return Err(PacketDecodingError("packet has invalid number of surbs".into()));
                            }

                            if chunks.remainder().len() < remainder_data as usize {
                                return Err(PacketDecodingError("packet has invalid remainder data".into()));
                            }

                            let data = if remainder_data > 0 {
                                Box::from(&chunks.remainder()[..remainder_data as usize])
                            } else {
                                Box::default()
                            };

                            let surbs = chunks
                                .map(|c| {
                                    HoprSurb::try_from(c)
                                        .map(|surb| (pseudonym, surb))
                                        .map_err(|_| PacketDecodingError("packet has invalid surb".into()))
                                })
                                .collect::<Result<Vec<_>>>()?;

                            (surbs, data)
                        }
                    };

                    // The pre_ticket is not parsed nor verified on the final hop
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
    use super::{HoprPacket, PacketRouting};

    use anyhow::{bail, Context};
    use bimap::BiHashMap;
    use hex_literal::hex;
    use hopr_crypto_sphinx::surb::ReplyOpener;
    use hopr_crypto_types::prelude::*;
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use parameterized::parameterized;

    use crate::{HoprPseudonym, HoprSurb};

    lazy_static::lazy_static! {
        static ref PEERS: [(ChainKeypair, OffchainKeypair); 5] = [
            (hex!("a7c486ceccf5ab53bd428888ab1543dc2667abd2d5e80aae918da8d4b503a426"), hex!("5eb212d4d6aa5948c4f71574d45dad43afef6d330edb873fca69d0e1b197e906")),
            (hex!("9a82976f7182c05126313bead5617c623b93d11f9f9691c87b1a26f869d569ed"), hex!("e995db483ada5174666c46bafbf3628005aca449c94ebdc0c9239c3f65d61ae0")),
            (hex!("ca4bdfd54a8467b5283a0216288fdca7091122479ccf3cfb147dfa59d13f3486"), hex!("9dec751c00f49e50fceff7114823f726a0425a68a8dc6af0e4287badfea8f4a4")),
            (hex!("e306ebfb0d01d0da0952c9a567d758093a80622c6cb55052bf5f1a6ebd8d7b5c"), hex!("9a82976f7182c05126313bead5617c623b93d11f9f9691c87b1a26f869d569ed")),
            (hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"), hex!("e0bf93e9c916104da00b1850adc4608bd7e9087bbd3f805451f4556aa6b3fd6e")),
        ].map(|(p1,p2)| (ChainKeypair::from_secret(&p1).expect("lazy static keypair should be valid"), OffchainKeypair::from_secret(&p2).expect("lazy static keypair should be valid")));

        static ref MAPPER: bimap::BiMap<KeyIdent, OffchainPublicKey> = PEERS
            .iter()
            .enumerate()
            .map(|(i, (_, k))| (KeyIdent::from(i as u32), k.public().clone()))
            .collect::<BiHashMap<_, _>>();
    }

    fn forward(
        packet: HoprPacket,
        chain_keypair: &ChainKeypair,
        next_ticket: TicketBuilder,
        domain_separator: &Hash,
    ) -> HoprPacket {
        match packet {
            HoprPacket::Forwarded {
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
                HoprPacket::Forwarded {
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

    impl HoprPacket {
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

    fn create_packet(
        forward_hops: usize,
        return_hops: Option<(HoprPseudonym, Vec<usize>)>,
        msg: &[u8],
    ) -> anyhow::Result<(HoprPacket, Vec<ReplyOpener>)> {
        assert!((0..=3).contains(&forward_hops), "forward hops must be between 1 and 3");
        assert!(
            return_hops
                .as_ref()
                .is_none_or(|(_, ret_hops)| ret_hops.iter().all(|h| (0..=3).contains(h))),
            "return hops must be between 1 and 3"
        );

        let ticket = mock_ticket(&PEERS[1].0.public().0, forward_hops + 1, &PEERS[0].0)?;
        let path = PEERS[1..=forward_hops + 1]
            .iter()
            .map(|kp| *kp.1.public())
            .collect::<Vec<_>>();

        if let Some((pseudonym, ret_hops)) = return_hops {
            let return_paths = ret_hops
                .into_iter()
                .map(|h| PEERS[0..=h].iter().rev().map(|kp| *kp.1.public()).collect::<Vec<_>>())
                .collect::<Vec<_>>();

            let return_paths_refs = return_paths
                .iter()
                .map(|return_path| return_path.as_slice())
                .collect::<Vec<_>>();

            Ok(HoprPacket::into_outgoing(
                msg,
                PacketRouting::ForwardPath(&path),
                Some((pseudonym, &return_paths_refs)),
                &PEERS[0].0,
                ticket,
                &*MAPPER,
                &Hash::default(),
            )?)
        } else {
            Ok(HoprPacket::into_outgoing(
                msg,
                PacketRouting::ForwardPath(&path),
                None,
                &PEERS[0].0,
                ticket,
                &*MAPPER,
                &Hash::default(),
            )?)
        }
    }

    fn create_packet_from_surb(sender_node: usize, surb: HoprSurb, msg: &[u8]) -> anyhow::Result<HoprPacket> {
        assert!((1..=4).contains(&sender_node), "sender_node must be between 1 and 4");

        let ticket = mock_ticket(
            &PEERS[sender_node - 1].0.public().0,
            surb.additional_data_receiver.chain_length() as usize,
            &PEERS[sender_node].0,
        )?;

        Ok(HoprPacket::into_outgoing(
            msg,
            PacketRouting::Surb(surb),
            None,
            &PEERS[sender_node].0,
            ticket,
            &*MAPPER,
            &Hash::default(),
        )?
        .0)
    }

    fn process_packet_at_node<F>(
        path_len: usize,
        node_pos: usize,
        is_reply: bool,
        packet: HoprPacket,
        openers: F,
    ) -> anyhow::Result<HoprPacket>
    where
        F: FnMut(&HoprPseudonym) -> Option<ReplyOpener>,
    {
        assert!((0..=4).contains(&node_pos), "node position must be between 1 and 3");

        let prev_hop = match (node_pos, is_reply) {
            (1, false) => PEERS[0].1.public().clone(),
            (_, false) => PEERS[node_pos - 1].1.public().clone(),
            (3, true) => PEERS[4].1.public().clone(),
            (_, true) => PEERS[node_pos + 1].1.public().clone(),
        };

        let packet = HoprPacket::from_incoming(&packet.to_bytes(), &PEERS[node_pos].1, prev_hop, &*MAPPER, openers)
            .context(format!("deserialization failure at hop {node_pos}"))?;

        match &packet {
            HoprPacket::Final { .. } => Ok(packet),
            HoprPacket::Forwarded { .. } => {
                let next_hop = match (node_pos, is_reply) {
                    (3, false) => PEERS[4].0.public().0.clone(),
                    (_, false) => PEERS[node_pos + 1].0.public().0.clone(),
                    (1, true) => PEERS[0].0.public().0.clone(),
                    (_, true) => PEERS[node_pos - 1].0.public().0.clone(),
                };

                let next_ticket = mock_ticket(&next_hop, path_len, &PEERS[node_pos].0)?;
                Ok(forward(
                    packet.clone(),
                    &PEERS[node_pos].0,
                    next_ticket,
                    &Hash::default(),
                ))
            }
            HoprPacket::Outgoing { .. } => bail!("invalid packet state"),
        }
    }

    #[parameterized(hops = { 0,1,2,3 })]
    fn test_packet_forward_message_no_surb(hops: usize) -> anyhow::Result<()> {
        let msg = b"some testing forward message";
        let (mut packet, opener) = create_packet(hops, None, msg)?;

        assert!(opener.is_empty());
        match &packet {
            HoprPacket::Outgoing { .. } => {}
            _ => bail!("invalid packet initial state"),
        }

        let mut actual_plain_text = Box::default();
        for hop in 1..=hops + 1 {
            packet = process_packet_at_node(hops + 1, hop, false, packet, |_| None)
                .context(format!("packet decoding failed at hop {hop}"))?;

            match &packet {
                HoprPacket::Final { plain_text, .. } => {
                    assert_eq!(hop - 1, hops, "final packet must be at the last hop");
                    actual_plain_text = plain_text.clone();
                }
                HoprPacket::Forwarded {
                    previous_hop,
                    next_hop,
                    path_pos,
                    ..
                } => {
                    assert_eq!(PEERS[hop - 1].1.public(), previous_hop, "invalid previous hop");
                    assert_eq!(PEERS[hop + 1].1.public(), next_hop, "invalid next hop");
                    assert_eq!(hops + 1 - hop, *path_pos as usize, "invalid path position");
                }
                HoprPacket::Outgoing { .. } => bail!("invalid packet state at hop {hop}"),
            }
        }

        assert_eq!(actual_plain_text.as_ref(), msg, "invalid plaintext");
        Ok(())
    }

    #[parameterized(forward_hops = { 0,1,2,3 }, return_hops = { 0, 1, 2, 3})]
    fn test_packet_forward_message_with_surb(forward_hops: usize, return_hops: usize) -> anyhow::Result<()> {
        let msg = b"some testing forward message";
        let pseudonym = SimplePseudonym::random();
        let (mut packet, openers) = create_packet(forward_hops, Some((pseudonym, vec![return_hops])), msg)?;

        assert_eq!(1, openers.len(), "invalid number of openers");
        match &packet {
            HoprPacket::Outgoing { .. } => {}
            _ => bail!("invalid packet initial state"),
        }

        let mut received_plain_text = Box::default();
        let mut received_surbs = vec![];
        for hop in 1..=forward_hops + 1 {
            packet = process_packet_at_node(forward_hops + 1, hop, false, packet, |_| None)
                .context(format!("packet decoding failed at hop {hop}"))?;

            match &packet {
                HoprPacket::Final { plain_text, surbs, .. } => {
                    assert_eq!(hop - 1, forward_hops, "final packet must be at the last hop");
                    received_plain_text = plain_text.clone();
                    received_surbs.extend(surbs.clone());
                }
                HoprPacket::Forwarded {
                    previous_hop,
                    next_hop,
                    path_pos,
                    ..
                } => {
                    assert_eq!(PEERS[hop - 1].1.public(), previous_hop, "invalid previous hop");
                    assert_eq!(PEERS[hop + 1].1.public(), next_hop, "invalid next hop");
                    assert_eq!(forward_hops + 1 - hop, *path_pos as usize, "invalid path position");
                }
                HoprPacket::Outgoing { .. } => bail!("invalid packet state at hop {hop}"),
            }
        }

        assert_eq!(received_plain_text.as_ref(), msg, "invalid plaintext");
        assert_eq!(1, received_surbs.len(), "invalid number of surbs");
        assert_eq!(pseudonym, received_surbs[0].0, "invalid pseudonym");
        assert_eq!(
            return_hops as u8 + 1,
            received_surbs[0].1.additional_data_receiver.chain_length(),
            "surb has invalid por chain length"
        );

        Ok(())
    }

    #[parameterized(
        forward_hops = { 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3 },
        return_hops  = { 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3 }
    )]
    fn test_packet_forward_and_reply_message(forward_hops: usize, return_hops: usize) -> anyhow::Result<()> {
        let pseudonym = SimplePseudonym::random();

        // Forward packet
        let fwd_msg = b"some testing forward message";
        let (mut fwd_packet, mut openers) = create_packet(forward_hops, Some((pseudonym, vec![return_hops])), fwd_msg)?;

        assert_eq!(1, openers.len(), "invalid number of openers");
        match &fwd_packet {
            HoprPacket::Outgoing { .. } => {}
            _ => bail!("invalid packet initial state"),
        }

        let mut received_fwd_plain_text = Box::default();
        let mut received_surbs = vec![];
        for hop in 1..=forward_hops + 1 {
            fwd_packet = process_packet_at_node(forward_hops + 1, hop, false, fwd_packet, |_| None)
                .context(format!("packet decoding failed at hop {hop}"))?;

            match &fwd_packet {
                HoprPacket::Final { plain_text, surbs, .. } => {
                    assert_eq!(hop - 1, forward_hops, "final packet must be at the last hop");
                    received_fwd_plain_text = plain_text.clone();
                    received_surbs.extend(surbs.clone());
                }
                HoprPacket::Forwarded {
                    previous_hop,
                    next_hop,
                    path_pos,
                    ..
                } => {
                    assert_eq!(PEERS[hop - 1].1.public(), previous_hop, "invalid previous hop");
                    assert_eq!(PEERS[hop + 1].1.public(), next_hop, "invalid next hop");
                    assert_eq!(forward_hops + 1 - hop, *path_pos as usize, "invalid path position");
                }
                HoprPacket::Outgoing { .. } => bail!("invalid packet state at hop {hop}"),
            }
        }

        assert_eq!(received_fwd_plain_text.as_ref(), fwd_msg, "invalid plaintext");
        assert_eq!(1, received_surbs.len(), "invalid number of surbs");
        assert_eq!(pseudonym, received_surbs[0].0, "invalid pseudonym");
        assert_eq!(
            return_hops as u8 + 1,
            received_surbs[0].1.additional_data_receiver.chain_length(),
            "surb has invalid por chain length"
        );

        // Reply packet
        let re_msg = b"some testing reply message";
        let mut re_packet = create_packet_from_surb(forward_hops + 1, received_surbs[0].1.clone(), re_msg)?;

        let mut openers_fn = |p: &HoprPseudonym| {
            assert_eq!(p, &pseudonym);
            openers.pop()
        };

        match &re_packet {
            HoprPacket::Outgoing { .. } => {}
            _ => bail!("invalid packet initial state"),
        }

        let mut received_re_plain_text = Box::default();
        for hop in (0..=return_hops).rev() {
            re_packet = process_packet_at_node(return_hops + 1, hop, true, re_packet, &mut openers_fn)
                .context(format!("packet decoding failed at hop {hop}"))?;

            match &re_packet {
                HoprPacket::Final { plain_text, surbs, .. } => {
                    assert_eq!(hop, 0, "final packet must be at the last hop");
                    assert!(surbs.is_empty(), "must not receive surbs on reply");
                    received_re_plain_text = plain_text.clone();
                }
                HoprPacket::Forwarded {
                    previous_hop,
                    next_hop,
                    path_pos,
                    ..
                } => {
                    assert_eq!(PEERS[hop + 1].1.public(), previous_hop, "invalid previous hop");
                    assert_eq!(PEERS[hop - 1].1.public(), next_hop, "invalid next hop");
                    assert_eq!(hop, *path_pos as usize, "invalid path position");
                }
                HoprPacket::Outgoing { .. } => bail!("invalid packet state at hop {hop}"),
            }
        }

        assert_eq!(received_re_plain_text.as_ref(), re_msg, "invalid plaintext");

        Ok(())
    }

    #[parameterized(
        forward_hops = { 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3 },
        return_hops  = { 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3 }
    )]
    fn test_packet_surbs_only_and_reply_message(forward_hops: usize, return_hops: usize) -> anyhow::Result<()> {
        let pseudonym = SimplePseudonym::random();

        // Forward packet
        let (mut fwd_packet, mut openers) = create_packet(forward_hops, Some((pseudonym, vec![return_hops; 2])), &[])?;

        assert_eq!(2, openers.len(), "invalid number of openers");
        match &fwd_packet {
            HoprPacket::Outgoing { .. } => {}
            _ => bail!("invalid packet initial state"),
        }

        let mut received_surbs = vec![];
        for hop in 1..=forward_hops + 1 {
            fwd_packet = process_packet_at_node(forward_hops + 1, hop, false, fwd_packet, |_| None)
                .context(format!("packet decoding failed at hop {hop}"))?;

            match &fwd_packet {
                HoprPacket::Final { plain_text, surbs, .. } => {
                    assert_eq!(hop - 1, forward_hops, "final packet must be at the last hop");
                    assert!(plain_text.is_empty(), "must not receive plaintext on surbs only packet");
                    assert_eq!(2, surbs.len(), "invalid number of received surbs per packet");
                    received_surbs.extend(surbs.clone());
                }
                HoprPacket::Forwarded {
                    previous_hop,
                    next_hop,
                    path_pos,
                    ..
                } => {
                    assert_eq!(PEERS[hop - 1].1.public(), previous_hop, "invalid previous hop");
                    assert_eq!(PEERS[hop + 1].1.public(), next_hop, "invalid next hop");
                    assert_eq!(forward_hops + 1 - hop, *path_pos as usize, "invalid path position");
                }
                HoprPacket::Outgoing { .. } => bail!("invalid packet state at hop {hop}"),
            }
        }

        assert_eq!(2, received_surbs.len(), "invalid number of surbs");
        for (recv_pseudonym, recv_surb) in &received_surbs {
            assert_eq!(pseudonym, *recv_pseudonym, "invalid pseudonym");
            assert_eq!(
                return_hops as u8 + 1,
                recv_surb.additional_data_receiver.chain_length(),
                "surb has invalid por chain length"
            );
        }

        let mut openers_fn = |p: &HoprPseudonym| {
            assert_eq!(p, &pseudonym);
            Some(openers.remove(0))
        };

        // Reply packet
        for (i, (_, recv_surb)) in received_surbs.into_iter().enumerate() {
            let re_msg = format!("some testing reply message {i}");
            let mut re_packet = create_packet_from_surb(forward_hops + 1, recv_surb, re_msg.as_bytes())?;

            match &re_packet {
                HoprPacket::Outgoing { .. } => {}
                _ => bail!("invalid packet initial state in reply {i}"),
            }

            let mut received_re_plain_text = Box::default();
            for hop in (0..=return_hops).rev() {
                re_packet = process_packet_at_node(return_hops + 1, hop, true, re_packet, &mut openers_fn)
                    .context(format!("packet decoding failed at hop {hop} in reply {i}"))?;

                match &re_packet {
                    HoprPacket::Final { plain_text, surbs, .. } => {
                        assert_eq!(hop, 0, "final packet must be at the last hop for reply {i}");
                        assert!(surbs.is_empty(), "must not receive surbs on reply for reply {i}");
                        received_re_plain_text = plain_text.clone();
                    }
                    HoprPacket::Forwarded {
                        previous_hop,
                        next_hop,
                        path_pos,
                        ..
                    } => {
                        assert_eq!(
                            PEERS[hop + 1].1.public(),
                            previous_hop,
                            "invalid previous hop in reply {i}"
                        );
                        assert_eq!(PEERS[hop - 1].1.public(), next_hop, "invalid next hop in reply {i}");
                        assert_eq!(hop, *path_pos as usize, "invalid path position in reply {i}");
                    }
                    HoprPacket::Outgoing { .. } => bail!("invalid packet state at hop {hop} in reply {i}"),
                }
            }

            assert_eq!(
                received_re_plain_text.as_ref(),
                re_msg.as_bytes(),
                "invalid plaintext in reply {i}"
            );
        }
        Ok(())
    }
}
