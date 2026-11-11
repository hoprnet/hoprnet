//! The reordering a Session's incoming frames show, and the gap wait it supports.

use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::protocol::FrameId;

/// How long a reordering sample informs the gap wait.
///
/// Long enough to carry what a Session learned across an idle spell: a forward path that died
/// while nothing was sent shows only once data resumes. Short enough that reordering on paths
/// since replaced stops lengthening every later gap.
const REORDER_MEMORY: Duration = Duration::from_secs(60);

/// The gap wait as a multiple of the deepest reordering remembered.
const REORDER_FACTOR: u32 = 2;

/// The shortest gap wait reordering can bring the wait down to.
pub(crate) const REORDER_FLOOR: Duration = Duration::from_millis(250);

/// Abandoned gaps remembered, to tell a frame that comes after its gap was given up.
pub(crate) const ABANDONED_MEMORY: usize = 64;

/// The fewest samples, and the shortest time they must span within [`REORDER_MEMORY`], before the
/// gap wait is sized by them rather than kept at `max_wait`.
///
/// A window that trusts its first samples is sized by whatever paths those happened to take. On a
/// cluster, the Entry's first echo frames assembled within 24–66 ms while frames a second later took
/// 330 ms: the wait fell to its floor at once, and frames whose last segments came later than it
/// were given up, then their segments dropped as already seen, 15 % of a warm-up lost. Five seconds
/// span the paths a Session's first refreshes plan.
const MIN_SAMPLES: u32 = 8;
const MIN_OBSERVATION: Duration = Duration::from_secs(5);

/// The reordering a Session's incoming frames have shown, and the gap wait it supports.
///
/// Two kinds of sample, both a span over which data sent together arrived apart:
/// - the sequencer's: how long a frame came after the earliest frame already waiting behind it, because its gap closed
///   when it arrived or because it came after its gap was given up (which is what a wait too short for the Session
///   leads to, and what lengthens the wait again);
/// - the reassembler's: how long a frame of several segments took from its first segment to its last. Its segments are
///   sent together and take different paths, so this is the spread of the paths' latencies, and frames sent one after
///   another cannot arrive further apart than that. Frames that are many segments each rarely arrive out of order, so
///   without this a Session of such frames would have no sample at all.
///
/// A late arrival counts too: a segment that comes after its frame's gap was given up is noticed
/// where segments come in, since its frame never completes, and is a sample of how late it came.
///
/// The gap wait is [`REORDER_FACTOR`] times the deepest sample of the last [`REORDER_MEMORY`],
/// within `[REORDER_FLOOR, max_wait]`; it is `max_wait` until at least [`MIN_SAMPLES`] samples
/// span [`MIN_OBSERVATION`] of that memory.
#[derive(Debug, Default)]
struct ReorderWindow {
    /// The samples that are still the deepest of everything recorded after them, oldest first, so
    /// each is shallower than the one before it and the front is the deepest of the memory: a
    /// sliding-window maximum, constant time per sample however many frames arrive.
    deepest: VecDeque<(Instant, Duration)>,
    /// How many samples were recorded, per second of the memory: when its first sample came, and
    /// how many came within the second after it.
    counts: VecDeque<(Instant, u32)>,
    /// When the latest sample was recorded: the samples in the memory span from the first of
    /// [`counts`](Self::counts) to this.
    last: Option<Instant>,
    abandoned: VecDeque<(FrameId, Instant)>,
}

impl ReorderWindow {
    fn forget_before(&mut self, now: Instant) {
        let gone = |at: &Instant| now.saturating_duration_since(*at) > REORDER_MEMORY;
        while self.deepest.front().is_some_and(|(at, _)| gone(at)) {
            self.deepest.pop_front();
        }
        while self.counts.front().is_some_and(|(at, _)| gone(at)) {
            self.counts.pop_front();
        }
    }

    /// Whether the samples in the memory are enough to size the wait by: [`MIN_SAMPLES`] of them,
    /// the first and the latest [`MIN_OBSERVATION`] apart. It is the span of the samples that counts,
    /// not the age of the oldest: samples that all came in the first second of a Session that then
    /// went quiet span that second, however long ago it was.
    fn seen_enough(&self) -> bool {
        self.counts.iter().map(|(_, count)| *count).sum::<u32>() >= MIN_SAMPLES
            && self
                .counts
                .front()
                .zip(self.last)
                .is_some_and(|((first, _), last)| last.saturating_duration_since(*first) >= MIN_OBSERVATION)
    }
}

