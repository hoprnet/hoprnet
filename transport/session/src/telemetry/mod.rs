use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use hopr_protocol_session::SessionMessageDiscriminants;

pub use crate::balancer::{AtomicSurbFlowEstimator, BalancerStateValues};
use crate::{Capability, HoprSessionConfig, SessionId, types::SESSION_SOCKET_CAPACITY};

// Bounded node-level PIX Exit aggregates, kept apart from the per-Session instruments above because
// they answer a different question for a different consumer and reach a different exporter.
//
// A `//` comment rather than a `///` doc on purpose: rustdoc merges an outer doc on a `mod` item
// with that module's own `//!` header and then resolves the *whole* merged block in the parent's
// scope, which silently breaks every intra-doc link the module wrote against its own scope. The
// module documents itself.
pub(crate) mod pix;

/// Wrapper type to implement SessionTelemetryTracker for SessionId (HoprPseudonym).
/// This is needed to satisfy the orphan rule - we can only implement external traits
/// for local types, so we create a local wrapper.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[allow(dead_code)]
struct SessionIdWrapper(SessionId);

impl SessionIdWrapper {
    /// Returns the session label as a `&str` without allocating.
    #[allow(unused)]
    fn label(&self) -> &str {
        self.0.as_ref()
    }
}

impl std::ops::Deref for SessionIdWrapper {
    type Target = SessionId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<SessionId> for SessionIdWrapper {
    fn from(id: SessionId) -> Self {
        Self(id)
    }
}

lazy_static::lazy_static! {
    static ref METRIC_SESSION_SNAPSHOT_AT_MS: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_session_snapshot_at_ms",
        "Session telemetry sample time in unix milliseconds",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_LIFETIME_CREATED_AT_MS: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_session_lifetime_created_at_ms",
        "Session creation time in unix milliseconds",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_LIFETIME_LAST_ACTIVITY_AT_MS: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_session_lifetime_last_activity_at_ms",
        "Last session activity time in unix milliseconds",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_LIFETIME_UPTIME_MS: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_session_lifetime_uptime_ms",
        "Session uptime in milliseconds",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_LIFETIME_IDLE_MS: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_session_lifetime_idle_ms",
        "Session idle time in milliseconds",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_LIFETIME_STATE: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_session_lifetime_state",
        "Session lifecycle state encoded as Active=0, Closing=1, Closed=2",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_LIFETIME_PIPELINE_ERRORS_TOTAL: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_session_lifetime_pipeline_errors_total",
        "Session pipeline processing errors",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_FRAME_MTU_BYTES: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_session_frame_mtu_bytes",
        "Configured frame MTU in bytes",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_FRAME_TIMEOUT_MS: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_session_frame_timeout_ms",
        "Configured frame timeout in milliseconds",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_FRAME_FRAME_CAPACITY: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_session_frame_frame_capacity",
        "Configured frame buffer capacity",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_FRAME_BEING_ASSEMBLED: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_session_frame_being_assembled",
        "Number of frames currently being assembled",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_FRAME_COMPLETED_TOTAL: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_session_frame_completed_total",
        "Number of frames successfully completed",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_FRAME_EMITTED_TOTAL: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_session_frame_emitted_total",
        "Number of frames emitted from the sequencer",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_FRAME_DISCARDED_TOTAL: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_session_frame_discarded_total",
        "Number of frames discarded by the session protocol",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_ACK_MODE: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_session_ack_mode",
        "Configured ack mode encoded as None=0, Partial=1, Full=2, Both=3",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_ACK_INCOMING_SEGMENTS_TOTAL: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_session_ack_incoming_segments_total",
        "Incoming session segments",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_ACK_INCOMING_RETRANSMISSION_REQUESTS_TOTAL: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_session_ack_incoming_retransmission_requests_total",
        "Incoming session retransmission requests",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_ACK_INCOMING_ACKNOWLEDGED_FRAMES_TOTAL: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_session_ack_incoming_acknowledged_frames_total",
        "Incoming session acknowledgements",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_ACK_OUTGOING_SEGMENTS_TOTAL: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_session_ack_outgoing_segments_total",
        "Outgoing session segments",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_ACK_OUTGOING_RETRANSMISSION_REQUESTS_TOTAL: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_session_ack_outgoing_retransmission_requests_total",
        "Outgoing session retransmission requests",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_ACK_OUTGOING_ACKNOWLEDGED_FRAMES_TOTAL: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_session_ack_outgoing_acknowledged_frames_total",
        "Outgoing session acknowledgements",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_SURB_PRODUCED_TOTAL: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_session_surb_produced_total",
        "Produced SURBs per session",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_SURB_CONSUMED_TOTAL: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_session_surb_consumed_total",
        "Consumed SURBs per session",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_SURB_BUFFER_ESTIMATE: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_session_surb_buffer_estimate",
        "Balancer estimate of the SURBs the counterparty holds, bounded by its store and corrected by its reports",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_SURB_TARGET_BUFFER: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_session_surb_target_buffer",
        "Configured SURB target buffer size",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_SURB_RATE_PER_SEC: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_session_surb_rate_per_sec",
        "SURB production rate per second, averaged over at least one second; see hopr_surb_balancer_surbs_rate for the net rate",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_SURB_REFILL_IN_FLIGHT: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_session_surb_refill_in_flight",
        "Whether SURB refill is currently configured for a session (1 or 0)",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_TRANSPORT_BYTES_IN_TOTAL: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_session_transport_bytes_in_total",
        "Session ingress bytes",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_TRANSPORT_BYTES_OUT_TOTAL: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_session_transport_bytes_out_total",
        "Session egress bytes",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_TRANSPORT_PACKETS_IN_TOTAL: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_session_transport_packets_in_total",
        "Session ingress packets",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_TRANSPORT_PACKETS_OUT_TOTAL: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_session_transport_packets_out_total",
        "Session egress packets",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_PIX_GATE_MODE: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_session_pix_gate_mode",
        "PIX egress gate mode encoded as Predeposit=0, Funded=1",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_PIX_CLOSURES_TOTAL: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_session_pix_closures_total",
        "Sessions closed by the PIX supervisor, by reason",
        &["reason"]
    ).unwrap();
    static ref METRIC_SESSION_PIX_RECOVERY_PROGRESS: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_session_pix_recovery_progress",
        "Recovery progress of a session's most recently advanced SSA, as a ratio of useful shares to target",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_PIX_FILL_RATE: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_session_pix_fill_rate",
        "Planned rate in packets per second for the Exit's own PIX fill keep-alives; emission is bounded by the SURB reserve, so see hopr_session_pix_fill_packets_total for what went out",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_PIX_FILL_PACKETS_TOTAL: hopr_api::types::telemetry::SimpleCounter = hopr_api::types::telemetry::SimpleCounter::new(
        "hopr_session_pix_fill_packets_total",
        "PIX fill keep-alives originated by this Exit, above its SURB-level notification rate"
    ).unwrap();
    static ref METRIC_SESSION_PIX_FILL_BACKOFF_TOTAL: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_session_pix_fill_backoff_total",
        "Times PIX fill held back, by reason",
        &["reason"]
    ).unwrap();
    static ref SESSION_RUNTIME: parking_lot::Mutex<HashMap<SessionId, SessionRuntimeState>> = parking_lot::Mutex::new(HashMap::new());
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde_repr::Serialize_repr)]
#[repr(u8)]
pub enum SessionLifecycleState {
    Active = 0,
    Closing = 1,
    Closed = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde_repr::Serialize_repr)]
