use std::{borrow::Cow, fmt::Formatter, marker::PhantomData, ops::Not};

use hopr_protocol_pix::{CofactorGroup, GroupEncoding, PixGroup};
use hopr_types::primitive::prelude::{BytesRepresentable, GeneralError};

use crate::{
    HoprEncryptedPartialSsaShare, HoprPixGroupRepr, HoprPixSpec, HoprSphinxHeaderSpec, HoprSphinxSuite,
    PAYLOAD_SIZE_INT,
    por::ProofOfRelayValues,
    sphinx::{
        errors::SphinxError,
        prelude::{PaddedPayload, SURB, SphinxHeaderSpec, SphinxSuite},
    },
};

flagset::flags! {
   /// Individual packet signals passed up between the packet sender and destination.
   #[repr(u8)]
   #[derive(PartialOrd, Ord, strum::EnumString, strum::Display)]
   pub enum PacketSignal: u8 {
        /// The other party is in a "SURB distress" state, potentially running out of SURBs soon.
        ///
        /// Has no effect on packets that take the "forward path".
        SurbDistress = 0b0000_0001,
        /// The other party has run out of SURBs, and this was potentially the last message they could
        /// send.
        ///
        /// Has no effect on packets that take the "forward path".
        ///
        /// Implies [`SurbDistress`].
        OutOfSurbs = 0b0000_0011,
   }
}

/// Packet signal states that can be passed between the packet sender and destination.
///
/// These signals can be typically propagated up to the application layer to take an appropriate
/// action to the signaled states.
pub type PacketSignals = flagset::FlagSet<PacketSignal>;

/// Additional encoding of a packet message that can be preceded by a number of [`SURBs`](SURB).
pub struct PacketMessage<S: SphinxSuite, H: SphinxHeaderSpec, const P: usize>(PaddedPayload<P>, PhantomData<(S, H)>);

/// Convenience alias for HOPR specific [`PacketMessage`].
pub type HoprPacketMessage = PacketMessage<HoprSphinxSuite, HoprSphinxHeaderSpec, PAYLOAD_SIZE_INT>;

/// Individual parts of a [`PacketMessage`]: SURBs, the actual message (payload) and additional signals for the
/// recipient.
pub struct PacketParts<'a, S: SphinxSuite, H: SphinxHeaderSpec> {
    /// Contains (a potentially empty) list of SURBs.
    pub surbs: Vec<SURB<S, H>>,
    /// Contains the actual packet payload.
    pub payload: Cow<'a, [u8]>,
    /// Additional packet signals from the sender to the recipient.
    pub signals: PacketSignals,
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> Clone for PacketParts<'_, S, H>
where
    H::KeyId: Clone,
    H::SurbReceiverData: Clone,
{
    fn clone(&self) -> Self {
        Self {
            surbs: self.surbs.clone(),
            payload: self.payload.clone(),
            signals: self.signals,
        }
    }
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> std::fmt::Debug for PacketParts<'_, S, H>
where
    H::KeyId: std::fmt::Debug,
    H::SurbReceiverData: std::fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PacketParts")
            .field("surbs", &self.surbs)
            .field("payload", &self.payload)
            .field("signals", &self.signals)
            .finish()
    }
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> PartialEq for PacketParts<'_, S, H>
where
    H::KeyId: PartialEq,
    H::SurbReceiverData: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.surbs == other.surbs && self.payload == other.payload && self.signals == other.signals
    }
}

impl<S: SphinxSuite, H: SphinxHeaderSpec> Eq for PacketParts<'_, S, H>
where
    H::KeyId: Eq,
    H::SurbReceiverData: Eq,
{
}

/// Convenience alias for HOPR specific [`PacketParts`].
pub type HoprPacketParts<'a> = PacketParts<'a, HoprSphinxSuite, HoprSphinxHeaderSpec>;

// Coerces PacketSignals to only lower 4 bits.
pub(crate) const S_MASK: u8 = 0b0000_1111;

impl<S: SphinxSuite, H: SphinxHeaderSpec, const P: usize> PacketMessage<S, H, P> {
    /// Size of the message header.
    ///
    /// This is currently 1 byte to indicate the number of SURBs that precede the message.
    pub const HEADER_LEN: usize = 1;
    /// The maximum number of SURBs a packet message can hold, according to RFC-0003.
    ///
    /// The number of SURBs in a `PacketMessage` is intentionally limited to 15, so that
    /// the upper 4 bits remain reserved for additional flags.
    pub const MAX_SURBS_PER_MESSAGE: usize = S_MASK as usize;
}

