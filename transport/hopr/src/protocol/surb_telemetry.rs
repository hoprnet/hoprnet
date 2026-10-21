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
    collections::{HashMap, VecDeque},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use bytes::Bytes;
use dashmap::{DashMap, DashSet};
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
pub(crate) const PATH_ID_SLOTS: usize = 5;

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
#[derive(Debug, Clone)]
pub struct SurbRoundTripRegistry {
    inner: Arc<DashMap<ForwardAndReturnPath, Arc<SurbRoundTripCounters>>>,
    /// Per-relayer health from reply races and connectivity, shared with the path planner.
    relayer_health: Arc<RelayerHealth>,
    /// Every outstanding SURB, shared by both codec halves and by the write-off of a dead relayer.
    pending: PendingLegs,
}

/// Memory bound only: at the 60 s TTL it is reached above ~1.7k SURBs minted per second.
const MAX_PENDING_SURBS: u64 = 100_000;

impl Default for SurbRoundTripRegistry {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            relayer_health: Default::default(),
            pending: pending_legs(MAX_PENDING_SURBS),
        }
    }
}

impl SurbRoundTripRegistry {
    fn counters(&self, paths: ForwardAndReturnPath) -> Arc<SurbRoundTripCounters> {
        self.inner.entry(paths).or_default().value().clone()
    }

    /// Records that `count` SURBs were minted over these legs and are expected back.
    pub fn record_expected(&self, paths: ForwardAndReturnPath, count: u64) {
        self.counters(paths).record_expected(count);
    }

    /// The per-relayer health, for callers that weight or exclude paths by it.
    pub fn relayer_health(&self) -> Arc<RelayerHealth> {
        self.relayer_health.clone()
    }

    /// Per destination, how many outstanding SURBs return through any of `relayers`, and their share
    /// of all outstanding. SURBs already spent into a dead relayer stay outstanding too, so the count
    /// alone can outgrow what the counterparty still holds.
    pub fn unusable_through(&self, relayers: &[OffchainPublicKey]) -> HashMap<OffchainPublicKey, (u64, f64)> {
        if relayers.is_empty() {
            return HashMap::new();
        }
        let mut counts: HashMap<OffchainPublicKey, (u64, u64)> = HashMap::new();
        for (_, pending) in self.pending.iter() {
            let (through, total) = counts.entry(pending.destination).or_default();
            *total += 1;
            if pending.return_relayers.iter().flatten().any(|r| relayers.contains(r)) {
                *through += 1;
            }
        }
        counts
            .into_iter()
            .filter(|(_, (through, _))| *through > 0)
            .map(|(destination, (through, total))| (destination, (through, through as f64 / total as f64)))
            .collect()
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

/// The relayers of a reply leg, from the first after the replier to the last before us.
pub(crate) type ReturnRelayers = [Option<OffchainPublicKey>; PATH_ID_SLOTS];

/// What a minted SURB is remembered by until its reply comes home.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PendingSurb {
    /// Legs the round-trip traverses, for the counts and graph attribution.
    paths: ForwardAndReturnPath,
    /// Who the SURB was handed to.
    destination: OffchainPublicKey,
    /// Relayers the reply passes, any of which a late or missing reply implicates.
    return_relayers: ReturnRelayers,
    /// When this SURB and its partner were minted; both round-trips run from here.
    minted_at: Instant,
    /// The SURB minted in the same packet over another return path, which this one races.
    partner: Option<HoprSurbId>,
    /// Whether this is the second SURB of its pair, the only one never buried behind newer SURBs in
    /// the replier's store, so the only one whose lateness is down to its return path.
    second: bool,
    /// The partner's relayers and when its reply came home, once it has.
    partner_replied: Option<(ReturnRelayers, Instant)>,
}

/// What each outstanding SURB is remembered by, shared between the two codec halves.
///
/// Minting happens on the encoder and the reply arrives at the decoder, so this **must** be one map
/// shared by both. Giving each half its own leaves every lookup missing and silently discards the
/// entire observation side of the metric.
type PendingLegs = moka::sync::Cache<HoprSurbId, PendingSurb>;

/// Builds the shared pending map, bounded by capacity and TTL.
fn pending_legs(max_pending: u64) -> PendingLegs {
    moka::sync::Cache::builder()
        .max_capacity(max_pending)
        .time_to_live(PENDING_SURB_TTL)
        .build()
}

/// Per-relayer health from reply races: two SURBs of one packet share mint time, forward path and
/// store slot, so the gap between their replies is down to the return paths alone.
#[derive(Debug, Default)]
pub struct RelayerHealth {
    relayers: DashMap<OffchainPublicKey, RaceHistory>,
    /// Relayers we lost our own connection to, until it comes back.
    unreachable: DashSet<OffchainPublicKey>,
    /// Relayers judged dead at the last flush.
    dead: DashSet<OffchainPublicKey>,
    /// Factors per candidate set, cleared every telemetry flush, so a per-packet draw only reads.
    cache: parking_lot::Mutex<HashMap<Vec<Vec<OffchainPublicKey>>, Vec<f64>>>,
}

/// One race as one of its relayers saw it.
#[derive(Debug, Clone, Copy)]
struct Race {
    at: Instant,
    /// Seconds this relayer finished after its partner, negative when before.
    margin: f64,
    /// Seconds the faster of the two replies took to come home.
    winning_round_trip: f64,
}

#[derive(Debug, Default)]
struct RaceHistory {
    races: VecDeque<Race>,
    /// Races whose partner is home while this relayer's reply is not, by the SURB still out. Each
    /// counts as lost by at least the time since the partner came home.
    open: HashMap<HoprSurbId, Race>,
    /// How far back its races count, set when it is judged; zero until then.
    memory: Duration,
}

/// The fewest races whose median no single race decides.
const MIN_RACES: usize = 3;

impl RaceHistory {
    fn forget_older_than(&mut self, memory: Duration) {
        while self.races.len() > MIN_RACES && self.races.front().is_some_and(|race| race.at.elapsed() > memory) {
            self.races.pop_front();
        }
    }

