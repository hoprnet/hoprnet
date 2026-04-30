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

pub struct SsaReconstructor<S: PixSpec> {
    channel: (
        async_broadcast::Sender<PixScalar<S>>,
        async_broadcast::InactiveReceiver<PixScalar<S>>,
    ),
    ssa_builders: moka::sync::Cache<SsaIndex, std::sync::Arc<parking_lot::Mutex<SsaBuilder<S>>>>,
    ssa_verifiers:
        moka::sync::Cache<SsaPolynomialIndex<S>, std::sync::Arc<parking_lot::Mutex<SsaPartReconstructor<S>>>>,
    awaiting_acks: moka::sync::Cache<HalfKeyChallenge, AwaitingPartialShare<S>>,
}

impl<S: PixSpec + 'static> SsaReconstructor<S> {
    // TODO: replace this with add_verifier_part
    pub fn add_ssa_commitment(&self, ssa_index: SsaIndex, num_parts: NonZeroUsize, ssa_commitment: PixGroup<S>) {
        self.ssa_builders.insert(
            ssa_index,
            std::sync::Arc::new(parking_lot::Mutex::new(SsaBuilder::new(ssa_commitment, num_parts))),
        );
    }

    pub fn add_verifier(&self, verifier: PartialSsaShareVerifier<S>) {
        self.ssa_verifiers.insert(
            verifier.spi,
            std::sync::Arc::new(parking_lot::Mutex::new(SsaPartReconstructor::new(verifier))),
        );
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

        if let Err(error) = self.channel.0.try_broadcast(ssa) {
            tracing::error!(%error, "failed to broadcast new ssa");
        }

        Ok(())
    }

    pub fn ssa_stream(&self) -> impl futures::Stream<Item = PixScalar<S>> {
        self.channel.1.activate_cloned()
    }
}
