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
//! Order is guaranteed within a pseudonym, not across them. A packet whose return routing finds no
//! SURB waits in a lane of its own pseudonym (see [`SurbWait`]), outside the ordered stream, and
//! leaves it when its SURBs arrive. The order the reassembler needs is per Session, and a wait that
//! held this stage would withhold every other Session's packets for as long as it lasted.

use std::{
    collections::{HashMap, VecDeque},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::Poll,
    time::{Duration, Instant},
};

use futures::{Stream, StreamExt};
use hopr_api::types::internal::{
    prelude::HoprPseudonym,
    routing::{DestinationRouting, ResolvedTransportRouting},
};
use hopr_crypto_packet::prelude::{HoprSurb, PacketSignal};
use hopr_protocol_app::prelude::ApplicationDataOut;
use parking_lot::Mutex;
use tracing::{Instrument, error, trace, warn};

use super::errors::{PathPlannerError, Result as PathResult};

/// How often a return route whose SURBs have not arrived is retried.
const SURB_RETRY_INTERVAL: std::time::Duration = std::time::Duration::from_millis(5);

/// Most packets one pseudonym may have waiting for its SURBs.
///
/// A parked packet leaves the bounded channel that feeds this stage, so the hold needs a bound of its
/// own: a counterparty that never sends another SURB must not pin memory for the life of the process.
/// The figure matches [`MAXIMUM_MSG_OUTGOING_BUFFER_SIZE`], the capacity of the channel the packets
/// came from, so a Session can hold no more here than the pipeline held before. A packet that finds
/// the lane full is dropped the same way, and counted the same way, as one that runs out of wait.
///
/// [`MAXIMUM_MSG_OUTGOING_BUFFER_SIZE`]: crate::constants::MAXIMUM_MSG_OUTGOING_BUFFER_SIZE
const MAX_PARKED_PER_PSEUDONYM: usize = 256;

/// Default for [`PacketPipelineConfig::surb_resolution_wait`][cfg], used when it is unset.
///
/// A momentary gap is normal — the SURB pool refills asynchronously, and dropping a return packet
/// on the first miss loses data on a session that was only waiting. That is why the retry exists.
///
/// What it must not do is wait indefinitely. When the counterparty is gone no SURB is ever coming,
/// so a packet that waited for one holds a place in its pseudonym's lane for no purpose. The wait is
/// the whole of the bound: past it the packet is dropped, counted, and its SURB given back.
///
/// Six seconds is deliberately generous, because this is a *library* default and the useful ceiling
/// belongs to the caller. A packet held past the session's frame timeout — 3 s for the sessions
/// hoprd runs — cannot be used by the receiver anyway, so a deployment that knows its own timeout
/// should configure something tighter, as hoprd does. What a library default must not do is drop a
/// packet a slower or reliable session would still have made use of, so it errs long and leaves the
/// tightening to whoever knows the traffic.
///
/// [cfg]: crate::protocol::PacketPipelineConfig::surb_resolution_wait
pub(crate) const DEFAULT_SURB_RESOLUTION_WAIT: std::time::Duration = std::time::Duration::from_secs(6);

/// The wait a node should use, given what its configuration says.
///
/// Unset means the default; zero is honoured as "do not wait", which drops a return packet the
/// first time its SURBs are missing. Kept as one function so both meanings live in one place
/// rather than being re-derived at the call site.
pub(crate) fn surb_resolution_wait(configured: Option<std::time::Duration>) -> std::time::Duration {
    configured.unwrap_or(DEFAULT_SURB_RESOLUTION_WAIT)
}

