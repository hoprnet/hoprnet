use std::{str::FromStr, time::Duration};

use anyhow::Context;
use futures_time::future::FutureExt as _;
use hopr_lib::{
    HoprBalance, RoutingOptions, SessionCapabilities, SessionClientConfig, SessionTarget, SurbBalancerConfig,
    errors::{HoprLibError, HoprTransportError},
    exports::transport::session::{IpOrHost, SealedHost},
    testing::{
        fixtures::{ClusterGuard, TEST_GLOBAL_TIMEOUT, cluster_fixture},
        hopr::ChannelGuard,
    },
};
use hopr_primitive_types::bounded::BoundedVec;
use rstest::rstest;
use serial_test::serial;
use tokio::time::sleep;

const FUNDING_AMOUNT: &str = "10 wxHOPR";

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
#[cfg(feature = "session-client")]
async fn test_create_0_hop_session(#[with(2)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [src, dst] = cluster_fixture.sample_nodes::<2>();

    let ip = IpOrHost::from_str(":0")?;

    let _session = src
        .inner()
        .connect_to(
            dst.address(),
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
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
#[cfg(feature = "session-client")]
async fn test_create_1_hop_session(#[with(3)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [src, mid, dst] = cluster_fixture.sample_nodes::<3>();

    let _channels_there = ChannelGuard::try_open_channels_for_path(
        [src.instance.clone(), mid.instance.clone(), dst.instance.clone()],
        FUNDING_AMOUNT.parse::<HoprBalance>()?,
    )
    .await?;

    let _channels_back = ChannelGuard::try_open_channels_for_path(
        [dst.instance.clone(), mid.instance.clone(), src.instance.clone()],
        FUNDING_AMOUNT.parse::<HoprBalance>()?,
    )
    .await?;

    sleep(Duration::from_secs(1)).await;

    let ip = IpOrHost::from_str(":0")?;
    let routing = RoutingOptions::IntermediatePath(BoundedVec::from_iter(std::iter::once(mid.address().into())));

    let _session = src
        .inner()
        .connect_to(
            dst.address(),
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
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
#[cfg(feature = "session-client")]
async fn test_keep_alive_session(#[with(2)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    // Test keepalive as well as sending 0 hop messages without channels
    let [src, dst] = cluster_fixture.sample_nodes::<2>();

    let ip = IpOrHost::from_str(":0")?;

    let session = src
        .inner()
        .connect_to(
            dst.address(),
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

    src.inner()
        .keep_alive_session(&session.id())
        .await
        .context("failed to keep alive session")?;

    sleep(Duration::from_secs(3)).await; // sleep longer than the session timeout

    match src.inner().keep_alive_session(&session.id()).await {
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
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
#[cfg(feature = "session-client")]
async fn test_session_surb_balancer_config(#[with(3)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [src, mid, dst] = cluster_fixture.sample_nodes::<3>();

    let _channels_there = ChannelGuard::try_open_channels_for_path(
        [src.instance.clone(), mid.instance.clone(), dst.instance.clone()],
        FUNDING_AMOUNT.parse::<HoprBalance>()?,
    )
    .await?;
    let _channels_back = ChannelGuard::try_open_channels_for_path(
        [dst.instance.clone(), mid.instance.clone(), src.instance.clone()],
        FUNDING_AMOUNT.parse::<HoprBalance>()?,
    )
    .await?;

    sleep(Duration::from_secs(1)).await;

    let exp_config = SurbBalancerConfig {
        target_surb_buffer_size: 10,
        max_surbs_per_sec: 100,
        ..Default::default()
    };

    let ip = IpOrHost::from_str(":0")?;
    let routing = RoutingOptions::IntermediatePath(BoundedVec::from_iter(std::iter::once(mid.address().into())));

    let session = src
        .inner()
        .connect_to(
            dst.address(),
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

    let config = src
        .inner()
        .get_session_surb_balancer_config(&session.id())
        .await
        .context("failed to get surb balancer config")?;

    assert_eq!(config, Some(exp_config));

    src.inner()
        .update_session_surb_balancer_config(&session.id(), SurbBalancerConfig::default())
        .await
        .context("failed to update surb balancer config")?;

    let config = src
        .inner()
        .get_session_surb_balancer_config(&session.id())
        .await
        .context("failed to get surb balancer config")?;

    assert_eq!(config, Some(SurbBalancerConfig::default()));

    Ok(())
}
