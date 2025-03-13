use hopr_crypto_sphinx::routing::SphinxHeaderSpec;
use hopr_crypto_sphinx::surb::{ReplyOpener, SphinxRecipientMessage, SURB};
use hopr_crypto_sphinx::{
    derivation::derive_packet_tag,
    routing::{forward_header, ForwardedHeader, RoutingInfo},
    shared_keys::{Alpha, GroupElement, SharedKeys, SharedSecret, SphinxSuite},
};
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

use hopr_crypto_types::crypto_traits::StreamCipher;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use typenum::Unsigned;

use crate::errors::PacketError;
use crate::errors::{PacketError::PacketDecodingError, Result};

/// Tag used to separate padding from data
const PADDING_TAG: u8 = 0xaa;

/// Size of the packet payload when padded.
const PADDED_PAYLOAD_SIZE: usize = PAYLOAD_SIZE + 1;

fn add_padding(msg: &[u8]) -> [u8; PADDED_PAYLOAD_SIZE] {
    assert!(msg.len() <= PAYLOAD_SIZE, "message too long for padding");

    // Zeroes followed by the PADDING_TAG and then the message
    let mut ret = [0u8; PADDED_PAYLOAD_SIZE];
    ret[PADDED_PAYLOAD_SIZE - msg.len() - 1] = PADDING_TAG;
    ret[PADDED_PAYLOAD_SIZE - msg.len()..].copy_from_slice(msg);
    ret
}

fn remove_padding(msg: &[u8]) -> Option<&[u8]> {
    if !msg.is_empty() {
        for i in 0..msg.len() - 1 {
            if msg[i] == PADDING_TAG {
                return Some(&msg[i + 1..]);
            }
        }
    }
    None
}

/// Trait that defines 1:1 mapper between key identifiers and the actual public keys.
///
/// This is used to uniquely map between short public key identifiers used in the Sphinx header,
/// and actual routing addresses (public keys) of the nodes.
pub trait KeyIdMapper<S: SphinxSuite, H: SphinxHeaderSpec> {
    /// Maps public key to its unique identifier.
    fn map_key_to_id(&self, key: &<S::P as Keypair>::Public) -> Option<H::KeyId>;
    /// Maps public key identifier to the actual public key.
    fn map_id_to_public(&self, id: &H::KeyId) -> Option<<S::P as Keypair>::Public>;
    /// Convenience method to map a slice of public keys to IDs.
    fn map_keys_to_ids(&self, keys: &[<S::P as Keypair>::Public]) -> Vec<Option<H::KeyId>> {
        keys.iter().map(|key| self.map_key_to_id(key)).collect()
    }
    /// Convenience method to map a slice of IDs to public keys.
    fn map_ids_to_keys(&self, ids: &[H::KeyId]) -> Vec<Option<<S::P as Keypair>::Public>> {
        ids.iter().map(|id| self.map_id_to_public(id)).collect()
    }
}

/// Describes how a [`MetaPacket`] should be routed to the destination.
pub enum MetaPacketRouting<'a, S: SphinxSuite, H: SphinxHeaderSpec> {
    /// Uses an explicitly given path to deliver the packet.
    ForwardPath {
        /// Shared keys with individual hops
        shared_keys: SharedKeys<S::E, S::G>,
        /// Public keys on the path corresponding to the shared keys
        forward_path: &'a [<S::P as Keypair>::Public],
        /// Additional data for individual relayers
        additional_data_relayer: &'a [H::RelayerData],
        /// Additional data for the packt recipient
        additional_data_last_hop: H::LastHopData,
    },
    /// Uses a SURB to deliver the packet.
    Surb(SURB<S, H>),
}

