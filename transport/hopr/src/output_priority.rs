//! Priority scheduling of the three outgoing traffic classes.
//!
//! Every outgoing packet belongs to one of three lanes: session data, probes and noise (cover
//! traffic). The lanes are merged here, so that the class order is session data first, then
//! probes, then noise, and so that a probe burst cannot delay session packets.
//!
//! The order comes from two merges:
//!
//! 1. `select_with_strategy` merges the session and the probe lane. `PacedPollState` chooses the session lane for at
//!    most `max_session_run` slots in a row, then yields one probe slot. The probe lane therefore keeps the configured
//!    share of the slots while session data is queued, and gets every free slot while it is not.
//! 2. The result is merged with the noise lane behind a constant preference for the left input, so noise takes a slot
//!    only when the session and the probe lane are both empty.
//!
//! The strategy counts choices, not emissions: when the chosen lane has nothing to yield,
//! `select_with_strategy` falls through to the other lane, unseen by the counter. Both guarantees
//! hold while both lanes are queued, which is what the tests drive. Transient occupancy just
//! favours probes.
//!
//! Pongs count as session data. They ride the session lane, because a peer waits on a pong and a
//! late pong records a failed probe on the other side.

use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll},
};

use futures::{
    Sink, Stream,
    stream::{PollNext, select_with_strategy},
};
use validator::Validate;

#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
    /// Probes waiting in the probe lane for an emission slot. A reading that stays at the lane
    /// capacity means the probe emitter runs ahead of its share, so probes queue behind session
    /// data instead of pacing beside it.
    static ref METRIC_PROBE_LANE_DEPTH: hopr_api::types::telemetry::SimpleGauge =
        hopr_api::types::telemetry::SimpleGauge::new(
            "hopr_probe_queue_depth",
            "Probes waiting in the probe lane for an emission slot",
        )
        .unwrap();
}

/// Scheduling configuration for the outgoing traffic classes.
///
/// The class order is fixed and not configurable: session data (pongs included), then probes, then
/// noise. Only the probe share is configurable.
#[derive(Debug, Clone, Copy, PartialEq, smart_default::SmartDefault, Validate)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(default, deny_unknown_fields)
)]
pub struct OutputPriorityConfig {
    /// Share of the emission slots the probe lane keeps while session data is queued.
    ///
    /// The value is a fraction of all slots, so the longest run of session slots follows as
    /// `max_session_run`: `0.1` allows 9 session slots between two probe slots, and the maximum
    /// accepted value of `0.5` allows 1.
    ///
    /// `0.0` gives probes a slot only while the session lane is empty. Valid values are finite and in
    /// `0.0..=0.5`: a larger share leaves no session slot in the derived run, which would serve
    /// session data only while the probe lane is empty.
    #[default = 0.1]
    #[validate(custom(function = "validate_probe_share"))]
    pub probe_share: f64,
}

/// Rejects a probe share outside `0.0..=0.5`, and the non-finite values that would otherwise pass a
/// plain range check.
///
/// `0.5` is the largest share the scheduler can honour: above it the derived session run is 0, so the
/// probe lane would be polled first on every slot and session data would be served only while that
/// lane is empty.
fn validate_probe_share(share: f64) -> Result<(), validator::ValidationError> {
    if share.is_finite() && (0.0..=0.5).contains(&share) {
        Ok(())
    } else {
        let mut error = validator::ValidationError::new("probe_share_out_of_range");
        error.message = Some(format!("probe share {share} must be finite and in 0.0..=0.5").into());
        Err(error)
    }
}

/// Longest run of session slots that may precede one probe slot, for a probe share of `share`.
///
/// The share `f` is a fraction of all emission slots, so the run is `floor((1 - f) / f)`: `9` at
/// `0.1`, `4` at `0.2`, `3` at `0.25` and `1` at `0.5`. Flooring can only shorten the run, so the
/// probe lane never takes less than `f` of the slots. A share of `0.0` returns [`usize::MAX`], which
/// means probes take a slot only while the session lane is empty; the configuration validation caps
/// the share at `0.5`, so the run is never shorter than 1.
fn max_session_run(share: f64) -> usize {
    // A share of zero, and the `NaN` that no validated config carries but that must not reach the
    // cast, both mean: probes take a slot only while the session lane is empty.
    if !share.is_finite() || share <= 0.0 {
        return usize::MAX;
    }
    let run = (1.0 - share) / share;
    // The division can land just below an exact integer: a share of 0.05 gives 18.999999999999996,
    // and a plain floor would shorten the run by one slot.
    let nearest = run.round();
    let whole = if (run - nearest).abs() < 1e-9 {
        nearest
    } else {
        run.floor()
    };
    whole as usize
}

/// Chooses the lane to poll first on the next slot, holding the session lane to
/// `max_session_run` slots in a row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PacedPollState {
    max_session_run: usize,
    session_run: usize,
}

impl PacedPollState {
    fn new(share: f64) -> Self {
        Self {
            max_session_run: max_session_run(share),
            session_run: 0,
        }
    }

