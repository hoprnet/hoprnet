use futures::{channel::mpsc, future::poll_fn, Stream};
use futures_lite::StreamExt;
use libp2p::identity::PeerId;
use std::pin::Pin;

use core_network::messaging::ControlMessage;
use utils_log::error;

use crate::errors::{P2PError, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct HeartbeatChallenge(pub PeerId, pub ControlMessage);

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPingChallenge(pub PeerId, pub ControlMessage);

// NOTE: UnboundedSender and UnboundedReceiver are bound only by available memory in
// case of faster input than output the memory might run out, but these are protected
// by a bounded queue managing the request generation.
pub type HeartbeaRequestRx = mpsc::UnboundedReceiver<(PeerId, ControlMessage)>;
pub type HeartbeatResponseTx = mpsc::UnboundedSender<(PeerId, std::result::Result<ControlMessage, ()>)>;

pub struct HeartbeatResponder {
    sender: HeartbeatResponseTx,
}

/// Wrapper around the heartbeat responding functionality used by the heartbeat recorder
impl HeartbeatResponder {
    pub fn new(sender: HeartbeatResponseTx) -> Self {
        Self { sender }
    }

    pub async fn record_pong(&mut self, pong: (PeerId, std::result::Result<ControlMessage, ()>)) -> Result<()> {
        match poll_fn(|cx| Pin::new(&mut self.sender).poll_ready(cx)).await {
            Ok(_) => match self.sender.start_send(pong) {
                Err(e) => Err(P2PError::Notification(format!(
                    "Failed to send notification to heartbeat mechanism: {}",
                    e
                ))),
                _ => Ok(()),
            },
            Err(_) => Err(P2PError::ProtocolHeartbeat(format!(
                "The heartbeat mechanism cannot be notified, the receiver was closed"
            ))),
        }
    }

    pub fn generate_challenge_response(challenge: &ControlMessage) -> ControlMessage {
        match ControlMessage::generate_pong_response(challenge) {
            Ok(value) => value,
            Err(_) => {
                error!("Failed to generate a pong response, creating random failing one");
                ControlMessage::generate_pong_response(&ControlMessage::generate_ping_request())
                    .expect("Pong from correct Ping is always creatable")
            }
        }
    }
}

/// Requester of heartbeats implementing the `std::future::Stream` trait.
pub struct HeartbeatRequester {
    receiver: HeartbeaRequestRx,
}

impl HeartbeatRequester {
    pub fn new(receiver: HeartbeaRequestRx) -> Self {
        Self { receiver }
    }
}

impl Stream for HeartbeatRequester {
    type Item = HeartbeatChallenge;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = Pin::new(&mut self);

        return match this.receiver.poll_next(cx) {
            std::task::Poll::Ready(Some((peer, challenge))) => {
                std::task::Poll::Ready(Some(HeartbeatChallenge(peer, challenge)))
            }
            std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
            std::task::Poll::Pending => std::task::Poll::Pending,
        };
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

pub struct ManualPingRequester {
    receiver: HeartbeaRequestRx,
}

impl ManualPingRequester {
    pub fn new(receiver: HeartbeaRequestRx) -> Self {
        Self { receiver }
    }
}

impl Stream for ManualPingRequester {
    type Item = ManualPingChallenge;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = Pin::new(&mut self);

        return match this.receiver.poll_next(cx) {
            std::task::Poll::Ready(Some((peer, challenge))) => {
                std::task::Poll::Ready(Some(ManualPingChallenge(peer, challenge)))
            }
            std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
            std::task::Poll::Pending => std::task::Poll::Pending,
        };
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::future::timeout;

    #[async_std::test]
    pub async fn test_heartbeat_requestor_should_not_return_a_value_if_no_action_was_requested() {
        let (_api_tx, hb_rx) = mpsc::unbounded::<(PeerId, ControlMessage)>();
        let mut stream = HeartbeatRequester::new(hb_rx);

        assert!(timeout(std::time::Duration::from_millis(5), stream.next())
            .await
            .is_err());
    }

    #[async_std::test]
    pub async fn test_heartbeat_requestor_should_return_value_when_ping_is_requested() {
        let (mut request_ping, hb_rx) = mpsc::unbounded::<(PeerId, ControlMessage)>();
        let mut stream = HeartbeatRequester::new(hb_rx);

        let peer = PeerId::random();
        let payload = ControlMessage::generate_ping_request();

        request_ping
            .start_send((peer, payload.clone()))
            .expect("Must be transmitted correctly");

        let next_item = timeout(std::time::Duration::from_millis(5), stream.next())
            .await
            .unwrap();

        assert!(next_item.is_some());

        let HeartbeatChallenge(_peer, _challenge) = next_item.unwrap();

        assert_eq!(peer, _peer);
        assert_eq!(payload, _challenge);
    }

    #[async_std::test]
    pub async fn test_heartbeat_requestor_should_return_all_events_when_multiple_are_polled() {
        let (mut request_ping, hb_rx) = mpsc::unbounded::<(PeerId, ControlMessage)>();
        let mut stream = HeartbeatRequester::new(hb_rx);

        let peers = vec![PeerId::random(), PeerId::random()];
        let challenges = vec![
            ControlMessage::generate_ping_request(),
            ControlMessage::generate_ping_request(),
        ];

        for i in 0..peers.len().min(challenges.len()) {
            request_ping
                .start_send((peers[i], challenges[i].clone()))
                .expect("Must be transmitted correctly");
        }

        for _ in 0..peers.len().min(challenges.len()) {
            let next_item = timeout(std::time::Duration::from_millis(5), stream.next())
                .await
                .unwrap();

            assert!(next_item.is_some());

            let HeartbeatChallenge(_peer, _challenge) = next_item.unwrap();

            assert!(peers.contains(&_peer));
            assert!(challenges.contains(&_challenge));
        }
    }

    #[async_std::test]
    pub async fn test_heartbeat_responder_should_update_the_pong_if_the_challenge_is_replied_correctly() {
        let (hb_tx, mut pong_receiver) = mpsc::unbounded::<(PeerId, std::result::Result<ControlMessage, ()>)>();
        let mut responder = HeartbeatResponder::new(hb_tx);

        let peer = PeerId::random();
        let pong = ControlMessage::generate_pong_response(&ControlMessage::generate_ping_request()).unwrap();

        let result = responder.record_pong((peer, Ok(pong.clone()))).await;
        assert!(result.is_ok());

        let notification = pong_receiver.next().await;
        assert!(notification.is_some());
        let (_peer, _response) = notification.unwrap();

        assert_eq!(peer, _peer);
        assert_eq!(Ok(pong), _response);
    }

    #[async_std::test]
    pub async fn test_heartbeat_responder_should_fail_if_the_receiver_is_closed() {
        let (hb_tx, mut pong_receiver) = mpsc::unbounded::<(PeerId, std::result::Result<ControlMessage, ()>)>();
        let mut responder = HeartbeatResponder::new(hb_tx);

        let peer = PeerId::random();
        let pong = ControlMessage::generate_pong_response(&ControlMessage::generate_ping_request()).unwrap();

        pong_receiver.close();
        let result = responder.record_pong((peer, Ok(pong.clone()))).await;
        assert!(result.is_err());

        let notification = pong_receiver.next().await;
        assert!(notification.is_none());
    }
}
