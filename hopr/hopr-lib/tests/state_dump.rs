//! Exercise the published diagnostic interfaces without depending on private snapshot models.
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{Context, ensure};
use hopr_api::{
    StateDump,
    graph::NetworkGraphView,
    types::crypto::prelude::{Keypair, OffchainKeypair},
};
use hopr_lib::testing::{
    fixtures::{TestNodeConfig, build_role_cluster, cluster_fixture},
    hopr::TestedHopr,
};
use hopr_transport_p2p::HoprLibp2pNetworkBuilder;
use parking_lot::Mutex;
use serde_json::Value;

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<Value>>>, Arc<std::sync::atomic::AtomicUsize>);
struct Visitor<'a>(&'a Capture);
impl tracing::field::Visit for Visitor<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "state"
            && let Ok(value) = serde_json::from_str::<Value>(&format!("{value:?}"))
        {
            let mut records = self.0.0.lock();
            if records.len() < 128 {
                records.push(value);
            }
        }
    }
}
impl tracing::Subscriber for Capture {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        *metadata.level() <= tracing::Level::INFO
    }

    fn new_span(&self, _: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        tracing::span::Id::from_u64(1)
    }

    fn record(&self, _: &tracing::span::Id, _: &tracing::span::Record<'_>) {}

    fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}

    fn event(&self, event: &tracing::Event<'_>) {
        if *event.metadata().level() == tracing::Level::INFO {
            event.record(&mut Visitor(self));
        }
        if *event.metadata().level() == tracing::Level::ERROR
            && self.1.fetch_add(1, std::sync::atomic::Ordering::Relaxed) < 8
        {
            eprintln!("{event:?}");
        }
    }

    fn enter(&self, _: &tracing::span::Id) {}

    fn exit(&self, _: &tracing::span::Id) {}
}

/// Check the documented assessment envelope; the component-specific state stays opaque.
async fn poll_snapshot(source: &impl StateDump) -> anyhow::Result<()> {
    let snapshot = source.dump().await?;
    let json = serde_json::to_string(&snapshot)?;
    let value: Value = serde_json::from_str(&json)?;
    ensure!(value["analysis_required"].is_boolean(), "missing assessment: {value}");
    ensure!(value["reason"].as_str().is_some_and(|reason| !reason.is_empty()));
    ensure!(value.get("state").is_some(), "missing opaque snapshot: {value}");
    Ok(())
}

async fn poll_node<TMgr: Send + Sync + 'static>(node: &TestedHopr<TMgr>) -> anyhow::Result<()> {
    let graph = node.inner().graph();
    let view = node
        .connector()
        .state_view(node.config().safe_module.safe_address, graph.ticket_face_value());
    poll_snapshot(&view).await?;
    poll_snapshot(graph).await?;
    Ok(())
}

