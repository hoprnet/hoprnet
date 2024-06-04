use async_trait::async_trait;
use crossbeam_queue::ArrayQueue;
use crossbeam_skiplist::SkipMap;
use futures::future::BoxFuture;
use futures::{AsyncRead, AsyncWrite, FutureExt, StreamExt, TryStreamExt};
use pin_project::pin_project;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use crate::errors::NetworkTypeError;
use crate::frame::{segment, FrameId, FrameReassembler, SegmentId};
use crate::prelude::Segment;
use crate::session::protocol::{FrameAcknowledgements, SessionMessage};

#[async_trait]
pub trait NetworkTransport {
    async fn send_to_counterparty(&self, data: &[u8]) -> crate::errors::Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionConfig {
    pub max_buffered_segments: usize,
    pub acknowledged_frames_buffer: usize,
    pub frame_expiration_age: Duration,
    pub mtu: u16,
}

#[derive(Debug)]
pub struct SessionState<T: NetworkTransport> {
    transport: Arc<T>,
    lookbehind: Arc<SkipMap<SegmentId, Segment>>,
    acknowledged: Arc<Option<ArrayQueue<FrameId>>>,
    outgoing_frame_id: Arc<AtomicU32>,
    frame_reassembler: Arc<FrameReassembler>,
    cfg: SessionConfig,
}

impl<T: NetworkTransport> SessionState<T> {
    pub async fn received_packet(&self, data: &[u8]) -> crate::errors::Result<()> {
        match SessionMessage::try_from(data)? {
            SessionMessage::Segment(s) => self.frame_reassembler.push_segment(s)?,
            SessionMessage::Request(r) => {
                let frame_id = r.frame_id;
                for segment_id in r
                    .missing_segments
                    .into_ones()
                    .map(|seq_idx| SegmentId(frame_id, seq_idx as u16))
                {
                    if let Some(segment) = self.lookbehind.get(&segment_id) {
                        let msg = SessionMessage::Segment(segment.value().clone());
                        self.transport.send_to_counterparty(&msg.into_encoded()).await?;
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

    pub async fn request_segments(&self, max_requests: Option<usize>) -> crate::errors::Result<()> {
        self.frame_reassembler.evict()?;

        let mut incomplete = self
            .frame_reassembler
            .incomplete_frames()
            .values()
            .cloned()
            .collect::<Vec<_>>();
        incomplete.sort_unstable_by(|a, b| a.last_update.cmp(&b.last_update));
        let max = max_requests.unwrap_or(incomplete.len());

        for req in incomplete
            .into_iter()
            .take(max)
            .map(|info| SessionMessage::Request(info.into()))
        {
            self.transport.send_to_counterparty(&req.into_encoded()).await?;
        }

        Ok(())
    }

    pub async fn acknowledge_segments(&self) -> crate::errors::Result<usize> {
        let mut ack_frames = FrameAcknowledgements::default();
        if let Some(acked_buffer) = self.acknowledged.deref() {
            let mut len = 0;
            while !ack_frames.is_full() && !acked_buffer.is_empty() {
                ack_frames.push(acked_buffer.pop().unwrap());
                len += 1;
            }

            self.transport
                .send_to_counterparty(&SessionMessage::Acknowledge(ack_frames).into_encoded())
                .await?;
            Ok(len)
        } else {
            Ok(0)
        }
    }

    pub async fn send_frame_data(&self, data: &[u8]) -> crate::errors::Result<()> {
        for segment in segment(
            data,
            self.cfg.mtu,
            self.outgoing_frame_id.fetch_add(1, Ordering::SeqCst),
        ) {
            self.lookbehind.insert((&segment).into(), segment.clone());

            let msg = SessionMessage::Segment(segment.clone());
            self.transport.send_to_counterparty(&msg.into_encoded()).await?;

            // TODO: prevent stalling here
            while self.lookbehind.len() > self.cfg.max_buffered_segments {
                self.lookbehind.pop_front();
            }
        }

        Ok(())
    }
}

impl<T: NetworkTransport> Clone for SessionState<T> {
    fn clone(&self) -> Self {
        Self {
            transport: self.transport.clone(),
            lookbehind: self.lookbehind.clone(),
            acknowledged: self.acknowledged.clone(),
            outgoing_frame_id: self.outgoing_frame_id.clone(),
            frame_reassembler: self.frame_reassembler.clone(),
            cfg: self.cfg.clone(),
        }
    }
}

#[pin_project]
pub struct SessionSocket<T: NetworkTransport> {
    state: SessionState<T>,
    #[pin]
    egress: Box<dyn AsyncRead + Send + Unpin>,
}

impl<T: NetworkTransport> SessionSocket<T> {
    pub fn new(transport: T, cfg: SessionConfig) -> Self {
        let (reassembler, egress) = FrameReassembler::new(cfg.frame_expiration_age.into());
        let acknowledged =
            Arc::new((cfg.acknowledged_frames_buffer > 0).then(|| ArrayQueue::new(cfg.acknowledged_frames_buffer)));
        let state = SessionState {
            transport: Arc::new(transport),
            lookbehind: Arc::new(SkipMap::new()),
            acknowledged: acknowledged.clone(),
            outgoing_frame_id: Arc::new(AtomicU32::new(1)),
            frame_reassembler: Arc::new(reassembler),
            cfg,
        };

        let egress = egress
            .inspect(move |frame| {
                if let Some(ack_buf) = acknowledged.deref() {
                    let _ = ack_buf.push(frame.frame_id);
                }
            })
            .map(Ok)
            .into_async_read();

        Self {
            state,
            egress: Box::new(egress),
        }
    }
}

#[pin_project]
struct SocketWriter<'a, T: NetworkTransport + Send + Sync> {
    state: &'a SessionState<T>,
    #[pin]
    future: BoxFuture<'a, Result<(), NetworkTypeError>>,
}

impl<'a, T: NetworkTransport + Send + Sync> SocketWriter<'a, T> {
    fn new(state: &'a SessionState<T>, data: &'a [u8]) -> Self {
        Self {
            state,
            future: state.send_frame_data(data).boxed(),
        }
    }
}

impl<'a, T: NetworkTransport + Send + Sync> Future for SocketWriter<'a, T> {
    type Output = Result<(), NetworkTypeError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.future.poll(cx)
    }
}

impl<T: NetworkTransport + Send + Sync> AsyncWrite for SessionSocket<T> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        let mut socket_future = SocketWriter::new(&self.state, buf);
        match Pin::new(&mut socket_future).poll(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(buf.len())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, e))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

impl<T: NetworkTransport + Send + Sync> AsyncRead for SessionSocket<T> {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        self.project().egress.poll_read(cx, buf)
    }
}
