use crate::{MAX_POLY_THRESHOLD, MAX_POLYS_PER_SSA, MIN_POLY_THRESHOLD, generator::SsaGeneratorConfig};

/// Why a [`PixParams`] triple was rejected.
///
/// `surplus_shares` has no variant: its permitted range is the whole of `u8`, so it cannot be out of
/// range once it has a type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidPixParams {
    #[error("polynomials per SSA must be between 1 and {MAX_POLYS_PER_SSA}, got {0}")]
    PolysPerSsa(u16),
    #[error("polynomial threshold must be between {MIN_POLY_THRESHOLD} and {MAX_POLY_THRESHOLD}, got {0}")]
    SharesPerPoly(u8),
}

/// Bit position of [`PixParams::polys_per_ssa`] within the packed `u32`.
const POLYS_SHIFT: u32 = 16;
/// Bit position of [`PixParams::shares_per_poly`] within the packed `u32`.
const SHARES_SHIFT: u32 = 8;
/// Bit position of the packed `u32` within [`PixParams::into_additional_data`]'s `u64`.
const ADDITIONAL_DATA_SHIFT: u32 = 32;

/// The PIX dimensions two nodes agree on for a Session, and the only encoding of them.
///
/// The same triple is packed into two different wire fields — the `SsaRequest` `params` word and the
/// upper half of `StartInitiation::additional_data` — and both go through this type. Every earlier
/// version of this had the shifts written out by hand at each site, in two mutually inconsistent
/// shapes, which is why the packing lives behind a constructor rather than in the callers.
///
/// Named fields rather than a `(u16, u8, u8)` tuple: `polys_per_ssa` and `shares_per_poly` are
/// interchangeable to the type system and *not* interchangeable to the protocol, while their product
/// — which is all the Exit compares — is identical either way. A transposition therefore announced
/// valid-looking dimensions against a correct quota, and the only thing that caught it was
/// `SessionManager::new_session` requiring both to match the locally installed generator exactly,
/// which is a check about something else entirely.
///
/// The fields are private because [`try_new`](Self::try_new) is what enforces the ranges; holding a
/// `PixParams` is what makes [`to_u32`](Self::to_u32) infallible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PixParams {
    polys_per_ssa: u16,
    shares_per_poly: u8,
    surplus_shares: u8,
}

impl PixParams {
    /// Validates and assembles the triple.
    pub const fn try_new(
        polys_per_ssa: u16,
        shares_per_poly: u8,
        surplus_shares: u8,
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
        })
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

    /// Packs into 32 bits: `polys_per_ssa` in bits 31..16, `shares_per_poly` in bits 15..8, and
    /// `surplus_shares` in bits 7..0.
    #[inline]
    pub const fn to_u32(&self) -> u32 {
        ((self.polys_per_ssa as u32) << POLYS_SHIFT)
            | ((self.shares_per_poly as u32) << SHARES_SHIFT)
            | self.surplus_shares as u32
    }

    /// Inverse of [`to_u32`](Self::to_u32), rejecting out-of-range values.
    pub const fn try_from_u32(packed: u32) -> Result<Self, InvalidPixParams> {
        Self::try_new(
            (packed >> POLYS_SHIFT) as u16,
            (packed >> SHARES_SHIFT) as u8,
            packed as u8,
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
            "{} polys x {} shares (+{} surplus)",
            self.polys_per_ssa, self.shares_per_poly, self.surplus_shares
        )
    }
}

impl TryFrom<&SsaGeneratorConfig> for PixParams {
    type Error = InvalidPixParams;

    /// The Entry's only source of a [`PixParams`]: the surplus is a property of the installed
    /// generator, not something a Session caller gets to pick.
    ///
    /// Fallible because [`SsaGeneratorConfig`]'s fields are public and its ranges are enforced by
    /// `validator` rather than by construction.
    fn try_from(cfg: &SsaGeneratorConfig) -> Result<Self, Self::Error> {
        Self::try_new(cfg.polynomials_per_ssa, cfg.threshold, cfg.surplus_shares)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DEFAULT_POLY_THRESHOLD, DEFAULT_POLYS_PER_SSA};

    /// Pins the byte layout. A reshuffle of the three fields is otherwise invisible: every value
    /// still round-trips through `to_u32`/`try_from_u32`, and only the peer notices.
    #[test]
    fn packed_layout_must_stay_polys_then_threshold_then_surplus() -> anyhow::Result<()> {
        assert_eq!(0x1234_5678, PixParams::try_new(0x1234, 0x56, 0x78)?.to_u32());
        assert_eq!(
            0x1234_5678_9abc_def0_u64,
            PixParams::try_new(0x1234, 0x56, 0x78)?.into_additional_data(0x9abc_def0)
        );
        Ok(())
    }

    #[test]
    fn packed_u32_must_round_trip_over_the_whole_range() -> anyhow::Result<()> {
        for polys in [1, 2, DEFAULT_POLYS_PER_SSA, MAX_POLYS_PER_SSA] {
            for shares in [MIN_POLY_THRESHOLD, DEFAULT_POLY_THRESHOLD, MAX_POLY_THRESHOLD] {
                for surplus in [0, 1, 20, u8::MAX] {
                    let params = PixParams::try_new(polys, shares, surplus)?;
                    assert_eq!(params, PixParams::try_from_u32(params.to_u32())?);
                }
            }
        }
        Ok(())
    }

    #[test]
    fn additional_data_must_round_trip_and_leave_the_surb_target_alone() -> anyhow::Result<()> {
        let params = PixParams::try_new(DEFAULT_POLYS_PER_SSA, DEFAULT_POLY_THRESHOLD, 32)?;
        for surb_target in [0, 1, 1234, u32::MAX] {
            let additional_data = params.into_additional_data(surb_target);
            assert_eq!(params, PixParams::try_from_additional_data(additional_data)?);
            assert_eq!(surb_target, additional_data as u32);
        }
        Ok(())
    }

    #[test]
    fn out_of_range_dimensions_must_be_rejected() {
        assert_eq!(
            Err(InvalidPixParams::PolysPerSsa(0)),
            PixParams::try_new(0, DEFAULT_POLY_THRESHOLD, 0)
        );
        assert_eq!(
            Err(InvalidPixParams::PolysPerSsa(MAX_POLYS_PER_SSA + 1)),
            PixParams::try_new(MAX_POLYS_PER_SSA + 1, DEFAULT_POLY_THRESHOLD, 0)
        );
        for shares in [0, 1] {
            assert_eq!(
                Err(InvalidPixParams::SharesPerPoly(shares)),
                PixParams::try_new(DEFAULT_POLYS_PER_SSA, shares, 0)
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

    #[test]
    fn generator_config_must_convert() -> anyhow::Result<()> {
        let cfg = SsaGeneratorConfig {
            polynomials_per_ssa: 8,
            threshold: 2,
            surplus_shares: 3,
        };
        let params = PixParams::try_from(&cfg)?;
        assert_eq!(8, params.polys_per_ssa());
        assert_eq!(2, params.shares_per_poly());
        assert_eq!(3, params.surplus_shares());
        assert_eq!(5, params.emitted_shares_per_poly());

        assert_eq!(
            PixParams::try_from(&SsaGeneratorConfig::default())?,
            PixParams::try_new(
                DEFAULT_POLYS_PER_SSA,
                DEFAULT_POLY_THRESHOLD,
                SsaGeneratorConfig::default().surplus_shares
            )?
        );
        Ok(())
    }
}
