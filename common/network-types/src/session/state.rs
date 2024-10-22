//! # `Session` protocol state machine
//!
//! The protocol always forms a middle layer between a *lower layer* transport (such as an unreliable
//! UDP-like network) and any upstream protocol.
//! The communication with the *lower layer* is done via [`SessionState`];
//! the *upper layer* is using the [`SessionSocket`] to pass data with the
//! `Session` protocol.
//!
//! ## Instantiation
//! The instantiation of the protocol state machine is done by creating the [`SessionSocket`]
//! object, by [providing it](SessionSocket::new) an underlying transport writer and its MTU `C`.
//! The protocol can be instantiated over any transport that implements [`AsyncWrite`] + [`AsyncRead`]
//! for sending and receiving raw data packets.
//!
//! ## Passing data between the protocol and the upper layer
//! The [`SessionSocket`] exposes as [`AsyncRead`] +
//! [`AsyncWrite`] and can be used to read and write arbitrary data
//! to the protocol. If the writer is [closed](AsyncWrite::poll_close), the session is closed
//! as well.
//!
//! ## Passing of data between the protocol and the lower layer
//!
//! As long as the underlying transport implements [`AsyncRead`] + [`AsyncWrite`],
//! the [`SessionSocket`] automatically polls data from the underlying transport,
//! and sends the data to the underlying transport as needed.
//!
//! ## Protocol features
//!
//! ### Data segmentation
//! Once data is written to the [`SessionSocket`], it is segmented and written
//! automatically to the underlying transport. Every writing to the `SessionSocket` corresponds to
//! a [`Frame`](crate::session::frame::Frame).
//!
//! ## Frame reassembly
//! The receiving side performs frame reassembly and sequencing of the frames.
//! Frames are never emitted to the upper layer transport out of order, but frames
//! can be skipped if they exceed the [`frame_expiration_age`](SessionConfig).
//!
//! ## Frame acknowledgement
//!
//! The recipient can acknowledge frames to the sender once all its segments have been received.
//! This is done with a [`FrameAcknowledgements`] message sent back
//! to the sender.
//!
//! ## Segment retransmission
//!
//! There are two means of segment retransmission:
//!
//! ### Recipient requested retransmission
//! This is useful in situations when the recipient has received only some segments of a frame.
//! At this point, the recipient knows which segments are missing in a frame and can initiate
//! [`SegmentRequest`] sent back to the sender.
//! This method is more targeted, as it requests only those segments of a frame that are needed.
//! Once the sender receives the segment request, it will retransmit the segments in question
//! over to the receiver.
//! The recipient can make repeating requests on retransmission, based on the network reliability.
//! However, retransmission requests decay with an exponential backoff given by `backoff_base`
//! and `rto_base_receiver` timeout in [`SessionConfig`] up
//! until the `frame_expiration_age`.
//!
//!
//! ### Sender initiated retransmission
//! The frame sender can also automatically retransmit entire frames (= all their segments)
//! to the recipient. This happens if the sender (within a time period) did not receive the
//! frame acknowledgement *and* the recipient also did not request retransmission of any segment in
//! that frame.
//! This is useful in situations when the recipient did not receive any segment of a frame. Once
//! the recipient receives at least one segment of a frame, the recipient requested retransmission
//! is the preferred way.
//!
//! The sender can make repeating frame retransmissions, based on the network reliability.
//! However, retransmissions decay with an exponential backoff given by `backoff_base`
//! and `rto_base_sender` timeout in [`SessionConfig`] up until
//! the `frame_expiration_age`.
//! The retransmissions of a frame by the sender stop if the frame has been acknowledged by the
//! recipient *or* the recipient started requesting segment retransmission.
//!
//! ### Retransmission timing
//! Both retransmission methods will work up until `frame_expiration_age`. Since the
//! recipient-request-based method is more targeted, at least one should be allowed to happen
//! before the sender-initiated retransmission kicks in. Therefore, it is recommended to set
//! the `rto_base_sender` at least twice the `rto_base_receiver`.
//!
//! The above protocol features can be enabled by setting [SessionFeature] options in the configuration
//! during [SessionSocket] construction.
use crossbeam_queue::ArrayQueue;
use crossbeam_skiplist::SkipMap;
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use futures::channel::mpsc::UnboundedSender;
use futures::future::BoxFuture;
use futures::{
    pin_mut, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, FutureExt, Sink, SinkExt, StreamExt, TryStreamExt,
};
use governor::prelude::StreamRateLimitExt;
use governor::{Jitter, Quota, RateLimiter};
use smart_default::SmartDefault;
use std::collections::{BTreeSet, HashSet};
use std::fmt::{Debug, Display};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tracing::{debug, error, trace, warn};

use hopr_async_runtime::prelude::spawn;

use crate::errors::NetworkTypeError;
use crate::session::errors::SessionError;
use crate::session::frame::{segment, FrameId, FrameReassembler, Segment, SegmentId};
use crate::session::protocol::{FrameAcknowledgements, SegmentRequest, SessionMessage};
use crate::session::utils::{AsyncReadStreamer, RetryResult, RetryToken};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_TIME_TO_ACK: hopr_metrics::MultiHistogram =
        hopr_metrics::MultiHistogram::new(
            "hopr_session_time_to_ack",
            "Time in seconds until a complete frame gets acknowledged by the recipient",
            vec![0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0],
            &["session_id"]
    ).unwrap();
}

