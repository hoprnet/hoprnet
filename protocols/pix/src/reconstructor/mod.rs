mod events;

use std::num::NonZeroUsize;

use hopr_types::{crypto::prelude::HalfKeyChallenge, internal::prelude::VerifiedAcknowledgement};
use vsss_rs::{
    ReadableShareSet,
    elliptic_curve::{CurveArithmetic, Group, group::Curve},
};

use crate::{
    CompletedShare, PartialSsaShare, PartialSsaShareVerifier, PixGroup, PixScalar, PixSpec, SsaPolynomialIndex,
    complete_share, errors,
    types::{EncryptedPartialSsaShare, SsaIndex},
};
use crate::reconstructor::events::ReconstructorEvent;

pub type AffineElement<S> = <<S as PixSpec>::Curve as CurveArithmetic>::AffinePoint;

struct SsaBuilder<S: PixSpec> {
    commitment: AffineElement<S>,
    num_parts: usize,
    builder: PixScalar<S>,
}

impl<S: PixSpec> SsaBuilder<S> {
    pub fn new(commitment: PixGroup<S>, num_parts: NonZeroUsize) -> Self {
        Self {
            commitment: commitment.to_affine(),
            num_parts: num_parts.get(),
            builder: PixScalar::<S>::default(),
        }
    }

    pub fn add_ssa_part(&mut self, sub_secret: PixScalar<S>) -> errors::Result<Option<PixScalar<S>>> {
        if let Some(n) = self.num_parts.checked_sub(1) {
            self.builder += sub_secret;
            if n > 0 {
                // SSA private scalar is not yet complete
                return Ok(None);
            }
        }

        if self.commitment == (PixGroup::<S>::generator() * self.builder).to_affine() {
            Ok(Some(self.builder))
        } else {
            Err(errors::PixError::InvalidSsa)
        }
    }
}

struct SsaPartReconstructor<S: PixSpec> {
    verifier: PartialSsaShareVerifier<S>,
    shares: Vec<CompletedShare<S>>,
}

impl<S: PixSpec> SsaPartReconstructor<S> {
    pub fn new(verifier: PartialSsaShareVerifier<S>) -> Self {
        Self {
            verifier,
            shares: Vec::new(),
        }
    }

    pub fn add_share(
        &mut self,
        spi: SsaPolynomialIndex<S>,
        msg: impl AsRef<[u8]>,
        share: PartialSsaShare<S>,
    ) -> errors::Result<Option<PixScalar<S>>> {
        let share = complete_share(spi, msg, &share)?;

        self.verifier.verify_complete_share(&share)?;
        self.shares.push(share);
        if self.shares.len() >= self.verifier.min_shares() {
            Ok(Some(self.shares.combine()?.0))
        } else {
            Ok(None)
        }
    }
}

#[derive(PartialEq, Eq)]
struct AwaitingPartialShare<S: PixSpec> {
    spi: SsaPolynomialIndex<S>,
    msg: Box<[u8]>,
    enc_share: EncryptedPartialSsaShare<S>,
}

impl<S: PixSpec> Clone for AwaitingPartialShare<S> {
    fn clone(&self) -> Self {
        Self {
            spi: self.spi,
            msg: self.msg.clone(),
            enc_share: self.enc_share.clone(),
        }
    }
}

/// Configuration for the SSA reconstructor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, smart_default::SmartDefault, validator::Validate)]
pub struct SsaReconstructorConfig {
    /// Number of polynomials needed to reconstruct a single SSA.
    ///
    /// Default is 1000, must be between 2 and 1 000 000.
    #[default(1000)]
    #[validator(range(min = 2, max = 1_000_000))]
    pub polys_per_ssa: usize,
    /// Number of shares needed to reconstruct a single polynomial.
    ///
    /// Default is 100, must be between 2 and 1000.
    #[default(100)]
    #[validator(range(min = 2, max = 1000))]
    pub poly_threshold: usize,
    /// Maximum time an SSA can be incomplete before it is discarded.
    ///
    /// Default is 10 minutes.
    #[default(std::time::Duration::from_secs(600))]
    pub incomplete_ssa_lifetime: std::time::Duration,
    /// Maximum time a verifier can be unused before it is discarded.
    ///
    /// Default is 30 minutes.
    #[default(std::time::Duration::from_secs(1800))]
    pub unused_verifier_lifetime: std::time::Duration,
    /// Maximum number of awaited acknowledgements to extract a single share.
    ///
    /// Default is 10 000 000, must be at least 10 000.
    #[default(10_000_000)]
    #[validator(range(min = 10000))]
    pub max_awaiting_acks: usize,
    /// Maximum time an acknowledgement can be awaited before it is discarded.
    ///
    /// Default is 30 seconds.
    #[default(std::time::Duration::from_secs(30))]
    pub max_ack_await_time: std::time::Duration,
}

