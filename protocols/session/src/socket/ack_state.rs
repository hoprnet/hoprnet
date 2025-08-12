//! This module defines the [`SocketState`] that turns [`SessionSocket`](super::SessionSocket) into
//! a reliable socket, with segment/frame retransmission and frame acknowledgements.

use std::{
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};

use futures::{FutureExt, StreamExt, channel::mpsc::UnboundedSender};
use futures_time::stream::StreamExt as TimeStreamExt;
use tracing::Instrument;

use crate::{
    errors::SessionError,
    frames::{FrameId, Segment, SegmentId, SeqIndicator},
    processing::types::FrameInspector,
    protocol::{FrameAcknowledgements, SegmentRequest, SessionMessage},
    socket::{SocketState, state::SocketComponents},
    utils::{
        RetriedFrameId, RingBufferProducer, RingBufferView, next_deadline_with_backoff, searchable_ringbuffer,
        skip_queue::{Skip, SkipDelaySender, skip_delay_channel},
    },
};

/// Indicates the acknowledgement mode of a [stateful](AcknowledgementState) Session socket.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AcknowledgementMode {
    /// Partial frames are acknowledged, leading to receiver-driven retransmission requests.
    /// The frame sender is never going to retransmit the entire frame.
    Partial,
    /// Only full frames are acknowledged, leading to the full-frame retransmission if
    /// no acknowledgement is received by the frame sender.
    Full,
    /// Both partial and full acknowledgements are sent by the receiver.
    ///
    /// If a frame is partially acknowledged first, only receiver-driven retransmission requests follow
    /// (as with [`AcknowledgementMode::Partial`].
    ///
    /// If the frame sender receives no acknowledgement (partial nor full), it retransmits
    /// the entire frame (as in [`AcknowledgementMode::Full`]).
    #[default]
    Both,
}

impl AcknowledgementMode {
    /// Indicates if `self` is [`AcknowledgementMode::Partial`] or [`AcknowledgementMode::Both`].
    #[inline]
    fn is_partial_ack_enabled(&self) -> bool {
        matches!(self, Self::Partial | Self::Both)
    }

    /// Indicates if `self` is [`AcknowledgementMode::Full`] or [`AcknowledgementMode::Both`].
    #[inline]
    fn is_full_ack_enabled(&self) -> bool {
        matches!(self, Self::Full | Self::Both)
    }
}

/// Configuration object of the [`AcknowledgementState`].
#[derive(Debug, Clone, Copy, PartialEq, smart_default::SmartDefault)]
pub struct AcknowledgementStateConfig {
    /// Mode of frame acknowledgement.
    ///
    /// Default is [`AcknowledgementMode::Both`]
    pub mode: AcknowledgementMode,

    /// The expected (average) latency of a packet (= single frame segment).
    ///
    /// Default is 20 ms
    #[default(Duration::from_millis(20))]
    pub expected_packet_latency: Duration,

    /// Backoff base applied for segment or frame retransmissions.
    ///
    /// Default is 1.2
    #[default(1.2)]
    pub backoff_base: f64,

    /// The maximum number of receiver-driven segment retransmission requests.
    ///
    /// Default is 3
    #[default(3)]
    pub max_incoming_frame_retries: usize,

    /// The maximum number of sender-driven full-frame retransmissions.
    ///
    /// Default is 3
    #[default(3)]
    pub max_outgoing_frame_retries: usize,

    /// Delay between acknowledgement batches.
    ///
    /// Default is 50 ms
    #[default(Duration::from_millis(50))]
    pub acknowledgement_delay: Duration,

    /// Number of segments to hold back for retransmission upon other party's request.
    /// Minimum is 1024.
    ///
    /// Default is 16 384.
    #[default(16384)]
    pub lookbehind_segments: usize,
}

impl AcknowledgementStateConfig {
    fn normalize(self) -> AcknowledgementStateConfig {
        Self {
            mode: self.mode,
            expected_packet_latency: self.expected_packet_latency.max(Duration::from_millis(1)),
            backoff_base: self.backoff_base.max(1.0),
            max_incoming_frame_retries: self.max_incoming_frame_retries,
            max_outgoing_frame_retries: self.max_outgoing_frame_retries,
            acknowledgement_delay: self.acknowledgement_delay.max(Duration::from_millis(1)),
            lookbehind_segments: self.lookbehind_segments.max(1024),
        }
    }
}

