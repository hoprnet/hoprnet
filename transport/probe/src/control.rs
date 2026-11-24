//! Runtime control of the probing rate.
//!
//! The probe traffic generator hands out a probe whenever it is pulled and keeps no clock of its
//! own, so the rate is set where the pull happens: in [`Probe::continuously_scan`](crate::Probe).
//! [`ProbeControl`] is the handle an application uses to change that rate while the node runs.

use std::{sync::Arc, time::Duration};

use futures::{Stream, StreamExt};
use tokio::{sync::watch, time::Instant};

/// Multiplier on the probing rate.
///
/// A dial of `m` spaces generated probes `pace / m` apart, where `pace` is [`ProbeConfig::pace`](crate::ProbeConfig).
/// It carries any positive multiplier, not only the presets. Probing off is [`ProbeControl::set_dial`] with `None`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Dial(f64);

impl Dial {
    /// The base rate.
    pub const HIGH: Self = Self(1.0);
    /// One tenth of the base rate.
    pub const LOW: Self = Self(0.1);
    /// Half of the base rate.
    pub const MEDIUM: Self = Self(0.5);
    /// Twice the base rate. A short burst over the base rate, such as a new session that needs path
    /// evidence fast. The default dial sits above it.
    pub const ULTRA: Self = Self(2.0);

    /// A dial at `multiplier`, or `None` when it is not a positive finite number.
    ///
    /// Zero and negatives are rejected, not clamped: a dial of zero would stop probing through the value that
    /// is meant to keep it running, which is what `None` is for.
    pub fn new(multiplier: f64) -> Option<Self> {
        (multiplier.is_finite() && multiplier > 0.0).then_some(Self(multiplier))
    }

    /// The multiplier this dial carries.
    pub const fn multiplier(self) -> f64 {
        self.0
    }
}

impl Default for Dial {
    /// Three times the base rate: with the default probe share of 0.4, the lowest pair that held a
    /// stalling relayer session stable in integration runs.
    fn default() -> Self {
        Self(3.0)
    }
}

impl TryFrom<f64> for Dial {
    type Error = &'static str;

    fn try_from(multiplier: f64) -> Result<Self, Self::Error> {
        Self::new(multiplier).ok_or("dial multiplier must be a positive finite number")
    }
}

/// Cloneable handle that sets the probing rate of a running node.
///
/// A change reaches the running probe loop on its next pull, with no rebuild.
#[derive(Debug, Clone)]
pub struct ProbeControl {
    dial: Arc<watch::Sender<Option<Dial>>>,
}

impl Default for ProbeControl {
    fn default() -> Self {
        Self {
            dial: Arc::new(watch::channel(Some(Dial::default())).0),
        }
    }
}

impl ProbeControl {
    /// Sets the dial. `None` stops generated probing; manual pings are not affected.
    pub fn set_dial(&self, dial: Option<Dial>) {
        self.dial.send_if_modified(|d| std::mem::replace(d, dial) != dial);
    }

    /// The current dial, `None` while generated probing is stopped.
    pub fn dial(&self) -> Option<Dial> {
        *self.dial.borrow()
    }

    pub(crate) fn subscribe(&self) -> watch::Receiver<Option<Dial>> {
        self.dial.subscribe()
    }
}

/// Longest wait between two probes. A near-zero dial would otherwise overflow `Duration` or `Instant`.
const MAX_GAP: Duration = Duration::from_secs(24 * 60 * 60);

