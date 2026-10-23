//! A bounded SURB buffer must not bulk-fail a PIX cycle it is carrying.
//!
//! # The property
//!
//! A PIX share reaches the Exit's reconstructor only when the SURB holding it is *used*. The Exit
//! emits a cycle round-robin across [`SHARE_EMISSION_WINDOW`] polynomials at a time, so a buffer
//! that hands back a **contiguous** run of that emission starves whole blocks of polynomials — and a
//! cycle recovers only if *every* polynomial reaches its threshold. The surplus absorbs loss that is
//! independent per share (measured: 40 % spread loss still recovers); it cannot absorb a contiguous
//! block of it (measured: 5 % contiguous loss does not).
//!
//! `SurbRingBuffer` therefore buckets share-bearing SURBs by emission window and rotates across the
//! buckets, so what a full buffer discards is spread over the whole cycle. This test pins that
//! against the real `SsaReconstructor` rather than against a model of it.
//!
//! # Scale
//!
//! Deliberately small — 4 windows' worth of polynomials at the smallest legal threshold — so it runs
//! in CI in well under a second. The *shape* is what is under test, not the magnitude: the same
//! assertion at the deployed 4096 × (32 + 79) takes minutes and produces identical behaviour.

use hopr_api::types::{
    crypto::prelude::*,
    crypto_random::Randomizable,
    internal::prelude::{Acknowledgement, VerifiedAcknowledgement},
    primitive::prelude::{Address, BytesRepresentable},
};
use hopr_crypto_packet::{prelude::*, sphinx::prelude::SphinxHeaderSpec};
use hopr_protocol_hopr::{MemorySurbStore, SurbStore, SurbStoreConfig};
use hopr_protocol_pix::{
    EntryShareGenerator, ExitAcknowledgementShareProcessor, PixGroup, PixParams, PixScalar, PixSpec, PixSuite,
    SHARE_EMISSION_WINDOW, SsaGeneratorConfig, SsaId, SsaIndex, SsaReconstructor, SsaReconstructorConfig,
    SsaShareGenerator, TaggedEncryptedPartialSsaShare,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
struct Spec;

impl PixSpec for Spec {
    type AddressPrivateKey = ChainKeypair;
    type Cipher = ChaCha20;
    type Curve = Secp256k1;
    type DepositAddress = Address;
    type Digest = Blake3;
    type Pseudonym = SimplePseudonym;

    const HASH_TO_SCALAR_SUITE_ID: &'static [u8] = b"Secp256k1_XMD:BLAKE3_SSWU_RO_";
    const PIX_SUITE: PixSuite = PixSuite::Secp256k1;

    fn group_to_deposit_address(group: PixGroup<Self>) -> Option<Address> {
        PublicKey::try_from(group.to_affine()).ok().map(|pk| pk.to_address())
    }

    fn scalar_to_private_key(scalar: PixScalar<Self>) -> Option<ChainKeypair> {
        ChainKeypair::from_secret(scalar.to_bytes().as_ref()).ok()
    }
}

/// Four emission windows' worth of polynomials — enough that a buffer holding only a fraction of the
/// cycle spans more than one window, which is the case the bucketing exists for.
const POLYS: u16 = 4 * SHARE_EMISSION_WINDOW as u16;
/// The smallest legal threshold, so a cycle completes quickly without weakening the per-polynomial
/// requirement the test asserts.
const THRESHOLD: u8 = 2;
/// Enough surplus that the spread loss this test inflicts is comfortably within it.
const SURPLUS: u8 = 8;
/// A funded cycle always has a non-zero index; zero means "share-less SURB" to the store.
const SSA_INDEX: u32 = 1;

/// Builds a SURB whose plaintext receiver block names `(SSA_INDEX, poly_index)`, laid out exactly as
/// `SurbReceiverInfo` lays it out: `por_values | (ssa_index | poly_index | scalar) | generation`.
fn surb_for(poly_index: u16) -> anyhow::Result<HoprSurb> {
    let mut bytes = vec![0u8; HoprSurb::SIZE];

    let key_id_size = HoprSphinxHeaderSpec::KEY_ID_SIZE.get();
    bytes[..key_id_size].copy_from_slice(HoprKeyIdent::from(1u32).as_ref());
    // Chain length 1 = a direct return path, which needs no payment channel and is never filtered
    // out by relayer invalidation — so the store's *selection* is the only thing under test.
    bytes[HoprSurb::SIZE - HoprSphinxHeaderSpec::SURB_RECEIVER_DATA_SIZE] = 1;

    let share_at = HoprSurb::SIZE - 1 - HoprEncryptedPartialSsaShare::SIZE;
    bytes[share_at..share_at + size_of::<SsaIndex>()].copy_from_slice(&SSA_INDEX.to_be_bytes());
    bytes[share_at + size_of::<SsaIndex>()..share_at + size_of::<SsaIndex>() + size_of::<u16>()]
        .copy_from_slice(&poly_index.to_be_bytes());

    Ok(HoprSurb::try_from(bytes.as_slice())?)
}

/// Runs one funded cycle through the real store and the real reconstructor.
///
/// Shares are encrypted under their own acknowledgement, as `create_surb_for_path` does, and handed
/// to the reconstructor unacknowledged. A share is only *credited* when the Exit pops its SURB and
/// the Entry answers with that acknowledgement — which is what makes the store's selection the sole
/// determinant of what the cycle can reconstruct.
///
/// `ingest_per_reply` is how many SURBs arrive per reply the Exit spends; above 1 the Entry
/// out-produces the Exit and the buffer overflows.
fn run_cycle(rb_capacity: usize, ingest_per_reply: f64) -> anyhow::Result<(u32, bool)> {
    let generator = SsaShareGenerator::<Spec>::new(SsaGeneratorConfig {
        polynomials_per_ssa: POLYS,
        threshold: THRESHOLD,
        surplus_shares: SURPLUS,
    });
    let pseudonym = SimplePseudonym::random();
    let peer = OffchainKeypair::random();

    let reconstructor = SsaReconstructor::<Spec>::new(SsaReconstructorConfig {
        early_recovery_threshold: 1.0,
        ..Default::default()
    });

    let commitment = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN)?;
    let ssa_id = SsaId::new(pseudonym, 1.try_into()?);
    reconstructor.new_exit_commitment(ssa_id, PixParams::try_from_config::<Spec>(generator.config())?)?;
    commitment.process_into_reconstructor(&reconstructor)?;

    let exit = MemorySurbStore::new(SurbStoreConfig {
        rb_capacity,
        ..Default::default()
    });
    let entry = MemorySurbStore::new(SurbStoreConfig {
        // Generous, so the opener cache never binds: this test is about the SURB store.
        max_openers_per_pseudonym: 1_000_000,
        ..Default::default()
    });

    // SURB id -> (its acknowledgement, its polynomial).
    let mut awaiting: std::collections::HashMap<HoprSurbId, (HalfKey, u16)> = Default::default();
    let mut acks: Vec<Acknowledgement> = Vec::new();
    // Per-emission-window count of shares whose SURB the buffer handed back and the Entry
    // acknowledged: the spread the bucketing is supposed to produce, printable below.
    let mut acked_w: std::collections::BTreeMap<u16, usize> = Default::default();
    let mut reply_acc = 0.0f64;
    let mut ctr = 0u64;
    let mut total_emitted = 0usize;

    loop {
        let msg = ctr.to_be_bytes();
        let Some(share) = generator.next_share(&pseudonym, &msg)? else {
            break;
        };
        let poly = u16::try_from(u32::from(share.id.poly_index()))?;

        let ack = HalfKey::random();
        let enc = share.share.encrypt(&share.id, &ack)?;
        reconstructor.insert_encrypted_share(
            peer.public(),
            ack.to_challenge()?,
            TaggedEncryptedPartialSsaShare::new(pseudonym, &msg, enc)?,
        )?;

        total_emitted += 1;
        let id: HoprSurbId = ctr.to_be_bytes();
        awaiting.insert(id, (ack, poly));
        entry.insert_reply_opener(
            HoprSenderId::from_pseudonym_and_id(&pseudonym, id),
            ReplyOpener {
                sender_key: SecretKey16::random(),
                shared_secrets: Vec::new(),
            },
        );
        exit.insert_surbs(pseudonym, vec![(id, surb_for(poly)?)]);
        ctr += 1;

        reply_acc += 1.0 / ingest_per_reply.max(f64::EPSILON);
        while reply_acc >= 1.0 {
            reply_acc -= 1.0;
            let Some(found) = exit.find_surb(hopr_api::types::internal::routing::SurbMatcher::Pseudonym(pseudonym))
            else {
                break;
            };
            if let Some((ack, poly)) = awaiting.remove(&found.sender_id.surb_id()) {
                *acked_w.entry(poly / SHARE_EMISSION_WINDOW as u16).or_default() += 1;
                acks.push(VerifiedAcknowledgement::new(ack, &peer).leak());
            }
        }
    }

    // The Exit keeps replying on whatever the buffer still holds.
    while let Some(found) = exit.find_surb(hopr_api::types::internal::routing::SurbMatcher::Pseudonym(pseudonym)) {
        if let Some((ack, poly)) = awaiting.remove(&found.sender_id.surb_id()) {
            *acked_w.entry(poly / SHARE_EMISSION_WINDOW as u16).or_default() += 1;
            acks.push(VerifiedAcknowledgement::new(ack, &peer).leak());
        }
    }

    {
        // The shares the buffer never handed back, by window: what the reconstructor must cover
        // from surplus. Spread across windows is the property under test; a concentration at the
        // low windows would mean the rotation stalled.
        let mut left_w: std::collections::BTreeMap<u16, usize> = Default::default();
        for &(_, poly) in awaiting.values() {
            *left_w.entry(poly / SHARE_EMISSION_WINDOW as u16).or_default() += 1;
        }
        eprintln!(
            "total_shares={} acks={} awaiting_left={} acked_per_window={acked_w:?} left_per_window={left_w:?}",
            total_emitted,
            acks.len(),
            awaiting.len()
        );
    }
    let mut recovered = false;
    for chunk in acks.chunks(20_000) {
        for resolution in reconstructor.acknowledge_shares(*peer.public(), chunk.to_vec())? {
            if matches!(resolution, hopr_protocol_pix::ShareResolution::RecoveredSsa(_)) {
                recovered = true;
            }
        }
    }

    Ok((0, recovered))
}

