use std::marker::PhantomData;

use hopr_crypto_sphinx::{
    errors::SphinxError,
    prelude::{PaddedPayload, SURB, SphinxHeaderSpec, SphinxSuite},
};
use hopr_crypto_types::prelude::Hash;
use hopr_internal_types::prelude::HoprPseudonym;
use hopr_primitive_types::prelude::{BytesRepresentable, GeneralError};

use crate::{HoprSphinxHeaderSpec, HoprSphinxSuite, PAYLOAD_SIZE_INT};

pub const SURB_ID_SIZE: usize = 8;

pub type HoprSurbId = [u8; SURB_ID_SIZE];

/// Identifier of the packet sender.
///
/// This consists of two parts:
/// - [`HoprSenderId::pseudonym`] of the sender
/// - [`HoprSenderId::surb_id`] is an identifier a single SURB that routes the packet back to the sender.
///
/// The `surb_id` always identifies a single SURB. The instance can be turned into a pseudorandom
/// sequence using [`HoprSenderId::into_sequence`] to create identifiers for more SURBs
/// with the same pseudonym.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HoprSenderId(#[cfg_attr(feature = "serde", serde(with = "serde_bytes"))] [u8; Self::SIZE]);

impl HoprSenderId {
    pub fn new(pseudonym: &HoprPseudonym) -> Self {
        let mut ret: [u8; Self::SIZE] = hopr_crypto_random::random_bytes();
        ret[..HoprPseudonym::SIZE].copy_from_slice(pseudonym.as_ref());
        Self(ret)
    }

    pub fn from_pseudonym_and_id(pseudonym: &HoprPseudonym, id: HoprSurbId) -> Self {
        let mut ret = [0u8; Self::SIZE];
        ret[..HoprPseudonym::SIZE].copy_from_slice(pseudonym.as_ref());
        ret[HoprPseudonym::SIZE..HoprPseudonym::SIZE + SURB_ID_SIZE].copy_from_slice(&id);
        Self(ret)
    }

    pub fn pseudonym(&self) -> HoprPseudonym {
        HoprPseudonym::try_from(&self.0[..HoprPseudonym::SIZE]).expect("must have valid pseudonym")
    }

    pub fn surb_id(&self) -> HoprSurbId {
        self.0[HoprPseudonym::SIZE..HoprPseudonym::SIZE + SURB_ID_SIZE]
            .try_into()
            .expect("must have valid nonce")
    }

    pub fn into_sequence(self) -> impl Iterator<Item = Self> {
        std::iter::successors(Some((1u32, self)), |&(i, prev)| {
            let hash = Hash::create(&[&i.to_be_bytes(), prev.as_ref()]);
            Some((
                i + 1,
                Self::from_pseudonym_and_id(&prev.pseudonym(), hash.as_ref()[0..SURB_ID_SIZE].try_into().unwrap()),
            ))
        })
        .map(|(_, v)| v)
    }
}

impl AsRef<[u8]> for HoprSenderId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<'a> TryFrom<&'a [u8]> for HoprSenderId {
    type Error = GeneralError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        value
            .try_into()
            .map(Self)
            .map_err(|_| GeneralError::ParseError("HoprPacketReceiverData.size".into()))
    }
}

impl BytesRepresentable for HoprSenderId {
    const SIZE: usize = HoprPseudonym::SIZE + SURB_ID_SIZE;
}

impl hopr_crypto_random::Randomizable for HoprSenderId {
    fn random() -> Self {
        Self::new(&HoprPseudonym::random())
    }
}

/// Additional encoding of a packet message that can be preceded by a number of [`SURBs`](SURB).
pub struct PacketMessage<S: SphinxSuite, H: SphinxHeaderSpec, const P: usize>(PaddedPayload<P>, PhantomData<(S, H)>);

/// Convenience alias for HOPR specific [`PacketMessage`].
pub type HoprPacketMessage = PacketMessage<HoprSphinxSuite, HoprSphinxHeaderSpec, PAYLOAD_SIZE_INT>;

/// Individual parts of a [`PacketMessage`]: SURBs and the actual message.
pub struct PacketParts<S: SphinxSuite, H: SphinxHeaderSpec> {
    pub surbs: Vec<SURB<S, H>>,
    pub payload: Box<[u8]>,
    pub flags: u8,
}

impl<S: SphinxSuite, H: SphinxHeaderSpec, const P: usize> PacketMessage<S, H, P> {
    /// Mask for the flags in the first byte.
    const F_MASK: u8 = Self::N_MASK.wrapping_neg();
    /// Size of the message header.
    ///
    /// This is currently 1 byte to indicate the number of SURBs that precede the message.
    pub const HEADER_LEN: usize = 1;
    /// The number of SURBs in a `PacketMessage` is intentionally limited so that
    /// the upper 4 bits remain reserved for additional flags.
    pub const MAX_SURBS_PER_MESSAGE: usize = 31;
    /// Mask for the number of SURBs in the first byte.
    const N_MASK: u8 = Self::MAX_SURBS_PER_MESSAGE as u8;

