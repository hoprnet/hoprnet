use crate::errors::PacketError::PacketDecodingError;
use hopr_crypto_sphinx::{
    derivation::derive_packet_tag,
    prp::{PRPParameters, PRP},
    routing::{forward_header, header_length, ForwardedHeader, RoutingInfo},
    shared_keys::{Alpha, GroupElement, SharedKeys, SharedSecret, SphinxSuite},
};
use hopr_crypto_types::{
    keypairs::Keypair,
    primitives::{DigestLike, SimpleMac},
    types::PacketTag,
};
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use typenum::Unsigned;

use crate::{
    errors::Result,
    packet::ForwardedMetaPacket::{FinalPacket, RelayedPacket},
    por::POR_SECRET_LENGTH,
};

/// Currently used ciphersuite for Sphinx
pub type CurrentSphinxSuite = hopr_crypto_sphinx::ec_groups::X25519Suite;

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

/// An encrypted packet.
///
/// A sender can create a new packet via [MetaPacket::new] and send it.
/// Once received by the recipient, it is parsed first by calling [MetaPacket::from_bytes]
/// and then it can be transformed into [ForwardedMetaPacket] by calling
/// the [MetaPacket::forward] method. The [ForwardedMetaPacket] then contains the information
/// about the next recipient of this packet.
///
/// The packet format is directly dependent on the used [SphinxSuite](hopr_crypto_sphinx::shared_keys::SphinxSuite).
pub struct MetaPacket<S: SphinxSuite> {
    packet: Box<[u8]>,
    alpha: Alpha<<S::G as GroupElement<S::E>>::AlphaLen>,
}

// Needs manual Clone implementation to not impose Clone restriction on S
impl<S: SphinxSuite> Clone for MetaPacket<S> {
    fn clone(&self) -> Self {
        Self {
            packet: self.packet.clone(),
            alpha: self.alpha.clone()
        }
    }
}

/// Represent a [MetaPacket] with one layer of encryption removed, exposing the details
/// about the next hop.
///
/// There are two possible states - either the packet is intended for the recipient,
/// and is thus [Final](ForwardedMetaPacket::FinalPacket), or it is meant to be sent (relayed)
/// to the next hop - thus it is [Relayed](ForwardedMetaPacket::RelayedPacket).
#[allow(dead_code)]
pub enum ForwardedMetaPacket<S: SphinxSuite> {
    /// The content is another [MetaPacket] meant to be sent to the next hop.
    RelayedPacket {
        /// Packet for the next hop.
        packet: MetaPacket<S>,
        /// Public key of the next hop.
        next_node: <S::P as Keypair>::Public,
        /// Position in the channel path of this packet.
        path_pos: u8,
        /// Contains the PoR challenge that will be solved when we receive
        /// the acknowledgement after we forward the inner packet to the next hop.
        additional_info: Box<[u8]>,
        /// Shared secret that was used to encrypt the removed layer.
        derived_secret: SharedSecret,
        /// Packet checksum.
        packet_tag: PacketTag,
    },
    /// The content is the actual payload for the packet's destination.
    FinalPacket {
        /// Decrypted payload
        plain_text: Box<[u8]>,
        /// Reserved. Currently not used
        additional_data: Box<[u8]>,
        /// Shared secret that was used to encrypt the removed layer.
        derived_secret: SharedSecret,
        /// Packet checksum.
        packet_tag: PacketTag,
    },
}

impl<S: SphinxSuite> MetaPacket<S> {
    /// Fixed length of the Sphinx packet header.
    pub const HEADER_LEN: usize = header_length::<S>(INTERMEDIATE_HOPS + 1, POR_SECRET_LENGTH, 0);

    /// Creates a new outgoing packet with given payload `msg`, `path` and `shared_keys` computed along the path.
    ///
    /// The `additional_data_relayer` contain the PoR challenges for the individual relayers along the path,
    /// each of the challenges have the same size of `additional_relayer_data_len`.
    ///
    /// Optionally, there could be some additional data (`additional_data_last_hop`) for the packet destination.
    /// This is reserved for the future use by SURBs.
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

        let mut payload = add_padding(msg);
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
            prp.forward_inplace(&mut payload)
                .unwrap_or_else(|e| panic!("onion encryption error {e}"))
        }

        Self::new_from_parts(shared_keys.alpha, routing_info, &payload)
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
        assert_eq!(PAYLOAD_SIZE, payload.len(), "payload has incorrect length");

        let mut packet = Vec::with_capacity(Self::SIZE);
        packet.extend_from_slice(&alpha);
        packet.extend_from_slice(&routing_info.routing_information);
        packet.extend_from_slice(&routing_info.mac);
        packet.extend_from_slice(payload);

        Self {
            packet: packet.into_boxed_slice(),
            alpha,
        }
    }

    pub fn routing_info(&self) -> &[u8] {
        let base = <S::G as GroupElement<S::E>>::AlphaLen::USIZE;
        &self.packet[base..base + Self::HEADER_LEN]
    }

    pub fn mac(&self) -> &[u8] {
        let base = <S::G as GroupElement<S::E>>::AlphaLen::USIZE + Self::HEADER_LEN;
        &self.packet[base..base + SimpleMac::SIZE]
    }

    pub fn payload(&self) -> &[u8] {
        let base = <S::G as GroupElement<S::E>>::AlphaLen::USIZE + Self::HEADER_LEN + SimpleMac::SIZE;
        &self.packet[base..base + PAYLOAD_SIZE]
    }

    /// Attempts to remove the layer of encryption of this packet by using the given `node_keypair`.
    /// This will transform this packet into the [ForwardedMetaPacket].
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

impl<S: SphinxSuite> BinarySerializable for MetaPacket<S> {
    const SIZE: usize =
        <S::G as GroupElement<S::E>>::AlphaLen::USIZE + Self::HEADER_LEN + SimpleMac::SIZE + PAYLOAD_SIZE;

    fn from_bytes(data: &[u8]) -> hopr_primitive_types::errors::Result<Self> {
        if data.len() == Self::SIZE {
            let mut ret = Self {
                packet: data.into(),
                alpha: Default::default(),
            };
            ret.alpha
                .copy_from_slice(&data[..<S::G as GroupElement<S::E>>::AlphaLen::USIZE]);
            Ok(ret)
        } else {
            Err(GeneralError::ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        self.packet.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::packet::{add_padding, remove_padding, ForwardedMetaPacket, MetaPacket, PADDING_TAG};
    use crate::por::{ProofOfRelayString, POR_SECRET_LENGTH};
    use hopr_crypto_sphinx::{
        ec_groups::{Ed25519Suite, Secp256k1Suite, X25519Suite},
        shared_keys::SphinxSuite,
    };
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair, OffchainKeypair};
    use hopr_internal_types::protocol::INTERMEDIATE_HOPS;
    use parameterized::parameterized;

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
                .unwrap_or_else(|_| panic!("failed to unwrap at {i}"));

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
    fn test_x25519_meta_packet(amount: usize) {
        generic_test_meta_packet::<X25519Suite>((0..amount).map(|_| OffchainKeypair::random()).collect())
    }

    #[parameterized(amount = { 4, 3, 2 })]
    fn test_ed25519_meta_packet(amount: usize) {
        generic_test_meta_packet::<Ed25519Suite>((0..amount).map(|_| OffchainKeypair::random()).collect());
    }

    #[parameterized(amount = { 4, 3, 2 })]
    fn test_secp256k1_meta_packet(amount: usize) {
        generic_test_meta_packet::<Secp256k1Suite>((0..amount).map(|_| ChainKeypair::random()).collect())
    }
}
