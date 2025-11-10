use std::{str::FromStr, time::Duration};

use anyhow::Context;
use hopr_lib::{
    HoprBalance, HoprTransportError, RoutingOptions, SessionCapabilities, SessionClientConfig, SessionTarget,
    SurbBalancerConfig,
    errors::HoprLibError,
    exports::transport::session::{IpOrHost, SealedHost},
    testing::{
        fixtures::{ClusterGuard, cluster_fixture, exclusive_indexes},
        hopr::ChannelGuard,
    },
};
use hopr_primitive_types::bounded::BoundedVec;
use rstest::rstest;
use serial_test::serial;
use tokio::time::sleep;

const FUNDING_AMOUNT: &str = "0.1 wxHOPR";

#[rstest]
#[tokio::test]
#[serial]
#[cfg(feature = "session-client")]
async fn test_create_0_hop_session(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [src, dst] = exclusive_indexes::<2>();

    let ip = IpOrHost::from_str(":0")?;

    let _session = cluster_fixture[src]
        .inner()
        .connect_to(
            cluster_fixture[dst].address(),
            SessionTarget::UdpStream(SealedHost::Plain(ip)),
            SessionClientConfig {
                forward_path_options: RoutingOptions::Hops(0_u32.try_into()?),
                return_path_options: RoutingOptions::Hops(0_u32.try_into()?),
                capabilities: SessionCapabilities::empty(),
                pseudonym: None,
                surb_management: None,
                always_max_out_surbs: false,
            },
        )
        .await?;

    // TODO: check here that the destination sees the new session created

    Ok(())
}

#[rstest]
#[timeout(Duration::from_secs(60))]
#[tokio::test]
#[serial]
#[cfg(feature = "session-client")]
#[test_log::test]
async fn test_create_1_hop_session(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    use futures_time::future::FutureExt as _;

    let [src, mid, dst] = exclusive_indexes::<3>();

    let _channels_there = ChannelGuard::try_open_channels_for_path(
        vec![
            cluster_fixture[src].instance.clone(),
            cluster_fixture[mid].instance.clone(),
            cluster_fixture[dst].instance.clone(),
        ],
        FUNDING_AMOUNT.parse::<HoprBalance>()?,
    )
    .await?;

    let _channels_back = ChannelGuard::try_open_channels_for_path(
        vec![
            cluster_fixture[dst].instance.clone(),
            cluster_fixture[mid].instance.clone(),
            cluster_fixture[src].instance.clone(),
        ],
        FUNDING_AMOUNT.parse::<HoprBalance>()?,
    )
    .await?;

    sleep(std::time::Duration::from_secs(3)).await;

    let ip = IpOrHost::from_str(":0")?;
    let routing = RoutingOptions::IntermediatePath(BoundedVec::from_iter(std::iter::once(
        cluster_fixture[mid].address().into(),
    )));

    let _session = cluster_fixture[src]
        .inner()
        .connect_to(
            cluster_fixture[dst].address(),
            SessionTarget::UdpStream(SealedHost::Plain(ip)),
            SessionClientConfig {
                forward_path_options: routing.clone(),
                return_path_options: routing,
                capabilities: Default::default(),
                pseudonym: None,
                surb_management: None,
                always_max_out_surbs: false,
            },
        )
        .timeout(futures_time::time::Duration::from_secs(30))
        .await?;

    // TODO: check here that the destination sees the new session created

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
#[cfg(feature = "session-client")]
async fn test_keep_alive_session(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    // Test keepalive as well as sending 0 hop messages without channels
    let [src, dst] = exclusive_indexes::<2>();

    let ip = IpOrHost::from_str(":0")?;

    let session = cluster_fixture[src]
        .inner()
        .connect_to(
            cluster_fixture[dst].address(),
            SessionTarget::UdpStream(SealedHost::Plain(ip)),
            SessionClientConfig {
                forward_path_options: RoutingOptions::Hops(0_u32.try_into()?),
                return_path_options: RoutingOptions::Hops(0_u32.try_into()?),
                capabilities: SessionCapabilities::empty(),
                pseudonym: None,
                surb_management: None,
                always_max_out_surbs: false,
            },
        )
        .await?;

    sleep(Duration::from_secs(2)).await;

    cluster_fixture[src]
        .inner()
        .keep_alive_session(&session.id())
        .await
        .context("failed to keep alive session")?;

    sleep(Duration::from_secs(3)).await; // sleep longer than the session timeout

    match cluster_fixture[src].inner().keep_alive_session(&session.id()).await {
        Err(HoprLibError::TransportError(HoprTransportError::Session(hopr_lib::TransportSessionError::Manager(
            hopr_lib::SessionManagerError::NonExistingSession,
        )))) => {}
        Err(e) => panic!(
            "expected SessionNotFound error when keeping alive session, but got different error: {:?}",
            e
        ),
        Ok(_) => panic!("expected error when keeping alive session, but got Ok"),
    }

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
#[cfg(feature = "session-client")]
async fn test_session_surb_balancer_config(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    use hopr_primitive_types::bounded::BoundedVec;

    let [src, mid, dst] = exclusive_indexes::<3>();

    let _channels_there = ChannelGuard::try_open_channels_for_path(
        vec![
            cluster_fixture[src].instance.clone(),
            cluster_fixture[mid].instance.clone(),
            cluster_fixture[dst].instance.clone(),
        ],
        FUNDING_AMOUNT.parse::<HoprBalance>()?,
    )
    .await?;
    let _channels_back = ChannelGuard::try_open_channels_for_path(
        vec![
            cluster_fixture[dst].instance.clone(),
            cluster_fixture[mid].instance.clone(),
            cluster_fixture[src].instance.clone(),
        ],
        FUNDING_AMOUNT.parse::<HoprBalance>()?,
    )
    .await?;

    sleep(std::time::Duration::from_secs(5)).await;

    let exp_config = SurbBalancerConfig {
        target_surb_buffer_size: 10,
        max_surbs_per_sec: 100,
        ..Default::default()
    };

    let ip = IpOrHost::from_str(":0")?;
    let routing = RoutingOptions::IntermediatePath(BoundedVec::from_iter(std::iter::once(
        cluster_fixture[mid].address().into(),
    )));

    let session = cluster_fixture[src]
        .inner()
        .connect_to(
            cluster_fixture[dst].address(),
            SessionTarget::UdpStream(SealedHost::Plain(ip)),
            SessionClientConfig {
                forward_path_options: routing.clone(),
                return_path_options: routing,
                capabilities: Default::default(),
                pseudonym: None,
                surb_management: Some(exp_config),
                always_max_out_surbs: false,
            },
        )
        .await
        .context("creating a session must succeed")?;

    let config = cluster_fixture[src]
        .inner()
        .get_session_surb_balancer_config(&session.id())
        .await
        .context("failed to get surb balancer config")?;

    assert_eq!(config, Some(exp_config));

    cluster_fixture[src]
        .inner()
        .update_session_surb_balancer_config(&session.id(), SurbBalancerConfig::default())
        .await
        .context("failed to update surb balancer config")?;

    let config = cluster_fixture[src]
        .inner()
        .get_session_surb_balancer_config(&session.id())
        .await
        .context("failed to get surb balancer config")?;

    assert_eq!(config, Some(SurbBalancerConfig::default()));

    Ok(())
}
