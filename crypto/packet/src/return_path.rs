use hopr_crypto_sphinx::surb::SphinxRecipientMessage;
use hopr_crypto_types::types::Pseudonym;
use hopr_primitive_types::errors::GeneralError;
use hopr_primitive_types::prelude::BytesRepresentable;
use std::marker::PhantomData;

/// Encodes the [`SphinxRecipientMessage`] into a wire-format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedRecipientMessage<P: Pseudonym>(Box<[u8]>, PhantomData<P>); // cannot use generics in consts

impl<P: Pseudonym> From<SphinxRecipientMessage<P>> for EncodedRecipientMessage<P> {
    fn from(value: SphinxRecipientMessage<P>) -> Self {
        let mut ret = vec![0u8; Self::SIZE];
        const SL: usize = size_of::<u16>();
        match value {
            SphinxRecipientMessage::DataOnly => { /* ret[..] is zeroes */ }
            SphinxRecipientMessage::DataAndSurbs {
                num_surbs,
                pseudonym,
                remainder_data: 0,
            } => {
                let tag = num_surbs & 0x7fff;
                ret[0..SL].copy_from_slice(&tag.to_be_bytes());
                // ret[SL.. 2* SL] is zeroes
                ret[2 * SL..2 * SL + P::SIZE].copy_from_slice(pseudonym.as_ref());
            }
            SphinxRecipientMessage::DataAndSurbs {
                num_surbs,
                pseudonym,
                remainder_data,
            } => {
                let tag = num_surbs & 0x7fff;
                ret[0..SL].copy_from_slice(&tag.to_be_bytes());
                ret[SL..2 * SL].copy_from_slice(&remainder_data.to_be_bytes());
                ret[2 * SL..2 * SL + P::SIZE].copy_from_slice(pseudonym.as_ref());
            }
            SphinxRecipientMessage::ReplyOnly(pseudonym) => {
                ret[0..SL].copy_from_slice(&0x8000_u16.to_be_bytes());
                // ret[SL.. 2* SL] is zeroes
                ret[2 * SL..2 * SL + P::SIZE].copy_from_slice(pseudonym.as_ref());
            }
        }
        Self(ret.into_boxed_slice(), PhantomData)
    }
}

impl<P: Pseudonym> TryFrom<EncodedRecipientMessage<P>> for SphinxRecipientMessage<P> {
    type Error = GeneralError;

    fn try_from(value: EncodedRecipientMessage<P>) -> Result<Self, Self::Error> {
        let tag = u16::from_be_bytes([value.0[0], value.0[1]]);
        let is_reply = (tag & 0x8000) >> 15 == 1;
        let num_surbs = tag & 0x7fff;

        Ok(match (is_reply, num_surbs) {
            (false, 0) => Self::DataOnly,
            (false, _) => Self::DataAndSurbs {
                num_surbs,
                remainder_data: u16::from_be_bytes([value.0[2], value.0[3]]),
                pseudonym: P::try_from(&value.0[4..4 + P::SIZE])?,
            },
            (true, _) => Self::ReplyOnly(P::try_from(&value.0[4..4 + P::SIZE])?),
        })
    }
}

impl<P: Pseudonym> Default for EncodedRecipientMessage<P> {
    fn default() -> Self {
        SphinxRecipientMessage::<P>::DataOnly.into()
    }
}

impl<P: Pseudonym> AsRef<[u8]> for EncodedRecipientMessage<P> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<'a, P: Pseudonym> TryFrom<&'a [u8]> for EncodedRecipientMessage<P> {
    type Error = GeneralError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.len() == Self::SIZE {
            Ok(Self(value.into(), PhantomData))
        } else {
            Err(GeneralError::ParseError("HoprPseudonym".into()))
        }
    }
}

impl<P: Pseudonym> BytesRepresentable for EncodedRecipientMessage<P> {
    const SIZE: usize = size_of::<u16>() + size_of::<u16>() + P::SIZE;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HoprPseudonym;

    #[test]
    fn encoded_recipient_serialize_deserialize() -> anyhow::Result<()> {
        let m = SphinxRecipientMessage::<HoprPseudonym>::DataOnly;
        assert_eq!(m, SphinxRecipientMessage::try_from(EncodedRecipientMessage::from(m))?);

        let m = SphinxRecipientMessage::DataAndSurbs {
            num_surbs: 10,
            pseudonym: HoprPseudonym::random(),
            remainder_data: 20,
        };
        assert_eq!(m, SphinxRecipientMessage::try_from(EncodedRecipientMessage::from(m))?);

        let m = SphinxRecipientMessage::ReplyOnly(HoprPseudonym::random());
        assert_eq!(m, SphinxRecipientMessage::try_from(EncodedRecipientMessage::from(m))?);

        Ok(())
    }
}
