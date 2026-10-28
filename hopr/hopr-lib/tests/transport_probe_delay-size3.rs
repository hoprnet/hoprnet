//! Simulation: are quality probes delayed when the node's egress line is saturated?
//!
//! A 0-hop probe and a bulk session packet leave a node through the same stages: the
//! `unresolved_routing_msg_tx` buffer, the path-resolution stage, the order-preserving encode stage,
//! the shared mixer, and the per-peer wire channel. Probe latency measured under load can therefore
//! reflect egress queueing rather than path quality, which is a plausible contributor to the tunnel
//! stalls seen under heavy usage. This test measures the effect instead of assuming it.
//!
//! # Phases
//!
//! 1. `idle`: probes to the direct neighbor before any session traffic exists.
//! 2. `saturated`: the same probes while a bulk session (`src→relay→dst`) writes back-to-back, which keeps the source's
//!    egress queues full.
//! 3. `recovery`: the same probes after the flood stops and the pipeline has drained.
//!
//! [`HoprTransport::ping`] reports half the measured round trip, so every latency below is one-way.
//! The cover-traffic prober keeps running for the whole simulation, exactly as it does in production;
//! this test does not try to silence it.
//!
//! # What is asserted
//!
//! Probes have to keep getting through while the egress is saturated: every probe taken under load
//! must be answered inside the probe layer's own timeout, since a probe that stalls there is what
//! degrades the graph parameters path selection reads. On top of that no probe anywhere may hang past
//! its timeout plus a small grace, and both probe latency and the probe-derived graph latency have to
//! return to a loose multiple of the idle baseline once the flood stops.
//!
//! The delay itself is reported, never asserted: the saturated latency is the measurement this test
//! exists to produce, not a failure.
//!
//! Saturation here floods the node's shared internal queues. Link-level shaping (NIC or WAN) is not
//! simulated; the shaped reproduction belongs to hoprnet/hopr-integration-tests#75.
//!
//! # Running
//!
//! ```sh
//! cargo nextest run -p hopr-lib --features testing,session-client \
//!     --test transport_probe_delay-size3 -j 1 -- --nocapture
//! ```
//!
//! [`HoprTransport::ping`]: hopr_lib::exports::transport::HoprTransport::ping

#![cfg(feature = "session-client")]

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use anyhow::Context;
use futures::future::try_join_all;
use hopr_chain_connector::blokli_client::BlokliQueryClient;
use hopr_lib::{
    HoprSession, SessionCapabilities,
    api::{
        OffchainPublicKey,
        graph::traits::{EdgeLinkObservable, EdgeObservableRead},
        node::HasTransportApi,
        types::primitive::prelude::HoprBalance,
    },
    testing::{
        fixtures::{
            Probe, STRESS_WIN_PROB, TEST_GLOBAL_TIMEOUT, chain_propagation_delay, stress_cluster_fixture,
            wait_for_convergence,
        },
        hopr::{ChannelGuard, TestedHopr},
    },
};
use rstest::*;
use serial_test::serial;
use tokio::{io::AsyncWriteExt, time::Instant};

/// Funding per channel on the measured path.
///
/// A saturated flood spends one ticket per forwarded packet on each hop. 4 M wxHOPR per channel
/// covers several times [`FLOOD_BYTE_BUDGET`] at the stress network's winning probability, so the
/// channels cannot run dry in the middle of the measurement.
const FUNDING_AMOUNT: &str = "4000000 wxHOPR";

/// Payload per bulk write. The session fragments it into frame-sized packets.
const FLOOD_PAYLOAD_SIZE: usize = 16 * 1024;

/// Upper bound on the flooded volume, keeping ticket spend well inside the channel funding.
const FLOOD_BYTE_BUDGET: u64 = 256 * 1024 * 1024;

/// How long the flood runs. The saturated phase ends with it.
const SATURATION_WINDOW: Duration = Duration::from_secs(15);

/// Minimum volume that counts as a saturated phase.
const MIN_FLOOD_BYTES: u64 = 1024 * 1024;

/// Fewest probes the saturated phase must produce for its liveness verdict to mean anything.
const MIN_SATURATION_PROBES: usize = 3;

