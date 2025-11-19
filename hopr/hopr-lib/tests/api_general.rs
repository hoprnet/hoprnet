use anyhow::Context;
use hopr_chain_connector::testing::{BlokliTestClient, FullStateEmulator};
use hopr_lib::testing::fixtures::{ClusterGuard, TEST_GLOBAL_TIMEOUT, build_cluster_fixture, chainenv_fixture};
use rstest::rstest;
use serial_test::serial;

#[rstest::fixture]
pub async fn cluster_fixture(#[future(awt)] chainenv_fixture: BlokliTestClient<FullStateEmulator>) -> ClusterGuard {
    build_cluster_fixture(chainenv_fixture, 2).await
}

#[rstest]
#[test_log::test(tokio::test)]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
async fn peerids_should_be_convertible_to_chain_keys_and_vice_versa(
    #[future(awt)] cluster_fixture: ClusterGuard,
) -> anyhow::Result<()> {
    let [candidate, tester] = cluster_fixture.exclusive_indexes::<2>();

    let peer_id = candidate.peer_id();
    let chain_key = candidate.address();

    let derived_chain_key = tester
        .inner()
        .peerid_to_chain_key(&peer_id)
        .await
        .context("failed to convert peer id to chain key")?
        .context("no chain key found for peer id")?;

    assert_eq!(chain_key, derived_chain_key);

    let derived_peer_id = tester
        .inner()
        .chain_key_to_peerid(&chain_key)
        .await
        .context("failed to convert chain key to peer id")?
        .context("no peer id found for chain key")?;

    assert_eq!(peer_id, derived_peer_id);

    Ok(())
}