/// An encrypted packet.
///
/// A sender can create a new packet via [MetaPacket::new] and send it.
/// Once received by the recipient, it is parsed first by calling [MetaPacket::try_from]
/// and then it can be transformed into [ForwardedMetaPacket] by calling
/// the [MetaPacket::into_forwarded] method. The [ForwardedMetaPacket] then contains the information
/// about the next recipient of this packet.
///
/// The packet format is directly dependent on the used [SphinxSuite].
pub struct MetaPacket<S, H> {
    packet: Box<[u8]>,
    _s: PhantomData<S>,
    _h: PhantomData<H>,
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> Debug for MetaPacket<S, H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

// Needs manual Clone implementation to not impose Clone restriction on `S` and `H`
impl<S, H> Clone for MetaPacket<S, H> {
    fn clone(&self) -> Self {
        Self {
            packet: self.packet.clone(),
            _s: PhantomData,
            _h: PhantomData,
        }
    }
}

/// Represent a [MetaPacket] with one layer of encryption removed, exposing the details
/// about the next hop.
///
/// There are two possible states - either the packet is intended for the recipient,
/// and is thus [Final], or it is meant to be sent (relayed)
/// to the next hop - thus it is [Relayed].
#[allow(dead_code)]
pub enum ForwardedMetaPacket<S: SphinxSuite, H: SphinxHeaderSpec> {
    /// The content is another [MetaPacket] meant to be sent to the next hop.
    Relayed {
        /// Packet for the next hop.
        packet: MetaPacket<S, H>,
        /// Public key of the next hop.
        next_node: <S::P as Keypair>::Public,
        /// Position in the channel path of this packet.
        path_pos: u8,
        /// Additional data for the relayer.
        ///
        /// In HOPR protocol, this contains the PoR challenge that will be solved when we receive
        /// the acknowledgement after we forward the inner packet to the next hop.
        additional_info: H::RelayerData,
        /// Shared secret that was used to encrypt the removed layer.
        derived_secret: SharedSecret,
        /// Packet checksum.
        packet_tag: PacketTag,
    },
    /// The content is the actual payload for the packet's destination.
    Final {
        /// Decrypted payload
        plain_text: Box<[u8]>,
        /// Contains additional information about the packet payload.
        additional_data: SphinxRecipientMessage<H::Pseudonym>,
        /// Shared secret that was used to encrypt the removed layer.
        derived_secret: SharedSecret,
        /// Packet checksum.
        packet_tag: PacketTag,
    },
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> MetaPacket<S, H> {
    /// The fixed length of the padded packet.
    pub const PACKET_LEN: usize = <S::P as Keypair>::Public::SIZE + RoutingInfo::<H>::SIZE + PADDED_PAYLOAD_SIZE;

    /// Creates a new outgoing packet with the given payload `msg`, `routing` and `shared_keys` computed along the path.
    ///
    /// The size of the `msg` must be less or equal [PAYLOAD_SIZE], otherwise the
    /// constructor will return an error.
    pub fn new<M: KeyIdMapper<S, H>>(msg: &[u8], routing: MetaPacketRouting<S, H>, key_mapper: &M) -> Result<Self> {
        if msg.len() > PAYLOAD_SIZE {
            return Err(PacketError::PacketConstructionError(
                "message too long to fit into a packet".into(),
            ));
        }

        let mut payload = add_padding(msg);
        match routing {
            MetaPacketRouting::ForwardPath {
                shared_keys,
                forward_path,
                additional_data_relayer,
                additional_data_last_hop,
            } => {
                let routing_info = RoutingInfo::<H>::new(
                    &forward_path
                        .iter()
                        .map(|key| {
                            key_mapper.map_key_to_id(key).ok_or_else(|| {
                                PacketError::PacketConstructionError(format!("key id not found for {}", key.to_hex()))
                            })
                        })
                        .collect::<Result<Vec<_>>>()?,
                    &shared_keys.secrets,
                    additional_data_relayer,
                    additional_data_last_hop,
                )?;

                // Encrypt the packet payload using the derived shared secrets
                for secret in shared_keys.secrets.into_iter().rev() {
                    let mut prp = S::new_prp(&secret)?;
                    prp.apply_keystream(&mut payload);
                }

                Self::new_from_parts(shared_keys.alpha, routing_info, &payload)
            }
            MetaPacketRouting::Surb(surb) => {
                // Encrypt the packet using the sender's key from the SURB
                let mut prp = S::new_reply_prp(&surb.sender_key)?;
                prp.apply_keystream(&mut payload);

                Self::new_from_parts(surb.alpha, surb.header, &payload)
            }
        }
    }

    fn new_from_parts(
        alpha: Alpha<<S::G as GroupElement<S::E>>::AlphaLen>,
        routing_info: RoutingInfo<H>,
        payload: &[u8],
    ) -> Result<Self> {
        if payload.len() != PADDED_PAYLOAD_SIZE {
            return Err(PacketError::PacketConstructionError(format!(
                "packet payload must be exactly {PADDED_PAYLOAD_SIZE} bytes long"
            )));
        }

        let mut packet = Vec::with_capacity(Self::SIZE);
        packet.extend_from_slice(&alpha);
        packet.extend_from_slice(routing_info.as_ref());
        packet.extend_from_slice(payload);

        Ok(Self {
            packet: packet.into_boxed_slice(),
            _s: PhantomData,
            _h: PhantomData,
        })
    }

    /// Returns the Alpha value subslice from the packet data.
    fn alpha(&self) -> &[u8] {
        let len = <S::G as GroupElement<S::E>>::AlphaLen::USIZE;
        &self.packet[..len]
    }

    /// Returns the routing information from the packet data as a mutable slice.
    fn header_mut(&mut self) -> &mut [u8] {
        let base = <S::G as GroupElement<S::E>>::AlphaLen::USIZE;
        &mut self.packet[base..base + H::HEADER_LEN]
    }

    /// Returns the packet checksum (MAC) subslice from the packet data.
    fn mac(&self) -> &[u8] {
        let base = <S::G as GroupElement<S::E>>::AlphaLen::USIZE + H::HEADER_LEN;
        &self.packet[base..base + H::TAG_SIZE]
    }

    /// Returns the payload subslice from the packet data.
    fn payload_mut(&mut self) -> &mut [u8] {
        let base = <S::G as GroupElement<S::E>>::AlphaLen::USIZE + RoutingInfo::<H>::SIZE;
        &mut self.packet[base..base + PADDED_PAYLOAD_SIZE]
    }

    /// Attempts to remove the layer of encryption in this packet by using the given `node_keypair`.
    /// This will transform this packet into the [ForwardedMetaPacket].
    pub fn into_forwarded<K, F>(
        mut self,
        node_keypair: &S::P,
        key_mapper: &K,
        mut reply_openers: F,
    ) -> Result<ForwardedMetaPacket<S, H>>
    where
        K: KeyIdMapper<S, H>,
        F: FnMut(&H::Pseudonym) -> Option<ReplyOpener>,
    {
        let (alpha, secret) = SharedKeys::<S::E, S::G>::forward_transform(
            Alpha::<<S::G as GroupElement<S::E>>::AlphaLen>::from_slice(self.alpha()),
            &(node_keypair.into()),
            &(node_keypair.public().into()),
        )?;

        // Forward the packet header
        let mac_cpy = self.mac().to_vec(); // TODO: change this so we can avoid the re-allocation
        let fwd_header = forward_header::<H>(&secret, self.header_mut(), &mac_cpy)?;

        // Perform initial decryption over the payload
        let decrypted = self.payload_mut();
        let mut prp = S::new_prp(&secret)?;
        prp.apply_keystream(decrypted);

        Ok(match fwd_header {
            ForwardedHeader::RelayNode {
                next_header,
                path_pos,
                next_node,
                additional_info,
            } => ForwardedMetaPacket::Relayed {
                packet: Self::new_from_parts(alpha, next_header, decrypted)?,
                packet_tag: derive_packet_tag(&secret),
                derived_secret: secret,
                next_node: key_mapper.map_id_to_public(&next_node).ok_or_else(|| {
                    PacketDecodingError(format!("couldn't map id to public key: {}", next_node.to_hex()))
                })?,
                path_pos,
                additional_info,
            },
            ForwardedHeader::FinalNode { additional_data } => {
                // If the received packet contains a reply message for a pseudonym,
                // we must perform additional steps to decrypt it
                let additional_data: SphinxRecipientMessage<H::Pseudonym> = additional_data.try_into()?;
                if let SphinxRecipientMessage::<H::Pseudonym>::ReplyOnly(pseudonym) = &additional_data {
                    let local_surb = reply_openers(pseudonym).ok_or_else(|| {
                        PacketDecodingError(format!("couldn't find local SURB entry for pseudonym: {pseudonym}"))
                    })?;

                    // Encrypt the packet payload using the derived shared secrets,
                    // to reverse the decryption done by individual hops
                    for secret in local_surb.shared_keys.into_iter().rev() {
                        let mut prp = S::new_prp(&secret)?;
                        prp.apply_keystream(decrypted);
                    }

                    // Invert the initial encryption using the sender key
                    let mut prp = S::new_reply_prp(&local_surb.sender_key)?;
                    prp.apply_keystream(decrypted);
                }

                ForwardedMetaPacket::Final {
                    packet_tag: derive_packet_tag(&secret),
                    derived_secret: secret,
                    plain_text: remove_padding(decrypted)
                        .ok_or_else(|| PacketDecodingError("couldn't remove padding".into()))?
                        .into(),
                    additional_data,
                }
            }
        })
    }
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> AsRef<[u8]> for MetaPacket<S, H> {
    fn as_ref(&self) -> &[u8] {
        self.packet.as_ref()
    }
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> TryFrom<&[u8]> for MetaPacket<S, H> {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        if value.len() == Self::SIZE {
            Ok(Self {
                packet: value.into(),
                _s: PhantomData,
                _h: PhantomData,
            })
        } else {
            Err(GeneralError::ParseError("MetaPacket".into()))
        }
    }
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> BytesRepresentable for MetaPacket<S, H> {
    const SIZE: usize = <S::G as GroupElement<S::E>>::AlphaLen::USIZE + RoutingInfo::<H>::SIZE + PADDED_PAYLOAD_SIZE;
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    use crate::por::{ProofOfRelayString, ProofOfRelayValues};
    use crate::{return_path, HoprPseudonym};
    use anyhow::anyhow;
    use bimap::BiHashMap;
    use hopr_crypto_sphinx::ec_groups::{Ed25519Suite, Secp256k1Suite, X25519Suite};
    use hopr_crypto_sphinx::surb::{create_surb, SphinxRecipientMessage};
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair, OffchainKeypair};
    use parameterized::parameterized;
    use std::hash::Hash;
    use std::num::NonZeroUsize;

    impl<S, H> KeyIdMapper<S, H> for BiHashMap<H::KeyId, <S::P as Keypair>::Public>
    where
        S: SphinxSuite,
        H: SphinxHeaderSpec,
        <S::P as Keypair>::Public: Eq + Hash,
        H::KeyId: Eq + Hash,
    {
        fn map_key_to_id(&self, key: &<S::P as Keypair>::Public) -> Option<H::KeyId> {
            self.get_by_right(key).cloned()
        }

        fn map_id_to_public(&self, id: &H::KeyId) -> Option<<S::P as Keypair>::Public> {
            self.get_by_left(id).cloned()
        }
    }

    struct TestHeader<S: SphinxSuite>(PhantomData<S>);

    impl<S: SphinxSuite> SphinxHeaderSpec for TestHeader<S> {
        const MAX_HOPS: NonZeroUsize = NonZeroUsize::new(INTERMEDIATE_HOPS + 1).unwrap();
        type KeyId = KeyIdent<4>;
        type Pseudonym = HoprPseudonym;
        type RelayerData = ProofOfRelayString;
        type LastHopData = return_path::EncodedRecipientMessage<HoprPseudonym>;
        type SurbReceiverData = ProofOfRelayValues;
        type PRG = hopr_crypto_types::primitives::ChaCha20;
        type UH = hopr_crypto_types::primitives::Poly1305;
    }

    #[test]
    fn test_padding() {
        let data = b"test";
        let padded = add_padding(data);

        let mut expected = vec![0u8; PAYLOAD_SIZE - data.len()];
        expected.push(PADDING_TAG);
        expected.extend_from_slice(data);
        assert_eq!(&expected, padded.as_ref());

        let unpadded = remove_padding(&padded);
        assert!(unpadded.is_some());
        assert_eq!(data, &unpadded.unwrap());
    }

    fn generic_test_meta_packet<S: SphinxSuite>(keypairs: Vec<S::P>) -> anyhow::Result<()>
    where
        S: SphinxSuite,
        <S::P as Keypair>::Public: Eq + Hash,
    {
        let pubkeys = keypairs.iter().map(|kp| kp.public().clone()).collect::<Vec<_>>();
        let mapper = keypairs
            .iter()
            .enumerate()
            .map(|(i, k)| (KeyIdent::from(i as u32), k.public().clone()))
            .collect::<BiHashMap<_, _>>();

        let shared_keys = S::new_shared_keys(&pubkeys)?;
        let por_strings = ProofOfRelayString::from_shared_secrets(&shared_keys.secrets)?;

        assert_eq!(shared_keys.secrets.len() - 1, por_strings.len());

        let msg = b"some random test message";

        let mut mp = MetaPacket::<S, TestHeader<S>>::new(
            msg,
            MetaPacketRouting::ForwardPath {
                shared_keys,
                forward_path: &pubkeys,
                additional_data_relayer: &por_strings,
                additional_data_last_hop: SphinxRecipientMessage::DataOnly.into(),
            },
            &mapper,
        )?;

        assert!(mp.as_ref().len() < 1492, "metapacket too long {}", mp.as_ref().len());

        let mut received_plaintext = Box::default();
        for (i, pair) in keypairs.iter().enumerate() {
            let fwd = mp
                .clone()
                .into_forwarded(pair, &mapper, |_| None)
                .unwrap_or_else(|_| panic!("failed to unwrap at {i}"));

            match fwd {
                ForwardedMetaPacket::Relayed { packet, .. } => {
                    assert!(i < keypairs.len() - 1);
                    mp = packet;
                }
                ForwardedMetaPacket::Final {
                    plain_text,
                    additional_data,
                    ..
                } => {
                    assert_eq!(keypairs.len() - 1, i);
                    assert_eq!(additional_data, SphinxRecipientMessage::DataOnly);
                    received_plaintext = plain_text;
                }
            }
        }

        assert_eq!(msg, received_plaintext.as_ref());

        Ok(())
    }

    fn generic_meta_packet_reply_test<S: SphinxSuite>(keypairs: Vec<S::P>) -> anyhow::Result<()>
    where
        S: SphinxSuite,
        <S::P as Keypair>::Public: Eq + Hash,
    {
        let pubkeys = keypairs.iter().map(|kp| kp.public().clone()).collect::<Vec<_>>();
        let mapper = keypairs
            .iter()
            .enumerate()
            .map(|(i, k)| (KeyIdent::from(i as u32), k.public().clone()))
            .collect::<BiHashMap<_, _>>();

        let shared_keys = S::new_shared_keys(&pubkeys)?;
        let por_strings = ProofOfRelayString::from_shared_secrets(&shared_keys.secrets)?;
        let por_values = ProofOfRelayValues::new(
            &shared_keys.secrets[0],
            shared_keys.secrets.get(1),
            shared_keys.secrets.len() as u8,
        )?;

        let pseudonym = SimplePseudonym::random();
        let ids = <BiHashMap<_, _> as KeyIdMapper<S, TestHeader<S>>>::map_keys_to_ids(&mapper, &pubkeys)
            .into_iter()
            .map(|v| v.ok_or_else(|| anyhow!("failed to map keys to ids")))
            .collect::<anyhow::Result<Vec<KeyIdent>>>()?;

        let (surb, opener) = create_surb::<S, TestHeader<S>>(
            shared_keys,
            &ids,
            &por_strings,
            SphinxRecipientMessage::ReplyOnly(pseudonym).into(),
            por_values,
        )?;

        let msg = b"some random reply test message";

        let mut mp = MetaPacket::<S, TestHeader<S>>::new(msg, MetaPacketRouting::Surb(surb), &mapper)?;

        let surb_retriever = |p: &HoprPseudonym| {
            assert_eq!(pseudonym, *p);
            Some(opener.clone())
        };

        let mut received_plaintext = Box::default();
        for (i, pair) in keypairs.iter().enumerate() {
            let fwd = mp
                .clone()
                .into_forwarded(pair, &mapper, surb_retriever)
                .unwrap_or_else(|_| panic!("failed to unwrap at {i}"));

            match fwd {
                ForwardedMetaPacket::Relayed { packet, .. } => {
                    assert!(i < keypairs.len() - 1);
                    mp = packet;
                }
                ForwardedMetaPacket::Final {
                    plain_text,
                    additional_data,
                    ..
                } => {
                    assert_eq!(keypairs.len() - 1, i);
                    assert_eq!(additional_data, SphinxRecipientMessage::ReplyOnly(pseudonym));
                    received_plaintext = plain_text;
                }
            }
        }

        assert_eq!(msg, received_plaintext.as_ref());

        Ok(())
    }

    #[parameterized(amount = { 4, 3, 2, 1 })]
    fn test_x25519_meta_packet(amount: usize) -> anyhow::Result<()> {
        generic_test_meta_packet::<X25519Suite>((0..amount).map(|_| OffchainKeypair::random()).collect())
    }

    #[parameterized(amount = { 4, 3, 2, 1 })]
    fn test_x25519_reply_meta_packet(amount: usize) -> anyhow::Result<()> {
        generic_meta_packet_reply_test::<X25519Suite>((0..amount).map(|_| OffchainKeypair::random()).collect())
    }

    #[parameterized(amount = { 4, 3, 2, 1 })]
    fn test_ed25519_meta_packet(amount: usize) -> anyhow::Result<()> {
        generic_test_meta_packet::<Ed25519Suite>((0..amount).map(|_| OffchainKeypair::random()).collect())
    }

    #[parameterized(amount = { 4, 3, 2, 1 })]
    fn test_ed25519_reply_meta_packet(amount: usize) -> anyhow::Result<()> {
        generic_meta_packet_reply_test::<Ed25519Suite>((0..amount).map(|_| OffchainKeypair::random()).collect())
    }

    #[parameterized(amount = { 4, 3, 2, 1 })]
    fn test_secp256k1_meta_packet(amount: usize) -> anyhow::Result<()> {
        generic_test_meta_packet::<Secp256k1Suite>((0..amount).map(|_| ChainKeypair::random()).collect())
    }

    #[parameterized(amount = { 4, 3, 2, 1 })]
    fn test_secp256k1_reply_meta_packet(amount: usize) -> anyhow::Result<()> {
        generic_meta_packet_reply_test::<Secp256k1Suite>((0..amount).map(|_| ChainKeypair::random()).collect())
    }
}
