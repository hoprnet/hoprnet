//! Local-cluster throughput stress harness.
//!
//! Drives a running [`ClusterGuard`] with bulk traffic across random paths of
//! configurable hop depth and records a per-window throughput time-series.
//! The caller can observe whether delivery rate climbs, holds steady, or
//! degrades under sustained saturation at 1-hop, 2-hop, or 3-hop depth.
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
        atomic::{AtomicU64, AtomicUsize, Ordering},
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
use crate::{SessionCapabilities, SessionCapability, SurbBalancerConfig, api::types::primitive::prelude::HoprBalance};

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
    /// Number of intermediate relay hops used in this run (mirrors [`StressConfig::hops`]).
    pub hops: usize,
    /// Human-readable label describing the run's key parameters (hops, write strategy,
    /// rate control, SURB config).  Printed in the `print_series` header for easy
    /// comparison between OFAT variants.
    pub label: String,
    /// Total application bytes written into the pipeline across all sessions.
    pub total_bytes_delivered: u64,
    /// Total bytes confirmed received by destination EchoServers (delta from run start).
    pub total_bytes_received: u64,
    /// Wall-clock duration of the workload phase, excluding channel/session setup.
    pub duration: Duration,
    /// Mean send throughput in MB/s averaged across all sample windows.
    pub mean_mbps: f64,
    /// Mean receive throughput in MB/s averaged across all sample windows.
    pub mean_recv_mbps: f64,
    /// Theoretical SURB keep-alive overhead in MB/s per session (infrastructure traffic,
    /// not counted in throughput above — invisible at the session layer, carried entirely
    /// inside the SPHINX packet payload fields separate from application data).
    ///
    /// Computed as: `max_surbs_per_sec ÷ MAX_SURBS_IN_PACKET × SPHINX_packet_size`.
    /// With defaults: 5000 ÷ 2 × 1461 B ≈ 3.48 MB/s per session.
    pub surb_overhead_mbps_per_session: f64,
    /// Peak simultaneous encode (packet_encode + SURB generation) tasks in the Rayon pool
    /// observed during the workload phase.
    pub peak_encode_outstanding: usize,
    /// Peak simultaneous decode (packet_decode) tasks in the Rayon pool observed during
    /// the workload phase.
    pub peak_decode_outstanding: usize,
    /// Decode timeout drops that occurred during the workload phase (delta from baseline).
    /// Each unit is one incoming packet that was discarded because Rayon did not return
    /// the decoded result within `PACKET_DECODING_TIMEOUT` (150 ms).
    pub decode_timeout_drops: u64,
    /// Session inbox drops that occurred during the workload phase (delta from baseline).
    /// Each unit is one packet that was decoded successfully but could not be forwarded
    /// to the session because the per-session bounded inbox was full.
    pub session_inbox_drops: u64,
    /// Encode timeout drops that occurred during the workload phase (delta from baseline).
    /// Each unit is one outgoing packet (data or SURB keep-alive) that was dropped because
    /// the Rayon encode future exceeded its 150 ms budget — a direct sign of pool saturation
    /// on the encode path.
    pub encode_timeout_drops: u64,
    /// Packets dispatched to the session manager but dropped because no matching session
    /// slot was found (UnknownData path — the session may have been torn down or the ID
    /// mismatched).
    pub session_unknown_data_drops: u64,
    /// Packets that reached dispatch_message but matched neither session protocol tag nor
    /// any session application tag — fell through to the "unrelated" path without being
    /// counted as a drop. Non-zero indicates packets reaching the session manager but
    /// being silently discarded as "not session data".
    pub session_unrelated_dispatches: u64,
    /// Cumulative count of packets that entered the routing resolution stage (all nodes combined).
    pub routing_resolution_attempts: u64,
    /// Cumulative count of packets that failed routing resolution (dropped before encoding).
    pub routing_resolution_failures: u64,
    /// Cumulative count of packets that entered the SPHINX encode stage (spawn_encode_blocking called).
    pub encode_stage_entries: u64,
    /// Cumulative count of calls to `smgr.dispatch_message` in `SessionsManagement(0)`.
    /// Zero means data never reached the session manager (stuck in pipeline buffers or dropped earlier).
    pub dispatch_message_calls: u64,
}

