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
//!
//! # Mitigations driven from here
//!
//! Each names its trigger, its effect, its setting and what it must not work against.
//!
//! - **Relayer health.** Trigger: a relayer losing reply races (two SURBs of one packet, different return paths) by
//!   more than a round trip. Effect: its health in the graph scales every edge into it, so forward and return paths
//!   through it are drawn less. Settings: `HoprProtocolConfig::latency_race_memory`,
//!   `HoprProtocolConfig::relayer_health_weight` (`0` off). Must not fight: the return-path temper, which flattens it
//!   along with path quality.
//! - **Dead verdict.** Trigger: at least `MIN_RACES` replies trailing their partners by more than any race the relayer
//!   settled, none home since. Effect: health `0`, the walk prunes it. Lapses once no reply through it is out (about
//!   `PENDING_SURB_TTL`); its lost races keep its health near zero. Must not fight: common-mode outages, where no
//!   partner gets home and nothing is judged.
//! - **Invalid SURBs.** Trigger: a dead verdict. Effect: each session is told how many SURBs in its counterparty's
//!   store run through that relayer; the balancer subtracts them from its level and sends replacements. Lasts until a
//!   reply through the relayer comes home or the session ends, not just the verdict: the counterparty still holds them.
//!   Must not fight: flow control, which keeps the raw level so the replacements are not throttled.
//! - **Fresh SURB generation.** Trigger: a dead verdict reaching a session for the first time, at most once per
//!   (session, relayer) per `REBUMP_COOLDOWN`. Effect: the session's SURB generation advances, so the counterparty
//!   drops its whole stock on its next push, the dead SURBs it cannot recognise along with the rest, and the balancer
//!   refills from empty, paced. Must not fight: the cooldown, since each advance costs the counterparty everything it
//!   holds, and a PIX cycle can absorb only so many such losses.
//! - **Lost neighbour.** Trigger: our own connection drops. Effect: re-plan only; the graph's connection status already
//!   rejects that edge, so its SURBs are not invalidated.

use std::{
    collections::{HashMap, VecDeque},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
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
use hopr_protocol_hopr::{
    IncomingPacket, IncomingPacketError, OutgoingPacket, PacketDecoder, PacketEncoder, SurbPopOrder,
};

/// Slots in a [`PathId`], and therefore the longest leg that can be reported.
pub(crate) const PATH_ID_SLOTS: usize = 5;

/// How long a minted SURB stays eligible to be observed coming back.
///
/// A SURB the replier never uses would otherwise sit in the pending map forever, so the expectation
/// it raised is retired after this long. Generous relative to a round-trip, because expiring a SURB
/// that is merely slow would count a success as a loss.
const PENDING_SURB_TTL: Duration = Duration::from_secs(60);

/// How long after a session's SURB generation was advanced for a relayer's dead verdict the same
/// session is not advanced again for the same relayer: twice [`PENDING_SURB_TTL`].
///
/// A verdict lapses about a pending TTL after the last SURB through the relayer was minted, and the
/// planner's exploration then draws the relayer again, so a relayer that is still dead is judged dead
/// anew. Each advance costs the counterparty the whole stock it holds, so one verdict must not cost it
/// twice: this spans the lapse and the re-verdict that follows it.
const REBUMP_COOLDOWN: Duration = Duration::from_secs(2 * 60);
const _: () = assert!(REBUMP_COOLDOWN.as_secs() == 2 * PENDING_SURB_TTL.as_secs());

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
    /// Per-relayer health from reply races, published into the graph.
    relayer_health: Arc<RelayerHealth>,
    /// Every outstanding SURB, shared by both codec halves.
    pending: PendingLegs,
    /// Per session, SURBs written off because their return path runs through a relayer judged dead,
    /// by that relayer. Kept past the verdict, which only times out: until a reply through the relayer
    /// comes home, or the session is gone.
    written_off: Arc<DashMap<HoprPseudonym, HashMap<OffchainPublicKey, u64>>>,
    /// When a dead verdict last made [`write_off_dead`](Self::write_off_dead) report a session for a
    /// relayer, so that the same pair is not reported again within [`REBUMP_COOLDOWN`].
    bumped: Arc<DashMap<(HoprPseudonym, OffchainPublicKey), Instant>>,
    /// The order the counterparty is assumed to spend its SURBs in, which decides what a race between
    /// two SURBs of a packet says about their return paths; see [`judges`](Self::judges).
    pop_order: SurbPopOrder,
}

/// Memory bound only: at the 60 s TTL it is reached above ~1.7k SURBs minted per second.
const MAX_PENDING_SURBS: u64 = 100_000;

impl Default for SurbRoundTripRegistry {
    fn default() -> Self {
        Self::new(SurbPopOrder::default())
    }
}

impl SurbRoundTripRegistry {
    /// A registry for a counterparty that spends its SURBs in `pop_order`.
    ///
    /// That is our own configuration, assumed to be the network's: the order is the replier's, and
    /// nothing tells us what the replier runs.
    pub fn new(pop_order: SurbPopOrder) -> Self {
        Self {
            inner: Default::default(),
            relayer_health: Default::default(),
            pending: pending_legs(MAX_PENDING_SURBS),
            written_off: Default::default(),
            bumped: Default::default(),
            pop_order,
        }
    }

    /// Whether the reply of the SURB at `pair_index` in its packet says anything about its return path
    /// when it races its partner's: only the SURB the replier spends first does, see
    /// [`first_spent_index`].
    ///
    /// The other one is spent after it, so its reply trails the first's by however long the replier
    /// took to get to it, a pause in its reply stream included, and that says nothing about a relayer.
    /// A race therefore opens only in the abnormal order, the later-spent SURB home while the
    /// first-spent one is still out, and what it measures is lateness down to the return path. Judging
    /// both would settle every pair in the normal order, and the largest inter-reply gap the replier
    /// ever had would become the bar a reply must stay out longer than to count as overdue.
    fn judges(&self, pair_index: usize) -> bool {
        pair_index == first_spent_index(self.pop_order)
    }

    fn counters(&self, paths: ForwardAndReturnPath) -> Arc<SurbRoundTripCounters> {
        self.inner.entry(paths).or_default().value().clone()
    }

    /// Records that `count` SURBs were minted over these legs and are expected back.
    pub fn record_expected(&self, paths: ForwardAndReturnPath, count: u64) {
        self.counters(paths).record_expected(count);
    }

    /// The per-relayer health, judged from reply races and published into the graph.
    pub fn relayer_health(&self) -> Arc<RelayerHealth> {
        self.relayer_health.clone()
    }

