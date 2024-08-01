mod common;

use hopr_crypto_types::prelude::{Keypair, OffchainKeypair};
use std::time::Duration;

use crate::common::{deploy_test_environment, onboard_node};

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(all(feature = "runtime-tokio", not(feature = "runtime-async-std")), tokio::test)]
async fn hopr_node_integration_test() {
    let block_time = Duration::from_secs(1);
    let finality = 2;

    let chain_env = deploy_test_environment(block_time, finality).await;

    let alice_chain_key = chain_env.node_chain_keys[0].clone();
    let bob_chain_key = chain_env.node_chain_keys[1].clone();

    let _alice_offchain_key = OffchainKeypair::random();
    let _bob_offchain_key = OffchainKeypair::random();

    let (_alice_module_addr, _alice_safe_addr) =
        onboard_node(&chain_env, &alice_chain_key, 10_u32.into(), 10_000_u32.into()).await;

    let (_bob_module_addr, _bob_safe_addr) =
        onboard_node(&chain_env, &bob_chain_key, 10_u32.into(), 10_000_u32.into()).await;

    // TODO: instantiate Hopr for both nodes
}
