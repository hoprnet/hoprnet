use async_trait::async_trait;
use crossbeam_queue::ArrayQueue;
use crossbeam_skiplist::SkipMap;
use futures::future::BoxFuture;
use futures::{AsyncRead, AsyncWrite, FutureExt, StreamExt, TryStreamExt};
use pin_project::pin_project;
use smart_default::SmartDefault;
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

/// Configuration of session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, SmartDefault)]
pub struct SessionConfig {
    /// Maximum number of buffered segments.
    #[default = 30000]
    pub max_buffered_segments: usize,
    /// Size of the buffer for acknowledged frame IDs.
    /// If set to 0, no frame acknowledgement will be sent.
    #[default = 0]
    pub acknowledged_frames_buffer: usize,
    /// Incomplete frames will be discarded after being in the reassembler
    /// this long.
    #[default(Duration::from_secs(30))]
    pub frame_expiration_age: Duration,
    /// Payload size available for the session sub-protocol.
    #[default = 466]
    pub mtu: u16,
}

/// Contains the state of the session bound to a [`SessionSocket`].
///
/// It implements the entire [`SessionMessage`] state machine and
/// performs the frame reassembly and sequencing.
/// The underlying transport operations are bound to [`NetworkTransport`]
///
/// The `SessionState` cannot be created directly, it must always be created via [`SessionSocket`].
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
    /// Should be called by the underlying transport when raw packet data are received.
    /// The `data` argument must contain a valid [`SessionMessage`], otherwise the method throws an error.
    pub async fn received_message(&self, data: &[u8]) -> crate::errors::Result<()> {
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

    /// Sends a requests for missing segments in incomplete frames.
    /// One [request](SessionMessage::Request) message is sent per incomplete frame. The message contains
    /// the segment indices missing from that frame. The `max_requests` argument can provide a maximum
    /// number of request messages sent by this call. If `max_requests` is `None`, request messages
    /// are set for all incomplete frames.
    /// Returns the number of sent request messages.
    pub async fn request_missing_segments(&self, max_requests: Option<usize>) -> crate::errors::Result<usize> {
        self.frame_reassembler.evict()?;

        let mut incomplete = self
            .frame_reassembler
            .incomplete_frames()
            .values()
            .cloned()
            .collect::<Vec<_>>();
        incomplete.sort_unstable_by(|a, b| a.last_update.cmp(&b.last_update));
        let max = max_requests.unwrap_or(incomplete.len());

        let mut sent = 0;
        for req in incomplete
            .into_iter()
            .take(max)
            .map(|info| SessionMessage::Request(info.into()))
        {
            self.transport.send_to_counterparty(&req.into_encoded()).await?;
            sent += 1;
        }

        Ok(sent)
    }

    /// Sends [acknowledgement](SessionMessage::Acknowledge) messages containing frames IDs
    /// of all frames that were successfully processed.
    /// If [`acknowledged_frames_buffer`](SessionConfig) was set to `0` during the construction,
    /// this method will do nothing and return `0`.
    /// Otherwise, it returns the number of acknowledged frames.
    /// If `acknowledged_frames_buffer` is non-zero, the buffer behaves like a ring buffer,
    /// which means if this method is not called sufficiently often, the oldest acknowledged
    /// frame IDs will be discarded.
    /// Single [message](SessionMessage::Acknowledge) can accommodate up to [`FrameAcknowledgements::MAX_ACK_FRAMES`] frame IDs, so
    /// this method sends as many messages as needed, or at most `max_message` if was given.
    pub async fn acknowledge_segments(&self, max_messages: Option<usize>) -> crate::errors::Result<usize> {
        if let Some(acked_buffer) = self.acknowledged.deref() {
            let mut len = 0;
            let mut msgs = 0;
            while !acked_buffer.is_empty() {
                let mut ack_frames = FrameAcknowledgements::default();

                if max_messages.map(|max| msgs < max).unwrap_or(true) {
                    while !ack_frames.is_full() {
                        if let Some(ack_id) = acked_buffer.pop() {
                            ack_frames.push(ack_id);
                            len += 1;
                        }
                    }

                    self.transport
                        .send_to_counterparty(&SessionMessage::Acknowledge(ack_frames).into_encoded())
                        .await?;
                    msgs += 1;
                } else {
                    // Break out if we sent max allowed
                    return Ok(len);
                }
            }

            Ok(len)
        } else {
            Ok(0)
        }
    }

    /// Segments the `data` and sends them as (possibly multiple) [`SessionMessage::Segment`].
    /// Therefore, this method sends as many messages as needed after the data was segmented.
    /// Each segment is inserted into the lookbehind ring buffer for possible retransmissions.
    ///
    /// The size of the lookbehind ring buffer is given by the [`max_buffered_segments`](SessionConfig)
    /// given during the construction. It needs to accommodate as many segments as
    /// is the expected underlying transport bandwidth (segment/sec) to guarantee the retransmission
    /// can still happen within some time window.
    pub async fn send_frame_data(&self, data: &[u8]) -> crate::errors::Result<()> {
        for segment in segment(
            data,
            self.cfg.mtu - SessionMessage::HEADER_SIZE as u16,
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

/// Represents a socket for a session between two nodes bound by the
/// underlying [`NetworkTransport`].
///
/// It also implements [`AsyncRead`] and [`AsyncWrite`] so that it can
/// be used on top of the usual transport stack.
#[pin_project]
pub struct SessionSocket<T: NetworkTransport> {
    state: SessionState<T>,
    #[pin]
    egress: Box<dyn AsyncRead + Send + Unpin>,
}

impl<T: NetworkTransport> SessionSocket<T> {
    /// Create a new socket over the given `transport` that binds the communicating parties.
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

        let egress = Box::new(
            egress
                .inspect(move |frame| {
                    if let Some(ack_buf) = acknowledged.deref() {
                        // Act as a ring buffer, so the buffer is full, any unsent acknowledgements
                        // will be discarded.
                        ack_buf.force_push(frame.frame_id);
                    }
                })
                .map(Ok)
                .into_async_read(),
        );

        Self { state, egress }
    }

    /// The [state](SessionState) of this socket.
    pub fn state(&self) -> &SessionState<T> {
        &self.state
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

#[cfg(test)]
mod tests {}
