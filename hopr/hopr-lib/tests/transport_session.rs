use hopr_lib::testing::fixtures::{
    MINIMUM_INCOMING_WIN_PROB, TEST_GLOBAL_TIMEOUT, TestNodeConfig, chain_propagation_delay, cluster_fixture,
};
#[cfg(feature = "session-client")]
use {
    futures::future::try_join_all, hopr_chain_connector::blokli_client::BlokliQueryClient,
    hopr_lib::api::types::primitive::prelude::HoprBalance, hopr_lib::testing::hopr::ChannelGuard,
    rand::seq::SliceRandom, rstest::*, serial_test::serial,
};

const FUNDING_AMOUNT: &str = "500 wxHOPR";

#[rstest]
#[case(0)]
#[case(1)]
#[case(2)]
#[case(3)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[cfg(feature = "session-client")]
/// Tests n-hop session establishment over a fully connected channel network.
///
/// Creates a cluster sized exactly to `hops + 2` nodes (0-hop: 2, 1-hop: 3, etc.)
/// and opens bidirectional channels between every pair, so the path planner has
/// the full graph available.
///
/// - 0-hop: 2 nodes, no channels needed (direct connection)
/// - n-hop (n >= 1): n+2 nodes with n*(n+1) bidirectional channels
async fn create_n_hop_session(#[case] hops: usize) -> anyhow::Result<()> {
    // 2-hop and 3-hop tests are too slow under coverage instrumentation
    #[allow(unexpected_cfgs)]
    if cfg!(coverage) && hops > 1 {
        return Ok(());
    }

    let node_count = if hops == 0 { 2 } else { hops + 2 };
    let cluster = cluster_fixture(vec![
        TestNodeConfig::with_probability(MINIMUM_INCOMING_WIN_PROB);
        node_count
    ]);
    let mut nodes: Vec<&_> = cluster.iter().collect();
    nodes.shuffle(&mut rand::rng());

    let src = nodes[0];
    let dst = nodes[nodes.len() - 1];
    let mid = &nodes[1..nodes.len() - 1];

    tracing::info!(hops, src = %src.address(), dst = %dst.address(), "session test node mapping");

    // For n-hop (n >= 1), open channels between ALL pairs to create a fully connected network,
    // so the path planner can leverage the full graph data.
    let all_channels = if hops > 0 {
        let funding = FUNDING_AMOUNT.parse::<HoprBalance>()?;
        let mut channels = Vec::new();
        for i in 0..nodes.len() {
            for j in 0..nodes.len() {
                if i != j {
                    channels.push(
                        ChannelGuard::open_channel_between_nodes(
                            nodes[i].instance.clone(),
                            nodes[j].instance.clone(),
                            funding,
                        )
                        .await?,
                    );
                }
            }
        }
        channels
    } else {
        Vec::new()
    };

    // Wait for channel state to propagate across all nodes by polling the graph
    // instead of using a fixed sleep. Each node must see all N*(N-1) open channels.
    if !all_channels.is_empty() {
        let chain_info = cluster.chain_client.query_chain_info().await?;
        let timeout = chain_propagation_delay(&chain_info) * 12;
        cluster.wait_for_channel_graph(src, all_channels.len(), timeout).await?;
    }

    let path: Vec<&_> = std::iter::once(src)
        .chain(mid.iter().copied())
        .chain(std::iter::once(dst))
        .collect();

    // Disable the SURB balancer so its background replenishment traffic does not
    // exhaust the 100 wxHOPR channels (10 tickets at win_prob=0.2) before the
    // handshake completes. This test verifies session establishment, not SURB management.
    let _session = cluster.create_session_with(&path, Default::default(), None).await?;

    // TODO: check here that the destination sees the new session created

    try_join_all(
        all_channels
            .into_iter()
            .map(move |guard| async move { guard.try_close_channels_all_channels().await }),
    )
    .await?;

    Ok(())
}

// =========================================================================
//  Bidirectional data flow through sessions
// =========================================================================

use std::{str::FromStr, time::Duration};

use anyhow::Context;
use futures::{AsyncReadExt, AsyncWriteExt};
use hopr_lib::{
    api::node::HoprSessionClientOperations,
    exports::{
        network::types::prelude::{IpOrHost, SealedHost},
        transport::{SessionCapability, session::counters::routing_resolution_surb_timeout_count},
    },
};

