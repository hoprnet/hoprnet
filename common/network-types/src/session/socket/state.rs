use crate::prelude::errors::SessionError;
use crate::prelude::protocol::{FrameAcknowledgements, SegmentRequest, SessionMessage};
use crate::prelude::{FrameId, Segment, SegmentId};
use crate::session::frame::SeqNum;
use crate::session::reassembly::FrameInspector;
use crate::session::utils::skip_queue::{skip_delay_channel, DelayedItem, Skip, SkipDelaySender};
use crate::session::utils::{searchable_ringbuffer, RetriedFrameId, RingBufferProducer, RingBufferView};
use futures::channel::mpsc::UnboundedSender;
use futures::FutureExt;
use futures::StreamExt;
use governor::prelude::StreamRateLimitExt;
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;
use std::time::Duration;

pub struct SocketComponents<const C: usize> {
    pub inspector: FrameInspector,
    pub ctl_tx: UnboundedSender<SessionMessage<C>>,
}

/// Abstraction of the [SessionSocket] state.
pub trait SocketState<'a, const C: usize> {
    /// Gets ID of this Session.
    fn session_id(&self) -> &str;

    /// Starts the necessary processes inside the state.
    /// Should be idempotent if called multiple times.
    fn run(&mut self, components: SocketComponents<C>) -> Result<(), SessionError>;

    /// Stops processes inside the state.
    fn stop(&mut self) -> Result<(), SessionError>;

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
    fn segment_sent(&mut self, segment: &Segment) -> Result<(), SessionError>;
}

/// Represents a stateless Session socket.
/// Does nothing by default, only logs warnings and events for tracing.
#[derive(Clone)]
pub struct Stateless<const C: usize>(String);

impl<'a, const C: usize> SocketState<'a, C> for Stateless<C> {
    fn session_id(&self) -> &str {
        &self.0
    }

    fn run(&mut self, _: SocketComponents<C>) -> Result<(), SessionError> {
        tracing::debug!(session_id = self.session_id(), "stateless socket started");
        Ok(())
    }

    fn stop(&mut self) -> Result<(), SessionError> {
        tracing::debug!(session_id = self.session_id(), "stateless socket stopped");
        Ok(())
    }

    fn incoming_segment(&mut self, id: &SegmentId, _segment_count: SeqNum) -> Result<(), SessionError> {
        tracing::trace!(session_id = self.session_id(), %id, "incoming segment");
        Ok(())
    }

    fn incoming_retransmission_request(&mut self, _request: SegmentRequest<C>) -> Result<(), SessionError> {
        tracing::debug!(session_id = self.session_id(), "incoming retransmission request");
        Ok(())
    }