    /// A black hole: at least [`MIN_RACES`] replies through it are out for longer than any settled
    /// race ever took, and none has come home since the oldest of them was due.
    fn is_black_hole(&self) -> bool {
        if self.races.len() < MIN_RACES {
            return false;
        }
        let usual = self.races.iter().map(|race| race.margin.abs()).fold(0.0, f64::max);
        let overdue: Vec<Instant> = self
            .open
            .values()
            .filter(|race| race.at.elapsed().as_secs_f64() > usual)
            .map(|race| race.at)
            .collect();
        let Some(oldest_overdue) = overdue.iter().min().copied() else {
            return false;
        };
        overdue.len() >= MIN_RACES && self.races.back().is_none_or(|settled| settled.at < oldest_overdue)
    }
}

fn relayers_of(leg: &ReturnRelayers) -> impl Iterator<Item = OffchainPublicKey> + '_ {
    leg.iter().flatten().copied()
}

impl RelayerHealth {
    /// Records that the reply over `later` came home `by` after the one over `earlier`, which took
    /// `winning_round_trip`, both on SURBs of one packet. Relayers on both legs cancel out.
    pub(crate) fn record_race(
        &self,
        later: &ReturnRelayers,
        later_surb: HoprSurbId,
        earlier: &ReturnRelayers,
        winning_round_trip: Duration,
        by: Duration,
    ) {
        let (at, winning_round_trip) = (Instant::now(), winning_round_trip.as_secs_f64());
        let race = |margin| Race {
            at,
            margin,
            winning_round_trip,
        };
        for relayer in relayers_of(later) {
            if let Some(mut history) = self.relayers.get_mut(&relayer) {
                history.open.remove(&later_surb);
            }
            if !earlier.contains(&Some(relayer)) {
                self.record(relayer, race(by.as_secs_f64()));
            }
        }
        for relayer in relayers_of(earlier).filter(|relayer| !later.contains(&Some(*relayer))) {
            self.record(relayer, race(-by.as_secs_f64()));
        }
    }

    /// Records that the reply over `home` came home in `winning_round_trip` while `surb`, over
    /// `waiting`, is still out.
    pub(crate) fn open_race(
        &self,
        waiting: &ReturnRelayers,
        surb: HoprSurbId,
        home: &ReturnRelayers,
        winning_round_trip: Duration,
    ) {
        let race = Race {
            at: Instant::now(),
            margin: 0.0,
            winning_round_trip: winning_round_trip.as_secs_f64(),
        };
        for relayer in relayers_of(waiting).filter(|relayer| !home.contains(&Some(*relayer))) {
            self.relayers.entry(relayer).or_default().open.insert(surb, race);
        }
    }

    fn record(&self, relayer: OffchainPublicKey, race: Race) {
        let mut history = self.relayers.entry(relayer).or_default();
        history.races.push_back(race);
        let memory = history.memory;
        if !memory.is_zero() {
            history.forget_older_than(memory);
        }
    }

    /// Our own connection to `relayer` came or went.
    pub fn set_reachable(&self, relayer: OffchainPublicKey, reachable: bool) {
        if reachable {
            self.unreachable.remove(&relayer);
        } else {
            self.unreachable.insert(relayer);
        }
    }

    /// Whether `relayer` must not be on any path, in any position.
    pub fn is_dead(&self, relayer: &OffchainPublicKey) -> bool {
        self.unreachable.contains(relayer) || self.dead.contains(relayer)
    }

