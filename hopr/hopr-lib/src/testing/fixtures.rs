use std::{convert::identity, ops::Div, str::FromStr, time::Duration};

use futures_time::future::FutureExt;
use hex_literal::hex;
use hopr_api::chain::DeployedSafe;
use hopr_chain_connector::{
    BlockchainConnectorConfig,
    blokli_client::BlokliQueryClient,
    create_trustful_hopr_blokli_connector,
    testing::{BlokliTestClient, BlokliTestStateBuilder, FullStateEmulator},
};
use hopr_crypto_types::prelude::*;
use hopr_db_node::HoprNodeDb;
use hopr_internal_types::prelude::WinningProbability;
use hopr_network_types::prelude::{IpOrHost, RoutingOptions, SealedHost};
use hopr_primitive_types::prelude::*;
use hopr_transport::{HoprSession, SessionClientConfig, SessionTarget};
use rand::prelude::{IteratorRandom, SliceRandom};
use rstest::fixture;
use tokio::time::sleep;
use tracing::info;

use crate::{
    Address,
    state::HoprState,
    testing::{
        dummies::EchoServer,
        hopr::{ChannelGuard, NodeSafeConfig, TestedHopr, create_hopr_instance},
    },
};

/// A guard that holds a reference to the cluster and ensures exclusive access
pub struct ClusterGuard {
    pub cluster: Vec<TestedHopr>,
    pub chain_client: BlokliTestClient<FullStateEmulator>,
}

impl std::ops::Deref for ClusterGuard {
    type Target = Vec<TestedHopr>;

    fn deref(&self) -> &Self::Target {
        &self.cluster
    }
}

impl ClusterGuard {
    /// Size of the cluster.
    pub fn size(&self) -> usize {
        self.cluster.len()
    }

    /// Get oracle ticket price from the chain
    pub async fn get_oracle_ticket_price(&self) -> anyhow::Result<HoprBalance> {
        Ok(self.chain_client.query_chain_info().await?.ticket_price.0.parse()?)
    }

    /// Update winning probability
    pub async fn update_winning_probability(&self, new_prob: f64) -> anyhow::Result<()> {
        tracing::debug!(new_prob, "updating winning probability");
        Ok(self.chain_client.update_price_and_win_prob(None, Some(new_prob)))
    }

    /// Create a session between two nodes, ensuring channels are open and funded as needed
    pub async fn create_session(
        &self,
        path: &[&TestedHopr],
        funding_amount: HoprBalance,
    ) -> anyhow::Result<(HoprSession, ChannelGuard, ChannelGuard)> {
        let fw_channels = ChannelGuard::try_open_channels_for_path(
            path.iter().map(|n| n.instance.clone()).collect::<Vec<_>>(),
            funding_amount,
        )
        .await?;
        let bw_channels = ChannelGuard::try_open_channels_for_path(
            path.iter().rev().map(|n| n.instance.clone()).collect::<Vec<_>>(),
            funding_amount,
        )
        .await?;

        sleep(Duration::from_secs(1)).await;

        let ip = IpOrHost::from_str(":0")?;
        let routing = RoutingOptions::IntermediatePath(
            path.iter()
                .skip(1)
                .take(path.len() - 2)
                .map(|n| n.address().into())
                .collect(),
        );
        let session_result = path[0]
            .inner()
            .connect_to(
                path[path.len() - 1].address(),
                SessionTarget::UdpStream(SealedHost::Plain(ip)),
                SessionClientConfig {
                    forward_path_options: routing.clone(),
                    return_path_options: routing.invert(),
                    capabilities: Default::default(),
                    pseudonym: None,
                    surb_management: None,
                    always_max_out_surbs: false,
                },
            )
            .timeout(futures_time::time::Duration::from_secs(5))
            .await;

        match session_result {
            Ok(Ok(s)) => Ok((s, fw_channels, bw_channels)),
            Ok(Err(e)) => Err(e.into()),
            Err(_) => Err(anyhow::anyhow!("Session opening timed out after 5s")),
        }
    }

    pub fn sample_nodes<const N: usize>(&self) -> [&TestedHopr; N] {
        assert!(N <= self.size(), "Requested count exceeds {}", self.size());

        let mut res = self.cluster.iter().choose_multiple(&mut rand::thread_rng(), N);

        res.shuffle(&mut rand::thread_rng());

        res.try_into().unwrap()
    }

    pub fn sample_nodes_with_win_prob_1<const N: usize>(&self) -> [&TestedHopr; N] {
        assert!(N <= self.size(), "Requested count {N} exceeds {}", self.size());

        let mut res = self
            .cluster
            .iter()
            .filter(|&n| {
                n.config()
                    .protocol
                    .packet
                    .codec
                    .outgoing_win_prob
                    .is_some_and(|p| p.as_f64() > 0.99)
            })
            .choose_multiple(&mut rand::thread_rng(), N);

        res.shuffle(&mut rand::thread_rng());

        res.try_into()
            .expect(&format!("cannot find {N} nodes with win prob 1.0"))
    }

