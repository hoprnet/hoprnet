//! Judges forward relayers from the receipts an Exit sends for the Session packets it received.
//!
//! Each Session packet sent over a forward path is recorded with its secret-derived tag. A receipt lists the tags
//! that reached the Exit; a packet still unreceipted once later packets of the same Session are (`FORWARD_GRACE`)
//! is overdue, and counts against the relayers on its path that nothing has been delivered through since. A relayer
//! that the overdue packets point at, after `explain_away` and `VERDICT_CONFIRMATION`, is excluded from the forward
//! draws to that Exit until the verdict is lifted. Everything is judged within one pseudonym, so a Session's receipts
//! never judge another Session's packets.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

use hopr_api::types::{
    crypto::types::OffchainPublicKey,
    internal::prelude::{HoprPseudonym, HoprSurbId, RoutingOptions, SURB_ID_SIZE},
};
use hopr_transport_session::ForwardReceiptSink;
use parking_lot::{Mutex, RwLock};

use super::surb_telemetry::{MIN_RACES, Suspect, VERDICT_CONFIRMATION, explain_away};

/// The most packets recorded at once; the oldest is evicted first.
const MAX_RECORDS: usize = 16_384;
/// How long a packet is waited for, after which its record is dropped and counts for nothing.
const RECORD_TTL: Duration = Duration::from_secs(10);
/// How long a packet may trail a receipted one, sent after it, before it is overdue.
const FORWARD_GRACE: Duration = Duration::from_millis(500);
/// How long a verdict excludes a relayer.
const FORWARD_VERDICT_TTL: Duration = Duration::from_secs(30);
/// The most relayers excluded for one Exit at once, which bounds how far a lying Exit can steer.
const MAX_FORWARD_VERDICTS_PER_EXIT: usize = 2;
/// How long a stream or a delivery is kept without news.
const STATE_TTL: Duration = Duration::from_secs(60);
/// The most receipts one receipt is held to have skipped over; a larger gap is a loss.
const MAX_MISSING: u32 = 16;
/// The highest `seq` a new Session sends first (see the Exit's overflow jump).
const NEW_SESSION_SEQ_MAX: u32 = 3;

type Key = OffchainPublicKey;

struct Record {
    exit: Key,
    /// The relayers, nearest to us first.
    relayers: [Option<Key>; RoutingOptions::MAX_INTERMEDIATE_HOPS],
    sent_at: Instant,
}

impl Record {
    fn relayers(&self) -> impl Iterator<Item = &Key> {
        self.relayers.iter().flatten()
    }
}

/// The receipts of one pseudonym.
struct Stream {
    last_seq: u32,
    /// Packets sent before it are no evidence: receipts may have been lost.
    floor: Instant,
    /// The latest send time among the packets receipted.
    watermark: Option<Instant>,
    seen: Instant,
    /// Receipts skipped over, and since when; they may only be late, so nothing is judged while they are awaited.
    missing: Vec<(u32, Instant)>,
}

/// A relayer that has been a black hole since `since`, over packets from `oldest` on.
struct Suspicion {
    since: Instant,
    oldest: Instant,
    pseudonym: HoprPseudonym,
}

#[derive(Default)]
struct State {
    /// By pseudonym, so that judging a Session touches only its packets.
    records: HashMap<HoprPseudonym, HashMap<u32, Record>>,
    /// How many records there are in all.
    count: usize,
    /// Records by send time; entries of records gone are skipped.
    order: VecDeque<(HoprPseudonym, u32, Instant)>,
    streams: HashMap<HoprPseudonym, Stream>,
    /// When the packet sent last got through each (exit, relayer).
    delivered: HashMap<(Key, Key), Instant>,
    /// The (exit, relayer) pairs excluded, and since when.
    verdicts: HashMap<(Key, Key), Instant>,
    /// When each verdict was last lifted; a relapse within the TTL is judged without the wait.
    lifted: HashMap<(Key, Key), Instant>,
    /// The relayers that are black holes but have not yet stood for `VERDICT_CONFIRMATION`.
    suspected: HashMap<(Key, Key), Suspicion>,
    /// Whether the verdicts changed since they were last published.
    changed: bool,
}

/// Records forward Session packets and judges their relayers from the Exit's receipts.
#[derive(Default)]
pub struct ForwardReceipts {
    any_verdict: AtomicBool,
    /// The relayers excluded per Exit, for the planner.
    excluded: RwLock<HashMap<Key, Vec<Key>>>,
    state: Mutex<State>,
}

impl ForwardReceipts {
    #[cfg(test)]
    pub(crate) fn recorded(&self) -> usize {
        self.state.lock().count
    }

    /// The relayers excluded from forward draws to `exit`; the only cost without verdicts is an atomic load.
    pub fn excluded_relayers(&self, exit: &Key) -> Vec<Key> {
        if !self.any_verdict.load(Ordering::Relaxed) {
            return Vec::new();
        }
        self.excluded.read().get(exit).cloned().unwrap_or_default()
    }

    /// Records a Session packet sent over `path`, the relayers and then the Exit.
    pub fn record(&self, pseudonym: HoprPseudonym, tag: u32, path: &[Key]) {
        self.record_at(pseudonym, tag, path, Instant::now());
    }

    fn record_at(&self, pseudonym: HoprPseudonym, tag: u32, path: &[Key], sent_at: Instant) {
        let Some((exit, relayers)) = path.split_last() else {
            return;
        };
        let mut slots = [None; RoutingOptions::MAX_INTERMEDIATE_HOPS];
        for (slot, relayer) in slots.iter_mut().zip(relayers) {
            *slot = Some(*relayer);
        }
        let mut state = self.state.lock();
        while state.count >= MAX_RECORDS || state.order.len() >= 2 * MAX_RECORDS {
            let Some((owner, tag, at)) = state.order.pop_front() else {
                break;
            };
            if state.is_live(owner, tag, at) {
                state.remove(owner, tag);
            }
        }
        state.insert(
            pseudonym,
            tag,
            Record {
                exit: *exit,
                relayers: slots,
                sent_at,
            },
        );
        state.order.push_back((pseudonym, tag, sent_at));
    }