pub struct SsaReconstructor<S: PixSpec> {
    channel: (
        async_broadcast::Sender<ReconstructorEvent<S>>,
        async_broadcast::InactiveReceiver<ReconstructorEvent<S>>,
    ),
    ssa_builders: moka::sync::Cache<SsaIndex, std::sync::Arc<parking_lot::Mutex<SsaBuilder<S>>>>,
    ssa_verifiers:
        moka::sync::Cache<SsaPolynomialIndex<S>, std::sync::Arc<parking_lot::Mutex<SsaPartReconstructor<S>>>>,
    awaiting_acks: moka::sync::Cache<HalfKeyChallenge, AwaitingPartialShare<S>>,
    pseudonym: S::Pseudonym,
    cfg: SsaReconstructorConfig,
}

impl<S: PixSpec + 'static> SsaReconstructor<S> {

    pub fn new(pseudonym: S::Pseudonym, cfg: SsaReconstructorConfig) -> Self {
        let (mut event_send, event_recv) = async_broadcast::broadcast(1024);
        event_send.set_await_active(false);
        event_send.set_overflow(true);
        Self {
            channel: (event_send, event_recv.deactivate()),
            ssa_builders: moka::sync::CacheBuilder::new(3 * cfg.polys_per_ssa as u64)
                .time_to_idle(cfg.incomplete_ssa_lifetime)
                .build(),
            ssa_verifiers: moka::sync::CacheBuilder::new(3 * cfg.polys_per_ssa as u64)
                .time_to_idle(cfg.unused_verifier_lifetime)
                .build(),
            awaiting_acks: moka::sync::CacheBuilder::new(cfg.max_awaiting_acks as u64)
                .time_to_live(cfg.max_ack_await_time)
                .build(),
            pseudonym,
            cfg
        }
    }

    pub fn add_client_commitment_data(ssa_index: SsaIndex) -> errors::Result<()> {
        todo!()
    }

    pub fn add_share(
        &self,
        challenge: HalfKeyChallenge,
        spi: SsaPolynomialIndex<S>,
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

    pub fn new_acknowledgement(&self, ack: VerifiedAcknowledgement) -> errors::Result<()> {
        let challenge = ack.ack_key_share().to_challenge()?;
        let Some(share) = self.awaiting_acks.remove(&challenge) else {
            tracing::trace!(?challenge, "received ack for unknown share");
            return Ok(());
        };

        let reconstructor = self
            .ssa_verifiers
            .get(&share.spi)
            .ok_or(errors::PixError::MissingVerifier)?;

        let partial_share = share.enc_share.decrypt(&share.spi, ack.ack_key_share())?;
        let Some(ssa_part) = reconstructor.lock().add_share(share.spi, share.msg, partial_share)? else {
            tracing::trace!(spi = %share.spi, "ssa part not yet complete, waiting for more shares");
            return Ok(());
        };

        let builder = self
            .ssa_builders
            .get(&share.spi.ssa_index())
            .ok_or(errors::PixError::MissingSsaCommitment)?;
        let Some(ssa) = builder.lock().add_ssa_part(ssa_part)? else {
            tracing::trace!(spi = %share.spi, "ssa not yet complete, waiting for more ssa parts");
            return Ok(());
        };

        if let Err(error) = self.channel.0
            .try_broadcast(ReconstructorEvent::SsaRecovered(share.spi.ssa_index(), ssa)) {
            tracing::error!(%error, "failed to broadcast new ssa");
        }

        Ok(())
    }

    pub fn event_stream(&self) -> impl futures::Stream<Item = ReconstructorEvent<S>> {
        self.channel.1.activate_cloned()
    }
}
