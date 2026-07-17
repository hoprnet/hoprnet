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
            transport::session::{IncomingSessionPixConfig, SupervisorConfig},
            transport::{SessionCapability, SessionTarget},
        },
    },
    rstest::rstest,
    serial_test::serial,
    std::{
        str::FromStr,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        time::Duration,
    },
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
                supervisor_cfg: SupervisorConfig {
                    max_ssa_delivery_time: Duration::from_secs(10),
                    max_deposit_wait: Duration::from_secs(60),
                    ..Default::default()
                },
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

        let completed = new_deposit_ids
            .iter()
            .filter(|id| deposit_received_ids.contains(id) && pk_recovered_ids.contains(id))
            .count();
        if completed >= target_cycles as usize {
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

/// Helper: build a role-typed 1-hop cluster with the given Exit PIX config and
/// open bidirectional channels. Returns the cluster and chain info.
#[cfg(feature = "session-client")]
async fn setup_one_hop_cluster(
    exit_cfg: TestNodeConfig,
) -> anyhow::Result<hopr_lib::testing::fixtures::RoleClusterGuard> {
    let cluster = build_role_cluster(
        TestNodeConfig {
            win_prob: 1.0,
            pix_global_config: Some(hopr_lib::exports::transport::config::PixGlobalConfig {
                num_ssa_parts: PIX_POLYS as usize,
                ssa_part_size: PIX_SHARES as usize,
                additional_shares: 2,
                ..Default::default()
            }),
            ..Default::default()
        },
        vec![TestNodeConfig::with_probability(MINIMUM_INCOMING_WIN_PROB); 1],
        exit_cfg,
    )
    .await?;

    let funding = FUNDING_AMOUNT.parse::<HoprBalance>()?;
    macro_rules! open_chan {
        ($from:expr, $to:expr) => {{
            IncentiveChannelOperations::open_channel(&*$from.instance, $to.instance.identity().node_address, funding)
                .await
                .context("opening channel must succeed")?;
        }};
    }

    open_chan!(cluster.entry, cluster.relays[0]);
    open_chan!(cluster.relays[0], cluster.exit);
    open_chan!(cluster.exit, cluster.relays[0]);
    open_chan!(cluster.relays[0], cluster.entry);

    let chain_info = cluster.chain_client.query_chain_info().await?;
    tokio::time::sleep(chain_propagation_delay(&chain_info) * 6).await;

    Ok(cluster)
}

/// Helper: establish a PIX-enabled session from Entry to Exit (1-hop).
#[cfg(feature = "session-client")]
async fn establish_pix_session(
    cluster: &hopr_lib::testing::fixtures::RoleClusterGuard,
) -> anyhow::Result<hopr_lib::exports::transport::HoprSession> {
    let ip = IpOrHost::from_str(":0")?;
    let (session, _) = cluster
        .entry
        .inner()
        .connect_to(
            cluster.exit.address(),
            SessionTarget::UdpStream(SealedHost::Plain(ip)),
            HoprSessionClientConfig {
                forward_path: 1usize.try_into()?,
                return_path: 1usize.try_into()?,
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
        .await?;
    Ok(session)
}

/// Helper: spawn a background data task that writes 32-byte messages and reads
/// back echoes, with a per-read timeout. Sets `session_died` when the read
/// times out or fails.
#[cfg(feature = "session-client")]
fn spawn_data_task(
    session: hopr_lib::exports::transport::HoprSession,
    session_died: Arc<AtomicBool>,
    read_timeout: Duration,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let (mut rd, mut wr) = session.split();
        loop {
            let msg = hopr_lib::api::types::crypto_random::random_bytes::<32>();
            if wr.write_all(&msg).await.is_err() {
                tracing::warn!("bg write_all failed");
                session_died.store(true, Ordering::SeqCst);
                break;
            }
            if wr.flush().await.is_err() {
                tracing::warn!("bg flush failed");
                session_died.store(true, Ordering::SeqCst);
                break;
            }
            let mut echoed = vec![0u8; 32];
            let read_result = tokio::time::timeout(read_timeout, rd.read_exact(&mut echoed)).await;
            if read_result.is_err() || read_result.unwrap().is_err() {
                tracing::warn!("bg read timeout or error");
                session_died.store(true, Ordering::SeqCst);
                break;
            }
        }
    })
}

// =========================================================================
//  Test 1: Deposit timeout — no deposit signals close
// =========================================================================

#[cfg(feature = "session-client")]
#[rstest]
#[case(1)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
/// Verifies that when no deposit is signaled after `DepositAddressReceived`,
/// the PIX supervisor's deposit deadline expires and closes the session.
async fn deposit_timeout_closes_session(#[case] hops: usize) -> anyhow::Result<()> {
    #[allow(unexpected_cfgs)]
    if cfg!(coverage) && hops > 1 {
        return Ok(());
    }

    // Exit with very short deposit wait: 5s
    let cluster = setup_one_hop_cluster(TestNodeConfig {
        win_prob: 1.0,
        incoming_pix_config: Some(IncomingSessionPixConfig {
            quota_range: 0..=100_000,
            enforce_pix: false,
            supervisor_cfg: SupervisorConfig {
                max_ssa_delivery_time: Duration::from_secs(10),
                max_deposit_wait: Duration::from_secs(5),
                ..Default::default()
            },
            ..Default::default()
        }),
        idle_timeout_ms: Duration::from_secs(90).as_millis() as u64,
        ..Default::default()
    })
    .await?;

    let mut entry_events = Box::pin(cluster.entry.inner().subscribe_pix_events());
    let mut exit_events = Box::pin(cluster.exit.inner().subscribe_pix_events());

    let session = establish_pix_session(&cluster).await?;
    tracing::info!("session established");

    // Start the bg data task with a read timeout of 8s.
    let session_died = Arc::new(AtomicBool::new(false));
    let sd = session_died.clone();
    let _bg_handle = spawn_data_task(session, sd, Duration::from_secs(8));

    let mut deposit_address_received = false;

    // Consume events — do NOT signal the deposit.
    loop {
        // Also check the session-died flag periodically via a short sleep
        // so the loop doesn't hang when both event streams are exhausted.
        if session_died.load(Ordering::SeqCst) {
            tracing::info!("session closed (bg task detected) in polling loop");
            break;
        }
        tokio::select! {
            biased;
            _ = tokio::time::sleep(Duration::from_millis(500)) => {
                // Timeout — fall through to the top of the loop.
            }
            Some(event) = entry_events.next() => {
                match event {
                    PixEvent::NewDepositAddress(data) => {
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
                        deposit_address_received = true;
                        tracing::info!(id = ?data.id, quota = data.quota,
                            "Exit: DepositAddressReceived — NOT signaling deposit");
                    }
                    PixEvent::PrivateKeyRecovered(data) => {
                        tracing::info!(id = ?data.id, "Exit: PrivateKeyRecovered (unexpected)");
                    }
                    other => {
                        anyhow::bail!("unexpected Exit PixEvent: {other:?}");
                    }
                }
            }
        }
    }

    assert!(deposit_address_received, "must have received DepositAddressReceived");

    tracing::info!("deposit timeout test PASSED");
    Ok(())
}

// =========================================================================
//  Test 2: Recovery hard deadline — recovery takes too long
// =========================================================================

#[cfg(feature = "session-client")]
#[rstest]
#[case(1)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
/// Verifies that when recovery takes longer than `max_recovery_time`,
/// the PIX supervisor closes the session with RecoveryDeadline.
///
/// Shares only arrive via data-packet acknowledgements. We signal deposit
/// first (no data flowing yet) then wait for the hard deadline to expire
/// before starting a data task. Since no shares were exchanged during the
/// wait, recovery made no progress, and the deadline fires. The bg data
/// task detects the closure through a failed read.
async fn recovery_hard_deadline_closes_session(#[case] hops: usize) -> anyhow::Result<()> {
    #[allow(unexpected_cfgs)]
    if cfg!(coverage) && hops > 1 {
        return Ok(());
    }

    // Exit with very short recovery time: 5s; longer deposit wait (60s) so
    // deposit doesn't timeout before we signal it.
    let cluster = setup_one_hop_cluster(TestNodeConfig {
        win_prob: 1.0,
        incoming_pix_config: Some(IncomingSessionPixConfig {
            quota_range: 0..=100_000,
            enforce_pix: false,
            supervisor_cfg: SupervisorConfig {
                max_ssa_delivery_time: Duration::from_secs(10),
                max_deposit_wait: Duration::from_secs(60),
                max_recovery_time: Duration::from_secs(5),
                max_recovery_idle: Duration::from_secs(60),
                ..Default::default()
            },
            ..Default::default()
        }),
        idle_timeout_ms: Duration::from_secs(120).as_millis() as u64,
        ..Default::default()
    })
    .await?;

    let mut exit_events = Box::pin(cluster.exit.inner().subscribe_pix_events());

    let session = establish_pix_session(&cluster).await?;
    tracing::info!("session established");

    let mut deposit_signaled = false;

    // Wait for DepositAddressReceived and signal deposit — do NOT start
    // a data task yet, so no shares arrive and recovery makes no progress.
    loop {
        tokio::select! {
            biased;
            _ = tokio::time::sleep(Duration::from_millis(500)) => {}
            Some(event) = exit_events.next() => {
                match event {
                    PixEvent::DepositAddressReceived(data) => {
                        tracing::info!(id = ?data.id, quota = data.quota,
                            "Exit: DepositAddressReceived — signaling deposit");
                        if let Some(mut notifier) = data.deposit_updated {
                            notifier
                                .send((data.id, HoprBalance::new_base(1)))
                                .await
                                .context("failed to signal deposit via notifier")?;
                            deposit_signaled = true;
                            tracing::info!(id = ?data.id, "deposit signaled");
                        }
                    }
                    PixEvent::PrivateKeyRecovered(data) => {
                        tracing::info!(id = ?data.id, "Exit: PrivateKeyRecovered (early)");
                    }
                    PixEvent::NewDepositAddress(data) => {
                        tracing::info!(id = ?data.id, quota = data.quota, "Entry: NewDepositAddress");
                    }
                }
            }
        }
        if deposit_signaled {
            break;
        }
    }

    // Wait for the hard deadline to expire (~5s) plus margin, with NO data
    // flowing (no shares → no recovery progress). The deadline fires and
    // the supervisor closes the session.
    tracing::info!("sleeping for recovery hard deadline...");
    tokio::time::sleep(Duration::from_secs(8)).await;

    // After the deadline, start a data task. The session socket should be
    // in the process of closing, so the read will eventually time out and
    // set session_died.
    let session_died = Arc::new(AtomicBool::new(false));
    let sd = session_died.clone();
    let _bg_handle = spawn_data_task(session, sd, Duration::from_secs(5));

    // Poll for session close.
    for _ in 0..20 {
        // max 10s polling (20 × 500ms)
        if session_died.load(Ordering::SeqCst) {
            tracing::info!("session closed detected via bg task");
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    assert!(
        session_died.load(Ordering::SeqCst),
        "session should close after recovery hard deadline"
    );
    assert!(deposit_signaled, "must have signaled deposit");
    tracing::info!("recovery hard deadline test PASSED");
    Ok(())
}

// =========================================================================
//  Note: Recovery idle timeout
// =========================================================================
//
// A recovery-idle integration test is not feasible with the current test
// infrastructure. The idle deadline is service-gated: it only produces a
// close when `served_total > served_total_at_last_progress`. Meeting that
// requires sending data packets that increase the service counter WITHOUT
// advancing recovery progress — but PIX shares are embedded in data-packet
// acknowledgements, so every packet that flows also advances recovery.
//
// In the fast local cluster (win_prob=1.0) recovery completes in <5s of
// continuous traffic, while max_recovery_idle minimum is 30s (constrained
// by >= reconstructor's max_ack_await_time). That window is too wide for
// any integration-level idle-close test to be reliable.
//
// The RecoveryIdle code path IS covered by unit tests in supervisor.rs.

// =========================================================================
//  Test 4: PIX-enforced rejection — Exit requires PIX, client omits it
// =========================================================================

#[cfg(feature = "session-client")]
#[rstest]
#[case(1)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
/// Verifies that connecting without `UsePIX` capability to an Exit
/// configured with `enforce_pix: true` is rejected.
async fn enforce_pix_rejects_non_pix_session(#[case] hops: usize) -> anyhow::Result<()> {
    #[allow(unexpected_cfgs)]
    if cfg!(coverage) && hops > 1 {
        return Ok(());
    }

    // Exit enforces PIX. Entry does NOT include UsePIX.
    let cluster = build_role_cluster(
        TestNodeConfig {
            win_prob: 1.0,
            // No pix_global_config needed — we won't use PIX.
            ..Default::default()
        },
        vec![TestNodeConfig::with_probability(MINIMUM_INCOMING_WIN_PROB); hops],
        TestNodeConfig {
            win_prob: 1.0,
            incoming_pix_config: Some(IncomingSessionPixConfig {
                enforce_pix: true,
                ..Default::default()
            }),
            idle_timeout_ms: Duration::from_secs(5).as_millis() as u64,
            ..Default::default()
        },
    )
    .await?;

    let funding = FUNDING_AMOUNT.parse::<HoprBalance>()?;
    macro_rules! open_chan {
        ($from:expr, $to:expr) => {{
            IncentiveChannelOperations::open_channel(&*$from.instance, $to.instance.identity().node_address, funding)
                .await
                .context("opening channel must succeed")?;
        }};
    }

    open_chan!(cluster.entry, cluster.relays[0]);
    open_chan!(cluster.relays[0], cluster.exit);
    open_chan!(cluster.exit, cluster.relays[0]);
    open_chan!(cluster.relays[0], cluster.entry);

    let chain_info = cluster.chain_client.query_chain_info().await?;
    tokio::time::sleep(chain_propagation_delay(&chain_info) * 6).await;

    // Attempt connection WITHOUT UsePIX capability.
    let ip = IpOrHost::from_str(":0")?;
    let routing: hopr_lib::HopRouting = hops.try_into()?;
    let result = tokio::time::timeout(
        Duration::from_secs(15),
        cluster.entry.inner().connect_to(
            cluster.exit.address(),
            SessionTarget::UdpStream(SealedHost::Plain(ip)),
            HoprSessionClientConfig {
                forward_path: routing,
                return_path: routing,
                capabilities: (SessionCapability::Segmentation | SessionCapability::NoRateControl).into(),
                pseudonym: None,
                surb_management: None,
                always_max_out_surbs: false,
                pix_ssa_quota: None,
            },
        ),
    )
    .await;

    match result {
        Ok(Err(e)) => {
            tracing::info!("connection rejected as expected: {e:?}");
        }
        Ok(Ok((_session, _configurator))) => {
            anyhow::bail!("expected connection rejection but session was established");
        }
        Err(_) => {
            tracing::info!("connection timed out as expected (no PIX-enabled session accepted)");
        }
    }

    tracing::info!("enforce PIX rejection test PASSED");
    Ok(())
}
