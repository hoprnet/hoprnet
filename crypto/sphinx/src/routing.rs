use std::{
    fmt::{Debug, Formatter},
    marker::PhantomData,
    num::NonZeroUsize,
};

use hopr_crypto_random::random_fill;
use hopr_crypto_types::{
    crypto_traits::{StreamCipher, StreamCipherSeek, UniversalHash},
    prelude::*,
    types::Pseudonym,
};
use hopr_primitive_types::prelude::*;
use typenum::Unsigned;

use crate::{
    derivation::{generate_key, generate_key_iv},
    shared_keys::SharedSecret,
};

/// Current version of the header
const SPHINX_HEADER_VERSION: u8 = 1;

const HASH_KEY_PRG: &str = "HASH_KEY_PRG";

const HASH_KEY_TAG: &str = "HASH_KEY_TAG";

/// Contains the necessary size and type specifications for the Sphinx packet header.
pub trait SphinxHeaderSpec {
    /// Maximum number of hops.
    const MAX_HOPS: NonZeroUsize;

    /// Public key identifier type.
    type KeyId: BytesRepresentable + Clone;

    /// Pseudonym used to represent node for SURBs.
    type Pseudonym: Pseudonym;

    /// Type representing additional data for relayers.
    type RelayerData: BytesRepresentable;

    /// Type representing additional data delivered to the packet receiver.
    ///
    /// It is delivered on both forward and return paths.
    type PacketReceiverData: BytesRepresentable;

    /// Type representing additional data delivered with each SURB to the packet receiver.
    ///
    /// It is delivered only on the forward path.
    type SurbReceiverData: BytesRepresentable;

    /// Pseudo-Random Generator function used to encrypt and decrypt the Sphinx header.
    type PRG: crypto_traits::StreamCipher + crypto_traits::StreamCipherSeek + crypto_traits::KeyIvInit;

    /// One-time authenticator used for Sphinx header tag.
    type UH: crypto_traits::UniversalHash + crypto_traits::KeyInit;

    /// Size of the additional data for relayers.
    const RELAYER_DATA_SIZE: usize = Self::RelayerData::SIZE;

    /// Size of the additional data included in SURBs.
    const SURB_RECEIVER_DATA_SIZE: usize = Self::SurbReceiverData::SIZE;

    /// Size of the additional data for the packet receiver.
    const RECEIVER_DATA_SIZE: usize = Self::PacketReceiverData::SIZE;

    /// Size of the public key identifier
    const KEY_ID_SIZE: NonZeroUsize = NonZeroUsize::new(Self::KeyId::SIZE).unwrap();

    /// Size of the one-time authenticator tag.
    const TAG_SIZE: usize = <Self::UH as crypto_traits::BlockSizeUser>::BlockSize::USIZE;

    /// Length of the header routing information per hop.
    ///
    /// **The value shall not be overridden**.
    const ROUTING_INFO_LEN: usize =
        HeaderPrefix::SIZE + Self::KEY_ID_SIZE.get() + Self::TAG_SIZE + Self::RELAYER_DATA_SIZE;

    /// Length of the whole Sphinx header.
    ///
    /// **The value shall not be overridden**.
    const HEADER_LEN: usize =
        HeaderPrefix::SIZE + Self::RECEIVER_DATA_SIZE + (Self::MAX_HOPS.get() - 1) * Self::ROUTING_INFO_LEN;

    /// Extended header size used for computations.
    ///
    /// **The value shall not be overridden**.
    const EXT_HEADER_LEN: usize =
        HeaderPrefix::SIZE + Self::RECEIVER_DATA_SIZE + Self::MAX_HOPS.get() * Self::ROUTING_INFO_LEN;

