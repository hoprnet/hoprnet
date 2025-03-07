use crate::errors::PacketError;
use crate::errors::{PacketError::PacketDecodingError, Result};
use hopr_crypto_sphinx::routing::SphinxHeaderSpec;
use hopr_crypto_sphinx::{
    derivation::derive_packet_tag,
    prp::{PRPParameters, PRP},
    routing::{forward_header, ForwardedHeader, RoutingInfo},
    shared_keys::{Alpha, GroupElement, SharedKeys, SharedSecret, SphinxSuite},
};
use hopr_crypto_types::{
    keypairs::Keypair,
    primitives::{DigestLike, SimpleMac},
    types::PacketTag,
};
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use typenum::Unsigned;

/// Tag used to separate padding from data
const PADDING_TAG: &[u8] = b"HOPR";

// TODO: Make padding length prefixed in 3.0

fn add_padding(msg: &[u8]) -> Box<[u8]> {
    assert!(msg.len() <= PAYLOAD_SIZE, "message too long for padding");

    let padded_len = PAYLOAD_SIZE + PADDING_TAG.len();
    let mut ret = vec![0u8; padded_len];
    ret[padded_len - msg.len()..padded_len].copy_from_slice(msg);

    // Zeroes and the PADDING_TAG are prepended to the message to form padding
    ret[padded_len - msg.len() - PADDING_TAG.len()..padded_len - msg.len()].copy_from_slice(PADDING_TAG);
    ret.into_boxed_slice()
}

fn remove_padding(msg: &[u8]) -> Option<&[u8]> {
    assert_eq!(
        PAYLOAD_SIZE + PADDING_TAG.len(),
        msg.len(),
        "padded message must be {} bytes long",
        PAYLOAD_SIZE + PADDING_TAG.len()
    );
    let pos = msg
        .windows(PADDING_TAG.len())
        .position(|window| window == PADDING_TAG)?;
    Some(&msg.split_at(pos).1[PADDING_TAG.len()..])
}

pub trait KeyIdMapper<S: SphinxSuite, H: SphinxHeaderSpec> {
    fn map_key_to_id(&self, key_id: &<S::P as Keypair>::Public) -> Result<H::KeyId>;
    fn map_id_to_public(&self, key_id: &H::KeyId) -> Result<<S::P as Keypair>::Public>;
}

pub struct SimpleKeyMapper<S, H> {
    _s: PhantomData<S>,
    _h: PhantomData<H>,
}