    pub fn sample_nodes_with_lower_win_prob<const N: usize>(&self) -> [&TestedHopr; N] {
        assert!(N <= self.size(), "Requested count {N} exceeds {}", self.size());

        let mut res = self
            .cluster
            .iter()
            .filter(|&n| {
                n.config()
                    .protocol
                    .packet
                    .codec
                    .outgoing_win_prob
                    .is_some_and(|p| p.as_f64() < 0.99)
            })
            .choose_multiple(&mut rand::thread_rng(), N);

        res.shuffle(&mut rand::thread_rng());

        res.try_into()
            .expect(&format!("cannot find {N} nodes with win prob < 0.99"))
    }

    pub fn sample_nodes_with_win_prob_1_intermediaries<const N: usize>(&self) -> [&TestedHopr; N] {
        assert!(N <= self.size(), "Requested count {N} exceeds {}", self.size());
        assert!(N > 2, "N must be greater than 2 to have intermediaries");

        let win_prob_lower = self.sample_nodes_with_lower_win_prob::<2>();
        let mut res = self
            .cluster
            .iter()
            .filter(|&n| {
                n.config()
                    .protocol
                    .packet
                    .codec
                    .outgoing_win_prob
                    .is_some_and(|p| p.as_f64() > 0.99)
            })
            .choose_multiple(&mut rand::thread_rng(), N - 2);

        res.shuffle(&mut rand::thread_rng());

        res.insert(0, win_prob_lower[0]);
        res.push(win_prob_lower[1]);

        res.try_into().expect("cannot find sufficient number of nodes")
    }

    pub fn sample_nodes_with_lower_win_prob_intermediaries<const N: usize>(&self) -> [&TestedHopr; N] {
        assert!(N <= self.size(), "Requested count {N} exceeds {}", self.size());
        assert!(N > 2, "N must be greater than 2 to have intermediaries");

        let win_prob_1 = self.sample_nodes_with_win_prob_1::<2>();
        let mut res = self
            .cluster
            .iter()
            .filter(|&n| {
                n.config()
                    .protocol
                    .packet
                    .codec
                    .outgoing_win_prob
                    .is_some_and(|p| p.as_f64() < 0.99)
            })
            .choose_multiple(&mut rand::thread_rng(), N - 2);

        res.shuffle(&mut rand::thread_rng());

        res.insert(0, win_prob_1[0]);
        res.push(win_prob_1[1]);

        res.try_into().expect("cannot find sufficient number of nodes")
    }
}

pub const SWARM_N: usize = 9;

pub const TEST_GLOBAL_TIMEOUT: Duration = Duration::from_mins(3);

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

pub const INITIAL_SAFE_NATIVE: u64 = 1;
pub const INITIAL_SAFE_TOKEN: u64 = 1000;
pub const INITIAL_NODE_NATIVE: u64 = 1;
pub const INITIAL_NODE_TOKEN: u64 = 10;
pub const DEFAULT_SAFE_ALLOWANCE: u128 = 1000_000_000_000_u128;
pub const MINIMUM_INCOMING_WIN_PROB: f64 = 0.2;

pub fn build_blokli_client() -> BlokliTestClient<FullStateEmulator> {
    BlokliTestStateBuilder::default()
        .with_balances(
            NODE_CHAIN_KEYS
                .iter()
                .map(|c| (c.public().to_address(), XDaiBalance::new_base(INITIAL_NODE_NATIVE))),
        )
        .with_balances(
            NODE_CHAIN_KEYS
                .iter()
                .map(|c| (c.public().to_address(), HoprBalance::new_base(INITIAL_NODE_TOKEN))),
        )
        .with_balances(
            NODE_SAFES_MODULES
                .iter()
                .map(|&(safe_addr, _)| (safe_addr, XDaiBalance::new_base(INITIAL_SAFE_NATIVE))),
        )
        .with_balances(
            NODE_SAFES_MODULES
                .iter()
                .map(|&(safe_addr, _)| (safe_addr, HoprBalance::new_base(INITIAL_SAFE_TOKEN))),
        )
        .with_safe_allowances(
            NODE_SAFES_MODULES
                .iter()
                .map(|&(safe_addr, _)| (safe_addr, HoprBalance::new_base(DEFAULT_SAFE_ALLOWANCE))),
        )
        .with_deployed_safes(NODE_SAFES_MODULES.iter().zip(NODE_CHAIN_KEYS.iter()).map(
            |((safe_address, module_address), chain_key)| DeployedSafe {
                address: *safe_address,
                owner: chain_key.public().to_address(),
                module: *module_address,
            },
        ))
        .with_minimum_win_prob(WinningProbability::try_from(MINIMUM_INCOMING_WIN_PROB).unwrap())
        .with_ticket_price(HoprBalance::new_base(1))
        .with_closure_grace_period(Duration::from_secs(1))
        .build_dynamic_client(Address::default()) // Placeholder module address, to be filled in later
        .with_tx_simulation_delay(Duration::from_millis(300))
}