    /// Re-judges which relayers are dead, once per flush, returning those newly dead and recovered.
    /// Races open longer than a SURB is remembered can no longer settle, so they are dropped.
    pub fn refresh_verdicts(&self) -> (Vec<OffchainPublicKey>, Vec<OffchainPublicKey>) {
        let mut dead: Vec<OffchainPublicKey> = self.unreachable.iter().map(|relayer| *relayer).collect();
        for mut entry in self.relayers.iter_mut() {
            entry.open.retain(|_, race| race.at.elapsed() <= PENDING_SURB_TTL);
            if entry.is_black_hole() && !dead.contains(entry.key()) {
                dead.push(*entry.key());
            }
        }
        let newly_dead: Vec<OffchainPublicKey> = dead
            .iter()
            .filter(|relayer| !self.dead.contains(relayer))
            .copied()
            .collect();
        // A relayer gone from the candidates races no more; its history only grows the map.
        self.relayers.retain(|_, history| {
            !history.open.is_empty()
                || history
                    .races
                    .back()
                    .is_some_and(|race| race.at.elapsed() <= history.memory.max(PENDING_SURB_TTL))
        });
        let recovered: Vec<OffchainPublicKey> = self
            .dead
            .iter()
            .map(|relayer| *relayer)
            .filter(|relayer| !dead.contains(relayer))
            .collect();
        self.dead.retain(|relayer| dead.contains(relayer));
        for relayer in &newly_dead {
            self.dead.insert(*relayer);
        }
        (newly_dead, recovered)
    }

    /// Every relayer currently judged dead.
    pub fn dead_relayers(&self) -> Vec<OffchainPublicKey> {
        let unreachable = self
            .unreachable
            .iter()
            .map(|relayer| *relayer)
            .filter(|r| !self.dead.contains(r));
        self.dead.iter().map(|relayer| *relayer).chain(unreachable).collect()
    }

    fn forget_factors(&self) {
        self.cache.lock().clear();
    }

    /// Per-candidate weight from the reply races of every relayer on it, relative to the best
    /// candidate: a candidate is as good as its worst relayer, and `0.0` with a dead one. `memory`
    /// is how many of its own margins a relayer's races are remembered for (`PathPlannerConfig`).
    pub fn factors(&self, candidates: &[Vec<OffchainPublicKey>], memory: u32) -> Vec<f64> {
        if let Some(factors) = self.cache.lock().get(candidates) {
            return factors.clone();
        }
        let factors = self.compute_factors(candidates, memory);
        self.cache.lock().insert(candidates.to_vec(), factors.clone());
        factors
    }

    /// A relayer is weighted down by how many round-trips it finishes behind its partners, the
    /// fastest round-trip any candidate's relayer made being the unit: lagging by less costs
    /// nothing, as healthy relayers differ by that much anyway.
    fn compute_factors(&self, candidates: &[Vec<OffchainPublicKey>], memory: u32) -> Vec<f64> {
        let mut judged: HashMap<OffchainPublicKey, Option<(f64, f64)>> = HashMap::new();
        for relayer in candidates.iter().flatten() {
            judged.entry(*relayer).or_insert_with(|| self.judge(relayer, memory));
        }
        let unit = judged
            .values()
            .flatten()
            .map(|(_, fastest)| *fastest)
            .min_by(f64::total_cmp);
        let relayer_factor = |relayer: &OffchainPublicKey| {
            if self.is_dead(relayer) {
                return 0.0;
            }
            match (judged.get(relayer).copied().flatten(), unit) {
                (Some((lateness, _)), Some(unit)) if lateness > unit => unit / lateness,
                _ => 1.0,
            }
        };
        let factors: Vec<f64> = candidates
            .iter()
            .map(|relayers| relayers.iter().map(relayer_factor).fold(1.0, f64::min))
            .collect();
        // Never all zero, which would leave nothing to draw: the best candidate sets the scale.
        let best = factors.iter().copied().fold(0.0, f64::max);
        if best > 0.0 {
            factors.iter().map(|factor| factor / best).collect()
        } else {
            vec![1.0; candidates.len()]
        }
    }

    /// Lateness (median margin behind its partners, less its uncertainty) and fastest round-trip.
    /// Races are remembered for `memory` times margin plus spread, so the window scales with the relayer.
    fn judge(&self, relayer: &OffchainPublicKey, memory: u32) -> Option<(f64, f64)> {
        let mut history = self.relayers.get_mut(relayer)?;
        let remembered = history.memory;
        let recent = |race: &Race| remembered.is_zero() || race.at.elapsed() <= remembered;
        let still_out = history.open.values().filter(|race| recent(race)).map(|race| Race {
            margin: race.at.elapsed().as_secs_f64(),
            ..*race
        });
        let races: Vec<Race> = history
            .races
            .iter()
            .rev()
            .enumerate()
            .take_while(|(i, race)| *i < MIN_RACES || recent(race))
            .map(|(_, race)| *race)
            .chain(still_out)
            .collect();
        if races.len() < MIN_RACES {
            return None;
        }
        let mut margins: Vec<f64> = races.iter().map(|race| race.margin).collect();
        let margin = median(&mut margins);
        let spread = median(&mut margins.iter().map(|m| (m - margin).abs()).collect::<Vec<_>>());
        history.memory = Duration::from_secs_f64(f64::from(memory) * (margin.abs() + spread));
        let memory = history.memory;
        history.forget_older_than(memory);

        let fastest = races
            .iter()
            .map(|race| race.winning_round_trip)
            .fold(f64::INFINITY, f64::min);
        Some((margin - spread / (races.len() as f64).sqrt(), fastest))
    }
}

