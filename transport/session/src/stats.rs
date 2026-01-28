//! Session stats tracking and snapshotting.
//!
//! This module provides functionality to track various stats of a HOPR session,
//! including data throughput, packet counts, frame events, and session lifecycle.
//! It allows creating immutable snapshots of the stats state for monitoring and reporting.

use std::{
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, AtomicU8, AtomicU64, AtomicUsize, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use hopr_protocol_session::{
    AcknowledgementMode, FrameAcknowledgements, FrameId, FrameInspector, Segment, SegmentId, SegmentRequest,
    SeqIndicator, SocketComponents, SocketState,
};

use crate::{SessionId, balancer::AtomicSurbFlowEstimator};

/// The lifecycle state of a session from the perspective of metrics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum SessionLifecycleState {
    /// Session is active and running.
    Active,
    /// Session is in the process of closing (e.g. sending/receiving close frames).
    Closing,
    /// Session has been fully closed.
    Closed,
}

/// The acknowledgement mode configured for the session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum SessionAckMode {
    /// No acknowledgements.
    None,
    /// Partial acknowledgements (some frames/segments).
    Partial,
    /// Full acknowledgements (all frames/segments).
    Full,
    /// Both (if applicable, though typically maps to Full in some contexts).
    Both,
}

impl SessionAckMode {
    fn from_ack_mode(mode: Option<AcknowledgementMode>) -> Self {
        match mode {
            None => SessionAckMode::None,
            Some(AcknowledgementMode::Partial) => SessionAckMode::Partial,
            Some(AcknowledgementMode::Full) => SessionAckMode::Full,
            Some(AcknowledgementMode::Both) => SessionAckMode::Both,
        }
    }
}

impl SessionLifecycleState {
    fn as_u8(self) -> u8 {
        match self {
            SessionLifecycleState::Active => 0,
            SessionLifecycleState::Closing => 1,
            SessionLifecycleState::Closed => 2,
        }
    }

    fn from_u8(value: u8) -> SessionLifecycleState {
        match value {
            1 => SessionLifecycleState::Closing,
            2 => SessionLifecycleState::Closed,
            _ => SessionLifecycleState::Active,
        }
    }
}

/// Snapshot of session lifetime metrics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionLifetimeSnapshot {
    /// Time when the session was created.
    pub created_at: SystemTime,
    /// Time of the last read or write activity.
    pub last_activity_at: SystemTime,
    /// Total duration the session has been alive.
    pub uptime: Duration,
    /// Duration since the last activity.
    pub idle: Duration,
    /// Current lifecycle state of the session.
    pub state: SessionLifecycleState,
}

/// Snapshot of frame buffer metrics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FrameBufferSnapshot {
    /// Maximum Transmission Unit for frames.
    pub frame_mtu: usize,
    /// Configured timeout for frame reassembly/acknowledgement.
    pub frame_timeout: Duration,
    /// Configured capacity of the frame buffer.
    pub frame_capacity: usize,
    /// Number of frames currently being assembled (incomplete).
    pub incomplete_frames: usize,
    /// Total number of frames successfully completed/assembled.
    pub frames_completed: u64,
    /// Total number of frames emitted to the application.
    pub frames_emitted: u64,
    /// Total number of frames discarded (e.g. due to timeout or errors).
    pub frames_discarded: u64,
}

/// Snapshot of acknowledgement metrics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AckSnapshot {
    /// Configured acknowledgement mode.
    pub mode: SessionAckMode,
    /// Total incoming segments received.
    pub incoming_segments: u64,
    /// Total outgoing segments sent.
    pub outgoing_segments: u64,
    /// Total retransmission requests received.
    pub retransmission_requests: u64,
    /// Total frames acknowledged by the peer.
    pub acknowledged_frames: u64,
}

/// Snapshot of SURB (Single Use Reply Block) metrics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SurbSnapshot {
    /// Total SURBs produced/minted.
    pub produced_total: u64,
    /// Total SURBs consumed/used.
    pub consumed_total: u64,
    /// Estimated number of SURBs currently available.
    pub buffer_estimate: u64,
    /// Target number of SURBs to maintain in buffer (if configured).
    pub target_buffer: Option<u64>,
    /// Rate of SURB consumption/production per second.
    pub rate_per_sec: f64,
    /// Whether a SURB refill request is currently in flight.
    pub refill_in_flight: bool,
}