/// Run bidirectional echo on a session, verifying data round-trips via EchoServer.
async fn verify_session_echo(
    entry_session: &mut hopr_lib::exports::transport::HoprSession,
    label: &str,
) -> anyhow::Result<()> {
    let msg: [u8; 32] = hopr_lib::api::types::crypto_random::random_bytes();

    let mut echoed = vec![0u8; 32];
    tokio::time::timeout(Duration::from_secs(10), async {
        entry_session.write_all(&msg).await?;
        entry_session.flush().await?;
        entry_session.read_exact(&mut echoed).await?;
        anyhow::Ok(())
    })
    .await
    .with_context(|| format!("{label}: echo round-trip timeout"))??;
    assert_eq!(&msg[..], &echoed[..], "{label}: echo mismatch");

    tracing::info!("{label}: echo verified");
    Ok(())
}

#[rstest]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[cfg(feature = "session-client")]
/// 0-hop bidirectional data flow through a HOPR session.
async fn capture_direct_session() -> anyhow::Result<()> {
    let cluster = cluster_fixture(vec![TestNodeConfig::default(); 2]);
    assert_eq!(cluster.size(), 2);

    let src = &cluster[0];
    let dst = &cluster[1];

    let ip = IpOrHost::from_str(":0")?;
    let (mut session, _) = src
        .inner()
        .connect_to(
            dst.address(),
            hopr_lib::exports::transport::SessionTarget::UdpStream(SealedHost::Plain(ip)),
            hopr_lib::HoprSessionClientConfig {
                forward_path: 0.try_into()?,
                return_path: 0.try_into()?,
                capabilities: SessionCapability::Segmentation | SessionCapability::NoRateControl,
                pseudonym: None,
                surb_management: None,
                max_surbs_per_data_packet: usize::MAX,
                flow_control: None,
                max_frames_behind_gap: None,
            },
        )
        .await?;
    verify_session_echo(&mut session, "0-hop").await?;
    Ok(())
}

#[rstest]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[cfg(feature = "session-client")]
/// 1-hop bidirectional data flow through a HOPR session via a relay.
///
/// SURB balancing is not used — with `max_surbs_per_data_packet: usize::MAX`, each small
/// data packet (32 bytes) carries enough SURBs alongside the payload for a
/// symmetric 1:1 echo pattern (1 SURB delivered → 1 reply).
async fn capture_one_hop_session() -> anyhow::Result<()> {
    let cluster = cluster_fixture(vec![
        TestNodeConfig::default(),                                   // src:  win_prob=1.0
        TestNodeConfig::with_probability(MINIMUM_INCOMING_WIN_PROB), // relay: win_prob=0.2
        TestNodeConfig::default(),                                   // dst:  win_prob=1.0
    ]);
    let src = &cluster[0];
    let relay = &cluster[1];
    let dst = &cluster[2];

    // Open path channels in both directions for forward and SURB return routing
    let funding = FUNDING_AMOUNT.parse::<HoprBalance>()?;
    let mut channels = Vec::new();
    for (from, to) in [(src, relay), (relay, dst), (dst, relay), (relay, src)] {
        channels
            .push(ChannelGuard::open_channel_between_nodes(from.instance.clone(), to.instance.clone(), funding).await?);
    }

    let chain_info = cluster.chain_client.query_chain_info().await?;
    cluster
        .wait_for_channel_graph(src, channels.len(), chain_propagation_delay(&chain_info) * 6)
        .await?;

    let ip = IpOrHost::from_str(":0")?;
    let (mut session, _) = src
        .inner()
        .connect_to(
            dst.address(),
            hopr_lib::exports::transport::SessionTarget::UdpStream(SealedHost::Plain(ip)),
            hopr_lib::HoprSessionClientConfig {
                forward_path: 1.try_into()?,
                return_path: 1.try_into()?,
                capabilities: SessionCapability::Segmentation | SessionCapability::NoRateControl,
                pseudonym: None,
                surb_management: None,
                max_surbs_per_data_packet: usize::MAX,
                flow_control: None,
                max_frames_behind_gap: None,
            },
        )
        .await?;
    verify_session_echo(&mut session, "1-hop").await?;

    try_join_all(
        channels
            .into_iter()
            .map(|guard| async move { guard.try_close_channels_all_channels().await }),
    )
    .await?;

    Ok(())
}