    fn generate_filler(secrets: &[SharedSecret]) -> hopr_crypto_types::errors::Result<Box<[u8]>> {
        if secrets.len() < 2 {
            return Ok(vec![].into_boxed_slice());
        }

        if secrets.len() > Self::MAX_HOPS.into() {
            return Err(CryptoError::InvalidInputValue("secrets.len"));
        }

        let padding_len = (Self::MAX_HOPS.get() - secrets.len()) * Self::ROUTING_INFO_LEN;

        let mut ret = vec![0u8; Self::HEADER_LEN - padding_len - Self::Pseudonym::SIZE - 1];
        let mut length = Self::ROUTING_INFO_LEN;
        let mut start = Self::HEADER_LEN;

        for secret in secrets.iter().take(secrets.len() - 1) {
            let mut prg = Self::new_prg(secret)?;
            prg.seek(start);
            prg.apply_keystream(&mut ret[0..length]);

            length += Self::ROUTING_INFO_LEN;
            start -= Self::ROUTING_INFO_LEN;
        }

        Ok(ret.into_boxed_slice())
    }

    /// Instantiates a new Pseudo-Random Generator.
    fn new_prg(secret: &SecretKey) -> hopr_crypto_types::errors::Result<Self::PRG> {
        generate_key_iv(secret, HASH_KEY_PRG, None)
    }
}

/// Sphinx header byte prefix
///
/// ### Layout (MSB first)
/// `Version (3 bits), No Ack flag (1 bit), Reply flag (1 bit), Path position (3 bits)`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HeaderPrefix(u8);

impl HeaderPrefix {
    pub const SIZE: usize = 1;

    pub fn new(is_reply: bool, no_ack: bool, path_pos: u8) -> Result<Self, GeneralError> {
        // Due to size restriction, we do not allow greater than 7 hop paths.
        if path_pos > 7 {
            return Err(GeneralError::ParseError("HeaderPrefixByte".into()));
        }

        let mut out = 0;
        out |= (SPHINX_HEADER_VERSION & 0x07) << 5;
        out |= (no_ack as u8) << 4;
        out |= (is_reply as u8) << 3;
        out |= path_pos & 0x07;
        Ok(Self(out))
    }

    #[inline]
    pub fn is_reply(&self) -> bool {
        (self.0 & 0x08) != 0
    }

    #[inline]
    pub fn is_no_ack(&self) -> bool {
        (self.0 & 0x10) != 0
    }

    #[inline]
    pub fn path_position(&self) -> u8 {
        self.0 & 0x07
    }

    #[inline]
    pub fn is_final_hop(&self) -> bool {
        self.path_position() == 0
    }
}

impl From<HeaderPrefix> for u8 {
    fn from(value: HeaderPrefix) -> Self {
        value.0
    }
}

impl TryFrom<u8> for HeaderPrefix {
    type Error = GeneralError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if (value & 0xe0) >> 5 == SPHINX_HEADER_VERSION {
            Ok(Self(value))
        } else {
            Err(GeneralError::ParseError("invalid header version".into()))
        }
    }
}

/// Carries routing information for the mixnet packet.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RoutingInfo<H: SphinxHeaderSpec>(Box<[u8]>, PhantomData<H>);

impl<H: SphinxHeaderSpec> Debug for RoutingInfo<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl<H: SphinxHeaderSpec> Clone for RoutingInfo<H> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<H: SphinxHeaderSpec> PartialEq for RoutingInfo<H> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<H: SphinxHeaderSpec> Eq for RoutingInfo<H> {}

impl<H: SphinxHeaderSpec> Default for RoutingInfo<H> {
    fn default() -> Self {
        Self(vec![0u8; Self::SIZE].into_boxed_slice(), PhantomData)
    }
}

impl<H: SphinxHeaderSpec> AsRef<[u8]> for RoutingInfo<H> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<'a, H: SphinxHeaderSpec> TryFrom<&'a [u8]> for RoutingInfo<H> {
    type Error = GeneralError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.len() == Self::SIZE {
            Ok(Self(value.into(), PhantomData))
        } else {
            Err(GeneralError::ParseError("RoutingInfo".into()))
        }
    }
}

