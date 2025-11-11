use std::{str::FromStr, time::Duration};

use alloy::providers::ext::AnvilApi;
use futures_time::future::FutureExt as _;
use hopr_api::chain::HoprBalance;
use hopr_primitive_types::{bounded::BoundedVec, traits::IntoEndian};
use hopr_transport::{
    HoprSession, RoutingOptions, SessionClientConfig, SessionTarget,
    session::{IpOrHost, SealedHost},
};
use lazy_static::lazy_static;
use rand::seq::index::sample;
use serde_json::json;
use tokio::{sync::Mutex, time::sleep};
use tracing::info;

use crate::{
    Address, ProtocolsConfig,
    state::HoprState,
    testing::{
        chain::{NodeSafeConfig, TestChainEnv, TestChainEnvConfig, deploy_test_environment, onboard_node},
        dummies::EchoServer,
        hopr::{ChannelGuard, TestedHopr},
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

impl ClusterGuard {
    /// Get oracle ticket price from chain
    pub async fn get_oracle_ticket_price(&self) -> anyhow::Result<HoprBalance> {
        if let Some(instances) = &self.chain_env.contract_instances {
            let price: HoprBalance = instances.price_oracle.currentTicketPrice().call().await.map(|v| {
                HoprBalance::from(hopr_primitive_types::prelude::U256::from_be_bytes(
                    v.to_be_bytes::<32>(),
                ))
            })?;

            Ok(price)
        } else {
            info!("Contract instances not available, cannot get oracle ticket price");
            Err(anyhow::anyhow!("Contract instances not available"))
        }
    }

    /// Update winning probability in anvil
    pub async fn update_winning_probability(&self, new_prob: f64) -> anyhow::Result<()> {
        let epsilon: f64 = 0.000001;

        if let Some(instances) = &self.chain_env.contract_instances {
            match instances.update_winning_probability(new_prob).await {
                Ok(_) => {
                    sleep(Duration::from_secs(5)).await;

                    let [node] = exclusive_indexes::<1>();

                    let winning_prob = self.cluster[node]
                        .inner()
                        .get_minimum_incoming_ticket_win_probability()
                        .await?;

                    if (winning_prob.as_f64() - new_prob).abs() < epsilon {
                        Ok(())
                    } else {
                        Err(anyhow::anyhow!("Winning probability not reflected in the node"))
                    }
                }
                Err(e) => Err(anyhow::anyhow!("Failed to update winning probability: {}", e)),
            }
        } else {
            info!("Contract instances not available, cannot get current winning probability");
            Err(anyhow::anyhow!("Contract instances not available"))
        }
    }

    /// Create a session between two nodes, ensuring channels are open and funded as needed
    pub async fn create_session_between(
        &self,
        src: usize,
        mid: usize,
        dst: usize,
        funding_amount: HoprBalance,
    ) -> anyhow::Result<(HoprSession, ChannelGuard, ChannelGuard)> {
        let fw_channels = ChannelGuard::try_open_channels_for_path(
            vec![
                self.cluster[src].instance.clone(),
                self.cluster[mid].instance.clone(),
                self.cluster[dst].instance.clone(),
            ],
            funding_amount,
        )
        .await?;
        let bw_channels = ChannelGuard::try_open_channels_for_path(
            vec![
                self.cluster[dst].instance.clone(),
                self.cluster[mid].instance.clone(),
                self.cluster[src].instance.clone(),
            ],
            funding_amount,
        )
        .await?;

        sleep(std::time::Duration::from_secs(3)).await;

        let ip = IpOrHost::from_str(":0")?;
        let routing = RoutingOptions::IntermediatePath(BoundedVec::from_iter(std::iter::once(
            self.cluster[mid].address().into(),
        )));

        let session = self.cluster[src]
            .inner()
            .connect_to(
                self.cluster[dst].address(),
                SessionTarget::UdpStream(SealedHost::Plain(ip)),
                SessionClientConfig {
                    forward_path_options: routing.clone(),
                    return_path_options: routing,
                    capabilities: Default::default(),
                    pseudonym: None,
                    surb_management: None,
                    always_max_out_surbs: false,
                },
            )
            .await?;

        Ok((session, fw_channels, bw_channels))
    }
}

lazy_static! {
    static ref CLUSTER_MUTEX: Mutex<()> = Mutex::new(());
}

pub const SNAPSHOT_BASE: &str = "/tmp/hopr-tests";
pub const PATH_TO_PROTOCOL_CONFIG: &str = "tests/protocol-config-anvil.json";
pub const SWARM_N: usize = 5;

pub fn exclusive_indexes<const N: usize>() -> [usize; N] {
    assert!(N <= SWARM_N, "Requested count exceeds SWARM_N");
    let indices = sample(&mut rand::thread_rng(), SWARM_N, N);
    let mut arr = [0; N];

    for (i, idx) in indices.iter().enumerate() {
        arr[i] = idx;
    }
    arr
}

pub fn exclusive_indexes_not_auto_redeeming<const N: usize>() -> [usize; N] {
    assert!(N <= SWARM_N, "Requested count exceeds SWARM_N");
    assert!(N <= (SWARM_N + 1) / 2, "Not enough non-auto-redeeming nodes");

    let not_auto_redeem_indices_candidates: Vec<usize> = (0..SWARM_N).filter(|i| i % 2 == 0).collect();
    let selected_indices = sample(&mut rand::thread_rng(), not_auto_redeem_indices_candidates.len(), N);
    let mut arr = [0; N];

    for (i, idx) in selected_indices.iter().enumerate() {
        arr[i] = not_auto_redeem_indices_candidates[idx];
    }

    arr
}

/// Select N unique indexes, ensuring all intermediates indexes (not source and destination) are nodes with auto redeem
/// enabled
pub fn exclusive_indexes_with_auto_redeem_intermediaries<const N: usize>() -> [usize; N] {
    assert!(N <= SWARM_N, "Requested count exceeds SWARM_N");
    assert!(N > 2, "N must be greater than 2 to have intermediaries");

    let auto_redeem_indices_candidates: Vec<usize> = (0..SWARM_N).filter(|i| i % 2 != 0).collect();
    let not_auto_redeem_indices_candidates: Vec<usize> = (0..SWARM_N).filter(|i| i % 2 == 0).collect();

    let auto_redeeming_indices = sample(&mut rand::thread_rng(), auto_redeem_indices_candidates.len(), N - 2);
    let non_auto_redeeming_index = sample(&mut rand::thread_rng(), not_auto_redeem_indices_candidates.len(), 2);
    let mut arr = [0; N];

    // Select source and destination from non-auto-redeem nodes
    let non_auto_redeeming = non_auto_redeeming_index.iter().collect::<Vec<_>>();
    arr[0] = not_auto_redeem_indices_candidates[non_auto_redeeming[0]];
    arr[N - 1] = not_auto_redeem_indices_candidates[non_auto_redeeming[1]];

    // Select intermediaries from auto-redeem nodes
    for (i, idx) in auto_redeeming_indices.iter().enumerate() {
        arr[i + 1] = auto_redeem_indices_candidates[idx];
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
    let protocol_config = ProtocolsConfig::from_str(
        &std::fs::read_to_string(PATH_TO_PROTOCOL_CONFIG).expect("failed to read protocol config file"),
    )
    .expect("failed to parse protocol config");
    let res = deploy_test_environment(TestChainEnvConfig {
        from_file: Some(load_file.into()),
        network: Some(protocol_config.networks["anvil-localhost"].clone()),
        ..Default::default()
    })
    .await;

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
                alloy::primitives::U256::from(1_000_000_000_000_000_000_u128),
                alloy::primitives::U256::from(10_000_000_000_000_000_000_u128),
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
        let do_auto_redeem = i % 2 != 0; // every other node does auto redeem and uses a custom winn_prob

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
                        do_auto_redeem,
                        if do_auto_redeem { Some(0.2) } else { None },
                    )
                    .await
                })
            })
            .join()
        }
    }))
    .await
    .into_iter()
    .collect::<Result<Vec<_>, _>>()
    .expect("One or more threads panicked");

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
    let res = futures::future::join_all(hopr_instances.iter().map(|instance| {
        wait_for_status(instance, &HoprState::Running).timeout(futures_time::time::Duration::from_secs(180))
    }))
    .await;

    for r in res {
        r.expect("status wait failed");
    }

    // Wait for full mesh connectivity
    let res = futures::future::join_all(
        hopr_instances
            .iter()
            .map(|instance| wait_for_connectivity(instance).timeout(futures_time::time::Duration::from_secs(120))),
    )
    .await;

    for r in res {
        r.expect("connectivity wait failed");
    }

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
