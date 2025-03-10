use hopr_crypto_sphinx::surb::SphinxRecipientMessage;
use hopr_crypto_types::types::Pseudonym;
use hopr_primitive_types::errors::GeneralError;
use hopr_primitive_types::prelude::BytesRepresentable;
use std::fmt::Display;
use std::marker::PhantomData;

/// Encodes the [`SphinxRecipientMessage`] into a wire-format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedRecipientMessage<P: Pseudonym>(Box<[u8]>, PhantomData<P>); // cannot use generics in consts

impl<P: Pseudonym> From<SphinxRecipientMessage<P>> for EncodedRecipientMessage<P> {
    fn from(value: SphinxRecipientMessage<P>) -> Self {
        let mut ret = Self::default();
        match value {
            SphinxRecipientMessage::DataOnly => {}
            SphinxRecipientMessage::ReplyOnly(p) => {
                ret.0[0] = 1 << 4;
                ret.0[1..].copy_from_slice(p.as_ref());
            }
            SphinxRecipientMessage::DataWithSurb(p) => {
                ret.0[0] = 2 << 4;
                ret.0[1..].copy_from_slice(p.as_ref());
            }
            SphinxRecipientMessage::SurbsOnly(n, p) => {
                ret.0[0] = (3 << 4) | (n & 0x0f);
                ret.0[1..].copy_from_slice(p.as_ref());
            }
        }
        ret
    }
}

impl<P: Pseudonym> TryFrom<EncodedRecipientMessage<P>> for SphinxRecipientMessage<P> {
    type Error = GeneralError;

    fn try_from(value: EncodedRecipientMessage<P>) -> Result<Self, Self::Error> {
        let message_id = (value.0[0] & 0xf0) >> 4;
        let num_surbs = value.0[0] & 0x0f;
        match message_id {
            0 => Ok(SphinxRecipientMessage::DataOnly),
            1 => Ok(SphinxRecipientMessage::ReplyOnly(P::try_from(&value.0[1..])?)),
            2 => {
                let pseudonym_data = &value.0[1..];
                let pseudonym = P::try_from(pseudonym_data)?;
                Ok(SphinxRecipientMessage::DataWithSurb(pseudonym))
            }
            3 => {
                let pseudonym_data = &value.0[1..];
                let pseudonym = P::try_from(pseudonym_data)?;
                Ok(SphinxRecipientMessage::SurbsOnly(num_surbs, pseudonym))
            }
            _ => Err(GeneralError::ParseError("HoprPseudonym".into())),
        }
    }
}

impl<P: Pseudonym> PartialEq<SphinxRecipientMessage<P>> for EncodedRecipientMessage<P> {
    fn eq(&self, other: &SphinxRecipientMessage<P>) -> bool {
        let message_id = (self.0[0] & 0xf0) >> 4;
        match other {
            SphinxRecipientMessage::DataOnly => message_id == 0,
            SphinxRecipientMessage::ReplyOnly(p) => message_id == 1 && self.0[1..].eq(p.as_ref()),
            SphinxRecipientMessage::DataWithSurb(p) => message_id == 2 && self.0[1..].eq(p.as_ref()),
            SphinxRecipientMessage::SurbsOnly(n, p) => {
                let num_surbs = self.0[0] & 0x0f;
                message_id == 3 && num_surbs == *n && self.0[1..].eq(p.as_ref())
            }
        }
    }
}

impl<P: Pseudonym> Default for EncodedRecipientMessage<P> {
    fn default() -> Self {
        Self(vec![0u8; Self::SIZE].into_boxed_slice(), PhantomData)
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
    const SIZE: usize = 1 + P::SIZE;
}