#[derive(Clone)]
struct AcknowledgementStateContext<const C: usize> {
    rb_tx: RingBufferProducer<Segment>,
    rb_rx: RingBufferView<Segment>,
    incoming_frame_retries_tx: SkipDelaySender<RetriedFrameId>,
    outgoing_frame_retries_tx: SkipDelaySender<RetriedFrameId>,
    ack_tx: futures::channel::mpsc::Sender<FrameId>,
    inspector: FrameInspector,
    ctl_tx: UnboundedSender<SessionMessage<C>>,
}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// Represents a Session socket state is able to process acknowledgements.
///
/// # Retransmission driven by the Receiver
/// ```mermaid
/// sequenceDiagram
///     Note over Sender,Receiver: Frame 1
///     rect rgb(191, 223, 255)
///     Note left of Sender: Frame 1 in buffer
///     Sender->>Receiver: Segment 1/3 of Frame 1
///     Sender->>Receiver: Segment 2/3 of Frame 1
///     Sender--xReceiver: Segment 3/3 of Frame 1
///     Note right of Receiver: RTO_BASE_RECEIVER elapsed
///     Receiver->>Sender: Request Segment 3 of Frame 1
///     Sender->>Receiver: Segment 3/3 of Frame 1
///     Receiver->>Sender: Acknowledge Frame 1
///     Note left of Sender: Frame 1 dropped from buffer
///     end
///     Note over Sender,Receiver: Frame 1 delivered
/// ```
///
/// # Retransmission driven by the Sender
/// ```mermaid
/// sequenceDiagram
///     Note over Sender,Receiver: Frame 1
///     rect rgb(191, 223, 255)
///     Note left of Sender: Frame 1 in buffer
///     Sender->>Receiver: Segment 1/3 of Frame 1
///     Sender->>Receiver: Segment 2/3 of Frame 1
///     Sender--xReceiver: Segment 3/3 of Frame 1
///     Note right of Receiver: RTO_BASE_RECEIVER elapsed
///     Receiver--xSender: Request Segment 3 of Frame 1
///     Note left of Sender: RTO_BASE_SENDER elapsed
///     Sender->>Receiver: Segment 1/3 of Frame 1
///     Sender->>Receiver: Segment 2/3 of Frame 1
///     Sender->>Receiver: Segment 3/3 of Frame 1
///     Receiver->>Sender: Acknowledge Frame 1
///     Note left of Sender: Frame 1 dropped from buffer
///     end
///     Note over Sender,Receiver: Frame 1 delivered
/// ```
///
/// # Sender-Receiver retransmission handover
///
/// ```mermaid
///    sequenceDiagram
///     Note over Sender,Receiver: Frame 1
///     rect rgb(191, 223, 255)
///     Note left of Sender: Frame 1 in buffer
///     Sender->>Receiver: Segment 1/3 of Frame 1
///     Sender--xReceiver: Segment 2/3 of Frame 1
///     Sender--xReceiver: Segment 3/3 of Frame 1
///     Note right of Receiver: RTO_BASE_RECEIVER elapsed
///     Receiver->>Sender: Request Segments 2,3 of Frame 1
///     Note left of Sender: RTO_BASE_SENDER cancelled
///     Sender->>Receiver: Segment 2/3 of Frame 1
///     Sender--xReceiver: Segment 3/3 of Frame 1
///     Note right of Receiver: RTO_BASE_RECEIVER elapsed
///     Receiver--xSender: Request Segments 3 of Frame 1
///     Note right of Receiver: RTO_BASE_RECEIVER elapsed
///     Receiver->>Sender: Request Segments 3 of Frame 1
///     Sender->>Receiver: Segment 3/3 of Frame 1
///     Receiver->>Sender: Acknowledge Frame 1
///     Note left of Sender: Frame 1 dropped from buffer
///     end
///     Note over Sender,Receiver: Frame 1 delivered
/// ```
///
/// # Retransmission failure
///
/// ```mermaid
///    sequenceDiagram
///     Note over Sender,Receiver: Frame 1
///     rect rgb(191, 223, 255)
///     Note left of Sender: Frame 1 in buffer
///     Sender->>Receiver: Segment 1/3 of Frame 1
///     Sender->>Receiver: Segment 2/3 of Frame 1
///     Sender--xReceiver: Segment 3/3 of Frame 1
///     Note right of Receiver: RTO_BASE_RECEIVER elapsed
///     Receiver--xSender: Request Segment 3 of Frame 1
///     Note left of Sender: RTO_BASE_SENDER elapsed
///     Sender--xReceiver: Segment 1/3 of Frame 1
///     Sender--xReceiver: Segment 2/3 of Frame 1
///     Sender--xReceiver: Segment 3/3 of Frame 1
///     Note left of Sender: FRAME_MAX_AGE elapsed<br/>Frame 1 dropped from buffer
///     Note right of Receiver: FRAME_MAX_AGE elapsed<br/>Frame 1 dropped from buffer
///     end
///     Note over Sender,Receiver: Frame 1 never delivered
/// ```
#[derive(Clone)]
pub struct AcknowledgementState<const C: usize> {
    id: String,
    cfg: AcknowledgementStateConfig,
    context: Option<AcknowledgementStateContext<C>>,
    started: std::sync::Arc<AtomicBool>,
}

impl<const C: usize> AcknowledgementState<C> {
    pub fn new<I: std::fmt::Display>(session_id: I, cfg: AcknowledgementStateConfig) -> Self {
        Self {
            id: session_id.to_string(),
            cfg: cfg.normalize(),
            context: Default::default(),
            started: std::sync::Arc::new(AtomicBool::new(false)),
        }
    }
}

impl<const C: usize> SocketState<C> for AcknowledgementState<C> {
    fn session_id(&self) -> &str {
        &self.id
    }