impl StressReport {
    /// Prints the per-window throughput series and a detailed summary to stdout.
    pub fn print_series(&self) {
        let total_mb = self.total_bytes_delivered as f64 / (1024.0 * 1024.0);
        println!("\n=== {} · {:.0} MB ===", self.label, total_mb);
        println!(
            "  {:>8}  {:>12}  {:>12}  {:>10}",
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
            "  Loss:  {} ({:.2}%)  [bytes in-flight or dropped at drain timeout]",
            bytesize::ByteSize(in_flight),
            loss_pct,
        );
        let surb_total_mb = self.surb_overhead_mbps_per_session * self.duration.as_secs_f64();
        println!(
            "  SURB:  ~{:.2} MB/s per session (infrastructure overhead — not counted above)",
            self.surb_overhead_mbps_per_session,
        );
        println!(
            "         ~{:.1} MB theoretical keep-alive traffic over {:.2}s",
            surb_total_mb,
            self.duration.as_secs_f64(),
        );
        println!(
            "  Pool:  encode peak {} tasks  decode peak {} tasks",
            self.peak_encode_outstanding, self.peak_decode_outstanding,
        );
        println!(
            "  Drops: encode_timeout {}  decode_timeout {}  session_inbox {}  unknown_session {}  unrelated_dispatch \
             {}",
            self.encode_timeout_drops,
            self.decode_timeout_drops,
            self.session_inbox_drops,
            self.session_unknown_data_drops,
            self.session_unrelated_dispatches,
        );
        println!(
            "  Route: attempts {}  failures {}  encode_stage_entries {}",
            self.routing_resolution_attempts, self.routing_resolution_failures, self.encode_stage_entries,
        );
        println!("  Dispatch: dispatch_message_calls {}", self.dispatch_message_calls,);
    }
}

// ── Configuration ────────────────────────────────────────────────────────────

/// Strategy controlling how each session worker writes data into the pipeline.
///
/// The choice between `Continuous` and `Batched` controls whether the Rayon encode
/// pool is given breathing room between bursts — which is the key variable for
/// measuring encode-timeout silent drops.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WriteStrategy {
    /// Write messages back-to-back as fast as the sink accepts them.
    ///
    /// Stresses the 150 ms encode budget: if the Rayon pool saturates, encode
    /// futures time out and packets are silently dropped (visible as loss in the
    /// report).  This is the baseline that matches the edge-client's behaviour.
    #[default]
    Continuous,
    /// Write `batch_bytes` of data per burst, then sleep `pause` before the next burst.
    ///
    /// Gives the encode pool time to drain between bursts, reducing encode-timeout
    /// drops at the cost of lower peak throughput.  Compare against `Continuous` to
    /// quantify the drop penalty of aggressive bulk writes.
    Batched {
        /// Total bytes per burst (rounded down to the nearest message boundary).
        batch_bytes: usize,
        /// Sleep duration between bursts.
        pause: Duration,
    },
}

impl std::fmt::Display for WriteStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WriteStrategy::Continuous => write!(f, "continuous"),
            WriteStrategy::Batched { batch_bytes, pause } => {
                write!(f, "batched({}B,{}ms)", batch_bytes, pause.as_millis())
            }
        }
    }
}

/// Configuration for [`run_stress`].
#[derive(Debug, Clone)]
pub struct StressConfig {
    /// Total application bytes to write into the pipeline before stopping.
    ///
    /// Counted when each `write_all` + `flush` cycle completes on the sender side.
    /// Delivery at the destination is tracked via [`ClusterGuard::echo_received`].
    /// SURB keep-alive traffic (infrastructure overhead) is excluded from all statistics.
    ///
    /// [`ClusterGuard::echo_received`]: super::fixtures::ClusterGuard::echo_received
    pub total_bytes: u64,

    /// Number of intermediate relay hops (1 = `src→relay→dst`, 2 = `src→r1→r2→dst`, …).
    ///
    /// The cluster must have at least `hops + 2` nodes.
    /// Maximum supported by the HOPR protocol is 3.
    pub hops: usize,

    /// Number of concurrent sessions to maintain.
    ///
    /// Each session uses a distinct randomly-chosen path of `hops + 2` nodes.
    /// Must not exceed the number of valid distinct paths for the cluster size
    /// (`P(n, hops+2) = n × (n-1) × … × (n-hops-1)`).
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