    /// Takes the receipt numbered `seq` for `pseudonym`, which lists the `tags` that reached the Exit in the
    /// intervals `first_seq..=seq`.
    fn receipts_at(&self, pseudonym: HoprPseudonym, seq: u32, first_seq: u32, tags: &[u32], now: Instant) {
        let first_seq = first_seq.min(seq);
        let mut state = self.state.lock();

        let mut gap = 0;
        match state.streams.get_mut(&pseudonym) {
            // The pseudonym was reused: a new Session numbers from 1, or a little above it
            Some(stream)
                if (seq == 1 && stream.last_seq > 1)
                    || (seq <= NEW_SESSION_SEQ_MAX && seq.saturating_add(2) < stream.last_seq) =>
            {
                *stream = Stream::new(seq, now);
            }
            Some(stream) => {
                // The seqs between the last one and the first this receipt covers are skipped over: they may only be
                // late
                let next = stream.last_seq.saturating_add(1);
                gap = first_seq.saturating_sub(next);
                if gap > MAX_MISSING {
                    stream.floor = now;
                } else {
                    stream.missing.extend((next..first_seq).map(|missing| (missing, now)));
                }
                stream
                    .missing
                    .retain(|(missing, _)| !(first_seq..=seq).contains(missing));
                // Still missing after the grace: lost. It can only have carried packets that reached the Exit
                // before the jump, so the evidence from then on stays.
                let lost = stream
                    .missing
                    .iter()
                    .filter(|(_, since)| now.saturating_duration_since(*since) > FORWARD_GRACE)
                    .map(|(_, since)| *since)
                    .max();
                if let Some(since) = lost {
                    stream.floor = stream.floor.max(since);
                    stream
                        .missing
                        .retain(|(_, since)| now.saturating_duration_since(*since) <= FORWARD_GRACE);
                }
                stream.last_seq = stream.last_seq.max(seq);
                stream.seen = now;
            }
            None => {
                state.streams.insert(pseudonym, Stream::new(seq, now));
            }
        }

        // A packet is timely if it is not far behind what was receipted before this receipt
        let watermark_before = state.streams.get(&pseudonym).and_then(|stream| stream.watermark);
        let mut matched = 0;
        let mut lifts = Vec::new();
        // Nothing to match for a pseudonym with no records
        let to_match = if state.records.contains_key(&pseudonym) {
            tags
        } else {
            &[]
        };
        for tag in to_match {
            let Some(record) = state.remove(pseudonym, *tag) else {
                continue;
            };
            if now.saturating_duration_since(record.sent_at) > RECORD_TTL {
                continue;
            }
            matched += 1;
            if let Some(stream) = state.streams.get_mut(&pseudonym) {
                stream.watermark = stream.watermark.max(Some(record.sent_at));
            }
            let timely = watermark_before.is_some_and(|watermark| record.sent_at + FORWARD_GRACE > watermark);
            for relayer in record.relayers() {
                let key = (record.exit, *relayer);
                let delivered = state.delivered.entry(key).or_insert(record.sent_at);
                *delivered = (*delivered).max(record.sent_at);
                // Only a timely packet sent after the verdict says the relayer is delivering again; a late one
                // may have been drawn just before the verdict and be what it was about.
                if timely
                    && state
                        .verdicts
                        .get(&key)
                        .is_some_and(|verdict| record.sent_at > *verdict)
                {
                    lifts.push(key);
                }
            }
        }
        for (exit, relayer) in lifts {
            state.lift(exit, relayer, now);
        }
        let waiting = state
            .streams
            .get(&pseudonym)
            .is_some_and(|stream| !stream.missing.is_empty());
        let outstanding = if waiting {
            state.records.get(&pseudonym).map_or(0, HashMap::len)
        } else {
            state.evaluate(pseudonym, now)
        };
        state.publish(&self.excluded, &self.any_verdict);
        tracing::debug!(%pseudonym, seq, first_seq, tags = tags.len(), matched, gap, outstanding, "forward receipts received");
    }

    /// Lifts the verdicts that have run their course and forgets what has expired. Called once a second.
    pub fn refresh(&self) {
        self.refresh_at(Instant::now());
    }

    fn refresh_at(&self, now: Instant) {
        let mut state = self.state.lock();
        let expired: Vec<_> = state
            .verdicts
            .iter()
            .filter(|(_, since)| now.saturating_duration_since(**since) >= FORWARD_VERDICT_TTL)
            .map(|(key, _)| *key)
            .collect();
        for (exit, relayer) in expired {
            state.lift(exit, relayer, now);
        }
        state.remove_where(|record| now.saturating_duration_since(record.sent_at) > RECORD_TTL);
        state.prune_order(now);
        state
            .streams
            .retain(|_, stream| now.saturating_duration_since(stream.seen) <= STATE_TTL);
        state
            .delivered
            .retain(|_, at| now.saturating_duration_since(*at) <= STATE_TTL);
        state
            .lifted
            .retain(|_, at| now.saturating_duration_since(*at) <= FORWARD_VERDICT_TTL);
        state
            .suspected
            .retain(|_, suspicion| now.saturating_duration_since(suspicion.since) <= RECORD_TTL);
        state.publish(&self.excluded, &self.any_verdict);
    }
}

impl Stream {
    fn new(seq: u32, now: Instant) -> Self {
        Self {
            last_seq: seq,
            floor: now,
            watermark: None,
            seen: now,
            missing: Vec::new(),
        }
    }
}

impl State {
    fn is_live(&self, owner: HoprPseudonym, tag: u32, at: Instant) -> bool {
        self.records
            .get(&owner)
            .and_then(|records| records.get(&tag))
            .is_some_and(|record| record.sent_at == at)
    }

    fn insert(&mut self, owner: HoprPseudonym, tag: u32, record: Record) {
        if self.records.entry(owner).or_default().insert(tag, record).is_none() {
            self.count += 1;
        }
    }

    fn remove(&mut self, owner: HoprPseudonym, tag: u32) -> Option<Record> {
        let records = self.records.get_mut(&owner)?;
        let record = records.remove(&tag)?;
        if records.is_empty() {
            self.records.remove(&owner);
        }
        self.count -= 1;
        Some(record)
    }

    fn remove_where(&mut self, gone: impl Fn(&Record) -> bool) {
        for records in self.records.values_mut() {
            records.retain(|_, record| !gone(record));
        }
        self.records.retain(|_, records| !records.is_empty());
        self.count = self.records.values().map(HashMap::len).sum();
    }

    /// Drops the order entries of records gone or expired.
    fn prune_order(&mut self, now: Instant) {
        while let Some((owner, tag, at)) = self.order.front().copied() {
            let live = self.is_live(owner, tag, at);
            if live && now.saturating_duration_since(at) <= RECORD_TTL {
                break;
            }
            if live {
                self.remove(owner, tag);
            }
            self.order.pop_front();
        }
    }

