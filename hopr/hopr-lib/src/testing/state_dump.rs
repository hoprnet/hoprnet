//! Concrete state-dump adapter for emulated-node tests.
use std::sync::Arc;

use hopr_chain_connector::state_dump::{Direction, IntervalCounters, PhaseOutcome, SearchOutcome, SurbCounts};
use hopr_transport::{
    TransportEvent, TransportObserver,
    observer::{PathDirection, PhaseCounts},
};

/// An observer that records the transport's events in `counters`.
pub(crate) fn transport_observer(counters: IntervalCounters) -> TransportObserver {
    Arc::new(move |event| record(&counters, event))
}

/// Translate each transport event into the companion collector counter operation.
fn record(counters: &IntervalCounters, event: TransportEvent) {
    match event {
        TransportEvent::PathResolved {
            direction,
            destination,
            succeeded,
        } => counters.resolution(direction_of(direction), destination, succeeded),
        TransportEvent::PathSearched {
            direction,
            destination,
            phase1,
            phase2,
            retained,
        } => counters.search(
            direction_of(direction),
            destination,
            SearchOutcome {
                phase1: phase_of(phase1),
                phase2: phase2.map(phase_of),
                retained,
            },
        ),
        TransportEvent::Surb {
            destination,
            paths,
            minted,
            returned,
        } => counters.surb(destination, paths, SurbCounts { minted, returned }),
    }
}

/// Map the transport direction to the collector direction without changing its meaning.
fn direction_of(direction: PathDirection) -> Direction {
    match direction {
        PathDirection::Forward => Direction::Forward,
        PathDirection::Return => Direction::Return,
    }
}

/// Preserve returned and eligible candidate counts when adapting a search phase.
fn phase_of(phase: PhaseCounts) -> PhaseOutcome {
    PhaseOutcome {
        returned: phase.returned,
        eligible: phase.eligible,
    }
}

use hopr_api::{chain::HoprChainApi, graph::HoprGraphApi, network::NetworkView, types::primitive::prelude::Address};
use hopr_chain_connector::state_dump::{Collector, Config, StateRead};

use crate::{
    errors::HoprLibError,
    state_dump::{StateDump, StateDumpFactory, StateDumpSections},
};

/// Connector adapter used only by emulated-node tests.
#[derive(Default)]
pub(crate) struct Factory {
    config: Option<Config>,
}

impl<Chain, Graph, Net> StateDumpFactory<Chain, Graph, Net> for Factory
where
    Chain: HoprChainApi + StateRead + Send + Sync + 'static,
    Graph: HoprGraphApi<HoprNodeId = hopr_api::OffchainPublicKey> + Send + Sync + 'static,
    <Graph as hopr_api::graph::NetworkGraphTraverse>::Observed:
        hopr_api::graph::traits::EdgeObservableRead + Send + 'static,
    Net: NetworkView + Send + Sync + 'static,
{
    /// Validate and retain the environment configuration before constructing test components.
    fn prepare(&mut self) -> Result<(), HoprLibError> {
        self.config = Some(Config::from_env().map_err(|e| HoprLibError::GeneralError(e.to_string()))?);
        Ok(())
    }

    /// Attach counters and named sections to a lazy collector, or disable it for a zero interval.
    fn create(
        self,
        chain: Chain,
        graph: Graph,
        network: Net,
        safe: Address,
        sections: StateDumpSections,
    ) -> Option<StateDump> {
        let config = self.config?;
        config.interval()?;
        let counters = IntervalCounters::default();
        let observer = transport_observer(counters.clone());
        let collector = sections.into_iter().fold(
            Collector::new(chain, graph, network, safe, config),
            |collector, (name, section)| collector.with_section(name, move || section()),
        );
        Some(StateDump::new(Some(observer), async move {
            collector.run(Some(counters)).await
        }))
    }
}

#[cfg(test)]
mod tests {
    use hopr_api::{
        graph::ForwardAndReturnPath,
        types::{crypto::prelude::*, internal::NodeId, primitive::prelude::Address},
    };

    use super::*;

    #[test]
    /// Verify that every event variant updates its matching collector counters.
    fn every_transport_event_reaches_the_counters() -> anyhow::Result<()> {
        let counters = IntervalCounters::default();
        let observer = transport_observer(counters.clone());
        let destination = NodeId::Chain(Address::from([1; 20]));
        let peer = *OffchainKeypair::random().public();

        observer(TransportEvent::PathResolved {
            direction: PathDirection::Forward,
            destination,
            succeeded: false,
        });
        observer(TransportEvent::PathSearched {
            direction: PathDirection::Return,
            destination,
            phase1: PhaseCounts {
                returned: 3,
                eligible: 2,
            },
            phase2: Some(PhaseCounts {
                returned: 1,
                eligible: 0,
            }),
            retained: 0,
        });
        observer(TransportEvent::Surb {
            destination: peer,
            paths: ForwardAndReturnPath {
                forward: [1; 5],
                reply: [2; 5],
            },
            minted: 4,
            returned: 1,
        });

        let snapshot = counters.take_snapshot();
        assert_eq!(snapshot.totals.failed, 1);
        assert_eq!((snapshot.totals.searches, snapshot.totals.empty_searches), (1, 1));
        assert_eq!((snapshot.totals.phase1_returned, snapshot.totals.phase2_runs), (3, 1));
        assert_eq!((snapshot.surb_totals.minted, snapshot.surb_totals.returned), (4, 1));
        assert!(snapshot.probes.is_none(), "probe reporting stays in the producer");
        Ok(())
    }

    #[test]
    /// Keep probe outcomes out of state dumps, including idle intervals.
    fn successful_events_and_idle_windows_do_not_register_probe_counters() -> anyhow::Result<()> {
        let counters = IntervalCounters::default();
        let observer = transport_observer(counters.clone());
        let destination = NodeId::Chain(Address::from([2; 20]));
        observer(TransportEvent::PathResolved {
            direction: PathDirection::Return,
            destination,
            succeeded: true,
        });
        observer(TransportEvent::PathSearched {
            direction: PathDirection::Forward,
            destination,
            phase1: PhaseCounts {
                returned: 2,
                eligible: 1,
            },
            phase2: None,
            retained: 1,
        });
        let snapshot = counters.take_snapshot();
        assert_eq!(snapshot.totals.succeeded, 1);
        assert_eq!(snapshot.totals.failed, 0);
        assert_eq!(snapshot.totals.searches, 1);
        assert_eq!(snapshot.totals.empty_searches, 0);
        assert_eq!(snapshot.totals.phase2_runs, 0);
        assert!(snapshot.probes.is_none(), "probe reporting stays in the producer");

        let idle = counters.take_snapshot();
        assert_eq!(idle.totals.succeeded, 0);
        assert_eq!(idle.totals.searches, 0);
        assert!(idle.probes.is_none(), "idle intervals must not invent probe counters");
        Ok(())
    }
}