    /// Write strategy for workload tasks.
    ///
    /// `Continuous` (default) writes back-to-back — matching the edge-client and stressing
    /// the encode pool.  `Batched` inserts pauses between bursts to allow the pool to drain,
    /// which reduces encode-timeout drops at the cost of peak throughput.
    pub write_strategy: WriteStrategy,

    /// SURB balancer configuration for each session.
    ///
    /// `Some(cfg)` enables the SURB balancer; `None` disables it.
    /// Default: `Some(SurbBalancerConfig::default())` (target 7000, max 5000/s) — matching
    /// the production fixture and enabling the exit-side egress rate limiter warm-up.
    pub surb_config: Option<SurbBalancerConfig>,

    /// Whether to apply exit-side egress rate control on each session.
    ///
    /// `true` (default) leaves capabilities at their default (empty) so the Exit's
    /// SURB-based rate limiter (10 pkt/s cold-start, then balancer-scaled) is active.
    /// `false` sets `SessionCapability::NoRateControl`, disabling the limiter entirely.
    ///
    /// The rate limiter exists so the Exit never sends replies faster than the initiator
    /// supplies SURBs.  Disabling it reveals the maximum pipeline throughput without the
    /// cold-start throttle — useful for isolating whether the limiter is the bottleneck.
    pub rate_control: bool,
}

