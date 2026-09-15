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
    MINIMUM_INCOMING_WIN_PROB, TEST_GLOBAL_TIMEOUT, TestNodeConfig, build_role_cluster,
    build_role_cluster_with_exit_server, chain_propagation_delay,
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
            HasChainApi, HasExitIncentivization, HoprSessionClientOperations, IncentiveChannelOperations,
            PixDepositData, PixEvent,
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
    build_pix_cluster_with(
        hops,
        exit_pix,
        idle_timeout,
        entry_max_ssas_per_request,
        hopr_lib::testing::dummies::EchoServer::new(),
    )
    .await
}

/// As [`build_pix_cluster`], but with the Exit's session server under the caller's control.
///
/// Only [`a_closed_exit_session_drains_its_surbs_into_the_funded_cycle`] needs this. An
/// [`EchoServer`](hopr_lib::testing::dummies::EchoServer) owns the Exit-side Session for its whole
/// life and never hands it back, so there is no way to *close* one from a test; a
/// [`SessionCaptureServer`](hopr_lib::testing::dummies::SessionCaptureServer) parks instead and hands
/// the `IncomingSession` out, which is what makes the Exit-side close reachable at all.
#[cfg(feature = "session-client")]
async fn build_pix_cluster_with_exit_server<Srv>(
    hops: usize,
    exit_pix: IncomingSessionPixConfig,
    idle_timeout: Duration,
    exit_server: Srv,
) -> anyhow::Result<hopr_lib::testing::fixtures::RoleClusterGuard>
where
    Srv: hopr_api::node::HoprSessionServer<
            Session = hopr_lib::exports::transport::IncomingSession,
            Error: std::fmt::Display,
        > + Clone
        + Send
        + Sync
        + 'static,
{
    let default_cap = hopr_lib::exports::transport::config::PixGlobalConfig::default().max_ssas_per_request;
    build_pix_cluster_with(hops, exit_pix, idle_timeout, default_cap, exit_server).await
}

