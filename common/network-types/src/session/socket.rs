use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use asynchronous_codec::Framed;
use futures::{pin_mut, Sink, StreamExt, TryStreamExt, AsyncReadExt, AsyncWriteExt, SinkExt};
use futures_concurrency::stream::Merge;
use pin_project::pin_project;
use crate::prelude::errors::SessionError;
use crate::prelude::{frame_reconstructor, Segment, SegmentId};
use crate::prelude::protocol::{FrameAcknowledgements, SessionCodec, SessionMessage};
use crate::session::protocol::SegmentRequest;
use crate::session::segmenter::Segmenter;

#[derive(Clone)]
struct SocketState<const C: usize> {
    ctl_tx: futures::channel::mpsc::UnboundedSender<SessionMessage<C>>,
}

impl<const C: usize> SocketState<C> {
    pub fn notify_segment(&self, id: SegmentId) {
        todo!()
    }

    pub fn acknowledge_frames(&mut self, ack: FrameAcknowledgements<C>) -> Result<(), SessionError>{
        todo!()
    }

    pub fn retransmit_frames(&mut self, retransmit: SegmentRequest<C>) -> Result<(), SessionError> {
        todo!()
    }

    pub fn close(&mut self) {
        self.ctl_tx.close_channel();
    }
}


pub struct SessionSocket<const C: usize> {
    state: SocketState<C>,
    upstream_frames_in: Pin<Box<dyn futures::io::AsyncWrite + Send>>,
    downstream_frames_out: Pin<Box<dyn futures::io::AsyncRead + Send>>
}

impl<const C: usize> SessionSocket<C> {
    pub fn new<T, I>(_id: I, transport: T) -> Self
    where T: futures::io::AsyncRead + futures::io::AsyncWrite + Send + Unpin + 'static, I: std::fmt::Display {
        let (downstream_segment_in, downstream_frames_out) = frame_reconstructor(Duration::from_secs(10), 1024);

        let (upstream_frames_in, data_rx) = Segmenter::<C>::new(1024, 1024);
        let (ctl_tx, ctl_rx) = futures::channel::mpsc::unbounded::<SessionMessage<C>>();

        let state = SocketState {
            ctl_tx,
        };

        // Frames coming out from the Reconstructor can be read upstream
        let downstream_frames_out = Box::pin(downstream_frames_out
            .filter_map(move |maybe_frame| {
                // TODO
                match maybe_frame {
                    Ok(frame) => {
                        futures::future::ready(Some(Ok(frame)))
                    }
                    Err(err) => {
                        futures::future::ready(Some(Err(std::io::Error::other(err))))
                    }
                }
            }).into_async_read());

        let (packets_out, packets_in) = StreamExt::split::<SessionMessage<C>>(Framed::new(transport, SessionCodec::<C>));

        // Messages coming from Upstream and from the State go downstream as Packets
        hopr_async_runtime::prelude::spawn(async move {
            // TODO: save outgoing segments into the State after sending them out
            (ctl_rx, data_rx.map(SessionMessage::<C>::Segment)).merge().map(Ok).forward(packets_out)
        });

        // Packets coming in from Downstream
        let state_clone = state.clone();
        hopr_async_runtime::prelude::spawn(async move {
            pin_mut!(packets_in);
            pin_mut!(downstream_segment_in);
            pin_mut!(state_clone);
            while let Some(in_packet) = packets_in.next().await {
                let res = match in_packet {
                    // Received Segments go straight to the Reassembler,
                    // but the State is also notified about them
                    Ok(SessionMessage::Segment(segment)) => {
                        let id = segment.id();
                        let res = downstream_segment_in.send(segment).await;
                        if res.is_ok() {
                            state_clone.notify_segment(id);
                        }
                        res
                    },
                    // Other Session messages go into the State only
                    Ok(SessionMessage::Acknowledge(ack)) => {
                        state_clone.acknowledge_frames(ack)
                    },
                    Ok(SessionMessage::Request(request)) => {
                        state_clone.retransmit_frames(request)
                    }
                    // Errors are simply propagated
                    Err(e) => Err(e)
                };
                if let Err(e) = res {
                    tracing::error!(error = %e, "failed to process incoming packet");
                }
            }
            tracing::trace!("incoming downstream completed");
        });

        Self { state, upstream_frames_in: Box::pin(upstream_frames_in), downstream_frames_out }
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
        self.state.close();
        let inner = &mut self.upstream_frames_in;
        pin_mut!(inner);
        inner.poll_close(cx)
    }
}
