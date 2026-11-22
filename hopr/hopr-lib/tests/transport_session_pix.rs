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
            HasChainApi, HasExitIncentivization, HasTransportApi, HoprSessionClientOperations,
            IncentiveChannelOperations, PixDepositData, PixEvent,
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

// PIX params: 8 polys × (2 + 2) shares × one packet payload per share, per SSA cycle.
const PIX_POLYS: u16 = 8;
const PIX_SHARES: u8 = 2;

/// Surplus shares the Entry is configured with.
///
/// Set explicitly rather than left to derive, because every surplus share is another round-trip
/// packet — see the comment on `additional_shares` below — and because a value that differs from
/// what the derivation would produce is what makes it visible whether the surplus really crossed
/// the wire. At `PIX_SHARES = 2` the derivation yields 1, so this being 2 is observable end to end.
const PIX_SURPLUS: u8 = 2;

/// The three above as the node announces them.
///
/// Not something the Session asks for: the dimensions are read off the installed generator, so this
/// is the same quadruple arriving by the other route — what `PIX_POLYS`/`PIX_SHARES`/`PIX_SURPLUS`
/// assemble into once they have been through `PixGlobalConfig` below.
///
/// `const` rather than built at the call site so the range check runs at compile time — the values
/// are constants, so a typo here should not need a cluster to boot before it is noticed.
const PIX_PARAMS: hopr_lib::PixParams =
    match hopr_lib::PixParams::try_new(PIX_POLYS, PIX_SHARES, PIX_SURPLUS, hopr_lib::LOCAL_PIX_SUITE) {
        Ok(params) => params,
        Err(_) => panic!("test PIX parameters must be within the protocol ranges"),
    };

/// Quota one SSA cycle costs, as the Exit computes it when deciding whether to accept the Session.
///
/// Mirrors `pix_params_to_quota`: `polys × (shares + surplus) × PIX_QUOTA_BYTES_PER_SHARE`. Derived
/// rather than written as a literal because the share size is a build-time constant that moves —
/// a hard-coded ceiling here does not fail loudly when it goes stale, it just makes every PIX
/// session in this file get rejected with `UnacceptablePixParams` after a cluster has booted.
const PIX_QUOTA_PER_SSA: u64 = PIX_POLYS as u64
    * (PIX_SHARES as u64 + PIX_SURPLUS as u64)
    * hopr_lib::exports::transport::session::PIX_QUOTA_BYTES_PER_SHARE;

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
        None,
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
    build_pix_cluster_with(hops, exit_pix, idle_timeout, default_cap, exit_server, None).await
}

/// The cluster the three builders above are all a special case of.
#[cfg(feature = "session-client")]
async fn build_pix_cluster_with<Srv>(
    hops: usize,
    exit_pix: IncomingSessionPixConfig,
    idle_timeout: Duration,
    entry_max_ssas_per_request: usize,
    exit_server: Srv,
    entry_establish_retry_timeout: Option<Duration>,
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
            establish_retry_timeout: entry_establish_retry_timeout,
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
    establish_pix_session_with_surbs(cluster, hops, surb_management, rate_control, 1).await
}

/// As [`establish_pix_session_with`], but with the number of SURBs each organic data packet may carry
/// under the caller's control (`HoprSessionClientConfig::max_surbs_per_data_packet`).
///
/// Everything else here pins it to `1`, which follows the Entry's SURB balancer: a packet carries a
/// SURB only while the Exit is estimated to be below the balancer's target. `usize::MAX` lifts the cap,
/// so every packet carries as many SURBs as fit beside its payload, whatever the balancer says. That is
/// the production shape [`capture_n_hop_pix_session`] needs to reproduce an Entry that out-produces its
/// Exit.
#[cfg(feature = "session-client")]
async fn establish_pix_session_with_surbs(
    cluster: &hopr_lib::testing::fixtures::RoleClusterGuard,
    hops: usize,
    surb_management: Option<hopr_lib::SurbBalancerConfig>,
    rate_control: bool,
    max_surbs_per_data_packet: usize,
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
                max_surbs_per_data_packet,
                flow_control: None,
                max_frames_behind_gap: None,
            },
        ),
    )
    .await
    .context("session connection timed out after 120s")??;
    Ok(session)
}

/// As [`establish_pix_session`], but under a pseudonym the caller names, the way a client reconnects.
#[cfg(feature = "session-client")]
async fn establish_pix_session_under(
    cluster: &hopr_lib::testing::fixtures::RoleClusterGuard,
    hops: usize,
    pseudonym: Option<hopr_lib::api::types::internal::protocol::HoprPseudonym>,
) -> anyhow::Result<hopr_lib::HoprSession> {
    establish_pix_session_under_with(cluster, hops, pseudonym, None, false).await
}

/// As [`establish_pix_session_under`], with the Entry's SURB balancer and the Exit's rate control under
/// the caller's control (see [`establish_pix_session_with`]).
#[cfg(feature = "session-client")]
async fn establish_pix_session_under_with(
    cluster: &hopr_lib::testing::fixtures::RoleClusterGuard,
    hops: usize,
    pseudonym: Option<hopr_lib::api::types::internal::protocol::HoprPseudonym>,
    surb_management: Option<hopr_lib::SurbBalancerConfig>,
    rate_control: bool,
) -> anyhow::Result<hopr_lib::HoprSession> {
    try_establish_pix_session_under_with(cluster, hops, pseudonym, surb_management, rate_control)
        .await?
        .context("session connection failed")
}

