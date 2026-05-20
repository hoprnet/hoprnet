use std::collections::HashMap;

use hopr_protocol_pix::{
    EntryShareGenerator, ExitAcknowledgementShareProcessor, PixGroup, PixSpec, SsaCommitment, SsaGeneratorConfig,
    SsaId, SsaReconstructor, SsaReconstructorConfig, SsaShareGenerator, TaggedEncryptedPartialSsaShare,
};
use hopr_types::{
    crypto::prelude::{HalfKey, Keypair, OffchainKeypair, SimplePseudonym},
    crypto_random::Randomizable,
    internal::prelude::VerifiedAcknowledgement,
};
use rand::prelude::SliceRandom;
use vsss_rs::elliptic_curve::{group::GroupEncoding, ops::MulByGenerator};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct TestSpec;

impl PixSpec for TestSpec {
    type Cipher = hopr_types::crypto::primitives::ChaCha20;
    type Curve = k256::Secp256k1;
    type Digest = hopr_types::crypto::primitives::Sha3_256;
    type Pseudonym = SimplePseudonym;
}

#[test]
fn test_generator_reconstructor() -> anyhow::Result<()> {
    let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
        polynomials_per_ssa: 10,
        threshold: 10,
        surplus_shares: 0,
    });

    let pseudonym = SimplePseudonym::random();
    let peer = OffchainKeypair::random();

    let SsaCommitment {
        ssa_commitment,
        verifiers,
        ..
    } = generator.new_ssa_commitment(&pseudonym)?;

    // Use the already transposed verifiers
    let mut transposed = verifiers
        .into_iter()
        .map(|(k, v)| {
            (
                k,
                v.into_iter()
                    .map(|(pi, c)| (pi, c.0.to_bytes()))
                    .collect::<HashMap<_, _>>(),
            )
        })
        .collect::<HashMap<_, _>>();

    let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig {
        polys_per_ssa: 10,
        poly_threshold: 10,
        ..Default::default()
    });

    let ssa_id = SsaId::new(pseudonym, 1.try_into()?);

    // In the transposed form, remove the first coefficient commitments of all polynomials
    let mut first_coeffs = transposed.remove(&0).unwrap();

    // Remove the first polynomial completely
    let remainder = first_coeffs.remove(&0).unwrap();

    // Insert all first coefficient commitments, except the first coefficient commitment of the first polynomial
    let res = reconstructor.insert_coefficient_commitments(ssa_id, 0, first_coeffs.into_iter())?;
    assert_eq!(ssa_id, res.ssa_id);
    assert!(res.is_first_encountered);
    assert!(res.ssa_commitment.is_none());
    assert!(!res.is_verifiable);

    // Now add the first coefficient commitment of the first polynomial
    let res = reconstructor.insert_coefficient_commitments(ssa_id, 0, HashMap::from([(0, remainder)]).into_iter())?;
    assert_eq!(ssa_id, res.ssa_id);
    assert!(!res.is_first_encountered);
    assert_eq!(Some(ssa_commitment), res.ssa_commitment);
    assert!(!res.is_verifiable);

    // Add all the remaining coefficient commitments for all polynomials except one
    let remainder = transposed.remove(&5).unwrap();
    for (coeff_index, poly_coeff_commitments) in transposed {
        let res =
            reconstructor.insert_coefficient_commitments(ssa_id, coeff_index, poly_coeff_commitments.into_iter())?;
        assert_eq!(ssa_id, res.ssa_id);
        assert!(!res.is_first_encountered);
        assert_eq!(Some(ssa_commitment), res.ssa_commitment);
        assert!(!res.is_verifiable);
    }

    // Now the SSA should be fully committed
    let res = reconstructor.insert_coefficient_commitments(ssa_id, 5, remainder.into_iter())?;
    assert_eq!(ssa_id, res.ssa_id);
    assert!(!res.is_first_encountered);
    assert_eq!(Some(ssa_commitment), res.ssa_commitment);
    assert!(res.is_verifiable);

    let mut acks = Vec::new();

    while let Some((msg, share)) = {
        let msg = hopr_types::crypto_random::random_bytes::<20>();
        generator.next_share(&pseudonym, &msg).map(|v| v.map(|u| (msg, u)))
    }? {
        let ack = HalfKey::random();
        let ack_challenge = ack.to_challenge()?;
        let enc_share = share.share.encrypt(&share.id, &ack)?;

        reconstructor.insert_encrypted_share(
            peer.public(),
            ack_challenge,
            TaggedEncryptedPartialSsaShare::new(pseudonym, &msg, enc_share)?,
        )?;
        acks.push(VerifiedAcknowledgement::new(ack, &peer).leak());
    }

    acks.shuffle(&mut rand::rng());

    let one_ack = acks.remove(0);

    assert!(reconstructor.acknowledge_shares(*peer.public(), acks)?.is_empty());

    let res = reconstructor.acknowledge_shares(*peer.public(), vec![one_ack])?;
    assert!(!res.is_empty());

    assert_eq!(res[0].ssa_id, ssa_id);
    assert_eq!(PixGroup::<TestSpec>::mul_by_generator(&res[0].ssa), ssa_commitment);

    Ok(())
}
