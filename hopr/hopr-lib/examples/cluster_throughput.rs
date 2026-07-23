//! Multi-node cluster throughput profiling runner.
//!
//! Spins up a real-QUIC local cluster of HOPR nodes against a **mock Blokli
//! chain**, then drives a configurable number of concurrent sessions to
//! saturation and prints a per-window throughput series.  The *measured subject*
//! is the concurrent packet pipelines (SPHINX encode/decode, mixer, relay
//! forwarding, SURB balancing, session layer).  The chain is emulated
//! in-process; an optional Gaussian latency shim can simulate WAN conditions.
//!
//! Cluster size is set automatically: `hops + 2` nodes (minimum 3).  Bootstrap
//! takes ~60–120 s while the full mesh establishes real libp2p/QUIC connectivity
//! and probe warmup completes.
//!
//! # Usage
//!
//! ```sh
//! # Basic run (3 nodes / 1 hop, 20 MB, no flame graph):
//! cargo run --features testing --example cluster_throughput -- --hops 1 --mb 20
//!
//! # With in-process flame graph (pprof, works on macOS + Linux, no sudo):
//! cargo run --profile profiling \
//!     --features testing,profiling \
//!     --example cluster_throughput \
//!     -- --hops 1 --mb 20 --out flame.svg
//!
//! # Linux external flamegraph via perf (no pprof dep needed):
//! cargo flamegraph --profile profiling \
//!     --features testing \
//!     --example cluster_throughput \
//!     -- --hops 1 --mb 20
//! ```
//!
//! # Flags
//!
//! | Flag | Default | Description |
//! |------|---------|-------------|
//! | `--hops N` | `1` | Relay hops per path (1–3); cluster size = hops + 2. |
//! | `--mb N` | `100` | Megabytes to deliver before stopping. |
//! | `--routes N` | `4` | Concurrent sessions (independent pipelines). |
//! | `--seed N` | `42` | RNG seed for reproducible route selection. |
//! | `--out PATH` | — | Write flame graph SVG here (`profiling` feature only). |
//! | `--help` | — | Print usage and exit. |
//!
//! # Profiling notes
//!
//! The `profiling` feature enables in-process CPU sampling via `pprof` and
//! writes an SVG flame graph to `--out`.  The profiler is started **after**
//! cluster bootstrap and channel setup so the flame graph captures only the
//! sustained packet-pipeline workload — not node startup noise.
//!
//! Without the feature every `ProfilerGuard` call is a zero-cost no-op so the
//! binary still compiles and prints the throughput series.

use std::{path::PathBuf, time::Duration};

use hopr_lib::testing::{
    fixtures::{STRESS_WIN_PROB, stress_cluster_fixture},
    loadgen::{ProfilerGuard, StressConfig, run_stress},
};

// ── Argument parsing ──────────────────────────────────────────────────────────

struct Args {
    hops: usize,
    total_bytes: u64,
    routes: usize,
    seed: u64,
    flamegraph_out: Option<PathBuf>,
}

impl Args {
    fn parse() -> anyhow::Result<Self> {
        let mut iter = std::env::args().skip(1);
        let mut hops = 1usize;
        let mut mb = 100u64;
        let mut routes = 4usize;
        let mut seed = 42u64;
        let mut flamegraph_out: Option<PathBuf> = None;

        while let Some(flag) = iter.next() {
            match flag.as_str() {
                "--help" | "-h" => {
                    eprintln!(
                        "Usage: stress_cluster [--hops N] [--mb N] [--routes N] [--seed N] [--out PATH]\n\
                         \n\
                         --hops N     Relay hops per path, 1–3   (default: 1)\n\
                         --mb N       Megabytes to deliver        (default: 100)\n\
                         --routes N   Concurrent sessions          (default: 4)\n\
                         --seed N     RNG seed                    (default: 42)\n\
                         --out PATH   Write flame graph SVG here  (profiling feature only)\n\
                         \n\
                         Cluster size is set automatically to hops + 2."
                    );
                    std::process::exit(0);
                }
                "--hops" => hops = next_usize(&mut iter, "--hops")?,
                "--mb" => mb = next_u64(&mut iter, "--mb")?,
                "--routes" => routes = next_usize(&mut iter, "--routes")?,
                "--seed" => seed = next_u64(&mut iter, "--seed")?,
                "--out" => {
                    let path = iter.next().ok_or_else(|| anyhow::anyhow!("--out requires a path"))?;
                    flamegraph_out = Some(PathBuf::from(path));
                }
                other => anyhow::bail!("unknown flag: {other}  (try --help)"),
            }
        }

        Ok(Args {
            hops,
            total_bytes: mb * 1024 * 1024,
            routes,
            seed,
            flamegraph_out,
        })
    }
}

