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

    /// Reads both counts without resetting them.
    fn peek(&self) -> (u64, u64) {
        (
            self.expected.load(Ordering::Relaxed),
            self.observed.load(Ordering::Relaxed),
        )
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
    /// Destination each pair of legs leads to, so a collapse can name what to re-plan.
    destinations: Arc<DashMap<ForwardAndReturnPath, OffchainPublicKey>>,
    /// What each pair's recent flushes say about whether it still works.
    silence: Arc<DashMap<ForwardAndReturnPath, Silence>>,
    /// Flushes since each destination was last reported, so one is not re-planned repeatedly.
    replanned: Arc<DashMap<OffchainPublicKey, u32>>,
}

/// Per-pair silence bookkeeping, carried between flushes.
#[derive(Debug, Default, Clone, Copy)]
struct Silence {
    /// Consecutive flushes in which the pair minted SURBs and got nothing back.
    runs: u32,
    /// Whether a reply has ever come back over this pair.
    ///
    /// Load-bearing: it is what turns the signal from "silent" into "stopped working". A pair that
    /// has never delivered may simply be too young -- a reply is credited to the flush it *arrives*
    /// in, so the first flushes of a new pair legitimately show mints with no replies yet.
    delivered: bool,
}

/// SURBs a pair must have minted in one flush before its silence counts as evidence.
///
/// Measured after a relayer was killed: the dead leg minted 2270 SURBs in the 5s that followed
/// while returning none. A handful is noise; thousands is not.
const MIN_EXPECTED_FOR_SILENCE: u64 = 20;

/// Consecutive silent flushes before a pair that used to deliver is called dead.
///
/// Flushes are one second apart. Three was measured to be too few: it fired six times during
/// healthy operation, once per flush, because a burst of mints can outrun its replies for a couple
/// of seconds without anything being wrong. Five keeps detection well inside the fifteen-second
/// budget -- the dead leg reads exactly zero from t+5s onward while its siblings keep returning.
const SILENT_FLUSHES_BEFORE_DEGRADED: u32 = 5;

/// Flushes a destination must wait before it can be re-planned again.
///
/// Silence accrues per pair of legs, but re-planning acts on the *destination*, and a destination
/// carries several pairs at once. Re-arming each pair individually therefore still allows one
/// destination to be re-planned every couple of seconds -- faster than a freshly chosen return path
/// can be established and start delivering, so each re-plan destroys the candidate the previous one
/// selected. Measured with the invalidation finally reaching the cache: fifteen re-plans during a
/// thirty-second healthy baseline, which collapsed it from 100% to 0.1% arrival.
///
/// Long enough for a new path to prove itself (it needs [`SILENT_FLUSHES_BEFORE_DEGRADED`] flushes
/// just to be judged), short enough to retry twice inside the recovery budget.
const FLUSHES_BETWEEN_REPLANS: u32 = 8;

impl SurbRoundTripRegistry {
    fn counters(&self, paths: ForwardAndReturnPath) -> Arc<SurbRoundTripCounters> {
        self.inner.entry(paths).or_default().value().clone()
    }

    /// Records that `count` SURBs were minted over these legs and are expected back.
    pub fn record_expected(&self, paths: ForwardAndReturnPath, count: u64, destination: OffchainPublicKey) {
        self.destinations.insert(paths, destination);
        self.counters(paths).record_expected(count);
    }