    /// Returns the lane to poll first on the next slot: the probe lane once `max_session_run`
    /// session choices ran back to back, the session lane otherwise.
    fn next_side(&mut self) -> PollNext {
        if self.session_run >= self.max_session_run {
            self.session_run = 0;
            PollNext::Right
        } else {
            self.session_run = self.session_run.saturating_add(1);
            PollNext::Left
        }
    }
}

/// Merges the session, probe and noise lanes into one stream, in that class order.
///
/// `probe_share` is the fraction of emission slots the probe lane keeps while session data is
/// queued, as described on [`OutputPriorityConfig::probe_share`].
pub(crate) fn merge_lanes<S, P, N>(session: S, probes: P, noise: N, probe_share: f64) -> impl Stream<Item = S::Item>
where
    S: Stream,
    P: Stream<Item = S::Item>,
    N: Stream<Item = S::Item>,
{
    let mut state = PacedPollState::new(probe_share);
    let prioritized = select_with_strategy(session, probes, move |_: &mut ()| state.next_side());
    select_with_strategy(prioritized, noise, |_: &mut ()| PollNext::Left)
}

/// Number of items enqueued in a lane but not yet emitted.
///
/// The count follows the lane endpoints only, so a send the channel accepts and then drops because
/// the receiver went away leaves the count high. The lane is torn down with the transport, so no
/// operator reads the figure after that.
#[derive(Debug, Clone, Default)]
pub(crate) struct LaneDepth(Arc<AtomicUsize>);

impl LaneDepth {
    /// Number of items waiting in the lane. The transport exports the figure as a gauge; only the
    /// tests read it back.
    #[cfg(test)]
    pub(crate) fn get(&self) -> usize {
        self.0.load(Ordering::Relaxed)
    }

    fn raise(&self) {
        self.record(self.0.fetch_add(1, Ordering::Relaxed) + 1);
    }

    fn lower(&self) {
        let depth = self
            .0
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |depth| {
                Some(depth.saturating_sub(1))
            })
            .map(|previous| previous.saturating_sub(1))
            .unwrap_or_default();
        self.record(depth);
    }

    /// Publishes a new depth, so that the exported gauge and the counter never disagree.
    fn record(&self, depth: usize) {
        #[cfg(all(feature = "telemetry", not(test)))]
        METRIC_PROBE_LANE_DEPTH.set(depth as f64);
        #[cfg(not(all(feature = "telemetry", not(test))))]
        let _ = depth;
    }
}

/// A [`Sink`] that raises the depth of the lane behind it for every item it accepts.
pub(crate) struct DepthSink<S> {
    inner: S,
    depth: LaneDepth,
}

impl<S> DepthSink<S> {
    /// Wraps a lane sender and the depth shared with the lane receiver.
    pub(crate) fn new(inner: S, depth: LaneDepth) -> Self {
        Self { inner, depth }
    }
}

impl<S: Clone> Clone for DepthSink<S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            depth: self.depth.clone(),
        }
    }
}

impl<S, T> Sink<T> for DepthSink<S>
where
    S: Sink<T> + Unpin,
{
    type Error = S::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        // Count first, then hand the item over, and take the count back if the sink refuses it. The
        // other order loses one count per refusal: the receiver can lower the depth between the
        // accepted item and the `raise`, and the later `raise` then leaves a non-zero depth behind
        // an empty lane.
        self.depth.raise();
        let accepted = Pin::new(&mut self.inner).start_send(item);
        if accepted.is_err() {
            self.depth.lower();
        }
        accepted
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_close(cx)
    }
}

/// A [`Stream`] that lowers the depth of the lane behind it for every item it yields.
pub(crate) struct DepthStream<S> {
    inner: S,
    depth: LaneDepth,
}

impl<S> DepthStream<S> {
    /// Wraps a lane receiver and the depth shared with the lane sender.
    pub(crate) fn new(inner: S, depth: LaneDepth) -> Self {
        Self { inner, depth }
    }
}

