//! This module defines the [`SocketState`] that turns [`SessionSocket`](super::SessionSocket) into
//! a reliable socket, with segment/frame retransmission and frame acknowledgements.

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, atomic::AtomicBool},
    time::{Duration, Instant},
};

use futures::{FutureExt, StreamExt};
use futures_time::stream::StreamExt as TimeStreamExt;
use hopr_utils::network_types::crossfire_sink::{CrossfireSink, bounded_sink_channel};
use tracing::Instrument;

use crate::{
    errors::SessionError,
    flow_control::DeliveryTap,
    processing::types::FrameInspector,
    protocol::{FrameAcknowledgements, FrameId, Segment, SegmentId, SegmentRequest, SeqIndicator, SessionMessage},
    socket::{SocketState, state::SocketComponents},
    utils::{
        RetriedFrameId, RingBufferProducer, RingBufferView, next_deadline_with_backoff, searchable_ringbuffer,
        skip_queue::{Skip, SkipDelaySender, skip_delay_channel},
    },
};

/// Indicates the acknowledgement mode of a [stateful](AcknowledgementState) Session socket.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AcknowledgementMode {
    /// Partial frames are acknowledged, leading to receiver-driven retransmission requests.
    /// The frame sender is never going to retransmit the entire frame.
    Partial,
    /// Only full frames are acknowledged, leading to the full-frame retransmission if
    /// no acknowledgement is received by the frame sender.
    Full,
    /// Both partial and full acknowledgements are sent by the receiver.
    ///
    /// If a frame is partially acknowledged first, only receiver-driven retransmission requests follow
    /// (as with [`AcknowledgementMode::Partial`].
    ///
    /// If the frame sender receives no acknowledgement (partial nor full), it retransmits
    /// the entire frame (as in [`AcknowledgementMode::Full`]).
    #[default]
    Both,
}

impl AcknowledgementMode {
    /// Indicates if `self` is [`AcknowledgementMode::Partial`] or [`AcknowledgementMode::Both`].
    #[inline]
    fn is_partial_ack_enabled(&self) -> bool {
        matches!(self, Self::Partial | Self::Both)
    }

    /// Indicates if `self` is [`AcknowledgementMode::Full`] or [`AcknowledgementMode::Both`].
    #[inline]
    fn is_full_ack_enabled(&self) -> bool {
        matches!(self, Self::Full | Self::Both)
    }
}

/// Configuration object of the [`AcknowledgementState`].
#[derive(Debug, Clone, Copy, PartialEq, smart_default::SmartDefault)]
pub struct AcknowledgementStateConfig {
    /// Mode of frame acknowledgement.
    ///
    /// Default is [`AcknowledgementMode::Both`]
    pub mode: AcknowledgementMode,

    /// The expected (average) latency of a packet (= single frame segment).
    ///
    /// Default is 20 ms
    #[default(Duration::from_millis(20))]
    pub expected_packet_latency: Duration,

    /// Backoff base applied for segment or frame retransmissions.
    ///
    /// Default is 1.2
    #[default(1.2)]
    pub backoff_base: f64,

    /// The maximum number of receiver-driven segment retransmission requests.
    ///
    /// Default is 3
    #[default(3)]
    pub max_incoming_frame_retries: usize,

    /// The maximum number of sender-driven full-frame retransmissions.
    ///
    /// Default is 3
    #[default(3)]
    pub max_outgoing_frame_retries: usize,

    /// Delay between acknowledgement batches.
    ///
    /// Default is 50 ms
    #[default(Duration::from_millis(50))]
    pub acknowledgement_delay: Duration,

    /// Number of segments to hold back for retransmission upon other party's request.
    /// Minimum is 1024.
    ///
    /// Default is 16 384.
    #[default(16384)]
    pub lookbehind_segments: usize,

    /// Maximum age of an outgoing frame before it is retired as lost instead of retransmitted.
    ///
    /// Bounds how stale data on the wire may be after a stall, and feeds the retirement to the
    /// honest delivery clock so flow control sees the stall as loss.
    ///
    /// `None` (default) bounds frames only by [`Self::max_outgoing_frame_retries`] and backoff.
    #[default(None)]
    pub max_frame_age: Option<Duration>,

    /// Minimum interval between two peer-driven retransmission services of the same frame.
    ///
    /// Bounds the egress (and SURB/ticket) cost a single peer can induce by replaying
    /// [segment retransmission requests](SegmentRequest). Deliberately HALF the RTO base
    /// ([`Self::expected_packet_latency`]): an honest peer's requests for the same frame arrive
    /// `RTO + jitter` apart, so an interval equal to the RTO could marginally throttle it.
    ///
    /// `None` (default) or `ZERO` derives `expected_packet_latency / 2`, so the bound cannot be
    /// disabled by accident. Internal defaults only; not operator-visible.
    #[default(None)]
    pub min_frame_retransmission_interval: Option<Duration>,

    /// Maximum number of times a single frame may be served on a peer's request.
    ///
    /// `0` (default) derives
    /// `max_incoming_frame_retries.max(1) * SegmentRequest::<C>::MAX_MISSING_SEGMENTS_PER_FRAME`,
    /// i.e. an 8x margin over the at most `max_incoming_frame_retries` requests the receiver
    /// itself could ever have issued for the same frame.
    /// Internal defaults only; not operator-visible.
    #[default(0)]
    pub max_frame_retransmission_services: usize,

    /// Maximum number of frames served across a [`Self::frame_retransmission_refill_interval`].
    ///
    /// Caps the total amplification one peer's request stream can induce, regardless of how many
    /// distinct frames it names. `0` (default) derives `2 * SegmentRequest::<C>::MAX_ENTRIES`
    /// (a full request is worth at most `MAX_ENTRIES` frames).
    /// Internal defaults only; not operator-visible.
    #[default(0)]
    pub max_frame_retransmission_burst: usize,

    /// Interval over which the [`Self::max_frame_retransmission_burst`] allowance refills.
    ///
    /// `None` (default) derives `expected_packet_latency`. Internal defaults only; not
    /// operator-visible.
    #[default(None)]
    pub frame_retransmission_refill_interval: Option<Duration>,
}

impl AcknowledgementStateConfig {
    fn normalize<const C: usize>(self) -> AcknowledgementStateConfig {
        // Derived budget bounds build on the clamped latency, so a zero latency can never
        // derive a zero interval (which would disable the bound entirely).
        let expected_packet_latency = self.expected_packet_latency.max(Duration::from_millis(1));

        Self {
            mode: self.mode,
            expected_packet_latency,
            backoff_base: self.backoff_base.max(1.0),
            max_incoming_frame_retries: self.max_incoming_frame_retries,
            max_outgoing_frame_retries: self.max_outgoing_frame_retries,
            acknowledgement_delay: self.acknowledgement_delay.max(Duration::from_millis(1)),
            lookbehind_segments: self.lookbehind_segments.max(1024),
            // A zero bound would retire every frame on sight; treat it as "not set".
            max_frame_age: self.max_frame_age.filter(|age| !age.is_zero()),
            // An interval of zero would allow unlimited same-frame replay; treat it as "unset",
            // so the peer-driven path can never lose its budget by accident. Deliberately HALF
            // the RTO base: an honest peer's requests for the same frame arrive `RTO + jitter`
            // apart, so an interval equal to the RTO could marginally throttle it.
            min_frame_retransmission_interval: Some(
                self.min_frame_retransmission_interval
                    .filter(|interval| !interval.is_zero())
                    .unwrap_or_else(|| expected_packet_latency / 2),
            ),
            // A service cap of zero would refuse every request; derive it from how many times
            // the receiver itself could ever have asked for the frame. An honest receiver issues
            // at most `max_incoming_frame_retries` requests per frame, so the product of that and
            // the per-frame segment count is an 8x margin over what an honest peer can generate.
            max_frame_retransmission_services: if self.max_frame_retransmission_services == 0 {
                self.max_incoming_frame_retries.max(1) * SegmentRequest::<C>::MAX_MISSING_SEGMENTS_PER_FRAME
            } else {
                self.max_frame_retransmission_services
            },
            // A burst of zero would refuse every request; a full request names at most
            // `MAX_ENTRIES` frames, so twice that is a comfortable ceiling.
            max_frame_retransmission_burst: if self.max_frame_retransmission_burst == 0 {
                2 * SegmentRequest::<C>::MAX_ENTRIES
            } else {
                self.max_frame_retransmission_burst
            },
            frame_retransmission_refill_interval: Some(
                self.frame_retransmission_refill_interval
                    .filter(|interval| !interval.is_zero())
                    .unwrap_or(expected_packet_latency),
            ),
        }
    }
}

/// How many times a single frame has been served on a peer's request, and when last.
#[derive(Debug, Clone, Copy)]
struct FrameServices {
    served: usize,
    last: Instant,
}

/// Peer-driven retransmission budget.
///
/// Locally-initiated retransmissions are budgeted (`max_incoming_frame_retries` with backoff,
/// `max_outgoing_frame_retries`); the peer-driven path historically was not, so a single peer
/// could replay a `SegmentRequest` naming thousands of segments and force unbounded paid egress.
/// This bounds it with three limits, derived from [`AcknowledgementStateConfig`]:
/// a per-frame service count, a minimum interval between two services of the same frame, and a
/// session-wide token bucket capping how many frames can be served per refill interval.
///
/// The budget is shared by all clones of an [`AcknowledgementState`] (it lives behind an
/// `Arc` in [`AcknowledgementStateContext`]), since the state is cloned per pipeline stage.
#[derive(Debug)]
struct RetransmissionBudget {
    /// Minimum interval between two services of the same frame.
    min_interval: Duration,
    /// Maximum number of times a single frame may be served.
    max_services: usize,
    /// Interval over which the token bucket refills.
    refill_interval: Duration,
    /// Maximum number of tokens the bucket can hold.
    max_tokens: usize,
    /// Currently available tokens; one is charged per admitted frame.
    tokens: usize,
    /// Last time the bucket was refilled.
    last_refill: Instant,
    /// Per-frame service bookkeeping. Bounded by [`Self::capacity`]: frame ids are arbitrary
    /// `u32`, so recording an entry per *requested* frame would let a bogus request grow this
    /// map without bound — entries are therefore created only for frames actually found in the
    /// ring buffer, which is itself bounded by the lookbehind window.
    services: HashMap<FrameId, FrameServices>,
    /// Hard bound on [`Self::services`], pruned at as defence in depth.
    capacity: usize,
    /// Number of frames refused by [`Self::admit`] since the last [`Self::take_throttled`].
    throttled: u64,
}