    fn incoming_acknowledged_frames(&mut self, _ack: FrameAcknowledgements<C>) -> Result<(), SessionError> {
        tracing::debug!(session_id = self.session_id(), "incoming frame acknowledgements");
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

    fn segment_sent(&mut self, segment: &Segment) -> Result<(), SessionError> {
        tracing::trace!(session_id = self.session_id(), id = %segment.id(), "segment sent");
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcknowledgementStateConfig {}

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

/// Represents a Session socket state is able to process acknowledgements.
#[derive(Clone)]
pub struct AcknowledgementState<const C: usize> {
    id: String,
    cfg: AcknowledgementStateConfig,
    context: Option<AcknowledgementStateContext<C>>,
}

impl<const C: usize> AcknowledgementState<C> {
    pub fn new<I: std::fmt::Display>(session_id: I, cfg: AcknowledgementStateConfig) -> Self {
        Self {
            id: session_id.to_string(),
            cfg,
            context: Default::default(),
        }
    }
}

impl<'a, const C: usize> SocketState<'a, C> for AcknowledgementState<C> {
    fn session_id(&self) -> &str {
        &self.id
    }

    fn run(&mut self, socket_components: SocketComponents<C>) -> Result<(), SessionError> {
        if self.context.is_some() {
            tracing::warn!(session_id = self.session_id(), "state is already running");
            return Ok(());
        }

        let (incoming_frame_retries_tx, incoming_frame_retries_rx) = skip_delay_channel();
        let (outgoing_frame_retries_tx, outgoing_frame_retries_rx) = skip_delay_channel();
        let (rb_tx, rb_rx) = searchable_ringbuffer(1024);

        // Acknowledgements get a special channel with fixed capacity
        let (mut ack_tx, ack_rx) = futures::channel::mpsc::channel(200_000);

        let context = self.context.insert(AcknowledgementStateContext {
            rb_tx,
            rb_rx,
            incoming_frame_retries_tx,
            outgoing_frame_retries_tx,
            ack_tx,
            ctl_tx: socket_components.ctl_tx,
            inspector: socket_components.inspector,
        });

        // Incoming segment resend requests (that are incomplete for too long)
        let mut incoming_frame_retries_tx_clone = context.incoming_frame_retries_tx.clone();
        let ctl_tx_clone = context.ctl_tx.clone();
        let frame_inspector_clone = context.inspector.clone();
        let sid_1 = self.id.clone();
        let sid_2 = self.id.clone();
        hopr_async_runtime::prelude::spawn(incoming_frame_retries_rx
            .map(move |rf| {
                let frame_id = rf.frame_id;
                // TODO: take frame max_retries from the config
                // Find out if the frame's segments should be requested
                if let Some(next) = rf.next(5) {
                    // Register the next retry if still possible
                    let retry_at = next.at();
                    if let Err(error) = incoming_frame_retries_tx_clone.send_one((next, retry_at)) {
                        tracing::error!(session_id = sid_1, frame_id, %error, "failed to register next resend of incoming frame");
                    } else {
                        tracing::debug!(session_id = sid_1, frame_id, ?retry_at, "next resend request of incoming frame segments");
                    }
                }
                (frame_id, frame_inspector_clone.missing_segments(&frame_id).unwrap_or_default())
            })
            .ready_chunks(SegmentRequest::<C>::MAX_ENTRIES)
            .map(|a| Ok(SessionMessage::<C>::Request(a.into_iter().collect())))
            .forward(ctl_tx_clone)
            .map(move |res| match res {
                Ok(_) => tracing::trace!(session_id = sid_2, "incoming frame resends processing done"),
                Err(error) => tracing::error!(session_id = sid_2, %error, "error while processing incoming frame resends")
            })
        );

        // Forward acknowledged frames chunked as a Control messages
        let ctl_tx_clone = context.ctl_tx.clone();
        let sid = self.id.clone();
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
                tracing::error!(session_id = sid, %error, "acknowledgement forwarding failed");
            } else {
                tracing::trace!(session_id = sid, "acknowledgement forwarding done");
            }
        });

        // Task for outgoing frame retries
        let mut outgoing_frame_retries_tx_clone = context.outgoing_frame_retries_tx.clone();
        let ctl_tx_clone = context.ctl_tx.clone();
        let rb_rx_clone = context.rb_rx.clone();
        let sid_1 = self.id.clone();
        let sid_2 = self.id.clone();
        hopr_async_runtime::prelude::spawn(
           outgoing_frame_retries_rx
                .map(move |rf: RetriedFrameId| {
                    // TODO: take frame max_retries from the config
                    // Find out if the frame can be retried
                    let frame_id = rf.frame_id;
                    if let Some(next) = rf.next(5) {
                        // Register the next retry if still possible
                        let retry_at = next.at();
                        if let Err(error) = outgoing_frame_retries_tx_clone.send_one((next, retry_at)) {
                            tracing::error!(session_id = sid_1, frame_id, %error, "failed to register next retry of frame");
                        } else {
                            tracing::debug!(session_id = sid_1, frame_id, ?retry_at, "next resend of outgoing frame");
                        }
                    }
                    frame_id
                })
                .flat_map(move |frame_id| {
                        // Find out all the segments of that frame to be retransmitted
                        futures::stream::iter(
                            rb_rx_clone.find(|s: &Segment| s.id().0 == frame_id)
                                .into_iter()
                                .map(|s| Ok(SessionMessage::<C>::Segment(s)))
                        )
                })
                .forward(ctl_tx_clone) // Retransmit all the segments
                .map(move |res| match res {
                   Ok(_) => tracing::trace!(session_id = sid_2, "outgoing frame retries processing done"),
                   Err(error) => tracing::error!(session_id = sid_2, %error, "error while processing outgoing frame retries")
               })
        );
        Ok(())
    }

    fn stop(&mut self) -> Result<(), SessionError> {
        if let Some(mut ctx) = self.context.take() {
            ctx.outgoing_frame_retries_tx.force_close();
            ctx.incoming_frame_retries_tx.force_close();
            ctx.ack_tx.close_channel();

            tracing::debug!(session_id = self.session_id(), "state has been stopped");
        }
        Ok(())
    }