/// Snapshot of transport-level data metrics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TransportSnapshot {
    /// Total bytes received.
    pub bytes_in: u64,
    /// Total bytes sent.
    pub bytes_out: u64,
    /// Total packets received.
    pub packets_in: u64,
    /// Total packets sent.
    pub packets_out: u64,
}

/// Complete snapshot of all session metrics at a point in time.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionStatsSnapshot {
    /// The ID of the session.
    pub session_id: SessionId,
    /// Time when this snapshot was taken.
    pub snapshot_at: SystemTime,
    /// Lifetime related metrics.
    pub lifetime: SessionLifetimeSnapshot,
    /// Frame buffer related metrics.
    pub frame_buffer: FrameBufferSnapshot,
    /// Acknowledgement related metrics.
    pub ack: AckSnapshot,
    /// SURB management related metrics.
    pub surb: SurbSnapshot,
    /// Transport level metrics (bytes/packets).
    pub transport: TransportSnapshot,
}

/// Internal metrics tracker for a session.
///
/// This struct uses atomic counters to allow lock-free updates from multiple threads/tasks.
#[derive(Debug)]
pub struct SessionStats {
    session_id: SessionId,
    created_at_us: AtomicU64,
    last_activity_us: AtomicU64,
    state: AtomicU8,
    ack_mode: SessionAckMode,
    frame_mtu: usize,
    frame_timeout: Duration,
    frame_capacity: usize,
    incomplete_frames: AtomicUsize,
    frames_completed: AtomicU64,
    frames_emitted: AtomicU64,
    frames_discarded: AtomicU64,
    incoming_segments: AtomicU64,
    outgoing_segments: AtomicU64,
    retransmission_requests: AtomicU64,
    acknowledged_frames: AtomicU64,
    bytes_in: AtomicU64,
    bytes_out: AtomicU64,
    packets_in: AtomicU64,
    packets_out: AtomicU64,
    surb_refill_in_flight: AtomicBool,
    /// Previous (buffer_estimate, timestamp_us) for rate calculation, protected by mutex
    /// to ensure atomic read/update of the pair.
    last_rate_snapshot: Mutex<(u64, u64)>,
    inspector: OnceLock<FrameInspector>,
    surb_estimator: OnceLock<AtomicSurbFlowEstimator>,
    surb_target_buffer: OnceLock<u64>,
}

impl SessionStats {
    pub fn new(
        session_id: SessionId,
        ack_mode: Option<AcknowledgementMode>,
        frame_mtu: usize,
        frame_timeout: Duration,
        frame_capacity: usize,
    ) -> Self {
        let now = now_us();
        Self {
            session_id,
            created_at_us: AtomicU64::new(now),
            last_activity_us: AtomicU64::new(now),
            state: AtomicU8::new(SessionLifecycleState::Active.as_u8()),
            ack_mode: SessionAckMode::from_ack_mode(ack_mode),
            frame_mtu,
            frame_timeout,
            frame_capacity,
            incomplete_frames: AtomicUsize::new(0),
            frames_completed: AtomicU64::new(0),
            frames_emitted: AtomicU64::new(0),
            frames_discarded: AtomicU64::new(0),
            incoming_segments: AtomicU64::new(0),
            outgoing_segments: AtomicU64::new(0),
            retransmission_requests: AtomicU64::new(0),
            acknowledged_frames: AtomicU64::new(0),
            bytes_in: AtomicU64::new(0),
            bytes_out: AtomicU64::new(0),
            packets_in: AtomicU64::new(0),
            packets_out: AtomicU64::new(0),
            surb_refill_in_flight: AtomicBool::new(false),
            last_rate_snapshot: Mutex::new((0, now)),
            inspector: OnceLock::new(),
            surb_estimator: OnceLock::new(),
            surb_target_buffer: OnceLock::new(),
        }
    }

    /// Returns a reference to the session ID.
    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    /// Updates the session lifecycle state.
    ///
    /// This method atomically stores the new state, which affects the metrics snapshot
    /// and indicates the session's current phase (Active, Closing, or Closed).
    pub fn set_state(&self, state: SessionLifecycleState) {
        self.state.store(state.as_u8(), Ordering::Relaxed);
    }

    /// Records activity on the session by updating the last activity timestamp.
    ///
    /// This is called on read/write operations to track session idleness.
    pub fn touch_activity(&self) {
        self.last_activity_us.store(now_us(), Ordering::Relaxed);
    }