impl RetransmissionBudget {
    fn new(min_interval: Duration, max_services: usize, refill_interval: Duration, max_tokens: usize) -> Self {
        // Guard against zero/absurd refill intervals: a zero interval would grant unlimited
        // tokens on every call, turning the burst cap off entirely.
        let refill_interval = if refill_interval.is_zero() {
            Duration::from_millis(1)
        } else {
            refill_interval
        };
        Self {
            min_interval,
            max_services,
            refill_interval,
            max_tokens,
            tokens: max_tokens,
            last_refill: Instant::now(),
            services: HashMap::with_capacity(max_tokens.min(1024)),
            capacity: max_tokens.saturating_mul(16).max(1024),
            throttled: 0,
        }
    }

    /// Refills the token bucket proportionally to the time elapsed since the last refill.
    ///
    /// The bucket earns [`Self::max_tokens`] over one [`Self::refill_interval`], so a full
    /// allowance is available again after that interval — matching what the config documents
    /// (`max_frame_retransmission_burst` frames served *across* a refill interval). Earning a
    /// single token per interval instead would make a full refill take `max_tokens` intervals,
    /// starving honest retry requests for tens of seconds after any burst.
    ///
    /// Elapsed time that is not yet worth a whole token is kept, so a steady stream of calls
    /// refills at the same average rate as one call would.
    fn refill(&mut self, now: Instant) {
        // Nothing to earn while full; keep the clock current so a later drain starts accruing
        // from now rather than from a stale instant.
        if self.tokens >= self.max_tokens {
            self.last_refill = now;
            return;
        }

        let interval_nanos = self.refill_interval.as_nanos();
        if interval_nanos == 0 || self.max_tokens == 0 {
            return;
        }

        let elapsed_nanos = now.saturating_duration_since(self.last_refill).as_nanos();
        let earned = (elapsed_nanos.saturating_mul(self.max_tokens as u128) / interval_nanos) as usize;
        if earned == 0 {
            return;
        }

        // Advance the clock only by the time those whole tokens were worth, so the sub-token
        // remainder is not discarded on every call.
        let consumed_nanos = interval_nanos.saturating_mul(earned as u128) / (self.max_tokens as u128);
        self.last_refill += Duration::from_nanos(consumed_nanos.min(u64::MAX as u128) as u64);
        self.tokens = self.max_tokens.min(self.tokens.saturating_add(earned));
    }

    /// Pure admission check for `frame_id`; charges one session-wide token on success.
    ///
    /// MUST NOT create any per-frame bookkeeping: requests can name arbitrary, never-seen frame
    /// ids, and admitting them must not allocate (see [`Self::services`]). Call [`Self::record`]
    /// once the frame has actually been found in the ring buffer.
    fn admit(&mut self, frame_id: FrameId, now: Instant) -> bool {
        self.refill(now);

        let admitted = self.tokens > 0
            && self.services.get(&frame_id).is_none_or(|s| {
                now.saturating_duration_since(s.last) >= self.min_interval && s.served < self.max_services
            });

        if admitted {
            self.tokens -= 1;
        } else {
            self.throttled = self.throttled.saturating_add(1);
        }

        admitted
    }

    /// Charges the per-frame budget for a frame that has actually been served.
    ///
    /// Called only for frames found in the ring buffer, so this cannot be driven to grow
    /// [`Self::services`] with bogus frame ids.
    fn record(&mut self, frame_id: FrameId, now: Instant) {
        // Never let the bookkeeping map grow without bound, even if a pathological ring-buffer
        // contents somehow outruns the ring size.
        if self.services.len() >= self.capacity && !self.services.contains_key(&frame_id) {
            self.services.clear();
        }

        let entry = self
            .services
            .entry(frame_id)
            .or_insert(FrameServices { served: 0, last: now });
        entry.served = entry.served.saturating_add(1);
        entry.last = now;
    }

    /// Takes the number of frames refused since the last call, for count-batched logging.
    fn take_throttled(&mut self) -> u64 {
        std::mem::take(&mut self.throttled)
    }
}

#[derive(Clone)]
struct AcknowledgementStateContext<const C: usize> {
    rb_tx: RingBufferProducer<Segment>,
    rb_rx: RingBufferView<Segment>,
    incoming_frame_retries_tx: SkipDelaySender<RetriedFrameId>,
    outgoing_frame_retries_tx: SkipDelaySender<RetriedFrameId>,
    ack_tx: CrossfireSink<FrameId>,
    inspector: FrameInspector,
    ctl_tx: CrossfireSink<SessionMessage<C>>,
    /// Peer-driven retransmission budget, shared by all clones of the state.
    retransmission_budget: Arc<parking_lot::Mutex<RetransmissionBudget>>,
}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// Represents a Session socket state is able to process acknowledgements.
///
/// # Retransmission driven by the Receiver
/// ```mermaid
/// sequenceDiagram
///     Note over Sender,Receiver: Frame 1
///     rect rgb(191, 223, 255)
///     Note left of Sender: Frame 1 in buffer
///     Sender->>Receiver: Segment 1/3 of Frame 1
///     Sender->>Receiver: Segment 2/3 of Frame 1
///     Sender--xReceiver: Segment 3/3 of Frame 1
///     Note right of Receiver: RTO_BASE_RECEIVER elapsed
///     Receiver->>Sender: Request Segment 3 of Frame 1
///     Sender->>Receiver: Segment 3/3 of Frame 1
///     Receiver->>Sender: Acknowledge Frame 1
///     Note left of Sender: Frame 1 dropped from buffer
///     end
///     Note over Sender,Receiver: Frame 1 delivered
/// ```
///
/// # Retransmission driven by the Sender
/// ```mermaid
/// sequenceDiagram
///     Note over Sender,Receiver: Frame 1
///     rect rgb(191, 223, 255)
///     Note left of Sender: Frame 1 in buffer
///     Sender->>Receiver: Segment 1/3 of Frame 1
///     Sender->>Receiver: Segment 2/3 of Frame 1
///     Sender--xReceiver: Segment 3/3 of Frame 1
///     Note right of Receiver: RTO_BASE_RECEIVER elapsed
///     Receiver--xSender: Request Segment 3 of Frame 1
///     Note left of Sender: RTO_BASE_SENDER elapsed
///     Sender->>Receiver: Segment 1/3 of Frame 1
///     Sender->>Receiver: Segment 2/3 of Frame 1
///     Sender->>Receiver: Segment 3/3 of Frame 1
///     Receiver->>Sender: Acknowledge Frame 1
///     Note left of Sender: Frame 1 dropped from buffer
///     end
///     Note over Sender,Receiver: Frame 1 delivered
/// ```
///
/// # Sender-Receiver retransmission handover
///
/// ```mermaid
///    sequenceDiagram
///     Note over Sender,Receiver: Frame 1
///     rect rgb(191, 223, 255)
///     Note left of Sender: Frame 1 in buffer
///     Sender->>Receiver: Segment 1/3 of Frame 1
///     Sender--xReceiver: Segment 2/3 of Frame 1
///     Sender--xReceiver: Segment 3/3 of Frame 1
///     Note right of Receiver: RTO_BASE_RECEIVER elapsed
///     Receiver->>Sender: Request Segments 2,3 of Frame 1
///     Note left of Sender: RTO_BASE_SENDER cancelled
///     Sender->>Receiver: Segment 2/3 of Frame 1
///     Sender--xReceiver: Segment 3/3 of Frame 1
///     Note right of Receiver: RTO_BASE_RECEIVER elapsed
///     Receiver--xSender: Request Segments 3 of Frame 1
///     Note right of Receiver: RTO_BASE_RECEIVER elapsed
///     Receiver->>Sender: Request Segments 3 of Frame 1
///     Sender->>Receiver: Segment 3/3 of Frame 1
///     Receiver->>Sender: Acknowledge Frame 1
///     Note left of Sender: Frame 1 dropped from buffer
///     end
///     Note over Sender,Receiver: Frame 1 delivered
/// ```
///
/// # Retransmission failure
///
/// ```mermaid
///    sequenceDiagram
///     Note over Sender,Receiver: Frame 1
///     rect rgb(191, 223, 255)
///     Note left of Sender: Frame 1 in buffer
///     Sender->>Receiver: Segment 1/3 of Frame 1
///     Sender->>Receiver: Segment 2/3 of Frame 1
///     Sender--xReceiver: Segment 3/3 of Frame 1
///     Note right of Receiver: RTO_BASE_RECEIVER elapsed
///     Receiver--xSender: Request Segment 3 of Frame 1
///     Note left of Sender: RTO_BASE_SENDER elapsed
///     Sender--xReceiver: Segment 1/3 of Frame 1
///     Sender--xReceiver: Segment 2/3 of Frame 1
///     Sender--xReceiver: Segment 3/3 of Frame 1
///     Note left of Sender: FRAME_MAX_AGE elapsed<br/>Frame 1 dropped from buffer
///     Note right of Receiver: FRAME_MAX_AGE elapsed<br/>Frame 1 dropped from buffer
///     end
///     Note over Sender,Receiver: Frame 1 never delivered
/// ```
#[derive(Clone)]
pub struct AcknowledgementState<const C: usize> {
    id: String,
    cfg: AcknowledgementStateConfig,
    context: Option<AcknowledgementStateContext<C>>,
    started: std::sync::Arc<AtomicBool>,
    /// Optional honest-clock tap for client-side flow control. Purely observational: acknowledged
    /// and retransmission-exhausted frames bump the tap's atomic meter, but acknowledgement and
    /// retransmission behaviour is unchanged whether or not it is installed.
    delivery_tap: Option<DeliveryTap>,
}

impl<const C: usize> AcknowledgementState<C> {
    pub fn new<I: std::fmt::Display>(session_id: I, cfg: AcknowledgementStateConfig) -> Self {
        Self {
            id: session_id.to_string(),
            cfg: cfg.normalize::<C>(),
            context: Default::default(),
            started: std::sync::Arc::new(AtomicBool::new(false)),
            delivery_tap: None,
        }
    }