#[repr(u8)]
pub enum SessionAckMode {
    None,
    Partial,
    Full,
    Both,
}

#[derive(Debug)]
struct SessionSurbRuntimeState {
    state: Arc<BalancerStateValues>,
    estimator: AtomicSurbFlowEstimator,
    last_snapshot_produced: u64,
    last_snapshot_us: u64,
}

/// Shortest rate window; refreshes happen on activity, so one SURB 1 ms after the last snapshot would read 1000/s.
const SURB_RATE_WINDOW_US: u64 = 1_000_000;

#[derive(Debug)]
struct SessionRuntimeState {
    created_at_us: u64,
    last_activity_us: u64,
    #[allow(dead_code)]
    frames_being_assembled: u64,
    surb: Option<SessionSurbRuntimeState>,
}

impl SessionRuntimeState {
    fn new(now_us: u64) -> Self {
        Self {
            created_at_us: now_us,
            last_activity_us: now_us,
            frames_being_assembled: 0,
            surb: None,
        }
    }
}

fn session_ack_mode(capabilities: CapabilitySet) -> SessionAckMode {
    if capabilities.contains(Capability::RetransmissionAck | Capability::RetransmissionNack) {
        SessionAckMode::Both
    } else if capabilities.contains(Capability::RetransmissionAck) {
        SessionAckMode::Full
    } else if capabilities.contains(Capability::RetransmissionNack) {
        SessionAckMode::Partial
    } else {
        SessionAckMode::None
    }
}

