use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use futures_lite::Future;
use async_broadcast::{Receiver, Sender, SendError};
use crate::errors::{EventError::QueueIsFull, Result};
use crate::events::HoprEvent;

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::spawn_local;

#[cfg(all(feature = "wasm", not(test)))]
use wasm_bindgen_futures::spawn_local;
use utils_log::{debug, warn};

pub struct EventBroker {
    registry: HashMap<String, (Sender<HoprEvent>, Receiver<HoprEvent>)>
}

#[derive(Debug, Clone)]
pub struct EventEmitter {
    name: String,
    sender: Sender<HoprEvent>
}

impl Display for EventEmitter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "emitter for {}", self.name)
    }
}

impl EventEmitter {
    pub async fn emit(&self, event: HoprEvent) -> Result<()> {
        assert_eq!(event.to_string(), self.name, "{self} used for event {event}");

        if !self.sender.is_closed() {
            self.sender.broadcast(event)
                .await
                .map(|_| ())
                .map_err(|_| QueueIsFull)
        } else {
            // do nothing when there are no more subscribers
            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct EventSubscriber<C, F>
where C: Fn(HoprEvent) -> F, F: Future<Output = ()> {
    name: String,
    recv: Receiver<HoprEvent>,
    callback: C
}

impl<C,F> Display for EventSubscriber<C,F>
where C: Fn(HoprEvent) -> F, F: Future<Output = ()>{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "subscriber for {}", self.name)
    }
}

impl<C,F> EventSubscriber<C, F>
where C: Fn(HoprEvent) -> F + 'static, F: Future<Output = ()> + 'static {
    pub fn subscribe(mut self) {
        spawn_local(async move {
            debug!("{self} started");
            while let Ok(event) = self.recv.recv().await {
                assert_eq!(event.to_string(), self.name, "{self} used for event {event}");
                (self.callback)(event).await
            }
        });
        debug!("{self} completed");
    }
}

impl EventBroker {

    pub fn create_emitter_for(&self,event_type: &str) -> EventEmitter {
        EventEmitter {
            name: event_type.into(),
            sender: self.registry.get(event_type).expect("invalid event type {event_type}").0.clone()
        }
    }

    pub fn create_subscriber_for<C,F>(&self, event_type: &str, callback: C) -> EventSubscriber<C,F>
    where C: Fn(HoprEvent) -> F + 'static, F: Future<Output = ()> + 'static {
        EventSubscriber {
            name: event_type.into(),
            recv: self.registry.get(event_type).expect("invalid event type {event_type}").1.clone(),
            callback
        }

    }
}