    /// Installs a read-only honest-clock tap. Each acknowledged frame bumps the tap's acked-bytes
    /// counter; each frame whose sender-side retransmissions are exhausted bumps its lost-bytes
    /// counter. This drives the client-side flow-control window ([`crate::flow_control`]) and never
    /// alters delivery semantics.
    pub fn with_delivery_tap(mut self, tap: DeliveryTap) -> Self {
        self.delivery_tap = Some(tap);
        self
    }
}

/// Whether the state is still running.
///
/// Background tasks use this to tell a failure from the normal end that follows
/// [`AcknowledgementState::stop`], which closes their channels on purpose.
fn is_running(started: &AtomicBool) -> bool {
    started.load(std::sync::atomic::Ordering::Acquire)
}

impl<const C: usize> SocketState<C> for AcknowledgementState<C> {
    fn session_id(&self) -> &str {
        &self.id
    }

    #[tracing::instrument(name = "AcknowledgementState", skip(self, socket_components), fields(session_id = self.id))]
    fn run(&mut self, socket_components: SocketComponents<C>) -> Result<(), SessionError> {
        if self.started.load(std::sync::atomic::Ordering::Relaxed) && self.context.is_some() {
            return Err(SessionError::InvalidState("state is already running".into()));
        }

        let (incoming_frame_retries_tx, incoming_frame_retries_rx) = skip_delay_channel();
        let (outgoing_frame_retries_tx, outgoing_frame_retries_rx) = skip_delay_channel();
        let (rb_tx, rb_rx) = searchable_ringbuffer(self.cfg.lookbehind_segments);

        // Full frame acknowledgements get a special channel with fixed capacity
        let (ack_tx, ack_rx) = bounded_sink_channel::<FrameId>(2 * self.cfg.lookbehind_segments);

        // Shared by every clone of this state, so all pipeline stages draw on one budget.
        let retransmission_budget = Arc::new(parking_lot::Mutex::new(RetransmissionBudget::new(
            self.cfg
                .min_frame_retransmission_interval
                .unwrap_or_else(|| self.cfg.expected_packet_latency / 2),
            self.cfg.max_frame_retransmission_services,
            self.cfg
                .frame_retransmission_refill_interval
                .unwrap_or(self.cfg.expected_packet_latency),
            self.cfg.max_frame_retransmission_burst,
        )));

        let context = self.context.insert(AcknowledgementStateContext {
            rb_tx,
            rb_rx,
            incoming_frame_retries_tx,
            outgoing_frame_retries_tx,
            ack_tx,
            ctl_tx: socket_components.ctl_tx,
            inspector: socket_components
                .inspector
                .ok_or(SessionError::InvalidState("inspector is not available".into()))?,
            retransmission_budget,
        });

        if self.cfg.mode.is_partial_ack_enabled() {
            // For partially received frames incomplete for too long,
            // missing segments will be asked for retransmission
            let mut incoming_frame_retries_tx_clone = context.incoming_frame_retries_tx.clone();
            let ctl_tx_clone = context.ctl_tx.clone();
            let frame_inspector_clone = context.inspector.clone();
            let cfg = self.cfg;
            let (started_1, started_2) = (self.started.clone(), self.started.clone());
            hopr_utils::runtime::prelude::spawn(incoming_frame_retries_rx
                .filter_map(move |rf| {
                    let frame_id = rf.frame_id;
                    let missing_segments = frame_inspector_clone.missing_segments(&frame_id).unwrap_or_default();
                    if !missing_segments.is_empty() {
                        // Find out if we need to subscribe for further retries of this Frame
                        if let Some(next) = rf.next() {
                            // Register the next retry if still possible
                            let retry_at = next_deadline_with_backoff(next.retry_count, cfg.backoff_base, cfg.expected_packet_latency);
                            if let Err(error) = incoming_frame_retries_tx_clone.send_one((next, retry_at)) {
                                if is_running(&started_1) {
                                    tracing::error!(frame_id, %error, "failed to register next resend of incoming frame");
                                } else {
                                    tracing::debug!(frame_id, %error, "state stopped, next resend of incoming frame not registered");
                                }
                            } else {
                                tracing::debug!(frame_id, retry_in = ?retry_at.saturating_duration_since(Instant::now()), "next resend request of incoming frame segments");
                            }
                        } else {
                            tracing::debug!(frame_id, "last request of incoming frame segments");
                        }

                        futures::future::ready(Some((frame_id, missing_segments)))
                    } else {
                        tracing::debug!(frame_id, "no more missing segments in frame");
                        futures::future::ready(None)
                    }
                })
                .ready_chunks(SegmentRequest::<C>::MAX_ENTRIES)
                .inspect(|r| tracing::trace!(req = ?r, "requesting segments resend"))
                .map(|a| Ok(SessionMessage::<C>::Request(a.into_iter().collect())))
                .forward(ctl_tx_clone)
                .map(move |res| match res {
                    Ok(_) => tracing::debug!("incoming frame resends processing done"),
                    Err(error) if is_running(&started_2) => tracing::error!(%error, "error while processing incoming frame resends"),
                    Err(error) => tracing::debug!(%error, "incoming frame resends ended after the state stopped")
                })
                .instrument(tracing::debug_span!("incoming_frame_retries_sender"))
            );
        }

        // Send out Frame Acknowledgements chunked as Control messages
        let ctl_tx_clone = context.ctl_tx.clone();
        let ack_delay = self.cfg.acknowledgement_delay;
        hopr_utils::runtime::prelude::spawn(
            ack_rx
                .buffer(futures_time::time::Duration::from(ack_delay))
                .flat_map(|acks| futures::stream::iter(FrameAcknowledgements::<C>::new_multiple(acks)))
                .filter(|acks| futures::future::ready(!acks.is_empty()))
                .inspect(|acks| tracing::trace!(?acks, "acknowledgements sent"))
                .map(|acks| Ok(SessionMessage::<C>::Acknowledge(acks)))
                .forward(ctl_tx_clone)
                .map(move |res| match res {
                    Ok(_) => tracing::debug!("acknowledgement forwarding done"),
                    Err(error) => tracing::debug!(%error, "acknowledgement forwarding failed"),
                })
                .instrument(tracing::debug_span!("acknowledgement_sender")),
        );

        // Resend outgoing frame Segments if they were not (partially or fully) acknowledged
        let mut outgoing_frame_retries_tx_clone = context.outgoing_frame_retries_tx.clone();
        let ctl_tx_clone = context.ctl_tx.clone();
        let rb_rx_clone = context.rb_rx.clone();
        let cfg = self.cfg;
        let delivery_tap = self.delivery_tap.clone();
        let (started_1, started_2) = (self.started.clone(), self.started.clone());
        hopr_utils::runtime::prelude::spawn(
            outgoing_frame_retries_rx
                .filter_map(move |rf: RetriedFrameId| {
                    let frame_id = rf.frame_id;

                    // Anti-bufferbloat: data this stale is worthless to the peer, but the latency
                    // it would add on a burst drain is not. Retire it as lost instead — which also
                    // tells the honest clock the truth about the stall.
                    if cfg.max_frame_age.is_some_and(|max_age| rf.age() >= max_age) {
                        tracing::trace!(frame_id, age = ?rf.age(), "outgoing frame exceeded max age; retiring as lost");
                        if let Some(tap) = &delivery_tap {
                            tap.on_lost_frame();
                        }
                        return futures::future::ready(None);
                    }

                    // Find out if the frame can be retried again in the future
                    if let Some(next) = rf.next() {
                        // Register the next retry if still possible
                        let retry_at =
                            next_deadline_with_backoff(next.retry_count, cfg.backoff_base, cfg.expected_packet_latency);
                        if let Err(error) = outgoing_frame_retries_tx_clone.send_one((next, retry_at)) {
                            if is_running(&started_1) {
                                tracing::error!(frame_id, %error, "failed to register next retry of frame");
                            } else {
                                tracing::debug!(frame_id, %error, "state stopped, next retry of frame not registered");
                            }
                        } else {
                            tracing::debug!(frame_id, retry_in = ?retry_at.saturating_duration_since(Instant::now()), "next resend of outgoing frame");
                        }
                    } else {
                        tracing::debug!(frame_id, "last outgoing retry of frame");
                        // Honest-clock tap: retransmissions exhausted → treat the frame as lost.
                        // An earlier ack cancels this retry (Skip), so a delivered frame never
                        // reaches here; a late ack after give-up is harmless (bounded over-credit).
                        if let Some(tap) = &delivery_tap {
                            tap.on_lost_frame();
                        }
                    }
                    tracing::trace!(frame_id, "going to re-send entire frame");
                    futures::future::ready(Some(frame_id))
                })
                .flat_map(move |frame_id| {
                    // Find out all the segments of that frame to be retransmitted
                    futures::stream::iter(
                        rb_rx_clone
                            .find(|s: &Segment| s.id().0 == frame_id)
                            .into_iter()
                            .inspect(|s| tracing::trace!(seg_id = %s.id(), "segment retransmit"))
                            .map(|s| Ok(SessionMessage::<C>::Segment(s))),
                    )
                })
                .forward(ctl_tx_clone) // Retransmit all the segments
                .map(move |res| match res {
                    Ok(_) => tracing::debug!("outgoing frame retries processing done"),
                    Err(error) if is_running(&started_2) => {
                        tracing::error!(%error, "error while processing outgoing frame retries")
                    }
                    Err(error) => tracing::debug!(%error, "outgoing frame retries ended after the state stopped"),
                })
                .instrument(tracing::debug_span!("outgoing_frame_retries_sender")),
        );

        tracing::debug!("acknowledgement state has been started");
        self.started.store(true, std::sync::atomic::Ordering::Relaxed);

        Ok(())
    }

    #[tracing::instrument(name = "AcknowledgementState::stop", skip(self), fields(session_id = self.id))]
    fn stop(&mut self) -> Result<(), SessionError> {
        if let Some(mut ctx) = self.context.take() {
            // Mark the state as stopped before closing the channels, so the background tasks
            // that fail on the closed channels know this is a normal stop and not an error.
            self.started.store(false, std::sync::atomic::Ordering::Release);
            ctx.outgoing_frame_retries_tx.force_close();
            ctx.incoming_frame_retries_tx.force_close();
            // ack_tx and ctl_tx close when their last clone is dropped;
            // dropping ctx here drops the originals, spawned-task clones drop when those tasks complete.

            tracing::debug!("state has been stopped");
        } else {
            // A socket is stopped both when it is closed and when it is dropped,
            // so a second stop is expected.
            tracing::debug!("state already stopped");
        }

        Ok(())
    }

