use hopr_crypto_random::random_fill;
use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;
use std::num::NonZeroUsize;

use crate::derivation::derive_mac_key;
use crate::prg::{PRGParameters, PRG};
use crate::shared_keys::SharedSecret;

const RELAYER_END_PREFIX: u8 = 0xff;

const PATH_POSITION_LEN: usize = 1;

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

pub trait SphinxHeaderSpec {
    /// Maximum number of hops.
    const MAX_HOPS: NonZeroUsize;

    /// Size of the public key identifier
    const KEY_ID_SIZE: NonZeroUsize;

    /// Public key identifier type.
    type KeyId: AsRef<[u8]> + for<'a> TryFrom<&'a [u8], Error = GeneralError>;

    /// Size of the additional data for relayers.
    const RELAYER_DATA_SIZE: usize;

    /// Type representing additional data for relayers.
    type RelayerData: AsRef<[u8]> + for<'a> TryFrom<&'a [u8], Error = std::array::TryFromSliceError>;

    /// Size of the additional data for the recipient (last hop).
    const LAST_HOP_DATA_SIZE: usize;

    /// Type representing additional data for the recipient (last hop).
    type LastHopData: AsRef<[u8]> + for<'a> TryFrom<&'a [u8], Error = std::array::TryFromSliceError>;

    /// Length of the header routing information per hop.
    ///
    /// **The value shall not be overridden**.
    const ROUTING_INFO_LEN: usize =
        PATH_POSITION_LEN + Self::KEY_ID_SIZE.get() + SimpleMac::SIZE + Self::RELAYER_DATA_SIZE;

    /// Length of the whole Sphinx header.
    ///
    /// **The value shall not be overridden**.
    const HEADER_LEN: usize = 1 + Self::LAST_HOP_DATA_SIZE + (Self::MAX_HOPS.get() - 1) * Self::ROUTING_INFO_LEN;

    /// Extended header size used for computations.
    ///
    /// **The value shall not be overridden**.
    const EXT_HEADER_LEN: usize = 1 + Self::LAST_HOP_DATA_SIZE + Self::MAX_HOPS.get() * Self::ROUTING_INFO_LEN;

    fn generate_filler(secrets: &[SharedSecret]) -> hopr_crypto_types::errors::Result<Box<[u8]>> {
        if secrets.len() < 2 {
            return Ok(vec![].into_boxed_slice());
        }

        if secrets.len() > Self::MAX_HOPS.into() {
            return Err(CryptoError::InvalidInputValue);
        }

        let padding_len = (Self::MAX_HOPS.get() - secrets.len()) * Self::ROUTING_INFO_LEN;

        let mut ret = vec![0u8; Self::HEADER_LEN - padding_len - Self::LAST_HOP_DATA_SIZE - 1];

        let mut length = Self::ROUTING_INFO_LEN;
        let mut start = Self::HEADER_LEN;

        for secret in secrets.iter().take(secrets.len() - 1) {
            let prg = PRG::from_parameters(PRGParameters::new(secret));

            let digest = prg.digest(start, Self::HEADER_LEN + Self::ROUTING_INFO_LEN);
            xor_inplace(&mut ret[0..length], digest.as_ref());

            length += Self::ROUTING_INFO_LEN;
            start -= Self::ROUTING_INFO_LEN;
        }

        Ok(ret.into_boxed_slice())
    }
}

