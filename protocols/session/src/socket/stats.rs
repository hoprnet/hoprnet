use std::sync::atomic::{AtomicU64, Ordering};
use crate::{FrameAcknowledgements, FrameId, Segment, SegmentId, SegmentRequest, SeqIndicator, SocketComponents, SocketState};
use crate::errors::SessionError;

/// Various statistics for a [`SessionSocket`](crate::SessionSocket).
#[derive(Debug, Default)]
pub struct SessionSocketStats {
    incomplete_frames: AtomicU64,
    frames_completed: AtomicU64,
    frames_emitted: AtomicU64,
    frames_discarded: AtomicU64,
    incoming_segments: AtomicU64,
    outgoing_segments: AtomicU64,
    retransmission_requests: AtomicU64,
    acknowledged_frames: AtomicU64,
}

impl SessionSocketStats {
    pub fn incomplete_frames(&self) -> u64 {
        self.incomplete_frames.load(Ordering::Relaxed)
    }

    pub fn frames_completed(&self) -> u64 {
        self.frames_completed.load(Ordering::Relaxed)
    }

    pub fn frames_emitted(&self) -> u64 {
        self.frames_emitted.load(Ordering::Relaxed)
    }

    pub fn frames_discarded(&self) -> u64 {
        self.frames_discarded.load(Ordering::Relaxed)
    }

    pub fn incoming_segments(&self) -> u64 {
        self.incoming_segments.load(Ordering::Relaxed)
    }

    pub fn outgoing_segments(&self) -> u64 {
        self.outgoing_segments.load(Ordering::Relaxed)
    }

    pub fn retransmission_requests(&self) -> u64 {
        self.retransmission_requests.load(Ordering::Relaxed)
    }

    pub fn acknowledged_frames(&self) -> u64 {
        self.acknowledged_frames.load(Ordering::Relaxed)
    }
}

pub(crate) struct StatsStateWrapper<S>(pub(crate) S, pub(crate) std::sync::Arc<SessionSocketStats>);

impl<S: Clone> Clone for StatsStateWrapper<S> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

impl<const C: usize, S: SocketState<C>> SocketState<C> for StatsStateWrapper<S> {
    fn session_id(&self) -> &str {
        self.0.session_id()
    }

    fn run(&mut self, components: SocketComponents<C>) -> Result<(), SessionError> {
        self.0.run(components)
    }

    fn stop(&mut self) -> Result<(), SessionError> {
        self.0.stop()
    }

    fn incoming_segment(&mut self, id: &SegmentId, ind: SeqIndicator) -> Result<(), SessionError> {
        let r = self.0.incoming_segment(id, ind);
        self.1.incoming_segments.fetch_add(1, Ordering::Relaxed);
        r
    }

    fn incoming_retransmission_request(&mut self, request: SegmentRequest<C>) -> Result<(), SessionError> {
        let r = self.0.incoming_retransmission_request(request);
        self.1.retransmission_requests.fetch_add(1, Ordering::Relaxed);
        r
    }

    fn incoming_acknowledged_frames(&mut self, ack: FrameAcknowledgements<C>) -> Result<(), SessionError> {
        let r = self.0.incoming_acknowledged_frames(ack);
        self.1.acknowledged_frames.fetch_add(1, Ordering::Relaxed);
        r
    }

    fn frame_complete(&mut self, id: FrameId) -> Result<(), SessionError> {
        let r = self.0.frame_complete(id);
        self.1.frames_completed.fetch_add(1, Ordering::Relaxed);
        r
    }

    fn frame_emitted(&mut self, id: FrameId) -> Result<(), SessionError> {
        let r =  self.0.frame_emitted(id);
        self.1.frames_emitted.fetch_add(1, Ordering::Relaxed);
        r
    }

    fn frame_discarded(&mut self, id: FrameId) -> Result<(), SessionError> {
        let r = self.0.frame_discarded(id);
        self.1.frames_discarded.fetch_add(1, Ordering::Relaxed);
        r
    }

    fn segment_sent(&mut self, segment: &Segment) -> Result<(), SessionError> {
        let r = self.0.segment_sent(segment);
        self.1.outgoing_segments.fetch_add(1, Ordering::Relaxed);
        r
    }
}