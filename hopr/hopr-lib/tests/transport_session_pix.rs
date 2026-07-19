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

#[cfg(feature = "session-client")]
use hopr_lib::testing::fixtures::RoleClusterGuard;
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
            // Require a PIX-specific rejection, not just any error.
            let err_msg = e.to_string();
            assert!(
                err_msg.to_lowercase().contains("pix") || err_msg.to_lowercase().contains("enforce"),
                "expected PIX-related rejection, got: {e:?}"
            );
        }
        Ok(Ok((_session, _configurator))) => {
            anyhow::bail!("expected connection rejection but session was established");
        }
        Err(_) => {
            anyhow::bail!("expected explicit rejection, got timeout");
        }
    }

    tracing::info!("enforce PIX rejection test PASSED");
    Ok(())
}

// =========================================================================
//  Drain test infrastructure
// =========================================================================

use hopr_lib::{
    exports::transport::session::{
        SurbBalancerConfig,
        drain::{DrainOutcome, DrainResult, DrainStopReason, SkipReason, SurbDrainConfig},
    },
    testing::hopr::TestedHopr,
};

/// Spawn a one-way UDP sink that binds a local socket, receives but never
/// replies.  Returns the bound `SocketAddr` and a handle that can be aborted.
fn spawn_one_way_sink() -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
    let bind_addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let socket = std::net::UdpSocket::bind(bind_addr).unwrap();
    let addr = socket.local_addr().unwrap();
    socket.set_nonblocking(true).unwrap();
    let handle = tokio::spawn(async move {
        let mut buf = vec![0u8; 2048];
        let socket = tokio::net::UdpSocket::from_std(socket).unwrap();
        loop {
            if socket.recv_from(&mut buf).await.is_err() {
                break;
            }
        }
    });
    (addr, handle)
}

/// Wait for a drain outcome matching the given predicate on the exit's drain
/// stream.  The exit node must have `SurbDrainConfig::enabled = true`.
async fn expect_drain_outcome(
    exit: &TestedHopr<()>,
    predicate: impl Fn(&DrainOutcome) -> bool,
    timeout: Duration,
) -> anyhow::Result<DrainOutcome> {
    use futures::StreamExt;
    let rx = exit
        .inner()
        .drain_outcome_rx()
        .context("drain_outcome_rx returned None — drainer not active")?;
    let mut stream = rx.into_stream();
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            anyhow::bail!("timed out waiting for drain outcome");
        }
        match tokio::time::timeout(remaining, stream.next()).await {
            Ok(Some(outcome)) if predicate(&outcome) => return Ok(outcome),
            Ok(Some(_)) => continue,
            Ok(None) => anyhow::bail!("drain outcome stream ended"),
            Err(_) => anyhow::bail!("timed out waiting for drain outcome"),
        }
    }
}

/// Wait for `PrivateKeyRecovered` on the given PixEvent stream.
async fn expect_key_recovered(
    exit_events: &mut (impl futures::Stream<Item = PixEvent> + Unpin + Send),
    timeout: Duration,
) -> anyhow::Result<()> {
    use futures::StreamExt;
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            anyhow::bail!("timed out waiting for PrivateKeyRecovered");
        }
        match tokio::time::timeout(remaining, exit_events.next()).await {
            Ok(Some(PixEvent::PrivateKeyRecovered(_))) => return Ok(()),
            Ok(Some(_)) => continue,
            Ok(None) => anyhow::bail!("PixEvent stream ended"),
            Err(_) => anyhow::bail!("timed out waiting for PrivateKeyRecovered"),
        }
    }
}

/// Assert no PrivateKeyRecovered within the given window.
async fn assert_no_key_recovered(
    exit_events: &mut (impl futures::Stream<Item = PixEvent> + Unpin + Send),
    window: Duration,
) {
    use futures::StreamExt;
    tokio::select! {
        biased;
        Some(PixEvent::PrivateKeyRecovered(data)) = exit_events.next() => {
            panic!("unexpected PrivateKeyRecovered during silence window: {data:?}");
        }
        _ = tokio::time::sleep(window) => {}
    }
}