/// Represents individual Session protocol features that can be enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SessionFeature {
    /// Enable requesting of incomplete frames by the recipient.
    RequestIncompleteFrames,
    /// Enable frame retransmission by the sender.
    /// This requires `AcknowledgeFrames` to be enabled at the recipient.
    RetransmitFrames,
    /// Enable frame acknowledgement by the recipient.
    AcknowledgeFrames,
}

impl SessionFeature {
    /// All features
    fn all() -> Vec<SessionFeature> {
        vec![
            SessionFeature::AcknowledgeFrames,
            SessionFeature::RequestIncompleteFrames,
            SessionFeature::RetransmitFrames,
        ]
    }
}

/// Configuration of Session protocol.
#[derive(Debug, Clone, SmartDefault)]
pub struct SessionConfig {
    /// Maximum number of buffered segments.
    ///
    /// The value should be large enough to accommodate segments for an at least
    /// `frame_expiration_age` period, considering the expected maximum bandwidth.
    #[default = 50_000]
    pub max_buffered_segments: usize,

    /// Size of the buffer for acknowledged frame IDs.
    ///
    /// The value should be large enough so that the buffer can accommodate segments
    /// for an at least `frame_expiration_age` period, given the expected maximum bandwidth.
    ///
    /// If set to 0, no frame acknowledgements will be sent.
    #[default = 1024]
    pub acknowledged_frames_buffer: usize,

    /// Specifies the maximum period a frame should be kept by the sender and
    /// asked for retransmission by the recipient.
    #[default(Duration::from_secs(30))]
    pub frame_expiration_age: Duration,

    /// If a frame is incomplete (on the receiver), retransmission requests will be made
    /// with exponential backoff starting at this initial retry timeout (RTO).
    ///
    /// Requests will be sent until `frame_expiration_age` is reached.
    ///
    /// NOTE: this value should be offset from `rto_base_sender`, so that the receiver's
    /// retransmission requests are interleaved with the sender's retransmissions.
    ///
    /// In *most* cases, you want to 0 < `rto_base_receiver` < `rto_base_sender` < `frame_expiration_age`.
    #[default(Duration::from_millis(1000))]
    pub rto_base_receiver: Duration,

    /// If a frame is unacknowledged (on the sender), entire frame retransmissions will be made
    /// with exponential backoff starting at this initial retry timeout (RTO).
    ///
    /// Frames will be retransmitted until `frame_expiration_age` is reached.
    ///
    /// NOTE: this value should be offset from `rto_base_receiver`, so that the receiver's
    /// retransmission requests are interleaved with the sender's retransmissions.
    ///
    /// In *most* cases, you want to 0 < `rto_base_receiver` < `rto_base_sender` < `frame_expiration_age`.
    #[default(Duration::from_millis(1500))]
    pub rto_base_sender: Duration,

    /// Base for the exponential backoff on retries.
    #[default(2.0)]
    pub backoff_base: f64,

    /// Standard deviation of a Gaussian jitter applied to `rto_base_receiver` and
    /// `rto_base_sender`. Must be between 0 and 0.25.
    #[default(0.05)]
    pub rto_jitter: f64,

    /// Optional rate limiting of egress messages per second.
    /// This will force the [SessionSocket] not to pass more than this quota of messages
    /// to the underlying transport.
    #[default(None)]
    pub max_msg_per_sec: Option<usize>,

    /// Set of [features](SessionFeature) that should be enabled on this Session.
    #[default(_code = "HashSet::from_iter(SessionFeature::all())")]
    pub enabled_features: HashSet<SessionFeature>,
}

/// Contains the cloneable state of the session bound to a [`SessionSocket`].
///
/// It implements the entire [`SessionMessage`] state machine and
/// performs the frame reassembly and sequencing.
/// The MTU size is specified by `C`.
///
/// The `SessionState` cannot be created directly, it must always be created via [`SessionSocket`] and
/// then retrieved via [`SessionSocket::state`].
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
    segment_egress_send: UnboundedSender<SessionMessage<C>>,
}

fn maybe_fused_future<'a, F>(condition: bool, future: F) -> futures::future::Fuse<BoxFuture<'a, ()>>
where
    F: Future<Output = ()> + Send + Sync + 'a,
{
    if condition {
        future.boxed()
    } else {
        futures::future::pending().boxed()
    }
    .fuse()
}

impl<const C: usize> SessionState<C> {
    fn consume_segment(&mut self, segment: Segment) -> crate::errors::Result<()> {
        let id = segment.id();

        match self.frame_reassembler.push_segment(segment) {
            Ok(_) => {
                trace!(session_id = self.session_id, segment = %id, "RECEIVED: segment");
                match self.incoming_frame_retries.entry(id.0) {
                    Entry::Occupied(e) => {
                        // Receiving a frame segment restarts the retry token for this frame
                        let rt = *e.get();
                        e.replace_entry(rt.replenish(Instant::now(), self.cfg.backoff_base));
                    }
                    Entry::Vacant(v) => {
                        // Create the retry token for this frame
                        v.insert(RetryToken::new(Instant::now(), self.cfg.backoff_base));
                    }
                }
            }
            // The error here is intentionally not propagated
            Err(e) => warn!(session_id = self.session_id, ?id, error = %e, "segment not pushed"),
        }

        Ok(())
    }

