//! End-to-end PIX multi-cycle session test (n-hop).
//!
//! Establishes an n-hop session between Entry and Exit with PIX enabled, keeps
//! symmetric traffic flowing in the background, and observes the PIX event cycle
//! repeat multiple times:
//!
//!   1. [Entry] `NewDepositAddress`   — deposit address generated
//!   2. [Exit]  `DepositAddressReceived` — deposit needed, notifier provided
//!   3. [Test]  Signal deposit via notifier
//!   4. [Exit]  `PrivateKeyRecovered` — quota exhausted, key recovered → SessionManager requests next SSA → goto 1

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
    std::{str::FromStr, time::Duration},
};

const FUNDING_AMOUNT: &str = "15000 wxHOPR";

// PIX params: 8 polys × 2 shares × ~1440 bytes = ~23 KB per SSA cycle
const PIX_POLYS: u16 = 8;
const PIX_SHARES: u16 = 2;

/// Builds an Entry → N relays → Exit cluster with the Exit's PIX config, opens bidirectional
/// channels along the path, and waits for the graph to propagate.
///
/// The Entry's PIX dimensions are set to match what the session negotiates, so the Exit's
/// `quota_range` check has something acceptable to accept.
#[cfg(feature = "session-client")]
async fn build_pix_cluster(
    hops: usize,
    exit_pix: IncomingSessionPixConfig,
    exit_idle_timeout: Duration,
) -> anyhow::Result<hopr_lib::testing::fixtures::RoleClusterGuard> {
    let cluster = build_role_cluster(
        TestNodeConfig {
            win_prob: 1.0,
            pix_global_config: Some(hopr_lib::exports::transport::config::PixGlobalConfig {
                num_ssa_parts: PIX_POLYS as usize,
                ssa_part_size: PIX_SHARES as usize,
                additional_shares: 2,
            }),
            ..Default::default()
        },
        vec![TestNodeConfig::with_probability(MINIMUM_INCOMING_WIN_PROB); hops],
        TestNodeConfig {
            win_prob: 1.0,
            incoming_pix_config: Some(exit_pix),
            idle_timeout_ms: exit_idle_timeout.as_millis() as u64,
            ..Default::default()
        },
    )
    .await?;

    open_path_channels(&cluster, hops).await?;
    Ok(cluster)
}

/// Opens bidirectional channels along Entry → relays → Exit and waits for the graph.
#[cfg(feature = "session-client")]
async fn open_path_channels(
    cluster: &hopr_lib::testing::fixtures::RoleClusterGuard,
    hops: usize,
) -> anyhow::Result<()> {
    tracing::info!("opening channels");
    let funding = FUNDING_AMOUNT.parse::<HoprBalance>()?;

    macro_rules! open_chan {
        ($from:expr, $to:expr) => {{
            IncentiveChannelOperations::open_channel(&*$from.instance, $to.instance.identity().node_address, funding)
                .await
                .context("opening channel must succeed")?;
        }};
    }

    // Forward: Entry → Relay[0] → ... → Exit
    open_chan!(cluster.entry, cluster.relays[0]);
    for i in 0..hops.saturating_sub(1) {
        open_chan!(cluster.relays[i], cluster.relays[i + 1]);
    }
    open_chan!(cluster.relays[hops - 1], cluster.exit);

    // Backward: Exit → Relay[N-1] → ... → Entry
    open_chan!(cluster.exit, cluster.relays[hops - 1]);
    for i in (1..hops).rev() {
        open_chan!(cluster.relays[i], cluster.relays[i - 1]);
    }
    open_chan!(cluster.relays[0], cluster.entry);

    let chain_info = cluster.chain_client.query_chain_info().await?;
    tracing::info!("waiting for channel graph");
    tokio::time::sleep(chain_propagation_delay(&chain_info) * 6).await;
    tracing::info!("channel graph ready");
    Ok(())
}

