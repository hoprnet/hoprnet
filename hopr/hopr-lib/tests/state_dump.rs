//! Exercises the production builder, transport producers and INFO reporter in an emulated node cluster.
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{Context, ensure};
use futures::{AsyncReadExt, AsyncWriteExt, StreamExt};
use hopr_lib::{
    api::{
        chain::{ChainReadChannelOperations, ChannelSelector},
        node::HasChainApi,
    },
    testing::fixtures::{TestNodeConfig, build_role_cluster, cluster_fixture},
};
use parking_lot::Mutex;
use serde_json::Value;

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<Value>>>, Arc<std::sync::atomic::AtomicUsize>);
struct Visitor<'a>(&'a Capture);
impl tracing::field::Visit for Visitor<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "state"
            && let Some(path) = std::env::var_os("HOPR_STATE_DUMP_TEST_OUTPUT")
        {
            use std::io::Write;
            if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
                let _ = writeln!(file, "{value:?}");
            }
        }
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

#[test]
fn node_emits_bounded_info_state_with_real_producers_and_supports_disabling() -> anyhow::Result<()> {
    let Some(mode) = std::env::var_os("HOPR_STATE_DUMP_TEST_CHILD") else {
        for (mode, interval) in [("enabled", "5"), ("disabled", "0"), ("roles", "5")] {
            let mut child = std::process::Command::new(std::env::current_exe()?)
                .arg("--exact")
                .arg("node_emits_bounded_info_state_with_real_producers_and_supports_disabling")
                .arg("--nocapture")
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
            let addresses = [
                cluster.entry.address(),
                cluster.relays[0].address(),
                cluster.exit.address(),
            ]
            .map(serde_json::to_value)
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
            tokio::time::timeout(Duration::from_secs(20), async {
                loop {
                    let all_reported = {
                        let records = capture.0.lock();
                        addresses.iter().all(|address| {
                            records.iter().any(|record| {
                                &record["node_address"] == address
                                    && record["sequence"].as_u64().is_some_and(|sequence| sequence > 0)
                            })
                        })
                    };
                    if all_reported {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            })
            .await
            .context("entry, relay and exit must each run their reporter")?;
            Ok(())
        });
    }
    let disabled = mode == "disabled";
    eprintln!("starting {mode:?} state-dump cluster");
    let cluster = cluster_fixture(vec![
        TestNodeConfig {
            idle_timeout_ms: 30_000,
            ..Default::default()
        };
        if disabled { 2 } else { 3 }
    ]);
    eprintln!("state-dump cluster ready");
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        if disabled {
            tokio::time::sleep(Duration::from_secs(6)).await;
            ensure!(capture.0.lock().is_empty(), "disabled reporting emitted a state record");
            return Ok(());
        }
        let source = &cluster[0];
        let middle = &cluster[1];
        let destination = &cluster[2];
        let _channels = cluster
            .open_channels(
                &[&[source, middle, destination], &[destination, middle, source]],
                "20000 wxHOPR".parse()?,
            )
            .await?;
        let chain = source.inner().chain_api();
        let expected_channels = chain
            .stream_channels(ChannelSelector::default().with_source(source.address()))?
            .count()
            .await
            + chain
                .stream_channels(ChannelSelector::default().with_destination(source.address()))?
                .count()
                .await;
        ensure!(expected_channels > 0);
        eprintln!("state-dump channels ready: {expected_channels} source-visible local channels");
        let mut session = cluster.create_session(&[source, middle, destination]).await?;
        eprintln!("state-dump session ready");
        let payload = [42u8; 32];
        let mut echoed = [0u8; 32];
        tokio::time::timeout(Duration::from_secs(30), async {
            session.write_all(&payload).await?;
            session.flush().await?;
            session.read_exact(&mut echoed).await?;
            anyhow::Ok(())
        })
        .await
        .context("session echo deadline")??;
        ensure!(payload == echoed, "session echo did not return");
        let node_address = serde_json::to_value(source.address())?;
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            let records = capture.0.lock().clone();
            let ours: Vec<_> = records.iter().filter(|r| r["node_address"] == node_address).collect();
            let resolutions = ours
                .iter()
                .any(|r| r["routing"]["value"]["totals"]["succeeded"].as_u64().unwrap_or(0) > 0);
            let searches = ours
                .iter()
                .any(|r| r["routing"]["value"]["totals"]["searches"].as_u64().unwrap_or(0) > 0);
            let minted = ours
                .iter()
                .any(|r| r["routing"]["value"]["surb_totals"]["minted"].as_u64().unwrap_or(0) > 0);
            let returned = ours
                .iter()
                .any(|r| r["routing"]["value"]["surb_totals"]["returned"].as_u64().unwrap_or(0) > 0);
            let probes = ours.iter().any(|r| {
                r["routing"]["value"]["probes"]["neighbor"]["succeeded"]
                    .as_u64()
                    .unwrap_or(0)
                    + r["routing"]["value"]["probes"]["neighbor"]["timed_out"]
                        .as_u64()
                        .unwrap_or(0)
                    > 0
            });
            let complete = ours.iter().rev().find(|r| {
                r["channels"]["status"] == "ok"
                    && r["channels"]["value"]["total"].as_u64() == Some(expected_channels as u64)
            });
            if resolutions && searches && minted && returned && probes && complete.is_some() {
                let record = complete.context("complete channel section matching source inventory")?;
                ensure!(record["safe_balance"]["status"] == "ok");
                ensure!(record["safe_allowance"]["status"] == "ok");
                ensure!(record["channels"]["value"]["total"].as_u64() == Some(expected_channels as u64));
                ensure!(record["channels"]["value"]["funding_value"].is_string());
                ensure!(record["channels"]["value"]["peers"]["total"].as_u64().unwrap_or(0) >= 1);
                ensure!(record["connected_peers"].as_u64().unwrap_or(0) >= 1);
                ensure!(record["announced_peers"]["value"] == 2);
                ensure!(record["routing"]["value"]["probes"].is_object());
                for record in &records {
                    ensure!(serde_json::to_vec(&record)?.len() <= hopr_chain_connector::state_dump::MAX_SNAPSHOT_BYTES);
                }
                let sequence = record["sequence"].as_u64().context("sequence")?;
                // No further application writes: verify the timer continues to emit.
                tokio::time::timeout(Duration::from_secs(12), async {
                    loop {
                        if capture.0.lock().iter().any(|r| {
                            r["node_address"] == node_address && r["sequence"].as_u64().is_some_and(|s| s > sequence)
                        }) {
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                })
                .await
                .context("idle interval did not emit")?;
                break;
            }
            ensure!(
                Instant::now() < deadline,
                "missing real producer observations: resolutions={resolutions} searches={searches} minted={minted} \
                 returned={returned}; records={ours:?}"
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        Ok(())
    })
}
