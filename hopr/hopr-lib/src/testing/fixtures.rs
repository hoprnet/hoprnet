use std::time::Duration;

use futures_time::future::FutureExt;
use hex_literal::hex;
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
        ChainKeypair::from_secret(&hex!("76a4edbc3f595d4d07671779a0055e30b2b8477ecfd5d23c37afd7b5aa83781d")).unwrap(),
        ChainKeypair::from_secret(&hex!("c90f09e849aa512be3dd007452977e32c7cfdc1e3de1a62bd92ba6592bcc9e90")).unwrap(),
        ChainKeypair::from_secret(&hex!("40d4749a620d1a4278d030a3153b5b94d6fcd4f9677f6ce8e37e6ebb1987ad53")).unwrap(),
        ChainKeypair::from_secret(&hex!("e539f1ac48270be4e84b6acfe35252df5e141a29b50ddb07b50670271bb574ee")).unwrap(),
        ChainKeypair::from_secret(&hex!("9ab557eb14d8b081c7e1750eb87407d8c421aa79bdeb420f38980829e7dbf936")).unwrap(),
        ChainKeypair::from_secret(&hex!("afba85b6cf1433e22d257f4ae2ef8e74317c4e18482e90e841abb77e3331ad58")).unwrap(),
        ChainKeypair::from_secret(&hex!("4ba798854868f7f77975019ef5a3a89c6518a7ff5b1ac5b39f9ebb619b0f17da")).unwrap(),
        ChainKeypair::from_secret(&hex!("5a4a67919f3b1bd09de351056a9cfd82054092648c4658e36621a46870a44c77")).unwrap(),
        ChainKeypair::from_secret(&hex!("73680d318cca7f0a6280c21daee02cc13ba988b0de9be5d333bbc19003d1f86b")).unwrap(),
    ];
    static ref NODE_OFFCHAIN_KEYS: Vec<OffchainKeypair> = vec![
        OffchainKeypair::from_secret(&hex!("71bf1f42ebbfcd89c3e197a3fd7cda79b92499e509b6fefa0fe44d02821d146a")).unwrap(),
        OffchainKeypair::from_secret(&hex!("c3659450e994f3ad086373440e4e7070629a1bfbd555387237ccb28d17acbfc8")).unwrap(),
        OffchainKeypair::from_secret(&hex!("4a14c5aeb53629a2dd45058a8d233f24dd90192189e8200a1e5f10069868f963")).unwrap(),
        OffchainKeypair::from_secret(&hex!("8c1edcdebfe508031e4124168bb4a133180e8ee68207a7946fcdc4ad0068ef0d")).unwrap(),
        OffchainKeypair::from_secret(&hex!("6075c595103667537c33cdb954e3e5189921cab942e5fc0ba9ec27fe6d7787d1")).unwrap(),
        OffchainKeypair::from_secret(&hex!("ca45a38bed988daadcebd5333abcbfd0dbb2ae5ed1917dbab8cd932970ba6b9b")).unwrap(),
        OffchainKeypair::from_secret(&hex!("7dddac7f5873d416e837a51351cc776b94799c7953ba9ab8d8541825fc342e94")).unwrap(),
        OffchainKeypair::from_secret(&hex!("dd5e0e05aea4c6a6b120e635be806b9c118123ab64f30b4210e9568a1272f617")).unwrap(),
        OffchainKeypair::from_secret(&hex!("7c8fca94af22557421b5e4ee8a4532a77b4ee2ce5c15b410825ffe7b5b60462d")).unwrap(),
    ];
    // (Safe address, Module address)
    static ref NODE_SAFES_MODULES: Vec<(Address, Address)> = vec![
        ("7e641055ee5427572aafb1de11b56f0472f3af47".parse().unwrap(), "cd4d1e4c3a9acf604e6715d14cae64a165a381ec".parse().unwrap()),
        ("e4d759ab6e1c57d5cc0b271c0bf5fa5137bcefd9".parse().unwrap(), "6fb8f33c1ac1a1c56a959680e8a14d918cbb2be7".parse().unwrap()),
        ("1f835eb942f39dfac4c007bea41ce547de404f02".parse().unwrap(), "093364b60d2b4083958d779a6368ad3559985e38".parse().unwrap()),
        ("65fec51266c3e5da55792d1a7a0700a5b246efe8".parse().unwrap(), "bbf4d38bfb0c80641a57937c44fcc42aa01c77bd".parse().unwrap()),
        ("5af6633066297f257f0438f3d7a8c411ff7c823d".parse().unwrap(), "fa3a4eec4ea7c8404e2cd686a5842a0040fe5c67".parse().unwrap()),
        ("5e99bb000c2a615e98ee5e9c1128e14b563ff497".parse().unwrap(), "0e536c87591767655f842025b5d5e2d178aade92".parse().unwrap()),
        ("70416f2ad90b7773919bd3a822c7bb7d92b42b2f".parse().unwrap(), "3431fcd4ad1ed1ff4906069bb34279a3fd8145bc".parse().unwrap()),
        ("a3d811f7efe65fcd10b7b97ce9bf85429ef657f1".parse().unwrap(), "0ad7675c28f93a161e4b2815326af7f0e866a14e".parse().unwrap()),
        ("5eb2888c6184d9bea2d7a3ab478845b9aa5c812b".parse().unwrap(), "d8ede85de102862e2311d23263342730db18a770".parse().unwrap()),
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
