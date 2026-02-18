use std::str::FromStr;

use hopr_lib::BoundedVec;
#[cfg(feature = "session-client")]
use hopr_lib::{
    HoprBalance, RoutingOptions, SessionCapabilities, SessionClientConfig, SessionTarget,
    exports::transport::session::{IpOrHost, SealedHost},
};
use hopr_reference::testing::{
    fixtures::{ClusterGuard, TEST_GLOBAL_TIMEOUT, size_3_cluster_fixture as cluster},
    hopr::ChannelGuard,
};
use rstest::*;
use serial_test::serial;

const FUNDING_AMOUNT: &str = "10 wxHOPR";

#[rstest]
#[case(0)]
#[case(1)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(2*TEST_GLOBAL_TIMEOUT)]
#[cfg(feature = "session-client")]
/// Spins up clusters of varying hops, funds the channels along the entire
/// path and ensures the session client can successfully establish multi-hop sessions.
async fn test_create_n_hop_session(cluster: &ClusterGuard, #[case] hops: usize) -> anyhow::Result<()> {
    let path = cluster.sample_nodes::<3>(); // only shuffles the nodes. 

    let [src, dst] = [&path[0], &path[path.len() - 1]];
    let mid = match hops {
        0 => &[],
        1.. => &path[1..path.len() - 1],
    };

    let channels_there = ChannelGuard::try_open_channels_for_path(
        path.iter().map(|node| node.instance.clone()).collect::<Vec<_>>(),
        FUNDING_AMOUNT.parse::<HoprBalance>()?,
    )
    .await?;

    let channels_back = ChannelGuard::try_open_channels_for_path(
        path.iter().rev().map(|node| node.instance.clone()).collect::<Vec<_>>(),
        FUNDING_AMOUNT.parse::<HoprBalance>()?,
    )
    .await?;

    let (routing, capabilities) = if hops == 0 {
        (RoutingOptions::Hops(0_u32.try_into()?), SessionCapabilities::empty())
    } else {
        (
            RoutingOptions::IntermediatePath(BoundedVec::from_iter(mid.iter().map(|node| node.address().into()))),
            SessionCapabilities::default(),
        )
    };

    let _session = src
        .inner()
        .connect_to(
            dst.address(),
            SessionTarget::UdpStream(SealedHost::Plain(IpOrHost::from_str(":0")?)),
            SessionClientConfig {
                forward_path_options: routing.clone(),
                return_path_options: routing,
                capabilities,
                pseudonym: None,
                surb_management: None,
                always_max_out_surbs: false,
            },
        )
        .await?;

    // TODO: check here that the destination sees the new session created

    channels_there.try_close_channels_all_channels().await?;
    channels_back.try_close_channels_all_channels().await?;

    Ok(())
}
