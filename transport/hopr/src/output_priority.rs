//! Priority scheduling of the three outgoing traffic classes.
//!
//! Every outgoing packet belongs to one of three lanes: session data, probes and noise (cover
//! traffic). The lanes are merged here, so that the class order is session data first, then
//! probes, then noise, and so that a probe burst cannot delay session packets.
//!
//! The order comes from two merges:
//!
//! 1. [`select_with_strategy`] merges the session and the probe lane. [`PacedPollState`] chooses the session lane for
//!    at most [`max_session_run`] slots in a row, then yields one probe slot. The probe lane therefore keeps the
//!    configured share of the slots while session data is queued, and gets every free slot while it is not.
//! 2. The result is merged with the noise lane behind a constant preference for the left input, so noise takes a slot
//!    only when the session and the probe lane are both empty.
//!
//! The strategy counts choices, not emissions: when the chosen lane has nothing to yield,
//! [`select_with_strategy`] falls through to the other lane, unseen by the counter. Both guarantees
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
    /// [`max_session_run`]: `0.1` allows 9 session slots between two probe slots.
    ///
    /// `0.0` gives probes a slot only while the session lane is empty. `1.0` gives probes the
    /// first slot of every run, which is one probe slot for every session slot.
    ///
    /// Valid values are finite and in `0.0..=1.0`.
    #[default = 0.1]
    #[validate(custom(function = "validate_probe_share"))]
    pub probe_share: f64,
}

/// Rejects a probe share outside `0.0..=1.0`, and the non-finite values that would otherwise pass a
/// plain range check.
fn validate_probe_share(share: f64) -> Result<(), validator::ValidationError> {
    if share.is_finite() && (0.0..=1.0).contains(&share) {
        Ok(())
    } else {
        let mut error = validator::ValidationError::new("probe_share_out_of_range");
        error.message = Some(format!("probe share {share} must be finite and in 0.0..=1.0").into());
        Err(error)
    }
}

/// Longest run of session slots that may precede one probe slot, for a probe share of `share`.
///
/// The share `f` is a fraction of all emission slots, so the run is `floor((1 - f) / f)`: `9` at
/// `0.1`, `4` at `0.2`, `3` at `0.25` and `1` at `0.5`. A share of `0.0` returns [`usize::MAX`],
/// which means probes take a slot only while the session lane is empty.
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
/// [`max_session_run`] slots in a row.
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
        let accepted = Pin::new(&mut self.inner).start_send(item);
        if accepted.is_ok() {
            self.depth.raise();
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

    /// A2: a queued burst of probes must not hold back one session packet, whatever its size and
    /// whatever the point of the pacing cycle the packet arrives in.
    #[test]
    fn session_packet_enters_ahead_of_a_probe_burst() {
        for burst in [650, MAXIMUM_PROBE_MSG_OUTGOING_BUFFER_SIZE] {
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

                // Drain `phase` slots first, so every position of the pacing cycle is covered. With
                // an idle session lane every slot is a probe.
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
                        "burst {burst} phase {phase}: the session packet is still queued after {waited} slots, more \
                         than the one owed probe slot"
                    );
                    match draw(&mut merged) {
                        Some(Class::Session) => break,
                        Some(Class::Probe) => drawn_probes += 1,
                        other => panic!("burst {burst} phase {phase}: unexpected slot {other:?}"),
                    }
                }
                assert!(waited <= 2, "the session packet took {waited} slots");
                assert_eq!(
                    burst - drawn_probes,
                    lanes.probes.len(),
                    "burst {burst} phase {phase}: the session packet must not consume the burst"
                );
            }
        }
    }

    /// A1: with both lanes continuously queued, the probe lane gets its configured share and no
    /// session run exceeds the derived bound.
    #[test]
    fn probe_share_holds_while_both_lanes_are_queued() {
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
            emitted.push(draw(&mut merged).expect("a queued lane must always yield a slot"));
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
    fn a_queued_probe_waits_at_most_one_session_run() {
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
                let item = draw(&mut merged).expect("the session lane is queued");
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
    }

    /// A5: a pong queued behind a full session lane is emitted inside one lane depth plus the probe
    /// slots owed between them, and no probe that is not owed overtakes it.
    #[test]
    fn a_pong_waits_inside_one_session_lane_depth() {
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
            let item = draw(&mut merged).expect("a queued lane must always yield a slot");
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
    }

    /// A8: with all three classes queued the order is session, then probe, then noise.
    #[test]
    fn the_class_order_is_session_then_probe_then_noise() {
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
            cycle.push(draw(&mut merged).expect("a queued lane must always yield a slot"));
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
    /// integer and shorten the run by one slot.
    #[test]
    fn max_session_run_follows_the_probe_share() {
        for (share, expected) in [(0.1, 9), (0.2, 4), (0.25, 3), (0.5, 1)] {
            assert_eq!(expected, max_session_run(share), "share {share}");
        }
        assert_eq!(
            usize::MAX,
            max_session_run(0.0),
            "a zero share means probes wait for an idle session lane"
        );
        assert_eq!(0, max_session_run(1.0), "a share of one gives probes the first slot");
    }

    /// A7: the default is a tenth, and the range is closed at both ends.
    #[test]
    fn probe_share_defaults_to_a_tenth_and_must_be_a_finite_share_of_one() {
        let default = OutputPriorityConfig::default();
        assert_eq!(0.1, default.probe_share);
        assert!(default.validate().is_ok(), "the default must be valid");

        for share in [0.0, 0.1, 0.5, 1.0] {
            let cfg = OutputPriorityConfig { probe_share: share };
            assert!(cfg.validate().is_ok(), "a share of {share} must be accepted");
        }
        for share in [-0.1, 1.1, f64::NAN, f64::INFINITY] {
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
    fn lane_depth_follows_enqueued_and_emitted_items() {
        let depth = LaneDepth::default();
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        let mut sink = DepthSink::new(futures::sink::drain::<u32>(), depth.clone());
        for value in 0..3u32 {
            assert!(matches!(Pin::new(&mut sink).poll_ready(&mut cx), Poll::Ready(Ok(()))));
            Sink::start_send(Pin::new(&mut sink), value).expect("drain accepts every item");
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
