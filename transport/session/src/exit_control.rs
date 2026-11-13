//! What the Exit pays for on behalf of a peer that asked for it, and the bounds put on it.
//!
//! The Exit sends every reply to a `StartSession` over the SURBs the requester supplied, and the
//! requester chooses the return path. A multi-hop return path costs the Exit one ticket per packet
//! (see the packet encoder in `hopr-protocol-hopr`), while the request itself and the SURBs it
//! carries can be delivered for free over a 0-hop forward path. A requester can therefore make the
//! Exit spend at the rate it can send requests, unless the replies are bounded.
//!
//! This module holds that bound for refusals ([`RefusalReplyBudget`], [`RefusalReplyBucket`]) and
//! the closed vocabulary the bounded `hopr_exit_control_packets_total` metric is labelled with
//! ([`ExitControlKind`], [`ExitSessionKind`], [`ExitControlOutcome`]). The metric records how many
//! such packets were sent, not what they cost: the SURB, and with it the path length and the ticket,
//! is chosen later in the packet pipeline.

use std::time::{Duration, Instant};

use parking_lot::Mutex;

use crate::{Capabilities, Capability, supervision::telemetry::PixAdmissionRejection};

/// The default [`RefusalReplyBudget::burst`].
///
/// What it bounds is the most replies a flood can make the Exit pay for back to back, before the
/// refill rate applies. It is large enough that an Exit that is merely full or briefly busy still
/// answers the honest Entries that arrive together.
pub const DEFAULT_REFUSAL_REPLY_BURST: u32 = 32;

/// The default [`RefusalReplyBudget::refill_period`].
pub const DEFAULT_REFUSAL_REPLY_REFILL_PERIOD: Duration = Duration::from_secs(1);

/// The shortest [`RefusalReplyBudget::refill_period`] the node accepts.
///
/// A zero period would earn a reply on every clock reading, which is no budget at all. A node that
/// wants no bound says so with `None` rather than with a degenerate period.
pub const MIN_REFUSAL_REPLY_REFILL_PERIOD: Duration = Duration::from_millis(1);

/// How many refusal replies an Exit sends, shared by every reason it refuses a `StartSession`.
///
/// Every refusal is answered with a `SessionError` that travels over the requester's SURB, and the
/// requester picks that SURB's path. A reply over a multi-hop path costs the Exit a ticket, so
/// without a bound a flood of requests that are going to be refused drains the Exit's funds at the
/// flood's rate. The bound is one budget for all reasons rather than one per reason, because a peer
/// can pick whichever reason is cheapest to provoke.
///
/// It is a token bucket: up to [`burst`](Self::burst) replies at once, and then one more every
/// [`refill_period`](Self::refill_period). A refusal that finds the bucket empty is dropped without
/// a reply, and the requester finds out by timing out, as it would on a lost packet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, smart_default::SmartDefault, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RefusalReplyBudget {
    /// The most refusal replies that can go out back to back, which is also the bucket's capacity.
    ///
    /// Zero means a refusal is never answered.
    ///
    /// Default is [`DEFAULT_REFUSAL_REPLY_BURST`].
    #[default(DEFAULT_REFUSAL_REPLY_BURST)]
    pub burst: u32,

    /// How long it takes to earn one more reply once the burst is spent, so the long-run rate is one
    /// reply per period.
    ///
    /// Floored at [`MIN_REFUSAL_REPLY_REFILL_PERIOD`] by the manager.
    ///
    /// Default is [`DEFAULT_REFUSAL_REPLY_REFILL_PERIOD`] (1 second).
    #[default(DEFAULT_REFUSAL_REPLY_REFILL_PERIOD)]
    #[serde(with = "humantime_serde")]
    pub refill_period: Duration,
}

#[derive(Debug)]
struct BucketState {
    tokens: u32,
    /// The instant the tokens were last topped up to, which is the start of the period now running.
    refilled_at: Instant,
}

/// A token bucket over a [`RefusalReplyBudget`].
///
/// Shared by every clone of the manager through an `Arc`, since the budget is the node's and not a
/// Session's. The lock is held for a few integer operations only.
#[derive(Debug)]
pub(crate) struct RefusalReplyBucket {
    budget: RefusalReplyBudget,
    state: Mutex<BucketState>,
}

impl RefusalReplyBucket {
    /// Creates a bucket that starts full.
    ///
    /// A [`refill_period`](RefusalReplyBudget::refill_period) below
    /// [`MIN_REFUSAL_REPLY_REFILL_PERIOD`] is raised to it.
    pub(crate) fn new(budget: RefusalReplyBudget) -> Self {
        Self::new_at(budget, Instant::now())
    }

    fn new_at(mut budget: RefusalReplyBudget, now: Instant) -> Self {
        budget.refill_period = budget.refill_period.max(MIN_REFUSAL_REPLY_REFILL_PERIOD);
        Self {
            budget,
            state: Mutex::new(BucketState {
                tokens: budget.burst,
                refilled_at: now,
            }),
        }
    }