type CapabilitySet = flagset::FlagSet<Capability>;

pub fn initialize_session_metrics(session_id: SessionId, cfg: HoprSessionConfig) {
    let now = now_us();
    let ack_mode = session_ack_mode(cfg.capabilities);
    let session_id_str: &str = session_id.as_ref();

    METRIC_SESSION_LIFETIME_CREATED_AT_MS.set(&[session_id_str], now as f64 / 1_000.0);
    METRIC_SESSION_LIFETIME_STATE.set(&[session_id_str], SessionLifecycleState::Active as u8 as f64);
    METRIC_SESSION_ACK_MODE.set(&[session_id_str], ack_mode as u8 as f64);
    METRIC_SESSION_FRAME_MTU_BYTES.set(&[session_id_str], cfg.frame_mtu as f64);
    METRIC_SESSION_FRAME_TIMEOUT_MS.set(&[session_id_str], cfg.frame_timeout.as_millis() as f64);
    METRIC_SESSION_FRAME_FRAME_CAPACITY.set(&[session_id_str], SESSION_SOCKET_CAPACITY as f64);
    METRIC_SESSION_FRAME_BEING_ASSEMBLED.set(&[session_id_str], 0.0);
    METRIC_SESSION_SURB_BUFFER_ESTIMATE.set(&[session_id_str], 0.0);
    METRIC_SESSION_SURB_TARGET_BUFFER.set(&[session_id_str], 0.0);
    METRIC_SESSION_SURB_RATE_PER_SEC.set(&[session_id_str], 0.0);
    METRIC_SESSION_SURB_REFILL_IN_FLIGHT.set(&[session_id_str], 0.0);

    {
        let mut state = SESSION_RUNTIME.lock();
        state.insert(session_id, SessionRuntimeState::new(now));
    }

    refresh_lifetime_metrics(&session_id, now, now, now);
}

pub fn remove_session_metrics_state(session_id: &SessionId, has_pix: bool) {
    // Before the zeroing below: a concurrent touch landing in between would refresh the gauges back to nonzero.
    SESSION_RUNTIME.lock().remove(session_id);

    let session_id_str: &str = session_id.as_ref();
    METRIC_SESSION_FRAME_BEING_ASSEMBLED.set(&[session_id_str], 0.0);
    // Unconditional unlike PIX below: minted for every Session anyway, and a closed one must stop reporting them.
    METRIC_SESSION_SURB_BUFFER_ESTIMATE.set(&[session_id_str], 0.0);
    METRIC_SESSION_SURB_RATE_PER_SEC.set(&[session_id_str], 0.0);
    METRIC_SESSION_SURB_REFILL_IN_FLIGHT.set(&[session_id_str], 0.0);
    if has_pix {
        // Only for a Session that had a gate: setting these unconditionally would mint a
        // `hopr_session_pix_*` series for every non-PIX Session that ever closed.
        METRIC_SESSION_PIX_GATE_MODE.set(&[session_id_str], 0.0);
        METRIC_SESSION_PIX_RECOVERY_PROGRESS.set(&[session_id_str], 0.0);
        // Zeroed rather than left where it was, because this gauge is the one an operator would read
        // as "this node is currently sending". A closed Session that kept its last rate would be
        // indistinguishable from one that is still filling at it.
        METRIC_SESSION_PIX_FILL_RATE.set(&[session_id_str], 0.0);
    }
}

pub fn set_session_state(session_id: &SessionId, state: SessionLifecycleState) {
    let session_id_str: &str = session_id.as_ref();
    METRIC_SESSION_LIFETIME_STATE.set(&[session_id_str], state as u8 as f64);
    touch_session_activity(session_id);
}

