use crate::{MAX_POLY_THRESHOLD, MAX_POLYS_PER_SSA, MIN_POLY_THRESHOLD, generator::SsaGeneratorConfig};

/// Why a [`PixParams`] quadruple was rejected.
///
/// `surplus_shares` has no variant: its permitted range is the whole of `u8`, so it cannot be out of
/// range once it has a type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidPixParams {
    #[error("polynomials per SSA must be between 1 and {MAX_POLYS_PER_SSA}, got {0}")]
    PolysPerSsa(u16),
    #[error("polynomial threshold must be between {MIN_POLY_THRESHOLD} and {MAX_POLY_THRESHOLD}, got {0}")]
    SharesPerPoly(u8),
    #[error("unknown PIX curve suite identifier {0}")]
    UnknownSuite(u8),
}

/// The elliptic curve a PIX deployment instantiates [`PixSpec`](crate::PixSpec) over.
///
/// Every curve-sized field in the PIX handshake — each coefficient commitment, the commitment proof
/// of knowledge — is sized by this choice, while the messages carrying them are versioned only by
/// `StartProtocol`'s own version byte. Two peers built for different curves therefore agree on the
/// protocol version and disagree on where every element boundary falls, so this rides in
/// [`PixParams`] to make the disagreement visible in the one field whose size *does not* depend on
/// the curve, before either side interprets one that does.
///
/// It is not negotiated. The Exit accepts or refuses what the Entry offers; see
/// [`PixSpec::PIX_SUITE`](crate::PixSpec::PIX_SUITE).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum PixSuite {
    /// Baby JubJub, the production default (`pix-bjj`).
    ///
    /// Zero so that a word packed before this field existed decodes as the curve those builds
    /// overwhelmingly ran. See [`PixParams::try_from_u32`].
    #[default]
    BabyJubJub = 0,
    /// secp256k1, giving Ethereum-shaped deposit addresses (`pix-secp256k1`).
    Secp256k1 = 1,
}

impl PixSuite {
    /// Decodes the two-bit wire form, rejecting the two values no curve claims.
    pub const fn try_from_bits(bits: u8) -> Result<Self, InvalidPixParams> {
        match bits {
            0 => Ok(Self::BabyJubJub),
            1 => Ok(Self::Secp256k1),
            other => Err(InvalidPixParams::UnknownSuite(other)),
        }
    }
}

impl std::fmt::Display for PixSuite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::BabyJubJub => "BabyJubJub",
            Self::Secp256k1 => "secp256k1",
        })
    }
}

/// Bit position of [`PixParams::suite`] within the packed `u32`.
const SUITE_SHIFT: u32 = 30;
/// Bit position of [`PixParams::polys_per_ssa`] within the packed `u32`.
const POLYS_SHIFT: u32 = 16;
/// Mask applied to the polynomial count after shifting, i.e. the 14 bits below the suite.
const POLYS_MASK: u32 = 0x3fff;
/// Bit position of [`PixParams::shares_per_poly`] within the packed `u32`.
const SHARES_SHIFT: u32 = 8;
/// Bit position of the packed `u32` within [`PixParams::into_additional_data`]'s `u64`.
const ADDITIONAL_DATA_SHIFT: u32 = 32;

/// The suite occupies the top two bits of the polynomial-count field, which are free only because
/// [`MAX_POLYS_PER_SSA`] fits in 14 bits. Raising that ceiling into them would silently corrupt the
/// suite of every packed word, so it is a build failure instead.
const _: () = assert!(
    MAX_POLYS_PER_SSA as u32 <= POLYS_MASK,
    "MAX_POLYS_PER_SSA no longer fits below the PixParams suite bits"
);

/// Everything about PIX two nodes must agree on for a Session, and the only encoding of it: three
/// dimensions and the curve suite they are dimensions of.
///
/// The same quadruple is packed into two different wire fields — the `SsaRequest` `params` word and
/// the upper half of `StartInitiation::additional_data` — and both go through this type. Every
/// earlier version of this had the shifts written out by hand at each site, in two mutually
/// inconsistent shapes, which is why the packing lives behind a constructor rather than in the
/// callers.
///
/// Named fields rather than a `(u16, u8, u8, PixSuite)` tuple: `polys_per_ssa` and `shares_per_poly`
/// are interchangeable to the type system and *not* interchangeable to the protocol, while their
/// product — which is all the Exit compares — is identical either way. A transposition therefore
/// announced valid-looking dimensions against a correct quota, and the only thing that caught it was
/// `SessionManager::new_session` requiring both to match the locally installed generator exactly,
/// which is a check about something else entirely.
///
/// The fields are private because [`try_new`](Self::try_new) is what enforces the ranges; holding a
/// `PixParams` is what makes [`to_u32`](Self::to_u32) infallible.
///
/// The [`suite`](Self::suite) is here for the same reason the dimensions are: it is something both
/// nodes must agree on, and equality of this type is what the Entry already checks against the
/// Exit's echo, so carrying it here gets that direction for free. Note that it is not a dimension —
/// it does not enter the quota — so code that speaks about what the deposit buys is right to name
/// only the other three.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PixParams {
    polys_per_ssa: u16,
    shares_per_poly: u8,
    surplus_shares: u8,
    suite: PixSuite,
}

