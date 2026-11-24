//! The stage that turns a [`DestinationRouting`] into a [`ResolvedTransportRouting`] for every
//! packet this node originates.
//!
//! It sits between the merged outgoing-data stream and the SPHINX encoder, so **every** packet the
//! node originates passes through it: session payloads, Start-protocol replies, SURB keep-alives,
//! probes and cover traffic. Forwarded packets and acknowledgements do not — they are relayed from
//! inside the ingress pipeline and never reach this stage.
//!
//! That asymmetry is why this stage deserves its own tests: a fault here silences origination
//! node-wide while forwarding, acking and receiving carry on looking perfectly healthy.
//!
//! # Where the SURB wait happens
//!
//! A return packet that finds no SURB waits up to `surb_resolution_wait`. That wait must not happen
//! inside the ordered stream: [`buffered`](futures::StreamExt::buffered) withholds every packet
//! submitted behind an unfinished one, so one session out of SURBs stalled all the traffic the node
//! originated, once per starved packet.
//!
//! A packet that finds no SURB is therefore parked in a [`HoldLane`], keyed by its pseudonym, and
//! its future ends at once. The head of each pseudonym is retried every [`SURB_RETRY_INTERVAL`]
//! outside that stream and emitted as soon as its SURBs arrive, so the wait delays only the session
//! that is waiting.
//!
//! Order is kept where it is required: a packet never overtakes an earlier one of the same
//! pseudonym, because the entry's reassembler discards frames that arrive after `frame_timeout`.
//! Packets of different pseudonyms no longer wait for each other.

use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::{Duration, Instant},
};

use futures::{Stream, StreamExt};
use hopr_api::types::internal::{
    prelude::HoprPseudonym,
    routing::{DestinationRouting, ResolvedTransportRouting},
};
use hopr_crypto_packet::prelude::{HoprSurb, PacketSignal};
use hopr_protocol_app::prelude::ApplicationDataOut;
use tracing::{Instrument, error, trace, warn};

use super::errors::{PathPlannerError, Result as PathResult};

/// How often a return route whose SURBs have not arrived is retried.
const SURB_RETRY_INTERVAL: Duration = Duration::from_millis(5);

/// The most packets a [`HoldLane`] keeps for one pseudonym.
///
/// A parked packet waits at most `surb_resolution_wait` (1 s on hoprd), so this bound fills only
/// while a session submits faster than its SURBs arrive. Past it a packet is dropped and counted
/// like one that timed out: a long outage must not pin memory for as long as it lasts.
const MAX_PARKED_PER_PSEUDONYM: usize = 64;

/// Default for [`PacketPipelineConfig::surb_resolution_wait`][cfg], used when it is unset.
///
/// A momentary gap is normal — the SURB pool refills asynchronously, and dropping a return packet
/// on the first miss loses data on a session that was only waiting. That is why the retry exists.
///
/// What it must not do is wait indefinitely. When the counterparty is gone no SURB is ever coming.
///
/// Six seconds is deliberately generous, because this is a *library* default and the useful ceiling
/// belongs to the caller. A packet held past the session's frame timeout — 3 s for the sessions
/// hoprd runs — cannot be used by the receiver anyway, so a deployment that knows its own timeout
/// should configure something tighter, as hoprd does. What a library default must not do is drop a
/// packet a slower or reliable session would still have made use of, so it errs long and leaves the
/// tightening to whoever knows the traffic.
///
/// [cfg]: crate::protocol::PacketPipelineConfig::surb_resolution_wait
pub(crate) const DEFAULT_SURB_RESOLUTION_WAIT: Duration = Duration::from_secs(6);

/// The wait a node should use, given what its configuration says.
///
/// Unset means the default; zero is honoured as "do not wait", which drops a return packet the
/// first time its SURBs are missing. Kept as one function so both meanings live in one place
/// rather than being re-derived at the call site.
pub(crate) fn surb_resolution_wait(configured: Option<Duration>) -> Duration {
    configured.unwrap_or(DEFAULT_SURB_RESOLUTION_WAIT)
}

/// What a successful resolution carries beyond the resolved routing.
type Resolved = (ResolvedTransportRouting<HoprSurb>, Option<usize>, Option<u8>);

/// One packet this stage has to resolve, in the context it was submitted under.
type Originated = crate::trace_context::WithContext<(ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)>;

/// A packet that has to wait for its return route, with the deadlines that bound the wait.
struct ParkedPacket {
    /// Routing that found no SURB, retried unchanged until it resolves or the deadline passes.
    routing: DestinationRouting,
    data: ApplicationDataOut,
    /// Context the packet was submitted under; the retries run inside it.
    span: tracing::Span,
    /// Context the packet is handed on under once it resolves.
    output_span: tracing::Span,
    /// Past this instant the packet is dropped instead of retried.
    deadline: Instant,
    /// Earliest instant of the next attempt.
    next_attempt: Instant,
}

/// The pseudonym that orders a packet against the others of its session, if it has one.
///
/// Return-routed packets always have one. A forward-routed packet has one only where the caller set
/// it, and one without a pseudonym cannot starve for SURBs, so it needs no ordering key.
fn parking_key(routing: &DestinationRouting) -> Option<HoprPseudonym> {
    match routing {
        DestinationRouting::Return(matcher) => Some(matcher.pseudonym()),
        DestinationRouting::Forward { pseudonym, .. } => *pseudonym,
    }
}

/// Packets that wait for SURBs, one FIFO per pseudonym, kept outside the ordered stage.
///
/// Parking ends a packet's future at once, which frees its [`buffered`] slot and with it the
/// emission of every packet submitted behind it.
///
/// Only the head of a queue is retried, and a packet for a pseudonym that has one parked cannot
/// resolve ahead of it: that is the ordering guarantee the reassembler needs, kept per session
/// rather than node-wide.
#[derive(Default)]
struct HoldLane {
    inner: parking_lot::Mutex<HoldState>,
    /// Wakes the waiter in [`HoldLane::wait_for_work`] on a park or a close.
    waker: futures::task::AtomicWaker,
}

#[derive(Default)]
struct HoldState {
    lanes: HashMap<HoprPseudonym, PseudonymLane>,
    /// Pseudonym whose resolved packet was handed on last, until the next step clears it.
    handed: Option<HoprPseudonym>,
    /// Counts parks and closes, so a waiter that has not taken the lock cannot miss either.
    revisions: u64,
    /// Set when the ordered stream has ended and the lane is expected to drain.
    closed: bool,
}