fn update_session_activity_locked(
    session_id: &SessionId,
    now: u64,
    state: &mut HashMap<SessionId, SessionRuntimeState>,
) {
    if let Some(runtime) = state.get_mut(session_id) {
        runtime.last_activity_us = now;
        refresh_lifetime_metrics(session_id, now, runtime.created_at_us, runtime.last_activity_us);
        refresh_surb_gauges(session_id, runtime, now);
    }
}

pub fn touch_session_activity(session_id: &SessionId) {
    let now = now_us();
    if let Some(mut state) = SESSION_RUNTIME.try_lock() {
        update_session_activity_locked(session_id, now, &mut state);
    }
}

pub fn record_session_read(session_id: &SessionId, bytes: usize) {
    if bytes == 0 {
        return;
    }

    touch_session_activity(session_id);
    let session_id_str: &str = session_id.as_ref();
    METRIC_SESSION_TRANSPORT_BYTES_IN_TOTAL.increment_by(&[session_id_str], bytes as u64);
    METRIC_SESSION_TRANSPORT_PACKETS_IN_TOTAL.increment_by(&[session_id_str], 1);
}

pub fn record_session_write(session_id: &SessionId, bytes: usize) {
    if bytes == 0 {
        return;
    }

    touch_session_activity(session_id);
    let session_id_str: &str = session_id.as_ref();
    METRIC_SESSION_TRANSPORT_BYTES_OUT_TOTAL.increment_by(&[session_id_str], bytes as u64);
    METRIC_SESSION_TRANSPORT_PACKETS_OUT_TOTAL.increment_by(&[session_id_str], 1);
}

pub fn set_session_balancer_data(
    session_id: &SessionId,
    estimator: AtomicSurbFlowEstimator,
    state: Arc<BalancerStateValues>,
) {
    let now = now_us();
    let produced = estimator.produced.load(std::sync::atomic::Ordering::Relaxed);
    {
        let mut all = SESSION_RUNTIME.lock();
        let runtime = all.entry(*session_id).or_insert_with(|| SessionRuntimeState::new(now));
        runtime.surb = Some(SessionSurbRuntimeState {
            state,
            estimator,
            // Seeded from the running total: production from before telemetry attached is not new production.
            last_snapshot_produced: produced,
            last_snapshot_us: now,
        });
    }

    let session_id_str: &str = session_id.as_ref();
    METRIC_SESSION_SURB_REFILL_IN_FLIGHT.set(&[session_id_str], 1.0);
    touch_session_activity(session_id);
}

pub fn record_session_surb_produced(session_id: &SessionId, by: u64) {
    let session_id_str: &str = session_id.as_ref();
    METRIC_SESSION_SURB_PRODUCED_TOTAL.increment_by(&[session_id_str], by);
    touch_session_activity(session_id);
}

pub fn record_session_surb_consumed(session_id: &SessionId, by: u64) {
    let session_id_str: &str = session_id.as_ref();
    METRIC_SESSION_SURB_CONSUMED_TOTAL.increment_by(&[session_id_str], by);
    touch_session_activity(session_id);
}

/// Records whether a PIX Session's current front cycle has funded service.
///
/// The supervisor moves this in both directions as the front rotates. An unfunded successor reports
/// `false` while it consumes its bounded predeposit allowance; its deposit moves the gauge to `true`.
pub fn set_pix_gate_mode(session_id: &SessionId, funded: bool) {
    let session_id_str: &str = session_id.as_ref();
    METRIC_SESSION_PIX_GATE_MODE.set(&[session_id_str], if funded { 1.0 } else { 0.0 });
    touch_session_activity(session_id);
}

