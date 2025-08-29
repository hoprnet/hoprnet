use hopr_crypto_random::Randomizable;
use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;

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

/// Unverified packet acknowledgement.
///
/// This acknowledgement can be serialized and deserialized to be sent over the wire.
///
/// To retrieve useful data, it has to be [verified](`Acknowledgement::verify`) using the public
/// key of its sender.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Acknowledgement(#[cfg_attr(feature = "serde", serde(with = "serde_bytes"))] [u8; Self::SIZE]);

impl AsRef<[u8]> for Acknowledgement {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&[u8]> for Acknowledgement {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        Ok(Self(
            value
                .try_into()
                .map_err(|_| GeneralError::ParseError("Acknowledgement".into()))?,
        ))
    }
}

impl BytesRepresentable for Acknowledgement {
    const SIZE: usize = HalfKey::SIZE + OffchainSignature::SIZE;
}

impl Acknowledgement {
    /// Attempts to verify the acknowledgement given the `sender_node_key` that sent the acknowledgement.
    pub fn verify(self, sender_node_key: &OffchainPublicKey) -> Result<VerifiedAcknowledgement> {
        let signature = OffchainSignature::try_from(&self.0[HalfKey::SIZE..HalfKey::SIZE + OffchainSignature::SIZE])?;
        if signature.verify_message(&self.0[0..HalfKey::SIZE], sender_node_key) {
            Ok(VerifiedAcknowledgement {
                ack_key_share: HalfKey::try_from(&self.0[0..HalfKey::SIZE])?,
                signature,
            })
        } else {
            Err(CoreTypesError::InvalidAcknowledgement)
        }
    }
}

/// Represents packet acknowledgement whose signature has been already verified.
///
/// This acknowledgement cannot be serialized.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct VerifiedAcknowledgement {
    ack_key_share: HalfKey,
    signature: OffchainSignature,
}

impl VerifiedAcknowledgement {
    pub fn new(ack_key_share: HalfKey, node_keypair: &OffchainKeypair) -> Self {
        let signature = OffchainSignature::sign_message(ack_key_share.as_ref(), node_keypair);
        Self {
            ack_key_share,
            signature,
        }
    }

    /// Generates random but still a valid acknowledgement.
    pub fn random(offchain_keypair: &OffchainKeypair) -> Self {
        Self::new(HalfKey::random(), offchain_keypair)
    }

    /// Downgrades this verified acknowledgement to an unverified serializable one.
    pub fn leak(self) -> Acknowledgement {
        let mut ret = [0u8; Acknowledgement::SIZE];
        ret[0..HalfKey::SIZE].copy_from_slice(self.ack_key_share.as_ref());
        ret[HalfKey::SIZE..HalfKey::SIZE + OffchainSignature::SIZE].copy_from_slice(self.signature.as_ref());
        Acknowledgement(ret)
    }

    /// Gets the acknowledged key out of this acknowledgement.
    ///
    /// This is the remaining part of the solution of the `Ticket` challenge.
    pub fn ack_key_share(&self) -> &HalfKey {
        &self.ack_key_share
    }
}

/// Contains either unacknowledged ticket if we're waiting for the acknowledgement as a relayer
/// or information if we wait for the acknowledgement as a sender.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PendingAcknowledgement {
    /// We're waiting for acknowledgement as a sender
    WaitingAsSender,
    /// We're waiting for the acknowledgement as a relayer with a ticket
    WaitingAsRelayer(Box<UnacknowledgedTicket>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acknowledgement_should_verify() -> anyhow::Result<()> {
        let kp = OffchainKeypair::random();
        let v_ack_1 = VerifiedAcknowledgement::random(&kp);
        let v_ack_2 = v_ack_1.leak().verify(kp.public())?;

        assert_eq!(v_ack_1, v_ack_2);
        Ok(())
    }
}
