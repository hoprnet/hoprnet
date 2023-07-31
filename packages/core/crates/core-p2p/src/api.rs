use std::pin::Pin;
use futures::{Stream, channel::mpsc, future::poll_fn};
use futures_lite::StreamExt;
use libp2p::identity::PeerId;

#[cfg(any(not(feature = "wasm"), test))]
use utils_misc::time::native::current_timestamp;
#[cfg(all(feature = "wasm", not(test)))]
use utils_misc::time::wasm::current_timestamp;

use crate::errors::{Result, P2PError};

#[derive(Debug,Clone, PartialEq)]
pub enum NotifierMessage {
    HeartbeatSendPing(PeerId),
    MixerMessage(String),       // TODO: This should hold the `Packet` object
}

pub struct Notifier {
    events: std::collections::VecDeque<NotifierMessage>,
    active_pings: std::collections::HashMap<PeerId, u64>,
    // NOTE: UnboundedSender and UnboundedReceiver are bound only by available memory
    // in case of faster input than output the memory might run out
    trigger_heartbeat_ping: mpsc::UnboundedReceiver<PeerId>,
    notify_heartbeat_pong: mpsc::UnboundedSender<(PeerId, std::result::Result<std::time::Duration, ()>)>,
}

impl Notifier {
    pub fn new(heartbeat_ping: mpsc::UnboundedReceiver<PeerId>, heartbeat_pong: mpsc::UnboundedSender<(PeerId, std::result::Result<std::time::Duration, ()>)>) -> Self {
        Self {
            events: std::collections::VecDeque::new(),
            active_pings: std::collections::HashMap::new(),
            trigger_heartbeat_ping: heartbeat_ping,
            notify_heartbeat_pong: heartbeat_pong,
        }
    }

    pub async fn notify_received_pong(&mut self, pong: (PeerId, std::result::Result<(),()>)) -> Result<()> {
        match poll_fn(|cx| Pin::new(&mut self.notify_heartbeat_pong).poll_ready(cx)).await {
            Ok(_) => {
                let key = self.active_pings.remove(&pong.0);
                
                // TODO: the active pings queue is trivial, if multiple pings are triggered before the previous one
                // is finished, the first returned ping will invalidate the following requests
                if key.is_none() {
                    return Err(P2PError::Logic("Received a pong for an unregistered ping".to_owned()));
                }

                let duration: std::result::Result<std::time::Duration, ()> = match pong.1 {
                    Ok(_) => Ok(std::time::Duration::from_millis(current_timestamp() - key.unwrap())),
                    Err(_) => Err(())
                };

                match self.notify_heartbeat_pong.start_send((pong.0, duration)) {
                    Err(e) => Err(P2PError::Notification(format!("Failed to send notification to heartbeat mechanism: {}", e))),
                    _ => Ok(()),
                }
            },
            Err(_) => Err(P2PError::Notification(format!("The heartbeat mechanism cannot be notified, the receiver was closed"))),
        }
    }
}

impl Stream for Notifier {
    type Item = NotifierMessage;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        let mut this = Pin::new(&mut self);

        match this.trigger_heartbeat_ping.poll_next(cx) {
            std::task::Poll::Ready(Some(ping)) => {
                this.events.push_back(NotifierMessage::HeartbeatSendPing(ping))
            },
            _ => {},
        }

        if this.events.is_empty() {
            std::task::Poll::Pending
        } else {
            let event = this.events.pop_front().expect("Attempted to get notifier message when none was present");
            match event {
                NotifierMessage::HeartbeatSendPing(peer_id) => {
                    let _ = this.active_pings.insert(peer_id, current_timestamp());
                },
                _ => {},
            }
            std::task::Poll::Ready(Some(event))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::future;

    #[async_std::test]
    pub async fn test_notifier_should_not_return_a_value_if_no_action_was_requested() {
        let (hb_tx, hb_rx) = mpsc::unbounded::<ControlMessage>();
        let mut notifier = Notifier::new(hb_rx, hb_tx);

        assert!(future::timeout(std::time::Duration::from_millis(5), notifier.next()).await.is_err());
    }

    #[async_std::test]
    pub async fn test_notifier_should_return_value_when_ping_is_requested() {
        let (hb_tx, hb_rx) = mpsc::unbounded::<ControlMessage>();
        let mut notify_with_ping = hb_tx.clone();
        let mut notifier = Notifier::new(hb_rx, hb_tx);

        let ping = ControlMessage::generate_ping_request(None);
        notify_with_ping.start_send(ping.clone()).expect("Must be transmitted correctly");
        
        let next_notify_item = future::timeout(std::time::Duration::from_millis(5), notifier.next()).await.unwrap();
        assert_eq!(next_notify_item, Some(NotifierMessage::HeartbeatSendPing(ping.clone())));
    }

    #[async_std::test]
    pub async fn test_notifier_should_return_all_events_when_multiple_are_polled() {
        let (hb_tx, hb_rx) = mpsc::unbounded::<ControlMessage>();
        let mut notify_with_ping = hb_tx.clone();
        let mut notifier = Notifier::new(hb_rx, hb_tx);

        let ping1 = ControlMessage::generate_ping_request(None);
        let ping2 = ControlMessage::generate_ping_request(None);
        notify_with_ping.start_send(ping1.clone()).expect("Must be transmitted correctly");
        notify_with_ping.start_send(ping2.clone()).expect("Must be transmitted correctly");
        
        assert_eq!(notifier.next().await, Some(NotifierMessage::HeartbeatSendPing(ping1)));
        assert_eq!(notifier.next().await, Some(NotifierMessage::HeartbeatSendPing(ping2)));
    }
}