use futures::channel::mpsc::UnboundedSender;
use futures::stream::BoxStream;
use futures::{Sink, Stream, StreamExt};
use governor::prelude::StreamRateLimitExt;
use governor::{Quota, RateLimiter};
use pin_project::pin_project;
use std::num::NonZeroU32;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use crate::prelude::errors::SessionError;
use crate::prelude::protocol::{FrameAcknowledgements, SegmentRequest, SessionMessage};
use crate::prelude::{FrameId, Segment, SegmentId};
use crate::session::frame::SeqNum;
use crate::session::utils::skip_queue::{skip_delay_channel, DelayedItem, SkipDelayReceiver, SkipDelaySender};
use crate::session::utils::{offloaded_ringbuffer, OffloadedRbConsumer, OffloadedRbProducer, RetriedFrameId};

/// Abstraction of the [SessionSocket] state.
pub trait SocketState<'a, const C: usize> {
    /// Gets ID of this Session.
    fn session_id(&self) -> &str;

    fn event_subscriptions(&self) -> Vec<SocketStateEventDiscriminants>;

    fn control_stream(&self) -> Option<BoxStream<'a, SessionMessage<C>>>;

    /// Called when the Socket receives a new segment from Downstream.
    /// When the error is returned, the incoming segment is not passed Upstream.
    fn incoming_segment(&mut self, id: &SegmentId, segment_count: SeqNum) -> Result<(), SessionError>;

    /// Called when [segment retransmission request](SegmentRequest) is received from Downstream.
    fn incoming_retransmission_request(&mut self, request: SegmentRequest<C>) -> Result<(), SessionError>;

    /// Called when an [acknowledgement of frames](FrameAcknowledgements) is received from Downstream.
    fn incoming_acknowledged_frames(&mut self, ack: FrameAcknowledgements<C>) -> Result<(), SessionError>;

    /// Called when a complete Frame has been finalized from segments received from Downstream.
    fn frame_received(&mut self, id: FrameId) -> Result<(), SessionError>;

    /// Called when a frame could not be completed from the segments received from Downstream.
    fn frame_discarded(&mut self, id: FrameId) -> Result<(), SessionError>;

    /// Called when a segment of a Frame was sent to the Downstream.
    fn segment_sent(&mut self, segment: Segment) -> Result<(), SessionError>;
}

#[derive(Clone, strum::EnumDiscriminants)]
pub enum SocketStateEvent<const C: usize> {
    MessageReceived(SessionMessage<C>),
    FrameReceived(FrameId),
    FrameDiscarded(FrameId),
    SegmentSent(Segment),
}

pub(crate) struct StateManager<'a, const C: usize> {
    state_events_in: UnboundedSender<SocketStateEvent<C>>,
    ctl_msg_out: Option<BoxStream<'a, SessionMessage<C>>>,
}

impl<'a, const C: usize> StateManager<'a, C> {
    pub fn new<S: SocketState<'a, C> + Send + 'static>(state: S) -> Self {
        let (state_events_in, state_events_out) = futures::channel::mpsc::unbounded();
        let ctl_msg_out = state.control_stream();

        // Pass all the socket state events into the SocketState
        hopr_async_runtime::prelude::spawn(state_events_out.map(Ok).forward(StateSink(state)));

        Self {
            state_events_in,
            ctl_msg_out,
        }
    }

    pub fn state_events(&self) -> UnboundedSender<SocketStateEvent<C>> {
        self.state_events_in.clone()
    }

    pub fn control_stream(&mut self) -> Option<BoxStream<'a, SessionMessage<C>>> {
        self.ctl_msg_out.take()
    }
}

/// Represents a stateless Session socket.
/// Does nothing by default, only logs warnings and events for tracing.
#[derive(Clone)]
pub struct Stateless<const C: usize>(String);

impl<'a, const C: usize> SocketState<'a, C> for Stateless<C> {
    fn session_id(&self) -> &str {
        &self.0
    }

    fn event_subscriptions(&self) -> Vec<SocketStateEventDiscriminants> {
        vec![
            SocketStateEventDiscriminants::FrameReceived,
            SocketStateEventDiscriminants::FrameDiscarded,
        ]
    }

    fn control_stream(&self) -> Option<BoxStream<'a, SessionMessage<C>>> {
        None
    }

    fn incoming_segment(&mut self, id: &SegmentId, _segment_count: SeqNum) -> Result<(), SessionError> {
        Ok(())
    }

    fn incoming_retransmission_request(&mut self, _request: SegmentRequest<C>) -> Result<(), SessionError> {
        Ok(())
    }

    fn incoming_acknowledged_frames(&mut self, _ack: FrameAcknowledgements<C>) -> Result<(), SessionError> {
        Ok(())
    }

    fn frame_received(&mut self, id: FrameId) -> Result<(), SessionError> {
        tracing::trace!(session_id = self.session_id(), %id, "frame completed");
        Ok(())
    }

    fn frame_discarded(&mut self, id: FrameId) -> Result<(), SessionError> {
        tracing::warn!(session_id = self.session_id(), %id, "frame discarded");
        Ok(())
    }

    fn segment_sent(&mut self, segment: Segment) -> Result<(), SessionError> {
        Ok(())
    }
}

