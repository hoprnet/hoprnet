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
const PIX_SHARES: u8 = 2;

/// Surplus shares the Entry is configured with.
///
/// Set explicitly rather than left to derive, because every surplus share is another round-trip
/// packet — see the comment on `additional_shares` below — and because a value that differs from
/// what the derivation would produce is what makes it visible whether the surplus really crossed
/// the wire. At `PIX_SHARES = 2` the derivation yields 1, so this being 2 is observable end to end.
const PIX_SURPLUS: u8 = 2;

/// The three above as the Session asks for them.
///
/// `const` rather than built at the call site so the range check runs at compile time — the values
/// are constants, so a typo here should not need a cluster to boot before it is noticed.
const PIX_PARAMS: hopr_lib::PixParams =
    match hopr_lib::PixParams::try_new(PIX_POLYS, PIX_SHARES, PIX_SURPLUS, hopr_lib::LOCAL_PIX_SUITE) {
        Ok(params) => params,
        Err(_) => panic!("test PIX parameters must be within the protocol ranges"),
    };

/// Number of SSAs the Exit packs into one `SsaRequest` in [`batched_ssa_request_drives_pix_cycles`].
///
/// Deliberately above the Entry's default cap of 2, so the test also proves that
/// `pix.max_ssas_per_request` is really plumbed from node configuration through to the
/// `SessionManager`: a batch of 2 would be accepted even by an Entry that ignored the knob entirely,
/// whereas a batch of 3 is refused outright unless the raised cap takes effect.
const SSA_BATCH: usize = 3;

/// Budget for observing two full SSA batches, measured from a live session.
///
/// Generous next to the ~3 cycles of traffic it actually takes — the 1-hop
/// [`capture_n_hop_pix_session`] case completes 3 cycles well inside a minute — because the point of
/// the bound is not to be tight. It is to turn "the Exit stopped requesting SSAs" into a named
/// failure instead of a bare `rstest` timeout that says only that the test did not finish.
#[cfg(feature = "session-client")]
#[allow(unexpected_cfgs)]
const BATCH_OBSERVATION_BUDGET: Duration = if cfg!(coverage) {
    Duration::from_secs(300)
} else {
    Duration::from_secs(150)
};

/// Builds an Entry → N relays → Exit cluster with the Exit's PIX config, opens bidirectional
/// channels along the path, and waits for the graph to propagate.
///
/// The Entry's PIX dimensions are set to match what the session negotiates, so the Exit's
/// `quota_range` check has something acceptable to accept.
///
/// `idle_timeout` applies to *both* ends. It has to: the fixture disables the Exit→Entry SURB
/// keep-alive stream (so that eviction tests can work at all), and an Entry slot's idle timer is only
/// reset by traffic arriving on it. Leaving the Entry at the fixture default of 2.5 s therefore
/// evicts it a few seconds into any test where the Exit is legitimately quiet — which reads exactly
/// like the Exit tearing the Session down.
#[cfg(feature = "session-client")]
async fn build_pix_cluster(
    hops: usize,
    exit_pix: IncomingSessionPixConfig,
    idle_timeout: Duration,
) -> anyhow::Result<hopr_lib::testing::fixtures::RoleClusterGuard> {
    let default_cap = hopr_lib::exports::transport::config::PixGlobalConfig::default().max_ssas_per_request;
    build_pix_cluster_with_entry_cap(hops, exit_pix, idle_timeout, default_cap).await
}

