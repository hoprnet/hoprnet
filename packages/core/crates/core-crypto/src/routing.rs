use crate::derivation::derive_mac_key;
use crate::errors::CryptoError::TagMismatch;
use crate::errors::Result;
use crate::keypairs::Keypair;
use crate::prg::{PRGParameters, PRG};
use crate::primitives::{DigestLike, SecretKey, SimpleMac};
use crate::random::random_fill;
use crate::routing::ForwardedHeader::{FinalNode, RelayNode};
use crate::shared_keys::{SharedSecret, SphinxSuite};
use crate::utils::xor_inplace;
use utils_types::traits::BinarySerializable;

const RELAYER_END_PREFIX: u8 = 0xff;

const PATH_POSITION_LEN: usize = 1;

/// Length of the header routing information per hop
const fn routing_information_length<S: SphinxSuite>(additional_data_relayer_len: usize) -> usize {
    <S::P as Keypair>::Public::SIZE + PATH_POSITION_LEN + SimpleMac::SIZE + additional_data_relayer_len
}

/// Returns the size of the packet header given the information about the number of hops and additional relayer info.
pub const fn header_length<S: SphinxSuite>(
    max_hops: usize,
    additional_data_relayer_len: usize,
    additional_data_last_hop_len: usize,
) -> usize {
    let last_hop = additional_data_last_hop_len + 1; // 1 = end prefix length
    last_hop + (max_hops - 1) * routing_information_length::<S>(additional_data_relayer_len)
}

fn generate_filler(
    max_hops: usize,
    routing_info_len: usize,
    routing_info_last_hop_len: usize,
    secrets: &[SharedSecret],
) -> Box<[u8]> {
    if secrets.len() < 2 {
        return vec![].into_boxed_slice();
    }

    assert!(max_hops >= secrets.len(), "too few hops");
    assert!(routing_info_len > 0, "invalid routing info length");

    let header_len = routing_info_last_hop_len + (max_hops - 1) * routing_info_len;
    let padding_len = (max_hops - secrets.len()) * routing_info_len;

    let mut ret = vec![0u8; header_len - padding_len - routing_info_last_hop_len];

    let mut length = routing_info_len;
    let mut start = header_len;

    for secret in secrets.iter().take(secrets.len() - 1) {
        let prg = PRG::from_parameters(PRGParameters::new(secret));

        let digest = prg.digest(start, header_len + routing_info_len);
        xor_inplace(&mut ret[0..length], digest.as_ref());

        length += routing_info_len;
        start -= routing_info_len;
    }

    ret.into_boxed_slice()
}

/// Carries routing information for the mixnet packet.
pub struct RoutingInfo {
    pub routing_information: Box<[u8]>,
    pub mac: [u8; SimpleMac::SIZE],
}

impl Default for RoutingInfo {
    fn default() -> Self {
        Self {
            routing_information: Box::default(),
            mac: [0u8; SimpleMac::SIZE],
        }
    }
}

