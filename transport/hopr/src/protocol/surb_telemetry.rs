//! Turns SURB round-trips into edge telemetry for the network graph.
//!
//! A SURB rides a forward path to reach its destination and carries a return path for the reply to
//! come back on. When the reply arrives, that is proof both legs passed end to end -- evidence the
//! graph already wants, produced by traffic a session was sending anyway. Unlike a probe it costs
//! no extra packets and accrues at data rates, which is what lets a dead relayer be noticed in
//! seconds rather than after a probe success rate has moved behind a path cache.
//!
//! # Why one layer owns both directions
//!
//! Minting and consuming happen at opposite ends of the pipeline, but only together do they mean
//! anything: the mint says what was expected, the reply says what was observed. Wrapping both codec
//! halves keeps the SURB-to-path association private to this module, so nothing below has to carry
//! a path identity it has no other use for.

use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use bytes::Bytes;
use dashmap::DashMap;
use hopr_api::{
    graph::{ForwardAndReturnPath, MeasurableEdge, MeasurablePath, MeasurablePeer, NetworkGraphView, SurbTelemetry},
    types::{
        crypto::{prelude::*, types::OffchainPublicKey},
        internal::prelude::*,
    },
};
use hopr_crypto_packet::{HoprSurb, prelude::PacketSignals};
use hopr_protocol_hopr::{IncomingPacket, IncomingPacketError, OutgoingPacket, PacketDecoder, PacketEncoder};

/// Slots in a [`PathId`], and therefore the longest leg that can be reported.
const PATH_ID_SLOTS: usize = 5;

/// How long a minted SURB stays eligible to be observed coming back.
///
/// A SURB the replier never uses would otherwise sit in the pending map forever, so the expectation
/// it raised is retired after this long. Generous relative to a round-trip, because expiring a SURB
/// that is merely slow would count a success as a loss.
const PENDING_SURB_TTL: Duration = Duration::from_secs(60);

/// Expected/observed counts for one pair of legs, accumulated between flushes.
///
/// Both are monotonic within an interval, so recording is a pair of relaxed atomic adds on the
/// packet hot path -- no lock, no allocation.
#[derive(Debug, Default)]
pub struct SurbRoundTripCounters {
    expected: AtomicU64,
    observed: AtomicU64,
}

impl SurbRoundTripCounters {
    fn record_expected(&self, count: u64) {
        self.expected.fetch_add(count, Ordering::Relaxed);
    }

    fn record_observed(&self, count: u64) {
        self.observed.fetch_add(count, Ordering::Relaxed);
    }

    /// Takes both counts, resetting them to zero.
    fn take(&self) -> (u64, u64) {
        (
            self.expected.swap(0, Ordering::Relaxed),
            self.observed.swap(0, Ordering::Relaxed),
        )
    }
}

/// Round-trip counts keyed by the legs they were observed over.
///
/// Batching is not an optimisation here but a requirement: recording an edge takes a write lock on
/// the whole graph, a packet carries several SURBs, and a session at 0.5 MB/s moves hundreds of
/// packets a second -- reporting per event would contend the graph thousands of times a second.
#[derive(Debug, Clone, Default)]
pub struct SurbRoundTripRegistry {
    inner: Arc<DashMap<ForwardAndReturnPath, Arc<SurbRoundTripCounters>>>,
}

impl SurbRoundTripRegistry {
    fn counters(&self, paths: ForwardAndReturnPath) -> Arc<SurbRoundTripCounters> {
        self.inner.entry(paths).or_default().value().clone()
    }

    /// Records that `count` SURBs were minted over these legs and are expected back.
    pub fn record_expected(&self, paths: ForwardAndReturnPath, count: u64) {
        self.counters(paths).record_expected(count);
    }

    /// Records that `count` replies arrived over these legs.
    pub fn record_observed(&self, paths: ForwardAndReturnPath, count: u64) {
        self.counters(paths).record_observed(count);
    }

    /// Takes every non-empty entry, resetting the counts.
    pub fn drain(&self) -> Vec<(ForwardAndReturnPath, u64, u64)> {
        self.inner
            .iter()
            .filter_map(|entry| {
                let (expected, observed) = entry.value().take();
                (expected > 0 || observed > 0).then(|| (*entry.key(), expected, observed))
            })
            .collect()
    }
}