impl RoutingInfo {
    /// Creates the routing information of the mixnet packet.
    ///
    /// # Arguments
    /// * `path` IDs of the nodes along the path (usually its public key or public key identifier).
    /// * `secrets` shared secrets with the nodes along the path
    /// * `additional_data_relayer` additional data for each relayer
    /// * `additional_data_last_hop` additional data for the final recipient
    pub fn new<H: SphinxHeaderSpec>(
        path: &[H::KeyId],
        secrets: &[SharedSecret],
        additional_data_relayer: &[H::RelayerData],
        additional_data_last_hop: H::LastHopData,
    ) -> hopr_crypto_types::errors::Result<Self> {
        if path.len() != secrets.len() {
            return Err(CryptoError::InvalidParameterSize {
                name: "path",
                expected: secrets.len(),
            });
        }
        if secrets.len() > H::MAX_HOPS.get() || H::MAX_HOPS.get() > RELAYER_END_PREFIX as usize {
            return Err(CryptoError::InvalidInputValue);
        }

        let mut extended_header = vec![0u8; H::EXT_HEADER_LEN];
        let mut ret = RoutingInfo::default();

        for idx in 0..secrets.len() {
            let inverted_idx = secrets.len() - idx - 1;
            let prg = PRG::from_parameters(PRGParameters::new(&secrets[inverted_idx]));

            if idx == 0 {
                // End prefix
                extended_header[0] = RELAYER_END_PREFIX;

                // Last hop additional data
                if H::LAST_HOP_DATA_SIZE > 0 {
                    if additional_data_last_hop.as_ref().len() != H::LAST_HOP_DATA_SIZE {
                        return Err(CryptoError::InvalidParameterSize {
                            name: "additional_data_last_hop",
                            expected: H::LAST_HOP_DATA_SIZE,
                        });
                    }
                    extended_header[1..1 + H::LAST_HOP_DATA_SIZE].copy_from_slice(additional_data_last_hop.as_ref());
                }

                // Random padding for the rest of the extended header
                let padding_len = (H::MAX_HOPS.get() - secrets.len()) * H::ROUTING_INFO_LEN;
                if padding_len > 0 {
                    random_fill(
                        &mut extended_header[1 + H::LAST_HOP_DATA_SIZE..1 + H::LAST_HOP_DATA_SIZE + padding_len],
                    );
                }

                // Encrypt last hop data and padding
                let key_stream = prg.digest(0, 1 + H::LAST_HOP_DATA_SIZE + padding_len);
                xor_inplace(
                    &mut extended_header[0..1 + H::LAST_HOP_DATA_SIZE + padding_len],
                    &key_stream,
                );

                if secrets.len() > 1 {
                    let filler = H::generate_filler(secrets)?;
                    extended_header[1 + H::LAST_HOP_DATA_SIZE + padding_len
                        ..1 + H::LAST_HOP_DATA_SIZE + padding_len + filler.len()]
                        .copy_from_slice(&filler);
                }
            } else {
                // Shift everything to the right to make space for next hop's routing info
                extended_header.copy_within(0..H::HEADER_LEN, H::ROUTING_INFO_LEN);

                // Path position must come first,to ensure prefix RELAYER_END_PREFIX prefix safety
                // of Ed25519 public keys.
                extended_header[0] = idx as u8;

                // Each public key identifier must have an equal length
                let key_ident = path[inverted_idx + 1].as_ref();
                if key_ident.len() != H::KEY_ID_SIZE.into() {
                    return Err(CryptoError::InvalidParameterSize {
                        name: "path[..]",
                        expected: H::KEY_ID_SIZE.into(),
                    });
                }
                // Copy the public key identifier
                extended_header[PATH_POSITION_LEN..PATH_POSITION_LEN + H::KEY_ID_SIZE.get()].copy_from_slice(key_ident);

                // Include the last computed authentication tag
                extended_header[PATH_POSITION_LEN + H::KEY_ID_SIZE.get()
                    ..PATH_POSITION_LEN + H::KEY_ID_SIZE.get() + SimpleMac::SIZE]
                    .copy_from_slice(&ret.mac);

                // The additional relayer data is optional
                if H::RELAYER_DATA_SIZE > 0 {
                    if let Some(relayer_data) = additional_data_relayer.get(inverted_idx).map(|d| d.as_ref()) {
                        if relayer_data.len() != H::RELAYER_DATA_SIZE {
                            return Err(CryptoError::InvalidParameterSize {
                                name: "additional_data_relayer[..]",
                                expected: H::RELAYER_DATA_SIZE,
                            });
                        }

                        extended_header[PATH_POSITION_LEN + H::KEY_ID_SIZE.get() + SimpleMac::SIZE
                            ..PATH_POSITION_LEN + H::KEY_ID_SIZE.get() + SimpleMac::SIZE + H::RELAYER_DATA_SIZE]
                            .copy_from_slice(relayer_data);
                    }
                }

                // Encrypt the entire extended header
                let key_stream = prg.digest(0, H::HEADER_LEN);
                xor_inplace(&mut extended_header, &key_stream);
            }

            let mut m = SimpleMac::new(&derive_mac_key(&secrets[inverted_idx]));
            m.update(&extended_header[0..H::HEADER_LEN]);
            m.finalize_into(&mut ret.mac);
        }

        ret.routing_information = (&extended_header[0..H::HEADER_LEN]).into();
        Ok(ret)
    }
}

