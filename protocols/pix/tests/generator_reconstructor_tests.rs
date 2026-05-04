use std::collections::HashMap;

use futures::{StreamExt, pin_mut};
use hopr_protocol_pix::{
    CoefficientIndex, PixGroup, PixGroupRepr, PixSpec, PolynomialIndex, ReconstructorEvent, SsaId, SsaReconstructor,
    SsaShareGenerator,
};
use hopr_types::{
    crypto::prelude::{HalfKey, Keypair, OffchainKeypair, SimplePseudonym},
    crypto_random::Randomizable,
    internal::prelude::VerifiedAcknowledgement,
};
use vsss_rs::elliptic_curve::ops::MulByGenerator;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct TestSpec;

impl PixSpec for TestSpec {
    type Cipher = hopr_types::crypto::primitives::ChaCha20;
    type Curve = k256::Secp256k1;
    type Digest = hopr_types::crypto::primitives::Sha3_256;
    type Pseudonym = SimplePseudonym;
}

#[tokio::test]
async fn test_generator_reconstructor() -> anyhow::Result<()> {
    let generator = SsaShareGenerator::<TestSpec>::new(Default::default());

    let pseudonym = SimplePseudonym::random();

    let (ssa_client, commitments) = generator.new_ssa_commitment(&pseudonym)?;

    // Transpose the commitments so they have the on-wire structure
    let mut transposed = HashMap::<CoefficientIndex, HashMap<PolynomialIndex, PixGroupRepr<TestSpec>>>::new();
    commitments
        .into_iter()
        .map(|c| c.into_serializable_commitments())
        .for_each(|(spi, committed_polynomial)| {
            for (coeff_id, coeff) in committed_polynomial.into_iter().enumerate() {
                transposed
                    .entry(coeff_id as CoefficientIndex)
                    .or_default()
                    .insert(spi.poly_index(), coeff);
            }
        });

    let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());

    let event_stream = reconstructor.event_stream();
    pin_mut!(event_stream);

    let ssa_id = SsaId::new(pseudonym, 1);

    reconstructor.add_client_commitment_data(ssa_id, 0, transposed.remove(&0).unwrap())?;

    // TODO: split this
    assert_eq!(Some(ReconstructorEvent::NewSsa(ssa_id)), event_stream.next().await);
    assert!(matches!(event_stream.next().await, Some(ReconstructorEvent::SsaCommitmentKnown(sid, _)) if sid == ssa_id));

    for (coeff_index, poly_coeff_commitments) in transposed.into_iter().skip(1) {
        reconstructor.add_client_commitment_data(ssa_id, coeff_index, poly_coeff_commitments)?;
    }

    assert!(matches!(event_stream.next().await, Some(ReconstructorEvent::SsaFullyCommitted(sid, _)) if sid == ssa_id));

    let mut acks = Vec::new();
    let relay_pk = OffchainKeypair::random();
    let msg = b"test message";

    while let Some(share) = generator.next_share(&pseudonym, msg)? {
        let ack = HalfKey::random();
        let enc_share = share.1.encrypt(&share.0, &ack)?;

        reconstructor.add_pending_share(ack.to_challenge()?, share.0, msg, enc_share)?;
        acks.push(VerifiedAcknowledgement::new(ack, &relay_pk));
    }

    for ack in acks.iter().take(acks.len() - 1) {
        assert_eq!(None, reconstructor.new_acknowledgement(*ack)?);
    }

    let recovered_1 = reconstructor.new_acknowledgement(acks.last().unwrap().clone())?;
    let recovered_2 = event_stream.next().await;

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
