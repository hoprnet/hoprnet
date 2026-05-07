mod events;
mod utils;

pub use events::ReconstructorEvent;
use hopr_types::{crypto::prelude::HalfKeyChallenge, internal::prelude::VerifiedAcknowledgement};
use utils::{AwaitingPartialShare, CommitmentResult, SsaBuilder, SsaCommitmentBuilder, SsaPartBuilder};

use crate::{
    CoefficientIndex, DEFAULT_POLY_THRESHOLD, DEFAULT_POLYS_PER_SSA, PixGroupRepr, PixScalar, PixSpec, PolynomialIndex,
    SsaPolynomialId, errors,
    types::{EncryptedPartialSsaShare, SsaId},
};

/// Configuration for the SSA reconstructor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, smart_default::SmartDefault, validator::Validate)]
pub struct SsaReconstructorConfig {
    /// Number of polynomials needed to reconstruct a single SSA.
    ///
    /// Default is [`DEFAULT_POLYS_PER_SSA`], must be between 2 and 65 535.
    #[default(DEFAULT_POLYS_PER_SSA)]
    #[validate(range(min = 2, max = 65535))]
    pub polys_per_ssa: usize,
    /// Number of shares needed to reconstruct a single polynomial.
    ///
    /// Default is [`DEFAULT_POLY_THRESHOLD`], must be between 2 and 1000.
    #[default(DEFAULT_POLY_THRESHOLD)]
    #[validate(range(min = 2, max = 1000))]
    pub poly_threshold: usize,
    /// Maximum time an SSA can be incomplete before it is discarded.
    ///
    /// Default is 10 minutes.
    #[default(std::time::Duration::from_secs(600))]
    pub incomplete_ssa_lifetime: std::time::Duration,
    /// Time until the complete commitment to an SSA must be received.
    ///
    /// Default is 2 minutes.
    #[default(std::time::Duration::from_secs(120))]
    pub incomplete_commitment_lifetime: std::time::Duration,
    /// Maximum time a verifier can be unused before it is discarded.
    ///
    /// Default is 30 minutes.
    #[default(std::time::Duration::from_secs(1800))]
    pub unused_verifier_lifetime: std::time::Duration,
    /// Maximum number of awaited acknowledgements to extract a single share.
    ///
    /// Default is 10 000 000, must be at least 10 000.
    #[default(10_000_000)]
    #[validate(range(min = 10000))]
    pub max_awaiting_acks: usize,
    /// Maximum time an acknowledgement can be awaited before it is discarded.
    ///
    /// Default is 30 seconds.
    #[default(std::time::Duration::from_secs(30))]
    pub max_ack_await_time: std::time::Duration,
}

/// Allows server-side reconstruction of SSAs.
///
/// There are 3 inputs that reconstructor is dependent on (in order):
/// 1. SSA commitments from the Client (delivered via
///    [`add_client_commitment_data`](SsaReconstructor::add_client_commitment_data))
/// 2. Extraction of pending encrypted shares (added via [`add_pending_share`](SsaReconstructor::add_pending_share)
/// 3. Decryption of pending encrypted shares via [`VerifiedAcknowledgement`] (via
///    [`new_acknowledgement`](SsaReconstructor::new_acknowledgement))
///
/// The `SsaReconstructor` is emitting [`ReconstructorEvent`] during the above process.
/// The most important event is [`ReconstructorEvent::SsaRecovered`], which is emitted when an SSA is fully
/// reconstructed.
///
/// It is able to track SSA for multiple different pseudonyms (Sessions).
pub struct SsaReconstructor<S: PixSpec> {
    channel: (
        async_broadcast::Sender<ReconstructorEvent<S>>,
        async_broadcast::InactiveReceiver<ReconstructorEvent<S>>,
    ),
    commitment_builder: moka::sync::Cache<SsaId<S>, std::sync::Arc<parking_lot::Mutex<SsaCommitmentBuilder<S>>>>,
    ssa_builders: moka::sync::Cache<SsaId<S>, std::sync::Arc<parking_lot::Mutex<SsaBuilder<S>>>>,
    ssa_verifiers: moka::sync::Cache<SsaPolynomialId<S>, std::sync::Arc<parking_lot::Mutex<SsaPartBuilder<S>>>>,
    awaiting_acks: moka::sync::Cache<HalfKeyChallenge, AwaitingPartialShare<S>>,
    cfg: SsaReconstructorConfig,
}