/// Resolves the routing of every outgoing packet, emitting each resolved packet once it can leave.
///
/// `resolve` is the resolution step itself — in production
/// [`PathPlanner::resolve_routing`](super::PathPlanner::resolve_routing), taking the payload size
/// hint, the maximum number of SURBs the packet can carry, and the unresolved routing.
///
/// A packet whose routing cannot be resolved is dropped and counted. A packet whose *return* routing
/// finds no SURB is parked on its pseudonym's lane in [`SurbWait`] and retried off this stage for up
/// to `surb_wait`; see [`DEFAULT_SURB_RESOLUTION_WAIT`] for why that bound has to exist.
///
/// Order is per pseudonym: [`buffered`](futures::StreamExt::buffered) still withholds a packet
/// behind an earlier one of the *same* submission stream, which serves the entry's reassembler, but a
/// packet parked for its SURBs no longer holds that stream: it leaves from the lane instead, so only
/// its own pseudonym can be waiting behind it.
pub(crate) fn resolve_routing_stage_with_context<St, F, Fut>(
    input: St,
    resolve: F,
    distress_threshold: usize,
    concurrency: usize,
    surb_wait: std::time::Duration,
) -> impl Stream<Item = crate::trace_context::WithContext<(ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)>>
where
    St: Stream<Item = crate::trace_context::WithContext<(DestinationRouting, ApplicationDataOut)>>,
    F: Fn(usize, usize, DestinationRouting) -> Fut + Clone,
    Fut: Future<Output = PathResult<(ResolvedTransportRouting<HoprSurb>, Option<usize>, Option<u8>)>>,
{
    let (wait, wake) = SurbWait::new();

    let resolved = {
        let wait = wait.clone();
        let resolve = resolve.clone();
        input
            .map(move |message| {
                let wait = wait.clone();
                let resolve = resolve.clone();
                let ((unresolved, mut data), span) = message.into_parts();
                let request_span = tracing::debug_span!(
                    parent: &span,
                    "transport.send",
                    packet_id = crate::trace_context::next_packet_id()
                );
                let span = if request_span.is_disabled() { span } else { request_span };
                let output_span = span.clone();
                async move {
                    // Zero wait means "drop a return packet on the first miss", so nothing is ever
                    // parked and no lane is consulted.
                    let parking = !surb_wait.is_zero();
                    let pseudonym = lane_of(&unresolved);

                    // Everything about one pseudonym's lane runs under its gate: the check for a
                    // queue, the park, and the resolve itself. A packet that resolved while an
                    // earlier one of its pseudonym already waited would overtake it.
                    let gate = pseudonym.map(|pseudonym| wait.gate(pseudonym));
                    let _lane = match gate {
                        Some(ref gate) => Some(gate.clone().lock_owned().await),
                        None => None,
                    };

                    if parking
                        && let Some(pseudonym) = pseudonym
                        && wait.queued(pseudonym) > 0
                    {
                        trace!(
                            ?unresolved,
                            "holding an outgoing packet behind one waiting for its SURBs"
                        );
                        if let Some(full) = wait.park(pseudonym, Parked::new(unresolved, data, output_span, surb_wait))
                        {
                            // The lane is full: the packet goes the way one that ran out of wait goes.
                            drop_hold_full(full);
                        }
                        return None;
                    }

                    hopr_transport_session::counters::ROUTING_RESOLUTION_ATTEMPTS.fetch_add(1, Ordering::Relaxed);
                    trace!(?unresolved, "resolving routing for packet");
                    match resolve(
                        data.data.total_len(),
                        data.estimate_surbs_with_msg(),
                        unresolved.clone(),
                    )
                    .await
                    {
                        Ok(resolution) => {
                            let resolved = stamp_resolution(&mut data, resolution, distress_threshold);
                            trace!(?resolved, "resolved routing for packet");
                            Some(crate::trace_context::WithContext::new((resolved, data), output_span))
                        }
                        Err(error) if error.is_surb() => {
                            // The wait must not happen here. This stage emits in submission order, so
                            // a packet waiting for its SURBs withholds every packet behind it, for
                            // every Session on the node. Park it on its own pseudonym's lane, where
                            // only that Session waits, and let the pump retry it.
                            match pseudonym.filter(|_| parking) {
                                Some(pseudonym) => {
                                    trace!(?unresolved, %error, "holding an outgoing packet until its SURBs arrive");
                                    match wait.park(pseudonym, Parked::new(unresolved, data, output_span, surb_wait)) {
                                        None => {}
                                        Some(full) => drop_hold_full(full),
                                    }
                                }
                                None => drop_starved(
                                    Parked::new(unresolved, data, output_span, surb_wait),
                                    surb_wait,
                                    &error,
                                ),
                            }
                            None
                        }
                        Err(error) => {
                            drop_unroutable(data, &error);
                            None
                        }
                    }
                }
                .instrument(tracing::debug_span!(parent: &span, "routing.resolve"))
                .instrument(span)
            })
            .buffered(concurrency)
            .filter_map(futures::future::ready)
    };

    // Held packets go out ahead of new data, so a Session whose SURBs have come back is served
    // before the packets that queued behind it.
    let held_packets = wait.clone();
    let mut resolved = Box::pin(resolved);
    let mut held = Box::pin(parked_packets(wait, wake, resolve, distress_threshold, surb_wait));

    // The stream ends when nothing can be emitted any more: the stage's input is spent, and no
    // packet is left waiting for SURBs. Ending on the input alone would drop the packets the hold is
    // still carrying; ending on the hold alone would leave a stream that never completes once the
    // node stops originating.
    let (mut input_done, mut held_done) = (false, false);
    futures::stream::poll_fn(move |cx| {
        if !held_done {
            match held.as_mut().poll_next(cx) {
                Poll::Ready(Some(emitted)) => return Poll::Ready(Some(emitted)),
                Poll::Ready(None) => held_done = true,
                // Pending, and the pump has registered for the wake that would change that.
                Poll::Pending => {}
            }
        }
        if !input_done {
            match resolved.as_mut().poll_next(cx) {
                Poll::Ready(Some(emitted)) => return Poll::Ready(Some(emitted)),
                Poll::Ready(None) => input_done = true,
                Poll::Pending => {}
            }
        }
        if input_done && (held_done || held_packets.is_empty()) {
            return Poll::Ready(None);
        }
        // At least one side returned `Pending` above, so this poll has a waker to be woken by.
        Poll::Pending
    })
}

