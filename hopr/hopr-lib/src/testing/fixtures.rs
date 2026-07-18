use std::{convert::identity, ops::Div, str::FromStr, sync::Arc, time::Duration};

use futures::StreamExt as _;
use futures_time::future::FutureExt as _;
use hex_literal::hex;
use hopr_chain_connector::{
    BlockchainConnectorConfig,
    api::DeployedSafe,
    blokli_client::BlokliQueryClient,
    create_trustful_hopr_blokli_connector,
    testing::{BlokliTestClient, BlokliTestStateBuilder, FullStateEmulator},
};
use rand::seq::{IteratorRandom, SliceRandom};
use rstest::fixture;
use tokio::time::sleep;
use tracing::info;

#[cfg(feature = "explicit-path")]
#[allow(deprecated)]
use crate::HoprSessionClientExplicitPathConfig;
use crate::{
    HopRouting, HoprSessionClientConfig,
    api::{
        network::NetworkView,
        node::{HasNetworkView, HoprNodeOperations, HoprSessionClientOperations, HoprState},
        types::{
            crypto::{
                keypairs::Keypair,
                prelude::{ChainKeypair, OffchainKeypair},
            },
            internal::prelude::{ChannelEntry, ChannelStatus, WinningProbability},
            primitive::prelude::{Address, HoprBalance, XDaiBalance},
        },
    },
    exports::{
        network::types::prelude::{IpOrHost, SealedHost},
        transport::{HoprSession, SessionTarget},
    },
    testing::{
        TestingConnector,
        dummies::EchoServer,
        hopr::{ChannelGuard, NodeSafeConfig, TestedHopr, create_hopr_instance_config},
    },
};