    /// Records an incoming read operation with the specified number of bytes.
    ///
    /// Increments byte and packet counters, and updates activity timestamp.
    /// Zero-byte reads are ignored.
    pub fn record_read(&self, bytes: usize) {
        if bytes == 0 {
            return;
        }
        self.touch_activity();
        self.bytes_in.fetch_add(bytes as u64, Ordering::Relaxed);
        self.packets_in.fetch_add(1, Ordering::Relaxed);
    }

    /// Records an outgoing write operation with the specified number of bytes.
    ///
    /// Increments byte and packet counters, and updates activity timestamp.
    /// Zero-byte writes are ignored.
    pub fn record_write(&self, bytes: usize) {
        if bytes == 0 {
            return;
        }
        self.touch_activity();
        self.bytes_out.fetch_add(bytes as u64, Ordering::Relaxed);
        self.packets_out.fetch_add(1, Ordering::Relaxed);
    }

    /// Sets whether a SURB (Single Use Reply Block) refill request is currently in flight.
    pub fn set_refill_in_flight(&self, active: bool) {
        self.surb_refill_in_flight.store(active, Ordering::Relaxed);
    }

    /// Sets the frame inspector for tracking incomplete frames.
    ///
    /// The inspector is initialized only once via `OnceLock`.
    pub fn set_inspector(&self, inspector: FrameInspector) {
        let _ = self.inspector.set(inspector);
    }

    /// Sets the SURB flow estimator for tracking produced/consumed SURBs.
    ///
    /// The estimator and target buffer are initialized only once via `OnceLock`.
    pub fn set_surb_estimator(&self, estimator: AtomicSurbFlowEstimator, target_buffer: u64) {
        let _ = self.surb_estimator.set(estimator);
        let _ = self.surb_target_buffer.set(target_buffer);
    }

    /// Updates the count of incomplete frames from the frame inspector.
    fn record_incomplete_frames(&self) {
        if let Some(inspector) = self.inspector.get() {
            self.incomplete_frames.store(inspector.len(), Ordering::Relaxed);
        }
    }

    /// Records the receipt of an incoming segment.
    fn record_incoming_segment(&self) {
        self.incoming_segments.fetch_add(1, Ordering::Relaxed);
    }

    /// Records the transmission of an outgoing segment.
    fn record_outgoing_segment(&self) {
        self.outgoing_segments.fetch_add(1, Ordering::Relaxed);
    }

    /// Records retransmission requests with the specified count.
    fn record_retransmission_request(&self, count: usize) {
        self.retransmission_requests.fetch_add(count as u64, Ordering::Relaxed);
    }

    /// Records acknowledged frames with the specified count.
    fn record_acknowledged_frames(&self, count: usize) {
        self.acknowledged_frames.fetch_add(count as u64, Ordering::Relaxed);
    }

    /// Records a frame completion event.
    fn record_frame_complete(&self) {
        self.frames_completed.fetch_add(1, Ordering::Relaxed);
    }

    /// Records a frame emission event.
    fn record_frame_emitted(&self) {
        self.frames_emitted.fetch_add(1, Ordering::Relaxed);
    }

    /// Records a frame discard event.
    fn record_frame_discarded(&self) {
        self.frames_discarded.fetch_add(1, Ordering::Relaxed);
    }