/// As [`build_pix_cluster`], but with the Entry's `max_ssas_per_request` under the caller's control.
///
/// Only [`batched_ssa_request_drives_pix_cycles`] needs this. The batch size is not negotiated, so an
/// Exit asking for more SSAs per request than the Entry accepts has every request refused — raising
/// one side means raising the other in step, and this is the other side.
#[cfg(feature = "session-client")]
async fn build_pix_cluster_with_entry_cap(
    hops: usize,
    exit_pix: IncomingSessionPixConfig,
    idle_timeout: Duration,
    entry_max_ssas_per_request: usize,
) -> anyhow::Result<hopr_lib::testing::fixtures::RoleClusterGuard> {
    let cluster = build_role_cluster(
        TestNodeConfig {
            win_prob: 1.0,
            pix_global_config: Some(hopr_lib::exports::transport::config::PixGlobalConfig {
                num_ssa_parts: PIX_POLYS as usize,
                ssa_part_size: PIX_SHARES as usize,
                additional_shares: Some(PIX_SURPLUS as usize),
                max_ssas_per_request: entry_max_ssas_per_request,
                ..Default::default()
            }),
            idle_timeout_ms: idle_timeout.as_millis() as u64,
            ..Default::default()
        },
        vec![TestNodeConfig::with_probability(MINIMUM_INCOMING_WIN_PROB); hops],
        TestNodeConfig {
            win_prob: 1.0,
            incoming_pix_config: Some(exit_pix),
            idle_timeout_ms: idle_timeout.as_millis() as u64,
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
                pix_ssa_quota: Some(PIX_PARAMS),
                flow_control: None,
                max_frames_behind_gap: None,
            },
        ),
    )
    .await
    .context("session connection timed out after 120s")??;
    Ok(session)
}

/// Keeps 32-byte echo traffic flowing, and records into `stopped` why it stopped.
///
/// Shares only travel with data-packet acknowledgements, so a PIX cycle makes no progress without
/// traffic. The stop *reason* is reported rather than a bare "died" flag because the three ways this
/// loop can end are not equivalent evidence — see [`EchoStop`].
#[cfg(feature = "session-client")]
fn spawn_echo_task(
    session: hopr_lib::HoprSession,
    stopped: EchoStopCell,
    read_timeout: Duration,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let (mut rd, mut wr) = session.split();
        loop {
            let msg = hopr_lib::api::types::crypto_random::random_bytes::<32>();
            if wr.write_all(&msg).await.is_err() || wr.flush().await.is_err() {
                tracing::warn!("echo task: write failed, the Session is closed");
                stopped.set(EchoStop::WriteFailed);
                break;
            }
            let mut echoed = vec![0u8; 32];
            match tokio::time::timeout(read_timeout, rd.read_exact(&mut echoed)).await {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    tracing::warn!(%error, "echo task: read failed, the Session is closed");
                    stopped.set(EchoStop::ReadFailed);
                    break;
                }
                Err(_) => {
                    tracing::warn!("echo task: read timed out — not necessarily a closure");
                    stopped.set(EchoStop::ReadTimedOut);
                    break;
                }
            }
        }
        tracing::info!("echo task exited");
    })
}

/// Why [`spawn_echo_task`] stopped.
///
/// The distinction is load-bearing, not diagnostic. A test that treats "the echo stopped" as "the
/// Session closed" can pass on a Session that is merely *quiet*, and on the PIX Exit quiet is a normal
/// state: the egress gate parks the writer when the predeposit budget is spent, which stalls reads
/// while the Session is very much alive. Only the write side distinguishes them — a PIX closure sends
/// the Entry a `SessionError`, the Entry closes its own half, and the next write fails; a gate stall
/// never fails a write.
#[cfg(feature = "session-client")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EchoStop {
    /// The Session is closed: the write half is gone.
    WriteFailed,
    /// The Session is closed: the read half reported an error.
    ReadFailed,
    /// No echo came back in time. Says nothing about whether the Session is open.
    ReadTimedOut,
}

#[cfg(feature = "session-client")]
impl EchoStop {
    /// Whether this outcome actually establishes that the Session was closed.
    fn is_closure(self) -> bool {
        matches!(self, Self::WriteFailed | Self::ReadFailed)
    }
}

