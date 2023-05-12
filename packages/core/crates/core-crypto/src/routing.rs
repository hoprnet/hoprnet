use crate::derivation::derive_mac_key;
use crate::errors::CryptoError;
use crate::errors::CryptoError::TagMismatch;
use crate::errors::Result;
use crate::parameters::SECRET_KEY_LENGTH;
use crate::prg::{PRGParameters, PRG};
use crate::primitives::{create_tagged_mac, DigestLike, SimpleMac};
use crate::random::random_fill;
use crate::routing::ForwardedHeader::{FinalNode, RelayNode};
use crate::types::PublicKey;
use crate::utils::xor_inplace;
use std::ops::Not;
use subtle::ConstantTimeEq;

const RELAYER_END_PREFIX: u8 = 0xff;

/// Returns the size of the packet header given the information about the number of hops and additional relayer info.
pub const fn header_length(
    max_hops: usize,
    additional_data_relayer_len: usize,
    additional_data_last_hop_len: usize,
) -> usize {
    let per_hop = PublicKey::SIZE_COMPRESSED + SimpleMac::SIZE + additional_data_relayer_len;
    let last_hop = 1 + additional_data_last_hop_len;

    last_hop + (max_hops - 1) * per_hop
}

fn generate_filler(
    max_hops: usize,
    routing_info_len: usize,
    routing_info_last_hop_len: usize,
    secrets: &[&[u8]],
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

    for &secret in secrets.iter().take(secrets.len() - 1) {
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
    pub fn new(
        max_hops: usize,
        path: &[PublicKey],
        secrets: &[&[u8]],
        additional_data_relayer_len: usize,
        additional_data_relayer: &[&[u8]],
        additional_data_last_hop: Option<&[u8]>,
    ) -> Self {
        assert!(
            secrets.len() <= max_hops && !secrets.is_empty(),
            "invalid number of secrets given"
        );
        assert!(
            secrets.iter().all(|s| s.len() == SECRET_KEY_LENGTH),
            "invalid secret length"
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

        let routing_info_len = additional_data_relayer_len + SimpleMac::SIZE + PublicKey::SIZE_COMPRESSED;
        let last_hop_len = additional_data_last_hop.map(|d| d.len()).unwrap_or(0) + 1; // end prefix length

        let header_len = last_hop_len + (max_hops - 1) * routing_info_len;
        let extended_header_len = last_hop_len + max_hops * routing_info_len;

        let mut extended_header = vec![0u8; extended_header_len];
        let mut ret = RoutingInfo::default();

        for idx in 0..secrets.len() {
            let inverted_idx = secrets.len() - idx - 1;
            let secret = secrets[inverted_idx];
            let prg = PRG::from_parameters(PRGParameters::new(secret));

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
                extended_header[0..PublicKey::SIZE_COMPRESSED].copy_from_slice(&path[inverted_idx + 1].to_bytes(true));
                extended_header[PublicKey::SIZE_COMPRESSED..PublicKey::SIZE_COMPRESSED + SimpleMac::SIZE]
                    .copy_from_slice(&ret.mac);

                extended_header[PublicKey::SIZE_COMPRESSED + SimpleMac::SIZE
                    ..PublicKey::SIZE_COMPRESSED + SimpleMac::SIZE + additional_data_relayer[inverted_idx].len()]
                    .copy_from_slice(additional_data_relayer[inverted_idx]);

                let key_stream = prg.digest(0, header_len);
                xor_inplace(&mut extended_header, &key_stream);
            }

            let mut m = derive_mac_key(secret).and_then(|k| SimpleMac::new(&k)).unwrap();
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
        /// Public key of the next node
        next_node: PublicKey,
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
pub fn forward_header(
    secret: &[u8],
    header: &mut [u8],
    mac: &[u8],
    max_hops: usize,
    additional_data_relayer_len: usize,
    additional_data_last_hop_len: usize,
) -> Result<ForwardedHeader> {
    assert_eq!(SECRET_KEY_LENGTH, secret.len(), "invalid secret length");
    assert_eq!(SimpleMac::SIZE, mac.len(), "invalid mac length");

    let routing_info_len = additional_data_relayer_len + PublicKey::SIZE_COMPRESSED + SimpleMac::SIZE;
    let last_hop_len = additional_data_last_hop_len + 1; // end prefix

    let header_len = last_hop_len + (max_hops - 1) * routing_info_len;

    assert_eq!(header_len, header.len(), "invalid pre-header length");

    let computed_mac = create_tagged_mac(secret, header).unwrap();
    let choice = computed_mac.as_ref().ct_eq(mac).not();
    if choice.into() {
        return Err(TagMismatch);
    }

    // Unmask the header using the keystream
    let prg = PRG::from_parameters(PRGParameters::new(secret));
    let key_stream = prg.digest(0, header_len);
    xor_inplace(header, &key_stream);

    if header[0] != RELAYER_END_PREFIX {
        // Try to deserialize the public key to validate it
        let next_node =
            PublicKey::from_bytes(&header[0..PublicKey::SIZE_COMPRESSED]).map_err(|_| CryptoError::CalculationError)?;

        let mac: Box<[u8]> = (&header[PublicKey::SIZE_COMPRESSED..PublicKey::SIZE_COMPRESSED + SimpleMac::SIZE]).into();

        let additional_info: Box<[u8]> = (&header[PublicKey::SIZE_COMPRESSED + SimpleMac::SIZE
            ..PublicKey::SIZE_COMPRESSED + SimpleMac::SIZE + additional_data_relayer_len])
            .into();

        header.copy_within(routing_info_len.., 0);
        let key_stream = prg.digest(header_len, header_len + routing_info_len);
        header[header_len - routing_info_len..].copy_from_slice(&key_stream);

        Ok(RelayNode {
            header: (&header[..header_len]).into(),
            mac,
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
    use crate::parameters::SECRET_KEY_LENGTH;
    use crate::prg::{PRGParameters, PRG};
    use crate::primitives::{DigestLike, SimpleMac};
    use crate::random::random_bytes;
    use crate::routing::{forward_header, generate_filler, ForwardedHeader, RoutingInfo};
    use crate::shared_keys::SharedKeys;
    use crate::types::PublicKey;
    use crate::utils::xor_inplace;
    use parameterized::parameterized;
    use rand::rngs::OsRng;

    #[parameterized(hops = { 3, 4 })]
    fn test_filler_generate_verify(hops: usize) {
        let per_hop = 3;
        let last_hop = 5;
        let max_hops = hops;

        let secrets = (0..hops)
            .map(|_| random_bytes::<SECRET_KEY_LENGTH>())
            .collect::<Vec<_>>();
        let extended_header_len = per_hop * max_hops + last_hop;
        let header_len = per_hop * (max_hops - 1) + last_hop;

        let mut extended_header = vec![0u8; per_hop * max_hops + last_hop];

        let filler = generate_filler(
            max_hops,
            per_hop,
            last_hop,
            &secrets.iter().map(|s| s.as_ref()).collect::<Vec<_>>(),
        );

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

        let secrets = (0..hops)
            .map(|_| random_bytes::<SECRET_KEY_LENGTH>())
            .collect::<Vec<_>>();

        let first_filler = generate_filler(
            max_hops,
            per_hop,
            last_hop,
            &secrets.iter().map(|s| s.as_ref()).collect::<Vec<_>>(),
        );
        assert_eq!(0, first_filler.len());

        let second_filler = generate_filler(0, per_hop, last_hop, &[]);
        assert_eq!(0, second_filler.len());
    }

    #[parameterized(amount = { 3, 2, 1 })]
    fn test_generate_routing_info_and_forward(amount: usize) {
        const MAX_HOPS: usize = 3;
        let mut additional_data: Vec<&[u8]> = Vec::with_capacity(amount);
        for _ in 0..amount {
            let e: &[u8] = &[];
            additional_data.push(e);
        }

        let pub_keys = (0..amount).into_iter().map(|_| PublicKey::random()).collect::<Vec<_>>();

        let shares = SharedKeys::generate(&mut OsRng, &pub_keys).unwrap();

        let rinfo = RoutingInfo::new(
            MAX_HOPS,
            &pub_keys,
            &shares.secrets().iter().map(|s| s.as_ref()).collect::<Vec<_>>(),
            0,
            &additional_data,
            None,
        );

        let mut header: Vec<u8> = rinfo.routing_information.into();

        let mut last_mac = [0u8; SimpleMac::SIZE];
        last_mac.copy_from_slice(&rinfo.mac);

        for (i, secret) in shares.secrets().iter().enumerate() {
            let fwd = forward_header(secret, &mut header, &last_mac, MAX_HOPS, 0, 0).unwrap();

            match &fwd {
                ForwardedHeader::RelayNode { mac, next_node, .. } => {
                    last_mac.copy_from_slice(&mac);
                    assert!(i < shares.secrets().len() - 1);
                    assert_eq!(&pub_keys[i + 1], next_node);
                }
                ForwardedHeader::FinalNode { additional_data } => {
                    assert_eq!(shares.secrets().len() - 1, i);
                    assert_eq!(0, additional_data.len());
                }
            }
        }
    }
}