/// Gap between probes while the egress is saturated.
const SATURATION_PROBE_PERIOD: Duration = Duration::from_millis(200);

/// Probes taken before the flood starts.
const IDLE_PROBE_COUNT: usize = 20;

/// Gap between idle probes.
const IDLE_PROBE_PERIOD: Duration = Duration::from_millis(100);

/// Time allowed past the probe layer's own timeout before a probe counts as hung.
const PROBE_HANG_GRACE: Duration = Duration::from_secs(2);

/// Time given to the pipeline to drain after the flood stops.
const RECOVERY_SETTLE: Duration = Duration::from_secs(3);

/// Probes taken after the flood stops.
const RECOVERY_PROBE_COUNT: usize = 10;

/// Gap between recovery probes.
const RECOVERY_PROBE_PERIOD: Duration = Duration::from_millis(200);

/// Multiplier on the idle median that the recovery median must stay under.
const RECOVERY_FACTOR: u32 = 10;

/// Fixed allowance added to the recovery bound, so sub-millisecond idle baselines stay usable.
const RECOVERY_GRACE: Duration = Duration::from_millis(500);

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
async fn probe_delay_should_be_measured_when_egress_is_saturated() -> anyhow::Result<()> {
    let cluster = stress_cluster_fixture(STRESS_WIN_PROB, 3);
    let nodes: Vec<&TestedHopr> = cluster.iter().collect();
    let (src, relay, dst) = (nodes[0], nodes[1], nodes[2]);

    // Open all four directed channels of the 1-hop path, as the other 1-hop session tests do.
    let funding = FUNDING_AMOUNT
        .parse::<HoprBalance>()
        .context("parsing the channel funding amount")?;
    let channels = try_join_all([
        ChannelGuard::open_channel_between_nodes(src.instance.clone(), relay.instance.clone(), funding),
        ChannelGuard::open_channel_between_nodes(relay.instance.clone(), dst.instance.clone(), funding),
        ChannelGuard::open_channel_between_nodes(dst.instance.clone(), relay.instance.clone(), funding),
        ChannelGuard::open_channel_between_nodes(relay.instance.clone(), src.instance.clone(), funding),
    ])
    .await
    .context("opening channels on the src→relay→dst path")?;

    let chain_info = cluster
        .chain_client
        .query_chain_info()
        .await
        .context("querying chain info")?;
    cluster
        .wait_for_channel_graph(src, channels.len(), chain_propagation_delay(&chain_info) * 12)
        .await
        .context("waiting for the channel graph to converge")?;

    // The probe target is the direct neighbor that also carries the flood, so probe and bulk packets
    // share the same egress queues.
    let relay_key = hopr_lib::peer_id_to_offchain_key(&relay.peer_id()).context("relay peer id to offchain key")?;
    let probe_limit = src.config().protocol.probe.timeout + PROBE_HANG_GRACE;

    // The graph edge has to exist before `ping` can return observations, so wait for the first answer
    // instead of recording a spurious failure in the idle baseline.
    wait_for_convergence("first probe response", || async {
        Ok(match probe_once(src, &relay_key, probe_limit).await {
            ProbeOutcome::Responded(_) => Probe::Ready(()),
            other => Probe::Pending(format!("{other:?}")),
        })
    })
    .await
    .context("the probe layer should answer a 0-hop ping before the simulation starts")?;

    // ── 1. Idle baseline ─────────────────────────────────────────────────────
    let mut idle = ProbePhaseStats::new("idle");
    for _ in 0..IDLE_PROBE_COUNT {
        tokio::time::sleep(IDLE_PROBE_PERIOD).await;
        idle.record(probe_once(src, &relay_key, probe_limit).await);
    }
    let idle_graph_latency = observed_latency(src, &relay_key);

    // ── 2. Saturation ────────────────────────────────────────────────────────
    // No SURB balancer: the workload has to be plain bulk data, otherwise SURB keep-alive traffic
    // would be part of whatever delay this measures.
    let path = [src, relay, dst];
    let session = cluster
        .create_session_with(&path, SessionCapabilities::default(), None)
        .await
        .context("opening the bulk session")?;

    // The flood clears the flag when it stops, so the saturated phase can never outlive the traffic
    // it is measuring, whether it ends at the deadline, at the byte budget, or on an error.
    let flood_active = Arc::new(AtomicBool::new(true));
    let saturation_started = Instant::now();
    let saturation_deadline = saturation_started + SATURATION_WINDOW;
    let flood = tokio::spawn({
        let flood_active = Arc::clone(&flood_active);
        async move {
            let result = flood_session(session, saturation_deadline).await;
            flood_active.store(false, Ordering::Relaxed);
            result
        }
    });

    let mut saturated = ProbePhaseStats::new("saturated");
    while Instant::now() < saturation_deadline && flood_active.load(Ordering::Relaxed) {
        saturated.record(probe_once(src, &relay_key, probe_limit).await);
        tokio::time::sleep(SATURATION_PROBE_PERIOD).await;
    }

    let flood_duration = saturation_started.elapsed();
    let flooded_bytes = flood
        .await
        .context("the flood task panicked")?
        .context("the bulk flood failed")?;
    let saturated_graph_latency = observed_latency(src, &relay_key);

    // ── 3. Recovery ──────────────────────────────────────────────────────────
    tokio::time::sleep(RECOVERY_SETTLE).await;

    let mut recovery = ProbePhaseStats::new("recovery");
    for _ in 0..RECOVERY_PROBE_COUNT {
        tokio::time::sleep(RECOVERY_PROBE_PERIOD).await;
        recovery.record(probe_once(src, &relay_key, probe_limit).await);
    }
    let recovery_graph_latency = observed_latency(src, &relay_key);

    ProbeDelayReport {
        idle: &idle,
        saturated: &saturated,
        recovery: &recovery,
        flooded_bytes,
        flood_duration,
        idle_graph_latency,
        saturated_graph_latency,
        recovery_graph_latency,
    }
    .print();

    // Liveness: a probe that hangs is a broken probe layer, not a delay. The pinger has its own
    // timeout, so exceeding timeout plus grace means the caller never got an answer at all.
    let hangs = idle.hangs + saturated.hangs + recovery.hangs;
    anyhow::ensure!(
        hangs == 0,
        "{hangs} probe(s) did not resolve within {probe_limit:?} (probe timeout plus grace)"
    );

    // A flooded run that wrote almost nothing did not test saturation, so its numbers mean nothing.
    anyhow::ensure!(
        flooded_bytes >= MIN_FLOOD_BYTES,
        "the flood wrote only {flooded_bytes} bytes in {flood_duration:?}, which is not saturation"
    );
    anyhow::ensure!(
        saturated.attempts() >= MIN_SATURATION_PROBES,
        "the saturated phase recorded only {} probe(s), too few to judge whether probes get through",
        saturated.attempts()
    );

    // Probes get through while the egress is saturated, which is what keeps the graph parameters path
    // selection reads from degrading into a stall. A probe the pinger itself gave up on is a stalled
    // probe, so neither a failure nor a hang is tolerated here; only the magnitude of the delay is
    // left to the report.
    anyhow::ensure!(
        saturated.failures.is_empty() && saturated.hangs == 0,
        "{} of {} probes did not get through while the egress was saturated: {} failed, {} hung{}",
        saturated.failures.len() + saturated.hangs,
        saturated.attempts(),
        saturated.failures.len(),
        saturated.hangs,
        saturated
            .failures
            .first()
            .map(|first| format!(" (first failure: {first})"))
            .unwrap_or_default(),
    );

    // Recovery: once the flood stops the pipeline drains and probes return to roughly their idle
    // latency. The bound is deliberately loose, because a probe may still queue behind the tail of
    // the flood; only a permanent stall (the tunnel-stall symptom) fails here.
    let idle_median = idle
        .median()
        .context("the idle phase needs at least one successful probe")?;
    let recovery_median = recovery
        .median()
        .context("no probe succeeded after the flood stopped, so the probe layer did not recover from saturation")?;
    let recovery_bound = idle_median * RECOVERY_FACTOR + RECOVERY_GRACE;
    anyhow::ensure!(
        recovery_median <= recovery_bound,
        "probe latency did not recover: median {recovery_median:?} after the flood exceeds {recovery_bound:?} (idle \
         median {idle_median:?} × {RECOVERY_FACTOR} + {RECOVERY_GRACE:?}), with {} of {} recovery probes failing",
        recovery.failures.len(),
        recovery.attempts(),
    );

    // The graph parameters path selection consumes come from these same probes, so they have to
    // recover as well: an edge left degraded by the flood keeps steering traffic off a healthy route.
    let idle_graph =
        idle_graph_latency.context("the graph reported no latency for the probed peer before the flood")?;
    let recovery_graph =
        recovery_graph_latency.context("the graph reported no latency for the probed peer after the flood")?;
    anyhow::ensure!(
        recovery_graph <= idle_graph * RECOVERY_FACTOR + RECOVERY_GRACE,
        "the probe-derived graph latency stayed degraded after the flood: {recovery_graph:?} against an idle baseline \
         of {idle_graph:?} (bound {RECOVERY_FACTOR}× the baseline plus {RECOVERY_GRACE:?})"
    );

    // Saturated latencies are intentionally not asserted: their magnitude is a finding, and this
    // test would fail every time the expected delay appears if it enforced a threshold.

    try_join_all(
        channels
            .into_iter()
            .map(|guard| async move { guard.try_close_channels_all_channels().await }),
    )
    .await
    .context("closing the channels of the measured path")?;

    Ok(())
}

