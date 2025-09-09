mod common;

use std::sync::Arc;
use std::time::Duration;

use alloy::primitives::U256;
use hopr_chain_rpc::client::SnapshotRequestor;
use hopr_crypto_types::prelude::{Keypair, OffchainKeypair};
use hopr_lib::Hopr;
use hopr_lib::config::HoprLibConfig;

use crate::common::{deploy_test_environment, onboard_node};

const SNAPSHOT_BASE: &str = "tests/snapshots/node_snapshot_base";

#[ignore]
// #[tracing_test::traced_test]
#[cfg(feature = "runtime-tokio")]
#[tokio::test]
async fn hopr_node_integration_test() -> anyhow::Result<()> {
    // use hopr_db_api::info;

    use hopr_lib::config::SafeModule;
    use tracing::{debug, info};

    env_logger::init();

    let block_time = Duration::from_secs(1);
    let finality = 2;

    let requestor_base = SnapshotRequestor::new(SNAPSHOT_BASE)
        .with_ignore_snapshot(!hopr_crypto_random::is_rng_fixed())
        .load(true)
        .await;

    debug!("Loaded snapshot requestor from {}", SNAPSHOT_BASE);

    let chain_env = deploy_test_environment(requestor_base, block_time, finality).await;
    info!("Deployed test environment");

    let alice_chain_key = chain_env.node_chain_keys[0].clone();
    let bob_chain_key = chain_env.node_chain_keys[1].clone();
    info!(
        "Using chain keys: \nAlice: {}\nBob: {}",
        alice_chain_key.public(),
        bob_chain_key.public()
    );

    let _alice_node_safe = onboard_node(&chain_env, &alice_chain_key, U256::from(10_u32), U256::from(10_000_u32)).await;
    let _bob_node_safe = onboard_node(&chain_env, &bob_chain_key, U256::from(10_u32), U256::from(10_000_u32)).await;

    // Instantiate Hopr for both nodes
    let _alice = Arc::new(Hopr::new(
        HoprLibConfig {
            host: hopr_lib::config::HostConfig {
                address: hopr_lib::config::HostType::IPv4("127.0.0.1".into()),
                port: 3001,
            },
            db: hopr_lib::config::Db {
                data: "/tmp/hopr-tests/alice".into(),
                initialize: true,
                force_initialize: false,
            },
            safe_module: SafeModule {
                safe_transaction_service_provider: "".into(),
                safe_address: _alice_node_safe.safe_address,
                module_address: _alice_node_safe.module_address,
            },
            ..Default::default()
        },
        &OffchainKeypair::random(),
        &alice_chain_key,
    )?);
    let _bob = Arc::new(Hopr::new(
        HoprLibConfig {
            host: hopr_lib::config::HostConfig {
                address: hopr_lib::config::HostType::IPv4("127.0.0.1".into()),
                port: 3002,
            },
            db: hopr_lib::config::Db {
                data: "/tmp/hopr-tests/bob".into(),
                initialize: true,
                force_initialize: false,
            },
            safe_module: SafeModule {
                safe_transaction_service_provider: "".into(),
                safe_address: _bob_node_safe.safe_address,
                module_address: _bob_node_safe.module_address,
            },
            ..Default::default()
        },
        &OffchainKeypair::random(),
        &bob_chain_key,
    )?);

    debug!("Created Hopr instances for both nodes");

    // finish the test
    Ok(())
}