    #[tracing::instrument(name = "AcknowledgementState", skip(self, socket_components), fields(session_id = self.id))]
    fn run(&mut self, socket_components: SocketComponents<C>) -> Result<(), SessionError> {
        if self.started.load(std::sync::atomic::Ordering::Relaxed) && self.context.is_some() {
            return Err(SessionError::InvalidState("state is already running".into()));
        }

        let (incoming_frame_retries_tx, incoming_frame_retries_rx) = skip_delay_channel();
        let (outgoing_frame_retries_tx, outgoing_frame_retries_rx) = skip_delay_channel();
        let (rb_tx, rb_rx) = searchable_ringbuffer(self.cfg.lookbehind_segments);

        // Full frame acknowledgements get a special channel with fixed capacity
        let (ack_tx, ack_rx) = futures::channel::mpsc::channel(2 * self.cfg.lookbehind_segments);

        let context = self.context.insert(AcknowledgementStateContext {
            rb_tx,
            rb_rx,
            incoming_frame_retries_tx,
            outgoing_frame_retries_tx,
            ack_tx,
            ctl_tx: socket_components.ctl_tx,
            inspector: socket_components
                .inspector
                .ok_or(SessionError::InvalidState("inspector is not available".into()))?,
        });

        if self.cfg.mode.is_partial_ack_enabled() {
            // For partially received frames incomplete for too long,
            // missing segments will be asked for retransmission
            let mut incoming_frame_retries_tx_clone = context.incoming_frame_retries_tx.clone();
            let ctl_tx_clone = context.ctl_tx.clone();
            let frame_inspector_clone = context.inspector.clone();
            let cfg = self.cfg;
            hopr_async_runtime::prelude::spawn(incoming_frame_retries_rx
                .filter_map(move |rf| {
                    let frame_id = rf.frame_id;
                    let missing_segments = frame_inspector_clone.missing_segments(&frame_id).unwrap_or_default();
                    if !missing_segments.is_empty() {
                        // Find out if we need to subscribe for further retries of this Frame
                        if let Some(next) = rf.next() {
                            // Register the next retry if still possible
                            let retry_at = next_deadline_with_backoff(next.retry_count, cfg.backoff_base, cfg.expected_packet_latency);
                            if let Err(error) = incoming_frame_retries_tx_clone.send_one((next, retry_at)) {
                                tracing::error!(frame_id, %error, "failed to register next resend of incoming frame");
                            } else {
                                tracing::debug!(frame_id, retry_in = ?retry_at.saturating_duration_since(Instant::now()), "next resend request of incoming frame segments");
                            }
                        } else {
                            tracing::debug!(frame_id, "last request of incoming frame segments");
                        }

                        futures::future::ready(Some((frame_id, missing_segments)))
                    } else {
                        tracing::debug!(frame_id, "no more missing segments in frame");
                        futures::future::ready(None)
                    }
                })
                .ready_chunks(SegmentRequest::<C>::MAX_ENTRIES)
                .inspect(|r| tracing::trace!(req = ?r, "requesting segments resend"))
                .map(|a| Ok(SessionMessage::<C>::Request(a.into_iter().collect())))
                .forward(ctl_tx_clone)
                .map(move |res| match res {
                    Ok(_) => tracing::debug!("incoming frame resends processing done"),
                    Err(error) => tracing::error!(%error, "error while processing incoming frame resends")
                })
                .instrument(tracing::debug_span!("incoming_frame_retries_sender"))
            );
        }

        // Send out Frame Acknowledgements chunked as Control messages
        let ctl_tx_clone = context.ctl_tx.clone();
        let ack_delay = self.cfg.acknowledgement_delay;
        hopr_async_runtime::prelude::spawn(
            ack_rx
                .buffer(futures_time::time::Duration::from(ack_delay))
                .flat_map(|acks| futures::stream::iter(FrameAcknowledgements::<C>::new_multiple(acks)))
                .filter(|acks| futures::future::ready(!acks.is_empty()))
                .inspect(|acks| tracing::trace!(?acks, "acknowledgements sent"))
                .map(|acks| Ok(SessionMessage::<C>::Acknowledge(acks)))
                .forward(ctl_tx_clone)
                .map(move |res| match res {
                    Ok(_) => tracing::debug!("acknowledgement forwarding done"),
                    Err(error) => tracing::debug!(%error, "acknowledgement forwarding failed"),
                })
                .instrument(tracing::debug_span!("acknowledgement_sender")),
        );

        // Resend outgoing frame Segments if they were not (partially or fully) acknowledged
        let mut outgoing_frame_retries_tx_clone = context.outgoing_frame_retries_tx.clone();
        let ctl_tx_clone = context.ctl_tx.clone();
        let rb_rx_clone = context.rb_rx.clone();
        let cfg = self.cfg;
        hopr_async_runtime::prelude::spawn(
            outgoing_frame_retries_rx
                .map(move |rf: RetriedFrameId| {
                    // Find out if the frame can be retried again in the future
                    let frame_id = rf.frame_id;
                    if let Some(next) = rf.next() {
                        // Register the next retry if still possible
                        let retry_at =
                            next_deadline_with_backoff(next.retry_count, cfg.backoff_base, cfg.expected_packet_latency);
                        if let Err(error) = outgoing_frame_retries_tx_clone.send_one((next, retry_at)) {
                            tracing::error!(frame_id, %error, "failed to register next retry of frame");
                        } else {
                            tracing::debug!(frame_id, retry_in = ?retry_at.saturating_duration_since(Instant::now()), "next resend of outgoing frame");
                        }
                    } else {
                        tracing::debug!(frame_id, "last outgoing retry of frame");
                    }
                    tracing::trace!(frame_id, "going to re-send entire frame");
                    frame_id
                })
                .flat_map(move |frame_id| {
                    // Find out all the segments of that frame to be retransmitted
                    futures::stream::iter(
                        rb_rx_clone
                            .find(|s: &Segment| s.id().0 == frame_id)
                            .into_iter()
                            .inspect(|s| tracing::trace!(seg_id = %s.id(), "segment retransmit"))
                            .map(|s| Ok(SessionMessage::<C>::Segment(s))),
                    )
                })
                .forward(ctl_tx_clone) // Retransmit all the segments
                .map(move |res| match res {
                    Ok(_) => tracing::debug!("outgoing frame retries processing done"),
                    Err(error) => tracing::error!(%error, "error while processing outgoing frame retries"),
                })
                .instrument(tracing::debug_span!("outgoing_frame_retries_sender")),
        );

        tracing::debug!("acknowledgement state has been started");
        self.started.store(true, std::sync::atomic::Ordering::Relaxed);

        Ok(())
    }