/// Reports how far a Session's most recently advanced SSA has got towards recovery.
///
/// Keyed by Session, not by `(session, ssa_index)`. The index rises for the lifetime of a Session
/// and never repeats, so labelling by it would mint a new time series per SSA cycle and never
/// retire any of them. A Session runs at most a few cycles concurrently and they advance together,
/// so the latest snapshot is a fair summary of where recovery stands.
///
/// Unlike its siblings this does **not** call [`touch_session_activity`], and the asymmetry is
/// deliberate rather than an omission. Shares reach the Exit only on data-packet acknowledgements,
/// so a Session whose recovery is advancing is a Session passing data, and
/// [`record_session_read`]/[`record_session_write`] have already stamped its activity on the packets
/// that carried those shares. Stamping again here would put a second atomic on a path that runs per
/// progress snapshot and change nothing: `hopr_session_lifetime_idle_ms` cannot climb for a Session
/// that is making progress, because the traffic producing the progress is what keeps it down.
pub fn set_pix_recovery_progress(session_id: &SessionId, useful_shares: u64, target_useful_shares: u64) {
    if target_useful_shares == 0 {
        return;
    }
    let session_id_str: &str = session_id.as_ref();
    let ratio = (useful_shares as f64 / target_useful_shares as f64).clamp(0.0, 1.0);
    METRIC_SESSION_PIX_RECOVERY_PROGRESS.set(&[session_id_str], ratio);
}

/// Records the rate at which the Exit is currently originating PIX fill keep-alives.
///
/// Set from the supervisor's planned rate rather than measured at the stream, so it reports the
/// decision. What actually goes out can be lower — the SURB reserve withholds packets, and that shows
/// up in [`record_pix_fill_backoff`] — and the pair is more informative than either alone: a fill
/// rate pinned at its ceiling with a climbing backoff counter is a Session whose SURB supply, not its
/// deadline, is the thing failing.
///
/// Deliberately not [`touch_session_activity`]: fill is this node talking to itself about a Session
/// the application has abandoned, and counting that as activity would make
/// `hopr_session_lifetime_idle_ms` report the opposite of what it means.
pub fn set_pix_fill_rate(session_id: &SessionId, packets_per_sec: f64) {
    let session_id_str: &str = session_id.as_ref();
    METRIC_SESSION_PIX_FILL_RATE.set(&[session_id_str], packets_per_sec);
}

/// Counts one PIX fill keep-alive put on the wire.
///
/// Node-wide rather than per Session, unlike its gauge sibling. This is the figure an operator sizes
/// bandwidth against, and that question is about the node; per-Session totals would mint a counter
/// series per Session that is never retired, for a number that is only interesting in aggregate.
pub fn record_pix_fill_packet() {
    METRIC_SESSION_PIX_FILL_PACKETS_TOTAL.increment();
}

/// Why PIX fill sent less than its planned rate.
///
/// A closed enum, so it can be a metric label without unbounded cardinality — the same argument as
/// [`record_pix_closure`], and for the same reason it takes the enum rather than a string.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::Display)]
pub enum PixFillBackoff {
    /// The Session's estimated SURB level was below `fill.min_surb_reserve`, so the packet was
    /// withheld to leave the Session able to answer its own application.
    SurbReserve,
    /// The cycle being filled for stopped making progress for `max_recovery_idle`, so the planner
    /// dropped back to its heartbeat rather than spend a whole deadline's worth of SURBs on a cycle
    /// that may never recover.
    Stalled,
}

/// Counts one occasion on which PIX fill held back, labelled by why.
///
/// The two reasons are counted at different granularities on purpose, because they are different
/// events: `SurbReserve` is per withheld packet, and `Stalled` is per stall — the planner warns and
/// counts once when a cycle goes motionless, not once per second for as long as it stays that way.
pub fn record_pix_fill_backoff(reason: PixFillBackoff) {
    METRIC_SESSION_PIX_FILL_BACKOFF_TOTAL.increment(&[reason.to_string().as_str()]);
}

/// Counts a Session closed by the PIX supervisor, labelled by why.
///
/// Labelled by reason rather than by Session: the reasons are a closed enum, so the cardinality is
/// bounded, which is what makes this safe to keep after the Session is gone.
///
/// Takes the enum rather than a `&str` so that boundedness is a property of the signature instead of
/// a promise the doc makes on behalf of every future caller. `&'static str` would not have done it
/// either — it admits any string literal, and a literal is exactly what an unbounded label looks
/// like at the call site. The label is derived here, so there is one spelling of each reason.
pub fn record_pix_closure(reason: crate::supervision::SessionPixCloseReason) {
    METRIC_SESSION_PIX_CLOSURES_TOTAL.increment(&[reason.to_string().as_str()]);
}

