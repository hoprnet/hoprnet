use crossbeam_skiplist::SkipMap;
use crate::frame::{FrameId, FrameInfo, FrameReassembler, SegmentId};
use crate::prelude::Segment;
use crate::session::protocol::SessionMessage;


pub struct SessionConfig {
    pub max_buffered_segments: usize
}

pub struct SessionState {
    lookbehind: SkipMap<SegmentId, Segment>,
    frame_reassembler: FrameReassembler,
    cfg: SessionConfig
}

impl SessionState {
    pub async fn received_packet(&self, data: &[u8]) -> crate::errors::Result<()>{
        match SessionMessage::try_from(data)? {
            SessionMessage::Segment(s) => self.frame_reassembler.push_segment(s)?,
            SessionMessage::Request(r) => {
                let frame_id = r.frame_id;
                for segment_id in r.missing_segments.into_ones().map(|seq_idx| SegmentId(frame_id, seq_idx as u16)) {
                    if let Some(segment) = self.lookbehind.get(&segment_id) {
                        self.send_segment(segment.value()).await?;
                    }
                }
            }
            SessionMessage::Acknowledge(f) => {
                for frame_id in f {
                    for seg in self.lookbehind.iter() {
                        if seg.key().0 == frame_id {
                            seg.remove();
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn add_and_send_segment(&self, segment: &Segment) -> crate::errors::Result<()> {
        self.lookbehind.insert(segment.into(), segment.clone());

        self.send_segment(segment).await?;

        // TODO: prevent stalling here
        while self.lookbehind.len() > self.cfg.max_buffered_segments {
            self.lookbehind.pop_front();
        }

        Ok(())
    }

    async fn send_segment_request(&self, frame_info: FrameInfo) -> crate::errors::Result<()> {
        let msg = SessionMessage::Request(frame_info.into());
        self.send_raw(&Vec::from(msg).into_boxed_slice()).await
    }

    async fn send_segment(&self, segment: &Segment) -> crate::errors::Result<()> {
        let msg = SessionMessage::Segment(segment.clone());
        self.send_raw(&Vec::from(msg).into_boxed_slice()).await
    }

    async fn send_acknowledgement(&self, frame_ids: Vec<FrameId>) -> crate::errors::Result<()> {
        let msg = SessionMessage::Acknowledge(frame_ids.into());
        self.send_raw(&Vec::from(msg).into_boxed_slice()).await
    }

    async fn send_raw(&self, data: &[u8]) -> crate::errors::Result<()> {
        todo!()
    }
}