use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use futures_lite::Future;
use async_broadcast::{broadcast, Receiver, Sender};
use crate::events::HoprEvent;
use utils_log::{debug};

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::spawn_local;
use strum::VariantNames;

#[cfg(all(feature = "wasm", not(test)))]
use wasm_bindgen_futures::spawn_local;

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
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
    /// Emits an event in the channel. The event may or maybe not delivered.
    pub fn emit(&self, event: HoprEvent) {
        assert_eq!(event.to_string(), self.name, "{self} used for event {event}");
        if let Err(e) = self.sender.try_broadcast(event) {
            debug!("{self} could not deliver an event: {e}");
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

impl<C, F> EventSubscriber<C, F>
where C: Fn(HoprEvent) -> F + 'static, F: Future<Output = ()> + 'static {

    /// Consumes this instance and spawns a task that pop events from the queue as they arrive,
    /// calling the callback given in `create_subscriber_for` per each event.
    pub fn subscribe(mut self) {
        spawn_local(async move {
            debug!("{self} started");
            while let Ok(event) = self.recv.recv().await {
                assert_eq!(event.to_string(), self.name, "{self} used for event {event}");
                (self.callback)(event).await
            }
            debug!("{self} completed");
        });
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug)]
pub struct EventBroker {
    registry: HashMap<String, (Sender<HoprEvent>, Receiver<HoprEvent>)>
}

impl EventBroker {

    const EVENT_QUEUE_SIZE: usize = 256;

    pub fn new() -> Self {
        let mut registry = HashMap::new();
        HoprEvent::VARIANTS.into_iter().for_each(|v| {
            let mut channel = broadcast(Self::EVENT_QUEUE_SIZE);
            channel.0.set_await_active(false);  // do not wait for receivers
            channel.0.set_overflow(true);       // queue is a ring buffer
            registry.insert((*v).into(), channel);
        });
        Self { registry }
    }

    /// List all the event names supported by the broker.
    pub fn supported_events(&self) -> Vec<String> {
        self.registry.keys().map(|s| s.clone()).collect()
    }

    /// Creates a new emitter for the given event name.
    /// The event name must be one of `supported_events()`
    pub fn create_emitter_for(&self, event_type: &str) -> EventEmitter {
        EventEmitter {
            name: event_type.into(),
            sender: self.registry.get(event_type).expect("invalid event type {event_type}").0.clone()
        }
    }

    /// Creates a new subscriber for the given event name.
    /// The event name must be one of `supported_events()`
    pub fn create_subscriber_for<C, F>(&self, event_type: &str, callback: C) -> EventSubscriber<C, F>
    where C: Fn(HoprEvent) -> F + 'static, F: Future<Output = ()> + 'static {
        EventSubscriber {
            name: event_type.into(),
            recv: self.registry.get(event_type).expect("invalid event type {event_type}").1.clone(),
            callback
        }
    }
}

#[cfg(test)]
mod tests {
    use strum::VariantNames;
    use crate::broker::EventBroker;
    use crate::events::HoprEvent;

    #[test]
    fn test_broker_variants() {
        let broker = EventBroker::new();

        assert!(HoprEvent::VARIANTS.iter().all(|v| broker.supported_events().contains(&(*v).into())));
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::wasm_bindgen;
    use crate::broker::EventEmitter;
    use crate::events;

    #[wasm_bindgen]
    impl EventEmitter {
        pub fn _emit(&self, event: events::wasm::HoprEvent) {
            self.emit(event.w)
        }
    }
}