/// Enum carry information about the packet based on whether it is destined for the current node (`FinalNode`)
/// or if the packet is supposed to be only relayed (`RelayNode`).
pub enum ForwardedHeader<H: SphinxHeaderSpec> {
    /// The packet is supposed to be relayed
    RelayNode {
        /// Transformed header
        header: Box<[u8]>, // cannot be defined as [u8; H::HEADER_LEN], due to Rust limitation
        /// Authentication tag
        mac: [u8; SimpleMac::SIZE],
        /// Position of the relay in the path
        path_pos: u8,
        /// Public key of the next node
        next_node: H::KeyId,
        /// Additional data for the relayer
        additional_info: H::RelayerData,
    },

    /// The packet is at its final destination
    FinalNode {
        /// Additional data for the final destination
        additional_data: H::LastHopData,
    },
}

/// Applies the forward transformation to the header.
/// If the packet is destined for this node, it returns the additional data
/// for the final destination (`FinalNode`).
/// Otherwise, it returns the transformed header, the
/// next authentication tag, the public key of the next node, and the additional data
/// for the relayer (`RelayNode`).
///
/// # Arguments
/// * `secret` - the shared secret with the creator of the packet
/// * `header` - entire sphinx header to be forwarded
/// * `mac` - current authentication tag of the `header`
pub fn forward_header<H: SphinxHeaderSpec>(
    secret: &SecretKey,
    header: &mut [u8],
    mac: &[u8],
) -> hopr_crypto_types::errors::Result<ForwardedHeader<H>> {
    if mac.len() != SimpleMac::SIZE {
        return Err(CryptoError::InvalidParameterSize {
            name: "mac",
            expected: SimpleMac::SIZE,
        });
    }

    if header.len() != H::HEADER_LEN {
        return Err(CryptoError::InvalidParameterSize {
            name: "header",
            expected: H::HEADER_LEN,
        });
    }

    let mut computed_mac = SimpleMac::new(&derive_mac_key(secret));
    computed_mac.update(header);
    if !mac.eq(computed_mac.finalize().as_slice()) {
        return Err(CryptoError::TagMismatch);
    }

    // Decrypt the header using the keystream
    let prg = PRG::from_parameters(PRGParameters::new(secret));
    let key_stream = prg.digest(0, H::HEADER_LEN);
    xor_inplace(header, &key_stream);

    if header[0] != RELAYER_END_PREFIX {
        // Path position
        let path_pos: u8 = header[0];

        // Try to deserialize the public key to validate it
        let next_node = (&header[PATH_POSITION_LEN..PATH_POSITION_LEN + H::KEY_ID_SIZE.get()])
            .try_into()
            .map_err(|_| CryptoError::InvalidInputValue)?;

        // Authentication tag
        let mac: [u8; SimpleMac::SIZE] = (&header
            [PATH_POSITION_LEN + H::KEY_ID_SIZE.get()..PATH_POSITION_LEN + H::KEY_ID_SIZE.get() + SimpleMac::SIZE])
            .try_into()
            .map_err(|_| CryptoError::CalculationError)?;

        // Optional additional relayer data
        let additional_info = (&header[PATH_POSITION_LEN + H::KEY_ID_SIZE.get() + SimpleMac::SIZE
            ..PATH_POSITION_LEN + H::KEY_ID_SIZE.get() + SimpleMac::SIZE + H::RELAYER_DATA_SIZE])
            .try_into()
            .map_err(|_| CryptoError::InvalidInputValue)?;

        // Shift the entire header left, to discard the data we just read
        header.copy_within(H::ROUTING_INFO_LEN.., 0);

        // Erase the read data from the header
        let key_stream = prg.digest(H::HEADER_LEN, H::HEADER_LEN + H::ROUTING_INFO_LEN);
        header[H::HEADER_LEN - H::ROUTING_INFO_LEN..].copy_from_slice(&key_stream);

        Ok(ForwardedHeader::RelayNode {
            header: (&header[..H::HEADER_LEN]).into(),
            mac,
            path_pos,
            next_node,
            additional_info,
        })
    } else {
        Ok(ForwardedHeader::FinalNode {
            additional_data: (&header[1..1 + H::LAST_HOP_DATA_SIZE])
                .try_into()
                .map_err(|_| CryptoError::InvalidInputValue)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::marker::PhantomData;

    use crate::shared_keys::SphinxSuite;
    use hopr_crypto_types::keypairs::OffchainKeypair;
    use parameterized::parameterized;
    use std::num::NonZero;

    struct TestSpec<K, const HOPS: usize, const RELAYER_DATA: usize, const LAST_HOP_DATA: usize>(PhantomData<K>);
    impl<K, const HOPS: usize, const RELAYER_DATA: usize, const LAST_HOP_DATA: usize> SphinxHeaderSpec
        for TestSpec<K, HOPS, RELAYER_DATA, LAST_HOP_DATA>
    where
        K: AsRef<[u8]> + for<'a> TryFrom<&'a [u8], Error = GeneralError> + BytesRepresentable,
    {
        const MAX_HOPS: NonZeroUsize = NonZero::new(HOPS).unwrap();
        const KEY_ID_SIZE: NonZeroUsize = NonZero::new(K::SIZE).unwrap();
        type KeyId = K;
        const RELAYER_DATA_SIZE: usize = RELAYER_DATA;
        type RelayerData = [u8; RELAYER_DATA];
        const LAST_HOP_DATA_SIZE: usize = LAST_HOP_DATA;
        type LastHopData = [u8; LAST_HOP_DATA];
    }

    #[test]
    fn test_filler_generate_verify() -> anyhow::Result<()> {
        let per_hop = 3 + OffchainPublicKey::SIZE + SimpleMac::SIZE + 1;
        let last_hop = 4;
        let max_hops = 4;

        let secrets = (0..max_hops).map(|_| SharedSecret::random()).collect::<Vec<_>>();
        let extended_header_len = per_hop * max_hops + last_hop + 1;
        let header_len = per_hop * (max_hops - 1) + last_hop + 1;

        let mut extended_header = vec![0u8; extended_header_len];

        let filler = TestSpec::<OffchainPublicKey, 4, 3, 4>::generate_filler(&secrets)?;

        extended_header[1 + last_hop..1 + last_hop + filler.len()].copy_from_slice(&filler);
        extended_header.copy_within(0..header_len, per_hop);

        for i in 0..max_hops - 1 {
            let idx = secrets.len() - i - 2;
            let mask = PRG::from_parameters(PRGParameters::new(&secrets[idx])).digest(0, extended_header_len);

            xor_inplace(&mut extended_header, &mask);

            let mut erased = extended_header.clone();
            erased[header_len..].iter_mut().for_each(|x| *x = 0);
            assert_eq!(erased, extended_header, "xor blinding must erase last bits {i}");

            extended_header.copy_within(0..header_len, per_hop);
        }

        Ok(())
    }

    #[test]
    fn test_filler_edge_case() -> anyhow::Result<()> {
        let hops = 1;

        let secrets = (0..hops).map(|_| SharedSecret::random()).collect::<Vec<_>>();

        let first_filler = TestSpec::<OffchainPublicKey, 1, 0, 0>::generate_filler(&secrets)?;
        assert_eq!(0, first_filler.len());

        Ok(())
    }

    fn generic_test_generate_routing_info_and_forward_no_data<S>(keypairs: Vec<S::P>) -> anyhow::Result<()>
    where
        S: SphinxSuite,
    {
        let pub_keys = keypairs.iter().map(|kp| kp.public().clone()).collect::<Vec<_>>();
        let shares = S::new_shared_keys(&pub_keys)?;

        let rinfo =
            RoutingInfo::new::<TestSpec<<S::P as Keypair>::Public, 3, 0, 0>>(&pub_keys, &shares.secrets, &[], [])?;

        let mut header: Vec<u8> = rinfo.routing_information.into();

        let mut last_mac = [0u8; SimpleMac::SIZE];
        last_mac.copy_from_slice(&rinfo.mac);

        for (i, secret) in shares.secrets.iter().enumerate() {
            let fwd = forward_header::<TestSpec<<S::P as Keypair>::Public, 3, 0, 0>>(secret, &mut header, &last_mac)?;

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
                        pub_keys[i + 1].as_ref(),
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

        Ok(())
    }

    fn generic_test_generate_routing_info_and_forward_with_data<S>(keypairs: Vec<S::P>) -> anyhow::Result<()>
    where
        S: SphinxSuite,
    {
        let pub_keys = keypairs.iter().map(|kp| kp.public().clone()).collect::<Vec<_>>();
        let shares = S::new_shared_keys(&pub_keys)?;

        let relayer_data = (0..keypairs.len()).map(|i| [(i + 1) as u8; 32]).collect::<Vec<_>>();
        let last_hop_data = [0xff_u8; 32];

        let rinfo = RoutingInfo::new::<TestSpec<<S::P as Keypair>::Public, 3, 32, 32>>(
            &pub_keys,
            &shares.secrets,
            &relayer_data,
            last_hop_data,
        )?;

        let mut header: Vec<u8> = rinfo.routing_information.into();

        let mut last_mac = [0u8; SimpleMac::SIZE];
        last_mac.copy_from_slice(&rinfo.mac);

        for (i, secret) in shares.secrets.iter().enumerate() {
            let fwd = forward_header::<TestSpec<<S::P as Keypair>::Public, 3, 32, 32>>(secret, &mut header, &last_mac)?;

            match fwd {
                ForwardedHeader::RelayNode {
                    mac,
                    next_node,
                    path_pos,
                    additional_info,
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
                        pub_keys[i + 1].as_ref(),
                        next_node.as_ref(),
                        "invalid public key of the next node"
                    );
                    assert_eq!(additional_info, relayer_data[i], "invalid additional relayer data");
                }
                ForwardedHeader::FinalNode { additional_data } => {
                    assert_eq!(shares.secrets.len() - 1, i, "cannot be a final node");
                    assert_eq!(
                        last_hop_data, additional_data,
                        "final node must not have any additional data"
                    );
                }
            }
        }

        Ok(())
    }

    #[cfg(feature = "ed25519")]
    #[parameterized(amount = { 3, 2, 1 })]
    fn test_ed25519_generate_routing_info_and_forward(amount: usize) -> anyhow::Result<()> {
        generic_test_generate_routing_info_and_forward_no_data::<crate::ec_groups::Ed25519Suite>(
            (0..amount).map(|_| OffchainKeypair::random()).collect(),
        )?;
        generic_test_generate_routing_info_and_forward_with_data::<crate::ec_groups::Ed25519Suite>(
            (0..amount).map(|_| OffchainKeypair::random()).collect(),
        )
    }

    #[cfg(feature = "x25519")]
    #[parameterized(amount = { 3, 2, 1 })]
    fn test_x25519_generate_routing_info_and_forward(amount: usize) -> anyhow::Result<()> {
        generic_test_generate_routing_info_and_forward_no_data::<crate::ec_groups::X25519Suite>(
            (0..amount).map(|_| OffchainKeypair::random()).collect(),
        )?;
        generic_test_generate_routing_info_and_forward_with_data::<crate::ec_groups::X25519Suite>(
            (0..amount).map(|_| OffchainKeypair::random()).collect(),
        )
    }

    #[cfg(feature = "secp256k1")]
    #[parameterized(amount = { 3, 2, 1 })]
    fn test_secp256k1_generate_routing_info_and_forward(amount: usize) -> anyhow::Result<()> {
        generic_test_generate_routing_info_and_forward_no_data::<crate::ec_groups::Secp256k1Suite>(
            (0..amount).map(|_| ChainKeypair::random()).collect(),
        )?;
        generic_test_generate_routing_info_and_forward_with_data::<crate::ec_groups::Secp256k1Suite>(
            (0..amount).map(|_| ChainKeypair::random()).collect(),
        )
    }
}
