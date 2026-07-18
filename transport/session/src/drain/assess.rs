use hopr_api::types::primitive::{
    balance::HoprBalance,
    primitives::U256,
};
use hopr_protocol_pix::SsaDrainSnapshot;

use super::{
    ClosedSessionOffer, SsaHandover, SurbDrainConfig,
    SkipReason,
};
use crate::supervision::SessionPixCloseReason;

/// Verdict returned by [`evaluate_offer`].
#[derive(Debug)]
pub enum DrainVerdict {
    /// Eligible — spawn a drain task with these parameters.
    Drain(DrainParams),
    /// Ineligible — reason for skipping.
    Skip(SkipReason),
}

/// Parameters for a drain task, derived from the eligibility check.
#[derive(Debug)]
pub struct DrainParams {
    /// Maximum number of drain packets to send.
    pub max_packets: u64,
    /// Per-SSA deficit (useful shares still needed).
    pub deficits: Vec<DrainSsaDeficit>,
}

/// Deficit for one SSA targeted by the drain.
#[derive(Debug)]
pub struct DrainSsaDeficit {
    /// The guard (ownership) — retired on drop if deficit > 0.
    pub guard_index: usize,
    /// Number of useful shares still needed for full recovery.
    pub deficit: u64,
}

/// Pure-function assessment of whether a closed session is worth draining.
///
/// No I/O, no side effects — fully unit-testable.
pub fn evaluate_offer(
    cfg: &SurbDrainConfig,
    offer: &ClosedSessionOffer,
    snapshots: &[Option<SsaDrainSnapshot<hopr_api::types::internal::prelude::HoprPseudonym>>],
    surb_count: usize,
    packet_price: HoprBalance,
    active_drains: usize,
) -> DrainVerdict {
    if !cfg.enabled {
        return DrainVerdict::Skip(SkipReason::Disabled);
    }

    if active_drains >= cfg.max_concurrent_drains {
        return DrainVerdict::Skip(SkipReason::ConcurrencyLimit);
    }

    // Fault closes are never drained.
    if let Some(reason) = &offer.pix_close_reason {
        match reason {
            SessionPixCloseReason::TooManyUnverifiableShares
            | SessionPixCloseReason::CounterRegression
            | SessionPixCloseReason::InvalidTransition => {
                return DrainVerdict::Skip(SkipReason::FaultClose);
            }
            _ => {}
        }
    }

    if offer.ssas.is_empty() {
        return DrainVerdict::Skip(SkipReason::NoFundedSsa);
    }

    // Collect candidate SSAs: must have funded > 0 and deficit > 0.
    let candidates: Vec<(usize, &SsaHandover, u64)> = offer
        .ssas
        .iter()
        .enumerate()
        .filter_map(|(i, ssa)| {
            let deficit = snapshots.get(i).and_then(|snap| {
                let snap = snap.as_ref()?;
                let target = snap.progress.target_useful_shares;
                let useful = snap.progress.useful_shares;
                (useful < target).then_some(target - useful)
            }).unwrap_or(0);

            if ssa.funded > HoprBalance::zero() && deficit > 0 {
                Some((i, ssa, deficit))
            } else {
                None
            }
        })
        .collect();

    if candidates.is_empty() {
        // Check if any had funded > 0 but deficit == 0 (already recovered)
        let has_funded = offer.ssas.iter().any(|s| s.funded > HoprBalance::zero());
        return if has_funded {
            DrainVerdict::Skip(SkipReason::NoDeficit)
        } else {
            DrainVerdict::Skip(SkipReason::NoFundedSsa)
        };
    }

    // Economic gate: the deposit must cover the drain cost.
    // All arithmetic on HoprBalance is implicitly saturating (U256 backing).
    let total_deficit: u64 = candidates.iter().map(|(_, _, d)| d).sum();
    let total_funded: HoprBalance = candidates.iter().fold(HoprBalance::zero(), |acc, (_, s, _)| acc + s.funded);

    // Compute required = cost_per_packet * total_deficit * safety_factor / 100
    // using U256 arithmetic to avoid the need for Balance::Div.
    let required = *packet_price.as_ref() * U256::from(total_deficit);
    let safety_num = U256::from((cfg.cost_safety_factor * 100.0) as u64);
    let required_with_safety = required * safety_num / U256::from(100);
    let total_funded_u256: &U256 = total_funded.as_ref();

    if *total_funded_u256 < required_with_safety {
        return DrainVerdict::Skip(SkipReason::UneconomicalDeposit);
    }

    // SURB sufficiency: we need at least `total_deficit` SURBs.
    if (surb_count as u64) < total_deficit {
        return DrainVerdict::Skip(SkipReason::InsufficientSurbs);
    }

    // Budget: min(surb_count, deficit + slack, floor(funded / cost))
    let budget_by_funding = if packet_price > HoprBalance::zero() {
        let cost = *packet_price.as_ref();
        if cost.is_zero() {
            u64::MAX
        } else {
            let count = total_funded_u256 / cost;
            count.low_u64()
        }
    } else {
        u64::MAX
    };

    let max_packets = (surb_count as u64)
        .min(total_deficit + cfg.surplus_slack_packets)
        .min(budget_by_funding);

    let deficits = candidates
        .into_iter()
        .map(|(i, _, d)| DrainSsaDeficit {
            guard_index: i,
            deficit: d,
        })
        .collect();

    DrainVerdict::Drain(DrainParams {
        max_packets,
        deficits,
    })
}