impl HoldState {
    /// Marks the state changed, so a waiter cannot sleep through it.
    ///
    /// Every path that could make a parked packet due must bump this: a wake that lands before the
    /// waiter registered its waker would otherwise be lost, and the packet would sit until some
    /// unrelated event woke the lane.
    fn bump(&mut self) {
        self.revisions = self.revisions.wrapping_add(1);
    }

    /// Whether a packet of `pseudonym` is parked, being resolved, or on its way out.
    fn is_busy(&self, pseudonym: HoprPseudonym) -> bool {
        self.handed == Some(pseudonym)
            || self
                .lanes
                .get(&pseudonym)
                .is_some_and(|lane| lane.busy || !lane.parked.is_empty())
    }

    /// Drops the marker that keeps a pseudonym busy until its packet reached the output.
    fn release_handed(&mut self) {
        self.handed = None;
    }

    /// What the lane should do next.
    fn next_due(&mut self) -> NextDue {
        self.lanes.retain(|_, lane| lane.busy || !lane.parked.is_empty());

        let now = Instant::now();
        let mut earliest: Option<(Instant, HoprPseudonym)> = None;
        for (pseudonym, lane) in self.lanes.iter() {
            if lane.busy {
                continue;
            }
            let Some(head) = lane.parked.front() else {
                continue;
            };
            if earliest.is_none_or(|(at, _)| head.next_attempt < at) {
                earliest = Some((head.next_attempt, *pseudonym));
            }
        }

        match earliest {
            None if self.closed => NextDue::Finished,
            None => NextDue::Idle,
            Some((at, pseudonym)) if at <= now => {
                let lane = self
                    .lanes
                    .get_mut(&pseudonym)
                    .expect("the lane was just found to be non-empty");
                lane.busy = true;
                NextDue::Ready {
                    pseudonym,
                    packet: lane.parked.pop_front().expect("the head was just found present"),
                }
            }
            Some((at, _)) => NextDue::Later(at.saturating_duration_since(now)),
        }
    }
}

/// One FIFO of parked packets, plus whether its packet is between resolve attempts.
#[derive(Default)]
struct PseudonymLane {
    parked: VecDeque<Box<ParkedPacket>>,
    /// Set while a packet of this pseudonym is claimed by the stage or the hold lane, so that a
    /// later packet cannot resolve ahead of it.
    busy: bool,
}

/// What [`HoldLane::next_step`] resolved to.
enum HoldStep {
    /// The input ended and nothing is left to retry.
    Finished,
    /// The head of `pseudonym` is due for an attempt.
    Resolve {
        pseudonym: HoprPseudonym,
        /// Boxed so the two variants do not differ by the size of a packet.
        packet: Box<ParkedPacket>,
    },
}

/// What [`HoldState::next_due`] resolved to.
enum NextDue {
    Finished,
    Idle,
    Later(Duration),
    Ready {
        pseudonym: HoprPseudonym,
        packet: Box<ParkedPacket>,
    },
}

impl HoldLane {
    /// Claims the right to resolve a packet of `pseudonym`.
    ///
    /// Returns `None` when a packet of the same pseudonym is already parked, being resolved, or on
    /// its way out: resolving would let the caller overtake it, so it parks instead. The claim is
    /// released when it is dropped.
    fn claim(self: &Arc<Self>, pseudonym: HoprPseudonym) -> Option<HoldClaim> {
        let mut state = self.inner.lock();
        if state.is_busy(pseudonym) {
            return None;
        }
        state.lanes.entry(pseudonym).or_default().busy = true;
        Some(HoldClaim {
            hold: self.clone(),
            pseudonym,
        })
    }

    /// Queues a packet until its SURBs arrive, returning it back when its queue is full.
    fn park(&self, pseudonym: HoprPseudonym, packet: Box<ParkedPacket>) -> Option<Box<ParkedPacket>> {
        let rejected = {
            let mut state = self.inner.lock();
            state.bump();
            let lane = state.lanes.entry(pseudonym).or_default();
            if lane.parked.len() >= MAX_PARKED_PER_PSEUDONYM {
                Some(packet)
            } else {
                lane.parked.push_back(packet);
                None
            }
        };
        self.waker.wake();
        rejected
    }

    /// Records that the packet just resolved was handed to the output.
    ///
    /// It stays busy until the next step, so that a packet of the same pseudonym cannot resolve
    /// ahead of one the output has not taken yet.
    fn emitted(&self, pseudonym: HoprPseudonym) {
        let mut state = self.inner.lock();
        if let Some(lane) = state.lanes.get_mut(&pseudonym) {
            lane.busy = false;
        }
        state.handed = Some(pseudonym);
        state.bump();
    }

    /// Frees the pseudonym after a failed attempt, re-queueing the packet when it may be retried.
    fn release(&self, pseudonym: HoprPseudonym, retry: Option<Box<ParkedPacket>>) {
        let queued = {
            let mut state = self.inner.lock();
            let Some(lane) = state.lanes.get_mut(&pseudonym) else {
                return;
            };
            lane.busy = false;
            if let Some(packet) = retry {
                lane.parked.push_front(packet);
            }
            let queued = !lane.parked.is_empty();
            state.bump();
            queued
        };
        if queued {
            self.waker.wake();
        }
    }

    /// Tells the lane that the ordered stream ended, so it can finish once it drained.
    fn close(&self) {
        let mut state = self.inner.lock();
        state.closed = true;
        state.bump();
        drop(state);
        self.waker.wake();
    }

    /// Waits until the next parked packet is due, and takes it out of its queue.
    ///
    /// Returns [`HoldStep::Finished`] only once the input ended and nothing is left to retry, which
    /// ends the stream this backs.
    async fn next_step(&self) -> HoldStep {
        loop {
            // The revision is read under the same lock as the decision, so a park that lands right
            // after it is not mistaken for the state this waiter already saw.
            let (wait, revision) = {
                let mut state = self.inner.lock();
                state.release_handed();
                match state.next_due() {
                    NextDue::Finished => return HoldStep::Finished,
                    NextDue::Ready { pseudonym, packet } => return HoldStep::Resolve { pseudonym, packet },
                    NextDue::Later(wait) => (Some(wait), state.revisions),
                    NextDue::Idle => (None, state.revisions),
                }
            };

            match wait {
                Some(wait) => futures_timer::Delay::new(wait).await,
                None => self.wait_for_work(revision).await,
            }
        }
    }

