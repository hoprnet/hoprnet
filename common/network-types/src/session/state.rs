use crossbeam_queue::ArrayQueue;
use crossbeam_skiplist::SkipMap;
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use futures::channel::mpsc::UnboundedSender;
use futures::{AsyncRead, AsyncWrite, AsyncWriteExt, FutureExt, Sink, SinkExt, StreamExt, TryStreamExt};
use governor::prelude::StreamRateLimitExt;
use governor::{Jitter, Quota, RateLimiter};
use pin_project::pin_project;
use smart_default::SmartDefault;
use std::collections::BTreeSet;
use std::fmt::{Debug, Display};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tracing::{debug, warn};

use crate::errors::NetworkTypeError;
use crate::frame::{segment, FrameId, FrameReassembler, SegmentId};
use crate::prelude::Segment;
use crate::session::errors::SessionError;
use crate::session::protocol::{FrameAcknowledgements, SegmentRequest, SessionMessage};
use crate::session::utils::{RetryResult, RetryToken};

#[cfg(feature = "runtime-async-std")]
use async_std::task::spawn;

#[cfg(feature = "runtime-tokio")]
use tokio::task::spawn;

/// Configuration of session.
#[derive(Debug, Clone, Copy, PartialEq, SmartDefault)]
pub struct SessionConfig {
    /// Maximum number of buffered segments.
    /// This value should be large enough to accommodate segments for an at least
    /// `frame_expiration_age` period, given the expected maximum bandwidth.
    #[default = 50_000]
    pub max_buffered_segments: usize,

    /// Size of the buffer for acknowledged frame IDs.
    /// This value should be large enough, so that the buffer can accommodate segments
    /// for an at least `frame_expiration_age` period, given the expected maximum bandwidth.
    /// If set to 0, no frame acknowledgement will be sent.
    #[default = 1024]
    pub acknowledged_frames_buffer: usize,

    /// Specifies the maximum period a frame should be kept by the sender and
    /// asked for retransmission by the recipient.
    #[default(Duration::from_secs(30))]
    pub frame_expiration_age: Duration,

    /// If a frame is incomplete (on the receiver), retransmission requests will be made
    /// with exponential backoff starting at this initial retry timeout (RTO).
    /// Requests will be sent until `frame_expiration_age` is reached.
    /// NOTE that this value should be offset from `rto_base_sender`, so that the receiver's
    /// retransmission requests are interleaved with sender's retransmissions.
    #[default(Duration::from_millis(1000))]
    pub rto_base_receiver: Duration,

    /// If a frame is unacknowledged (on the sender), entire frame retransmissions will be made
    /// with exponential backoff starting at this initial retry timeout (RTO).
    /// Frames will be retransmitted until `frame_expiration_age` is reached.
    /// NOTE that this value should be offset from `rto_base_receiver`, so that the receiver's
    /// retransmission requests are interleaved with sender's retransmissions.
    #[default(Duration::from_millis(1500))]
    pub rto_base_sender: Duration,

    /// Base for the exponential backoff on retries.
    #[default(2.0)]
    pub backoff_base: f64,

    /// Optional rate limiting of egress messages per second.
    /// This will force the [SessionSocket] not to pass more than this quota of messages
    /// to the underlying transport.
    #[default(None)]
    pub max_msg_per_sec: Option<usize>,
}

/// Contains the cloneable state of the session bound to a [`SessionSocket`].
///
/// It implements the entire [`SessionMessage`] state machine and
/// performs the frame reassembly and sequencing.
/// The MTU size is specified by `C`.
///
/// The `SessionState` cannot be created directly, it must always be created via [`SessionSocket`] and
/// then retrieved via [`SessionSocket::state`].
#[pin_project]
#[derive(Debug, Clone)]
pub struct SessionState<const C: usize> {
    session_id: String,
    lookbehind: Arc<SkipMap<SegmentId, Segment>>,
    to_acknowledge: Arc<ArrayQueue<FrameId>>,
    incoming_frame_retries: Arc<DashMap<FrameId, RetryToken>>,
    outgoing_frame_resends: Arc<DashMap<FrameId, RetryToken>>,
    outgoing_frame_id: Arc<AtomicU32>,
    frame_reassembler: Arc<FrameReassembler>,
    cfg: SessionConfig,
    #[pin]
    segment_ingress: UnboundedSender<SessionMessage<C>>,
}