    fn retransmit_segments(&mut self, request: SegmentRequest<C>) -> crate::errors::Result<()> {
        trace!(
            session_id = self.session_id,
            count_of_segments = request.len(),
            "RECEIVED: request",
        );

        let mut count = 0;
        request
            .into_iter()
            .filter_map(|segment_id| {
                // No need to retry this frame ourselves, since the other side will request on its own
                self.outgoing_frame_resends.remove(&segment_id.0);
                let ret = self
                    .lookbehind
                    .get(&segment_id)
                    .map(|e| SessionMessage::<C>::Segment(e.value().clone()));
                if ret.is_some() {
                    trace!(
                        session_id = self.session_id,
                        %segment_id,
                        "SENDING: retransmitted segment"
                    );
                    count += 1;
                } else {
                    warn!(
                        session_id = self.session_id,
                        id = ?segment_id,
                        "segment not in lookbehind buffer anymore",
                    );
                }
                ret
            })
            .try_for_each(|msg| self.segment_egress_send.unbounded_send(msg))
            .map_err(|e| SessionError::ProcessingError(e.to_string()))?;

        trace!(session_id = self.session_id, count, "retransmitted requested segments");

        Ok(())
    }

    fn acknowledged_frames(&mut self, acked: FrameAcknowledgements<C>) -> crate::errors::Result<()> {
        trace!(
            session_id = self.session_id,
            count = acked.len(),
            "RECEIVED: acknowledgement frames",
        );

        for frame_id in acked {
            // Frame acknowledged, we won't need to resend it
            if let Some((_, rt)) = self.outgoing_frame_resends.remove(&frame_id) {
                let to_ack = rt.time_since_creation();
                trace!(
                    session_id = self.session_id,
                    frame_id,
                    duration_in_ms = to_ack.as_millis(),
                    "frame acknowledgement duratin"
                );

                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_TIME_TO_ACK.observe(&[self.session_id()], to_ack.as_secs_f64())
            }

            for seg in self.lookbehind.iter().filter(|s| frame_id == s.key().0) {
                seg.remove();
            }
        }

        Ok(())
    }

