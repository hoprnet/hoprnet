use anyhow::Context;
use hopr_lib::{
    Address,
    testing::fixtures::{ClusterGuard, TEST_GLOBAL_TIMEOUT, size_3_cluster_fixture as cluster},
};
use rstest::*;
use serial_test::serial;

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
/// Ensures nodes expose discoverable peers by fetching the list of public nodes
/// from a random cluster member and asserting it equals the expected count.
async fn all_visible_peers_should_be_listed(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [node] = cluster.sample_nodes::<1>();

    let nodes = node
        .inner()
        .get_public_nodes()
        .await
        .context("should get public nodes")?;

    assert_eq!(nodes.len(), cluster.size());

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
/// Confirms peer-to-peer reachability by pinging another sampled node and
/// verifying the transport API reports success.
async fn ping_should_succeed_for_all_visible_nodes(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [src, dst] = cluster.sample_nodes::<2>();

    let _ = src.inner().ping(&dst.peer_id()).await.context("failed to ping peer")?;

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
/// Guards against self-pings by attempting to ping the same node and asserting
/// the operation fails.
async fn ping_should_fail_for_self(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [random_int] = cluster.sample_nodes::<1>();
    let res = random_int.inner().ping(&random_int.peer_id()).await;

    assert!(res.is_err());

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
/// Verifies discovery stays consistent by comparing the announced account list
/// returned by two nodes and ensuring both contain each other's addresses.
async fn discovery_should_produce_the_same_public_announcements_inside_the_network(
    cluster: &ClusterGuard,
) -> anyhow::Result<()> {
    let [idx1, idx2] = cluster.sample_nodes::<2>();

    let accounts_addresses_1 = idx1
        .inner()
        .accounts_announced_on_chain()
        .await
        .context("failed to get announced accounts")?
        .into_iter()
        .map(|acc| acc.chain_addr)
        .collect::<Vec<Address>>();

    let accounts_addresses_2 = idx2
        .inner()
        .accounts_announced_on_chain()
        .await
        .context("failed to get announced accounts")?
        .into_iter()
        .map(|acc| acc.chain_addr)
        .collect::<Vec<Address>>();

    assert!(accounts_addresses_1.contains(&idx1.address()));
    assert!(accounts_addresses_1.contains(&idx2.address()));

    assert_eq!(accounts_addresses_1, accounts_addresses_2);
    Ok(())
}