    #[tracing::instrument(name = "AcknowledgementState::stop", skip(self), fields(session_id = self.id))]
    fn stop(&mut self) -> Result<(), SessionError> {
        if let Some(mut ctx) = self.context.take() {
            ctx.outgoing_frame_retries_tx.force_close();
            ctx.incoming_frame_retries_tx.force_close();
            ctx.ack_tx.close_channel();
            ctx.ctl_tx.close_channel();

            self.started.store(false, std::sync::atomic::Ordering::Relaxed);
            tracing::debug!("state has been stopped");
        } else {
            tracing::warn!("cannot be stopped, because it is not running");
        }

        Ok(())
    }

    #[tracing::instrument(name = "AcknowledgementState::incoming_segment", skip(self), fields(session_id = self.id, frame_id = seg_id.0))]
    fn incoming_segment(&mut self, seg_id: &SegmentId, _ind: SeqIndicator) -> Result<(), SessionError> {
        tracing::trace!("segment received");

        let ctx = self
            .started
            .load(std::sync::atomic::Ordering::Relaxed)
            .then_some(self.context.as_mut())
            .flatten()
            .ok_or(SessionError::StateNotRunning)?;

        // Register future requesting of segments for this frame
        if self.cfg.mode.is_partial_ack_enabled() {
            // Every incoming segment of this frame will move the deadline further
            // into the future.
            if let Err(error) = ctx.incoming_frame_retries_tx.send_one((
                RetriedFrameId::with_retries(seg_id.0, self.cfg.max_incoming_frame_retries),
                self.cfg.expected_packet_latency, // RTO_BASE_RECEIVER - when we expect the next segment to arrive
            )) {
                tracing::error!(%error, "failed to register incoming retry for frame");
            }
        }
        Ok(())
    }

    #[tracing::instrument(name = "AcknowledgementState::incoming_retransmission_request", skip(self, request), fields(session_id = self.id))]
    fn incoming_retransmission_request(&mut self, request: SegmentRequest<C>) -> Result<(), SessionError> {
        // The state will respond to segment retransmission requests even
        // if it has this feature disabled in the config.
        tracing::trace!(count = request.len(), "segment retransmission requested");

        let ctx = self
            .started
            .load(std::sync::atomic::Ordering::Relaxed)
            .then_some(self.context.as_mut())
            .flatten()
            .ok_or(SessionError::StateNotRunning)?;

        let (mut missing_seg_ids, mut missing_frame_ids): (Vec<_>, Vec<_>) =
            request.into_iter().map(|s| (s, s.0)).unzip();

        // Perform a single find to lock the RB only once
        let segments = ctx.rb_rx.find(|s| {
            // SegmentIds are guaranteed to be sorted, so we can use binary search
            if let Ok(i) = missing_seg_ids.binary_search(&s.id()) {
                missing_seg_ids.remove(i);
                true
            } else {
                false
            }
        });

        tracing::trace!(
            found = segments.len(),
            requested = missing_frame_ids.len(),
            "found matching segments to be retransmitted"
        );

        // Partially acknowledged frames will not need to be fully resent in the future.
        // Cancel all partially acknowledged frame resends.
        if self.cfg.mode.is_full_ack_enabled() {
            // Since the FrameIds are guaranteed to be sorted, we can simply dedup them.
            missing_frame_ids.dedup();

            if let Err(error) = ctx.outgoing_frame_retries_tx.send_many(
                missing_frame_ids
                    .into_iter()
                    .map(|frame_id| (RetriedFrameId::no_retries(frame_id), Skip).into()),
            ) {
                tracing::error!(%error, "failed to cancel frame resend of partially acknowledged frames");
            }
        }

        // Resend the segments via the Control Stream
        segments
            .into_iter()
            .try_for_each(|s| {
                tracing::trace!(seg_id = %s.id(), "retransmit segment on request");
                ctx.ctl_tx.unbounded_send(SessionMessage::Segment(s))
            })
            .map_err(|e| SessionError::ProcessingError(e.to_string()))
    }

    #[tracing::instrument(name = "AcknowledgementState::incoming_acknowledged_frames", skip(self), fields(session_id = self.id))]
    fn incoming_acknowledged_frames(&mut self, ack: FrameAcknowledgements<C>) -> Result<(), SessionError> {
        tracing::trace!(count = ack.len(), "frame acknowledgements received");

        let ctx = self
            .started
            .load(std::sync::atomic::Ordering::Relaxed)
            .then_some(self.context.as_mut())
            .flatten()
            .ok_or(SessionError::StateNotRunning)?;

        // Frame acknowledged, we will not need to resend it
        if self.cfg.mode.is_full_ack_enabled()
            && let Err(error) = ctx.outgoing_frame_retries_tx.send_many(
                ack.into_iter()
                    .inspect(|frame_id| tracing::trace!(frame_id, "frame acknowledged"))
                    .map(|frame_id| (RetriedFrameId::no_retries(frame_id), Skip).into()),
            )
        {
            tracing::error!(%error, "failed to cancel frame resend");
        }

        Ok(())
    }

