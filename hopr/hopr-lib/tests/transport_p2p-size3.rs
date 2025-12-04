use anyhow::Context;
use hopr_lib::{
    Address,
    testing::fixtures::{ClusterGuard, TEST_GLOBAL_TIMEOUT, size_3_cluster_fixture as cluster},
};
use rstest::rstest;
use serial_test::serial;

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
async fn all_visible_peers_should_be_listed(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [node] = cluster.sample_nodes::<1>();

    let config = node
        .inner()
        .get_public_nodes()
        .await
        .context("should get public nodes")?;

    assert!(!config.is_empty()); // TODO: change to exact number of public nodes

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
async fn ping_should_succeed_for_all_visible_nodes(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [src, dst] = cluster.sample_nodes::<2>();

    let _ = src.inner().ping(&dst.peer_id()).await.context("failed to ping peer")?;

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
async fn ping_should_fail_for_self(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [random_int] = cluster.sample_nodes::<1>();
    let res = random_int.inner().ping(&random_int.peer_id()).await;

    assert!(res.is_err());

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
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
