//! The cross-curve tests a single build cannot otherwise produce.
//!
//! Every other PIX test runs one curve, because a build selects one — which is exactly why a
//! curve-compatibility defect is invisible to them. `common`'s `TestSpecBjj` and `TestSpecK256` are
//! the only place in the tree where both curves exist at once, so this is where the wire
//! consequence of that choice is pinned.
//!
//! Its own target rather than a module inside `common`, because the benchmarks include
//! `../tests/common.rs` and would compile these tests without running them.

mod common;

use common::{TestSpecBjj, TestSpecK256};
use hopr_protocol_pix::{PixGroupRepr, PixParams, PixSpec, PixSuite};

/// The two specs must not claim the same suite, or announcing it would say nothing.
#[test]
fn the_two_specs_announce_different_suites() {
    assert_eq!(PixSuite::BabyJubJub, TestSpecBjj::PIX_SUITE);
    assert_eq!(PixSuite::Secp256k1, TestSpecK256::PIX_SUITE);
    assert_ne!(TestSpecBjj::PIX_SUITE, TestSpecK256::PIX_SUITE);
}

/// A node built for one curve can tell that params came from the other, from the packed word alone
/// — before any curve-sized field is read.
///
/// This is the whole mechanism: the dimensions here are identical, so nothing but the suite
/// distinguishes the two words, and the difference survives the round trip through the wire
/// encoding an Exit actually receives.
#[test]
fn params_from_one_curve_are_distinguishable_from_the_other() -> anyhow::Result<()> {
    let bjj = PixParams::try_new_for::<TestSpecBjj>(8192, 64, 16)?;
    let secp = PixParams::try_new_for::<TestSpecK256>(8192, 64, 16)?;

    assert_ne!(
        bjj, secp,
        "identical dimensions on different curves must not compare equal"
    );
    assert_ne!(bjj.to_u32(), secp.to_u32(), "and must not pack to the same word");

    // What an Exit does with them: decode, then compare against its own build's suite.
    for (params, expected) in [(bjj, TestSpecBjj::PIX_SUITE), (secp, TestSpecK256::PIX_SUITE)] {
        let decoded = PixParams::try_from_additional_data(params.into_additional_data(0))?;
        assert_eq!(expected, decoded.suite());
        assert_eq!(params, decoded);
    }
    Ok(())
}

/// The curve is what makes the *rest* of the handshake incompatible, and that is measurable here:
/// the commitment these two specs put on the wire is not the same size.
///
/// If these ever coincided, a mismatch would be undetectable by length and the suite field would be
/// carrying the whole burden alone.
#[test]
fn the_two_curves_disagree_about_element_width() {
    assert_ne!(
        size_of::<PixGroupRepr<TestSpecBjj>>(),
        size_of::<PixGroupRepr<TestSpecK256>>(),
        "the suite field exists because these differ"
    );
}