/// Wraps a [`SocketState`] into a [`futures::Sink`] that consumes [`SocketStateEvent`]
/// and emits [`SessionError`].
#[pin_project]
pub(crate) struct StateSink<const C: usize, S>(S);

impl<'a, const C: usize, S: SocketState<'a, C>> Sink<SocketStateEvent<C>> for StateSink<C, S> {
    type Error = SessionError;

    fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: SocketStateEvent<C>) -> Result<(), Self::Error> {
        let this = self.project();
        match item {
            SocketStateEvent::MessageReceived(SessionMessage::Segment(s)) => {
                this.0.incoming_segment(&s.id(), s.seq_len)
            }
            SocketStateEvent::MessageReceived(SessionMessage::Request(r)) => this.0.incoming_retransmission_request(r),
            SocketStateEvent::MessageReceived(SessionMessage::Acknowledge(a)) => this.0.incoming_acknowledged_frames(a),
            SocketStateEvent::FrameReceived(id) => this.0.frame_received(id),
            SocketStateEvent::FrameDiscarded(id) => this.0.frame_discarded(id),
            SocketStateEvent::SegmentSent(s) => this.0.segment_sent(s),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcknowledgementStateConfig {}

/// Represents a Session socket state is able to process acknowledgements.
#[derive(Clone)]
pub struct AcknowledgementState<const C: usize> {
    id: String,
    cfg: AcknowledgementStateConfig,
    rb_tx: OffloadedRbProducer<Segment>,
    rb_rx: OffloadedRbConsumer<Segment>,
    ctl_tx: futures::channel::mpsc::UnboundedSender<SessionMessage<C>>,
    incoming_frame_retries_tx: SkipDelaySender<RetriedFrameId>,
    outgoing_frame_retries_tx: SkipDelaySender<RetriedFrameId>,
}

impl<const C: usize> AcknowledgementState<C> {
    pub fn new<I: std::fmt::Display>(
        session_id: I,
        ctl_tx: futures::channel::mpsc::UnboundedSender<SessionMessage<C>>,
        cfg: AcknowledgementStateConfig,
    ) -> Self {
        let id = session_id.to_string();

        let (incoming_frame_retries_tx, incoming_frame_retries_rx) = skip_delay_channel();
        let (outgoing_frame_retries_tx, outgoing_frame_retries_rx) = skip_delay_channel();
        let (rb_tx, rb_rx) = offloaded_ringbuffer(1024);

        // Acknowledgements get a special channel with fixed capacity
        let (mut ack_tx, ack_rx) = futures::channel::mpsc::channel(200_000);

        // Incoming segment resend requests (that are incomplete for too long)
        /*let mut incoming_frame_retries_tx_clone = incoming_frame_retries_tx.clone();
        let ctl_tx_clone = ctl_tx.clone();
        hopr_async_runtime::prelude::spawn(async move {
            if let Err(error) = incoming_frame_retries_rx
                .map(|rf| {
                    let frame_id = rf.frame_id;
                    // TODO: take frame max_retries from the config
                    // Find out if the frame's segments should be requested
                    if let Some(next) = rf.next(5) {
                        // Register the next retry if still possible
                        let retry_at = next.at();
                        if let Err(error) = incoming_frame_retries_tx_clone.send_one((next, retry_at)) {
                            tracing::error!(frame_id, %error, "failed to register next resend of incoming frame");
                        } else {
                            tracing::debug!(frame_id, ?retry_at, "next resend request of incoming frame segments");
                        }
                    }
                    (frame_id, frame_inspector.missing_segments(&frame_id).unwrap_or_default())
                })
                .ready_chunks(SegmentRequest::<C>::MAX_ENTRIES)
                .map(|a| Ok(SessionMessage::<C>::Request(a.into_iter().collect())))
                .forward(ctl_tx_clone)
                .await {
                tracing::error!(%error, "error while processing incoming frame resends");
            } else {
                tracing::trace!("incoming segment resends processing done");
            }
        });*/

        // Forward acknowledged frames chunked as a Control messages
        let ctl_tx_clone = ctl_tx.clone();
        let id_clone = id.clone();
        hopr_async_runtime::prelude::spawn(async move {
            // TODO: chunk size and rate limit should be configurable
            let ack_rate_limiter = RateLimiter::direct(Quota::per_second(NonZeroU32::MIN));
            if let Err(error) = ack_rx
                .ready_chunks(FrameAcknowledgements::<C>::MAX_ACK_FRAMES)
                .map(|v| Ok(SessionMessage::Acknowledge(v.into())))
                .ratelimit_stream(&ack_rate_limiter)
                .forward(ctl_tx_clone)
                .await
            {
                tracing::error!(session_id = id_clone, %error, "acknowledgement forwarding failed");
            } else {
                tracing::trace!(session_id = id_clone, "acknowledgement forwarding done");
            }
        });

        // Task for outgoing frame retries
        let mut outgoing_frame_retries_tx_clone = outgoing_frame_retries_tx.clone();
        let ctl_tx_clone = ctl_tx.clone();
        let rb_rx_clone = rb_rx.clone();
        hopr_async_runtime::prelude::spawn(async move {
            if let Err(error) = outgoing_frame_retries_rx
                .map(move |rf: RetriedFrameId| {
                    // TODO: take frame max_retries from the config
                    // Find out if the frame can be retried
                    let frame_id = rf.frame_id;
                    if let Some(next) = rf.next(5) {
                        // Register the next retry if still possible
                        let retry_at = next.at();
                        if let Err(error) = outgoing_frame_retries_tx_clone.send_one((next, retry_at)) {
                            tracing::error!(frame_id, %error, "failed to register next retry of frame");
                        } else {
                            tracing::debug!(frame_id, ?retry_at, "next resend of outgoing frame");
                        }
                    }
                    frame_id
                })
                .flat_map(move |frame_id|
                    // Find out all the segments of that frame to be retransmitted
                    futures::stream::iter(rb_rx_clone.find(|s: &Segment| s.id().0 == frame_id))
                        .map(|s| Ok(SessionMessage::<C>::Segment(s))))
                .forward(ctl_tx_clone) // Retransmit all the segments
                .await
            {
                tracing::error!(%error, "error while processing outgoing frame retries");
            } else {
                tracing::trace!("outgoing frame retries processing done");
            }
        });

        Self {
            id,
            cfg,
            rb_tx,
            rb_rx,
            ctl_tx,
            incoming_frame_retries_tx,
            outgoing_frame_retries_tx,
        }
    }
}

impl<'a, const C: usize> SocketState<'a, C> for AcknowledgementState<C> {
    fn session_id(&self) -> &str {
        &self.id
    }

    fn event_subscriptions(&self) -> Vec<SocketStateEventDiscriminants> {
        // TODO: make this config specific
        vec![
            SocketStateEventDiscriminants::FrameReceived,
            SocketStateEventDiscriminants::FrameDiscarded,
            SocketStateEventDiscriminants::SegmentSent,
            SocketStateEventDiscriminants::MessageReceived,
        ]
    }

    fn control_stream(&self) -> Option<BoxStream<'a, SessionMessage<C>>> {
        todo!()
    }

    fn incoming_segment(&mut self, id: &SegmentId, segment_count: SeqNum) -> Result<(), SessionError> {
        tracing::trace!(%id, "RECEIVED: segment");

        // Register future requesting of segments for this frame
        let first_request = Duration::from_secs(1); // TODO: take this from the config
        if let Err(error) = self.incoming_frame_retries_tx.send_one((id.0.into(), first_request)) {
            tracing::error!(frame_id = id.0, %error, "failed to register incoming retry for frame");
        }

        Ok(())
    }

    fn incoming_retransmission_request(&mut self, request: SegmentRequest<C>) -> Result<(), SessionError> {
        let missing = request.into_iter().collect::<Vec<_>>();

        // Perform a single find to lock the RB only once
        let segments = self.rb_rx.find(|s| missing.contains(&s.id()));

        // Partially acknowledged frames will not need to be fully resent in the future
        if let Err(error) = self.outgoing_frame_retries_tx.send_many(
            missing
                .into_iter()
                .map(|seg_id| (RetriedFrameId::new(seg_id.0), Duration::from_secs(1)).into()),
        ) {
            tracing::error!(%error, "failed to cancel frame resend of partially acknowledged frames");
        }

        segments
            .into_iter()
            .try_for_each(|s| self.ctl_tx.unbounded_send(SessionMessage::Segment(s)))
            .map_err(|e| SessionError::ProcessingError(e.to_string()))
    }

    fn incoming_acknowledged_frames(&mut self, ack: FrameAcknowledgements<C>) -> Result<(), SessionError> {
        // Frame acknowledged, we will not need to resend it
        if let Err(error) = self.outgoing_frame_retries_tx.send_many(
            ack.into_iter()
                .inspect(|frame_id| tracing::trace!(frame_id, "frame acknowledged"))
                .map(|frame_id| DelayedItem::Cancel(RetriedFrameId::new(frame_id))),
        ) {
            tracing::error!(%error, "failed to cancel frame resend");
        }

        Ok(())
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

    fn segment_sent(&mut self, segment: Segment) -> Result<(), SessionError> {
        // TODO: make sure segments that we resend do not make it back into the RB
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
}
