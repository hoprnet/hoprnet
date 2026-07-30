mod common;

use std::collections::HashMap;

use common::TestSpec;
use hopr_protocol_pix::{
    EntryShareGenerator, ExitAcknowledgementShareProcessor, PixSpec, ShareResolution, SsaCommitment,
    SsaGeneratorConfig, SsaId, SsaIndex, SsaReconstructor, SsaReconstructorConfig, SsaShareGenerator,
    TaggedEncryptedPartialSsaShare,
};
use hopr_types::{
    crypto::prelude::{HalfKey, Keypair, OffchainKeypair, SimplePseudonym},
    crypto_random::Randomizable,
    internal::prelude::VerifiedAcknowledgement,
};
use rand::prelude::SliceRandom;

#[test]
fn test_generator_reconstructor_stepwise() -> anyhow::Result<()> {
    let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
        polynomials_per_ssa: 10,
        threshold: 10,
        surplus_shares: 0,
    });

    let pseudonym = SimplePseudonym::random();
    let peer = OffchainKeypair::random();

    let SsaCommitment {
        ssa_commitment: client_commitment,
        commitment_proof,
        verifiers,
        ..
    } = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;

    // Use the already transposed verifiers
    let mut transposed = verifiers
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().collect::<HashMap<_, _>>()))
        .collect::<HashMap<_, _>>();

    let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig {
        early_recovery_threshold: 1.0,
        ..Default::default()
    });

    let ssa_id = SsaId::new(pseudonym, 1.try_into()?);

    let server_commitment = reconstructor.new_exit_commitment(ssa_id, 10, 10)?;

    let full_ssa_deposit_address = TestSpec::group_to_deposit_address(client_commitment + server_commitment)
        .ok_or(anyhow::anyhow!("Failed to convert to address"))?;

    // The whole commitment is the constant-term row; there is nothing else on the wire.
    assert_eq!(1, transposed.len(), "PIX commits to the constant term and nothing else");
    let mut constant_terms = transposed.remove(&0).unwrap();

    // Hold back the first polynomial so the set can be completed in two steps.
    let remainder = constant_terms.remove(&0).unwrap();

    // Insert every constant term except the first polynomial's. Every constant-term message
    // carries the proof of knowledge, as it does on the wire.
    let res =
        reconstructor.insert_coefficient_commitments(ssa_id, 0, Some(commitment_proof), constant_terms.into_iter())?;
    assert_eq!(ssa_id, res.ssa_id);
    assert!(res.is_first_encountered);
    assert!(res.ssa_deposit_address.is_none());
    assert!(!res.is_verifiable);

    // The last constant term closes the set: the deposit address becomes known and, in the same
    // call, every polynomial becomes reconstructible.
    let res = reconstructor.insert_coefficient_commitments(
        ssa_id,
        0,
        Some(commitment_proof),
        HashMap::from([(0, remainder)]).into_iter(),
    )?;
    assert_eq!(ssa_id, res.ssa_id);
    assert!(!res.is_first_encountered);
    assert_eq!(Some(full_ssa_deposit_address), res.ssa_deposit_address);
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

    assert!(matches!(&res[0], ShareResolution::RecoveredSsa(res)
        if res.ssa_id == ssa_id && <TestSpec as PixSpec>::DepositAddress::from(&res.ssa) == full_ssa_deposit_address
    ));

    Ok(())
}

#[test]
fn test_generator_reconstructor_basic() -> anyhow::Result<()> {
    let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
        polynomials_per_ssa: 10,
        threshold: 10,
        surplus_shares: 0,
    });

    let pseudonym = SimplePseudonym::random();
    let peer = OffchainKeypair::random();

    let client_commitment_msg = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;
    let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig {
        early_recovery_threshold: 1.0,
        ..Default::default()
    });

    let ssa_id = SsaId::new(pseudonym, 1.try_into()?);

    let server_commitment = reconstructor.new_exit_commitment(ssa_id, 10, 10)?;

    let full_ssa_deposit_address =
        TestSpec::group_to_deposit_address(client_commitment_msg.ssa_commitment + server_commitment)
            .ok_or(anyhow::anyhow!("failed to convert to address"))?;

    client_commitment_msg.process_into_reconstructor(&reconstructor)?;

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

    assert!(matches!(&res[0], ShareResolution::RecoveredSsa(res)
        if res.ssa_id == ssa_id && <TestSpec as PixSpec>::DepositAddress::from(&res.ssa) == full_ssa_deposit_address
    ));

    Ok(())
}
