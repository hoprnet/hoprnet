//! Priority ordering of the three outgoing traffic classes.
//!
//! Packets leave the node as session data, as probes, or as cover traffic (noise). The classes
//! reach the routing stage through two nested `select_with_strategy` merges, so their precedence
//! is fixed and explicit:
//!
//! 1. **Session data** first. Pongs count as session data: a peer waits on a pong, and a late one
//!    records a failed probe on the other side.
//! 2. **Probes** take a minimum share of the slots while session data is queued, and every free
//!    slot when it is not.
//! 3. **Noise** only when the session and probe lanes are both empty.
//!
//! A *slot* is one item the merged stream yields to the routing stage. Every packet is 3,669
//! bytes, so a share of slots is a share of bytes.
//!
//! Probes are paced rather than starved: [`paced_probe_share`] grants the probe lane one slot
//! after at most [`max_session_run`] consecutive session slots. That bounds both the wait of a
//! probe behind a saturated session lane and the wait of a session packet behind a queued probe
//! burst, which is one owed probe slot.
//!
//! The bounds hold while both lanes are continuously queued, which is what the tests drive. The
//! strategy counts choices, not emissions: when the chosen lane is empty, the merge falls through
//! to the other lane unseen by the counter, so transient occupancy only favours probes.

use futures::stream::PollNext;

#[cfg(all(feature = "telemetry", not(test)))]
use futures::Sink;
#[cfg(all(feature = "telemetry", not(test)))]
use std::{
    pin::Pin,
    task::{Context, Poll},
};

/// Default share of the slots given to probes while session data is queued.
pub const DEFAULT_PROBE_SHARE: f64 = 0.1;

/// Longest run of consecutive session-data slots between two probe slots at `probe_share`.
///
/// Derived as `floor((1 - share) / share)`: the number of session slots that fits before one probe
/// slot restores `share`. `0.0` returns [`usize::MAX`], meaning probes only while the session lane
/// is idle; `1.0` returns `0`, meaning probes first. NaN is treated as `0.0` and the infinities as
/// their nearest end of the range; the configuration rejects both before they reach here.
///
/// Computed as `floor(1 / share - 1)`, which is algebraically the same expression but avoids the
/// last-bit underflow of `(1 - share) / share` at values such as `0.05`.
pub fn max_session_run(probe_share: f64) -> usize {
    if probe_share.is_nan() || probe_share <= 0.0 {
        usize::MAX
    } else if probe_share >= 1.0 {
        0
    } else {
        (1.0f64 / probe_share - 1.0).floor() as usize
    }
}

/// Strategy for merging the session-data lane (`Left`) with the probe lane (`Right`).
///
/// The returned closure keeps the count of consecutive session choices, so it must be handed to
/// [`select_with_strategy`](futures::stream::select_with_strategy) exactly once and must not be
/// shared.
pub fn paced_probe_share(probe_share: f64) -> impl FnMut(&mut ()) -> PollNext {
    let max_session_run = max_session_run(probe_share);
    let mut session_run = 0usize;

    move |_: &mut ()| {
        if session_run < max_session_run {
            session_run = session_run.saturating_add(1);
            PollNext::Left
        } else {
            session_run = 0;
            PollNext::Right
        }
    }
}

/// Records one packet handed to the probe lane.
#[cfg(all(feature = "telemetry", not(test)))]
#[inline]
fn note_probe_lane_enqueued() {
    metrics::METRIC_PROBE_LANE_DEPTH.increment(1.0);
}

/// Records one packet emitted from the probe lane.
#[inline]
pub(crate) fn note_probe_lane_emitted() {
    #[cfg(all(feature = "telemetry", not(test)))]
    metrics::METRIC_PROBE_LANE_DEPTH.decrement(1.0);
}