impl<S: SphinxSuite, H: SphinxHeaderSpec, const P: usize> TryFrom<PacketParts<'_, S, H>> for PacketMessage<S, H, P> {
    type Error = SphinxError;

    fn try_from(value: PacketParts<S, H>) -> Result<Self, Self::Error> {
        if value.surbs.len() > Self::MAX_SURBS_PER_MESSAGE {
            return Err(GeneralError::ParseError("HoprPacketMessage.num_surbs not valid".into()).into());
        }

        if value.signals.bits() > S_MASK {
            return Err(GeneralError::ParseError("HoprPacketMessage.flags not valid".into()).into());
        }

        // The total size of the packet message must not exceed the maximum packet size.
        if Self::HEADER_LEN + value.surbs.len() * SURB::<S, H>::SIZE + value.payload.len() > P {
            return Err(GeneralError::ParseError("HoprPacketMessage.size not valid".into()).into());
        }

        let mut ret = Vec::with_capacity(PaddedPayload::<P>::SIZE);
        let flags_and_len = (value.signals.bits() << S_MASK.trailing_ones()) | (value.surbs.len() as u8 & S_MASK);
        ret.push(flags_and_len);
        for surb in value.surbs.into_iter().map(|s| s.into_boxed()) {
            ret.extend(surb);
        }
        ret.extend_from_slice(value.payload.as_ref());

        // Save one reallocation by using the vector that we just created
        Ok(Self(PaddedPayload::new_from_vec(ret)?, PhantomData))
    }
}

