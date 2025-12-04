use std::str::FromStr;

use hopr_lib::{
    HoprBalance, RoutingOptions, SessionCapabilities, SessionClientConfig, SessionTarget,
    exports::transport::session::{IpOrHost, SealedHost},
    testing::{
        fixtures::{TEST_GLOBAL_TIMEOUT, cluster_fixture},
        hopr::ChannelGuard,
    },
};
use hopr_primitive_types::bounded::BoundedVec;
use rand::{seq::SliceRandom, thread_rng};
use rstest::rstest;
use serial_test::serial;

const FUNDING_AMOUNT: &str = "10 wxHOPR";

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[case(0)]
#[case(1)]
#[serial]
#[cfg(feature = "session-client")]
/// Spins up clusters of varying hops, funds the channels along the entire
/// path and ensures the session client can successfully establish multi-hop sessions.
async fn test_create_n_hop_session(#[case] hops: usize) -> anyhow::Result<()> {
    let mut path = cluster_fixture(hops + 2).cluster;
    path.shuffle(&mut thread_rng());

    // extract src and dst as the first and last nodes in the path. Extract mid as the middle nodes in the path
    let src = &path[0];
    let dst = &path[path.len() - 1];
    let mid = &path[1..path.len() - 1];

    let _channels_there = ChannelGuard::try_open_channels_for_path(
        path.iter().map(|node| node.instance.clone()).collect::<Vec<_>>(),
        FUNDING_AMOUNT.parse::<HoprBalance>()?,
    )
    .await?;

    let _channels_back = ChannelGuard::try_open_channels_for_path(
        path.iter().rev().map(|node| node.instance.clone()).collect::<Vec<_>>(),
        FUNDING_AMOUNT.parse::<HoprBalance>()?,
    )
    .await?;

    let routing = if hops == 0 {
        RoutingOptions::Hops(0_u32.try_into()?)
    } else {
        RoutingOptions::IntermediatePath(BoundedVec::from_iter(mid.iter().map(|node| node.address().into())))
    };

    let _session = src
        .inner()
        .connect_to(
            dst.address(),
            SessionTarget::UdpStream(SealedHost::Plain(IpOrHost::from_str(":0")?)),
            SessionClientConfig {
                forward_path_options: routing.clone(),
                return_path_options: routing,
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