/// Builds the sink the probe generator writes into.
///
/// Without the `telemetry` feature this is the inner sink itself.
pub(crate) fn probe_lane_sink<S>(inner: S) -> ProbeLaneSink<S> {
    #[cfg(all(feature = "telemetry", not(test)))]
    {
        metrics::initialize();
        ProbeLaneDepthSink { inner }
    }
    #[cfg(not(all(feature = "telemetry", not(test))))]
    {
        inner
    }
}

#[cfg(all(feature = "telemetry", not(test)))]
mod metrics {
    lazy_static::lazy_static! {
        pub(super) static ref METRIC_PROBE_LANE_DEPTH: hopr_api::types::telemetry::SimpleGauge =
            hopr_api::types::telemetry::SimpleGauge::new(
                "hopr_transport_probe_lane_depth",
                "Packets queued in the probe lane of the outgoing scheduler",
            )
            .unwrap();
    }

    /// Forces the gauge to be registered even when nothing has been queued yet.
    pub(super) fn initialize() {
        lazy_static::initialize(&METRIC_PROBE_LANE_DEPTH);
    }
}

/// Pass-through sender that keeps the probe-lane depth gauge current.
///
/// The depth is counted as enqueues minus emissions. A packet counts as enqueued when the probe
/// generator hands it over, which is at most one packet ahead of the channel, so the gauge may
/// read one high while a send is in flight.
#[cfg(all(feature = "telemetry", not(test)))]
#[derive(Debug)]
pub(crate) struct ProbeLaneDepthSink<S> {
    inner: S,
}

#[cfg(all(feature = "telemetry", not(test)))]
impl<S: Clone> Clone for ProbeLaneDepthSink<S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[cfg(all(feature = "telemetry", not(test)))]
impl<S: Sink<T> + Unpin, T> Sink<T> for ProbeLaneDepthSink<S> {
    type Error = S::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.get_mut().inner).poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        let this = self.get_mut();
        let result = Pin::new(&mut this.inner).start_send(item);
        if result.is_ok() {
            note_probe_lane_enqueued();
        }
        result
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.get_mut().inner).poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.get_mut().inner).poll_close(cx)
    }
}

/// The probe-lane sink: the inner sink itself unless the `telemetry` feature is on.
#[cfg(not(all(feature = "telemetry", not(test))))]
pub(crate) type ProbeLaneSink<S> = S;

#[cfg(all(feature = "telemetry", not(test)))]
pub(crate) type ProbeLaneSink<S> = ProbeLaneDepthSink<S>;

#[cfg(test)]
mod tests {
    use futures::{
        StreamExt,
        channel::mpsc,
        executor::block_on,
        stream::{self, PollNext, select_with_strategy},
    };

    use super::{DEFAULT_PROBE_SHARE, max_session_run, paced_probe_share};

    const SESSION: u8 = 0;
    const PROBE: u8 = 1;
    const NOISE: u8 = 2;

    /// Merges the session lane with the probe lane under the paced strategy, exactly as the
    /// transport does.
    fn data_merge<Sess, Probe>(session: Sess, probes: Probe, probe_share: f64) -> impl futures::Stream<Item = u8>
    where
        Sess: futures::Stream<Item = u8>,
        Probe: futures::Stream<Item = u8>,
    {
        select_with_strategy(session, probes, paced_probe_share(probe_share))
    }

    /// Merges the data merge with a noise stream, exactly as the transport does.
    fn three_way_merge<Sess, Probe, Noise>(
        session: Sess,
        probes: Probe,
        noise: Noise,
        probe_share: f64,
    ) -> impl futures::Stream<Item = u8>
    where
        Sess: futures::Stream<Item = u8>,
        Probe: futures::Stream<Item = u8>,
        Noise: futures::Stream<Item = u8>,
    {
        select_with_strategy(data_merge(session, probes, probe_share), noise, |_: &mut ()| {
            PollNext::Left
        })
    }

    fn longest_run(slots: &[u8], class: u8) -> usize {
        let (mut longest, mut current) = (0usize, 0usize);
        for slot in slots {
            if *slot == class {
                current += 1;
                longest = longest.max(current);
            } else {
                current = 0;
            }
        }
        longest
    }

