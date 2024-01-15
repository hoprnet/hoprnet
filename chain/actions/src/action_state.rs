use async_lock::RwLock;
use async_trait::async_trait;
use chain_types::chain_events::{ChainEventType, SignificantChainEvent};
use futures::{channel, FutureExt, TryFutureExt};
use hopr_crypto_types::types::Hash;
use log::{debug, error};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::errors::{CoreEthereumActionsError, Result};

/// Future that resolves once an expectation is matched by some `SignificantChainEvent`
/// Also allows mocking in tests.
pub type ExpectationResolver = Pin<Box<dyn Future<Output = Result<SignificantChainEvent>> + Send>>;

/// Allows tracking state of an `Action` via registering `IndexerExpectation`s on
/// `SignificantChainEvents` coming from the Indexer and resolving them as they are
/// matched. Once expectations are matched, they are automatically unregistered.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ActionState {
    /// Tries to match the given event against the registered expectations.
    /// Each matched expectation is resolved, unregistered and returned.
    async fn match_and_resolve(&self, event: &SignificantChainEvent) -> Vec<IndexerExpectation>;

    /// Registers new `IndexerExpectation`
    async fn register_expectation(&self, exp: IndexerExpectation) -> Result<ExpectationResolver>;

    /// Manually unregisters `IndexerExpectation` given its TX hash.
    async fn unregister_expectation(&self, tx_hash: Hash);
}

/// Expectation on a chain event within a TX indexed by the Indexer.
pub struct IndexerExpectation {
    /// Required TX hash
    pub tx_hash: Hash,
    predicate: Box<dyn Fn(&ChainEventType) -> bool + Send + Sync>,
}

impl Debug for IndexerExpectation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexerExpectation")
            .field("tx_hash", &self.tx_hash)
            .finish_non_exhaustive()
    }
}

impl IndexerExpectation {
    /// Constructs new expectation given the required TX hash and chain event matcher in that TX.
    pub fn new<F>(tx_hash: Hash, expectation: F) -> Self
    where
        F: Fn(&ChainEventType) -> bool + Send + Sync + 'static,
    {
        Self {
            tx_hash,
            predicate: Box::new(expectation),
        }
    }

    /// Evaluates if the given event satisfies this expectation.
    pub fn test(&self, event: &SignificantChainEvent) -> bool {
        event.tx_hash == self.tx_hash && (self.predicate)(&event.event_type)
    }
}

type ExpectationTable = HashMap<Hash, (IndexerExpectation, channel::oneshot::Sender<SignificantChainEvent>)>;

#[derive(Debug)]
pub struct IndexerActionTracker {
    expectations: Arc<RwLock<ExpectationTable>>,
}