/// The cluster the three builders above are all a special case of.
#[cfg(feature = "session-client")]
async fn build_pix_cluster_with<Srv>(
    hops: usize,
    exit_pix: IncomingSessionPixConfig,
    idle_timeout: Duration,
    entry_max_ssas_per_request: usize,
    exit_server: Srv,
) -> anyhow::Result<hopr_lib::testing::fixtures::RoleClusterGuard>
where
    Srv: hopr_api::node::HoprSessionServer<
            Session = hopr_lib::exports::transport::IncomingSession,
            Error: std::fmt::Display,
        > + Clone
        + Send
        + Sync
        + 'static,
{
    let cluster = build_role_cluster_with_exit_server(
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
        exit_server,
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
    // `hops.saturating_sub(1)` below shows the zero case was considered, but the two `relays`
    // indices around it are unguarded and `hops - 1` underflows before either can fail. No caller
    // passes zero today; this is a shared helper taking `hops` as a parameter, so say so rather
    // than leaving the next one an arithmetic panic.
    if hops == 0 {
        anyhow::bail!("open_path_channels needs at least one relay; a zero-hop path has no channels to open");
    }

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
    establish_pix_session_with(cluster, hops, None, false).await
}

/// As [`establish_pix_session`], but with the Entry's SURB supply under the caller's control.
///
/// Two knobs, and both of them matter only to the fill tests:
///
/// * `surb_management` turns the Entry's SURB balancer on. Without it an Entry produces SURBs only alongside its own
///   outgoing packets, so an *idle* Session hands the Exit nothing to send with — and fill has nothing to fill with.
///   That is a real deployment shape rather than a test artefact, which is why one of the tests below deliberately
///   keeps it off: the recovery deadline must still close a Session whose Entry supplies nothing.
/// * `rate_control` selects the Exit's rate-limited egress branch instead of `NoRateControl`. `NoRateControl` is the
///   default here and what most of these tests use, but it is the rate-controlled branch that every gnosis client opens
///   — and the only one on which the Entry announces its SURB buffer target, which is what fill's reserve is derived
///   from. [`idle_session_is_completed_by_exit_fill`] runs over both.
#[cfg(feature = "session-client")]
async fn establish_pix_session_with(
    cluster: &hopr_lib::testing::fixtures::RoleClusterGuard,
    hops: usize,
    surb_management: Option<hopr_lib::SurbBalancerConfig>,
    rate_control: bool,
) -> anyhow::Result<hopr_lib::HoprSession> {
    let routing = hops.try_into()?;
    let ip = IpOrHost::from_str(":0")?;
    let capabilities = if rate_control {
        SessionCapability::Segmentation | SessionCapability::UsePIX
    } else {
        SessionCapability::Segmentation | SessionCapability::NoRateControl | SessionCapability::UsePIX
    };
    let (session, _) = tokio::time::timeout(
        Duration::from_secs(120),
        cluster.entry.inner().connect_to(
            cluster.exit.address(),
            SessionTarget::UdpStream(SealedHost::Plain(ip)),
            HoprSessionClientConfig {
                forward_path: routing,
                return_path: routing,
                capabilities,
                pseudonym: None,
                surb_management,
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

/// The Exit-side PIX configuration the fill tests share.
///
/// `max_recovery_time` is the budget fill plans against, and 60 s is as short as a legal pair gets
/// here: `max_recovery_idle` must clear the reconstructor's 30 s acknowledgement window and the hard
/// deadline must exceed it. A whole cycle is 32 packets at [`PIX_PARAMS`], so the aim point of
/// `0.75 x 60 s` asks fill for about three quarters of a packet a second — slow enough to be
/// unmistakably fill rather than a burst, fast enough to finish inside one test.
///
/// `fill` is taken at its shipped defaults, `min_surb_reserve` included, and that is deliberate: the
/// shipped 500 is sized against a production SURB buffer while this cluster's whole cycle is 32
/// packets, so an *absolute* reserve would refuse every fill packet until the Entry's balancer had
/// produced half a thousand SURBs for a Session that needs thirty-two. What makes the shipped value
/// workable here is that it is a ceiling on a reserve derived per Session from the buffer target the
/// Entry announced — a quarter of [`idle_surb_balancer`]'s 64 on the rate-controlled branch, and a
/// quarter of the Exit's own fallback on the `NoRateControl` branch, which announces no target at
/// all. Overriding it here would test the override rather than the derivation.
#[cfg(feature = "session-client")]
fn fill_pix_config(fill: hopr_lib::exports::transport::session::PixFillConfig) -> IncomingSessionPixConfig {
    IncomingSessionPixConfig {
        quota_range: 0..=100_000,
        enforce_pix: false,
        max_live_cycle_bytes: IncomingSessionPixConfig::default().max_live_cycle_bytes,
        supervision: SupervisorConfig {
            max_ssa_delivery_time: Duration::from_secs(10),
            // Far out of reach, so a deposit that silently failed to register cannot be mistaken for
            // the deadline under test.
            max_deposit_wait: Duration::from_secs(600),
            // At its floor, which puts it below the hard deadline. It does not fire here because it
            // is service-gated and these Sessions consume no gated service at all.
            max_recovery_idle: Duration::from_secs(30),
            max_recovery_time: Duration::from_secs(60),
            fill,
            ..Default::default()
        },
    }
}

/// A PIX lifecycle milestone the fill tests wait on, with when it happened.
#[cfg(feature = "session-client")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PixMilestone {
    /// A deposit was asked for and signalled.
    Funded(hopr_api::node::PixAddressId),
    /// A cycle recovered its private key — the thing fill exists to make happen.
    Recovered(hopr_api::node::PixAddressId),
}

/// Drives the Exit's PIX event stream in the background, reporting milestones as they land.
///
/// Started *before* the Session is established, and that ordering is load-bearing rather than tidy:
/// the Exit blocks its first `SsaRequest` on the deposit pool with a three-second budget, while
/// `connect_to` on a Session with a SURB balancer does not return until the balancer has filled —
/// comfortably longer than that. A test that only begins answering once it holds the Session has
/// already lost the first cycle to `UnacceptablePixParams`, and would report that as a fill failure.
///
/// Every deposit is signalled and every deposit-data request is answered with empty data, which is
/// what the deposit pool this cluster does not have would do. The tests then only have to say which
/// milestones they are waiting for.
#[cfg(feature = "session-client")]
fn spawn_exit_pix_driver(
    cluster: &hopr_lib::testing::fixtures::RoleClusterGuard,
) -> (
    tokio::task::JoinHandle<()>,
    futures::channel::mpsc::UnboundedReceiver<(PixMilestone, std::time::Instant)>,
) {
    let mut events = Box::pin(cluster.exit.inner().subscribe_pix_events());
    let (tx, rx) = futures::channel::mpsc::unbounded();
    let handle = tokio::spawn(async move {
        while let Some(event) = events.next().await {
            match event {
                PixEvent::DepositAddressReceived(data) => {
                    tracing::info!(id = ?data.id, "Exit: DepositAddressReceived");
                    let mut notifier = data.deposit_updated;
                    if notifier.send((data.id, HoprBalance::new_base(1))).await.is_err() {
                        tracing::warn!(id = ?data.id, "the deposit notifier is gone");
                        break;
                    }
                    if tx
                        .unbounded_send((PixMilestone::Funded(data.id), std::time::Instant::now()))
                        .is_err()
                    {
                        break;
                    }
                }
                PixEvent::PrivateKeyRecovered(data) => {
                    tracing::info!(id = ?data.id, "Exit: PrivateKeyRecovered");
                    if tx
                        .unbounded_send((PixMilestone::Recovered(data.id), std::time::Instant::now()))
                        .is_err()
                    {
                        break;
                    }
                }
                PixEvent::DepositDataRequest(request) => {
                    let mut created = request.deposit_data_created;
                    for id in request.deposit_ids {
                        if created
                            .send(PixDepositData {
                                id,
                                data: Box::default(),
                            })
                            .await
                            .is_err()
                        {
                            tracing::warn!("the deposit-data channel is gone");
                            break;
                        }
                    }
                }
                other => tracing::debug!("Exit PixEvent: {other:?}"),
            }
        }
        tracing::info!("exit PIX driver exited");
    });
    (handle, rx)
}

/// Runs exactly `count` 32-byte echo round-trips on `session`, one every `pace`.
///
/// Borrows the Session rather than taking it, which [`spawn_echo_task`] cannot: aborting that task
/// *drops* the Session, which closes it. A test about what fill does once an application goes quiet
/// needs the traffic to stop while the Session lives on, so it has to keep the Session in its own
/// scope.
///
/// Counted rather than timed, and paced rather than as fast as the link allows, because the caller is
/// choosing how much of a cycle the application contributes. An unpaced loop manages a hundred-odd
/// round trips in a couple of seconds, which at [`PIX_PARAMS`]' 32-packet cycles is several whole
/// cycles — enough to leave the Exit's SURB FIFO holding a run of share-less SURBs minted between one
/// cycle and the next, which no rate of fill can get past before the idle deadline.
#[cfg(feature = "session-client")]
async fn echo_n(session: &mut hopr_lib::HoprSession, count: usize, pace: Duration) -> usize {
    let mut completed = 0usize;
    for _ in 0..count {
        let msg = hopr_lib::api::types::crypto_random::random_bytes::<32>();
        let round_trip = async {
            session.write_all(&msg).await?;
            session.flush().await?;
            let mut echoed = [0u8; 32];
            session.read_exact(&mut echoed).await?;
            std::io::Result::Ok(())
        };
        match tokio::time::timeout(Duration::from_secs(20), round_trip).await {
            Ok(Ok(())) => completed += 1,
            Ok(Err(error)) => {
                tracing::warn!(%error, "echo round trip failed");
                break;
            }
            Err(_) => {
                tracing::warn!("echo round trip timed out");
                break;
            }
        }
        tokio::time::sleep(pace).await;
    }
    completed
}

/// Collects milestones until `done` accepts the set, or the budget expires.
#[cfg(feature = "session-client")]
async fn await_milestones(
    rx: &mut futures::channel::mpsc::UnboundedReceiver<(PixMilestone, std::time::Instant)>,
    budget: Duration,
    mut done: impl FnMut(&[(PixMilestone, std::time::Instant)]) -> bool,
) -> Vec<(PixMilestone, std::time::Instant)> {
    let mut seen = Vec::new();
    let _ = tokio::time::timeout(budget, async {
        while let Some(milestone) = rx.next().await {
            seen.push(milestone);
            if done(&seen) {
                break;
            }
        }
    })
    .await;
    seen
}

/// The Entry-side balancer that keeps an idle Session's Exit supplied with SURBs to fill with.
///
/// Without it an Entry produces SURBs only alongside its own outgoing packets, so an idle Session
/// hands the Exit nothing to fill with. What the target must *not* be is much larger than a cycle's
/// emission, and that is a property of these dimensions rather than of production: a share rides a
/// SURB, the Entry mints SURBs ahead of demand, and a SURB minted while no cycle is committed carries
/// no share at all. A buffer many multiples of a cycle deep therefore parks a long run of share-less
/// SURBs at the head of the Exit's FIFO, and the next cycle makes no progress until they are spent —
/// which at fill's rate is minutes. Two cycles' worth keeps the queue shallow enough that this cannot
/// dominate, while still covering the Exit between refills.
///
/// Production does not have the problem: the shipped 7 000 SURBs against a 655 360-packet cycle is
/// one per cent of a cycle, so the share-less run between cycles is a rounding error rather than the
/// whole buffer.
#[cfg(feature = "session-client")]
fn idle_surb_balancer() -> hopr_lib::SurbBalancerConfig {
    hopr_lib::SurbBalancerConfig {
        // Two cycles at PIX_PARAMS' 8 x (2 + 2) emitted shares.
        target_surb_buffer_size: 64,
        max_surbs_per_sec: 40,
        ..Default::default()
    }
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
                        // Signal deposit immediately to abort the kill switch. Not optional as of
                        // hopr-api 4.0.1: the event always carries a channel to report it on.
                        let mut notifier = data.deposit_updated;
                        notifier
                            .send((data.id, HoprBalance::new_base(1)))
                            .await
                            .context("failed to signal deposit via notifier")?;
                        tracing::info!(id = ?data.id, "deposit signaled");
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
                    PixEvent::DepositDataRequest(request) => {
                        // Stands in for the deposit pool. The Exit blocks its SSA request on this and
                        // fails the Session if it goes unanswered, so a test driving PIX cycles has to
                        // answer it. Empty data is a valid answer: it says the pool has nothing to
                        // attach, which is what this test models.
                        let mut created = request.deposit_data_created;
                        for id in request.deposit_ids {
                            created
                                .send(PixDepositData {
                                    id,
                                    data: Box::default(),
                                })
                                .await
                                .context("failed to answer the deposit data request")?;
                        }
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
                // Strictly above the idle deadline, which `validate_pix_supervision` requires: a
                // backstop at or below the rule it backs up pre-empts that rule on every cycle.
                // Still far outside the observation window, which is all this test asks of it.
                max_recovery_time: OTHER_CLOCKS * 2,
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
                            held_notifiers.push(data.deposit_updated);
                            tracing::info!(id = ?data.id, quota = data.quota,
                                "Exit: DepositAddressReceived — holding the notifier, never signalling a deposit");
                        }
                        PixEvent::PrivateKeyRecovered(data) => {
                            anyhow::bail!("recovery completed without a deposit: {:?}", data.id);
                        }
                        PixEvent::DepositDataRequest(request) => {
                            // Stands in for the deposit pool, which the Exit blocks its SSA request
                            // on. Unanswered, the Session dies of *that* rather than of the deposit
                            // deadline under test — and it dies before the Entry ever commits, so
                            // the clock this test measures is never armed at all.
                            let mut created = request.deposit_data_created;
                            for id in request.deposit_ids {
                                created
                                    .send(PixDepositData {
                                        id,
                                        data: Box::default(),
                                    })
                                    .await
                                    .context("failed to answer the deposit data request")?;
                            }
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

    // Attribution comes from the observation budget, not from comparing `elapsed` against
    // OTHER_CLOCKS. The loop above is wrapped in a timeout of MAX_DEPOSIT_WAIT + OBSERVATION_SLACK,
    // so `elapsed` cannot reach OTHER_CLOCKS and an assertion against it could never fail — it
    // looked like it established attribution and established nothing. What actually has to hold is
    // a property of the setup, so that is what is asserted: every other clock configured beyond the
    // window in which the closure is observed. Lower one of them under the budget and this fails,
    // which is the edit that would silently destroy the attribution.
    assert!(
        OTHER_CLOCKS > MAX_DEPOSIT_WAIT + OBSERVATION_SLACK,
        "the other supervisor clocks ({OTHER_CLOCKS:?}) must sit outside the observation budget ({:?}), or a closure \
         seen inside it cannot be attributed to the deposit deadline",
        MAX_DEPOSIT_WAIT + OBSERVATION_SLACK
    );

    // And it must not have closed before the deadline could expire, which is what rules out an
    // immediate failure wearing the deposit clock's clothes.
    //
    // `armed_at` is taken when this test's event stream *delivers* `DepositAddressReceived`, which is
    // strictly after the Exit armed the clock, so the measured interval understates the real one by
    // however long that delivery took. Against a 5 s deadline on a loaded cluster that is not a
    // negligible fraction, hence the tolerance: without it a slow stream fails the test and reports
    // it as "something other than the deposit deadline closed it".
    const DELIVERY_LAG_TOLERANCE: Duration = Duration::from_secs(1);
    let elapsed = stopped_at.saturating_duration_since(armed_at);
    assert!(
        elapsed + DELIVERY_LAG_TOLERANCE >= MAX_DEPOSIT_WAIT,
        "closed {elapsed:?} after the deposit clock was observed armed, more than {DELIVERY_LAG_TOLERANCE:?} short of \
         its {MAX_DEPOSIT_WAIT:?} deadline — so something other than the deposit deadline closed it"
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
                // The sender is taken only on the first request, and only inside the branch that
                // uses it: taking it unconditionally would close the channel on a later event, and
                // `notifier_rx` below would resolve to `Canceled` and fail the test with a message
                // naming a supervisor defect that had not occurred.
                PixEvent::DepositAddressReceived(data) => {
                    if let Some(notifier_tx) = notifier_tx.take() {
                        tracing::info!(id = ?data.id, "Exit: DepositAddressReceived — withholding the deposit");
                        let _ = notifier_tx.send((data.id, data.deposit_updated));
                    } else {
                        tracing::debug!(id = ?data.id, "further deposit request");
                    }
                }
                // Stands in for the deposit pool. Left unanswered, the Exit never gets as far as
                // asking for a deposit, and the `notifier_rx` below never resolves.
                PixEvent::DepositDataRequest(request) => {
                    let mut created = request.deposit_data_created;
                    for id in request.deposit_ids {
                        if created
                            .send(PixDepositData {
                                id,
                                data: Box::default(),
                            })
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }
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
                // Far out of reach, so that a deposit which silently failed to register cannot look
                // like the deadline under test firing.
                max_deposit_wait: Duration::from_secs(600),
                // At its floor (`>= max_ack_await_time`), which puts it *below* the deadline under
                // test. What stops it firing first is not its value but its gating: the idle rule
                // only runs while the Session is consuming service, and nothing is served between
                // the deposit below and the wait that follows. `validate_pix_supervision` requires
                // `max_recovery_time > max_recovery_idle`, so the previous formulation — idle parked
                // at 600 s to put it "out of reach" — is no longer expressible; it was belt and
                // braces rather than what isolated the deadline.
                max_recovery_idle: Duration::from_secs(30),
                // The deadline under test.
                max_recovery_time: Duration::from_secs(40),
                // Off, so that this test measures the deadline and nothing else. With fill on, an
                // Exit that had SURBs would carry the cycle to recovery and the deadline would never
                // fire — which is the whole point of fill, and is what
                // `recovery_hard_deadline_closes_a_session_fill_cannot_supply` covers instead.
                fill: hopr_lib::exports::transport::session::PixFillConfig {
                    enabled: false,
                    ..Default::default()
                },
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
                    // Returns only once something was actually funded: an SSA left unfunded never
                    // enters recovery, so `max_recovery_time` would not be the clock that governs —
                    // and the assertion 80 s below would still name it, reporting a supervisor
                    // defect that did not occur. The `bail!` past the loop covers the case where no
                    // request arrives at all.
                    let mut notifier = data.deposit_updated;
                    notifier
                        .send((data.id, HoprBalance::new_base(1)))
                        .await
                        .context("failed to signal deposit via notifier")?;
                    tracing::info!(id = ?data.id, "deposit signalled, no traffic flowing");
                    return anyhow::Ok(());
                }
                // Stands in for the deposit pool, which the Exit blocks its SSA request on. Left
                // unanswered, no deposit is ever asked for and the `bail!` below is what this test
                // reports — naming the Exit where the fault would be the missing pool.
                PixEvent::DepositDataRequest(request) => {
                    let mut created = request.deposit_data_created;
                    for id in request.deposit_ids {
                        created
                            .send(PixDepositData {
                                id,
                                data: Box::default(),
                            })
                            .await
                            .context("failed to answer the deposit data request")?;
                    }
                }
                other => tracing::debug!("Exit PixEvent while awaiting the deposit request: {other:?}"),
            }
        }
        anyhow::bail!("the Exit never asked for a deposit")
    })
    .await
    .context("timed out waiting for the deposit request")??;

    // Wait out the recovery deadline with the Session idle. Comfortably past the 40 s deadline, and
    // still inside `TEST_GLOBAL_TIMEOUT` with the cluster bootstrap and the observation below.
    tokio::time::sleep(Duration::from_secs(50)).await;

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

/// The headline property: an idle funded cycle is carried to recovery by the Exit's own fill.
///
/// Nothing is written on this Session after it is established. Every share that reaches the Exit's
/// reconstructor therefore rode a keep-alive the Exit originated for itself, and the cycle recovering
/// at all is the proof — before fill existed this Session's deposit was simply lost, because a cycle
/// only recovers once its whole emission has ridden back to the Entry and an idle application sends
/// none of it.
///
/// Three assertions, in the order they matter:
///
/// * the cycle recovers, and does so inside the aim point fill was planned against rather than merely inside the
///   deadline — a cycle that only just scrapes the hard deadline has no room for its successor's commitment and deposit
///   round trip;
/// * the *successor* is requested and funded, which is what proves the completion was not a dead end. This is the half
///   that the Entry-side `returned_packets` fix exists for: a successor asked for on the strength of keep-alives alone
///   is refused as under-served by an Entry that does not credit them, and the Session then dies on
///   `max_ssa_delivery_time` blaming a timer;
/// * an echo still round-trips afterwards, so the Session was kept alive rather than merely kept accounted for.
///
/// Run over both Exit egress branches. `NoRateControl` is what every other test here uses, but it is
/// the *rate-controlled* branch that every gnosis client opens: it shapes the Exit's data egress with
/// a SURB balancer, and it is the only branch on which the Entry announces its buffer target at all —
/// so it is also the branch on which fill's SURB reserve is derived from something the peer said
/// rather than from this node's own fallback. Fill itself is unaffected by the shaping, since the
/// keep-alive stream carries its own rate controller rather than riding the data path's, and this is
/// what says so rather than leaving it to be inferred from the code.
#[cfg(feature = "session-client")]
#[rstest]
#[case(1, false)]
#[case(1, true)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
async fn idle_session_is_completed_by_exit_fill(#[case] hops: usize, #[case] rate_control: bool) -> anyhow::Result<()> {
    let exit_pix = fill_pix_config(Default::default());
    let aim_point = exit_pix
        .supervision
        .max_recovery_time
        .mul_f64(exit_pix.supervision.fill.finish_fraction);

    let cluster = build_pix_cluster(hops, exit_pix, Duration::from_secs(120)).await?;

    // Before the Session exists: the Exit's very first SSA request blocks on the deposit pool.
    let (driver, mut milestones) = spawn_exit_pix_driver(&cluster);
    let mut session = establish_pix_session_with(&cluster, hops, Some(idle_surb_balancer()), rate_control).await?;
    tracing::info!(
        rate_control,
        "session established; not one application byte will be written to it"
    );

    // Both milestones carry `PixAddressId::new(pseudonym, ssa_index)`, so the cycle that recovers and
    // the deposit that funded it share one id. Correlating on that is what makes "the successor" mean
    // a *different* cycle: counting `Funded`s instead would accept a second report about the same one,
    // and at the default `ssas_per_request` of 1 the only other id a batch can produce is the
    // successor's. The successor's deposit routinely lands *before* full recovery is reported, since
    // it rides the early-recovery signal, so neither ordering may be assumed.
    let successor_of = |seen: &[(PixMilestone, std::time::Instant)], recovered| {
        seen.iter().find_map(|(milestone, at)| match milestone {
            PixMilestone::Funded(id) if *id != recovered => Some(*at),
            _ => None,
        })
    };
    let recovered_in = |seen: &[(PixMilestone, std::time::Instant)]| {
        seen.iter().find_map(|(milestone, at)| match milestone {
            PixMilestone::Recovered(id) => Some((*id, *at)),
            _ => None,
        })
    };

    let started = std::time::Instant::now();
    let seen = await_milestones(&mut milestones, aim_point * 2, |seen| {
        // The successor being funded is the end of the property: the first cycle recovered on fill
        // alone, and the Entry admitted the request that recovery earned.
        recovered_in(seen).is_some_and(|(recovered, _)| successor_of(seen, recovered).is_some())
    })
    .await;

    let (recovered_id, recovered_at) = recovered_in(&seen).with_context(|| {
        format!(
            "no idle cycle was completed by fill within {:?}: {seen:?}",
            aim_point * 2
        )
    })?;
    let successor_at = successor_of(&seen, recovered_id).with_context(|| {
        format!(
            "cycle {recovered_id:?} recovered, but no *other* cycle was ever funded, so the completion bought \
             nothing: {seen:?}"
        )
    })?;

    // The later of the two, because the successor's deposit rides the *early* recovery signal and so
    // routinely lands before full recovery is reported. Asserting on whichever happened to be first
    // would let a late completion pass.
    let elapsed = recovered_at.max(successor_at).duration_since(started);
    tracing::info!(
        recovered_after = ?recovered_at.duration_since(started),
        successor_after = ?successor_at.duration_since(started),
        ?aim_point,
        "idle cycle completed and its successor funded"
    );
    assert!(
        elapsed < aim_point,
        "the idle cycle and its successor took {elapsed:?}, past the {aim_point:?} fill was planned against"
    );

    // And the Session is alive rather than merely accounted for. A *completed* round trip, not an
    // absence of failure: `spawn_echo_task` only reports a stop on a write error, a read error, or a
    // 30 s read timeout, so watching it for ten seconds passes while the very first `read_exact` is
    // still hanging — which is exactly the state a Session that had been kept accounted for but not
    // alive would be in.
    const ECHOES: usize = 3;
    let completed = echo_n(&mut session, ECHOES, Duration::from_millis(200)).await;
    assert_eq!(
        ECHOES, completed,
        "a Session completed by fill must still carry application traffic, but only {completed} of {ECHOES} echo \
         round trips finished"
    );
    driver.abort();

    tracing::info!(hops, rate_control, "idle PIX fill test PASSED");
    Ok(())
}

/// A Session closed at the Exit still finishes the cycle its deposit has already paid for.
///
/// The close here is the real one rather than a simulated one: the Exit's session server hands the
/// `IncomingSession` back to the test, which closes its write half — the same `WriteClosed` route a
/// session server takes when its downstream connection ends. Before this, that tore the Exit's
/// Session down at once, taking the reconstructor state with it; the deposit the Entry had already
/// paid for the cycle in flight was stranded, since the address derives from both nodes' commitments
/// and nothing refunds it.
///
/// The two cases are the property and its control, and they differ in one config flag:
///
/// * with `drain_after_close` on, a `Recovered` milestone arrives **after** the close instant, which can only mean the
///   Exit went on spending its buffered SURBs on the cycle with no Session left to serve;
/// * with it off, none does, because the close is the immediate teardown it has always been.
///
/// Two guards keep the comparison honest. The cycle must not have recovered *before* the close — at
/// [`PIX_PARAMS`]' 32-packet cycle against the 45 s aim point, fill alone needs about forty seconds,
/// so closing a few seconds in leaves most of the cycle outstanding — and no successor may be funded
/// after the close, since a drained Session has no quota left to serve one with.
#[cfg(feature = "session-client")]
#[rstest]
#[case(1, true)]
#[case(1, false)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
async fn a_closed_exit_session_drains_its_surbs_into_the_funded_cycle(
    #[case] hops: usize,
    #[case] drain_after_close: bool,
) -> anyhow::Result<()> {
    /// Long enough for the Entry's balancer to reach its 64-SURB target, short enough that fill has
    /// spent only a handful of the cycle's 32 packets by the time the close lands.
    const SETTLE: Duration = Duration::from_secs(8);
    /// What the drain gets, and what the control case waits out in full. Above the 30 s
    /// `max_recovery_idle`, which is no longer service-gated while draining and is therefore the
    /// deadline a drain that stops making progress actually dies on.
    const DRAIN_BUDGET: Duration = Duration::from_secs(35);

    let exit_pix = fill_pix_config(hopr_lib::exports::transport::session::PixFillConfig {
        drain_after_close,
        ..Default::default()
    });
    let aim_point = exit_pix
        .supervision
        .max_recovery_time
        .mul_f64(exit_pix.supervision.fill.finish_fraction);

    let (exit_server, captured) = hopr_lib::testing::dummies::SessionCaptureServer::new();
    let cluster = build_pix_cluster_with_exit_server(hops, exit_pix, Duration::from_secs(120), exit_server).await?;

    // Before the Session exists, as in every fill test here: the Exit's first SSA request blocks on
    // the deposit pool with a budget shorter than `connect_to` takes to return.
    let (driver, mut milestones) = spawn_exit_pix_driver(&cluster);
    // The rate-controlled branch, because it is the only one on which the Entry announces its SURB
    // buffer target — and that target is what the drain's reserve, and so its own admission
    // threshold, is derived from.
    let _session = establish_pix_session_with(&cluster, hops, Some(idle_surb_balancer()), true).await?;

    let funded = await_milestones(&mut milestones, aim_point, |seen| {
        seen.iter()
            .any(|(milestone, _)| matches!(milestone, PixMilestone::Funded(_)))
    })
    .await;
    anyhow::ensure!(
        funded
            .iter()
            .any(|(milestone, _)| matches!(milestone, PixMilestone::Funded(_))),
        "no cycle was funded within {aim_point:?}, so there was never a deposit at stake: {funded:?}"
    );

    // Doubles as the settle window and as a drain of the milestone channel, so that everything
    // observed after the close really did arrive after it.
    let before_close = await_milestones(&mut milestones, SETTLE, |_| false).await;
    anyhow::ensure!(
        !before_close
            .iter()
            .any(|(milestone, _)| matches!(milestone, PixMilestone::Recovered(_))),
        "the funded cycle recovered before the Session was closed, so this test cannot tell a drain from ordinary \
         fill: {before_close:?}"
    );

    let mut exit_session = captured
        .lock()
        .map_err(|_| anyhow::anyhow!("the captured-session mutex was poisoned"))?
        .take()
        .context("the Exit's session server must have captured the incoming session")?;
    let closed_at = std::time::Instant::now();
    tokio::time::timeout(Duration::from_secs(30), exit_session.session.close())
        .await
        .context("closing the exit-side session timed out")?
        .context("closing the exit-side session failed")?;
    tracing::info!(drain_after_close, "exit-side session closed");

    let after_close = await_milestones(&mut milestones, DRAIN_BUDGET, |seen| {
        seen.iter()
            .any(|(milestone, at)| matches!(milestone, PixMilestone::Recovered(_)) && *at >= closed_at)
    })
    .await;
    let drained = after_close
        .iter()
        .find(|(milestone, at)| matches!(milestone, PixMilestone::Recovered(_)) && *at >= closed_at);

    // The observation path has to have outlived the budget, or "nothing recovered after the close"
    // would be a statement about this test rather than about the Exit. Checked for both cases: it is
    // what gives the control branch's negative assertion any force, and in the drain branch it turns
    // a dead event driver into a diagnosis rather than an accusation of a stranded deposit.
    anyhow::ensure!(
        !driver.is_finished(),
        "the Exit PIX event driver stopped before the {DRAIN_BUDGET:?} drain budget elapsed, so nothing observed \
         after the close proves anything either way"
    );

    if drain_after_close {
        let (_, recovered_at) = drained.with_context(|| {
            format!(
                "the Exit held enough SURBs to finish its funded cycle but stopped working for it when the Session \
                 closed, so the deposit is stranded; nothing recovered within {DRAIN_BUDGET:?}: {after_close:?}"
            )
        })?;
        tracing::info!(drained_after = ?recovered_at.duration_since(closed_at), "post-close drain completed");

        assert!(
            !after_close
                .iter()
                .any(|(milestone, at)| matches!(milestone, PixMilestone::Funded(_)) && *at >= closed_at),
            "a drained Session has no quota left to serve, so it must never order a successor cycle the Entry would \
             have to deposit for: {after_close:?}"
        );
    } else {
        assert!(
            drained.is_none(),
            "with draining disabled an Exit-side close must tear the Session down at once, but a cycle recovered {:?} \
             after it: {after_close:?}",
            drained.map(|(_, at)| at.duration_since(closed_at))
        );
    }

    driver.abort();

    tracing::info!(hops, drain_after_close, "post-close SURB drain test PASSED");
    Ok(())
}

/// Fill picks a cycle up from wherever the application left it.
///
/// The deployment shape this is about is the common one — a VPN Session that transfers and then goes
/// quiet — and the trap in it is arithmetic rather than plumbing: the planner has to re-derive its
/// rate from the shares that are actually outstanding, not from the figure it would have asked for at
/// the start. A planner replaying its opening rate would over-send by however much the application
/// already contributed; one that never re-planned at all would keep filling at the heartbeat it fell
/// to while the application was busy, and the cycle would strand.
#[cfg(feature = "session-client")]
#[rstest]
#[case(1)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
async fn fill_resumes_after_organic_traffic_stops(#[case] hops: usize) -> anyhow::Result<()> {
    let exit_pix = fill_pix_config(Default::default());
    let aim_point = exit_pix
        .supervision
        .max_recovery_time
        .mul_f64(exit_pix.supervision.fill.finish_fraction);

    let cluster = build_pix_cluster(hops, exit_pix, Duration::from_secs(120)).await?;

    let (driver, mut milestones) = spawn_exit_pix_driver(&cluster);
    // Held in this scope for the rest of the test: the Session must outlive its own traffic, since
    // dropping it is what closes it.
    let mut session = establish_pix_session_with(&cluster, hops, Some(idle_surb_balancer()), false).await?;

    // A short burst of application traffic, then silence for the rest of the Session.
    // A quarter of a cycle of application traffic: enough that the remainder fill has to plan against
    // is genuinely smaller than the whole, and few enough that the cycle cannot finish on the
    // application's own packets. Then silence for the rest of the Session.
    const ECHOED: usize = 8;
    let echoed = echo_n(&mut session, ECHOED, Duration::from_millis(200)).await;
    assert_eq!(
        ECHOED, echoed,
        "the application must actually contribute before this test can say fill finished the rest"
    );

    let mut drained = Vec::new();
    while let Ok(Some(milestone)) = milestones.try_next() {
        drained.push(milestone);
    }
    assert!(
        !drained
            .iter()
            .any(|(milestone, _)| matches!(milestone, PixMilestone::Recovered(_))),
        "the application finished the cycle by itself, so there is nothing left for fill to prove: {drained:?}"
    );
    tracing::info!(
        echoed,
        ?drained,
        "application traffic stopped; the cycle is fill's to finish"
    );

    let started = std::time::Instant::now();
    let seen = await_milestones(&mut milestones, aim_point, |seen| {
        seen.iter()
            .any(|(milestone, _)| matches!(milestone, PixMilestone::Recovered(_)))
    })
    .await;
    driver.abort();

    let recovered_at = seen
        .iter()
        .find(|(milestone, _)| matches!(milestone, PixMilestone::Recovered(_)))
        .map(|(_, at)| *at)
        .with_context(|| format!("fill did not finish the cycle the application abandoned: {seen:?}"))?;
    tracing::info!(after = ?recovered_at.duration_since(started), "the abandoned cycle recovered on fill alone");
    drop(session);

    tracing::info!(hops, "fill resumption test PASSED");
    Ok(())
}

/// A busy Session's cycles complete, and fill does not get in their way.
///
/// The unit-level property — that fill drops to its heartbeat once organic egress covers the
/// requirement — is pinned where it is decided, in the planner. What this adds is the consequence on
/// a real cluster: an application sending far above the required rate completes its cycles and keeps
/// echoing throughout, so the extra stream neither starves the data path of SURBs nor wedges the
/// egress gate behind its own traffic.
#[cfg(feature = "session-client")]
#[rstest]
#[case(1)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
async fn fill_yields_to_organic_traffic(#[case] hops: usize) -> anyhow::Result<()> {
    let exit_pix = fill_pix_config(Default::default());
    let cluster = build_pix_cluster(hops, exit_pix, Duration::from_secs(120)).await?;

    let (driver, mut milestones) = spawn_exit_pix_driver(&cluster);
    let session = establish_pix_session_with(&cluster, hops, Some(idle_surb_balancer()), false).await?;

    let stopped = EchoStopCell::default();
    let echo = spawn_echo_task(session, stopped.clone(), Duration::from_secs(30));

    let seen = await_milestones(&mut milestones, Duration::from_secs(90), |seen| {
        seen.iter()
            .filter(|(milestone, _)| matches!(milestone, PixMilestone::Recovered(_)))
            .count()
            >= 2
    })
    .await;
    driver.abort();

    let recovered = seen
        .iter()
        .filter(|(milestone, _)| matches!(milestone, PixMilestone::Recovered(_)))
        .count();
    assert!(
        recovered >= 2,
        "a busy PIX Session completed only {recovered} cycle(s): {seen:?}"
    );
    assert_eq!(
        None,
        stopped.get().map(|(stop, _)| stop),
        "the echo must survive its own Session's fill"
    );
    echo.abort();

    tracing::info!(hops, "fill yields to organic traffic test PASSED");
    Ok(())
}

/// The backstop still closes a Session whose Entry supplies nothing to fill with.
///
/// Fill is not a way around the recovery deadline, and this is the case that proves it: no balancer
/// on the Entry, so the Exit is handed no SURBs, so the filler — which yields to the SURB reserve
/// precisely so that it cannot stall the node — sends nothing and the cycle strands exactly as it
/// would have before. Its sibling above, with fill disabled outright, isolates the deadline itself;
/// this one shows that turning fill *on* does not disarm it.
#[cfg(feature = "session-client")]
#[rstest]
#[case(1)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
async fn recovery_hard_deadline_closes_a_session_fill_cannot_supply(#[case] hops: usize) -> anyhow::Result<()> {
    #[allow(unexpected_cfgs)]
    if cfg!(coverage) && hops > 1 {
        return Ok(());
    }

    let cluster = build_pix_cluster(
        hops,
        IncomingSessionPixConfig {
            quota_range: 0..=100_000,
            enforce_pix: false,
            max_live_cycle_bytes: IncomingSessionPixConfig::default().max_live_cycle_bytes,
            supervision: SupervisorConfig {
                max_ssa_delivery_time: Duration::from_secs(10),
                max_deposit_wait: Duration::from_secs(600),
                max_recovery_idle: Duration::from_secs(30),
                // The deadline under test, and fill is on.
                max_recovery_time: Duration::from_secs(40),
                ..Default::default()
            },
        },
        Duration::from_secs(120),
    )
    .await?;

    let mut exit_events = Box::pin(cluster.exit.inner().subscribe_pix_events());

    // No balancer: the Entry produces SURBs only alongside its own packets, and it sends none.
    let session = establish_pix_session_with(&cluster, hops, None, false).await?;

    tokio::time::timeout(Duration::from_secs(60), async {
        while let Some(event) = exit_events.next().await {
            match event {
                PixEvent::DepositAddressReceived(data) => {
                    let mut notifier = data.deposit_updated;
                    notifier
                        .send((data.id, HoprBalance::new_base(1)))
                        .await
                        .context("failed to signal deposit via notifier")?;
                    tracing::info!(id = ?data.id, "deposit signalled, no traffic and no SURBs");
                    return anyhow::Ok(());
                }
                PixEvent::DepositDataRequest(request) => {
                    let mut created = request.deposit_data_created;
                    for id in request.deposit_ids {
                        created
                            .send(PixDepositData {
                                id,
                                data: Box::default(),
                            })
                            .await
                            .context("failed to answer the deposit data request")?;
                    }
                }
                other => tracing::debug!("Exit PixEvent while awaiting the deposit request: {other:?}"),
            }
        }
        anyhow::bail!("the Exit never asked for a deposit")
    })
    .await
    .context("timed out waiting for the deposit request")??;

    tokio::time::sleep(Duration::from_secs(50)).await;

    let stopped = EchoStopCell::default();
    let _echo = spawn_echo_task(session, stopped.clone(), Duration::from_secs(10));
    tokio::time::timeout(Duration::from_secs(30), async {
        while stopped.get().is_none() {
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    })
    .await
    .context("a session fill could not supply survived its absolute recovery deadline")?;

    let (stop, _) = stopped.get().context("the echo task never stopped")?;
    assert!(
        stop.is_closure(),
        "the Session must be observed closed after its recovery deadline, not merely quiet; got {stop:?}"
    );

    tracing::info!(hops, "recovery hard deadline with fill enabled test PASSED");
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

/// 1-hop PIX session in which the Exit derives [`SSA_BATCH`] SSAs per `SsaRequest` from the Entry's
/// sub-range per-SSA quota.
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
    let quota_per_ssa = PIX_POLYS as u64
        * (PIX_SHARES as u64 + PIX_SURPLUS as u64)
        * hopr_lib::exports::transport::PACKET_PAYLOAD_SIZE as u64;
    let accepted_batch_quota = quota_per_ssa * SSA_BATCH as u64;

    // The Exit's range admits the Entry's dimensions only as a batch of SSA_BATCH, and the Entry has
    // to accept that many or it refuses every derived request outright.
    let cluster = build_pix_cluster_with_entry_cap(
        hops,
        IncomingSessionPixConfig {
            quota_range: accepted_batch_quota..=accepted_batch_quota,
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

    // Traffic is what moves shares, so the cycles only advance while this runs. The stop reason is
    // kept rather than discarded: it is not what this test asserts, but it is the difference between
    // the two ways the observation below can time out — see the diagnostic there.
    let echo_stopped = EchoStopCell::default();
    let echo_handle = spawn_echo_task(session, echo_stopped.clone(), Duration::from_secs(10));

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
                            let mut notifier = data.deposit_updated;
                            notifier
                                .send((data.id, HoprBalance::new_base(1)))
                                .await
                                .context("failed to signal deposit via notifier")?;
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
                        PixEvent::DepositDataRequest(request) => {
                            // See the first test: unanswered, this is fatal to the Session.
                            let mut created = request.deposit_data_created;
                            for id in request.deposit_ids {
                                created
                                    .send(PixDepositData {
                                        id,
                                        data: Box::default(),
                                    })
                                    .await
                                    .context("failed to answer the deposit data request")?;
                            }
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

    // The stop cell is reported rather than a cause asserted. Shares travel only with data-packet
    // acknowledgements, so a dead echo task halts every cycle and produces exactly the same timeout
    // as the supervisor defect this test is looking for — and at a 150 s budget, naming the wrong
    // one is an expensive thing to hand an operator.
    tokio::time::timeout(BATCH_OBSERVATION_BUDGET, observe)
        .await
        .with_context(|| {
            format!(
                "timed out observing two SSA batches (echo task: {:?}). Still running means the Exit most likely \
                 stopped requesting SSAs after the first batch, which is how an off-by-one in the supervisor's index \
                 bookkeeping manifests; already stopped means the cycles simply ran out of traffic and this says \
                 nothing about the supervisor",
                echo_stopped.get()
            )
        })??;

    // ── 1. The whole batch is allocated before the first cycle completes ──
    let before_first_recovery = addresses_before_first_recovery.context("the Exit never recovered a key")?;
    assert!(
        before_first_recovery >= SSA_BATCH,
        "the Exit was told about only {before_first_recovery} deposit address(es) before it recovered its first key; \
         a batch of {SSA_BATCH} is allocated up front, so it should already know all {SSA_BATCH}. Getting one or two \
         here means the batch size did not take effect. exit_ids={exit_ids:?}",
    );

    // ── 2. Contiguous indices from 1, across the batch boundary ───────────
    // `PixAddressId` is an opaque struct as of hopr-api 4.0.1, not a (pseudonym, index) tuple.
    let mut indices: Vec<u32> = exit_ids.iter().map(|id| id.ssa_index().get()).collect();
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
