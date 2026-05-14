use std::collections::HashMap;

use futures::{StreamExt, pin_mut};
use futures_time::future::FutureExt;
use hopr_protocol_pix::{
    CoefficientIndex, PixGroup, PixGroupRepr, PixSpec, PolynomialIndex, ReconstructorEvent, SsaGeneratorConfig, SsaId,
    SsaReconstructor, SsaReconstructorConfig, SsaShareGenerator, transpose_commitments,
};
use hopr_types::{
    crypto::prelude::{HalfKey, Keypair, OffchainKeypair, SimplePseudonym},
    crypto_random::Randomizable,
    internal::prelude::VerifiedAcknowledgement,
};
use rand::prelude::SliceRandom;
use vsss_rs::elliptic_curve::ops::MulByGenerator;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct TestSpec;

impl PixSpec for TestSpec {
    type Cipher = hopr_types::crypto::primitives::ChaCha20;
    type Curve = k256::Secp256k1;
    type Digest = hopr_types::crypto::primitives::Sha3_256;
    type Pseudonym = SimplePseudonym;
}

#[test_log::test(tokio::test)]
async fn test_generator_reconstructor() -> anyhow::Result<()> {
    let timeout = futures_time::time::Duration::from_secs(1);
    let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
        polynomials_per_ssa: 10,
        threshold: 10,
        surplus_shares: 0,
    });

    let pseudonym = SimplePseudonym::random();

    let (ssa_client, commitments) = generator.new_ssa_commitment(&pseudonym)?;

    // Transpose the commitments so they have the on-wire structure
    let mut transposed = transpose_commitments(commitments);

    let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig {
        polys_per_ssa: 10,
        poly_threshold: 10,
        ..Default::default()
    });

    let event_stream = reconstructor.event_stream();
    pin_mut!(event_stream);

    let ssa_id = SsaId::new(pseudonym, 1);

    let mut first_coeffs = transposed.remove(&0).unwrap();

    // Remove the first polynomial to test the NewSsa event independently of the rest
    let remainder = first_coeffs.remove(&0).unwrap();

    reconstructor.add_client_commitment_data(ssa_id, 0, first_coeffs)?;
    assert_eq!(
        event_stream.next().timeout(timeout).await?,
        Some(ReconstructorEvent::NewSsa(ssa_id))
    );

    reconstructor.add_client_commitment_data(ssa_id, 0, HashMap::from([(0, remainder)]))?;
    assert!(
        matches!(event_stream.next().timeout(timeout).await?, Some(ReconstructorEvent::SsaCommitmentKnown(sid, _)) if sid == ssa_id)
    );

    for (coeff_index, poly_coeff_commitments) in transposed {
        reconstructor.add_client_commitment_data(ssa_id, coeff_index, poly_coeff_commitments)?;
    }

    assert!(
        matches!(event_stream.next().timeout(timeout).await?, Some(ReconstructorEvent::SsaFullyCommitted(sid, _)) if sid == ssa_id)
    );

    let mut acks = Vec::new();

    while let Some((msg, share)) = {
        let msg = hopr_types::crypto_random::random_bytes::<20>();
        generator.next_share(&pseudonym, &msg).map(|v| v.map(|u| (msg, u)))
    }? {
        let ack = HalfKey::random();
        let ack_challenge = ack.to_challenge()?;
        let enc_share = share.1.encrypt(&share.0, &ack)?;

        reconstructor.add_pending_share(ack_challenge, share.0.pseudonym(), &msg, enc_share)?;
        acks.push((ack, ack_challenge));
    }

    acks.shuffle(&mut rand::rng());

    for (ack, ack_challenge) in acks.iter().take(acks.len() - 1) {
        assert_eq!(None, reconstructor.new_acknowledgement(ack, ack_challenge)?);
    }

    let (last_ack, last_ack_challenge) = acks.last().unwrap();
    let recovered_1 = reconstructor.new_acknowledgement(last_ack, last_ack_challenge)?;
    let recovered_2 = event_stream.next().timeout(timeout).await?;

    match recovered_2 {
        Some(ReconstructorEvent::SsaRecovered(sid, scalar)) => {
            assert_eq!(sid, ssa_id);
            assert_eq!(PixGroup::<TestSpec>::mul_by_generator(&scalar), ssa_client);
            assert_eq!(recovered_1, Some((sid, scalar)));
        }
        _ => panic!("expected ssa recovered event"),
    }

    Ok(())
}