/// A [`ReorderWindow`] shared by a Session's reassembler and sequencer.
#[derive(Clone, Debug, Default)]
pub(crate) struct SharedReorder(Arc<parking_lot::Mutex<ReorderWindow>>);

impl SharedReorder {
    /// Records a span over which data sent together arrived apart.
    pub(crate) fn record(&self, now: Instant, sample: Duration) {
        let mut window = self.0.lock();
        window.forget_before(now);
        // A sample no deeper than this one, and older, can never be the deepest again.
        while window.deepest.back().is_some_and(|(_, deepest)| *deepest <= sample) {
            window.deepest.pop_back();
        }
        window.deepest.push_back((now, sample));
        match window.counts.back_mut() {
            Some((second, count)) if now.saturating_duration_since(*second) < Duration::from_secs(1) => *count += 1,
            _ => window.counts.push_back((now, 1)),
        }
        window.last = window.last.max(Some(now));
    }

    /// How long to wait on a gap: see [`ReorderWindow`].
    pub(crate) fn wait(&self, now: Instant, max_wait: Duration) -> Duration {
        let mut window = self.0.lock();
        window.forget_before(now);
        if !window.seen_enough() {
            return max_wait;
        }
        window
            .deepest
            .front()
            .map(|(_, deepest)| *deepest)
            .map_or(max_wait, |deepest| {
                deepest
                    .saturating_mul(REORDER_FACTOR)
                    .clamp(REORDER_FLOOR.min(max_wait), max_wait)
            })
    }

    /// Remembers that the gap at `id` was given up while the frame behind it had waited since
    /// `behind_since`.
    pub(crate) fn note_abandoned(&self, id: FrameId, behind_since: Instant) {
        let mut window = self.0.lock();
        if window.abandoned.len() == ABANDONED_MEMORY {
            window.abandoned.pop_front();
        }
        window.abandoned.push_back((id, behind_since));
    }

    /// A window that has already seen enough to size the wait by: [`MIN_SAMPLES`] samples of 1 ms
    /// spread over the [`MIN_OBSERVATION`] before the last second.
    #[cfg(test)]
    pub(crate) fn warmed() -> Self {
        let window = Self::default();
        let now = Instant::now();
        let first = now
            .checked_sub(MIN_OBSERVATION + Duration::from_secs(1))
            .expect("monotonic clock older than the observation");
        for i in 0..MIN_SAMPLES {
            window.record(
                first + MIN_OBSERVATION * i / (MIN_SAMPLES - 1),
                Duration::from_millis(1),
            );
        }
        window
    }