/// Connects Entry → Exit with PIX enabled.
#[cfg(feature = "session-client")]
async fn establish_pix_session(
    cluster: &hopr_lib::testing::fixtures::RoleClusterGuard,
    hops: usize,
) -> anyhow::Result<hopr_lib::HoprSession> {
    let routing = hops.try_into()?;
    let ip = IpOrHost::from_str(":0")?;
    let (session, _) = tokio::time::timeout(
        Duration::from_secs(120),
        cluster.entry.inner().connect_to(
            cluster.exit.address(),
            SessionTarget::UdpStream(SealedHost::Plain(ip)),
            HoprSessionClientConfig {
                forward_path: routing,
                return_path: routing,
                capabilities: SessionCapability::Segmentation
                    | SessionCapability::NoRateControl
                    | SessionCapability::UsePIX,
                pseudonym: None,
                surb_management: None,
                always_max_out_surbs: false,
                pix_ssa_quota: Some((PIX_POLYS, PIX_SHARES)),
            },
        ),
    )
    .await
    .context("session connection timed out after 120s")??;
    Ok(session)
}

/// Keeps 32-byte echo traffic flowing and flips `session_died` when the Session stops answering.
///
/// Shares only travel with data-packet acknowledgements, so a PIX cycle makes no progress without
/// traffic — and a closed Session is observed here as a failed write or a read that never returns.
#[cfg(feature = "session-client")]
fn spawn_echo_task(
    session: hopr_lib::HoprSession,
    session_died: std::sync::Arc<std::sync::atomic::AtomicBool>,
    read_timeout: Duration,
) -> tokio::task::JoinHandle<()> {
    use std::sync::atomic::Ordering;

    tokio::spawn(async move {
        let (mut rd, mut wr) = session.split();
        loop {
            let msg = hopr_lib::api::types::crypto_random::random_bytes::<32>();
            if wr.write_all(&msg).await.is_err() || wr.flush().await.is_err() {
                tracing::warn!("echo task: write failed, session is gone");
                session_died.store(true, Ordering::SeqCst);
                break;
            }
            let mut echoed = vec![0u8; 32];
            match tokio::time::timeout(read_timeout, rd.read_exact(&mut echoed)).await {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    tracing::warn!(%error, "echo task: read failed, session is gone");
                    session_died.store(true, Ordering::SeqCst);
                    break;
                }
                Err(_) => {
                    tracing::warn!("echo task: read timed out, session is gone");
                    session_died.store(true, Ordering::SeqCst);
                    break;
                }
            }
        }
        tracing::info!("echo task exited");
    })
}

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
    let cluster = build_pix_cluster(
        hops,
        IncomingSessionPixConfig {
            quota_range: 0..=100_000,
            enforce_pix: false,
            supervision: SupervisorConfig {
                max_ssa_delivery_time: Duration::from_secs(10),
                max_deposit_wait: Duration::from_secs(60),
                ..Default::default()
            },
        },
        Duration::from_secs(90),
    )
    .await?;

    // ── Subscribe to PixEvent streams BEFORE creating the session ─────────
    tracing::info!("subscribing to PIX events");
    let mut entry_events = Box::pin(cluster.entry.inner().subscribe_pix_events());
    let mut exit_events = Box::pin(cluster.exit.inner().subscribe_pix_events());

    // ── Establish PIX-enabled session: Entry → Exit, n-hop ────────────────
    tracing::info!("establishing PIX session");
    let session = establish_pix_session(&cluster, hops).await?;
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