impl<H: SphinxHeaderSpec> BytesRepresentable for RoutingInfo<H> {
    const SIZE: usize = H::HEADER_LEN + H::TAG_SIZE;
}

impl<H: SphinxHeaderSpec> RoutingInfo<H> {
    /// Creates the routing information of the mixnet packet.
    ///
    /// # Arguments
    /// * `path` IDs of the nodes along the path (usually its public key or public key identifier).
    /// * `secrets` shared secrets with the nodes along the path
    /// * `additional_data_relayer` additional data for each relayer
    /// * `receiver_data` data for the packet receiver (this usually contains also `H::Pseudonym`).
    /// * `is_reply` flag indicating whether this is a reply packet
    /// * `no_ack` special flag used for acknowledgement signaling to the recipient
    pub fn new(
        path: &[H::KeyId],
        secrets: &[SharedSecret],
        additional_data_relayer: &[H::RelayerData],
        receiver_data: &H::PacketReceiverData,
        is_reply: bool,
        no_ack: bool,
    ) -> hopr_crypto_types::errors::Result<Self> {
        assert!(H::MAX_HOPS.get() <= 7, "maximum number of hops supported is 7");

        if path.len() != secrets.len() {
            return Err(CryptoError::InvalidParameterSize {
                name: "path",
                expected: secrets.len(),
            });
        }

        if secrets.len() > H::MAX_HOPS.get() {
            return Err(CryptoError::InvalidInputValue("secrets.len"));
        }

        let mut extended_header = vec![0u8; H::EXT_HEADER_LEN];
        let mut ret = RoutingInfo::default();

        for idx in 0..secrets.len() {
            let inverted_idx = secrets.len() - idx - 1;
            let prefix = HeaderPrefix::new(is_reply, no_ack, idx as u8)?;

            let mut prg = H::new_prg(&secrets[inverted_idx])?;

            if idx == 0 {
                // Prefix byte
                extended_header[0] = prefix.into();

                // Last hop additional data
                extended_header[HeaderPrefix::SIZE..HeaderPrefix::SIZE + H::PacketReceiverData::SIZE]
                    .copy_from_slice(receiver_data.as_ref());

                // Random padding for the rest of the extended header
                let padding_len = (H::MAX_HOPS.get() - secrets.len()) * H::ROUTING_INFO_LEN;
                if padding_len > 0 {
                    random_fill(
                        &mut extended_header[HeaderPrefix::SIZE + H::PacketReceiverData::SIZE
                            ..HeaderPrefix::SIZE + H::PacketReceiverData::SIZE + padding_len],
                    );
                }

                // Encrypt last hop data and padding
                prg.apply_keystream(
                    &mut extended_header[0..HeaderPrefix::SIZE + H::PacketReceiverData::SIZE + padding_len],
                );

                if secrets.len() > 1 {
                    let filler = H::generate_filler(secrets)?;
                    extended_header[HeaderPrefix::SIZE + H::PacketReceiverData::SIZE + padding_len
                        ..HeaderPrefix::SIZE + H::PacketReceiverData::SIZE + padding_len + filler.len()]
                        .copy_from_slice(&filler);
                }
            } else {
                // Shift everything to the right to make space for the next hop's routing info
                extended_header.copy_within(0..H::HEADER_LEN, H::ROUTING_INFO_LEN);

                // Prefix byte must come first to ensure prefix RELAYER_END_PREFIX prefix safety
                // of Ed25519 public keys.
                extended_header[0] = prefix.into();

                // Each public key identifier must have an equal length
                let key_ident = path[inverted_idx + 1].as_ref();
                if key_ident.len() != H::KEY_ID_SIZE.get() {
                    return Err(CryptoError::InvalidParameterSize {
                        name: "path[..]",
                        expected: H::KEY_ID_SIZE.into(),
                    });
                }
                // Copy the public key identifier
                extended_header[HeaderPrefix::SIZE..HeaderPrefix::SIZE + H::KEY_ID_SIZE.get()]
                    .copy_from_slice(key_ident);

                // Include the last computed authentication tag
                extended_header[HeaderPrefix::SIZE + H::KEY_ID_SIZE.get()
                    ..HeaderPrefix::SIZE + H::KEY_ID_SIZE.get() + H::TAG_SIZE]
                    .copy_from_slice(ret.mac());

                // The additional relayer data is optional
                if H::RELAYER_DATA_SIZE > 0 {
                    if let Some(relayer_data) = additional_data_relayer.get(inverted_idx).map(|d| d.as_ref()) {
                        if relayer_data.len() != H::RELAYER_DATA_SIZE {
                            return Err(CryptoError::InvalidParameterSize {
                                name: "additional_data_relayer[..]",
                                expected: H::RELAYER_DATA_SIZE,
                            });
                        }

                        extended_header[HeaderPrefix::SIZE + H::KEY_ID_SIZE.get() + H::TAG_SIZE
                            ..HeaderPrefix::SIZE + H::KEY_ID_SIZE.get() + H::TAG_SIZE + H::RELAYER_DATA_SIZE]
                            .copy_from_slice(relayer_data);
                    }
                }

                // Encrypt the entire extended header
                prg.apply_keystream(&mut extended_header[0..H::HEADER_LEN]);
            }

            let mut uh: H::UH = generate_key(&secrets[inverted_idx], HASH_KEY_TAG, None)
                .map_err(|_| CryptoError::InvalidInputValue("mac_key"))?;
            uh.update_padded(&extended_header[0..H::HEADER_LEN]);
            ret.mac_mut().copy_from_slice(&uh.finalize());
        }

        ret.routing_mut().copy_from_slice(&extended_header[0..H::HEADER_LEN]);
        Ok(ret)
    }

