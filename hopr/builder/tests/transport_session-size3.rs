use std::{str::FromStr, time::Duration};

use anyhow::Context;
use hopr_lib::{
    RoutingOptions, SessionCapabilities, SessionClientConfig, SessionTarget,
    errors::{HoprLibError, HoprTransportError},
    exports::transport::session::{IpOrHost, SealedHost},
};
use hopr_builder::testing::fixtures::{ClusterGuard, TEST_GLOBAL_TIMEOUT, size_3_cluster_fixture as cluster};
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
        .keep_alive_session(session.id())
        .await
        .context("failed to keep alive session")?;

    sleep(Duration::from_secs(3)).await; // sleep longer than the session timeout

    match src.inner().keep_alive_session(session.id()).await {
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