/// Verifies that the supervisor's deposit deadline closes a Session whose Entry commits but never
/// funds.
///
/// The Exit-side unit tests cannot reach this: arming the deposit deadline needs a
/// `CommitmentVerified`, and that needs a real Entry to answer the `SsaRequest`. Here one does, and
/// then the test simply declines to signal the deposit.
#[cfg(feature = "session-client")]
#[rstest]
#[case(1)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
async fn deposit_timeout_closes_session(#[case] hops: usize) -> anyhow::Result<()> {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    #[allow(unexpected_cfgs)]
    if cfg!(coverage) && hops > 1 {
        return Ok(());
    }

    let cluster = build_pix_cluster(
        hops,
        IncomingSessionPixConfig {
            quota_range: 0..=100_000,
            enforce_pix: false,
            supervision: SupervisorConfig {
                max_ssa_delivery_time: Duration::from_secs(10),
                // The deadline under test.
                max_deposit_wait: Duration::from_secs(5),
                ..Default::default()
            },
        },
        Duration::from_secs(90),
    )
    .await?;

    let mut exit_events = Box::pin(cluster.exit.inner().subscribe_pix_events());

    let session = establish_pix_session(&cluster, hops).await?;
    tracing::info!("session established");

    // Traffic keeps flowing so the closure shows up as a failed exchange rather than as silence.
    let session_died = Arc::new(AtomicBool::new(false));
    let _echo = spawn_echo_task(session, session_died.clone(), Duration::from_secs(8));

    let mut deposit_address_received = false;

    // Consume events without ever signalling a deposit, until the Session dies. The window is kept
    // close to `max_ssa_delivery_time + max_deposit_wait` so that a Session dying much later — of
    // the idle timeout, say — fails rather than passes.
    let outcome = tokio::time::timeout(Duration::from_secs(45), async {
        loop {
            if session_died.load(Ordering::SeqCst) {
                return Ok(());
            }
            tokio::select! {
                biased;
                _ = tokio::time::sleep(Duration::from_millis(500)) => {}
                Some(event) = exit_events.next() => {
                    match event {
                        PixEvent::DepositAddressReceived(data) => {
                            deposit_address_received = true;
                            tracing::info!(id = ?data.id, quota = data.quota,
                                "Exit: DepositAddressReceived — deliberately not signalling a deposit");
                        }
                        PixEvent::PrivateKeyRecovered(data) => {
                            anyhow::bail!("recovery completed without a deposit: {:?}", data.id);
                        }
                        other => anyhow::bail!("unexpected Exit PixEvent: {other:?}"),
                    }
                }
            }
        }
    })
    .await;

    assert!(
        deposit_address_received,
        "the Entry never committed, so the deposit deadline was never armed and this proves nothing"
    );
    outcome.context("session outlived its deposit deadline")??;

    tracing::info!(hops, "deposit timeout test PASSED");
    Ok(())
}

/// Verifies that the supervisor's absolute recovery deadline closes a Session whose SSA is funded
/// but never recovers.
///
/// Shares only travel with data-packet acknowledgements, so the deadline is provoked by funding the
/// SSA while no traffic is flowing and then waiting it out. Recovery makes no progress in that
/// window, and the backstop fires. Traffic starts afterwards purely to observe the closure.
#[cfg(feature = "session-client")]
#[rstest]
#[case(1)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
async fn recovery_hard_deadline_closes_session(#[case] hops: usize) -> anyhow::Result<()> {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    #[allow(unexpected_cfgs)]
    if cfg!(coverage) && hops > 1 {
        return Ok(());
    }

    let cluster = build_pix_cluster(
        hops,
        IncomingSessionPixConfig {
            quota_range: 0..=100_000,
            enforce_pix: false,
            supervision: SupervisorConfig {
                max_ssa_delivery_time: Duration::from_secs(10),
                // Both far out of reach, so that neither can be what closes the Session: a deposit
                // that silently failed to register would otherwise look exactly like the deadline
                // under test firing.
                max_deposit_wait: Duration::from_secs(600),
                max_recovery_idle: Duration::from_secs(600),
                // The deadline under test.
                max_recovery_time: Duration::from_secs(15),
                ..Default::default()
            },
        },
        Duration::from_secs(120),
    )
    .await?;

    let mut exit_events = Box::pin(cluster.exit.inner().subscribe_pix_events());

    let session = establish_pix_session(&cluster, hops).await?;
    tracing::info!("session established");

    // Fund the SSA, with no traffic flowing: recovery enters its window and then stalls there.
    tokio::time::timeout(Duration::from_secs(60), async {
        while let Some(event) = exit_events.next().await {
            match event {
                PixEvent::DepositAddressReceived(data) => {
                    if let Some(mut notifier) = data.deposit_updated {
                        notifier
                            .send((data.id, HoprBalance::new_base(1)))
                            .await
                            .context("failed to signal deposit via notifier")?;
                        tracing::info!(id = ?data.id, "deposit signalled, no traffic flowing");
                    }
                    return anyhow::Ok(());
                }
                other => tracing::debug!("Exit PixEvent while awaiting the deposit request: {other:?}"),
            }
        }
        anyhow::bail!("the Exit never asked for a deposit")
    })
    .await
    .context("timed out waiting for the deposit request")??;

    // Wait out the recovery deadline with the Session idle.
    tokio::time::sleep(Duration::from_secs(20)).await;

    // Now send: the Session must already be gone.
    let session_died = Arc::new(AtomicBool::new(false));
    let _echo = spawn_echo_task(session, session_died.clone(), Duration::from_secs(10));

    tokio::time::timeout(Duration::from_secs(30), async {
        while !session_died.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    })
    .await
    .context("session survived its absolute recovery deadline")?;

    tracing::info!(hops, "recovery hard deadline test PASSED");
    Ok(())
}