/// Helper: signal deposit via `DepositAddressReceived` and return once the
/// notifier has been sent.  This consumes events so the caller gets a fresh
/// subscription afterwards.
async fn signal_deposit(
    exit_events: &mut (impl futures::Stream<Item = PixEvent> + Unpin + Send),
    amount: HoprBalance,
) -> anyhow::Result<()> {
    use futures::StreamExt;
    loop {
        match exit_events.next().await {
            Some(PixEvent::DepositAddressReceived(data)) => {
                tracing::info!(id = ?data.id, ?amount, "DepositAddressReceived, signaling");
                if let Some(mut notifier) = data.deposit_updated {
                    notifier.send((data.id, amount)).await?;
                }
                return Ok(());
            }
            Some(PixEvent::NewDepositAddress(data)) => {
                tracing::info!(id = ?data.id, "NewDepositAddress (ignored)");
            }
            Some(other) => {
                tracing::warn!("unexpected event in signal_deposit: {other:?}");
            }
            None => anyhow::bail!("PixEvent stream ended before DepositAddressReceived"),
        }
    }
}

/// Connect to the Exit with a session that tunnels to a one-way UDP sink.
/// Entry-side SURB preloading is configured to 64.
async fn connect_to_sink(
    cluster: &RoleClusterGuard,
    sink_addr: std::net::SocketAddr,
) -> anyhow::Result<hopr_lib::exports::transport::HoprSession> {
    let ip = IpOrHost::from_str(&format!("0.0.0.0:{}", sink_addr.port()))?;
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
                surb_management: Some(SurbBalancerConfig {
                    target_surb_buffer_size: 64,
                    max_surbs_per_sec: 100,
                    ..Default::default()
                }),
                always_max_out_surbs: false,
                pix_ssa_quota: Some((PIX_POLYS, PIX_SHARES)),
            },
        )
        .await?;
    Ok(session)
}

// =========================================================================
//  Test T1: Drain recovers deposit after Entry close (happy path)
// =========================================================================

#[cfg(feature = "session-client")]
#[rstest]
#[case(1)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
/// Verifies that after the Entry drops the session, the Exit's drainer
/// consumes leftover SURBs, recovers the SSA, and emits PrivateKeyRecovered
/// followed by DrainFinished(AllRecovered).
async fn drain_recovers_deposit_after_entry_close(#[case] hops: usize) -> anyhow::Result<()> {
    let (_sink_addr, _sink_handle) = spawn_one_way_sink();

    let cluster = setup_one_hop_cluster(TestNodeConfig {
        win_prob: 1.0,
        incoming_pix_config: Some(IncomingSessionPixConfig {
            quota_range: 0..=100_000,
            enforce_pix: false,
            supervisor_cfg: SupervisorConfig {
                max_ssa_delivery_time: Duration::from_secs(10),
                max_deposit_wait: Duration::from_secs(60),
                max_recovery_time: Duration::from_secs(10),
                max_recovery_idle: Duration::from_secs(60),
                ..Default::default()
            },
            drain_cfg: SurbDrainConfig {
                enabled: true,
                max_drain_time: Duration::from_secs(60),
                ack_grace: Duration::from_secs(30),
                drain_rate_packets_per_sec: 50,
                cost_safety_factor: 1.0,
                ..Default::default()
            },
            ..Default::default()
        }),
        idle_timeout_ms: Duration::from_secs(120).as_millis() as u64,
        ..Default::default()
    })
    .await?;

    let mut exit_events = Box::pin(cluster.exit.inner().subscribe_pix_events());
    let session = connect_to_sink(&cluster, _sink_addr).await?;
    tracing::info!("T1: session established");

    signal_deposit(&mut exit_events, HoprBalance::new_base(1_000_000)).await?;

    // Allow deposit confirmation to propagate before dropping the session.
    // Without this wait, the deposit channel may close unconfirmed, leaving
    // no funded SSA for the drainer to recover.
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Drop the session at Entry.
    // The Exit supervisor will close via the hard deadline (max_recovery_time).
    drop(session);
    tracing::info!("T1: session dropped at Entry");

    // Wait for the supervisor's hard deadline to fire and the drain to complete.
    // Expect PrivateKeyRecovered from the drain.
    expect_key_recovered(&mut exit_events, Duration::from_secs(60)).await?;
    tracing::info!("T1: PrivateKeyRecovered received");

    // Expect DrainFinished(AllRecovered).
    let outcome = expect_drain_outcome(
        &cluster.exit,
        |o| matches!(&o.result, DrainResult::Finished(DrainStopReason::AllRecovered)),
        Duration::from_secs(60),
    )
    .await?;
    tracing::info!(packets = outcome.packets_sent, "T1: drain AllRecovered");
    assert!(outcome.ssas_recovered >= 1, "at least one SSA recovered");

    tracing::info!("T1 drain happy path PASSED");
    Ok(())
}