/// [`establish_pix_session_under_with`], keeping the library's own error (the outer error is a timeout).
#[cfg(feature = "session-client")]
async fn try_establish_pix_session_under_with(
    cluster: &hopr_lib::testing::fixtures::RoleClusterGuard,
    hops: usize,
    pseudonym: Option<hopr_lib::api::types::internal::protocol::HoprPseudonym>,
    surb_management: Option<hopr_lib::SurbBalancerConfig>,
    rate_control: bool,
) -> anyhow::Result<Result<hopr_lib::HoprSession, hopr_lib::errors::HoprLibError>> {
    let routing = hops.try_into()?;
    let capabilities = if rate_control {
        SessionCapability::Segmentation | SessionCapability::UsePIX
    } else {
        SessionCapability::Segmentation | SessionCapability::NoRateControl | SessionCapability::UsePIX
    };
    let connected = tokio::time::timeout(
        Duration::from_secs(120),
        cluster.entry.inner().connect_to(
            cluster.exit.address(),
            SessionTarget::UdpStream(SealedHost::Plain(IpOrHost::from_str(":0")?)),
            HoprSessionClientConfig {
                forward_path: routing,
                return_path: routing,
                capabilities,
                pseudonym,
                surb_management,
                max_surbs_per_data_packet: 1,
                flow_control: None,
                max_frames_behind_gap: None,
            },
        ),
    )
    .await
    .context("session connection timed out after 120s")?;
    Ok(connected.map(|(session, _)| session))
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
        resume_window: IncomingSessionPixConfig::default().resume_window,
        quota_range: 0..=PIX_QUOTA_PER_SSA,
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
/// Production does not have the problem: the shipped 7 000 SURBs against a 454 656-packet cycle is
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
#[case(1, 1)]
#[case(1, usize::MAX)]
#[case(2, 1)]
#[case(3, 1)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
/// n-hop PIX multi-cycle session test.
///
/// Creates a (n+2)-node role-typed cluster (Entry, N relays, Exit) where each
/// node is built with the correct transport role. The Exit accepts tiny PIX
/// quotas. Keeps symmetric 32-byte traffic flowing Entry↔Exit while observing
/// the PIX event cycle repeat 3 times.
///
/// The second case parameter is the Entry's `max_surbs_per_data_packet`. `1` follows the SURB balancer's
/// gate, the shipped default. `usize::MAX` lifts the cap, so every 32-byte packet carries as many SURBs
/// as fit beside its payload, and that case reproduces hoprnet#8466. The Entry then out-produces the
/// Exit, which spends one SURB per echoed reply. A PIX share is sealed into a SURB when the Entry mints
/// it and reaches the Exit only when the Exit *uses* that SURB, and the Entry has no share left to seal
/// once it has emitted the committed cycle: from then on every SURB it mints is share-less. The next
/// cycle's shares are minted only after the Exit requests that cycle, so they queue behind an
/// ever-growing share-less backlog. Without an Exit SURB store that hands out share-bearing SURBs first,
/// a later SSA is never recovered and the supervisor closes the Session as idle. That case observes one
/// cycle more than the others, because the early cycles still slip through; see `target_cycles` below.
async fn capture_n_hop_pix_session(
    #[case] hops: usize,
    #[case] max_surbs_per_data_packet: usize,
) -> anyhow::Result<()> {
    // 2-hop and 3-hop tests are too slow under coverage instrumentation
    #[allow(unexpected_cfgs)]
    if cfg!(coverage) && hops > 1 {
        return Ok(());
    }

    // ── Role-typed cluster: Entry + N relays + Exit ─────────────────────────
    let cluster = build_pix_cluster(
        hops,
        IncomingSessionPixConfig {
            resume_window: IncomingSessionPixConfig::default().resume_window,
            quota_range: 0..=PIX_QUOTA_PER_SSA,
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
    let session = establish_pix_session_with_surbs(&cluster, hops, None, false, max_surbs_per_data_packet).await?;
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
    // Three cycles are plenty while the Entry's SURB supply follows the balancer's gate. With the cap
    // lifted they cannot tell a fixed Exit store from an unfixed one: the share-less backlog compounds,
    // because each cycle's shares queue behind everything the Entry minted while the previous cycle's
    // were being consumed, so the wait grows roughly (SURBs per packet - 1)-fold per cycle. Measured on
    // this cluster against the plain FIFO store that predates the share-first tiers, SSA #2 waited ~9 s
    // and SSA #3 57-60 s, just inside the 60 s `max_recovery_idle`, so three cycles still passed. SSA #4
    // received no useful share for the whole idle window and was closed as `RecoveryIdle` with
    // `largest_useful_shares=0`, the failure reported in hoprnet#8466. That fourth cycle is what makes
    // this case a reproduction.
    let target_cycles = if max_surbs_per_data_packet == usize::MAX {
        4u32
    } else {
        3u32
    };
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

    tracing::info!(hops, max_surbs_per_data_packet, "PIX multi-cycle session test PASSED");
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
            resume_window: IncomingSessionPixConfig::default().resume_window,
            quota_range: 0..=PIX_QUOTA_PER_SSA,
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
            resume_window: IncomingSessionPixConfig::default().resume_window,
            quota_range: 0..=PIX_QUOTA_PER_SSA,
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
            resume_window: IncomingSessionPixConfig::default().resume_window,
            quota_range: 0..=PIX_QUOTA_PER_SSA,
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
///   is refused as under-served by an Entry that does not credit them. The Exit re-sends the request while the cycle is
///   still being served, but an Entry that never credits them refuses every re-send, and once the cycle's emission is
///   spent the Session dies on `max_ssa_delivery_time` blaming a timer;
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
    while let Ok(milestone) = milestones.try_recv() {
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
            resume_window: IncomingSessionPixConfig::default().resume_window,
            quota_range: 0..=PIX_QUOTA_PER_SSA,
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
                max_surbs_per_data_packet: 1,
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
    // Through `PIX_PARAMS` rather than the three constants directly: this must be the quota the
    // node announces, and the node announces whatever its configured dimensions assemble into.
    let quota_per_ssa = PIX_PARAMS.polys_per_ssa() as u64
        * PIX_PARAMS.emitted_shares_per_poly() as u64
        * hopr_lib::exports::transport::session::PIX_QUOTA_BYTES_PER_SHARE;
    let accepted_batch_quota = quota_per_ssa * SSA_BATCH as u64;

    // The Exit's range admits the Entry's dimensions only as a batch of SSA_BATCH, and the Entry has
    // to accept that many or it refuses every derived request outright.
    let cluster = build_pix_cluster_with_entry_cap(
        hops,
        IncomingSessionPixConfig {
            resume_window: IncomingSessionPixConfig::default().resume_window,
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

/// A target raised mid-Session must widen the Exit's SURB ring, not just its balancer set-point.
///
/// This is the gnosis_vpn shape (gnosis/gnosis_vpn#739). The client opens at a ping-tier target and
/// announces its main tier a few seconds later, slewing in steps via
/// `HoprSessionConfigurator::update_surb_balancer_config`, each of which reaches the Exit as a
/// `BalancerTarget` keep-alive. The Exit sized its ring once, at establishment, off the ping tier:
/// with the target raised to the main tier it minted into a ring that could not grow, evicted every
/// SURB past the old ceiling on arrival — and, under PIX, the share each carried.
///
/// The assertion is on the ring's *ceiling*, which is the property that was wrong. Capacity is the
/// one figure the Session cannot correct after the fact: a SURB evicted here is gone for good.
#[rstest]
#[case(1)]
#[case(2)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
async fn a_raised_surb_target_widens_the_exits_ring(#[case] hops: usize) -> anyhow::Result<()> {
    use hopr_lib::exports::transport::{HoprSessionConfigurator, SurbBalancerConfig};

    let exit_pix = fill_pix_config(Default::default());
    let cluster = build_pix_cluster(hops, exit_pix, Duration::from_secs(120)).await?;
    // The Exit's first SSA request blocks on the deposit pool, and its keep-alive stream only runs
    // once a cycle is live; both must be served before the Session can be established.
    let (_driver, _milestones) = spawn_exit_pix_driver(&cluster);

    // The ping tier first, so the Session is admitted at the smaller ceiling.
    let ping_target = 64_u64;
    let main_target = 512_u64;
    assert!(
        main_target > ping_target,
        "the fixture's point is that the target rises"
    );

    let ping = SurbBalancerConfig {
        target_surb_buffer_size: ping_target,
        max_surbs_per_sec: 40,
        ..Default::default()
    };

    let ip = IpOrHost::from_str(":0")?;
    let routing = hops.try_into()?;
    let (session, configurator): (hopr_lib::HoprSession, HoprSessionConfigurator) = tokio::time::timeout(
        Duration::from_secs(120),
        cluster.entry.inner().connect_to(
            cluster.exit.address(),
            SessionTarget::UdpStream(SealedHost::Plain(ip)),
            HoprSessionClientConfig {
                forward_path: routing,
                return_path: routing,
                // Rate control on, which is the branch on which the Entry announces a target at
                // all — and the branch every gnosis client opens.
                capabilities: SessionCapability::Segmentation | SessionCapability::UsePIX,
                pseudonym: None,
                surb_management: Some(ping),
                max_surbs_per_data_packet: 1,
                flow_control: None,
                max_frames_behind_gap: None,
            },
        ),
    )
    .await
    .context("session connection timed out after 120s")??;

    let pseudonym = *session.id();
    let ring_ceiling = || {
        cluster
            .exit
            .inner()
            .transport()
            .surb_store()
            .admitted_ring_ceiling(&pseudonym)
    };

    let ping_ceiling = ring_ceiling().context("the Exit admitted no ring for the Session")?;
    assert!(
        ping_ceiling < main_target as usize,
        "fixture: the ping-tier ceiling ({ping_ceiling}) must be too narrow for the main target"
    );

    // Raise the target the way the client's ramp does. Several steps, so this also covers the
    // repeated announce rather than a single jump.
    for step in [128_u64, 256, 384, main_target] {
        configurator
            .update_surb_balancer_config(SurbBalancerConfig {
                target_surb_buffer_size: step,
                max_surbs_per_sec: 40,
                ..Default::default()
            })
            .context("raising the SURB target on a live Session must succeed")?;
    }

    // The raise rides a keep-alive, on the config's own notify cadence.
    let widened = tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            match ring_ceiling() {
                Some(ceiling) if ceiling >= main_target as usize => break Some(ceiling),
                _ => tokio::time::sleep(Duration::from_millis(100)).await,
            }
        }
    })
    .await
    .context("the Exit's ring was never widened")?;

    let widened = widened.context("the Session's ring disappeared")?;
    tracing::info!(
        ping_ceiling,
        widened,
        main_target,
        "the Exit's ring grew with the target"
    );
    assert!(
        widened >= main_target as usize,
        "the Exit's ring must reach the announced target: ceiling {widened}, target {main_target}"
    );

    Ok(())
}

/// A client reconnecting under the pseudonym it used before, after the Exit has forgotten the old
/// Session, is refused as not resumable, without retries; a fresh pseudonym is served.
///
/// The Entry still holds PIX state for the pseudonym and so asks to resume. An Exit that lost the
/// Session cannot, and answers `NotResumable`, which `connect_to` does not retry.
///
/// ## Steps
/// 1. A Session is opened and its first deposit, `(p, 1)`, is signalled on the Exit.
/// 2. The Exit closes the Session explicitly, with no resume window and no drain, so that it has forgotten it (an idle
///    eviction cannot be relied on: the Exit's keep-alive and fill keep the slot alive).
/// 3. A Session is opened under `p`: it fails with `Rejected(NotResumable)`. The Entry's retry delay is far beyond the
///    connect timeout, so a retry would show as a timeout rather than as a slow success.
/// 4. A Session under a fresh pseudonym `p'` is funded at `(p', 1)` and echoes.
#[cfg(feature = "session-client")]
#[rstest]
#[case(1)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
async fn a_reconnect_under_the_same_pseudonym_after_the_exit_closed_the_session_is_refused_as_not_resumable(
    #[case] hops: usize,
) -> anyhow::Result<()> {
    use hopr_lib::{
        errors::{HoprLibError, HoprTransportError},
        exports::transport::{StartErrorReason, TransportSessionError},
    };

    const IDLE_TIMEOUT: Duration = Duration::from_secs(120);

    // The Exit must forget the Session for certain, and not leave it to an idle eviction that its
    // keep-alive and fill keep from happening: no resume window and no drain, then an operator-style
    // close of the Session at the Exit.
    let mut exit_pix = fill_pix_config(hopr_lib::exports::transport::session::PixFillConfig {
        drain_after_close: false,
        ..Default::default()
    });
    exit_pix.resume_window = Duration::ZERO;
    let (exit_server, captured) = CaptureThenEchoServer::new();
    let cluster = build_pix_cluster_with(
        hops,
        exit_pix,
        IDLE_TIMEOUT,
        hopr_lib::exports::transport::config::PixGlobalConfig::default().max_ssas_per_request,
        exit_server,
        // Far beyond the 120 s connect timeout: a retry after the refusal cannot succeed in time.
        Some(Duration::from_secs(3600)),
    )
    .await?;
    let (_driver, mut milestones) = spawn_exit_pix_driver(&cluster);

    let funded = |seen: &[(PixMilestone, _)], pseudonym| {
        seen.iter().any(
            |(milestone, _)| matches!(milestone, PixMilestone::Funded(id) if id.session_id() == pseudonym && id.ssa_index().get() == 1),
        )
    };

    let first = establish_pix_session_under(&cluster, hops, None).await?;
    let pseudonym = *first.id();
    let seen = await_milestones(&mut milestones, Duration::from_secs(60), |seen| funded(seen, pseudonym)).await;
    anyhow::ensure!(
        funded(&seen, pseudonym),
        "the first Session's deposit was never signalled: {seen:?}"
    );

    // The Exit closes the Session, and with nothing to drain or wait for, forgets it.
    let mut exit_side = captured
        .lock()
        .map_err(|_| anyhow::anyhow!("the captured-session mutex was poisoned"))?
        .take()
        .context("the Exit's session server must have captured the incoming session")?;
    tokio::time::timeout(Duration::from_secs(30), exit_side.session.close())
        .await
        .context("closing the exit-side session timed out")?
        .context("closing the exit-side session failed")?;
    drop(first);
    tracing::info!(%pseudonym, "the Exit closed the Session; letting it wind down");
    tokio::time::sleep(Duration::from_secs(5)).await;
    // Whatever the first Session's wind-down reported, only what follows the reconnect counts.
    while milestones.try_recv().is_ok() {}

    // A retry would wait out the one-hour retry delay, and so fail as a timeout here.
    let refused = try_establish_pix_session_under_with(&cluster, hops, Some(pseudonym), None, false)
        .await
        .context("the refusal was retried instead of being final")?;
    match refused {
        Err(HoprLibError::TransportError(HoprTransportError::Session(TransportSessionError::Rejected(
            StartErrorReason::NotResumable,
        )))) => {}
        Err(other) => anyhow::bail!("expected Rejected(NotResumable), got {other:?}"),
        Ok(_session) => anyhow::bail!("the reconnect under {pseudonym} must be refused"),
    }

    let mut fresh = establish_pix_session_under(&cluster, hops, None).await?;
    let fresh_pseudonym = *fresh.id();
    anyhow::ensure!(fresh_pseudonym != pseudonym, "the new Session needs a fresh pseudonym");
    let seen = await_milestones(&mut milestones, Duration::from_secs(60), |seen| {
        funded(seen, fresh_pseudonym)
    })
    .await;
    anyhow::ensure!(
        funded(&seen, fresh_pseudonym),
        "the fresh Session got no (p', 1) deposit: {seen:?}"
    );

    const ECHOES: usize = 3;
    let completed = echo_n(&mut fresh, ECHOES, Duration::from_millis(200)).await;
    assert_eq!(
        ECHOES, completed,
        "the fresh Session must carry traffic, but only {completed} of {ECHOES} echo round trips finished"
    );
    Ok(())
}

/// An echo server that counts the Sessions it holds and the ones that have ended.
#[cfg(feature = "session-client")]
#[derive(Clone, Default)]
struct TrackingEchoServer {
    started: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    ended: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

#[cfg(feature = "session-client")]
#[async_trait::async_trait]
impl hopr_api::node::HoprSessionServer for TrackingEchoServer {
    type Error = hopr_lib::errors::HoprLibError;
    type Session = hopr_lib::exports::transport::IncomingSession;

    async fn process(&self, session: Self::Session) -> Result<(), Self::Error> {
        use std::sync::atomic::Ordering;
        self.started.fetch_add(1, Ordering::SeqCst);
        let ended = self.ended.clone();
        tokio::spawn(async move {
            let (mut reader, mut writer) = session.session.split();
            let mut buf = vec![0u8; 8192];
            loop {
                let n = match reader.read(&mut buf).await {
                    Ok(0) | Err(_) => break,
                    Ok(n) => n,
                };
                if writer.write_all(&buf[..n]).await.is_err() || writer.flush().await.is_err() {
                    break;
                }
            }
            ended.fetch_add(1, Ordering::SeqCst);
        });
        Ok(())
    }
}

#[cfg(feature = "session-client")]
impl TrackingEchoServer {
    /// Sessions the server holds right now.
    fn live(&self) -> usize {
        use std::sync::atomic::Ordering;
        self.started
            .load(Ordering::SeqCst)
            .saturating_sub(self.ended.load(Ordering::SeqCst))
    }
}

/// What the two nodes' PIX event streams said about one pseudonym, as they said it.
#[cfg(feature = "session-client")]
#[derive(Default, Debug)]
struct PixTally {
    /// Deposit-data requests the Exit made, by SSA index.
    deposit_data_requests: Vec<u32>,
    /// Deposit addresses the Exit was handed, by SSA index.
    exit_addresses: Vec<u32>,
    /// `NewDepositAddress` events on the Entry, by SSA index: the Entry's deposits.
    entry_deposits: Vec<u32>,
    /// Cycles the Exit recovered, by SSA index.
    recovered: Vec<u32>,
}

/// Answers the Exit's PIX events as [`spawn_exit_pix_driver`] does, and tallies the events of both
/// nodes for `pseudonym`.
#[cfg(feature = "session-client")]
fn spawn_pix_tally(
    cluster: &hopr_lib::testing::fixtures::RoleClusterGuard,
    pseudonym: hopr_lib::api::types::internal::protocol::HoprPseudonym,
) -> (tokio::task::JoinHandle<()>, std::sync::Arc<std::sync::Mutex<PixTally>>) {
    let tally = std::sync::Arc::new(std::sync::Mutex::new(PixTally::default()));
    let mut exit_events = Box::pin(cluster.exit.inner().subscribe_pix_events());
    let mut entry_events = Box::pin(cluster.entry.inner().subscribe_pix_events());
    let shared = tally.clone();
    let handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(event) = entry_events.next() => {
                    if let PixEvent::NewDepositAddress(data) = event
                        && data.id.session_id() == pseudonym
                    {
                        shared.lock().unwrap().entry_deposits.push(data.id.ssa_index().get());
                    }
                }
                Some(event) = exit_events.next() => match event {
                    PixEvent::DepositAddressReceived(data) => {
                        let mut notifier = data.deposit_updated;
                        let _ = notifier.send((data.id, HoprBalance::new_base(1))).await;
                        if data.id.session_id() == pseudonym {
                            shared.lock().unwrap().exit_addresses.push(data.id.ssa_index().get());
                        }
                    }
                    PixEvent::PrivateKeyRecovered(data) if data.id.session_id() == pseudonym => {
                        shared.lock().unwrap().recovered.push(data.id.ssa_index().get());
                    }
                    PixEvent::DepositDataRequest(request) => {
                        let mut created = request.deposit_data_created;
                        for id in request.deposit_ids {
                            if id.session_id() == pseudonym {
                                shared.lock().unwrap().deposit_data_requests.push(id.ssa_index().get());
                            }
                            if created.send(PixDepositData { id, data: Box::default() }).await.is_err() {
                                break;
                            }
                        }
                    }
                    _ => {}
                },
                else => break,
            }
        }
    });
    (handle, tally)
}

/// How a client leaves the Session it will reconnect from.
#[cfg(feature = "session-client")]
#[derive(Clone, Copy, Debug)]
enum Leave {
    /// A clean close, whose terminating segment reaches the Exit.
    Close,
    /// Dropped without a close: nothing tells the Exit, which still holds the live Session.
    Drop,
}

/// A client that reconnects in place under its paid pseudonym gets its funded Session back: no second
/// deposit for `(p, 1)`, one Session on the Exit, and the cycle goes on.
///
/// ## Steps
/// 1. A PIX Session is opened under `p`, rate-controlled and with a SURB balancer (the gnosis VPN shape).
/// 2. It echoes until the Exit has signalled the deposit for `(p, 1)`, and a few round trips after.
/// 3. It is closed cleanly (or dropped). A Session under `p` is opened again, which must complete in under 10 s.
/// 4. It echoes again.
/// 5. The Exit asked for no deposit data, handed out no address, and the Entry made no deposit, for `(p, 1)` a second
///    time; the Exit holds exactly one Session and the first has ended; and `(p, 1)` later recovers, after which the
///    next request is for SSA 2.
///
/// Compiled here, to be run on a cluster.
#[cfg(feature = "session-client")]
#[rstest]
#[case(1, Leave::Close)]
#[case(1, Leave::Drop)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
async fn a_reconnect_under_the_same_pseudonym_takes_over_the_paid_session(
    #[case] hops: usize,
    #[case] leave: Leave,
) -> anyhow::Result<()> {
    use std::sync::atomic::Ordering;

    const IDLE_TIMEOUT: Duration = Duration::from_secs(120);
    const REOPEN_BUDGET: Duration = Duration::from_secs(10);

    let mut exit_pix = fill_pix_config(Default::default());
    exit_pix.resume_window = Duration::from_secs(60);
    let server = TrackingEchoServer::default();
    let cluster = build_pix_cluster_with(
        hops,
        exit_pix,
        IDLE_TIMEOUT,
        hopr_lib::exports::transport::config::PixGlobalConfig::default().max_ssas_per_request,
        server.clone(),
        None,
    )
    .await?;

    // The pseudonym is chosen here, so the tally can follow it before the Session exists.
    use hopr_lib::api::types::crypto_random::Randomizable as _;
    let pseudonym = hopr_lib::api::types::internal::protocol::HoprPseudonym::random();
    let (driver, tally) = spawn_pix_tally(&cluster, pseudonym);

    let mut first =
        establish_pix_session_under_with(&cluster, hops, Some(pseudonym), Some(idle_surb_balancer()), true).await?;
    assert_eq!(pseudonym, *first.id());

    // Echo until the first cycle is funded.
    let funded = |tally: &std::sync::Mutex<PixTally>| tally.lock().unwrap().exit_addresses.contains(&1);
    let started = std::time::Instant::now();
    while !funded(&tally) && started.elapsed() < Duration::from_secs(60) {
        echo_n(&mut first, 1, Duration::from_millis(200)).await;
    }
    anyhow::ensure!(
        funded(&tally),
        "the first cycle was never funded: {:?}",
        tally.lock().unwrap()
    );
    anyhow::ensure!(
        echo_n(&mut first, 3, Duration::from_millis(200)).await == 3,
        "the first Session must carry traffic"
    );
    anyhow::ensure!(server.live() == 1, "the Exit holds one Session before the reconnect");
    // For SSA 1 only: fill or the echoes below may legitimately make the Exit ask for SSA 2 meanwhile.
    let before = {
        let t = tally.lock().unwrap();
        let ssa_1 = |indices: &[u32]| indices.iter().filter(|index| **index == 1).count();
        (
            ssa_1(&t.deposit_data_requests),
            ssa_1(&t.exit_addresses),
            ssa_1(&t.entry_deposits),
        )
    };

    match leave {
        Leave::Close => {
            // The Exit sees the end of the stream and detaches the Session; the takeover may even
            // overtake that FIN, and then takes the live Session over.
            use futures::AsyncWriteExt as _;
            first.close().await.context("closing the first Session")?;
        }
        Leave::Drop => {
            // Nothing tells the Exit, which still holds the Session live. The reconnect's challenge is
            // greater than the one the Session was established by, which is what lets it take the live
            // Session over (see `SessionClientConfig::pseudonym`).
            drop(first);
        }
    }

    let reopened_at = std::time::Instant::now();
    let mut second = tokio::time::timeout(
        REOPEN_BUDGET,
        establish_pix_session_under_with(&cluster, hops, Some(pseudonym), Some(idle_surb_balancer()), true),
    )
    .await
    .context("the reconnect under the paid pseudonym must complete in under 10 s")??;
    tracing::info!(took = ?reopened_at.elapsed(), "reconnected");
    assert_eq!(pseudonym, *second.id());

    let completed = echo_n(&mut second, 5, Duration::from_millis(200)).await;
    assert_eq!(5, completed, "the taken-over Session must carry traffic");

    // Nothing was paid for twice.
    {
        let t = tally.lock().unwrap();
        let ssa_1 = |indices: &[u32]| indices.iter().filter(|index| **index == 1).count();
        assert_eq!(
            before.0,
            ssa_1(&t.deposit_data_requests),
            "no further deposit-data request for SSA 1: {t:?}"
        );
        assert_eq!(
            before.1,
            ssa_1(&t.exit_addresses),
            "no further deposit address for SSA 1: {t:?}"
        );
        assert_eq!(
            before.2,
            ssa_1(&t.entry_deposits),
            "no further Entry deposit for SSA 1: {t:?}"
        );
    }

    // One Session on the Exit, and the first one ended.
    tokio::time::timeout(Duration::from_secs(10), async {
        while server.started.load(Ordering::SeqCst) < 2 || server.ended.load(Ordering::SeqCst) < 1 {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .context("the Exit must have handed over a second Session and ended the first")?;
    assert_eq!(1, server.live(), "the Exit holds exactly one Session");

    // The paid cycle is recovered on the taken-over Session, and the next request is for SSA 2.
    let started = std::time::Instant::now();
    loop {
        {
            let t = tally.lock().unwrap();
            if t.recovered.contains(&1) && t.exit_addresses.contains(&2) {
                break;
            }
        }
        anyhow::ensure!(
            started.elapsed() < Duration::from_secs(120),
            "(p, 1) never recovered and led to SSA 2: {:?}",
            tally.lock().unwrap()
        );
        echo_n(&mut second, 1, Duration::from_millis(100)).await;
    }
    {
        let t = tally.lock().unwrap();
        assert_eq!(1, t.exit_addresses.iter().filter(|index| **index == 1).count(), "{t:?}");
        assert_eq!(1, t.entry_deposits.iter().filter(|index| **index == 1).count(), "{t:?}");
    }
    driver.abort();
    Ok(())
}

/// A client that closes cleanly and never comes back: the detached Session ends at the window as it
/// would have without one, and no successor cycle is funded in between.
///
/// ## Steps
/// 1. A PIX Session is opened under `p` and echoes until the Exit has signalled the deposit for `(p, 1)`.
/// 2. It is closed cleanly. The Exit detaches it, for a short window (3 s).
/// 3. Nobody returns. Past the window the Exit has closed the Session, or drained it into the funded cycle if its SURB
///    stock covered the rest.
/// 4. The Exit asked for no deposit data and was handed no deposit address beyond `(p, 1)` at any point: no successor
///    cycle was funded during the window, which the drain or close would have stranded.
///
/// Compiled here, to be run on a cluster.
#[cfg(feature = "session-client")]
#[rstest]
#[case(1)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
async fn a_closed_session_that_never_returns_ends_at_the_window_and_funds_no_successor(
    #[case] hops: usize,
) -> anyhow::Result<()> {
    use futures::AsyncWriteExt as _;
    use hopr_lib::{
        api::types::crypto_random::Randomizable as _,
        errors::{HoprLibError, HoprTransportError},
        exports::transport::{StartErrorReason, TransportSessionError},
    };

    const WINDOW: Duration = Duration::from_secs(3);

    let mut exit_pix = fill_pix_config(Default::default());
    exit_pix.resume_window = WINDOW;
    let cluster = build_pix_cluster_with(
        hops,
        exit_pix,
        Duration::from_secs(120),
        hopr_lib::exports::transport::config::PixGlobalConfig::default().max_ssas_per_request,
        TrackingEchoServer::default(),
        None,
    )
    .await?;
    let pseudonym = hopr_lib::api::types::internal::protocol::HoprPseudonym::random();
    let (driver, tally) = spawn_pix_tally(&cluster, pseudonym);

    let mut session =
        establish_pix_session_under_with(&cluster, hops, Some(pseudonym), Some(idle_surb_balancer()), true).await?;
    let started = std::time::Instant::now();
    while !tally.lock().unwrap().exit_addresses.contains(&1) && started.elapsed() < Duration::from_secs(60) {
        echo_n(&mut session, 1, Duration::from_millis(200)).await;
    }
    anyhow::ensure!(
        tally.lock().unwrap().exit_addresses.contains(&1),
        "the first cycle was never funded: {:?}",
        tally.lock().unwrap()
    );
    let before = {
        let t = tally.lock().unwrap();
        (
            t.deposit_data_requests.len(),
            t.exit_addresses.len(),
            t.entry_deposits.len(),
        )
    };

    session.close().await.context("closing the Session")?;

    // The window, and the drain or close it ends in, with the same budget as a close with no window.
    tokio::time::sleep(WINDOW + Duration::from_secs(30)).await;
    {
        let t = tally.lock().unwrap();
        assert_eq!(
            before.0,
            t.deposit_data_requests.len(),
            "no successor deposit-data request: {t:?}"
        );
        assert_eq!(before.1, t.exit_addresses.len(), "no successor deposit address: {t:?}");
        assert_eq!(before.2, t.entry_deposits.len(), "no successor Entry deposit: {t:?}");
    }

    // The Session was ended, not kept with its successor requests suppressed: a reconnect under the retained
    // pseudonym is refused as not resumable.
    let refused =
        try_establish_pix_session_under_with(&cluster, hops, Some(pseudonym), Some(idle_surb_balancer()), true).await?;
    match refused {
        Err(HoprLibError::TransportError(HoprTransportError::Session(TransportSessionError::Rejected(
            StartErrorReason::NotResumable,
        )))) => {}
        Err(other) => anyhow::bail!("expected Rejected(NotResumable), got {other:?}"),
        Ok(_session) => anyhow::bail!("the reconnect under {pseudonym} after the window must be refused"),
    }
    driver.abort();
    Ok(())
}

/// Captures the first incoming Session, so that a test can close it on the Exit, and echoes every later one.
#[cfg(feature = "session-client")]
#[derive(Clone)]
struct CaptureThenEchoServer {
    captured: std::sync::Arc<std::sync::Mutex<Option<hopr_lib::exports::transport::IncomingSession>>>,
    seen: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

#[cfg(feature = "session-client")]
impl CaptureThenEchoServer {
    #[allow(clippy::type_complexity)]
    fn new() -> (
        Self,
        std::sync::Arc<std::sync::Mutex<Option<hopr_lib::exports::transport::IncomingSession>>>,
    ) {
        let captured = std::sync::Arc::new(std::sync::Mutex::new(None));
        (
            Self {
                captured: captured.clone(),
                seen: Default::default(),
            },
            captured,
        )
    }
}

#[cfg(feature = "session-client")]
#[async_trait::async_trait]
impl hopr_api::node::HoprSessionServer for CaptureThenEchoServer {
    type Error = hopr_lib::errors::HoprLibError;
    type Session = hopr_lib::exports::transport::IncomingSession;

    async fn process(&self, session: Self::Session) -> Result<(), Self::Error> {
        if !self.seen.swap(true, std::sync::atomic::Ordering::SeqCst) {
            *self.captured.lock().unwrap() = Some(session);
            return Ok(());
        }
        tokio::spawn(async move {
            let (mut reader, mut writer) = session.session.split();
            let mut buf = vec![0u8; 8192];
            while let Ok(n @ 1..) = reader.read(&mut buf).await {
                if writer.write_all(&buf[..n]).await.is_err() || writer.flush().await.is_err() {
                    break;
                }
            }
        });
        Ok(())
    }
}

/// Whether the Exit signalled `milestone`-kind progress for `(pseudonym, index)`: the deposit (`recovered` false) or
/// the recovery of the cycle's key (`recovered` true).
#[cfg(feature = "session-client")]
fn cycle_signalled(
    seen: &[(PixMilestone, std::time::Instant)],
    recovered: bool,
    pseudonym: hopr_lib::api::types::internal::protocol::HoprPseudonym,
    index: u32,
) -> bool {
    seen.iter().any(|(milestone, _)| match milestone {
        PixMilestone::Funded(id) if !recovered => id.session_id() == pseudonym && id.ssa_index().get() == index,
        PixMilestone::Recovered(id) if recovered => id.session_id() == pseudonym && id.ssa_index().get() == index,
        _ => false,
    })
}

/// After the Exit has closed a Session, a reconnect under its pseudonym is refused, and a Session under a fresh
/// pseudonym runs a clean cycle that recovers, however long the Entry waited. The Entry used to reuse `(p, 1)` and
/// its late replies carried shares of the closed Session's cycle, which failed as `unverifiable_shares`.
///
/// ## Steps
/// 1. A Session is opened, echoes a few times and is closed by the client; with no resume window and no drain the Exit
///    closes its side.
/// 2. After the gap (shorter and longer than the 7 s SURB-retire grace) a Session under `p` is refused with
///    `Rejected(NotResumable)`.
/// 3. A Session under a fresh pseudonym `p'` is funded at `(p', 1)`, and echo traffic drives it until `(p', 1)`
///    recovers.
#[cfg(feature = "session-client")]
#[rstest]
#[case(1, 2)]
#[case(1, 12)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
async fn a_fresh_pseudonym_after_a_closed_session_recovers_its_first_cycle(
    #[case] hops: usize,
    #[case] gap_secs: u64,
) -> anyhow::Result<()> {
    use hopr_lib::{
        errors::{HoprLibError, HoprTransportError},
        exports::transport::{StartErrorReason, TransportSessionError},
    };

    const IDLE_TIMEOUT: Duration = Duration::from_secs(120);
    const FIRST_LEG_ECHOES: usize = 4;

    // No resume window and no drain: a clean client close ends the Exit's Session at once.
    let mut exit_pix = fill_pix_config(hopr_lib::exports::transport::session::PixFillConfig {
        drain_after_close: false,
        ..Default::default()
    });
    exit_pix.resume_window = Duration::ZERO;
    let cluster =
        build_pix_cluster_with_exit_server(hops, exit_pix, IDLE_TIMEOUT, TrackingEchoServer::default()).await?;
    let (_driver, mut milestones) = spawn_exit_pix_driver(&cluster);

    let mut first = establish_pix_session_under(&cluster, hops, None).await?;
    let pseudonym = *first.id();
    let seen = await_milestones(&mut milestones, Duration::from_secs(60), |seen| {
        cycle_signalled(seen, false, pseudonym, 1)
    })
    .await;
    anyhow::ensure!(
        cycle_signalled(&seen, false, pseudonym, 1),
        "the first (p, 1) was never funded: {seen:?}"
    );
    let completed = echo_n(&mut first, FIRST_LEG_ECHOES, Duration::from_millis(300)).await;
    anyhow::ensure!(
        completed == FIRST_LEG_ECHOES,
        "only {completed} echoes of the first Session finished"
    );
    first.close().await.context("closing the first Session")?;
    drop(first);
    tokio::time::sleep(Duration::from_secs(gap_secs)).await;

    match try_establish_pix_session_under_with(&cluster, hops, Some(pseudonym), None, false).await? {
        Err(HoprLibError::TransportError(HoprTransportError::Session(TransportSessionError::Rejected(
            StartErrorReason::NotResumable,
        )))) => {}
        Err(other) => anyhow::bail!("expected Rejected(NotResumable), got {other:?}"),
        Ok(_session) => anyhow::bail!("the reconnect under {pseudonym} must be refused"),
    }

    // Only what follows counts.
    while milestones.try_recv().is_ok() {}
    let second = establish_pix_session_under(&cluster, hops, None).await?;
    let fresh = *second.id();
    anyhow::ensure!(fresh != pseudonym, "the new Session needs a fresh pseudonym");
    let seen = await_milestones(&mut milestones, Duration::from_secs(60), |seen| {
        cycle_signalled(seen, false, fresh, 1)
    })
    .await;
    anyhow::ensure!(
        cycle_signalled(&seen, false, fresh, 1),
        "the fresh Session got no (p', 1) deposit: {seen:?}"
    );

    let stopped = EchoStopCell::default();
    let echo = spawn_echo_task(second, stopped.clone(), Duration::from_secs(20));
    let seen = await_milestones(&mut milestones, Duration::from_secs(90), |seen| {
        cycle_signalled(seen, true, fresh, 1)
    })
    .await;
    let stop = stopped.get().map(|(stop, _)| stop);
    echo.abort();
    anyhow::ensure!(
        cycle_signalled(&seen, true, fresh, 1),
        "the fresh (p', 1) never recovered (echo stop: {stop:?}): {seen:?}"
    );
    Ok(())
}
