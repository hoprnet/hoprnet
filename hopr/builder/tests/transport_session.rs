use std::str::FromStr;

#[cfg(feature = "session-client")]
use futures::future::try_join_all;
use hopr_builder::testing::{
    fixtures::{ClusterGuard, TEST_GLOBAL_TIMEOUT, chain_propagation_delay, size_5_cluster_fixture as cluster},
    hopr::ChannelGuard,
};
use hopr_chain_connector::blokli_client::BlokliQueryClient;
#[cfg(feature = "session-client")]
use hopr_lib::{
    HopRouting, HoprBalance, HoprSessionClientConfig, SessionCapabilities, SessionTarget,
    exports::transport::session::{IpOrHost, SealedHost},
};
use rand::seq::SliceRandom;
use rstest::*;
use serial_test::serial;
#[cfg(feature = "session-client")]
use tokio::time::sleep;

const FUNDING_AMOUNT: &str = "100 wxHOPR";

#[rstest]
#[case(0)]
#[case(1)]
#[case(2)]
#[case(3)]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(2*TEST_GLOBAL_TIMEOUT)]
#[cfg(feature = "session-client")]
/// Tests n-hop session establishment over a fully connected channel network.
///
/// Selects `hops + 2` nodes from a 5-node cluster and opens bidirectional channels
/// between every pair, so the path planner has the full graph available.
///
/// - 0-hop: 2 nodes, no channels needed (direct connection)
/// - n-hop (n >= 1): n+2 nodes with n*(n+1) bidirectional channels
async fn create_n_hop_session(cluster: &ClusterGuard, #[case] hops: usize) -> anyhow::Result<()> {
    // 2-hop and 3-hop tests are too slow under coverage instrumentation
    #[allow(unexpected_cfgs)]
    if cfg!(coverage) && hops > 1 {
        return Ok(());
    }

    let node_count = if hops == 0 { 2 } else { hops + 2 };
    let mut nodes: Vec<&_> = cluster.iter().collect();
    nodes.shuffle(&mut rand::rng());
    nodes.truncate(node_count);

    let src = nodes[0];
    let dst = nodes[nodes.len() - 1];
    let mid = &nodes[1..nodes.len() - 1];

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

    // Wait for probing to register opened channels in the graph.
    // Scale the wait by hop count: larger meshes need more probing rounds.
    if !all_channels.is_empty() {
        let chain_info = cluster.chain_client.query_chain_info().await?;
        let base_delay = chain_propagation_delay(&chain_info);
        sleep(base_delay * hops as u32).await;
    }

    let (routing, capabilities) = if hops == 0 {
        (HopRouting::try_from(0_usize)?, SessionCapabilities::empty())
    } else {
        (HopRouting::try_from(mid.len())?, SessionCapabilities::default())
    };

    let _session = src
        .inner()
        .connect_to(
            dst.address(),
            SessionTarget::UdpStream(SealedHost::Plain(IpOrHost::from_str(":0")?)),
            HoprSessionClientConfig {
                forward_path: routing,
                return_path: routing,
                capabilities,
                pseudonym: None,
                surb_management: None,
                always_max_out_surbs: false,
            },
        )
        .await?;

    // TODO: check here that the destination sees the new session created

    try_join_all(
        all_channels
            .into_iter()
            .map(move |guard| async move { guard.try_close_channels_all_channels().await }),
    )
    .await?;

    Ok(())
}
