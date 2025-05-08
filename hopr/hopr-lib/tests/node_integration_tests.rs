mod common;

use alloy::primitives::U256;
use std::time::Duration;

use hopr_chain_rpc::client::SnapshotRequestor;
use hopr_crypto_types::prelude::{Keypair, OffchainKeypair};

use crate::common::{deploy_test_environment, onboard_node};

const SNAPSHOT_BASE: &str = "tests/snapshots/node_snapshot_base";

#[ignore] // Ignore for now, until the actual test is implemented
#[cfg_attr(feature = "runtime-async-std", async_std::test)]
async fn hopr_node_integration_test() {
    let block_time = Duration::from_secs(1);
    let finality = 2;

    let requestor_base = SnapshotRequestor::new(SNAPSHOT_BASE)
        .with_ignore_snapshot(!hopr_crypto_random::is_rng_fixed())
        .load(true)
        .await;

    let chain_env = deploy_test_environment(requestor_base, block_time, finality).await;

    let alice_chain_key = chain_env.node_chain_keys[0].clone();
    let bob_chain_key = chain_env.node_chain_keys[1].clone();

    let _alice_offchain_key = OffchainKeypair::random();
    let _bob_offchain_key = OffchainKeypair::random();

    let _alice_node_safe = onboard_node(&chain_env, &alice_chain_key, U256::from(10_u32), U256::from(10_000_u32)).await;

    let _bob_node_safe = onboard_node(&chain_env, &bob_chain_key, U256::from(10_u32), U256::from(10_000_u32)).await;

    // TODO: instantiate Hopr for both nodes
}