/// Builds the [`PathId`] of a leg from the nodes it visits.
///
/// Returns `None` if any node is unknown to the graph or the leg is longer than a [`PathId`] can
/// hold -- in both cases the id would name edges the round-trip did not use, which is worse than
/// reporting nothing.
fn path_id<G>(graph: &G, nodes: impl IntoIterator<Item = OffchainPublicKey>) -> Option<PathId>
where
    G: NetworkGraphView<NodeId = OffchainPublicKey>,
{
    let mut id = [0u64; PATH_ID_SLOTS];
    let mut len = 0;

    for node in nodes {
        if len == PATH_ID_SLOTS {
            return None;
        }
        id[len] = graph.path_slot(&node)?;
        len += 1;
    }

    (len > 1).then_some(id)
}

/// Derives both legs of a round-trip from the routing that produced the SURB.
///
/// The forward leg starts at us and ends at the destination; the reply leg starts at that same
/// destination and ends back at us. Joining them at the destination is what lets the graph credit
/// the whole loop, and it is why the forward path's last hop seeds the reply leg.
fn round_trip_paths<G>(
    graph: &G,
    forward: &[OffchainPublicKey],
    reply: &[OffchainPublicKey],
) -> Option<ForwardAndReturnPath>
where
    G: NetworkGraphView<NodeId = OffchainPublicKey>,
{
    let me = *graph.identity();
    let destination = *forward.last()?;

    Some(ForwardAndReturnPath {
        forward: path_id(graph, std::iter::once(me).chain(forward.iter().copied()))?,
        reply: path_id(graph, std::iter::once(destination).chain(reply.iter().copied()))?,
    })
}

/// `record_edge` is generic over peer and path telemetry that a SURB observation does not carry.
///
/// These are uninhabited, so they satisfy the bounds without claiming a round-trip has neighbour or
/// probe telemetry attached.
#[derive(Debug, Clone)]
pub enum NoPeerTelemetry {}

impl MeasurablePeer for NoPeerTelemetry {
    fn peer(&self) -> &OffchainPublicKey {
        match *self {}
    }

    fn rtt(&self) -> Duration {
        match *self {}
    }
}

/// Counterpart to [`NoPeerTelemetry`] for the path half of `record_edge`.
#[derive(Debug, Clone)]
pub enum NoPathTelemetry {}

impl MeasurablePath for NoPathTelemetry {
    fn id(&self) -> &[u8] {
        match *self {}
    }

    fn path(&self) -> &[u8] {
        match *self {}
    }

    fn timestamp(&self) -> u128 {
        match *self {}
    }
}

/// Turns accumulated counts into graph observations.
///
/// Separate from the flush task itself so the batching can be tested without a timer.
pub fn flush_into<G>(registry: &SurbRoundTripRegistry, graph: &G, timestamp: u128)
where
    G: hopr_api::graph::NetworkGraphUpdate,
{
    for (paths, expected, observed) in registry.drain() {
        tracing::trace!(expected, observed, "flushing surb round-trip counts");
        graph.record_edge::<NoPeerTelemetry, NoPathTelemetry>(MeasurableEdge::Surb(SurbTelemetry {
            paths,
            timestamp,
            expected,
            observed,
        }));
    }
}

/// Wraps a codec so the SURBs it mints and consumes are counted per pair of legs.
///
/// Encoding and decoding are otherwise passed straight through; a failure to attribute a round-trip
/// never fails a packet.
#[derive(Debug, Clone)]
pub struct SurbTelemetryCodec<C, G> {
    inner: C,
    graph: G,
    registry: SurbRoundTripRegistry,
    /// Legs each outstanding SURB was minted over.
    ///
    /// Bounded by capacity and TTL rather than by replies arriving, so a peer that never replies
    /// cannot grow it without limit.
    pending: moka::sync::Cache<HoprSurbId, ForwardAndReturnPath>,
}

