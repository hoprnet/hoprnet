use std::{str::FromStr, time::Duration};

use anyhow::Context;
use hopr_lib::{
    HopRouting, HoprSessionClientConfig,
    api::node::HoprSessionClientOperations,
    errors::HoprTransportError,
    exports::{
        network::types::prelude::{IpOrHost, SealedHost},
        transport::{SessionCapabilities, SessionManagerError, SessionTarget, TransportSessionError},
    },
    testing::fixtures::{ClusterGuard, TEST_GLOBAL_TIMEOUT, size_3_cluster_fixture as cluster},
};
use rstest::*;
use serial_test::serial;
use tokio::time::sleep;

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
#[cfg(feature = "session-client")]
/// Verifies keep-alive semantics by establishing a session without channels,
/// pinging it before and after expiration and asserting stale sessions vanish.
async fn test_keep_alive_session(cluster: &ClusterGuard) -> anyhow::Result<()> {
    // Test keepalive as well as sending 0 hop messages without channels
    let [src, dst] = cluster.sample_nodes::<2>();

    let ip = IpOrHost::from_str(":0")?;

    let (_session, configurator) = src
        .inner()
        .connect_to(
            dst.address(),
            SessionTarget::UdpStream(SealedHost::Plain(ip)),
            HoprSessionClientConfig {
                forward_path: HopRouting::try_from(0_usize)?,
                return_path: HopRouting::try_from(0_usize)?,
                capabilities: SessionCapabilities::empty(),
                pseudonym: None,
                surb_management: None,
                always_max_out_surbs: false,
                pix_ssa_quota: None,
            },
        )
        .await?;

    sleep(Duration::from_secs(2)).await;

    configurator.ping().await.context("failed to keep alive session")?;

    sleep(Duration::from_secs(3)).await; // sleep longer than the session timeout

    match configurator.ping().await {
        Err(HoprTransportError::Session(TransportSessionError::Manager(SessionManagerError::NonExistingSession))) => {}
        Err(e) => panic!(
            "expected SessionNotFound error when keeping alive session, but got different error: {:?}",
            e
        ),
        Ok(_) => panic!("expected error when keeping alive session, but got Ok"),
    }

    Ok(())
}