impl RoutingInfo {
    /// Creates the routing information of the mixnet packet
    /// # Arguments
    /// * `max_hops` maximal number of hops
    /// * `path` IDs of the nodes along the path
    /// * `secrets` shared secrets with the nodes along the path
    /// * `additional_data_relayer_len` length of each additional data for all relayers
    /// * `additional_data_relayer` additional data for each relayer
    /// * `additional_data_last_hop` additional data for the final recipient
    pub fn new<S: SphinxSuite>(
        max_hops: usize,
        path: &[<S::P as Keypair>::Public],
        secrets: &[SharedSecret],
        additional_data_relayer_len: usize,
        additional_data_relayer: &[&[u8]],
        additional_data_last_hop: Option<&[u8]>,
    ) -> Self {
        assert!(
            secrets.len() <= max_hops && !secrets.is_empty(),
            "invalid number of secrets given"
        );
        assert!(
            additional_data_relayer
                .iter()
                .all(|r| r.len() == additional_data_relayer_len),
            "invalid relayer data length"
        );
        assert!(
            additional_data_last_hop.is_none() || !additional_data_last_hop.unwrap().is_empty(),
            "invalid additional data for last hop"
        );

        let routing_info_len = routing_information_length::<S>(additional_data_relayer_len);
        let last_hop_len = additional_data_last_hop.map(|d| d.len()).unwrap_or(0) + 1; // end prefix length
        let header_len = header_length::<S>(max_hops, additional_data_relayer_len, last_hop_len - 1);

        let extended_header_len = last_hop_len + max_hops * routing_info_len;

        let mut extended_header = vec![0u8; extended_header_len];
        let mut ret = RoutingInfo::default();

        for idx in 0..secrets.len() {
            let inverted_idx = secrets.len() - idx - 1;
            let prg = PRG::from_parameters(PRGParameters::new(&secrets[inverted_idx]));

            if idx == 0 {
                extended_header[0] = RELAYER_END_PREFIX;

                if let Some(data) = additional_data_last_hop {
                    extended_header[1..data.len()].copy_from_slice(data);
                }

                let padding_len = (max_hops - secrets.len()) * routing_info_len;
                if padding_len > 0 {
                    random_fill(&mut extended_header[last_hop_len..padding_len]);
                }

                let key_stream = prg.digest(0, last_hop_len + padding_len);
                xor_inplace(&mut extended_header[0..last_hop_len + padding_len], &key_stream);

                if secrets.len() > 1 {
                    let filler = generate_filler(max_hops, routing_info_len, last_hop_len, secrets);
                    extended_header[last_hop_len + padding_len..last_hop_len + padding_len + filler.len()]
                        .copy_from_slice(&filler);
                }
            } else {
                extended_header.copy_within(0..header_len, routing_info_len);

                let pub_key_size = <S::P as Keypair>::Public::SIZE;
                extended_header[0..pub_key_size].copy_from_slice(&path[inverted_idx + 1].to_bytes());
                extended_header[pub_key_size] = idx as u8;
                extended_header[pub_key_size + PATH_POSITION_LEN..pub_key_size + PATH_POSITION_LEN + SimpleMac::SIZE]
                    .copy_from_slice(&ret.mac);
                extended_header[pub_key_size + PATH_POSITION_LEN + SimpleMac::SIZE
                    ..pub_key_size + PATH_POSITION_LEN + SimpleMac::SIZE + additional_data_relayer[inverted_idx].len()]
                    .copy_from_slice(additional_data_relayer[inverted_idx]);

                let key_stream = prg.digest(0, header_len);
                xor_inplace(&mut extended_header, &key_stream);
            }

            let mut m = SimpleMac::new(&derive_mac_key(&secrets[inverted_idx]));
            m.update(&extended_header[0..header_len]);
            m.finalize_into(&mut ret.mac);
        }

        ret.routing_information = (&extended_header[0..header_len]).into();
        ret
    }
}

/// Enum carry information about the packet based on whether it is destined for the current node (`FinalNode`)
/// or if the packet is supposed to be only relayed (`RelayNode`).
pub enum ForwardedHeader {
    /// Packet is supposed to be relayed
    RelayNode {
        /// Transformed header
        header: Box<[u8]>,
        /// Authentication tag
        mac: Box<[u8]>,
        /// Position of the relay in the path
        path_pos: u8,
        /// Public key of the next node
        next_node: Box<[u8]>,
        /// Additional data for the relayer
        additional_info: Box<[u8]>,
    },

    /// Packet is at its final destination
    FinalNode {
        /// Additional data for the final destination
        additional_data: Box<[u8]>,
    },
}

