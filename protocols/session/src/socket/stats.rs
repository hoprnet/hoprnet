use std::sync::atomic::{AtomicU64, Ordering};
use crate::protocol::SessionMessage;

/// Various statistics for a [`SessionSocket`](crate::SessionSocket).
#[derive(Debug, Default)]
pub struct SessionSocketStats {
    errors: AtomicU64,
    incomplete_frames: AtomicU64,
    frames_completed: AtomicU64,
    frames_emitted: AtomicU64,
    frames_discarded: AtomicU64,
    incoming_segments: AtomicU64,
    incoming_retransmission_requests: AtomicU64,
    incoming_acknowledged_frames: AtomicU64,
    outgoing_segments: AtomicU64,
    outgoing_retransmission_requests: AtomicU64,
    outgoing_acknowledged_frames: AtomicU64,
}

impl SessionSocketStats {
    pub(crate) fn inc_errors(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn inc_incomplete_frames(&self) {
        self.incomplete_frames.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn inc_frames_completed(&self) {
        self.frames_completed.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn inc_frames_emitted(&self) {
        self.frames_emitted.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn inc_frames_discarded(&self) {
        self.frames_discarded.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn inc_outgoing_segments(&self) {
        self.outgoing_segments.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn inc_incoming_session_message<const C: usize>(&self, msg: &SessionMessage<C>) {
        match msg {
            SessionMessage::Segment(_) => self.incoming_segments.fetch_add(1, Ordering::Relaxed),
            SessionMessage::Request(_) => self.incoming_retransmission_requests.fetch_add(1, Ordering::Relaxed),
            SessionMessage::Acknowledge(f) => self.incoming_acknowledged_frames.fetch_add(f.len() as u64, Ordering::Relaxed),
        };
    }

    pub(crate) fn inc_outgoing_session_message<const C: usize>(&self, msg: &SessionMessage<C>) {
        match msg {
            SessionMessage::Segment(_) => self.outgoing_segments.fetch_add(1, Ordering::Relaxed),
            SessionMessage::Request(_) => self.outgoing_retransmission_requests.fetch_add(1, Ordering::Relaxed),
            SessionMessage::Acknowledge(f) => self.outgoing_acknowledged_frames.fetch_add(f.len() as u64, Ordering::Relaxed),
        };
    }

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

    pub fn incoming_retransmission_requests(&self) -> u64 {
        self.incoming_retransmission_requests.load(Ordering::Relaxed)
    }
    pub fn incoming_acknowledged_frames(&self) -> u64 {
        self.incoming_acknowledged_frames.load(Ordering::Relaxed)
    }

    pub fn outgoing_retransmission_requests(&self) -> u64 {
        self.outgoing_retransmission_requests.load(Ordering::Relaxed)
    }

    pub fn outgoing_acknowledged_frames(&self) -> u64 {
        self.outgoing_acknowledged_frames.load(Ordering::Relaxed)
    }
}