impl<C, G> SurbTelemetryCodec<C, G>
where
    G: NetworkGraphView<NodeId = OffchainPublicKey>,
{
    /// Wraps `inner`, resolving paths against `graph` and accumulating into `registry`.
    pub fn new(inner: C, graph: G, registry: SurbRoundTripRegistry, max_pending: u64) -> Self {
        Self {
            inner,
            graph,
            registry,
            pending: moka::sync::Cache::builder()
                .max_capacity(max_pending)
                .time_to_live(PENDING_SURB_TTL)
                .build(),
        }
    }

    /// Associates freshly minted SURBs with the legs they were minted over.
    ///
    /// Opener order is significant, so the ids zip positionally with the return paths that produced
    /// them.
    fn on_minted(&self, routing: &ResolvedTransportRouting<HoprSurb>, minted: &[HoprSurbId]) {
        let ResolvedTransportRouting::Forward {
            forward_path,
            return_paths,
            ..
        } = routing
        else {
            return;
        };

        let forward: Vec<_> = forward_path.transport_path().iter().copied().collect();

        for (surb_id, return_path) in minted.iter().zip(return_paths.iter()) {
            let reply: Vec<_> = return_path.transport_path().iter().copied().collect();
            let Some(paths) = round_trip_paths(&self.graph, &forward, &reply) else {
                continue;
            };

            self.registry.record_expected(paths, 1);
            self.pending.insert(*surb_id, paths);
        }
    }

    /// Credits the legs a reply came back on.
    fn on_replied(&self, surb_id: &HoprSurbId) {
        // A SURB is single-use, so the association is consumed with it.
        if let Some(paths) = self.pending.remove(surb_id) {
            self.registry.record_observed(paths, 1);
        }
    }
}

impl<C, G> PacketEncoder for SurbTelemetryCodec<C, G>
where
    C: PacketEncoder,
    G: NetworkGraphView<NodeId = OffchainPublicKey>,
{
    type Error = C::Error;

    fn encode_packet<T: AsRef<[u8]> + Send + 'static, S: Into<PacketSignals> + Send + 'static>(
        &self,
        data: T,
        routing: ResolvedTransportRouting<HoprSurb>,
        signals: S,
    ) -> Result<OutgoingPacket, <Self as PacketEncoder>::Error> {
        let packet = self.inner.encode_packet(data, routing.clone(), signals)?;
        self.on_minted(&routing, &packet.minted_surbs);
        Ok(packet)
    }

    fn encode_acknowledgements(
        &self,
        acks: &[VerifiedAcknowledgement],
        destination: &OffchainPublicKey,
    ) -> Result<OutgoingPacket, <Self as PacketEncoder>::Error> {
        self.inner.encode_acknowledgements(acks, destination)
    }
}