    #[tracing::instrument(name = "AcknowledgementState::frame_complete", skip(self), fields(session_id = self.id))]
    fn frame_complete(&mut self, frame_id: FrameId) -> Result<(), SessionError> {
        tracing::trace!("frame complete");

        let ctx = self
            .started
            .load(std::sync::atomic::Ordering::Relaxed)
            .then_some(self.context.as_mut())
            .flatten()
            .ok_or(SessionError::StateNotRunning)?;

        // Since the frame has been completed, push its ID into the acknowledgement queue
        if let Err(error) = ctx.ack_tx.try_send(frame_id) {
            tracing::error!(%error, "failed to acknowledge frame");
        }

        if self.cfg.mode.is_partial_ack_enabled() {
            // No more requesting of segment retransmissions from frames that were completed
            if let Err(error) = ctx
                .incoming_frame_retries_tx
                .send_one((RetriedFrameId::no_retries(frame_id), Skip))
            {
                tracing::error!(%error, "failed to cancel retry of acknowledged frame");
            }
        }

        Ok(())
    }

    #[tracing::instrument(name = "AcknowledgementState::frame_emitted", skip(self), fields(session_id = self.id))]
    fn frame_emitted(&mut self, id: FrameId) -> Result<(), SessionError> {
        tracing::trace!("frame emitted");
        let _ = self
            .started
            .load(std::sync::atomic::Ordering::Relaxed)
            .then_some(self.context.as_mut())
            .flatten()
            .ok_or(SessionError::StateNotRunning)?;
        Ok(())
    }

    #[tracing::instrument(name = "AcknowledgementState::frame_discarded", skip(self), fields(session_id = self.id))]
    fn frame_discarded(&mut self, frame_id: FrameId) -> Result<(), SessionError> {
        tracing::trace!("frame discarded");

        let ctx = self
            .started
            .load(std::sync::atomic::Ordering::Relaxed)
            .then_some(self.context.as_mut())
            .flatten()
            .ok_or(SessionError::StateNotRunning)?;

        if self.cfg.mode.is_partial_ack_enabled() {
            // No more requesting of segment retransmissions from frames that were discarded
            if let Err(error) = ctx
                .incoming_frame_retries_tx
                .send_one((RetriedFrameId::no_retries(frame_id), Skip))
            {
                tracing::error!(%error, "failed to cancel retry of acknowledged frame");
            }
        }

        Ok(())
    }

