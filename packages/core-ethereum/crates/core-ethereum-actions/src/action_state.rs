use async_lock::RwLock;
use async_trait::async_trait;
use core_crypto::types::Hash;
use core_ethereum_types::chain_events::{ChainEventType, SignificantChainEvent};
use futures::{FutureExt, TryFutureExt};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use utils_log::error;

use crate::errors::{CoreEthereumActionsError, Result};

/// Future that resolves once an expectation is matched by some `SignificantChainEvent`
/// Also allows mocking in tests.
pub type ExpectationResolver = Pin<Box<dyn Future<Output = Result<SignificantChainEvent>> + Send>>;

/// Allows tracking state of an `Action` via registering `IndexerExpectation`s on
/// `SignificantChainEvents` coming from the Indexer and resolving them as they are
/// matched. Once expectations are matched, they are automatically unregistered.
#[cfg_attr(test, mockall::automock)]
#[async_trait(? Send)]
pub trait ActionState {
    /// Tries to match the given event against the registered expectations.
    /// Each matched expectation is resolved, unregistered and returned.
    async fn match_and_resolve(&self, event: &SignificantChainEvent) -> Vec<IndexerExpectation>;
    /// Registers new `IndexerExpectations`
    async fn register_expectation(&self, exp: IndexerExpectation) -> Result<ExpectationResolver>;
    /// Manually unregisters `IndexerExpectation` given its TX hash.
    async fn unregister_expectation(&self, tx_hash: Hash);
}

/// Expectation on a chain event within a TX indexed by the Indexer.
pub struct IndexerExpectation {
    /// Required TX hash
    pub tx_hash: Hash,
    event_expectation: Box<dyn Fn(&ChainEventType) -> bool>,
}

impl IndexerExpectation {
    /// Constructs new expectation given the required TX hash and chain event matcher in that TX.
    pub fn new<F>(tx_hash: Hash, expectation: F) -> Self
    where
        F: Fn(&ChainEventType) -> bool + 'static,
    {
        Self {
            tx_hash,
            event_expectation: Box::new(expectation),
        }
    }

    /// Evaluates if the given event satisfies this expectation.
    pub fn test(&self, event: &SignificantChainEvent) -> bool {
        event.tx_hash == self.tx_hash && (self.event_expectation)(&event.event_type)
    }
}

pub struct IndexerActionTracker {
    expectations: RwLock<
        HashMap<
            Hash,
            (
                IndexerExpectation,
                futures::channel::oneshot::Sender<SignificantChainEvent>,
            ),
        >,
    >,
}

impl Default for IndexerActionTracker {
    fn default() -> Self {
        Self {
            expectations: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait(? Send)]
impl ActionState for IndexerActionTracker {
    async fn match_and_resolve(&self, event: &SignificantChainEvent) -> Vec<IndexerExpectation> {
        let matches = self
            .expectations
            .read()
            .await
            .iter()
            .filter_map(|(h, (e, _))| e.test(event).then_some(*h))
            .collect::<Vec<_>>();
        let mut resolved = Vec::new();
        if !matches.is_empty() {
            let mut db = self.expectations.write().await;
            for hash in matches {
                let exp = db.remove(&hash).unwrap();
                if let Err(_) = exp.1.send(event.clone()) {
                    error!(
                        "failed to resolve actions in {:?}, because the action confirmation already timed out",
                        event
                    );
                } else {
                    resolved.push(exp.0);
                }
            }
        }
        resolved
    }
    async fn register_expectation(&self, exp: IndexerExpectation) -> Result<ExpectationResolver> {
        match self.expectations.write().await.entry(exp.tx_hash) {
            Entry::Occupied(_) => {
                return Err(CoreEthereumActionsError::InvalidState(format!(
                    "expectation of {} already present",
                    exp.tx_hash
                )))
            }
            Entry::Vacant(e) => {
                let (tx, rx) = futures::channel::oneshot::channel();
                e.insert((exp, tx));
                Ok(rx
                    .map_err(|_| CoreEthereumActionsError::InvalidState("action confirmation channel closed".into()))
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
    use async_std::prelude::FutureExt;
    use core_crypto::random::random_bytes;
    use core_crypto::types::Hash;
    use core_ethereum_types::chain_events::{ChainEventType, SignificantChainEvent};
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

        let exp = IndexerActionTracker::default();
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

        let sample_event_clone = sample_event.clone();
        async_std::task::spawn_local(async move {
            let hash = exp.match_and_resolve(&sample_event_clone).await;
            assert!(
                hash.iter().all(|e| e.tx_hash == random_hash),
                "hash must be present as resolved"
            );
        });

        assert_eq!(sample_event, resolution, "resolving event must be equal");
    }
}
