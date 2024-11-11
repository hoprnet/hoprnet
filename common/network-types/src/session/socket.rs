use asynchronous_codec::Framed;
use futures::{pin_mut, Sink, SinkExt, StreamExt, TryStreamExt};
use futures_concurrency::stream::Merge;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use crate::prelude::errors::SessionError;
use crate::prelude::protocol::{FrameAcknowledgements, SessionCodec, SessionMessage};
use crate::prelude::{frame_reconstructor, Segment};
use crate::session::protocol::SegmentRequest;
use crate::session::segmenter::Segmenter;
use crate::session::utils::{offloaded_ringbuffer, OffloadedRbConsumer};

struct SocketState<const C: usize> {
    rb: OffloadedRbConsumer<Segment>,
    ctl_tx: futures::channel::mpsc::UnboundedSender<SessionMessage<C>>,
    downstream_segment_in: Pin<Box<dyn Sink<Segment, Error = SessionError> + Send>>,
}

impl<const C: usize> SocketState<C> {
    async fn incoming_segment(&mut self, segment: Segment) -> Result<(), SessionError> {
        self.downstream_segment_in
            .send(segment)
            .await
            .map_err(|e| SessionError::ProcessingError(e.to_string()))?;

        todo!()
    }

    fn incoming_acknowledged_frames(&mut self, ack: FrameAcknowledgements<C>) -> Result<(), SessionError> {
        todo!()
    }

    async fn incoming_retransmission_request(&mut self, retransmit: SegmentRequest<C>) -> Result<(), SessionError> {
        let missing = retransmit.into_iter().collect::<Vec<_>>();
        let segments = self.rb.find(|s| missing.contains(&s.id()));

        // TODO:

        self.downstream_segment_in
            .send_all(&mut futures::stream::iter(segments).map(Ok))
            .await
    }

    pub async fn handle_incoming_message(&mut self, message: SessionMessage<C>) -> Result<(), SessionError> {
        match message {
            SessionMessage::Segment(s) => self.incoming_segment(s).await,
            SessionMessage::Request(r) => self.incoming_retransmission_request(r).await,
            SessionMessage::Acknowledge(a) => self.incoming_acknowledged_frames(a),
        }
    }
}

pub struct SessionSocket<const C: usize> {
    upstream_frames_in: Pin<Box<dyn futures::io::AsyncWrite + Send>>,
    downstream_frames_out: Pin<Box<dyn futures::io::AsyncRead + Send>>,
}

impl<const C: usize> SessionSocket<C> {
    pub fn new<T, I>(_id: I, transport: T) -> Self
    where
        T: futures::io::AsyncRead + futures::io::AsyncWrite + Send + Unpin + 'static,
        I: std::fmt::Display,
    {
        let (downstream_segment_in, downstream_frames_out) = frame_reconstructor(Duration::from_secs(10), 1024);

        let (upstream_frames_in, data_rx) = Segmenter::<C, 1500>::new(1024);
        let (ctl_tx, ctl_rx) = futures::channel::mpsc::unbounded::<SessionMessage<C>>();

        // Frames coming out from the Reconstructor can be read upstream
        let downstream_frames_out = Box::pin(
            downstream_frames_out
                .filter_map(move |maybe_frame| {
                    // TODO: acknowledge received frames
                    match maybe_frame {
                        Ok(frame) => futures::future::ready(Some(Ok(frame))),
                        Err(err) => futures::future::ready(Some(Err(std::io::Error::other(err)))),
                    }
                })
                .into_async_read(),
        );

        let (packets_out, packets_in) =
            StreamExt::split::<SessionMessage<C>>(Framed::new(transport, SessionCodec::<C>));

        let (rb_tx, rb_rx) = offloaded_ringbuffer(1024);

        // Messages incoming from Upstream and from the State go downstream as Packets
        hopr_async_runtime::prelude::spawn(
            (
                ctl_rx,
                data_rx
                    .inspect(move |s| {
                        rb_tx.push(s.clone());
                    })
                    .map(SessionMessage::<C>::Segment),
            )
                .merge()
                .map(Ok)
                .forward(packets_out),
        );

        let mut state = SocketState {
            rb: rb_rx,
            ctl_tx,
            downstream_segment_in: Box::pin(downstream_segment_in),
        };

        // Packets incoming from Downstream
        hopr_async_runtime::prelude::spawn(async move {
            pin_mut!(packets_in);

            // TODO: refactor
            while let Some(in_packet) = packets_in.next().await {
                let res = match in_packet {
                    Ok(msg) => state.handle_incoming_message(msg).await,
                    Err(e) => Err(e), // errors are simply propagated
                };
                if let Err(e) = res {
                    tracing::error!(error = %e, "failed to process incoming packet");
                }
            }
            tracing::trace!("incoming downstream completed");
        });

        Self {
            upstream_frames_in: Box::pin(upstream_frames_in),
            downstream_frames_out,
        }
    }
}

impl<const C: usize> futures::io::AsyncRead for SessionSocket<C> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        let inner = &mut self.downstream_frames_out;
        pin_mut!(inner);
        inner.poll_read(cx, buf)
    }
}

impl<const C: usize> futures::io::AsyncWrite for SessionSocket<C> {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        let inner = &mut self.upstream_frames_in;
        pin_mut!(inner);
        inner.poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let inner = &mut self.upstream_frames_in;
        pin_mut!(inner);
        inner.poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let inner = &mut self.upstream_frames_in;
        pin_mut!(inner);
        inner.poll_close(cx)
    }
}
