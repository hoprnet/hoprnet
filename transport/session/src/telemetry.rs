use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[cfg(all(feature = "telemetry", not(test)))]
use hopr_protocol_pix::SsaIndex;
use hopr_protocol_session::SessionMessageDiscriminants;

pub use crate::balancer::{AtomicSurbFlowEstimator, BalancerStateValues};
use crate::{Capability, HoprSessionConfig, SessionId, types::SESSION_SOCKET_CAPACITY};

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
        "Estimated SURB buffer size",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_SURB_TARGET_BUFFER: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_session_surb_target_buffer",
        "Configured SURB target buffer size",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_SURB_RATE_PER_SEC: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_session_surb_rate_per_sec",
        "Estimated SURB buffer rate change per second",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_SURB_REFILL_IN_FLIGHT: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_session_surb_refill_in_flight",
        "Whether SURB refill is currently configured for a session (1 or 0)",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_PIX_GATE_MODE: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_session_pix_gate_mode",
        "PIX egress gate mode: 0=predeposit, 1=funded",
        &["session_id"]
    ).unwrap();
    static ref METRIC_SESSION_PIX_CURRENT_SSA_PHASE: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_session_pix_current_ssa_phase",
        "Current PIX SSA phase per SSA: 0=AwaitingCommitment, 1=AwaitingDeposit, 3=Recovered",
        &["session_id", "ssa_index"]
    ).unwrap();
    static ref METRIC_SESSION_PIX_RECOVERY_PROGRESS: hopr_api::types::telemetry::MultiGauge = hopr_api::types::telemetry::MultiGauge::new(
        "hopr_session_pix_recovery_progress",
        "PIX SSA recovery progress as a ratio (0.0–1.0) of useful shares to target",
        &["session_id", "ssa_index"]
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
    last_snapshot_total: u64,
    last_snapshot_us: u64,
}

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
    let session_id_str: &str = session_id.as_ref();
    METRIC_SESSION_FRAME_BEING_ASSEMBLED.set(&[session_id_str], 0.0);
    if has_pix {
        METRIC_SESSION_PIX_GATE_MODE.set(&[session_id_str], 0.0);
        // METRIC_SESSION_PIX_RECOVERY_PROGRESS is keyed by (session_id, ssa_index)
        // and cannot be reset to 0 here without tracking all SSA indices per session.
    }
    SESSION_RUNTIME.lock().remove(session_id);
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
    {
        let mut all = SESSION_RUNTIME.lock();
        let runtime = all.entry(*session_id).or_insert_with(|| SessionRuntimeState::new(now));
        runtime.surb = Some(SessionSurbRuntimeState {
            state,
            estimator,
            last_snapshot_total: 0,
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

#[cfg(all(feature = "telemetry", not(test)))]
pub fn set_pix_gate_mode(session_id: &SessionId, funded: bool) {
    let session_id_str: &str = session_id.as_ref();
    METRIC_SESSION_PIX_GATE_MODE.set(&[session_id_str], if funded { 1.0 } else { 0.0 });
}

/// Encodes the supervisor SSA phase as a numeric value.
#[cfg(all(feature = "telemetry", not(test)))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PixSsaPhase {
    AwaitingCommitment = 0,
    AwaitingDeposit = 1,
    Recovered = 3,
}

#[cfg(all(feature = "telemetry", not(test)))]
pub fn set_pix_current_ssa_phase(session_id: &SessionId, ssa_index: SsaIndex, phase: PixSsaPhase) {
    let session_id_str: &str = session_id.as_ref();
    METRIC_SESSION_PIX_CURRENT_SSA_PHASE.set(&[session_id_str, &ssa_index.to_string()], phase as u8 as f64);
}

#[cfg(all(feature = "telemetry", not(test)))]
pub fn set_pix_recovery_progress(session_id: &SessionId, ssa_index: SsaIndex, progress: f64) {
    let session_id_str: &str = session_id.as_ref();
    METRIC_SESSION_PIX_RECOVERY_PROGRESS.set(&[session_id_str, &ssa_index.to_string()], progress.clamp(0.0, 1.0));
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

    let produced = surb.estimator.produced.load(std::sync::atomic::Ordering::Relaxed);
    let consumed = surb.estimator.consumed.load(std::sync::atomic::Ordering::Relaxed);
    let total = produced.saturating_sub(consumed);

    let elapsed_us = now_us.saturating_sub(surb.last_snapshot_us);
    let rate_per_sec = if elapsed_us == 0 {
        0.0
    } else {
        let delta = total as i64 - surb.last_snapshot_total as i64;
        delta as f64 / (elapsed_us as f64 / 1_000_000.0)
    };

    surb.last_snapshot_total = total;
    surb.last_snapshot_us = now_us;

    let session_id_str: &str = session_id.as_ref();
    METRIC_SESSION_SURB_TARGET_BUFFER.set(
        &[session_id_str],
        surb.state
            .target_surb_buffer_size
            .load(std::sync::atomic::Ordering::Relaxed) as f64,
    );
    METRIC_SESSION_SURB_BUFFER_ESTIMATE.set(&[session_id_str], total as f64);
    METRIC_SESSION_SURB_RATE_PER_SEC.set(&[session_id_str], rate_per_sec);
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
}