    /// A1: both lanes continuously queued, a share of 0.1 puts 100 probes in 1,000 slots and never
    /// lets more than 9 session slots run in a row.
    #[test]
    fn probe_share_holds_under_sustained_load_on_both_lanes() {
        let slots = block_on(
            data_merge(stream::repeat(SESSION), stream::repeat(PROBE), 0.1)
                .take(1_000)
                .collect::<Vec<_>>(),
        );

        assert_eq!(1_000, slots.len());
        let probes = slots.iter().filter(|slot| **slot == PROBE).count();
        assert!(
            (99..=101).contains(&probes),
            "expected 100 probes (+-1) out of 1,000 slots, got {probes}"
        );
        assert!(
            longest_run(&slots, SESSION) <= 9,
            "session run of {} slots exceeds the derived bound of 9",
            longest_run(&slots, SESSION)
        );
    }

    /// A2: a session packet behind a queued probe burst waits at most one owed probe slot,
    /// whatever the burst size.
    #[test]
    fn session_packet_overtakes_a_queued_probe_burst() {
        for burst in [650usize, 1_024] {
            for drained in [0usize, 5, 8, 9] {
                let (session_tx, session_rx) = mpsc::unbounded::<u8>();
                let mut merged = Box::pin(data_merge(
                    session_rx,
                    stream::repeat(PROBE).take(burst),
                    DEFAULT_PROBE_SHARE,
                ));

                for _ in 0..drained {
                    assert_eq!(Some(PROBE), block_on(merged.next()));
                }

                session_tx
                    .unbounded_send(SESSION)
                    .expect("session lane must accept the packet");

                let first = block_on(merged.next());
                let second = block_on(merged.next());
                assert!(
                    first == Some(SESSION) || second == Some(SESSION),
                    "burst {burst}, drained {drained}: session packet not emitted within one owed probe slot, got \
                     {first:?} then {second:?}"
                );
            }
        }
    }

    /// A3: an idle session lane gives every slot to probes.
    #[test]
    fn idle_session_lane_gives_every_slot_to_probes() {
        let slots = block_on(
            data_merge(stream::empty(), stream::repeat(PROBE), DEFAULT_PROBE_SHARE)
                .take(50)
                .collect::<Vec<_>>(),
        );

        assert_eq!(vec![PROBE; 50], slots);
    }

    /// A3: with both data lanes empty, noise takes the slots.
    #[test]
    fn noise_flows_only_when_both_data_lanes_are_idle() {
        let slots = block_on(
            three_way_merge(
                stream::empty(),
                stream::empty(),
                stream::repeat(NOISE),
                DEFAULT_PROBE_SHARE,
            )
            .take(10)
            .collect::<Vec<_>>(),
        );

        assert_eq!(vec![NOISE; 10], slots);
    }

    /// A3 and A6: a queued session packet beats ready noise.
    #[test]
    fn session_data_beats_ready_noise() {
        let slots = block_on(
            three_way_merge(
                stream::iter(vec![SESSION, SESSION]),
                stream::empty(),
                stream::repeat(NOISE),
                DEFAULT_PROBE_SHARE,
            )
            .take(2)
            .collect::<Vec<_>>(),
        );

        assert_eq!(vec![SESSION, SESSION], slots);
    }

    /// A4: a probe queued behind a saturated session lane is emitted within one more than the
    /// derived session run.
    #[test]
    fn probe_is_not_starved_by_a_saturated_session_lane() {
        let bound = max_session_run(DEFAULT_PROBE_SHARE) + 1;
        let mut merged = Box::pin(data_merge(
            stream::repeat(SESSION),
            stream::iter(vec![PROBE]),
            DEFAULT_PROBE_SHARE,
        ));

        let mut slots = 0usize;
        loop {
            slots += 1;
            match block_on(merged.next()) {
                Some(PROBE) => break,
                Some(SESSION) => assert!(slots <= bound, "probe not emitted within {bound} slots"),
                other => panic!("unexpected slot {other:?}"),
            }
        }

        assert_eq!(bound, slots);
    }