/// Verifies that an Exit configured with `enforce_pix` rejects a client that does not offer PIX.
///
/// `SessionManager` has a unit test for the rejection itself; what this adds is that it surfaces to
/// the client as a failed `connect_to` rather than being swallowed somewhere in the Start protocol.
#[cfg(feature = "session-client")]
#[rstest]
#[case(1)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
async fn enforce_pix_rejects_non_pix_session(#[case] hops: usize) -> anyhow::Result<()> {
    #[allow(unexpected_cfgs)]
    if cfg!(coverage) && hops > 1 {
        return Ok(());
    }

    // No `pix_global_config` on the Entry: this client is not going to offer PIX at all.
    let cluster = build_role_cluster(
        TestNodeConfig {
            win_prob: 1.0,
            ..Default::default()
        },
        vec![TestNodeConfig::with_probability(MINIMUM_INCOMING_WIN_PROB); hops],
        TestNodeConfig {
            win_prob: 1.0,
            incoming_pix_config: Some(IncomingSessionPixConfig {
                enforce_pix: true,
                ..Default::default()
            }),
            idle_timeout_ms: Duration::from_secs(30).as_millis() as u64,
            ..Default::default()
        },
    )
    .await?;
    open_path_channels(&cluster, hops).await?;

    let routing: hopr_lib::HopRouting = hops.try_into()?;
    let ip = IpOrHost::from_str(":0")?;
    let result = tokio::time::timeout(
        Duration::from_secs(30),
        cluster.entry.inner().connect_to(
            cluster.exit.address(),
            SessionTarget::UdpStream(SealedHost::Plain(ip)),
            HoprSessionClientConfig {
                forward_path: routing,
                return_path: routing,
                capabilities: SessionCapability::Segmentation | SessionCapability::NoRateControl,
                pseudonym: None,
                surb_management: None,
                always_max_out_surbs: false,
                pix_ssa_quota: None,
            },
        ),
    )
    .await;

    match result {
        Ok(Ok(_)) => anyhow::bail!("the Exit accepted a non-PIX session despite enforce_pix"),
        Ok(Err(error)) => {
            // The Exit answers with a Start-protocol rejection rather than dropping the request, so
            // the client learns why instead of waiting out its own timeout.
            tracing::info!(%error, "connection rejected as expected");
        }
        Err(_) => anyhow::bail!(
            "connect_to neither succeeded nor failed — the Exit dropped the request instead of rejecting it, which \
             leaves the client waiting out its own timeout"
        ),
    }

    tracing::info!(hops, "enforce_pix rejection test PASSED");
    Ok(())
}