    /// Takes one reply from the bucket, returning whether there was one.
    pub(crate) fn try_take(&self) -> bool {
        self.try_take_at(Instant::now())
    }

    /// [`try_take`](Self::try_take) at a given instant, so the refill can be tested without sleeping.
    fn try_take_at(&self, now: Instant) -> bool {
        let mut state = self.state.lock();
        self.refill(&mut state, now);

        if state.tokens == 0 {
            return false;
        }
        // A full bucket has no partial period to keep: the next one starts with this reply.
        if state.tokens == self.budget.burst {
            state.refilled_at = now;
        }
        state.tokens -= 1;
        true
    }

    fn refill(&self, state: &mut BucketState, now: Instant) {
        let period = self.budget.refill_period.as_nanos().max(1);
        let elapsed = now.saturating_duration_since(state.refilled_at).as_nanos();
        let periods = u32::try_from(elapsed / period).unwrap_or(u32::MAX);
        if periods == 0 {
            return;
        }

        state.tokens = state.tokens.saturating_add(periods).min(self.budget.burst);
        if state.tokens == self.budget.burst {
            state.refilled_at = now;
        } else {
            // Advance by whole periods only, so the part of a period already waited is not lost and
            // the long-run rate is exact. Not full means `periods` is small, and the product is
            // at most the elapsed time, so the checks are never the ones that decide.
            state.refilled_at = self
                .budget
                .refill_period
                .checked_mul(periods)
                .and_then(|advance| state.refilled_at.checked_add(advance))
                .unwrap_or(now);
        }
    }
}

/// What a packet the Exit sent on a peer's behalf was for, as `hopr_exit_control_packets_total`
/// labels it.
///
/// Closed, so that no call site can invent a label value. The `Refusal*` kinds answer a
/// `StartSession` that was declined; the others are what an admitted Session makes the Exit send.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum ExitControlKind {
    /// A refusal because the offer was over a direct (0-hop) return path.
    RefusalDirectReturnPath,
    /// A refusal because this node has no PIX support installed.
    RefusalNoPixSupport,
    /// A refusal because the offered PIX parameters were unacceptable.
    RefusalUnacceptableParams,
    /// A refusal because the session server did not admit the target.
    RefusalTargetPolicy,
    /// A refusal because the node's live-cycle budget could not cover the Session.
    RefusalLiveCycleCapacity,
    /// A refusal because the node has no Session slot to give.
    RefusalNoSessionSlot,
    /// A refusal because the node could not decide right now.
    RefusalBusy,
    /// The SURB-level report that tells the Entry's balancer how many SURBs the Exit holds.
    ///
    /// Backs off on an idle Session, see [`PixFillControl::admit`](crate::manager::PixFillControl::admit).
    SurbReport,
    /// A packet of the Exit's own PIX fill, sent above the report rate.
    PixFill,
    /// The `SessionEstablished` that answers an admitted `StartSession`.
    SessionEstablished,
    /// The first request for the Entry's SSA commitments.
    SsaRequest,
    /// The same SSA request, sent again because no answer came.
    SsaRequestResend,
    /// A request for the parts of one SSA's commitment that never arrived.
    CommitmentRetransmissionRequest,
    /// A `SessionError` about an established Session, sent over its return path.
    SessionError,
}

impl From<PixAdmissionRejection> for ExitControlKind {
    fn from(rejection: PixAdmissionRejection) -> Self {
        match rejection {
            PixAdmissionRejection::NoPixSupport => Self::RefusalNoPixSupport,
            PixAdmissionRejection::UnacceptableParams => Self::RefusalUnacceptableParams,
            PixAdmissionRejection::TargetPolicy => Self::RefusalTargetPolicy,
            PixAdmissionRejection::LiveCycleCapacity => Self::RefusalLiveCycleCapacity,
            PixAdmissionRejection::NoSessionSlot => Self::RefusalNoSessionSlot,
            PixAdmissionRejection::Busy => Self::RefusalBusy,
            PixAdmissionRejection::DirectReturnPath => Self::RefusalDirectReturnPath,
        }
    }
}

/// Whether the Session a packet belongs to uses PIX, as `hopr_exit_control_packets_total` labels it.
///
/// Splits the funded population from the rest, so a node can tell a flood of PIX offers from a flood
/// of plain Sessions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum ExitSessionKind {
    /// The Session offers PIX.
    Pix,
    /// The Session does not.
    Plain,
}

impl ExitSessionKind {
    /// The kind of a Session with these capabilities.
    pub(crate) fn of(capabilities: Capabilities) -> Self {
        if capabilities.contains(Capability::UsePIX) {
            Self::Pix
        } else {
            Self::Plain
        }
    }
}

