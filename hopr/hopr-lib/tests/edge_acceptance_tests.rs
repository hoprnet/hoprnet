mod common;

use common::fixtures::{SWARM_N, cluster_fixture, random_int_pair};
use common::hopr_tester::HoprTester;

use std::time::Duration;

use rstest::rstest;

#[rstest]
#[cfg(feature = "runtime-tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_get_addresses(#[future(awt)] cluster_fixture: &Vec<HoprTester>) -> anyhow::Result<()> {
    assert!(cluster_fixture[0].address() != Address::default());
    Ok(())
}

#[rstest]
#[cfg(feature = "runtime-tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_get_infos(#[future(awt)] cluster_fixture: &Vec<HoprTester>) -> anyhow::Result<()> {
    let node: &HoprTester = &cluster_fixture[0];

    assert!(node.inner.network() != "");
    assert!(node.inner.get_safe_config().safe_address != Address::default());
    Ok(())
}

#[rstest]
#[cfg(feature = "runtime-tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_get_channels(#[future(awt)] cluster_fixture: &Vec<HoprTester>) -> anyhow::Result<()> {
    let node = &cluster_fixture[0];

    let channels = node.outgoing_channels_by_status(None).await?;
    assert!(channels.is_empty());

    Ok(())
}

#[rstest]
#[cfg(all(feature = "runtime-tokio", feature = "session-client"))]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_open_and_close_sessions(#[future(awt)] cluster_fixture: &Vec<HoprTester>) -> anyhow::Result<()> {
    use hopr_lib::{SessionClientConfig, SessionTarget};
    use hopr_transport_session::{Capabilities, Capability}; // TODO: should use hopr-lib instead

    let session = cluster_fixture[0]
        .inner
        .connect_to(
            &cluster_fixture[1].address(),
            SessionTarget::UdpStream(":0"),
            SessionClientConfig {
                forward_path_options: hopr_lib::RoutingOptions::Hops(0_u32.try_into()?),
                return_path_options: hopr_lib::RoutingOptions::Hops(0_u32.try_into()?),
                capabilities: Capabilities::from(Capability::Segmentation),
                pseudonym: None,
                surb_management: None,
                always_max_out_surbs: true,
            },
        )
        .await
        .expect("creating a session must succeed");

    // let sessions = cluster_fixture[0].list_sessions().await?; // TODO. Do once integrated into edgli

    Ok(())
}

#[rstest]
#[cfg(feature = "runtime-tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_update_session_config(#[future(awt)] cluster_fixture: &Vec<HoprTester>) -> anyhow::Result<()> {
    use hopr_lib::SurbBalancerConfig;

    let node = &cluster_fixture[0];

    let session = node
        .inner
        .connect_to(
            &cluster_fixture[1].address(),
            SessionTarget::UdpStream(":0"),
            SessionClientConfig {
                forward_path_options: hopr_lib::RoutingOptions::Hops(0_u32.try_into()?),
                return_path_options: hopr_lib::RoutingOptions::Hops(0_u32.try_into()?),
                capabilities: Capabilities::from(Capability::Segmentation),
                pseudonym: None,
                surb_management: None,
                always_max_out_surbs: true,
            },
        )
        .await
        .expect("creating a session must succeed");

    let config = node
        .get_session_surb_balancer_config(&session.id())
        .await
        .expect("should get session config");

    assert_eq!(config, None);

    let new_config = SurbBalancerConfig {
        target_surb_buffer_size: 5_000,
        max_surbs_per_sec: 2500,
        surb_decay: Some((Duration::from_millis(200), 0.05)),
    };

    node.update_session_surb_balancer_config(&session.id(), new_config.clone())
        .await
        .expect("should update session config");

    assert_eq!(
        config = node
            .get_session_surb_balancer_config(&session.id())
            .await
            .expect("should get session config"),
        Some(new_config)
    );

    Ok(())
}