// =========================================================================
//  The Exit's SURB-level reports on a `NoRateControl` Session
// =========================================================================
//
// Since #8396 the Exit runs a SURB-level keep-alive stream on a `NoRateControl` Session, one packet
// every `surb_balance_notify_period` (15 s by default). Every such packet is return-routed, so it
// spends one SURB from the Exit's store for the Session. When the Entry on that branch runs no SURB
// balancer, nothing replenishes that buffer beyond what the Entry's own outgoing packets carry; it
// drains, and once it is empty each further keep-alive waits out `surb_resolution_wait` and is
// dropped with "no SURB for its return path within the wait". A report already queued when a client
// request arrives takes the SURB that request needed, which is the client-visible stall
// gnosis_vpn#748 reports.
//
// The fix withholds the notification from a Session that announced no SURB target, which on this
// branch means one whose Entry runs no balancer: the report exists to correct that balancer, and
// there is none. A `NoRateControl` Entry that *does* enable `surb_management` announces a target and
// keeps its reports. These tests pin the cluster-visible half of that — that neither shape drops
// packets; that the reports are still delivered to a balanced Session is pinned by the unit test
// `a_no_rate_control_session_that_announced_a_target_keeps_its_level_reports` in `hopr-transport-session`.
//
// The period is compressed so the drain would complete inside a test against the unfixed code.

/// The Exit's SURB-level notification period. The shipped default is 15 s; compressed here so that
/// an unfixed Exit would drain the buffer inside a test. The floor the manager enforces is 1 s.
const NOTIFY_PERIOD: Duration = Duration::from_secs(1);

/// How long a Session is driven after establishment — many times the period, so a drain that still
/// happened would be unmistakable.
const OBSERVE: Duration = Duration::from_secs(25);

/// Pacing of the Entry's own traffic, which is what refreshes the Exit's idle timer.
///
/// Deliberately four times the notification period: each ping also hands the Exit one SURB, so pinging
/// faster than the notification spends them would hide a drain behind the traffic.
const PING: Duration = Duration::from_secs(4);

/// The Exit's routing wait for a missing return SURB.
///
/// hoprd configures 1 s for its 3 s frame timeout. With the library default of 6 s each drop would
/// spend six periods waiting for a SURB that is not coming, and far fewer would complete inside the
/// observation window.
const SURB_WAIT: Duration = Duration::from_secs(1);

/// A two-node cluster with a long idle timeout on both nodes and a short routing wait.
///
/// `notify_period` is the Exit's `surb_balance_notify_period`; the fixture otherwise pins it to
/// `None`, which would disable the very stream under test.
///
/// Both nodes raise their idle timeout, and the Entry's matters as much as the Exit's: a Session is
/// evicted once *either* side sees no traffic for its own timeout, and this fixture's default is
/// 2.5 s — shorter than one [`PING`] interval, so the Entry would evict a Session whose own traffic
/// had not yet arrived. That produced a silent stall rather than a test failure until the pings were
/// made to assert, which is why the fixture states the constraint here rather than relying on the
/// default.
fn surb_report_cluster(notify_period: Option<Duration>) -> hopr_lib::testing::fixtures::ClusterGuard {
    let idle_timeout_ms = Duration::from_secs(60).as_millis() as u64;
    cluster_fixture(vec![
        TestNodeConfig {
            win_prob: 1.0,
            idle_timeout_ms,
            ..Default::default()
        },
        TestNodeConfig {
            win_prob: 1.0,
            idle_timeout_ms,
            pipeline: Some(hopr_lib::exports::transport::protocol::PacketPipelineConfig {
                surb_resolution_wait: Some(SURB_WAIT),
                ..Default::default()
            }),
            surb_balance_notify_period: notify_period,
            ..Default::default()
        },
    ])
}

