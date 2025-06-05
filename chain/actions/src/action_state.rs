//! This module adds functionality of tracking the action results via expectations.
//!
//! It contains implementation of types necessary to perform tracking the
//! on-chain state of [Actions](hopr_chain_types::actions::Action).
//! Once an [Action](hopr_chain_types::actions::Action) is submitted to the chain, an [IndexerExpectation]
//! can be created and registered in an object implementing the [ActionState] trait.
//! The expectation typically consists of a required transaction hash and a predicate of [ChainEventType]
//! that must match on any chain event log in a block containing the given transaction hash.
//!
//! ### Example
//! Once the [RegisterSafe(`0x0123..ef`)](hopr_chain_types::actions::Action) action that has been submitted via
//! [ActionQueue](crate::action_queue::ActionQueue) in a transaction with hash `0xabcd...00`.
//! The [IndexerExpectation] is such that whatever block that will contain the TX hash `0xabcd..00` must also contain
//! a log that matches [NodeSafeRegistered(`0x0123..ef`)](ChainEventType) event type.
//! If such event is never encountered by the Indexer, the safe registration action naturally times out.
use std::{
    collections::{HashMap, hash_map::Entry},
    fmt::{Debug, Formatter},
    future::Future,
    pin::Pin,
    sync::Arc,
};

use async_lock::{RwLock, RwLockUpgradableReadGuardArc};
use async_trait::async_trait;
use futures::{FutureExt, TryFutureExt, channel};
use hopr_chain_types::chain_events::{ChainEventType, SignificantChainEvent};
use hopr_crypto_types::types::Hash;
use tracing::{debug, error, trace};

use crate::errors::{ChainActionsError, Result};

/// Future that resolves once an expectation is matched by some [SignificantChainEvent].
/// Also allows mocking in tests.
pub type ExpectationResolver = Pin<Box<dyn Future<Output = Result<SignificantChainEvent>> + Send>>;

/// Allows tracking state of an [Action](hopr_chain_types::actions::Action) via registering
/// [IndexerExpectations](IndexerExpectation) on [SignificantChainEvents](SignificantChainEvent) coming from the Indexer
/// and resolving them as they are matched. Once expectations are matched, they are automatically unregistered.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ActionState {
    /// Tries to match the given event against the registered expectations.
    /// Each matched expectation is resolved, unregistered and returned.
    async fn match_and_resolve(&self, event: &SignificantChainEvent) -> Vec<IndexerExpectation>;

    /// Registers new [IndexerExpectation].
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