impl<C, G> PacketDecoder for SurbTelemetryCodec<C, G>
where
    C: PacketDecoder,
    G: NetworkGraphView<NodeId = OffchainPublicKey>,
{
    type Error = C::Error;

    fn decode(
        &self,
        sender: PeerId,
        data: Bytes,
    ) -> Result<IncomingPacket, IncomingPacketError<<Self as PacketDecoder>::Error>> {
        let packet = self.inner.decode(sender, data)?;

        if let IncomingPacket::Final(final_packet) = &packet
            && let Some(surb_id) = final_packet.replied_on_surb
        {
            self.on_replied(&surb_id);
        }

        Ok(packet)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use hopr_api::{
        graph::NetworkGraphWrite,
        types::{
            crypto::{
                crypto_traits::Randomizable,
                prelude::{ChainKeypair, Keypair, OffchainKeypair},
            },
            primitive::primitives::Address,
        },
    };
    use hopr_network_graph::petgraph::ChannelGraph;

    use super::*;

    /// The trait impls are pass-throughs, so the tests drive the attribution logic directly.
    fn recorder(graph: ChannelGraph) -> SurbTelemetryCodec<(), ChannelGraph> {
        SurbTelemetryCodec::new((), graph, SurbRoundTripRegistry::default(), 128)
    }

    fn surb_id(seed: u8) -> HoprSurbId {
        [seed; SURB_ID_SIZE]
    }

    /// A cluster of `me` plus `count` peers, all known to the graph.
    fn address() -> Address {
        ChainKeypair::random().public().to_address()
    }

    fn graph_with(count: usize) -> (ChannelGraph, OffchainPublicKey, Vec<OffchainPublicKey>) {
        let me = *OffchainKeypair::random().public();
        let graph = ChannelGraph::new(me);

        let peers = (0..count)
            .map(|_| {
                let key = *OffchainKeypair::random().public();
                graph.add_node(key);
                key
            })
            .collect();

        (graph, me, peers)
    }

    #[test]
    fn drain_should_report_nothing_before_anything_is_recorded() {
        assert!(SurbRoundTripRegistry::default().drain().is_empty());
    }

    #[test]
    fn drain_should_take_the_counts_and_leave_the_entry_empty() {
        let registry = SurbRoundTripRegistry::default();
        let paths = ForwardAndReturnPath {
            forward: [0, 1, 0, 0, 0],
            reply: [1, 0, 0, 0, 0],
        };

        registry.record_expected(paths, 3);
        registry.record_observed(paths, 2);

        assert_eq!(vec![(paths, 3, 2)], registry.drain());
        // Counts belong to the interval that produced them, so a second flush must not re-report.
        assert!(registry.drain().is_empty());
    }

    #[test]
    fn round_trip_paths_should_join_the_legs_at_the_destination() -> anyhow::Result<()> {
        let (graph, me, peers) = graph_with(1);
        let destination = peers[0];

        let paths = round_trip_paths(&graph, &[destination], &[me]).context("both nodes are in the graph")?;

        let me_slot = graph.path_slot(&me).context("self is in the graph")?;
        let dest_slot = graph.path_slot(&destination).context("destination is in the graph")?;

        // The forward leg starts at us and ends at the destination; the reply leg starts at that
        // same destination, which is what lets the graph credit the loop as one continuous walk.
        assert_eq!([me_slot, dest_slot, 0, 0, 0], paths.forward);
        assert_eq!([dest_slot, me_slot, 0, 0, 0], paths.reply);
        Ok(())
    }

    #[test]
    fn round_trip_paths_should_be_none_when_a_node_is_unknown_to_the_graph() {
        let (graph, me, _) = graph_with(0);
        let stranger = *OffchainKeypair::random().public();

        // Reporting an id built from a node the graph cannot place would credit whichever edges the
        // wrong slots happened to name.
        assert!(round_trip_paths(&graph, &[stranger], &[me]).is_none());
    }

    #[test]
    fn round_trip_paths_should_be_none_for_a_leg_too_long_to_identify() {
        let (graph, me, peers) = graph_with(5);
        let forward: Vec<_> = peers.clone();

        // A `PathId` holds five slots; a longer leg cannot be named without dropping hops.
        assert!(round_trip_paths(&graph, &forward, &[me]).is_none());
    }

    #[test]
    fn minting_a_surb_should_raise_an_expectation_over_the_legs_it_was_minted_on() -> anyhow::Result<()> {
        let (graph, me, peers) = graph_with(1);
        let destination = peers[0];
        let recorder = recorder(graph);

        let routing = ResolvedTransportRouting::Forward {
            pseudonym: HoprPseudonym::random(),
            forward_path: ValidatedPath::direct(destination, address()),
            return_paths: vec![ValidatedPath::direct(me, address())],
        };

        recorder.on_minted(&routing, &[surb_id(1)]);

        let drained = recorder.registry.drain();
        assert_eq!(1, drained.len());
        let (_, expected, observed) = drained[0];
        assert_eq!(1, expected);
        assert_eq!(0, observed, "nothing has come back yet");
        Ok(())
    }

    #[test]
    fn a_reply_should_be_credited_to_the_legs_its_surb_was_minted_on() -> anyhow::Result<()> {
        let (graph, me, peers) = graph_with(1);
        let destination = peers[0];
        let recorder = recorder(graph);

        let routing = ResolvedTransportRouting::Forward {
            pseudonym: HoprPseudonym::random(),
            forward_path: ValidatedPath::direct(destination, address()),
            return_paths: vec![ValidatedPath::direct(me, address())],
        };

        recorder.on_minted(&routing, &[surb_id(1)]);
        recorder.on_replied(&surb_id(1));

        let drained = recorder.registry.drain();
        assert_eq!(1, drained.len());
        let (_, expected, observed) = drained[0];
        assert_eq!((1, 1), (expected, observed));
        Ok(())
    }

    #[test]
    fn a_surb_should_only_be_credited_once() -> anyhow::Result<()> {
        let (graph, me, peers) = graph_with(1);
        let destination = peers[0];
        let recorder = recorder(graph);

        let routing = ResolvedTransportRouting::Forward {
            pseudonym: HoprPseudonym::random(),
            forward_path: ValidatedPath::direct(destination, address()),
            return_paths: vec![ValidatedPath::direct(me, address())],
        };

        recorder.on_minted(&routing, &[surb_id(1)]);
        recorder.on_replied(&surb_id(1));
        // A SURB is single-use; a second reply on it cannot be genuine, and counting it would push
        // the delivery ratio above what was actually expected.
        recorder.on_replied(&surb_id(1));

        let (_, expected, observed) = recorder.registry.drain()[0];
        assert_eq!((1, 1), (expected, observed));
        Ok(())
    }

    #[test]
    fn a_reply_on_a_surb_we_never_minted_should_be_ignored() {
        let (graph, ..) = graph_with(1);
        let recorder = recorder(graph);

        recorder.on_replied(&surb_id(9));

        assert!(recorder.registry.drain().is_empty());
    }
}