/// The Session shape gnosis_vpn opens for its bridge and probe sessions: `NoRateControl`, with SURB
/// balancing under the caller's control.
fn nrc_session_config(surb_target: Option<u64>) -> anyhow::Result<hopr_lib::HoprSessionClientConfig> {
    Ok(hopr_lib::HoprSessionClientConfig {
        forward_path: hopr_lib::HopRouting::try_from(0usize)?,
        return_path: hopr_lib::HopRouting::try_from(0usize)?,
        capabilities: SessionCapability::Segmentation | SessionCapability::NoRateControl,
        pseudonym: None,
        surb_management: surb_target.map(|target_surb_buffer_size| hopr_lib::SurbBalancerConfig {
            target_surb_buffer_size,
            ..Default::default()
        }),
        // One SURB per packet, which is what the client's balancer asks for on this branch.
        max_surbs_per_data_packet: 1,
        flow_control: None,
        max_frames_behind_gap: None,
    })
}

/// Drives the Session with a ping every [`PING`] for `total`, returning the SURB-timeout drops the
/// node made on the way.
///
/// The pings keep the Session from being evicted and each hands the Exit one SURB for its reply, so a
/// Session that still starves is spending SURBs faster than one ping at a time can supply them.
///
/// A ping that fails is counted rather than ignored. The drops are measured as a delta of a
/// process-global counter, so a Session that died halfway through would otherwise read as "zero drops"
/// — the same result the fix is meant to produce, indistinguishable from it. The last ping is allowed
/// to fail because the Session may be torn down as the window closes; more than one means the traffic
/// driving the test stopped, and the drop count no longer measures anything.
///
/// The counter is process-global and every node in the cluster shares it, so the delta covers all of
/// them. That is sound here because the Exit is the only node that originates *return-routed* packets
/// in these tests — the Entry's own traffic is forward-routed — and each test builds its own cluster.
async fn drive_session(session: &mut hopr_lib::HoprSession, total: Duration) -> anyhow::Result<usize> {
    let started = tokio::time::Instant::now();
    let before = routing_resolution_surb_timeout_count();
    let mut buf = [0u8; 4];
    let mut failed_pings = 0usize;
    let mut pings = 0usize;
    while started.elapsed() < total {
        let ping = tokio::time::timeout(PING * 3, async {
            session.write_all(b"ping").await?;
            session.flush().await?;
            session.read_exact(&mut buf).await
        })
        .await;
        pings += 1;
        if !matches!(ping, Ok(Ok(()))) {
            failed_pings += 1;
        }
        tokio::time::sleep(PING).await;
    }

    anyhow::ensure!(
        failed_pings <= 1,
        "{failed_pings} of {pings} pings failed, so the Session stopped carrying traffic and its drop count measures \
         nothing"
    );
    Ok(routing_resolution_surb_timeout_count() - before)
}

/// The bug: a `NoRateControl` Session whose Entry runs no balancer must not drop packets for want of a
/// SURB.
///
/// Before the fix this dropped a packet every period — measured at 19–20 drops in this window, at
/// exactly the period's cadence, with the Exit's own `announced_surb_target=… effective=…` derivation
/// confirming it was the notification stream doing it.
#[rstest]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[cfg(feature = "session-client")]
async fn a_no_rate_control_entry_without_a_balancer_triggers_no_surb_drops() -> anyhow::Result<()> {
    let cluster = surb_report_cluster(Some(NOTIFY_PERIOD));
    let ip = IpOrHost::from_str(":0")?;
    let (mut session, _cfg) = cluster[0]
        .inner()
        .connect_to(
            cluster[1].address(),
            hopr_lib::exports::transport::SessionTarget::UdpStream(SealedHost::Plain(ip)),
            nrc_session_config(None)?,
        )
        .await
        .context("the NoRateControl session must establish")?;

    let drops = drive_session(&mut session, OBSERVE).await?;
    tracing::info!(drops, "SURB timeout drops observed");
    assert_eq!(
        0, drops,
        "an Exit Session with no balancer on either side has a level report to deliver to no one, so every one of \
         them is a SURB spent for nothing; {drops} were dropped where none should have been originated"
    );
    Ok(())
}