/// One probe outcome, as the simulation records it.
#[derive(Debug)]
enum ProbeOutcome {
    /// The probe round trip completed. The transport reports half the measured RTT, so this is a
    /// one-way latency.
    Responded(Duration),
    /// The probe layer reported a failure, for example its own timeout expiring before the pong.
    Failed(String),
    /// The probe future did not resolve within the probe timeout plus grace.
    Hung,
}

/// Sends one 0-hop probe to `peer` and bounds how long it may take to resolve.
async fn probe_once(node: &TestedHopr, peer: &OffchainPublicKey, limit: Duration) -> ProbeOutcome {
    match tokio::time::timeout(limit, node.inner().transport().ping(peer)).await {
        Ok(Ok((latency, _observations))) => ProbeOutcome::Responded(latency),
        Ok(Err(error)) => ProbeOutcome::Failed(error.to_string()),
        Err(_) => ProbeOutcome::Hung,
    }
}

/// Probe latencies recorded for one phase of the simulation.
#[derive(Debug)]
struct ProbePhaseStats {
    name: &'static str,
    latencies: Vec<Duration>,
    failures: Vec<String>,
    hangs: usize,
}

impl ProbePhaseStats {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            latencies: Vec::new(),
            failures: Vec::new(),
            hangs: 0,
        }
    }

    fn record(&mut self, outcome: ProbeOutcome) {
        match outcome {
            ProbeOutcome::Responded(latency) => self.latencies.push(latency),
            ProbeOutcome::Failed(reason) => self.failures.push(reason),
            ProbeOutcome::Hung => self.hangs += 1,
        }
    }

    fn attempts(&self) -> usize {
        self.latencies.len() + self.failures.len() + self.hangs
    }

    fn mean(&self) -> Option<Duration> {
        if self.latencies.is_empty() {
            return None;
        }

        Some(self.latencies.iter().sum::<Duration>() / self.latencies.len() as u32)
    }

    fn median(&self) -> Option<Duration> {
        self.percentile(0.5)
    }

    /// Nearest-rank percentile over the successful probes.
    fn percentile(&self, quantile: f64) -> Option<Duration> {
        let mut sorted = self.latencies.clone();
        sorted.sort_unstable();

        let rank = ((sorted.len() as f64 - 1.0) * quantile).round() as usize;

        sorted.get(rank).copied()
    }

    fn max(&self) -> Option<Duration> {
        self.latencies.iter().copied().max()
    }

    /// Prints this phase as one table row, in the style of `StressReport::print_series`.
    fn print_row(&self) {
        println!(
            "  {:<10} {:>7} {:>6} {:>5} {:>5} {:>10} {:>10} {:>10} {:>10}",
            self.name,
            self.attempts(),
            self.latencies.len(),
            self.failures.len(),
            self.hangs,
            format_duration(self.mean()),
            format_duration(self.median()),
            format_duration(self.percentile(0.95)),
            format_duration(self.max()),
        );
    }
}

