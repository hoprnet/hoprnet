//! What the Exit pays for on a peer's behalf, and the bounds put on it: the refusal reply budget and
//! the labels of `hopr_exit_control_packets_total`.

use std::time::{Duration, Instant};

use parking_lot::Mutex;

use crate::{Capabilities, Capability, supervision::telemetry::PixAdmissionRejection};

/// The default [`RefusalReplyBudget::burst`]: the most replies a flood can make the Exit pay for back
/// to back, while a briefly busy Exit still answers the Entries that arrive together.
pub const DEFAULT_REFUSAL_REPLY_BURST: u32 = 32;

/// The default [`RefusalReplyBudget::refill_period`].
pub const DEFAULT_REFUSAL_REPLY_REFILL_PERIOD: Duration = Duration::from_secs(1);

/// The shortest [`RefusalReplyBudget::refill_period`] accepted; a zero period would be no budget.
pub const MIN_REFUSAL_REPLY_REFILL_PERIOD: Duration = Duration::from_millis(1);

/// How many refusal replies an Exit sends, shared by every reason it refuses a `StartSession`.
///
/// A reply travels over a SURB the requester chose, and a multi-hop one costs the Exit a ticket while
/// the request reaches it for free, so unbounded replies let a flood drain the Exit's funds. One
/// budget covers all reasons because a peer picks whichever is cheapest to provoke.
///
/// A token bucket: up to [`burst`](Self::burst) replies at once, then one every
/// [`refill_period`](Self::refill_period). A refusal that finds it empty gets no reply and the
/// requester times out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, smart_default::SmartDefault, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RefusalReplyBudget {
    /// The bucket's capacity. Zero never answers a refusal.
    ///
    /// Default is [`DEFAULT_REFUSAL_REPLY_BURST`].
    #[default(DEFAULT_REFUSAL_REPLY_BURST)]
    pub burst: u32,

    /// How long it takes to earn one more reply once the burst is spent. Floored at
    /// [`MIN_REFUSAL_REPLY_REFILL_PERIOD`] by the manager.
    ///
    /// Default is [`DEFAULT_REFUSAL_REPLY_REFILL_PERIOD`].
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

/// A token bucket over a [`RefusalReplyBudget`], shared by every clone of the manager.
#[derive(Debug)]
pub(crate) struct RefusalReplyBucket {
    budget: RefusalReplyBudget,
    state: Mutex<BucketState>,
}

impl RefusalReplyBucket {
    /// Creates a bucket that starts full, raising a too-short refill period to the minimum.
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

    /// [`try_take`](Self::try_take) at a given instant, for tests without sleeping.
    fn try_take_at(&self, now: Instant) -> bool {
        let mut state = self.state.lock();
        self.refill(&mut state, now);

        if state.tokens == 0 {
            return false;
        }
        // A full bucket has no partial period to keep.
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
            // Advance by whole periods only, keeping partial progress so the long-run rate is exact.
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
/// labels it. Closed, so no call site can invent a label value.
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum ExitControlKind {
    /// Refused: PIX offered over a direct (0-hop) return path.
    RefusalDirectReturnPath,
    /// Refused: no PIX support installed.
    RefusalNoPixSupport,
    /// Refused: unacceptable PIX parameters.
    RefusalUnacceptableParams,
    /// Refused: the session server did not admit the target.
    RefusalTargetPolicy,
    /// Refused: the live-cycle budget cannot cover the Session.
    RefusalLiveCycleCapacity,
    /// Refused: no Session slot.
    RefusalNoSessionSlot,
    /// Refused: the node could not decide right now.
    RefusalBusy,
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

/// Whether the Session a packet belongs to offers PIX, as `hopr_exit_control_packets_total` labels it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum ExitSessionKind {
    /// Offers PIX.
    Pix,
    /// Does not.
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

/// What became of a packet, as `hopr_exit_control_packets_total` labels it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum ExitControlOutcome {
    /// Handed to the transport.
    Sent,
    /// Not sent: the budget was spent.
    DroppedBudget,
    /// The transport refused it or timed out.
    SendFailed,
}

/// Counts one packet the Exit sent, or declined to send, on a peer's behalf.
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

        // 1.5 periods in: one earned, half a period kept.
        assert!(bucket.try_take_at(t0 + SECOND * 3 / 2));
        assert!(!bucket.try_take_at(t0 + SECOND * 3 / 2));

        // The kept half completes at 2 s.
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