fn refresh_lifetime_metrics(session_id: &SessionId, now_us: u64, created_at_us: u64, last_activity_us: u64) {
    let session_id_str: &str = session_id.as_ref();
    METRIC_SESSION_SNAPSHOT_AT_MS.set(&[session_id_str], now_us as f64 / 1_000.0);
    METRIC_SESSION_LIFETIME_LAST_ACTIVITY_AT_MS.set(&[session_id_str], last_activity_us as f64 / 1_000.0);
    METRIC_SESSION_LIFETIME_UPTIME_MS.set(&[session_id_str], now_us.saturating_sub(created_at_us) as f64 / 1_000.0);
    METRIC_SESSION_LIFETIME_IDLE_MS.set(
        &[session_id_str],
        now_us.saturating_sub(last_activity_us) as f64 / 1_000.0,
    );
}

fn refresh_surb_gauges(session_id: &SessionId, runtime: &mut SessionRuntimeState, now_us: u64) {
    let Some(surb) = runtime.surb.as_mut() else {
        return;
    };

    let session_id_str: &str = session_id.as_ref();
    METRIC_SESSION_SURB_TARGET_BUFFER.set(
        &[session_id_str],
        surb.state
            .target_surb_buffer_size
            .load(std::sync::atomic::Ordering::Relaxed) as f64,
    );
    // The balancer level, not `produced - consumed`: only it is clamped to the store and corrected by its reports.
    METRIC_SESSION_SURB_BUFFER_ESTIMATE.set(&[session_id_str], surb.state.buffer_level() as f64);

    let elapsed_us = now_us.saturating_sub(surb.last_snapshot_us);
    if elapsed_us < SURB_RATE_WINDOW_US {
        return;
    }

    // Production alone: the level jumps on counterparty reports and clamps, which are not flow.
    let produced = surb.estimator.produced.load(std::sync::atomic::Ordering::Relaxed);
    let produced_since = produced.saturating_sub(surb.last_snapshot_produced);
    surb.last_snapshot_produced = produced;
    surb.last_snapshot_us = now_us;

    METRIC_SESSION_SURB_RATE_PER_SEC.set(
        &[session_id_str],
        produced_since as f64 / (elapsed_us as f64 / 1_000_000.0),
    );
}

fn now_us() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_micros(0))
        .as_micros() as u64
}

#[allow(dead_code)]
fn increment_frame_assembly_gauge(session_id: &SessionId) {
    let mut state = SESSION_RUNTIME.lock();
    let runtime = state
        .entry(*session_id)
        .or_insert_with(|| SessionRuntimeState::new(now_us()));
    runtime.frames_being_assembled = runtime.frames_being_assembled.saturating_add(1);
    let session_id_str: &str = session_id.as_ref();
    METRIC_SESSION_FRAME_BEING_ASSEMBLED.increment(&[session_id_str], 1.0);
}

#[allow(dead_code)]
fn decrement_frame_assembly_gauge(session_id: &SessionId) {
    let mut state = SESSION_RUNTIME.lock();
    let Some(runtime) = state.get_mut(session_id) else {
        return;
    };

    if runtime.frames_being_assembled == 0 {
        return;
    }

    runtime.frames_being_assembled -= 1;
    let session_id_str: &str = session_id.as_ref();
    METRIC_SESSION_FRAME_BEING_ASSEMBLED.decrement(&[session_id_str], 1.0);
}

impl hopr_protocol_session::SessionTelemetryTracker for SessionIdWrapper {
    fn frame_emitted(&self) {
        METRIC_SESSION_FRAME_EMITTED_TOTAL.increment(&[self.label()]);
    }

    fn frame_completed(&self) {
        METRIC_SESSION_FRAME_COMPLETED_TOTAL.increment(&[self.label()]);
        decrement_frame_assembly_gauge(&self.0);
    }

    fn frame_discarded(&self) {
        METRIC_SESSION_FRAME_DISCARDED_TOTAL.increment(&[self.label()]);
        decrement_frame_assembly_gauge(&self.0);
    }

    fn incomplete_frame(&self) {
        decrement_frame_assembly_gauge(&self.0);
    }