impl PixParams {
    /// Validates and assembles the quadruple.
    pub const fn try_new(
        polys_per_ssa: u16,
        shares_per_poly: u8,
        surplus_shares: u8,
        suite: PixSuite,
    ) -> Result<Self, InvalidPixParams> {
        if polys_per_ssa == 0 || polys_per_ssa > MAX_POLYS_PER_SSA {
            return Err(InvalidPixParams::PolysPerSsa(polys_per_ssa));
        }
        // No upper-bound check: `MAX_POLY_THRESHOLD` is `u8::MAX`, so the field cannot hold a value
        // above it. That is the point of narrowing the threshold to a byte — see the constant.
        if shares_per_poly < MIN_POLY_THRESHOLD {
            return Err(InvalidPixParams::SharesPerPoly(shares_per_poly));
        }
        Ok(Self {
            polys_per_ssa,
            shares_per_poly,
            surplus_shares,
            suite,
        })
    }

    /// Validates and assembles the dimensions under the curve suite of `S`.
    ///
    /// The constructor to prefer wherever a concrete spec is in scope, because it makes the suite
    /// impossible to state wrongly: the shares a node produces and the suite it announces then come
    /// from the same place.
    pub fn try_new_for<S: crate::PixSpec>(
        polys_per_ssa: u16,
        shares_per_poly: u8,
        surplus_shares: u8,
    ) -> Result<Self, InvalidPixParams> {
        Self::try_new(polys_per_ssa, shares_per_poly, surplus_shares, S::PIX_SUITE)
    }

    /// The [`SsaGeneratorConfig`]'s dimensions under the curve suite of `S`.
    ///
    /// The Entry's only source of a [`PixParams`]: the dimensions and the surplus are properties of
    /// the installed generator, and the suite is a property of the spec that generator is
    /// instantiated over — neither is something a Session caller gets to pick.
    ///
    /// Fallible because [`SsaGeneratorConfig`]'s fields are public and its ranges are enforced by
    /// `validator` rather than by construction.
    pub fn try_from_config<S: crate::PixSpec>(cfg: &SsaGeneratorConfig) -> Result<Self, InvalidPixParams> {
        Self::try_new_for::<S>(cfg.polynomials_per_ssa, cfg.threshold, cfg.surplus_shares)
    }

    /// Number of polynomials the SSA secret is split across.
    #[inline]
    pub const fn polys_per_ssa(&self) -> u16 {
        self.polys_per_ssa
    }

    /// Shares required to reconstruct one polynomial.
    #[inline]
    pub const fn shares_per_poly(&self) -> u8 {
        self.shares_per_poly
    }

    /// Shares emitted per polynomial beyond [`shares_per_poly`](Self::shares_per_poly), to absorb
    /// losses.
    #[inline]
    pub const fn surplus_shares(&self) -> u8 {
        self.surplus_shares
    }

    /// Total shares the Entry emits per polynomial, i.e. threshold plus surplus.
    ///
    /// Widened to `u16` because the sum of two `u8`s does not fit one.
    #[inline]
    pub const fn emitted_shares_per_poly(&self) -> u16 {
        self.shares_per_poly as u16 + self.surplus_shares as u16
    }

    /// Useful shares needed to recover the whole SSA.
    ///
    /// A share count, unlike the byte quota those shares pay for, which the Session layer derives
    /// from the same two fields. Excludes the surplus by construction: a surplus share is one that
    /// arrives after its polynomial already has a full set, so it advances nothing.
    #[inline]
    pub const fn target_useful_shares(&self) -> u64 {
        self.polys_per_ssa as u64 * self.shares_per_poly as u64
    }

    /// The elliptic curve suite these parameters were produced under.
    #[inline]
    pub const fn suite(&self) -> PixSuite {
        self.suite
    }

    /// Packs into 32 bits: `suite` in bits 31..30, `polys_per_ssa` in bits 29..16,
    /// `shares_per_poly` in bits 15..8, and `surplus_shares` in bits 7..0.
    #[inline]
    pub const fn to_u32(&self) -> u32 {
        ((self.suite as u32) << SUITE_SHIFT)
            | ((self.polys_per_ssa as u32 & POLYS_MASK) << POLYS_SHIFT)
            | ((self.shares_per_poly as u32) << SHARES_SHIFT)
            | self.surplus_shares as u32
    }