/// Estimated time for on-chain state to propagate across all nodes.
///
/// In the Blokli test mock, `expected_block_time` is a nominal value (e.g. 5s)
/// that doesn't reflect actual mock processing speed (controlled by `tx_simulation_delay`).
/// We use `expected_block_time + 2s` as a reasonable upper bound that accounts for
/// coverage instrumentation overhead without being tied to the unrealistic
/// `finality * expected_block_time` product.
pub fn chain_propagation_delay(chain_info: &hopr_chain_connector::blokli_client::types::ChainInfo) -> Duration {
    let block_time_secs = chain_info.expected_block_time.0.parse::<u64>().unwrap_or(5);
    Duration::from_secs(block_time_secs + 2)
}

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
        self.chain_client.update_price_and_win_prob(None, Some(new_prob));
        Ok(())
    }

    /// Open channels for a list of paths.
    ///
    /// Each entry in `paths` is a slice of nodes defining a channel path.
    /// Returns a `ChannelGuard` per path, in the same order.
    pub async fn open_channels(
        &self,
        paths: &[&[&TestedHopr]],
        funding_amount: HoprBalance,
    ) -> anyhow::Result<Vec<ChannelGuard>> {
        let mut guards = Vec::with_capacity(paths.len());
        for path in paths {
            guards.push(
                ChannelGuard::try_open_channels_for_path(
                    path.iter().map(|n| n.instance.clone()).collect::<Vec<_>>(),
                    funding_amount,
                )
                .await?,
            );
        }

        let chain_info = self.chain_client.query_chain_info().await?;
        sleep(chain_propagation_delay(&chain_info)).await;

        Ok(guards)
    }

    /// Polls the network graph on `observer` until it sees at least `expected_channels`
    /// edges with non-zero balance, or until `timeout` expires.
    ///
    /// This replaces fixed-duration sleeps after channel opening: instead of guessing
    /// how long chain propagation takes, we actively check the graph state.
    pub async fn wait_for_channel_graph(
        &self,
        observer: &TestedHopr,
        expected_channels: usize,
        timeout: Duration,
    ) -> anyhow::Result<()> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let channels: Vec<ChannelEntry> = {
                use crate::api::{chain::ChainReadChannelOperations, node::HasChainApi};
                match observer
                    .inner()
                    .chain_api()
                    .stream_channels(crate::api::chain::ChannelSelector::default())
                {
                    Ok(stream) => stream.collect().await,
                    Err(_) => vec![],
                }
            };

            let open_count = channels.iter().filter(|c| c.status == ChannelStatus::Open).count();

            if open_count >= expected_channels {
                tracing::info!(open_count, expected_channels, "channel graph converged");
                return Ok(());
            }

            if tokio::time::Instant::now() >= deadline {
                anyhow::bail!(
                    "channel graph did not converge: {open_count}/{expected_channels} open channels after {timeout:?}"
                );
            }

            tracing::trace!(open_count, expected_channels, "waiting for channel graph convergence");
            sleep(Duration::from_millis(500)).await;
        }
    }

    /// Create a session between the first and last nodes in the path.
    ///
    /// Channels must already be open before calling this method.
    pub async fn create_session(&self, path: &[&TestedHopr]) -> anyhow::Result<HoprSession> {
        debug_assert!(path.len() >= 2, "path must contain at least source and destination");

        let chain_info = self.chain_client.query_chain_info().await?;
        // Session establishment retries internally with ~20s per attempt.
        // Use 9x propagation delay (~63s) to allow at least 3 retry cycles,
        // which is needed under coverage instrumentation overhead and when
        // smaller clusters start faster but still need warmup time.
        let timeout = chain_propagation_delay(&chain_info) * 9;

        let ip = IpOrHost::from_str(":0")?;
        let routing = HopRouting::try_from(path.len() - 2)?;
        let session_result = path[0]
            .inner()
            .connect_to(
                path[path.len() - 1].address(),
                SessionTarget::UdpStream(SealedHost::Plain(ip)),
                HoprSessionClientConfig {
                    forward_path: routing,
                    return_path: routing,
                    capabilities: Default::default(),
                    pseudonym: None,
                    surb_management: None,
                    always_max_out_surbs: false,
                    pix_ssa_quota: None,
                },
            )
            .timeout(futures_time::time::Duration::from(timeout))
            .await;

        match session_result {
            Ok(Ok((session, _configurator))) => Ok(session),
            Ok(Err(e)) => Err(e.into()),
            Err(_) => Err(anyhow::anyhow!("Session opening timed out after {timeout:?}")),
        }
    }

    /// Create a session between the first and last nodes in the path using explicit intermediate nodes.
    ///
    /// Channels must already be open before calling this method.
    #[cfg(feature = "explicit-path")]
    #[allow(deprecated)]
    pub async fn create_session_with_explicit_path(&self, path: &[&TestedHopr]) -> anyhow::Result<HoprSession> {
        debug_assert!(path.len() >= 2, "path must contain at least source and destination");

        let chain_info = self.chain_client.query_chain_info().await?;
        let timeout = chain_propagation_delay(&chain_info) * 9;

        let ip = IpOrHost::from_str(":0")?;
        let forward_path = path[1..path.len() - 1]
            .iter()
            .map(|node| {
                crate::peer_id_to_offchain_key(&node.peer_id())
                    .map(crate::api::types::internal::NodeId::from)
                    .map_err(anyhow::Error::from)
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        let mut return_path = forward_path.clone();
        return_path.reverse();

        let session_result = path[0]
            .inner()
            .connect_to_using_explicit_path(
                path[path.len() - 1].address(),
                SessionTarget::UdpStream(SealedHost::Plain(ip)),
                HoprSessionClientExplicitPathConfig {
                    forward_path,
                    return_path,
                    capabilities: Default::default(),
                    pseudonym: None,
                    surb_management: None,
                    always_max_out_surbs: false,
                    pix_ssa_quota: None,
                },
            )
            .timeout(futures_time::time::Duration::from(timeout))
            .await;

        match session_result {
            Ok(Ok((session, _configurator))) => Ok(session),
            Ok(Err(e)) => Err(e.into()),
            Err(_) => Err(anyhow::anyhow!("Session opening timed out after {timeout:?}")),
        }
    }

    pub fn sample_nodes<const N: usize>(&self) -> [&TestedHopr; N] {
        assert!(N <= self.size(), "Requested count exceeds {}", self.size());

        let mut res = self.cluster.iter().sample(&mut rand::rng(), N);

        res.shuffle(&mut rand::rng());

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
            .sample(&mut rand::rng(), N);

        res.shuffle(&mut rand::rng());

        res.try_into()
            .unwrap_or_else(|_| panic!("cannot find {N} nodes with win prob 1.0"))
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
            .sample(&mut rand::rng(), N);

        res.shuffle(&mut rand::rng());

        res.try_into()
            .unwrap_or_else(|_| panic!("cannot find {N} nodes with win prob < 0.99"))
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
            .sample(&mut rand::rng(), N - 2);

        res.shuffle(&mut rand::rng());

        res.insert(0, win_prob_lower[0]);
        res.push(win_prob_lower[1]);

        res.try_into()
            .unwrap_or_else(|_| panic!("cannot find sufficient number of nodes"))
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
            .sample(&mut rand::rng(), N - 2);

        res.shuffle(&mut rand::rng());

        res.insert(0, win_prob_1[0]);
        res.push(win_prob_1[1]);

        res.try_into().expect("cannot find sufficient number of nodes")
    }
}

pub const SWARM_N: usize = 9;

/// Global per-test timeout.
///
/// Coverage instrumentation adds ~2-3x overhead, so we double the timeout
/// when running under `cargo llvm-cov` (which sets `cfg(coverage)`).
///
/// Even non-coverage runs set 8 minutes because `#[serial]` tests within
/// the same binary queue behind each other.  A binary with ~6 serial tests
/// totaling ~280 s of wall-clock can cause the last test to fire the rstest
/// timeout before acquiring the serial mutex.
#[allow(unexpected_cfgs)]
pub const TEST_GLOBAL_TIMEOUT: Duration = if cfg!(coverage) {
    Duration::from_mins(16)
} else {
    Duration::from_mins(8)
};

lazy_static::lazy_static! {
    pub static ref NODE_CHAIN_KEYS: Vec<ChainKeypair> = vec![
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
    pub static ref NODE_OFFCHAIN_KEYS: Vec<OffchainKeypair> = vec![
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
    pub static ref NODE_SAFES_MODULES: Vec<(Address, Address)> = vec![
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
pub const INITIAL_SAFE_TOKEN: u64 = 90000;
pub const INITIAL_NODE_NATIVE: u64 = 1;
pub const INITIAL_NODE_TOKEN: u64 = 10;
pub const DEFAULT_SAFE_ALLOWANCE: u128 = 1_000_000_000_000_u128;
pub const MINIMUM_INCOMING_WIN_PROB: f64 = 0.2;

/// Set the mixer delay range to a low value so tests don't stall on mixing latency.
/// Must be called before any tokio runtime is created (safe via `Once`).
static INIT_TEST_ENV: std::sync::Once = std::sync::Once::new();

/// Per-node configuration for test clusters.
#[derive(Debug, Clone)]
pub struct TestNodeConfig {
    /// Outgoing winning probability for this node.
    pub win_prob: f64,
    /// Optional PIX session config for incoming sessions on this node (Exit side).
    pub incoming_pix_config: Option<crate::exports::transport::session::IncomingSessionPixConfig>,
    /// Session idle timeout in milliseconds (default 2500).
    pub idle_timeout_ms: u64,
    /// Optional PIX global config override (num_ssa_parts, ssa_part_size).
    /// When set, configures the transport-level SsaShareGenerator dimensions.
    /// Must match the dimensions used in pix_ssa_quota for PIX sessions.
    pub pix_global_config: Option<crate::exports::transport::config::PixGlobalConfig>,
}

impl Default for TestNodeConfig {
    fn default() -> Self {
        Self {
            win_prob: 1.0,
            incoming_pix_config: None,
            idle_timeout_ms: 2500,
            pix_global_config: None,
        }
    }
}

impl TestNodeConfig {
    pub fn with_probability(win_prob: f64) -> Self {
        Self {
            win_prob,
            incoming_pix_config: None,
            idle_timeout_ms: 2500,
            pix_global_config: None,
        }
    }
}

/// Generates configs with alternating win probabilities (even=1.0, odd=MINIMUM_INCOMING_WIN_PROB).
fn alternating_configs(n: usize) -> Vec<TestNodeConfig> {
    (0..n)
        .map(|i| {
            if i % 2 != 0 {
                TestNodeConfig::with_probability(MINIMUM_INCOMING_WIN_PROB)
            } else {
                TestNodeConfig::default()
            }
        })
        .collect()
}

pub fn build_blokli_client() -> BlokliTestClient<FullStateEmulator> {
    BlokliTestStateBuilder::default()
        .with_hopr_network_chain_info("anvil-localhost")
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
                owners: vec![chain_key.public().to_address()],
                module: *module_address,
                registered_nodes: vec![],
                deployer: chain_key.public().to_address(),
            },
        ))
        .with_minimum_win_prob(WinningProbability::try_from(MINIMUM_INCOMING_WIN_PROB).unwrap())
        .with_ticket_price(HoprBalance::new_base(1))
        .with_closure_grace_period(Duration::ZERO)
        .build_dynamic_client(Address::default()) // Placeholder module address, to be filled in later
        .with_tx_simulation_delay(Duration::from_millis(100))
}

#[fixture]
#[once]
pub fn size_2_cluster_fixture() -> ClusterGuard {
    cluster_fixture(alternating_configs(2))
}

#[fixture]
#[once]
pub fn size_3_cluster_fixture() -> ClusterGuard {
    cluster_fixture(alternating_configs(3))
}

#[fixture]
#[once]
pub fn size_5_cluster_fixture() -> ClusterGuard {
    cluster_fixture(alternating_configs(5))
}

#[fixture]
pub fn cluster_fixture(#[default(vec![TestNodeConfig::default(); 3])] configs: Vec<TestNodeConfig>) -> ClusterGuard {
    let size = configs.len();
    if !(1..=SWARM_N).contains(&size) {
        panic!("{size} must be between 1 and {SWARM_N}");
    }

    // Reduce mixer delay range so tests don't stall on mixing latency.
    // SAFETY: `Once` guarantees this runs only once, before any async context.
    INIT_TEST_ENV.call_once(|| {
        // Safety: `Once` ensures single execution in a context where no
        // tokio runtime has been started yet (cluster_fixture runs before
        // any async block, and build_role_cluster's sub-tasks create their
        // own runtimes after spawning).
        unsafe { std::env::set_var("HOPR_INTERNAL_MIXER_DELAY_RANGE_IN_MS", "20") };
    });

    let chain_client = build_blokli_client();

    // Use the first SWARM_N onchain keypairs from the chainenv fixture
    let onchain_keys = NODE_CHAIN_KEYS[0..size].to_vec();
    let offchain_keys = NODE_OFFCHAIN_KEYS[0..size].to_vec();
    let safes = NODE_SAFES_MODULES[0..size]
        .iter()
        .map(|(safe, module)| NodeSafeConfig {
            safe_address: *safe,
            module_address: *module,
        })
        .collect::<Vec<_>>();

    // Setup nodes
    let cluster: Vec<TestedHopr> = (0..size)
        .map(|i| {
            let onchain_keys = onchain_keys.clone();
            let offchain_keys = offchain_keys.clone();
            let safes = safes.clone();
            let win_prob = configs[i].win_prob;
            let incoming_pix_config = configs[i].incoming_pix_config.clone();
            let idle_timeout_ms = configs[i].idle_timeout_ms;
            let pix_global_config = configs[i].pix_global_config;

            let blokli_client = chain_client
                .clone()
                .with_mutator(FullStateEmulator::new(safes[i].module_address));

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

                    let connector = std::sync::Arc::new(connector);

                    let config = create_hopr_instance_config(
                        3001 + i as u16,
                        safes[i],
                        win_prob,
                        incoming_pix_config,
                        idle_timeout_ms,
                        pix_global_config,
                    );

                    let instance = crate::testing::wiring::build_full_with_chain(
                        &onchain_keys[i],
                        &offchain_keys[i],
                        config,
                        Some(hopr_ct_full_network::ProberConfig {
                            interval: std::time::Duration::from_secs(3),
                            ..Default::default()
                        }), // moderate setting to allow probing without saturating relay traffic
                        connector.clone(),
                        EchoServer::new(),
                    )
                    .await?;

                    anyhow::Ok((instance, connector))
                });

                result.map(|(instance, connector)| TestedHopr::new(runtime, instance, connector))
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
            // Wait for all nodes to reach the 'Running' state.
            // Use generous timeouts to accommodate CI and coverage instrumentation overhead.
            futures::future::try_join_all(cluster.iter().map(|instance| {
                wait_for_status(instance, &HoprState::Running).timeout(futures_time::time::Duration::from_secs(360))
            }))
            .await
            .expect("status wait failed");

            // Wait for full mesh connectivity and probe warmup.
            // Connection establishment in the test environment is slow (~100s for 3 nodes)
            // and probe warmup needs additional rounds after connections are up.
            // Use generous timeouts to accommodate CI and coverage instrumentation overhead.
            futures::future::try_join_all(cluster.iter().map(|instance| {
                wait_for_connectivity(instance, swarm_size).timeout(futures_time::time::Duration::from_secs(480))
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

/// Intermediate result from a `block_on` build — avoids moving `runtime` into the async block.
enum RawRoleNode {
    Entry(
        Arc<crate::testing::wiring::EdgeHopr<TestingConnector>>,
        TestingConnector,
    ),
    Relay(
        Arc<crate::testing::wiring::FullHopr<TestingConnector>>,
        TestingConnector,
    ),
    Exit(
        Arc<crate::testing::wiring::EdgeHopr<TestingConnector>>,
        TestingConnector,
    ),
}

enum RoleThreadNode {
    Entry(TestedHopr<()>),
    Relay(TestedHopr),
    Exit(TestedHopr<()>),
}

/// A guard for role-typed clusters (Entry + Relays + Exit).
///
/// Unlike `ClusterGuard` which holds a flat `Vec<TestedHopr>` of relays,
/// this guard preserves the node roles so callers can access `entry`,
/// `relays`, and `exit` by name while keeping their distinct `TMgr` types.
pub struct RoleClusterGuard {
    pub entry: TestedHopr<()>,
    pub relays: Vec<TestedHopr>,
    pub exit: TestedHopr<()>,
    pub chain_client: BlokliTestClient<FullStateEmulator>,
}

/// Builds a role-typed cluster with one Entry, N Relays, and one Exit.
///
/// Each node is built with the correct transport role (`run_entry` / `run_relay` / `run_exit`).
/// Entry and Exit nodes use `()` as their ticket manager (no ticket processing).
/// Relay nodes use `SharedTicketManager` (full ticket processing).
///
/// This function is separate from `cluster_fixture` and `ClusterGuard` — those
/// remain unchanged for backward compatibility with existing tests.
pub async fn build_role_cluster(
    entry_cfg: TestNodeConfig,
    relay_cfgs: Vec<TestNodeConfig>,
    exit_cfg: TestNodeConfig,
) -> anyhow::Result<RoleClusterGuard> {
    let total_size = 1 + relay_cfgs.len() + 1;
    if !(3..=SWARM_N).contains(&total_size) {
        anyhow::bail!("total cluster size {total_size} must be between 3 and {SWARM_N}");
    }

    // Reduce mixer delay range so tests don't stall on mixing latency.
    INIT_TEST_ENV.call_once(|| {
        // Safety: `Once` ensures single execution in a context where no
        // tokio runtime has been started yet (build_role_cluster spawns
        // threads with their own runtimes afterwards).
        unsafe { std::env::set_var("HOPR_INTERNAL_MIXER_DELAY_RANGE_IN_MS", "20") };
    });

    let chain_client = build_blokli_client();

    // Assign onchain keys: entry = 0, relays = 1..N, exit = N
    let all_cfgs = {
        let mut cfgs = vec![entry_cfg];
        cfgs.extend(relay_cfgs);
        cfgs.push(exit_cfg);
        cfgs
    };
    let onchain_keys = NODE_CHAIN_KEYS[0..total_size].to_vec();
    let offchain_keys = NODE_OFFCHAIN_KEYS[0..total_size].to_vec();
    let safes = NODE_SAFES_MODULES[0..total_size]
        .iter()
        .map(|(safe, module)| NodeSafeConfig {
            safe_address: *safe,
            module_address: *module,
        })
        .collect::<Vec<_>>();

    let handles: Vec<_> = (0..total_size)
        .map(|i| {
            let onchain_keys = onchain_keys.clone();
            let offchain_keys = offchain_keys.clone();
            let safes = safes.clone();
            let cfg = all_cfgs[i].clone();
            let blokli_client = chain_client
                .clone()
                .with_mutator(FullStateEmulator::new(safes[i].module_address));
            let is_entry = i == 0;
            let is_exit = i == total_size - 1;

            std::thread::spawn(move || {
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(
                        std::thread::available_parallelism()
                            .map(|v| v.get())
                            .unwrap_or(1)
                            .div(total_size)
                            .max(3)
                            - 1,
                    )
                    .thread_stack_size(5 * 1024 * 1024)
                    .thread_name(format!("hopr-node-{i}"))
                    .enable_all()
                    .build()
                    .expect("failed to build Tokio runtime");

                let result = runtime.block_on(async {
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

                    let connector = std::sync::Arc::new(connector);

                    let config = create_hopr_instance_config(
                        3001 + i as u16,
                        safes[i],
                        cfg.win_prob,
                        cfg.incoming_pix_config,
                        cfg.idle_timeout_ms,
                        cfg.pix_global_config,
                    );

                    let prober = Some(hopr_ct_full_network::ProberConfig {
                        interval: std::time::Duration::from_secs(3),
                        ..Default::default()
                    });

                    let node: anyhow::Result<RawRoleNode> = if is_entry {
                        let instance = crate::testing::wiring::build_entry_with_chain(
                            &onchain_keys[i],
                            &offchain_keys[i],
                            config,
                            prober,
                            connector.clone(),
                            EchoServer::new(),
                        )
                        .await?;
                        Ok(RawRoleNode::Entry(instance, connector))
                    } else if is_exit {
                        let instance = crate::testing::wiring::build_exit_with_chain(
                            &onchain_keys[i],
                            &offchain_keys[i],
                            config,
                            prober,
                            connector.clone(),
                            EchoServer::new(),
                        )
                        .await?;
                        Ok(RawRoleNode::Exit(instance, connector))
                    } else {
                        let instance = crate::testing::wiring::build_full_with_chain(
                            &onchain_keys[i],
                            &offchain_keys[i],
                            config,
                            prober,
                            connector.clone(),
                            EchoServer::new(),
                        )
                        .await?;
                        Ok(RawRoleNode::Relay(instance, connector))
                    };

                    node
                });

                result.map(|raw| match raw {
                    RawRoleNode::Entry(instance, connector) => {
                        RoleThreadNode::Entry(TestedHopr::<()>::new(runtime, instance, connector))
                    }
                    RawRoleNode::Exit(instance, connector) => {
                        RoleThreadNode::Exit(TestedHopr::<()>::new(runtime, instance, connector))
                    }
                    RawRoleNode::Relay(instance, connector) => {
                        RoleThreadNode::Relay(TestedHopr::new(runtime, instance, connector))
                    }
                })
            })
        })
        .collect();

    let mut entry: Option<TestedHopr<()>> = None;
    let mut relays: Vec<TestedHopr> = Vec::new();
    let mut exit: Option<TestedHopr<()>> = None;

    for handle in handles {
        let thread_result = handle
            .join()
            .map_err(|_| anyhow::anyhow!("a hopr node thread panicked"))?
            .map_err(|e| anyhow::anyhow!("hopr node build failed: {e}"))?;

        match thread_result {
            RoleThreadNode::Entry(n) => entry = Some(n),
            RoleThreadNode::Relay(n) => relays.push(n),
            RoleThreadNode::Exit(n) => exit = Some(n),
        }
    }

    let entry = entry.expect("entry should be set");
    let exit = exit.expect("exit should be set");

    // Wait for all nodes to reach Running state
    wait_for_status(&entry, &HoprState::Running).await;
    for relay in &relays {
        wait_for_status(relay, &HoprState::Running).await;
    }
    wait_for_status(&exit, &HoprState::Running).await;

    // Wait for P2P connectivity on all nodes (libp2p connections).
    // Probe warmup is only meaningful for relays — Entry/Exit nodes use
    // `drain_incoming_data` for their socket reader which drops probe
    // echo responses, making probe warmup stall indefinitely.
    wait_for_p2p_connectivity(&entry, total_size).await;
    for relay in &relays {
        wait_for_connectivity(relay, total_size).await;
    }
    wait_for_p2p_connectivity(&exit, total_size).await;

    info!(total_size, "ROLE CLUSTER STARTED");

    Ok(RoleClusterGuard {
        entry,
        relays,
        exit,
        chain_client,
    })
}

/// Waits for P2P connectivity without probe warmup.
///
/// Checks only that `connected_peers` reaches `swarm_size - 1` — suitable for
/// Entry/Exit nodes which do not run a probe-based liveness system.
pub async fn wait_for_p2p_connectivity<TMgr: 'static + Send + Sync>(instance: &TestedHopr<TMgr>, swarm_size: usize) {
    info!("Waiting for full connectivity");
    loop {
        let peers = instance.inner().network_view().connected_peers();

        if peers.len() == swarm_size - 1 {
            break;
        }

        tracing::trace!(
            "{} peers connected on {}, waiting for full mesh",
            peers.len(),
            instance.address()
        );
        sleep(Duration::from_secs(1)).await;
    }
}

pub async fn wait_for_connectivity<TMgr: 'static + Send + Sync>(instance: &TestedHopr<TMgr>, swarm_size: usize) {
    info!("Waiting for full connectivity");
    loop {
        let peers = instance.inner().network_view().connected_peers();

        if peers.len() == swarm_size - 1 {
            break;
        }

        tracing::trace!(
            "{} peers connected on {}, waiting for full mesh",
            peers.len(),
            instance.address()
        );
        sleep(Duration::from_millis(200)).await;
    }

    // Wait for probe warmup: all connected peers should be individually reachable.
    // Since `network_peer_info` was removed, we use `is_connected` as the liveness check.
    info!("Waiting for probe warmup");
    loop {
        let peers = instance.inner().network_view().connected_peers();

        let observed = peers
            .iter()
            .filter(|p| instance.inner().network_view().is_connected(p))
            .count();

        if observed == swarm_size - 1 {
            break;
        }

        tracing::trace!(
            "{observed}/{} peers observed on {}, waiting for probe warmup",
            swarm_size - 1,
            instance.address()
        );
        sleep(Duration::from_millis(200)).await;
    }
}

pub async fn wait_for_status<TMgr: 'static + Send + Sync>(instance: &TestedHopr<TMgr>, expected_status: &HoprState) {
    info!(
        "Waiting for node {} to reach status {:?}",
        instance.address(),
        expected_status
    );
    loop {
        let status = HoprNodeOperations::status(instance.inner());
        if &status == expected_status {
            break;
        }

        tracing::trace!("{status} on {}, waiting for {expected_status}", instance.address());
        sleep(Duration::from_millis(200)).await;
    }
}