    /// Sends a request for missing segments in incomplete frames.
    /// One [request](SessionMessage::Request) message is sent per incomplete frame. The message contains
    /// the segment indices missing from that frame.
    /// Recurring requests have an [`rto_base_receiver`](SessionConfig) timeout with backoff.
    /// Returns the number of sent request messages.
    async fn request_missing_segments(&mut self) -> crate::errors::Result<usize> {
        let tracked_incomplete = self.frame_reassembler.incomplete_frames();
        trace!(
            session_id = self.session_id,
            count = tracked_incomplete.len(),
            "tracking incomplete frames",
        );

        // Filter the frames which we are allowed to retry now
        let mut to_retry = Vec::with_capacity(tracked_incomplete.len());
        let now = Instant::now();
        for info in tracked_incomplete {
            match self.incoming_frame_retries.entry(info.frame_id) {
                Entry::Occupied(e) => {
                    // Check if we can retry this frame now
                    let rto_check = e.get().check(
                        now,
                        self.cfg.rto_base_receiver,
                        self.cfg.frame_expiration_age,
                        self.cfg.rto_jitter,
                    );
                    match rto_check {
                        RetryResult::RetryNow(next_rto) => {
                            // Retry this frame and plan ahead of the time of the next retry
                            trace!(
                                session_id = self.session_id,
                                frame_id = info.frame_id,
                                retransmission_number = next_rto.num_retry,
                                "performing frame retransmission",
                            );
                            e.replace_entry(next_rto);
                            to_retry.push(info);
                        }
                        RetryResult::Expired => {
                            // Frame is expired, so no more retries
                            debug!(
                                session_id = self.session_id,
                                frame_id = info.frame_id,
                                "frame is already expired and will be evicted"
                            );
                            e.remove();
                        }
                        RetryResult::Wait(d) => trace!(
                            session_id = self.session_id,
                            frame_id = info.frame_id,
                            timeout_in_ms = d.as_millis(),
                            next_retransmission_request_number = e.get().num_retry,
                            "frame needs to wait for next retransmission request",
                        ),
                    }
                }
                Entry::Vacant(v) => {
                    // Happens when no segment of this frame has been received yet
                    debug!(
                        session_id = self.session_id,
                        frame_id = info.frame_id,
                        "frame does not have a retry token"
                    );
                    v.insert(RetryToken::new(now, self.cfg.backoff_base));
                    to_retry.push(info);
                }
            }
        }

        let mut sent = 0;
        let to_retry = to_retry
            .chunks(SegmentRequest::<C>::MAX_ENTRIES)
            .map(|chunk| Ok(SessionMessage::<C>::Request(chunk.iter().cloned().collect())))
            .inspect(|r| {
                trace!(
                    session_id = self.session_id,
                    result = ?r,
                    "SENDING: retransmission request"
                );
                sent += 1;
            })
            .collect::<Vec<_>>();

        self.segment_egress_send
            .send_all(&mut futures::stream::iter(to_retry))
            .await
            .map_err(|e| SessionError::ProcessingError(e.to_string()))?;

        trace!(
            session_id = self.session_id,
            count = sent,
            "RETRANSMISSION BATCH COMPLETE: sent {sent} re-send requests",
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
    /// this method sends as many messages as needed.
    async fn acknowledge_segments(&mut self) -> crate::errors::Result<usize> {
        let mut len = 0;
        let mut msgs = 0;

        while !self.to_acknowledge.is_empty() {
            let mut ack_frames = FrameAcknowledgements::<C>::default();

            while !ack_frames.is_full() && !self.to_acknowledge.is_empty() {
                if let Some(ack_id) = self.to_acknowledge.pop() {
                    ack_frames.push(ack_id);
                    len += 1;
                }
            }

            trace!(
                session_id = self.session_id,
                count = ack_frames.len(),
                "SENDING: acknowledgements of frames",
            );
            self.segment_egress_send
                .feed(SessionMessage::Acknowledge(ack_frames))
                .await
                .map_err(|e| SessionError::ProcessingError(e.to_string()))?;
            msgs += 1;
        }
        self.segment_egress_send
            .flush()
            .await
            .map_err(|e| SessionError::ProcessingError(e.to_string()))?;

        trace!(
            session_id = self.session_id,
            count = len,
            messages = msgs,
            "ACK BATCH COMPLETE: sent acks in messages",
        );
        Ok(len)
    }

    /// Performs retransmission of entire unacknowledged frames as sender.
    /// If [`acknowledged_frames_buffer`](SessionConfig) was set to `0` during the construction,
    /// this method will do nothing and return `0`.
    /// Otherwise, it returns the number of retransmitted frames.
    /// Recurring retransmissions have an [`rto_base_sender`](SessionConfig) timeout with backoff.
    async fn retransmit_unacknowledged_frames(&mut self) -> crate::errors::Result<usize> {
        if self.cfg.acknowledged_frames_buffer == 0 {
            return Ok(0);
        }

        let now = Instant::now();

        // Retain only non-expired frames, collect all which are due for re-send
        let mut frames_to_resend = BTreeSet::new();
        self.outgoing_frame_resends.retain(|frame_id, retry_log| {
            let check_res = retry_log.check(
                now,
                self.cfg.rto_base_sender,
                self.cfg.frame_expiration_age,
                self.cfg.rto_jitter,
            );
            match check_res {
                RetryResult::Wait(d) => {
                    trace!(
                        session_id = self.session_id,
                        frame_id,
                        wait_timeout_in_ms = d.as_millis(),
                        "frame will retransmit"
                    );
                    true
                }
                RetryResult::RetryNow(next_retry) => {
                    // Single segment frame scenario
                    frames_to_resend.insert(*frame_id);
                    *retry_log = next_retry;
                    debug!(session_id = self.session_id, frame_id, "frame will self-resend now");
                    true
                }
                RetryResult::Expired => {
                    debug!(session_id = self.session_id, frame_id, "frame expired");
                    false
                }
            }
        });

        trace!(
            session_id = self.session_id,
            count = frames_to_resend.len(),
            "frames will auto-resend",
        );

        // Find all segments of the frames to resend in the lookbehind buffer,
        // skip those that are not in the lookbehind buffer anymore
        let mut count = 0;
        let frames_to_resend = frames_to_resend
            .into_iter()
            .flat_map(|f| self.lookbehind.iter().filter(move |e| e.key().0 == f))
            .inspect(|e| {
                trace!(
                    session_id = self.session_id,
                    key = ?e.key(),
                    "SENDING: auto-retransmitted"
                );
                count += 1
            })
            .map(|e| Ok(SessionMessage::<C>::Segment(e.value().clone())))
            .collect::<Vec<_>>();

        self.segment_egress_send
            .send_all(&mut futures::stream::iter(frames_to_resend))
            .await
            .map_err(|e| SessionError::ProcessingError(e.to_string()))?;

        trace!(
            session_id = self.session_id,
            count,
            "AUTO-RETRANSMIT BATCH COMPLETE: re-sent segments",
        );

        Ok(count)
    }

    const PAYLOAD_CAPACITY: usize = C - SessionMessage::<C>::HEADER_SIZE - Segment::HEADER_SIZE;

    const MAX_WRITE_SIZE: usize = SessionMessage::<C>::MAX_SEGMENTS_PER_FRAME * Self::PAYLOAD_CAPACITY;

    /// Segments the `data` and sends them as (possibly multiple) [`SessionMessage::Segment`].
    /// Therefore, this method sends as many messages as needed after the data was segmented.
    /// Each segment is inserted into the lookbehind ring buffer for possible retransmissions.
    ///
    /// The size of the lookbehind ring buffer is given by the [`max_buffered_segments`](SessionConfig)
    /// given during the construction. It needs to accommodate as many segments as
    /// is the expected underlying transport bandwidth (segment/sec) to guarantee the retransmission
    /// can still happen within some time window.
    pub async fn send_frame_data(&mut self, data: &[u8]) -> crate::errors::Result<()> {
        if data.is_empty() || data.len() > Self::MAX_WRITE_SIZE {
            return Err(SessionError::IncorrectMessageLength.into());
        }

        let frame_id = self.outgoing_frame_id.fetch_add(1, Ordering::SeqCst);
        let segments = segment(data, Self::PAYLOAD_CAPACITY, frame_id)?;
        let count = segments.len();

        for segment in segments {
            let msg = SessionMessage::<C>::Segment(segment.clone());
            trace!(session_id = self.session_id, id = ?segment.id(), "SENDING: segment");
            self.segment_egress_send
                .feed(msg)
                .await
                .map_err(|e| SessionError::ProcessingError(e.to_string()))?;

            // This is the only place where we insert into the lookbehind buffer
            self.lookbehind.insert((&segment).into(), segment.clone());
            while self.lookbehind.len() > self.cfg.max_buffered_segments {
                self.lookbehind.pop_front();
            }
        }

        self.segment_egress_send
            .flush()
            .await
            .map_err(|e| SessionError::ProcessingError(e.to_string()))?;
        self.outgoing_frame_resends
            .insert(frame_id, RetryToken::new(Instant::now(), self.cfg.backoff_base));

        trace!(
            session_id = self.session_id,
            frame_id,
            count,
            "FRAME SEND COMPLETE: sent segments",
        );

        Ok(())
    }

    /// Convenience method to advance the state by calling all three methods in order:
    /// - [`SessionState::acknowledge_segments`]
    /// - [`SessionState::request_missing_segments`]
    /// - [`SessionState::retransmit_unacknowledged_frames`]
    ///
    async fn state_loop(&mut self) -> crate::errors::Result<()> {
        // Rate limiter for reassembler evictions:
        // tries to evict 10 times before a frame expires
        let eviction_limiter =
            governor::RateLimiter::direct(Quota::with_period(self.cfg.frame_expiration_age / 10).ok_or(
                NetworkTypeError::Other("rate limiter frame_expiration_age invalid".into()),
            )?);

        // Rate limiter for acknowledgements:
        // sends acknowledgements 4 times more often
        // than the other side can retransmit them, or we ask for retransmissions.
        let ack_rate_limiter = governor::RateLimiter::direct(
            Quota::with_period(self.cfg.rto_base_sender.min(self.cfg.rto_base_receiver) / 4)
                .ok_or(NetworkTypeError::Other("rate limiter ack rate invalid".into()))?,
        );

        // Rate limiter for retransmissions by the sender
        let sender_retransmit = governor::RateLimiter::direct(
            Quota::with_period(self.cfg.rto_base_sender)
                .ok_or(NetworkTypeError::Other("rate limiter rto sender invalid".into()))?,
        );

        // Rate limiter for retransmissions by the receiver
        let receiver_retransmit = governor::RateLimiter::direct(
            Quota::with_period(self.cfg.rto_base_receiver)
                .ok_or(NetworkTypeError::Other("rate limiter rto receiver invalid".into()))?,
        );

        loop {
            let mut evict_fut = eviction_limiter.until_ready().boxed().fuse();
            let mut ack_fut = maybe_fused_future(
                self.cfg.enabled_features.contains(&SessionFeature::AcknowledgeFrames),
                ack_rate_limiter.until_ready(),
            );
            let mut r_snd_fut = maybe_fused_future(
                self.cfg.enabled_features.contains(&SessionFeature::RetransmitFrames),
                sender_retransmit.until_ready(),
            );
            let mut r_rcv_fut = maybe_fused_future(
                self.cfg
                    .enabled_features
                    .contains(&SessionFeature::RequestIncompleteFrames),
                receiver_retransmit.until_ready(),
            );
            let mut is_done = maybe_fused_future(self.segment_egress_send.is_closed(), futures::future::ready(()));

            // Futures in select_biased! are ordered from the least often happening first
            if let Err(e) = futures::select_biased! {
                _ = is_done => {
                    Err(NetworkTypeError::Other("session writer has been closed".into()))
                },
                _ = r_rcv_fut => {
                    self.request_missing_segments().await
                },
                _ = r_snd_fut => {
                    self.retransmit_unacknowledged_frames().await
                },
                _ = ack_fut => {
                    self.acknowledge_segments().await
                },
                 _ = evict_fut => {
                    self.frame_reassembler.evict().map_err(NetworkTypeError::from)
                },
            } {
                debug!(session_id = self.session_id, "session is closing: {e}");
                break;
            }
        }

        Ok(())
    }

    /// Returns the ID of this session.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

// Sink for data coming from downstream
impl<const C: usize> Sink<SessionMessage<C>> for SessionState<C> {
    type Error = NetworkTypeError;

    fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(mut self: Pin<&mut Self>, item: SessionMessage<C>) -> Result<(), Self::Error> {
        match item {
            SessionMessage::Segment(s) => self.consume_segment(s),
            SessionMessage::Request(r) => self.retransmit_segments(r),
            SessionMessage::Acknowledge(f) => self.acknowledged_frames(f),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.frame_reassembler.close();
        Poll::Ready(Ok(()))
    }
}

/// Represents a socket for a session between two nodes bound by the
/// underlying [network transport](AsyncWrite) and the maximum transmission unit (MTU) of `C`.
///
/// It also implements [`AsyncRead`] and [`AsyncWrite`] so that it can
/// be used on top of the usual transport stack.
pub struct SessionSocket<const C: usize> {
    state: SessionState<C>,
    frame_egress: Box<dyn AsyncRead + Send + Unpin>,
}

impl<const C: usize> SessionSocket<C> {
    /// Payload capacity is MTU minus the sizes of the Session protocol headers.
    pub const PAYLOAD_CAPACITY: usize = SessionState::<C>::PAYLOAD_CAPACITY;

    /// Maximum number of bytes that can be written in a single `poll_write` to the Session.
    pub const MAX_WRITE_SIZE: usize = SessionState::<C>::MAX_WRITE_SIZE;

    /// Create a new socket over the given underlying `transport` that binds the communicating parties.
    /// A human-readable session `id` also must be supplied.
    pub fn new<T, I>(id: I, transport: T, cfg: SessionConfig) -> Self
    where
        T: AsyncWrite + AsyncRead + Send + 'static,
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
                            trace!(session_id = id_clone, frame_id = frame.frame_id, "frame completed");
                            // The frame has been completed, so remove its retry record
                            incoming_frame_retries_clone.remove(&frame.frame_id);
                            if let Some(ack_buffer) = &is_acknowledging {
                                // Acts as a ring buffer, so if the buffer is full, any unsent acknowledgements
                                // will be discarded.
                                ack_buffer.force_push(frame.frame_id);
                            }
                        }
                        Err(SessionError::FrameDiscarded(id)) | Err(SessionError::IncompleteFrame(id)) => {
                            // Remove the retry token because the frame has been discarded
                            incoming_frame_retries_clone.remove(id);
                            warn!(session_id = id_clone, frame_id = id, "frame skipped");
                        }
                        _ => {}
                    }
                })
                .filter_map(|r| futures::future::ready(r.ok().map(Ok))) // Skip discarded frames
                .into_async_read(),
        );

        let (segment_egress_send, segment_egress_recv) = futures::channel::mpsc::unbounded();
        let segment_egress_recv = segment_egress_recv.map(|m: SessionMessage<C>| Ok(m.into_encoded()));

        let state = SessionState {
            lookbehind: Arc::new(SkipMap::new()),
            outgoing_frame_id: Arc::new(AtomicU32::new(1)),
            frame_reassembler: Arc::new(reassembler),
            outgoing_frame_resends: Arc::new(DashMap::new()),
            session_id: id.to_string(),
            to_acknowledge,
            incoming_frame_retries,
            segment_egress_send,
            cfg: cfg.clone(),
        };

        let (downstream_read, downstream_write) = transport.split();

        // Segment egress to downstream
        if let Some(rate_limit) = cfg.max_msg_per_sec.filter(|r| *r > 0).map(|r| r as u32) {
            // Apply rate-limiting for egress segments if configured
            let rate_limiter = RateLimiter::direct(Quota::per_second(rate_limit.try_into().unwrap()));
            let jitter = Jitter::up_to(Duration::from_millis(5));

            spawn(async move {
                if let Err(e) = segment_egress_recv
                    .ratelimit_stream_with_jitter(&rate_limiter, jitter)
                    .forward(downstream_write.into_sink())
                    .await
                {
                    error!("FINISHED: forwarding to downstream terminated with error {e}")
                } else {
                    debug!("FINISHED: forwarding to downstream done");
                }
            });
        } else {
            spawn(async move {
                if let Err(e) = segment_egress_recv.forward(downstream_write.into_sink()).await {
                    error!("FINISHED: forwarding to downstream terminated with error {e}")
                } else {
                    debug!("FINISHED: forwarding to downstream done");
                }
            });
        }

        // Segment ingress from downstream
        spawn(
            AsyncReadStreamer::<_, C>::new(downstream_read)
                .map_err(|e| NetworkTypeError::SessionProtocolError(SessionError::ProcessingError(e.to_string())))
                .and_then(|m| {
                    futures::future::ready(SessionMessage::try_from(m.as_ref()).map_err(NetworkTypeError::from))
                })
                .forward(state.clone()),
        );

        // Advance the state until the socket is closed
        let mut state_clone = state.clone();
        spawn(async move {
            let loop_done = state_clone.state_loop().await;
            debug!(
                session_id = state_clone.session_id,
                "FINISHED: state loop {loop_done:?}"
            );
        });

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
        tracing::trace!(
            session_id = self.state.session_id(),
            number_of_bytes = buf.len(),
            "polling write of bytes on socket reader inside session",
        );
        let mut socket_future = self.state.send_frame_data(buf).boxed();
        match Pin::new(&mut socket_future).poll(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(buf.len())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(std::io::Error::other(e))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        tracing::trace!(
            session_id = self.state.session_id(),
            "polling flush on socket reader inside session"
        );
        let inner = &mut self.state.segment_egress_send;
        pin_mut!(inner);
        inner.poll_flush(cx).map_err(std::io::Error::other)
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        tracing::trace!(
            session_id = self.state.session_id(),
            "polling close on socket reader inside session"
        );
        // TODO: fix so it calls poll_close
        self.state.segment_egress_send.close_channel();
        Poll::Ready(Ok(()))
    }
}