    #[tracing::instrument(name = "AcknowledgementState::incoming_segment", skip(self), fields(session_id = self.id, frame_id = seg_id.0))]
    fn incoming_segment(&mut self, seg_id: &SegmentId, _ind: SeqIndicator) -> Result<(), SessionError> {
        tracing::trace!("segment received");

        let ctx = self
            .started
            .load(std::sync::atomic::Ordering::Relaxed)
            .then_some(self.context.as_mut())
            .flatten()
            .ok_or(SessionError::StateNotRunning)?;

        // Register future requesting of segments for this frame
        if self.cfg.mode.is_partial_ack_enabled() {
            // Every incoming segment of this frame will move the deadline further
            // into the future.
            if let Err(error) = ctx.incoming_frame_retries_tx.send_one((
                RetriedFrameId::with_retries(seg_id.0, self.cfg.max_incoming_frame_retries),
                self.cfg.expected_packet_latency, // RTO_BASE_RECEIVER - when we expect the next segment to arrive
            )) {
                tracing::error!(%error, "failed to register incoming retry for frame");
            }
        }
        Ok(())
    }

    #[tracing::instrument(name = "AcknowledgementState::incoming_retransmission_request", skip(self, request), fields(session_id = self.id))]
    fn incoming_retransmission_request(&mut self, request: SegmentRequest<C>) -> Result<(), SessionError> {
        // The state will respond to segment retransmission requests even
        // if it has this feature disabled in the config.
        tracing::trace!(count = request.len(), "segment retransmission requested");

        let ctx = self
            .started
            .load(std::sync::atomic::Ordering::Relaxed)
            .then_some(self.context.as_mut())
            .flatten()
            .ok_or(SessionError::StateNotRunning)?;

        let now = Instant::now();

        // Admission pass: decide per frame *before* the request is expanded into one SegmentId
        // per requested bit. The mutex guard is confined to this block, so it is never held
        // across the ring-buffer scan below.
        let admitted_frame_ids: Vec<FrameId> = {
            let mut budget = ctx.retransmission_budget.lock();
            request
                .frames()
                .filter(|frame_id| budget.admit(*frame_id, now))
                .collect()
        };

        if admitted_frame_ids.is_empty() {
            // Fully throttled: no scan, no egress, no cancellation of our own retry schedule.
            let throttled = ctx.retransmission_budget.lock().take_throttled();
            if throttled > 0 {
                tracing::debug!(throttled, "retransmission request entirely throttled");
            }
            return Ok(());
        }

        // Keep only the segments of the admitted frames; the ids are unique and
        // sorted, so a set lookup replaces the previous shift-on-every-hit removal.
        let mut requested_seg_ids: HashSet<SegmentId> = request
            .into_iter()
            .filter(|seg_id| admitted_frame_ids.binary_search(&seg_id.0).is_ok())
            .collect();

        // Perform a single find to lock the RB only once
        let segments = ctx.rb_rx.find(|s| requested_seg_ids.remove(&s.id()));

        tracing::trace!(
            found = segments.len(),
            requested = admitted_frame_ids.len(),
            "found matching segments to be retransmitted"
        );

        // Charge the per-frame budget only for frames that were actually served, so bogus
        // frame ids in a request cannot grow the budget's bookkeeping. One service per
        // request and frame, no matter how many of the frame's segments were found.
        // The set must stay alive: on a fully successful queue below, it also drives the
        // resend cancellation.
        // NOTE: the budget mutex guard is dropped at the end of this block and is never
        // held across the control-stream sends below.
        let served_frame_ids: Vec<FrameId> = {
            let mut budget = ctx.retransmission_budget.lock();
            let mut served_frame_ids: Vec<FrameId> = segments.iter().map(|s| s.frame_id).collect();
            served_frame_ids.sort_unstable();
            served_frame_ids.dedup();
            served_frame_ids
                .iter()
                .for_each(|frame_id| budget.record(*frame_id, now));
            // Count-batched: the handler is already a tracing span per packet.
            let throttled = budget.take_throttled();
            if throttled > 0 {
                tracing::debug!(throttled, "throttled peer-driven retransmission requests");
            }
            served_frame_ids
        };

        // Resend the segments via the Control Stream. This MUST happen before the
        // full-frame resend cancellation below: `ctl_tx` is bounded, and `try_send`
        // fails once it is full. Only if every found segment was queued successfully
        // can the frames in `served_frame_ids` be considered actually served. Cancelling
        // earlier (on merely *found* frames) would remove a frame from our own retry
        // pipeline without ever sending it, so it would be neither NACK-served nor
        // full-frame-retransmitted — a permanent gap in reliable mode, with the eventual
        // `DeliveryTap::on_lost_frame()` report suppressed as well.
        segments
            .into_iter()
            .try_for_each(|s| {
                tracing::trace!(seg_id = %s.id(), "retransmit segment on request");
                ctx.ctl_tx.try_send(SessionMessage::Segment(s))
            })
            .map_err(|e| SessionError::ProcessingError(e.to_string()))?;

        // Partially acknowledged frames will not need to be fully resent in the future.
        // Cancel the resend of the SERVED frames only, and only after all their segments
        // have been queued successfully above: a frame can be admitted by the budget
        // without being served (e.g. the request names a segment that does not exist in
        // the ring buffer), or its segments may fail to queue if the control stream is
        // full (in which case the error above aborts this handler before we get here).
        // Cancelling a frame we did not serve would let a peer suppress our own
        // retransmission while getting nothing back, and would suppress the eventual
        // `DeliveryTap::on_lost_frame()` report.
        if self.cfg.mode.is_full_ack_enabled()
            && !served_frame_ids.is_empty()
            && let Err(error) = ctx.outgoing_frame_retries_tx.send_many(
                served_frame_ids
                    .into_iter()
                    .map(|frame_id| (RetriedFrameId::no_retries(frame_id), Skip).into()),
            )
        {
            tracing::error!(%error, "failed to cancel frame resend of partially acknowledged frames");
        }

        Ok(())
    }

    #[tracing::instrument(name = "AcknowledgementState::incoming_acknowledged_frames", skip(self), fields(session_id = self.id))]
    fn incoming_acknowledged_frames(&mut self, ack: FrameAcknowledgements<C>) -> Result<(), SessionError> {
        tracing::trace!(count = ack.len(), "frame acknowledgements received");

        // Cloned out before the mutable `ctx` borrow so the honest-clock tap can run inside the
        // acknowledgement iteration without conflicting with the retry-cancel borrow.
        let tap = self.delivery_tap.clone();
        let full_ack = self.cfg.mode.is_full_ack_enabled();

        let ctx = self
            .started
            .load(std::sync::atomic::Ordering::Relaxed)
            .then_some(self.context.as_mut())
            .flatten()
            .ok_or(SessionError::StateNotRunning)?;

        // Honest-clock tap: every received frame acknowledgement is proof of delivery, and must feed
        // the flow-control window regardless of the ack mode. `full_ack` only governs whether we
        // *also* cancel the outgoing full-frame retransmission below (in `Partial` mode the receiver
        // drives retransmission, so there is nothing to cancel here — but the acks still arrive).
        if let Some(tap) = &tap {
            (0..ack.len()).for_each(|_| tap.on_acked_frame());
        }

        // Frame acknowledged, we will not need to resend it (full-ack mode only).
        if full_ack
            && let Err(error) = ctx.outgoing_frame_retries_tx.send_many(
                ack.into_iter()
                    .inspect(|frame_id| tracing::trace!(frame_id, "frame acknowledged"))
                    .map(|frame_id| (RetriedFrameId::no_retries(frame_id), Skip).into()),
            )
        {
            tracing::error!(%error, "failed to cancel frame resend");
        }

        Ok(())
    }

    #[tracing::instrument(name = "AcknowledgementState::frame_complete", skip(self), fields(session_id = self.id))]
    fn frame_complete(&mut self, frame_id: FrameId) -> Result<(), SessionError> {
        tracing::trace!("frame complete");

        let ctx = self
            .started
            .load(std::sync::atomic::Ordering::Relaxed)
            .then_some(self.context.as_mut())
            .flatten()
            .ok_or(SessionError::StateNotRunning)?;

        // Since the frame has been completed, push its ID into the acknowledgement queue
        if let Err(error) = ctx.ack_tx.try_send(frame_id) {
            tracing::error!(%error, "failed to acknowledge frame");
        }

        if self.cfg.mode.is_partial_ack_enabled() {
            // No more requesting of segment retransmissions from frames that were completed
            if let Err(error) = ctx
                .incoming_frame_retries_tx
                .send_one((RetriedFrameId::no_retries(frame_id), Skip))
            {
                tracing::error!(%error, "failed to cancel retry of acknowledged frame");
            }
        }

        Ok(())
    }

    #[tracing::instrument(name = "AcknowledgementState::frame_emitted", skip(self), fields(session_id = self.id))]
    fn frame_emitted(&mut self, id: FrameId) -> Result<(), SessionError> {
        tracing::trace!("frame emitted");
        let _ = self
            .started
            .load(std::sync::atomic::Ordering::Relaxed)
            .then_some(self.context.as_mut())
            .flatten()
            .ok_or(SessionError::StateNotRunning)?;
        Ok(())
    }

    #[tracing::instrument(name = "AcknowledgementState::frame_discarded", skip(self), fields(session_id = self.id))]
    fn frame_discarded(&mut self, frame_id: FrameId) -> Result<(), SessionError> {
        tracing::trace!("frame discarded");

        let ctx = self
            .started
            .load(std::sync::atomic::Ordering::Relaxed)
            .then_some(self.context.as_mut())
            .flatten()
            .ok_or(SessionError::StateNotRunning)?;

        if self.cfg.mode.is_partial_ack_enabled() {
            // No more requesting of segment retransmissions from frames that were discarded
            if let Err(error) = ctx
                .incoming_frame_retries_tx
                .send_one((RetriedFrameId::no_retries(frame_id), Skip))
            {
                tracing::error!(%error, "failed to cancel retry of acknowledged frame");
            }
        }

        Ok(())
    }