    /// How late `id` came, if its gap was given up.
    pub(crate) fn late(&self, id: FrameId, now: Instant) -> Option<Duration> {
        self.0
            .lock()
            .abandoned
            .iter()
            .find(|(abandoned, _)| *abandoned == id)
            .map(|(_, behind_since)| now.duration_since(*behind_since))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wait_should_be_max_wait_before_any_sample() {
        let window = SharedReorder::default();
        assert_eq!(
            Duration::from_secs(3),
            window.wait(Instant::now(), Duration::from_secs(3))
        );
    }

    /// A window that has seen [`MIN_SAMPLES`] shallow samples spread over [`MIN_OBSERVATION`],
    /// ending at the returned instant.
    fn warmed(window: &SharedReorder, start: Instant) -> Instant {
        for i in 0..MIN_SAMPLES {
            window.record(
                start + MIN_OBSERVATION * i / (MIN_SAMPLES - 1),
                Duration::from_millis(1),
            );
        }
        start + MIN_OBSERVATION
    }

    /// The first samples say little about the paths a Session will take: until enough of them span
    /// enough time, the wait stays at `max_wait`, however shallow they are and however long ago they
    /// came.
    #[test]
    fn the_wait_should_stay_at_max_wait_until_enough_samples_span_enough_time() {
        let window = SharedReorder::default();
        let start = Instant::now();
        let max_wait = Duration::from_secs(3);

        for i in 0..100u32 {
            window.record(start + Duration::from_millis(u64::from(i)), Duration::from_millis(20));
        }
        assert_eq!(
            max_wait,
            window.wait(start + Duration::from_secs(1), max_wait),
            "many, but within a second"
        );
        assert_eq!(
            max_wait,
            window.wait(start + MIN_OBSERVATION, max_wait),
            "many, and long ago, but all within a second: a Session that went quiet"
        );

        let few = SharedReorder::default();
        few.record(start, Duration::from_millis(20));
        few.record(start + MIN_OBSERVATION, Duration::from_millis(20));
        assert_eq!(
            max_wait,
            few.wait(start + MIN_OBSERVATION, max_wait),
            "long enough, but too few"
        );

        window.record(start + MIN_OBSERVATION, Duration::from_millis(20));
        assert_eq!(
            REORDER_FLOOR,
            window.wait(start + MIN_OBSERVATION, max_wait),
            "enough of both"
        );
    }

    #[test]
    fn the_wait_should_be_twice_the_deepest_sample_within_its_bounds() {
        let window = SharedReorder::default();
        let now = warmed(&window, Instant::now());
        let max_wait = Duration::from_secs(3);

        window.record(now, Duration::from_millis(50));
        assert_eq!(REORDER_FLOOR, window.wait(now, max_wait), "floored");

        window.record(now, Duration::from_millis(400));
        window.record(now, Duration::from_millis(300));
        assert_eq!(
            Duration::from_millis(800),
            window.wait(now, max_wait),
            "the deepest counts"
        );

        window.record(now, Duration::from_secs(2));
        assert_eq!(max_wait, window.wait(now, max_wait), "capped");

        assert_eq!(
            Duration::from_millis(100),
            window.wait(now, Duration::from_millis(100)),
            "a max_wait under the floor is kept"
        );
    }

    #[test]
    fn a_sample_should_be_forgotten_after_the_memory() {
        let window = SharedReorder::default();
        let then = warmed(&window, Instant::now());
        let max_wait = Duration::from_secs(3);

        window.record(then, Duration::from_millis(400));
        assert_eq!(Duration::from_millis(800), window.wait(then, max_wait));
        let later = then + REORDER_MEMORY + Duration::from_secs(1);
        assert_eq!(max_wait, window.wait(later, max_wait), "nothing left in the memory");

        warmed(&window, later);
        window.record(later + MIN_OBSERVATION, Duration::from_millis(200));
        assert_eq!(
            Duration::from_millis(400),
            window.wait(later + MIN_OBSERVATION, max_wait),
            "only the fresh one"
        );
    }

    /// However many frames arrive, the window keeps only the samples that can still be the deepest.
    #[test]
    fn the_window_should_keep_only_samples_that_can_still_be_the_deepest() {
        let window = SharedReorder::default();
        let start = Instant::now();
        let max_wait = Duration::from_secs(3);

        // Shrinking samples are each still the deepest of what follows them ...
        for i in 0..5u64 {
            window.record(start + Duration::from_millis(i), Duration::from_millis(500 - 100 * i));
        }
        assert_eq!(5, window.0.lock().deepest.len());
        // ... until a deeper one makes all of them moot.
        window.record(start + Duration::from_millis(10), Duration::from_millis(600));
        assert_eq!(1, window.0.lock().deepest.len());

        for i in 0..100_000u64 {
            window.record(
                start + Duration::from_millis(20 + i / 100),
                Duration::from_millis(i % 7),
            );
        }
        assert!(window.0.lock().deepest.len() <= 8, "{}", window.0.lock().deepest.len());
        // A shallow sample well past the observation, so that the samples span it before and after
        // the deep one is forgotten.
        let spanned = start + MIN_OBSERVATION + Duration::from_secs(2);
        window.record(spanned, Duration::from_millis(1));
        assert_eq!(Duration::from_millis(1200), window.wait(spanned, max_wait));

        // Once the deep one is past the memory, the next deepest takes over.
        let later = start + Duration::from_millis(10) + REORDER_MEMORY + Duration::from_millis(1);
        assert_eq!(REORDER_FLOOR, window.wait(later, max_wait));
    }

    #[test]
    fn an_abandoned_gap_should_be_remembered_for_a_bounded_number_of_gaps() {
        let window = SharedReorder::default();
        let then = Instant::now();
        for id in 1..=(ABANDONED_MEMORY as FrameId + 1) {
            window.note_abandoned(id, then);
        }
        let now = then + Duration::from_millis(700);
        assert_eq!(None, window.late(1, now), "the oldest is dropped");
        assert_eq!(Some(Duration::from_millis(700)), window.late(2, now));
        assert_eq!(None, window.late(1000, now), "never abandoned");
    }
}