    /// Destinations whose return path has been silent for long enough to act on.
    ///
    /// Deliberately keyed on *no reply at all* rather than on a delivery rate. A rate cannot
    /// separate these cases: measured immediately after a kill, a healthy leg read 0.089 while the
    /// dead one read 0.123, and replies straddling a flush boundary push a healthy leg above 1.
    /// Only sustained silence distinguishes them, and it does so within seconds.
    ///
    /// The claim made is narrower than "this pair is silent": it is "this pair **used to deliver**
    /// and has now stopped". Silence alone was measured to fire during healthy operation, because
    /// a pair that has not yet returned its first reply is indistinguishable from a dead one.
    pub fn degraded_destinations(&self) -> Vec<OffchainPublicKey> {
        let mut degraded = Vec::new();

        // Age the per-destination cooldowns once per flush, dropping those that have served out.
        self.replanned.retain(|_, since| {
            *since += 1;
            *since < FLUSHES_BETWEEN_REPLANS
        });

        // Which destinations had *some* pair deliver in this flush.
        //
        // This is what makes silence mean anything. On its own, "minted and got nothing back" is
        // produced identically by a dead relayer and by a peer with nothing to say -- keep-alives
        // mint either way. Only a sibling pair still delivering to the same destination tells the
        // two apart, and it does so without a threshold: if the peer had gone quiet, every pair
        // would be silent together.
        let delivering: std::collections::HashSet<OffchainPublicKey> = self
            .inner
            .iter()
            .filter(|entry| entry.value().peek().1 > 0)
            .filter_map(|entry| self.destinations.get(entry.key()).map(|d| *d))
            .collect();

        for entry in self.inner.iter() {
            let paths = *entry.key();
            let (expected, observed) = entry.value().peek();
            let mut state = self.silence.entry(paths).or_default();

            if observed > 0 {
                // Delivering. Note that it ever worked, so future silence is meaningful.
                state.delivered = true;
                state.runs = 0;
                continue;
            }

            if expected < MIN_EXPECTED_FOR_SILENCE {
                // Too little went out to conclude anything. An idle pair is not a failing one.
                state.runs = 0;
                continue;
            }

            if !state.delivered {
                // Never returned anything yet, so there is no "stopped" to observe -- only a pair
                // too young to have been measured.
                continue;
            }

            let Some(dest) = self.destinations.get(&paths).map(|d| *d) else {
                continue;
            };

            // Corroboration: some other pair to this destination is still getting replies home, so
            // the peer is demonstrably talking and this pair's silence is its own fault. The run
            // resets when nothing corroborates, so `runs` counts consecutive flushes in which this
            // pair was silent *while the peer was demonstrably answering elsewhere* -- not merely
            // flushes in which it was quiet.
            if !delivering.contains(&dest) {
                state.runs = 0;
                continue;
            }

            state.runs += 1;
            if state.runs >= SILENT_FLUSHES_BEFORE_DEGRADED {
                // Re-arm rather than accumulate: re-planning the same destination on every
                // subsequent flush churns its cached candidates instead of letting the new
                // selection settle.
                state.runs = 0;
                // Only if this destination is not already serving out a cooldown: another of its
                // pairs may have reported it moments ago, and re-planning again now would discard
                // the candidate that re-plan just chose.
                if !self.replanned.contains_key(&dest) {
                    self.replanned.insert(dest, 0);
                    degraded.push(dest);
                }
            }
        }

        degraded
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

/// Resolves a node to the slot it occupies in a [`PathId`].
///
/// Taken as a function rather than as a graph so the codec decorator stays free of the graph's type
/// parameters -- the packet pipeline builder is already deeply generic and has no graph handle of
/// its own to thread through.
pub type PathSlotResolver = Arc<dyn Fn(&OffchainPublicKey) -> Option<u64> + Send + Sync>;

/// A resolver that places no node, so nothing is ever attributed.
///
/// Lets the decorator be installed unconditionally: with no graph to resolve against, every leg
/// fails to build an id and is skipped before it reaches the pending map.
pub fn no_path_slots() -> PathSlotResolver {
    Arc::new(|_| None)
}

/// Legs each outstanding SURB was minted over, shared between the two codec halves.
///
/// Minting happens on the encoder and the reply arrives at the decoder, so this **must** be one map
/// shared by both. Giving each half its own leaves every lookup missing and silently discards the
/// entire observation side of the metric.
pub type PendingLegs = moka::sync::Cache<HoprSurbId, ForwardAndReturnPath>;

/// Builds the shared pending map, bounded by capacity and TTL.
pub fn pending_legs(max_pending: u64) -> PendingLegs {
    moka::sync::Cache::builder()
        .max_capacity(max_pending)
        .time_to_live(PENDING_SURB_TTL)
        .build()
}

/// Reads path slots out of a network graph.
pub fn path_slots_of<G>(graph: G) -> PathSlotResolver
where
    G: NetworkGraphView<NodeId = OffchainPublicKey> + Send + Sync + 'static,
{
    Arc::new(move |key| graph.path_slot(key))
}

/// Builds the [`PathId`] of a leg from the nodes it visits.
///
/// Returns `None` if any node is unknown to the graph or the leg is longer than a [`PathId`] can
/// hold -- in both cases the id would name edges the round-trip did not use, which is worse than
/// reporting nothing.
fn path_id(slots: &PathSlotResolver, nodes: impl IntoIterator<Item = OffchainPublicKey>) -> Option<PathId> {
    let mut id = [0u64; PATH_ID_SLOTS];
    let mut len = 0;

    for node in nodes {
        if len == PATH_ID_SLOTS {
            return None;
        }
        id[len] = slots(&node)?;
        len += 1;
    }

    (len > 1).then_some(id)
}

/// Derives both legs of a round-trip from the routing that produced the SURB.
///
/// The forward leg starts at us and ends at the destination; the reply leg starts at that same
/// destination and ends back at us. Joining them at the destination is what lets the graph credit
/// the whole loop, and it is why the forward path's last hop seeds the reply leg.
fn round_trip_paths(
    slots: &PathSlotResolver,
    me: &OffchainPublicKey,
    forward: &[OffchainPublicKey],
    reply: &[OffchainPublicKey],
) -> Option<ForwardAndReturnPath> {
    let destination = *forward.last()?;

    Some(ForwardAndReturnPath {
        forward: path_id(slots, std::iter::once(*me).chain(forward.iter().copied()))?,
        reply: path_id(slots, std::iter::once(destination).chain(reply.iter().copied()))?,
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
    let (mut legs, mut total_expected, mut total_observed) = (0usize, 0u64, 0u64);

    for (paths, expected, observed) in registry.drain() {
        legs += 1;
        total_expected += expected;
        total_observed += observed;
        // Per pair, at debug: the aggregate cannot answer whether a dead relayer's legs diverge
        // from healthy ones, which is the property any trigger built on this signal depends on.
        tracing::debug!(
            forward = ?paths.forward,
            reply = ?paths.reply,
            expected,
            observed,
            "surb round-trip pair"
        );
        graph.record_edge::<NoPeerTelemetry, NoPathTelemetry>(MeasurableEdge::Surb(SurbTelemetry {
            paths,
            timestamp,
            expected,
            observed,
        }));
    }

    {
        // One aggregate line per interval rather than one per pair of legs.
        //
        // DIAGNOSTIC: at `info` while the recovery gap is under investigation. This is the pair of
        // numbers that separates "the counterparty never got SURBs" from "it got them, replied, and
        // the replies did not arrive" -- the two explanations left for a return path carrying
        // packets while the application receives almost nothing. Drop back to `debug` once settled.
        tracing::info!(
            legs,
            expected = total_expected,
            observed = total_observed,
            "surb round-trip flush tick"
        );
    }
}

/// Wraps a codec so the SURBs it mints and consumes are counted per pair of legs.
///
/// Encoding and decoding are otherwise passed straight through; a failure to attribute a round-trip
/// never fails a packet.
#[derive(Clone)]
pub struct SurbTelemetryCodec<C> {
    inner: C,
    me: OffchainPublicKey,
    slots: PathSlotResolver,
    registry: SurbRoundTripRegistry,
    /// Legs each outstanding SURB was minted over.
    ///
    /// Bounded by capacity and TTL rather than by replies arriving, so a peer that never replies
    /// cannot grow it without limit. Shared with the other codec half -- see [`PendingLegs`].
    pending: PendingLegs,
}

impl<C: std::fmt::Debug> std::fmt::Debug for SurbTelemetryCodec<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The slot resolver is a closure and has nothing meaningful to show.
        f.debug_struct("SurbTelemetryCodec")
            .field("inner", &self.inner)
            .field("me", &self.me)
            .finish_non_exhaustive()
    }
}

impl<C> SurbTelemetryCodec<C> {
    /// Wraps `inner`, resolving path slots with `slots` and accumulating into `registry`.
    pub fn new(
        inner: C,
        me: OffchainPublicKey,
        slots: PathSlotResolver,
        registry: SurbRoundTripRegistry,
        pending: PendingLegs,
    ) -> Self {
        Self {
            inner,
            me,
            slots,
            registry,
            pending,
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
        let Some(destination) = forward.last().copied() else {
            return;
        };

        tracing::debug!(
            minted = minted.len(),
            return_paths = return_paths.len(),
            forward_hops = forward.len(),
            "surb mint seen by telemetry"
        );

        for (surb_id, return_path) in minted.iter().zip(return_paths.iter()) {
            let reply: Vec<_> = return_path.transport_path().iter().copied().collect();
            let Some(paths) = round_trip_paths(&self.slots, &self.me, &forward, &reply) else {
                tracing::debug!(reply_hops = reply.len(), "surb round-trip legs did not resolve");
                continue;
            };

            self.registry.record_expected(paths, 1, destination);
            self.pending.insert(*surb_id, paths);
        }
    }

    /// Credits the legs a reply came back on.
    fn on_replied(&self, surb_id: &HoprSurbId) {
        // A SURB is single-use, so the association is consumed with it.
        match self.pending.remove(surb_id) {
            Some(paths) => self.registry.record_observed(paths, 1),
            None => tracing::debug!("reply on a surb with no pending legs"),
        }
    }
}

impl<C> PacketEncoder for SurbTelemetryCodec<C>
where
    C: PacketEncoder,
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

impl<C> PacketDecoder for SurbTelemetryCodec<C>
where
    C: PacketDecoder,
{
    type Error = C::Error;

    fn decode(
        &self,
        sender: PeerId,
        data: Bytes,
    ) -> Result<IncomingPacket, IncomingPacketError<<Self as PacketDecoder>::Error>> {
        let packet = self.inner.decode(sender, data)?;

        if let IncomingPacket::Final(f) = &packet {
            tracing::debug!(on_surb = f.replied_on_surb.is_some(), "final packet decoded");
        }

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
    fn recorder(graph: ChannelGraph) -> SurbTelemetryCodec<()> {
        let me = *graph.identity();
        SurbTelemetryCodec::new(
            (),
            me,
            path_slots_of(graph),
            SurbRoundTripRegistry::default(),
            pending_legs(128),
        )
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
        let (_graph, _me, peers) = graph_with(1);
        let registry = SurbRoundTripRegistry::default();
        let paths = ForwardAndReturnPath {
            forward: [0, 1, 0, 0, 0],
            reply: [1, 0, 0, 0, 0],
        };

        registry.record_expected(paths, 3, peers[0]);
        registry.record_observed(paths, 2);

        assert_eq!(vec![(paths, 3, 2)], registry.drain());
        // Counts belong to the interval that produced them, so a second flush must not re-report.
        assert!(registry.drain().is_empty());
    }

    #[test]
    fn round_trip_paths_should_join_the_legs_at_the_destination() -> anyhow::Result<()> {
        let (graph, me, peers) = graph_with(1);
        let destination = peers[0];

        let paths = round_trip_paths(&path_slots_of(graph.clone()), &me, &[destination], &[me])
            .context("both nodes are in the graph")?;

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
        assert!(round_trip_paths(&path_slots_of(graph), &me, &[stranger], &[me]).is_none());
    }

    #[test]
    fn round_trip_paths_should_be_none_for_a_leg_too_long_to_identify() {
        let (graph, me, peers) = graph_with(5);
        let forward: Vec<_> = peers.clone();

        // A `PathId` holds five slots; a longer leg cannot be named without dropping hops.
        assert!(round_trip_paths(&path_slots_of(graph), &me, &forward, &[me]).is_none());
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
    fn flushing_should_leave_nothing_behind_to_report_twice() -> anyhow::Result<()> {
        let (graph, me, peers) = graph_with(1);
        let destination = peers[0];
        let recorder = recorder(graph.clone());

        let routing = ResolvedTransportRouting::Forward {
            pseudonym: HoprPseudonym::random(),
            forward_path: ValidatedPath::direct(destination, address()),
            return_paths: vec![ValidatedPath::direct(me, address())],
        };
        recorder.on_minted(&routing, &[surb_id(1)]);
        recorder.on_replied(&surb_id(1));

        flush_into(&recorder.registry, &graph, 0);

        // Counts belong to the interval that produced them; carrying them into the next flush would
        // keep reporting a round-trip that happened once as though it kept happening.
        assert!(recorder.registry.drain().is_empty());
        Ok(())
    }

    #[test]
    fn a_reply_should_be_credited_when_the_halves_are_separate_instances() -> anyhow::Result<()> {
        // Regression: the pipeline wraps the encoder and the decoder in *separate* instances, so a
        // pending map built per-instance leaves every lookup missing and silently discards the
        // entire observation side. Every other test here drives one instance and cannot see it.
        let (graph, me, peers) = graph_with(1);
        let destination = peers[0];
        let registry = SurbRoundTripRegistry::default();
        let pending = pending_legs(128);

        let minting = SurbTelemetryCodec::new((), me, path_slots_of(graph.clone()), registry.clone(), pending.clone());
        let observing = SurbTelemetryCodec::new((), me, path_slots_of(graph), registry.clone(), pending);

        minting.on_minted(
            &ResolvedTransportRouting::Forward {
                pseudonym: HoprPseudonym::random(),
                forward_path: ValidatedPath::direct(destination, address()),
                return_paths: vec![ValidatedPath::direct(me, address())],
            },
            &[surb_id(1)],
        );
        observing.on_replied(&surb_id(1));

        let (_, expected, observed) = registry.drain()[0];
        assert_eq!(
            (1, 1),
            (expected, observed),
            "the reply must reach the legs the mint recorded"
        );
        Ok(())
    }

    /// One flush interval, in the order the flush task runs it: detect, then drain.
    ///
    /// Draining matters to these tests -- without it a single reply stays visible forever and
    /// silence can never be observed at all.
    fn flush(registry: &SurbRoundTripRegistry) -> Vec<OffchainPublicKey> {
        let degraded = registry.degraded_destinations();
        registry.drain();
        degraded
    }

    /// A pair that mints steadily and returns replies -- the healthy case.
    fn deliver(registry: &SurbRoundTripRegistry, paths: ForwardAndReturnPath, destination: OffchainPublicKey) {
        registry.record_expected(paths, MIN_EXPECTED_FOR_SILENCE, destination);
        registry.record_observed(paths, 1);
    }

    /// A pair that mints steadily and returns nothing.
    fn mint_only(registry: &SurbRoundTripRegistry, paths: ForwardAndReturnPath, destination: OffchainPublicKey) {
        registry.record_expected(paths, MIN_EXPECTED_FOR_SILENCE, destination);
    }

    /// A sibling pair to the same destination that keeps answering.
    ///
    /// Silence is only actionable against a peer that is demonstrably still talking, so any test
    /// of the silence logic has to keep one pair delivering or nothing can ever fire.
    fn heartbeat() -> ForwardAndReturnPath {
        ForwardAndReturnPath {
            forward: [0, 3, 0, 0, 0],
            reply: [3, 0, 0, 0, 0],
        }
    }

    fn legs() -> ForwardAndReturnPath {
        ForwardAndReturnPath {
            forward: [0, 1, 0, 0, 0],
            reply: [1, 0, 0, 0, 0],
        }
    }

    #[test]
    fn sustained_silence_should_name_the_destination_to_replan() {
        let (graph, me, peers) = graph_with(1);
        let destination = peers[0];
        let registry = SurbRoundTripRegistry::default();
        let paths = legs();

        // First establish that the pair works. Silence only means something against that.
        deliver(&registry, paths, destination);
        assert!(flush(&registry).is_empty());

        // Then it goes quiet while still minting, with a sibling still answering to prove the
        // peer is talking. One quiet interval is not a dead path, which is why the gate counts runs.
        for _ in 1..SILENT_FLUSHES_BEFORE_DEGRADED {
            mint_only(&registry, paths, destination);
            deliver(&registry, heartbeat(), destination);
            assert!(flush(&registry).is_empty());
        }
        mint_only(&registry, paths, destination);
        deliver(&registry, heartbeat(), destination);
        assert_eq!(vec![destination], flush(&registry));
        let _ = (graph, me);
    }

    /// Several pairs lead to one destination, but re-planning acts on the destination.
    ///
    /// Regression: silence accrues per pair, so pairs re-armed independently and between them kept
    /// re-planning the same destination every couple of seconds. With the invalidation finally
    /// reaching the path cache, that destroyed each freshly chosen return path before it could
    /// deliver -- a healthy baseline measured 0.1% arrival with fifteen re-plans inside it.
    ///
    /// The two pairs are staggered so they come due on *different* flushes; collapsing duplicates
    /// within one flush was already handled and is not what failed.
    #[test]
    fn one_destination_should_not_be_replanned_again_while_a_new_path_is_settling() {
        let (graph, me, peers) = graph_with(2);
        let destination = peers[0];
        let registry = SurbRoundTripRegistry::default();

        let first = legs();
        let second = ForwardAndReturnPath {
            forward: [0, 2, 0, 0, 0],
            reply: [2, 0, 0, 0, 0],
        };
        assert_ne!(
            first, second,
            "the two pairs must be distinct for this to test anything"
        );

        deliver(&registry, first, destination);
        deliver(&registry, second, destination);
        assert!(flush(&registry).is_empty());

        // `first` goes quiet immediately; `second` keeps answering for three more flushes, so its
        // own silence comes due well after the first re-plan.
        for _ in 1..SILENT_FLUSHES_BEFORE_DEGRADED {
            mint_only(&registry, first, destination);
            deliver(&registry, second, destination);
            assert!(flush(&registry).is_empty());
        }
        mint_only(&registry, first, destination);
        deliver(&registry, second, destination);
        assert_eq!(
            vec![destination],
            flush(&registry),
            "the first silence must be acted on"
        );

        // Now both stay silent. Suppression still resets the pair's counter, so it comes due
        // again and again; what must be bounded is how often the *destination* is acted on.
        const WINDOW: u32 = 4 * SILENT_FLUSHES_BEFORE_DEGRADED;
        let mut reports = 0;
        for _ in 0..WINDOW {
            mint_only(&registry, first, destination);
            mint_only(&registry, second, destination);
            deliver(&registry, heartbeat(), destination);
            reports += flush(&registry).len();
        }

        // Two pairs coming due every five flushes would otherwise report roughly eight times in
        // this window; measured in the cluster, fifteen re-plans in thirty flushes was enough to
        // hold a healthy session at 0.1% arrival.
        // An absolute bound, deliberately not derived from `FLUSHES_BETWEEN_REPLANS` -- a limit
        // computed from the constant under test moves with it and can never fail. Suppression
        // resets the pair, so it comes due every five flushes; the cooldown lets roughly every
        // other one through, which over twenty flushes is at most three.
        assert!(
            reports <= 3,
            "a destination must not be re-planned faster than a new path can settle: {reports} re-plans in {WINDOW} \
             flushes"
        );

        let _ = (graph, me);
    }

    /// A peer that simply stops talking must not be mistaken for a dead relayer.
    ///
    /// This is the case that killed every absolute gate: after a busy stretch the counters look
    /// exactly like a failing path -- SURBs still minted by keep-alives, nothing coming back. A
    /// delivered-ever bool, a recency decay, a volume threshold and a ratio collapse all fire here.
    /// Corroboration cannot, because when the peer goes quiet it goes quiet on *every* pair, so
    /// there is never a sibling to corroborate against.
    #[test]
    fn a_peer_that_goes_quiet_should_not_be_mistaken_for_a_dead_return_path() {
        let (graph, me, peers) = graph_with(2);
        let destination = peers[0];
        let registry = SurbRoundTripRegistry::default();

        let first = legs();
        let second = heartbeat();

        // A busy stretch: both pairs carrying real return traffic.
        for _ in 0..10 {
            deliver(&registry, first, destination);
            deliver(&registry, second, destination);
            assert!(flush(&registry).is_empty());
        }

        // The application goes idle. Keep-alives keep minting on both pairs; the peer has nothing
        // to say on either.
        for _ in 0..(4 * SILENT_FLUSHES_BEFORE_DEGRADED) {
            mint_only(&registry, first, destination);
            mint_only(&registry, second, destination);
            assert!(
                flush(&registry).is_empty(),
                "a quiet peer is not a dead return path, however long it stays quiet"
            );
        }

        let _ = (graph, me);
    }

    #[test]
    fn a_pair_that_never_delivered_should_never_be_called_degraded() {
        let (graph, me, peers) = graph_with(1);
        let destination = peers[0];
        let registry = SurbRoundTripRegistry::default();
        let paths = legs();

        // Measured regression: silence alone fired six times during healthy operation, once per
        // flush, on pairs whose replies had simply not landed yet. Minting hard and returning
        // nothing *yet* is what a young pair looks like, not what a dead one looks like.
        for _ in 0..SILENT_FLUSHES_BEFORE_DEGRADED + 3 {
            registry.record_expected(paths, MIN_EXPECTED_FOR_SILENCE * 10, destination);
            assert!(
                flush(&registry).is_empty(),
                "a pair that has never returned a reply is too young to call dead"
            );
        }
        let _ = (graph, me);
    }

    #[test]
    fn a_single_reply_should_clear_the_silence() {
        let (graph, me, peers) = graph_with(1);
        let destination = peers[0];
        let registry = SurbRoundTripRegistry::default();
        let paths = legs();

        deliver(&registry, paths, destination);
        assert!(flush(&registry).is_empty());

        for _ in 1..SILENT_FLUSHES_BEFORE_DEGRADED {
            mint_only(&registry, paths, destination);
            assert!(flush(&registry).is_empty());
        }

        // A path that delivers anything at all is not the failure this looks for, and the count
        // starts over rather than resuming where it left off.
        deliver(&registry, paths, destination);
        assert!(flush(&registry).is_empty());
        for _ in 1..SILENT_FLUSHES_BEFORE_DEGRADED {
            mint_only(&registry, paths, destination);
            assert!(flush(&registry).is_empty());
        }
        let _ = (graph, me);
    }

    #[test]
    fn a_degraded_pair_should_not_fire_again_on_the_next_flush() {
        let (graph, me, peers) = graph_with(1);
        let destination = peers[0];
        let registry = SurbRoundTripRegistry::default();
        let paths = legs();

        deliver(&registry, paths, destination);
        flush(&registry);
        for _ in 1..SILENT_FLUSHES_BEFORE_DEGRADED {
            mint_only(&registry, paths, destination);
            deliver(&registry, heartbeat(), destination);
            flush(&registry);
        }
        mint_only(&registry, paths, destination);
        deliver(&registry, heartbeat(), destination);
        assert_eq!(vec![destination], flush(&registry));

        // Re-planning the same destination every second churns its cached candidates instead of
        // letting the new selection settle, so the gate re-arms from zero.
        for _ in 1..SILENT_FLUSHES_BEFORE_DEGRADED {
            mint_only(&registry, paths, destination);
            deliver(&registry, heartbeat(), destination);
            assert!(flush(&registry).is_empty());
        }
        let _ = (graph, me);
    }

    #[test]
    fn an_idle_path_should_never_be_called_degraded() {
        let (graph, me, peers) = graph_with(1);
        let destination = peers[0];
        let registry = SurbRoundTripRegistry::default();
        let paths = legs();

        deliver(&registry, paths, destination);
        assert!(flush(&registry).is_empty());

        // Below the evidence floor: a trickle that happens not to have returned yet says nothing,
        // and treating it as failure would re-plan healthy paths during quiet periods.
        for _ in 0..SILENT_FLUSHES_BEFORE_DEGRADED + 2 {
            registry.record_expected(paths, MIN_EXPECTED_FOR_SILENCE - 1, destination);
            assert!(flush(&registry).is_empty());
        }
        let _ = (graph, me);
    }

    #[test]
    fn a_reply_on_a_surb_we_never_minted_should_be_ignored() {
        let (graph, ..) = graph_with(1);
        let recorder = recorder(graph);

        recorder.on_replied(&surb_id(9));

        assert!(recorder.registry.drain().is_empty());
    }
}