/// A packet that found no SURB for its return path, waiting outside the ordered stage.
struct Parked {
    /// Its unresolved routing. Every retry resolves it again, because the SURB store may serve a
    /// newer route than the one that failed, and the encode stage needs the one the SURB came from.
    routing: DestinationRouting,
    data: ApplicationDataOut,
    /// The packet's tracing context, reattached when the packet leaves the hold.
    span: tracing::Span,
    /// When the packet is dropped instead of waiting further.
    deadline: Instant,
}

impl Parked {
    /// Wraps `data` as waiting, with the deadline `surb_wait` away.
    fn new(routing: DestinationRouting, data: ApplicationDataOut, span: tracing::Span, surb_wait: Duration) -> Self {
        Self {
            routing,
            data,
            span,
            deadline: Instant::now() + surb_wait,
        }
    }
}

/// One pseudonym's lane of parked packets.
#[derive(Default)]
struct Lane {
    /// Serializes every decision about this lane.
    ///
    /// The stage takes it to look for a queue, park, and resolve, and the pump takes it to retry the
    /// head. Holding it across all three is what keeps a packet from resolving ahead of one already
    /// parked. It is an async lock because it spans the resolve; the queue beside it is behind a
    /// lock that never spans an `await`.
    gate: Arc<futures::lock::Mutex<()>>,
    queue: VecDeque<Parked>,
}

/// The packets waiting for SURBs, one FIFO for each pseudonym.
///
/// The wait cannot live in the stage's ordered stream. [`buffered`](futures::StreamExt::buffered)
/// emits in submission order, so one packet waiting for its SURBs withholds every packet behind it,
/// for every Session on the node, for as long as the wait lasts. That is a node-wide stall caused by
/// one Session that owes more return packets than it has SURBs, which is exactly what a busy Exit
/// does when a client without a SURB balancer enables retransmission.
struct SurbWait {
    /// One lane for each pseudonym with a parked packet.
    lanes: Mutex<HashMap<HoprPseudonym, Lane>>,
    /// Packets parked across all lanes, so the pump can tell an idle node from a busy one without
    /// taking the map lock.
    parked: AtomicUsize,
    /// Wakes the pump when a lane goes from empty to non-empty.
    wake: futures::channel::mpsc::UnboundedSender<()>,
}

impl SurbWait {
    /// Creates the hold, with the receiver the pump waits on while it has nothing to retry.
    fn new() -> (Arc<Self>, futures::channel::mpsc::UnboundedReceiver<()>) {
        let (wake, wake_rx) = futures::channel::mpsc::unbounded();
        (
            Arc::new(Self {
                lanes: Mutex::new(HashMap::new()),
                parked: AtomicUsize::new(0),
                wake,
            }),
            wake_rx,
        )
    }

    /// The gate that serializes everything about `pseudonym`'s lane.
    fn gate(&self, pseudonym: HoprPseudonym) -> Arc<futures::lock::Mutex<()>> {
        self.lanes.lock().entry(pseudonym).or_default().gate.clone()
    }

    /// Whether no packet is waiting anywhere.
    ///
    /// Approximate off the map lock, and safe: a park that loses the race also wakes the pump.
    fn is_empty(&self) -> bool {
        self.parked.load(Ordering::Acquire) == 0
    }

    /// Packets waiting for `pseudonym`'s SURBs.
    fn queued(&self, pseudonym: HoprPseudonym) -> usize {
        self.lanes.lock().get(&pseudonym).map_or(0, |lane| lane.queue.len())
    }

