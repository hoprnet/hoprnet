use anyhow::Context;
use hopr_lib::Address;
use rstest::rstest;
use serial_test::serial;

mod common;
use common::fixtures::{ClusterGuard, cluster_fixture, exclusive_indexes};

#[rstest]
#[tokio::test]
#[serial]
async fn all_visible_peers_should_be_listed(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [idx] = exclusive_indexes::<1>();

    let config = cluster_fixture[idx]
        .inner()
        .get_public_nodes()
        .await
        .context("should get public nodes")?;

    assert!(!config.is_empty()); // TODO: change to exact number of public nodes

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
async fn ping_should_succeed_for_all_visible_nodes(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [src, dst] = exclusive_indexes::<2>();

    let _ = cluster_fixture[src]
        .inner()
        .ping(&cluster_fixture[dst].peer_id())
        .await
        .context("failed to ping peer")?;

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]

async fn ping_should_fail_for_self(#[future(awt)] cluster_fixture: ClusterGuard) -> anyhow::Result<()> {
    let [random_int] = exclusive_indexes::<1>();
    let res = cluster_fixture[random_int]
        .inner()
        .ping(&cluster_fixture[random_int].peer_id())
        .await;

    assert!(res.is_err());

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
async fn discovery_should_produce_the_same_public_announcements_inside_the_network(
    #[future(awt)] cluster_fixture: ClusterGuard,
) -> anyhow::Result<()> {
    let [idx1, idx2] = exclusive_indexes::<2>();

    let accounts_addresses_1 = cluster_fixture[idx1]
        .inner()
        .accounts_announced_on_chain()
        .await
        .context("failed to get announced accounts")?
        .into_iter()
        .map(|acc| acc.chain_addr)
        .collect::<Vec<Address>>();

    let accounts_addresses_2 = cluster_fixture[idx2]
        .inner()
        .accounts_announced_on_chain()
        .await
        .context("failed to get announced accounts")?
        .into_iter()
        .map(|acc| acc.chain_addr)
        .collect::<Vec<Address>>();

    assert!(accounts_addresses_1.contains(&cluster_fixture[idx1].address()));
    assert!(accounts_addresses_1.contains(&cluster_fixture[idx2].address()));

    assert_eq!(accounts_addresses_1, accounts_addresses_2);
    Ok(())
}