    /// Moves outstanding SURBs whose return path runs through a relayer judged dead into the
    /// per-session write-off.
    ///
    /// Only the SURBs through the dead relayer are written off, and only they are counted: the
    /// counterparty's other SURBs are as good as ever until the caller supersedes the whole stock.
    ///
    /// Returns the sessions a dead verdict reached for the first time, i.e. the ones for which a
    /// relayer has SURBs outstanding that were not written off before. The counterparty cannot tell
    /// which of the SURBs it holds run through the dead relayer, since it sees only their first hop,
    /// so the caller advances each such session's SURB generation and the counterparty drops its
    /// whole stock on the next push. A (session, relayer) pair is reported at most once per
    /// [`REBUMP_COOLDOWN`]: a verdict that lapses and is reached again inside it is written off as
    /// usual but reports nothing.
    pub fn write_off_dead(&self) -> Vec<HoprPseudonym> {
        let health = &self.relayer_health;
        self.bumped.retain(|_, at| at.elapsed() < REBUMP_COOLDOWN);
        if health.dead.is_empty() {
            return Vec::new();
        }
        let through_dead: Vec<(HoprSurbId, HoprPseudonym, OffchainPublicKey)> = self
            .pending
            .iter()
            .filter_map(|(id, pending)| {
                relayers_of(&pending.return_relayers)
                    .find(|relayer| health.dead.contains(relayer))
                    .map(|dead| (*id, pending.pseudonym, dead))
            })
            .collect();
        let mut reached: Vec<HoprPseudonym> = Vec::new();
        for (id, pseudonym, dead) in through_dead {
            self.pending.invalidate(&id);
            let mut session = self.written_off.entry(pseudonym).or_default();
            let first_for_pair = !session.contains_key(&dead);
            *session.entry(dead).or_default() += 1;
            drop(session);
            if !first_for_pair {
                continue;
            }
            if self.bumped.contains_key(&(pseudonym, dead)) {
                tracing::debug!(
                    %pseudonym,
                    relayer = %dead,
                    "dead verdict re-reached within the re-bump cooldown, written off without a new SURB generation"
                );
            } else {
                self.bumped.insert((pseudonym, dead), Instant::now());
                if !reached.contains(&pseudonym) {
                    reached.push(pseudonym);
                }
            }
        }
        reached
    }

    /// Per session, how many SURBs held by the counterparty cannot bring a reply home, and their share
    /// of all it was handed that are still accounted for.
    pub fn invalid(&self) -> HashMap<HoprPseudonym, (u64, f64)> {
        if self.written_off.is_empty() {
            return HashMap::new();
        }
        let mut outstanding: HashMap<HoprPseudonym, u64> = HashMap::new();
        for (_, pending) in self.pending.iter() {
            *outstanding.entry(pending.pseudonym).or_default() += 1;
        }
        self.written_off
            .iter()
            .map(|session| {
                let count: u64 = session.values().sum();
                let total = count + outstanding.get(session.key()).copied().unwrap_or_default();
                (*session.key(), (count, count as f64 / total.max(1) as f64))
            })
            .collect()
    }

    /// A reply came home through these relayers, so the SURBs written off through them work again.
    fn end_write_off(&self, relayers: &ReturnRelayers) {
        if self.written_off.is_empty() {
            return;
        }
        for mut session in self.written_off.iter_mut() {
            for relayer in relayers_of(relayers) {
                session.remove(&relayer);
            }
        }
        self.written_off.retain(|_, session| !session.is_empty());
    }

