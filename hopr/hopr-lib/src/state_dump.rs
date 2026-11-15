//! Interfaces for an optional periodic state reporter supplied by the embedding application.
//!
//! `hopr-lib` attaches the observer before transport starts and owns the reporter's cancellation.
//! Collection, counter aggregation and serialization belong to the supplied implementation.
use std::{future::Future, sync::Arc};

use futures::future::BoxFuture;
use hopr_api::types::primitive::prelude::Address;
use hopr_transport::TransportObserver;
pub use serde_json::Value as SectionValue;

use crate::errors::HoprLibError;

/// Produces an extra state section. Called by the reporter; it must not block.
pub type StateDumpSection = Arc<dyn Fn() -> SectionValue + Send + Sync>;
/// Named sections registered by the embedding application.
pub type StateDumpSections = Vec<(&'static str, StateDumpSection)>;

/// An observer and its associated reporting task, ready to be managed by the node.
pub struct StateDump {
    pub(crate) observer: Option<TransportObserver>,
    pub(crate) process: BoxFuture<'static, ()>,
}

impl StateDump {
    /// Pair an optional transport observer with a lazy reporting future.
    ///
    /// The future is first polled after account readiness. The node aborts it with its other tasks.
    pub fn new(observer: Option<TransportObserver>, process: impl Future<Output = ()> + Send + 'static) -> Self {
        Self {
            observer,
            process: Box::pin(process),
        }
    }
}

/// Supplies a reporter using the same components as the node, without depending on their implementations.
///
/// Register it with [`HoprBuilderConfigured::with_state_dump`](crate::builder::HoprBuilderConfigured::with_state_dump).
pub trait StateDumpFactory<Chain, Graph, Net>: Send {
    /// Validate configuration before any component factory is called.
    fn prepare(&mut self) -> Result<(), HoprLibError> {
        Ok(())
    }

    /// Create the observer and lazy task before transport construction.
    /// Return `None` when reporting is disabled. This must not spawn or start the task itself.
    fn create(
        self,
        chain: Chain,
        graph: Graph,
        network: Net,
        safe: Address,
        sections: StateDumpSections,
    ) -> Option<StateDump>;
}

impl<Chain, Graph, Net> StateDumpFactory<Chain, Graph, Net> for () {
    fn create(self, _: Chain, _: Graph, _: Net, _: Address, _: StateDumpSections) -> Option<StateDump> {
        None
    }
}
