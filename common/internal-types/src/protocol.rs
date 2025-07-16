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
    /// Returns [`InvalidAcknowledgement`](CoreTypesError::InvalidAcknowledgement)
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
    /// Returns [`InvalidAcknowledgement`](CoreTypesError::InvalidAcknowledgement)
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