/// Spaces the items of `inner` `pace / dial` apart.
///
/// The first item is not delayed. Each later item waits until `pace / dial` after the previous one, and `inner` is
/// polled only once that time is up, so a generator that reads state on each pull sees it at release time. A dial
/// change during a wait applies at once; a `None` dial holds the stream until a dial is set again.
pub(crate) fn paced<S>(inner: S, control: &ProbeControl, pace: Duration) -> impl Stream<Item = S::Item> + use<S>
where
    S: Stream + Unpin,
{
    futures::stream::unfold(
        // The stream owns a handle: with every other handle dropped the dial keeps its last value, and probing
        // must go on while that value is on.
        (inner, control.subscribe(), control.clone(), None::<Instant>),
        move |(mut inner, mut dial, control, last)| async move {
            loop {
                let multiplier = match dial.wait_for(Option::is_some).await {
                    Ok(dial) => dial.map_or(Dial::HIGH.multiplier(), Dial::multiplier),
                    // Unreachable while `control` is held: the channel cannot close.
                    Err(_) => return None,
                };
                let Some(last) = last else { break };
                let gap = Duration::try_from_secs_f64(pace.as_secs_f64() / multiplier)
                    .map_or(MAX_GAP, |gap| gap.min(MAX_GAP));
                let due = last + gap;
                // A deadline in the past still costs a timer tick (1 ms), which would cap a zero pace at 1000/s.
                if Instant::now() >= due {
                    break;
                }

                tokio::select! {
                    () = tokio::time::sleep_until(due) => break,
                    changed = dial.changed() => if changed.is_err() { return None },
                }
            }

            let item = inner.next().await?;
            Some((item, (inner, dial, control, Some(Instant::now()))))
        },
    )
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use super::*;

    const PACE: Duration = Duration::from_millis(100);

    /// A stream that is always ready and counts how often it was polled for an item.
    fn counting() -> (impl Stream<Item = usize> + Unpin, Arc<AtomicUsize>) {
        let polled = Arc::new(AtomicUsize::new(0));
        let counter = polled.clone();
        let stream = futures::stream::unfold((), move |()| {
            let counter = counter.clone();
            async move { Some((counter.fetch_add(1, Ordering::SeqCst), ())) }
        });
        (Box::pin(stream), polled)
    }

    async fn time_to_take(n: usize, dial: Dial) -> Duration {
        let control = ProbeControl::default();
        control.set_dial(Some(dial));
        let (inner, _) = counting();
        let mut stream = std::pin::pin!(paced(inner, &control, PACE));

        let start = Instant::now();
        for _ in 0..n {
            stream.next().await.expect("the stream never ends");
        }
        start.elapsed()
    }

    #[tokio::test(start_paused = true)]
    async fn the_first_probe_is_not_delayed() {
        assert_eq!(time_to_take(1, Dial::LOW).await, Duration::ZERO);
    }

    #[tokio::test(start_paused = true)]
    async fn a_dial_of_one_spaces_probes_one_pace_apart() {
        assert_eq!(time_to_take(4, Dial::HIGH).await, PACE * 3);
    }

    #[tokio::test(start_paused = true)]
    async fn a_higher_dial_spaces_probes_closer_together() {
        assert_eq!(time_to_take(4, Dial::ULTRA).await, PACE.div_f64(2.0) * 3);
    }

    #[tokio::test(start_paused = true)]
    async fn a_lower_dial_spaces_probes_further_apart() {
        assert_eq!(time_to_take(3, Dial::LOW).await, PACE * 10 * 2);
    }

    #[tokio::test(start_paused = true)]
    async fn a_dial_change_applies_to_the_next_gap() {
        let control = ProbeControl::default();
        let (inner, _) = counting();
        let mut stream = std::pin::pin!(paced(inner, &control, PACE));

        stream.next().await;
        stream.next().await;
        control.set_dial(Some(Dial::ULTRA));
        let start = Instant::now();
        stream.next().await;

        assert_eq!(start.elapsed(), PACE.div_f64(2.0));
    }

    #[tokio::test(start_paused = true)]
    async fn a_dial_change_during_a_wait_shortens_that_wait() {
        let control = ProbeControl::default();
        control.set_dial(Some(Dial::LOW));
        let (inner, _) = counting();
        let mut stream = std::pin::pin!(paced(inner, &control, PACE));

        stream.next().await;
        let start = Instant::now();
        let raise = async {
            tokio::time::sleep(PACE).await;
            control.set_dial(Some(Dial::HIGH));
        };
        let take = async {
            stream.next().await;
            start.elapsed()
        };
        let (waited, ()) = futures::join!(take, raise);

        // The gap is 1 s at LOW and due at 100 ms at HIGH, and the dial rises at 100 ms.
        assert_eq!(waited, PACE);
    }

    #[tokio::test(start_paused = true)]
    async fn probing_off_holds_the_stream_until_a_dial_is_set() {
        let control = ProbeControl::default();
        let (inner, polled) = counting();
        let mut stream = std::pin::pin!(paced(inner, &control, PACE));

        stream.next().await;
        control.set_dial(None);
        let held = tokio::time::timeout(PACE * 100, stream.next()).await;
        assert!(held.is_err(), "a stopped dial must not yield probes");
        assert_eq!(
            polled.load(Ordering::SeqCst),
            1,
            "the generator must not be pulled while stopped"
        );

        control.set_dial(Some(Dial::HIGH));
        assert!(stream.next().await.is_some());
    }

    #[tokio::test(start_paused = true)]
    async fn the_generator_is_pulled_only_when_a_probe_is_due() {
        let control = ProbeControl::default();
        // The default dial pulls the second probe sooner than the wait below, so the test runs at
        // the base rate, where the second probe is due a full pace after the first.
        control.set_dial(Some(Dial::HIGH));
        let (inner, polled) = counting();
        let mut stream = std::pin::pin!(paced(inner, &control, PACE));

        stream.next().await;
        let waiting = tokio::time::timeout(PACE / 2, stream.next()).await;

        assert!(waiting.is_err());
        assert_eq!(polled.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn the_stream_keeps_running_when_every_handle_is_dropped() {
        let control = ProbeControl::default();
        let (inner, _) = counting();
        let mut stream = std::pin::pin!(paced(inner, &control, PACE));
        drop(control);

        for _ in 0..3 {
            assert!(stream.next().await.is_some(), "the dial is on, so probing goes on");
        }
    }

    #[tokio::test(start_paused = true)]
    async fn a_dial_near_zero_waits_a_bounded_time_instead_of_panicking() {
        let control = ProbeControl::default();
        control.set_dial(Dial::new(1e-30));
        let (inner, _) = counting();
        let mut stream = std::pin::pin!(paced(inner, &control, Duration::MAX));

        stream.next().await;
        let held = tokio::time::timeout(Duration::from_secs(1), stream.next()).await;

        assert!(held.is_err());
    }

    /// The default dial is three times the base rate: with the default probe share of 0.4, the
    /// pair that held a stalling relayer session stable in integration runs.
    #[test]
    fn the_default_dial_is_three_times_the_base_rate() {
        assert_eq!(Dial::default().multiplier(), 3.0);
        assert_eq!(ProbeControl::default().dial(), Some(Dial(3.0)));
    }

    #[test]
    fn a_dial_must_be_a_positive_finite_number() {
        assert!(Dial::new(0.0).is_none());
        assert!(Dial::new(-1.0).is_none());
        assert!(Dial::new(f64::NAN).is_none());
        assert!(Dial::new(f64::INFINITY).is_none());
        assert_eq!(Dial::new(3.5).map(Dial::multiplier), Some(3.5));
    }
}