// =========================================================================
//  Test T2: Drain after supervisor close (RecoveryDeadline)
// =========================================================================

#[cfg(feature = "session-client")]
#[rstest]
#[case(1)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
/// Verifies that when the supervisor closes with RecoveryDeadline (benign),
/// the drainer still recovers the deposit.
async fn drain_after_supervisor_close(#[case] hops: usize) -> anyhow::Result<()> {
    let (_sink_addr, _sink_handle) = spawn_one_way_sink();

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
            drain_cfg: SurbDrainConfig {
                enabled: true,
                max_drain_time: Duration::from_secs(60),
                ack_grace: Duration::from_secs(30),
                drain_rate_packets_per_sec: 50,
                cost_safety_factor: 1.0,
                ..Default::default()
            },
            ..Default::default()
        }),
        idle_timeout_ms: Duration::from_secs(120).as_millis() as u64,
        ..Default::default()
    })
    .await?;

    let mut exit_events = Box::pin(cluster.exit.inner().subscribe_pix_events());
    let _session = connect_to_sink(&cluster, _sink_addr).await?;
    tracing::info!("T2: session established");

    // Signal deposit, then let session sit idle — the hard deadline fires.
    signal_deposit(&mut exit_events, HoprBalance::new_base(1_000_000)).await?;

    tracing::info!("T2: waiting for supervisor hard deadline...");
    tokio::time::sleep(Duration::from_secs(10)).await;

    // Supervisor has closed the session.  Handover arrived at the drainer.
    drop(_session); // also drop Entry side
    tracing::info!("T2: session dropped at Entry");

    // The drainer should now recover.
    expect_key_recovered(&mut exit_events, Duration::from_secs(60)).await?;
    tracing::info!("T2: PrivateKeyRecovered received");

    let outcome = expect_drain_outcome(
        &cluster.exit,
        |o| matches!(&o.result, DrainResult::Finished(DrainStopReason::AllRecovered)),
        Duration::from_secs(60),
    )
    .await?;
    tracing::info!(packets = outcome.packets_sent, "T2: drain AllRecovered");
    assert!(outcome.ssas_recovered >= 1, "at least one SSA recovered");

    tracing::info!("T2 drain after supervisor close PASSED");
    Ok(())
}

// =========================================================================
//  Test T3: Drain stops on first unverifiable share (zero tolerance)
// =========================================================================

