use futures::stream::BoxStream;
use crate::prelude::errors::SessionError;
use crate::prelude::protocol::{FrameAcknowledgements, SegmentRequest, SessionMessage};
use crate::prelude::{FrameId, Segment, SegmentId};
use crate::session::frame::SeqNum;
use crate::session::socket::SocketState;

#[derive(Clone, Copy)]
pub struct Stateless<const C: usize>;

impl<'a, const C: usize> SocketState<'a, C> for Stateless<C> {
    fn incoming_segment(&mut self, _id: &SegmentId, _segment_count: SeqNum) -> Result<(), SessionError> {
        Ok(())
    }

    fn incoming_retransmission_request(&mut self, _request: SegmentRequest<C>) -> Result<(), SessionError> {
        Ok(())
    }

    fn incoming_acknowledged_frames(&mut self, _ack: FrameAcknowledgements<C>) -> Result<(), SessionError> {
        Ok(())
    }

    fn frame_received(&mut self, _id: FrameId) -> Result<(), SessionError> {
        Ok(())
    }

    fn frame_discarded(&mut self, _id: FrameId) -> Result<(), SessionError> {
        Ok(())
    }

    fn segment_sent(&mut self, segment: &Segment) -> Result<(), SessionError> {
        Ok(())
    }

    fn control_message_stream(&self) -> Option<BoxStream<'a, SessionMessage<C>>> {
        None
    }
}

pub struct AcknowledgementState<const C: usize> {

}

impl<const C: usize> AcknowledgementState<C> {
    pub fn new() -> Self {
        Self {}
    }
}

impl<const C: usize> Clone for AcknowledgementState<C> {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl<'a, const C: usize> SocketState<'a, C> for AcknowledgementState<C> {
    fn incoming_segment(&mut self, id: &SegmentId, segment_count: SeqNum) -> Result<(), SessionError> {
        todo!()
    }

    fn incoming_retransmission_request(&mut self, request: SegmentRequest<C>) -> Result<(), SessionError> {
        todo!()
    }

    fn incoming_acknowledged_frames(&mut self, ack: FrameAcknowledgements<C>) -> Result<(), SessionError> {
        todo!()
    }

    fn frame_received(&mut self, id: FrameId) -> Result<(), SessionError> {
        /*
        // TODO: ack_tx gets dropped when downstream_frames_out is dropped?
        if let Err(error) = ack_tx.try_send(frame.frame_id) {
            tracing::error!(session_id = id, frame_id = frame.frame_id, %error, "failed to acknowledge frame");
        }
        if let Err(error) = incoming_frame_retries_tx_clone.send_one((RetriedFrameId::new(frame.frame_id), Skip)) {
            tracing::error!(session_id = id, frame_id = frame.frame_id, %error, "failed to cancel retry of acknowledged frame");
        }
         */
        todo!()
    }

    fn frame_discarded(&mut self, id: FrameId) -> Result<(), SessionError> {
        /*
        if let Err(error) = incoming_frame_retries_tx_clone.send_one((RetriedFrameId::new(frame_id), Skip)) {
            tracing::error!(session_id = id, frame_id, %error, "failed to cancel retry of acknowledged frame");
        }
         */
        todo!()
    }

    fn segment_sent(&mut self, segment: &Segment) -> Result<(), SessionError> {
        /*
        rb_tx.push(s.clone());
        // When the last segment of a frame has been sent,
        // add it to outgoing retries
        if s.is_last() {
            // TODO: retry token period should be set dynamic based on s.seq_len
            let first_retry = Duration::from_secs(1);
            if let Err(error) = outgoing_frame_retries_tx_clone.send_one((RetriedFrameId::new(s.frame_id), first_retry)) {
                tracing::trace!(frame_id = s.frame_id, %error, "failed to insert outgoing retry of a frame");
            }
        }
         */
        todo!()
    }

    fn control_message_stream(&self) -> Option<BoxStream<'a, SessionMessage<C>>> {
        todo!()
    }
}