    /// Inverse of [`to_u32`](Self::to_u32), rejecting out-of-range values and unknown suites.
    ///
    /// # Compatibility with words packed before the suite existed
    ///
    /// Those words carried the polynomial count in the full top 16 bits, but the count is bounded by
    /// [`MAX_POLYS_PER_SSA`] and so never set the two the suite now occupies. Reading such a word
    /// therefore yields [`PixSuite::BabyJubJub`], which is what those builds ran by default. In the
    /// other direction a peer that predates this field rejects a `Secp256k1` word outright: the
    /// suite bit reads to it as a polynomial count of at least 16 384, above the maximum it already
    /// enforced. Neither side mis-parses the other; both refuse.
    pub const fn try_from_u32(packed: u32) -> Result<Self, InvalidPixParams> {
        let suite = match PixSuite::try_from_bits((packed >> SUITE_SHIFT) as u8) {
            Ok(suite) => suite,
            Err(error) => return Err(error),
        };
        Self::try_new(
            ((packed >> POLYS_SHIFT) & POLYS_MASK) as u16,
            (packed >> SHARES_SHIFT) as u8,
            packed as u8,
            suite,
        )
    }

    /// Packs into the upper half of a `StartInitiation::additional_data` word, leaving `surb_target`
    /// in the lower half.
    ///
    /// The two halves are the whole of that field: there is no room left in it to negotiate anything
    /// further.
    #[inline]
    pub const fn into_additional_data(self, surb_target: u32) -> u64 {
        ((self.to_u32() as u64) << ADDITIONAL_DATA_SHIFT) | surb_target as u64
    }

    /// Inverse of [`into_additional_data`](Self::into_additional_data), ignoring the SURB target in
    /// the lower half.
    pub const fn try_from_additional_data(additional_data: u64) -> Result<Self, InvalidPixParams> {
        Self::try_from_u32((additional_data >> ADDITIONAL_DATA_SHIFT) as u32)
    }
}