    /// Converts this instance into [`PacketParts`].
    pub fn try_into_parts(self) -> Result<PacketParts<S, H>, SphinxError> {
        let data = self.0.into_unpadded()?;
        let num_surbs = (data[0] & Self::N_MASK) as usize;
        let flags = (data[0] & Self::F_MASK) >> Self::N_MASK.trailing_ones();

        if num_surbs > 0 {
            let surb_end = num_surbs * SURB::<S, H>::SIZE;
            if surb_end >= data.len() {
                return Err(GeneralError::ParseError("HoprPacketMessage.num_surbs not valid".into()).into());
            }

            let mut data = data.into_vec();

            let surb_data = data.drain(0..=surb_end).skip(1).collect::<Vec<_>>();

            let surbs = surb_data
                .as_slice()
                .chunks_exact(SURB::<S, H>::SIZE)
                .map(SURB::<S, H>::try_from)
                .collect::<Result<Vec<_>, _>>()?;

            Ok(PacketParts {
                surbs,
                payload: data.into_boxed_slice(),
                flags,
            })
        } else {
            let mut data = data.into_vec();
            data.remove(0);
            Ok(PacketParts {
                surbs: Vec::with_capacity(0),
                payload: data.into_boxed_slice(),
                flags,
            })
        }
    }

    /// Allocates a new instance from the given parts.
    pub fn from_parts(surbs: Vec<SURB<S, H>>, payload: &[u8], flags: u8) -> Result<Self, SphinxError> {
        if surbs.len() > Self::MAX_SURBS_PER_MESSAGE {
            return Err(GeneralError::ParseError("HoprPacketMessage.num_surbs not valid".into()).into());
        }

        if flags > Self::F_MASK >> Self::N_MASK.trailing_ones() {
            return Err(GeneralError::ParseError("HoprPacketMessage.flags not valid".into()).into());
        }

        // The total size of the packet message must not exceed the maximum packet size.
        if Self::HEADER_LEN + surbs.len() * SURB::<S, H>::SIZE + payload.len() > P {
            return Err(GeneralError::ParseError("HoprPacketMessage.size not valid".into()).into());
        }

        let mut ret = Vec::with_capacity(PaddedPayload::<P>::SIZE);
        let flags_and_len = ((flags & (Self::F_MASK >> Self::N_MASK.trailing_ones())) << Self::N_MASK.trailing_ones())
            | (surbs.len() as u8 & Self::N_MASK);
        ret.push(flags_and_len);
        for surb in surbs.into_iter().map(|s| s.into_boxed()) {
            ret.extend_from_slice(surb.as_ref());
        }
        ret.extend_from_slice(payload);

        // Save one reallocation by using the vector that we just created
        Ok(Self(PaddedPayload::new_from_vec(ret)?, PhantomData))
    }
}

impl<S: SphinxSuite, H: SphinxHeaderSpec, const P: usize> From<PaddedPayload<P>> for PacketMessage<S, H, P> {
    fn from(value: PaddedPayload<P>) -> Self {
        Self(value, PhantomData)
    }
}

impl<S: SphinxSuite, H: SphinxHeaderSpec, const P: usize> From<PacketMessage<S, H, P>> for PaddedPayload<P> {
    fn from(value: PacketMessage<S, H, P>) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use bimap::BiHashMap;
    use hex_literal::hex;
    use hopr_crypto_random::Randomizable;
    use hopr_crypto_sphinx::prelude::*;
    use hopr_crypto_types::prelude::*;
    use hopr_primitive_types::prelude::*;

    use super::*;
    use crate::{
        HoprSphinxHeaderSpec, HoprSphinxSuite, HoprSurb,
        packet::HoprPacket,
        por::{SurbReceiverInfo, generate_proof_of_relay},
    };

    lazy_static::lazy_static! {
        static ref PEERS: [(ChainKeypair, OffchainKeypair); 4] = [
            (hex!("a7c486ceccf5ab53bd428888ab1543dc2667abd2d5e80aae918da8d4b503a426"), hex!("5eb212d4d6aa5948c4f71574d45dad43afef6d330edb873fca69d0e1b197e906")),
            (hex!("9a82976f7182c05126313bead5617c623b93d11f9f9691c87b1a26f869d569ed"), hex!("e995db483ada5174666c46bafbf3628005aca449c94ebdc0c9239c3f65d61ae0")),
            (hex!("ca4bdfd54a8467b5283a0216288fdca7091122479ccf3cfb147dfa59d13f3486"), hex!("9dec751c00f49e50fceff7114823f726a0425a68a8dc6af0e4287badfea8f4a4")),
            (hex!("e306ebfb0d01d0da0952c9a567d758093a80622c6cb55052bf5f1a6ebd8d7b5c"), hex!("9a82976f7182c05126313bead5617c623b93d11f9f9691c87b1a26f869d569ed"))
        ].map(|(p1,p2)| (ChainKeypair::from_secret(&p1).expect("lazy static keypair should be valid"), OffchainKeypair::from_secret(&p2).expect("lazy static keypair should be valid")));

        static ref MAPPER: bimap::BiMap<KeyIdent, OffchainPublicKey> = PEERS
            .iter()
            .enumerate()
            .map(|(i, (_, k))| (KeyIdent::from(i as u32), *k.public()))
            .collect::<BiHashMap<_, _>>();
    }

