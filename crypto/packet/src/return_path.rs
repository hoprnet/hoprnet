use hopr_crypto_sphinx::surb::Pseudonym;
use hopr_primitive_types::errors::GeneralError;
use hopr_primitive_types::prelude::{BytesRepresentable, ToHex};
use std::fmt::Display;
use std::marker::PhantomData;

/// Represents a simple UUID-like pseudonym consisting of 16 bytes.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SimplePseudonym(pub [u8; Self::SIZE]);

impl Display for SimplePseudonym {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl BytesRepresentable for SimplePseudonym {
    const SIZE: usize = 16;
}

impl AsRef<[u8]> for SimplePseudonym {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<'a> TryFrom<&'a [u8]> for SimplePseudonym {
    type Error = GeneralError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        value
            .try_into()
            .map(Self)
            .map_err(|_| GeneralError::ParseError("HoprPseudonym".into()))
    }
}

impl Pseudonym for SimplePseudonym {}

/// Represents an additional message delivered to the recipient of a Sphinx packet.
///
/// This message serves as an indication of what is included in the packet payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SphinxRecipientMessage<P: Pseudonym> {
    /// The packet payload contains only data.
    DataOnly,
    /// The packet payload contains a SURB followed by data.
    DataWithSurb(P),
    /// The packet contains only multiple SURBs with no more data.
    SurbsOnly(u8, P),
}

/// Encodes the [`SphinxRecipientMessage`] into a wire-format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedRecipientMessage<P: Pseudonym>(Box<[u8]>, PhantomData<P>); // cannot use generics in consts

impl<P: Pseudonym> From<SphinxRecipientMessage<P>> for EncodedRecipientMessage<P> {
    fn from(value: SphinxRecipientMessage<P>) -> Self {
        let mut ret = Self::default();
        match value {
            SphinxRecipientMessage::DataOnly => {}
            SphinxRecipientMessage::DataWithSurb(p) => {
                ret.0[0] = 1;
                ret.0[1..].copy_from_slice(p.as_ref());
            }
            SphinxRecipientMessage::SurbsOnly(n, p) => {
                ret.0[0] = n;
                ret.0[1..].copy_from_slice(p.as_ref());
            }
        }
        ret
    }
}

impl<P: Pseudonym> PartialEq<SphinxRecipientMessage<P>> for EncodedRecipientMessage<P> {
    fn eq(&self, other: &SphinxRecipientMessage<P>) -> bool {
        match other {
            SphinxRecipientMessage::DataOnly => self.0[0] == 0,
            SphinxRecipientMessage::DataWithSurb(p) => self.0[0] == 1 && self.0[1..].eq(p.as_ref()),
            SphinxRecipientMessage::SurbsOnly(n, p) => self.0[1] == *n && self.0[1..].eq(p.as_ref()),
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