impl<S: SphinxSuite, H: SphinxHeaderSpec, const P: usize> TryFrom<PacketMessage<S, H, P>> for PacketParts<'_, S, H> {
    type Error = SphinxError;

    fn try_from(value: PacketMessage<S, H, P>) -> Result<Self, Self::Error> {
        let data = value.0.into_unpadded()?;
        if data.is_empty() {
            return Err(GeneralError::ParseError("HoprPacketMessage.size".into()).into());
        }

        let num_surbs = (data[0] & S_MASK) as usize;
        let signals = PacketSignals::new((data[0] & S_MASK.not()) >> S_MASK.trailing_ones())
            .map_err(|_| GeneralError::ParseError("HoprPacketMessage.signals".into()))?;

        if num_surbs > 0 {
            let surb_end = num_surbs * SURB::<S, H>::SIZE;
            if surb_end >= data.len() {
                return Err(GeneralError::ParseError("HoprPacketMessage.num_surbs not valid".into()).into());
            }

            let mut data = data.into_vec();

            let surbs = data[1..=surb_end]
                .chunks_exact(SURB::<S, H>::SIZE)
                .map(SURB::<S, H>::try_from)
                .collect::<Result<Vec<_>, _>>()?;

            // Skip buffer all the way to the end of the SURBs.
            data.drain(0..=surb_end).for_each(drop);

            Ok(PacketParts {
                surbs,
                payload: Cow::Owned(data),
                signals,
            })
        } else {
            let mut data = data.into_vec();
            data.remove(0);
            Ok(PacketParts {
                surbs: Vec::with_capacity(0),
                payload: Cow::Owned(data),
                signals,
            })
        }
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

/// Wraps the [`ProofOfRelayValues`] with some additional information about the sender of the packet,
/// that is supposed to be passed along with the SURB.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SurbReceiverInfo(#[cfg_attr(feature = "serde", serde(with = "serde_bytes"))] [u8; Self::SIZE]);

impl SurbReceiverInfo {
    pub fn new(pov: ProofOfRelayValues, encrypted_partial_ssa_share: HoprEncryptedPartialSsaShare) -> Self {
        let mut ret = [0u8; Self::SIZE];
        ret[0..ProofOfRelayValues::SIZE].copy_from_slice(pov.as_ref());
        ret[ProofOfRelayValues::SIZE..ProofOfRelayValues::SIZE + HoprEncryptedPartialSsaShare::SIZE]
            .copy_from_slice(encrypted_partial_ssa_share.as_ref());
        Self(ret)
    }

    pub fn proof_of_relay_values(&self) -> ProofOfRelayValues {
        ProofOfRelayValues::try_from(&self.0[0..ProofOfRelayValues::SIZE])
            .expect("SurbReceiverInfo always contains valid ProofOfRelayValues")
    }

    pub fn encrypted_partial_ssa_share(&self) -> HoprEncryptedPartialSsaShare {
        HoprEncryptedPartialSsaShare::try_from(
            &self.0[ProofOfRelayValues::SIZE..ProofOfRelayValues::SIZE + HoprEncryptedPartialSsaShare::SIZE],
        )
        .expect("SurbReceiverInfo always contains valid HoprEncryptedPartialSsaShare")
    }
}

impl AsRef<[u8]> for SurbReceiverInfo {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<'a> TryFrom<&'a [u8]> for SurbReceiverInfo {
    type Error = GeneralError;

    fn try_from(value: &'a [u8]) -> std::result::Result<Self, Self::Error> {
        value
            .try_into()
            .map(Self)
            .map_err(|_| GeneralError::ParseError("SurbReceiverInfo".into()))
    }
}

impl BytesRepresentable for SurbReceiverInfo {
    const SIZE: usize = ProofOfRelayValues::SIZE + HoprEncryptedPartialSsaShare::SIZE;
}

/// New-type wrapper for the PIX group element representation to provide additional functionality.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HoprPixGroupElement(pub HoprPixGroupRepr);

impl std::hash::Hash for HoprPixGroupElement {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_ref().hash(state);
    }
}

impl HoprPixGroupElement {
    /// Tries to convert the instance into a `PixGroup<HoprPixSpec>`.
    pub fn try_into_pix_group(self) -> Result<PixGroup<HoprPixSpec>, GeneralError> {
        Option::<PixGroup<HoprPixSpec>>::from(PixGroup::<HoprPixSpec>::from_bytes(&self.0))
            .filter(|pt| {
                // Reject points outside the prime-order subgroup. Baby JubJub has
                // cofactor 8, so small-order points can pass the on-curve check.
                bool::from(pt.is_torsion_free())
            })
            .ok_or(GeneralError::ParseError("pix group from bytes failed".into()))
    }
}

impl From<HoprPixGroupRepr> for HoprPixGroupElement {
    fn from(value: HoprPixGroupRepr) -> Self {
        Self(value)
    }
}

impl AsRef<[u8]> for HoprPixGroupElement {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<'a> TryFrom<&'a [u8]> for HoprPixGroupElement {
    type Error = GeneralError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.len() != size_of::<HoprPixGroupRepr>() {
            return Err(GeneralError::ParseError("pix repr length".into()));
        }
        let mut arr = HoprPixGroupRepr::default();
        arr.as_mut().copy_from_slice(value);
        Ok(Self(arr))
    }
}

impl std::fmt::Display for HoprPixGroupElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", const_hex::encode(self.0))
    }
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use bimap::BiHashMap;
    use hex_literal::hex;
    use hopr_types::{
        crypto::prelude::*, crypto_random::Randomizable, internal::routing::HoprSenderId, primitive::prelude::*,
    };

    use super::*;
    use crate::{
        HoprEncryptedPartialSsaShare, HoprSphinxHeaderSpec, HoprSphinxSuite, HoprSurb, packet::HoprPacket,
        por::generate_proof_of_relay, sphinx::prelude::*, types::SurbReceiverInfo,
    };

    lazy_static::lazy_static! {
        static ref PEERS: [(ChainKeypair, OffchainKeypair); 4] = [
            (hex!("a7c486ceccf5ab53bd428888ab1543dc2667abd2d5e80aae918da8d4b503a426"), hex!("5eb212d4d6aa5948c4f71574d45dad43afef6d330edb873fca69d0e1b197e906")),
            (hex!("9a82976f7182c05126313bead5617c623b93d11f9f9691c87b1a26f869d569ed"), hex!("e995db483ada5174666c46bafbf3628005aca449c94ebdc0c9239c3f65d61ae0")),
            (hex!("ca4bdfd54a8467b5283a0216288fdca7091122479ccf3cfb147dfa59d13f3486"), hex!("9dec751c00f49e50fceff7114823f726a0425a68a8dc6af0e4287badfea8f4a4")),
            (hex!("e306ebfb0d01d0da0952c9a567d758093a80622c6cb55052bf5f1a6ebd8d7b5c"), hex!("9a82976f7182c05126313bead5617c623b93d11f9f9691c87b1a26f869d569ed"))
        ].map(|(p1,p2)| (ChainKeypair::from_secret(&p1).expect("lazy static keypair should be valid"), OffchainKeypair::from_secret(&p2).expect("lazy static keypair should be valid")));

        static ref MAPPER: SimpleBiMapper<HoprSphinxSuite, HoprSphinxHeaderSpec> = PEERS
            .iter()
            .enumerate()
            .map(|(i, (_, k))| (KeyIdent::from(i as u32), *k.public()))
            .collect::<BiHashMap<_, _>>()
            .into();
    }

    fn generate_surbs(count: usize) -> anyhow::Result<Vec<SURB<HoprSphinxSuite, HoprSphinxHeaderSpec>>> {
        let path = PEERS.iter().map(|(_, k)| *k.public()).collect::<Vec<_>>();
        let path_ids = MAPPER
            .map_keys_to_ids(&path)
            .into_iter()
            .map(|v| v.ok_or(anyhow!("missing id")))
            .collect::<Result<Vec<_>, _>>()?;
        let pseudonym = SimplePseudonym::random();
        let recv_data = HoprSenderId::new(&pseudonym);

        Ok((0..count)
            .map(|_| {
                let shared_keys = HoprSphinxSuite::new_shared_keys(&path)?;
                let (por_strings, (por_values, _)) = generate_proof_of_relay(&shared_keys.secrets)
                    .map_err(|e| CryptoError::Other(GeneralError::NonSpecificError(e.to_string())))?;

                create_surb::<HoprSphinxSuite, HoprSphinxHeaderSpec>(
                    shared_keys,
                    &path_ids,
                    &por_strings,
                    recv_data,
                    SurbReceiverInfo::new(por_values, HoprEncryptedPartialSsaShare::default()),
                )
                .map(|(s, _)| s)
            })
            .collect::<Result<Vec<_>, _>>()?)
    }

    #[test]
    fn hopr_packet_message_message_only() -> anyhow::Result<()> {
        let parts_1 = HoprPacketParts {
            surbs: vec![],
            payload: b"test message".into(),
            signals: PacketSignal::OutOfSurbs.into(),
        };

        let parts_2: HoprPacketParts = HoprPacketMessage::try_from(parts_1.clone())?.try_into()?;
        assert_eq!(parts_1, parts_2);

        Ok(())
    }

    #[test]
    fn hopr_packet_message_surbs_only() -> anyhow::Result<()> {
        let parts_1 = HoprPacketParts {
            surbs: generate_surbs(2)?,
            payload: Cow::default(),
            signals: PacketSignal::OutOfSurbs.into(),
        };

        let parts_2: HoprPacketParts = HoprPacketMessage::try_from(parts_1.clone())?.try_into()?;
        assert_eq!(parts_1, parts_2);

        Ok(())
    }

    #[test]
    fn hopr_packet_message_surbs_and_msg() -> anyhow::Result<()> {
        let parts_1 = HoprPacketParts {
            surbs: generate_surbs(2)?,
            payload: b"test msg".into(),
            signals: PacketSignal::OutOfSurbs.into(),
        };

        let parts_2: HoprPacketParts = HoprPacketMessage::try_from(parts_1.clone())?.try_into()?;
        assert_eq!(parts_1, parts_2);

        Ok(())
    }

    #[test]
    fn hopr_packet_size_msg_size_limit() {
        let res = HoprPacketMessage::try_from(HoprPacketParts {
            surbs: vec![],
            payload: (&[1u8; HoprPacket::PAYLOAD_SIZE + 1]).into(),
            signals: None.into(),
        });
        assert!(res.is_err());
    }

    #[test]
    fn hopr_packet_message_surbs_size_limit() -> anyhow::Result<()> {
        let res = HoprPacketMessage::try_from(PacketParts {
            surbs: generate_surbs(HoprPacketMessage::MAX_SURBS_PER_MESSAGE + 1)?,
            payload: Cow::default(),
            signals: None.into(),
        });
        assert!(res.is_err());

        let res = HoprPacketMessage::try_from(HoprPacketParts {
            surbs: generate_surbs(3)?,
            payload: Cow::default(),
            signals: None.into(),
        });
        assert!(res.is_err());

        Ok(())
    }

    #[test]
    fn hopr_packet_message_surbs_flag_limit() -> anyhow::Result<()> {
        let res = HoprPacketMessage::try_from(PacketParts {
            surbs: generate_surbs(3)?,
            payload: Cow::default(),
            signals: unsafe { PacketSignals::new_unchecked(16) },
        });
        assert!(res.is_err());

        Ok(())
    }

    #[test]
    fn hopr_packet_size_msg_and_surb_size_limit() -> anyhow::Result<()> {
        let res = HoprPacketMessage::try_from(PacketParts {
            surbs: generate_surbs(2)?,
            payload: (&[1u8; HoprPacket::PAYLOAD_SIZE - 2 * HoprSurb::SIZE + 1]).into(),
            signals: None.into(),
        });
        assert!(res.is_err());

        Ok(())
    }
}