    /// Lifts the verdict on `relayer` for `exit`, dropping the packets sent through it before the lift.
    fn lift(&mut self, exit: Key, relayer: Key, now: Instant) {
        if self.verdicts.remove(&(exit, relayer)).is_none() {
            return;
        }
        tracing::info!(%relayer, %exit, "forward verdict lifted");
        self.changed = true;
        self.lifted.insert((exit, relayer), now);
        self.suspected.remove(&(exit, relayer));
        self.remove_where(|record| record.exit == exit && record.relayers().any(|other| *other == relayer));
    }

    /// Judges the packets of `pseudonym` that are overdue; returns how many it has outstanding.
    fn evaluate(&mut self, pseudonym: HoprPseudonym, now: Instant) -> usize {
        let (Some(stream), Some(records)) = (self.streams.get(&pseudonym), self.records.get(&pseudonym)) else {
            return 0;
        };
        let (floor, watermark) = (stream.floor, stream.watermark);
        let live = || {
            records
                .iter()
                .filter(|(_, record)| now.saturating_duration_since(record.sent_at) <= RECORD_TTL)
        };

        // Per (exit, relayer): the unanswered overdue packets.
        let mut unanswered: HashMap<(Key, Key), Vec<(HoprSurbId, Instant)>> = HashMap::new();
        let mut outstanding = 0;
        for (tag, record) in live() {
            outstanding += 1;
            let overdue = watermark
                .is_some_and(|watermark| record.sent_at >= floor && record.sent_at + FORWARD_GRACE <= watermark);
            if !overdue {
                continue;
            }
            for relayer in record.relayers() {
                let key = (record.exit, *relayer);
                if !self.delivered.get(&key).is_some_and(|at| record.sent_at <= *at) {
                    unanswered.entry(key).or_default().push((id_of(*tag), record.sent_at));
                }
            }
        }
        unanswered.retain(|_, packets| packets.len() >= MIN_RACES);

        // The packets outstanding through each of those, with the relayer's position on their paths.
        let mut open: HashMap<(Key, Key), HashMap<HoprSurbId, usize>> = HashMap::new();
        if !unanswered.is_empty() {
            for (tag, record) in live() {
                for (position, relayer) in record.relayers().enumerate() {
                    let key = (record.exit, *relayer);
                    if unanswered.contains_key(&key) {
                        open.entry(key).or_default().insert(id_of(*tag), position);
                    }
                }
            }
        }

        let mut suspects_by_exit: HashMap<Key, Vec<Suspect<Key>>> = HashMap::new();
        let mut oldest: HashMap<(Key, Key), Instant> = HashMap::new();
        for (key, packets) in unanswered {
            if let Some(at) = packets.iter().map(|(_, at)| *at).min() {
                oldest.insert(key, at);
            }
            suspects_by_exit.entry(key.0).or_default().push(Suspect {
                relayer: key.1,
                unanswered: packets.into_iter().map(|(id, _)| id).collect(),
                open: open.remove(&key).unwrap_or_default(),
                needed: MIN_RACES,
            });
        }

        let mut black_holes: HashSet<(Key, Key)> = HashSet::new();
        for (exit, suspects) in suspects_by_exit {
            let known_dead = |relayer: &Key| self.verdicts.contains_key(&(exit, *relayer));
            for relayer in explain_away(suspects, known_dead) {
                black_holes.insert((exit, relayer));
            }
        }

        for key in &black_holes {
            if self.verdicts.contains_key(key) {
                continue;
            }
            let Some(oldest) = oldest.get(key).copied() else {
                continue;
            };
            let delivered = self.delivered.get(key).copied();
            let suspicion = self.suspected.entry(*key).or_insert(Suspicion {
                since: now,
                oldest,
                pseudonym,
            });
            if delivered.is_some_and(|at| at >= suspicion.oldest) {
                // It delivers after all.
                *suspicion = Suspicion {
                    since: now,
                    oldest,
                    pseudonym,
                };
                continue;
            }
            let relapse = self
                .lifted
                .get(key)
                .is_some_and(|at| now.saturating_duration_since(*at) < FORWARD_VERDICT_TTL);
            if !relapse && now.saturating_duration_since(suspicion.since) < VERDICT_CONFIRMATION {
                continue;
            }
            if self.verdicts.keys().filter(|(exit, _)| *exit == key.0).count() >= MAX_FORWARD_VERDICTS_PER_EXIT {
                continue;
            }
            let (exit, relayer) = *key;
            tracing::info!(%relayer, %exit, over = "forward", "relayer judged dead");
            self.suspected.remove(key);
            self.verdicts.insert(*key, now);
            self.changed = true;
        }
        // A suspicion that is no black hole any more has not stood.
        self.suspected
            .retain(|key, suspicion| suspicion.pseudonym != pseudonym || black_holes.contains(key));
        outstanding
    }

    /// Publishes the verdicts for the planner, if they changed.
    fn publish(&mut self, excluded: &RwLock<HashMap<Key, Vec<Key>>>, any_verdict: &AtomicBool) {
        if !std::mem::take(&mut self.changed) {
            return;
        }
        let mut view: HashMap<Key, Vec<Key>> = HashMap::new();
        for (exit, relayer) in self.verdicts.keys() {
            view.entry(*exit).or_default().push(*relayer);
        }
        *excluded.write() = view;
        any_verdict.store(!self.verdicts.is_empty(), Ordering::Relaxed);
    }
}

/// The record of a tag, in the form `explain_away` keys its replies by.
fn id_of(tag: u32) -> HoprSurbId {
    let mut id = [0u8; SURB_ID_SIZE];
    id[..size_of::<u32>()].copy_from_slice(&tag.to_be_bytes());
    id
}

impl ForwardReceiptSink for ForwardReceipts {
    fn receipts(&self, pseudonym: HoprPseudonym, seq: u32, first_seq: u32, tags: &[u32]) {
        self.receipts_at(pseudonym, seq, first_seq, tags, Instant::now());
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Write as _, sync::Arc};

    use anyhow::Context;
    use hopr_api::types::crypto::{
        crypto_traits::Randomizable,
        prelude::{Keypair, OffchainKeypair},
    };

    use super::*;

    fn key() -> Key {
        *OffchainKeypair::random().public()
    }

    /// Collects the events of a test as `LEVEL field=value ...` lines.
    #[derive(Default, Clone)]
    struct Logs(Arc<Mutex<Vec<String>>>);

    struct Fields(String);