/// Everything the simulation measured, ready to be printed.
struct ProbeDelayReport<'a> {
    idle: &'a ProbePhaseStats,
    saturated: &'a ProbePhaseStats,
    recovery: &'a ProbePhaseStats,
    flooded_bytes: u64,
    flood_duration: Duration,
    idle_graph_latency: Option<Duration>,
    saturated_graph_latency: Option<Duration>,
    recovery_graph_latency: Option<Duration>,
}

impl ProbeDelayReport<'_> {
    /// Prints the per-phase table, the saturated-to-idle ratio, and the graph latency the probe layer
    /// fed back into the network graph.
    fn print(&self) {
        println!("\n=== quality probes under saturated egress · 3-node cluster · src→relay 0-hop ===");
        println!(
            "  {:<10} {:>7} {:>6} {:>5} {:>5} {:>10} {:>10} {:>10} {:>10}",
            "phase", "probes", "ok", "fail", "hang", "mean", "p50", "p95", "max"
        );
        println!("  {}", "─".repeat(83));

        for phase in [self.idle, self.saturated, self.recovery] {
            phase.print_row();
        }

        println!("  {}", "─".repeat(83));
        println!(
            "  Ratio saturated/idle: mean {}  p50 {}",
            format_ratio(self.saturated.mean(), self.idle.mean()),
            format_ratio(self.saturated.median(), self.idle.median()),
        );
        println!(
            "  Flood: {} written over {:.2}s into src→relay→dst",
            format_mib(self.flooded_bytes),
            self.flood_duration.as_secs_f64(),
        );
        println!(
            "  Graph latency (probe EMA that path selection uses): idle {}  saturated {}  recovered {}",
            format_duration(self.idle_graph_latency),
            format_duration(self.saturated_graph_latency),
            format_duration(self.recovery_graph_latency),
        );
        println!("  Note: the transport reports half the measured RTT, so all latencies above are one-way.");

        for phase in [self.idle, self.saturated, self.recovery] {
            if let Some(first_failure) = phase.failures.first() {
                println!("  {} first failure: {first_failure}", phase.name);
            }
        }
    }
}

