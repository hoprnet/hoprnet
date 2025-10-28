use std::{str::FromStr, time::Duration};

use alloy::{primitives::U256, providers::ext::AnvilApi};
use futures_time::future::FutureExt as _;
use lazy_static::lazy_static;
use serde_json::json;
use tokio::{sync::Mutex, time::sleep};
use tracing::info;

use crate::{
    Address, ProtocolsConfig,
    state::HoprState,
    testing::{
        chain::{NodeSafeConfig, TestChainEnv, deploy_test_environment, onboard_node},
        dummies::EchoServer,
        hopr::TestedHopr,
    },
};

/// A guard that holds a reference to the cluster and ensures exclusive access
pub struct ClusterGuard {
    pub cluster: Vec<TestedHopr>,
    #[allow(dead_code)]
    pub chain_env: TestChainEnv, // the object lives to hold the final reference to the anvil provider
    _lock: tokio::sync::MutexGuard<'static, ()>,
}

impl std::ops::Deref for ClusterGuard {
    type Target = Vec<TestedHopr>;

    fn deref(&self) -> &Self::Target {
        &self.cluster
    }
}

lazy_static! {
    static ref CLUSTER_MUTEX: Mutex<()> = Mutex::new(());
}

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
pub async fn chainenv_fixture() -> TestChainEnv {
    // create the all parent folder of SNAPSHOT_BASE
    std::fs::create_dir_all(SNAPSHOT_BASE).expect("failed to create snapshot base directory");

    if std::process::Command::new("pkill")
        .arg("-f")
        .arg("anvil")
        .output()
        .is_ok()
    {
        info!("Terminating existing anvil instances");
    } else {
        info!("No existing anvil instances found");
    }

    let load_file = format!("{SNAPSHOT_BASE}/anvil");
    let res = deploy_test_environment(Duration::from_secs(1), 2, None, Some(load_file.as_str())).await;
    match res {
        Ok(env) => env,
        Err(e) => {
            panic!("Failed to deploy test environment: {e}");
        }
    }
}

#[rstest::fixture]
pub async fn cluster_fixture(#[future(awt)] chainenv_fixture: TestChainEnv) -> ClusterGuard {
    if !(3..=9).contains(&SWARM_N) {
        panic!("SWARM_N must be between 3 and 9");
    }

    // Acquire the mutex lock to ensure exclusive access to the cluster
    let lock = CLUSTER_MUTEX.lock().await;

    // Load or not load from snapshot
    let load_state = std::path::Path::new(&format!("{SNAPSHOT_BASE}/anvil")).exists();

    // SWARM_N_FIXTURE
    let protocol_config = ProtocolsConfig::from_str(
        &std::fs::read_to_string(PATH_TO_PROTOCOL_CONFIG).expect("failed to read protocol config file"),
    )
    .expect("failed to parse protocol config");

    // Use the first SWARM_N onchain keypairs from the chainenv fixture
    let onchain_keys = chainenv_fixture.node_chain_keys[0..SWARM_N].to_vec();
    assert!(onchain_keys.len() == SWARM_N);

    // Setup SWARM_N safes
    let mut safes = Vec::with_capacity(SWARM_N);
    if load_state {
        // read safe address and module from file {SNAPSHOT_BASE}/node_i/safe_addresses.json
        for i in 0..SWARM_N {
            let addresses: serde_json::Value = serde_json::from_str(
                &std::fs::read_to_string(format!("{SNAPSHOT_BASE}/node_{i}/safe_addresses.json"))
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

            // remove folders under SNAPSHOT_BASE/node_i
            std::fs::remove_dir_all(format!("{SNAPSHOT_BASE}/node_{i}/index_db")).ok();
            std::fs::remove_dir_all(format!("{SNAPSHOT_BASE}/node_{i}/node_db")).ok();
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
            std::fs::create_dir_all(format!("{SNAPSHOT_BASE}/node_{i}")).expect("failed to create directory");
            let _ = std::fs::write(
                format!("{SNAPSHOT_BASE}/node_{i}/safe_addresses.json"),
                serde_json::to_string_pretty(&safe_addresses).unwrap(),
            );

            safes.push(safe);
        }
    }

    assert!(safes.len() == SWARM_N);

    // Setup SWARM_N nodes
    let hopr_instances: Vec<TestedHopr> = futures::future::join_all((0..SWARM_N).map(|i| {
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
                    TestedHopr::new(
                        moved_keys[i].clone(),
                        endpoint.clone(),
                        moved_config,
                        3001 + i as u16,
                        format!("{SNAPSHOT_BASE}/node_{i}"),
                        moved_safes[i].clone(),
                    )
                })
            })
            .join()
            .expect("Thread panicked")
        }
    }))
    .await;

    let dump_state = !std::path::Path::new(&format!("{SNAPSHOT_BASE}/anvil")).exists();
    if dump_state {
        let state = chainenv_fixture
            .provider
            .anvil_dump_state()
            .await
            .expect("failed to dump anvil state");

        std::fs::write(format!("{SNAPSHOT_BASE}/anvil"), state.as_ref()).expect("failed to write anvil state to file");
    }

    // Run all nodes in parallel
    futures::future::join_all(
        hopr_instances
            .iter()
            .map(|instance| instance.inner().run(EchoServer::new())),
    )
    .await;
    // Wait for all nodes to reach the 'Running' state
    futures::future::join_all(hopr_instances.iter().map(|instance| {
        wait_for_status(instance, &HoprState::Running).timeout(futures_time::time::Duration::from_secs(120))
    }))
    .await;

    // Wait for full mesh connectivity
    futures::future::join_all(
        hopr_instances
            .iter()
            .map(|instance| wait_for_connectivity(instance).timeout(futures_time::time::Duration::from_secs(120))),
    )
    .await;

    ClusterGuard {
        cluster: hopr_instances,
        chain_env: chainenv_fixture,
        _lock: lock,
    }
}

async fn wait_for_connectivity(instance: &TestedHopr) {
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

async fn wait_for_status(instance: &TestedHopr, expected_status: &HoprState) {
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
