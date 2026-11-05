//! Connects the transport's events to the periodic state dump of `hopr-chain-connector`.
//!
//! The transport only reports [`TransportEvent`]s and knows nothing about the dump. This is the one
//! place that knows both sides.
use std::sync::Arc;

pub use hopr_chain_connector::state_dump::SectionValue;
use hopr_chain_connector::state_dump::{Direction, IntervalCounters, PhaseOutcome, SearchOutcome, SurbCounts};
use hopr_transport::{
    TransportEvent, TransportObserver,
    observer::{PathDirection, PhaseCounts, ProbeOutcome},
};

/// Produces one extra section of the state dump, such as the states of the node's strategies.
///
/// Called once per record from the reporter task: it must be cheap and must not block.
pub type StateDumpSection = Arc<dyn Fn() -> SectionValue + Send + Sync>;

/// An observer that records the transport's events in `counters`.
pub(crate) fn transport_observer(counters: IntervalCounters) -> TransportObserver {
    // The transport always runs the prober, so an idle window reports zero probes, not "unknown".
    counters.enable_probes();
    Arc::new(move |event| record(&counters, event))
}

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
        TransportEvent::Probe(ProbeOutcome::Neighbor { peer, succeeded }) => counters.neighbor_probe(peer, succeeded),
        TransportEvent::Probe(ProbeOutcome::Loopback { succeeded }) => counters.loopback_probe(succeeded),
    }
}

fn direction_of(direction: PathDirection) -> Direction {
    match direction {
        PathDirection::Forward => Direction::Forward,
        PathDirection::Return => Direction::Return,
    }
}

fn phase_of(phase: PhaseCounts) -> PhaseOutcome {
    PhaseOutcome {
        returned: phase.returned,
        eligible: phase.eligible,
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use hopr_api::{
        graph::ForwardAndReturnPath,
        types::{crypto::prelude::*, internal::NodeId, primitive::prelude::Address},
    };

    use super::*;

    #[test]
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
        observer(TransportEvent::Probe(ProbeOutcome::Neighbor {
            peer: destination,
            succeeded: false,
        }));
        observer(TransportEvent::Probe(ProbeOutcome::Loopback { succeeded: true }));

        let snapshot = counters.take_snapshot();
        assert_eq!(snapshot.totals.failed, 1);
        assert_eq!((snapshot.totals.searches, snapshot.totals.empty_searches), (1, 1));
        assert_eq!((snapshot.totals.phase1_returned, snapshot.totals.phase2_runs), (3, 1));
        assert_eq!((snapshot.surb_totals.minted, snapshot.surb_totals.returned), (4, 1));
        let probes = snapshot.probes.context("probes are enabled with the observer")?;
        assert_eq!((probes.neighbor.timed_out, probes.loopback.succeeded), (1, 1));
        Ok(())
    }

    #[test]
    fn successful_events_and_idle_windows_preserve_probe_registration() -> anyhow::Result<()> {
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
        observer(TransportEvent::Probe(ProbeOutcome::Neighbor {
            peer: destination,
            succeeded: true,
        }));
        observer(TransportEvent::Probe(ProbeOutcome::Loopback { succeeded: false }));
        let snapshot = counters.take_snapshot();
        assert_eq!(snapshot.totals.succeeded, 1);
        assert_eq!(snapshot.totals.failed, 0);
        assert_eq!(snapshot.totals.searches, 1);
        assert_eq!(snapshot.totals.empty_searches, 0);
        assert_eq!(snapshot.totals.phase2_runs, 0);
        let probes = snapshot.probes.context("registered probes")?;
        assert_eq!(probes.neighbor.succeeded, 1);
        assert_eq!(probes.loopback.timed_out, 1);

        let idle = counters.take_snapshot();
        assert_eq!(idle.totals.succeeded, 0);
        assert_eq!(idle.totals.searches, 0);
        let probes = idle.probes.context("registration survives the interval reset")?;
        assert_eq!(probes.neighbor.succeeded, 0);
        assert_eq!(probes.loopback.timed_out, 0);
        Ok(())
    }
}