    fn incoming_message(&self, msg: SessionMessageDiscriminants) {
        match msg {
            SessionMessageDiscriminants::Segment => {
                METRIC_SESSION_ACK_INCOMING_SEGMENTS_TOTAL.increment(&[self.label()])
            }
            SessionMessageDiscriminants::Request => {
                METRIC_SESSION_ACK_INCOMING_RETRANSMISSION_REQUESTS_TOTAL.increment(&[self.label()])
            }
            SessionMessageDiscriminants::Acknowledge => {
                METRIC_SESSION_ACK_INCOMING_ACKNOWLEDGED_FRAMES_TOTAL.increment(&[self.label()])
            }
        }
    }

    fn outgoing_message(&self, msg: SessionMessageDiscriminants) {
        match msg {
            SessionMessageDiscriminants::Segment => {
                METRIC_SESSION_ACK_OUTGOING_SEGMENTS_TOTAL.increment(&[self.label()])
            }
            SessionMessageDiscriminants::Request => {
                METRIC_SESSION_ACK_OUTGOING_RETRANSMISSION_REQUESTS_TOTAL.increment(&[self.label()])
            }
            SessionMessageDiscriminants::Acknowledge => {
                METRIC_SESSION_ACK_OUTGOING_ACKNOWLEDGED_FRAMES_TOTAL.increment(&[self.label()])
            }
        }
    }

    fn error(&self) {
        METRIC_SESSION_LIFETIME_PIPELINE_ERRORS_TOTAL.increment(&[self.label()]);
    }
}

#[cfg(test)]
mod tests {
    use hopr_api::types::{crypto_random::Randomizable, internal::prelude::HoprPseudonym};
    use hopr_protocol_session::SessionTelemetryTracker;

    use super::*;

    #[test]
    fn session_metrics_are_exported_through_hopr_metrics() {
        let id: SessionId = HoprPseudonym::random();
        initialize_session_metrics(id, HoprSessionConfig::default());
        record_session_read(&id, 10);
        SessionIdWrapper::from(id).frame_completed();

        let text = hopr_api::types::telemetry::gather_all_metrics().expect("must gather metrics");
        let session_id: &str = id.as_ref();
        let ingress_metric = format!("hopr_session_transport_bytes_in_total{{session_id=\"{session_id}\"}} 10");
        let frame_metric = format!("hopr_session_frame_completed_total{{session_id=\"{session_id}\"}} 1");
        let mode_metric = format!(
            "hopr_session_ack_mode{{session_id=\"{session_id}\"}} {}",
            SessionAckMode::None as u8
        );

        assert!(text.contains(&ingress_metric));
        assert!(text.contains(&frame_metric));
        assert!(text.contains(&mode_metric));
    }

    #[test]
    fn surb_metrics_are_exported_through_hopr_metrics() {
        let id: SessionId = HoprPseudonym::random();
        initialize_session_metrics(id, HoprSessionConfig::default());
        let estimator = AtomicSurbFlowEstimator::default();
        let state = Arc::new(BalancerStateValues::new(Default::default()));

        set_session_balancer_data(&id, estimator, Arc::clone(&state));
        record_session_surb_produced(&id, 8);
        record_session_surb_consumed(&id, 3);

        let text = hopr_api::types::telemetry::gather_all_metrics().expect("must gather metrics");
        let session_id: &str = id.as_ref();
        let produced_metric = format!("hopr_session_surb_produced_total{{session_id=\"{session_id}\"}} 8");
        let consumed_metric = format!("hopr_session_surb_consumed_total{{session_id=\"{session_id}\"}} 3");

        assert!(text.contains(&produced_metric));
        assert!(text.contains(&consumed_metric));
    }

    /// Attaches balancer data to a fresh Session and returns the pieces a gauge test drives.
    fn session_with_balancer() -> (SessionId, AtomicSurbFlowEstimator, Arc<BalancerStateValues>) {
        let id: SessionId = HoprPseudonym::random();
        initialize_session_metrics(id, HoprSessionConfig::default());
        let estimator = AtomicSurbFlowEstimator::default();
        let state = Arc::new(BalancerStateValues::new(Default::default()));
        set_session_balancer_data(&id, estimator.clone(), Arc::clone(&state));
        (id, estimator, state)
    }