impl Default for IndexerActionTracker {
    fn default() -> Self {
        Self {
            expectations: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ActionState for IndexerActionTracker {
    async fn match_and_resolve(&self, event: &SignificantChainEvent) -> Vec<IndexerExpectation> {
        let matched_keys = self
            .expectations
            .read()
            .await
            .iter()
            .filter_map(|(k, (e, _))| e.test(event).then_some(*k))
            .collect::<Vec<_>>();

        debug!("found {} expectations to match event {:?}", matched_keys.len(), event);

        if matched_keys.is_empty() {
            return Vec::new();
        }

        let mut db = self.expectations.write().await;
        matched_keys
            .into_iter()
            .filter_map(|key| {
                db.remove(&key)
                    .and_then(|(exp, sender)| match sender.send(event.clone()) {
                        Ok(_) => {
                            debug!("expectation resolved in {:?}", event);
                            Some(exp)
                        }
                        Err(_) => {
                            error!(
                                "failed to resolve actions in {:?}, because the action confirmation already timed out",
                                event
                            );
                            None
                        }
                    })
            })
            .collect()
    }

    async fn register_expectation(&self, exp: IndexerExpectation) -> Result<ExpectationResolver> {
        match self.expectations.write().await.entry(exp.tx_hash) {
            Entry::Occupied(_) => {
                // TODO: currently cannot register multiple expectations for the same TX hash
                return Err(CoreEthereumActionsError::InvalidState(format!(
                    "expectation for tx {} already present",
                    exp.tx_hash
                )));
            }
            Entry::Vacant(e) => {
                let (tx, rx) = channel::oneshot::channel();
                e.insert((exp, tx));
                Ok(rx
                    .map_err(|_| CoreEthereumActionsError::ExpectationUnregistered)
                    .boxed())
            }
        }
    }

    async fn unregister_expectation(&self, tx_hash: Hash) {
        self.expectations.write().await.remove(&tx_hash);
    }
}

#[cfg(test)]
mod tests {
    use crate::action_state::{ActionState, IndexerActionTracker, IndexerExpectation};
    use crate::errors::CoreEthereumActionsError;
    use async_std::prelude::FutureExt;
    use chain_types::chain_events::{ChainEventType, NetworkRegistryStatus, SignificantChainEvent};
    use hopr_crypto_random::random_bytes;
    use hopr_crypto_types::types::Hash;
    use std::sync::Arc;
    use std::time::Duration;
    use utils_types::primitives::Address;
    use utils_types::traits::BinarySerializable;

    #[async_std::test]
    async fn test_expectation_should_resolve() {
        let random_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());
        let sample_event = SignificantChainEvent {
            tx_hash: random_hash,
            event_type: ChainEventType::NodeSafeRegistered(Address::random()),
        };

        let exp = Arc::new(IndexerActionTracker::default());

        let sample_event_clone = sample_event.clone();
        let exp_clone = exp.clone();
        async_std::task::spawn(async move {
            let hash = exp_clone
                .match_and_resolve(&sample_event_clone)
                .delay(Duration::from_millis(200))
                .await;
            assert!(
                hash.iter().all(|e| e.tx_hash == random_hash),
                "hash must be present as resolved"
            );
        });

        let resolution = exp
            .register_expectation(IndexerExpectation::new(random_hash, move |e| {
                matches!(e, ChainEventType::NodeSafeRegistered(_))
            }))
            .await
            .expect("should register")
            .timeout(Duration::from_secs(5))
            .await
            .expect("should not timeout")
            .expect("resolver must not be cancelled");

        assert_eq!(sample_event, resolution, "resolving event must be equal");
    }

    #[async_std::test]
    async fn test_expectation_should_error_when_unregistered() {
        let sample_event = SignificantChainEvent {
            tx_hash: Hash::new(&random_bytes::<{ Hash::SIZE }>()),
            event_type: ChainEventType::NodeSafeRegistered(Address::random()),
        };

        let exp = Arc::new(IndexerActionTracker::default());

        let sample_event_clone = sample_event.clone();
        let exp_clone = exp.clone();
        async_std::task::spawn(async move {
            exp_clone
                .unregister_expectation(sample_event_clone.tx_hash)
                .delay(Duration::from_millis(200))
                .await;
        });

        let err = exp
            .register_expectation(IndexerExpectation::new(sample_event.tx_hash, move |e| {
                matches!(e, ChainEventType::NodeSafeRegistered(_))
            }))
            .await
            .expect("should register")
            .timeout(Duration::from_secs(5))
            .await
            .expect("should not timeout")
            .expect_err("should return with error");

        assert!(
            matches!(err, CoreEthereumActionsError::ExpectationUnregistered),
            "should notify on unregistration"
        );
    }

    #[async_std::test]
    async fn test_expectation_should_resolve_and_filter() {
        let tx_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());
        let sample_events = vec![
            SignificantChainEvent {
                tx_hash: Hash::new(&random_bytes::<{ Hash::SIZE }>()),
                event_type: ChainEventType::NodeSafeRegistered(Address::random()),
            },
            SignificantChainEvent {
                tx_hash,
                event_type: ChainEventType::NetworkRegistryUpdate(Address::random(), NetworkRegistryStatus::Denied),
            },
            SignificantChainEvent {
                tx_hash,
                event_type: ChainEventType::NetworkRegistryUpdate(Address::random(), NetworkRegistryStatus::Allowed),
            },
        ];

        let exp = Arc::new(IndexerActionTracker::default());

        let sample_events_clone = sample_events.clone();
        let exp_clone = exp.clone();
        async_std::task::spawn(async move {
            for sample_event in sample_events_clone {
                exp_clone
                    .match_and_resolve(&sample_event)
                    .delay(Duration::from_millis(200))
                    .await;
            }
        });

        let resolution = exp
            .register_expectation(IndexerExpectation::new(tx_hash, move |e| {
                matches!(
                    e,
                    ChainEventType::NetworkRegistryUpdate(_, NetworkRegistryStatus::Allowed)
                )
            }))
            .await
            .expect("should register")
            .timeout(Duration::from_secs(5))
            .await
            .expect("should not timeout")
            .expect("resolver must not be cancelled");

        assert_eq!(sample_events[2], resolution, "resolving event must be equal");
    }

    #[async_std::test]
    async fn test_expectation_should_resolve_multiple_expectations() {
        let sample_events = vec![
            SignificantChainEvent {
                tx_hash: Hash::new(&random_bytes::<{ Hash::SIZE }>()),
                event_type: ChainEventType::NodeSafeRegistered(Address::random()),
            },
            SignificantChainEvent {
                tx_hash: Hash::new(&random_bytes::<{ Hash::SIZE }>()),
                event_type: ChainEventType::NetworkRegistryUpdate(Address::random(), NetworkRegistryStatus::Denied),
            },
            SignificantChainEvent {
                tx_hash: Hash::new(&random_bytes::<{ Hash::SIZE }>()),
                event_type: ChainEventType::NetworkRegistryUpdate(Address::random(), NetworkRegistryStatus::Allowed),
            },
        ];

        let exp = Arc::new(IndexerActionTracker::default());

        let sample_events_clone = sample_events.clone();
        let exp_clone = exp.clone();
        async_std::task::spawn(async move {
            for sample_event in sample_events_clone {
                exp_clone
                    .match_and_resolve(&sample_event)
                    .delay(Duration::from_millis(100))
                    .await;
            }
        });

        let registered_exps = vec![
            exp.register_expectation(IndexerExpectation::new(sample_events[2].tx_hash, move |e| {
                matches!(
                    e,
                    ChainEventType::NetworkRegistryUpdate(_, NetworkRegistryStatus::Allowed)
                )
            }))
            .await
            .expect("should register 1"),
            exp.register_expectation(IndexerExpectation::new(sample_events[0].tx_hash, move |e| {
                matches!(e, ChainEventType::NodeSafeRegistered(_))
            }))
            .await
            .expect("should register 2"),
        ];

        let resolutions = futures::future::try_join_all(registered_exps)
            .timeout(Duration::from_secs(5))
            .await
            .expect("should not timeout")
            .expect("no resolver can cancel");

        assert_eq!(sample_events[2], resolutions[0], "resolving event 1 must be equal");
        assert_eq!(sample_events[0], resolutions[1], "resolving event 2 must be equal");
    }
}
