//! Local-cluster throughput stress harness.
//!
//! Drives a running [`ClusterGuard`] with bulk 1-hop traffic across random
//! `src → relay → dst` triples and records a per-window throughput
//! time-series.  The caller can observe whether delivery rate climbs, holds
//! steady, or degrades under sustained saturation.
//!
//! The module is intentionally *not* a criterion benchmark: one long sustained
//! transfer (configurable in bytes) is far more representative of real network
//! behaviour than many repeated micro-iterations.
//!
//! # Profiling
//!
//! Build with the **`profiling`** feature to get an in-process flame graph
//! written to a caller-supplied path:
//!
//! ```sh
//! # macOS or Linux — in-process SVG (no sudo required):
//! cargo run --profile profiling \
//!     --features testing,profiling \
//!     --example stress_cluster \
//!     -- --nodes 5 --mb 100 --out flame.svg
//!
//! # Linux — external perf/flamegraph (also works without the profiling feature):
//! cargo flamegraph --profile profiling \
//!     --features testing \
//!     --example stress_cluster \
//!     -- --nodes 5 --mb 100
//! ```
//!
//! Without the feature every `ProfilerGuard` method is a zero-cost no-op and
//! the module compiles unchanged.

use std::{
    ops::RangeInclusive,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use anyhow::Context;
use futures::future::try_join_all;
use hopr_chain_connector::blokli_client::BlokliQueryClient;
use rand::{RngExt, SeedableRng, rngs::StdRng, seq::SliceRandom};
use tokio::io::AsyncWriteExt;

use super::{
    fixtures::{ClusterGuard, chain_propagation_delay},
    hopr::ChannelGuard,
};
use crate::api::types::primitive::prelude::HoprBalance;

// ── Throughput reporting ─────────────────────────────────────────────────────

/// Throughput measurement for a single sample window.
#[derive(Debug, Clone)]
pub struct ThroughputSample {
    /// Wall time since the workload started.
    pub elapsed: Duration,
    /// Bytes written into the send pipeline during this window.
    pub window_bytes: u64,
    /// Send throughput for this window in MB/s.
    pub mbps: f64,
    /// Bytes confirmed received by destination EchoServers during this window.
    pub recv_window_bytes: u64,
    /// Receive throughput for this window in MB/s.
    pub recv_mbps: f64,
}

/// Result of a completed stress run.
#[derive(Debug, Clone)]
pub struct StressReport {
    /// Per-window throughput samples, in chronological order.
    pub samples: Vec<ThroughputSample>,
    /// Total bytes written into the forward pipeline (src→relay→dst) across all sessions.
    pub total_bytes_delivered: u64,
    /// Total bytes confirmed received by destination EchoServers.
    pub total_bytes_received: u64,
    /// Wall-clock duration of the workload phase, excluding channel/session setup.
    pub duration: Duration,
    /// Mean send throughput in MB/s averaged across all sample windows.
    pub mean_mbps: f64,
    /// Mean receive throughput in MB/s averaged across all sample windows.
    pub mean_recv_mbps: f64,
}

impl StressReport {
    /// Prints the per-window throughput series and a detailed summary to stdout.
    pub fn print_series(&self) {
        println!(
            "\n  {:>8}  {:>12}  {:>12}  {:>10}",
            "time(s)", "sent MB/s", "recv MB/s", "in-flight"
        );
        println!("  {}", "─".repeat(52));
        let mut cum_sent = 0u64;
        let mut cum_recv = 0u64;
        for s in &self.samples {
            cum_sent += s.window_bytes;
            cum_recv += s.recv_window_bytes;
            let in_flight = cum_sent.saturating_sub(cum_recv);
            println!(
                "  {:>8.2}  {:>12.3}  {:>12.3}  {:>10}",
                s.elapsed.as_secs_f64(),
                s.mbps,
                s.recv_mbps,
                bytesize::ByteSize(in_flight),
            );
        }
        println!("  {}", "─".repeat(52));

        // Derived stats
        let in_flight = self.total_bytes_delivered.saturating_sub(self.total_bytes_received);
        let loss_pct = if self.total_bytes_delivered > 0 {
            (in_flight as f64 / self.total_bytes_delivered as f64) * 100.0
        } else {
            0.0
        };

        // Peak throughput (highest non-zero window)
        let peak_send = self.samples.iter().map(|s| s.mbps).fold(0.0f64, f64::max);
        let peak_recv = self.samples.iter().map(|s| s.recv_mbps).fold(0.0f64, f64::max);

        println!(
            "  Sent:  {} in {:.2}s — mean {:.3} MB/s  peak {:.3} MB/s",
            bytesize::ByteSize(self.total_bytes_delivered),
            self.duration.as_secs_f64(),
            self.mean_mbps,
            peak_send,
        );
        println!(
            "  Recv:  {} — mean {:.3} MB/s  peak {:.3} MB/s",
            bytesize::ByteSize(self.total_bytes_received),
            self.mean_recv_mbps,
            peak_recv,
        );
        println!(
            "  Loss:  {} ({:.2}%)  [packets dropped or still in pipeline at drain timeout]",
            bytesize::ByteSize(in_flight),
            loss_pct,
        );
    }
}

// ── Configuration ────────────────────────────────────────────────────────────

/// Configuration for [`run_stress`].
#[derive(Debug, Clone)]
pub struct StressConfig {
    /// Total bytes to write into the pipeline before stopping.
    ///
    /// Counted when each `write_all` + `flush` cycle completes on the sender side.
    /// Measures the forward path (src→relay→dst); the return path is not read back
    /// because sessions do not provision SURBs for bidirectional flow.
    /// Delivery at the destination is tracked via [`ClusterGuard::echo_received`].
    ///
    /// [`ClusterGuard::echo_received`]: super::fixtures::ClusterGuard::echo_received
    pub total_bytes: u64,

    /// Number of concurrent 1-hop sessions to maintain.
    ///
    /// Each session is a distinct randomly-chosen `src → relay → dst` triple.
    /// Must not exceed the number of valid distinct triples for the cluster size
    /// (`n * (n-1) * (n-2)`).
    pub routes: usize,

    /// Inclusive byte-size range for each individual write.
    ///
    /// A uniform-random size in this range is chosen per message, producing a
    /// realistic mix of small and large transfers.
    pub msg_size_range: RangeInclusive<usize>,

    /// How often a throughput sample is recorded.
    pub sample_interval: Duration,

    /// Seed for route selection, giving reproducible traffic patterns across runs.
    pub seed: u64,
}

impl Default for StressConfig {
    fn default() -> Self {
        Self {
            total_bytes: 100 * 1024 * 1024, // 100 MB
            routes: 4,
            msg_size_range: 4096..=65536, // 4 KB – 64 KB
            sample_interval: Duration::from_millis(500),
            seed: 42,
        }
    }
}

// ── Profiler guard ────────────────────────────────────────────────────────────

/// Guard that holds an active CPU profiler session.
///
/// Enabled by the **`profiling`** feature.  When the feature is absent every
/// method compiles away to a zero-cost no-op.
#[cfg(feature = "profiling")]
pub struct ProfilerGuard {
    inner: pprof::ProfilerGuard<'static>,
}

/// Zero-cost stub used when the `profiling` feature is absent.
#[cfg(not(feature = "profiling"))]
pub struct ProfilerGuard;

#[cfg(feature = "profiling")]
impl ProfilerGuard {
    /// Start the CPU profiler, sampling at `frequency` Hz.
    ///
    /// Returns an error if the profiler cannot be initialised (e.g. a signal
    /// handler conflict with another library).
    pub fn start(frequency: i32) -> anyhow::Result<Self> {
        let inner = pprof::ProfilerGuardBuilder::default()
            .frequency(frequency)
            .blocklist(&["libc", "libgcc", "pthread"])
            .build()
            .context("building pprof profiler guard")?;
        Ok(Self { inner })
    }

    /// Capture the current profile and write an SVG flame graph to `path`.
    pub fn write_flamegraph(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let report = self.inner.report().build().context("building profiler report")?;
        let file =
            std::fs::File::create(path).with_context(|| format!("creating flame graph at {}", path.display()))?;
        report.flamegraph(file).context("writing flame graph SVG")?;
        Ok(())
    }
}

#[cfg(not(feature = "profiling"))]
impl ProfilerGuard {
    /// No-op: returns immediately when the `profiling` feature is disabled.
    pub fn start(_frequency: i32) -> anyhow::Result<Self> {
        Ok(Self)
    }

    /// No-op: returns immediately when the `profiling` feature is disabled.
    pub fn write_flamegraph(&self, _path: &std::path::Path) -> anyhow::Result<()> {
        Ok(())
    }
}

// ── Core stress runner ────────────────────────────────────────────────────────

/// Run the throughput stress workload against `cluster`.
///
/// The function:
/// 1. Opens a full directed channel mesh (every ordered pair of cluster nodes).
/// 2. Selects `cfg.routes` random distinct 1-hop triples `[src, relay, dst]`.
/// 3. Establishes one long-lived [`HoprSession`] per triple.
/// 4. Spawns one worker task per session; each task streams random-sized writes through the session and reads the echo
///    back to confirm delivery.
/// 5. Records per-window throughput samples from a background task.
/// 6. Stops once `cfg.total_bytes` have been delivered across all sessions.
/// 7. Closes all channels and returns the [`StressReport`].
///
/// The cluster must already be fully started and connected before calling this
/// function (i.e. [`cluster_fixture`] must have returned).
///
/// [`cluster_fixture`]: super::fixtures::cluster_fixture
/// [`HoprSession`]: crate::HoprSession
pub async fn run_stress(cluster: &ClusterGuard, cfg: &StressConfig) -> anyhow::Result<StressReport> {
    let n = cluster.size();
    let max_triples = n
        .saturating_mul(n.saturating_sub(1))
        .saturating_mul(n.saturating_sub(2));

    anyhow::ensure!(cfg.routes >= 1, "routes must be at least 1");
    anyhow::ensure!(
        cfg.routes <= max_triples,
        "routes ({}) exceeds available distinct triples for a {n}-node cluster ({max_triples})",
        cfg.routes,
    );
    anyhow::ensure!(!cfg.msg_size_range.is_empty(), "msg_size_range must be non-empty");

    let funding: HoprBalance = "100 wxHOPR".parse().context("parsing channel funding amount")?;

    // ── 1. Full directed channel mesh ─────────────────────────────────────────

    let channel_pairs: Vec<(usize, usize)> = (0..n)
        .flat_map(|i| (0..n).filter(move |&j| j != i).map(move |j| (i, j)))
        .collect();

    tracing::info!(
        channels = channel_pairs.len(),
        nodes = n,
        "opening directed channel mesh"
    );

    let channel_guards: Vec<ChannelGuard> = try_join_all(channel_pairs.iter().map(|&(i, j)| {
        ChannelGuard::open_channel_between_nodes(cluster[i].instance.clone(), cluster[j].instance.clone(), funding)
    }))
    .await
    .context("opening channel mesh")?;

    let chain_info = cluster
        .chain_client
        .query_chain_info()
        .await
        .context("querying chain info")?;
    let convergence_timeout = chain_propagation_delay(&chain_info) * 12;

    cluster
        .wait_for_channel_graph(&cluster[0], channel_guards.len(), convergence_timeout)
        .await
        .context("waiting for channel graph convergence")?;

    // ── 2. Route selection ────────────────────────────────────────────────────

    let mut rng = StdRng::seed_from_u64(cfg.seed);
    let mut all_triples: Vec<(usize, usize, usize)> = (0..n)
        .flat_map(|s| (0..n).flat_map(move |r| (0..n).map(move |d| (s, r, d))))
        .filter(|&(s, r, d)| s != r && r != d && s != d)
        .collect();
    all_triples.shuffle(&mut rng);

    let routes = &all_triples[..cfg.routes];
    tracing::info!(sessions = routes.len(), ?routes, "selected 1-hop routes");

    // ── 3. Session establishment ──────────────────────────────────────────────

    let mut sessions = Vec::with_capacity(routes.len());
    for &(s, r, d) in routes {
        tracing::info!(src = s, relay = r, dst = d, "opening session");
        let path = [&cluster[s], &cluster[r], &cluster[d]];
        let session = cluster.create_session(&path).await.context("opening 1-hop session")?;
        sessions.push(session);
    }

    // ── 4. Workload ───────────────────────────────────────────────────────────

    let delivered = Arc::new(AtomicU64::new(0));
    let total_target = cfg.total_bytes;
    let sample_interval = cfg.sample_interval;

    // Sampler task: records a ThroughputSample every `sample_interval`.
    // Tracks both sent bytes (written into the pipeline) and received bytes
    // (confirmed arrived at destination EchoServers).
    let (sample_tx, mut sample_rx) = tokio::sync::mpsc::unbounded_channel::<ThroughputSample>();
    let sampler_delivered = Arc::clone(&delivered);
    let sampler_received = Arc::clone(&cluster.echo_received);
    let workload_start = tokio::time::Instant::now();
    let sampler = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(sample_interval);
        ticker.tick().await; // skip the first immediate (t=0) tick
        let mut last_sent = 0u64;
        let mut last_recv = 0u64;
        loop {
            ticker.tick().await;
            let sent = sampler_delivered.load(Ordering::Relaxed);
            let recv = sampler_received.load(Ordering::Relaxed);
            let window_bytes = sent - last_sent;
            let recv_window_bytes = recv - last_recv;
            last_sent = sent;
            last_recv = recv;
            let elapsed = workload_start.elapsed();
            let secs = sample_interval.as_secs_f64();
            let mbps = (window_bytes as f64 / (1024.0 * 1024.0)) / secs;
            let recv_mbps = (recv_window_bytes as f64 / (1024.0 * 1024.0)) / secs;
            // Ignore send errors — they happen only if the receiver is dropped.
            let _ = sample_tx.send(ThroughputSample {
                elapsed,
                window_bytes,
                mbps,
                recv_window_bytes,
                recv_mbps,
            });
        }
    });

    // Worker tasks: one per session. Each loops: write → flush → count.
    //
    // The EchoServer at the destination counts bytes received on the forward path
    // (src→relay→dst) without echoing them back.  Echoing requires SURBs on the
    // exit node (DestinationRouting::Return), which are not provisioned here.
    // Forward throughput is sufficient to saturate SPHINX encoding, ticket
    // creation, the relay, and the mixer — the primary profiling targets.
    let msg_size_range = cfg.msg_size_range.clone();
    let seed = cfg.seed;
    let worker_handles: Vec<_> = sessions
        .into_iter()
        .enumerate()
        .map(|(idx, mut session)| {
            let delivered = Arc::clone(&delivered);
            let msg_size_range = msg_size_range.clone();
            tokio::spawn(async move {
                // Each worker uses a distinct seed so write sizes differ per session.
                let mut rng = StdRng::seed_from_u64(seed.wrapping_add(idx as u64 + 1));
                loop {
                    let msg_size = rng.random_range(msg_size_range.clone());
                    // Payload: sequential bytes — content is irrelevant; HOPR encrypts it.
                    let payload: Vec<u8> = (0..msg_size).map(|i| (i % 256) as u8).collect();

                    session.write_all(&payload).await.context("write_all")?;
                    session.flush().await.context("flush")?;

                    // Count bytes written into the pipeline (forward path only).
                    let prev = delivered.fetch_add(msg_size as u64, Ordering::Relaxed);
                    if prev + msg_size as u64 >= total_target {
                        break;
                    }
                }
                // Return the session to keep it alive through the drain phase.
                // Dropping here would send a close signal to the destination, which
                // would abort the EchoServer read loop while in-flight packets are
                // still traversing the mixer and relay.
                anyhow::Ok(session)
            })
        })
        .collect();

    tracing::info!(
        total_mb = cfg.total_bytes / (1024 * 1024),
        sessions = cfg.routes,
        "workload started"
    );

    // Collect finished sessions back so they stay open during the drain phase.
    let mut live_sessions = Vec::with_capacity(worker_handles.len());
    for handle in worker_handles {
        let session = handle
            .await
            .context("worker task panicked")?
            .context("worker task error")?;
        live_sessions.push(session);
    }

    let workload_duration = workload_start.elapsed();

    tracing::info!(
        elapsed_secs = workload_duration.as_secs_f64(),
        "workload finished — draining receive pipeline"
    );

    // ── Drain phase ───────────────────────────────────────────────────────────
    // After all sends complete the HOPR pipeline (mixer + relay) continues
    // delivering packets to the destination EchoServers.  Keep the sampler
    // running until every sent byte has been received, or until 5 s elapses
    // with no progress (indicating packet loss or a pipeline stall).
    let total_target_recv = cfg.total_bytes;
    let mut last_recv = cluster.echo_received.load(Ordering::Relaxed);
    let mut stall_elapsed = Duration::ZERO;
    let stall_limit = Duration::from_secs(5);
    loop {
        tokio::time::sleep(sample_interval).await;
        let received = cluster.echo_received.load(Ordering::Relaxed);
        if received >= total_target_recv {
            break;
        }
        if received > last_recv {
            // Progress — reset the stall clock.
            last_recv = received;
            stall_elapsed = Duration::ZERO;
        } else {
            stall_elapsed += sample_interval;
            if stall_elapsed >= stall_limit {
                tracing::warn!(
                    received,
                    total_target_recv,
                    "drain stalled for {}s — assuming packet loss",
                    stall_limit.as_secs()
                );
                break;
            }
        }
    }

    // Sessions are no longer needed — drop them now to cleanly close the
    // destination EchoServer sessions before we tear down the channels.
    drop(live_sessions);

    // Stop the sampler now that the receive side has drained (or timed out).
    sampler.abort();
    let _ = sampler.await; // JoinError::Cancelled is expected and ignored

    let mut samples = Vec::new();
    while let Ok(s) = sample_rx.try_recv() {
        samples.push(s);
    }

    // ── 5. Channel teardown ───────────────────────────────────────────────────

    tracing::info!(channels = channel_guards.len(), "closing channels");
    try_join_all(
        channel_guards
            .into_iter()
            .map(|g| async move { g.try_close_channels_all_channels().await }),
    )
    .await
    .context("closing channels")?;

    // ── Report ────────────────────────────────────────────────────────────────

    let total_bytes_delivered = delivered.load(Ordering::Relaxed);
    let total_bytes_received = cluster.echo_received.load(Ordering::Relaxed);
    let mean_mbps = if samples.is_empty() {
        (total_bytes_delivered as f64 / (1024.0 * 1024.0)) / workload_duration.as_secs_f64()
    } else {
        samples.iter().map(|s| s.mbps).sum::<f64>() / samples.len() as f64
    };
    let mean_recv_mbps = if samples.is_empty() {
        (total_bytes_received as f64 / (1024.0 * 1024.0)) / workload_duration.as_secs_f64()
    } else {
        samples.iter().map(|s| s.recv_mbps).sum::<f64>() / samples.len() as f64
    };

    Ok(StressReport {
        samples,
        total_bytes_delivered,
        total_bytes_received,
        duration: workload_duration,
        mean_mbps,
        mean_recv_mbps,
    })
}