    fn mac(&self) -> &[u8] {
        &self.0[H::HEADER_LEN..H::HEADER_LEN + H::TAG_SIZE]
    }

    fn routing_mut(&mut self) -> &mut [u8] {
        &mut self.0[0..H::HEADER_LEN]
    }

    fn mac_mut(&mut self) -> &mut [u8] {
        &mut self.0[H::HEADER_LEN..H::HEADER_LEN + H::TAG_SIZE]
    }
}

/// Enum carry information about the packet based on whether it is destined for the current node (`FinalNode`)
/// or if the packet is supposed to be only relayed (`RelayNode`).
pub enum ForwardedHeader<H: SphinxHeaderSpec> {
    /// The packet is supposed to be relayed
    Relayed {
        /// Transformed header
        next_header: RoutingInfo<H>,
        /// Position of the relay in the path
        path_pos: u8,
        /// Public key of the next node
        next_node: H::KeyId,
        /// Additional data for the relayer
        additional_info: H::RelayerData,
    },

    /// The packet is at its final destination
    Final {
        /// Data from the sender to the packet receiver.
        /// This usually contains also `H::Pseudonym`.
        receiver_data: H::PacketReceiverData,
        /// Indicates whether this message is a reply and a [`ReplyOpener`](crate::surb::ReplyOpener)
        /// should be used to further decrypt the message.
        is_reply: bool,
        /// Special flag used for acknowledgement signaling.
        no_ack: bool,
    },
}