#[fixture]
#[once]
pub fn size_2_cluster_fixture() -> ClusterGuard {
    cluster_fixture(2)
}

#[fixture]
#[once]
pub fn size_3_cluster_fixture() -> ClusterGuard {
    cluster_fixture(3)
}

#[fixture]
pub fn cluster_fixture(#[default(3)] size: usize) -> ClusterGuard {
    if !(1..=SWARM_N).contains(&size) {
        panic!("{size} must be between 1 and {SWARM_N}");
    }

    let chain_client = build_blokli_client();

    // Use the first SWARM_N onchain keypairs from the chainenv fixture
    let onchain_keys = NODE_CHAIN_KEYS[0..size].to_vec();
    let offchain_keys = NODE_OFFCHAIN_KEYS[0..size].to_vec();
    let safes = NODE_SAFES_MODULES[0..size]
        .iter()
        .map(|(safe, module)| NodeSafeConfig {
            safe_address: safe.clone(),
            module_address: module.clone(),
        })
        .collect::<Vec<_>>();

    // Setup nodes
    let cluster: Vec<TestedHopr> = (0..size)
        .map(|i| {
            let onchain_keys = onchain_keys.clone();
            let offchain_keys = offchain_keys.clone();
            let safes = safes.clone();

            let blokli_client = chain_client
                .clone()
                .with_mutator(FullStateEmulator::new(safes[i].module_address.clone()));

            std::thread::spawn(move || {
                // This runtime holds all the tasks spawned by the Hopr node.
                // It must live as long as the Hopr node, otherwise the tasks will terminate.
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(
                        std::thread::available_parallelism()
                            .map(|v| v.get())
                            .unwrap_or(1)
                            .div(size)
                            .max(3)
                            - 1,
                    )
                    .thread_stack_size(5 * 1024 * 1024)
                    .thread_name(format!("hopr-node-{i}"))
                    .enable_all()
                    .build()
                    .expect("failed to build Tokio runtime");

                let result = runtime.block_on(async {
                    let node_db = HoprNodeDb::new_in_memory()
                        .await
                        .expect("failed to create HoprNodeDb for node");

                    let mut connector = create_trustful_hopr_blokli_connector(
                        &onchain_keys[i],
                        BlockchainConnectorConfig::default(),
                        blokli_client,
                        safes[i].module_address,
                    )
                    .await
                    .expect("failed to create HoprBlockchainSafeConnector for node");

                    connector
                        .connect()
                        .await
                        .expect("failed to connect to HoprBlockchainSafeConnector");

                    let instance = create_hopr_instance(
                        (&onchain_keys[i], &offchain_keys[i]),
                        3001 + i as u16,
                        node_db,
                        std::sync::Arc::new(connector),
                        safes[i],
                        if i % 2 != 0 { MINIMUM_INCOMING_WIN_PROB } else { 1.0 },
                    )
                    .await;

                    let socket = instance
                        .run(
                            hopr_ct_telemetry::ImmediateNeighborProber::new(Default::default()),
                            EchoServer::new(),
                        )
                        .await?;
                    anyhow::Ok((instance, socket))
                });

                result.map(|(instance, socket)| TestedHopr::new(runtime, instance, socket))
            })
            .join()
            .map_err(|_| anyhow::anyhow!("hopr node starting thread panicked"))
            .and_then(identity)
            .inspect_err(|error| tracing::error!(%error, "hopr node failed to start"))
        })
        .collect::<Result<Vec<_>, _>>()
        .expect("one or more HOPR nodes could not be created");

    let swarm_size = cluster.len();
    let cluster = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("failed to build Tokio runtime in local thread");

        rt.block_on(async {
            // Wait for all nodes to reach the 'Running' state
            futures::future::try_join_all(cluster.iter().map(|instance| {
                wait_for_status(instance, &HoprState::Running).timeout(futures_time::time::Duration::from_secs(180))
            }))
            .await
            .expect("status wait failed");

            // Wait for full mesh connectivity
            futures::future::try_join_all(cluster.iter().map(|instance| {
                wait_for_connectivity(instance, swarm_size).timeout(futures_time::time::Duration::from_secs(120))
            }))
            .await
            .expect("connectivity wait failed");
        });
        cluster
    })
    .join()
    .expect("cluster readiness thread panicked");

    info!(swarm_size, "CLUSTER STARTED");

    ClusterGuard { cluster, chain_client }
}

async fn wait_for_connectivity(instance: &TestedHopr, swarm_size: usize) {
    info!("Waiting for full connectivity");
    loop {
        let peers = instance
            .inner()
            .network_connected_peers()
            .await
            .expect("failed to get connected peers");

        if peers.len() == swarm_size - 1 {
            break;
        }

        tracing::trace!(
            "{} peers connected on {}, waiting for full mesh",
            peers.len(),
            instance.instance.me_onchain()
        );
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

        tracing::trace!(
            "{status} on {}, waiting for {expected_status}",
            instance.instance.me_onchain()
        );
        sleep(Duration::from_secs(1)).await;
    }
}