#[test]
fn public_component_diagnostics_support_polling_and_periodic_reporting() -> anyhow::Result<()> {
    let Some(mode) = std::env::var_os("HOPR_STATE_DUMP_TEST_CHILD") else {
        // Isolate the reporter configuration and global tracing subscriber from other tests.
        for (mode, interval) in [
            ("enabled", "5"),
            ("disabled", "0"),
            ("invalid", "invalid"),
            ("roles", "5"),
        ] {
            let mut child = std::process::Command::new(std::env::current_exe()?)
                .args([
                    "--exact",
                    "public_component_diagnostics_support_polling_and_periodic_reporting",
                    "--nocapture",
                ])
                .env("HOPR_STATE_DUMP_TEST_CHILD", mode)
                .env("HOPR_STATE_DUMP_INTERVAL", interval)
                .spawn()?;
            let deadline = Instant::now() + Duration::from_secs(180);
            loop {
                if let Some(status) = child.try_wait()? {
                    ensure!(status.success(), "state dump {mode} child failed");
                    break;
                }
                if Instant::now() >= deadline {
                    child.kill()?;
                    child.wait()?;
                    anyhow::bail!("state dump {mode} child exceeded its deadline");
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
        return Ok(());
    };
    let capture = Capture::default();
    tracing::subscriber::set_global_default(capture.clone()).context("install INFO-only subscriber")?;
    if mode == "roles" {
        return tokio::runtime::Runtime::new()?.block_on(async {
            let cluster = build_role_cluster(
                TestNodeConfig::default(),
                vec![TestNodeConfig::default()],
                TestNodeConfig::default(),
            )
            .await?;
            poll_node(&cluster.entry).await?;
            poll_node(&cluster.relays[0]).await?;
            poll_node(&cluster.exit).await?;
            ensure!(capture.0.lock().is_empty(), "polling must not start a reporter");
            Ok(())
        });
    }
    let cluster = cluster_fixture(vec![TestNodeConfig::default(); 2]);
    tokio::runtime::Runtime::new()?.block_on(verify_reporting(&cluster[0], &capture, &mode))
}

async fn verify_reporting(node: &TestedHopr, capture: &Capture, mode: &std::ffi::OsStr) -> anyhow::Result<()> {
    let graph = node.inner().graph();
    // The node exposes NetworkView, not its concrete P2P implementation. Build a real P2P
    // handle through its public constructor to exercise its diagnostic interface separately.
    let (network, _network_process) = HoprLibp2pNetworkBuilder::new(futures::stream::empty())
        .build(&OffchainKeypair::random(), vec![], "/hopr/state-dump-test/1", true)
        .await
        .map_err(|error| anyhow::anyhow!("build P2P network: {error}"))?;
    // On-demand snapshots remain available when periodic reporting is disabled or misconfigured.
    poll_node(node).await?;
    poll_snapshot(&network).await?;
    ensure!(
        capture.0.lock().is_empty(),
        "enabling diagnostics must not install a reporter"
    );

    let instance = node.address().to_string();
    let reporters = async {
        futures::join!(
            node.connector().run_state_reporting_with_funding_policy(
                &instance,
                node.config().safe_module.safe_address,
                || graph.ticket_face_value()
            ),
            graph.run_state_reporting(&instance, Vec::new),
            network.run_state_reporting(&instance),
        )
    };
    if mode == "disabled" || mode == "invalid" {
        let (chain, graph, network) = tokio::time::timeout(Duration::from_secs(2), reporters).await?;
        for (name, result) in [("chain", chain), ("graph", graph), ("transport", network)] {
            if mode == "invalid" {
                let error = result.expect_err("invalid reporter configuration must be rejected");
                ensure!(error.contains("HOPR_STATE_DUMP_INTERVAL"), "{name}: {error}");
            } else {
                result.map_err(anyhow::Error::msg)?;
            }
        }
        ensure!(
            capture.0.lock().is_empty(),
            "disabled/invalid reporters emitted records"
        );
        return Ok(());
    }

    // The published reporters own timers, budgets and assessment policy.
    let reported = wait_for_components(capture);
    tokio::select! {
        result = reporters => anyhow::bail!("enabled reporters stopped unexpectedly: {result:?}"),
        result = reported => result.context("all components must report periodically")?,
    }
    let records = capture.0.lock().clone();
    for component in ["chain", "graph", "transport"] {
        let sequences: Vec<_> = records
            .iter()
            .filter(|record| record["component"] == component)
            .filter_map(|record| record["sequence"].as_u64())
            .collect();
        ensure!(
            sequences.starts_with(&[0, 1]),
            "{component} must have its own sequence: {sequences:?}"
        );
    }
    for record in &records {
        ensure!(record["instance"] == instance);
        ensure!(record["analysis_required"].is_boolean());
        ensure!(record["reason"].as_str().is_some_and(|reason| !reason.is_empty()));
        ensure!(
            record["state"]["status"] == "ok",
            "component collection failed: {record}"
        );
        ensure!(record["state"].get("value").is_some());
        // Documented per-record budget of the published component reporters.
        ensure!(serde_json::to_vec(record)?.len() <= 32 * 1024);
    }
    // Dropping the reporting futures above is the host's cancellation mechanism.
    tokio::time::sleep(Duration::from_secs(6)).await;
    ensure!(
        capture.0.lock().len() == records.len(),
        "cancelled reporters kept emitting"
    );
    Ok(())
}

async fn wait_for_components(capture: &Capture) -> Result<(), tokio::time::error::Elapsed> {
    tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            let complete = {
                let records = capture.0.lock();
                ["chain", "graph", "transport"].iter().all(|name| {
                    records.iter().any(|record| {
                        record["component"] == *name
                            && record["sequence"].as_u64().is_some_and(|sequence| sequence >= 1)
                    })
                })
            };
            if complete {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
}
