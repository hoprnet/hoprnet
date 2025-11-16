use anyhow::Context;
use hopr_lib::testing::fixtures::{ClusterGuard, cluster_fixture, exclusive_indexes};
use rstest::rstest;
use serial_test::serial;

#[rstest]
#[test_log::test(tokio::test)]
#[serial]
async fn peerids_should_be_convertible_to_chain_keys_and_vice_versa(
    #[future(awt)] cluster_fixture: ClusterGuard,
) -> anyhow::Result<()> {
    let [candidate, tester] = exclusive_indexes::<2>();

    let peer_id = cluster_fixture[candidate].peer_id();
    let chain_key = cluster_fixture[candidate].address();

    let derived_chain_key = cluster_fixture[tester]
        .inner()
        .peerid_to_chain_key(&peer_id)
        .await
        .context("failed to convert peer id to chain key")?
        .context("no chain key found for peer id")?;

    assert_eq!(chain_key, derived_chain_key);

    let derived_peer_id = cluster_fixture[tester]
        .inner()
        .chain_key_to_peerid(&chain_key)
        .await
        .context("failed to convert chain key to peer id")?
        .context("no peer id found for chain key")?;

    assert_eq!(peer_id, derived_peer_id);

    Ok(())
}