    #[tracing::instrument(name = "AcknowledgementState::segment_sent", skip(self, segment), fields(session_id = self.id, frame_id = segment.frame_id, seq_idx = segment.seq_idx))]
    fn segment_sent(&mut self, segment: &Segment) -> Result<(), SessionError> {
        tracing::trace!("segment sent");

        let ctx = self
            .started
            .load(std::sync::atomic::Ordering::Relaxed)
            .then_some(self.context.as_mut())
            .flatten()
            .ok_or(SessionError::StateNotRunning)?;

        // Since segments are re-sent via Control stream, they are not later fed again
        // into the ring buffer.
        if !ctx.rb_tx.push(segment.clone()) {
            tracing::error!("failed to push segment into ring buffer");
        }

        // When the last segment of a frame has been sent,
        // add it to outgoing retries (if the full ack mode is enabled).
        if segment.is_last() && self.cfg.mode.is_full_ack_enabled() {
            tracing::trace!("last segment of frame sent");

            if let Err(error) = ctx.outgoing_frame_retries_tx.send_one((
                RetriedFrameId::with_retries(segment.frame_id, self.cfg.max_outgoing_frame_retries),
                // The whole frame should be delivered and acknowledged
                // once all its segments (seq_len) are sent,
                // and the acknowledgement also comes back to us.
                // Therefore, RTO_BASE_SENDER = latency * (seq_len + 1)
                self.cfg.expected_packet_latency * (segment.seq_flags.seq_len() + 1) as u32,
            )) {
                tracing::error!(%error, "failed to insert outgoing retry of a frame");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;

    use super::*;
    use crate::{
        frames::SeqNum,
        processing::types::{FrameBuilder, FrameDashMap, FrameMap},
        utils::segment,
    };

    const FRAME_SIZE: usize = 1500;

    const MTU: usize = 1000;

    #[test_log::test(tokio::test)]
    async fn ack_state_sender_must_acknowledge_completed_frames() -> anyhow::Result<()> {
        let cfg = AcknowledgementStateConfig {
            expected_packet_latency: Duration::from_millis(10),
            acknowledgement_delay: Duration::from_millis(2),
            ..Default::default()
        };

        let inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = futures::channel::mpsc::unbounded();

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        let acked_frame_ids = [1, 2, 3];

        for &frame_id in &acked_frame_ids {
            state.frame_complete(frame_id)?;
        }

        tokio::time::sleep(cfg.acknowledgement_delay * 2).await;

        state.stop()?;

        let ctl_msgs = tokio::time::timeout(Duration::from_millis(100), ctl_rx.collect::<Vec<_>>())
            .await
            .context("timeout receiving Control messages")?;

        assert_eq!(1, ctl_msgs.len());

        assert_eq!(
            ctl_msgs[0],
            SessionMessage::Acknowledge(acked_frame_ids.to_vec().try_into()?)
        );

        Ok(())
    }

    #[parameterized::parameterized(num_frames = { 1, 2, 3 })]
    #[parameterized_macro(test_log::test(tokio::test))]
    async fn ack_state_sender_must_resend_unacknowledged_frames(num_frames: usize) -> anyhow::Result<()> {
        const NUM_RETRIES: usize = 2;

        let cfg = AcknowledgementStateConfig {
            mode: AcknowledgementMode::Full,
            expected_packet_latency: Duration::from_millis(2),
            max_outgoing_frame_retries: NUM_RETRIES,
            ..Default::default()
        };

        let inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = futures::channel::mpsc::unbounded();

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        let mut expected_frame_segments = Vec::new();
        let num_segments_in_frame = FRAME_SIZE / MTU + 1;
        for i in 1..=num_frames {
            let expected_segments = segment(hopr_crypto_random::random_bytes::<FRAME_SIZE>(), MTU, i as FrameId)?;
            for segment in &expected_segments {
                state.segment_sent(segment)?;
            }
            expected_frame_segments.push(expected_segments);
        }

        let expected_frame_delivery = cfg.expected_packet_latency * (num_segments_in_frame + 1) as u32;
        tokio::time::sleep(2 * expected_frame_delivery).await;
        state.stop()?;

        let ctl_msg = tokio::time::timeout(Duration::from_millis(100), ctl_rx.collect::<Vec<_>>())
            .await
            .context("timeout receiving Control message")?;

        let retransmitted_segments = ctl_msg
            .into_iter()
            .map(|m| m.try_as_segment().ok_or(anyhow::anyhow!("must be segment")))
            .collect::<Result<Vec<_>, _>>()?;

        assert_eq!(
            NUM_RETRIES * num_segments_in_frame * num_frames,
            retransmitted_segments.len()
        );

        let total_segments = expected_frame_segments.iter().map(|m| m.len()).sum::<usize>();
        let expected_segments = expected_frame_segments
            .into_iter()
            .flatten()
            .cycle()
            .take(total_segments * NUM_RETRIES)
            .collect::<Vec<_>>();
        assert_eq!(expected_segments, retransmitted_segments);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn ack_state_sender_must_not_resend_unacknowledged_frame_when_full_resend_disabled() -> anyhow::Result<()> {
        const NUM_RETRIES: usize = 2;

        let cfg = AcknowledgementStateConfig {
            mode: AcknowledgementMode::Partial,
            expected_packet_latency: Duration::from_millis(2),
            max_outgoing_frame_retries: NUM_RETRIES,
            ..Default::default()
        };

        let inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = futures::channel::mpsc::unbounded();

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        let expected_segments = segment(hopr_crypto_random::random_bytes::<FRAME_SIZE>(), MTU, 1)?;
        for segment in &expected_segments {
            state.segment_sent(segment)?;
        }

        let expected_frame_delivery = cfg.expected_packet_latency * (expected_segments.len() + 1) as u32;
        tokio::time::sleep(2 * expected_frame_delivery).await;
        state.stop()?;

        // No retransmission should be sent because it is disabled
        assert!(ctl_rx.collect::<Vec<_>>().await.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn ack_state_sender_must_not_resend_acknowledged_frame() -> anyhow::Result<()> {
        let cfg = AcknowledgementStateConfig {
            mode: AcknowledgementMode::Full,
            expected_packet_latency: Duration::from_millis(2),
            max_outgoing_frame_retries: 1,
            ..Default::default()
        };

        let inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = futures::channel::mpsc::unbounded();

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        let expected_segments = segment(hopr_crypto_random::random_bytes::<{ FRAME_SIZE * 2 }>(), MTU, 1)?;
        for segment in &expected_segments {
            state.segment_sent(segment)?;
        }

        // Acknowledge the frame
        state.incoming_acknowledged_frames(vec![1].try_into()?)?;

        tokio::time::sleep(10 * cfg.expected_packet_latency).await;

        state.stop()?;

        // No retransmission should be sent because the frame was already acknowledged.
        assert!(ctl_rx.collect::<Vec<_>>().await.is_empty());

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn ack_state_sender_must_not_resend_entire_frame_when_already_partially_acknowledged() -> anyhow::Result<()> {
        let cfg = AcknowledgementStateConfig {
            mode: AcknowledgementMode::Full,
            expected_packet_latency: Duration::from_millis(2),
            max_outgoing_frame_retries: 1,
            ..Default::default()
        };

        let inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = futures::channel::mpsc::unbounded();

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        let expected_segments = segment(hopr_crypto_random::random_bytes::<{ FRAME_SIZE * 2 }>(), MTU, 1)?;

        // Load segments into the ring buffer
        for segment in &expected_segments {
            state.segment_sent(segment)?;
        }

        tokio::time::sleep(cfg.expected_packet_latency).await;

        // Partially acknowledge the frame (report the first segment as missing)
        state.incoming_retransmission_request(SegmentRequest::from_iter([(1, [0b10000000].into())]))?;

        state.stop()?;

        // Only segment 1 must be retransmitted
        let ctl_msgs = ctl_rx.collect::<Vec<_>>().await;
        assert_eq!(1, ctl_msgs.len());
        assert_eq!(ctl_msgs[0], SessionMessage::Segment(expected_segments[0].clone()));

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn ack_state_sender_must_retransmit_segments_when_requested() -> anyhow::Result<()> {
        let cfg = AcknowledgementStateConfig {
            mode: AcknowledgementMode::Full,
            expected_packet_latency: Duration::from_millis(2),
            max_outgoing_frame_retries: 1,
            ..Default::default()
        };

        let inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = futures::channel::mpsc::unbounded();

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        let expected_segments_1 = segment(hopr_crypto_random::random_bytes::<{ FRAME_SIZE * 2 }>(), MTU, 1)?;
        // Load frame 1 segments into the ring buffer
        for segment in &expected_segments_1 {
            state.segment_sent(segment)?;
        }

        let expected_segments_2 = segment(hopr_crypto_random::random_bytes::<{ FRAME_SIZE * 2 }>(), MTU, 2)?;
        // Load frame 2 segments into the ring buffer
        for segment in &expected_segments_2 {
            state.segment_sent(segment)?;
        }

        tokio::time::sleep(cfg.expected_packet_latency).await;

        // Request different segments to be retransmitted
        state.incoming_retransmission_request(SegmentRequest::from_iter([
            (1, [0b11100000].into()),
            (2, [0b11100000].into()),
        ]))?;
        tokio::time::sleep(cfg.expected_packet_latency).await;

        state.incoming_retransmission_request(SegmentRequest::from_iter([(2, [0b11000000].into())]))?;
        tokio::time::sleep(cfg.expected_packet_latency).await;

        state.incoming_retransmission_request(SegmentRequest::from_iter([(2, [0b01000000].into())]))?;
        tokio::time::sleep(cfg.expected_packet_latency).await;

        state.stop()?;

        let ctl_msgs = ctl_rx.collect::<Vec<_>>().await;

        assert_eq!(9, ctl_msgs.len());
        // Request 1 - frame 1
        assert_eq!(ctl_msgs[0], SessionMessage::Segment(expected_segments_1[0].clone()));
        assert_eq!(ctl_msgs[1], SessionMessage::Segment(expected_segments_1[1].clone()));
        assert_eq!(ctl_msgs[2], SessionMessage::Segment(expected_segments_1[2].clone()));
        // Request 1 - frame 2
        assert_eq!(ctl_msgs[3], SessionMessage::Segment(expected_segments_2[0].clone()));
        assert_eq!(ctl_msgs[4], SessionMessage::Segment(expected_segments_2[1].clone()));
        assert_eq!(ctl_msgs[5], SessionMessage::Segment(expected_segments_2[2].clone()));

        // Request 2 - frame 2
        assert_eq!(ctl_msgs[6], SessionMessage::Segment(expected_segments_2[0].clone()));
        assert_eq!(ctl_msgs[7], SessionMessage::Segment(expected_segments_2[1].clone()));

        // Request 3 - frame 2
        assert_eq!(ctl_msgs[8], SessionMessage::Segment(expected_segments_2[1].clone()));

        Ok(())
    }

    #[tokio::test]
    async fn ack_state_receiver_must_request_missing_frames_when_partial_acks_are_enabled() -> anyhow::Result<()> {
        let cfg = AcknowledgementStateConfig {
            mode: AcknowledgementMode::Partial,
            expected_packet_latency: Duration::from_millis(2),
            max_incoming_frame_retries: 1,
            ..Default::default()
        };

        let mut inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = futures::channel::mpsc::unbounded();

        let segments = segment(hopr_crypto_random::random_bytes::<FRAME_SIZE>(), MTU, 1)?;

        inspector
            .0
            .entry(1)
            .try_as_vacant()
            .ok_or(anyhow::anyhow!("frame 1 must be vacant"))?
            .insert(FrameBuilder::from(segments[0].clone()));

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        state.incoming_segment(&segments[0].id(), (segments.len() as SeqNum).try_into()?)?;

        tokio::time::sleep(cfg.expected_packet_latency * 2).await;

        state.stop()?;

        let ctl_msgs = tokio::time::timeout(Duration::from_millis(100), ctl_rx.collect::<Vec<_>>())
            .await
            .context("timeout receiving Control messages")?;

        assert_eq!(1, ctl_msgs.len());
        assert_eq!(
            ctl_msgs[0],
            SessionMessage::Request(SegmentRequest::from_iter([(1, [0b01000000].into())]))
        );

        Ok(())
    }

    #[tokio::test]
    async fn ack_state_receiver_must_not_request_missing_frames_when_partial_acks_are_disabled() -> anyhow::Result<()> {
        let cfg = AcknowledgementStateConfig {
            mode: AcknowledgementMode::Full,
            expected_packet_latency: Duration::from_millis(2),
            max_incoming_frame_retries: 1,
            ..Default::default()
        };

        let mut inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = futures::channel::mpsc::unbounded();

        let segments = segment(hopr_crypto_random::random_bytes::<FRAME_SIZE>(), MTU, 1)?;

        inspector
            .0
            .entry(1)
            .try_as_vacant()
            .ok_or(anyhow::anyhow!("frame 1 must be vacant"))?
            .insert(FrameBuilder::from(segments[0].clone()));

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.into(),
            ctl_tx,
        })?;

        state.incoming_segment(&segments[0].id(), (segments.len() as SeqNum).try_into()?)?;

        tokio::time::sleep(cfg.expected_packet_latency * 2).await;

        state.stop()?;

        let ctl_msgs = tokio::time::timeout(Duration::from_millis(100), ctl_rx.collect::<Vec<_>>())
            .await
            .context("timeout receiving Control messages")?;

        assert!(ctl_msgs.iter().all(|m| !matches!(m, SessionMessage::Request(_))));

        Ok(())
    }

    #[tokio::test]
    async fn ack_state_receiver_must_continue_requesting_missing_frames_when_frame_not_completed() -> anyhow::Result<()>
    {
        let cfg = AcknowledgementStateConfig {
            mode: AcknowledgementMode::Partial,
            expected_packet_latency: Duration::from_millis(2),
            max_incoming_frame_retries: 3,
            ..Default::default()
        };

        let mut inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = futures::channel::mpsc::unbounded();

        let segments = segment(hopr_crypto_random::random_bytes::<{ 2 * FRAME_SIZE }>(), MTU, 1)?;

        inspector
            .0
            .entry(1)
            .try_as_vacant()
            .ok_or(anyhow::anyhow!("frame 1 must be vacant"))?
            .insert(FrameBuilder::from(segments[0].clone()));

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.clone().into(),
            ctl_tx,
        })?;

        state.incoming_segment(&segments[0].id(), (segments.len() as SeqNum).try_into()?)?;

        tokio::time::sleep(cfg.expected_packet_latency * 2).await;

        inspector
            .0
            .entry(1)
            .try_as_occupied()
            .ok_or(anyhow::anyhow!("frame 1 must be occupied"))?
            .get_mut()
            .add_segment(segments[1].clone())?;

        state.incoming_segment(&segments[1].id(), (segments.len() as SeqNum).try_into()?)?;

        tokio::time::sleep(cfg.expected_packet_latency * 2).await;

        state.stop()?;

        let ctl_msgs = tokio::time::timeout(Duration::from_millis(100), ctl_rx.collect::<Vec<_>>())
            .await
            .context("timeout receiving Control messages")?;

        assert_eq!(2, ctl_msgs.len());
        assert_eq!(
            ctl_msgs[0],
            SessionMessage::Request(SegmentRequest::from_iter([(1, [0b01100000].into())]))
        );

        assert_eq!(
            ctl_msgs[1],
            SessionMessage::Request(SegmentRequest::from_iter([(1, [0b00100000].into())]))
        );

        Ok(())
    }

    #[tokio::test]
    async fn ack_state_receiver_must_continue_requesting_missing_frames_and_acknowledge_once_complete()
    -> anyhow::Result<()> {
        let cfg = AcknowledgementStateConfig {
            mode: AcknowledgementMode::Partial,
            expected_packet_latency: Duration::from_millis(2),
            max_incoming_frame_retries: 3,
            acknowledgement_delay: Duration::from_millis(5),
            ..Default::default()
        };

        let mut inspector = FrameInspector(FrameDashMap::with_capacity(10));
        let (ctl_tx, ctl_rx) = futures::channel::mpsc::unbounded();

        let segments = segment(hopr_crypto_random::random_bytes::<{ 2 * FRAME_SIZE }>(), MTU, 1)?;

        // Segment 1
        inspector
            .0
            .entry(1)
            .try_as_vacant()
            .ok_or(anyhow::anyhow!("frame 1 must be vacant"))?
            .insert(FrameBuilder::from(segments[0].clone()));

        let mut state = AcknowledgementState::<MTU>::new("test", cfg);
        state.run(SocketComponents {
            inspector: inspector.clone().into(),
            ctl_tx,
        })?;

        // Segment 2
        state.incoming_segment(&segments[0].id(), (segments.len() as SeqNum).try_into()?)?;

        tokio::time::sleep(cfg.expected_packet_latency * 2).await;

        inspector
            .0
            .entry(1)
            .try_as_occupied()
            .ok_or(anyhow::anyhow!("frame 1 must be occupied"))?
            .get_mut()
            .add_segment(segments[1].clone())?;

        state.incoming_segment(&segments[1].id(), (segments.len() as SeqNum).try_into()?)?;

        tokio::time::sleep(cfg.expected_packet_latency * 2).await;

        // Segment 3
        inspector
            .0
            .entry(1)
            .try_as_occupied()
            .ok_or(anyhow::anyhow!("frame 1 must be occupied"))?
            .get_mut()
            .add_segment(segments[2].clone())?;

        state.incoming_segment(&segments[2].id(), (segments.len() as SeqNum).try_into()?)?;
        state.frame_complete(1)?;

        tokio::time::sleep(cfg.acknowledgement_delay * 2).await;

        state.stop()?;

        let ctl_msgs = tokio::time::timeout(Duration::from_millis(100), ctl_rx.collect::<Vec<_>>())
            .await
            .context("timeout receiving Control messages")?;

        assert_eq!(3, ctl_msgs.len());
        assert_eq!(
            ctl_msgs[0],
            SessionMessage::Request(SegmentRequest::from_iter([(1, [0b01100000].into())]))
        );

        assert_eq!(
            ctl_msgs[1],
            SessionMessage::Request(SegmentRequest::from_iter([(1, [0b00100000].into())]))
        );

        assert_eq!(ctl_msgs[2], SessionMessage::Acknowledge(vec![1].try_into()?));

        Ok(())
    }
}