impl<S: Stream + Unpin> Stream for DepthStream<S> {
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let polled = Pin::new(&mut self.inner).poll_next(cx);
        if matches!(polled, Poll::Ready(Some(_))) {
            self.depth.lower();
        }
        polled
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::VecDeque, rc::Rc};

    use anyhow::Context as _;

    use super::*;
    use crate::constants::{MAXIMUM_MSG_OUTGOING_BUFFER_SIZE, MAXIMUM_PROBE_MSG_OUTGOING_BUFFER_SIZE};

    /// The traffic class a merged slot belongs to.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Class {
        Session,
        /// A pong answers a ping the peer already sent, so it rides the session lane.
        Pong,
        Probe,
        Noise,
    }

    struct LaneState<T> {
        queue: VecDeque<T>,
        capacity: usize,
    }

    /// One lane stub. It reports `Pending` while empty, never `None`: a completed lane leaves the
    /// merge for good, and every test here needs the lane back once it is refilled.
    struct Lane<T> {
        state: Rc<RefCell<LaneState<T>>>,
    }

    /// Test-side handle on a lane, for queueing items and reading its depth.
    #[derive(Clone)]
    struct LaneHandle<T> {
        state: Rc<RefCell<LaneState<T>>>,
    }

    impl<T> LaneHandle<T> {
        fn new(capacity: usize) -> Self {
            Self {
                state: Rc::new(RefCell::new(LaneState {
                    queue: VecDeque::new(),
                    capacity,
                })),
            }
        }

        /// Queues one item, and returns `false` when the lane holds `capacity` items.
        fn push(&self, item: T) -> bool {
            let mut state = self.state.borrow_mut();
            if state.queue.len() >= state.capacity {
                return false;
            }
            state.queue.push_back(item);
            true
        }

        /// Queues items until the lane is at capacity.
        fn keep_full(&self, item: T)
        where
            T: Copy,
        {
            while self.push(item) {}
        }

        fn len(&self) -> usize {
            self.state.borrow().queue.len()
        }

        fn stream(&self) -> Lane<T> {
            Lane {
                state: Rc::clone(&self.state),
            }
        }
    }

    impl<T: Unpin> Stream for Lane<T> {
        type Item = T;

        fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<T>> {
            match self.state.borrow_mut().queue.pop_front() {
                Some(item) => Poll::Ready(Some(item)),
                None => Poll::Pending,
            }
        }
    }

    /// Three lanes fed by the constants the transport uses.
    struct Lanes {
        session: LaneHandle<Class>,
        probes: LaneHandle<Class>,
        noise: LaneHandle<Class>,
    }

    impl Lanes {
        fn new() -> Self {
            Self {
                session: LaneHandle::new(MAXIMUM_MSG_OUTGOING_BUFFER_SIZE),
                probes: LaneHandle::new(MAXIMUM_PROBE_MSG_OUTGOING_BUFFER_SIZE),
                noise: LaneHandle::new(16),
            }
        }
    }

    /// Draws one slot from the merged stream, or `None` when every lane is empty.
    fn draw<S: Stream + Unpin>(merged: &mut S) -> Option<S::Item> {
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        match Pin::new(merged).poll_next(&mut cx) {
            Poll::Ready(item) => item,
            Poll::Pending => None,
        }
    }

    /// A2: a full probe lane must not hold back one session packet, whatever point of the pacing
    /// cycle the packet arrives in. The burst is the lane itself, which is the most the lane can hold
    /// at one time; a longer round is paced by back-pressure and covered by
    /// [`a_probe_round_never_queues_more_than_the_probe_lane_holds`].
    #[test]
    fn session_packet_enters_ahead_of_a_probe_burst() {
        let burst = MAXIMUM_PROBE_MSG_OUTGOING_BUFFER_SIZE;
        for phase in 0..10 {
            let lanes = Lanes::new();
            for _ in 0..burst {
                assert!(lanes.probes.push(Class::Probe), "the burst must fit the probe lane");
            }
            let mut merged = Box::pin(merge_lanes(
                lanes.session.stream(),
                lanes.probes.stream(),
                lanes.noise.stream(),
                0.1,
            ));

            // Drain `phase` slots first, so every position of the pacing cycle is covered. With an
            // idle session lane every slot is a probe.
            let mut drawn_probes = 0;
            for _ in 0..phase {
                assert_eq!(Some(Class::Probe), draw(&mut merged));
                drawn_probes += 1;
            }

            assert!(lanes.session.push(Class::Session));
            let mut waited = 0;
            loop {
                waited += 1;
                assert!(
                    waited <= 2,
                    "phase {phase}: the session packet is still queued after {waited} slots, more than the one owed \
                     probe slot"
                );
                match draw(&mut merged) {
                    Some(Class::Session) => break,
                    Some(Class::Probe) => drawn_probes += 1,
                    other => panic!("phase {phase}: unexpected slot {other:?}"),
                }
            }
            assert!(waited <= 2, "the session packet took {waited} slots");
            assert_eq!(
                burst - drawn_probes,
                lanes.probes.len(),
                "phase {phase}: the session packet must not consume the burst"
            );
        }
    }

    /// A1: with both lanes continuously queued, the probe lane gets its configured share and no
    /// session run exceeds the derived bound.
    #[test]
    fn probe_share_holds_while_both_lanes_are_queued() -> anyhow::Result<()> {
        let lanes = Lanes::new();
        lanes.session.keep_full(Class::Session);
        lanes.probes.keep_full(Class::Probe);

        let mut merged = Box::pin(merge_lanes(
            lanes.session.stream(),
            lanes.probes.stream(),
            lanes.noise.stream(),
            0.1,
        ));

        let mut emitted = Vec::with_capacity(1_000);
        for _ in 0..1_000 {
            emitted.push(draw(&mut merged).context("a queued lane must always yield a slot")?);
            lanes.session.keep_full(Class::Session);
            lanes.probes.keep_full(Class::Probe);
        }

        let probe_slots = emitted.iter().filter(|class| **class == Class::Probe).count();
        assert!(
            (99..=101).contains(&probe_slots),
            "probes took {probe_slots} of 1,000 slots at a share of 0.1"
        );
        assert_eq!(
            9,
            longest_session_run(&emitted),
            "a share of 0.1 allows 9 session slots between two probe slots"
        );
        assert!(
            !emitted.contains(&Class::Noise),
            "noise must not take a slot while another lane is queued"
        );
        Ok(())
    }

    /// A3 (first part): an idle session lane gives the probe lane every slot.
    #[test]
    fn an_idle_session_lane_gives_probes_every_slot() {
        let lanes = Lanes::new();
        lanes.probes.keep_full(Class::Probe);
        lanes.noise.keep_full(Class::Noise);

        let mut merged = Box::pin(merge_lanes(
            lanes.session.stream(),
            lanes.probes.stream(),
            lanes.noise.stream(),
            0.1,
        ));

        for slot in 0..1_000 {
            assert_eq!(
                Some(Class::Probe),
                draw(&mut merged),
                "slot {slot} must be a probe while the session lane is empty"
            );
            lanes.probes.keep_full(Class::Probe);
            lanes.noise.keep_full(Class::Noise);
        }
    }

    /// A3 (second and third parts) and A6: noise flows only while both other lanes are empty, and
    /// session data still precedes it when the session lane is the only queued lane.
    #[test]
    fn noise_takes_a_slot_only_while_both_lanes_are_empty() {
        let lanes = Lanes::new();
        lanes.noise.keep_full(Class::Noise);
        let mut merged = Box::pin(merge_lanes(
            lanes.session.stream(),
            lanes.probes.stream(),
            lanes.noise.stream(),
            0.1,
        ));
        assert_eq!(
            Some(Class::Noise),
            draw(&mut merged),
            "noise must flow while both other lanes are empty"
        );

        let lanes = Lanes::new();
        assert!(lanes.session.push(Class::Session));
        lanes.noise.keep_full(Class::Noise);
        let mut merged = Box::pin(merge_lanes(
            lanes.session.stream(),
            lanes.probes.stream(),
            lanes.noise.stream(),
            0.1,
        ));
        assert_eq!(
            Some(Class::Session),
            draw(&mut merged),
            "session data must precede noise"
        );
    }

    /// A4: a probe never starves behind a session lane that is always queued.
    #[test]
    fn a_queued_probe_waits_at_most_one_session_run() -> anyhow::Result<()> {
        for phase in 0..10 {
            let lanes = Lanes::new();
            lanes.session.keep_full(Class::Session);
            let mut merged = Box::pin(merge_lanes(
                lanes.session.stream(),
                lanes.probes.stream(),
                lanes.noise.stream(),
                0.1,
            ));

            for _ in 0..phase {
                assert_eq!(Some(Class::Session), draw(&mut merged));
                lanes.session.keep_full(Class::Session);
            }

            assert!(lanes.probes.push(Class::Probe));
            let mut waited = 0;
            loop {
                waited += 1;
                let item = draw(&mut merged).context("the session lane is queued")?;
                lanes.session.keep_full(Class::Session);
                if item == Class::Probe {
                    break;
                }
                assert!(
                    waited <= 10,
                    "phase {phase}: the probe waited {waited} slots, more than one session run plus its own slot"
                );
            }
            assert!(
                waited <= 10,
                "phase {phase}: the probe waited {waited} slots, more than one session run plus its own slot"
            );
        }
        Ok(())
    }

    /// A5: a pong queued behind a full session lane is emitted inside one lane depth plus the probe
    /// slots owed between them, and no probe that is not owed overtakes it.
    #[test]
    fn a_pong_waits_inside_one_session_lane_depth() -> anyhow::Result<()> {
        let lanes = Lanes::new();
        for _ in 0..MAXIMUM_MSG_OUTGOING_BUFFER_SIZE - 1 {
            assert!(lanes.session.push(Class::Session));
        }
        lanes.probes.keep_full(Class::Probe);
        let mut merged = Box::pin(merge_lanes(
            lanes.session.stream(),
            lanes.probes.stream(),
            lanes.noise.stream(),
            0.1,
        ));

        // The pong takes the one free place in the session lane.
        assert!(lanes.session.push(Class::Pong), "one place must be free");

        let mut slots = 0;
        let mut probes_ahead = 0;
        loop {
            slots += 1;
            let item = draw(&mut merged).context("a queued lane must always yield a slot")?;
            lanes.probes.keep_full(Class::Probe);
            match item {
                Class::Pong => break,
                Class::Probe => probes_ahead += 1,
                Class::Session => {}
                Class::Noise => panic!("noise overtook the session lane"),
            }
            assert!(slots <= 284, "the pong is still queued after {slots} slots");
        }

        assert!(slots <= 284, "the pong took {slots} slots, above the 284 bound");
        assert!(
            probes_ahead <= 28,
            "{probes_ahead} probes overtook the pong, more than the probe slots owed over one lane depth"
        );
        Ok(())
    }

    /// A8: with all three classes queued the order is session, then probe, then noise.
    #[test]
    fn the_class_order_is_session_then_probe_then_noise() -> anyhow::Result<()> {
        let lanes = Lanes::new();
        lanes.session.keep_full(Class::Session);
        lanes.probes.keep_full(Class::Probe);
        lanes.noise.keep_full(Class::Noise);
        let mut merged = Box::pin(merge_lanes(
            lanes.session.stream(),
            lanes.probes.stream(),
            lanes.noise.stream(),
            0.1,
        ));

        // One full pacing cycle, both other lanes staying queued.
        let mut cycle = Vec::with_capacity(10);
        for _ in 0..10 {
            cycle.push(draw(&mut merged).context("a queued lane must always yield a slot")?);
            lanes.session.keep_full(Class::Session);
            lanes.probes.keep_full(Class::Probe);
            lanes.noise.keep_full(Class::Noise);
        }
        let mut expected = vec![Class::Session; 9];
        expected.push(Class::Probe);
        assert_eq!(expected, cycle, "one cycle is nine session slots and one probe slot");

        // Now drain everything that is queued, without refilling. Each class shows up only after
        // the classes above it are gone.
        let lanes = Lanes::new();
        for _ in 0..8 {
            assert!(lanes.session.push(Class::Session));
        }
        for _ in 0..3 {
            assert!(lanes.probes.push(Class::Probe));
        }
        for _ in 0..2 {
            assert!(lanes.noise.push(Class::Noise));
        }
        let mut merged = Box::pin(merge_lanes(
            lanes.session.stream(),
            lanes.probes.stream(),
            lanes.noise.stream(),
            0.1,
        ));
        for _ in 0..8 {
            assert_eq!(Some(Class::Session), draw(&mut merged));
        }
        for _ in 0..3 {
            assert_eq!(Some(Class::Probe), draw(&mut merged));
        }
        for _ in 0..2 {
            assert_eq!(Some(Class::Noise), draw(&mut merged));
        }
        Ok(())
    }

    /// A7: a share of `0.0` gives probes a slot only while the session lane is empty.
    #[test]
    fn a_zero_share_gives_probes_only_while_the_session_lane_is_empty() {
        let lanes = Lanes::new();
        lanes.session.keep_full(Class::Session);
        lanes.probes.keep_full(Class::Probe);
        let mut merged = Box::pin(merge_lanes(
            lanes.session.stream(),
            lanes.probes.stream(),
            lanes.noise.stream(),
            0.0,
        ));

        for _ in 0..1_000 {
            assert_eq!(
                Some(Class::Session),
                draw(&mut merged),
                "a zero share leaves no probe slot while session data is queued"
            );
            lanes.session.keep_full(Class::Session);
        }

        // Emptying the session lane hands every slot to the probes.
        while lanes.session.len() > 0 {
            assert_eq!(Some(Class::Session), draw(&mut merged));
        }
        assert_eq!(Some(Class::Probe), draw(&mut merged));
    }

    /// A7: the derived run length is pinned, because the f64 division can land one below an exact
    /// integer and shorten the run by one slot. Only shares the configuration accepts are pinned.
    #[test]
    fn max_session_run_follows_the_probe_share() {
        for (share, expected) in [(0.1, 9), (0.2, 4), (0.25, 3), (0.5, 1)] {
            assert_eq!(expected, max_session_run(share), "share {share}");
        }
        // Shares whose run is not a whole number floor down, so the probe lane keeps more than its
        // share: 0.3 gives 2.33, and the run is 2.
        for (share, expected) in [(0.3, 2), (0.35, 1), (0.4, 1)] {
            assert_eq!(expected, max_session_run(share), "share {share}");
        }
        assert_eq!(
            usize::MAX,
            max_session_run(0.0),
            "a zero share means probes wait for an idle session lane"
        );
    }

    /// A7: the default is a tenth, and the range is capped at one probe slot for every session slot.
    #[test]
    fn probe_share_defaults_to_a_tenth_and_accepts_half_at_most() {
        let default = OutputPriorityConfig::default();
        assert_eq!(0.1, default.probe_share);
        assert!(default.validate().is_ok(), "the default must be valid");

        for share in [0.0, 0.1, 0.5] {
            let cfg = OutputPriorityConfig { probe_share: share };
            assert!(cfg.validate().is_ok(), "a share of {share} must be accepted");
        }
        for share in [-0.1, 0.51, 1.0, 1.1, f64::NAN, f64::INFINITY] {
            let cfg = OutputPriorityConfig { probe_share: share };
            assert!(cfg.validate().is_err(), "a share of {share} must be rejected");
        }
    }

    /// A7: the default config is pinned as an operator writes it.
    #[cfg(feature = "serde")]
    #[test]
    fn output_priority_config_default_is_pinned() {
        insta::assert_yaml_snapshot!(OutputPriorityConfig::default());
    }

    /// A9: the lane depth follows what was enqueued and not yet emitted.
    #[test]
    fn lane_depth_follows_enqueued_and_emitted_items() -> anyhow::Result<()> {
        let depth = LaneDepth::default();
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        let mut sink = DepthSink::new(futures::sink::drain::<u32>(), depth.clone());
        for value in 0..3u32 {
            assert!(matches!(Pin::new(&mut sink).poll_ready(&mut cx), Poll::Ready(Ok(()))));
            Sink::start_send(Pin::new(&mut sink), value).context("drain accepts every item")?;
        }
        assert_eq!(3, depth.get(), "three enqueued items must raise the depth");

        let mut stream = DepthStream::new(futures::stream::iter([1u32]), depth.clone());
        assert_eq!(Some(1), draw(&mut stream));
        assert_eq!(2, depth.get());
        assert_eq!(None, draw(&mut stream), "the stub lane holds one item");
        assert_eq!(2, depth.get(), "an empty poll must not lower the depth");

        // An emission the sink never accounted for must clamp at zero, not wrap around.
        let depth = LaneDepth::default();
        let mut stream = DepthStream::new(futures::stream::iter([1u32]), depth.clone());
        assert_eq!(Some(1), draw(&mut stream));
        assert_eq!(0, depth.get());
        Ok(())
    }

    /// A refused item must take the depth back, or the count drifts up once per refusal.
    #[test]
    fn a_refused_send_leaves_the_lane_depth_alone() {
        struct RefusingSink;

        #[derive(Debug, PartialEq, Eq)]
        struct Refused;

        impl Sink<u32> for RefusingSink {
            type Error = Refused;

            fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }

            fn start_send(self: Pin<&mut Self>, _item: u32) -> Result<(), Self::Error> {
                Err(Refused)
            }

            fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }

            fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }
        }

        let depth = LaneDepth::default();
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        let mut sink = DepthSink::new(RefusingSink, depth.clone());

        assert!(matches!(Pin::new(&mut sink).poll_ready(&mut cx), Poll::Ready(Ok(()))));
        assert!(Sink::start_send(Pin::new(&mut sink), 1).is_err());
        assert_eq!(0, depth.get(), "a refused item must not raise the depth");
    }

    /// The lanes the transport builds: a send must wake a pending merge, and the session lane
    /// closing must leave the probe lane serving it.
    #[tokio::test]
    async fn a_real_lane_pair_wakes_the_merge_and_survives_the_other_lane_closing() -> anyhow::Result<()> {
        use anyhow::Context as _;
        use futures::{SinkExt, StreamExt};

        let (session_tx, session_rx) =
            hopr_utils::network_types::crossfire_sink::bounded_sink_channel::<u8>(MAXIMUM_MSG_OUTGOING_BUFFER_SIZE);
        let (probe_tx, probe_rx) = hopr_utils::network_types::crossfire_sink::bounded_sink_channel::<u8>(
            MAXIMUM_PROBE_MSG_OUTGOING_BUFFER_SIZE,
        );
        let depth = LaneDepth::default();
        let mut probe_tx = DepthSink::new(probe_tx, depth.clone());
        let probe_rx = DepthStream::new(probe_rx, depth.clone());

        let mut merged = Box::pin(merge_lanes(session_rx, probe_rx, futures::stream::pending::<u8>(), 0.1));

        // Both lanes empty: the merge is pending, not finished.
        assert!(matches!(futures::poll!(merged.next()), Poll::Pending));

        let mut session_tx = session_tx;
        session_tx.send(1u8).await.context("session lane send")?;
        assert_eq!(Some(1), merged.next().await);

        // A probe send wakes the pending merge, and the depth follows the item through the lane.
        probe_tx.send(2u8).await.context("probe lane send")?;
        assert_eq!(1, depth.get());
        assert_eq!(Some(2), merged.next().await);
        assert_eq!(0, depth.get());

        // The session lane closing leaves the probe lane serving the merge.
        drop(session_tx);
        probe_tx.send(3u8).await.context("probe lane send")?;
        assert_eq!(Some(3), merged.next().await);
        Ok(())
    }

    /// A slot drawn from a merge of the real probe lane and a synthetic session lane.
    ///
    /// A probe carries its index, so the test can match an emission to its send.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Slot {
        Session,
        Probe(usize),
    }

    /// A quiet-time probe round: `count` neighbor probes to one destination.
    struct ProbeRound {
        destination: hopr_api::types::crypto::types::OffchainPublicKey,
        pseudonym: hopr_api::types::internal::prelude::HoprPseudonym,
        hops: hopr_api::types::internal::routing::RoutingOptions,
        count: usize,
    }

    impl hopr_api::ct::ProbingTrafficGeneration for ProbeRound {
        fn build(&self) -> futures::stream::BoxStream<'static, hopr_api::ct::ProbeRouting> {
            use futures::StreamExt as _;
            use hopr_api::ct::ProbeRouting;

            let destination = self.destination;
            let pseudonym = self.pseudonym;
            let hops = self.hops.clone();
            futures::stream::iter(0..self.count)
                .map(move |_| {
                    ProbeRouting::Neighbor(hopr_api::types::internal::routing::DestinationRouting::Forward {
                        destination: Box::new(destination.into()),
                        pseudonym: Some(pseudonym),
                        forward_options: hops.clone(),
                        return_options: Some(hops.clone()),
                    })
                })
                .boxed()
        }
    }

    /// A saturated session lane merged with the probe lane as the transport builds it.
    struct RealLanes {
        merged: std::pin::Pin<Box<dyn Stream<Item = Slot>>>,
        session: LaneHandle<Slot>,
        probe_tx: DepthSink<hopr_utils::network_types::crossfire_sink::CrossfireSink<Slot>>,
        depth: LaneDepth,
    }

    fn real_lanes() -> RealLanes {
        let session = LaneHandle::<Slot>::new(MAXIMUM_MSG_OUTGOING_BUFFER_SIZE);
        session.keep_full(Slot::Session);
        let (probe_tx, probe_rx) = hopr_utils::network_types::crossfire_sink::bounded_sink_channel::<Slot>(
            MAXIMUM_PROBE_MSG_OUTGOING_BUFFER_SIZE,
        );
        let depth = LaneDepth::default();
        let probe_tx = DepthSink::new(probe_tx, depth.clone());
        let probe_rx = DepthStream::new(probe_rx, depth.clone());
        RealLanes {
            merged: Box::pin(merge_lanes(
                session.stream(),
                probe_rx,
                futures::stream::pending::<Slot>(),
                0.1,
            )),
            session,
            probe_tx,
            depth,
        }
    }

    /// Waits until a producer has filled the probe lane.
    ///
    /// Without it a draw can start against a lane that is empty only because the producer task has
    /// not been scheduled yet, which measures the harness rather than the lane.
    async fn wait_for_full_lane(depth: &LaneDepth) -> anyhow::Result<()> {
        for _ in 0..100_000 {
            if depth.get() >= MAXIMUM_PROBE_MSG_OUTGOING_BUFFER_SIZE {
                return Ok(());
            }
            tokio::task::yield_now().await;
        }
        anyhow::bail!("the probe lane never filled")
    }

    /// A10: a 650-probe round against a saturated session lane never leaves more than one lane of
    /// probes waiting, and no probe waits longer than one full lane can take at the default share.
    ///
    /// The lane is the real bounded channel, so the producer blocks once it is full and the round is
    /// paced by back-pressure instead of queueing as a whole.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_probe_round_never_queues_more_than_the_probe_lane_holds() -> anyhow::Result<()> {
        use std::sync::atomic::AtomicUsize;

        use anyhow::Context as _;
        use futures::{SinkExt, StreamExt};

        const ROUND: usize = 650;
        // A probe that just left its send has at most one full lane of probes ahead of it, and each
        // probe slot is at most `max_session_run + 1` = 10 slots away.
        const BOUND_SLOTS: usize = MAXIMUM_PROBE_MSG_OUTGOING_BUFFER_SIZE * 10;

        let lanes = real_lanes();
        let RealLanes {
            mut merged,
            session,
            mut probe_tx,
            depth,
        } = lanes;

        let slots = Arc::new(AtomicUsize::new(0));
        let sent_at: Arc<Vec<AtomicUsize>> = Arc::new((0..ROUND).map(|_| AtomicUsize::new(usize::MAX)).collect());

        let producer = {
            let slots = Arc::clone(&slots);
            let sent_at = Arc::clone(&sent_at);
            tokio::spawn(async move {
                let mut sent = 0;
                for index in 0..ROUND {
                    if probe_tx.send(Slot::Probe(index)).await.is_err() {
                        break;
                    }
                    // The slot count once the send returned: the probe's timer starts here. A probe
                    // the consumer takes before this store lands was emitted at once, so the read
                    // below falls back to no wait instead of a negative one.
                    sent_at[index].store(slots.load(Ordering::SeqCst), Ordering::SeqCst);
                    sent += 1;
                }
                sent
            })
        };

        wait_for_full_lane(&depth).await?;

        let mut received = 0;
        let mut deepest = 0;
        while received < ROUND {
            slots.fetch_add(1, Ordering::SeqCst);
            let item = merged.next().await.context("the session lane is always queued")?;
            session.keep_full(Slot::Session);
            deepest = deepest.max(depth.get());
            if let Slot::Probe(index) = item {
                let sent = sent_at[index].load(Ordering::SeqCst);
                let waited = if sent == usize::MAX {
                    0
                } else {
                    slots.load(Ordering::SeqCst).saturating_sub(sent)
                };
                assert!(
                    waited <= BOUND_SLOTS,
                    "probe {index} waited {waited} slots after its send returned, above the {BOUND_SLOTS} bound"
                );
                received += 1;
            }
        }
        let sent = producer.await.context("the producer task panicked")?;
        assert_eq!(ROUND, sent, "the producer must have sent the whole round");
        assert!(
            deepest <= MAXIMUM_PROBE_MSG_OUTGOING_BUFFER_SIZE + 1,
            "{deepest} probes were counted in the lane, above the {} the lane and its sink can hold",
            MAXIMUM_PROBE_MSG_OUTGOING_BUFFER_SIZE + 1
        );
        Ok(())
    }

    /// A11, first half: the real lane keeps the probe share when the producer always has a probe
    /// waiting.
    ///
    /// The producer is polled once per slot, which is a fair share of the scheduler and removes the
    /// runtime from the measurement: what is measured is how many slots a refill costs after a pop.
    /// A real slot runs the routing and encoding stages, so a producer is never slower than that.
    #[tokio::test]
    async fn the_real_probe_lane_keeps_its_share_with_a_producer_that_never_waits() -> anyhow::Result<()> {
        use anyhow::Context as _;
        use futures::{SinkExt, StreamExt};

        let RealLanes {
            mut merged,
            session,
            mut probe_tx,
            depth,
        } = real_lanes();

        // The producer always has the next probe ready.
        let producer = async {
            let mut index = 0usize;
            while probe_tx.send(Slot::Probe(index)).await.is_ok() {
                index += 1;
            }
        };
        futures::pin_mut!(producer);

        // Fill the lane before the first draw, so the run does not measure the producer's start-up.
        for _ in 0..1_000 {
            if depth.get() >= MAXIMUM_PROBE_MSG_OUTGOING_BUFFER_SIZE {
                break;
            }
            let _ = futures::poll!(producer.as_mut());
        }
        assert!(
            depth.get() >= MAXIMUM_PROBE_MSG_OUTGOING_BUFFER_SIZE,
            "the producer could not fill the probe lane"
        );

        let mut probes = 0;
        for _ in 0..1_000 {
            let _ = futures::poll!(producer.as_mut());
            if let Slot::Probe(_) = merged.next().await.context("the session lane is always queued")? {
                probes += 1;
            }
            session.keep_full(Slot::Session);
        }
        assert!(
            (99..=101).contains(&probes),
            "the real probe lane delivered {probes} probes in 1,000 slots at a share of 0.1"
        );
        Ok(())
    }

    /// A11, second half: the real probe emitter, with `max_parallel_probes` sends in flight, also
    /// keeps the share.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn the_real_probe_emitter_keeps_its_share() -> anyhow::Result<()> {
        use anyhow::Context as _;
        use futures::StreamExt;
        use hopr_api::types::internal::{prelude::HoprPseudonym, routing::DestinationRouting};
        use hopr_protocol_app::prelude::ReservedTag;

        use crate::Keypair;

        const ROUND: usize = 650;

        let allocator = hopr_transport_tag_allocator::create_allocators_from_config(&Default::default())?
            .pop()
            .context("missing probe tag allocator")?
            .1;
        let destination = *crate::OffchainKeypair::from_secret(&[0x11; 32])
            .context("valid destination secret")?
            .public();
        let graph = hopr_network_graph::ChannelGraph::new(
            *crate::OffchainKeypair::from_secret(&[0x22; 32])
                .context("valid graph secret")?
                .public(),
        );
        let pseudonym = HoprPseudonym::from([0x33u8; 10]);
        let hops =
            hopr_api::types::internal::routing::RoutingOptions::Hops(0u8.try_into().context("0 is a valid hop count")?);

        let (probe_tx, probe_rx) = hopr_utils::network_types::crossfire_sink::bounded_sink_channel::<(
            DestinationRouting,
            crate::ApplicationDataOut,
        )>(MAXIMUM_PROBE_MSG_OUTGOING_BUFFER_SIZE);
        let depth = LaneDepth::default();
        let probe_tx = DepthSink::new(probe_tx, depth.clone());
        let probe_rx = DepthStream::new(probe_rx, depth.clone());

        let (processes, _classifier) = crate::Probe::new(crate::config::ProbeConfig::default(), allocator)
            .continuously_scan(
                probe_tx,
                futures::stream::pending(),
                ProbeRound {
                    destination,
                    pseudonym,
                    hops: hops.clone(),
                    count: ROUND,
                },
                graph,
            )
            .await;

        // The probe lane carries probes only, so a session item is told apart by its tag.
        let session_item = (
            DestinationRouting::Forward {
                destination: Box::new(destination.into()),
                pseudonym: Some(pseudonym),
                forward_options: hops.clone(),
                return_options: Some(hops),
            },
            crate::ApplicationDataOut::with_no_packet_info(crate::ApplicationData::new(1234u64, b"session data")?),
        );
        let is_probe = |item: &(DestinationRouting, crate::ApplicationDataOut)| {
            item.1.data.application_tag == ReservedTag::Ping.into()
        };

        let mut merged = Box::pin(merge_lanes(
            futures::stream::repeat(session_item),
            probe_rx,
            futures::stream::pending(),
            0.1,
        ));

        wait_for_full_lane(&depth).await?;

        let mut probes = 0;
        for _ in 0..1_000 {
            let item = merged.next().await.context("the session lane is always queued")?;
            if is_probe(&item) {
                probes += 1;
            }
            // One slot costs the routing and encoding stages in the transport. Even this much delay
            // already beats the emitter by two orders of magnitude, so the emitter is not the
            // bottleneck the test could accidentally measure.
            futures_timer::Delay::new(std::time::Duration::from_micros(20)).await;
        }
        processes.abort_all();
        assert!(
            (99..=101).contains(&probes),
            "the real probe emitter delivered {probes} probes in 1,000 slots at a share of 0.1"
        );
        Ok(())
    }

    /// Longest run of consecutive session slots in the emitted sequence.
    fn longest_session_run(emitted: &[Class]) -> usize {
        let mut longest = 0;
        let mut current = 0;
        for class in emitted {
            if *class == Class::Session {
                current += 1;
                longest = longest.max(current);
            } else {
                current = 0;
            }
        }
        longest
    }
}