impl<const C: usize> SessionState<C> {
    fn consume_segment(&mut self, segment: Segment) {
        let id = segment.id();

        match self.frame_reassembler.push_segment(segment) {
            Ok(_) => {
                debug!("{:?}: RECEIVED: segment {id:?}", self.session_id);
                match self.incoming_frame_retries.entry(id.0) {
                    Entry::Occupied(e) => {
                        // Restart the retry token for this frame
                        e.replace_entry(RetryToken::new(Instant::now(), self.cfg.backoff_base));
                    }
                    Entry::Vacant(v) => {
                        // Create the retry token for this frame
                        v.insert(RetryToken::new(Instant::now(), self.cfg.backoff_base));
                    }
                }
            }
            Err(e) => warn!("{:?}: segment {id:?} not pushed: {e}", self.session_id),
        }
    }

    async fn retransmit_segments(&mut self, request: SegmentRequest<C>) -> crate::errors::Result<()> {
        debug!("{:?} RECEIVED: request for {} segments", self.session_id, request.len());

        let mut count = 0;
        for segment_id in request {
            // No need to retry this frame ourselves, since the other side will request on its own
            self.outgoing_frame_resends.remove(&segment_id.0);

            if let Some(segment) = self.lookbehind.get(&segment_id) {
                let msg = SessionMessage::<C>::Segment(segment.value().clone());
                debug!(
                    "{:?} SENDING: retransmitted segment: {:?}",
                    self.session_id,
                    segment.value().id()
                );
                self.segment_ingress
                    .feed(msg)
                    .await
                    .map_err(|e| SessionError::ProcessingError(e.to_string()))?;
                count += 1;
            } else {
                warn!(
                    "{:?}: segment {segment_id:?} not in lookbehind buffer anymore",
                    self.session_id
                );
            }
        }
        self.segment_ingress
            .flush()
            .await
            .map_err(|e| SessionError::ProcessingError(e.to_string()))?;
        debug!("{:?}: retransmitted {count} requested segments", self.session_id);
        Ok(())
    }

    fn acknowledged_frames(&mut self, acked: FrameAcknowledgements<C>) {
        debug!(
            "{:?} RECEIVED: acknowledgement of {} frames",
            self.session_id,
            acked.len()
        );

        for frame_id in acked {
            // Frame acknowledged, we won't need to resend it
            self.outgoing_frame_resends.remove(&frame_id);
            for seg in self.lookbehind.iter().filter(|s| frame_id == s.key().0) {
                seg.remove();
            }
        }
    }

    /// Should be called by the underlying transport when raw packet data are received.
    /// The `data` argument must contain a valid [`SessionMessage`], otherwise the method throws an error.
    pub async fn received_message(&mut self, data: &[u8]) -> crate::errors::Result<()> {
        match SessionMessage::try_from(data)? {
            SessionMessage::Segment(s) => self.consume_segment(s),
            SessionMessage::Request(r) => self.retransmit_segments(r).await?,
            SessionMessage::Acknowledge(f) => self.acknowledged_frames(f),
        }
        Ok(())
    }