    fn incoming_segment(&mut self, id: &SegmentId, _segment_count: SeqNum) -> Result<(), SessionError> {
        tracing::trace!(session_id = %self.session_id(), %id, "segment received");

        if let Some(ref mut ctx) = &mut self.context {
            // Register future requesting of segments for this frame
            let first_request = Duration::from_secs(1); // TODO: take this from the config
            if let Err(error) = ctx.incoming_frame_retries_tx.send_one((id.0.into(), first_request)) {
                tracing::error!(frame_id = id.0, %error, "failed to register incoming retry for frame");
            }

            Ok(())
        } else {
            Err(SessionError::ProcessingError("state is not running".into()))
        }
    }

    fn incoming_retransmission_request(&mut self, request: SegmentRequest<C>) -> Result<(), SessionError> {
        if let Some(ref mut ctx) = &mut self.context {
            let missing = request.into_iter().collect::<Vec<_>>();

            // Perform a single find to lock the RB only once
            let segments = ctx.rb_rx.find(|s| missing.contains(&s.id()));

            // Partially acknowledged frames will not need to be fully resent in the future
            if let Err(error) = ctx.outgoing_frame_retries_tx.send_many(
                missing
                    .into_iter()
                    .map(|seg_id| (RetriedFrameId::new(seg_id.0), Duration::from_secs(1)).into()),
            ) {
                tracing::error!(%error, "failed to cancel frame resend of partially acknowledged frames");
            }

            segments
                .into_iter()
                .try_for_each(|s| ctx.ctl_tx.unbounded_send(SessionMessage::Segment(s)))
                .map_err(|e| SessionError::ProcessingError(e.to_string()))
        } else {
            Err(SessionError::ProcessingError("state is not running".into()))
        }
    }

    fn incoming_acknowledged_frames(&mut self, ack: FrameAcknowledgements<C>) -> Result<(), SessionError> {
        if let Some(ref mut ctx) = &mut self.context {
            // Frame acknowledged, we will not need to resend it
            if let Err(error) = ctx.outgoing_frame_retries_tx.send_many(
                ack.into_iter()
                    .inspect(|frame_id| tracing::trace!(frame_id, "frame acknowledged"))
                    .map(|frame_id| DelayedItem::Cancel(RetriedFrameId::new(frame_id))),
            ) {
                tracing::error!(%error, "failed to cancel frame resend");
            }
            Ok(())
        } else {
            Err(SessionError::ProcessingError("state is not running".into()))
        }
    }

    fn frame_received(&mut self, frame_id: FrameId) -> Result<(), SessionError> {
        let session_id = self.session_id().to_string();
        if let Some(ref mut ctx) = &mut self.context {
            if let Err(error) = ctx.ack_tx.try_send(frame_id) {
                tracing::error!(session_id, frame_id, %error, "failed to acknowledge frame");
            }
            if let Err(error) = ctx
                .incoming_frame_retries_tx
                .send_one((RetriedFrameId::new(frame_id), Skip))
            {
                tracing::error!(session_id, frame_id, %error, "failed to cancel retry of acknowledged frame");
            }
            Ok(())
        } else {
            Err(SessionError::ProcessingError("state is not running".into()))
        }
    }

    fn frame_discarded(&mut self, id: FrameId) -> Result<(), SessionError> {
        if let Some(ref mut ctx) = &mut self.context {
            if let Err(error) = ctx.incoming_frame_retries_tx.send_one((RetriedFrameId::new(id), Skip)) {
                tracing::error!(session_id = self.session_id(), frame_id = id, %error, "failed to cancel retry of acknowledged frame");
            }
            Ok(())
        } else {
            Err(SessionError::ProcessingError("state is not running".into()))
        }
    }

    fn segment_sent(&mut self, segment: &Segment) -> Result<(), SessionError> {
        if let Some(ref mut ctx) = &mut self.context {
            // TODO: make sure segments that we resend do not make it back into the RB
            ctx.rb_tx.push(segment.clone());
            // When the last segment of a frame has been sent,
            // add it to outgoing retries
            if segment.is_last() {
                // TODO: retry token period should be set dynamic based on s.seq_len
                let first_retry = Duration::from_secs(1);
                if let Err(error) = ctx
                    .outgoing_frame_retries_tx
                    .send_one((RetriedFrameId::new(segment.frame_id), first_retry))
                {
                    tracing::trace!(session_id = self.session_id(), frame_id = segment.frame_id, %error, "failed to insert outgoing retry of a frame");
                }
            }
            Ok(())
        } else {
            Err(SessionError::ProcessingError("state is not running".into()))
        }
    }
}
