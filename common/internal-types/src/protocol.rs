use std::fmt::Display;

use hopr_crypto_random::Randomizable;
use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;
use tracing::warn;

use crate::{
    errors::{CoreTypesError, Result},
    prelude::UnacknowledgedTicket,
};

/// Number of intermediate hops: 3 relayers and 1 destination
pub const INTERMEDIATE_HOPS: usize = 3;

/// Default required minimum incoming ticket winning probability
pub const DEFAULT_MINIMUM_INCOMING_TICKET_WIN_PROB: f64 = 1.0;

/// Default maximum incoming ticket winning probability, above which tickets will not be accepted
/// due to privacy.
pub const DEFAULT_MAXIMUM_INCOMING_TICKET_WIN_PROB: f64 = 1.0; // TODO: change this in 3.0

/// Tags are represented as 4 bytes.
///
/// 2^32 should provide enough range for all usecases.
pub type Tag = u32;

/// Alias for the [`Pseudonym`](`hopr_crypto_types::types::Pseudonym`) used in the HOPR protocol.
pub type HoprPseudonym = SimplePseudonym;

/// Represents packet acknowledgement
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Acknowledgement {
    #[cfg_attr(feature = "serde", serde(with = "serde_bytes"))]
    data: [u8; Self::SIZE],
    #[cfg_attr(feature = "serde", serde(skip))]
    validated: bool,
}

impl AsRef<[u8]> for Acknowledgement {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl TryFrom<&[u8]> for Acknowledgement {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        if value.len() == Self::SIZE {
            Ok(Self {
                data: value.try_into().unwrap(),
                validated: false,
            })
        } else {
            Err(GeneralError::ParseError("Acknowledgement".into()))
        }
    }
}

impl Acknowledgement {
    pub fn new(ack_key_share: HalfKey, node_keypair: &OffchainKeypair) -> Self {
        let signature = OffchainSignature::sign_message(ack_key_share.as_ref(), node_keypair);
        let mut data = [0u8; Self::SIZE];
        data[0..HalfKey::SIZE].copy_from_slice(ack_key_share.as_ref());
        data[HalfKey::SIZE..HalfKey::SIZE + OffchainSignature::SIZE].copy_from_slice(signature.as_ref());

        Self { data, validated: true }
    }

    /// Generates random but still a valid acknowledgement.
    pub fn random(offchain_keypair: &OffchainKeypair) -> Self {
        Self::new(HalfKey::random(), offchain_keypair)
    }

    /// Validates the acknowledgement.
    ///
    /// Must be called immediately after deserialization, or otherwise
    /// any operations with the deserialized acknowledgement return an error.
    #[tracing::instrument(level = "debug", skip(self, sender_node_key))]
    pub fn validate(self, sender_node_key: &OffchainPublicKey) -> Result<Self> {
        if !self.validated {
            let signature =
                OffchainSignature::try_from(&self.data[HalfKey::SIZE..HalfKey::SIZE + OffchainSignature::SIZE])?;
            if signature.verify_message(&self.data[0..HalfKey::SIZE], sender_node_key) {
                Ok(Self {
                    data: self.data,
                    validated: true,
                })
            } else {
                Err(CoreTypesError::InvalidAcknowledgement)
            }
        } else {
            Ok(self)
        }
    }

    /// Gets the acknowledged key out of this acknowledgement.
    ///
    /// Returns [`InvalidAcknowledgement`]
    /// if the acknowledgement has not been [validated](Acknowledgement::validate).
    pub fn ack_key_share(&self) -> Result<HalfKey> {
        if self.validated {
            Ok(HalfKey::try_from(&self.data[0..HalfKey::SIZE])?)
        } else {
            Err(CoreTypesError::InvalidAcknowledgement)
        }
    }

    /// Gets the acknowledgement challenge out of this acknowledgement.
    ///
    /// Returns [`InvalidAcknowledgement`]
    /// if the acknowledgement has not been [validated](Acknowledgement::validate).
    pub fn ack_challenge(&self) -> Result<HalfKeyChallenge> {
        Ok(self.ack_key_share()?.to_challenge())
    }

    /// Indicates whether the acknowledgement has been [validated](Acknowledgement::validate).
    pub fn is_validated(&self) -> bool {
        self.validated
    }
}

impl BytesRepresentable for Acknowledgement {
    const SIZE: usize = HalfKey::SIZE + OffchainSignature::SIZE;
}

/// Contains either unacknowledged ticket if we're waiting for the acknowledgement as a relayer
/// or information if we wait for the acknowledgement as a sender.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PendingAcknowledgement {
    /// We're waiting for acknowledgement as a sender
    WaitingAsSender,
    /// We're waiting for the acknowledgement as a relayer with a ticket
    WaitingAsRelayer(UnacknowledgedTicket),
}

/// Represents the received decrypted packet carrying the application-layer data.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ApplicationData {
    pub application_tag: Tag,
    #[cfg_attr(feature = "serde", serde(with = "serde_bytes"))]
    pub plain_text: Box<[u8]>,
}

impl ApplicationData {
    pub fn new(application_tag: Tag, plain_text: &[u8]) -> Self {
        Self {
            application_tag,
            plain_text: plain_text.into(),
        }
    }

    pub fn new_from_owned(application_tag: Tag, plain_text: Box<[u8]>) -> Self {
        Self {
            application_tag,
            plain_text,
        }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        Self::TAG_SIZE + self.plain_text.len()
    }
}

impl Display for ApplicationData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}): {}", self.application_tag, hex::encode(&self.plain_text))
    }
}

impl ApplicationData {
    const TAG_SIZE: usize = size_of::<Tag>();

    // minimum size

    pub fn from_bytes(data: &[u8]) -> hopr_primitive_types::errors::Result<Self> {
        if data.len() >= Self::TAG_SIZE {
            Ok(Self {
                application_tag: Tag::from_be_bytes(
                    data[0..Self::TAG_SIZE]
                        .try_into()
                        .map_err(|_| GeneralError::ParseError("ApplicationData.tag".into()))?,
                ),
                plain_text: Box::from(&data[Self::TAG_SIZE..]),
            })
        } else {
            Err(GeneralError::ParseError("ApplicationData".into()))
        }
    }

    pub fn to_bytes(&self) -> Box<[u8]> {
        let mut buf = Vec::with_capacity(Self::TAG_SIZE + self.plain_text.len());
        buf.extend_from_slice(&self.application_tag.to_be_bytes());
        buf.extend_from_slice(&self.plain_text);
        buf.into_boxed_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_application_data() -> anyhow::Result<()> {
        let ad_1 = ApplicationData::new(10, &[0_u8, 1_u8]);
        let ad_2 = ApplicationData::from_bytes(&ad_1.to_bytes())?;
        assert_eq!(ad_1, ad_2);

        let ad_1 = ApplicationData::new(0, &[]);
        let ad_2 = ApplicationData::from_bytes(&ad_1.to_bytes())?;
        assert_eq!(ad_1, ad_2);

        let ad_1 = ApplicationData::new(10, &[0_u8, 1_u8]);
        let ad_2 = ApplicationData::from_bytes(&ad_1.to_bytes())?;
        assert_eq!(ad_1, ad_2);

        Ok(())
    }
}
