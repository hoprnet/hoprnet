mod common;

use std::time::Duration;

use hopr_chain_rpc::client::{reqwest_client::ReqwestRequestor, SnapshotRequestor};
use hopr_crypto_types::prelude::{Keypair, OffchainKeypair};

use crate::common::{deploy_test_environment, onboard_node};

const SNAPSHOT_BASE: &str = "tests/snapshots/node_snapshot_base";

#[ignore] // Ignore for now, until the actual test is implemented
// #[tracing_test::traced_test]
#[tokio::test]
async fn hopr_node_integration_test() {
    let block_time = Duration::from_secs(1);
    let finality = 2;

    let requestor_base = SnapshotRequestor::new(ReqwestRequestor::default(), SNAPSHOT_BASE)
        .with_ignore_snapshot(!hopr_crypto_random::is_rng_fixed())
        .load(true)
        .await;

    let chain_env = deploy_test_environment(requestor_base, block_time, finality).await;

    let alice_chain_key = chain_env.node_chain_keys[0].clone();
    let bob_chain_key = chain_env.node_chain_keys[1].clone();

    let _alice_offchain_key = OffchainKeypair::random();
    let _bob_offchain_key = OffchainKeypair::random();

    let _alice_node_safe = onboard_node(&chain_env, &alice_chain_key, 10_u32.into(), 10_000_u32.into()).await;

    let _bob_node_safe = onboard_node(&chain_env, &bob_chain_key, 10_u32.into(), 10_000_u32.into()).await;

    // TODO: instantiate Hopr for both nodes
}
