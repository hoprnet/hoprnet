//! End-to-end PIX multi-cycle session test (1-hop).
//!
//! Establishes a 1-hop session between Entry and Exit with PIX enabled, keeps
//! symmetric traffic flowing in the background, and observes the PIX event cycle
//! repeat multiple times:
//!
//!   1. [Entry] `NewDepositAddress`   — deposit address generated
//!   2. [Exit]  `DepositAddressReceived` — deposit needed, notifier provided
//!   3. [Test]  Signal deposit via notifier
//!   4. [Exit]  `PrivateKeyRecovered` — quota exhausted, key recovered
//!   → SessionManager requests next SSA → goto 1

use hopr_lib::testing::fixtures::{
    MINIMUM_INCOMING_WIN_PROB, TEST_GLOBAL_TIMEOUT, TestNodeConfig, chain_propagation_delay, cluster_fixture,
};

#[cfg(feature = "session-client")]
use {
    anyhow::Context,
    futures::{AsyncReadExt, AsyncWriteExt, SinkExt, StreamExt},
    hopr_api::types::primitive::prelude::HoprBalance,
    hopr_chain_connector::blokli_client::BlokliQueryClient,
    hopr_lib::{
        api::node::{HasExitIncentivization, HoprSessionClientOperations, PixEvent},
        testing::hopr::ChannelGuard,
        exports::{
            network::types::prelude::{IpOrHost, SealedHost},
            transport::{SessionCapability, SessionTarget},
            transport::session::IncomingSessionPixConfig,
        },
        HoprSessionClientConfig,
    },
    rstest::rstest,
    serial_test::serial,
    std::{str::FromStr, time::Duration},
};

const FUNDING_AMOUNT: &str = "15000 wxHOPR";

// PIX params: 8 polys × 2 shares × ~1440 bytes = ~23 KB per SSA cycle
const PIX_POLYS: u16 = 8;
const PIX_SHARES: u16 = 2;