    /// Creates a snapshot of all current metrics.
    ///
    /// This method atomically reads all metric counters and creates an immutable snapshot
    /// that includes lifetime, frame buffer, acknowledgement, SURB, and transport metrics.
    /// SURB metrics are loaded automatically from the stored estimator if one was set via
    /// [`set_surb_estimator`].
    pub fn snapshot(&self) -> SessionStatsSnapshot {
        self.record_incomplete_frames();

        let snapshot_at_us = now_us();
        let created_at_us = self.created_at_us.load(Ordering::Relaxed);
        let last_activity_us = self.last_activity_us.load(Ordering::Relaxed);
        let state = SessionLifecycleState::from_u8(self.state.load(Ordering::Relaxed));
        let uptime_us = snapshot_at_us.saturating_sub(created_at_us);
        let idle_us = snapshot_at_us.saturating_sub(last_activity_us);

        let (produced, consumed) = self
            .surb_estimator
            .get()
            .map(|e| (e.produced.load(Ordering::Relaxed), e.consumed.load(Ordering::Relaxed)))
            .unwrap_or((0, 0));

        let target = self.surb_target_buffer.get().copied();

        let buffer_estimate = produced.saturating_sub(consumed);
        let rate_per_sec = self.compute_rate_per_sec(produced, consumed, snapshot_at_us);

        SessionStatsSnapshot {
            session_id: self.session_id,
            snapshot_at: UNIX_EPOCH + Duration::from_micros(snapshot_at_us),
            lifetime: SessionLifetimeSnapshot {
                created_at: UNIX_EPOCH + Duration::from_micros(created_at_us),
                last_activity_at: UNIX_EPOCH + Duration::from_micros(last_activity_us),
                uptime: Duration::from_micros(uptime_us),
                idle: Duration::from_micros(idle_us),
                state,
            },
            frame_buffer: FrameBufferSnapshot {
                frame_mtu: self.frame_mtu,
                frame_timeout: self.frame_timeout,
                frame_capacity: self.frame_capacity,
                incomplete_frames: self.incomplete_frames.load(Ordering::Relaxed),
                frames_completed: self.frames_completed.load(Ordering::Relaxed),
                frames_emitted: self.frames_emitted.load(Ordering::Relaxed),
                frames_discarded: self.frames_discarded.load(Ordering::Relaxed),
            },
            ack: AckSnapshot {
                mode: self.ack_mode,
                incoming_segments: self.incoming_segments.load(Ordering::Relaxed),
                outgoing_segments: self.outgoing_segments.load(Ordering::Relaxed),
                retransmission_requests: self.retransmission_requests.load(Ordering::Relaxed),
                acknowledged_frames: self.acknowledged_frames.load(Ordering::Relaxed),
            },
            surb: SurbSnapshot {
                produced_total: produced,
                consumed_total: consumed,
                buffer_estimate,
                target_buffer: target,
                rate_per_sec,
                refill_in_flight: self.surb_refill_in_flight.load(Ordering::Relaxed),
            },
            transport: TransportSnapshot {
                bytes_in: self.bytes_in.load(Ordering::Relaxed),
                bytes_out: self.bytes_out.load(Ordering::Relaxed),
                packets_in: self.packets_in.load(Ordering::Relaxed),
                packets_out: self.packets_out.load(Ordering::Relaxed),
            },
        }
    }

    /// Computes the SURB buffer change rate in items per second.
    ///
    /// This uses a sliding window approach, tracking the delta since the last computation
    /// and the elapsed time to calculate the current rate.
    ///
    /// Returns:
    /// - Positive value: buffer is growing (production > consumption)
    /// - Negative value: buffer is depleting (consumption > production)
    /// - Zero: no change or no time has elapsed
    fn compute_rate_per_sec(&self, produced: u64, consumed: u64, now_us: u64) -> f64 {
        let total = produced.saturating_sub(consumed);

        // Atomically read and update the previous snapshot
        let (last_total, last_us) = {
            let mut snapshot = self.last_rate_snapshot.lock().unwrap_or_else(|e| e.into_inner());
            let prev = *snapshot;
            *snapshot = (total, now_us);
            prev
        };

        let elapsed_us = now_us.saturating_sub(last_us);
        if elapsed_us == 0 {
            return 0.0;
        }

        // Use signed arithmetic to capture buffer depletion as negative rates
        let delta = total as i64 - last_total as i64;
        (delta as f64) / (elapsed_us as f64 / 1_000_000.0)
    }
}

/// A wrapper that adds metrics tracking to a socket state implementation.
///
/// This struct wraps any `SocketState` implementation and intercepts lifecycle and data events
/// to record them in the associated `SessionStats`. It maintains a transparent interface
/// to the wrapped state while collecting comprehensive metrics.
#[derive(Clone)]
pub struct StatsState<S> {
    inner: S,
    metrics: Arc<SessionStats>,
}

impl<S> StatsState<S> {
    /// Creates a new metrics-tracking state wrapper.
    ///
    /// # Arguments
    ///
    /// * `inner` - The underlying socket state implementation
    /// * `metrics` - The metrics tracker shared across clones
    pub fn new(inner: S, metrics: Arc<SessionStats>) -> Self {
        Self { inner, metrics }
    }
}

impl<const C: usize, S: SocketState<C> + Clone> SocketState<C> for StatsState<S> {
    /// Delegates to the inner state's session ID.
    fn session_id(&self) -> &str {
        self.inner.session_id()
    }