    #[tracing::instrument(name = "AcknowledgementState::segment_sent", skip(self, segment), fields(session_id = self.id, frame_id = segment.frame_id, seq_idx = segment.seq_idx))]
    fn segment_sent(&mut self, segment: &Segment) -> Result<(), SessionError> {
        tracing::trace!("segment sent");

        let ctx = self
            .started
            .load(std::sync::atomic::Ordering::Relaxed)
            .then_some(self.context.as_mut())
            .flatten()
            .ok_or(SessionError::StateNotRunning)?;

        // Since segments are re-sent via Control stream, they are not later fed again
        // into the ring buffer.
        if !ctx.rb_tx.push(segment.clone()) {
            tracing::trace!(
                frame_id = segment.frame_id,
                seq_idx = segment.seq_idx,
                "ring buffer full: new segment enqueued, oldest buffered segment overwritten"
            );
        }

        // When the last segment of a frame has been sent,
        // add it to outgoing retries (if the full ack mode is enabled).
        if segment.is_last() && self.cfg.mode.is_full_ack_enabled() {
            tracing::trace!("last segment of frame sent");

            if let Err(error) = ctx.outgoing_frame_retries_tx.send_one((
                RetriedFrameId::with_retries(segment.frame_id, self.cfg.max_outgoing_frame_retries),
                // The whole frame should be delivered and acknowledged
                // once all its segments (seq_len) are sent,
                // and the acknowledgement also comes back to us.
                // Therefore, RTO_BASE_SENDER = latency * (seq_len + 1)
                self.cfg.expected_packet_latency * (segment.seq_flags.seq_len() + 1) as u32,
            )) {
                tracing::error!(%error, "failed to insert outgoing retry of a frame");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;

    use super::*;
    use crate::{
        MAX_SESSION_MTU,
        processing::types::{FrameBuilder, FrameDashMap, FrameMap},
        protocol::SeqNum,
        utils::segment,
    };

    /// The frame size production actually feeds a Session.
    const FRAME_SIZE: usize = MAX_SESSION_MTU;

    const MTU: usize = 1000;

    #[test_log::test(tokio::test)]
    async fn ack_state_sender_must_acknowledge_completed_frames() -> anyhow::Result<()> {
        let cfg = AcknowledgementStateConfig {
            expected_packet_latency: Duration::from_millis(10),
            acknowledgement_delay: Duration::from_millis(2),
            ..Default::default()
        };

        let inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = bounded_sink_channel::<SessionMessage<MTU>>(1024);

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        let acked_frame_ids = [1, 2, 3];

        for &frame_id in &acked_frame_ids {
            state.frame_complete(frame_id)?;
        }

        tokio::time::sleep(cfg.acknowledgement_delay * 2).await;

        state.stop()?;

        let ctl_msgs = tokio::time::timeout(Duration::from_millis(100), ctl_rx.collect::<Vec<_>>())
            .await
            .context("timeout receiving Control messages")?;

        assert_eq!(1, ctl_msgs.len());

        assert_eq!(
            ctl_msgs[0],
            SessionMessage::Acknowledge(acked_frame_ids.to_vec().try_into()?)
        );

        Ok(())
    }

    #[parameterized::parameterized(num_frames = { 1, 2, 3 })]
    #[parameterized_macro(test_log::test(tokio::test))]
    async fn ack_state_sender_must_resend_unacknowledged_frames(num_frames: usize) -> anyhow::Result<()> {
        const NUM_RETRIES: usize = 2;

        let cfg = AcknowledgementStateConfig {
            mode: AcknowledgementMode::Full,
            expected_packet_latency: Duration::from_millis(2),
            max_outgoing_frame_retries: NUM_RETRIES,
            ..Default::default()
        };

        let inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = bounded_sink_channel::<SessionMessage<MTU>>(1024);

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        let mut expected_frame_segments = Vec::new();
        let num_segments_in_frame = FRAME_SIZE / MTU + 1;
        for i in 1..=num_frames {
            let expected_segments = segment(
                hopr_types::crypto_random::random_bytes::<FRAME_SIZE>(),
                MTU,
                i as FrameId,
            )?;
            for segment in &expected_segments {
                state.segment_sent(segment)?;
            }
            expected_frame_segments.push(expected_segments);
        }

        let expected_frame_delivery = cfg.expected_packet_latency * (num_segments_in_frame + 1) as u32;
        tokio::time::sleep(2 * expected_frame_delivery).await;
        state.stop()?;

        let ctl_msg = tokio::time::timeout(Duration::from_millis(100), ctl_rx.collect::<Vec<_>>())
            .await
            .context("timeout receiving Control message")?;

        let retransmitted_segments = ctl_msg
            .into_iter()
            .map(|m| m.try_as_segment().ok_or(anyhow::anyhow!("must be segment")))
            .collect::<Result<Vec<_>, _>>()?;

        assert_eq!(
            NUM_RETRIES * num_segments_in_frame * num_frames,
            retransmitted_segments.len()
        );

        let total_segments = expected_frame_segments.iter().map(|m| m.len()).sum::<usize>();
        let expected_segments = expected_frame_segments
            .into_iter()
            .flatten()
            .cycle()
            .take(total_segments * NUM_RETRIES)
            .collect::<Vec<_>>();
        assert_eq!(expected_segments, retransmitted_segments);

        Ok(())
    }

    /// Runs one unacknowledged frame through the retry pipeline and returns how many segments were
    /// retransmitted before `run_for` elapsed.
    async fn retransmissions_with_max_frame_age(max_frame_age: Option<Duration>) -> anyhow::Result<usize> {
        // A generous retry budget, so the only thing that can cut the retries short is the age bound.
        let cfg = AcknowledgementStateConfig {
            mode: AcknowledgementMode::Full,
            expected_packet_latency: Duration::from_millis(5),
            max_outgoing_frame_retries: 20,
            max_frame_age,
            ..Default::default()
        };

        let inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = bounded_sink_channel::<SessionMessage<MTU>>(1024);

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        for segment in &segment(hopr_types::crypto_random::random_bytes::<FRAME_SIZE>(), MTU, 1)? {
            state.segment_sent(segment)?;
        }

        // Never acknowledge: the frame stays in the retry pipeline for the whole window.
        tokio::time::sleep(Duration::from_millis(500)).await;
        state.stop()?;

        let ctl_msgs = tokio::time::timeout(Duration::from_millis(100), ctl_rx.collect::<Vec<_>>())
            .await
            .context("timeout receiving Control messages")?;

        Ok(ctl_msgs.into_iter().filter_map(|m| m.try_as_segment()).count())
    }

    #[test_log::test(tokio::test)]
    async fn ack_state_sender_should_stop_resending_a_frame_that_exceeded_the_max_age() -> anyhow::Result<()> {
        let unbounded = retransmissions_with_max_frame_age(None).await?;
        let bounded = retransmissions_with_max_frame_age(Some(Duration::from_millis(50))).await?;

        assert!(
            unbounded > 0,
            "the retry pipeline must retransmit at all without an age bound"
        );
        assert!(
            bounded < unbounded,
            "an age bound must cut retransmissions short: got {bounded} bounded vs {unbounded} unbounded"
        );

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn ack_state_sender_must_not_resend_unacknowledged_frame_when_full_resend_disabled() -> anyhow::Result<()> {
        const NUM_RETRIES: usize = 2;

        let cfg = AcknowledgementStateConfig {
            mode: AcknowledgementMode::Partial,
            expected_packet_latency: Duration::from_millis(2),
            max_outgoing_frame_retries: NUM_RETRIES,
            ..Default::default()
        };

        let inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = bounded_sink_channel::<SessionMessage<MTU>>(1024);

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        let expected_segments = segment(hopr_types::crypto_random::random_bytes::<FRAME_SIZE>(), MTU, 1)?;
        for segment in &expected_segments {
            state.segment_sent(segment)?;
        }

        let expected_frame_delivery = cfg.expected_packet_latency * (expected_segments.len() + 1) as u32;
        tokio::time::sleep(2 * expected_frame_delivery).await;
        state.stop()?;

        // No retransmission should be sent because it is disabled
        assert!(ctl_rx.collect::<Vec<_>>().await.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn ack_state_sender_must_not_resend_acknowledged_frame() -> anyhow::Result<()> {
        let cfg = AcknowledgementStateConfig {
            mode: AcknowledgementMode::Full,
            expected_packet_latency: Duration::from_millis(2),
            max_outgoing_frame_retries: 1,
            ..Default::default()
        };

        let inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = bounded_sink_channel::<SessionMessage<MTU>>(1024);

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        let expected_segments = segment(hopr_types::crypto_random::random_bytes::<{ FRAME_SIZE * 2 }>(), MTU, 1)?;
        for segment in &expected_segments {
            state.segment_sent(segment)?;
        }

        // Acknowledge the frame
        state.incoming_acknowledged_frames(vec![1].try_into()?)?;

        tokio::time::sleep(10 * cfg.expected_packet_latency).await;

        state.stop()?;

        // No retransmission should be sent because the frame was already acknowledged.
        assert!(ctl_rx.collect::<Vec<_>>().await.is_empty());

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn ack_state_sender_must_not_resend_entire_frame_when_already_partially_acknowledged() -> anyhow::Result<()> {
        let cfg = AcknowledgementStateConfig {
            mode: AcknowledgementMode::Full,
            expected_packet_latency: Duration::from_millis(2),
            ..Default::default()
        };

        let inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = bounded_sink_channel::<SessionMessage<MTU>>(1024);

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        let expected_segments = segment(hopr_types::crypto_random::random_bytes::<{ FRAME_SIZE * 2 }>(), MTU, 1)?;
        let seq_len = expected_segments.len();

        // Load segments into the ring buffer
        for segment in &expected_segments {
            state.segment_sent(segment)?;
        }

        // Partially acknowledge the frame (report the first segment as missing)
        state.incoming_retransmission_request(SegmentRequest::from_iter([(1, [0b10000000].into())]))?;

        // Wait past the sender-side full-frame RTO (`expected_packet_latency * (seq_len + 1)`), so
        // a full-frame resend would have had every opportunity to fire. Asserting only that the
        // requested segment is retransmitted would pass without ever checking the property in the
        // test's name; waiting is what makes "must not resend the ENTIRE frame" observable. It also
        // removes a race: a short sleep could land the full-frame resend first, making the count
        // nondeterministic under CPU contention.
        tokio::time::sleep(cfg.expected_packet_latency * (seq_len as u32 + 2)).await;

        state.stop()?;

        // Only segment 1 must be retransmitted — never the whole frame.
        let ctl_msgs = ctl_rx.collect::<Vec<_>>().await;
        assert_eq!(1, ctl_msgs.len());
        assert_eq!(ctl_msgs[0], SessionMessage::Segment(expected_segments[0].clone()));

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn ack_state_sender_must_retransmit_segments_when_requested() -> anyhow::Result<()> {
        let cfg = AcknowledgementStateConfig {
            mode: AcknowledgementMode::Full,
            expected_packet_latency: Duration::from_millis(2),
            max_outgoing_frame_retries: 1,
            // This test exercises the request -> resend mapping, not the retransmission budget.
            // The per-frame service cap must stay out of the way, but the minimum interval
            // between two services of the same frame CANNOT be disabled: `normalize` re-derives
            // a `ZERO` interval into the safe default, so it is deliberately left at the default
            // here. That default is `expected_packet_latency / 2` = 1 ms, and the three requests
            // for frame 2 below are spaced 2 ms apart, so they all clear the interval.
            max_frame_retransmission_services: usize::MAX,
            ..Default::default()
        };

        let inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = bounded_sink_channel::<SessionMessage<MTU>>(1024);

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        let expected_segments_1 = segment(hopr_types::crypto_random::random_bytes::<{ FRAME_SIZE * 2 }>(), MTU, 1)?;
        // Load frame 1 segments into the ring buffer
        for segment in &expected_segments_1 {
            state.segment_sent(segment)?;
        }

        let expected_segments_2 = segment(hopr_types::crypto_random::random_bytes::<{ FRAME_SIZE * 2 }>(), MTU, 2)?;
        // Load frame 2 segments into the ring buffer
        for segment in &expected_segments_2 {
            state.segment_sent(segment)?;
        }

        tokio::time::sleep(cfg.expected_packet_latency).await;

        // Request different segments to be retransmitted
        state.incoming_retransmission_request(SegmentRequest::from_iter([
            (1, [0b11100000].into()),
            (2, [0b11100000].into()),
        ]))?;
        tokio::time::sleep(cfg.expected_packet_latency).await;

        state.incoming_retransmission_request(SegmentRequest::from_iter([(2, [0b11000000].into())]))?;
        tokio::time::sleep(cfg.expected_packet_latency).await;

        state.incoming_retransmission_request(SegmentRequest::from_iter([(2, [0b01000000].into())]))?;
        tokio::time::sleep(cfg.expected_packet_latency).await;

        state.stop()?;

        let ctl_msgs = ctl_rx.collect::<Vec<_>>().await;

        assert_eq!(9, ctl_msgs.len());
        // Request 1 - frame 1
        assert_eq!(ctl_msgs[0], SessionMessage::Segment(expected_segments_1[0].clone()));
        assert_eq!(ctl_msgs[1], SessionMessage::Segment(expected_segments_1[1].clone()));
        assert_eq!(ctl_msgs[2], SessionMessage::Segment(expected_segments_1[2].clone()));
        // Request 1 - frame 2
        assert_eq!(ctl_msgs[3], SessionMessage::Segment(expected_segments_2[0].clone()));
        assert_eq!(ctl_msgs[4], SessionMessage::Segment(expected_segments_2[1].clone()));
        assert_eq!(ctl_msgs[5], SessionMessage::Segment(expected_segments_2[2].clone()));

        // Request 2 - frame 2
        assert_eq!(ctl_msgs[6], SessionMessage::Segment(expected_segments_2[0].clone()));
        assert_eq!(ctl_msgs[7], SessionMessage::Segment(expected_segments_2[1].clone()));

        // Request 3 - frame 2
        assert_eq!(ctl_msgs[8], SessionMessage::Segment(expected_segments_2[1].clone()));

        Ok(())
    }

    #[tokio::test]
    async fn ack_state_receiver_must_request_missing_frames_when_partial_acks_are_enabled() -> anyhow::Result<()> {
        let cfg = AcknowledgementStateConfig {
            mode: AcknowledgementMode::Partial,
            expected_packet_latency: Duration::from_millis(2),
            max_incoming_frame_retries: 1,
            ..Default::default()
        };

        let mut inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = bounded_sink_channel::<SessionMessage<MTU>>(1024);

        let segments = segment(hopr_types::crypto_random::random_bytes::<FRAME_SIZE>(), MTU, 1)?;

        inspector
            .0
            .entry(1)
            .try_as_vacant()
            .ok_or(anyhow::anyhow!("frame 1 must be vacant"))?
            .insert(FrameBuilder::from(segments[0].clone()));

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        state.incoming_segment(&segments[0].id(), (segments.len() as SeqNum).try_into()?)?;

        tokio::time::sleep(cfg.expected_packet_latency * 2).await;

        state.stop()?;

        let ctl_msgs = tokio::time::timeout(Duration::from_millis(100), ctl_rx.collect::<Vec<_>>())
            .await
            .context("timeout receiving Control messages")?;

        assert_eq!(1, ctl_msgs.len());
        assert_eq!(
            ctl_msgs[0],
            SessionMessage::Request(SegmentRequest::from_iter([(1, [0b01000000].into())]))
        );

        Ok(())
    }

    #[tokio::test]
    async fn ack_state_receiver_must_not_request_missing_frames_when_partial_acks_are_disabled() -> anyhow::Result<()> {
        let cfg = AcknowledgementStateConfig {
            mode: AcknowledgementMode::Full,
            expected_packet_latency: Duration::from_millis(2),
            max_incoming_frame_retries: 1,
            ..Default::default()
        };

        let mut inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = bounded_sink_channel::<SessionMessage<MTU>>(1024);

        let segments = segment(hopr_types::crypto_random::random_bytes::<FRAME_SIZE>(), MTU, 1)?;

        inspector
            .0
            .entry(1)
            .try_as_vacant()
            .ok_or(anyhow::anyhow!("frame 1 must be vacant"))?
            .insert(FrameBuilder::from(segments[0].clone()));

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        state.incoming_segment(&segments[0].id(), (segments.len() as SeqNum).try_into()?)?;

        tokio::time::sleep(cfg.expected_packet_latency * 2).await;

        state.stop()?;

        let ctl_msgs = tokio::time::timeout(Duration::from_millis(100), ctl_rx.collect::<Vec<_>>())
            .await
            .context("timeout receiving Control messages")?;

        assert!(ctl_msgs.iter().all(|m| !matches!(m, SessionMessage::Request(_))));

        Ok(())
    }

    #[tokio::test]
    async fn ack_state_receiver_must_continue_requesting_missing_frames_when_frame_not_completed() -> anyhow::Result<()>
    {
        let cfg = AcknowledgementStateConfig {
            mode: AcknowledgementMode::Partial,
            expected_packet_latency: Duration::from_millis(2),
            max_incoming_frame_retries: 3,
            ..Default::default()
        };

        let mut inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = bounded_sink_channel::<SessionMessage<MTU>>(1024);

        let segments = segment(hopr_types::crypto_random::random_bytes::<{ 2 * FRAME_SIZE }>(), MTU, 1)?;

        inspector
            .0
            .entry(1)
            .try_as_vacant()
            .ok_or(anyhow::anyhow!("frame 1 must be vacant"))?
            .insert(FrameBuilder::from(segments[0].clone()));

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.clone().into(),
            ctl_tx,
        })?;

        state.incoming_segment(&segments[0].id(), (segments.len() as SeqNum).try_into()?)?;

        tokio::time::sleep(cfg.expected_packet_latency * 2).await;

        inspector
            .0
            .entry(1)
            .try_as_occupied()
            .ok_or(anyhow::anyhow!("frame 1 must be occupied"))?
            .get_mut()
            .add_segment(segments[1].clone())?;

        state.incoming_segment(&segments[1].id(), (segments.len() as SeqNum).try_into()?)?;

        tokio::time::sleep(cfg.expected_packet_latency * 2).await;

        state.stop()?;

        let ctl_msgs = tokio::time::timeout(Duration::from_millis(100), ctl_rx.collect::<Vec<_>>())
            .await
            .context("timeout receiving Control messages")?;

        assert_eq!(2, ctl_msgs.len());
        assert_eq!(
            ctl_msgs[0],
            SessionMessage::Request(SegmentRequest::from_iter([(1, [0b01100000].into())]))
        );

        assert_eq!(
            ctl_msgs[1],
            SessionMessage::Request(SegmentRequest::from_iter([(1, [0b00100000].into())]))
        );

        Ok(())
    }

    #[tokio::test]
    async fn ack_state_receiver_must_continue_requesting_missing_frames_and_acknowledge_once_complete()
    -> anyhow::Result<()> {
        let cfg = AcknowledgementStateConfig {
            mode: AcknowledgementMode::Partial,
            expected_packet_latency: Duration::from_millis(2),
            max_incoming_frame_retries: 3,
            acknowledgement_delay: Duration::from_millis(5),
            ..Default::default()
        };

        let mut inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = bounded_sink_channel::<SessionMessage<MTU>>(1024);

        let segments = segment(hopr_types::crypto_random::random_bytes::<{ 2 * FRAME_SIZE }>(), MTU, 1)?;

        // Segment 1
        inspector
            .0
            .entry(1)
            .try_as_vacant()
            .ok_or(anyhow::anyhow!("frame 1 must be vacant"))?
            .insert(FrameBuilder::from(segments[0].clone()));

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.clone().into(),
            ctl_tx,
        })?;

        // Segment 2
        state.incoming_segment(&segments[0].id(), (segments.len() as SeqNum).try_into()?)?;

        tokio::time::sleep(cfg.expected_packet_latency * 2).await;

        inspector
            .0
            .entry(1)
            .try_as_occupied()
            .ok_or(anyhow::anyhow!("frame 1 must be occupied"))?
            .get_mut()
            .add_segment(segments[1].clone())?;

        state.incoming_segment(&segments[1].id(), (segments.len() as SeqNum).try_into()?)?;

        tokio::time::sleep(cfg.expected_packet_latency * 2).await;

        // Segment 3
        inspector
            .0
            .entry(1)
            .try_as_occupied()
            .ok_or(anyhow::anyhow!("frame 1 must be occupied"))?
            .get_mut()
            .add_segment(segments[2].clone())?;

        state.incoming_segment(&segments[2].id(), (segments.len() as SeqNum).try_into()?)?;
        state.frame_complete(1)?;

        tokio::time::sleep(cfg.acknowledgement_delay * 2).await;

        state.stop()?;

        let ctl_msgs = tokio::time::timeout(Duration::from_millis(100), ctl_rx.collect::<Vec<_>>())
            .await
            .context("timeout receiving Control messages")?;

        assert_eq!(3, ctl_msgs.len());
        assert_eq!(
            ctl_msgs[0],
            SessionMessage::Request(SegmentRequest::from_iter([(1, [0b01100000].into())]))
        );

        assert_eq!(
            ctl_msgs[1],
            SessionMessage::Request(SegmentRequest::from_iter([(1, [0b00100000].into())]))
        );

        assert_eq!(ctl_msgs[2], SessionMessage::Acknowledge(vec![1].try_into()?));

        Ok(())
    }

    /// Builds a request for `frame_id` naming the given missing-segment bitmap.
    fn request_for(frame_id: FrameId, missing: u8) -> SegmentRequest<MTU> {
        SegmentRequest::from_iter([(frame_id, [missing].into())])
    }

    #[test_log::test(tokio::test)]
    async fn ack_state_receiver_must_not_service_a_replayed_request_twice_within_the_interval() -> anyhow::Result<()> {
        // Default budget: the same frame cannot be served again within half the RTO base.
        let cfg = AcknowledgementStateConfig {
            expected_packet_latency: Duration::from_millis(50),
            ..Default::default()
        };

        let inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = bounded_sink_channel::<SessionMessage<MTU>>(1024);

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        let expected_segments = segment(hopr_types::crypto_random::random_bytes::<{ 2 * MTU }>(), MTU, 1)?;
        for segment in &expected_segments {
            state.segment_sent(segment)?;
        }

        state.incoming_retransmission_request(request_for(1, 0b11000000))?;
        // Replay immediately: well within `expected_packet_latency / 2`.
        state.incoming_retransmission_request(request_for(1, 0b11000000))?;

        tokio::time::sleep(Duration::from_millis(100)).await;
        state.stop()?;

        let ctl_msgs = ctl_rx.collect::<Vec<_>>().await;
        assert_eq!(2, ctl_msgs.len(), "replayed request must not be served again");
        assert_eq!(SessionMessage::Segment(expected_segments[0].clone()), ctl_msgs[0]);
        assert_eq!(SessionMessage::Segment(expected_segments[1].clone()), ctl_msgs[1]);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn ack_state_receiver_must_service_the_same_frame_again_after_the_interval_elapses() -> anyhow::Result<()> {
        let cfg = AcknowledgementStateConfig {
            expected_packet_latency: Duration::from_millis(50),
            ..Default::default()
        };

        let inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = bounded_sink_channel::<SessionMessage<MTU>>(1024);

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        let expected_segments = segment(hopr_types::crypto_random::random_bytes::<{ 2 * MTU }>(), MTU, 1)?;
        for segment in &expected_segments {
            state.segment_sent(segment)?;
        }

        state.incoming_retransmission_request(request_for(1, 0b11000000))?;
        // The default interval is `expected_packet_latency / 2` = 25 ms; wait past it.
        tokio::time::sleep(cfg.expected_packet_latency).await;
        state.incoming_retransmission_request(request_for(1, 0b11000000))?;

        tokio::time::sleep(Duration::from_millis(100)).await;
        state.stop()?;

        let ctl_msgs = ctl_rx.collect::<Vec<_>>().await;
        assert_eq!(
            4,
            ctl_msgs.len(),
            "frame must be servable again once the interval elapses"
        );
        for (i, expected_segment) in expected_segments.iter().cycle().take(4).enumerate() {
            assert_eq!(SessionMessage::Segment(expected_segment.clone()), ctl_msgs[i]);
        }

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn ack_state_receiver_must_throttle_a_frame_past_the_service_cap() -> anyhow::Result<()> {
        let cfg = AcknowledgementStateConfig {
            // Serve the frame at most twice, regardless of how often it is requested.
            max_frame_retransmission_services: 2,
            // Freeze the session-wide allowance so only the per-frame cap is exercised.
            max_frame_retransmission_burst: usize::MAX,
            frame_retransmission_refill_interval: Some(Duration::from_secs(60)),
            min_frame_retransmission_interval: Some(Duration::from_millis(1)),
            ..Default::default()
        };

        let inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = bounded_sink_channel::<SessionMessage<MTU>>(1024);

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        let expected_segments = segment(hopr_types::crypto_random::random_bytes::<{ 2 * MTU }>(), MTU, 1)?;
        for segment in &expected_segments {
            state.segment_sent(segment)?;
        }

        // Exactly `max_frame_retransmission_services` services must pass...
        for _ in 0..2 {
            state.incoming_retransmission_request(request_for(1, 0b11000000))?;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        // ...and the next one must be refused.
        state.incoming_retransmission_request(request_for(1, 0b11000000))?;

        tokio::time::sleep(Duration::from_millis(100)).await;
        state.stop()?;

        let ctl_msgs = ctl_rx.collect::<Vec<_>>().await;
        assert_eq!(4, ctl_msgs.len(), "exactly two services must have been served");
        for (i, expected_segment) in expected_segments.iter().cycle().take(4).enumerate() {
            assert_eq!(SessionMessage::Segment(expected_segment.clone()), ctl_msgs[i]);
        }

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn ack_state_receiver_must_bound_total_service_while_frames_are_cycled() -> anyhow::Result<()> {
        let cfg = AcknowledgementStateConfig {
            // The session-wide allowance is the ceiling under test: only 3 frames may be served
            // before it refills.
            max_frame_retransmission_burst: 3,
            frame_retransmission_refill_interval: Some(Duration::from_secs(60)),
            max_frame_retransmission_services: usize::MAX,
            min_frame_retransmission_interval: Some(Duration::from_millis(1)),
            ..Default::default()
        };

        let inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = bounded_sink_channel::<SessionMessage<MTU>>(1024);

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        let expected_segments = (1..=2)
            .map(|i| segment(hopr_types::crypto_random::random_bytes::<{ 2 * MTU }>(), MTU, i))
            .collect::<Result<Vec<_>, _>>()?;
        for expected_segment in &expected_segments {
            for segment in expected_segment {
                state.segment_sent(segment)?;
            }
        }

        // Cycle the frames well beyond the allowance: only the first 3 services must pass.
        for i in 0..6 {
            state.incoming_retransmission_request(request_for(1 + (i % 2) as FrameId, 0b10000000))?;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
        state.stop()?;

        let ctl_msgs = ctl_rx.collect::<Vec<_>>().await;
        assert_eq!(
            3,
            ctl_msgs.len(),
            "the burst cap must bound the total number of services"
        );
        for (i, expected_frame) in expected_segments.iter().cycle().take(3).enumerate() {
            assert_eq!(SessionMessage::Segment(expected_frame[0].clone()), ctl_msgs[i]);
        }

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn ack_state_receiver_must_still_cancel_our_full_frame_resend_when_it_serves_a_frame() -> anyhow::Result<()> {
        let cfg = AcknowledgementStateConfig {
            mode: AcknowledgementMode::Full,
            expected_packet_latency: Duration::from_millis(100), // full-frame RTO = 3 * 100 ms
            ..Default::default()
        };

        let inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = bounded_sink_channel::<SessionMessage<MTU>>(1024);

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        let expected_segments = segment(hopr_types::crypto_random::random_bytes::<{ 2 * MTU }>(), MTU, 1)?;
        for segment in &expected_segments {
            state.segment_sent(segment)?;
        }

        // Serve the whole frame on request: this must also cancel the pending sender-side
        // full-frame resend of frame 1.
        state.incoming_retransmission_request(request_for(1, 0b11000000))?;

        // Wait well past the full-frame RTO.
        tokio::time::sleep(4 * cfg.expected_packet_latency).await;
        state.stop()?;

        let ctl_msgs = ctl_rx.collect::<Vec<_>>().await;
        assert_eq!(
            2,
            ctl_msgs.len(),
            "only the requested segments must be sent, no full-frame resend"
        );
        assert_eq!(SessionMessage::Segment(expected_segments[0].clone()), ctl_msgs[0]);
        assert_eq!(SessionMessage::Segment(expected_segments[1].clone()), ctl_msgs[1]);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn ack_state_receiver_must_not_cancel_our_full_frame_resend_when_it_throttles_a_frame() -> anyhow::Result<()>
    {
        let cfg = AcknowledgementStateConfig {
            mode: AcknowledgementMode::Full,
            expected_packet_latency: Duration::from_millis(100), // full-frame RTO = 3 * 100 ms
            // A single sender-side full-frame resend, so the assertion stays deterministic.
            max_outgoing_frame_retries: 1,
            // Only one frame may be served before the allowance refills; it is spent on a frame
            // that is not in the ring buffer, so the request for frame 1 below is refused.
            max_frame_retransmission_burst: 1,
            frame_retransmission_refill_interval: Some(Duration::from_secs(60)),
            max_frame_retransmission_services: usize::MAX,
            min_frame_retransmission_interval: Some(Duration::from_millis(1)),
            ..Default::default()
        };

        let inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = bounded_sink_channel::<SessionMessage<MTU>>(1024);

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        let expected_segments = segment(hopr_types::crypto_random::random_bytes::<{ 2 * MTU }>(), MTU, 1)?;
        for segment in &expected_segments {
            state.segment_sent(segment)?;
        }

        tokio::time::sleep(Duration::from_millis(10)).await;

        // Spend the allowance on a frame that is not in the ring buffer.
        state.incoming_retransmission_request(request_for(2, 0b10000000))?;
        tokio::time::sleep(Duration::from_millis(10)).await;

        // This request for frame 1 must be throttled and must NOT cancel frame 1's pending
        // sender-side full-frame resend.
        state.incoming_retransmission_request(request_for(1, 0b11000000))?;

        // Wait well past the full-frame RTO: the sender-side resend must still fire.
        tokio::time::sleep(4 * cfg.expected_packet_latency).await;
        state.stop()?;

        let ctl_msgs = ctl_rx.collect::<Vec<_>>().await;
        assert_eq!(
            2,
            ctl_msgs.len(),
            "no segments must be sent on the throttled request, but the full-frame resend must fire"
        );
        assert_eq!(SessionMessage::Segment(expected_segments[0].clone()), ctl_msgs[0]);
        assert_eq!(SessionMessage::Segment(expected_segments[1].clone()), ctl_msgs[1]);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn ack_state_receiver_must_not_cancel_our_full_frame_resend_when_it_admits_but_cannot_serve_a_frame()
    -> anyhow::Result<()> {
        let cfg = AcknowledgementStateConfig {
            mode: AcknowledgementMode::Full,
            expected_packet_latency: Duration::from_millis(100), // full-frame RTO = 3 * 100 ms
            // A single sender-side full-frame resend, so the assertion stays deterministic.
            max_outgoing_frame_retries: 1,
            // A generous budget, so the request below is admitted and the "nothing admitted"
            // early return is never taken: the frame is admitted but cannot be SERVED, because
            // the request names a segment index that does not exist in the ring buffer.
            max_frame_retransmission_services: usize::MAX,
            max_frame_retransmission_burst: usize::MAX,
            frame_retransmission_refill_interval: Some(Duration::from_secs(60)),
            min_frame_retransmission_interval: Some(Duration::from_millis(1)),
            ..Default::default()
        };

        let inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = bounded_sink_channel::<SessionMessage<MTU>>(1024);

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        let expected_segments = segment(hopr_types::crypto_random::random_bytes::<{ 2 * MTU }>(), MTU, 1)?;
        for segment in &expected_segments {
            state.segment_sent(segment)?;
        }

        tokio::time::sleep(Duration::from_millis(10)).await;

        // Frame 1 has only 2 segments (indices 0 and 1), so segment index 6 (bit 1 in the Msb0
        // bitmap, where the MSB is index 0) does not exist in the ring buffer: frame 1 is
        // admitted, but nothing is served for it and nothing is sent back. This must NOT cancel
        // frame 1's pending sender-side full-frame resend.
        state.incoming_retransmission_request(request_for(1, 0b00000010))?;

        // Wait well past the full-frame RTO: the sender-side resend must still fire.
        tokio::time::sleep(4 * cfg.expected_packet_latency).await;
        state.stop()?;

        let ctl_msgs = ctl_rx.collect::<Vec<_>>().await;
        assert_eq!(
            2,
            ctl_msgs.len(),
            "nothing must be sent on the unservable request, but the full-frame resend must fire"
        );
        assert_eq!(SessionMessage::Segment(expected_segments[0].clone()), ctl_msgs[0]);
        assert_eq!(SessionMessage::Segment(expected_segments[1].clone()), ctl_msgs[1]);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn ack_state_receiver_must_service_an_honest_request_unchanged() -> anyhow::Result<()> {
        // Entirely default configuration: an honest peer must observe no throttling at all.
        let cfg = AcknowledgementStateConfig::default();

        let inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = bounded_sink_channel::<SessionMessage<MTU>>(1024);

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        let expected_segments = segment(hopr_types::crypto_random::random_bytes::<{ 8 * MTU }>(), MTU, 1)?;
        assert_eq!(8, expected_segments.len());
        for segment in &expected_segments {
            state.segment_sent(segment)?;
        }

        // An honest request naming all 8 segments of the frame must get all 8 back.
        state.incoming_retransmission_request(request_for(1, 0b11111111))?;

        tokio::time::sleep(Duration::from_millis(100)).await;
        state.stop()?;

        let ctl_msgs = ctl_rx.collect::<Vec<_>>().await;
        assert_eq!(
            8,
            ctl_msgs.len(),
            "all requested segments must be sent under the default config"
        );
        for (i, expected_segment) in expected_segments.iter().enumerate() {
            assert_eq!(SessionMessage::Segment(expected_segment.clone()), ctl_msgs[i]);
        }

        Ok(())
    }

    #[tokio::test]
    async fn retransmission_budget_must_not_grow_on_unserved_frame_ids() -> anyhow::Result<()> {
        let cfg = AcknowledgementStateConfig::default();

        let inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = bounded_sink_channel::<SessionMessage<MTU>>(1024);

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        // A full-size request naming bogus frame ids that are not in the ring buffer: the budget
        // must admit them (charging tokens) but must not allocate any per-frame bookkeeping.
        let bogus_request = (1..=SegmentRequest::<MTU>::MAX_ENTRIES)
            .map(|i| (i as FrameId, [0b11111111].into()))
            .collect::<SegmentRequest<MTU>>();
        state.incoming_retransmission_request(bogus_request)?;

        // Cloned out before `stop`, which takes the context away.
        let budget = state
            .context
            .as_ref()
            .context("state must be running")?
            .retransmission_budget
            .clone();

        state.stop()?;

        assert!(ctl_rx.collect::<Vec<_>>().await.is_empty());

        let budget = budget.lock();
        assert!(
            budget.services.is_empty(),
            "unserved frame ids must not allocate bookkeeping"
        );
        assert_eq!(budget.throttled, 0);

        Ok(())
    }

    #[test]
    fn retransmission_budget_must_refill_a_full_allowance_within_one_refill_interval() {
        // The config documents the burst as "frames served ACROSS a refill interval", so one whole
        // interval must restore the entire allowance. Earning a single token per interval instead
        // would take `max_tokens` intervals to refill, starving honest requests for a long time.
        const TOKENS: usize = 10;
        let interval = Duration::from_millis(100);
        let t0 = Instant::now();

        // A fresh bucket, drained at `t0`. Each scenario gets its own so that the refill clock
        // advancing during one scenario cannot shift the baseline of the next.
        let drained = || {
            let mut budget = RetransmissionBudget::new(
                Duration::from_millis(1), // per-frame interval, irrelevant here
                usize::MAX,               // per-frame cap, irrelevant here
                interval,
                TOKENS,
            );
            for _ in 0..TOKENS {
                assert!(budget.admit(1, t0), "the initial allowance must be spendable");
            }
            assert!(!budget.admit(1, t0), "an exhausted bucket must refuse");
            budget
        };

        // Drains the bucket at `at`, returning how many frames were admitted.
        fn drain_to(budget: &mut RetransmissionBudget, at: Instant) -> usize {
            let mut admitted = 0;
            while budget.admit(1, at) {
                admitted += 1;
                assert!(admitted <= TOKENS, "must never exceed the allowance");
            }
            admitted
        }

        // Half an interval restores about half the allowance.
        let mut budget = drained();
        assert_eq!(TOKENS / 2, drain_to(&mut budget, t0 + interval / 2));

        // A full interval restores the entire allowance.
        let mut budget = drained();
        assert_eq!(
            TOKENS,
            drain_to(&mut budget, t0 + interval),
            "a full refill interval must restore the whole allowance"
        );

        // Idling far longer than the interval must not overfill the bucket.
        let mut budget = drained();
        assert_eq!(
            TOKENS,
            drain_to(&mut budget, t0 + Duration::from_secs(3600)),
            "the bucket must cap at max_tokens"
        );

        // A steady stream of sub-token calls must still refill at the full rate: the leftover
        // time is carried, so `TOKENS` calls a tenth of the interval apart restore the allowance.
        let mut budget = drained();
        let mut admitted = 0;
        for step in 1..=TOKENS as u32 {
            admitted += drain_to(&mut budget, t0 + interval * step / TOKENS as u32);
        }
        assert_eq!(
            TOKENS, admitted,
            "sub-token calls must not discard the elapsed remainder"
        );
    }

    #[test]
    fn ack_state_config_should_derive_retransmission_budget_defaults() {
        use hopr_protocol_app::prelude::ApplicationData;

        // The production constants these defaults are derived from.
        assert_eq!(3238, ApplicationData::PAYLOAD_SIZE);
        assert_eq!(646, SegmentRequest::<{ ApplicationData::PAYLOAD_SIZE }>::MAX_ENTRIES);
        assert_eq!(
            8,
            SegmentRequest::<{ ApplicationData::PAYLOAD_SIZE }>::MAX_MISSING_SEGMENTS_PER_FRAME
        );

        let derived = AcknowledgementStateConfig::default().normalize::<{ ApplicationData::PAYLOAD_SIZE }>();
        assert_eq!(
            Some(Duration::from_millis(10)),
            derived.min_frame_retransmission_interval
        );
        assert_eq!(3 * 8, derived.max_frame_retransmission_services);
        assert_eq!(2 * 646, derived.max_frame_retransmission_burst);
        assert_eq!(
            Some(Duration::from_millis(20)),
            derived.frame_retransmission_refill_interval
        );

        // ZERO and 0 mean "derive the safe default" and can never disable the bound.
        let explicit_zero = AcknowledgementStateConfig {
            min_frame_retransmission_interval: Some(Duration::ZERO),
            max_frame_retransmission_services: 0,
            max_frame_retransmission_burst: 0,
            frame_retransmission_refill_interval: Some(Duration::ZERO),
            ..Default::default()
        }
        .normalize::<{ ApplicationData::PAYLOAD_SIZE }>();
        assert_eq!(derived, explicit_zero);

        // Explicitly configured values are kept as they are.
        let explicit = AcknowledgementStateConfig {
            min_frame_retransmission_interval: Some(Duration::from_millis(5)),
            max_frame_retransmission_services: 7,
            max_frame_retransmission_burst: 9,
            frame_retransmission_refill_interval: Some(Duration::from_millis(11)),
            ..Default::default()
        }
        .normalize::<{ ApplicationData::PAYLOAD_SIZE }>();
        assert_eq!(
            Some(Duration::from_millis(5)),
            explicit.min_frame_retransmission_interval
        );
        assert_eq!(7, explicit.max_frame_retransmission_services);
        assert_eq!(9, explicit.max_frame_retransmission_burst);
        assert_eq!(
            Some(Duration::from_millis(11)),
            explicit.frame_retransmission_refill_interval
        );
    }
}
