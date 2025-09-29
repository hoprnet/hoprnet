use crate::common::{NodeSafeConfig, TestChainEnv, deploy_test_environment, hopr_tester::HoprTester, onboard_node};

use alloy::primitives::U256;

use hopr_lib::Address;
use once_cell::sync::Lazy;
use serde_json::json;
use std::{str::FromStr, time::Duration};
use tokio::sync::OnceCell;

static CHAINENV_FIXTURE: Lazy<OnceCell<TestChainEnv>> = Lazy::new(|| OnceCell::const_new());
static SWARM_N_FIXTURE: Lazy<OnceCell<Vec<HoprTester>>> = Lazy::new(|| OnceCell::const_new());

pub const SNAPSHOT_BASE: &str = "/tmp/hopr-tests/snapshots";
pub const PATH_TO_PROTOCOL_CONFIG: &str = "tests/protocol-config-anvil.json";
pub const SWARM_N: usize = 3;
pub const LOAD_FROM_FILES: bool = false; // TODO: need to port this to auto-detection

#[rstest::fixture]
pub fn random_int() -> usize {
    use rand::prelude::SliceRandom;

    let mut numbers: Vec<usize> = (0..SWARM_N).collect();
    numbers.shuffle(&mut rand::thread_rng());
    numbers[0]
}

#[rstest::fixture]
pub fn random_int_pair() -> (usize, usize) {
    use rand::prelude::SliceRandom;

    let mut numbers: Vec<usize> = (0..SWARM_N).collect();
    numbers.shuffle(&mut rand::thread_rng());
    let [a, b, ..] = numbers[..] else {
        panic!("Not enough numbers for pair")
    };
    (a, b)
}

#[rstest::fixture]
pub fn random_int_triple() -> (usize, usize, usize) {
    use rand::prelude::SliceRandom;

    let mut numbers: Vec<usize> = (0..SWARM_N).collect();
    numbers.shuffle(&mut rand::thread_rng());
    let [a, b, c, ..] = numbers[..] else {
        panic!("Not enough numbers for triple")
    };
    (a, b, c)
}

#[rstest::fixture]
pub async fn chainenv_fixture() -> &'static TestChainEnv {
    use hopr_chain_rpc::client::SnapshotRequestor;

    CHAINENV_FIXTURE
        .get_or_init(|| async {
            env_logger::init();

            let requestor_base = SnapshotRequestor::new(SNAPSHOT_BASE)
                .with_ignore_snapshot(!hopr_crypto_random::is_rng_fixed())
                .load(false)
                .await;
            let block_time = Duration::from_secs(1);
            let finality = 2;

            deploy_test_environment(requestor_base, block_time, finality).await
        })
        .await
}

#[rstest::fixture]
pub async fn cluster_fixture(#[future(awt)] chainenv_fixture: &TestChainEnv) -> &'static Vec<HoprTester> {
    use hopr_lib::ProtocolsConfig;
    use std::fs::read_to_string;
    if !(2..=9).contains(&SWARM_N) {
        panic!("SWARM_N must be between 2 and 9");
    }

    SWARM_N_FIXTURE
        .get_or_init(|| async {
            let protocol_config = ProtocolsConfig::from_str(
                &read_to_string(PATH_TO_PROTOCOL_CONFIG).expect("failed to read protocol config file"),
            )
            .expect("failed to parse protocol config");

            // Use the first SWARM_N onchain keypairs from the chainenv fixture
            let onchain_keys = chainenv_fixture.node_chain_keys[0..SWARM_N].to_vec();
            assert!(onchain_keys.len() == SWARM_N);

            // Setup SWARM_N safes
            let mut safes = Vec::with_capacity(SWARM_N);
            if LOAD_FROM_FILES {
                // read safe address and module from file /tmp/hopr-tests/node_i/safe_addresses.json
                for i in 0..SWARM_N {
                    let addresses: serde_json::Value = serde_json::from_str(
                        &std::fs::read_to_string(format!("/tmp/hopr-tests/node_{i}/safe_addresses.json"))
                            .expect("failed to read safe addresses from file"),
                    )
                    .expect("failed to parse safe addresses from file");
                    let safe_address = Address::from_str(
                        addresses
                            .get("safe")
                            .and_then(|v| v.as_str())
                            .expect("missing safe address"),
                    )
                    .expect("invalid safe address");

                    let module_address = Address::from_str(
                        addresses
                            .get("module")
                            .and_then(|v| v.as_str())
                            .expect("missing module address"),
                    )
                    .expect("invalid module address");

                    let safe = NodeSafeConfig {
                        safe_address: safe_address,
                        module_address: module_address,
                    };

                    safes.push(safe);
                }
            } else {
                for i in 0..SWARM_N {
                    let safe = onboard_node(
                        &chainenv_fixture,
                        &onchain_keys[i],
                        U256::from(1_000_000_000_000_000_000_u128),
                        U256::from(10_000_000_000_000_000_000_u128),
                    )
                    .await;
                    let safe_addresses = json!({
                        "safe": safe.safe_address.to_string(),
                        "module": safe.module_address.to_string(),
                    });
                    std::fs::create_dir_all(format!("/tmp/hopr-tests/node_{i}")).expect("failed to create directory");
                    std::fs::write(
                        format!("/tmp/hopr-tests/node_{i}/safe_addresses.json"),
                        serde_json::to_string_pretty(&safe_addresses).unwrap(),
                    )
                    .expect("failed to write safe addresses to file");
                    safes.push(safe);
                }
            }

            assert!(safes.len() == SWARM_N);

            // Setup SWARM_N nodes
            let hopr_instances: Vec<HoprTester> = futures::future::join_all((0..SWARM_N).map(|i| {
                let moved_keys = onchain_keys.clone();
                let moved_safes = safes.clone();
                let moved_config = protocol_config.clone();
                let endpoint = chainenv_fixture.anvil.endpoint().to_string();

                async move {
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Builder::new_multi_thread()
                            .worker_threads(2)
                            .enable_all()
                            .build()
                            .expect("Failed to build Tokio runtime");

                        rt.block_on(async {
                            HoprTester::new(
                                moved_keys[i].clone(),
                                endpoint.clone(),
                                moved_config,
                                3001 + i as u16,
                                format!("/tmp/hopr-tests/node_{i}"),
                                moved_safes[i].clone(),
                            )
                        })
                    })
                    .join()
                    .expect("Thread panicked")
                }
            }))
            .await;

            // Run all nodes in parallel
            futures::future::join_all(hopr_instances.iter().map(|instance| instance.run())).await;

            hopr_instances
        })
        .await
}
