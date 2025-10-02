use std::{str::FromStr, time::Duration};

use alloy::primitives::U256;
use hopr_lib::{Address, state::HoprState};
use once_cell::sync::Lazy;
use serde_json::json;
use tokio::{
    sync::{Mutex, OnceCell},
    time::sleep,
};
use tracing::info;

use crate::common::{NodeSafeConfig, TestChainEnv, deploy_test_environment, hopr_tester::HoprTester, onboard_node};

/// A guard that holds a reference to the cluster and ensures exclusive access
pub struct ClusterGuard {
    pub cluster: Vec<HoprTester>,
    _lock: tokio::sync::MutexGuard<'static, ()>,
}

impl std::ops::Deref for ClusterGuard {
    type Target = Vec<HoprTester>;

    fn deref(&self) -> &Self::Target {
        &self.cluster
    }
}

static CHAINENV_FIXTURE: Lazy<OnceCell<TestChainEnv>> = Lazy::new(|| OnceCell::const_new());
// static SWARM_N_FIXTURE: Lazy<OnceCell<Vec<HoprTester>>> = Lazy::new(|| OnceCell::const_new());
static CLUSTER_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub const SNAPSHOT_BASE: &str = "/tmp/hopr-tests";
pub const PATH_TO_PROTOCOL_CONFIG: &str = "tests/protocol-config-anvil.json";
pub const SWARM_N: usize = 3;

pub fn exclusive_indexes<const N: usize>() -> [usize; N] {
    use rand::seq::index::sample;
    assert!(N <= SWARM_N, "Requested count exceeds SWARM_N");
    let indices = sample(&mut rand::thread_rng(), SWARM_N, N);
    let mut arr = [0; N];

    for (i, idx) in indices.iter().enumerate() {
        arr[i] = idx;
    }
    arr
}

#[rstest::fixture]
pub async fn chainenv_fixture() -> &'static TestChainEnv {
    use hopr_chain_rpc::client::SnapshotRequestor;

    CHAINENV_FIXTURE
        .get_or_init(|| async {
            env_logger::init();

            match std::process::Command::new("pkill").arg("-f").arg("anvil").output() {
                Ok(_) => {
                    info!("Killed existing anvil instances");
                }
                Err(_) => {
                    info!("No existing anvil instances found");
                }
            };

            // create the all parent folder of SNAPSHOT_BASE
            std::fs::create_dir_all(SNAPSHOT_BASE).expect("failed to create snapshot base directory");

            let requestor_base = SnapshotRequestor::new(format!("{}/snapshot", SNAPSHOT_BASE).as_str())
                .with_ignore_snapshot(!hopr_crypto_random::is_rng_fixed())
                .load(true)
                .await;
            let block_time = Duration::from_secs(1);
            let finality = 2;

            deploy_test_environment(requestor_base, block_time, finality).await
        })
        .await
}

#[rstest::fixture]
pub async fn cluster_fixture(#[future(awt)] chainenv_fixture: &TestChainEnv) -> ClusterGuard {
    use std::fs::read_to_string;

    use hopr_lib::ProtocolsConfig;

    if !(3..=9).contains(&SWARM_N) {
        panic!("SWARM_N must be between 3 and 9");
    }

    // Acquire the mutex lock to ensure exclusive access to the cluster
    let lock = CLUSTER_MUTEX.lock().await;

    // SWARM_N_FIXTURE
    //     .get_or_init(|| async {
    let protocol_config = ProtocolsConfig::from_str(
        &read_to_string(PATH_TO_PROTOCOL_CONFIG).expect("failed to read protocol config file"),
    )
    .expect("failed to parse protocol config");

    // Use the first SWARM_N onchain keypairs from the chainenv fixture
    let onchain_keys = chainenv_fixture.node_chain_keys[0..SWARM_N].to_vec();
    assert!(onchain_keys.len() == SWARM_N);

    // Setup SWARM_N safes
    let mut safes = Vec::with_capacity(SWARM_N);
    if std::path::Path::new("/tmp/hopr-tests/load_addresses").exists() {
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
            let _ = std::fs::write(
                format!("/tmp/hopr-tests/node_{i}/safe_addresses.json"),
                serde_json::to_string_pretty(&safe_addresses).unwrap(),
            );
            let _ = std::fs::write(format!("/tmp/hopr-tests/load_addresses"), "")
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
    // Wait for all nodes to reach the 'Running' state
    futures::future::join_all(
        hopr_instances
            .iter()
            .map(|instance| wait_for_status(instance, &HoprState::Running)),
    )
    .await;
    // Wait for full mesh connectivity
    futures::future::join_all(hopr_instances.iter().map(|instance| wait_for_connectivity(instance))).await;

    ClusterGuard {
        cluster: hopr_instances,
        _lock: lock,
    }
}

async fn wait_for_connectivity(instance: &HoprTester) {
    info!("Waiting for full connectivity");
    loop {
        let peers = instance
            .inner()
            .network_connected_peers()
            .await
            .expect("failed to get connected peers");

        if peers.len() == SWARM_N - 1 {
            break;
        }
        sleep(Duration::from_secs(1)).await;
    }
}

async fn wait_for_status(instance: &HoprTester, expected_status: &HoprState) {
    info!(
        "Waiting for node {} to reach status {:?}",
        instance.address(),
        expected_status
    );
    loop {
        let status = instance.inner().status();
        if &status == expected_status {
            break;
        }
        sleep(Duration::from_secs(1)).await;
    }
}
