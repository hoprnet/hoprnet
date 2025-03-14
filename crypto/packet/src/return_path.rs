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
        let mut ret = Vec::with_capacity(Self::SIZE);
        match value {
            SphinxRecipientMessage::DataOnly => {
                ret.extend(std::iter::repeat(0u8).take(Self::SIZE));
            },
            SphinxRecipientMessage::DataAndSurbs { num_surbs, pseudonym, remainder_data: 0 } => {
                let tag = 0x4000_u16 | num_surbs & 0x3fff;
                ret.extend(tag.to_be_bytes());
                ret.extend(0u16.to_be_bytes());
                ret.extend(pseudonym.as_ref());
            },
            SphinxRecipientMessage::DataAndSurbs { num_surbs, pseudonym, remainder_data } => {
                let tag = 0xc000_u16 | num_surbs & 0x3fff;
                ret.extend(tag.to_be_bytes());
                ret.extend(remainder_data.to_be_bytes());
                ret.extend(pseudonym.as_ref());
            },
            SphinxRecipientMessage::ReplyOnly(pseudonym) => {
                let tag = 0x8000_u16;
                ret.extend(tag.to_be_bytes());
                ret.extend(0u16.to_be_bytes());
                ret.extend(pseudonym.as_ref());
            }
        }
        Self(ret.into_boxed_slice(), PhantomData)
    }
}

impl<P: Pseudonym> TryFrom<EncodedRecipientMessage<P>> for SphinxRecipientMessage<P> {
    type Error = GeneralError;

    fn try_from(value: EncodedRecipientMessage<P>) -> Result<Self, Self::Error> {
        todo!()
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
    const SIZE: usize = size_of::<u16>() + size_of::<u16>() + P::SIZE;
}