/// Shared slot the echo task reports its stop reason into, with the instant it happened.
#[cfg(feature = "session-client")]
#[derive(Clone, Default)]
struct EchoStopCell(std::sync::Arc<std::sync::Mutex<Option<(EchoStop, std::time::Instant)>>>);

#[cfg(feature = "session-client")]
impl EchoStopCell {
    fn set(&self, stop: EchoStop) {
        let mut guard = self.0.lock().expect("echo stop cell poisoned");
        guard.get_or_insert((stop, std::time::Instant::now()));
    }

    fn get(&self) -> Option<(EchoStop, std::time::Instant)> {
        *self.0.lock().expect("echo stop cell poisoned")
    }
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
            // Not what any of these tests is about; the shipped ceiling is far above one cluster
            // Session at these dimensions.
            max_live_cycle_bytes: IncomingSessionPixConfig::default().max_live_cycle_bytes,
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

/// Verifies that the supervisor's deposit deadline — and specifically *that* deadline — closes a
/// Session whose Entry commits but never funds.
///
/// The Exit-side unit tests cannot reach this: arming the deposit deadline needs a
/// `CommitmentVerified`, and that needs a real Entry to answer the `SsaRequest`. Here one does, and
/// then the test simply declines to signal the deposit.
///
/// Two things make this a test of the deposit deadline rather than of "the Session died eventually",
/// which is all it used to establish:
///
/// * **The stop reason has to be a closure.** The echo task also stops on a read timeout, and on a PIX Exit a stalled
///   read is a normal state — the egress gate parks the writer once the predeposit budget is spent, which is exactly
///   what happens here, since no deposit is ever made. Only the write side distinguishes a closed Session from a quiet
///   one: the supervisor's close sends the Entry a `SessionError`, the Entry drops its half, and the next write fails.
///   Accepting a read timeout let this pass on a gate stall.
/// * **The timing has to match, and only one clock can produce it.** The interval from `DepositAddressReceived` (which
///   is the Exit verifying the commitment, i.e. the moment the deposit clock is armed) to the closure is asserted
///   against `max_deposit_wait`. Every other clock is configured far out of reach, so no other deadline can land inside
///   the asserted window: the commitment clock is 60 s *and* was cleared when the commitment verified, recovery
///   deadlines need a funded cycle, and the idle timeouts are 90 s.
#[cfg(feature = "session-client")]
#[rstest]
#[case(1)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
async fn deposit_timeout_closes_session(#[case] hops: usize) -> anyhow::Result<()> {
    // The deadline under test, and the slack allowed on observing it across a real cluster.
    const MAX_DEPOSIT_WAIT: Duration = Duration::from_secs(5);
    const OBSERVATION_SLACK: Duration = Duration::from_secs(20);
    // Every other supervisor clock is set here, well clear of the window above, so that a Session
    // dying inside it can only have died of the deposit deadline.
    const OTHER_CLOCKS: Duration = Duration::from_secs(60);

    #[allow(unexpected_cfgs)]
    if cfg!(coverage) && hops > 1 {
        return Ok(());
    }

    let cluster = build_pix_cluster(
        hops,
        IncomingSessionPixConfig {
            quota_range: 0..=100_000,
            enforce_pix: false,
            // Not what any of these tests is about; the shipped ceiling is far above one cluster
            // Session at these dimensions.
            max_live_cycle_bytes: IncomingSessionPixConfig::default().max_live_cycle_bytes,
            supervision: SupervisorConfig {
                // The deadline under test.
                max_deposit_wait: MAX_DEPOSIT_WAIT,
                // Everything else pushed far out of reach, so the observed interval can only be the
                // deposit clock. The commitment clock is additionally cleared the moment the
                // commitment verifies, and the recovery clocks need a cycle that was funded.
                max_ssa_delivery_time: OTHER_CLOCKS,
                max_recovery_idle: OTHER_CLOCKS,
                max_recovery_time: OTHER_CLOCKS,
                ..Default::default()
            },
        },
        Duration::from_secs(90),
    )
    .await?;

    let mut exit_events = Box::pin(cluster.exit.inner().subscribe_pix_events());

    let session = establish_pix_session(&cluster, hops).await?;
    tracing::info!("session established");

    // Traffic keeps flowing so the closure shows up as a failed exchange rather than as silence. The
    // read timeout is deliberately longer than the deadline under test: a read that times out first
    // would stop the echo task without establishing anything, and is reported as such.
    let stopped = EchoStopCell::default();
    let _echo = spawn_echo_task(session, stopped.clone(), MAX_DEPOSIT_WAIT + OBSERVATION_SLACK);

    // When the Exit verified the commitment — i.e. when the deposit clock was armed. Everything is
    // measured from here rather than from establishment, because that is what the deadline is
    // measured from.
    let mut deposit_clock_armed: Option<std::time::Instant> = None;

    // The deposit notifiers are *held*, never signalled. This is the difference between declining to
    // deposit and going away: dropping a notifier is what the deposit observer reports as
    // `DepositObserverClosed`, and the supervisor closes on it at once rather than waiting out a
    // deadline for funds it has been told are not coming. Holding it keeps the observer alive with
    // nothing to report, which is the only state in which the deposit deadline is what fires.
    let mut held_notifiers = Vec::new();

    // Consume events without ever signalling a deposit, until the echo task stops. Bounded well
    // inside the other clocks, so a Session dying of one of those fails rather than passes.
    let outcome = tokio::time::timeout(MAX_DEPOSIT_WAIT + OBSERVATION_SLACK, async {
        loop {
            if stopped.get().is_some() {
                return Ok::<_, anyhow::Error>(());
            }
            tokio::select! {
                biased;
                _ = tokio::time::sleep(Duration::from_millis(200)) => {}
                Some(event) = exit_events.next() => {
                    match event {
                        PixEvent::DepositAddressReceived(data) => {
                            deposit_clock_armed.get_or_insert_with(std::time::Instant::now);
                            held_notifiers.extend(data.deposit_updated);
                            tracing::info!(id = ?data.id, quota = data.quota,
                                "Exit: DepositAddressReceived — holding the notifier, never signalling a deposit");
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

    let armed_at = deposit_clock_armed
        .context("the Entry never committed, so the deposit deadline was never armed and this proves nothing")?;
    outcome.context("session outlived its deposit deadline")??;

    let (stop, stopped_at) = stopped.get().context("the echo task never stopped")?;

    // A read timeout means the Session went quiet, which on this Exit is what an exhausted predeposit
    // budget looks like — not a closure. Only a failed write or a read error shows the Session gone.
    assert!(
        stop.is_closure(),
        "the Session must be observed *closed*, not merely quiet; got {stop:?}"
    );

    // And it must have closed on the deposit clock: no earlier than the deadline, and far enough
    // inside the others that none of them could have been what fired.
    let elapsed = stopped_at.saturating_duration_since(armed_at);
    assert!(
        elapsed >= MAX_DEPOSIT_WAIT,
        "closed {elapsed:?} after the deposit clock was armed, before its {MAX_DEPOSIT_WAIT:?} deadline could expire \
         — so something other than the deposit deadline closed it"
    );
    assert!(
        elapsed < OTHER_CLOCKS,
        "closed {elapsed:?} after the deposit clock was armed, which is past the {OTHER_CLOCKS:?} the other clocks \
         are set to — the closure cannot be attributed to the deposit deadline"
    );
    tracing::info!(?elapsed, ?stop, "closed on the deposit deadline");

    // Explicit, so that nothing reorders the notifiers' drop above the assertions: dropping them
    // early would close the Session by the observer path and invalidate everything measured here.
    drop(held_notifiers);

    tracing::info!(hops, "deposit timeout test PASSED");
    Ok(())
}

/// Verifies that an Exit configured for strict prepay (`max_predeposit_packets = 0`) serves nothing
/// until the deposit is confirmed, and serves normally once it is.
///
/// The Exit-side unit tests reach the gate, but not the property that makes a zero budget a usable
/// policy rather than a deadlock: the `SsaRequest` and the Entry's commitment both bypass the egress
/// gate, so the Session can still become fundable while nothing at all is being served. Only a real
/// Entry answering a real request exercises that — route either through the gate and this test hangs,
/// where every unit test would still pass.
///
/// The third bypass, the SURB keep-alive stream, is *not* covered here: the test fixture disables it
/// (`surb_balance_notify_period: None`) so that eviction tests can work. In production it is what
/// keeps the Entry's own session slot from idling out while the Exit is quiet; here the long
/// `idle_timeout` passed to `build_pix_cluster` stands in for it.
#[cfg(feature = "session-client")]
#[rstest]
#[case(1)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
async fn strict_prepay_serves_nothing_before_the_deposit(#[case] hops: usize) -> anyhow::Result<()> {
    #[allow(unexpected_cfgs)]
    if cfg!(coverage) && hops > 1 {
        return Ok(());
    }

    let cluster = build_pix_cluster(
        hops,
        IncomingSessionPixConfig {
            quota_range: 0..=100_000,
            enforce_pix: false,
            // Not what any of these tests is about; the shipped ceiling is far above one cluster
            // Session at these dimensions.
            max_live_cycle_bytes: IncomingSessionPixConfig::default().max_live_cycle_bytes,
            supervision: SupervisorConfig {
                max_ssa_delivery_time: Duration::from_secs(10),
                // The setting under test: not one packet before the deposit.
                max_predeposit_packets: 0,
                // Far out of reach, so the Session is still open to be funded after the stall window
                // below. At its default this would be measuring the deposit deadline instead.
                max_deposit_wait: Duration::from_secs(600),
                ..Default::default()
            },
        },
        Duration::from_secs(120),
    )
    .await?;

    let mut exit_events = Box::pin(cluster.exit.inner().subscribe_pix_events());

    let session = establish_pix_session(&cluster, hops).await?;
    tracing::info!("session established");
    let (mut rd, mut wr) = session.split();

    // Keep giving the Exit something it wants to answer, for the whole test. Writes towards the Exit
    // are not gated, so this keeps running throughout the stall below — which is the point: the Exit
    // is not quiet for want of anything to say.
    let writer = tokio::spawn(async move {
        loop {
            let msg = hopr_lib::api::types::crypto_random::random_bytes::<32>();
            if wr.write_all(&msg).await.is_err() || wr.flush().await.is_err() {
                tracing::warn!("writer: the Entry side stopped accepting writes");
                break;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });

    // Drain the Exit's PIX events for the whole test, handing the first deposit notifier back and
    // then carrying on draining. A client that stopped polling the stream mid-test would be a second
    // variable in a test meant to isolate the gate.
    let (notifier_tx, notifier_rx) = futures::channel::oneshot::channel();
    let drain = tokio::spawn(async move {
        let mut notifier_tx = Some(notifier_tx);
        while let Some(event) = exit_events.next().await {
            match event {
                PixEvent::DepositAddressReceived(data) => match (notifier_tx.take(), data.deposit_updated) {
                    (Some(tx), Some(notifier)) => {
                        tracing::info!(id = ?data.id, "Exit: DepositAddressReceived — withholding the deposit");
                        let _ = tx.send((data.id, notifier));
                    }
                    _ => tracing::debug!(id = ?data.id, "further deposit request"),
                },
                other => tracing::debug!("Exit PixEvent: {other:?}"),
            }
        }
    });

    // The Exit asks for a deposit despite serving nothing, because the `SsaRequest` never touches the
    // egress gate. Hold the notifier instead of answering it, so that the stall below is observed
    // against a Session that is committed and merely unfunded — rather than one that never got as
    // far as being asked to pay.
    let (deposit_id, mut deposit_notifier) = tokio::time::timeout(Duration::from_secs(60), notifier_rx)
        .await
        .context("timed out waiting for the deposit request")?
        .context("the Exit never asked for a deposit — a strict-prepay gate must not hold up the SsaRequest")?;

    // Nothing may come back yet, however much the Entry sends.
    let mut echoed = vec![0u8; 32];
    match tokio::time::timeout(Duration::from_secs(15), rd.read_exact(&mut echoed)).await {
        Err(_) => tracing::info!("nothing served before the deposit, as configured"),
        Ok(Ok(())) => {
            anyhow::bail!("the Exit served a packet before the deposit, with max_predeposit_packets = 0")
        }
        Ok(Err(error)) => anyhow::bail!("the Session failed instead of stalling on the gate: {error}"),
    }

    // Funding it must release the answer that was withheld, rather than merely stop refusing new
    // ones: the packet parked on the gate has to be woken, not dropped.
    deposit_notifier
        .send((deposit_id, HoprBalance::new_base(1)))
        .await
        .context("failed to signal deposit via notifier")?;
    tracing::info!(id = ?deposit_id, "deposit signalled");

    tokio::time::timeout(Duration::from_secs(60), rd.read_exact(&mut echoed))
        .await
        .context("the Exit never served the probe after the deposit was confirmed")?
        .context("the Session failed after funding")?;

    writer.abort();
    drain.abort();
    tracing::info!(hops, "strict prepay test PASSED");
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
    #[allow(unexpected_cfgs)]
    if cfg!(coverage) && hops > 1 {
        return Ok(());
    }

    let cluster = build_pix_cluster(
        hops,
        IncomingSessionPixConfig {
            quota_range: 0..=100_000,
            enforce_pix: false,
            // Not what any of these tests is about; the shipped ceiling is far above one cluster
            // Session at these dimensions.
            max_live_cycle_bytes: IncomingSessionPixConfig::default().max_live_cycle_bytes,
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

    // Now send: the Session must already be gone. The recovery deadline has passed, so the write half
    // has to be closed — a read timeout would only show the Session quiet, which it has been all along.
    let stopped = EchoStopCell::default();
    let _echo = spawn_echo_task(session, stopped.clone(), Duration::from_secs(10));

    tokio::time::timeout(Duration::from_secs(30), async {
        while stopped.get().is_none() {
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    })
    .await
    .context("session survived its absolute recovery deadline")?;

    let (stop, _) = stopped.get().context("the echo task never stopped")?;
    assert!(
        stop.is_closure(),
        "the Session must be observed closed after its recovery deadline, not merely quiet; got {stop:?}"
    );

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
                flow_control: None,
                max_frames_behind_gap: None,
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

/// 1-hop PIX session in which the Exit requests [`SSA_BATCH`] SSAs per `SsaRequest`.
///
/// The supervisor's own unit tests already pin the batch onto a single `RequestSsa` action, and
/// `hopr-transport-session`'s integration tests pin that action onto a single `SsaRequest` message.
/// What only a cluster shows is that the batch survives the real path: `SSA_BATCH` commitment sets
/// burst back through a relay over QUIC into the Exit's *bounded* Start-protocol ingress channel, and
/// every one of them has to land. A dropped `SsaCommit` has no NACK, so an ingress channel that was
/// not sized for the batch would lose a cycle silently here and the Session would die on a deposit
/// timeout minutes later.
///
/// It is also the only place the batch meets the supervisor's real per-cycle deadlines rather than a
/// mocked clock. Those deadlines are scaled by `ssas_per_request`, and they have to be: the Entry
/// works through a batch in order, so holding the last cycle to an unscaled window would close a
/// Session whose peer is behaving perfectly.
///
/// Three properties are checked, each regressing differently:
///
///  1. **The batch is allocated up front** — at least `SSA_BATCH` deposit addresses reach the Exit before it recovers
///     its first private key. Unbatched, the Exit learns of the next address only once the current cycle is nearly
///     recovered, so this count would be one or two, never three.
///  2. **Batch N+1 follows batch N** — enforced by [`BATCH_OBSERVATION_BUDGET`] rather than an `assert!`, since the
///     failure mode is a stall, not a wrong value. A batch is requested once per recovered cycle at most, so an
///     off-by-one in the supervisor's index bookkeeping leaves the Session with no further SSAs and it quietly stops
///     rolling.
///  3. **Indices are contiguous and addresses unique** across both batches — a batch is allocated as `first .. first +
///     batch`, and a wrapped or reused index would collide with a live cycle.
#[cfg(feature = "session-client")]
#[rstest]
#[case(1)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
async fn batched_ssa_request_drives_pix_cycles(#[case] hops: usize) -> anyhow::Result<()> {
    // Both PIX sides raised together: the Exit asks for SSA_BATCH per request, and the Entry has to
    // accept that many or it refuses every request outright.
    let cluster = build_pix_cluster_with_entry_cap(
        hops,
        IncomingSessionPixConfig {
            quota_range: 0..=100_000,
            enforce_pix: false,
            // Not what any of these tests is about; the shipped ceiling is far above one cluster
            // Session at these dimensions.
            max_live_cycle_bytes: IncomingSessionPixConfig::default().max_live_cycle_bytes,
            supervision: SupervisorConfig {
                max_ssa_delivery_time: Duration::from_secs(10),
                max_deposit_wait: Duration::from_secs(60),
                ssas_per_request: SSA_BATCH,
                ..Default::default()
            },
        },
        Duration::from_secs(90),
        SSA_BATCH,
    )
    .await?;

    // ── Subscribe to PixEvent streams BEFORE creating the session ─────────
    let mut entry_events = Box::pin(cluster.entry.inner().subscribe_pix_events());
    let mut exit_events = Box::pin(cluster.exit.inner().subscribe_pix_events());

    tracing::info!("establishing PIX session");
    let session = establish_pix_session(&cluster, hops).await?;
    tracing::info!("session established");

    // Traffic is what moves shares, so the cycles only advance while this runs. Why it eventually
    // stops is not this test's subject — the assertions below are all about PIX events.
    let echo_handle = spawn_echo_task(session, EchoStopCell::default(), Duration::from_secs(10));

    // ── Observe two consecutive batches ───────────────────────────────────
    let mut entry_ids: Vec<hopr_api::node::PixAddressId> = Vec::new();
    let mut exit_ids: Vec<hopr_api::node::PixAddressId> = Vec::new();
    let mut exit_addresses = Vec::new();
    let mut recovered_ids: Vec<hopr_api::node::PixAddressId> = Vec::new();
    // Latched on the first recovery: how many addresses the Exit had been handed by the time it
    // finished its first cycle. This is the establishment batch, counted from the Exit's own stream
    // so no cross-stream ordering is involved.
    let mut addresses_before_first_recovery: Option<usize> = None;

    let observe = async {
        loop {
            tokio::select! {
                event = entry_events.next() => {
                    match event.context("Entry PIX event stream ended while the session was live")? {
                        PixEvent::NewDepositAddress(data) => {
                            assert!(
                                !entry_ids.contains(&data.id),
                                "duplicate NewDepositAddress for {:?} — every cycle in a batch needs its own SSA",
                                data.id,
                            );
                            entry_ids.push(data.id);
                            tracing::info!(id = ?data.id, quota = data.quota, "Entry: NewDepositAddress");
                        }
                        other => anyhow::bail!("unexpected Entry PixEvent: {other:?}"),
                    }
                }
                event = exit_events.next() => {
                    match event.context("Exit PIX event stream ended while the session was live")? {
                        PixEvent::DepositAddressReceived(data) => {
                            tracing::info!(id = ?data.id, quota = data.quota, "Exit: DepositAddressReceived");
                            assert!(
                                !exit_ids.contains(&data.id),
                                "duplicate DepositAddressReceived for {:?} — a reused SSA index would \
                                 collide with a live cycle",
                                data.id,
                            );
                            assert!(
                                !exit_addresses.contains(&data.address),
                                "deposit address for {:?} was already used by an earlier cycle — each SSA \
                                 in a batch must get its own",
                                data.id,
                            );
                            exit_ids.push(data.id);
                            exit_addresses.push(data.address);
                            // Signal the deposit immediately so this cycle's deadline is disarmed.
                            if let Some(mut notifier) = data.deposit_updated {
                                notifier
                                    .send((data.id, HoprBalance::new_base(1)))
                                    .await
                                    .context("failed to signal deposit via notifier")?;
                            }
                        }
                        PixEvent::PrivateKeyRecovered(data) => {
                            addresses_before_first_recovery.get_or_insert(exit_ids.len());
                            assert!(
                                !recovered_ids.contains(&data.id),
                                "duplicate PrivateKeyRecovered for {:?}",
                                data.id,
                            );
                            recovered_ids.push(data.id);
                            tracing::info!(count = recovered_ids.len(), id = ?data.id, "Exit: PrivateKeyRecovered");
                        }
                        other => anyhow::bail!("unexpected Exit PixEvent: {other:?}"),
                    }
                }
            }

            // Twice the batch size proves a *second* batch was requested, and `SSA_BATCH` recoveries
            // prove the first batch's cycles actually reconstructed rather than merely being handed out.
            if exit_ids.len() >= 2 * SSA_BATCH && recovered_ids.len() >= SSA_BATCH {
                break;
            }
        }
        anyhow::Ok(())
    };

    tokio::time::timeout(BATCH_OBSERVATION_BUDGET, observe).await.context(
        "timed out observing two SSA batches — the Exit most likely stopped requesting SSAs after the first batch, \
         which is how an off-by-one in the supervisor's index bookkeeping manifests",
    )??;

    // ── 1. The whole batch is allocated before the first cycle completes ──
    let before_first_recovery = addresses_before_first_recovery.context("the Exit never recovered a key")?;
    assert!(
        before_first_recovery >= SSA_BATCH,
        "the Exit was told about only {before_first_recovery} deposit address(es) before it recovered its first key; \
         a batch of {SSA_BATCH} is allocated up front, so it should already know all {SSA_BATCH}. Getting one or two \
         here means the batch size did not take effect. exit_ids={exit_ids:?}",
    );

    // ── 2. Contiguous indices from 1, across the batch boundary ───────────
    let mut indices: Vec<u32> = exit_ids.iter().map(|(_, index)| index.get()).collect();
    indices.sort_unstable();
    assert_eq!(
        indices,
        (1..=exit_ids.len() as u32).collect::<Vec<_>>(),
        "SSA indices must be contiguous from 1 both within a batch and from one batch to the next",
    );

    // ── 3. Every recovered cycle passed through all three stages ──────────
    let fully_correlated = recovered_ids
        .iter()
        .filter(|id| entry_ids.contains(id) && exit_ids.contains(id))
        .count();
    assert!(
        fully_correlated >= SSA_BATCH,
        "expected at least {SSA_BATCH} SSA cycles seen at all three stages (NewDepositAddress, DepositAddressReceived \
         AND PrivateKeyRecovered), got {fully_correlated}. entry_ids={entry_ids:?}, exit_ids={exit_ids:?}, \
         recovered_ids={recovered_ids:?}",
    );

    echo_handle.abort();

    tracing::info!(
        batch = SSA_BATCH,
        addresses = exit_ids.len(),
        recovered = recovered_ids.len(),
        "batched PIX session test PASSED"
    );
    Ok(())
}