    fn generate_surbs(count: usize) -> anyhow::Result<Vec<SURB<HoprSphinxSuite, HoprSphinxHeaderSpec>>> {
        let path = PEERS.iter().map(|(_, k)| *k.public()).collect::<Vec<_>>();
        let path_ids =
            <BiHashMap<_, _> as KeyIdMapper<HoprSphinxSuite, HoprSphinxHeaderSpec>>::map_keys_to_ids(&*MAPPER, &path)
                .into_iter()
                .map(|v| v.ok_or(anyhow!("missing id")))
                .collect::<Result<Vec<_>, _>>()?;
        let pseudonym = SimplePseudonym::random();
        let recv_data = HoprSenderId::new(&pseudonym);

        Ok((0..count)
            .map(|_| {
                let shared_keys = HoprSphinxSuite::new_shared_keys(&path)?;
                let (por_strings, por_values) = generate_proof_of_relay(&shared_keys.secrets)
                    .map_err(|e| CryptoError::Other(GeneralError::NonSpecificError(e.to_string())))?;

                create_surb::<HoprSphinxSuite, HoprSphinxHeaderSpec>(
                    shared_keys,
                    &path_ids,
                    &por_strings,
                    recv_data,
                    SurbReceiverInfo::new(por_values, [0u8; 32]),
                )
                .map(|(s, _)| s)
            })
            .collect::<Result<Vec<_>, _>>()?)
    }

    #[test]
    fn hopr_packet_message_message_only() -> anyhow::Result<()> {
        let test_msg = b"test message";
        let PacketParts { surbs, payload, flags } =
            HoprPacketMessage::from_parts(vec![], test_msg, 7)?.try_into_parts()?;
        assert_eq!(surbs.len(), 0);
        assert_eq!(payload.as_ref(), test_msg);
        assert_eq!(flags, 7);

        Ok(())
    }

    #[test]
    fn hopr_packet_message_surbs_only() -> anyhow::Result<()> {
        let surbs_1 = generate_surbs(2)?;
        let PacketParts {
            surbs: surbs_2,
            payload,
            flags,
        } = HoprPacketMessage::from_parts(surbs_1.clone(), &[], 7)?.try_into_parts()?;

        assert_eq!(surbs_2.len(), surbs_1.len());
        for (surb_1, surb_2) in surbs_1.into_iter().zip(surbs_2.into_iter()) {
            assert_eq!(surb_1.into_boxed(), surb_2.into_boxed());
        }

        assert!(payload.is_empty());
        assert_eq!(flags, 7);

        Ok(())
    }

    #[test]
    fn hopr_packet_message_surbs_and_msg() -> anyhow::Result<()> {
        let test_msg = b"test msg";
        let surbs_1 = generate_surbs(2)?;
        let hopr_msg = HoprPacketMessage::from_parts(surbs_1.clone(), test_msg, 7)?;
        let PacketParts {
            surbs: surbs_2,
            payload,
            flags,
        } = hopr_msg.try_into_parts()?;

        assert_eq!(surbs_2.len(), surbs_1.len());
        for (surb_1, surb_2) in surbs_1.into_iter().zip(surbs_2.into_iter()) {
            assert_eq!(surb_1.into_boxed(), surb_2.into_boxed());
        }

        assert_eq!(payload.as_ref(), test_msg);
        assert_eq!(flags, 7);

        Ok(())
    }

    #[test]
    fn hopr_packet_size_msg_size_limit() {
        let test_msg = [0u8; HoprPacket::PAYLOAD_SIZE + 1];
        let res = HoprPacketMessage::from_parts(vec![], &test_msg, 0);
        assert!(res.is_err());
    }

    #[test]
    fn hopr_packet_message_surbs_size_limit() -> anyhow::Result<()> {
        let surbs = generate_surbs(3)?;
        let res = HoprPacketMessage::from_parts(surbs, &[], 0);
        assert!(res.is_err());
        Ok(())
    }

    #[test]
    fn hopr_packet_message_surbs_max_size_limit() -> anyhow::Result<()> {
        let surbs = generate_surbs(16)?;
        let res = HoprPacketMessage::from_parts(surbs, &[], 0);
        assert!(res.is_err());
        Ok(())
    }

    #[test]
    fn hopr_packet_message_surbs_flag_limit() -> anyhow::Result<()> {
        let surbs = generate_surbs(3)?;
        let res = HoprPacketMessage::from_parts(surbs, &[], 8);
        assert!(res.is_err());
        Ok(())
    }

    #[test]
    fn hopr_packet_size_msg_and_surb_size_limit() -> anyhow::Result<()> {
        let test_msg = [0u8; HoprPacket::PAYLOAD_SIZE - 2 * HoprSurb::SIZE + 1];
        let surbs = generate_surbs(2)?;
        let res = HoprPacketMessage::from_parts(surbs, &test_msg, 0);
        assert!(res.is_err());
        Ok(())
    }
}