/// A buffer holding a fraction of a cycle still lets that cycle reconstruct, because the shares it
/// must discard are spread across the emission instead of taken from one end of it.
///
/// This is the regression guard for `SurbShareInfo::emission_window`. Before the bucketing, a buffer
/// this small handed back one contiguous run of the emission, starving whole windows of polynomials,
/// and the cycle could never complete however much surplus it carried.
#[test]
fn a_bounded_surb_buffer_should_not_bulk_fail_a_pix_cycle() -> anyhow::Result<()> {
    // A quarter of the cycle's capacity, at an arrival rate that overflows it heavily.
    let cycle_shares = usize::from(POLYS) * usize::from(THRESHOLD + SURPLUS);
    let capacity = cycle_shares / 4;

    let (_, recovered) = run_cycle(capacity, 4.0)?;
    assert!(
        recovered,
        "a {capacity}-SURB buffer (a quarter of the {cycle_shares}-share cycle) must still let the cycle recover; if \
         this fails, the buffer is handing back a contiguous run of the emission again"
    );

    Ok(())
}

/// Nothing at all is lost when the buffer is large enough never to discard a share.
///
/// The floor under the test above: it proves the harness can reconstruct a cycle at this scale, so a
/// failure there is the buffering and not the fixture.
#[test]
fn an_unbounded_surb_buffer_should_recover_a_pix_cycle() -> anyhow::Result<()> {
    let cycle_shares = usize::from(POLYS) * usize::from(THRESHOLD + SURPLUS);
    let (_, recovered) = run_cycle(cycle_shares * 2, 1.0)?;
    assert!(recovered, "a buffer holding twice the cycle must recover it");
    Ok(())
}
