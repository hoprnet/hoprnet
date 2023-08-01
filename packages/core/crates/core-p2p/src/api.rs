use std::pin::Pin;
use futures::{Stream, channel::mpsc, future::poll_fn};
use futures_lite::StreamExt;
use libp2p::identity::PeerId;

use core_network::messaging::ControlMessage;
use utils_log::error;
#[cfg(any(not(feature = "wasm"), test))]
use utils_misc::time::native::current_timestamp;
#[cfg(all(feature = "wasm", not(test)))]
use utils_misc::time::wasm::current_timestamp;

use crate::errors::{Result, P2PError};

#[derive(Debug, Clone, PartialEq)]
pub struct HeartbeatChallenge(pub PeerId, pub ControlMessage);

#[derive(Debug, Clone, PartialEq)]
pub enum Triggers {
    Heartbeat(HeartbeatChallenge),
    MixerMessage(String),       // TODO: This should hold the `Packet` object
}

impl From<HeartbeatChallenge> for Triggers {
    fn from(value: HeartbeatChallenge) -> Self {
        Self::Heartbeat(value)
    }
}

// TODO: NOTE: UnboundedSender and UnboundedReceiver are bound only by available memory
// in case of faster input than output the memory might run out
pub type HeartbeaRequestRx = mpsc::UnboundedReceiver<(PeerId, ControlMessage)>;
pub type HeartbeatResponseTx = mpsc::UnboundedSender<(PeerId, std::result::Result<ControlMessage, ()>)>;

pub struct HeartbeatResponder {
    sender: HeartbeatResponseTx
}

impl HeartbeatResponder {
    pub fn new(sender: HeartbeatResponseTx) -> Self
    {
        Self { sender }
    }

    pub async fn record_pong(&mut self, pong: (PeerId, std::result::Result<ControlMessage,()>)) -> Result<()> {
        match poll_fn(|cx| Pin::new(&mut self.sender).poll_ready(cx)).await {
            Ok(_) => {
                match self.sender.start_send(pong) {
                    Err(e) => Err(P2PError::Notification(format!("Failed to send notification to heartbeat mechanism: {}", e))),
                    _ => Ok(()),
                }
            },
            Err(_) => Err(P2PError::ProtocolHeartbeat(format!("The heartbeat mechanism cannot be notified, the receiver was closed"))),
        }
    }

    pub fn generate_challenge_response(challenge: &ControlMessage) -> ControlMessage {
        match ControlMessage::generate_pong_response(challenge) {
            Ok(value) => value,
            Err(_) => {
                error!("Failed to generate a pong response, creating random failing one");
                ControlMessage::generate_pong_response(&ControlMessage::generate_ping_request())
                    .expect("Pong from correct Ping is always creatable")
            },
        }
    }
}

pub struct HeartbeatRequester {
    receiver: HeartbeaRequestRx,
}

impl HeartbeatRequester {
    pub fn new(receiver: HeartbeaRequestRx) -> Self
    {
        Self { receiver }
    }
}

impl Stream for HeartbeatRequester {
    type Item = HeartbeatChallenge;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        let mut this = Pin::new(&mut self);