/// Lower median; `values` must not be empty.
fn median(values: &mut [f64]) -> f64 {
    values.sort_unstable_by(f64::total_cmp);
    values[(values.len() - 1) / 2]
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
    registry.relayer_health.forget_factors();

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
    pub fn new(inner: C, me: OffchainPublicKey, slots: PathSlotResolver, registry: SurbRoundTripRegistry) -> Self {
        Self {
            inner,
            me,
            slots,
            registry,
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

        let replies: Vec<Vec<OffchainPublicKey>> = return_paths
            .iter()
            .map(|return_path| return_path.transport_path().iter().copied().collect())
            .collect();
        // A reply leg is `[R, ..., me]`: everything before us is a relayer a late or missing reply
        // implicates. A direct reply has none.
        let return_relayers: Vec<ReturnRelayers> = replies
            .iter()
            .map(|reply| {
                let mut relayers = [None; PATH_ID_SLOTS];
                for (slot, relayer) in relayers.iter_mut().zip(&reply[..reply.len().saturating_sub(1)]) {
                    *slot = Some(*relayer);
                }
                relayers
            })
            .collect();
        let partners = race_partners(minted, &return_relayers);
        let minted_at = Instant::now();

        for (i, surb_id) in minted.iter().enumerate().take(replies.len()) {
            let Some(paths) = round_trip_paths(&self.slots, &self.me, &forward, &replies[i]) else {
                tracing::debug!(reply_hops = replies[i].len(), "surb round-trip legs did not resolve");
                continue;
            };

            self.registry.record_expected(paths, 1);
            self.registry.pending.insert(
                *surb_id,
                PendingSurb {
                    paths,
                    destination,
                    return_relayers: return_relayers[i],
                    minted_at,
                    partner: partners[i],
                    second: i % 2 == 1,
                    partner_replied: None,
                },
            );
        }
    }

    /// Credits the legs a reply came back on, and settles its race once the partner's is home too.
    fn on_replied(&self, surb_id: &HoprSurbId) {
        // A SURB is single-use, so the association is consumed with it.
        let Some(pending) = self.registry.pending.remove(surb_id) else {
            tracing::debug!("reply on a surb with no pending legs");
            return;
        };
        self.registry.record_observed(pending.paths, 1);

        let health = &self.registry.relayer_health;
        // Only the second SURB settles: the first may have been buried in the replier's store.
        if let Some((partner_relayers, partner_home)) = pending.partner_replied {
            if !pending.second {
                return;
            }
            health.record_race(
                &pending.return_relayers,
                *surb_id,
                &partner_relayers,
                partner_home.duration_since(pending.minted_at),
                partner_home.elapsed(),
            );
        } else if let Some(partner) = pending.partner {
            // Atomic, so a partner consumed by a concurrent decode is not brought back.
            let home = Instant::now();
            let updated = self
                .registry
                .pending
                .entry(partner)
                .and_compute_with(|current| match current {
                    Some(current) => {
                        let mut waiting = current.into_value();
                        waiting.partner_replied = Some((pending.return_relayers, home));
                        moka::ops::compute::Op::Put(waiting)
                    }
                    None => moka::ops::compute::Op::Nop,
                });
            if let moka::ops::compute::CompResult::ReplacedWith(waiting) = updated
                && waiting.value().second
            {
                health.open_race(
                    &waiting.value().return_relayers,
                    partner,
                    &pending.return_relayers,
                    home.duration_since(pending.minted_at),
                );
            }
        }
    }
}

/// The SURB each one races: its neighbour in mint order (first with second, third with fourth),
/// when both return over relayers and not over the same ones.
fn race_partners(minted: &[HoprSurbId], return_relayers: &[ReturnRelayers]) -> Vec<Option<HoprSurbId>> {
    (0..minted.len())
        .map(|i| {
            let neighbour = i ^ 1;
            match (return_relayers.get(i), return_relayers.get(neighbour)) {
                (Some(own), Some(other)) if own[0].is_some() && other[0].is_some() && own != other => {
                    minted.get(neighbour).copied()
                }
                _ => None,
            }
        })
        .collect()
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
        SurbTelemetryCodec::new((), me, path_slots_of(graph), SurbRoundTripRegistry::default())
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

    /// A few to a few dozen milliseconds: what healthy relayers win or lose a race by.
    fn jitter(i: u64) -> Duration {
        Duration::from_millis(5 + i * 7 % 40)
    }

    /// The round-trip of the faster reply in the races below.
    const ROUND_TRIP: Duration = Duration::from_millis(200);

    fn key() -> OffchainPublicKey {
        *OffchainKeypair::random().public()
    }

    fn leg(relayers: &[OffchainPublicKey]) -> ReturnRelayers {
        let mut leg = [None; PATH_ID_SLOTS];
        for (slot, relayer) in leg.iter_mut().zip(relayers) {
            *slot = Some(*relayer);
        }
        leg
    }

    fn margins_of(health: &RelayerHealth, relayer: &OffchainPublicKey) -> Vec<f64> {
        health
            .relayers
            .get(relayer)
            .map(|history| history.races.iter().map(|race| race.margin).collect())
            .unwrap_or_default()
    }

    fn open_races_of(health: &RelayerHealth, relayer: &OffchainPublicKey) -> usize {
        health.relayers.get(relayer).map_or(0, |history| history.open.len())
    }

    fn ago(by: Duration) -> anyhow::Result<Instant> {
        Instant::now().checked_sub(by).context("monotonic clock too young")
    }

    fn settled_since(health: &RelayerHealth, relayer: OffchainPublicKey, at: Instant, margin: Duration) {
        for _ in 0..MIN_RACES {
            health.record(
                relayer,
                Race {
                    at,
                    margin: margin.as_secs_f64(),
                    winning_round_trip: ROUND_TRIP.as_secs_f64(),
                },
            );
        }
    }

    fn backdate_open(health: &RelayerHealth, relayer: OffchainPublicKey, at: Instant) {
        if let Some(mut history) = health.relayers.get_mut(&relayer) {
            history.open.values_mut().for_each(|race| race.at = at);
        }
    }

    fn open_since(health: &RelayerHealth, relayer: OffchainPublicKey, count: u8, since: Instant) {
        for seed in 1..=count {
            health.relayers.entry(relayer).or_default().open.insert(
                surb_id(seed),
                Race {
                    at: since,
                    margin: 0.0,
                    winning_round_trip: ROUND_TRIP.as_secs_f64(),
                },
            );
        }
    }

    #[test]
    fn a_relayer_losing_its_races_by_seconds_should_lose_its_weight() {
        let health = RelayerHealth::default();
        let [fast, also_fast, stalled] = [(); 3].map(|_| key());
        for i in 0..40 {
            let (winner, loser) = if i % 2 == 0 {
                (fast, also_fast)
            } else {
                (also_fast, fast)
            };
            health.record_race(&leg(&[loser]), surb_id(0), &leg(&[winner]), ROUND_TRIP, jitter(i));
            health.record_race(
                &leg(&[stalled]),
                surb_id(0),
                &leg(&[winner]),
                ROUND_TRIP,
                Duration::from_secs(3) + jitter(i),
            );
        }

        let factors = health.factors(&[vec![fast], vec![also_fast], vec![stalled], vec![]], 4);
        assert_eq!(
            &[1.0, 1.0],
            &factors[..2],
            "trading wins by milliseconds is not a stall"
        );
        assert!(
            factors[2] < 0.1,
            "a relayer fifteen round-trips late must lose its weight, got {}",
            factors[2]
        );
        assert_eq!(1.0, factors[3]);
    }

    /// The relayers both legs share see the same delay, so the race blames only the ones that differ.
    #[test]
    fn a_stalled_middle_relayer_should_take_the_blame_not_its_neighbours() {
        let health = RelayerHealth::default();
        let [entry, stalled, healthy, exit, other_entry, other_exit] = [(); 6].map(|_| key());
        for i in 0..40 {
            if i % 2 == 0 {
                health.record_race(
                    &leg(&[entry, stalled, exit]),
                    surb_id(0),
                    &leg(&[entry, healthy, exit]),
                    ROUND_TRIP,
                    Duration::from_secs(3) + jitter(i),
                );
            } else {
                health.record_race(
                    &leg(&[other_entry, healthy, other_exit]),
                    surb_id(0),
                    &leg(&[entry, healthy, exit]),
                    ROUND_TRIP,
                    jitter(i),
                );
            }
        }

        let factors = health.factors(
            &[
                vec![entry, stalled, exit],
                vec![entry, healthy, exit],
                vec![other_entry, healthy, other_exit],
            ],
            4,
        );
        assert!(
            factors[0] < 0.1,
            "the path through the stall must lose its weight, got {}",
            factors[0]
        );
        assert_eq!(
            &[1.0, 1.0],
            &factors[1..],
            "its neighbours on that path are not to blame"
        );
    }

    #[test]
    fn a_relayer_lagging_by_less_than_a_round_trip_should_keep_its_weight() {
        let health = RelayerHealth::default();
        let [near, far] = [(); 2].map(|_| key());
        for i in 0..40 {
            health.record_race(
                &leg(&[far]),
                surb_id(0),
                &leg(&[near]),
                ROUND_TRIP,
                Duration::from_millis(150) + jitter(i) / 10,
            );
        }

        assert_eq!(vec![1.0, 1.0], health.factors(&[vec![near], vec![far]], 4));
    }

    #[test]
    fn a_relayer_with_too_few_races_should_stay_neutral() {
        let health = RelayerHealth::default();
        let [fast, young] = [(); 2].map(|_| key());
        for _ in 1..MIN_RACES {
            health.record_race(
                &leg(&[young]),
                surb_id(0),
                &leg(&[fast]),
                ROUND_TRIP,
                Duration::from_secs(3),
            );
        }

        assert_eq!(vec![1.0, 1.0], health.factors(&[vec![fast], vec![young]], 4));
    }

    #[test]
    fn a_recovered_relayer_should_regain_its_weight_once_its_late_races_are_forgotten() -> anyhow::Result<()> {
        let health = RelayerHealth::default();
        let [fast, recovered] = [(); 2].map(|_| key());
        let long_ago = Instant::now()
            .checked_sub(Duration::from_secs(60))
            .context("monotonic clock younger than a minute")?;
        for i in 0..20 {
            health.record(
                recovered,
                Race {
                    at: long_ago,
                    margin: 3.0 + jitter(i).as_secs_f64(),
                    winning_round_trip: ROUND_TRIP.as_secs_f64(),
                },
            );
        }
        assert!(
            health.factors(&[vec![fast], vec![recovered]], 4)[1] < 0.1,
            "precondition: judged stalled"
        );

        health.forget_factors();
        for i in 0..20 {
            let (later, earlier) = if i % 2 == 0 {
                (fast, recovered)
            } else {
                (recovered, fast)
            };
            health.record_race(&leg(&[later]), surb_id(0), &leg(&[earlier]), ROUND_TRIP, jitter(i));
        }
        assert_eq!(vec![1.0, 1.0], health.factors(&[vec![fast], vec![recovered]], 4));
        Ok(())
    }

    /// A reply still out while its partner's is home has lost by at least that long already, so a
    /// stall shows before the held replies finally arrive.
    #[test]
    fn a_reply_still_out_should_count_as_lost_by_at_least_its_wait() -> anyhow::Result<()> {
        let health = RelayerHealth::default();
        let [fast, also_fast, stalled] = [(); 3].map(|_| key());
        for i in 0..20 {
            health.record_race(&leg(&[also_fast]), surb_id(0), &leg(&[fast]), ROUND_TRIP, jitter(i));
        }
        health.record_race(&leg(&[stalled]), surb_id(0), &leg(&[fast]), ROUND_TRIP, jitter(0));
        let partners_home = Instant::now()
            .checked_sub(Duration::from_secs(2))
            .context("monotonic clock younger than two seconds")?;
        open_since(&health, stalled, 5, partners_home);

        let factors = health.factors(&[vec![fast], vec![also_fast], vec![stalled]], 4);
        assert_eq!(&[1.0, 1.0], &factors[..2]);
        assert!(
            factors[2] < 0.2,
            "replies two seconds overdue must cost the relayer its weight, got {}",
            factors[2]
        );
        Ok(())
    }

    #[test]
    fn a_relayer_whose_replies_stop_coming_home_should_be_judged_dead_everywhere() -> anyhow::Result<()> {
        let health = RelayerHealth::default();
        let [entry, black_hole, exit, healthy] = [(); 4].map(|_| key());
        settled_since(&health, black_hole, ago(Duration::from_secs(10))?, jitter(0));
        for seed in 1..=MIN_RACES as u8 {
            health.open_race(
                &leg(&[entry, black_hole, exit]),
                surb_id(seed),
                &leg(&[entry, healthy, exit]),
                ROUND_TRIP,
            );
        }
        backdate_open(&health, black_hole, ago(jitter(0) * 2)?);

        assert_eq!(vec![black_hole], health.refresh_verdicts().0);
        assert!(health.is_dead(&black_hole));
        assert_eq!(
            0,
            open_races_of(&health, &entry),
            "relayers on both legs are not to blame"
        );
        assert_eq!(
            vec![0.0, 1.0],
            health.factors(&[vec![healthy, black_hole], vec![entry, healthy, exit]], 4),
            "a dead relayer sinks every path it is on, in any position"
        );
        assert!(health.refresh_verdicts().0.is_empty(), "only new verdicts are reported");
        Ok(())
    }

    /// A relayer that is always somewhat late has races open for that long as a matter of course;
    /// only replies out longer than it usually takes count against it.
    #[test]
    fn replies_out_no_longer_than_usual_should_not_make_a_black_hole() -> anyhow::Result<()> {
        let health = RelayerHealth::default();
        let slow = key();
        settled_since(&health, slow, ago(Duration::from_secs(5))?, Duration::from_millis(500));
        let just_now = Instant::now()
            .checked_sub(Duration::from_millis(200))
            .context("monotonic clock younger than 200 ms")?;
        open_since(&health, slow, MIN_RACES as u8, just_now);
        assert!(
            health.refresh_verdicts().0.is_empty(),
            "out for less than its usual 500 ms"
        );

        let overdue = Instant::now()
            .checked_sub(Duration::from_secs(2))
            .context("monotonic clock younger than two seconds")?;
        open_since(&health, slow, MIN_RACES as u8, overdue);
        assert_eq!(vec![slow], health.refresh_verdicts().0);
        Ok(())
    }

    #[test]
    fn a_dead_verdict_from_races_should_lapse_with_the_surbs_it_was_built_on() -> anyhow::Result<()> {
        let health = RelayerHealth::default();
        let black_hole = key();
        let expired = Instant::now()
            .checked_sub(PENDING_SURB_TTL + Duration::from_secs(1))
            .context("monotonic clock younger than a SURB's lifetime")?;
        settled_since(&health, black_hole, expired - Duration::from_secs(1), jitter(0));
        open_since(&health, black_hole, MIN_RACES as u8, expired);

        assert!(health.refresh_verdicts().0.is_empty());
        assert!(
            !health.is_dead(&black_hole),
            "it can be tried again once its SURBs are gone"
        );
        Ok(())
    }

    #[test]
    fn a_relayer_we_lost_our_connection_to_should_be_dead_until_it_returns() {
        let health = RelayerHealth::default();
        let relayer = key();

        health.set_reachable(relayer, false);
        assert_eq!((vec![relayer], vec![]), health.refresh_verdicts());
        assert!(health.is_dead(&relayer));

        health.set_reachable(relayer, true);
        assert_eq!(
            (vec![], vec![relayer]),
            health.refresh_verdicts(),
            "recovery is reported too"
        );
        assert!(!health.is_dead(&relayer));
    }

    #[test]
    fn settling_a_race_should_close_it() {
        let health = RelayerHealth::default();
        let [early, late] = [(); 2].map(|_| key());
        health.open_race(&leg(&[late]), surb_id(1), &leg(&[early]), ROUND_TRIP);
        health.record_race(&leg(&[late]), surb_id(1), &leg(&[early]), ROUND_TRIP, jitter(0));

        assert_eq!(0, open_races_of(&health, &late));
        assert_eq!(1, margins_of(&health, &late).len());
    }

    /// All zero would leave nothing to draw, so the best candidate keeps full weight even when every
    /// candidate finishes late against some other relayer.
    #[test]
    fn the_best_candidate_should_keep_full_weight_when_all_finish_late() {
        let health = RelayerHealth::default();
        let [outsider, late, later] = [(); 3].map(|_| key());
        for i in 0..20 {
            health.record_race(
                &leg(&[late]),
                surb_id(0),
                &leg(&[outsider]),
                ROUND_TRIP,
                Duration::from_secs(1) + jitter(i),
            );
            health.record_race(
                &leg(&[later]),
                surb_id(0),
                &leg(&[outsider]),
                ROUND_TRIP,
                Duration::from_secs(3) + jitter(i),
            );
        }

        let factors = health.factors(&[vec![late], vec![later]], 4);
        assert_eq!(1.0, factors[0]);
        assert!(factors[1] < 1.0, "the later one still weighs less, got {}", factors[1]);
    }

    /// Whatever two SURBs of one packet spent in the replier's store, they spent together, so only
    /// the gap between their replies counts against the later one.
    #[test]
    fn a_race_should_be_settled_by_the_gap_between_the_two_replies() -> anyhow::Result<()> {
        let (graph, _me, _peers) = graph_with(0);
        let recorder = recorder(graph);
        let [early, late, destination] = [(); 3].map(|_| key());
        let minted_at = Instant::now();
        let pending_via = |relayer, partner| PendingSurb {
            paths: ForwardAndReturnPath {
                forward: [0; PATH_ID_SLOTS],
                reply: [0; PATH_ID_SLOTS],
            },
            destination,
            return_relayers: leg(&[relayer]),
            minted_at,
            partner: Some(partner),
            second: relayer == late,
            partner_replied: None,
        };
        recorder
            .registry
            .pending
            .insert(surb_id(1), pending_via(early, surb_id(2)));
        recorder
            .registry
            .pending
            .insert(surb_id(2), pending_via(late, surb_id(1)));
        let health = &recorder.registry.relayer_health;

        recorder.on_replied(&surb_id(1));
        assert!(
            margins_of(health, &early).is_empty(),
            "no race settled until both are home"
        );
        assert_eq!(
            1,
            open_races_of(health, &late),
            "the reply still out is a race already being lost"
        );
        assert_eq!(
            HashMap::from([(destination, (1, 1.0))]),
            recorder.registry.unusable_through(&[late]),
            "a dead relayer's outstanding SURBs count against the destination they went to"
        );
        let mut waiting = recorder
            .registry
            .pending
            .get(&surb_id(2))
            .context("the late SURB is out")?;
        let (relayers, home) = waiting.partner_replied.context("its partner is home")?;
        waiting.partner_replied = Some((relayers, home.checked_sub(Duration::from_millis(20)).context("clock")?));
        recorder.registry.pending.insert(surb_id(2), waiting);
        recorder.on_replied(&surb_id(2));
        assert_eq!(0, open_races_of(health, &late));

        let lost_by = margins_of(health, &late);
        assert_eq!(1, lost_by.len());
        assert!(
            lost_by[0] >= 0.02,
            "the later reply loses by the gap, got {}",
            lost_by[0]
        );
        assert_eq!(vec![-lost_by[0]], margins_of(health, &early));
        Ok(())
    }

    #[test]
    fn unusable_surbs_should_be_counted_and_relative_to_what_is_outstanding() {
        let registry = SurbRoundTripRegistry::default();
        let [dead, alive, destination] = [(); 3].map(|_| key());
        for (seed, relayer) in [(1, dead), (2, alive), (3, alive), (4, alive)] {
            registry.pending.insert(
                surb_id(seed),
                PendingSurb {
                    paths: ForwardAndReturnPath {
                        forward: [0; PATH_ID_SLOTS],
                        reply: [0; PATH_ID_SLOTS],
                    },
                    destination,
                    return_relayers: leg(&[relayer]),
                    minted_at: Instant::now(),
                    partner: None,
                    second: false,
                    partner_replied: None,
                },
            );
        }

        assert_eq!(
            HashMap::from([(destination, (1, 0.25))]),
            registry.unusable_through(&[dead])
        );
        assert!(registry.unusable_through(&[]).is_empty());
    }

    /// The first SURB of a pair can be buried under newer ones in the replier's store, so neither its
    /// reply being out nor its late arrival says anything about its return path.
    #[test]
    fn a_first_surb_still_out_should_not_count_against_its_relayer() {
        let (graph, _me, _peers) = graph_with(0);
        let recorder = recorder(graph);
        let [buried, home, destination] = [(); 3].map(|_| key());
        let pending_via = |relayer, partner, second| PendingSurb {
            paths: ForwardAndReturnPath {
                forward: [0; PATH_ID_SLOTS],
                reply: [0; PATH_ID_SLOTS],
            },
            destination,
            return_relayers: leg(&[relayer]),
            minted_at: Instant::now(),
            partner: Some(partner),
            second,
            partner_replied: None,
        };
        recorder
            .registry
            .pending
            .insert(surb_id(1), pending_via(buried, surb_id(2), false));
        recorder
            .registry
            .pending
            .insert(surb_id(2), pending_via(home, surb_id(1), true));

        recorder.on_replied(&surb_id(2));
        assert_eq!(0, open_races_of(&recorder.registry.relayer_health, &buried));

        recorder.on_replied(&surb_id(1));
        assert!(
            recorder
                .registry
                .relayer_health
                .relayers
                .get(&buried)
                .is_none_or(|history| history.races.is_empty()),
            "a buried reply coming home late must not settle a race against its relayer"
        );
    }

    #[test]
    fn replies_out_without_settled_races_should_not_make_a_black_hole() {
        let health = RelayerHealth::default();
        let relayer = key();
        open_since(&health, relayer, 5, Instant::now() - Duration::from_secs(5));

        assert!(health.refresh_verdicts().0.is_empty());
        assert!(!health.is_dead(&relayer));
    }

    #[test]
    fn surbs_should_race_their_neighbour_only_over_another_return_path() {
        let [a, b] = [(); 2].map(|_| key());
        let ids = [surb_id(1), surb_id(2), surb_id(3)];

        assert_eq!(
            vec![Some(ids[1]), Some(ids[0]), None],
            race_partners(&ids, &[leg(&[a]), leg(&[b]), leg(&[a])]),
            "the odd one out has no partner"
        );
        assert_eq!(
            vec![None, None],
            race_partners(&ids[..2], &[leg(&[a, b]), leg(&[a, b])]),
            "a path cannot race itself"
        );
        assert_eq!(
            vec![None, None],
            race_partners(&ids[..2], &[leg(&[]), leg(&[b])]),
            "a direct reply has no relayer to blame"
        );
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

        let minting = SurbTelemetryCodec::new((), me, path_slots_of(graph.clone()), registry.clone());
        let observing = SurbTelemetryCodec::new((), me, path_slots_of(graph), registry.clone());

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

    #[test]
    fn a_reply_on_a_surb_we_never_minted_should_be_ignored() {
        let (graph, ..) = graph_with(1);
        let recorder = recorder(graph);

        recorder.on_replied(&surb_id(9));

        assert!(recorder.registry.drain().is_empty());
    }
}