    impl tracing::field::Visit for Fields {
        fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
            let _ = write!(self.0, "{}={:?} ", field.name(), value);
        }
    }

    impl tracing::Subscriber for Logs {
        fn enabled(&self, _: &tracing::Metadata<'_>) -> bool {
            true
        }

        fn register_callsite(&self, _: &'static tracing::Metadata<'static>) -> tracing::subscriber::Interest {
            tracing::subscriber::Interest::always()
        }

        fn max_level_hint(&self) -> Option<tracing::metadata::LevelFilter> {
            Some(tracing::metadata::LevelFilter::TRACE)
        }

        fn new_span(&self, _: &tracing::span::Attributes<'_>) -> tracing::span::Id {
            tracing::span::Id::from_u64(1)
        }

        fn record(&self, _: &tracing::span::Id, _: &tracing::span::Record<'_>) {}

        fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}

        fn event(&self, event: &tracing::Event<'_>) {
            let mut fields = Fields(String::new());
            event.record(&mut fields);
            self.0.lock().push(format!("{} {}", event.metadata().level(), fields.0));
        }

        fn enter(&self, _: &tracing::span::Id) {}

        fn exit(&self, _: &tracing::span::Id) {}
    }

    /// A Session sending a packet every 10 ms and receipted every 250 ms, with the Exit's view of time in
    /// milliseconds from `t0`.
    struct Sim {
        fr: ForwardReceipts,
        pseudonym: HoprPseudonym,
        exit: Key,
        t0: Instant,
        next_tag: u32,
        seq: u32,
        /// (tag, arrival at the Exit in ms, if it arrives)
        sent: Vec<(u32, Option<u64>)>,
        /// When the receipts were made, the lost ones included.
        receipted: Vec<u64>,
        /// How many intervals a receipt covers.
        cover: usize,
        /// The seq of the last overflow jump, and when the receipt before it was made: older intervals are not
        /// repeated.
        jump_base: (u32, u64),
        verdict_ms: Option<u64>,
        /// How long the receipt with a `seq` is delayed, and the receipts delayed: (release in ms, seq, first seq,
        /// tags).
        late: Vec<(u32, u64)>,
        held: Vec<(u64, u32, u32, Vec<u32>)>,
    }

    impl Sim {
        fn new() -> Self {
            Self::sharing(ForwardReceipts::default(), key(), Instant::now())
        }

        fn sharing(fr: ForwardReceipts, exit: Key, t0: Instant) -> Self {
            Self {
                fr,
                pseudonym: HoprPseudonym::random(),
                exit,
                t0,
                next_tag: 0,
                seq: 0,
                sent: Vec::new(),
                receipted: Vec::new(),
                cover: 2,
                jump_base: (0, 0),
                verdict_ms: None,
                late: Vec::new(),
                held: Vec::new(),
            }
        }

        fn at(&self, ms: u64) -> Instant {
            self.t0 + Duration::from_millis(ms)
        }

        fn send(&mut self, ms: u64, relayers: &[Key], delay: Option<u64>) {
            let mut path = relayers.to_vec();
            path.push(self.exit);
            self.fr.record_at(self.pseudonym, self.next_tag, &path, self.at(ms));
            self.sent.push((self.next_tag, delay.map(|delay| ms + delay)));
            self.next_tag += 1;
        }

        /// Makes the receipt at `ms` with the next `seq` plus `jump`, covering `cover` intervals but none from before
        /// an Exit overflow (a jump), which repeats nothing.
        fn receipt(&mut self, ms: u64, jump: u32) {
            self.seq += 1 + jump;
            let covered = if jump > 0 { 1 } else { self.cover };
            if jump > 0 {
                self.jump_base = (self.seq, self.receipted.last().copied().unwrap_or(0));
            }
            let first_seq = self.seq.saturating_sub(covered as u32 - 1).max(1).max(self.jump_base.0);
            let cut = self
                .receipted
                .len()
                .checked_sub(covered)
                .map_or(0, |i| self.receipted[i])
                .max(self.jump_base.1);
            let tags: Vec<u32> = self
                .sent
                .iter()
                .filter(|(_, arrival)| arrival.is_some_and(|a| a <= ms && a > cut))
                .map(|(tag, _)| *tag)
                .collect();
            self.receipted.push(ms);
            match self.late.iter().find(|(seq, _)| *seq == self.seq) {
                Some((_, delay)) => self.held.push((ms + delay, self.seq, first_seq, tags)),
                None => self.deliver(ms, self.seq, first_seq, &tags),
            }
        }

        fn deliver(&mut self, ms: u64, seq: u32, first_seq: u32, tags: &[u32]) {
            self.fr.receipts_at(self.pseudonym, seq, first_seq, tags, self.at(ms));
            if self.verdict_ms.is_none() && !self.fr.excluded_relayers(&self.exit).is_empty() {
                self.verdict_ms = Some(ms);
            }
        }

        /// Delivers the delayed receipts that are due at `ms`.
        fn release(&mut self, ms: u64) {
            let (due, held): (Vec<_>, Vec<_>) = std::mem::take(&mut self.held)
                .into_iter()
                .partition(|(at, ..)| *at <= ms);
            self.held = held;
            for (_, seq, first_seq, tags) in due {
                self.deliver(ms, seq, first_seq, &tags);
            }
        }

        /// Runs from `from` to `to` ms: a packet every 10 ms over `route(i)` that arrives after `fate(path)`, and
        /// a receipt every 250 ms unless `lost(seq)`.
        fn run(
            &mut self,
            from: u64,
            to: u64,
            route: impl Fn(usize, &[Key]) -> Vec<Key>,
            fate: impl Fn(&[Key]) -> Option<u64>,
            lost: impl Fn(u32) -> bool,
        ) {
            for ms in (from..to).step_by(10) {
                self.release(ms);
                let excluded = self.fr.excluded_relayers(&self.exit);
                let path = route((ms / 10) as usize, &excluded);
                let delay = fate(&path);
                self.send(ms, &path, delay);
                if ms % 250 == 0 && ms > 0 {
                    if lost(self.seq + 1) {
                        self.seq += 1;
                        self.receipted.push(ms);
                    } else {
                        self.receipt(ms, 0);
                    }
                }
            }
        }
    }

    /// Round-robin over `paths`, skipping the paths through an excluded relayer.
    fn rotate(paths: Vec<Vec<Key>>) -> impl Fn(usize, &[Key]) -> Vec<Key> {
        move |i, excluded| {
            let allowed: Vec<&Vec<Key>> = paths
                .iter()
                .filter(|path| !path.iter().any(|relayer| excluded.contains(relayer)))
                .collect();
            let pool: Vec<&Vec<Key>> = if allowed.is_empty() {
                paths.iter().collect()
            } else {
                allowed
            };
            pool[i % pool.len()].clone()
        }
    }

    /// Runs `scenario` under a capturing subscriber until its log has a line with `expect`. A scoped subscriber can
    /// miss events while other tests install theirs, so a miss runs the scenario again.
    fn logged<T>(expect: &str, scenario: impl Fn() -> anyhow::Result<T>) -> anyhow::Result<(T, Vec<String>)> {
        for _ in 0..20 {
            let logs = Logs::default();
            let out = tracing::subscriber::with_default(logs.clone(), &scenario)?;
            let lines = logs.0.lock().clone();
            if lines.iter().any(|line| line.contains(expect)) {
                return Ok((out, lines));
            }
        }
        anyhow::bail!("`{expect}` was never logged")
    }

    fn lose_nothing(_: u32) -> bool {
        false
    }

    const FAST: Option<u64> = Some(20);

    #[test]
    #[serial_test::serial(routing_logs)]
    fn a_dead_relayer_should_be_judged_after_the_confirmation_and_logged() -> anyhow::Result<()> {
        let [a, b, c] = [(); 3].map(|_| key());
        let (sim, lines) = logged("relayer judged dead", || {
            let mut sim = Sim::new();
            sim.run(
                0,
                4_000,
                rotate(vec![vec![a], vec![b], vec![c]]),
                |path| if path.contains(&a) { None } else { FAST },
                lose_nothing,
            );
            Ok(sim)
        })?;

        assert_eq!(vec![a], sim.fr.excluded_relayers(&sim.exit));
        // The first receipt that finds it a black hole is at 1000 ms: it stands 900 ms after that.
        let verdict = sim.verdict_ms.context("judged")?;
        assert!((1_900..=2_250).contains(&verdict), "judged at {verdict} ms");
        let judged: Vec<_> = lines
            .iter()
            .filter(|line| line.contains("relayer judged dead"))
            .collect();
        assert_eq!(1, judged.len(), "{lines:?}");
        assert!(
            judged[0].starts_with("INFO ") && judged[0].contains("over=\"forward\""),
            "{}",
            judged[0]
        );
        assert!(
            judged[0].contains("relayer=") && judged[0].contains("exit="),
            "{}",
            judged[0]
        );
        Ok(())
    }

    #[test]
    fn a_stalling_relayer_should_be_judged_and_stay_excluded_despite_its_late_receipts() {
        let [s, b, c] = [(); 3].map(|_| key());
        let mut sim = Sim::new();
        sim.run(
            0,
            9_000,
            rotate(vec![vec![s], vec![b], vec![c]]),
            |path| if path.contains(&s) { Some(3_000) } else { FAST },
            lose_nothing,
        );
        assert_eq!(vec![s], sim.fr.excluded_relayers(&sim.exit));
        assert!(
            sim.verdict_ms.is_some_and(|ms| ms < 3_000),
            "judged before its first late receipt"
        );
    }

    #[test]
    fn one_lost_receipt_should_lose_nothing() {
        let [a, b, c] = [(); 3].map(|_| key());
        let verdict_with = |lost_seq: u32| {
            let mut sim = Sim::new();
            sim.run(
                0,
                4_000,
                rotate(vec![vec![a], vec![b], vec![c]]),
                |path| if path.contains(&a) { None } else { FAST },
                |seq| seq == lost_seq,
            );
            (sim.verdict_ms, sim.fr.excluded_relayers(&sim.exit))
        };
        let (expected, _) = verdict_with(0);
        for lost in [3, 5, 6] {
            assert_eq!((expected, vec![a]), verdict_with(lost), "receipt {lost} lost");
        }
    }

    /// More than one receipt lost, or an overflow at the Exit, make the evidence before them void: only the
    /// packets sent after it can judge, which takes until after 2.5 s here, against 2 s without the jump.
    #[test]
    fn a_larger_gap_or_an_overflow_should_judge_nobody() {
        let [a, b, c] = [(); 3].map(|_| key());
        let judge_at_2500 = |jump: u32| {
            let mut sim = Sim::new();
            let route = rotate(vec![vec![a], vec![b], vec![c]]);
            let fate = |path: &[Key]| if path.contains(&a) { None } else { FAST };
            sim.run(0, 1_000, &route, fate, lose_nothing);
            sim.receipt(1_000, jump);
            sim.run(1_010, 2_500, &route, fate, lose_nothing);
            sim.fr.excluded_relayers(&sim.exit)
        };
        assert_eq!(vec![a], judge_at_2500(0), "the control run judges");
        for jump in [3, 5] {
            assert!(judge_at_2500(jump).is_empty(), "jump {jump}");
        }
    }

    #[test]
    fn withholding_every_receipt_should_judge_nobody() {
        let [a, b] = [(); 2].map(|_| key());
        let mut sim = Sim::new();
        sim.run(0, 5_000, rotate(vec![vec![a], vec![b]]), |_| None, |_| true);
        sim.fr.receipts_at(sim.pseudonym, 1, 1, &[], sim.at(5_000));
        sim.fr.refresh_at(sim.at(5_000));
        assert!(sim.fr.excluded_relayers(&sim.exit).is_empty());
    }

    #[test]
    fn one_sessions_receipts_should_never_judge_another_sessions_packets() {
        let [a, b] = [(); 2].map(|_| key());
        let exit = key();
        let t0 = Instant::now();
        let mut healthy = Sim::sharing(ForwardReceipts::default(), exit, t0);
        healthy.run(0, 100, rotate(vec![vec![b]]), |_| FAST, lose_nothing);

        let other = HoprPseudonym::random();
        for i in 0..100 {
            healthy.fr.record_at(other, i, &[a, exit], healthy.at(i as u64 * 10));
        }
        healthy.run(100, 5_000, rotate(vec![vec![a], vec![b]]), |_| FAST, lose_nothing);
        assert!(healthy.fr.excluded_relayers(&exit).is_empty());
        assert!(
            healthy.fr.state.lock().records.contains_key(&other),
            "the other Session's packets are only waited for"
        );
    }

    /// `x -> r` drops, while `x` and `r` each deliver other packets.
    #[test]
    fn a_relayer_dropping_only_one_neighbours_packets_should_frame_nobody() {
        let [x, y, r, z] = [(); 4].map(|_| key());
        let mut sim = Sim::new();
        sim.run(
            0,
            6_000,
            rotate(vec![vec![x, r], vec![y, r], vec![y, z], vec![x, z]]),
            |path| if path == [x, r] { None } else { FAST },
            lose_nothing,
        );
        assert!(sim.fr.excluded_relayers(&sim.exit).is_empty());
    }

    #[test]
    fn a_true_tie_should_wait() {
        let [a, b, c] = [(); 3].map(|_| key());
        let mut sim = Sim::new();
        sim.run(
            0,
            6_000,
            rotate(vec![vec![a, b], vec![b, a], vec![c]]),
            |path| if path == [c] { FAST } else { None },
            lose_nothing,
        );
        assert!(sim.fr.excluded_relayers(&sim.exit).is_empty());
    }

    #[test]
    fn the_relayer_downstream_of_a_dead_one_should_not_be_judged() {
        let [a, b, c] = [(); 3].map(|_| key());
        let mut sim = Sim::new();
        sim.run(
            0,
            6_000,
            rotate(vec![vec![a, b], vec![c]]),
            |path| if path.contains(&a) { None } else { FAST },
            lose_nothing,
        );
        assert_eq!(vec![a], sim.fr.excluded_relayers(&sim.exit));
    }

    #[test]
    fn a_loss_answered_by_a_later_delivery_should_not_count() {
        let [a, b] = [(); 2].map(|_| key());
        let mut sim = Sim::new();
        let lost = std::cell::Cell::new(0);
        sim.run(
            0,
            6_000,
            rotate(vec![vec![a], vec![b]]),
            |path| {
                if path.contains(&a) {
                    lost.set(lost.get() + 1);
                    // Every fifth packet through it is lost; the ones after it get home.
                    (lost.get() % 5 != 0).then_some(20)
                } else {
                    FAST
                }
            },
            lose_nothing,
        );
        assert!(lost.get() > 100);
        assert!(sim.fr.excluded_relayers(&sim.exit).is_empty());
    }

    #[test]
    fn a_verdict_should_be_per_exit_and_at_most_two_at_once() {
        let [a, b, c, d] = [(); 4].map(|_| key());
        let [e1, e2] = [(); 2].map(|_| key());
        let t0 = Instant::now();
        let mut one = Sim::sharing(ForwardReceipts::default(), e1, t0);
        one.run(
            0,
            6_000,
            rotate(vec![vec![a], vec![b], vec![c], vec![d]]),
            |path| if path.contains(&d) { FAST } else { None },
            lose_nothing,
        );
        let excluded = one.fr.excluded_relayers(&e1);
        assert_eq!(2, excluded.len(), "{excluded:?}");
        assert!(!excluded.contains(&d));
        assert!(one.fr.excluded_relayers(&e2).is_empty(), "other exits are unaffected");
    }

    #[test]
    #[serial_test::serial(routing_logs)]
    fn a_verdict_should_lapse_after_its_ttl_without_a_stale_rejudgement() -> anyhow::Result<()> {
        let [a, b, c] = [(); 3].map(|_| key());
        logged("forward verdict lifted", || {
            let mut sim = Sim::new();
            let fate = |path: &[Key]| if path.contains(&a) { None } else { FAST };
            let healthy = rotate(vec![vec![b], vec![c]]);
            sim.run(0, 4_000, rotate(vec![vec![a], vec![b], vec![c]]), fate, lose_nothing);
            assert_eq!(vec![a], sim.fr.excluded_relayers(&sim.exit));
            let lapse = sim.verdict_ms.context("judged")? + FORWARD_VERDICT_TTL.as_millis() as u64;

            // Packets through it, lost, within 10 s of the lift, which has to drop them
            sim.run(4_000, lapse - 1_100, &healthy, fate, lose_nothing);
            for i in 0..5 {
                sim.send(lapse - 1_100 + i, &[a], None);
            }
            sim.run(lapse - 1_000, lapse, &healthy, fate, lose_nothing);
            assert_eq!(vec![a], sim.fr.excluded_relayers(&sim.exit));

            sim.fr.refresh_at(sim.at(lapse));
            assert!(sim.fr.excluded_relayers(&sim.exit).is_empty());
            assert!(
                !sim.fr
                    .state
                    .lock()
                    .records
                    .values()
                    .flatten()
                    .any(|(_, r)| r.relayers().any(|r| *r == a))
            );

            // Without the drop, those packets would be judged again at once
            sim.run(lapse + 10, lapse + 2_000, &healthy, fate, lose_nothing);
            assert!(sim.fr.excluded_relayers(&sim.exit).is_empty());
            Ok(())
        })?;
        Ok(())
    }

    #[test]
    #[serial_test::serial(routing_logs)]
    fn a_receipt_for_a_packet_sent_after_the_verdict_should_lift_it_early_and_a_relapse_skips_the_wait()
    -> anyhow::Result<()> {
        let [a, b, c] = [(); 3].map(|_| key());
        logged("forward verdict lifted", || {
            let mut sim = Sim::new();
            let route = rotate(vec![vec![a], vec![b], vec![c]]);
            sim.run(
                0,
                4_000,
                &route,
                |path| if path.contains(&a) { None } else { FAST },
                lose_nothing,
            );
            assert_eq!(vec![a], sim.fr.excluded_relayers(&sim.exit));

            // The fallback draw sent a packet through it after the verdict, and it got home.
            sim.send(4_000, &[a], FAST);
            sim.receipt(4_250, 0);
            assert!(sim.fr.excluded_relayers(&sim.exit).is_empty());

            // The packets from before the lift are not judged again, and it is judged without the wait when it
            // fails again: an overdue packet needs 500 ms and a receipt, nothing more.
            sim.run(4_250, 4_750, &route, |_| FAST, lose_nothing);
            assert!(sim.fr.excluded_relayers(&sim.exit).is_empty());
            sim.verdict_ms = None;
            sim.run(
                4_750,
                7_000,
                &route,
                |path| if path.contains(&a) { None } else { FAST },
                lose_nothing,
            );
            let relapse = sim.verdict_ms.context("judged again")?;
            assert!(relapse <= 4_750 + 1_000, "judged at {relapse} ms");
            Ok(())
        })?;
        Ok(())
    }

    /// A relayer that delays packets drawn just before its verdict must not lift it with their late delivery.
    #[test]
    fn a_late_packet_sent_after_the_verdict_should_not_lift_it() -> anyhow::Result<()> {
        let [a, b, c] = [(); 3].map(|_| key());
        let mut sim = Sim::new();
        let fate = |path: &[Key]| if path.contains(&a) { None } else { FAST };
        sim.run(0, 2_010, rotate(vec![vec![a], vec![b], vec![c]]), fate, lose_nothing);
        let verdict = sim.verdict_ms.context("judged")?;
        sim.send(verdict + 1, &[a], Some(3_000));
        sim.run(2_010, 7_000, rotate(vec![vec![b], vec![c]]), fate, lose_nothing);
        assert_eq!(vec![a], sim.fr.excluded_relayers(&sim.exit));
        Ok(())
    }

    /// Receipts 5 and 6 are overtaken by 7 and arrive within the grace, before the receipt that judges; or, with a
    /// coverage of 3, long after it.
    #[test]
    fn receipts_arriving_out_of_order_within_the_grace_should_judge_as_in_order() {
        let [a, b, c] = [(); 3].map(|_| key());
        let verdict = |late: Vec<(u32, u64)>, cover: usize| {
            let mut sim = Sim::new();
            sim.late = late;
            sim.cover = cover;
            sim.run(
                0,
                4_000,
                rotate(vec![vec![a], vec![b], vec![c]]),
                |path| if path.contains(&a) { None } else { FAST },
                lose_nothing,
            );
            (sim.verdict_ms, sim.fr.excluded_relayers(&sim.exit))
        };
        let in_order = verdict(vec![], 2);
        assert!(in_order.0.is_some());
        assert_eq!(in_order, verdict(vec![(5, 600), (6, 300)], 2));

        // Receipts that arrive long after the grace are still covered by the one that overtook them
        let stalled = verdict(vec![], 3);
        assert_eq!((Some(2_000), vec![a]), stalled);
        assert_eq!(stalled, verdict(vec![(5, 2_000), (6, 2_000)], 3));
    }

    #[test]
    fn a_receipt_still_missing_after_the_grace_should_set_the_floor_to_the_jump() {
        let fr = ForwardReceipts::default();
        let session = HoprPseudonym::random();
        let t0 = Instant::now();
        let at = |ms| t0 + Duration::from_millis(ms);
        for seq in 1..=3 {
            fr.receipts_at(session, seq, seq, &[], at(0));
        }
        fr.receipts_at(session, 6, 6, &[], at(100));
        assert_eq!(at(0), fr.state.lock().streams[&session].floor, "4 and 5 are awaited");
        fr.receipts_at(session, 7, 7, &[], at(700));
        assert_eq!(at(100), fr.state.lock().streams[&session].floor, "4 and 5 never came");
        assert!(fr.state.lock().streams[&session].missing.is_empty());
    }

    /// Two jumps whose missing receipts expire at the same arrival: what was sent before the second jump may have
    /// been receipted by the receipt lost there, so it is no evidence.
    #[test]
    fn the_floor_should_be_the_latest_lost_gap() {
        let [x, y, exit] = [(); 3].map(|_| key());
        let fr = ForwardReceipts::default();
        let session = HoprPseudonym::random();
        let t0 = Instant::now();
        let at = |ms| t0 + Duration::from_millis(ms);
        for seq in 1..=3 {
            fr.receipts_at(session, seq, seq, &[], at(0));
        }
        // 4 and 5 go missing at 100 ms, and 7 and 8 at 300 ms
        fr.receipts_at(session, 6, 6, &[], at(100));
        for tag in 1..=3 {
            fr.record_at(session, tag, &[x, exit], at(200));
        }
        fr.record_at(session, 99, &[y, exit], at(850));
        fr.receipts_at(session, 9, 9, &[], at(300));
        fr.receipts_at(session, 10, 10, &[99], at(900));

        let state = fr.state.lock();
        assert_eq!(at(300), state.streams[&session].floor);
        assert!(
            state.suspected.is_empty(),
            "packets sent before the floor are no evidence"
        );
    }

    /// 6 overtakes, and 5 arrives late and covers 4 as well: nothing is lost.
    #[test]
    fn a_late_receipt_should_cover_the_intervals_it_repeats() {
        let fr = ForwardReceipts::default();
        let session = HoprPseudonym::random();
        let t0 = Instant::now();
        let at = |ms| t0 + Duration::from_millis(ms);
        for seq in 1..=3 {
            fr.receipts_at(session, seq, seq, &[], at(0));
        }
        fr.receipts_at(session, 6, 6, &[], at(100));
        fr.receipts_at(session, 5, 4, &[], at(200));
        fr.receipts_at(session, 7, 7, &[], at(800));
        let state = fr.state.lock();
        assert_eq!(at(0), state.streams[&session].floor);
        assert!(state.streams[&session].missing.is_empty());
    }

    /// Every receipt repeats the two intervals before it, so up to two lost receipts in a row lose nothing.
    #[test]
    fn as_many_lost_receipts_as_a_receipt_covers_should_lose_nothing() {
        let [a, b, c] = [(); 3].map(|_| key());
        let verdict_with = |lost: &'static [u32]| {
            let mut sim = Sim::new();
            sim.cover = 3;
            sim.run(
                0,
                4_000,
                rotate(vec![vec![a], vec![b], vec![c]]),
                |path| if path.contains(&a) { None } else { FAST },
                |seq| lost.contains(&seq),
            );
            (sim.verdict_ms, sim.fr.excluded_relayers(&sim.exit))
        };
        let expected = verdict_with(&[]);
        assert_eq!((Some(2_000), vec![a]), expected);
        assert_eq!(expected, verdict_with(&[6, 7]));
    }

    /// More receipts lost than a receipt covers hold judgement, and then floor at the jump.
    #[test]
    fn more_lost_receipts_than_a_receipt_covers_should_floor_at_the_jump() {
        let [a, b, c] = [(); 3].map(|_| key());
        let mut sim = Sim::new();
        sim.cover = 3;
        let route = rotate(vec![vec![a], vec![b], vec![c]]);
        let fate = |path: &[Key]| if path.contains(&a) { None } else { FAST };
        let lost = |seq: u32| (6..=9).contains(&seq);
        // Receipt 10 at 2500 ms is the first to arrive after the four, and covers 8 to 10: 6 and 7 are missing
        sim.run(0, 2_510, &route, fate, lost);
        let missing: Vec<u32> = sim.fr.state.lock().streams[&sim.pseudonym]
            .missing
            .iter()
            .map(|(seq, _)| *seq)
            .collect();
        assert_eq!(vec![6, 7], missing);
        sim.run(2_510, 3_600, &route, fate, lost);
        assert_eq!(sim.at(2_500), sim.fr.state.lock().streams[&sim.pseudonym].floor);
    }

    /// A gap of 16 is held, a larger one floors at once.
    #[test]
    fn a_gap_of_sixteen_should_be_held_and_a_larger_one_should_floor() {
        let fr = ForwardReceipts::default();
        let session = HoprPseudonym::random();
        let t0 = Instant::now();
        let at = |ms| t0 + Duration::from_millis(ms);
        for seq in 1..=9 {
            fr.receipts_at(session, seq, seq, &[], at(0));
        }
        fr.receipts_at(session, 40, 26, &[], at(100));
        {
            let state = fr.state.lock();
            assert_eq!(
                (16, at(0)),
                (state.streams[&session].missing.len(), state.streams[&session].floor)
            );
        }
        fr.receipts_at(session, 60, 60, &[], at(200));
        assert_eq!(at(200), fr.state.lock().streams[&session].floor);
    }

    /// A seq of `u32::MAX` must not overflow the next receipt.
    #[test]
    fn a_receipt_after_the_largest_seq_should_not_overflow() {
        let fr = ForwardReceipts::default();
        let session = HoprPseudonym::random();
        let t0 = Instant::now();
        fr.receipts_at(session, u32::MAX, u32::MAX, &[], t0);
        fr.receipts_at(session, u32::MAX, 1, &[], t0);
        assert!(fr.state.lock().streams[&session].missing.is_empty());
    }

    #[test]
    fn a_receipt_should_leave_only_the_seqs_before_its_first_seq_missing() {
        let fr = ForwardReceipts::default();
        let session = HoprPseudonym::random();
        let t0 = Instant::now();
        for seq in 1..=3 {
            fr.receipts_at(session, seq, seq, &[], t0);
        }
        fr.receipts_at(session, 9, 7, &[], t0);
        let missing = |fr: &ForwardReceipts| -> Vec<u32> {
            fr.state.lock().streams[&session]
                .missing
                .iter()
                .map(|(seq, _)| *seq)
                .collect()
        };
        assert_eq!(vec![4, 5, 6], missing(&fr));
        fr.receipts_at(session, 5, 4, &[], t0);
        assert_eq!(vec![6], missing(&fr), "5 covers 4 and 5");

        // A gap beyond what is held is a loss at once
        fr.receipts_at(session, 40, 40, &[], t0 + Duration::from_millis(10));
        assert_eq!(t0 + Duration::from_millis(10), fr.state.lock().streams[&session].floor);
    }

    #[test]
    fn a_receipt_three_places_late_should_not_reset_the_stream() {
        let fr = ForwardReceipts::default();
        let session = HoprPseudonym::random();
        let t0 = Instant::now();
        for seq in 1..=10 {
            fr.receipts_at(session, seq, seq, &[], t0);
        }
        fr.receipts_at(session, 7, 7, &[], t0 + Duration::from_millis(10));
        assert_eq!((10, t0), {
            let state = fr.state.lock();
            (state.streams[&session].last_seq, state.streams[&session].floor)
        });
    }

    /// A stream with nothing receipted yet cannot tell a timely packet from a late one.
    #[test]
    fn a_delivery_on_a_stream_without_a_watermark_should_not_lift_a_verdict() -> anyhow::Result<()> {
        let [a, b, c] = [(); 3].map(|_| key());
        let mut sim = Sim::new();
        sim.run(
            0,
            2_010,
            rotate(vec![vec![a], vec![b], vec![c]]),
            |path| if path.contains(&a) { None } else { FAST },
            lose_nothing,
        );
        let verdict = sim.verdict_ms.context("judged")?;
        sim.send(verdict + 1, &[a], FAST);
        // The pseudonym is reused: its new stream has no watermark
        sim.fr
            .receipts_at(sim.pseudonym, 1, 1, &[sim.next_tag - 1], sim.at(verdict + 300));
        assert_eq!(vec![a], sim.fr.excluded_relayers(&sim.exit));
        Ok(())
    }

    #[test]
    fn records_should_be_bounded_and_expire() {
        let [a, exit] = [(); 2].map(|_| key());
        let fr = ForwardReceipts::default();
        let session = HoprPseudonym::random();
        let t0 = Instant::now();
        for tag in 0..(MAX_RECORDS as u32 + 5_000) {
            fr.record_at(session, tag, &[a, exit], t0);
        }
        {
            let state = fr.state.lock();
            assert_eq!(MAX_RECORDS, state.count);
            assert!(state.order.len() <= 2 * MAX_RECORDS);
            assert!(!state.records[&session].contains_key(&0), "the oldest goes first");
        }
        fr.refresh_at(t0 + RECORD_TTL + Duration::from_secs(1));
        assert!(fr.state.lock().records.is_empty());
        assert!(fr.state.lock().order.is_empty());
    }

    #[test]
    fn a_reused_pseudonym_should_reset_its_stream() {
        let [a, exit] = [(); 2].map(|_| key());
        let fr = ForwardReceipts::default();
        let session = HoprPseudonym::random();
        let t0 = Instant::now();
        for seq in 1..=5 {
            fr.receipts_at(session, seq, seq, &[], t0 + Duration::from_millis(250 * seq as u64));
        }
        fr.record_at(session, 1, &[a, exit], t0);
        fr.receipts_at(session, 6, 6, &[1], t0 + Duration::from_secs(2));
        assert!(fr.state.lock().streams[&session].watermark.is_some());

        let again = t0 + Duration::from_secs(3);
        fr.receipts_at(session, 1, 1, &[], again);
        let state = fr.state.lock();
        let stream = &state.streams[&session];
        assert_eq!((1, None, again), (stream.last_seq, stream.watermark, stream.floor));
    }

    #[test]
    fn a_stream_a_new_session_would_start_with_should_reset() {
        let fr = ForwardReceipts::default();
        let session = HoprPseudonym::random();
        let t0 = Instant::now();
        for seq in 1..=10 {
            fr.receipts_at(session, seq, seq, &[], t0);
        }
        fr.receipts_at(session, 9, 9, &[], t0);
        assert_eq!(
            10,
            fr.state.lock().streams[&session].last_seq,
            "a late receipt is no reset"
        );
        fr.receipts_at(session, NEW_SESSION_SEQ_MAX, NEW_SESSION_SEQ_MAX, &[], t0);
        assert_eq!(NEW_SESSION_SEQ_MAX, fr.state.lock().streams[&session].last_seq);
    }
}