#[cfg(feature = "session-client")]
#[rstest]
#[case(1)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
/// Entry generates corrupted PIX shares.  The session closes cleanly (one-way
/// sink means no shares are verified during the session, so no fault close).
/// Post-closure drain hits the first corrupted share and stops immediately.
async fn drain_stops_on_unverifiable_share(#[case] hops: usize) -> anyhow::Result<()> {
    let (_sink_addr, _sink_handle) = spawn_one_way_sink();

    // Build the cluster ourselves so the Entry has corrupt_pix_shares.
    let cluster = build_role_cluster(
        TestNodeConfig {
            win_prob: 1.0,
            corrupt_pix_shares: true,
            pix_global_config: Some(hopr_lib::exports::transport::config::PixGlobalConfig {
                num_ssa_parts: PIX_POLYS as usize,
                ssa_part_size: PIX_SHARES as usize,
                additional_shares: 2,
                ..Default::default()
            }),
            ..Default::default()
        },
        vec![TestNodeConfig::with_probability(MINIMUM_INCOMING_WIN_PROB); hops],
        TestNodeConfig {
            win_prob: 1.0,
            incoming_pix_config: Some(IncomingSessionPixConfig {
                quota_range: 0..=100_000,
                enforce_pix: false,
                supervisor_cfg: SupervisorConfig {
                    max_ssa_delivery_time: Duration::from_secs(10),
                    max_deposit_wait: Duration::from_secs(60),
                    max_recovery_time: Duration::from_secs(10),
                    max_recovery_idle: Duration::from_secs(60),
                    ..Default::default()
                },
                drain_cfg: SurbDrainConfig {
                    enabled: true,
                    max_drain_time: Duration::from_secs(60),
                    ack_grace: Duration::from_secs(30),
                    drain_rate_packets_per_sec: 50,
                    cost_safety_factor: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            }),
            idle_timeout_ms: Duration::from_secs(120).as_millis() as u64,
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

    let mut exit_events = Box::pin(cluster.exit.inner().subscribe_pix_events());
    let session = connect_to_sink(&cluster, _sink_addr).await?;
    tracing::info!("T3: session established with corrupted shares");

    signal_deposit(&mut exit_events, HoprBalance::new_base(1_000_000)).await?;

    // Allow deposit confirmation to propagate before dropping the session
    // (same race as T1 — the deposit observer has a 100ms initial delay).
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Drop the session at Entry — clean close since no shares were verified.
    drop(session);
    tracing::info!("T3: session dropped at Entry");

    // Expect DrainFinished(UnverifiableShare).
    let outcome = expect_drain_outcome(
        &cluster.exit,
        |o| matches!(&o.result, DrainResult::Finished(DrainStopReason::UnverifiableShare)),
        Duration::from_secs(60),
    )
    .await?;
    tracing::info!(packets = outcome.packets_sent, "T3: drain UnverifiableShare");
    assert_eq!(outcome.ssas_recovered, 0, "no SSA recovered with corrupted shares");

    assert_no_key_recovered(&mut exit_events, Duration::from_secs(15)).await;
    tracing::info!("T3: confirmed no PrivateKeyRecovered");

    tracing::info!("T3 drain unverifiable share PASSED");
    Ok(())
}

// =========================================================================
//  Test T4: Drain skipped when uneconomical
// =========================================================================

#[cfg(feature = "session-client")]
#[rstest]
#[case(1)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
/// The deposit (1 base unit) is far below the ticket cost needed to drain the
/// deficit.  The drain should be skipped with UneconomicalDeposit.
async fn drain_skipped_when_uneconomical(#[case] hops: usize) -> anyhow::Result<()> {
    let (_sink_addr, _sink_handle) = spawn_one_way_sink();

    let cluster = setup_one_hop_cluster(TestNodeConfig {
        win_prob: 1.0,
        incoming_pix_config: Some(IncomingSessionPixConfig {
            quota_range: 0..=100_000,
            enforce_pix: false,
            supervisor_cfg: SupervisorConfig {
                max_ssa_delivery_time: Duration::from_secs(10),
                max_deposit_wait: Duration::from_secs(60),
                max_recovery_time: Duration::from_secs(10),
                max_recovery_idle: Duration::from_secs(60),
                ..Default::default()
            },
            drain_cfg: SurbDrainConfig {
                enabled: true,
                max_drain_time: Duration::from_secs(60),
                ack_grace: Duration::from_secs(30),
                drain_rate_packets_per_sec: 50,
                cost_safety_factor: 1.0,
                ..Default::default()
            },
            ..Default::default()
        }),
        idle_timeout_ms: Duration::from_secs(120).as_millis() as u64,
        ..Default::default()
    })
    .await?;

    let mut exit_events = Box::pin(cluster.exit.inner().subscribe_pix_events());
    let session = connect_to_sink(&cluster, _sink_addr).await?;
    tracing::info!("T4: session established");

    // Signal a tiny deposit (1 base unit — far below price × deficit).
    signal_deposit(&mut exit_events, HoprBalance::new_base(1)).await?;

    // Allow deposit confirmation to propagate before dropping the session
    // (same race as T1 — the deposit observer has a 100ms initial delay).
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Drop the session at Entry. The Exit supervisor will close via the
    // hard deadline (max_recovery_time).
    drop(session);
    tracing::info!("T4: session dropped at Entry");

    let outcome = expect_drain_outcome(
        &cluster.exit,
        |o| matches!(&o.result, DrainResult::Skipped(SkipReason::UneconomicalDeposit)),
        Duration::from_secs(30),
    )
    .await?;
    tracing::info!(packets = outcome.packets_sent, "T4: drain skipped");
    assert_eq!(outcome.packets_sent, 0, "no packets sent for skipped drain");

    assert_no_key_recovered(&mut exit_events, Duration::from_secs(10)).await;

    tracing::info!("T4 uneconomical drain PASSED");
    Ok(())
}

// =========================================================================
//  Test T5: Drain skipped on insufficient SURBs
// =========================================================================

#[cfg(feature = "session-client")]
#[rstest]
#[case(1)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
/// No SURB management and no data writes after establishment — only the
/// handful of initiation-time SURBs exist (≪ 16).  Drain should be
/// skipped with InsufficientSurbs.
async fn drain_skipped_on_insufficient_surbs(#[case] hops: usize) -> anyhow::Result<()> {
    let (_sink_addr, _sink_handle) = spawn_one_way_sink();

    let cluster = setup_one_hop_cluster(TestNodeConfig {
        win_prob: 1.0,
        incoming_pix_config: Some(IncomingSessionPixConfig {
            quota_range: 0..=100_000,
            enforce_pix: false,
            supervisor_cfg: SupervisorConfig {
                max_ssa_delivery_time: Duration::from_secs(10),
                max_deposit_wait: Duration::from_secs(60),
                max_recovery_time: Duration::from_secs(10),
                max_recovery_idle: Duration::from_secs(60),
                ..Default::default()
            },
            drain_cfg: SurbDrainConfig {
                enabled: true,
                max_drain_time: Duration::from_secs(60),
                ack_grace: Duration::from_secs(30),
                drain_rate_packets_per_sec: 50,
                cost_safety_factor: 1.0,
                ..Default::default()
            },
            ..Default::default()
        }),
        idle_timeout_ms: Duration::from_secs(120).as_millis() as u64,
        ..Default::default()
    })
    .await?;

    let mut exit_events = Box::pin(cluster.exit.inner().subscribe_pix_events());

    // Establish session with NO surb_management — no SURB preloading.
    let ip = IpOrHost::from_str(&format!("0.0.0.0:{}", _sink_addr.port()))?;
    let session = {
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
        session
    };
    tracing::info!("T5: session established without SURB preloading");

    signal_deposit(&mut exit_events, HoprBalance::new_base(1_000_000)).await?;

    // Allow deposit confirmation to propagate before dropping the session.
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Drop the session at Entry. The Exit supervisor will close via the
    // hard deadline (max_recovery_time).
    drop(session);
    tracing::info!("T5: session dropped at Entry");

    // Expect DrainFinished(Skipped(InsufficientSurbs)).
    let outcome = expect_drain_outcome(
        &cluster.exit,
        |o| matches!(&o.result, DrainResult::Skipped(SkipReason::InsufficientSurbs)),
        Duration::from_secs(30),
    )
    .await?;
    tracing::info!(packets = outcome.packets_sent, "T5: drain skipped");
    assert_eq!(outcome.packets_sent, 0, "no packets sent for skipped drain");

    assert_no_key_recovered(&mut exit_events, Duration::from_secs(10)).await;

    tracing::info!("T5 insufficient SURBs drain PASSED");
    Ok(())
}

// =========================================================================
//  Test T6: Drain disabled is inert
// =========================================================================

#[cfg(feature = "session-client")]
#[rstest]
#[case(1)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
/// Drain is disabled by default.  Verifies that no drain activity and no
/// post-closure recovery occurs — pinning today's behaviour.
async fn drain_disabled_is_inert(#[case] hops: usize) -> anyhow::Result<()> {
    let (_sink_addr, _sink_handle) = spawn_one_way_sink();

    // No drain_cfg — SurbDrainConfig defaults to enabled=false.
    let cluster = setup_one_hop_cluster(TestNodeConfig {
        win_prob: 1.0,
        incoming_pix_config: Some(IncomingSessionPixConfig {
            quota_range: 0..=100_000,
            enforce_pix: false,
            supervisor_cfg: SupervisorConfig {
                max_ssa_delivery_time: Duration::from_secs(10),
                max_deposit_wait: Duration::from_secs(60),
                max_recovery_time: Duration::from_secs(10),
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
    let session = connect_to_sink(&cluster, _sink_addr).await?;
    tracing::info!("T6: session established");

    signal_deposit(&mut exit_events, HoprBalance::new_base(1_000_000)).await?;

    // Drop the session at Entry. The Exit supervisor will close via the
    // hard deadline (max_recovery_time).
    drop(session);
    tracing::info!("T6: session dropped at Entry");

    // The drainer is always constructed when PIX is configured, but with
    // enabled=false it should skip all offers immediately.
    let outcome = expect_drain_outcome(
        &cluster.exit,
        |o| matches!(&o.result, DrainResult::Skipped(SkipReason::Disabled)),
        Duration::from_secs(30),
    )
    .await?;
    tracing::info!(?outcome.result, "T6: drain outcome");
    assert_eq!(outcome.packets_sent, 0);

    assert_no_key_recovered(&mut exit_events, Duration::from_secs(10)).await;
    tracing::info!("confirmed no recovery with drain disabled");

    tracing::info!("T6 drain disabled PASSED");
    Ok(())
}