fn next_usize(iter: &mut impl Iterator<Item = String>, flag: &str) -> anyhow::Result<usize> {
    let raw = iter.next().ok_or_else(|| anyhow::anyhow!("{flag} requires a value"))?;
    raw.parse::<usize>().map_err(|e| anyhow::anyhow!("{flag}: {e}"))
}

fn next_u64(iter: &mut impl Iterator<Item = String>, flag: &str) -> anyhow::Result<u64> {
    let raw = iter.next().ok_or_else(|| anyhow::anyhow!("{flag} requires a value"))?;
    raw.parse::<u64>().map_err(|e| anyhow::anyhow!("{flag}: {e}"))
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() -> anyhow::Result<()> {
    let args = Args::parse()?;
    anyhow::ensure!(
        (1..=3).contains(&args.hops),
        "--hops must be between 1 and 3, got {}",
        args.hops,
    );
    let n_nodes = args.hops + 2;

    // Initialize the Rayon thread pool for SPHINX crypto operations before any
    // HOPR nodes start.  Without this, pool_thread_count() returns 0 and all
    // pool-based concurrency limits fall back to avail_parallelism * 8, allowing
    // SURB bursts to flood Rayon and starve data-packet encoding.
    let rayon_threads = std::thread::available_parallelism()
        .map(|n| (n.get() / 2).max(1))
        .unwrap_or(4);
    // Ignore the error: returns Err if the pool was already initialized (e.g.
    // by a framework that ran before main); in that case pool_thread_count()
    // was already set by the prior initialiser, so this is a no-op.
    let _ = hopr_utils::parallelize::cpu::init_thread_pool(rayon_threads);

    eprintln!(
        "→ Starting {n_nodes}-node cluster ({}-hop) — full-mesh connectivity can take ~100 s for 3 nodes,\n  longer \
         for larger clusters. Please wait…",
        args.hops
    );

    // stress_cluster_fixture is a blocking call that waits for full-mesh
    // connectivity before returning.  Call it from the main thread before
    // creating the async runtime so we don't block a runtime thread.
    let cluster = stress_cluster_fixture(STRESS_WIN_PROB, n_nodes);

    eprintln!("→ Cluster ready.  Opening channels and sessions…");

    let cfg = StressConfig {
        hops: args.hops,
        total_bytes: args.total_bytes,
        routes: args.routes,
        seed: args.seed,
        msg_size_range: 4096..=65536, // 4 KB – 64 KB: realistic HOPR session messages
        sample_interval: Duration::from_millis(500),
        ..StressConfig::default()
    };

    // Build the async runtime *after* cluster startup so its thread pool does
    // not compete with node bootstrap.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4.max(n_nodes * 2))
        .enable_all()
        .build()?;

    // Start the profiler only when --out is given; starting without a destination wastes
    // sampling overhead on runs that don't need a flame graph.
    // When `profiling` feature is absent every call is a zero-cost no-op.
    let profiler = if args.flamegraph_out.is_some() {
        Some(ProfilerGuard::start(1000)?)
    } else {
        None
    };

    eprintln!(
        "→ Sending {} MB across {} {}-hop session(s) (seed={})…",
        args.total_bytes / (1024 * 1024),
        args.routes,
        args.hops,
        args.seed,
    );

    let report = rt.block_on(run_stress(&cluster, &cfg))?;

    // Write the flame graph while the profiler guard is still alive.
    if let Some(ref path) = args.flamegraph_out {
        if let Some(ref p) = profiler {
            p.write_flamegraph(path)?;
        }
        if cfg!(feature = "profiling") {
            eprintln!("→ Flame graph written to {}", path.display());
        } else {
            eprintln!("→ --out set but `profiling` feature is disabled; no flame graph written.");
        }
    }
    drop(profiler); // stop sampling if active

    report.print_series();

    Ok(())
}
