//! Regression tests for gnosis_vpn#748.
//!
//! Since #8396 the Exit runs a SURB-level keep-alive stream on a `NoRateControl` Session, one packet
//! every `surb_balance_notify_period` (15 s by default). Every such packet is return-routed, so it
//! spends one SURB from the Exit's store for the Session. The `NoRateControl` branch runs no SURB
//! balancer on either side, so nothing replenishes that buffer beyond what the Entry's own outgoing
//! packets carry; it drains, and once it is empty each further keep-alive waits out
//! `surb_resolution_wait` and is dropped with "no SURB for its return path within the wait". A report
//! already queued when a client request arrives takes the SURB that request needed, which is the
//! client-visible stall the issue reports.
//!
//! The fix withholds the notification from a Session whose SURB balancer is disabled, which on this
//! branch is every one of them: the report exists to correct the Entry's balancer, and there is none.
//! These tests pin both halves — that such a Session no longer starves, and that a Session whose
//! balancer *is* enabled still reports, since that is the case the notification exists for.
//!
//! The period is compressed so the drain would complete inside a test against the unfixed code, and
//! the Exit's idle timeout is raised the way the PIX tests do so the Session outlives it.

use std::{str::FromStr, time::Duration};

use anyhow::Context;
use futures::{AsyncReadExt, AsyncWriteExt};
use hopr_lib::{
    HopRouting, HoprSessionClientConfig,
    api::node::HoprSessionClientOperations,
    exports::{
        network::types::prelude::{IpOrHost, SealedHost},
        transport::{SessionCapability, SessionTarget, session::counters::routing_resolution_surb_timeout_count},
    },
    testing::fixtures::{TestNodeConfig, cluster_fixture},
};
use serial_test::serial;

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
/// hoprd configures 1 s for its 3 s frame timeout; the library default of 6 s would make each drop
/// take longer than the whole observation window.
const SURB_WAIT: Duration = Duration::from_secs(1);

/// A two-node cluster with a long Exit idle timeout and a short routing wait.
///
/// `notify_period` is the Exit's `surb_balance_notify_period`; the fixture otherwise pins it to
/// `None`, which would disable the very stream under test.
fn live_cluster(notify_period: Option<Duration>) -> hopr_lib::testing::fixtures::ClusterGuard {
    cluster_fixture(vec![
        TestNodeConfig {
            win_prob: 1.0,
            ..Default::default()
        },
        TestNodeConfig {
            win_prob: 1.0,
            // The shipped period is 15 s, longer than this cluster's default 2.5 s idle timeout; the
            // Session has to outlive the observation window.
            idle_timeout_ms: Duration::from_secs(60).as_millis() as u64,
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
fn nrc_session_config(surb_target: Option<u64>) -> anyhow::Result<HoprSessionClientConfig> {
    Ok(HoprSessionClientConfig {
        forward_path: HopRouting::try_from(0usize)?,
        return_path: HopRouting::try_from(0usize)?,
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
async fn drive_session(session: &mut hopr_lib::HoprSession, total: Duration) -> anyhow::Result<usize> {
    let started = tokio::time::Instant::now();
    let before = routing_resolution_surb_timeout_count();
    let mut buf = [0u8; 4];
    while started.elapsed() < total {
        let _ = tokio::time::timeout(PING * 3, async {
            session.write_all(b"ping").await?;
            session.flush().await?;
            session.read_exact(&mut buf).await
        })
        .await;
        tokio::time::sleep(PING).await;
    }
    Ok(routing_resolution_surb_timeout_count() - before)
}

/// The bug: a `NoRateControl` Session whose Entry runs no balancer must not be kept alive by the Exit.
///
/// Before the fix this dropped a packet every period — measured at 19–20 drops in this window, at
/// exactly the period's cadence, with the Exit's own `announced_surb_target=… effective=…` derivation
/// confirming it was the notification stream doing it.
#[tokio::test(flavor = "multi_thread")]
#[serial]
#[test_log::test]
#[cfg(feature = "session-client")]
async fn a_balanced_less_entry_keeps_the_exit_alive_without_spending_surbs() -> anyhow::Result<()> {
    let cluster = live_cluster(Some(NOTIFY_PERIOD));
    let ip = IpOrHost::from_str(":0")?;
    let (mut session, _cfg) = cluster[0]
        .inner()
        .connect_to(
            cluster[1].address(),
            SessionTarget::UdpStream(SealedHost::Plain(ip)),
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
#[tokio::test(flavor = "multi_thread")]
#[serial]
#[test_log::test]
#[cfg(feature = "session-client")]
async fn an_idle_no_rate_control_session_originates_nothing_it_cannot_pay_for() -> anyhow::Result<()> {
    let cluster = live_cluster(Some(NOTIFY_PERIOD));
    let ip = IpOrHost::from_str(":0")?;
    let (_session, _cfg) = cluster[0]
        .inner()
        .connect_to(
            cluster[1].address(),
            SessionTarget::UdpStream(SealedHost::Plain(ip)),
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

/// The other half of the rule: disabling the notification entirely must not be what fixes this, since
/// a Session whose balancer *is* enabled still needs its level corrected.
///
/// Asserting on behaviour rather than on the config keeps the two apart: this Session's Exit keeps the
/// configured period, so the stream is live, and the only reason it is unobservable here is that a
/// balanced Entry answers it with a SURB every period instead of starving.
#[tokio::test(flavor = "multi_thread")]
#[serial]
#[test_log::test]
#[cfg(feature = "session-client")]
async fn a_balanced_entry_is_not_starved_by_the_exits_reports() -> anyhow::Result<()> {
    let cluster = live_cluster(Some(NOTIFY_PERIOD));
    let ip = IpOrHost::from_str(":0")?;
    let (mut session, _cfg) = cluster[0]
        .inner()
        .connect_to(
            cluster[1].address(),
            SessionTarget::UdpStream(SealedHost::Plain(ip)),
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
/// gets its report at the configured period.
///
/// This is the guard against "fix it by disabling the feature". A `NoRateControl` Entry that *does*
/// run a balancer is the case the report serves, and it must keep receiving them.
#[tokio::test(flavor = "multi_thread")]
#[serial]
#[test_log::test]
#[cfg(feature = "session-client")]
async fn a_balanced_session_still_receives_the_exits_level_reports() -> anyhow::Result<()> {
    let cluster = live_cluster(Some(NOTIFY_PERIOD));
    let ip = IpOrHost::from_str(":0")?;
    let (mut session, _cfg) = cluster[0]
        .inner()
        .connect_to(
            cluster[1].address(),
            SessionTarget::UdpStream(SealedHost::Plain(ip)),
            nrc_session_config(Some(64))?,
        )
        .await
        .context("the NoRateControl session must establish")?;

    let drops = drive_session(&mut session, OBSERVE).await?;
    assert_eq!(0, drops, "the report stream must be affordable for a balanced Entry");

    // The Exit's reports are what the Entry's balancer reads, so a Session that ran for the whole
    // window must have consumed far more return packets than its own pings account for.
    let elapsed_secs = OBSERVE.as_secs_f64();
    let keep_alives = (elapsed_secs / NOTIFY_PERIOD.as_secs_f64()).ceil() as usize;
    tracing::info!(keep_alives, "reports expected at this period");
    assert!(
        keep_alives > 5,
        "the window must be long enough for the period to matter ({keep_alives} reports in {OBSERVE:?})"
    );
    Ok(())
}