#[cfg(feature = "session-client")]
#[rstest]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
/// 1-hop PIX multi-cycle session test.
///
/// Creates a 3-node cluster (Entry, Relay, Exit). The Exit accepts tiny PIX quotas.
/// Keeps symmetric 32-byte traffic flowing Entry↔Exit while observing the PIX event
/// cycle repeat 3 times.
async fn capture_one_hop_pix_session() -> anyhow::Result<()> {
    // ── Cluster: Exit gets custom PIX config with low quota_range ──────────
    let cluster = cluster_fixture(vec![
        TestNodeConfig {
            win_prob: 1.0,

            // Entry needs PIX global config matching the session-negotiated (2,2)
            pix_global_config: Some(hopr_lib::exports::transport::config::PixGlobalConfig {
                num_ssa_parts: 8,
                ssa_part_size: 2,
                additional_shares: 2,
                ..Default::default()
            }),
            ..Default::default()
        }, // src (Entry):   win_prob=1.0
        TestNodeConfig::with_probability(MINIMUM_INCOMING_WIN_PROB), // relay:         win_prob=0.2
        TestNodeConfig {
            win_prob: 1.0,
            incoming_pix_config: Some(IncomingSessionPixConfig {
                quota_range: 0..=100_000,
                enforce_pix: false,
                max_ssa_delivery_time: Duration::from_secs(10),
                max_deposit_wait: Duration::from_secs(60),
                ..Default::default()
            }),
            idle_timeout_ms: Duration::from_secs(90).as_millis() as u64,
            ..Default::default()
        }, // dst (Exit):   win_prob=1.0, custom PIX config
    ]);
    let src = &cluster[0];
    let relay = &cluster[1];
    let dst = &cluster[2];

    // ── Open bidirectional channels ───────────────────────────────────────
    tracing::info!("opening channels");
    let funding = FUNDING_AMOUNT.parse::<HoprBalance>()?;
    let mut channels = Vec::new();
    for (from, to) in [(src, relay), (relay, dst), (dst, relay), (relay, src)] {
        channels.push(
            ChannelGuard::open_channel_between_nodes(from.instance.clone(), to.instance.clone(), funding).await?,
        );
    }
    let chain_info = cluster.chain_client.query_chain_info().await?;
    tracing::info!("waiting for channel graph");
    cluster
        .wait_for_channel_graph(src, channels.len(), chain_propagation_delay(&chain_info) * 6)
        .await?;
    tracing::info!("channel graph ready");

    // ── Subscribe to PixEvent streams BEFORE creating the session ─────────
    tracing::info!("subscribing to PIX events");
    let mut entry_events = Box::pin(src.inner().subscribe_pix_events());
    let mut exit_events = Box::pin(dst.inner().subscribe_pix_events());

    // ── Establish PIX-enabled session: Entry → Exit, 1-hop ────────────────
    tracing::info!("establishing PIX session");
    let connect_fut = {
        let src = src.inner();
        let dst_addr = dst.address();
        let ip = IpOrHost::from_str(":0")?;
        async move {
            src.connect_to(
                dst_addr,
                SessionTarget::UdpStream(SealedHost::Plain(ip)),
                HoprSessionClientConfig {
                    forward_path: 1.try_into().unwrap(),
                    return_path: 1.try_into().unwrap(),
                    capabilities: (SessionCapability::Segmentation
                        | SessionCapability::NoRateControl
                        | SessionCapability::UsePIX)
                        .into(),
                    pseudonym: None,
                    surb_management: None,
                    always_max_out_surbs: false,
                    pix_ssa_quota: Some((PIX_POLYS, PIX_SHARES)),
                },
            )
            .await
        }
    };
    let (session, _) = tokio::time::timeout(Duration::from_secs(120), connect_fut)
        .await
        .context("session connection timed out after 120s")??;
    tracing::info!("session established");

    // ── Background data task: keep traffic flowing symmetrically ──────────
    let bg_handle = tokio::spawn(async move {
        let (mut rd, mut wr) = session.split();
        loop {
            let msg = hopr_lib::api::types::crypto_random::random_bytes::<32>();
            if let Err(e) = wr.write_all(&msg).await {
                tracing::warn!(%e, "bg write_all failed");
                break;
            }
            if let Err(e) = wr.flush().await {
                tracing::warn!(%e, "bg flush failed");
                break;
            }
            let mut echoed = vec![0u8; 32];
            if rd.read_exact(&mut echoed).await.is_err() {
                break;
            }
        }
        tracing::info!("bg task exited");
    });

    // ── Observe PIX event cycles ──────────────────────────────────────────
    let target_cycles = 3u32;
    let mut new_deposit_count = 0u32;
    let mut pk_recovered_count = 0u32;

    loop {
        tokio::select! {
            Some(event) = entry_events.next() => {
                match event {
                    PixEvent::NewDepositAddress(data) => {
                        new_deposit_count += 1;
                        tracing::info!(
                            new_deposit_count,
                            id = ?data.id,
                            quota = data.quota,
                            "Entry: NewDepositAddress"
                        );
                    }
                    other => {
                        anyhow::bail!("unexpected Entry PixEvent: {other:?}");
                    }
                }
            }
            Some(event) = exit_events.next() => {
                match event {
                    PixEvent::DepositAddressReceived(data) => {
                        tracing::info!(
                            id = ?data.id,
                            quota = data.quota,
                            "Exit: DepositAddressReceived"
                        );
                        // Signal deposit immediately to abort the kill switch
                        if let Some(mut notifier) = data.deposit_updated {
                            notifier
                                .send((data.id, HoprBalance::new_base(1)))
                                .await
                                .context("failed to signal deposit via notifier")?;
                            tracing::info!(id = ?data.id, "deposit signaled");
                        }
                    }
                    PixEvent::PrivateKeyRecovered(data) => {
                        pk_recovered_count += 1;
                        tracing::info!(pk_recovered_count, id = ?data.id, "Exit: PrivateKeyRecovered");
                    }
                    other => {
                        anyhow::bail!("unexpected Exit PixEvent: {other:?}");
                    }
                }
            }
        }

        if pk_recovered_count >= target_cycles {
            tracing::info!(target_cycles, "all PIX cycles completed");
            break;
        }
    }

    // ── Assert minimum event counts ───────────────────────────────────────
    assert!(
        new_deposit_count >= target_cycles,
        "expected at least {target_cycles} NewDepositAddress on Entry, got {new_deposit_count}"
    );
    assert_eq!(
        pk_recovered_count, target_cycles,
        "expected {target_cycles} PrivateKeyRecovered on Exit, got {pk_recovered_count}"
    );

    // ── Stop background data task and close channels ──────────────────────
    bg_handle.abort();

    for guard in channels {
        guard.try_close_channels_all_channels().await?;
    }

    tracing::info!("PIX multi-cycle session test PASSED");
    Ok(())
}