        return match this.receiver.poll_next(cx) {
            std::task::Poll::Ready(Some((peer, challenge))) => std::task::Poll::Ready(Some(HeartbeatChallenge(peer, challenge))),
            std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}


// pub struct PingMechanism {
//     active_pings: std::collections::HashMap<PeerId, (u64, ControlMessage)>,
//     // TODO: NOTE: UnboundedSender and UnboundedReceiver are bound only by available memory
//     // in case of faster input than output the memory might run out
//     trigger_heartbeat_ping: mpsc::UnboundedReceiver<PeerId>,
//     notify_heartbeat_pong: mpsc::UnboundedSender<(PeerId, std::result::Result<std::time::Duration, ()>)>,
// }

// impl PingMechanism {
//     pub fn new(heartbeat_ping: mpsc::UnboundedReceiver<PeerId>,
//         heartbeat_pong: mpsc::UnboundedSender<(PeerId, std::result::Result<std::time::Duration, ()>)>) -> Self
//     {
//         Self {
//             active_pings: std::collections::HashMap::new(),
//             trigger_heartbeat_ping: heartbeat_ping,
//             notify_heartbeat_pong: heartbeat_pong,
//         }
//     }

//     pub fn generate_challenge_response(challenge: &ControlMessage) -> ControlMessage {
//         match ControlMessage::generate_pong_response(challenge) {
//             Ok(value) => value,
//             Err(_) => {
//                 error!("Failed to generate a pong response, creating random failing one");
//                 ControlMessage::generate_pong_response(&ControlMessage::generate_ping_request())
//                     .expect("Pong from correct Ping is always creatable")
//             },
//         }
//     }

//     pub async fn register_pong(&mut self, pong: (PeerId, std::result::Result<ControlMessage,()>)) -> Result<()> {
//         match poll_fn(|cx| Pin::new(&mut self.notify_heartbeat_pong).poll_ready(cx)).await {
//             Ok(_) => {
//                 let (timestamp, challenge) = self.active_pings.remove(&pong.0)
//                     .ok_or_else(|| P2PError::Logic("Received a pong for an unregistered ping".to_owned()))?;
                
//                 let duration: std::result::Result<std::time::Duration, ()> = match pong.1 {
//                     Ok(response) => {
//                         if ControlMessage::validate_pong_response(&challenge, &response).is_ok() {
//                             Ok(std::time::Duration::from_millis(current_timestamp() - timestamp))
//                         } else {
//                             error!("Failed to verify the challenge for ping to peer: {}", pong.0.to_string());
//                             Err(())
//                         }
                        
//                     },
//                     Err(_) => Err(())
//                 };

//                 match self.notify_heartbeat_pong.start_send((pong.0, duration)) {
//                     Err(e) => Err(P2PError::Notification(format!("Failed to send notification to heartbeat mechanism: {}", e))),
//                     _ => Ok(()),
//                 }
//             },
//             Err(_) => Err(P2PError::Notification(format!("The heartbeat mechanism cannot be notified, the receiver was closed"))),
//         }
//     }
// }

// impl Stream for PingMechanism {
//     type Item = HeartbeatChallenge;

//     fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
//         let mut this = Pin::new(&mut self);

//         return match this.trigger_heartbeat_ping.poll_next(cx) {
//             std::task::Poll::Ready(Some(peer)) => {
//                 if this.active_pings.contains_key(&peer) {
//                     std::task::Poll::Pending
//                 } else {
//                     let ping_challenge = ControlMessage::generate_ping_request();
//                     let _ = this.active_pings.insert(peer, (current_timestamp(), ping_challenge.clone()));
//                     std::task::Poll::Ready(Some(HeartbeatChallenge(peer, ping_challenge)))
//                 }
//             },
//             std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
//             std::task::Poll::Pending => std::task::Poll::Pending,
//         }
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         (0, None)
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::future::timeout;

    #[async_std::test]
    pub async fn test_heartbeat_requestor_should_not_return_a_value_if_no_action_was_requested() {
        let (_api_tx, hb_rx) = mpsc::unbounded::<(PeerId, ControlMessage)>();
        let mut stream = HeartbeatRequester::new(hb_rx);

        assert!(timeout(std::time::Duration::from_millis(5), stream.next()).await.is_err());
    }

    #[async_std::test]
    pub async fn test_heartbeat_requestor_should_return_value_when_ping_is_requested() {
        let (mut request_ping, hb_rx) = mpsc::unbounded::<(PeerId, ControlMessage)>();
        let mut stream = HeartbeatRequester::new(hb_rx);

        let peer = PeerId::random();
        let payload = ControlMessage::generate_ping_request();

        request_ping.start_send((peer, payload.clone())).expect("Must be transmitted correctly");
        
        let next_item = timeout(std::time::Duration::from_millis(5), stream.next()).await.unwrap();

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
        let challenges = vec![ControlMessage::generate_ping_request(), ControlMessage::generate_ping_request()];

        for i in 0..peers.len().min(challenges.len()) {
            request_ping.start_send((peers[i], challenges[i].clone())).expect("Must be transmitted correctly");
        }
        
        for _ in 0..peers.len().min(challenges.len()) {
            let next_item = timeout(std::time::Duration::from_millis(5), stream.next()).await.unwrap();

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