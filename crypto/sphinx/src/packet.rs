use std::{
    fmt::{Debug, Formatter},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use hopr_crypto_types::{crypto_traits::PRP, prelude::*};
use hopr_primitive_types::prelude::*;
use typenum::Unsigned;

use crate::{
    derivation::derive_packet_tag,
    errors::SphinxError,
    routing::{ForwardedHeader, RoutingInfo, SphinxHeaderSpec, forward_header},
    shared_keys::{Alpha, GroupElement, SharedKeys, SharedSecret, SphinxSuite},
    surb::{ReplyOpener, SURB},
};

/// Holds data that are padded up to `P + 1`.
///
/// Data in this instance is guaranteed to be always `P + 1` bytes-long.
// TODO: make P a typenum argument
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaddedPayload<const P: usize>(Box<[u8]>);

impl<const P: usize> PaddedPayload<P> {
    /// Byte used to pad the data.
    pub const PADDING: u8 = 0x00;
    /// Tag used to separate padding from data
    pub const PADDING_TAG: u8 = 0xaa;
    /// Size of the padded data.
    pub const SIZE: usize = P + size_of_val(&Self::PADDING_TAG);

    /// Creates a new instance from the given message `msg` shorter than [`PaddedPayload::SIZE`] and pads it.
    ///
    /// The padding consists of prepending a [`PaddedPayload::PADDING_TAG`], preceded by as many zero bytes
    /// to fill it up to [`PaddedPayload::SIZE`]. If data is `P` bytes-long, only the padding tag is prepended.
    ///
    /// If the argument's length is greater or equal to [`PaddedPayload::SIZE`], [`SphinxError::PaddingError`] is
    /// returned.
    pub fn new(msg: &[u8]) -> Result<Self, SphinxError> {
        if msg.len() < Self::SIZE {
            // Zeroes followed by the PADDING_TAG and then the message
            let mut ret = vec![Self::PADDING; Self::SIZE];
            ret[Self::SIZE - msg.len() - 1] = Self::PADDING_TAG;
            ret[Self::SIZE - msg.len()..].copy_from_slice(msg);

            Ok(Self(ret.into_boxed_slice()))
        } else {
            Err(SphinxError::PaddingError)
        }
    }

    /// Similar like [`PaddedPayload::new`], but creates a new instance from a vector,
    /// reallocating only if the given vector has insufficient capacity.
    pub fn new_from_vec(mut msg: Vec<u8>) -> Result<Self, SphinxError> {
        let len = msg.len();
        if len >= Self::SIZE {
            return Err(SphinxError::PaddingError);
        }

        msg.resize(Self::SIZE, Self::PADDING); // Reallocates only if capacity is not enough
        msg.copy_within(0..len, Self::SIZE - len);
        msg[0..Self::SIZE - len].fill(Self::PADDING);
        msg[Self::SIZE - len - 1] = Self::PADDING_TAG;

        Ok(Self(msg.into_boxed_slice()))
    }

    /// Creates a new instance from an already padded message `msg` and takes its ownership.
    ///
    /// This method only checks the length of the argument, it does not verify
    /// the presence of the padding tag. If the padding tag is not present, an error
    /// is later returned when [`PaddedPayload::into_unpadded`] is called.
    ///
    /// If the vector has any excess capacity, it will be trimmed.
    ///
    /// If the argument's length is not equal to [`PaddedPayload::SIZE`], [`SphinxError::PaddingError`] is returned.
    pub fn from_padded(msg: Vec<u8>) -> Result<Self, SphinxError> {
        if msg.len() == Self::SIZE {
            Ok(Self(msg.into_boxed_slice()))
        } else {
            Err(SphinxError::PaddingError)
        }
    }

    /// Consumes the instance by removing the padding and taking ownership of
    /// the unpadded data. The original length of the data is restored.
    ///
    /// If the padding tag could not be found, [`SphinxError::PaddingError`] is returned.
    /// This means this instance was created using [`PaddedPayload::from_padded`] with invalid data.
    pub fn into_unpadded(self) -> Result<Box<[u8]>, SphinxError> {
        self.0
            .iter()
            .position(|x| *x == Self::PADDING_TAG)
            .map(|tag_pos| {
                let mut data = self.0.into_vec();
                data.drain(0..=tag_pos);
                data.into_boxed_slice()
            })
            .ok_or(SphinxError::PaddingError)
    }
}

impl<const P: usize> AsRef<[u8]> for PaddedPayload<P> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<const P: usize> Deref for PaddedPayload<P> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<const P: usize> DerefMut for PaddedPayload<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
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

impl<S, H> KeyIdMapper<S, H> for bimap::BiHashMap<H::KeyId, <S::P as Keypair>::Public>
where
    S: SphinxSuite,
    H: SphinxHeaderSpec,
    <S::P as Keypair>::Public: Eq + std::hash::Hash,
    H::KeyId: Eq + std::hash::Hash,
{
    fn map_key_to_id(&self, key: &<S::P as Keypair>::Public) -> Option<H::KeyId> {
        self.get_by_right(key).cloned()
    }

    fn map_id_to_public(&self, id: &H::KeyId) -> Option<<S::P as Keypair>::Public> {
        self.get_by_left(id).cloned()
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
        /// Additional data delivered to the packet's final recipient.
        receiver_data: &'a H::PacketReceiverData,
        /// Special flag used for acknowledgement signaling to the recipient
        no_ack: bool,
    },
    /// Uses a SURB to deliver the packet and some additional data to the SURB's creator.
    Surb(SURB<S, H>, &'a H::PacketReceiverData),
}

/// Represents a packet that is only partially instantiated,
/// that is - it contains only the routing information and the Alpha value.
///
/// This object can be used to pre-compute a packet without a payload
/// and possibly serialize it, and later to be
/// deserialized and used to construct the final [`MetaPacket`] via
/// a call to [`PartialPacket::into_meta_packet`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PartialPacket<S: SphinxSuite, H: SphinxHeaderSpec> {
    alpha: Alpha<<S::G as GroupElement<S::E>>::AlphaLen>,
    routing_info: RoutingInfo<H>,
    prp_inits: Vec<IvKey<S::PRP>>,
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> PartialPacket<S, H> {
    /// Creates a new partial packet using the given routing information and
    /// public key identifier mapper.
    pub fn new<M: KeyIdMapper<S, H>>(routing: MetaPacketRouting<S, H>, key_mapper: &M) -> Result<Self, SphinxError> {
        match routing {
            MetaPacketRouting::ForwardPath {
                shared_keys,
                forward_path,
                additional_data_relayer,
                receiver_data,
                no_ack,
            } => {
                let routing_info = RoutingInfo::<H>::new(
                    &forward_path
                        .iter()
                        .map(|key| {
                            key_mapper.map_key_to_id(key).ok_or_else(|| {
                                SphinxError::PacketConstructionError(format!("key id not found for {}", key.to_hex()))
                            })
                        })
                        .collect::<Result<Vec<_>, SphinxError>>()?,
                    &shared_keys.secrets,
                    additional_data_relayer,
                    receiver_data,
                    false,
                    no_ack,
                )?;

                Ok(Self {
                    alpha: shared_keys.alpha,
                    routing_info,
                    prp_inits: shared_keys
                        .secrets
                        .into_iter()
                        .rev()
                        .map(|key| S::new_prp_init(&key))
                        .collect::<Result<Vec<_>, _>>()?,
                })
            }
            MetaPacketRouting::Surb(surb, receiver_data) => Ok(Self {
                alpha: surb.alpha,
                routing_info: surb.header,
                prp_inits: vec![S::new_reply_prp_init(&surb.sender_key, receiver_data.as_ref())?],
            }),
        }
    }

    /// Transform this partial packet into an actual [`MetaPacket`] using the given payload.
    pub fn into_meta_packet<const P: usize>(self, mut payload: PaddedPayload<P>) -> MetaPacket<S, H, P> {
        for iv_key in self.prp_inits {
            let prp = iv_key.into_init::<S::PRP>();
            // The following won't panic, because PaddedPayload<P> is guaranteed to be S::PRP::BlockSize bytes-long
            // However, it would be nicer to make PaddedPayload take P as a typenum parameter
            // and enforce this invariant at compile time.
            let block = crypto_traits::Block::<S::PRP>::from_mut_slice(&mut payload);
            prp.forward(block);
        }

        MetaPacket::new_from_parts(self.alpha, self.routing_info, &payload)
    }
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> Debug for PartialPacket<S, H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PartialPacket")
            .field("alpha", &self.alpha)
            .field("routing_info", &self.routing_info)
            .field("prp_inits", &self.prp_inits)
            .finish()
    }
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> Clone for PartialPacket<S, H> {
    fn clone(&self) -> Self {
        Self {
            alpha: self.alpha.clone(),
            routing_info: self.routing_info.clone(),
            prp_inits: self.prp_inits.clone(),
        }
    }
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> PartialEq for PartialPacket<S, H> {
    fn eq(&self, other: &Self) -> bool {
        self.alpha == other.alpha && self.routing_info == other.routing_info && self.prp_inits == other.prp_inits
    }
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> Eq for PartialPacket<S, H> {}

/// An encrypted packet with a payload of size `P`.
/// The final packet size is given by [`MetaPacket::SIZE`].
///
/// A sender can create a new packet via [`MetaPacket::new`] and send it.
/// Once received by the recipient, it is parsed first by calling [`MetaPacket::try_from`]
/// and then it can be transformed into [`ForwardedMetaPacket`] by calling
/// the [`MetaPacket::into_forwarded`] method. The [`ForwardedMetaPacket`] then contains the information
/// about the next recipient of this packet or the payload for the final destination.
///
/// The packet format is directly dependent on the used [`SphinxSuite`].
pub struct MetaPacket<S, H, const P: usize> {
    packet: Box<[u8]>,
    _d: PhantomData<(S, H)>,
}

impl<S: SphinxSuite, H: SphinxHeaderSpec, const P: usize> Debug for MetaPacket<S, H, P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

// Needs manual Clone implementation to not impose Clone restriction on `S` and `H`
impl<S, H, const P: usize> Clone for MetaPacket<S, H, P> {
    fn clone(&self) -> Self {
        Self {
            packet: self.packet.clone(),
            _d: PhantomData,
        }
    }
}

/// Represent a [`MetaPacket`] with one layer of encryption removed, exposing the details
/// about the next hop.
///
/// There are two possible states - either the packet is intended for the recipient,
/// and is thus [`ForwardedMetaPacket::Final`], or it is meant to be sent (relayed)
/// to the next hop - thus it is [`ForwardedMetaPacket::Relayed`].
#[allow(dead_code)]
pub enum ForwardedMetaPacket<S: SphinxSuite, H: SphinxHeaderSpec, const P: usize> {
    /// The content is another [`MetaPacket`] meant to be sent to the next hop.
    Relayed {
        /// Packet for the next hop.
        packet: MetaPacket<S, H, P>,
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
        plain_text: PaddedPayload<P>,
        /// Data for the packet receiver (containing the sender's pseudonym).
        receiver_data: H::PacketReceiverData,
        /// Shared secret that was used to encrypt the removed layer.
        derived_secret: SharedSecret,
        /// Packet checksum.
        packet_tag: PacketTag,
        /// Special flag used for acknowledgement signaling to the recipient
        no_ack: bool,
    },
}

impl<S: SphinxSuite, H: SphinxHeaderSpec, const P: usize> MetaPacket<S, H, P> {
    /// The fixed length of the padded packet.
    pub const PACKET_LEN: usize = <S::P as Keypair>::Public::SIZE + RoutingInfo::<H>::SIZE + PaddedPayload::<P>::SIZE;

    /// Creates a new outgoing packet with the given payload `msg` and `routing`.
    ///
    /// The size of the `msg` must be less or equal `P`, otherwise the
    /// constructor will return an error.
    pub fn new<M: KeyIdMapper<S, H>>(
        payload: PaddedPayload<P>,
        routing: MetaPacketRouting<S, H>,
        key_mapper: &M,
    ) -> Result<Self, SphinxError> {
        Ok(PartialPacket::new(routing, key_mapper)?.into_meta_packet(payload))
    }

    fn new_from_parts(
        alpha: Alpha<<S::G as GroupElement<S::E>>::AlphaLen>,
        routing_info: RoutingInfo<H>,
        payload: &[u8],
    ) -> Self {
        let mut packet = Vec::with_capacity(Self::SIZE);
        packet.extend_from_slice(&alpha);
        packet.extend_from_slice(routing_info.as_ref());
        packet.extend_from_slice(&payload[0..PaddedPayload::<P>::SIZE]);

        Self {
            packet: packet.into_boxed_slice(),
            _d: PhantomData,
        }
    }

    /// Returns the Alpha value subslice from the packet data.
    fn alpha(&self) -> &[u8] {
        let len = <S::G as GroupElement<S::E>>::AlphaLen::USIZE;
        &self.packet[..len]
    }

    /// Returns the routing information from the packet data as a mutable slice.
    fn routing_info_mut(&mut self) -> &mut [u8] {
        let base = <S::G as GroupElement<S::E>>::AlphaLen::USIZE;
        &mut self.packet[base..base + RoutingInfo::<H>::SIZE]
    }

    /// Returns the payload subslice from the packet data.
    ///
    /// This data is guaranteed to be `PaddedPayload::<P>::SIZE` bytes-long, which currently
    /// is `P + 1` bytes.
    fn payload_mut(&mut self) -> &mut [u8] {
        let base = <S::G as GroupElement<S::E>>::AlphaLen::USIZE + RoutingInfo::<H>::SIZE;
        &mut self.packet[base..base + PaddedPayload::<P>::SIZE]
    }

    /// Attempts to remove the layer of encryption in this packet by using the given `node_keypair`.
    /// This will transform this packet into the [`ForwardedMetaPacket`].
    pub fn into_forwarded<'a, K, F>(
        mut self,
        node_keypair: &'a S::P,
        key_mapper: &K,
        mut reply_openers: F,
    ) -> Result<ForwardedMetaPacket<S, H, P>, SphinxError>
    where
        K: KeyIdMapper<S, H>,
        F: FnMut(&H::PacketReceiverData) -> Option<ReplyOpener>,
        &'a Alpha<<S::G as GroupElement<S::E>>::AlphaLen>: From<&'a <S::P as Keypair>::Public>,
    {
        let (alpha, secret) = SharedKeys::<S::E, S::G>::forward_transform(
            Alpha::<<S::G as GroupElement<S::E>>::AlphaLen>::from_slice(self.alpha()),
            &(node_keypair.into()),
            node_keypair.public().into(),
        )?;

        // Forward the packet header
        let fwd_header = forward_header::<H>(&secret, self.routing_info_mut())?;

        // Perform initial decryption over the payload
        let decrypted = self.payload_mut();
        let prp = S::new_prp_init(&secret)?.into_init::<S::PRP>();
        prp.inverse(decrypted.into());

        Ok(match fwd_header {
            ForwardedHeader::Relayed {
                next_header,
                path_pos,
                next_node,
                additional_info,
            } => ForwardedMetaPacket::Relayed {
                packet: Self::new_from_parts(alpha, next_header, decrypted),
                packet_tag: derive_packet_tag(&secret)?,
                derived_secret: secret,
                next_node: key_mapper.map_id_to_public(&next_node).ok_or_else(|| {
                    SphinxError::PacketDecodingError(format!("couldn't map id to public key: {}", next_node.to_hex()))
                })?,
                path_pos,
                additional_info,
            },
            ForwardedHeader::Final {
                receiver_data,
                is_reply,
                no_ack,
            } => {
                // If the received packet contains a reply message for a pseudonym,
                // we must perform additional steps to decrypt it
                if is_reply {
                    let local_surb = reply_openers(&receiver_data).ok_or_else(|| {
                        SphinxError::PacketDecodingError(format!(
                            "couldn't find reply opener for pseudonym: {}",
                            receiver_data.to_hex()
                        ))
                    })?;

                    // Encrypt the packet payload using the derived shared secrets
                    // to reverse the decryption done by individual hops
                    for secret in local_surb.shared_secrets.into_iter().rev() {
                        let prp = S::new_prp_init(&secret)?.into_init::<S::PRP>();
                        prp.forward(decrypted.into());
                    }

                    // Invert the initial encryption using the sender key
                    let prp =
                        S::new_reply_prp_init(&local_surb.sender_key, receiver_data.as_ref())?.into_init::<S::PRP>();
                    prp.inverse(decrypted.into());
                }

                // Remove all the data before the actual decrypted payload
                // and shrink the original allocation.
                let mut payload = self.packet.into_vec();
                payload.drain(..<S::G as GroupElement<S::E>>::AlphaLen::USIZE + RoutingInfo::<H>::SIZE);

                ForwardedMetaPacket::Final {
                    packet_tag: derive_packet_tag(&secret)?,
                    derived_secret: secret,
                    plain_text: PaddedPayload::from_padded(payload)?,
                    receiver_data,
                    no_ack,
                }
            }
        })
    }
}

impl<S: SphinxSuite, H: SphinxHeaderSpec, const P: usize> AsRef<[u8]> for MetaPacket<S, H, P> {
    fn as_ref(&self) -> &[u8] {
        self.packet.as_ref()
    }
}

impl<S: SphinxSuite, H: SphinxHeaderSpec, const P: usize> TryFrom<&[u8]> for MetaPacket<S, H, P> {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() == Self::SIZE {
            Ok(Self {
                packet: value.into(),
                _d: PhantomData,
            })
        } else {
            Err(GeneralError::ParseError("MetaPacket".into()))
        }
    }
}

impl<S: SphinxSuite, H: SphinxHeaderSpec, const P: usize> BytesRepresentable for MetaPacket<S, H, P> {
    const SIZE: usize =
        <S::G as GroupElement<S::E>>::AlphaLen::USIZE + RoutingInfo::<H>::SIZE + PaddedPayload::<P>::SIZE;
}

#[cfg(test)]
pub(crate) mod tests {
    use std::{hash::Hash, num::NonZeroUsize};

    use anyhow::anyhow;
    use bimap::BiHashMap;
    use hopr_crypto_random::Randomizable;
    use hopr_crypto_types::keypairs::{Keypair, OffchainKeypair};
    use parameterized::parameterized;

    use super::*;
    use crate::{surb::create_surb, tests::WrappedBytes};

    #[derive(Debug, Clone, Copy)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    struct TestHeader<S: SphinxSuite>(PhantomData<S>);

    impl<S: SphinxSuite> SphinxHeaderSpec for TestHeader<S> {
        type KeyId = KeyIdent<4>;
        type PRG = hopr_crypto_types::primitives::ChaCha20;
        type PacketReceiverData = SimplePseudonym;
        type Pseudonym = SimplePseudonym;
        type RelayerData = WrappedBytes<53>;
        type SurbReceiverData = WrappedBytes<54>;
        type UH = hopr_crypto_types::primitives::Poly1305;

        const MAX_HOPS: NonZeroUsize = NonZeroUsize::new(4).unwrap();
    }

    const PAYLOAD_SIZE: usize = 1021;

    #[test]
    fn test_padding() -> anyhow::Result<()> {
        let data = b"some testing forward message";
        let padded = PaddedPayload::<PAYLOAD_SIZE>::new(data)?;

        let mut expected = vec![0u8; PAYLOAD_SIZE - data.len()];
        expected.push(PaddedPayload::<PAYLOAD_SIZE>::PADDING_TAG);
        expected.extend_from_slice(data);
        assert_eq!(expected.len(), padded.len());
        assert_eq!(&expected, padded.as_ref());

        let padded_from_vec = PaddedPayload::<PAYLOAD_SIZE>::new_from_vec(data.to_vec())?;
        assert_eq!(padded, padded_from_vec);

        let unpadded = padded.into_unpadded()?;
        assert!(!unpadded.is_empty());
        assert_eq!(data, unpadded.as_ref());

        Ok(())
    }

    #[test]
    fn test_padding_zero_length() -> anyhow::Result<()> {
        let data = [];
        let padded = PaddedPayload::<9>::new(&data)?;
        assert_eq!(padded.len(), 10);
        assert_eq!(padded.as_ref()[9], PaddedPayload::<9>::PADDING_TAG);
        assert_eq!(&padded.as_ref()[0..9], &[0u8; 9]);

        Ok(())
    }

    #[test]
    fn test_padding_full_length() -> anyhow::Result<()> {
        let data = [1u8; 9];
        let padded = PaddedPayload::<9>::new(&data)?;
        assert_eq!(padded.len(), 10);
        assert_eq!(padded.as_ref()[0], PaddedPayload::<9>::PADDING_TAG);
        assert_eq!(padded.as_ref()[1..], data);

        Ok(())
    }

    #[cfg(feature = "serde")]
    fn generic_test_partial_packet_serialization<S>(keypairs: Vec<S::P>) -> anyhow::Result<()>
    where
        S: SphinxSuite + PartialEq,
        <S::P as Keypair>::Public: Eq + Hash,
        for<'a> &'a Alpha<<<S as SphinxSuite>::G as GroupElement<<S as SphinxSuite>::E>>::AlphaLen>:
            From<&'a <<S as SphinxSuite>::P as Keypair>::Public>,
    {
        let pubkeys = keypairs.iter().map(|kp| kp.public().clone()).collect::<Vec<_>>();
        let mapper = keypairs
            .iter()
            .enumerate()
            .map(|(i, k)| (KeyIdent::from(i as u32), k.public().clone()))
            .collect::<BiHashMap<_, _>>();

        let shared_keys = S::new_shared_keys(&pubkeys)?;
        let por_strings = vec![WrappedBytes::<53>::default(); shared_keys.secrets.len() - 1];
        let pseudonym = SimplePseudonym::random();

        let packet_1 = PartialPacket::<S, TestHeader<S>>::new(
            MetaPacketRouting::ForwardPath {
                shared_keys,
                forward_path: &pubkeys,
                additional_data_relayer: &por_strings,
                receiver_data: &pseudonym,
                no_ack: false,
            },
            &mapper,
        )?;

        const BINCODE_CONFIGURATION: bincode::config::Configuration = bincode::config::standard()
            .with_little_endian()
            .with_variable_int_encoding();

        let encoded_1 = bincode::serde::encode_to_vec(&packet_1, BINCODE_CONFIGURATION)?;
        let packet_2: PartialPacket<S, TestHeader<S>> =
            bincode::serde::decode_from_slice(&encoded_1, BINCODE_CONFIGURATION)?.0;

        assert_eq!(packet_1, packet_2);
        Ok(())
    }

    fn generic_test_meta_packet<S>(keypairs: Vec<S::P>) -> anyhow::Result<()>
    where
        S: SphinxSuite,
        <S::P as Keypair>::Public: Eq + Hash,
        for<'a> &'a Alpha<<S::G as GroupElement<S::E>>::AlphaLen>: From<&'a <S::P as Keypair>::Public>,
    {
        let pubkeys = keypairs.iter().map(|kp| kp.public().clone()).collect::<Vec<_>>();
        let mapper = keypairs
            .iter()
            .enumerate()
            .map(|(i, k)| (KeyIdent::from(i as u32), k.public().clone()))
            .collect::<BiHashMap<_, _>>();

        let shared_keys = S::new_shared_keys(&pubkeys)?;
        let por_strings = vec![WrappedBytes::<53>::default(); shared_keys.secrets.len() - 1];
        let pseudonym = SimplePseudonym::random();

        assert_eq!(shared_keys.secrets.len() - 1, por_strings.len());

        let msg = b"some random test message";

        let mut mp = MetaPacket::<S, TestHeader<S>, PAYLOAD_SIZE>::new(
            PaddedPayload::new(msg)?,
            MetaPacketRouting::ForwardPath {
                shared_keys,
                forward_path: &pubkeys,
                additional_data_relayer: &por_strings,
                receiver_data: &pseudonym,
                no_ack: false,
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
                ForwardedMetaPacket::Final { plain_text, .. } => {
                    assert_eq!(keypairs.len() - 1, i);
                    received_plaintext = plain_text.into_unpadded()?;
                }
            }
        }

        assert_eq!(msg, received_plaintext.as_ref());

        Ok(())
    }

    fn generic_meta_packet_reply_test<S>(keypairs: Vec<S::P>) -> anyhow::Result<()>
    where
        S: SphinxSuite,
        <S::P as Keypair>::Public: Eq + Hash,
        for<'a> &'a Alpha<<<S as SphinxSuite>::G as GroupElement<<S as SphinxSuite>::E>>::AlphaLen>:
            From<&'a <<S as SphinxSuite>::P as Keypair>::Public>,
    {
        let pubkeys = keypairs.iter().map(|kp| kp.public().clone()).collect::<Vec<_>>();
        let mapper = keypairs
            .iter()
            .enumerate()
            .map(|(i, k)| (KeyIdent::from(i as u32), k.public().clone()))
            .collect::<BiHashMap<_, _>>();

        let shared_keys = S::new_shared_keys(&pubkeys)?;
        let por_strings = vec![WrappedBytes::default(); shared_keys.secrets.len() - 1];
        let por_values = WrappedBytes::default();
        let pseudonym = SimplePseudonym::random();

        let ids = <BiHashMap<_, _> as KeyIdMapper<S, TestHeader<S>>>::map_keys_to_ids(&mapper, &pubkeys)
            .into_iter()
            .map(|v| v.ok_or_else(|| anyhow!("failed to map keys to ids")))
            .collect::<anyhow::Result<Vec<KeyIdent>>>()?;

        let (surb, opener) = create_surb::<S, TestHeader<S>>(shared_keys, &ids, &por_strings, pseudonym, por_values)?;

        let msg = b"some random reply test message";

        let mut mp = MetaPacket::<S, TestHeader<S>, PAYLOAD_SIZE>::new(
            PaddedPayload::new(msg)?,
            MetaPacketRouting::Surb(surb, &pseudonym),
            &mapper,
        )?;

        let surb_retriever = |p: &SimplePseudonym| {
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
                ForwardedMetaPacket::Final { plain_text, .. } => {
                    assert_eq!(keypairs.len() - 1, i);
                    received_plaintext = plain_text.into_unpadded()?;
                }
            }
        }

        assert_eq!(msg, received_plaintext.as_ref());

        Ok(())
    }

    #[cfg(feature = "x25519")]
    #[parameterized(amount = { 4, 3, 2, 1 })]
    fn test_x25519_meta_packet(amount: usize) -> anyhow::Result<()> {
        generic_test_meta_packet::<crate::ec_groups::X25519Suite>(
            (0..amount).map(|_| OffchainKeypair::random()).collect(),
        )
    }

    #[cfg(feature = "x25519")]
    #[parameterized(amount = { 4, 3, 2, 1 })]
    fn test_x25519_reply_meta_packet(amount: usize) -> anyhow::Result<()> {
        generic_meta_packet_reply_test::<crate::ec_groups::X25519Suite>(
            (0..amount).map(|_| OffchainKeypair::random()).collect(),
        )
    }

    #[cfg(all(feature = "x25519", feature = "serde"))]
    #[parameterized(amount = { 4, 3, 2, 1 })]
    fn test_x25519_partial_packet_serialize(amount: usize) -> anyhow::Result<()> {
        generic_test_partial_packet_serialization::<crate::ec_groups::X25519Suite>(
            (0..amount).map(|_| OffchainKeypair::random()).collect(),
        )
    }

    #[cfg(feature = "ed25519")]
    #[parameterized(amount = { 4, 3, 2, 1 })]
    fn test_ed25519_meta_packet(amount: usize) -> anyhow::Result<()> {
        generic_test_meta_packet::<crate::ec_groups::Ed25519Suite>(
            (0..amount).map(|_| OffchainKeypair::random()).collect(),
        )
    }

    #[cfg(feature = "ed25519")]
    #[parameterized(amount = { 4, 3, 2, 1 })]
    fn test_ed25519_reply_meta_packet(amount: usize) -> anyhow::Result<()> {
        generic_meta_packet_reply_test::<crate::ec_groups::Ed25519Suite>(
            (0..amount).map(|_| OffchainKeypair::random()).collect(),
        )
    }

    #[cfg(all(feature = "ed25519", feature = "serde"))]
    #[parameterized(amount = { 4, 3, 2, 1 })]
    fn test_ed25519_partial_packet_serialize(amount: usize) -> anyhow::Result<()> {
        generic_test_partial_packet_serialization::<crate::ec_groups::Ed25519Suite>(
            (0..amount).map(|_| OffchainKeypair::random()).collect(),
        )
    }

    #[cfg(feature = "secp256k1")]
    #[parameterized(amount = { 4, 3, 2, 1 })]
    fn test_secp256k1_meta_packet(amount: usize) -> anyhow::Result<()> {
        generic_test_meta_packet::<crate::ec_groups::Secp256k1Suite>(
            (0..amount)
                .map(|_| hopr_crypto_types::keypairs::ChainKeypair::random())
                .collect(),
        )
    }

    #[cfg(feature = "secp256k1")]
    #[parameterized(amount = { 4, 3, 2, 1 })]
    fn test_secp256k1_reply_meta_packet(amount: usize) -> anyhow::Result<()> {
        generic_meta_packet_reply_test::<crate::ec_groups::Secp256k1Suite>(
            (0..amount)
                .map(|_| hopr_crypto_types::keypairs::ChainKeypair::random())
                .collect(),
        )
    }

    #[cfg(all(feature = "secp256k1", feature = "serde"))]
    #[parameterized(amount = { 4, 3, 2, 1 })]
    fn test_secp256k1_partial_packet_serialize(amount: usize) -> anyhow::Result<()> {
        generic_test_partial_packet_serialization::<crate::ec_groups::Secp256k1Suite>(
            (0..amount)
                .map(|_| hopr_crypto_types::keypairs::ChainKeypair::random())
                .collect(),
        )
    }
}