    /// Sends a requests for missing segments in incomplete frames.
    /// One [request](SessionMessage::Request) message is sent per incomplete frame. The message contains
    /// the segment indices missing from that frame. The `max_requests` argument can provide a maximum
    /// number of request messages sent by this call. If `max_requests` is `None`, request messages
    /// are set for all incomplete frames.
    /// Recurring requests have a [`rto_base_receiver`](SessionConfig) timeout with backoff.
    /// Returns the number of sent request messages.
    pub async fn request_missing_segments(&mut self, max_requests: Option<usize>) -> crate::errors::Result<usize> {
        let num_evicted = self.frame_reassembler.evict()?;
        debug!("{:?}: evicted {} frames", self.session_id, num_evicted);

        if max_requests == Some(0) {
            return Ok(0); // don't even bother
        }

        let tracked_incomplete = self.frame_reassembler.incomplete_frames();
        debug!(
            "{:?}: tracking {} incomplete frames",
            self.session_id,
            tracked_incomplete.len()
        );

        // Filter the frames which we are allowed to retry now
        let mut to_retry = Vec::with_capacity(tracked_incomplete.len());
        let now = Instant::now();
        for info in tracked_incomplete {
            match self.incoming_frame_retries.entry(info.frame_id) {
                Entry::Occupied(e) => {
                    // Check if we can retry this frame now
                    let rto_check = e
                        .get()
                        .check(now, self.cfg.rto_base_receiver, self.cfg.frame_expiration_age);
                    match rto_check {
                        RetryResult::RetryNow(next_rto) => {
                            // Retry this frame and plan ahead the time of the next retry
                            debug!(
                                "{:?}: going to perform frame {} retransmission req. #{}",
                                self.session_id, info.frame_id, next_rto.num_retry
                            );
                            e.replace_entry(next_rto);
                            to_retry.push(info);
                        }
                        RetryResult::Expired => {
                            // Frame is expired, so no more retries
                            debug!(
                                "{:?}: frame {} is already expired and will be evicted",
                                self.session_id, info.frame_id
                            );
                            e.remove();
                        }
                        RetryResult::Wait(d) => debug!(
                            "{:?}: frame {} needs to wait {d:?} for next retransmission request (#{})",
                            self.session_id,
                            info.frame_id,
                            e.get().num_retry
                        ),
                    }
                }
                Entry::Vacant(v) => {
                    // Happens when no segment of this frame has been received yet
                    debug!(
                        "{:?}: frame {} does not have a retry token",
                        self.session_id, info.frame_id
                    );
                    v.insert(RetryToken::new(now, self.cfg.backoff_base));
                    to_retry.push(info);
                }
            }
        }

        let mut sent = 0;
        for chunk in to_retry
            .chunks(SegmentRequest::<C>::MAX_ENTRIES)
            .take(max_requests.unwrap_or(usize::MAX))
            .map(|chunk| SegmentRequest::from_iter(chunk.iter().cloned()))
        {
            debug!(
                "{:?}: SENDING: retransmission request for segments {chunk:?}",
                self.session_id
            );

            let req = SessionMessage::<C>::Request(chunk);
            self.segment_ingress
                .feed(req)
                .await
                .map_err(|e| SessionError::ProcessingError(e.to_string()))?;
            sent += 1;
        }
        self.segment_ingress
            .flush()
            .await
            .map_err(|e| SessionError::ProcessingError(e.to_string()))?;

        debug!(
            "{:?}: RETRANSMISSION BATCH COMPLETE: sent {sent} re-send requests",
            self.session_id
        );
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
    pub async fn acknowledge_segments(&mut self, max_messages: Option<usize>) -> crate::errors::Result<usize> {
        let mut len = 0;
        let mut msgs = 0;

        while !self.to_acknowledge.is_empty() {
            let mut ack_frames = FrameAcknowledgements::<C>::default();

            if max_messages.map(|max| msgs < max).unwrap_or(true) {
                while !ack_frames.is_full() && !self.to_acknowledge.is_empty() {
                    if let Some(ack_id) = self.to_acknowledge.pop() {
                        ack_frames.push(ack_id);
                        len += 1;
                    }
                }

                debug!("{:?}: SENDING: acks of {} frames", self.session_id, ack_frames.len());
                self.segment_ingress
                    .feed(SessionMessage::Acknowledge(ack_frames))
                    .await
                    .map_err(|e| SessionError::ProcessingError(e.to_string()))?;
                msgs += 1;
            } else {
                break; // break out if we sent max allowed
            }
        }
        self.segment_ingress
            .flush()
            .await
            .map_err(|e| SessionError::ProcessingError(e.to_string()))?;

        debug!(
            "{:?}: ACK BATCH COMPLETE: sent {len} acks in {msgs} messages",
            self.session_id
        );
        Ok(len)
    }

    /// Performs retransmission of entire unacknowledged frames as sender.
    /// If [`acknowledged_frames_buffer`](SessionConfig) was set to `0` during the construction,
    /// this method will do nothing and return `0`.
    /// Otherwise, it returns the number of retransmitted frames.
    /// Recurring retransmissions have a [`rto_base_sender`](SessionConfig) timeout with backoff.
    /// At most `max_messages` frames are resent if the argument was specified.
    pub async fn retransmit_unacknowledged_frames(
        &mut self,
        max_messages: Option<usize>,
    ) -> crate::errors::Result<usize> {
        if max_messages == Some(0) || self.cfg.acknowledged_frames_buffer == 0 {
            return Ok(0);
        }

        let now = Instant::now();
        let max = max_messages.unwrap_or(usize::MAX);

        let mut frames_to_resend = BTreeSet::new();
        self.outgoing_frame_resends.retain(|frame_id, retry_log| {
            let check_res = retry_log.check(now, self.cfg.rto_base_sender, self.cfg.frame_expiration_age);
            match check_res {
                RetryResult::Wait(d) => {
                    debug!("{:?}: frame {frame_id} will retransmit in {d:?}", self.session_id);
                    true
                }
                RetryResult::RetryNow(next_retry) => {
                    // Single segment frame scenario
                    if frames_to_resend.len() < max {
                        frames_to_resend.insert(*frame_id);
                        *retry_log = next_retry;
                        debug!(
                            "{:?}: frame {frame_id} will self-resend now, next retry in {:?}",
                            self.session_id,
                            next_retry.retry_in(self.cfg.rto_base_sender, self.cfg.frame_expiration_age)
                        )
                    }
                    true
                }
                RetryResult::Expired => {
                    debug!("{:?}: frame {frame_id} expired", self.session_id);
                    false
                }
            }
        });

        debug!(
            "{:?}: {} frames will auto-resend",
            self.session_id,
            frames_to_resend.len()
        );

        // Find all segments of the frames to resend in the lookbehind buffer
        let mut count = 0;
        for segment in frames_to_resend
            .into_iter()
            .flat_map(|f| self.lookbehind.iter().filter(move |e| e.key().0 == f))
            .take(max)
        {
            let msg = SessionMessage::<C>::Segment(segment.value().clone());
            debug!(
                "{:?}: SENDING: auto-retransmitted segment: {:?}",
                self.session_id,
                segment.key()
            );
            self.segment_ingress
                .feed(msg)
                .await
                .map_err(|e| SessionError::ProcessingError(e.to_string()))?;
            count += 1;
        }
        self.segment_ingress
            .flush()
            .await
            .map_err(|e| SessionError::ProcessingError(e.to_string()))?;

        debug!(
            "{:?}: AUTO-RETRANSMIT BATCH COMPLETE: re-sent {count} segments",
            self.session_id
        );

        Ok(count)
    }

    /// Segments the `data` and sends them as (possibly multiple) [`SessionMessage::Segment`].
    /// Therefore, this method sends as many messages as needed after the data was segmented.
    /// Each segment is inserted into the lookbehind ring buffer for possible retransmissions.
    ///
    /// The size of the lookbehind ring buffer is given by the [`max_buffered_segments`](SessionConfig)
    /// given during the construction. It needs to accommodate as many segments as
    /// is the expected underlying transport bandwidth (segment/sec) to guarantee the retransmission
    /// can still happen within some time window.
    pub async fn send_frame_data(&mut self, data: &[u8]) -> crate::errors::Result<()> {
        // Real space for payload is MTU minus sizes of the headers
        let real_payload_len = C - SessionMessage::<C>::HEADER_SIZE - Segment::HEADER_SIZE;
        if data.is_empty() || data.len() > SessionMessage::<C>::MAX_SEGMENTS_PER_FRAME * real_payload_len {
            return Err(SessionError::IncorrectMessageLength.into());
        }

        let frame_id = self.outgoing_frame_id.fetch_add(1, Ordering::SeqCst);
        let segments = segment(data, real_payload_len, frame_id);

        for segment in segments {
            self.lookbehind.insert((&segment).into(), segment.clone());

            let msg = SessionMessage::<C>::Segment(segment.clone());
            debug!("{:?}: SENDING: segment {:?}", self.session_id, segment.id());
            self.segment_ingress
                .feed(msg)
                .await
                .map_err(|e| SessionError::ProcessingError(e.to_string()))?;

            // TODO: prevent stalling here
            while self.lookbehind.len() > self.cfg.max_buffered_segments {
                self.lookbehind.pop_front();
            }
        }

        self.segment_ingress
            .flush()
            .await
            .map_err(|e| SessionError::ProcessingError(e.to_string()))?;
        self.outgoing_frame_resends
            .insert(frame_id, RetryToken::new(Instant::now(), self.cfg.backoff_base));

        Ok(())
    }

    /// Convenience method to advance the state by calling all three methods in order:
    /// - [`SessionState::acknowledge_segments`]
    /// - [`SessionState::request_missing_segments`]
    /// - [`SessionState::retransmit_unacknowledged_frames`]
    ///
    /// The given optional limit is per each method call and is not shared, meaning
    /// each method gets the same limit.
    pub async fn advance(&mut self, max_messages: Option<usize>) -> crate::errors::Result<()> {
        self.acknowledge_segments(max_messages).await?;
        self.request_missing_segments(max_messages).await?;
        self.retransmit_unacknowledged_frames(max_messages).await?;
        Ok(())
    }

    /// Returns the ID of this session.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

impl<const C: usize, T: AsRef<[u8]>> Sink<T> for SessionState<C> {
    type Error = NetworkTypeError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project()
            .segment_ingress
            .poll_ready(cx)
            .map_err(|e| SessionError::ProcessingError(e.to_string()).into())
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        self.project()
            .segment_ingress
            .start_send(item.as_ref().try_into()?)
            .map_err(|e| SessionError::ProcessingError(e.to_string()).into())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project()
            .segment_ingress
            .poll_flush(cx)
            .map_err(|e| SessionError::ProcessingError(e.to_string()).into())
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.frame_reassembler.close();
        self.project()
            .segment_ingress
            .poll_close(cx)
            .map_err(|e| SessionError::ProcessingError(e.to_string()).into())
    }
}

/// Represents a socket for a session between two nodes bound by the
/// underlying [network transport](AsyncWrite) and the maximum transmission unit (MTU) of `C`.
///
/// It also implements [`AsyncRead`] and [`AsyncWrite`] so that it can
/// be used on top of the usual transport stack.
#[pin_project]
pub struct SessionSocket<const C: usize> {
    state: SessionState<C>,
    #[pin]
    frame_egress: Box<dyn AsyncRead + Send + Unpin>,
}

impl<const C: usize> SessionSocket<C> {
    /// Create a new socket over the given underlying `transport` that binds the communicating parties.
    /// A human-readable session `id` also must be supplied.
    pub fn new<T, I>(id: I, transport: T, cfg: SessionConfig) -> Self
    where
        T: AsyncWrite + Send + 'static,
        I: Display,
    {
        assert!(
            C >= SessionMessage::<C>::minimum_message_size(),
            "given MTU is too small"
        );

        let (reassembler, egress) = FrameReassembler::new(cfg.frame_expiration_age);

        let to_acknowledge = Arc::new(ArrayQueue::new(cfg.acknowledged_frames_buffer.max(1)));
        let is_acknowledging = (cfg.acknowledged_frames_buffer > 0).then_some(to_acknowledge.clone());
        let incoming_frame_retries = Arc::new(DashMap::new());

        let incoming_frame_retries_clone = incoming_frame_retries.clone();
        let id_clone = id.to_string().clone();

        let frame_egress = Box::new(
            egress
                .inspect(move |maybe_frame| {
                    match maybe_frame {
                        Ok(frame) => {
                            debug!("{id_clone:?}: emit frame {}", frame.frame_id);
                            // The frame has been completed, so remove its retry record
                            incoming_frame_retries_clone.remove(&frame.frame_id);
                            if let Some(ack_buffer) = &is_acknowledging {
                                // Acts as a ring buffer, so if the buffer is full, any unsent acknowledgements
                                // will be discarded.
                                ack_buffer.force_push(frame.frame_id);
                            }
                        }
                        Err(NetworkTypeError::FrameDiscarded(id)) | Err(NetworkTypeError::IncompleteFrame(id)) => {
                            // Remove retry token, because the frame has been discarded
                            incoming_frame_retries_clone.remove(id);
                            debug!("{id_clone:?}: frame {id} skipped");
                        }
                        _ => {}
                    }
                })
                .filter_map(|r| futures::future::ready(r.ok().map(Ok))) // Skip discarded frames
                .into_async_read(),
        );

        let (segment_ingress, segment_egress) = futures::channel::mpsc::unbounded();

        // Apply rate-limiting for egress segments if configured
        if let Some(rate_limit) = cfg.max_msg_per_sec.filter(|r| *r > 0).map(|r| r as u32) {
            let rate_limiter = RateLimiter::direct(Quota::per_second(rate_limit.try_into().unwrap()));
            let jitter = Jitter::up_to(Duration::from_millis(5));

            spawn(async move {
                segment_egress
                    .map(|m: SessionMessage<C>| Ok(m.into_encoded()))
                    .ratelimit_stream_with_jitter(&rate_limiter, jitter)
                    .forward(transport.into_sink())
                    .await
            });
        } else {
            spawn(
                segment_egress
                    .map(|m| Ok(m.into_encoded()))
                    .forward(transport.into_sink()),
            );
        }

        let state = SessionState {
            lookbehind: Arc::new(SkipMap::new()),
            outgoing_frame_id: Arc::new(AtomicU32::new(1)),
            frame_reassembler: Arc::new(reassembler),
            outgoing_frame_resends: Arc::new(DashMap::new()),
            session_id: id.to_string(),
            to_acknowledge,
            incoming_frame_retries,
            segment_ingress,
            cfg,
        };

        Self { state, frame_egress }
    }