/// Implements [action state](ActionState) tracking using a non-persistent in-memory hash table of
/// assumed [IndexerExpectations](IndexerExpectation).
#[derive(Debug, Clone)]
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
    #[tracing::instrument(level = "debug", skip(self))]
    async fn match_and_resolve(&self, event: &SignificantChainEvent) -> Vec<IndexerExpectation> {
        let db_read_lock = self.expectations.upgradable_read_arc().await;

        let matched_keys = db_read_lock
            .iter()
            .filter_map(|(k, (e, _))| e.test(event).then_some(*k))
            .collect::<Vec<_>>();

        if matched_keys.is_empty() {
            trace!(%event, "no expectations matched for event");
            return Vec::new();
        }

        debug!(count = matched_keys.len(), %event, "found expectations to match event",);

        let mut db_write_lock = RwLockUpgradableReadGuardArc::upgrade(db_read_lock).await;

        matched_keys
            .into_iter()
            .filter_map(|key| {
                db_write_lock
                    .remove(&key)
                    .and_then(|(exp, sender)| match sender.send(event.clone()) {
                        Ok(_) => {
                            debug!(%event, tx_hash = %key, "expectation resolved");
                            Some(exp)
                        }
                        Err(_) => {
                            error!(
                                %event, "failed to resolve actions, because the action confirmation already timed out",
                            );
                            None
                        }
                    })
            })
            .collect()
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn register_expectation(&self, exp: IndexerExpectation) -> Result<ExpectationResolver> {
        match self.expectations.write_arc().await.entry(exp.tx_hash) {
            Entry::Occupied(_) => {
                // TODO: currently cannot register multiple expectations for the same TX hash
                return Err(ChainActionsError::InvalidState(format!(
                    "expectation for tx {} already present",
                    exp.tx_hash
                )));
            }
            Entry::Vacant(e) => {
                let (tx, rx) = channel::oneshot::channel();
                e.insert((exp, tx));
                Ok(rx.map_err(|_| ChainActionsError::ExpectationUnregistered).boxed())
            }
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn unregister_expectation(&self, tx_hash: Hash) {
        self.expectations.write_arc().await.remove(&tx_hash);
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use anyhow::Context;
    use hex_literal::hex;
    use hopr_chain_types::chain_events::{ChainEventType, NetworkRegistryStatus, SignificantChainEvent};
    use hopr_crypto_random::random_bytes;
    use hopr_crypto_types::types::Hash;
    use hopr_primitive_types::prelude::*;
    use tokio::time::timeout;

    use crate::{
        action_state::{ActionState, IndexerActionTracker, IndexerExpectation},
        errors::ChainActionsError,
    };

    lazy_static::lazy_static! {
        // some random address
        static ref RANDY: Address = hex!("60f8492b6fbaf86ac2b064c90283d8978a491a01").into();
    }

    #[tokio::test]
    async fn test_expectation_should_resolve() -> anyhow::Result<()> {
        let random_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());
        let sample_event = SignificantChainEvent {
            tx_hash: random_hash,
            event_type: ChainEventType::NodeSafeRegistered(*RANDY),
        };

        let exp = Arc::new(IndexerActionTracker::default());

        let sample_event_clone = sample_event.clone();
        let exp_clone = exp.clone();
        tokio::task::spawn(async move {
            tokio::time::sleep(Duration::from_millis(200)).await; // delay
            let hash = exp_clone.match_and_resolve(&sample_event_clone).await;
            assert!(
                hash.iter().all(|e| e.tx_hash == random_hash),
                "hash must be present as resolved"
            );
        });

        let resolution = timeout(
            Duration::from_secs(5),
            exp.register_expectation(IndexerExpectation::new(random_hash, move |e| {
                matches!(e, ChainEventType::NodeSafeRegistered(_))
            }))
            .await?,
        )
        .await?
        .context("resolver must not be cancelled")?;

        assert_eq!(sample_event, resolution, "resolving event must be equal");

        Ok(())
    }

    #[tokio::test]
    async fn test_expectation_should_error_when_unregistered() -> anyhow::Result<()> {
        let sample_event = SignificantChainEvent {
            tx_hash: Hash::from(random_bytes::<{ Hash::SIZE }>()),
            event_type: ChainEventType::NodeSafeRegistered(*RANDY),
        };

        let exp = Arc::new(IndexerActionTracker::default());

        let sample_event_clone = sample_event.clone();
        let exp_clone = exp.clone();
        tokio::task::spawn(async move {
            tokio::time::sleep(Duration::from_millis(200)).await; // delay
            exp_clone.unregister_expectation(sample_event_clone.tx_hash).await;
        });

        let err = timeout(
            Duration::from_secs(5),
            exp.register_expectation(IndexerExpectation::new(sample_event.tx_hash, move |e| {
                matches!(e, ChainEventType::NodeSafeRegistered(_))
            }))
            .await?,
        )
        .await?
        .expect_err("should return with error");

        assert!(
            matches!(err, ChainActionsError::ExpectationUnregistered),
            "should notify on unregistration"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_expectation_should_resolve_and_filter() -> anyhow::Result<()> {
        let tx_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());
        let sample_events = vec![
            SignificantChainEvent {
                tx_hash: Hash::from(random_bytes::<{ Hash::SIZE }>()),
                event_type: ChainEventType::NodeSafeRegistered(*RANDY),
            },
            SignificantChainEvent {
                tx_hash,
                event_type: ChainEventType::NetworkRegistryUpdate(*RANDY, NetworkRegistryStatus::Denied),
            },
            SignificantChainEvent {
                tx_hash,
                event_type: ChainEventType::NetworkRegistryUpdate(*RANDY, NetworkRegistryStatus::Allowed),
            },
        ];

        let exp = Arc::new(IndexerActionTracker::default());

        let sample_events_clone = sample_events.clone();
        let exp_clone = exp.clone();
        tokio::task::spawn(async move {
            for sample_event in sample_events_clone {
                tokio::time::sleep(Duration::from_millis(200)).await; // delay
                exp_clone.match_and_resolve(&sample_event).await;
            }
        });

        let resolution = timeout(
            Duration::from_secs(5),
            exp.register_expectation(IndexerExpectation::new(tx_hash, move |e| {
                matches!(
                    e,
                    ChainEventType::NetworkRegistryUpdate(_, NetworkRegistryStatus::Allowed)
                )
            }))
            .await?,
        )
        .await?
        .context("resolver must not be cancelled")?;

        assert_eq!(sample_events[2], resolution, "resolving event must be equal");

        Ok(())
    }

    #[tokio::test]
    async fn test_expectation_should_resolve_multiple_expectations() -> anyhow::Result<()> {
        let sample_events = vec![
            SignificantChainEvent {
                tx_hash: Hash::from(random_bytes::<{ Hash::SIZE }>()),
                event_type: ChainEventType::NodeSafeRegistered(*RANDY),
            },
            SignificantChainEvent {
                tx_hash: Hash::from(random_bytes::<{ Hash::SIZE }>()),
                event_type: ChainEventType::NetworkRegistryUpdate(*RANDY, NetworkRegistryStatus::Denied),
            },
            SignificantChainEvent {
                tx_hash: Hash::from(random_bytes::<{ Hash::SIZE }>()),
                event_type: ChainEventType::NetworkRegistryUpdate(*RANDY, NetworkRegistryStatus::Allowed),
            },
        ];

        let exp = Arc::new(IndexerActionTracker::default());

        let sample_events_clone = sample_events.clone();
        let exp_clone = exp.clone();
        tokio::task::spawn(async move {
            for sample_event in sample_events_clone {
                tokio::time::sleep(Duration::from_millis(100)).await; // delay
                exp_clone.match_and_resolve(&sample_event).await;
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
            .context("should register 1")?,
            exp.register_expectation(IndexerExpectation::new(sample_events[0].tx_hash, move |e| {
                matches!(e, ChainEventType::NodeSafeRegistered(_))
            }))
            .await
            .context("should register 2")?,
        ];

        let resolutions = timeout(Duration::from_secs(5), futures::future::try_join_all(registered_exps))
            .await?
            .context("no resolver can cancel")?;

        assert_eq!(sample_events[2], resolutions[0], "resolving event 1 must be equal");
        assert_eq!(sample_events[0], resolutions[1], "resolving event 2 must be equal");

        Ok(())
    }
}
