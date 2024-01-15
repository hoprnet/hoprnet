use crate::errors::PacketError::PacketDecodingError;
use hopr_internal_types::protocol::{INTERMEDIATE_HOPS, PAYLOAD_SIZE};
use hopr_crypto::{
    derivation::{derive_packet_tag, PacketTag},
    keypairs::Keypair,
    primitives::{DigestLike, SimpleMac},
    prp::{PRPParameters, PRP},
    routing::{forward_header, header_length, ForwardedHeader, RoutingInfo},
    shared_keys::{Alpha, GroupElement, SharedKeys, SharedSecret, SphinxSuite},
};
use typenum::Unsigned;
use hopr_primitive_types::{errors::GeneralError::ParseError, traits::BinarySerializable};

use crate::{
    errors::Result,
    packet::ForwardedMetaPacket::{FinalPacket, RelayedPacket},
    por::POR_SECRET_LENGTH,
};

/// Currently used ciphersuite for Sphinx
pub type CurrentSphinxSuite = hopr_crypto::ec_groups::X25519Suite;

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

pub struct MetaPacket<S: SphinxSuite> {
    packet: Box<[u8]>,
    alpha: Alpha<<S::G as GroupElement<S::E>>::AlphaLen>,
}

#[allow(dead_code)]
pub enum ForwardedMetaPacket<S: SphinxSuite> {
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
    pub const HEADER_LEN: usize = header_length::<S>(INTERMEDIATE_HOPS + 1, POR_SECRET_LENGTH, 0);

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
            Err(ParseError)
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
    use hopr_internal_types::protocol::INTERMEDIATE_HOPS;
    use hopr_crypto::{
        ec_groups::{Ed25519Suite, Secp256k1Suite, X25519Suite},
        keypairs::{ChainKeypair, Keypair, OffchainKeypair},
        shared_keys::SphinxSuite,
    };
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
    #[ignore]
    fn test_ed25519_meta_packet(amount: usize) {
        generic_test_meta_packet::<Ed25519Suite>((0..amount).map(|_| OffchainKeypair::random()).collect());
    }

    #[parameterized(amount = { 4, 3, 2 })]
    fn test_x25519_meta_packet(amount: usize) {
        generic_test_meta_packet::<X25519Suite>((0..amount).map(|_| OffchainKeypair::random()).collect())
    }

    #[parameterized(amount = { 4, 3, 2 })]
    #[ignore]
    fn test_secp256k1_meta_packet(amount: usize) {
        generic_test_meta_packet::<Secp256k1Suite>((0..amount).map(|_| ChainKeypair::random()).collect())
    }
}
