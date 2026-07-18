use hopr_lib::testing::fixtures::{
    MINIMUM_INCOMING_WIN_PROB, TEST_GLOBAL_TIMEOUT, TestNodeConfig, chain_propagation_delay, cluster_fixture,
};
#[cfg(feature = "session-client")]
use {
    futures::future::try_join_all, hopr_chain_connector::blokli_client::BlokliQueryClient,
    hopr_lib::api::types::primitive::prelude::HoprBalance, hopr_lib::testing::hopr::ChannelGuard,
    rand::seq::SliceRandom, rstest::*, serial_test::serial,
};

const FUNDING_AMOUNT: &str = "500 wxHOPR";

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

// =========================================================================
//  Bidirectional data flow through sessions
// =========================================================================

use std::{str::FromStr, time::Duration};

use anyhow::Context;
use futures::{AsyncReadExt, AsyncWriteExt};
use hopr_lib::{
    api::node::HoprSessionClientOperations,
    exports::{
        network::types::prelude::{IpOrHost, SealedHost},
        transport::SessionCapability,
    },
};

/// Run bidirectional echo on a session, verifying data round-trips via EchoServer.
async fn verify_session_echo(
    entry_session: &mut hopr_lib::exports::transport::HoprSession,
    label: &str,
) -> anyhow::Result<()> {
    let msg: [u8; 32] = hopr_lib::api::types::crypto_random::random_bytes();

    let mut echoed = vec![0u8; 32];
    tokio::time::timeout(Duration::from_secs(10), async {
        entry_session.write_all(&msg).await?;
        entry_session.flush().await?;
        entry_session.read_exact(&mut echoed).await?;
        anyhow::Ok(())
    })
        .await
        .with_context(|| format!("{label}: echo round-trip timeout"))??;
    assert_eq!(&msg[..], &echoed[..], "{label}: echo mismatch");

    tracing::info!("{label}: echo verified");
    Ok(())
}

#[rstest]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[cfg(feature = "session-client")]
/// 0-hop bidirectional data flow through a HOPR session.
async fn capture_direct_session() -> anyhow::Result<()> {
    let cluster = cluster_fixture(vec![TestNodeConfig::default(); 2]);
    assert_eq!(cluster.size(), 2);

    let src = &cluster[0];
    let dst = &cluster[1];

    let ip = IpOrHost::from_str(":0")?;
    let (mut session, _) = src
        .inner()
        .connect_to(
            dst.address(),
            hopr_lib::exports::transport::SessionTarget::UdpStream(SealedHost::Plain(ip)),
            hopr_lib::HoprSessionClientConfig {
                forward_path: 0.try_into()?,
                return_path: 0.try_into()?,
                capabilities: (SessionCapability::Segmentation | SessionCapability::NoRateControl).into(),
                pseudonym: None,
                surb_management: None,
                always_max_out_surbs: true,
                pix_ssa_quota: None,
            },
        )
        .await?;
    verify_session_echo(&mut session, "0-hop").await?;
    Ok(())
}

#[rstest]
#[serial]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[cfg(feature = "session-client")]
/// 1-hop bidirectional data flow through a HOPR session via a relay.
///
/// SURB balancing is not used — with `always_max_out_surbs: true`, each small
/// data packet (32 bytes) carries enough SURBs alongside the payload for a
/// symmetric 1:1 echo pattern (1 SURB delivered → 1 reply).
async fn capture_one_hop_session() -> anyhow::Result<()> {
    let cluster = cluster_fixture(vec![
        TestNodeConfig::default(),                                   // src:  win_prob=1.0
        TestNodeConfig::with_probability(MINIMUM_INCOMING_WIN_PROB), // relay: win_prob=0.2
        TestNodeConfig::default(),                                   // dst:  win_prob=1.0
    ]);
    let src = &cluster[0];
    let relay = &cluster[1];
    let dst = &cluster[2];

    // Open path channels in both directions for forward and SURB return routing
    let funding = FUNDING_AMOUNT.parse::<HoprBalance>()?;
    let mut channels = Vec::new();
    for (from, to) in [(src, relay), (relay, dst), (dst, relay), (relay, src)] {
        channels
            .push(ChannelGuard::open_channel_between_nodes(from.instance.clone(), to.instance.clone(), funding).await?);
    }

    let chain_info = cluster.chain_client.query_chain_info().await?;
    cluster
        .wait_for_channel_graph(src, channels.len(), chain_propagation_delay(&chain_info) * 6)
        .await?;

    let ip = IpOrHost::from_str(":0")?;
    let (mut session, _) = src
        .inner()
        .connect_to(
            dst.address(),
            hopr_lib::exports::transport::SessionTarget::UdpStream(SealedHost::Plain(ip)),
            hopr_lib::HoprSessionClientConfig {
                forward_path: 1.try_into()?,
                return_path: 1.try_into()?,
                capabilities: (SessionCapability::Segmentation | SessionCapability::NoRateControl).into(),
                pseudonym: None,
                surb_management: None,
                always_max_out_surbs: true,
                pix_ssa_quota: None,
            },
        )
        .await?;
    verify_session_echo(&mut session, "1-hop").await?;

    try_join_all(
        channels
            .into_iter()
            .map(|guard| async move { guard.try_close_channels_all_channels().await }),
    )
    .await?;

    Ok(())
}
