//! End-to-end PIX multi-cycle session test (n-hop).
//!
//! Establishes an n-hop session between Entry and Exit with PIX enabled, keeps
//! symmetric traffic flowing in the background, and observes the PIX event cycle
//! repeat multiple times:
//!
//!   1. [Entry] `NewDepositAddress`   — deposit address generated
//!   2. [Exit]  `DepositAddressReceived` — deposit needed, notifier provided
//!   3. [Test]  Signal deposit via notifier
//!   4. [Exit]  `PrivateKeyRecovered` — quota exhausted, key recovered
//!   → SessionManager requests next SSA → goto 1

use hopr_lib::testing::fixtures::{
    MINIMUM_INCOMING_WIN_PROB, TEST_GLOBAL_TIMEOUT, TestNodeConfig, build_role_cluster, chain_propagation_delay,
};
#[cfg(feature = "session-client")]
use {
    anyhow::Context,
    futures::{AsyncReadExt, AsyncWriteExt, SinkExt, StreamExt},
    hopr_api::types::primitive::prelude::HoprBalance,
    hopr_chain_connector::blokli_client::BlokliQueryClient,
    hopr_lib::{
        HoprSessionClientConfig,
        api::node::{
            HasChainApi, HasExitIncentivization, HoprSessionClientOperations, IncentiveChannelOperations, PixEvent,
        },
        exports::{
            network::types::prelude::{IpOrHost, SealedHost},
            transport::session::IncomingSessionPixConfig,
            transport::{SessionCapability, SessionTarget},
        },
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
#[case(1)]
#[case(2)]
#[case(3)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
/// n-hop PIX multi-cycle session test.
///
/// Creates a (n+2)-node role-typed cluster (Entry, N relays, Exit) where each
/// node is built with the correct transport role. The Exit accepts tiny PIX
/// quotas. Keeps symmetric 32-byte traffic flowing Entry↔Exit while observing
/// the PIX event cycle repeat 3 times.
async fn capture_n_hop_pix_session(#[case] hops: usize) -> anyhow::Result<()> {
    // 2-hop and 3-hop tests are too slow under coverage instrumentation
    #[allow(unexpected_cfgs)]
    if cfg!(coverage) && hops > 1 {
        return Ok(());
    }

    // ── Role-typed cluster: Entry + N relays + Exit ─────────────────────────
    let cluster = build_role_cluster(
        TestNodeConfig {
            win_prob: 1.0,
            // Entry needs PIX global config matching session-negotiated (2,2)
            pix_global_config: Some(hopr_lib::exports::transport::config::PixGlobalConfig {
                num_ssa_parts: 8,
                ssa_part_size: 2,
                additional_shares: 2,
                ..Default::default()
            }),
            ..Default::default()
        }, // Entry: win_prob=1.0
        vec![TestNodeConfig::with_probability(MINIMUM_INCOMING_WIN_PROB); hops], // N relays: win_prob=0.2
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
        }, /* Exit: win_prob=1.0, custom PIX
                                                                                  * config */
    )
    .await?;

    // ── Open bidirectional channels along the relay path ───────────────────
    tracing::info!("opening channels");
    let funding = FUNDING_AMOUNT.parse::<HoprBalance>()?;

    // Helper macro: open channel from `$from` to `$to` using IncentiveChannelOperations
    macro_rules! open_chan {
        ($from:expr, $to:expr) => {{
            IncentiveChannelOperations::open_channel(&*$from.instance, $to.instance.identity().node_address, funding)
                .await
                .context("opening channel must succeed")?;
        }};
    }

    // Forward: Entry → Relay[0] → Relay[1] → ... → Exit
    open_chan!(cluster.entry, cluster.relays[0]);
    for i in 0..hops.saturating_sub(1) {
        open_chan!(cluster.relays[i], cluster.relays[i + 1]);
    }
    open_chan!(cluster.relays[hops - 1], cluster.exit);

    // Backward: Exit → Relay[N-1] → ... → Relay[0] → Entry
    open_chan!(cluster.exit, cluster.relays[hops - 1]);
    for i in (1..hops).rev() {
        open_chan!(cluster.relays[i], cluster.relays[i - 1]);
    }
    open_chan!(cluster.relays[0], cluster.entry);

    let chain_info = cluster.chain_client.query_chain_info().await?;
    tracing::info!("waiting for channel graph");

    // Wait for channels to propagate
    tokio::time::sleep(chain_propagation_delay(&chain_info) * 6).await;

    tracing::info!("channel graph ready");

    // ── Subscribe to PixEvent streams BEFORE creating the session ─────────
    tracing::info!("subscribing to PIX events");
    let mut entry_events = Box::pin(cluster.entry.inner().subscribe_pix_events());
    let mut exit_events = Box::pin(cluster.exit.inner().subscribe_pix_events());

    // ── Establish PIX-enabled session: Entry → Exit, n-hop ────────────────
    tracing::info!("establishing PIX session");
    let routing = hops.try_into()?;
    let connect_fut = {
        let src_inner = cluster.entry.inner();
        let dst_addr = cluster.exit.address();
        let ip = IpOrHost::from_str(":0")?;
        async move {
            src_inner
                .connect_to(
                    dst_addr,
                    SessionTarget::UdpStream(SealedHost::Plain(ip)),
                    HoprSessionClientConfig {
                        forward_path: routing,
                        return_path: routing,
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
            let result = tokio::time::timeout(Duration::from_secs(10), async {
                wr.write_all(&msg).await?;
                wr.flush().await?;
                let mut echoed = vec![0u8; 32];
                rd.read_exact(&mut echoed).await?;
                anyhow::Ok(echoed)
            })
            .await;
            match result {
                Ok(Ok(_echoed)) => {}
                Ok(Err(e)) => {
                    tracing::warn!("bg task failed: {e:?}");
                    break;
                }
                Err(_) => {
                    tracing::warn!("bg task timed out");
                    break;
                }
            }
        }
        tracing::info!("bg task exited");
    });

    // ── Observe PIX event cycles ──────────────────────────────────────────
    let target_cycles = 3u32;
    let mut new_deposit_ids: Vec<hopr_api::node::PixAddressId> = Vec::new();
    let mut deposit_received_ids: Vec<hopr_api::node::PixAddressId> = Vec::new();
    let mut pk_recovered_ids: Vec<hopr_api::node::PixAddressId> = Vec::new();

    loop {
        tokio::select! {
            Some(event) = entry_events.next() => {
                match event {
                    PixEvent::NewDepositAddress(data) => {
                        assert!(
                            !new_deposit_ids.contains(&data.id),
                            "duplicate NewDepositAddress for same SSA — expected distinct cycles, got {:?}",
                            data.id,
                        );
                        new_deposit_ids.push(data.id);
                        tracing::info!(id = ?data.id, quota = data.quota, "Entry: NewDepositAddress");
                    }
                    other => {
                        anyhow::bail!("unexpected Entry PixEvent: {other:?}");
                    }
                }
            }
            Some(event) = exit_events.next() => {
                match event {
                    PixEvent::DepositAddressReceived(data) => {
                        tracing::info!(id = ?data.id, quota = data.quota, "Exit: DepositAddressReceived");
                        // Signal deposit immediately to abort the kill switch
                        if let Some(mut notifier) = data.deposit_updated {
                            notifier
                                .send((data.id, HoprBalance::new_base(1)))
                                .await
                                .context("failed to signal deposit via notifier")?;
                            tracing::info!(id = ?data.id, "deposit signaled");
                        }
                        deposit_received_ids.push(data.id);
                    }
                    PixEvent::PrivateKeyRecovered(data) => {
                        assert!(
                            !pk_recovered_ids.contains(&data.id),
                            "duplicate PrivateKeyRecovered for same SSA — expected distinct cycles, got {:?}",
                            data.id,
                        );
                        pk_recovered_ids.push(data.id);
                        tracing::info!(count = pk_recovered_ids.len(), id = ?data.id, "Exit: PrivateKeyRecovered");
                    }
                    other => {
                        anyhow::bail!("unexpected Exit PixEvent: {other:?}");
                    }
                }
            }
        }

        if pk_recovered_ids.len() as u32 >= target_cycles {
            tracing::info!(target_cycles, "all PIX cycles completed");
            break;
        }
    }

    // ── Assert lifecycle SSA ID correlation ───────────────────────────────
    // Every completed SSA cycle must pass through all three lifecycle stages
    // with the same ID: Entry generates a deposit address → Exit observes it
    // → Exit recovers the private key.
    let completed = new_deposit_ids
        .iter()
        .filter(|id| deposit_received_ids.contains(id) && pk_recovered_ids.contains(id))
        .count();
    assert!(
        completed >= target_cycles as usize,
        "expected at least {target_cycles} fully correlated SSA cycles (ID seen in: NewDepositAddress, \
         DepositAddressReceived, AND PrivateKeyRecovered), got {completed}. new_deposit_ids={new_deposit_ids:?}, \
         deposit_received_ids={deposit_received_ids:?}, pk_recovered_ids={pk_recovered_ids:?}",
    );

    // ── Stop background data task ─────────────────────────────────────────
    bg_handle.abort();

    tracing::info!(hops, "PIX multi-cycle session test PASSED");
    Ok(())
}