    /// Waits for a park or a close that happened after `revision`.
    ///
    /// The waker is registered under the same lock a park takes, so a park that lands between the
    /// caller's check and this registration cannot be missed.
    async fn wait_for_work(&self, revision: u64) {
        std::future::poll_fn(|cx| {
            let state = self.inner.lock();
            if state.revisions != revision || state.closed {
                return std::task::Poll::Ready(());
            }
            self.waker.register(cx.waker());
            if state.revisions != revision {
                return std::task::Poll::Ready(());
            }
            std::task::Poll::Pending
        })
        .await;
    }
}

/// The right to resolve packets of one pseudonym, held across the resolve await.
///
/// Released on drop, so a resolve that fails, times out or panics cannot leave the pseudonym
/// claimed.
struct HoldClaim {
    hold: Arc<HoldLane>,
    pseudonym: HoprPseudonym,
}

impl Drop for HoldClaim {
    fn drop(&mut self) {
        let queued = {
            let mut state = self.hold.inner.lock();
            let Some(lane) = state.lanes.get_mut(&self.pseudonym) else {
                return;
            };
            lane.busy = false;
            let queued = !lane.parked.is_empty();
            if !queued {
                // Nothing waits behind this packet, so the lane goes with it rather than lingering
                // as an empty entry for a session that has moved on.
                state.lanes.remove(&self.pseudonym);
            }
            state.bump();
            queued
        };
        // A packet that parked behind this one is due now, not after a retry interval.
        if queued {
            self.hold.waker.wake();
        }
    }
}

/// Applies the return-path verdict to the packet's signals and stamps the generation its plan was
/// captured with.
fn with_signals(
    resolved: ResolvedTransportRouting<HoprSurb>,
    rem_surbs: Option<usize>,
    generation: Option<u8>,
    distress_threshold: usize,
    mut data: ApplicationDataOut,
) -> (ResolvedTransportRouting<HoprSurb>, ApplicationDataOut) {
    // These flags are translated into HOPR protocol packet signals and are applicable only on the
    // return path.
    let mut signals_to_dst = data
        .packet_info
        .as_ref()
        .map(|info| info.signals_to_destination)
        .unwrap_or_default();

    if resolved.is_return() {
        signals_to_dst = match rem_surbs {
            Some(rem) if (1..distress_threshold.max(2)).contains(&rem) => signals_to_dst | PacketSignal::SurbDistress,
            Some(0) => signals_to_dst | PacketSignal::OutOfSurbs,
            _ => signals_to_dst - (PacketSignal::OutOfSurbs | PacketSignal::SurbDistress),
        };
    } else {
        // Unset these flags as they make no sense on the forward path.
        signals_to_dst -= PacketSignal::SurbDistress | PacketSignal::OutOfSurbs;
    }

    let info = data.packet_info.get_or_insert_default();
    info.signals_to_destination = signals_to_dst;
    // Carry the generation captured with the return-path plan alongside the packet, so the encode
    // stage stamps the plan's generation rather than re-reading a possibly-newer one after a
    // concurrent re-plan.
    info.surb_generation = generation;

    (resolved, data)
}

/// Drops a parked packet whose SURBs did not arrive before its deadline.
///
/// The warn is the only externally visible sign that a counterparty stopped replenishing, and the
/// refund returns the SURB its sender already counted for a packet that leaves without one.
fn drop_at_deadline(
    routing: &DestinationRouting,
    error: Option<&PathPlannerError>,
    surb_wait: Duration,
    data: &ApplicationDataOut,
) {
    hopr_transport_session::counters::ROUTING_RESOLUTION_SURB_TIMEOUTS
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    match error {
        Some(error) => warn!(
            ?routing,
            ?surb_wait,
            %error,
            "dropping an outgoing packet: no SURB for its return path within the wait"
        ),
        None => warn!(
            ?routing,
            ?surb_wait,
            "dropping an outgoing packet: no SURB for its return path within the wait"
        ),
    }
    data.surb_refund.give_back();
}

/// Parks a packet, or drops it when its pseudonym already holds [`MAX_PARKED_PER_PSEUDONYM`].
fn park_or_drop(hold: &Arc<HoldLane>, pseudonym: HoprPseudonym, packet: ParkedPacket, surb_wait: Duration) {
    if let Some(rejected) = hold.park(pseudonym, Box::new(packet)) {
        hopr_transport_session::counters::ROUTING_RESOLUTION_SURB_TIMEOUTS
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        warn!(
            routing = ?rejected.routing,
            ?surb_wait,
            max_parked = MAX_PARKED_PER_PSEUDONYM,
            "dropping an outgoing packet: too many packets of its session already wait for SURBs"
        );
        rejected.data.surb_refund.give_back();
    }
}

