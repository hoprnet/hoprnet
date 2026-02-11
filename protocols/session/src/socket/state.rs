use futures::channel::mpsc::Sender;

use crate::{
    errors::SessionError,
    processing::types::FrameInspector,
    protocol::{FrameAcknowledgements, FrameId, Segment, SegmentId, SegmentRequest, SeqIndicator, SessionMessage},
};

/// Components the [`SessionSocket`] exposes to a [`SocketState`].
///
/// This is the primary communication interface between the state and the socket.
pub struct SocketComponents<const C: usize> {
    /// Allows inspecting incomplete frames that are currently held by the socket.
    ///
    /// Some states might strictly require a frame inspector and may therefore
    /// return an error in [`SocketState::run`] if not present.
    pub inspector: Option<FrameInspector>,
    /// Allows emitting control messages to the socket.
    ///
    /// It is a regular [`SessionMessage`] injected into the downstream.
    pub ctl_tx: Sender<SessionMessage<C>>,
}

/// Abstraction of the [`SessionSocket`](super::SessionSocket) state.
pub trait SocketState<const C: usize>: Send {
    /// Gets ID of this Session.
    fn session_id(&self) -> &str;

    /// Starts the necessary processes inside the state.
    /// Should be idempotent if called multiple times.
    fn run(&mut self, components: SocketComponents<C>) -> Result<(), SessionError>;

    /// Stops processes inside the state for the given direction.
    fn stop(&mut self) -> Result<(), SessionError>;

    /// Called when the Socket receives a new segment from Downstream.
    /// When the error is returned, the incoming segment is not passed Upstream.
    fn incoming_segment(&mut self, id: &SegmentId, ind: SeqIndicator) -> Result<(), SessionError>;

    /// Called when [segment retransmission request](SegmentRequest) is received from Downstream.
    fn incoming_retransmission_request(&mut self, request: SegmentRequest<C>) -> Result<(), SessionError>;

    /// Called when an [acknowledgement of frames](FrameAcknowledgements) is received from Downstream.
    fn incoming_acknowledged_frames(&mut self, ack: FrameAcknowledgements<C>) -> Result<(), SessionError>;

    /// Called when a complete Frame has been finalized from segments received from Downstream.
    fn frame_complete(&mut self, id: FrameId) -> Result<(), SessionError>;

    /// Called when a complete Frame emitted to Upstream in-sequence.
    fn frame_emitted(&mut self, id: FrameId) -> Result<(), SessionError>;

    /// Called when a frame could not be completed from the segments received from Downstream.
    fn frame_discarded(&mut self, id: FrameId) -> Result<(), SessionError>;

    /// Called when a segment of a Frame was sent to the Downstream.
    fn segment_sent(&mut self, segment: &Segment) -> Result<(), SessionError>;

    /// Convenience method to dispatch a [`message`](SessionMessage) to one of the available handlers.
    fn incoming_message(&mut self, message: &SessionMessage<C>) -> Result<(), SessionError> {
        match &message {
            SessionMessage::Segment(s) => self.incoming_segment(&s.id(), s.seq_flags),
            SessionMessage::Request(r) => self.incoming_retransmission_request(r.clone()),
            SessionMessage::Acknowledge(a) => self.incoming_acknowledged_frames(a.clone()),
        }
    }
}

/// Represents a stateless Session socket.
///
/// Does nothing by default, only logs warnings and events for tracing.
#[derive(Clone)]
pub struct Stateless<const C: usize>(String);

impl<const C: usize> Stateless<C> {
    pub(crate) fn new<I: std::fmt::Display>(session_id: I) -> Self {
        Self(session_id.to_string())
    }
}

impl<const C: usize> SocketState<C> for Stateless<C> {
    fn session_id(&self) -> &str {
        &self.0
    }

    fn run(&mut self, _: SocketComponents<C>) -> Result<(), SessionError> {
        Ok(())
    }

    fn stop(&mut self) -> Result<(), SessionError> {
        Ok(())
    }

    fn incoming_segment(&mut self, _: &SegmentId, _: SeqIndicator) -> Result<(), SessionError> {
        Ok(())
    }

    fn incoming_retransmission_request(&mut self, _: SegmentRequest<C>) -> Result<(), SessionError> {
        Ok(())
    }

    fn incoming_acknowledged_frames(&mut self, _: FrameAcknowledgements<C>) -> Result<(), SessionError> {
        Ok(())
    }

