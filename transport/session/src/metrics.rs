use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, AtomicU8, AtomicU64, AtomicUsize, Ordering},
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hopr_protocol_session::{
    AcknowledgementMode, FrameAcknowledgements, FrameId, FrameInspector, Segment, SegmentId, SegmentRequest,
    SeqIndicator, SocketComponents, SocketState,
};

use crate::SessionId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionLifecycleState {
    Active,
    Closing,
    Closed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionAckMode {
    None,
    Partial,
    Full,
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

#[derive(Debug, Clone)]
pub struct SessionLifetimeSnapshot {
    pub created_at_ms: u64,
    pub last_activity_at_ms: u64,
    pub uptime_ms: u64,
    pub idle_ms: u64,
    pub state: SessionLifecycleState,
}

#[derive(Debug, Clone)]
pub struct FrameBufferSnapshot {
    pub frame_mtu: usize,
    pub frame_timeout_ms: u64,
    pub frame_capacity: usize,
    pub incomplete_frames: usize,
    pub frames_completed: u64,
    pub frames_emitted: u64,
    pub frames_discarded: u64,
}

#[derive(Debug, Clone)]
pub struct AckSnapshot {
    pub mode: SessionAckMode,
    pub incoming_segments: u64,
    pub outgoing_segments: u64,
    pub retransmission_requests: u64,
    pub acknowledged_frames: u64,
}

#[derive(Debug, Clone)]
pub struct SurbSnapshot {
    pub produced_total: u64,
    pub consumed_total: u64,
    pub buffer_estimate: u64,
    pub target_buffer: Option<u64>,
    pub rate_per_sec: f64,
    pub refill_in_flight: bool,
}

#[derive(Debug, Clone)]
pub struct TransportSnapshot {
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub packets_in: u64,
    pub packets_out: u64,
}

#[derive(Debug, Clone)]
pub struct SessionMetricsSnapshot {
    pub session_id: SessionId,
    pub snapshot_at_ms: u64,
    pub lifetime: SessionLifetimeSnapshot,
    pub frame_buffer: FrameBufferSnapshot,
    pub ack: AckSnapshot,
    pub surb: SurbSnapshot,
    pub transport: TransportSnapshot,
}

#[derive(Debug)]
pub struct SessionMetrics {
    session_id: SessionId,
    created_at_ms: AtomicU64,
    last_activity_ms: AtomicU64,
    state: AtomicU8,
    ack_mode: SessionAckMode,
    frame_mtu: usize,
    frame_timeout_ms: u64,
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
    last_rate_total: AtomicU64,
    last_rate_ms: AtomicU64,
    inspector: OnceLock<FrameInspector>,
}

impl SessionMetrics {
    pub fn new(
        session_id: SessionId,
        ack_mode: Option<AcknowledgementMode>,
        frame_mtu: usize,
        frame_timeout: Duration,
        frame_capacity: usize,
    ) -> Self {
        let now = now_ms();
        Self {
            session_id,
            created_at_ms: AtomicU64::new(now),
            last_activity_ms: AtomicU64::new(now),
            state: AtomicU8::new(SessionLifecycleState::Active.as_u8()),
            ack_mode: SessionAckMode::from_ack_mode(ack_mode),
            frame_mtu,
            frame_timeout_ms: frame_timeout.as_millis() as u64,
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
            last_rate_total: AtomicU64::new(0),
            last_rate_ms: AtomicU64::new(now),
            inspector: OnceLock::new(),
        }
    }

    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    pub fn set_state(&self, state: SessionLifecycleState) {
        self.state.store(state.as_u8(), Ordering::Relaxed);
    }

    pub fn touch_activity(&self) {
        self.last_activity_ms.store(now_ms(), Ordering::Relaxed);
    }

    pub fn record_read(&self, bytes: usize) {
        if bytes == 0 {
            return;
        }
        self.touch_activity();
        self.bytes_in.fetch_add(bytes as u64, Ordering::Relaxed);
        self.packets_in.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_write(&self, bytes: usize) {
        if bytes == 0 {
            return;
        }
        self.touch_activity();
        self.bytes_out.fetch_add(bytes as u64, Ordering::Relaxed);
        self.packets_out.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_refill_in_flight(&self, active: bool) {
        self.surb_refill_in_flight.store(active, Ordering::Relaxed);
    }

    pub fn set_inspector(&self, inspector: FrameInspector) {
        let _ = self.inspector.set(inspector);
    }

    fn record_incomplete_frames(&self) {
        if let Some(inspector) = self.inspector.get() {
            self.incomplete_frames.store(inspector.len(), Ordering::Relaxed);
        }
    }

    fn record_incoming_segment(&self) {
        self.incoming_segments.fetch_add(1, Ordering::Relaxed);
    }

    fn record_outgoing_segment(&self) {
        self.outgoing_segments.fetch_add(1, Ordering::Relaxed);
    }

    fn record_retransmission_request(&self, count: usize) {
        self.retransmission_requests.fetch_add(count as u64, Ordering::Relaxed);
    }

    fn record_acknowledged_frames(&self, count: usize) {
        self.acknowledged_frames.fetch_add(count as u64, Ordering::Relaxed);
    }

    fn record_frame_complete(&self) {
        self.frames_completed.fetch_add(1, Ordering::Relaxed);
    }

    fn record_frame_emitted(&self) {
        self.frames_emitted.fetch_add(1, Ordering::Relaxed);
    }

    fn record_frame_discarded(&self) {
        self.frames_discarded.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self, produced: u64, consumed: u64, target: Option<u64>) -> SessionMetricsSnapshot {
        self.record_incomplete_frames();

        let snapshot_at_ms = now_ms();
        let created_at_ms = self.created_at_ms.load(Ordering::Relaxed);
        let last_activity_ms = self.last_activity_ms.load(Ordering::Relaxed);
        let state = SessionLifecycleState::from_u8(self.state.load(Ordering::Relaxed));
        let uptime_ms = snapshot_at_ms.saturating_sub(created_at_ms);
        let idle_ms = snapshot_at_ms.saturating_sub(last_activity_ms);

        let buffer_estimate = produced.saturating_sub(consumed);
        let rate_per_sec = self.compute_rate_per_sec(produced, consumed, snapshot_at_ms);

        SessionMetricsSnapshot {
            session_id: self.session_id,
            snapshot_at_ms,
            lifetime: SessionLifetimeSnapshot {
                created_at_ms,
                last_activity_at_ms: last_activity_ms,
                uptime_ms,
                idle_ms,
                state,
            },
            frame_buffer: FrameBufferSnapshot {
                frame_mtu: self.frame_mtu,
                frame_timeout_ms: self.frame_timeout_ms,
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

    fn compute_rate_per_sec(&self, produced: u64, consumed: u64, now_ms: u64) -> f64 {
        let total = produced.saturating_sub(consumed);
        let last_total = self.last_rate_total.swap(total, Ordering::Relaxed);
        let last_ms = self.last_rate_ms.swap(now_ms, Ordering::Relaxed);
        let delta = total.saturating_sub(last_total);
        let elapsed_ms = now_ms.saturating_sub(last_ms);
        if elapsed_ms == 0 {
            return 0.0;
        }
        (delta as f64) / (elapsed_ms as f64 / 1000.0)
    }
}

#[derive(Clone)]
pub struct MetricsState<S> {
    inner: S,
    metrics: Arc<SessionMetrics>,
}

impl<S> MetricsState<S> {
    pub fn new(inner: S, metrics: Arc<SessionMetrics>) -> Self {
        Self { inner, metrics }
    }
}

impl<const C: usize, S: SocketState<C> + Clone> SocketState<C> for MetricsState<S> {
    fn session_id(&self) -> &str {
        self.inner.session_id()
    }

    fn run(&mut self, components: SocketComponents<C>) -> Result<(), hopr_protocol_session::errors::SessionError> {
        if let Some(inspector) = components.inspector.clone() {
            self.metrics.set_inspector(inspector);
        }
        self.inner.run(components)
    }

    fn stop(&mut self) -> Result<(), hopr_protocol_session::errors::SessionError> {
        self.metrics.set_state(SessionLifecycleState::Closing);
        self.inner.stop()
    }

    fn incoming_segment(
        &mut self,
        id: &SegmentId,
        ind: SeqIndicator,
    ) -> Result<(), hopr_protocol_session::errors::SessionError> {
        self.metrics.record_incoming_segment();
        self.inner.incoming_segment(id, ind)
    }

    fn incoming_retransmission_request(
        &mut self,
        request: SegmentRequest<C>,
    ) -> Result<(), hopr_protocol_session::errors::SessionError> {
        self.metrics.record_retransmission_request(request.len());
        self.inner.incoming_retransmission_request(request)
    }

    fn incoming_acknowledged_frames(
        &mut self,
        ack: FrameAcknowledgements<C>,
    ) -> Result<(), hopr_protocol_session::errors::SessionError> {
        self.metrics.record_acknowledged_frames(ack.len());
        self.inner.incoming_acknowledged_frames(ack)
    }

    fn frame_complete(&mut self, id: FrameId) -> Result<(), hopr_protocol_session::errors::SessionError> {
        self.metrics.record_frame_complete();
        self.inner.frame_complete(id)
    }

    fn frame_emitted(&mut self, id: FrameId) -> Result<(), hopr_protocol_session::errors::SessionError> {
        self.metrics.record_frame_emitted();
        self.inner.frame_emitted(id)
    }

    fn frame_discarded(&mut self, id: FrameId) -> Result<(), hopr_protocol_session::errors::SessionError> {
        self.metrics.record_frame_discarded();
        self.inner.frame_discarded(id)
    }

    fn segment_sent(&mut self, segment: &Segment) -> Result<(), hopr_protocol_session::errors::SessionError> {
        self.metrics.record_outgoing_segment();
        self.inner.segment_sent(segment)
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
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
        let metrics = SessionMetrics::new(id, None, 1500, Duration::from_millis(800), 1024);

        metrics.record_read(10);
        metrics.record_read(0);
        metrics.record_write(20);

        let snapshot = metrics.snapshot(0, 0, None);

        assert_eq!(snapshot.transport.bytes_in, 10);
        assert_eq!(snapshot.transport.bytes_out, 20);
        assert_eq!(snapshot.transport.packets_in, 1);
        assert_eq!(snapshot.transport.packets_out, 1);
    }

    #[test]
    fn metrics_snapshot_tracks_frame_events() {
        let id = SessionId::new(2_u64, HoprPseudonym::random());
        let metrics = SessionMetrics::new(id, None, 1500, Duration::from_millis(800), 1024);

        metrics.record_frame_complete();
        metrics.record_frame_emitted();
        metrics.record_frame_discarded();

        let snapshot = metrics.snapshot(0, 0, None);

        assert_eq!(snapshot.frame_buffer.frames_completed, 1);
        assert_eq!(snapshot.frame_buffer.frames_emitted, 1);
        assert_eq!(snapshot.frame_buffer.frames_discarded, 1);
    }
}