/// Applies the forward transformation to the header.
/// If the packet is destined for this node, returns the additional data
/// for the final destination (`FinalNode`), otherwise it returns the transformed header, the
/// next authentication tag, the public key of the next node, and the additional data
/// for the relayer (`RelayNode`).
/// # Arguments
/// * `secret` shared secret with the creator of the packet
/// * `header` u8a containing the header
/// * `mac` current mac
/// * `max_hops` maximal number of hops
/// * `additional_data_relayer_len` length of the additional data for each relayer
/// * `additional_data_last_hop_len` length of the additional data for the final destination
pub fn forward_header<S: SphinxSuite>(
    secret: &SecretKey,
    header: &mut [u8],
    mac: &[u8],
    max_hops: usize,
    additional_data_relayer_len: usize,
    additional_data_last_hop_len: usize,
) -> Result<ForwardedHeader> {
    assert_eq!(SimpleMac::SIZE, mac.len(), "invalid mac length");

    let routing_info_len = routing_information_length::<S>(additional_data_relayer_len);
    let last_hop_len = additional_data_last_hop_len + 1; // end prefix
    let header_len = header_length::<S>(max_hops, additional_data_relayer_len, last_hop_len - 1);

    assert_eq!(header_len, header.len(), "invalid pre-header length");

    let mut computed_mac = SimpleMac::new(&derive_mac_key(secret));
    computed_mac.update(header);
    if !mac.eq(computed_mac.finalize().as_slice()) {
        return Err(TagMismatch);
    }

    // Unmask the header using the keystream
    let prg = PRG::from_parameters(PRGParameters::new(secret));
    let key_stream = prg.digest(0, header_len);
    xor_inplace(header, &key_stream);

    if header[0] != RELAYER_END_PREFIX {
        let pub_key_size = <S::P as Keypair>::Public::SIZE;

        // Try to deserialize the public key to validate it
        let next_node: Box<[u8]> = (&header[0..pub_key_size]).into();
        let path_pos: u8 = header[pub_key_size]; // Path position is the secret key index
        let mac: Box<[u8]> =
            (&header[pub_key_size + PATH_POSITION_LEN..pub_key_size + PATH_POSITION_LEN + SimpleMac::SIZE]).into();

        let additional_info: Box<[u8]> = (&header[pub_key_size + PATH_POSITION_LEN + SimpleMac::SIZE
            ..pub_key_size + PATH_POSITION_LEN + SimpleMac::SIZE + additional_data_relayer_len])
            .into();

        header.copy_within(routing_info_len.., 0);
        let key_stream = prg.digest(header_len, header_len + routing_info_len);
        header[header_len - routing_info_len..].copy_from_slice(&key_stream);

        Ok(RelayNode {
            header: (&header[..header_len]).into(),
            mac,
            path_pos,
            next_node,
            additional_info,
        })
    } else {
        Ok(FinalNode {
            additional_data: (&header[1..1 + additional_data_last_hop_len]).into(),
        })
    }
}

#[cfg(test)]
pub mod tests {
    use crate::ec_groups::{Ed25519Suite, Secp256k1Suite, X25519Suite};
    use crate::keypairs::{ChainKeypair, Keypair, OffchainKeypair};
    use crate::prg::{PRGParameters, PRG};
    use crate::primitives::{DigestLike, SimpleMac};
    use crate::routing::{forward_header, generate_filler, ForwardedHeader, RoutingInfo};
    use crate::shared_keys::{SharedSecret, SphinxSuite};
    use crate::utils::xor_inplace;
    use parameterized::parameterized;
    use utils_types::traits::BinarySerializable;

    #[parameterized(hops = { 3, 4 })]
    fn test_filler_generate_verify(hops: usize) {
        let per_hop = 3;
        let last_hop = 5;
        let max_hops = hops;

        let secrets = (0..hops).map(|_| SharedSecret::random()).collect::<Vec<_>>();
        let extended_header_len = per_hop * max_hops + last_hop;
        let header_len = per_hop * (max_hops - 1) + last_hop;

        let mut extended_header = vec![0u8; per_hop * max_hops + last_hop];

        let filler = generate_filler(max_hops, per_hop, last_hop, &secrets);

        extended_header[last_hop..last_hop + filler.len()].copy_from_slice(&filler);
        extended_header.copy_within(0..header_len, per_hop);

        for i in 0..hops - 1 {
            let idx = secrets.len() - i - 2;
            let mask = PRG::from_parameters(PRGParameters::new(&secrets[idx])).digest(0, extended_header_len);

            xor_inplace(&mut extended_header, &mask);

            assert!(
                extended_header[header_len..].iter().all(|x| *x == 0),
                "xor blinding must erase last bits"
            );

            extended_header.copy_within(0..header_len, per_hop);
        }
    }