    /// A5: a pong queued behind a full session lane leaves within the session packets ahead of it
    /// plus the probe slots owed between them.
    #[test]
    fn pong_behind_a_full_session_lane_leaves_within_the_budget() {
        const PONG: u8 = 9;
        const AHEAD: usize = 255;
        let session_slots = AHEAD as u64 / 9;
        let budget = AHEAD + session_slots as usize + 1;

        let session_lane = stream::iter((0..AHEAD).map(|_| SESSION).chain(std::iter::once(PONG)));
        let mut merged = Box::pin(data_merge(session_lane, stream::repeat(PROBE), DEFAULT_PROBE_SHARE));

        let mut slots = 0usize;
        let mut probes = 0usize;
        loop {
            slots += 1;
            match block_on(merged.next()) {
                Some(PONG) => break,
                Some(PROBE) => probes += 1,
                Some(SESSION) => {}
                other => panic!("unexpected slot {other:?}"),
            }
            assert!(slots <= budget, "pong not emitted within the {budget}-slot budget");
        }

        assert!(
            probes <= session_slots as usize,
            "{probes} probes took slots ahead of the pong, at most {session_slots} are owed"
        );
    }

    /// A6: noise never takes a slot while a data lane is queued.
    #[test]
    fn noise_never_preempts_a_queued_data_lane() {
        let slots = block_on(
            three_way_merge(
                stream::repeat(SESSION),
                stream::repeat(PROBE),
                stream::repeat(NOISE),
                0.1,
            )
            .take(200)
            .collect::<Vec<_>>(),
        );

        assert!(
            slots.iter().all(|slot| *slot != NOISE),
            "noise took a slot while session data and probes were queued"
        );
    }

    /// A8: with all three classes queued, the order of precedence is session, probe, then noise.
    #[test]
    fn class_order_is_session_then_probe_then_noise() {
        let slots = block_on(
            three_way_merge(
                stream::iter(vec![SESSION]),
                stream::iter(vec![PROBE]),
                stream::repeat(NOISE),
                DEFAULT_PROBE_SHARE,
            )
            .take(3)
            .collect::<Vec<_>>(),
        );

        assert_eq!(vec![SESSION, PROBE, NOISE], slots);
    }

    /// A7: the derived run pins the documented values, and the extreme shares keep their meaning.
    #[test]
    fn max_session_run_pins_the_documented_values() {
        assert_eq!(9, max_session_run(0.1));
        assert_eq!(4, max_session_run(0.2));
        assert_eq!(3, max_session_run(0.25));
        assert_eq!(1, max_session_run(0.5));
        assert_eq!(usize::MAX, max_session_run(0.0));
        assert_eq!(0, max_session_run(1.0));
    }

    /// A7: a share of zero gives probes only while the session lane is idle.
    #[test]
    fn zero_share_gives_no_probe_while_session_data_is_queued() {
        let slots = block_on(
            data_merge(stream::repeat(SESSION), stream::repeat(PROBE), 0.0)
                .take(100)
                .collect::<Vec<_>>(),
        );
        assert_eq!(vec![SESSION; 100], slots);

        let slots = block_on(
            data_merge(stream::empty(), stream::repeat(PROBE), 0.0)
                .take(10)
                .collect::<Vec<_>>(),
        );
        assert_eq!(vec![PROBE; 10], slots);
    }

    /// A7: a share of one gives probes every slot they can take.
    #[test]
    fn full_share_gives_probes_every_slot() {
        let slots = block_on(
            data_merge(stream::repeat(SESSION), stream::repeat(PROBE), 1.0)
                .take(100)
                .collect::<Vec<_>>(),
        );
        assert_eq!(vec![PROBE; 100], slots);
    }
}
