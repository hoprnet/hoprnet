use std::time::Duration;

use futures_time::future::FutureExt;
use hopr_chain_connector::{
    BlockchainConnectorConfig, HoprBlockchainSafeConnector, create_trustful_hopr_blokli_connector,
    testing::{BlokliTestClient, BlokliTestStateBuilder, FullStateEmulator},
};
use hopr_crypto_types::{
    keypairs::{ChainKeypair, OffchainKeypair},
    prelude::Keypair,
};
use hopr_db_node::HoprNodeDb;
use hopr_internal_types::prelude::WinningProbability;
use hopr_primitive_types::{
    balance::XDaiBalance,
    prelude::{BytesRepresentable, HoprBalance},
};
use lazy_static::lazy_static;
use tokio::{sync::Mutex, time::sleep};
use tracing::info;

use crate::{
    Address,
    state::HoprState,
    testing::{
        dummies::EchoServer,
        hopr::{NodeSafeConfig, TestedHopr},
    },
};

type TestingConnector = std::sync::Arc<HoprBlockchainSafeConnector<BlokliTestClient<FullStateEmulator>>>;

/// A guard that holds a reference to the cluster and ensures exclusive access
pub struct ClusterGuard {
    pub cluster: Vec<TestedHopr<TestingConnector, HoprNodeDb>>,
    _lock: tokio::sync::MutexGuard<'static, ()>,
}

impl std::ops::Deref for ClusterGuard {
    type Target = Vec<TestedHopr<TestingConnector, HoprNodeDb>>;

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

lazy_static::lazy_static! {
    static ref NODE_CHAIN_KEYS: Vec<ChainKeypair> = vec![

    ];
    static ref NODE_SAFES_MODULES: Vec<(Address, Address)> = vec![

    ];
    static ref NODE_OFFCHAIN_KEYS: Vec<OffchainKeypair> = vec![

    ];
}

#[rstest::fixture]
pub async fn chainenv_fixture() -> BlokliTestClient<FullStateEmulator> {
    BlokliTestStateBuilder::default()
        .with_balances(
            NODE_CHAIN_KEYS
                .iter()
                .map(|c| (c.public().to_address(), XDaiBalance::new_base(1))),
        )
        .with_balances(
            NODE_SAFES_MODULES
                .iter()
                .map(|(safe_addr, _)| (*safe_addr, HoprBalance::new_base(1000))),
        )
        .with_safe_allowances(
            NODE_SAFES_MODULES
                .iter()
                .map(|(safe_addr, _)| (*safe_addr, HoprBalance::new_base(1000_000_000_000_u128))),
        )
        .with_minimum_win_prob(WinningProbability::ALWAYS)
        .with_ticket_price(HoprBalance::new_base(1))
        .build_dynamic_client([0u8; Address::SIZE].into()) // Placeholder module address, to be filled in later
}

#[rstest::fixture]
pub async fn cluster_fixture(#[future(awt)] chainenv_fixture: BlokliTestClient<FullStateEmulator>) -> ClusterGuard {
    if !(3..=9).contains(&SWARM_N) {
        panic!("SWARM_N must be between 3 and 9");
    }

    // Acquire the mutex lock to ensure exclusive access to the cluster
    let lock = CLUSTER_MUTEX.lock().await;

    // Use the first SWARM_N onchain keypairs from the chainenv fixture
    let onchain_keys = NODE_CHAIN_KEYS[0..SWARM_N].to_vec();
    let offchain_keys = NODE_OFFCHAIN_KEYS[0..SWARM_N].to_vec();
    let safes = NODE_SAFES_MODULES[0..SWARM_N]
        .iter()
        .map(|(safe, module)| NodeSafeConfig {
            safe_address: safe.clone(),
            module_address: module.clone(),
        })
        .collect::<Vec<_>>();

    // Setup SWARM_N nodes
    let hopr_instances: Vec<TestedHopr<TestingConnector, HoprNodeDb>> =
        futures::future::join_all((0..SWARM_N).map(|i| {
            let onchain_keys = onchain_keys.clone();
            let offchain_keys = offchain_keys.clone();
            let safes = safes.clone();
            let blokli_client = chainenv_fixture
                .clone()
                .with_mutator(FullStateEmulator::new(safes[i].module_address.clone()));

            async move {
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build()
                        .expect("failed to build Tokio runtime");

                    rt.block_on(async {
                        let node_db = HoprNodeDb::new_in_memory(onchain_keys[i].clone())
                            .await
                            .expect("failed to create HoprNodeDb for node");
                        let connector = create_trustful_hopr_blokli_connector(
                            &onchain_keys[i],
                            BlockchainConnectorConfig::default(),
                            blokli_client,
                            safes[i].module_address,
                        )
                        .await
                        .expect("failed to create HoprBlockchainSafeConnector for node");

                        TestedHopr::new(
                            onchain_keys[i].clone(),
                            offchain_keys[i].clone(),
                            3001 + i as u16,
                            node_db,
                            std::sync::Arc::new(connector),
                            safes[i],
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

    // Run all nodes in parallel
    futures::future::join_all(
        hopr_instances
            .iter()
            .map(|instance| instance.inner().run(EchoServer::new())),
    )
    .await;
    // Wait for all nodes to reach the 'Running' state
    let res = futures::future::join_all(hopr_instances.iter().map(|instance| {
        wait_for_status(instance, &HoprState::Running).timeout(futures_time::time::Duration::from_secs(120))
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
        _lock: lock,
    }
}

async fn wait_for_connectivity(instance: &TestedHopr<TestingConnector, HoprNodeDb>) {
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

async fn wait_for_status(instance: &TestedHopr<TestingConnector, HoprNodeDb>, expected_status: &HoprState) {
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
