use anyhow::Context;
use hopr_lib::api::{PeerId, chain::ChainKeyOperations, node::HasChainApi};
use hopr_lib::testing::fixtures::{ClusterGuard, TEST_GLOBAL_TIMEOUT, size_2_cluster_fixture as cluster};
use rstest::*;
use serial_test::serial;

#[rstest]
#[test_log::test]
#[timeout(TEST_GLOBAL_TIMEOUT)]
#[serial]
/// Ensures node APIs can convert peerIDs to chain keys and back by deriving both
/// representations for a sampled node and asserting the conversions round-trip.
fn peerids_should_be_convertible_to_chain_keys_and_vice_versa(cluster: &ClusterGuard) -> anyhow::Result<()> {
    let [candidate, tester] = cluster.sample_nodes::<2>();

    let peer_id = candidate.peer_id();
    let chain_key = candidate.address();

    let offchain_key = hopr_lib::peer_id_to_offchain_key(&peer_id)?;
    let derived_chain_key = tester
        .inner()
        .chain_api()
        .packet_key_to_chain_key(&offchain_key)
        .context("failed to convert peer id to chain key")?
        .context("no chain key found for peer id")?;

    assert_eq!(chain_key, derived_chain_key);

    let derived_offchain_key = tester
        .inner()
        .chain_api()
        .chain_key_to_packet_key(&chain_key)
        .context("failed to convert chain key to peer id")?
        .context("no peer id found for chain key")?;

    let derived_peer_id: PeerId = derived_offchain_key.into();
    assert_eq!(peer_id, derived_peer_id);

    Ok(())
}