    /// The pseudonyms with a lane, for one retry cycle.
    fn pseudonyms(&self) -> Vec<HoprPseudonym> {
        self.lanes.lock().keys().copied().collect()
    }

    /// Parks `parked` behind everything already queued for `pseudonym`.
    ///
    /// Hands it back when the lane is full, so the caller drops it the way it drops a packet whose
    /// wait ran out.
    fn park(&self, pseudonym: HoprPseudonym, parked: Parked) -> Option<Parked> {
        let mut lanes = self.lanes.lock();
        let lane = lanes.entry(pseudonym).or_default();
        if lane.queue.len() >= MAX_PARKED_PER_PSEUDONYM {
            return Some(parked);
        }
        lane.queue.push_back(parked);
        drop(lanes);
        self.note_parked();
        None
    }

    /// Puts `parked` back at the head of `pseudonym`'s lane, ahead of everything behind it.
    ///
    /// No wake: the pump is the one that took this packet out to retry it, so it is already running
    /// and will come back to this lane.
    fn return_front(&self, pseudonym: HoprPseudonym, parked: Parked) {
        self.lanes.lock().entry(pseudonym).or_default().queue.push_front(parked);
    }

    /// Takes the packet at the head of `pseudonym`'s lane.
    ///
    /// The packet still counts as parked while the pump holds it, so the hold reads as empty only
    /// when nothing is left to emit. The pump calls [`Self::forget`] once it has emitted or dropped
    /// the packet.
    fn pop_front(&self, pseudonym: HoprPseudonym) -> Option<Parked> {
        self.lanes.lock().get_mut(&pseudonym)?.queue.pop_front()
    }

    /// Drops one packet from the count, after the pump emitted or dropped it.
    fn forget(&self) {
        self.parked.fetch_sub(1, Ordering::AcqRel);
    }

    /// Drops `pseudonym`'s lane once it is empty and nobody else is using it.
    ///
    /// Safe against a park racing it: the gate is cloned under the map lock this takes, so a
    /// reference count of one means the map is the gate's only owner.
    fn release(&self, pseudonym: HoprPseudonym) {
        let mut lanes = self.lanes.lock();
        if lanes
            .get(&pseudonym)
            .is_some_and(|lane| lane.queue.is_empty() && Arc::strong_count(&lane.gate) == 1)
        {
            lanes.remove(&pseudonym);
        }
    }

    /// Counts one more parked packet, waking the pump when the hold was idle.
    fn note_parked(&self) {
        if self.parked.fetch_add(1, Ordering::AcqRel) == 0 {
            let _ = self.wake.unbounded_send(());
        }
    }
}

/// The pseudonym a routing names, which is what the hold keys on.
///
/// Both variants carry one. Only return routing starves for SURBs, so a forward routing without a
/// pseudonym needs no lane, and `None` is the whole of that case.
fn lane_of(routing: &DestinationRouting) -> Option<HoprPseudonym> {
    match routing {
        DestinationRouting::Return(matcher) => Some(matcher.pseudonym()),
        DestinationRouting::Forward { pseudonym, .. } => *pseudonym,
    }
}