impl std::fmt::Display for PixParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} polys x {} shares (+{} surplus) on {}",
            self.polys_per_ssa, self.shares_per_poly, self.surplus_shares, self.suite
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DEFAULT_POLY_THRESHOLD, DEFAULT_POLYS_PER_SSA, PixSpec, tests::TestSpec};

    /// Pins the byte layout. A reshuffle of the fields is otherwise invisible: every value still
    /// round-trips through `to_u32`/`try_from_u32`, and only the peer notices.
    ///
    /// The Baby JubJub word is the same one this test asserted before the suite existed, which is
    /// the compatibility claim: adding the field moved no bit of a default-curve deployment.
    #[test]
    fn packed_layout_must_stay_suite_then_polys_then_threshold_then_surplus() -> anyhow::Result<()> {
        let bjj = PixParams::try_new(0x1234, 0x56, 0x78, PixSuite::BabyJubJub)?;
        assert_eq!(0x1234_5678, bjj.to_u32());
        assert_eq!(0x1234_5678_9abc_def0_u64, bjj.into_additional_data(0x9abc_def0));

        // secp256k1 differs only in bit 30.
        let secp = PixParams::try_new(0x1234, 0x56, 0x78, PixSuite::Secp256k1)?;
        assert_eq!(0x5234_5678, secp.to_u32());
        assert_eq!(0x4000_0000, bjj.to_u32() ^ secp.to_u32());
        Ok(())
    }

    #[test]
    fn packed_u32_must_round_trip_over_the_whole_range() -> anyhow::Result<()> {
        for suite in [PixSuite::BabyJubJub, PixSuite::Secp256k1] {
            for polys in [1, 2, DEFAULT_POLYS_PER_SSA, MAX_POLYS_PER_SSA] {
                for shares in [MIN_POLY_THRESHOLD, DEFAULT_POLY_THRESHOLD, MAX_POLY_THRESHOLD] {
                    for surplus in [0, 1, 20, u8::MAX] {
                        let params = PixParams::try_new(polys, shares, surplus, suite)?;
                        assert_eq!(params, PixParams::try_from_u32(params.to_u32())?);
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn additional_data_must_round_trip_and_leave_the_surb_target_alone() -> anyhow::Result<()> {
        for suite in [PixSuite::BabyJubJub, PixSuite::Secp256k1] {
            let params = PixParams::try_new(DEFAULT_POLYS_PER_SSA, DEFAULT_POLY_THRESHOLD, 32, suite)?;
            for surb_target in [0, 1, 1234, u32::MAX] {
                let additional_data = params.into_additional_data(surb_target);
                assert_eq!(params, PixParams::try_from_additional_data(additional_data)?);
                assert_eq!(surb_target, additional_data as u32);
            }
        }
        Ok(())
    }

    #[test]
    fn out_of_range_dimensions_must_be_rejected() {
        assert_eq!(
            Err(InvalidPixParams::PolysPerSsa(0)),
            PixParams::try_new(0, DEFAULT_POLY_THRESHOLD, 0, PixSuite::BabyJubJub)
        );
        assert_eq!(
            Err(InvalidPixParams::PolysPerSsa(MAX_POLYS_PER_SSA + 1)),
            PixParams::try_new(MAX_POLYS_PER_SSA + 1, DEFAULT_POLY_THRESHOLD, 0, PixSuite::BabyJubJub)
        );
        for shares in [0, 1] {
            assert_eq!(
                Err(InvalidPixParams::SharesPerPoly(shares)),
                PixParams::try_new(DEFAULT_POLYS_PER_SSA, shares, 0, PixSuite::BabyJubJub)
            );
        }
    }

    /// The decode side is the one that matters: these words arrive from a peer.
    #[test]
    fn out_of_range_packed_words_must_be_rejected() {
        // polys = 0
        assert!(PixParams::try_from_u32(0x0000_4020).is_err());
        // threshold = 1
        assert!(PixParams::try_from_u32(0x2000_0100).is_err());
        // threshold = 0
        assert!(PixParams::try_from_u32(0x2000_0020).is_err());
        // Everything a `u8` surplus can say is legal.
        assert!(PixParams::try_from_u32(0x2000_40ff).is_ok());
    }

    /// The two suite values no curve claims are refused, rather than read as a third curve.
    #[test]
    fn unknown_suite_identifiers_must_be_rejected() {
        for (bits, word) in [(2u8, 0x8000_4020_u32), (3, 0xc000_4020)] {
            assert_eq!(Err(InvalidPixParams::UnknownSuite(bits)), PixParams::try_from_u32(word));
        }
    }

    /// What a peer built before the suite existed sees, and what it shows us.
    ///
    /// Both directions are refusals rather than mis-parses, which is the whole reason the suite went
    /// into these two bits rather than into a new field.
    #[test]
    fn pre_suite_words_stay_readable_and_a_foreign_one_is_out_of_range() -> anyhow::Result<()> {
        // A word packed before the field existed sets neither bit, so it reads as the curve those
        // builds ran by default.
        let pre_suite = ((DEFAULT_POLYS_PER_SSA as u32) << POLYS_SHIFT) | ((DEFAULT_POLY_THRESHOLD as u32) << 8) | 16;
        let decoded = PixParams::try_from_u32(pre_suite)?;
        assert_eq!(PixSuite::BabyJubJub, decoded.suite());
        assert_eq!(DEFAULT_POLYS_PER_SSA, decoded.polys_per_ssa());

        // And a secp256k1 word carries bit 30, which such a peer reads as a polynomial count of at
        // least 16 384 — above the `MAX_POLYS_PER_SSA` it already enforced, so it refuses.
        let secp = PixParams::try_new(DEFAULT_POLYS_PER_SSA, DEFAULT_POLY_THRESHOLD, 16, PixSuite::Secp256k1)?.to_u32();
        assert!(
            (secp >> POLYS_SHIFT) as u16 > MAX_POLYS_PER_SSA,
            "a pre-suite peer must reject a secp256k1 word by its existing polynomial range check"
        );
        Ok(())
    }

    #[test]
    fn generator_config_must_convert() -> anyhow::Result<()> {
        let cfg = SsaGeneratorConfig {
            polynomials_per_ssa: 8,
            threshold: 2,
            surplus_shares: 3,
        };
        let params = PixParams::try_from_config::<TestSpec>(&cfg)?;
        assert_eq!(8, params.polys_per_ssa());
        assert_eq!(2, params.shares_per_poly());
        assert_eq!(3, params.surplus_shares());
        assert_eq!(5, params.emitted_shares_per_poly());
        assert_eq!(
            TestSpec::PIX_SUITE,
            params.suite(),
            "the announced suite must come from the spec that will generate the shares"
        );

        assert_eq!(
            PixParams::try_from_config::<TestSpec>(&SsaGeneratorConfig::default())?,
            PixParams::try_new(
                DEFAULT_POLYS_PER_SSA,
                DEFAULT_POLY_THRESHOLD,
                SsaGeneratorConfig::default().surplus_shares,
                TestSpec::PIX_SUITE
            )?
        );
        Ok(())
    }
}
