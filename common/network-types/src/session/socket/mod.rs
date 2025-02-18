mod state;

use asynchronous_codec::Framed;
use futures::StreamExt;
use futures::{future, pin_mut, FutureExt, TryStreamExt};
use futures_concurrency::stream::Merge;
use state::SocketState;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use crate::prelude::errors::SessionError;
use crate::prelude::frame_reconstructor_with_inspector;
use crate::prelude::protocol::{SessionCodec, SessionMessage};
use crate::session::segmenter::Segmenter;
use crate::session::socket::state::SocketComponents;

pub struct SessionSocket<const C: usize> {
    // This is where upstream writes the to-be-segmented frame data to
    upstream_frames_in: Pin<Box<dyn futures::io::AsyncWrite + Send>>,
    // This is where upstream reads the reconstructed frame data from
    downstream_frames_out: Pin<Box<dyn futures::io::AsyncRead + Send>>,
}

impl<const C: usize> SessionSocket<C> {
    pub fn new<T, S>(transport: T, mut state: S) -> Self
    where
        T: futures::io::AsyncRead + futures::io::AsyncWrite + Send + Unpin + 'static,
        S: for<'a> SocketState<'a, C> + Clone + Send + 'static,
    {
        // Downstream Segments get reconstructed into Frames
        let (downstream_segment_in, downstream_frames_out, frame_inspector) =
            frame_reconstructor_with_inspector(Duration::from_secs(10), 1024);

        // Upstream frames get segmented and are yielded by the data_rx stream
        let (upstream_frames_in, segmented_data_rx) = Segmenter::<C, 1500>::new(1024);

        // Downstream transport
        let (packets_out, packets_in) = Framed::new(transport, SessionCodec::<C>).split();

        let (ctl_tx, ctl_rx) = futures::channel::mpsc::unbounded();
        state.run(SocketComponents {
            inspector: frame_inspector,
            ctl_tx,
        });

        // Messages incoming from Upstream and from the State go downstream as Packets
        // Segmented data coming from Upstream go out right away.
        let mut st = state.clone();
        let mut st_2 = state.clone();
        hopr_async_runtime::prelude::spawn(
            (
                ctl_rx,
                segmented_data_rx.map(move |s| {
                    if let Err(error) = st.segment_sent(&s) {
                        tracing::debug!(session_id = st.session_id(), %error, "outgoing segment state update failed");
                    }
                    SessionMessage::<C>::Segment(s)
                }),
            )
                .merge()
                .map(Ok)
                .forward(packets_out)
                .map(move |result| match st_2.stop().and(result) {
                    Ok(_) => tracing::debug!(session_id = st_2.session_id(), "outgoing packet processing done"),
                    Err(error) => {
                        tracing::error!(session_id = st_2.session_id(), %error, "error while processing outgoing packets")
                    }
                }),
        );

        // Packets incoming from Downstream:
        // - if the State requires it, packets passed (cloned) into the State
        // - Packets that represent Segments (filtered out) are passed to the Reconstructor
        let mut st = state.clone();
        let mut st_2 = state.clone();
        hopr_async_runtime::prelude::spawn(
            packets_in
                .try_filter_map(move |packet| {
                    if let Err(error) = match &packet {
                        SessionMessage::Segment(s) => st.incoming_segment(&s.id(), s.seq_len),
                        SessionMessage::Request(r) => st.incoming_retransmission_request(r.clone()),
                        SessionMessage::Acknowledge(a) => st.incoming_acknowledged_frames(a.clone()),
                    } {
                        tracing::debug!(session_id = st.session_id(), %error, "incoming message state update failed");
                    }
                    future::ok(packet.try_as_segment())
                })
                .forward(downstream_segment_in)
                .map(move |result| match st_2.stop().and(result) {
                    Ok(_) => tracing::debug!(session_id = st_2.session_id(), "incoming packet processing done"),
                    Err(error) => {
                        tracing::error!(session_id = st_2.session_id(), %error, "error while processing incoming packets")
                    }
                }),
        );

        Self {
            upstream_frames_in: Box::pin(upstream_frames_in),
            downstream_frames_out: Box::pin(
                downstream_frames_out
                    .filter_map(move |maybe_frame| {
                        // Filter out discarded Frames and dispatch events to the State if needed
                        future::ready(match maybe_frame {
                            Ok(frame) => {
                                if let Err(error) = state.frame_received(frame.frame_id) {
                                    tracing::debug!(session_id = state.session_id(), %error, "frame received state update failed");
                                }
                                Some(Ok(frame))
                            }
                            Err(SessionError::FrameDiscarded(frame_id))
                            | Err(SessionError::IncompleteFrame(frame_id)) => {
                                if let Err(error) = state.frame_discarded(frame_id) {
                                    tracing::debug!(session_id = state.session_id(), %error, "frame discarded state update failed");
                                }
                                None // Downstream skips discarded frames
                            }
                            Err(err) => Some(Err(std::io::Error::other(err))),
                        })
                    })
                    .into_async_read(),
            ),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::socket::state::Stateless;
    use crate::utils::DuplexIO;

    use crate::session::testing::*;

    const MTU: usize = 462;

    #[async_std::test]
    async fn stateless_socket_bidirectional_should_work() -> anyhow::Result<()> {
        let mut alice_socket = SessionSocket::<MTU>::new(DuplexIO(alice_reader, bob_writer), Stateless::new("alice"));
        let mut bob_socket = SessionSocket::<MTU>::new(DuplexIO(bob_reader, alice_writer), Stateless::new("bob"));

        Ok(())
    }
}