    /// Drops the write-off and the re-bump cooldown of a session that no longer exists.
    pub fn forget_session(&self, pseudonym: &HoprPseudonym) {
        self.written_off.remove(pseudonym);
        self.bumped.retain(|(session, _), _| session != pseudonym);
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
    /// The session it was minted for.
    pseudonym: HoprPseudonym,
    /// Relayers the reply passes, any of which a late or missing reply implicates.
    return_relayers: ReturnRelayers,
    /// When this SURB and its partner were minted; both round-trips run from here.
    minted_at: Instant,
    /// The SURB minted in the same packet over another return path, which this one races.
    partner: Option<HoprSurbId>,
    /// Its position in the pair, `0` or `1`; see [`SurbRoundTripRegistry::judges`].
    pair_index: usize,
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
    /// Neighbours we lost our own connection to, to re-plan when one comes or goes.
    lost_neighbours: DashSet<OffchainPublicKey>,
    neighbours_changed: AtomicBool,
    dead: DashSet<OffchainPublicKey>,
    /// Health last handed to the graph, below `1.0` only.
    published: DashMap<OffchainPublicKey, f64>,
}

/// The position, in a pair of SURBs minted together, of the one the replier spends first: the second
/// under LIFO, which spends the newest first, the first under FIFO.
///
/// Only that SURB's lateness is down to its return path. The other is spent after it, so it may wait
/// in the replier's store behind it (LIFO) or be spent right after it (FIFO), and in both cases it
/// trails the first-spent one by the replier's pace rather than by a relayer's fault. A race opens
/// only when the later-spent SURB is home while the first-spent one is still out.
///
/// That holds for the pop order configured here, which is assumed to be the replier's. Share-first
/// tiers can separate a pair even so: a SURB carrying a PIX share is spent before share-less ones,
/// whatever their ages. See [`SurbRoundTripRegistry::judges`].
const fn first_spent_index(pop_order: SurbPopOrder) -> usize {
    match pop_order {
        SurbPopOrder::Lifo => 1,
        SurbPopOrder::Fifo => 0,
    }
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

impl Race {
    /// Its reply never came home within the time a SURB is remembered.
    fn lost(&self) -> bool {
        self.margin >= PENDING_SURB_TTL.as_secs_f64()
    }
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

    /// A black hole: at least [`MIN_RACES`] replies through it are out for longer than any race it
    /// finished took, and none has come home since the oldest was due.
    fn is_black_hole(&self) -> bool {
        let finished: Vec<&Race> = self.races.iter().filter(|race| !race.lost()).collect();
        if finished.len() < MIN_RACES {
            return false;
        }
        // No round-trip floor: a round trip includes the time the SURB waited in the replier's store,
        // seconds to a minute after a drain, while both SURBs of a pair wait alike, so margins don't.
        let usual = finished.iter().map(|race| race.margin.abs()).fold(0.0, f64::max);
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

/// A relayer's standing in its races: median margin behind its partners, that median's
/// uncertainty, and the fastest round-trip among them.
#[derive(Debug, Clone, Copy)]
struct Judgement {
    margin: f64,
    uncertainty: f64,
    fastest: f64,
}

impl Judgement {
    /// Health from the relayer's lateness, `unit` being the fastest round-trip any relayer made:
    /// lagging by less than that costs nothing.
    fn health(&self, unit: f64) -> f64 {
        unit / unit.max(self.margin - self.uncertainty)
    }

    /// The healths its races cannot tell apart: lateness within its uncertainty, or within a
    /// round-trip, which healthy relayers differ by anyway.
    fn indistinct(&self, unit: f64) -> std::ops::RangeInclusive<f64> {
        let slack = self.uncertainty.max(unit);
        unit / unit.max(self.margin + slack)..=unit / unit.max(self.margin - slack)
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

    /// Our own connection to a neighbour came or went.
    pub fn set_neighbour_connected(&self, neighbour: OffchainPublicKey, connected: bool) {
        let changed = if connected {
            self.lost_neighbours.remove(&neighbour).is_some()
        } else {
            self.lost_neighbours.insert(neighbour)
        };
        if changed {
            self.neighbours_changed.store(true, Ordering::Relaxed);
        }
    }

    /// Whether a neighbour came or went since the last call.
    pub fn take_neighbour_change(&self) -> bool {
        self.neighbours_changed.swap(false, Ordering::Relaxed)
    }

    #[cfg(test)]
    fn is_dead(&self, relayer: &OffchainPublicKey) -> bool {
        self.dead.contains(relayer)
    }

    /// Re-judges every relayer once per flush; returns `(relayer, previous, current)` health changes for
    /// the graph, dead being `0.0`. A race open longer than a SURB is remembered counts as lost by that
    /// long. `memory`: margins a race is remembered for, `0` for dead or healthy.
    pub fn refresh(&self, memory: u32) -> Vec<(OffchainPublicKey, f64, f64)> {
        let mut black_holes = std::collections::HashSet::new();
        for mut entry in self.relayers.iter_mut() {
            // A race open past the time a SURB is remembered is lost, by that long: the relayer keeps
            // a health that low until it proves otherwise, rather than being forgiven by expiry.
            let mut lost = Vec::new();
            entry.open.retain(|_, race| {
                let open = race.at.elapsed() <= PENDING_SURB_TTL;
                if !open {
                    lost.push(Race {
                        at: Instant::now(),
                        margin: PENDING_SURB_TTL.as_secs_f64(),
                        ..*race
                    });
                }
                open
            });
            entry.races.extend(lost);
            if entry.is_black_hole() {
                black_holes.insert(*entry.key());
            }
        }
        self.dead.retain(|relayer| black_holes.contains(relayer));
        for relayer in black_holes {
            self.dead.insert(relayer);
        }

        let judged: HashMap<OffchainPublicKey, Judgement> = self
            .relayers
            .iter()
            .map(|entry| *entry.key())
            .collect::<Vec<_>>()
            .into_iter()
            .filter_map(|relayer| self.judge(&relayer, memory).map(|judgement| (relayer, judgement)))
            .collect();
        // A relayer gone from the paths races no more; its history only grows the map.
        self.relayers.retain(|_, history| {
            !history.open.is_empty()
                || history
                    .races
                    .back()
                    .is_some_and(|race| race.at.elapsed() <= history.memory.max(PENDING_SURB_TTL))
        });
        let unit = judged
            .values()
            .map(|judgement| judgement.fastest)
            .min_by(f64::total_cmp);

        let relayers: std::collections::HashSet<OffchainPublicKey> = judged
            .keys()
            .copied()
            .chain(self.published.iter().map(|entry| *entry.key()))
            .chain(self.dead.iter().map(|relayer| *relayer))
            .collect();

        let mut changed = Vec::new();
        for relayer in relayers {
            let previous = self.published.get(&relayer).map_or(1.0, |health| *health);
            let current = match (judged.get(&relayer), unit) {
                _ if self.dead.contains(&relayer) => 0.0,
                (Some(judgement), Some(unit)) if memory > 0 => {
                    if previous > 0.0 && judgement.indistinct(unit).contains(&previous) {
                        continue;
                    }
                    judgement.health(unit)
                }
                _ => 1.0,
            };
            if current == previous {
                continue;
            }
            if current < 1.0 {
                self.published.insert(relayer, current);
            } else {
                self.published.remove(&relayer);
            }
            changed.push((relayer, previous, current));
        }
        changed
    }

    /// Margin, its uncertainty and fastest round-trip. Races are remembered for `memory` times
    /// margin plus spread, so the window scales with the relayer.
    fn judge(&self, relayer: &OffchainPublicKey, memory: u32) -> Option<Judgement> {
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
        Some(Judgement {
            margin,
            uncertainty: spread / (races.len() as f64).sqrt(),
            fastest,
        })
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

    if legs == 0 {
        // An idle node flushes every interval; an INFO line for nothing would bury the ones that count.
        tracing::trace!("surb round-trip flush tick: nothing to flush");
        return;
    }

    {
        // One aggregate line per interval rather than one per pair of legs.
        tracing::info!(
            legs,
            expected = total_expected,
            observed = total_observed,
            "surb round-trip flush tick"
        );
    }
}

/// Re-judges relayer health, records what changed in the graph, and re-plans when it or a neighbour
/// changed, before writing off the SURBs of relayers judged dead. Recording precedes the re-plan, so
/// the re-plan already avoids them; the write-off follows it, so the refill goes over new paths.
///
/// Returns the sessions whose SURB generation the caller should advance, as reported by
/// [`SurbRoundTripRegistry::write_off_dead`].
pub async fn refresh_relayer_health<G, F>(
    registry: &SurbRoundTripRegistry,
    graph: &G,
    memory: u32,
    weight: f64,
    replan: impl FnOnce() -> F,
) -> Vec<HoprPseudonym>
where
    G: hopr_api::graph::NetworkGraphUpdate,
    F: std::future::Future<Output = ()>,
{
    let health = &registry.relayer_health;
    let changed = health.refresh(memory);
    for (relayer, previous, current) in &changed {
        match (*previous == 0.0, *current == 0.0) {
            (false, true) => tracing::info!(%relayer, "relayer judged dead"),
            (true, false) => tracing::info!(%relayer, health = current, "dead verdict lapsed"),
            _ => tracing::debug!(%relayer, previous, current, "relayer health changed"),
        }
    }
    // Every judged relayer on every flush, not only the changes: the graph keeps health on edges, and
    // an edge added since the last change would otherwise carry none.
    let healths: Vec<(OffchainPublicKey, f64)> = health
        .published
        .iter()
        .map(|entry| (*entry.key(), *entry.value()))
        .chain(
            changed
                .iter()
                .filter(|(.., current)| *current >= 1.0)
                .map(|(relayer, _, current)| (*relayer, *current)),
        )
        .map(|(relayer, health)| (relayer, health.powf(weight)))
        .collect();
    if !healths.is_empty() {
        graph.record_nodes_health(&healths);
    }
    if health.take_neighbour_change() || !changed.is_empty() {
        replan().await;
    }
    registry.write_off_dead()
}

/// The invalid SURBs to hand each session this tick: every current mark, re-applied so a session
/// opened since picks it up, and `(0, 0.0)` for sessions whose mark has gone.
pub fn invalid_updates(
    previous: &HashMap<HoprPseudonym, (u64, f64)>,
    current: &HashMap<HoprPseudonym, (u64, f64)>,
) -> Vec<(HoprPseudonym, (u64, f64))> {
    current
        .iter()
        .map(|(session, invalid)| (*session, *invalid))
        .chain(
            previous
                .keys()
                .filter(|session| !current.contains_key(session))
                .map(|session| (*session, (0, 0.0))),
        )
        .collect()
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
            pseudonym,
            forward_path,
            return_paths,
        } = routing
        else {
            return;
        };

        let forward: Vec<_> = forward_path.transport_path().iter().copied().collect();
        if forward.is_empty() {
            return;
        }

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
                    pseudonym: *pseudonym,
                    return_relayers: return_relayers[i],
                    minted_at,
                    partner: partners[i],
                    pair_index: i % 2,
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
        self.registry.end_write_off(&pending.return_relayers);

        let health = &self.registry.relayer_health;
        // Only a SURB the replier's pop order makes attributable settles: the other may have waited in
        // the replier's store.
        if let Some((partner_relayers, partner_home)) = pending.partner_replied {
            if !self.registry.judges(pending.pair_index) {
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
            if let moka::ops::compute::CompResult::ReplacedWith(waiting) = updated {
                if !self.registry.judges(waiting.value().pair_index) {
                    return;
                }
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
        generation: Option<u8>,
    ) -> Result<OutgoingPacket, <Self as PacketEncoder>::Error> {
        let packet = self.inner.encode_packet(data, routing.clone(), signals, generation)?;
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
        graph::{EdgeObservableRead, NetworkGraphWrite},
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
        recorder_with(graph, SurbPopOrder::default())
    }

    fn recorder_with(graph: ChannelGraph, pop_order: SurbPopOrder) -> SurbTelemetryCodec<()> {
        let me = *graph.identity();
        SurbTelemetryCodec::new((), me, path_slots_of(graph), SurbRoundTripRegistry::new(pop_order))
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

    /// Each relayer's health after one flush, `1.0` for one never reported below it.
    fn health_after_flush(health: &RelayerHealth, relayers: &[OffchainPublicKey]) -> Vec<f64> {
        health.refresh(4);
        relayers
            .iter()
            .map(|relayer| health.published.get(relayer).map_or(1.0, |health| *health))
            .collect()
    }

    fn judged_dead(changes: &[(OffchainPublicKey, f64, f64)]) -> Vec<OffchainPublicKey> {
        changes
            .iter()
            .filter(|(_, previous, current)| *current == 0.0 && *previous > 0.0)
            .map(|(relayer, ..)| *relayer)
            .collect()
    }

    fn pending_to(relayers: &[OffchainPublicKey], partner: Option<HoprSurbId>, pair_index: usize) -> PendingSurb {
        PendingSurb {
            paths: ForwardAndReturnPath {
                forward: [0; PATH_ID_SLOTS],
                reply: [0; PATH_ID_SLOTS],
            },
            pseudonym: *SESSION,
            return_relayers: leg(relayers),
            minted_at: Instant::now(),
            partner,
            pair_index,
            partner_replied: None,
        }
    }

    static SESSION: std::sync::LazyLock<HoprPseudonym> = std::sync::LazyLock::new(HoprPseudonym::random);

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

        let factors = health_after_flush(&health, &[fast, also_fast, stalled]);
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

        let factors = health_after_flush(&health, &[stalled, entry, healthy, exit, other_entry, other_exit]);
        assert!(
            factors[0] < 0.1,
            "the stalled relayer must lose its weight, got {}",
            factors[0]
        );
        assert_eq!(&[1.0; 5], &factors[1..], "its neighbours on that path are not to blame");
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

        assert_eq!(vec![1.0, 1.0], health_after_flush(&health, &[near, far]));
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

        assert_eq!(vec![1.0, 1.0], health_after_flush(&health, &[fast, young]));
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
            health_after_flush(&health, &[fast, recovered])[1] < 0.1,
            "precondition: judged stalled"
        );

        for i in 0..20 {
            let (later, earlier) = if i % 2 == 0 {
                (fast, recovered)
            } else {
                (recovered, fast)
            };
            health.record_race(&leg(&[later]), surb_id(0), &leg(&[earlier]), ROUND_TRIP, jitter(i));
        }
        assert_eq!(vec![1.0, 1.0], health_after_flush(&health, &[fast, recovered]));
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

        let factors = health_after_flush(&health, &[fast, also_fast, stalled]);
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
        backdate_open(&health, black_hole, ago(ROUND_TRIP * 2)?);

        assert_eq!(vec![black_hole], judged_dead(&health.refresh(4)));
        assert!(health.is_dead(&black_hole));
        assert_eq!(
            0,
            open_races_of(&health, &entry),
            "relayers on both legs are not to blame"
        );
        assert!(
            judged_dead(&health.refresh(4)).is_empty(),
            "only new verdicts are reported"
        );
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
            judged_dead(&health.refresh(4)).is_empty(),
            "out for less than its usual 500 ms"
        );

        let overdue = Instant::now()
            .checked_sub(Duration::from_secs(2))
            .context("monotonic clock younger than two seconds")?;
        open_since(&health, slow, MIN_RACES as u8, overdue);
        assert_eq!(vec![slow], judged_dead(&health.refresh(4)));
        Ok(())
    }

    /// Expiry alone proves nothing: replies that never came home count as races lost by the whole
    /// time a SURB is remembered, so the relayer comes back at a sliver of health, not in full.
    #[test]
    fn a_dead_relayer_should_not_be_revived_to_full_health_by_expiry() -> anyhow::Result<()> {
        let health = RelayerHealth::default();
        let [black_hole, healthy] = [(); 2].map(|_| key());
        let expired = Instant::now()
            .checked_sub(PENDING_SURB_TTL + Duration::from_secs(1))
            .context("monotonic clock younger than a SURB's lifetime")?;
        settled_since(&health, healthy, Instant::now(), jitter(0));
        settled_since(&health, black_hole, expired - Duration::from_secs(1), jitter(0));
        open_since(&health, black_hole, 2 * MIN_RACES as u8, expired);

        let [revived, _] = health_after_flush(&health, &[black_hole, healthy])[..] else {
            anyhow::bail!("two relayers asked for");
        };
        assert!(!health.is_dead(&black_hole), "no reply is out any more");
        assert!(revived < 0.01, "revived to a sliver only, got {revived}");
        Ok(())
    }

    /// A lost neighbour only re-plans the paths; its SURBs are not written off, as they work again
    /// once it is back, and during an outage of every path no replacement could arrive anyway.
    #[test]
    fn a_lost_neighbour_should_not_write_off_its_surbs() {
        let registry = SurbRoundTripRegistry::default();
        let [neighbour, middle] = [(); 2].map(|_| key());
        registry
            .pending
            .insert(surb_id(1), pending_to(&[middle, neighbour], None, 0));
        registry
            .pending
            .insert(surb_id(2), pending_to(&[neighbour, middle], None, 0));
        let health = &registry.relayer_health;

        health.set_neighbour_connected(neighbour, false);
        assert!(registry.invalid().is_empty());
        assert!(!health.is_dead(&neighbour), "no verdict on the relayer itself");
        assert!(health.take_neighbour_change());
        assert!(!health.take_neighbour_change(), "a change is reported once");

        health.set_neighbour_connected(neighbour, true);
        assert!(registry.invalid().is_empty());
        assert!(health.take_neighbour_change());
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

    /// Health is measured against the fastest round-trip any relayer made, so relayers late
    /// against a third keep their order.
    #[test]
    fn relayers_finishing_late_should_be_ordered_by_how_late() {
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

        let factors = health_after_flush(&health, &[late, later]);
        assert!(1.0 > factors[0] && factors[0] > factors[1], "{factors:?}");
    }

    /// A health that moved within the uncertainty of the relayer's own races is noise; reporting it
    /// would re-plan every path on every flush.
    #[test]
    fn a_health_within_its_own_uncertainty_should_not_be_reported_again() {
        let health = RelayerHealth::default();
        let [fast, stalled] = [(); 2].map(|_| key());
        let mut race = |i| {
            health.record_race(
                &leg(&[stalled]),
                surb_id(0),
                &leg(&[fast]),
                ROUND_TRIP,
                Duration::from_secs(3) + jitter(i),
            )
        };
        (0..20).for_each(&mut race);
        assert_eq!(1, health.refresh(4).len(), "precondition: reported once stalled");

        (20..24).for_each(&mut race);
        assert!(health.refresh(4).is_empty());
    }

    /// Whatever two SURBs of one packet spent in the replier's store, they spent together, so only
    /// the gap between their replies counts against the later one. The SURB that loses is the one the
    /// replier spent first: it is home second, which is the abnormal order a race opens in.
    #[rstest::rstest]
    #[case::lifo(SurbPopOrder::Lifo)]
    #[case::fifo(SurbPopOrder::Fifo)]
    fn a_race_should_be_settled_by_the_gap_between_the_two_replies(
        #[case] pop_order: SurbPopOrder,
    ) -> anyhow::Result<()> {
        let (graph, _me, _peers) = graph_with(0);
        let recorder = recorder_with(graph, pop_order);
        let [early, late] = [(); 2].map(|_| key());
        let minted_at = Instant::now();
        let first_spent = first_spent_index(pop_order);
        let pending_via = |relayer, partner| PendingSurb {
            minted_at,
            ..pending_to(
                &[relayer],
                Some(partner),
                if relayer == late { first_spent } else { 1 - first_spent },
            )
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
    fn invalid_surbs_should_be_counted_per_session_and_relative_to_what_is_outstanding() {
        let registry = SurbRoundTripRegistry::default();
        let [dead, alive] = [(); 2].map(|_| key());
        let other = HoprPseudonym::random();
        for (seed, relayer) in [(1, dead), (2, alive), (3, alive), (4, alive)] {
            registry.pending.insert(surb_id(seed), pending_to(&[relayer], None, 0));
        }
        registry.pending.insert(
            surb_id(5),
            PendingSurb {
                pseudonym: other,
                ..pending_to(&[dead], None, 0)
            },
        );
        registry.write_off_dead();
        assert!(registry.invalid().is_empty(), "nothing is dead yet");

        registry.relayer_health.dead.insert(dead);
        registry.write_off_dead();
        assert_eq!(
            HashMap::from([(*SESSION, (1, 0.25)), (other, (1, 1.0))]),
            registry.invalid()
        );
    }

    /// The counterparty holds dead SURBs long after we stop remembering them as pending, and a lapsed
    /// verdict is only a timeout: the write-off ends when a reply through the relayer comes home.
    #[test]
    fn written_off_surbs_should_stay_invalid_until_a_reply_proves_their_relayer() {
        let (graph, ..) = graph_with(1);
        let recorder = recorder(graph);
        let registry = &recorder.registry;
        let [dead, alive] = [(); 2].map(|_| key());
        for (seed, relayer) in [(1, dead), (2, alive)] {
            registry.pending.insert(surb_id(seed), pending_to(&[relayer], None, 0));
        }
        registry.relayer_health.dead.insert(dead);
        registry.write_off_dead();

        registry.pending.invalidate_all();
        registry.pending.run_pending_tasks();
        assert_eq!(
            HashMap::from([(*SESSION, (1, 1.0))]),
            registry.invalid(),
            "pending SURBs gone, the written-off one still counts"
        );

        registry.relayer_health.dead.remove(&dead);
        registry.write_off_dead();
        assert_eq!(
            HashMap::from([(*SESSION, (1, 1.0))]),
            registry.invalid(),
            "a lapsed verdict is no evidence of recovery"
        );

        registry.pending.insert(surb_id(4), pending_to(&[alive], None, 0));
        recorder.on_replied(&surb_id(4));
        assert!(
            !registry.invalid().is_empty(),
            "a reply over another relayer proves nothing"
        );

        registry.pending.insert(surb_id(5), pending_to(&[dead], None, 0));
        recorder.on_replied(&surb_id(5));
        assert!(
            registry.invalid().is_empty(),
            "a reply through it proves the relayer works again"
        );

        registry.relayer_health.dead.insert(dead);
        registry.pending.insert(surb_id(3), pending_to(&[dead], None, 0));
        registry.write_off_dead();
        registry.forget_session(&SESSION);
        assert!(registry.invalid().is_empty(), "a closed session is forgotten");
    }

    /// A verdict reaching a session is reported once, so that the caller advances its SURB generation
    /// once. Only the SURBs through the dead relayer are written off: the generation advance takes the
    /// counterparty's healthy ones along, but that is the counterparty's loss to report (and the
    /// balancer's to reset), not a verdict about those SURBs.
    #[test]
    fn a_dead_verdict_should_bump_a_session_once_and_write_off_only_its_surbs_through_the_relayer() {
        let registry = SurbRoundTripRegistry::default();
        let [dead, alive] = [(); 2].map(|_| key());
        registry.pending.insert(surb_id(1), pending_to(&[dead], None, 0));
        registry.pending.insert(surb_id(2), pending_to(&[alive], None, 0));
        registry.pending.insert(surb_id(3), pending_to(&[alive], None, 0));
        registry.relayer_health.dead.insert(dead);

        assert_eq!(
            vec![*SESSION],
            registry.write_off_dead(),
            "the verdict reaches the session once"
        );
        registry.pending.run_pending_tasks();
        assert_eq!(
            HashMap::from([(*SESSION, (1, 1.0 / 3.0))]),
            registry.invalid(),
            "one SURB of three runs through the dead relayer; the healthy two are not written off"
        );

        registry.pending.insert(surb_id(4), pending_to(&[dead], None, 0));
        assert!(
            registry.write_off_dead().is_empty(),
            "a standing verdict writes off what still routes through the relayer, but bumps nothing more"
        );
        assert_eq!(HashMap::from([(*SESSION, (2, 0.5))]), registry.invalid());

        registry.relayer_health.dead.remove(&dead);
        assert!(registry.write_off_dead().is_empty());
        assert!(registry.invalid().is_empty(), "the write-off lapses with the verdict");
    }

    /// Two relayers judged dead in the same flush make one bump for the session, not two.
    #[test]
    fn two_dead_relayers_reaching_a_session_together_should_bump_it_once() {
        let registry = SurbRoundTripRegistry::default();
        let [dead, also_dead] = [(); 2].map(|_| key());
        registry.pending.insert(surb_id(1), pending_to(&[dead], None, 0));
        registry.pending.insert(surb_id(2), pending_to(&[also_dead], None, 0));
        registry.relayer_health.dead.insert(dead);
        registry.relayer_health.dead.insert(also_dead);

        assert_eq!(vec![*SESSION], registry.write_off_dead());
        assert_eq!(HashMap::from([(*SESSION, (2, 1.0))]), registry.invalid());
    }

    /// A verdict lapses about a pending TTL after the last SURB through the relayer, and the planner's
    /// exploration then draws it again, so a relayer that is still dead is judged dead anew. Each bump
    /// costs the counterparty its whole stock, so the second verdict is written off but not acted on
    /// until the cooldown has run out.
    #[test]
    fn a_verdict_refiring_within_the_cooldown_should_write_off_but_not_bump() -> anyhow::Result<()> {
        let registry = SurbRoundTripRegistry::default();
        let dead = key();
        registry.pending.insert(surb_id(1), pending_to(&[dead], None, 0));
        registry.relayer_health.dead.insert(dead);
        assert_eq!(vec![*SESSION], registry.write_off_dead(), "the first verdict bumps");

        registry.relayer_health.dead.remove(&dead);
        assert!(registry.write_off_dead().is_empty());
        assert!(registry.invalid().is_empty(), "the lapse drops the write-off");

        registry.relayer_health.dead.insert(dead);
        registry.pending.insert(surb_id(2), pending_to(&[dead], None, 0));
        assert!(
            registry.write_off_dead().is_empty(),
            "the same pair, re-reached inside the cooldown, is not bumped again"
        );
        assert_eq!(
            HashMap::from([(*SESSION, (1, 1.0))]),
            registry.invalid(),
            "but the SURB through the relayer is written off all the same"
        );

        // Lapse again, let the cooldown run out, and the next verdict bumps.
        registry.relayer_health.dead.remove(&dead);
        registry.write_off_dead();
        for mut entry in registry.bumped.iter_mut() {
            *entry.value_mut() = ago(REBUMP_COOLDOWN + Duration::from_secs(1))?;
        }
        registry.relayer_health.dead.insert(dead);
        registry.pending.insert(surb_id(3), pending_to(&[dead], None, 0));
        assert_eq!(
            vec![*SESSION],
            registry.write_off_dead(),
            "the cooldown has run out, and the verdict is acted on"
        );
        assert_eq!(1, registry.bumped.len(), "one fresh entry in place of the expired one");
        Ok(())
    }

    /// The cooldown is per pair: a second relayer judged dead for the same session is a new verdict.
    #[test]
    fn the_cooldown_should_hold_per_relayer() {
        let registry = SurbRoundTripRegistry::default();
        let [first, second] = [(); 2].map(|_| key());
        registry.pending.insert(surb_id(1), pending_to(&[first], None, 0));
        registry.relayer_health.dead.insert(first);
        assert_eq!(vec![*SESSION], registry.write_off_dead());

        registry.pending.insert(surb_id(2), pending_to(&[second], None, 0));
        registry.relayer_health.dead.insert(second);
        assert_eq!(
            vec![*SESSION],
            registry.write_off_dead(),
            "a different relayer is not covered by the first one's cooldown"
        );
    }

    #[test]
    fn forgetting_a_session_should_drop_its_cooldown() {
        let registry = SurbRoundTripRegistry::default();
        let dead = key();
        let other = HoprPseudonym::random();
        registry.pending.insert(surb_id(1), pending_to(&[dead], None, 0));
        registry.pending.insert(
            surb_id(2),
            PendingSurb {
                pseudonym: other,
                ..pending_to(&[dead], None, 0)
            },
        );
        registry.relayer_health.dead.insert(dead);
        let reached = registry.write_off_dead();
        assert_eq!(2, reached.len());
        assert_eq!(2, registry.bumped.len());

        registry.forget_session(&SESSION);
        assert_eq!(
            vec![(other, dead)],
            registry.bumped.iter().map(|entry| *entry.key()).collect::<Vec<_>>(),
            "only the closed session's cooldown goes"
        );

        // A session of the same pseudonym that starts afresh is not held back by the old one's cooldown.
        registry.relayer_health.dead.remove(&dead);
        registry.write_off_dead();
        registry.relayer_health.dead.insert(dead);
        registry.pending.insert(surb_id(3), pending_to(&[dead], None, 0));
        assert_eq!(vec![*SESSION], registry.write_off_dead());
    }

    #[test]
    fn invalid_updates_should_reapply_every_mark_and_clear_the_vanished() {
        let [kept, gone, new] = [(); 3].map(|_| HoprPseudonym::random());
        let previous = HashMap::from([(kept, (5, 0.5)), (gone, (2, 0.1))]);
        let current = HashMap::from([(kept, (5, 0.5)), (new, (1, 1.0))]);

        let mut updates = invalid_updates(&previous, &current);
        updates.sort_by_key(|(session, _)| *session);
        let mut expected = vec![(kept, (5, 0.5)), (new, (1, 1.0)), (gone, (0, 0.0))];
        expected.sort_by_key(|(session, _)| *session);
        assert_eq!(expected, updates, "unchanged marks too, for sessions opened since");
    }

    /// The graph learns the verdict before the re-plan, so the re-plan avoids the relayer; the
    /// write-off comes after, so the refill it triggers goes over the new paths.
    #[tokio::test]
    async fn a_dead_verdict_should_reach_the_graph_before_the_replan_and_the_write_off_after() -> anyhow::Result<()> {
        let (graph, me, peers) = graph_with(1);
        let [dead] = peers[..] else {
            anyhow::bail!("one peer asked for");
        };
        graph.add_edge(&me, &dead)?;
        let registry = SurbRoundTripRegistry::default();
        registry.pending.insert(surb_id(1), pending_to(&[dead], None, 0));
        let health = registry.relayer_health();
        settled_since(&health, dead, ago(Duration::from_secs(10))?, jitter(0));
        open_since(&health, dead, MIN_RACES as u8, ago(Duration::from_secs(5))?);

        let seen_at_replan = std::cell::Cell::new(None);
        let bumped = refresh_relayer_health(&registry, &graph, 4, 1.0, || async {
            let edge_health = graph.edge(&me, &dead).and_then(|edge| edge.health());
            seen_at_replan.set(Some((edge_health, registry.invalid().is_empty())));
        })
        .await;

        assert_eq!(Some((Some(0.0), true)), seen_at_replan.get());
        assert_eq!(HashMap::from([(*SESSION, (1, 1.0))]), registry.invalid());
        assert_eq!(
            vec![*SESSION],
            bumped,
            "the session the verdict reached is handed back for its SURB generation to be advanced"
        );
        Ok(())
    }

    /// The graph keeps health on edges, so an edge added after a verdict gets it on the next flush.
    #[tokio::test]
    async fn an_edge_added_after_a_verdict_should_carry_it_from_the_next_flush() -> anyhow::Result<()> {
        let (graph, me, peers) = graph_with(1);
        let [dead] = peers[..] else {
            anyhow::bail!("one peer asked for");
        };
        let registry = SurbRoundTripRegistry::default();
        let health = registry.relayer_health();
        settled_since(&health, dead, ago(Duration::from_secs(10))?, jitter(0));
        open_since(&health, dead, MIN_RACES as u8, ago(Duration::from_secs(5))?);
        refresh_relayer_health(&registry, &graph, 4, 1.0, || async {}).await;

        graph.add_edge(&me, &dead)?;
        refresh_relayer_health(&registry, &graph, 4, 1.0, || async {}).await;
        assert_eq!(Some(0.0), graph.edge(&me, &dead).and_then(|edge| edge.health()));
        Ok(())
    }

    /// Lagging two seconds behind 0.2 s round trips is judged health 0.1; dead is 0.
    #[rstest::rstest]
    #[case::as_judged(1.0, false, 0.1)]
    #[case::harsher(2.0, false, 0.01)]
    #[case::switched_off(0.0, false, 1.0)]
    #[case::dead(1.0, true, 0.0)]
    #[case::switched_off_for_the_dead(0.0, true, 1.0)]
    #[tokio::test]
    async fn the_health_weight_should_shape_what_the_graph_gets(
        #[case] weight: f64,
        #[case] dead: bool,
        #[case] recorded: f64,
    ) -> anyhow::Result<()> {
        let (graph, me, peers) = graph_with(1);
        let [relayer] = peers[..] else {
            anyhow::bail!("one peer asked for");
        };
        graph.add_edge(&me, &relayer)?;
        let registry = SurbRoundTripRegistry::default();
        let health = registry.relayer_health();
        if dead {
            settled_since(&health, relayer, ago(Duration::from_secs(10))?, jitter(0));
            open_since(&health, relayer, MIN_RACES as u8, ago(Duration::from_secs(5))?);
        } else {
            settled_since(&health, relayer, ago(Duration::from_secs(1))?, Duration::from_secs(2));
        }
        refresh_relayer_health(&registry, &graph, 4, weight, || async {}).await;

        let on_edge = graph.edge(&me, &relayer).and_then(|edge| edge.health()).unwrap_or(1.0);
        assert!((on_edge - recorded).abs() < 1e-9, "got {on_edge}");
        Ok(())
    }

    /// The second SURB of a pair is home and the first is not. A LIFO replier spends the second
    /// first, so the first may simply have waited behind newer SURBs in its store, and its relayer
    /// must not be blamed for it. A FIFO replier spends the first first, so a first that is out
    /// while the second is home is a race its relayer is losing.
    #[rstest::rstest]
    #[case::lifo(SurbPopOrder::Lifo, false)]
    #[case::fifo(SurbPopOrder::Fifo, true)]
    fn a_first_surb_still_out_should_not_count_against_its_relayer_unless_the_replier_pops_fifo(
        #[case] pop_order: SurbPopOrder,
        #[case] counts: bool,
    ) {
        let (graph, _me, _peers) = graph_with(0);
        let recorder = recorder_with(graph, pop_order);
        let [buried, home] = [(); 2].map(|_| key());
        let pending_via = |relayer, partner, pair_index| pending_to(&[relayer], Some(partner), pair_index);
        recorder
            .registry
            .pending
            .insert(surb_id(1), pending_via(buried, surb_id(2), 0));
        recorder
            .registry
            .pending
            .insert(surb_id(2), pending_via(home, surb_id(1), 1));

        recorder.on_replied(&surb_id(2));
        assert_eq!(
            usize::from(counts),
            open_races_of(&recorder.registry.relayer_health, &buried)
        );

        recorder.on_replied(&surb_id(1));
        assert_eq!(
            counts,
            recorder
                .registry
                .relayer_health
                .relayers
                .get(&buried)
                .is_some_and(|history| !history.races.is_empty()),
            "a late reply settles a race against its relayer only if the pop order makes it attributable"
        );
    }

    /// Under FIFO the replier spends a pair's first SURB first. The first coming home and then the
    /// second is the normal order, and says nothing about either relayer: the second trails by the
    /// replier's pace. Only the second coming home while the first is still out is a race, and the
    /// first's relayer loses it by however much later it gets there.
    #[test]
    fn under_fifo_the_first_surb_home_opens_no_race_and_the_second_home_first_does() {
        let (graph, _me, _peers) = graph_with(0);
        let recorder = recorder_with(graph, SurbPopOrder::Fifo);
        let health = &recorder.registry.relayer_health;
        let pending_via = |relayer, partner, pair_index| pending_to(&[relayer], Some(partner), pair_index);

        // The normal order: first home, then second.
        let [first, second] = [(); 2].map(|_| key());
        recorder
            .registry
            .pending
            .insert(surb_id(1), pending_via(first, surb_id(2), 0));
        recorder
            .registry
            .pending
            .insert(surb_id(2), pending_via(second, surb_id(1), 1));
        recorder.on_replied(&surb_id(1));
        assert_eq!(0, open_races_of(health, &first));
        assert_eq!(
            0,
            open_races_of(health, &second),
            "the first coming home first opens no race on the second"
        );
        recorder.on_replied(&surb_id(2));
        assert!(margins_of(health, &first).is_empty() && margins_of(health, &second).is_empty());

        // The abnormal order: second home while the first is still out.
        let [spent_first, spent_second] = [(); 2].map(|_| key());
        recorder
            .registry
            .pending
            .insert(surb_id(3), pending_via(spent_first, surb_id(4), 0));
        recorder
            .registry
            .pending
            .insert(surb_id(4), pending_via(spent_second, surb_id(3), 1));
        recorder.on_replied(&surb_id(4));
        assert_eq!(
            1,
            open_races_of(health, &spent_first),
            "the first is out while the second is home"
        );
        assert_eq!(0, open_races_of(health, &spent_second));
        recorder.on_replied(&surb_id(3));
        assert_eq!(
            0,
            open_races_of(health, &spent_first),
            "and settled once it is home too"
        );
        assert_eq!(1, margins_of(health, &spent_first).len());
        assert_eq!(1, margins_of(health, &spent_second).len());
    }

    /// A replier that pauses between the two replies of a pair (a settle, a phase change, idle) must not
    /// raise the bar a reply has to stay out longer than to count as overdue. Under FIFO the pause falls
    /// between the first-spent SURB and the second, the normal order, which settles no race; had it
    /// settled one, the pause would have become the relayer's largest margin, and a dead relayer would
    /// only read as overdue after that long.
    #[test]
    fn a_pause_in_the_repliers_stream_must_not_inflate_the_overdue_bar_under_fifo() -> anyhow::Result<()> {
        let (graph, _me, _peers) = graph_with(0);
        let recorder = recorder_with(graph, SurbPopOrder::Fifo);
        let health = &recorder.registry.relayer_health;
        let [slow, other] = [(); 2].map(|_| key());
        let pause = Duration::from_secs(10);

        // Pairs replied in the normal order with ten seconds between the replies.
        for pair in 0..MIN_RACES as u8 {
            let (first, second) = (surb_id(1 + 2 * pair), surb_id(2 + 2 * pair));
            recorder
                .registry
                .pending
                .insert(first, pending_to(&[slow], Some(second), 0));
            recorder
                .registry
                .pending
                .insert(second, pending_to(&[other], Some(first), 1));
            recorder.on_replied(&first);
            let mut waiting = recorder
                .registry
                .pending
                .get(&second)
                .context("the second SURB is out")?;
            let (relayers, home) = waiting.partner_replied.context("its partner is home")?;
            waiting.partner_replied = Some((relayers, home.checked_sub(pause).context("clock")?));
            recorder.registry.pending.insert(second, waiting);
            recorder.on_replied(&second);
        }
        assert!(
            margins_of(health, &slow).is_empty() && margins_of(health, &other).is_empty(),
            "a pause between the replies of a pair settles no race"
        );

        // What the relayer's own races say: a margin of 50 ms. Then it dies, and its first-spent SURBs
        // stay out for a second -- far short of the pause, long past its margin.
        settled_since(health, slow, ago(Duration::from_secs(10))?, Duration::from_millis(50));
        open_since(health, slow, MIN_RACES as u8, ago(Duration::from_secs(1))?);
        assert_eq!(
            vec![slow],
            judged_dead(&health.refresh(4)),
            "overdue after its normal margin, not after the pause"
        );
        Ok(())
    }

    /// After the replier drained old SURBs, round trips include a minute in its store; the bar must
    /// stay at the margins, which that wait does not reach.
    #[test]
    fn time_waited_in_the_repliers_store_should_not_raise_the_bar_for_a_black_hole() -> anyhow::Result<()> {
        let health = RelayerHealth::default();
        let relayer = key();
        for _ in 0..MIN_RACES {
            health.record(
                relayer,
                Race {
                    at: ago(Duration::from_secs(10))?,
                    margin: jitter(0).as_secs_f64(),
                    winning_round_trip: 54.0,
                },
            );
        }
        open_since(&health, relayer, MIN_RACES as u8, ago(Duration::from_secs(1))?);

        assert_eq!(vec![relayer], judged_dead(&health.refresh(4)));
        Ok(())
    }

    /// Two healthy relayers trading races by equal jitter both way stay at full health.
    #[test]
    fn symmetric_jitter_should_cost_neither_relayer() {
        let health = RelayerHealth::default();
        let [a, b] = [(); 2].map(|_| key());
        for i in 0..20 {
            let (later, earlier) = if i % 2 == 0 { (a, b) } else { (b, a) };
            health.record_race(
                &leg(&[later]),
                surb_id(i as u8),
                &leg(&[earlier]),
                ROUND_TRIP,
                jitter(i),
            );
        }
        assert_eq!(vec![1.0, 1.0], health_after_flush(&health, &[a, b]));
    }

    #[test]
    fn replies_out_without_settled_races_should_not_make_a_black_hole() {
        let health = RelayerHealth::default();
        let relayer = key();
        open_since(&health, relayer, 5, Instant::now() - Duration::from_secs(5));

        for _ in 0..2 {
            assert!(judged_dead(&health.refresh(4)).is_empty());
        }
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

    /// Nothing assumes two SURBs per packet: more are raced in adjacent pairs.
    #[test]
    fn five_surbs_in_one_packet_should_race_in_adjacent_pairs() {
        let [a, b] = [(); 2].map(|_| key());
        let ids = [1, 2, 3, 4, 5].map(surb_id);
        assert_eq!(
            vec![Some(ids[1]), Some(ids[0]), Some(ids[3]), Some(ids[2]), None],
            race_partners(&ids, &[leg(&[a]), leg(&[b]), leg(&[b]), leg(&[a]), leg(&[a])])
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
    fn an_idle_flush_should_log_nothing_at_info() -> anyhow::Result<()> {
        let (graph, me, peers) = graph_with(1);
        let destination = peers[0];

        let idle = Arc::new(InfoCounter::default());
        tracing::subscriber::with_default(idle.clone(), || {
            flush_into(&SurbRoundTripRegistry::default(), &graph, 0)
        });
        assert_eq!(0, idle.infos.load(Ordering::Relaxed), "nothing was minted or replied");

        let recorder = recorder(graph.clone());
        recorder.on_minted(
            &ResolvedTransportRouting::Forward {
                pseudonym: HoprPseudonym::random(),
                forward_path: ValidatedPath::direct(destination, address()),
                return_paths: vec![ValidatedPath::direct(me, address())],
            },
            &[surb_id(1)],
        );
        let busy = Arc::new(InfoCounter::default());
        tracing::subscriber::with_default(busy.clone(), || flush_into(&recorder.registry, &graph, 0));
        assert_eq!(
            1,
            busy.infos.load(Ordering::Relaxed),
            "one summary line for the interval"
        );
        Ok(())
    }

    /// Counts INFO events; `enabled` is always true so the logging macros' bodies actually run.
    #[derive(Default)]
    struct InfoCounter {
        infos: AtomicU64,
    }

    impl tracing::Subscriber for InfoCounter {
        fn enabled(&self, _: &tracing::Metadata<'_>) -> bool {
            true
        }

        fn new_span(&self, _: &tracing::span::Attributes<'_>) -> tracing::span::Id {
            tracing::span::Id::from_u64(1)
        }

        fn record(&self, _: &tracing::span::Id, _: &tracing::span::Record<'_>) {}

        fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}

        fn event(&self, event: &tracing::Event<'_>) {
            if *event.metadata().level() == tracing::Level::INFO {
                self.infos.fetch_add(1, Ordering::Relaxed);
            }
        }

        fn enter(&self, _: &tracing::span::Id) {}

        fn exit(&self, _: &tracing::span::Id) {}
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