/// Resolves the routing of every outgoing packet, emitting the resolved packets as they resolve.
///
/// `resolve` is the resolution step itself — in production
/// [`PathPlanner::resolve_routing`](super::PathPlanner::resolve_routing), taking the payload size
/// hint, the maximum number of SURBs the packet can carry, and the unresolved routing.
///
/// A packet whose routing cannot be resolved is dropped and counted. A packet whose *return* routing
/// finds no SURB is parked for up to `surb_wait` and dropped the same way when that passes; see
/// [`DEFAULT_SURB_RESOLUTION_WAIT`] for why that bound has to exist.
///
/// The packets that resolve at once keep their submission order. A parked packet re-enters the
/// output where its SURBs arrive, ordered only against the other packets of its own pseudonym —
/// the constraint the entry's reassembler imposes — so a session out of SURBs delays nothing but
/// itself.
pub(crate) fn resolve_routing_stage_with_context<St, F, Fut>(
    input: St,
    resolve: F,
    distress_threshold: usize,
    concurrency: usize,
    surb_wait: Duration,
) -> impl Stream<Item = Originated>
where
    St: Stream<Item = crate::trace_context::WithContext<(DestinationRouting, ApplicationDataOut)>>,
    F: Fn(usize, usize, DestinationRouting) -> Fut + Clone,
    Fut: Future<Output = PathResult<Resolved>>,
{
    let hold = Arc::new(HoldLane::default());

    let resolved_on_the_spot = {
        let hold = hold.clone();
        let resolve = resolve.clone();
        input
            .map(move |message| {
                let hold = hold.clone();
                let resolve = resolve.clone();
                let ((routing, data), parent) = message.into_parts();
                let request_span = tracing::debug_span!(
                    parent: &parent,
                    "transport.send",
                    packet_id = crate::trace_context::next_packet_id()
                );
                let span = if request_span.is_disabled() {
                    parent
                } else {
                    request_span
                };
                let output_span = span.clone();
                let future_span = span.clone();
                async move {
                    let deadline = Instant::now() + surb_wait;

                    // Order against the rest of the session first: a packet whose pseudonym is
                    // already waiting for SURBs must not resolve ahead of it.
                    let (pseudonym, claim) = match parking_key(&routing) {
                        Some(pseudonym) => match hold.claim(pseudonym) {
                            Some(claim) => (Some(pseudonym), Some(claim)),
                            None => {
                                park_or_drop(
                                    &hold,
                                    pseudonym,
                                    ParkedPacket {
                                        routing,
                                        data,
                                        span,
                                        output_span,
                                        deadline,
                                        next_attempt: Instant::now(),
                                    },
                                    surb_wait,
                                );
                                return None;
                            }
                        },
                        None => (None, None),
                    };

                    hopr_transport_session::counters::ROUTING_RESOLUTION_ATTEMPTS
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    trace!(?routing, "resolving routing for packet");
                    match resolve(data.data.total_len(), data.estimate_surbs_with_msg(), routing.clone()).await {
                        Ok((resolved, rem_surbs, generation)) => {
                            let (resolved, data) =
                                with_signals(resolved, rem_surbs, generation, distress_threshold, data);
                            trace!(?resolved, "resolved routing for packet");
                            drop(claim);
                            Some(crate::trace_context::WithContext::new((resolved, data), output_span))
                        }
                        Err(error) if error.is_surb() && Instant::now() >= deadline => {
                            drop_at_deadline(&routing, Some(&error), surb_wait, &data);
                            drop(claim);
                            None
                        }
                        Err(error) if error.is_surb() => {
                            // No SURB available yet (possibly cache-wrapped): the hold lane retries
                            // this packet outside the ordered stream, so the packets behind it are
                            // not withheld.
                            match pseudonym {
                                Some(pseudonym) => park_or_drop(
                                    &hold,
                                    pseudonym,
                                    ParkedPacket {
                                        routing,
                                        data,
                                        span,
                                        output_span,
                                        deadline,
                                        next_attempt: Instant::now() + SURB_RETRY_INTERVAL,
                                    },
                                    surb_wait,
                                ),
                                None => drop_at_deadline(&routing, Some(&error), surb_wait, &data),
                            }
                            drop(claim);
                            None
                        }
                        Err(error) => {
                            hopr_transport_session::counters::ROUTING_RESOLUTION_FAILURES
                                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            error!(%error, "failed to resolve routing");
                            drop(claim);
                            None
                        }
                    }
                }
                .instrument(tracing::debug_span!(parent: &future_span, "routing.resolve"))
                .instrument(future_span)
            })
            .buffered(concurrency)
            .filter_map(futures::future::ready)
    };

    // The ordered stream ends when its input does; tell the hold lane, so the driver below can
    // finish once it drained. Without this the merged stream would never end.
    let input_ended = {
        let hold = hold.clone();
        futures::stream::poll_fn(move |_| {
            hold.close();
            std::task::Poll::Ready(None)
        })
    };

    let parked = hold_lane(hold, resolve, distress_threshold, surb_wait);

    futures::stream::select(Box::pin(resolved_on_the_spot.chain(input_ended)), Box::pin(parked))
}

/// Emits the parked packets as their SURBs arrive, in per-pseudonym order.
fn hold_lane<F, Fut>(
    hold: Arc<HoldLane>,
    resolve: F,
    distress_threshold: usize,
    surb_wait: Duration,
) -> impl Stream<Item = Originated>
where
    F: Fn(usize, usize, DestinationRouting) -> Fut + Clone,
    Fut: Future<Output = PathResult<Resolved>>,
{
    futures::stream::unfold(hold, move |hold| {
        let resolve = resolve.clone();
        async move {
            loop {
                match hold.next_step().await {
                    HoldStep::Finished => return None,
                    HoldStep::Resolve { pseudonym, packet } => {
                        let attempt =
                            retry_parked(&hold, &resolve, distress_threshold, surb_wait, pseudonym, packet).await;
                        if let Some(resolved) = attempt {
                            return Some((resolved, hold));
                        }
                    }
                }
            }
        }
    })
}

/// Attempts one parked packet, emitting it, re-queueing it, or dropping it.
async fn retry_parked<F, Fut>(
    hold: &Arc<HoldLane>,
    resolve: &F,
    distress_threshold: usize,
    surb_wait: Duration,
    pseudonym: HoprPseudonym,
    packet: Box<ParkedPacket>,
) -> Option<Originated>
where
    F: Fn(usize, usize, DestinationRouting) -> Fut,
    Fut: Future<Output = PathResult<Resolved>>,
{
    let ParkedPacket {
        routing,
        data,
        span,
        output_span,
        deadline,
        next_attempt: _,
    } = *packet;

    hopr_transport_session::counters::ROUTING_RESOLUTION_ATTEMPTS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    trace!(?routing, "retrying the routing of a parked packet");
    let resolved = resolve(data.data.total_len(), data.estimate_surbs_with_msg(), routing.clone())
        .instrument(tracing::debug_span!(parent: &span, "routing.resolve"))
        .await;

    match resolved {
        Ok((resolved, rem_surbs, generation)) => {
            let (resolved, data) = with_signals(resolved, rem_surbs, generation, distress_threshold, data);
            trace!(?resolved, "resolved routing of a parked packet");
            hold.emitted(pseudonym);
            Some(crate::trace_context::WithContext::new((resolved, data), output_span))
        }
        Err(error) if error.is_surb() && Instant::now() >= deadline => {
            drop_at_deadline(&routing, Some(&error), surb_wait, &data);
            hold.release(pseudonym, None);
            None
        }
        Err(error) if error.is_surb() => {
            hold.release(
                pseudonym,
                Some(Box::new(ParkedPacket {
                    routing,
                    data,
                    span,
                    output_span,
                    deadline,
                    next_attempt: Instant::now() + SURB_RETRY_INTERVAL,
                })),
            );
            None
        }
        Err(error) => {
            hopr_transport_session::counters::ROUTING_RESOLUTION_FAILURES
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            error!(%error, "failed to resolve routing");
            hold.release(pseudonym, None);
            None
        }
    }
}