/// An idle Session is the starkest case: no traffic at all, so any drop is attributable to the Exit
/// alone rather than competing with the client's own requests for the last SURB.
#[rstest]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[cfg(feature = "session-client")]
async fn an_idle_no_rate_control_session_originates_nothing_it_cannot_pay_for() -> anyhow::Result<()> {
    let cluster = surb_report_cluster(Some(NOTIFY_PERIOD));
    let ip = IpOrHost::from_str(":0")?;
    let (_session, _cfg) = cluster[0]
        .inner()
        .connect_to(
            cluster[1].address(),
            hopr_lib::exports::transport::SessionTarget::UdpStream(SealedHost::Plain(ip)),
            nrc_session_config(None)?,
        )
        .await
        .context("the NoRateControl session must establish")?;

    let before = routing_resolution_surb_timeout_count();
    tokio::time::sleep(OBSERVE).await;
    let drops = routing_resolution_surb_timeout_count() - before;

    tracing::info!(drops, "drops from an idle session");
    assert_eq!(0, drops, "an idle Session must not originate reports it cannot pay for");
    Ok(())
}

/// A `NoRateControl` Entry that *does* balance must not starve either.
///
/// The Exit keeps its configured period on this Session, so the reports are still sent; the Entry's
/// balancer answers each with a SURB, which is why nothing is dropped. That the reports are actually
/// delivered is asserted at unit level instead
/// (`a_no_rate_control_session_that_announced_a_target_keeps_its_level_reports`) — a cluster test
/// cannot see them, since a zero drop count holds whether or not any report arrived.
#[rstest]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[cfg(feature = "session-client")]
async fn a_balanced_entry_is_not_starved_by_the_exits_reports() -> anyhow::Result<()> {
    let cluster = surb_report_cluster(Some(NOTIFY_PERIOD));
    let ip = IpOrHost::from_str(":0")?;
    let (mut session, _cfg) = cluster[0]
        .inner()
        .connect_to(
            cluster[1].address(),
            hopr_lib::exports::transport::SessionTarget::UdpStream(SealedHost::Plain(ip)),
            // A small target: the production default of 7000 is indistinguishable from "never starves"
            // against this cluster's buffers.
            nrc_session_config(Some(64))?,
        )
        .await
        .context("the NoRateControl session must establish")?;

    let drops = drive_session(&mut session, OBSERVE).await?;
    tracing::info!(drops, "drops with Entry-side SURB management");
    assert_eq!(
        0, drops,
        "an Entry-side balancer is what the Exit's notification exists to inform, and it must keep the Exit supplied"
    );
    Ok(())
}

/// The notification must still be produced where it is consumed: a Session whose balancer is enabled
/// keeps receiving its report at the configured period.
///
/// That is asserted by the unit test `a_no_rate_control_session_that_announced_a_target_keeps_its_level_reports`,
/// which can observe the originated reports directly. A cluster test cannot: the Entry exposes no
/// counter for reports received, so the best it can do is a zero drop count, which holds whether or
/// not a single report arrived.
#[rstest]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[cfg(feature = "session-client")]
async fn a_balanced_session_keeps_its_period_and_affords_the_reports() -> anyhow::Result<()> {
    let cluster = surb_report_cluster(Some(NOTIFY_PERIOD));
    let ip = IpOrHost::from_str(":0")?;
    let (mut session, _cfg) = cluster[0]
        .inner()
        .connect_to(
            cluster[1].address(),
            hopr_lib::exports::transport::SessionTarget::UdpStream(SealedHost::Plain(ip)),
            nrc_session_config(Some(64))?,
        )
        .await
        .context("the NoRateControl session must establish")?;

    let drops = drive_session(&mut session, OBSERVE).await?;
    assert_eq!(
        0, drops,
        "a balanced Entry answers each report with a SURB, so the Exit's stream must stay affordable"
    );

    // Window sanity check only: no report is observed here, so this just asserts the window is long
    // enough for the notification period to matter.
    let reports_in_window = (OBSERVE.as_secs_f64() / NOTIFY_PERIOD.as_secs_f64()).ceil() as usize;
    assert!(
        reports_in_window > 5,
        "{OBSERVE:?} is too short to exercise a {NOTIFY_PERIOD:?} period ({reports_in_window} reports)"
    );
    Ok(())
}