/// Latency the network graph currently reports for `peer`, the probe-derived value path selection
/// consumes, as opposed to the individual samples recorded above.
fn observed_latency(node: &TestedHopr, peer: &OffchainPublicKey) -> Option<Duration> {
    let observations = node.inner().transport().network_peer_observations(peer)?;

    observations.immediate_qos()?.average_latency()
}

/// Writes bulk payloads into `session` back to back until `deadline` or [`FLOOD_BYTE_BUDGET`].
///
/// Back-to-back `write_all` plus `flush` keeps the node's outgoing queues full, which is the
/// condition whose effect on probes this simulation measures. Returns the bytes written.
async fn flood_session(mut session: HoprSession, deadline: Instant) -> anyhow::Result<u64> {
    let payload: Vec<u8> = (0..FLOOD_PAYLOAD_SIZE).map(|index| (index % 256) as u8).collect();
    let mut written = 0u64;

    while Instant::now() < deadline && written < FLOOD_BYTE_BUDGET {
        session
            .write_all(&payload)
            .await
            .context("writing a bulk session payload")?;
        session.flush().await.context("flushing a bulk session payload")?;
        written += payload.len() as u64;
    }

    Ok(written)
}

fn format_duration(value: Option<Duration>) -> String {
    value.map_or_else(|| "n/a".to_owned(), |duration| format!("{duration:?}"))
}

fn format_ratio(numerator: Option<Duration>, denominator: Option<Duration>) -> String {
    match (numerator, denominator) {
        (Some(numerator), Some(denominator)) if !denominator.is_zero() => {
            format!("{:.2}×", numerator.as_secs_f64() / denominator.as_secs_f64())
        }
        _ => "n/a".to_owned(),
    }
}

/// Formats a byte count as MiB, so the flood volume is readable without a formatting crate.
fn format_mib(bytes: u64) -> String {
    format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
}