#[cfg(test)]
pub(crate) fn resolve_routing_stage<St, F, Fut>(
    input: St,
    resolve: F,
    distress_threshold: usize,
    concurrency: usize,
    surb_wait: Duration,
) -> impl Stream<Item = (ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)>
where
    St: Stream<Item = (DestinationRouting, ApplicationDataOut)>,
    F: Fn(usize, usize, DestinationRouting) -> Fut + Clone,
    Fut: Future<Output = PathResult<Resolved>>,
{
    resolve_routing_stage_with_context(
        input.map(Into::into),
        resolve,
        distress_threshold,
        concurrency,
        surb_wait,
    )
    .map(|message| message.value)
}
#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    };

    use futures_time::future::FutureExt as _;
    use hopr_api::types::{
        crypto::{crypto_traits::Randomizable, prelude::*},
        internal::{
            path::ValidatedPath,
            prelude::HoprPseudonym,
            routing::{RoutingOptions, SurbMatcher},
        },
    };
    use hopr_protocol_app::prelude::{ApplicationData, SurbRefund, Tag};

    use super::*;
    use crate::path::errors::PathPlannerError;

    #[rstest::rstest]
    #[case(hopr_transport_session::Capabilities::default())]
    #[case(hopr_transport_session::Capability::Segmentation.into())]
    #[case(hopr_transport_session::Capability::Segmentation | hopr_transport_session::Capability::RetransmissionAck)]
    #[cfg(feature = "tracing-context")]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[serial_test::serial(routing_logs)]
    async fn queued_sessions_should_keep_their_context_through_routing_and_encoding(
        #[case] capabilities: hopr_transport_session::Capabilities,
    ) -> anyhow::Result<()> {
        use anyhow::Context as _;
        use futures::AsyncWriteExt;

        use crate::{tests::support::Logs, trace_context::ContextSender};

        let logs = Logs::default();
        let subscriber = logs.subscriber(tracing::Level::DEBUG);
        logs.with_cpu_subscriber(subscriber, async {
            let (tx, rx) = hopr_utils::network_types::crossfire_sink::bounded_sink_channel(4);
            let tx = ContextSender::new(tx);
            let ids = [HoprPseudonym::random(), HoprPseudonym::random()];
            for (index, id) in ids.iter().enumerate() {
                // Construct under one context, then poll outside it, as the session manager does.
                let mut session = hopr_transport_session::HoprSession::new(
                    *id,
                    forward_routing(),
                    hopr_transport_session::HoprSessionConfig {
                        capabilities,
                        ..Default::default()
                    },
                    (tx.clone(), futures::stream::pending()),
                    None,
                )?;
                session.write_all(&[index as u8]).await?;
                session.flush().await?;
            }
            drop(tx);
            let resolved_messages = resolve_routing_stage_with_context(
                rx.take(2),
                |_, _, _| async {
                    // Yield so buffered futures for different sessions overlap.
                    tokio::task::yield_now().await;
                    tracing::debug!("session route resolved");
                    Ok((resolved(), None, None))
                },
                TEST_DISTRESS_THRESHOLD,
                2,
                TEST_SURB_WAIT,
            )
            .collect::<Vec<_>>()
            .await;
            assert_eq!(resolved_messages.len(), 2);
            // The encoder receives these envelopes after routing's instrumented future exits.
            // Exercise the same CPU boundary used by the packet encoder.
            for message in resolved_messages {
                let marker = *message
                    .value
                    .1
                    .data
                    .plain_text
                    .last()
                    .context("missing session payload")?;
                let dispatch = tracing::dispatcher::get_default(Clone::clone);
                async move {
                    hopr_utils::parallelize::cpu::spawn_encode_blocking(
                        move || {
                            // Scoped test subscribers must also be installed on the Rayon thread.
                            tracing::dispatcher::with_default(&dispatch, || {
                                tracing::debug!(marker, "session CPU encoding");
                            });
                        },
                        "session_context_test",
                    )
                    .await?;
                    anyhow::Ok(())
                }
                .instrument(message.into_parts().1)
                .await?;
            }
            for event in logs
                .records("session route resolved")
                .into_iter()
                .chain(logs.records("session CPU encoding"))
            {
                assert!(event.ancestry.contains(&"session"), "missing session ancestor");
                assert!(event.ancestry.contains(&"transport.send"), "missing packet ancestor");
                assert!(!event.fields.contains_key("session_id"));
                assert!(!event.fields.contains_key("packet_id"));
            }
            assert_eq!(logs.records("session route resolved").len(), 2);
            assert_eq!(logs.records("session CPU encoding").len(), 2);
            let output = logs.output();
            for (marker, id) in ids.iter().enumerate() {
                let route = output
                    .lines()
                    .find(|line| line.contains("session route resolved") && line.contains(&id.to_string()))
                    .context("missing routed session")?;
                let packet_id = route
                    .split("packet_id=")
                    .nth(1)
                    .context("routing must expose its packet identifier")?
                    .chars()
                    .take_while(char::is_ascii_digit)
                    .collect::<String>();
                let line = output
                    .lines()
                    .find(|line| line.contains("session CPU encoding") && line.contains(&format!("marker={marker}")))
                    .context("missing encoded session")?;
                assert!(line.contains(&id.to_string()), "encoder lost its session: {line}");
                assert!(
                    line.contains(&format!("packet_id={packet_id}}}")),
                    "encoder changed its packet: {line}"
                );
                assert!(
                    !line.contains(&ids[1 - marker].to_string()),
                    "session context leaked: {line}"
                );
            }
            anyhow::Ok(())
        })?
    }

    /// Bound on every test in this module. The stall under test is unbounded, so a test that hits
    /// this limit has reproduced it rather than merely run slowly.
    const TEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

    /// The stage's SURB wait, shortened for the tests.
    ///
    /// Well clear of [`SURB_RETRY_INTERVAL`] so a retry-then-succeed case has room, and far enough
    /// below [`TEST_TIMEOUT`] that "the bound fired" and "the test timed out" cannot be confused.
    const TEST_SURB_WAIT: std::time::Duration = std::time::Duration::from_millis(200);

    const TEST_TAG: u64 = 1234;

    /// Concurrency of the stage under test. Any value ≥ 2 exhibits the ordering behaviour; a small
    /// one keeps the failure legible.
    const TEST_CONCURRENCY: usize = 8;

    const TEST_DISTRESS_THRESHOLD: usize = 2;

    /// Bound for a packet that must be served while another session waits for SURBs.
    ///
    /// A quarter of [`TEST_SURB_WAIT`], so a stage that withholds it behind the starved packet
    /// cannot pass, and wide enough that a loaded CI machine does not fail it by being slow.
    const TEST_UNBLOCKED_BOUND: std::time::Duration = std::time::Duration::from_millis(50);

    /// The stage under test with the test constants applied, so each test differs only in the two
    /// things that matter to it: what goes in, and what resolution does.
    fn stage<St, F, Fut>(
        input: St,
        resolve: F,
    ) -> impl Stream<Item = (ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)>
    where
        St: Stream<Item = (DestinationRouting, ApplicationDataOut)>,
        F: Fn(usize, usize, DestinationRouting) -> Fut + Clone,
        Fut: Future<Output = PathResult<(ResolvedTransportRouting<HoprSurb>, Option<usize>, Option<u8>)>>,
    {
        resolve_routing_stage(
            input,
            resolve,
            TEST_DISTRESS_THRESHOLD,
            TEST_CONCURRENCY,
            TEST_SURB_WAIT,
        )
    }

    /// A resolution result the stage accepts. The variant is a forward one because a
    /// [`ResolvedTransportRouting::Return`] needs a real `HoprSurb`, and no test here depends on
    /// which variant came back — only on *whether* the packet was emitted.
    fn resolved() -> ResolvedTransportRouting<HoprSurb> {
        ResolvedTransportRouting::Forward {
            pseudonym: HoprPseudonym::random(),
            forward_path: ValidatedPath::direct(
                *OffchainKeypair::random().public(),
                ChainKeypair::random().public().to_address(),
            ),
            return_paths: vec![],
        }
    }

    fn forward_routing() -> DestinationRouting {
        DestinationRouting::forward_only(
            *OffchainKeypair::random().public(),
            RoutingOptions::Hops(1.try_into().expect("1 is a valid hop count")),
        )
    }

    fn return_routing(pseudonym: HoprPseudonym) -> DestinationRouting {
        DestinationRouting::Return(SurbMatcher::Pseudonym(pseudonym))
    }

    /// A packet carrying `marker` as its only payload byte, so emitted packets can be identified.
    fn packet(marker: u8) -> ApplicationDataOut {
        ApplicationDataOut::with_no_packet_info(
            ApplicationData::new(Tag::from(TEST_TAG), &[marker]).expect("a one-byte payload is valid"),
        )
    }

    fn marker_of(data: &ApplicationDataOut) -> u8 {
        data.data.plain_text[0]
    }

    /// The error the planner raises when the SURB store holds nothing for a pseudonym.
    ///
    /// This is a *permanent* condition once the counterparty is gone — it is the same error whether
    /// the pool is momentarily empty or will never be refilled again, which is precisely what makes
    /// retrying on it indefinitely unsafe.
    fn no_surb(routing: &DestinationRouting) -> PathPlannerError {
        let pseudonym = match routing {
            DestinationRouting::Return(matcher) => matcher.pseudonym().to_string(),
            DestinationRouting::Forward { .. } => unreachable!("only return routing can starve for SURBs"),
        };
        PathPlannerError::Surb(format!("no surb for pseudonym {pseudonym}"))
    }

    /// A return packet for a pseudonym whose SURBs will never arrive must not withhold the packets
    /// queued behind it.
    ///
    /// This is the `london-01` outage in miniature. The exit kept a session slot alive after its
    /// initiator had gone, so its keep-alive stream went on emitting return-routed packets for a
    /// pseudonym with no SURBs. Resolution for that packet can never succeed, and because the stage
    /// preserves submission order, every subsequent packet was withheld behind it: the node
    /// originated nothing for 1h44m while forwarding, acking and receiving carried on normally.
    ///
    /// The assertion is that the packets behind the starved one are **emitted** — not merely that
    /// nothing errored. A stalled origination stage is externally indistinguishable from an idle
    /// node, which is what made the live outage cost a day to find.
    #[test_log::test(tokio::test)]
    async fn a_starved_return_packet_should_not_withhold_the_packets_behind_it() -> anyhow::Result<()> {
        let starved = HoprPseudonym::random();
        let dropped_before = hopr_transport_session::counters::routing_resolution_surb_timeout_count();

        let input = futures::stream::iter(vec![
            (return_routing(starved), packet(0)),
            (forward_routing(), packet(1)),
            (forward_routing(), packet(2)),
        ]);

        // Consumed to the end rather than stopped after the two forward packets: the parked packet's
        // drop is what this test also checks, and it happens at its deadline.
        let emitted = stage(
            input,
            |_size_hint, _max_surbs, routing: DestinationRouting| async move {
                match routing {
                    DestinationRouting::Return(_) => Err(no_surb(&routing)),
                    DestinationRouting::Forward { .. } => Ok((resolved(), None, None)),
                }
            },
        )
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TEST_TIMEOUT))
        .await;

        let emitted = emitted.map_err(|_| {
            anyhow::anyhow!(
                "origination stalled: the two forward packets queued behind a return packet whose pseudonym has no \
                 SURBs were never emitted within {TEST_TIMEOUT:?}. Every packet this node originates passes through \
                 this stage, so this is a node-wide origination outage."
            )
        })?;

        assert_eq!(
            emitted.iter().map(|(_, data)| marker_of(data)).collect::<Vec<_>>(),
            vec![1, 2],
            "the packets behind the starved one must be emitted, in order"
        );

        // The starved packet is dropped, and that has to be *countable*. A silent drop leaves the
        // operator with the same nothing the unbounded wait did: traffic missing and no signal
        // saying why.
        assert!(
            hopr_transport_session::counters::routing_resolution_surb_timeout_count() > dropped_before,
            "the starved packet was dropped without being counted, so the condition stays invisible"
        );

        Ok(())
    }

    /// A session that waits for SURBs must not withhold another session's packet.
    ///
    /// This is the exit-side head-of-line blocking in miniature: the wait of one session used to
    /// happen inside the ordered stream, where it delayed every packet submitted behind it, so a
    /// single client without SURBs slowed the whole node once per packet until the wait expired.
    ///
    /// The bound is what makes the assertion mean something: the packet of the other pseudonym has
    /// to be emitted well inside [`TEST_SURB_WAIT`], not merely before the stream ends.
    ///
    /// This is the acceptance test for the fix: on the code before it, the packet waited the whole
    /// [`TEST_SURB_WAIT`] behind the starved one and this bound fired.
    #[test_log::test(tokio::test)]
    async fn a_starved_return_packet_should_not_delay_another_pseudonym() -> anyhow::Result<()> {
        let starved = HoprPseudonym::random();
        let input = futures::stream::iter(vec![
            (return_routing(starved), packet(0)),
            (forward_routing(), packet(1)),
        ]);

        let emitted = stage(
            input,
            move |_size_hint, _max_surbs, routing: DestinationRouting| async move {
                match &routing {
                    DestinationRouting::Return(matcher) if matcher.pseudonym() == starved => Err(no_surb(&routing)),
                    _ => Ok((resolved(), None, None)),
                }
            },
        )
        .take(1)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TEST_UNBLOCKED_BOUND))
        .await
        .map_err(|_| {
            anyhow::anyhow!(
                "the packet of the second pseudonym was not emitted within {TEST_UNBLOCKED_BOUND:?} while another \
                 session waited {TEST_SURB_WAIT:?} for its SURBs: one starved session withholds the whole node's \
                 origination"
            )
        })?;

        assert_eq!(
            vec![1],
            emitted.iter().map(|(_, data)| marker_of(data)).collect::<Vec<_>>(),
            "the other session's packet is emitted while the starved session waits"
        );

        Ok(())
    }

    /// A packet of a session must not overtake an earlier one of the same session that waits.
    ///
    /// The wait now happens outside the ordered stream, so the order of a session rests on the hold
    /// lane alone: a later packet has to queue behind the parked one instead of resolving ahead.
    #[test_log::test(tokio::test)]
    async fn a_parked_packet_should_not_be_overtaken_by_a_later_one_of_its_pseudonym() -> anyhow::Result<()> {
        const FAILURES_BEFORE_SUCCESS: usize = 2;

        let attempts = Arc::new(AtomicUsize::new(0));
        let pseudonym = HoprPseudonym::random();
        let input = futures::stream::iter(vec![
            (return_routing(pseudonym), packet(0)),
            (return_routing(pseudonym), packet(1)),
        ]);

        let emitted = {
            let attempts = attempts.clone();
            stage(input, move |_size_hint, _max_surbs, routing: DestinationRouting| {
                let attempts = attempts.clone();
                async move {
                    if attempts.fetch_add(1, Ordering::Relaxed) < FAILURES_BEFORE_SUCCESS {
                        Err(no_surb(&routing))
                    } else {
                        Ok((resolved(), Some(1), None))
                    }
                }
            })
            .take(2)
            .collect::<Vec<_>>()
            .timeout(futures_time::time::Duration::from(TEST_TIMEOUT))
            .await
            .map_err(|_| anyhow::anyhow!("a packet of the session never resolved after its predecessor did"))?
        };

        assert_eq!(
            vec![0, 1],
            emitted.iter().map(|(_, data)| marker_of(data)).collect::<Vec<_>>(),
            "the later packet of the session must wait for the parked one, not overtake it"
        );

        Ok(())
    }

    /// A parked packet that never gets its SURBs is dropped at its deadline, counted, and refunds
    /// the SURB its sender counted — the same accounting the in-stage wait used to apply.
    #[test_log::test(tokio::test)]
    async fn a_parked_packet_should_give_back_the_surb_its_sender_counted() -> anyhow::Result<()> {
        let refunds = Arc::new(AtomicU64::new(0));
        let dropped_before = hopr_transport_session::counters::routing_resolution_surb_timeout_count();
        let mut counted = packet(0);
        counted.surb_refund = SurbRefund::to(refunds.clone());
        let input = futures::stream::iter(vec![(return_routing(HoprPseudonym::random()), counted)]);

        let emitted = stage(
            input,
            |_size_hint, _max_surbs, routing: DestinationRouting| async move { Err(no_surb(&routing)) },
        )
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TEST_TIMEOUT))
        .await
        .map_err(|_| anyhow::anyhow!("the parked packet never reached its deadline"))?;

        assert!(emitted.is_empty(), "a starved packet must not be emitted");
        assert_eq!(
            1,
            refunds.load(Ordering::Relaxed),
            "the SURB its sender counted must return when the parked packet is dropped"
        );
        assert!(
            hopr_transport_session::counters::routing_resolution_surb_timeout_count() > dropped_before,
            "the deadline drop of a parked packet must be counted"
        );

        Ok(())
    }

    #[test]
    fn an_unset_wait_should_fall_back_to_the_default() {
        assert_eq!(DEFAULT_SURB_RESOLUTION_WAIT, surb_resolution_wait(None));
    }

    #[test]
    fn a_configured_wait_should_be_honoured() {
        let configured = std::time::Duration::from_millis(2_500);
        assert_eq!(configured, surb_resolution_wait(Some(configured)));
    }

    /// Zero is honoured rather than treated as "unset".
    ///
    /// The sibling concurrency knobs in the same config read `Some(0)` as "use the default", so the
    /// difference is worth pinning: for a wait, zero has a meaning of its own — do not wait at all.
    #[test]
    fn a_zero_wait_should_mean_no_wait_rather_than_the_default() {
        assert_eq!(
            std::time::Duration::ZERO,
            surb_resolution_wait(Some(std::time::Duration::ZERO))
        );
    }

    /// With the wait disabled, a starved return packet is dropped on its first miss.
    ///
    /// This is the behaviour a zero wait buys, and the reason it is a footgun rather than a
    /// tuning: any momentary SURB gap becomes loss. Asserting the attempt count is what separates
    /// "did not wait" from "waited briefly".
    #[test_log::test(tokio::test)]
    async fn a_zero_wait_should_drop_a_starved_return_packet_without_retrying() -> anyhow::Result<()> {
        let attempts = Arc::new(AtomicUsize::new(0));
        let input = futures::stream::iter(vec![(return_routing(HoprPseudonym::random()), packet(0))]);

        let emitted = {
            let attempts = attempts.clone();
            resolve_routing_stage(
                input,
                move |_size_hint, _max_surbs, routing: DestinationRouting| {
                    let attempts = attempts.clone();
                    async move {
                        attempts.fetch_add(1, Ordering::Relaxed);
                        Err(no_surb(&routing))
                    }
                },
                TEST_DISTRESS_THRESHOLD,
                TEST_CONCURRENCY,
                surb_resolution_wait(Some(std::time::Duration::ZERO)),
            )
            .collect::<Vec<_>>()
            .timeout(futures_time::time::Duration::from(TEST_TIMEOUT))
            .await
            .map_err(|_| anyhow::anyhow!("a zero wait still stalled instead of dropping immediately"))?
        };

        assert!(emitted.is_empty(), "the starved packet must not be emitted");
        assert_eq!(
            1,
            attempts.load(Ordering::Relaxed),
            "a zero wait must resolve once and give up, not retry"
        );

        Ok(())
    }

    /// A starved return packet gives back the SURB its sender counted for it. One whose sender counted
    /// nothing gives nothing back, and a packet that finds its SURB keeps the count.
    #[test_log::test(tokio::test)]
    async fn a_starved_return_packet_should_give_back_the_surb_its_sender_counted() -> anyhow::Result<()> {
        let refunds = Arc::new(AtomicU64::new(0));
        let counted = |marker| {
            let mut data = packet(marker);
            data.surb_refund = SurbRefund::to(refunds.clone());
            data
        };
        let (starved, served) = (HoprPseudonym::random(), HoprPseudonym::random());
        let input = futures::stream::iter(vec![
            (return_routing(starved), counted(0)),
            (return_routing(starved), packet(1)),
            (return_routing(served), counted(2)),
        ]);

        let emitted = resolve_routing_stage(
            input,
            move |_size_hint, _max_surbs, routing: DestinationRouting| async move {
                match &routing {
                    DestinationRouting::Return(matcher) if matcher.pseudonym() == served => {
                        Ok((resolved(), Some(5), None))
                    }
                    _ => Err(no_surb(&routing)),
                }
            },
            TEST_DISTRESS_THRESHOLD,
            TEST_CONCURRENCY,
            surb_resolution_wait(Some(std::time::Duration::ZERO)),
        )
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TEST_TIMEOUT))
        .await
        .map_err(|_| anyhow::anyhow!("the stage stalled instead of dropping the starved packets"))?;

        assert_eq!(
            vec![2],
            emitted.iter().map(|(_, data)| marker_of(data)).collect::<Vec<_>>(),
            "only the packet that found its SURB goes on"
        );
        assert_eq!(
            1,
            refunds.load(Ordering::Relaxed),
            "only the counted packet that starved gives its SURB back"
        );

        Ok(())
    }

    /// A return packet whose SURBs are only momentarily absent must still be emitted once they
    /// arrive.
    ///
    /// This is the behaviour the retry exists for, and it constrains any bound placed on that
    /// retry: dropping a return packet on the first SURB error would lose data on a session that
    /// was merely waiting for its pool to refill.
    #[test_log::test(tokio::test)]
    async fn a_return_packet_should_be_emitted_once_its_surbs_arrive() -> anyhow::Result<()> {
        const FAILURES_BEFORE_SUCCESS: usize = 3;

        let attempts = Arc::new(AtomicUsize::new(0));
        let input = futures::stream::iter(vec![(return_routing(HoprPseudonym::random()), packet(7))]);

        let emitted = {
            let attempts = attempts.clone();
            stage(input, move |_size_hint, _max_surbs, routing: DestinationRouting| {
                let attempts = attempts.clone();
                async move {
                    if attempts.fetch_add(1, Ordering::Relaxed) < FAILURES_BEFORE_SUCCESS {
                        Err(no_surb(&routing))
                    } else {
                        Ok((resolved(), Some(0), None))
                    }
                }
            })
            .take(1)
            .collect::<Vec<_>>()
            .timeout(futures_time::time::Duration::from(TEST_TIMEOUT))
            .await
            .map_err(|_| {
                anyhow::anyhow!(
                    "a return packet whose SURBs arrived after {FAILURES_BEFORE_SUCCESS} retries was never emitted"
                )
            })?
        };

        assert_eq!(
            emitted.len(),
            1,
            "the return packet must be emitted once its SURBs arrive"
        );
        assert_eq!(marker_of(&emitted[0].1), 7);
        assert!(
            attempts.load(Ordering::Relaxed) > FAILURES_BEFORE_SUCCESS,
            "the stage must retry rather than drop on the first SURB error"
        );

        Ok(())
    }

    /// A hard (non-SURB) resolution failure drops that packet and lets the rest through.
    ///
    /// The contrast with the starvation case is the point: the stage already knows how to drop a
    /// packet it cannot route and carry on. Only the SURB branch retries without a bound.
    #[test_log::test(tokio::test)]
    async fn a_hard_resolution_failure_should_drop_only_its_own_packet() -> anyhow::Result<()> {
        let input = futures::stream::iter(vec![
            (forward_routing(), packet(0)),
            (forward_routing(), packet(1)),
            (forward_routing(), packet(2)),
        ]);

        let emitted = stage(
            input,
            |_size_hint, _max_surbs, _routing: DestinationRouting| async move {
                static SEEN: AtomicUsize = AtomicUsize::new(0);
                if SEEN.fetch_add(1, Ordering::Relaxed) == 0 {
                    Err(PathPlannerError::Api("no path".into()))
                } else {
                    Ok((resolved(), None, None))
                }
            },
        )
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TEST_TIMEOUT))
        .await
        .map_err(|_| anyhow::anyhow!("a hard resolution failure stalled the stage instead of dropping its packet"))?;

        assert_eq!(
            emitted.iter().map(|(_, data)| marker_of(data)).collect::<Vec<_>>(),
            vec![1, 2],
            "only the unroutable packet is dropped; the rest keep their order"
        );

        Ok(())
    }

    /// The SURB-batch generation captured with the return-path plan (the third element of the
    /// resolve result) must ride on the emitted packet's `packet_info`, so the encode stage stamps
    /// that value instead of re-reading a possibly-newer one after a concurrent re-plan.
    #[test_log::test(tokio::test)]
    async fn resolved_packet_should_carry_the_captured_surb_generation() -> anyhow::Result<()> {
        const CAPTURED: u8 = 7;

        let input = futures::stream::iter(vec![(forward_routing(), packet(0))]);

        let emitted = stage(input, |_size_hint, _max_surbs, _routing| async move {
            Ok((resolved(), None, Some(CAPTURED)))
        })
        .take(1)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TEST_TIMEOUT))
        .await
        .map_err(|_| anyhow::anyhow!("the stage did not emit the packet within {TEST_TIMEOUT:?}"))?;

        let (_, data) = emitted
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("no packet emitted"))?;
        assert_eq!(
            data.packet_info.and_then(|info| info.surb_generation),
            Some(CAPTURED),
            "the generation captured at resolution must ride on packet_info to the encode stage"
        );

        Ok(())
    }
}