    /// Initializes the socket with components, recording the frame inspector for metrics.
    fn run(&mut self, components: SocketComponents<C>) -> Result<(), hopr_protocol_session::errors::SessionError> {
        if let Some(inspector) = components.inspector.clone() {
            self.metrics.set_inspector(inspector);
        }
        self.inner.run(components)
    }

    /// Stops the session and records the state transition to Closing.
    fn stop(&mut self) -> Result<(), hopr_protocol_session::errors::SessionError> {
        self.metrics.set_state(SessionLifecycleState::Closing);
        self.inner.stop()
    }

    /// Records an incoming segment event and delegates to the inner state.
    fn incoming_segment(
        &mut self,
        id: &SegmentId,
        ind: SeqIndicator,
    ) -> Result<(), hopr_protocol_session::errors::SessionError> {
        self.metrics.record_incoming_segment();
        self.inner.incoming_segment(id, ind)
    }

    /// Records retransmission requests and delegates to the inner state.
    fn incoming_retransmission_request(
        &mut self,
        request: SegmentRequest<C>,
    ) -> Result<(), hopr_protocol_session::errors::SessionError> {
        self.metrics.record_retransmission_request(request.len());
        self.inner.incoming_retransmission_request(request)
    }

    /// Records acknowledged frames and delegates to the inner state.
    fn incoming_acknowledged_frames(
        &mut self,
        ack: FrameAcknowledgements<C>,
    ) -> Result<(), hopr_protocol_session::errors::SessionError> {
        self.metrics.record_acknowledged_frames(ack.len());
        self.inner.incoming_acknowledged_frames(ack)
    }

    /// Records a frame completion event and delegates to the inner state.
    fn frame_complete(&mut self, id: FrameId) -> Result<(), hopr_protocol_session::errors::SessionError> {
        self.metrics.record_frame_complete();
        self.inner.frame_complete(id)
    }

    /// Records a frame emission event and delegates to the inner state.
    fn frame_emitted(&mut self, id: FrameId) -> Result<(), hopr_protocol_session::errors::SessionError> {
        self.metrics.record_frame_emitted();
        self.inner.frame_emitted(id)
    }

    /// Records a frame discard event and delegates to the inner state.
    fn frame_discarded(&mut self, id: FrameId) -> Result<(), hopr_protocol_session::errors::SessionError> {
        self.metrics.record_frame_discarded();
        self.inner.frame_discarded(id)
    }

    /// Records an outgoing segment transmission and delegates to the inner state.
    fn segment_sent(&mut self, segment: &Segment) -> Result<(), hopr_protocol_session::errors::SessionError> {
        self.metrics.record_outgoing_segment();
        self.inner.segment_sent(segment)
    }
}

/// Returns the current time as microseconds since the Unix epoch.
fn now_us() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64
}

#[cfg(test)]
mod tests {
    use hopr_crypto_random::Randomizable;
    use hopr_internal_types::prelude::HoprPseudonym;

    use super::*;
    use crate::SessionId;

    #[test]
    fn metrics_snapshot_tracks_bytes_and_packets() {
        let id = SessionId::new(1_u64, HoprPseudonym::random());
        let metrics = SessionStats::new(id, None, 1500, Duration::from_millis(800), 1024);

        metrics.record_read(10);
        metrics.record_read(0);
        metrics.record_write(20);

        let snapshot = metrics.snapshot();

        assert_eq!(snapshot.transport.bytes_in, 10);
        assert_eq!(snapshot.transport.bytes_out, 20);
        assert_eq!(snapshot.transport.packets_in, 1);
        assert_eq!(snapshot.transport.packets_out, 1);
    }

    #[test]
    fn metrics_snapshot_tracks_frame_events() {
        let id = SessionId::new(2_u64, HoprPseudonym::random());
        let metrics = SessionStats::new(id, None, 1500, Duration::from_millis(800), 1024);

        metrics.record_frame_complete();
        metrics.record_frame_emitted();
        metrics.record_frame_discarded();

        let snapshot = metrics.snapshot();

        assert_eq!(snapshot.frame_buffer.frames_completed, 1);
        assert_eq!(snapshot.frame_buffer.frames_emitted, 1);
        assert_eq!(snapshot.frame_buffer.frames_discarded, 1);
    }
}