    /// Refreshes at a chosen instant; the production path uses `try_lock` and can skip under parallel tests.
    fn refresh_at(session_id: &SessionId, now_us: u64) {
        let mut all = SESSION_RUNTIME.lock();
        let runtime = all.get_mut(session_id).expect("session runtime must exist");
        refresh_surb_gauges(session_id, runtime, now_us);
    }

    /// Pins where the rate window starts, so a test can choose its own instants.
    fn set_snapshot_base(session_id: &SessionId, at_us: u64) {
        let mut all = SESSION_RUNTIME.lock();
        let surb = all
            .get_mut(session_id)
            .and_then(|runtime| runtime.surb.as_mut())
            .expect("balancer data must be attached");
        surb.last_snapshot_us = at_us;
    }

    fn gauge(metric: &hopr_api::types::telemetry::MultiGauge, session_id: &SessionId) -> f64 {
        let session_id_str: &str = session_id.as_ref();
        metric.get(&[session_id_str]).expect("gauge must have been set")
    }

    /// The regression: decay refills never land in `consumed`, so the raw difference must not reach the gauge.
    #[test]
    fn surb_buffer_estimate_reports_the_balancer_level_not_the_raw_counters() {
        let (id, estimator, state) = session_with_balancer();

        estimator
            .produced
            .fetch_add(1_960_000, std::sync::atomic::Ordering::Relaxed);
        state.buffer_level.store(7_000, std::sync::atomic::Ordering::Relaxed);

        refresh_at(&id, now_us());

        assert_eq!(gauge(&METRIC_SESSION_SURB_BUFFER_ESTIMATE, &id), 7_000.0);
    }

    /// The level is corrected downwards by what the counterparty reports, and the gauge follows.
    #[test]
    fn surb_buffer_estimate_follows_the_counterparty_report_downwards() {
        let (id, _estimator, state) = session_with_balancer();

        state.buffer_level.store(4_200, std::sync::atomic::Ordering::Relaxed);
        refresh_at(&id, now_us());
        assert_eq!(gauge(&METRIC_SESSION_SURB_BUFFER_ESTIMATE, &id), 4_200.0);

        state.buffer_level.store(900, std::sync::atomic::Ordering::Relaxed);
        refresh_at(&id, now_us());
        assert_eq!(gauge(&METRIC_SESSION_SURB_BUFFER_ESTIMATE, &id), 900.0);
    }

    #[test]
    fn surb_rate_reports_production_over_at_least_one_second() {
        let (id, estimator, _state) = session_with_balancer();

        const START_US: u64 = 1_000_000_000;
        set_snapshot_base(&id, START_US);
        estimator.produced.fetch_add(500, std::sync::atomic::Ordering::Relaxed);
        estimator.consumed.fetch_add(400, std::sync::atomic::Ordering::Relaxed);

        refresh_at(&id, START_US + 500_000);
        assert_eq!(
            gauge(&METRIC_SESSION_SURB_RATE_PER_SEC, &id),
            0.0,
            "half a second is too short a window to publish a rate from"
        );

        refresh_at(&id, START_US + 1_000_000);
        assert_eq!(
            gauge(&METRIC_SESSION_SURB_RATE_PER_SEC, &id),
            500.0,
            "the rate is production alone, not production net of consumption"
        );

        estimator.produced.fetch_add(200, std::sync::atomic::Ordering::Relaxed);
        refresh_at(&id, START_US + 3_000_000);
        assert_eq!(gauge(&METRIC_SESSION_SURB_RATE_PER_SEC, &id), 100.0);
    }

    /// A closed Session must not go on reporting the level and rate it died with.
    #[test]
    fn closing_a_session_zeroes_its_surb_gauges() {
        let (id, estimator, state) = session_with_balancer();

        estimator
            .produced
            .fetch_add(3_000, std::sync::atomic::Ordering::Relaxed);
        state.buffer_level.store(3_000, std::sync::atomic::Ordering::Relaxed);
        refresh_at(&id, now_us());

        remove_session_metrics_state(&id, false);

        assert_eq!(gauge(&METRIC_SESSION_SURB_BUFFER_ESTIMATE, &id), 0.0);
        assert_eq!(gauge(&METRIC_SESSION_SURB_RATE_PER_SEC, &id), 0.0);
        assert_eq!(gauge(&METRIC_SESSION_SURB_REFILL_IN_FLIGHT, &id), 0.0);
    }
}