    /// Gets the [state](SessionState) of this socket.
    pub fn state(&self) -> &SessionState<C> {
        &self.state
    }

    /// Gets the mutable [state](SessionState) of this socket.
    pub fn state_mut(&mut self) -> &mut SessionState<C> {
        &mut self.state
    }
}

impl<const C: usize> AsyncWrite for SessionSocket<C> {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        let mut socket_future = self.state.send_frame_data(buf).boxed();
        match Pin::new(&mut socket_future).poll(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(buf.len())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, e))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        // Only flush the underlying transport
        let mut flush_future = self.state.segment_ingress.flush();
        match Pin::new(&mut flush_future).poll(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, e))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        // Close the reassembler and the underlying transport
        self.state.frame_reassembler.close();
        self.state.segment_ingress.close_channel();
        let mut close_future = self.state.segment_ingress.close().boxed();
        match Pin::new(&mut close_future).poll(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, e))),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<const C: usize> AsyncRead for SessionSocket<C> {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        self.project().frame_egress.poll_read(cx, buf)
    }
}

/// Implementations of Tokio's AsyncRead and AsyncWrite for compatibility
#[cfg(feature = "runtime-tokio")]
pub mod tokio_compat {
    use std::pin::Pin;
    use std::task::{Context, Poll};

    use crate::session::state::SessionSocket;

