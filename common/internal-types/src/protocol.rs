use hopr_crypto_random::Randomizable;
use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;
#[cfg(feature = "rayon")]
use rayon::prelude::*;

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

const MIN_BATCH_SIZE: usize = 4;

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

    /// Performs batch verification of acknowledgements received from given senders.
    ///
    /// For batches of sizes than 4, the regular verification of each signature is performed.
    ///
    /// For larger batches, this method makes use of EdDSA batch signature verification algorithm,
    /// which more effectively verifies batch, while being faster than verifying those signatures individually.
    /// This comes at a cost of not knowing which signature was invalid in the case of failure.
    ///
    /// This method first tries to verify the batch using the batch signature verification, returning a fast
    /// successful verification result quickly.
    /// If one or more of the acknowledgements in the batch were invalid, it preforms individual checks for
    /// each signature in the batch, returning the vector of results.
    pub fn verify_batch(
        acknowledgements: Vec<(OffchainPublicKey, Acknowledgement)>,
    ) -> Vec<Result<VerifiedAcknowledgement>> {
        if acknowledgements.len() < MIN_BATCH_SIZE {
            return acknowledgements
                .into_iter()
                .map(|(key, ack)| ack.verify(&key))
                .collect();
        }

        let mut optimistic_result = Vec::with_capacity(acknowledgements.len());
        let optimistic_check = OffchainSignature::verify_batch(
            acknowledgements
                .iter()
                .map(|(key, ack)| {
                    let signature =
                        OffchainSignature::try_from(&ack.0[HalfKey::SIZE..HalfKey::SIZE + OffchainSignature::SIZE])?;
                    optimistic_result.push(Ok(VerifiedAcknowledgement {
                        ack_key_share: HalfKey::try_from(&ack.0[0..HalfKey::SIZE])?,
                        signature,
                    }));
                    Ok(((&ack.0[0..HalfKey::SIZE], signature), key))
                })
                .filter_map(|r: Result<((&[u8], OffchainSignature), &OffchainPublicKey)>| r.ok()),
        );

        // If the batch verification succeeded, we can return the entire batch as verified.
        // Otherwise, we need to check individual acknowledgements to see which ones failed.
        if optimistic_check {
            optimistic_result
        } else {
            #[cfg(feature = "rayon")]
            let iter = acknowledgements.into_par_iter();

            #[cfg(not(feature = "rayon"))]
            let iter = acknowledgements.into_iter();

            iter.map(|(key, ack)| ack.verify(&key)).collect()
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
    /// Creates a new verified acknowledgement by signing the given [`HalfKey`] with the given [`OffchainKeypair`].
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
    use std::ops::Not;

    use super::*;

    #[test]
    fn acknowledgement_should_verify() -> anyhow::Result<()> {
        let kp = OffchainKeypair::random();
        let v_ack_1 = VerifiedAcknowledgement::random(&kp);
        let v_ack_2 = v_ack_1.leak().verify(kp.public())?;

        assert_eq!(v_ack_1, v_ack_2);
        Ok(())
    }

    #[parameterized::parameterized(batch_size = {1, 2, 100 })]
    fn acknowledgement_should_verify_batch(batch_size: usize) -> anyhow::Result<()> {
        let mut verified_acks = Vec::with_capacity(100);
        let batch = (0..batch_size)
            .map(|_| {
                let kp = OffchainKeypair::random();
                let ack = VerifiedAcknowledgement::random(&kp);
                verified_acks.push(ack);
                (*kp.public(), ack.leak())
            })
            .collect::<Vec<_>>();

        let res = Acknowledgement::verify_batch(batch);

        assert_eq!(batch_size, res.len());
        assert!(res.iter().all(|r| r.is_ok()));
        assert_eq!(verified_acks, res.into_iter().map(|r| r.unwrap()).collect::<Vec<_>>());

        Ok(())
    }

    #[test]
    fn acknowledgement_should_return_failed_ack_in_the_batch_verification() -> anyhow::Result<()> {
        let mut verified_acks = Vec::with_capacity(100);
        let batch = (0..100)
            .map(|i| {
                let kp = OffchainKeypair::random();
                let ack = VerifiedAcknowledgement::random(&kp);
                if i == 50 {
                    let mut non_verified = ack.leak();
                    non_verified.0[3] = non_verified.0[3].not();
                    (*kp.public(), non_verified)
                } else {
                    verified_acks.push(ack);
                    (*kp.public(), ack.leak())
                }
            })
            .collect::<Vec<_>>();

        let res = Acknowledgement::verify_batch(batch);

        assert_eq!(100, res.len());
        assert!(res[50].is_err());
        assert!(res.iter().enumerate().all(|(i, r)| r.is_ok() || i == 50));
        assert_eq!(
            verified_acks,
            res.into_iter().filter_map(|r| r.ok()).collect::<Vec<_>>()
        );

        Ok(())
    }
}