    fn frame_complete(&mut self, _: FrameId) -> Result<(), SessionError> {
        Ok(())
    }

    fn frame_emitted(&mut self, _: FrameId) -> Result<(), SessionError> {
        Ok(())
    }

    fn frame_discarded(&mut self, _: FrameId) -> Result<(), SessionError> {
        Ok(())
    }

    fn segment_sent(&mut self, _: &Segment) -> Result<(), SessionError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, time::Duration};

    use anyhow::Context;
    use futures::{AsyncReadExt, AsyncWriteExt};
    use futures_time::future::FutureExt;

    use super::*;
    #[cfg(feature = "stats")]
    use crate::socket::stats::NoopTracker;
    use crate::{
        SessionSocket, SessionSocketConfig,
        utils::test::{FaultyNetworkConfig, setup_alice_bob},
    };

    const FRAME_SIZE: usize = 1500;

    const MTU: usize = 1000;

    mockall::mock! {
        SockState {}
        impl SocketState<MTU> for SockState {
            fn session_id(&self) -> &str;
            fn run(&mut self, components: SocketComponents<MTU>) -> Result<(), SessionError>;
            fn stop(&mut self) -> Result<(), SessionError>;
            fn incoming_segment(&mut self, id: &SegmentId, ind: SeqIndicator) -> Result<(), SessionError>;
            fn incoming_retransmission_request(&mut self, request: SegmentRequest<MTU>) -> Result<(), SessionError>;
            fn incoming_acknowledged_frames(&mut self, ack: FrameAcknowledgements<MTU>) -> Result<(), SessionError>;
            fn frame_complete(&mut self, id: FrameId) -> Result<(), SessionError>;
            fn frame_emitted(&mut self, id: FrameId) -> Result<(), SessionError>;
            fn frame_discarded(&mut self, id: FrameId) -> Result<(), SessionError>;
            fn segment_sent(&mut self, segment: &Segment) -> Result<(), SessionError>;
        }
    }

    #[derive(Clone)]
    struct CloneableMockState<'a>(std::sync::Arc<std::sync::Mutex<MockSockState>>, &'a str);

    impl<'a> CloneableMockState<'a> {
        pub fn new(state: MockSockState, id: &'a str) -> Self {
            Self(std::sync::Arc::new(std::sync::Mutex::new(state)), id)
        }
    }

    impl SocketState<MTU> for CloneableMockState<'_> {
        fn session_id(&self) -> &str {
            let _ = self.0.lock().unwrap().session_id();
            self.1
        }

        fn run(&mut self, components: SocketComponents<MTU>) -> Result<(), SessionError> {
            tracing::debug!(id = self.1, "run called");
            self.0.lock().unwrap().run(components)
        }

        fn stop(&mut self) -> Result<(), SessionError> {
            tracing::debug!(id = self.1, "stop called");
            self.0.lock().unwrap().stop()
        }

        fn incoming_segment(&mut self, id: &SegmentId, ind: SeqIndicator) -> Result<(), SessionError> {
            tracing::debug!(id = self.1, "incoming_segment called");
            self.0.lock().unwrap().incoming_segment(id, ind)
        }

        fn incoming_retransmission_request(&mut self, request: SegmentRequest<MTU>) -> Result<(), SessionError> {
            tracing::debug!(id = self.1, "incoming_retransmission_request called");
            self.0.lock().unwrap().incoming_retransmission_request(request)
        }

        fn incoming_acknowledged_frames(&mut self, ack: FrameAcknowledgements<MTU>) -> Result<(), SessionError> {
            tracing::debug!(id = self.1, "incoming_acknowledged_frames called");
            self.0.lock().unwrap().incoming_acknowledged_frames(ack)
        }

        fn frame_complete(&mut self, id: FrameId) -> Result<(), SessionError> {
            tracing::debug!(id = self.1, "frame_complete called");
            self.0.lock().unwrap().frame_complete(id)
        }

        fn frame_emitted(&mut self, id: FrameId) -> Result<(), SessionError> {
            tracing::debug!(id = self.1, "frame_received called");
            self.0.lock().unwrap().frame_emitted(id)
        }

        fn frame_discarded(&mut self, id: FrameId) -> Result<(), SessionError> {
            tracing::debug!(id = self.1, "frame_discarded called");
            self.0.lock().unwrap().frame_discarded(id)
        }

        fn segment_sent(&mut self, segment: &Segment) -> Result<(), SessionError> {
            tracing::debug!(id = self.1, "segment_sent called");
            self.0.lock().unwrap().segment_sent(segment)
        }
    }

    #[test_log::test(tokio::test)]
    async fn session_socket_must_correctly_dispatch_segment_and_frame_state_events() -> anyhow::Result<()> {
        const NUM_FRAMES: usize = 2;

        const NUM_SEGMENTS: usize = NUM_FRAMES * FRAME_SIZE / MTU + 1;

        let mut alice_seq = mockall::Sequence::new();
        let mut alice_state = MockSockState::new();
        alice_state.expect_session_id().return_const("alice".into());

        alice_state
            .expect_run()
            .once()
            .in_sequence(&mut alice_seq)
            .return_once(|_| Ok::<_, SessionError>(()));
        alice_state
            .expect_segment_sent()
            .times(NUM_SEGMENTS)
            .in_sequence(&mut alice_seq)
            .returning(|_| Ok::<_, SessionError>(()));
        alice_state
            .expect_stop()
            .once()
            .in_sequence(&mut alice_seq)
            .return_once(|| Ok::<_, SessionError>(()));
        alice_state
            .expect_segment_sent() // terminating segment
            .once()
            .in_sequence(&mut alice_seq)
            .return_once(|_| Ok::<_, SessionError>(()));

        let mut bob_seq = mockall::Sequence::new();
        let mut bob_state = MockSockState::new();
        bob_state.expect_session_id().return_const("bob".into());

        bob_state
            .expect_run()
            .once()
            .in_sequence(&mut bob_seq)
            .return_once(|_| Ok::<_, SessionError>(()));
        bob_state
            .expect_incoming_segment()
            .times(NUM_SEGMENTS - 1)
            .in_sequence(&mut bob_seq)
            .returning(|_, _| Ok::<_, SessionError>(()));
        bob_state
            .expect_frame_complete()
            .once()
            .in_sequence(&mut bob_seq)
            .with(mockall::predicate::eq(2))
            .returning(|_| Ok::<_, SessionError>(()));
        bob_state
            .expect_frame_discarded()
            .once()
            .in_sequence(&mut bob_seq)
            .with(mockall::predicate::eq(1))
            .returning(|_| Ok::<_, SessionError>(()));
        bob_state
            .expect_frame_emitted()
            .once()
            .in_sequence(&mut bob_seq)
            .with(mockall::predicate::eq(2))
            .returning(|_| Ok::<_, SessionError>(()));
        bob_state
            .expect_stop()
            .once()
            .in_sequence(&mut bob_seq)
            .return_once(|| Ok::<_, SessionError>(()));
        bob_state
            .expect_segment_sent() // terminating segment
            .once()
            .in_sequence(&mut bob_seq)
            .return_once(|_| Ok::<_, SessionError>(()));

        let (alice, bob) = setup_alice_bob::<MTU>(
            FaultyNetworkConfig {
                avg_delay: Duration::from_millis(10),
                ids_to_drop: HashSet::from_iter([0_usize]),
                ..Default::default()
            },
            None,
            None,
        );

        let cfg = SessionSocketConfig {
            frame_size: FRAME_SIZE,
            frame_timeout: Duration::from_millis(55),
            ..Default::default()
        };

        let mut alice_socket = SessionSocket::new(
            alice,
            CloneableMockState::new(alice_state, "alice"),
            cfg,
            #[cfg(feature = "stats")]
            NoopTracker,
        )?;
        let mut bob_socket = SessionSocket::new(
            bob,
            CloneableMockState::new(bob_state, "bob"),
            cfg,
            #[cfg(feature = "stats")]
            NoopTracker,
        )?;

        let alice_sent_data = hopr_crypto_random::random_bytes::<{ NUM_FRAMES * FRAME_SIZE }>();
        alice_socket
            .write_all(&alice_sent_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await
            .context("write_all timeout")??;
        alice_socket.flush().await?;

        // One entire frame is discarded
        let mut bob_recv_data = [0u8; (NUM_FRAMES - 1) * FRAME_SIZE];
        bob_socket
            .read_exact(&mut bob_recv_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await
            .context("read_exact timeout")??;

        tracing::debug!("stopping");
        alice_socket.close().await?;
        bob_socket.close().await?;

        Ok(())
    }
}