impl<const C: usize> AsyncRead for SessionSocket<C> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        tracing::trace!(
            session_id = self.state.session_id(),
            "polling read on socket reader inside session"
        );
        let inner = &mut self.frame_egress;
        pin_mut!(inner);
        inner.poll_read(cx, buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::DuplexIO;
    use async_std::prelude::FutureExt;
    use futures::future::Either;
    use futures::io::{AsyncReadExt, AsyncWriteExt};
    use futures::pin_mut;
    use futures::stream::BoxStream;
    use hex_literal::hex;
    use parameterized::parameterized;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    use std::fmt::Debug;
    use std::iter::Extend;
    use test_log::test;

    const MTU: usize = 466; // MTU used by HOPR

    // Using static RNG seed to make tests reproducible between different runs
    const RNG_SEED: [u8; 32] = hex!("d8a471f1c20490a3442b96fdde9d1807428096e1601b0cef0eea7e6d44a24c01");

    #[derive(Debug, Clone)]
    pub struct FaultyNetworkConfig {
        pub fault_prob: f64,
        pub mixing_factor: usize,
    }

    impl Default for FaultyNetworkConfig {
        fn default() -> Self {
            Self {
                fault_prob: 0.0,
                mixing_factor: 0,
            }
        }
    }

    pub struct FaultyNetwork<'a> {
        ingress: UnboundedSender<Box<[u8]>>,
        egress: BoxStream<'a, Box<[u8]>>,
    }

    impl AsyncWrite for FaultyNetwork<'_> {
        fn poll_write(self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
            if buf.len() > MTU {
                return Poll::Ready(Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "data length passed to downstream must be less or equal to MTU",
                )));
            }

            self.ingress.unbounded_send(buf.into()).unwrap();
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            self.ingress.close_channel();
            Poll::Ready(Ok(()))
        }
    }

    impl AsyncRead for FaultyNetwork<'_> {
        fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
            match self.egress.poll_next_unpin(cx) {
                Poll::Ready(Some(item)) => {
                    let len = buf.len().min(item.len());
                    buf[..len].copy_from_slice(&item.as_ref()[..len]);
                    Poll::Ready(Ok(item.len()))
                }
                Poll::Ready(None) => Poll::Ready(Ok(0)),
                Poll::Pending => Poll::Pending,
            }
        }
    }

    fn setup_alice_bob(
        cfg: SessionConfig,
        network_cfg: FaultyNetworkConfig,
    ) -> (SessionSocket<MTU>, SessionSocket<MTU>) {
        let (alice_reader, alice_writer) = FaultyNetwork::new(network_cfg.clone()).split();
        let (bob_reader, bob_writer) = FaultyNetwork::new(network_cfg.clone()).split();

        let alice_to_bob = SessionSocket::new("alice", DuplexIO(alice_reader, bob_writer), cfg.clone());
        let bob_to_alice = SessionSocket::new("bob", DuplexIO(bob_reader, alice_writer), cfg.clone());

        (alice_to_bob, bob_to_alice)
    }

    impl FaultyNetwork<'_> {
        pub fn new(cfg: FaultyNetworkConfig) -> Self {
            let (ingress, egress) = futures::channel::mpsc::unbounded::<Box<[u8]>>();

            let mut rng = StdRng::from_seed(RNG_SEED);
            let egress = egress.filter(move |_| futures::future::ready(rng.gen_bool(1.0 - cfg.fault_prob)));

            let egress = if cfg.mixing_factor > 0 {
                let mut rng = StdRng::from_seed(RNG_SEED);
                egress
                    .map(move |e| futures::future::ready(e).delay(Duration::from_micros(rng.gen_range(0..20))))
                    .buffer_unordered(cfg.mixing_factor)
                    .boxed()
            } else {
                egress.boxed()
            };

            Self { ingress, egress }
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
                    socket.write(&write).await?;
                    sent.extend(write);
                }
            }

            if d == Direction::Recv || d == Direction::Both {
                for _ in 0..num_frames {
                    let mut read = vec![0u8; frame_size];
                    socket.read_exact(&mut read).await?;
                    received.extend(read);
                }
            }

            // TODO: fix this so it works properly
            //socket.close().await.unwrap();

            Ok::<_, std::io::Error>((sent, received))
        };

        let alice_worker = async_std::task::spawn(socket_worker(
            alice,
            if one_way { Direction::Send } else { Direction::Both },
        ));
        let bob_worker = async_std::task::spawn(socket_worker(
            bob,
            if one_way { Direction::Recv } else { Direction::Both },
        ));

        let send_recv = futures::future::join(alice_worker, bob_worker);
        let timeout = async_std::task::sleep(timeout);

        pin_mut!(send_recv);
        pin_mut!(timeout);

        match futures::future::select(send_recv, timeout).await {
            Either::Left(((Ok((alice_sent, alice_recv)), Ok((bob_sent, bob_recv))), _)) => {
                assert_eq!(
                    hex::encode(alice_sent),
                    hex::encode(bob_recv),
                    "alice sent must be equal to bob received"
                );
                assert_eq!(
                    hex::encode(bob_sent),
                    hex::encode(alice_recv),
                    "bob sent must be equal to alice received",
                );
            }
            Either::Left(((Err(e), _), _)) => panic!("alice send recv error: {e}"),
            Either::Left(((_, Err(e)), _)) => panic!("bob send recv error: {e}"),
            Either::Right(_) => panic!("timeout"),
        }
    }

    fn spawn_single_byte_read_write<R, W>(
        mut recv: R,
        mut send: W,
        data: Vec<u8>,
    ) -> (impl Future<Output = Vec<u8>>, impl Future<Output = Vec<u8>>)
    where
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
        let len = data.len();
        let read = async_std::task::spawn(async move {
            let mut out = Vec::with_capacity(len);
            for _ in 0..len {
                let mut bytes = [0u8; 1];
                if recv.read(&mut bytes).await.unwrap() > 0 {
                    out.push(bytes[0]);
                } else {
                    break;
                }
            }
            out
        });

        let written = async_std::task::spawn(async move {
            let mut out = Vec::with_capacity(len);
            for byte in data {
                send.write(&[byte]).await.unwrap();
                out.push(byte);
            }
            send.close().await.unwrap();
            out
        });

        (read, written)
    }

    #[async_std::test]
    async fn faulty_network_mixing() {
        const MIX_FACTOR: usize = 2;
        const COUNT: usize = 20;

        let (recv, send) = FaultyNetwork::new(FaultyNetworkConfig {
            mixing_factor: MIX_FACTOR,
            ..Default::default()
        })
        .split();

        let (read, written) = spawn_single_byte_read_write(recv, send, (0..COUNT as u8).collect());
        let (read, _) = futures::future::join(read, written).await;

        for (pos, value) in read.into_iter().enumerate() {
            assert!(
                pos.abs_diff(value as usize) <= MIX_FACTOR,
                "packet must not be off from its position by more than then mixing factor"
            );
        }
    }

    #[async_std::test]
    async fn faulty_network_packet_drop() {
        const DROP: f64 = 0.3333;
        const COUNT: usize = 20;

        let (recv, send) = FaultyNetwork::new(FaultyNetworkConfig {
            fault_prob: DROP,
            ..Default::default()
        })
        .split();

        let (read, written) = spawn_single_byte_read_write(recv, send, (0..COUNT as u8).collect());
        let (read, written) = futures::future::join(read, written).await;

        let max_drop = (written.len() as f64 * (1.0 - DROP) - 2.0).floor() as usize;
        assert!(read.len() >= max_drop, "dropped more than {max_drop}: {}", read.len());
    }

    #[async_std::test]
    async fn faulty_network_reliable() {
        const COUNT: usize = 20;

        let (recv, send) = FaultyNetwork::new(Default::default()).split();

        let (read, written) = spawn_single_byte_read_write(recv, send, (0..COUNT as u8).collect());
        let (read, written) = futures::future::join(read, written).await;

        assert_eq!(read, written);
    }

    #[parameterized(num_frames = {10, 100, 1000}, frame_size = {1500, 1500, 1500})]
    #[parameterized_macro(async_std::test)]
    async fn reliable_send_recv_with_no_acks(num_frames: usize, frame_size: usize) {
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
    async fn reliable_send_recv_with_with_acks(num_frames: usize, frame_size: usize) {
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
    async fn unreliable_send_recv(num_frames: usize, frame_size: usize) {
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
    async fn unreliable_send_recv_with_mixing(num_frames: usize, frame_size: usize) {
        let cfg = SessionConfig {
            rto_base_sender: Duration::from_millis(500),
            rto_base_receiver: Duration::from_millis(10),
            frame_expiration_age: Duration::from_secs(30),
            backoff_base: 1.001,
            ..Default::default()
        };

        let net_cfg = FaultyNetworkConfig {
            fault_prob: 0.33,
            mixing_factor: 2,
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
    async fn almost_reliable_send_recv_with_mixing(num_frames: usize, frame_size: usize) {
        let cfg = SessionConfig {
            rto_base_sender: Duration::from_millis(500),
            rto_base_receiver: Duration::from_millis(10),
            frame_expiration_age: Duration::from_secs(30),
            backoff_base: 1.001,
            ..Default::default()
        };

        let net_cfg = FaultyNetworkConfig {
            fault_prob: 0.1,
            mixing_factor: 2,
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
    async fn reliable_send_recv_with_mixing(num_frames: usize, frame_size: usize) {
        let cfg = SessionConfig {
            rto_base_sender: Duration::from_millis(500),
            rto_base_receiver: Duration::from_millis(10),
            frame_expiration_age: Duration::from_secs(30),
            backoff_base: 1.001,
            ..Default::default()
        };

        let net_cfg = FaultyNetworkConfig {
            mixing_factor: 2,
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
    async fn receving_on_disconnected_network_should_timeout() {
        let cfg = SessionConfig {
            rto_base_sender: Duration::from_millis(250),
            rto_base_receiver: Duration::from_millis(300),
            frame_expiration_age: Duration::from_secs(2),
            ..Default::default()
        };

        let net_cfg = FaultyNetworkConfig {
            fault_prob: 1.0, // throws away 100% of packets
            mixing_factor: 0,
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
    async fn single_frame_resend_should_be_resent_on_unreliable_network() {
        let cfg = SessionConfig {
            rto_base_sender: Duration::from_millis(250),
            rto_base_receiver: Duration::from_millis(300),
            frame_expiration_age: Duration::from_secs(10),
            ..Default::default()
        };

        let net_cfg = FaultyNetworkConfig {
            fault_prob: 0.5, // throws away 50% of packets
            mixing_factor: 0,
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