    impl<const C: usize> tokio::io::AsyncWrite for SessionSocket<C> {
        fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
            futures::io::AsyncWrite::poll_write(self, cx, buf)
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            futures::io::AsyncWrite::poll_flush(self, cx)
        }

        fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            futures::io::AsyncWrite::poll_close(self, cx)
        }
    }

    impl<const C: usize> tokio::io::AsyncRead for SessionSocket<C> {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            // TODO: check correctness
            futures::io::AsyncRead::poll_read(self, cx, buf.filled_mut()).map(|_| Ok(()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::channel::mpsc::UnboundedSender;
    use futures::future::Either;
    use futures::io::{AsyncReadExt, AsyncWriteExt};
    use futures::pin_mut;
    use parameterized::parameterized;
    use rand::{thread_rng, Rng, SeedableRng};
    use std::fmt::Debug;
    use std::iter::Extend;
    use std::sync::OnceLock;
    use test_log::test;
    use tracing::warn;

    const MTU: usize = 466; // MTU used by HOPR

    #[derive(Debug, Clone)]
    pub struct FaultyNetworkConfig {
        pub fault_prob: f64,
        pub mixing_factor: usize,
        pub step_interval: Duration,
    }

    impl Default for FaultyNetworkConfig {
        fn default() -> Self {
            Self {
                fault_prob: 0.0,
                mixing_factor: 0,
                step_interval: Duration::from_millis(50),
            }
        }
    }

    #[derive(Clone)]
    pub struct FaultyNetwork {
        sender: UnboundedSender<Box<[u8]>>,
        counterparty: Arc<OnceLock<SessionState<MTU>>>,
        cfg: FaultyNetworkConfig,
    }

    impl AsyncWrite for FaultyNetwork {
        fn poll_write(mut self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
            self.send_to_counterparty(buf).unwrap();
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    fn setup_alice_bob(
        cfg: SessionConfig,
        network_cfg: FaultyNetworkConfig,
    ) -> (SessionSocket<MTU>, SessionSocket<MTU>) {
        let alice_to_bob_transport = FaultyNetwork::new(network_cfg.clone());
        let bob_to_alice_transport = FaultyNetwork::new(network_cfg.clone());

        let alice_to_bob_ctp = alice_to_bob_transport.counterparty.clone();
        let bob_to_alice_ctp = bob_to_alice_transport.counterparty.clone();

        let alice_to_bob = SessionSocket::new("alice", alice_to_bob_transport, cfg.clone());
        let bob_to_alice = SessionSocket::new("bob", bob_to_alice_transport, cfg.clone());

        alice_to_bob_ctp.set(bob_to_alice.state().clone()).unwrap();
        bob_to_alice_ctp.set(alice_to_bob.state().clone()).unwrap();

        let mut alice_bob_state = alice_to_bob.state().clone();
        let mut bob_alice_state = bob_to_alice.state().clone();
        async_std::task::spawn_local(async move {
            loop {
                alice_bob_state.advance(None).await.unwrap();
                async_std::task::sleep(network_cfg.step_interval).await;
            }
        });

        async_std::task::spawn_local(async move {
            loop {
                bob_alice_state.advance(None).await.unwrap();
                async_std::task::sleep(network_cfg.step_interval).await;
            }
        });

        (alice_to_bob, bob_to_alice)
    }

    impl FaultyNetwork {
        pub fn new(cfg: FaultyNetworkConfig) -> Self {
            let rng = rand::rngs::StdRng::from_rng(thread_rng()).unwrap();
            let counterparty = Arc::new(OnceLock::<SessionState<MTU>>::new());

            let (sender, recv) = futures::channel::mpsc::unbounded::<Box<[u8]>>();
            let mut rng_clone = rng.clone();
            let recv = if cfg.mixing_factor > 0 {
                recv.map(move |x| {
                    async_std::task::sleep(Duration::from_micros(rng_clone.gen_range(10..1000)))
                        .then(|_| futures::future::ready(x))
                })
                .buffer_unordered(cfg.mixing_factor)
                .boxed()
            } else {
                recv.boxed()
            };

            let counterparty_clone = counterparty.clone();
            async_std::task::spawn(async move {
                pin_mut!(recv);
                while let Some(data) = recv.next().await {
                    if let Some(mut counterparty) = counterparty_clone.get().cloned() {
                        counterparty.received_message(&data).await.unwrap();
                    }
                }
            });

            Self {
                sender,
                counterparty,
                cfg,
            }
        }

        fn send_to_counterparty(&mut self, data: &[u8]) -> crate::errors::Result<()> {
            if thread_rng().gen_bool(self.cfg.fault_prob) {
                warn!("msg discarded");
            } else {
                self.sender.unbounded_send(data.into()).unwrap();
            }
            Ok(())
        }
    }

    #[derive(PartialEq, Eq)]
    enum Direction {
        Send,
        Recv,
        Both,
    }

    async fn send_and_recv<S>(num_frames: usize, frame_size: usize, alice: S, bob: S, timeout: Duration, one_way: bool)
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let socket_worker = |mut socket: S, d: Direction| async move {
            let mut received = Vec::with_capacity(num_frames * frame_size);
            let mut sent = Vec::with_capacity(num_frames * frame_size);

            if d == Direction::Send || d == Direction::Both {
                for _ in 0..num_frames {
                    let mut write = vec![0u8; frame_size];
                    hopr_crypto_random::random_fill(&mut write);
                    socket.write(&write).await.unwrap();
                    sent.extend(write);
                }
            }

            if d == Direction::Recv || d == Direction::Both {
                for _ in 0..num_frames {
                    let mut read = vec![0u8; frame_size];
                    socket.read_exact(&mut read).await.unwrap();
                    received.extend(read);
                }
            }

            (sent, received)
        };

        let alice_worker = async_std::task::spawn(socket_worker(
            alice,
            if one_way { Direction::Send } else { Direction::Both },
        ));
        let bob_worker = async_std::task::spawn(socket_worker(
            bob,
            if one_way { Direction::Recv } else { Direction::Both },
        ));

        let send_recv = futures::future::join(bob_worker, alice_worker);
        let timeout = async_std::task::sleep(timeout);

        pin_mut!(send_recv);
        pin_mut!(timeout);

        match futures::future::select(send_recv, timeout).await {
            Either::Left((((alice_sent, alice_recv), (bob_sent, bob_recv)), _)) => {
                assert_eq!(alice_sent, bob_recv, "alice sent must be equal to bob received");
                assert_eq!(bob_sent, alice_recv, "bob sent must be equal to alice received",);
            }
            Either::Right(_) => panic!("timeout"),
        }
    }

    #[parameterized(num_frames = {10, 100, 1000}, frame_size = {1500, 1500, 1500})]
    #[parameterized_macro(async_std::test)]
    async fn test_reliable_send_recv_no_ack(num_frames: usize, frame_size: usize) {
        let cfg = SessionConfig {
            acknowledged_frames_buffer: 0,
            ..Default::default()
        };

        let (alice_to_bob, bob_to_alice) = setup_alice_bob(cfg, Default::default());

        send_and_recv(
            num_frames,
            frame_size,
            alice_to_bob,
            bob_to_alice,
            Duration::from_secs(10),
            false,
        )
        .await;
    }

    #[parameterized(num_frames = {10, 100, 1000}, frame_size = {1500, 1500, 1500})]
    #[parameterized_macro(async_std::test)]
    async fn test_reliable_send_recv(num_frames: usize, frame_size: usize) {
        let (alice_to_bob, bob_to_alice) = setup_alice_bob(Default::default(), Default::default());

        send_and_recv(
            num_frames,
            frame_size,
            alice_to_bob,
            bob_to_alice,
            Duration::from_secs(10),
            false,
        )
        .await;
    }

    #[parameterized(num_frames = {10, 100, 1000}, frame_size = {1500, 1500, 1500})]
    #[parameterized_macro(async_std::test)]
    async fn test_unreliable_send_recv(num_frames: usize, frame_size: usize) {
        let cfg = SessionConfig {
            rto_base_sender: Duration::from_millis(500),
            rto_base_receiver: Duration::from_millis(10),
            frame_expiration_age: Duration::from_secs(30),
            backoff_base: 1.001,
            ..Default::default()
        };

        let net_cfg = FaultyNetworkConfig {
            fault_prob: 0.33,
            ..Default::default()
        };

        let (alice_to_bob, bob_to_alice) = setup_alice_bob(cfg, net_cfg);

        send_and_recv(
            num_frames,
            frame_size,
            alice_to_bob,
            bob_to_alice,
            Duration::from_secs(30),
            false,
        )
        .await;
    }

    #[parameterized(num_frames = {10, 100, 1000}, frame_size = {1500, 1500, 1500})]
    #[parameterized_macro(async_std::test)]
    async fn test_unreliable_mixed_send_recv(num_frames: usize, frame_size: usize) {
        let cfg = SessionConfig {
            rto_base_sender: Duration::from_millis(500),
            rto_base_receiver: Duration::from_millis(10),
            frame_expiration_age: Duration::from_secs(30),
            backoff_base: 1.001,
            ..Default::default()
        };

        let net_cfg = FaultyNetworkConfig {
            fault_prob: 0.33,
            mixing_factor: 4,
            ..Default::default()
        };

        let (alice_to_bob, bob_to_alice) = setup_alice_bob(cfg, net_cfg);

        send_and_recv(
            num_frames,
            frame_size,
            alice_to_bob,
            bob_to_alice,
            Duration::from_secs(30),
            false,
        )
        .await;
    }

    #[parameterized(num_frames = {10, 100, 1000}, frame_size = {1500, 1500, 1500})]
    #[parameterized_macro(async_std::test)]
    async fn test_almost_reliable_mixed_send_recv(num_frames: usize, frame_size: usize) {
        let cfg = SessionConfig {
            rto_base_sender: Duration::from_millis(500),
            rto_base_receiver: Duration::from_millis(10),
            frame_expiration_age: Duration::from_secs(30),
            backoff_base: 1.001,
            ..Default::default()
        };

        let net_cfg = FaultyNetworkConfig {
            fault_prob: 0.1,
            mixing_factor: 4,
            ..Default::default()
        };

        let (alice_to_bob, bob_to_alice) = setup_alice_bob(cfg, net_cfg);

        send_and_recv(
            num_frames,
            frame_size,
            alice_to_bob,
            bob_to_alice,
            Duration::from_secs(30),
            false,
        )
        .await;
    }

    #[parameterized(num_frames = {10, 100, 1000}, frame_size = {1500, 1500, 1500})]
    #[parameterized_macro(async_std::test)]
    async fn test_reliable_mixed_send_recv(num_frames: usize, frame_size: usize) {
        let cfg = SessionConfig {
            rto_base_sender: Duration::from_millis(500),
            rto_base_receiver: Duration::from_millis(10),
            frame_expiration_age: Duration::from_secs(30),
            backoff_base: 1.001,
            ..Default::default()
        };

        let net_cfg = FaultyNetworkConfig {
            mixing_factor: 4,
            ..Default::default()
        };

        let (alice_to_bob, bob_to_alice) = setup_alice_bob(cfg, net_cfg);

        send_and_recv(
            num_frames,
            frame_size,
            alice_to_bob,
            bob_to_alice,
            Duration::from_secs(30),
            false,
        )
        .await;
    }

    #[test(async_std::test)]
    async fn test_disconnected_network_should_timeout() {
        let cfg = SessionConfig {
            rto_base_sender: Duration::from_millis(250),
            rto_base_receiver: Duration::from_millis(300),
            frame_expiration_age: Duration::from_secs(2),
            ..Default::default()
        };

        let net_cfg = FaultyNetworkConfig {
            fault_prob: 1.0, // throws away 100% of packets
            mixing_factor: 0,
            step_interval: Duration::from_millis(200),
        };

        let (mut alice_to_bob, mut bob_to_alice) = setup_alice_bob(cfg, net_cfg);
        let data = b"will not be delivered!";

        alice_to_bob.write(data.as_ref()).await.unwrap();

        let mut out = vec![0u8; data.len()];
        let f1 = bob_to_alice.read_exact(&mut out);
        let f2 = async_std::task::sleep(Duration::from_secs(3));
        pin_mut!(f1);
        pin_mut!(f2);

        match futures::future::select(f1, f2).await {
            Either::Left(_) => panic!("should timeout: {:?}", out),
            Either::Right(_) => {}
        }
    }

    #[test(async_std::test)]
    async fn test_single_frame_resend() {
        let cfg = SessionConfig {
            rto_base_sender: Duration::from_millis(250),
            rto_base_receiver: Duration::from_millis(300),
            frame_expiration_age: Duration::from_secs(2),
            ..Default::default()
        };

        let net_cfg = FaultyNetworkConfig {
            fault_prob: 0.5, // throws away 50% of packets
            mixing_factor: 0,
            step_interval: Duration::from_millis(200),
        };

        let (mut alice_to_bob, mut bob_to_alice) = setup_alice_bob(cfg, net_cfg);
        let data = b"will be re-delivered!";

        alice_to_bob.write(data.as_ref()).await.unwrap();

        let mut out = vec![0u8; data.len()];
        let f1 = bob_to_alice.read_exact(&mut out);
        let f2 = async_std::task::sleep(Duration::from_secs(5));
        pin_mut!(f1);
        pin_mut!(f2);

        match futures::future::select(f1, f2).await {
            Either::Left(_) => {}
            Either::Right(_) => panic!("timeout"),
        }
    }
}