impl<S: PixSpec + 'static> SsaReconstructor<S> {
    pub fn new(cfg: SsaReconstructorConfig) -> Self {
        let (mut event_send, event_recv) = async_broadcast::broadcast(1024);
        event_send.set_await_active(false);
        event_send.set_overflow(true);
        Self {
            channel: (event_send, event_recv.deactivate()),
            commitment_builder: moka::sync::CacheBuilder::new(10)
                .time_to_idle(cfg.incomplete_commitment_lifetime)
                .build(),
            ssa_builders: moka::sync::CacheBuilder::new(3 * cfg.polys_per_ssa as u64)
                .time_to_idle(cfg.incomplete_ssa_lifetime)
                .build(),
            ssa_verifiers: moka::sync::CacheBuilder::new(3 * cfg.polys_per_ssa as u64)
                .time_to_idle(cfg.unused_verifier_lifetime)
                .build(),
            awaiting_acks: moka::sync::CacheBuilder::new(cfg.max_awaiting_acks as u64)
                .time_to_live(cfg.max_ack_await_time)
                .build(),
            cfg,
        }
    }

    /// Adds the commitment data that the client feeds to the reconstructor.
    ///
    /// Each "data packet" should contain an `id` of the corresponding SSA, `coeff_index` is
    /// the polynomial coefficient index that is common to all the polynomial coefficient commitments included in
    /// `polynomial_coeff_commitments`. In other words, the `polynomial_coeff_commitments` contains commitments to
    /// the same polynomial coefficients across multiple polynomials.
    pub fn add_client_commitment_data(
        &self,
        id: SsaId<S>,
        coeff_index: CoefficientIndex,
        polynomial_coeff_commitments: std::collections::HashMap<PolynomialIndex, PixGroupRepr<S>>,
    ) -> errors::Result<()> {
        let maybe_complete_ssa_commitment = self
            .commitment_builder
            .get_with(id, || {
                if let Err(error) = self.channel.0.try_broadcast(ReconstructorEvent::NewSsa(id)) {
                    tracing::error!(%id, %error, "failed to broadcast new ssa");
                }
                std::sync::Arc::new(parking_lot::Mutex::new(SsaCommitmentBuilder::new(
                    id,
                    self.cfg.poly_threshold,
                    self.cfg.polys_per_ssa,
                )))
            })
            .lock()
            .add_transposed(coeff_index, polynomial_coeff_commitments)?;

        match maybe_complete_ssa_commitment {
            CommitmentResult::NotReady => {
                tracing::trace!(%id, "ssa commitment not yet complete, waiting for more data");
            }
            CommitmentResult::SsaCommitment(commitment) => {
                if let Err(error) = self
                    .channel
                    .0
                    .try_broadcast(ReconstructorEvent::SsaCommitmentKnown(id, commitment))
                {
                    tracing::error!(%id, %error, "failed to broadcast new ssa commitment");
                }
            }
            CommitmentResult::Completed(ssa_builder, ssa_reconstructors, did_commit_already) => {
                let commitment = ssa_builder.commitment;
                self.ssa_builders
                    .insert(id, std::sync::Arc::new(parking_lot::Mutex::new(ssa_builder)));

                for ssa_reconstructor in ssa_reconstructors {
                    self.ssa_verifiers.insert(
                        ssa_reconstructor.verifier.spi,
                        std::sync::Arc::new(parking_lot::Mutex::new(ssa_reconstructor)),
                    );
                }

                // Send the commitment event if it has not already happened before.
                // This is to cover situations when everything arrives at once.
                if !did_commit_already
                    && let Err(error) = self
                        .channel
                        .0
                        .try_broadcast(ReconstructorEvent::SsaCommitmentKnown(id, commitment))
                {
                    tracing::error!(%id, %error, "failed to broadcast new ssa commitment");
                }

                if let Err(error) = self
                    .channel
                    .0
                    .try_broadcast(ReconstructorEvent::SsaFullyCommitted(id, commitment))
                {
                    tracing::error!(%id, %error, "failed to broadcast new ssa commitment");
                }
            }
        }

        Ok(())
    }

    /// Adds an encrypted partial SSA share awaiting acknowledgement to be decrypted.
    ///
    /// The `challenge` is the acknowledgement challenge that must correspond to the
    /// acknowledgement that will be awaited.
    pub fn add_pending_share(
        &self,
        challenge: HalfKeyChallenge,
        spi: SsaPolynomialId<S>,
        msg: impl AsRef<[u8]>,
        enc: EncryptedPartialSsaShare<S>,
    ) -> errors::Result<()> {
        self.awaiting_acks.insert(
            challenge,
            AwaitingPartialShare {
                spi,
                msg: msg.as_ref().into(),
                enc_share: enc,
            },
        );
        Ok(())
    }

    /// Checks if the incoming verified acknowledgement is associated with a pending encrypted share,
    /// and if so, decrypts and adds the share to the corresponding reconstructor.
    ///
    /// If the acknowledgement has completed an SSA recovery, its [`SsaId`] and corresponding [`PixScalar`] are
    /// returned. This is the same data contained in the raised [`ReconstructorEvent::SsaRecovered`] event. The user
    /// can freely ignore the returned success value if they are actively processing the event stream.
    pub fn new_acknowledgement(
        &self,
        ack: VerifiedAcknowledgement,
    ) -> errors::Result<Option<(SsaId<S>, PixScalar<S>)>> {
        let challenge = ack.ack_key_share().to_challenge()?;
        let Some(share) = self.awaiting_acks.remove(&challenge) else {
            tracing::trace!(?challenge, "received ack for unknown share");
            return Ok(None);
        };

        let reconstructor = self
            .ssa_verifiers
            .get(&share.spi)
            .ok_or(errors::PixError::MissingVerifier)?;

        let partial_share = share.enc_share.decrypt(&share.spi, ack.ack_key_share())?;
        let Some(ssa_part) = reconstructor.lock().add_share(share.spi, share.msg, partial_share)? else {
            tracing::trace!(spi = %share.spi, "ssa part not yet complete, waiting for more shares");
            return Ok(None);
        };

        tracing::trace!(spi = %share.spi, "ssa part complete");

        let builder = self
            .ssa_builders
            .get(share.spi.as_ref())
            .ok_or(errors::PixError::MissingSsaCommitment)?;
        let Some(ssa) = builder.lock().add_recovered_ssa_part(ssa_part)? else {
            tracing::trace!(spi = %share.spi, "ssa not yet complete, waiting for more ssa parts");
            return Ok(None);
        };

        let id = *share.spi.as_ref();
        tracing::info!(%id, "ssa recovered");
        if let Err(error) = self.channel.0.try_broadcast(ReconstructorEvent::SsaRecovered(id, ssa)) {
            tracing::error!(%error, "failed to broadcast new ssa");
        }

        Ok(Some((id, ssa)))
    }

    /// Returns the output stream of [`ReconstructorEvents`](ReconstructorEvent).
    pub fn event_stream(&self) -> impl futures::Stream<Item = ReconstructorEvent<S>> + use<S> {
        self.channel.1.activate_cloned()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use hopr_types::{crypto::prelude::*, crypto_random::Randomizable};

    use super::*;
    use crate::{PartialSsaShare, tests::TestSpec};

    #[test]
    fn reconstructor_invalid_commitment_inputs() {
        let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig {
            polys_per_ssa: 2,
            poly_threshold: 2,
            ..Default::default()
        });

        let ssa_id = SsaId::new(SimplePseudonym::random(), 1);

        // 1. Invalid coefficient index (>= threshold)
        let result = reconstructor.add_client_commitment_data(
            ssa_id,
            2, // threshold is 2, so 2 is invalid
            HashMap::new(),
        );
        assert!(matches!(result, Err(errors::PixError::InvalidInput)));

        // 2. Invalid polynomial index (>= polys_per_ssa)
        let mut invalid_poly_map = HashMap::new();
        invalid_poly_map.insert(2 as PolynomialIndex, PixGroupRepr::<TestSpec>::default());
        let result = reconstructor.add_client_commitment_data(ssa_id, 0, invalid_poly_map);
        assert!(matches!(result, Err(errors::PixError::InvalidInput)));
    }

    #[test]
    fn reconstructor_duplicate_commitments() -> anyhow::Result<()> {
        let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig {
            polys_per_ssa: 2,
            poly_threshold: 2,
            ..Default::default()
        });

        let ssa_id = SsaId::new(SimplePseudonym::random(), 1);

        // Fill all commitments
        for coeff in 0..2 {
            let mut poly_map = HashMap::new();
            for poly in 0..2 {
                poly_map.insert(poly as PolynomialIndex, PixGroupRepr::<TestSpec>::default());
            }
            reconstructor.add_client_commitment_data(ssa_id, coeff as CoefficientIndex, poly_map)?;
        }

        // Now adding more should fail with DuplicateCommitment
        let result = reconstructor.add_client_commitment_data(ssa_id, 0, HashMap::new());
        assert!(matches!(result, Err(errors::PixError::DuplicateCommitment)));

        Ok(())
    }

    #[test]
    fn reconstructor_missing_verifier() -> anyhow::Result<()> {
        let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig { ..Default::default() });

        let ack_key = HalfKey::random();
        let challenge = ack_key.to_challenge()?;
        let relay_pk = OffchainKeypair::random();
        let ack = VerifiedAcknowledgement::new(ack_key, &relay_pk);

        // Add a pending share but NO commitment (so no verifier is created)
        let ssa_id = SsaId::new(SimplePseudonym::random(), 1);
        let spi = SsaPolynomialId::new(ssa_id, 0);

        // We need a valid-looking encrypted share even if it's junk.
        // EncryptedPartialSsaShare is basically a wrapper around bytes.
        let enc_share = PartialSsaShare::default().encrypt(&spi, &ack_key)?;

        reconstructor.add_pending_share(challenge, spi, b"msg", enc_share)?;

        // This should fail with MissingVerifier
        let result = reconstructor.new_acknowledgement(ack);
        assert!(matches!(result, Err(errors::PixError::MissingVerifier)));

        Ok(())
    }

    #[test]
    fn reconstructor_invalid_acknowledgement() -> anyhow::Result<()> {
        let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig { ..Default::default() });

        let ack_key = HalfKey::random();
        let relay_pk = OffchainKeypair::random();
        let ack = VerifiedAcknowledgement::new(ack_key, &relay_pk);

        // This should return None for unknown challenge
        assert!(reconstructor.new_acknowledgement(ack)?.is_none());

        Ok(())
    }

}