impl<S, H> Default for SimpleKeyMapper<S, H> {
    fn default() -> Self {
        Self {
            _s: PhantomData,
            _h: PhantomData,
        }
    }
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> KeyIdMapper<S, H> for SimpleKeyMapper<S, H> {
    fn map_key_to_id(&self, key_id: &<S::P as Keypair>::Public) -> Result<H::KeyId> {
        H::KeyId::try_from(key_id.as_ref()).map_err(|_| PacketError::PacketConstructionError("invalid key id".into()))
    }

    fn map_id_to_public(&self, key_id: &H::KeyId) -> Result<<S::P as Keypair>::Public> {
        <S::P as Keypair>::Public::try_from(key_id.as_ref())
            .map_err(|_| PacketError::PacketConstructionError("invalid key".into()))
    }
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

impl<S, H> Debug for MetaPacket<S, H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", hex::encode(&self.packet))
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
        /// Contains the PoR challenge that will be solved when we receive
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
        /// Reserved for SURBs. Currently not used
        additional_data: H::LastHopData,
        /// Shared secret that was used to encrypt the removed layer.
        derived_secret: SharedSecret,
        /// Packet checksum.
        packet_tag: PacketTag,
    },
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> MetaPacket<S, H> {
    /// The fixed length of the Sphinx packet header.
    pub const HEADER_LEN: usize = H::HEADER_LEN;

    /// The fixed length of the padded packet.
    pub const PACKET_LEN: usize =
        <S::P as Keypair>::Public::SIZE + H::HEADER_LEN + SimpleMac::SIZE + PAYLOAD_SIZE + PADDING_TAG.len();

    /// Creates a new outgoing packet with the given payload `msg`, `path` and `shared_keys` computed along the path.
    ///
    /// The size of the `msg` must be less or equal [PAYLOAD_SIZE], otherwise the
    /// constructor will panic. The caller **must** ensure the size is correct beforehand.
    /// The `additional_data_relayer` contains the PoR challenges for the individual relayers along the path,
    /// each of the challenges has the same size of `additional_relayer_data_len`.
    ///
    /// Optionally, there could be some additional data (`additional_data_last_hop`) for the packet destination.
    /// This is being used to transfer [`Pseudonym`](hopr_crypto_sphinx::surb::Pseudonym) for SURBs.
    pub fn new<M: KeyIdMapper<S, H>>(
        shared_keys: SharedKeys<S::E, S::G>,
        msg: &[u8],
        path: &[<S::P as Keypair>::Public],
        key_mapper: &M,
        additional_data_relayer: &[H::RelayerData],
        additional_data_last_hop: H::LastHopData,
    ) -> Result<Self> {
        if msg.len() > PAYLOAD_SIZE {
            return Err(PacketError::PacketConstructionError(
                "message too long to fit into a packet".into(),
            ));
        }

        let mut payload = add_padding(msg);

        let routing_info = RoutingInfo::new::<H>(
            &path
                .iter()
                .map(|key| key_mapper.map_key_to_id(key))
                .collect::<Result<Vec<_>>>()?,
            &shared_keys.secrets,
            additional_data_relayer,
            additional_data_last_hop,
        )?;

        // Encrypt the packet payload using the derived shared secrets
        for secret in shared_keys.secrets.iter().rev() {
            let prp = PRP::from_parameters(PRPParameters::new(secret));
            prp.forward_inplace(&mut payload)
                .unwrap_or_else(|e| panic!("onion encryption error {e}"))
        }

        Ok(Self::new_from_parts(shared_keys.alpha, routing_info, &payload))
    }

    fn new_from_parts(
        alpha: Alpha<<S::G as GroupElement<S::E>>::AlphaLen>,
        routing_info: RoutingInfo,
        payload: &[u8],
    ) -> Self {
        assert!(
            !routing_info.routing_information.is_empty(),
            "routing info must not be empty"
        );
        assert_eq!(
            PAYLOAD_SIZE + PADDING_TAG.len(),
            payload.len(),
            "payload has incorrect length"
        );

        let mut packet = Vec::with_capacity(Self::SIZE);
        packet.extend_from_slice(&alpha);
        packet.extend_from_slice(&routing_info.routing_information);
        packet.extend_from_slice(&routing_info.mac);
        packet.extend_from_slice(payload);

        Self {
            packet: packet.into_boxed_slice(),
            _s: PhantomData,
            _h: PhantomData,
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
        &mut self.packet[base..base + Self::HEADER_LEN]
    }

    /// Returns the packet checksum (MAC) subslice from the packet data.
    fn mac(&self) -> &[u8] {
        let base = <S::G as GroupElement<S::E>>::AlphaLen::USIZE + Self::HEADER_LEN;
        &self.packet[base..base + SimpleMac::SIZE]
    }

    /// Returns the payload subslice from the packet data.
    fn payload(&self) -> &[u8] {
        let base = <S::G as GroupElement<S::E>>::AlphaLen::USIZE + Self::HEADER_LEN + SimpleMac::SIZE;
        &self.packet[base..base + PAYLOAD_SIZE + PADDING_TAG.len()]
    }

    /// Attempts to remove the layer of encryption of this packet by using the given `node_keypair`.
    /// This will transform this packet into the [ForwardedMetaPacket].
    pub fn into_forwarded<K: KeyIdMapper<S, H>>(
        mut self,
        node_keypair: &S::P,
        key_mapper: &K,
    ) -> Result<ForwardedMetaPacket<S, H>> {
        let (alpha, secret) = SharedKeys::<S::E, S::G>::forward_transform(
            Alpha::<<S::G as GroupElement<S::E>>::AlphaLen>::from_slice(self.alpha()),
            &(node_keypair.into()),
            &(node_keypair.public().into()),
        )?;

        let mac_cpy = self.mac().to_vec();
        let fwd_header = forward_header::<H>(&secret, self.routing_info_mut(), &mac_cpy)?;

        let prp = PRP::from_parameters(PRPParameters::new(&secret));
        let decrypted = prp.inverse(self.payload())?;

        Ok(match fwd_header {
            ForwardedHeader::RelayNode {
                header,
                mac,
                path_pos,
                next_node,
                additional_info,
            } => ForwardedMetaPacket::Relayed {
                packet: Self::new_from_parts(
                    alpha,
                    RoutingInfo {
                        routing_information: header,
                        mac,
                    },
                    &decrypted,
                ),
                packet_tag: derive_packet_tag(&secret),
                derived_secret: secret,
                next_node: key_mapper.map_id_to_public(&next_node)?,
                path_pos,
                additional_info,
            },
            ForwardedHeader::FinalNode { additional_data } => ForwardedMetaPacket::Final {
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
    const SIZE: usize = <S::G as GroupElement<S::E>>::AlphaLen::USIZE
        + Self::HEADER_LEN
        + SimpleMac::SIZE
        + PAYLOAD_SIZE
        + PADDING_TAG.len();
}

#[cfg(test)]
mod tests {
    use super::*;

    use hopr_crypto_sphinx::ec_groups::{Ed25519Suite, Secp256k1Suite, X25519Suite};
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair, OffchainKeypair};
    use parameterized::parameterized;
    use std::num::NonZeroUsize;

    use crate::por::{ProofOfRelayString, POR_SECRET_LENGTH};

    struct TestHeader<S: SphinxSuite>(PhantomData<S>);

    impl<S: SphinxSuite> SphinxHeaderSpec for TestHeader<S> {
        const MAX_HOPS: NonZeroUsize = NonZeroUsize::new(INTERMEDIATE_HOPS + 1).unwrap();
        const KEY_ID_SIZE: NonZeroUsize = NonZeroUsize::new(<S::P as Keypair>::Public::SIZE).unwrap();
        type KeyId = <S::P as Keypair>::Public;

        const RELAYER_DATA_SIZE: usize = POR_SECRET_LENGTH;
        type RelayerData = [u8; POR_SECRET_LENGTH];
        const LAST_HOP_DATA_SIZE: usize = 0;
        type LastHopData = [u8; 0];
    }

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

    fn generic_test_meta_packet<S: SphinxSuite>(keypairs: Vec<S::P>) -> anyhow::Result<()> {
        let pubkeys = keypairs.iter().map(|kp| kp.public().clone()).collect::<Vec<_>>();

        let shared_keys = S::new_shared_keys(&pubkeys)?;
        let por_strings = ProofOfRelayString::from_shared_secrets(&shared_keys.secrets);

        assert_eq!(shared_keys.secrets.len() - 1, por_strings.len());

        let msg = b"some random test message";

        let mut mp = MetaPacket::<S, TestHeader<S>>::new(
            shared_keys,
            msg,
            &pubkeys,
            &SimpleKeyMapper::default(),
            &por_strings,
            [],
        )?;

        assert!(mp.as_ref().len() < 1492, "metapacket too long {}", mp.as_ref().len());

        for (i, pair) in keypairs.iter().enumerate() {
            let fwd = mp
                .clone()
                .into_forwarded(pair, &SimpleKeyMapper::default())
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
                    assert_eq!(msg, plain_text.as_ref());
                    assert!(additional_data.is_empty());
                }
            }
        }

        Ok(())
    }

    #[parameterized(amount = { 4, 3, 2 })]
    fn test_x25519_meta_packet(amount: usize) -> anyhow::Result<()> {
        generic_test_meta_packet::<X25519Suite>((0..amount).map(|_| OffchainKeypair::random()).collect())
    }

    #[parameterized(amount = { 4, 3, 2 })]
    fn test_ed25519_meta_packet(amount: usize) -> anyhow::Result<()> {
        generic_test_meta_packet::<Ed25519Suite>((0..amount).map(|_| OffchainKeypair::random()).collect())
    }

    #[parameterized(amount = { 4, 3, 2 })]
    fn test_secp256k1_meta_packet(amount: usize) -> anyhow::Result<()> {
        generic_test_meta_packet::<Secp256k1Suite>((0..amount).map(|_| ChainKeypair::random()).collect())
    }
}