/// What became of a packet the Exit was about to send, as `hopr_exit_control_packets_total` labels
/// it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum ExitControlOutcome {
    /// The packet was handed to the transport.
    Sent,
    /// The packet was not sent because the budget for it was spent.
    DroppedBudget,
    /// The transport did not take the packet, or timed out.
    SendFailed,
}

/// Counts one packet the Exit sent, or declined to send, on a peer's behalf.
///
/// The `telemetry` feature gates only the instrument. The probe in a test build is what lets the
/// default test configuration assert that this was called.
pub(crate) fn record_exit_control(kind: ExitControlKind, session: ExitSessionKind, outcome: ExitControlOutcome) {
    #[cfg(feature = "telemetry")]
    crate::telemetry::exit_control::record(kind, session, outcome);
    #[cfg(test)]
    crate::supervision::telemetry::probe::add(&format!("exit_control/{kind}/{session}/{outcome}"), 1);
    #[cfg(not(any(feature = "telemetry", test)))]
    let _ = (kind, session, outcome);
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECOND: Duration = Duration::from_secs(1);

    fn bucket(burst: u32, refill_period: Duration, now: Instant) -> RefusalReplyBucket {
        RefusalReplyBucket::new_at(RefusalReplyBudget { burst, refill_period }, now)
    }

    #[test]
    fn a_fresh_bucket_answers_its_burst_and_then_nothing() {
        let t0 = Instant::now();
        let bucket = bucket(3, SECOND, t0);

        assert!((0..3).all(|_| bucket.try_take_at(t0)));
        assert!(!bucket.try_take_at(t0));
        assert!(!bucket.try_take_at(t0 + SECOND / 2));
    }

    #[test]
    fn an_empty_bucket_earns_one_reply_per_refill_period() {
        let t0 = Instant::now();
        let bucket = bucket(2, SECOND, t0);
        assert!(bucket.try_take_at(t0));
        assert!(bucket.try_take_at(t0));

        // 1.5 periods in: one reply earned, half a period kept.
        assert!(bucket.try_take_at(t0 + SECOND * 3 / 2));
        assert!(!bucket.try_take_at(t0 + SECOND * 3 / 2));

        // The kept half period completes at 2 s, not at 2.5 s.
        assert!(!bucket.try_take_at(t0 + SECOND * 19 / 10));
        assert!(bucket.try_take_at(t0 + SECOND * 2));
        assert!(!bucket.try_take_at(t0 + SECOND * 2));
    }

    #[test]
    fn a_bucket_never_banks_more_than_its_burst() {
        let t0 = Instant::now();
        let bucket = bucket(3, SECOND, t0);

        let later = t0 + SECOND * 1000;
        assert!((0..3).all(|_| bucket.try_take_at(later)));
        assert!(!bucket.try_take_at(later));
    }

    #[test]
    fn a_zero_burst_never_answers() {
        let t0 = Instant::now();
        let bucket = bucket(0, SECOND, t0);

        assert!(!bucket.try_take_at(t0));
        assert!(!bucket.try_take_at(t0 + SECOND * 1000));
    }

    #[test]
    fn an_instant_before_the_last_refill_adds_nothing() {
        let t0 = Instant::now() + SECOND * 10;
        let bucket = bucket(1, SECOND, t0);
        assert!(bucket.try_take_at(t0));

        assert!(!bucket.try_take_at(t0 - SECOND * 5));
        assert!(bucket.try_take_at(t0 + SECOND));
    }

    #[test]
    fn a_refill_period_below_the_minimum_is_raised_to_it() {
        let t0 = Instant::now();
        let bucket = bucket(1, Duration::ZERO, t0);
        assert!(bucket.try_take_at(t0));

        assert!(!bucket.try_take_at(t0));
        assert!(bucket.try_take_at(t0 + MIN_REFUSAL_REPLY_REFILL_PERIOD));
    }

    #[test]
    fn the_refusal_reply_budget_defaults_to_a_burst_of_32_and_one_reply_a_second() {
        let budget = RefusalReplyBudget::default();

        assert_eq!(32, budget.burst);
        assert_eq!(Duration::from_secs(1), budget.refill_period);
    }

    #[test]
    fn every_admission_rejection_maps_to_a_distinct_refusal_kind() {
        let rejections = [
            PixAdmissionRejection::NoPixSupport,
            PixAdmissionRejection::UnacceptableParams,
            PixAdmissionRejection::TargetPolicy,
            PixAdmissionRejection::LiveCycleCapacity,
            PixAdmissionRejection::NoSessionSlot,
            PixAdmissionRejection::Busy,
            PixAdmissionRejection::DirectReturnPath,
        ];

        let labels: std::collections::BTreeSet<String> = rejections
            .into_iter()
            .map(|rejection| ExitControlKind::from(rejection).to_string())
            .collect();

        assert_eq!(rejections.len(), labels.len());
        assert!(labels.iter().all(|label| label.starts_with("refusal_")));
    }
}