/// Stamps the packet with what the resolution decided about its return path.
///
/// The flags become HOPR protocol packet signals the destination reads, and the generation is the
/// one the SURBs in the packet were minted under.
fn stamp_resolution(
    data: &mut ApplicationDataOut,
    (resolved, remaining_surbs, generation): (ResolvedTransportRouting<HoprSurb>, Option<usize>, Option<u8>),
    distress_threshold: usize,
) -> ResolvedTransportRouting<HoprSurb> {
    // Set the SURB distress/out-of-SURBs flag if applicable.
    // These flags are translated into HOPR protocol packet signals and are applicable only on the
    // return path.
    let mut signals_to_dst = data
        .packet_info
        .as_ref()
        .map(|info| info.signals_to_destination)
        .unwrap_or_default();

    if resolved.is_return() {
        signals_to_dst = match remaining_surbs {
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
    resolved
}

/// Drops a packet whose return path found no SURB within `surb_wait`, counted and refunded.
fn drop_starved(parked: Parked, surb_wait: Duration, error: &PathPlannerError) {
    let Parked { routing, data, .. } = parked;
    hopr_transport_session::counters::ROUTING_RESOLUTION_SURB_TIMEOUTS.fetch_add(1, Ordering::Relaxed);
    // Warn rather than trace: this is the only externally visible sign that a counterparty has
    // stopped replenishing, and the outage it replaces was invisible precisely because nothing on
    // this path said anything.
    warn!(
        unresolved = ?routing,
        ?surb_wait,
        %error,
        "dropping an outgoing packet: no SURB for its return path within the wait"
    );
    // Its sender may have counted the SURB as consumed already, and the packet leaves without one.
    data.surb_refund.give_back();
}

/// Drops a packet whose pseudonym already holds the most parked packets it may, counted and refunded.
fn drop_hold_full(parked: Parked) {
    let Parked { routing, data, .. } = parked;
    hopr_transport_session::counters::ROUTING_RESOLUTION_SURB_TIMEOUTS.fetch_add(1, Ordering::Relaxed);
    warn!(
        unresolved = ?routing,
        limit = MAX_PARKED_PER_PSEUDONYM,
        "dropping an outgoing packet: no SURB for its return path and the pseudonym holds too many"
    );
    data.surb_refund.give_back();
}

/// Drops a packet whose routing cannot be resolved at all, counted.
fn drop_unroutable(_data: ApplicationDataOut, error: &PathPlannerError) {
    hopr_transport_session::counters::ROUTING_RESOLUTION_FAILURES.fetch_add(1, Ordering::Relaxed);
    error!(%error, "failed to resolve routing");
}

/// The state of the pump that emits parked packets once their SURBs arrive.
struct Pump<F> {
    wait: Arc<SurbWait>,
    wake: futures::channel::mpsc::UnboundedReceiver<()>,
    resolve: F,
    distress_threshold: usize,
    surb_wait: Duration,
    /// The lanes to attempt in this cycle.
    cycle: VecDeque<HoprPseudonym>,
    /// The earliest time the next cycle may start.
    next_cycle: Instant,
}

impl<F, Fut> Pump<F>
where
    F: Fn(usize, usize, DestinationRouting) -> Fut,
    Fut: Future<Output = PathResult<(ResolvedTransportRouting<HoprSurb>, Option<usize>, Option<u8>)>>,
{
    /// Retries the head of `pseudonym`'s lane, emitting it when its SURBs have arrived.
    async fn retry(
        &self,
        pseudonym: HoprPseudonym,
    ) -> Option<crate::trace_context::WithContext<(ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)>> {
        let emitted = {
            let gate = self.wait.gate(pseudonym);
            let _lane = gate.clone().lock_owned().await;
            self.attempt(pseudonym).await
        };
        // A lane with nothing left goes away, so a pseudonym that starved once costs nothing after.
        if emitted.is_none() {
            self.wait.release(pseudonym);
        }
        emitted
    }

    /// Makes one resolve attempt on the head of `pseudonym`'s lane.
    async fn attempt(
        &self,
        pseudonym: HoprPseudonym,
    ) -> Option<crate::trace_context::WithContext<(ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)>> {
        let mut parked = self.wait.pop_front(pseudonym)?;

        hopr_transport_session::counters::ROUTING_RESOLUTION_ATTEMPTS.fetch_add(1, Ordering::Relaxed);
        trace!(unresolved = ?parked.routing, "retrying an outgoing packet that found no SURB");
        match (self.resolve)(
            parked.data.data.total_len(),
            parked.data.estimate_surbs_with_msg(),
            parked.routing.clone(),
        )
        .instrument(parked.span.clone())
        .await
        {
            Ok(resolution) => {
                let resolved = stamp_resolution(&mut parked.data, resolution, self.distress_threshold);
                trace!(?resolved, "resolved routing for a held packet");
                self.wait.forget();
                Some(crate::trace_context::WithContext::new(
                    (resolved, parked.data),
                    parked.span,
                ))
            }
            Err(error) if error.is_surb() => {
                if Instant::now() >= parked.deadline {
                    self.wait.forget();
                    drop_starved(parked, self.surb_wait, &error);
                } else {
                    // Still no SURB, and still time: back at the head, ahead of whatever parked
                    // behind it while it was out.
                    self.wait.return_front(pseudonym, parked);
                }
                None
            }
            Err(error) => {
                self.wait.forget();
                drop_unroutable(parked.data, &error);
                None
            }
        }
    }
}

/// Emits the packets the stage parked, as their SURBs arrive.
///
/// This runs beside the ordered stage instead of inside it, which is the point: a retry that
/// succeeded inside the stage would still have to wait for every earlier packet of every other
/// Session. One lane is attempted per cycle, which suffices because a retry is a lookup on the SURB
/// store, and the cycle itself is paced by [`SURB_RETRY_INTERVAL`].
fn parked_packets<F, Fut>(
    wait: Arc<SurbWait>,
    wake: futures::channel::mpsc::UnboundedReceiver<()>,
    resolve: F,
    distress_threshold: usize,
    surb_wait: Duration,
) -> impl Stream<Item = crate::trace_context::WithContext<(ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)>>
where
    F: Fn(usize, usize, DestinationRouting) -> Fut + Clone,
    Fut: Future<Output = PathResult<(ResolvedTransportRouting<HoprSurb>, Option<usize>, Option<u8>)>>,
{
    futures::stream::unfold(
        Pump {
            wait,
            wake,
            resolve,
            distress_threshold,
            surb_wait,
            cycle: VecDeque::new(),
            next_cycle: Instant::now() + SURB_RETRY_INTERVAL,
        },
        |mut pump| async move {
            loop {
                if pump.cycle.is_empty() {
                    if pump.wait.is_empty() {
                        // Nothing is parked: sleep until something is, instead of polling.
                        pump.wake.next().await?;
                    }
                    let now = Instant::now();
                    if pump.next_cycle > now {
                        futures_timer::Delay::new(pump.next_cycle - now).await;
                    }
                    pump.next_cycle = Instant::now() + SURB_RETRY_INTERVAL;
                    pump.cycle = pump.wait.pseudonyms().into();
                }
                let Some(pseudonym) = pump.cycle.pop_front() else {
                    continue;
                };
                if let Some(emitted) = pump.retry(pseudonym).await {
                    return Some((emitted, pump));
                }
            }
        },
    )
}

#[cfg(test)]
pub(crate) fn resolve_routing_stage<St, F, Fut>(
    input: St,
    resolve: F,
    distress_threshold: usize,
    concurrency: usize,
    surb_wait: std::time::Duration,
) -> impl Stream<Item = (ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)>
where
    St: Stream<Item = (DestinationRouting, ApplicationDataOut)>,
    F: Fn(usize, usize, DestinationRouting) -> Fut + Clone,
    Fut: Future<Output = PathResult<(ResolvedTransportRouting<HoprSurb>, Option<usize>, Option<u8>)>>,
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

    /// A packet with `len` payload bytes, the first of them `marker`, so its size hint identifies it.
    fn packet_of_len(marker: u8, len: usize) -> ApplicationDataOut {
        let mut payload = vec![marker];
        payload.resize(len, 0);
        ApplicationDataOut::with_no_packet_info(
            ApplicationData::new(Tag::from(TEST_TAG), &payload).expect("a short payload is valid"),
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

        // Polled past the starved packet's deadline, so its drop is observed here as well.
        let emitted = stage(
            input,
            |_size_hint, _max_surbs, routing: DestinationRouting| async move {
                match routing {
                    DestinationRouting::Return(_) => Err(no_surb(&routing)),
                    DestinationRouting::Forward { .. } => Ok((resolved(), None, None)),
                }
            },
        )
        .take_until(futures_timer::Delay::new(2 * TEST_SURB_WAIT))
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

    /// A packet waiting for its SURBs must not withhold another pseudonym's packet.
    ///
    /// This is the exit stall the issue reports: one Session that owes more return packets than it
    /// holds SURBs made every other Session's traffic wait for the full `surb_resolution_wait`, once
    /// per packet. The bound here is a fraction of the wait, so it cannot be met by waiting the
    /// starved packet out.
    #[test_log::test(tokio::test)]
    async fn a_packet_should_not_wait_for_another_pseudonyms_surbs() -> anyhow::Result<()> {
        let (starved, served) = (HoprPseudonym::random(), HoprPseudonym::random());
        let started = std::time::Instant::now();

        let emitted = stage(
            futures::stream::iter(vec![
                (return_routing(starved), packet(0)),
                (return_routing(served), packet(1)),
            ]),
            move |_size_hint, _max_surbs, routing: DestinationRouting| async move {
                match &routing {
                    DestinationRouting::Return(matcher) if matcher.pseudonym() == served => {
                        Ok((resolved(), Some(5), None))
                    }
                    _ => Err(no_surb(&routing)),
                }
            },
        )
        .take(1)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TEST_TIMEOUT))
        .await
        .map_err(|_| {
            anyhow::anyhow!(
                "the packet of a pseudonym with SURBs never came out while another pseudonym waited for its own"
            )
        })?;

        assert_eq!(
            vec![1],
            emitted.iter().map(|(_, data)| marker_of(data)).collect::<Vec<_>>(),
            "the served pseudonym's packet is the one that goes out"
        );
        assert!(
            started.elapsed() < TEST_SURB_WAIT / 4,
            "the served pseudonym's packet waited {:?}, so it was withheld behind the starved one",
            started.elapsed()
        );

        Ok(())
    }

    /// Packets of one pseudonym never overtake one another.
    ///
    /// The later packet of the pseudonym would resolve on its own and must still wait for the one
    /// ahead of it, which starves for its whole deadline. The two are told apart by their size hint,
    /// the only thing besides the routing that the resolution step sees.
    #[test_log::test(tokio::test)]
    async fn a_packet_should_not_overtake_a_starved_one_of_its_pseudonym() -> anyhow::Result<()> {
        let pseudonym = HoprPseudonym::random();
        // The starved packet's size hint: a tag and one payload byte.
        let starved_size_hint = packet(0).data.total_len();
        assert_eq!(
            starved_size_hint + 1,
            packet_of_len(1, 2).data.total_len(),
            "fixture: the two packets must differ by their size hint"
        );
        let started = std::time::Instant::now();

        let emitted = stage(
            futures::stream::iter(vec![
                (return_routing(pseudonym), packet(0)),
                (return_routing(pseudonym), packet_of_len(1, 2)),
            ]),
            move |size_hint, _max_surbs, routing: DestinationRouting| async move {
                if size_hint == starved_size_hint {
                    Err(no_surb(&routing))
                } else {
                    Ok((resolved(), Some(5), None))
                }
            },
        )
        .take(1)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TEST_TIMEOUT))
        .await
        .map_err(|_| anyhow::anyhow!("the packet behind the starved one was never emitted"))?;

        assert_eq!(
            vec![1],
            emitted.iter().map(|(_, data)| marker_of(data)).collect::<Vec<_>>(),
            "only the later packet can be emitted; the starved one is dropped"
        );
        assert!(
            started.elapsed() >= TEST_SURB_WAIT,
            "the later packet was emitted after {:?}, before the packet ahead of it reached its deadline at \
             {TEST_SURB_WAIT:?}",
            started.elapsed()
        );

        Ok(())
    }

    /// A pseudonym that holds the most packets it may parks no more: the excess is dropped and counted.
    ///
    /// A parked packet has left the bounded channel that feeds the stage, so the hold's own bound is
    /// what keeps a counterparty that never sends another SURB from pinning memory for good.
    #[test_log::test(tokio::test)]
    async fn a_full_lane_should_drop_the_excess_it_cannot_hold() -> anyhow::Result<()> {
        let starved = HoprPseudonym::random();
        let refunds = Arc::new(AtomicU64::new(0));
        let mut counted = packet(0);
        counted.surb_refund = SurbRefund::to(refunds.clone());
        let dropped_before = hopr_transport_session::counters::routing_resolution_surb_timeout_count();

        // One packet more than the lane holds, plus one that the stage must still be able to serve.
        let mut input: Vec<(DestinationRouting, ApplicationDataOut)> = (0..=MAX_PARKED_PER_PSEUDONYM)
            .map(|_| (return_routing(starved), counted.clone()))
            .collect();
        input.push((forward_routing(), packet(1)));

        let emitted = stage(
            futures::stream::iter(input),
            |_size_hint, _max_surbs, routing: DestinationRouting| async move {
                match routing {
                    DestinationRouting::Return(_) => Err(no_surb(&routing)),
                    DestinationRouting::Forward { .. } => Ok((resolved(), None, None)),
                }
            },
        )
        .take(1)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TEST_TIMEOUT))
        .await
        .map_err(|_| anyhow::anyhow!("a full lane withheld the packet behind it"))?;

        assert_eq!(
            vec![1],
            emitted.iter().map(|(_, data)| marker_of(data)).collect::<Vec<_>>(),
            "only the packet that finds a route goes on; the lane's excess is dropped"
        );
        assert!(
            hopr_transport_session::counters::routing_resolution_surb_timeout_count() > dropped_before,
            "the packet the full lane refused was dropped without being counted"
        );
        assert_eq!(
            1,
            refunds.load(Ordering::Relaxed),
            "the dropped packet must give back the SURB its sender counted, and only that one"
        );

        Ok(())
    }

    /// A packet that a retry can no longer route is dropped and counted, and the stage carries on.
    ///
    /// The retry runs outside the ordered stage, so its failure path is its own: it must count the
    /// failure and leave the stream able to serve the next packet. The stage has to stay polled for
    /// the retry to run at all, which is what this test does while it waits for the count.
    #[test_log::test(tokio::test)]
    async fn a_held_packet_that_cannot_be_routed_any_more_should_be_dropped_and_counted() -> anyhow::Result<()> {
        let starved = HoprPseudonym::random();
        let failed_before = hopr_transport_session::counters::routing_resolution_failure_count();
        let attempts = Arc::new(AtomicUsize::new(0));

        let mut stage = Box::pin({
            let attempts = attempts.clone();
            stage(
                futures::stream::iter(vec![
                    (return_routing(starved), packet(0)),
                    (forward_routing(), packet(1)),
                ]),
                move |_size_hint, _max_surbs, routing: DestinationRouting| {
                    let attempts = attempts.clone();
                    async move {
                        match routing {
                            // Starved on the first attempt, unroutable on the retry.
                            DestinationRouting::Return(_) => match attempts.fetch_add(1, Ordering::Relaxed) {
                                0 => Err(no_surb(&routing)),
                                _ => Err(PathPlannerError::Api("path is gone".into())),
                            },
                            DestinationRouting::Forward { .. } => Ok((resolved(), None, None)),
                        }
                    }
                },
            )
        });

        let first = stage
            .next()
            .timeout(futures_time::time::Duration::from(TEST_TIMEOUT))
            .await
            .map_err(|_| anyhow::anyhow!("the packet behind the held one was withheld"))?
            .ok_or_else(|| anyhow::anyhow!("the stage emitted nothing"))?;
        assert_eq!(1, marker_of(&first.1), "only the packet that finds a route goes on");

        // The retry fires a `SURB_RETRY_INTERVAL` or two later, and only while the stage is polled.
        let give_up = std::time::Instant::now() + 5 * TEST_SURB_WAIT;
        while hopr_transport_session::counters::routing_resolution_failure_count() == failed_before {
            anyhow::ensure!(
                std::time::Instant::now() < give_up,
                "the held packet was dropped on its retry without being counted"
            );
            match stage
                .next()
                .timeout(futures_time::time::Duration::from(4 * SURB_RETRY_INTERVAL))
                .await
            {
                // Nothing is left once the dropped packet has left the hold.
                Ok(None) => break,
                Ok(Some(_)) => anyhow::bail!("the packet whose retry cannot route must not be emitted"),
                Err(_) => {}
            }
        }
        anyhow::ensure!(
            hopr_transport_session::counters::routing_resolution_failure_count() > failed_before,
            "the held packet was dropped on its retry without being counted"
        );

        Ok(())
    }

    /// A packet that waits out its deadline is dropped, counted, and gives back the SURB it took.
    ///
    /// The packet is parked rather than dropped on the first miss, which is what makes the counter
    /// and the refund reachable only through the deadline path.
    #[test_log::test(tokio::test)]
    async fn a_parked_packet_should_be_dropped_counted_and_refunded_at_its_deadline() -> anyhow::Result<()> {
        let refunds = Arc::new(AtomicU64::new(0));
        let mut counted = packet(0);
        counted.surb_refund = SurbRefund::to(refunds.clone());
        let dropped_before = hopr_transport_session::counters::routing_resolution_surb_timeout_count();
        let started = std::time::Instant::now();

        // Nothing can ever be emitted, so the stage ends once the packet is dropped at its deadline.
        let emitted = stage(
            futures::stream::iter(vec![(return_routing(HoprPseudonym::random()), counted)]),
            |_size_hint, _max_surbs, routing: DestinationRouting| async move { Err(no_surb(&routing)) },
        )
        .next()
        .timeout(futures_time::time::Duration::from(2 * TEST_SURB_WAIT))
        .await
        .map_err(|_| anyhow::anyhow!("the stage never reached the packet's deadline"))?;

        assert!(
            emitted.is_none(),
            "a packet that never finds a SURB must not be emitted"
        );
        assert!(
            started.elapsed() >= TEST_SURB_WAIT,
            "the packet was dropped before the wait it is entitled to"
        );
        // Global counter, so other tests running beside this one may have moved it too: what this
        // test pins is that the drop counted at all.
        assert!(
            hopr_transport_session::counters::routing_resolution_surb_timeout_count() > dropped_before,
            "the dropped packet must be counted, or the drop stays invisible"
        );
        assert_eq!(
            1,
            refunds.load(Ordering::Relaxed),
            "a dropped packet must give back the SURB its sender counted"
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