    #[test]
    fn test_filler_edge_case() {
        let per_hop = 23;
        let last_hop = 31;
        let hops = 1;
        let max_hops = hops;

        let secrets = (0..hops).map(|_| SharedSecret::random()).collect::<Vec<_>>();

        let first_filler = generate_filler(max_hops, per_hop, last_hop, &secrets);
        assert_eq!(0, first_filler.len());

        let second_filler = generate_filler(0, per_hop, last_hop, &[]);
        assert_eq!(0, second_filler.len());
    }

    fn generic_test_generate_routing_info_and_forward<S>(keypairs: Vec<S::P>)
    where
        S: SphinxSuite,
    {
        const MAX_HOPS: usize = 3;
        let mut additional_data: Vec<&[u8]> = Vec::with_capacity(keypairs.len());
        for _ in 0..keypairs.len() {
            let e: &[u8] = &[];
            additional_data.push(e);
        }

        let pub_keys = keypairs.iter().map(|kp| kp.public().clone()).collect::<Vec<_>>();
        let shares = S::new_shared_keys(&pub_keys).unwrap();

        let rinfo = RoutingInfo::new::<S>(MAX_HOPS, &pub_keys, &shares.secrets, 0, &additional_data, None);

        let mut header: Vec<u8> = rinfo.routing_information.into();

        let mut last_mac = [0u8; SimpleMac::SIZE];
        last_mac.copy_from_slice(&rinfo.mac);

        for (i, secret) in shares.secrets.iter().enumerate() {
            let fwd = forward_header::<S>(secret, &mut header, &last_mac, MAX_HOPS, 0, 0).unwrap();

            match fwd {
                ForwardedHeader::RelayNode {
                    mac,
                    next_node,
                    path_pos,
                    ..
                } => {
                    last_mac.copy_from_slice(&mac);
                    assert!(i < shares.secrets.len() - 1, "cannot be a relay node");
                    assert_eq!(
                        path_pos,
                        (shares.secrets.len() - i - 1) as u8,
                        "invalid path position {path_pos}"
                    );
                    assert_eq!(
                        pub_keys[i + 1].to_bytes().as_ref(),
                        next_node.as_ref(),
                        "invalid public key of the next node"
                    );
                }
                ForwardedHeader::FinalNode { additional_data } => {
                    assert_eq!(shares.secrets.len() - 1, i, "cannot be a final node");
                    assert_eq!(0, additional_data.len(), "final node must not have any additional data");
                }
            }
        }
    }

    #[test]
    fn test() {
        generic_test_generate_routing_info_and_forward::<X25519Suite>(
            (0..3).map(|_| OffchainKeypair::random()).collect(),
        )
    }

    #[parameterized(amount = { 3, 2, 1 })]
    fn test_ed25519_generate_routing_info_and_forward(amount: usize) {
        generic_test_generate_routing_info_and_forward::<Ed25519Suite>(
            (0..amount).map(|_| OffchainKeypair::random()).collect(),
        )
    }

    #[parameterized(amount = { 3, 2, 1 })]
    fn test_x25519_generate_routing_info_and_forward(amount: usize) {
        generic_test_generate_routing_info_and_forward::<X25519Suite>(
            (0..amount).map(|_| OffchainKeypair::random()).collect(),
        )
    }

    #[parameterized(amount = { 3, 2, 1 })]
    fn test_secp256k1_generate_routing_info_and_forward(amount: usize) {
        generic_test_generate_routing_info_and_forward::<Secp256k1Suite>(
            (0..amount).map(|_| ChainKeypair::random()).collect(),
        )
    }
}