impl Default for StressConfig {
    fn default() -> Self {
        Self {
            total_bytes: 100 * 1024 * 1024, // 100 MB
            hops: 1,
            routes: 4,
            msg_size_range: 4096..=65536, // 4 KB – 64 KB
            sample_interval: Duration::from_millis(500),
            seed: 42,
            write_strategy: WriteStrategy::Continuous,
            surb_config: Some(SurbBalancerConfig::default()),
            rate_control: true,
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
/// 2. Selects `cfg.routes` random distinct paths of length `cfg.hops + 2`.
/// 3. Establishes one long-lived [`HoprSession`] per path.
/// 4. Spawns one worker task per session; each task streams random-sized writes and counts bytes.
/// 5. Records per-window throughput samples from a background task.
/// 6. Stops once `cfg.total_bytes` have been delivered across all sessions.
/// 7. Closes all channels and returns the [`StressReport`].
///
/// The cluster must already be fully started and connected before calling this
/// function (i.e. [`stress_cluster_fixture`] must have returned).  `run_stress`
/// can be called multiple times on the same cluster: each call tracks its own
/// baseline so echo counters from prior runs do not bleed into the report.
///
/// [`stress_cluster_fixture`]: super::fixtures::stress_cluster_fixture
/// [`HoprSession`]: crate::HoprSession
pub async fn run_stress(cluster: &ClusterGuard, cfg: &StressConfig) -> anyhow::Result<StressReport> {
    let n = cluster.size();
    let path_len = cfg.hops + 2;

    // P(n, path_len) = n × (n-1) × … × (n - path_len + 1)
    let max_paths = (0..path_len).fold(1usize, |acc, k| acc.saturating_mul(n.saturating_sub(k)));

    anyhow::ensure!(cfg.total_bytes > 0, "total_bytes must be greater than 0");
    anyhow::ensure!(!cfg.sample_interval.is_zero(), "sample_interval must be non-zero");
    anyhow::ensure!(cfg.hops >= 1, "hops must be at least 1");
    anyhow::ensure!(
        n >= path_len,
        "{}-hop routing requires at least {} nodes, but cluster has {}",
        cfg.hops,
        path_len,
        n,
    );
    anyhow::ensure!(cfg.routes >= 1, "routes must be at least 1");
    anyhow::ensure!(
        cfg.routes <= max_paths,
        "routes ({}) exceeds available distinct {}-hop paths for a {n}-node cluster ({max_paths})",
        cfg.routes,
        cfg.hops,
    );
    anyhow::ensure!(!cfg.msg_size_range.is_empty(), "msg_size_range must be non-empty");

    // 1 000 000 wxHOPR per channel accommodates both data-packet tickets AND SURB
    // echo-packet tickets on the return path at STRESS_WIN_PROB = 0.001.
    //
    // For a 100 MB run:
    //   ~175 000 data packets  × 0.001 × 2 000 wxHOPR ≈ 350 000 wxHOPR (forward path)
    //   ~175 000 SURB packets  × 0.001 × 2 000 wxHOPR ≈ 350 000 wxHOPR (return path)
    // Total budget needed per channel ≈ 700 000 wxHOPR; 1 M gives ≈1.4× headroom.
    // STRESS_INITIAL_SAFE_TOKEN (20 M wxHOPR) covers 4 outgoing channels × 1 M with 5×
    // margin for path-selection variability across a 5-node full-mesh cluster.
    let funding: HoprBalance = "1000000 wxHOPR".parse().context("parsing channel funding amount")?;

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
    // Enumerate all distinct paths of length `path_len` from 0..n, then shuffle
    // and take `cfg.routes`.  A DFS stack gives all permutations; sorting before
    // the shuffle ensures reproducibility given the same seed.

    let mut rng = StdRng::seed_from_u64(cfg.seed);
    let mut all_paths: Vec<Vec<usize>> = {
        let mut result = Vec::new();
        let mut stack: Vec<Vec<usize>> = (0..n).map(|i| vec![i]).collect();
        while let Some(path) = stack.pop() {
            if path.len() == path_len {
                result.push(path);
            } else {
                for next in (0..n).filter(|j| !path.contains(j)) {
                    let mut ext = path.clone();
                    ext.push(next);
                    stack.push(ext);
                }
            }
        }
        result.sort();
        result
    };
    all_paths.shuffle(&mut rng);

    let routes = &all_paths[..cfg.routes];
    tracing::info!(hops = cfg.hops, sessions = routes.len(), ?routes, "selected routes");

    // ── 3. Session establishment ──────────────────────────────────────────────

    // Build session capabilities from config.
    // `rate_control = false` → set NoRateControl to bypass the 10 pkt/s cold-start limiter.
    let capabilities: SessionCapabilities = if cfg.rate_control {
        Default::default() // empty = limiter active (production default)
    } else {
        SessionCapability::NoRateControl.into()
    };

    let mut sessions = Vec::with_capacity(routes.len());
    for path_indices in routes {
        tracing::info!(?path_indices, "opening session");
        let path: Vec<&_> = path_indices.iter().map(|&i| &cluster[i]).collect();
        let session = cluster
            .create_session_with(&path, capabilities, cfg.surb_config.clone())
            .await
            .context("opening session")?;
        sessions.push(session);
    }

    // ── 4. Workload ───────────────────────────────────────────────────────────

    // Baseline received bytes at the start of this run.  Using a delta allows
    // run_stress to be called multiple times on the same cluster without prior
    // runs' echo counts leaking into this run's drain detection and report.
    let recv_baseline = cluster.echo_received.load(Ordering::Relaxed);

    let delivered = Arc::new(AtomicU64::new(0));
    let total_target = cfg.total_bytes;
    let sample_interval = cfg.sample_interval;

    // Baseline counters before the workload starts so we can compute deltas.
    let baseline_decode_drops = hopr_utils::parallelize::cpu::decode_timeout_drop_count() as u64;
    let baseline_encode_drops = hopr_utils::parallelize::cpu::encode_timeout_drop_count() as u64;
    let baseline_inbox_drops = hopr_utils::parallelize::session_inbox_drop_count() as u64;
    let baseline_unknown_drops = hopr_utils::parallelize::session_unknown_data_drop_count() as u64;
    let baseline_routing_attempts = hopr_utils::parallelize::routing_resolution_attempt_count() as u64;
    let baseline_routing_failures = hopr_utils::parallelize::routing_resolution_failure_count() as u64;
    let baseline_encode_stage_entries = hopr_utils::parallelize::encode_stage_entry_count() as u64;
    let baseline_unrelated = hopr_utils::parallelize::session_unrelated_dispatch_count() as u64;
    let baseline_dispatch_calls = hopr_utils::parallelize::dispatch_message_call_count() as u64;

    // Sampler task: records a ThroughputSample every `sample_interval`.
    // Tracks both sent bytes (written into the pipeline) and received bytes
    // (confirmed arrived at destination EchoServers).
    // Also tracks peak Rayon encode/decode outstanding tasks for pool diagnostics.
    let (sample_tx, mut sample_rx) = tokio::sync::mpsc::unbounded_channel::<ThroughputSample>();
    let sampler_delivered = Arc::clone(&delivered);
    let sampler_received = Arc::clone(&cluster.echo_received);
    let peak_encode = Arc::new(AtomicUsize::new(0));
    let peak_decode = Arc::new(AtomicUsize::new(0));
    let sampler_peak_encode = Arc::clone(&peak_encode);
    let sampler_peak_decode = Arc::clone(&peak_decode);
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

            // Track peak Rayon encode/decode outstanding tasks.
            sampler_peak_encode.fetch_max(
                hopr_utils::parallelize::cpu::encode_outstanding_tasks(),
                Ordering::Relaxed,
            );
            sampler_peak_decode.fetch_max(
                hopr_utils::parallelize::cpu::decode_outstanding_tasks(),
                Ordering::Relaxed,
            );

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
    // The EchoServer at the destination counts application bytes received on the forward
    // path (src→relay→dst).  SURB keep-alive packets (infrastructure traffic carrying
    // SURBs in the SPHINX payload field) are transparent at the session layer — they
    // never reach the EchoServer and are not counted in any throughput figure.
    let msg_size_range = cfg.msg_size_range.clone();
    let seed = cfg.seed;
    let write_strategy = cfg.write_strategy;
    let worker_handles: Vec<_> = sessions
        .into_iter()
        .enumerate()
        .map(|(idx, mut session)| {
            let delivered = Arc::clone(&delivered);
            let msg_size_range = msg_size_range.clone();
            tokio::spawn(async move {
                // Each worker uses a distinct seed so write sizes differ per session.
                let mut rng = StdRng::seed_from_u64(seed.wrapping_add(idx as u64 + 1));

                // Batched strategy: track how many bytes remain in the current burst.
                let mut batch_remaining: usize = match write_strategy {
                    WriteStrategy::Batched { batch_bytes, .. } => batch_bytes,
                    WriteStrategy::Continuous => usize::MAX,
                };

                // Pre-allocate a max-size payload buffer once; content is irrelevant (HOPR encrypts it).
                let max_msg = *msg_size_range.end();
                let payload_buf: Vec<u8> = (0..max_msg).map(|i| (i % 256) as u8).collect();

                loop {
                    let msg_size = rng.random_range(msg_size_range.clone());

                    // Batched: if this write would exhaust the burst quota, sleep first.
                    if let WriteStrategy::Batched { batch_bytes, pause } = write_strategy {
                        if msg_size > batch_remaining {
                            session.flush().await.context("flush (batch pause)")?;
                            tokio::time::sleep(pause).await;
                            batch_remaining = batch_bytes;
                        }
                        batch_remaining = batch_remaining.saturating_sub(msg_size);
                    }

                    session.write_all(&payload_buf[..msg_size]).await.context("write_all")?;
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

    let label = {
        let surb_target = cfg
            .surb_config
            .as_ref()
            .map(|s| format!("surb={}", s.target_surb_buffer_size))
            .unwrap_or_else(|| "surb=off".to_owned());
        let rate = if cfg.rate_control { "rate=on" } else { "rate=off" };
        format!("{}-hop · {} · {} · {}", cfg.hops, cfg.write_strategy, rate, surb_target)
    };

    tracing::info!(
        total_mb = cfg.total_bytes / (1024 * 1024),
        hops = cfg.hops,
        sessions = cfg.routes,
        %label,
        "workload started"
    );

    // Collect finished sessions back so they stay open during the drain phase.
    // On error, abort the sampler before returning so its infinite loop doesn't outlive this call.
    let mut live_sessions = Vec::with_capacity(worker_handles.len());
    let mut worker_err: Option<anyhow::Error> = None;
    for handle in worker_handles {
        match handle.await {
            Err(e) => {
                worker_err = Some(anyhow::anyhow!("worker task panicked: {e}"));
                break;
            }
            Ok(Err(e)) => {
                worker_err = Some(e.context("worker task error"));
                break;
            }
            Ok(Ok(session)) => live_sessions.push(session),
        }
    }
    if let Some(e) = worker_err {
        sampler.abort();
        return Err(e);
    }

    let workload_duration = workload_start.elapsed();

    tracing::info!(
        elapsed_secs = workload_duration.as_secs_f64(),
        "workload finished — draining receive pipeline"
    );

    // ── Drain phase ───────────────────────────────────────────────────────────
    // After all sends complete the HOPR pipeline (mixer + relay) continues
    // delivering packets to the destination EchoServers.  Keep the sampler
    // running until every sent byte has been received (delta from recv_baseline),
    // or until 30 s elapses with no progress (indicating packet loss or a
    // pipeline stall).  The 30 s stall limit is intentionally generous to
    // accommodate the higher per-hop mixer and relay latency at 2-hop and 3-hop
    // depths, and SURB echo traffic on the return path.
    let total_target_recv_abs = recv_baseline + cfg.total_bytes;
    let mut last_recv = cluster.echo_received.load(Ordering::Relaxed);
    let mut stall_elapsed = Duration::ZERO;
    let stall_limit = Duration::from_secs(30);
    loop {
        tokio::time::sleep(sample_interval).await;
        let received = cluster.echo_received.load(Ordering::Relaxed);
        if received >= total_target_recv_abs {
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
                    total_target_recv_abs,
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
    let total_bytes_received = cluster.echo_received.load(Ordering::Relaxed) - recv_baseline;
    // Exclude drain-phase samples (window_bytes ≈ 0 because no sends occur after workload_duration)
    // so the reported means reflect sustained throughput during the active send window only.
    let active: Vec<_> = samples.iter().filter(|s| s.elapsed <= workload_duration).collect();
    let mean_mbps = if active.is_empty() {
        (total_bytes_delivered as f64 / (1024.0 * 1024.0)) / workload_duration.as_secs_f64()
    } else {
        active.iter().map(|s| s.mbps).sum::<f64>() / active.len() as f64
    };
    let mean_recv_mbps = if active.is_empty() {
        (total_bytes_received as f64 / (1024.0 * 1024.0)) / workload_duration.as_secs_f64()
    } else {
        active.iter().map(|s| s.recv_mbps).sum::<f64>() / active.len() as f64
    };

    // Theoretical SURB keep-alive overhead (infrastructure, not counted in throughput).
    // SurbBalancerConfig::default(): max_surbs_per_sec = 5000, MAX_SURBS_IN_PACKET = 2.
    // Each keep-alive is a full SPHINX packet = 1461 bytes (422 header + 1039 payload).
    const MAX_SURBS_IN_PACKET: f64 = 2.0;
    const SPHINX_PKT_BYTES: f64 = 1461.0;
    const MAX_SURBS_PER_SEC: f64 = 5000.0;
    let surb_overhead_mbps_per_session =
        (MAX_SURBS_PER_SEC / MAX_SURBS_IN_PACKET * SPHINX_PKT_BYTES) / (1024.0 * 1024.0);

    let decode_timeout_drops = hopr_utils::parallelize::cpu::decode_timeout_drop_count() as u64 - baseline_decode_drops;
    let encode_timeout_drops = hopr_utils::parallelize::cpu::encode_timeout_drop_count() as u64 - baseline_encode_drops;
    let session_inbox_drops = hopr_utils::parallelize::session_inbox_drop_count() as u64 - baseline_inbox_drops;
    let session_unknown_data_drops =
        hopr_utils::parallelize::session_unknown_data_drop_count() as u64 - baseline_unknown_drops;
    let session_unrelated_dispatches =
        hopr_utils::parallelize::session_unrelated_dispatch_count() as u64 - baseline_unrelated;
    let dispatch_message_calls =
        hopr_utils::parallelize::dispatch_message_call_count() as u64 - baseline_dispatch_calls;
    let routing_resolution_attempts =
        hopr_utils::parallelize::routing_resolution_attempt_count() as u64 - baseline_routing_attempts;
    let routing_resolution_failures =
        hopr_utils::parallelize::routing_resolution_failure_count() as u64 - baseline_routing_failures;
    let encode_stage_entries =
        hopr_utils::parallelize::encode_stage_entry_count() as u64 - baseline_encode_stage_entries;

    Ok(StressReport {
        samples,
        hops: cfg.hops,
        label,
        total_bytes_delivered,
        total_bytes_received,
        duration: workload_duration,
        mean_mbps,
        mean_recv_mbps,
        surb_overhead_mbps_per_session,
        peak_encode_outstanding: peak_encode.load(Ordering::Relaxed),
        peak_decode_outstanding: peak_decode.load(Ordering::Relaxed),
        decode_timeout_drops,
        session_inbox_drops,
        encode_timeout_drops,
        session_unknown_data_drops,
        session_unrelated_dispatches,
        routing_resolution_attempts,
        routing_resolution_failures,
        encode_stage_entries,
        dispatch_message_calls,
    })
}
