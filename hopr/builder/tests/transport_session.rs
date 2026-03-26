#[cfg(feature = "session-client")]
use futures::future::try_join_all;
use hopr_builder::testing::{
    fixtures::{
        MINIMUM_INCOMING_WIN_PROB, TEST_GLOBAL_TIMEOUT, TestNodeConfig, chain_propagation_delay, cluster_fixture,
    },
    hopr::ChannelGuard,
};
use hopr_chain_connector::blokli_client::BlokliQueryClient;
#[cfg(feature = "session-client")]
use hopr_lib::HoprBalance;
use rand::seq::SliceRandom;
use rstest::*;
use serial_test::serial;

const FUNDING_AMOUNT: &str = "100 wxHOPR";

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

    let _session = cluster.create_session(&path).await?;

    // TODO: check here that the destination sees the new session created

    try_join_all(
        all_channels
            .into_iter()
            .map(move |guard| async move { guard.try_close_channels_all_channels().await }),
    )
    .await?;

    Ok(())
}
