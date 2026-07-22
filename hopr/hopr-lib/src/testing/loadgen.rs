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
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
    /// Bytes delivered (echoed back and confirmed) during this window.
    pub window_bytes: u64,
    /// Throughput for this window in MB/s.
    pub mbps: f64,
}

/// Result of a completed stress run.
#[derive(Debug, Clone)]
pub struct StressReport {
    /// Per-window throughput samples, in chronological order.
    pub samples: Vec<ThroughputSample>,
    /// Total bytes confirmed delivered across all sessions.
    pub total_bytes_delivered: u64,
    /// Wall-clock duration of the workload phase, excluding channel/session setup.
    pub duration: Duration,
    /// Mean throughput in MB/s averaged across all sample windows.
    pub mean_mbps: f64,
}

impl StressReport {
    /// Prints the per-window throughput series and a summary line to stdout.
    pub fn print_series(&self) {
        println!("\n  {:>8}  {:>12}  {:>14}", "time(s)", "window MB/s", "cumulative");
        println!("  {}", "─".repeat(38));
        let mut cumulative = 0u64;
        for s in &self.samples {
            cumulative += s.window_bytes;
            println!(
                "  {:>8.2}  {:>12.3}  {:>14}",
                s.elapsed.as_secs_f64(),
                s.mbps,
                bytesize::ByteSize(cumulative),
            );
        }
        println!("  {}", "─".repeat(38));
        println!(
            "  Total: {} in {:.2}s — mean {:.3} MB/s",
            bytesize::ByteSize(self.total_bytes_delivered),
            self.duration.as_secs_f64(),
            self.mean_mbps,
        );
    }
}

// ── Configuration ────────────────────────────────────────────────────────────

/// Configuration for [`run_stress`].
#[derive(Debug, Clone)]
pub struct StressConfig {
    /// Total bytes to deliver before stopping.
    ///
    /// Counted when the echo is confirmed readable on the sender side, so this
    /// measures completed round-trips through the full HOPR pipeline.
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
    let (sample_tx, mut sample_rx) = tokio::sync::mpsc::unbounded_channel::<ThroughputSample>();
    let sampler_delivered = Arc::clone(&delivered);
    let workload_start = tokio::time::Instant::now();
    let sampler = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(sample_interval);
        ticker.tick().await; // skip the first immediate (t=0) tick
        let mut last_bytes = 0u64;
        loop {
            ticker.tick().await;
            let current = sampler_delivered.load(Ordering::Relaxed);
            let window_bytes = current - last_bytes;
            last_bytes = current;
            let elapsed = workload_start.elapsed();
            let mbps = (window_bytes as f64 / (1024.0 * 1024.0)) / sample_interval.as_secs_f64();
            // Ignore send errors — they happen only if the receiver is dropped.
            let _ = sample_tx.send(ThroughputSample {
                elapsed,
                window_bytes,
                mbps,
            });
        }
    });

    // Worker tasks: one per session. Each loops: write → flush → read echo → count.
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

                    let mut echo = vec![0u8; msg_size];
                    session.read_exact(&mut echo).await.context("read_exact")?;

                    // Count bytes after echo confirms delivery end-to-end.
                    let prev = delivered.fetch_add(msg_size as u64, Ordering::Relaxed);
                    if prev + msg_size as u64 >= total_target {
                        break;
                    }
                }
                anyhow::Ok(())
            })
        })
        .collect();

    tracing::info!(
        total_mb = cfg.total_bytes / (1024 * 1024),
        sessions = cfg.routes,
        "workload started"
    );

    for handle in worker_handles {
        handle
            .await
            .context("worker task panicked")?
            .context("worker task error")?;
    }

    let workload_duration = workload_start.elapsed();

    tracing::info!(
        elapsed_secs = workload_duration.as_secs_f64(),
        "workload finished — stopping sampler"
    );

    // Abort the sampler (it loops forever; abort is the clean shutdown).
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
    let mean_mbps = if samples.is_empty() {
        (total_bytes_delivered as f64 / (1024.0 * 1024.0)) / workload_duration.as_secs_f64()
    } else {
        samples.iter().map(|s| s.mbps).sum::<f64>() / samples.len() as f64
    };

    Ok(StressReport {
        samples,
        total_bytes_delivered,
        duration: workload_duration,
        mean_mbps,
    })
}
