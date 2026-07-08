use std::collections::HashMap;

use hopr_protocol_pix::{
    EntryShareGenerator, ExitAcknowledgementShareProcessor, PixGroup, PixScalar, PixSpec, ShareResolution,
    SsaCommitment, SsaGeneratorConfig, SsaId, SsaIndex, SsaReconstructor, SsaShareGenerator,
    TaggedEncryptedPartialSsaShare,
};
use hopr_types::{
    crypto::{
        keypairs::ChainKeypair,
        prelude::{HalfKey, Keypair, OffchainKeypair, PublicKey, SimplePseudonym},
    },
    crypto_random::Randomizable,
    internal::prelude::VerifiedAcknowledgement,
    primitive::prelude::Address,
};
use rand::prelude::SliceRandom;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TestSpec;

impl PixSpec for TestSpec {
    type AddressPrivateKey = ChainKeypair;
    type Cipher = hopr_types::crypto::primitives::ChaCha20;
    type Curve = k256::Secp256k1;
    type DepositAddress = Address;
    type Digest = hopr_types::crypto::primitives::Sha3_256;
    type Pseudonym = SimplePseudonym;

    fn group_to_deposit_address(group: PixGroup<Self>) -> Option<Self::DepositAddress> {
        PublicKey::try_from(group.to_affine()).ok().map(|pk| pk.to_address())
    }

    fn scalar_to_private_key(scalar: PixScalar<Self>) -> Option<Self::AddressPrivateKey> {
        ChainKeypair::from_secret(scalar.to_bytes().as_ref()).ok()
    }
}

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
        verifiers,
        ..
    } = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;

    // Use the already transposed verifiers
    let mut transposed = verifiers
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().collect::<HashMap<_, _>>()))
        .collect::<HashMap<_, _>>();

    let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());

    let ssa_id = SsaId::new(pseudonym, 1.try_into()?);

    let server_commitment = reconstructor.new_exit_commitment(ssa_id, 10, 10)?;

    let full_ssa_deposit_address = TestSpec::group_to_deposit_address(client_commitment + server_commitment)
        .ok_or(anyhow::anyhow!("Failed to convert to address"))?;

    // In the transposed form, remove the first coefficient commitments of all polynomials
    let mut first_coeffs = transposed.remove(&0).unwrap();

    // Remove the first polynomial completely
    let remainder = first_coeffs.remove(&0).unwrap();

    // Insert all constant term commitments, except the constant term commitments of the first polynomial
    let res = reconstructor.insert_coefficient_commitments(ssa_id, 0, first_coeffs.into_iter())?;
    assert_eq!(ssa_id, res.ssa_id);
    assert!(res.is_first_encountered);
    assert!(res.ssa_deposit_address.is_none());
    assert!(!res.is_verifiable);

    // Now add the constant term commitments of the first polynomial
    let res = reconstructor.insert_coefficient_commitments(ssa_id, 0, HashMap::from([(0, remainder)]).into_iter())?;
    assert_eq!(ssa_id, res.ssa_id);
    assert!(!res.is_first_encountered);
    assert_eq!(Some(full_ssa_deposit_address), res.ssa_deposit_address);
    assert!(!res.is_verifiable);

    // Add all the remaining coefficient commitments for all polynomials except one
    let remainder = transposed.remove(&5).unwrap();
    for (coeff_index, poly_coeff_commitments) in transposed {
        let res =
            reconstructor.insert_coefficient_commitments(ssa_id, coeff_index, poly_coeff_commitments.into_iter())?;
        assert_eq!(ssa_id, res.ssa_id);
        assert!(!res.is_first_encountered);
        assert_eq!(Some(full_ssa_deposit_address), res.ssa_deposit_address);
        assert!(!res.is_verifiable);
    }

    // Now the SSA should be fully committed
    let res = reconstructor.insert_coefficient_commitments(ssa_id, 5, remainder.into_iter())?;
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
        if res.ssa_id == ssa_id && res.ssa.public().to_address() == full_ssa_deposit_address
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
    let reconstructor = SsaReconstructor::<TestSpec>::new(Default::default());

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
        if res.ssa_id == ssa_id && res.ssa.public().to_address() == full_ssa_deposit_address
    ));

    Ok(())
}