/// Applies the forward transformation to the header.
/// If the packet is destined for this node, it returns the additional data
/// for the final destination ([`ForwardedHeader::Final`]).
/// Otherwise, it returns the transformed header, the
/// next authentication tag, the public key of the next node, and the additional data
/// for the relayer ([`ForwardedHeader::Relayed`]).
///
/// # Arguments
/// * `secret` - the shared secret with the creator of the packet
/// * `header` - entire sphinx header to be forwarded
pub fn forward_header<H: SphinxHeaderSpec>(
    secret: &SecretKey,
    header: &mut [u8],
) -> hopr_crypto_types::errors::Result<ForwardedHeader<H>> {
    if header.len() != RoutingInfo::<H>::SIZE {
        return Err(CryptoError::InvalidParameterSize {
            name: "header",
            expected: H::HEADER_LEN,
        });
    }

    // Compute and verify the authentication tag
    let mut uh: H::UH =
        generate_key(secret, HASH_KEY_TAG, None).map_err(|_| CryptoError::InvalidInputValue("mac_key"))?;
    uh.update_padded(&header[0..H::HEADER_LEN]);
    uh.verify(header[H::HEADER_LEN..H::HEADER_LEN + H::TAG_SIZE].into())
        .map_err(|_| CryptoError::TagMismatch)?;

    // Decrypt the header using the key=stream
    let mut prg = H::new_prg(secret)?;
    prg.apply_keystream(&mut header[0..H::HEADER_LEN]);

    let prefix = HeaderPrefix::try_from(header[0])?;

    if !prefix.is_final_hop() {
        // Try to deserialize the public key to validate it
        let next_node = (&header[HeaderPrefix::SIZE..HeaderPrefix::SIZE + H::KEY_ID_SIZE.get()])
            .try_into()
            .map_err(|_| CryptoError::InvalidInputValue("next_node"))?;

        let mut next_header = RoutingInfo::<H>::default();

        // Authentication tag
        next_header.mac_mut().copy_from_slice(
            &header[HeaderPrefix::SIZE + H::KEY_ID_SIZE.get()..HeaderPrefix::SIZE + H::KEY_ID_SIZE.get() + H::TAG_SIZE],
        );

        // Optional additional relayer data
        let additional_info = (&header[HeaderPrefix::SIZE + H::KEY_ID_SIZE.get() + H::TAG_SIZE
            ..HeaderPrefix::SIZE + H::KEY_ID_SIZE.get() + H::TAG_SIZE + H::RELAYER_DATA_SIZE])
            .try_into()
            .map_err(|_| CryptoError::InvalidInputValue("additional_relayer_data"))?;

        // Shift the entire header to the left to discard the data we just read
        header.copy_within(H::ROUTING_INFO_LEN..H::HEADER_LEN, 0);

        // Erase the read data from the header to apply the raw key-stream
        header[H::HEADER_LEN - H::ROUTING_INFO_LEN..H::HEADER_LEN].fill(0);
        prg.seek(H::HEADER_LEN);
        prg.apply_keystream(&mut header[H::HEADER_LEN - H::ROUTING_INFO_LEN..H::HEADER_LEN]);

        next_header.routing_mut().copy_from_slice(&header[0..H::HEADER_LEN]);

        Ok(ForwardedHeader::Relayed {
            next_header,
            path_pos: prefix.path_position(),
            next_node,
            additional_info,
        })
    } else {
        Ok(ForwardedHeader::Final {
            receiver_data: (&header[HeaderPrefix::SIZE..HeaderPrefix::SIZE + H::PacketReceiverData::SIZE])
                .try_into()
                .map_err(|_| CryptoError::InvalidInputValue("receiver_data"))?,
            is_reply: prefix.is_reply(),
            no_ack: prefix.is_no_ack(),
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use hopr_crypto_random::Randomizable;
    use hopr_crypto_types::{crypto_traits::BlockSizeUser, keypairs::OffchainKeypair};
    use parameterized::parameterized;

    use super::*;
    use crate::{
        shared_keys::{Alpha, GroupElement, SphinxSuite},
        tests::*,
    };

    #[test]
    fn test_filler_generate_verify() -> anyhow::Result<()> {
        let per_hop = 3 + OffchainPublicKey::SIZE + <Poly1305 as BlockSizeUser>::BlockSize::USIZE + 1;
        let last_hop = SimplePseudonym::SIZE;
        let max_hops = 4;

        let secrets = (0..max_hops).map(|_| SharedSecret::random()).collect::<Vec<_>>();
        let extended_header_len = per_hop * max_hops + last_hop + 1;
        let header_len = per_hop * (max_hops - 1) + last_hop + 1;

        let mut extended_header = vec![0u8; extended_header_len];

        let filler = TestSpec::<OffchainPublicKey, 4, 3>::generate_filler(&secrets)?;

        extended_header[1 + last_hop..1 + last_hop + filler.len()].copy_from_slice(&filler);
        extended_header.copy_within(0..header_len, per_hop);

        for i in 0..max_hops - 1 {
            let idx = secrets.len() - i - 2;

            let mut prg = generate_key_iv::<ChaCha20, _>(&secrets[idx], HASH_KEY_PRG, None)?;
            prg.apply_keystream(&mut extended_header);

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

        let first_filler = TestSpec::<OffchainPublicKey, 1, 0>::generate_filler(&secrets)?;
        assert_eq!(0, first_filler.len());

        Ok(())
    }

    fn generic_test_generate_routing_info_and_forward<S>(keypairs: Vec<S::P>, reply: bool) -> anyhow::Result<()>
    where
        S: SphinxSuite,
        for<'a> &'a Alpha<<S::G as GroupElement<S::E>>::AlphaLen>: From<&'a <S::P as Keypair>::Public>,
    {
        let pub_keys = keypairs.iter().map(|kp| kp.public().clone()).collect::<Vec<_>>();
        let shares = S::new_shared_keys(&pub_keys)?;
        let pseudonym = SimplePseudonym::random();
        let no_ack_flag = true;

        let mut rinfo = RoutingInfo::<TestSpec<<S::P as Keypair>::Public, 3, 0>>::new(
            &pub_keys,
            &shares.secrets,
            &[],
            &pseudonym,
            reply,
            no_ack_flag,
        )?;

        for (i, secret) in shares.secrets.iter().enumerate() {
            let fwd = forward_header::<TestSpec<<S::P as Keypair>::Public, 3, 0>>(secret, &mut rinfo.0)?;

            match fwd {
                ForwardedHeader::Relayed {
                    next_header,
                    next_node,
                    path_pos,
                    ..
                } => {
                    rinfo = next_header;
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
                ForwardedHeader::Final {
                    receiver_data,
                    is_reply,
                    no_ack,
                } => {
                    assert_eq!(shares.secrets.len() - 1, i, "cannot be a final node");
                    assert_eq!(pseudonym, receiver_data, "invalid pseudonym");
                    assert_eq!(is_reply, reply, "invalid reply flag");
                    assert_eq!(no_ack, no_ack_flag, "invalid no_ack flag");
                }
            }
        }

        Ok(())
    }

    #[cfg(feature = "ed25519")]
    #[parameterized(amount = { 3, 2, 1, 3, 2, 1 }, reply = { true, true, true, false, false, false })]
    fn test_ed25519_generate_routing_info_and_forward(amount: usize, reply: bool) -> anyhow::Result<()> {
        generic_test_generate_routing_info_and_forward::<crate::ec_groups::Ed25519Suite>(
            (0..amount).map(|_| OffchainKeypair::random()).collect(),
            reply,
        )
    }

    #[cfg(feature = "x25519")]
    #[parameterized(amount = { 3, 2, 1, 3, 2, 1 }, reply = { true, true, true, false, false, false })]
    fn test_x25519_generate_routing_info_and_forward(amount: usize, reply: bool) -> anyhow::Result<()> {
        generic_test_generate_routing_info_and_forward::<crate::ec_groups::X25519Suite>(
            (0..amount).map(|_| OffchainKeypair::random()).collect(),
            reply,
        )
    }

    #[cfg(feature = "secp256k1")]
    #[parameterized(amount = { 3, 2, 1, 3, 2, 1 }, reply = { true, true, true, false, false, false })]
    fn test_secp256k1_generate_routing_info_and_forward(amount: usize, reply: bool) -> anyhow::Result<()> {
        generic_test_generate_routing_info_and_forward::<crate::ec_groups::Secp256k1Suite>(
            (0..amount).map(|_| ChainKeypair::random()).collect(),
            reply,
        )
    }
}
