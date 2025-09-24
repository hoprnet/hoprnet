mod common;

use std::{str::FromStr, time::Duration};

use alloy::primitives::U256;
use common::TestChainEnv;
use hopr_lib::Hopr;
use once_cell::sync::Lazy;
use rstest::rstest;
use tokio::sync::OnceCell;

const SNAPSHOT_BASE: &str = "tests/snapshots/node_snapshot_base";
const PATH_TO_PROTOCOL_CONFIG: &str = "tests/protocol-config-anvil.json";
const SWARM_N: usize = 2;

static CHAINENV_FIXTURE: Lazy<OnceCell<TestChainEnv>> = Lazy::new(|| OnceCell::const_new());
static SWARM_N_FIXTURE: Lazy<OnceCell<Vec<Hopr>>> = Lazy::new(|| OnceCell::const_new());

#[rstest::fixture]
fn random_int_pair() -> (usize, usize) {
    use rand::prelude::SliceRandom;

    let mut numbers: Vec<usize> = (0..SWARM_N).collect();
    numbers.shuffle(&mut rand::thread_rng());

    (numbers[0], numbers[1])
}

#[rstest::fixture]
async fn chainenv_fixture() -> &'static TestChainEnv {
    use common::deploy_test_environment;
    use hopr_chain_rpc::client::SnapshotRequestor;

    CHAINENV_FIXTURE
        .get_or_init(|| async {
            env_logger::init();

            let requestor_base = SnapshotRequestor::new(SNAPSHOT_BASE)
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
async fn cluster_fixture(#[future(awt)] chainenv_fixture: &TestChainEnv) -> &'static Vec<Hopr> {
    use std::fs::read_to_string;

    use common::onboard_node;
    use hopr_crypto_types::prelude::{Keypair, OffchainKeypair};
    use hopr_lib::{
        ProtocolsConfig,
        config::{Chain, HoprLibConfig, SafeModule},
    };

    SWARM_N_FIXTURE
        .get_or_init(|| async {
            let protocol_config = ProtocolsConfig::from_str(
                &read_to_string(PATH_TO_PROTOCOL_CONFIG).expect("failed to read protocol config file"),
            )
            .expect("failed to parse protocol config");

            // Use the first SWARM_N onchain keypairs from the chainenv fixture
            let onchain_keys = chainenv_fixture.node_chain_keys[0..SWARM_N].to_vec();

            // Setup SWARM_N safes
            let mut safes = Vec::with_capacity(SWARM_N);
            for i in 0..SWARM_N {
                let safe = onboard_node(
                    &chainenv_fixture,
                    &onchain_keys[i],
                    U256::from(10_u32),
                    U256::from(10_000_u32),
                )
                .await;
                safes.push(safe);
            }

            // Setup SWARM_N nodes
            let hopr_instances: Vec<Hopr> = (0..SWARM_N)
                .map(|i| {
                    Hopr::new(
                        HoprLibConfig {
                            chain: Chain {
                                protocols: protocol_config.clone(),
                                provider: Some(chainenv_fixture.anvil.endpoint()),
                                ..Default::default()
                            },
                            host: hopr_lib::config::HostConfig {
                                address: hopr_lib::config::HostType::default(),
                                port: 3001 + i as u16,
                            },
                            db: hopr_lib::config::Db {
                                data: format!("/tmp/hopr-tests/node_{i}"),
                                force_initialize: false,
                                ..Default::default()
                            },
                            safe_module: SafeModule {
                                safe_address: safes[i].safe_address,
                                module_address: safes[i].module_address,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        &OffchainKeypair::random(),
                        &onchain_keys[i],
                    )
                    .expect(format!("failed to create hopr instance no. {i}").as_str())
                })
                .collect();

            // TODO: enable this once the network registry is removed
            // let (_a, _b) = hopr_instance.run().await?;

            hopr_instances
        })
        .await
}

#[rstest]
#[cfg(feature = "runtime-tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_get_addresses(#[future(awt)] cluster_fixture: &Vec<Hopr>) -> anyhow::Result<()> {
    assert!(cluster_fixture[0].me_onchain() != Address::default());
    Ok(())
}

#[rstest]
#[cfg(feature = "runtime-tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_get_infos(#[future(awt)] cluster_fixture: &Vec<Hopr>) -> anyhow::Result<()> {
    let node: &Hopr = &cluster_fixture[0];

    assert!(node.network() != "");
    assert!(node.get_safe_config().safe_address != Address::default());
    Ok(())
}

#[rstest]
#[cfg(feature = "runtime-tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_get_balance(#[future(awt)] cluster_fixture: &Vec<Hopr>) -> anyhow::Result<()> {
    use hopr_lib::{HoprBalance, XDaiBalance};

    let node: &Hopr = &cluster_fixture[0];

    let safe_native: XDaiBalance = node.get_safe_balance().await.expect("should get safe xdai balance");
    let native: XDaiBalance = node.get_balance().await.expect("should get node xdai balance");
    let safe_hopr: HoprBalance = node
        .get_safe_hopr_balance()
        .await
        .expect("should get safe hopr balance");

    let hopr: HoprBalance = node.get_balance().await.expect("should get node hopr balance");
    let safe_allowance: HoprBalance = node.safe_allowance().await.expect("should get safe hopr allowance");

    Ok(())
}

#[rstest]
#[cfg(feature = "runtime-tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_get_channels(#[future(awt)] cluster_fixture: &Vec<Hopr>) -> anyhow::Result<()> {
    let node = &cluster_fixture[0];

    let channels = node.channels_from(node.me_onchain()).await?;
    assert!(channels.is_empty());

    Ok(())
}

#[rstest]
#[cfg(feature = "runtime-tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_open_and_close_sessions(#[future(awt)] cluster_fixture: &Vec<Hopr>) -> anyhow::Result<()> {
    use hopr_lib::{SessionClientConfig, SessionTarget};
    use hopr_transport_session::{Capabilities, Capability}; // TODO: should use hopr-lib instead

    let session = cluster_fixture[0]
        .connect_to(
            &cluster_fixture[1].me_onchain(),
            SessionTarget::UdpStream(":0"),
            SessionClientConfig {
                forward_path_options: hopr_lib::RoutingOptions::Hops(0_u32.try_into()?),
                return_path_options: hopr_lib::RoutingOptions::Hops(0_u32.try_into()?),
                capabilities: Capabilities::from(Capability::Segmentation),
                pseudonym: None,
                surb_management: None,
                always_max_out_surbs: true,
            },
        )
        .await
        .expect("creating a session must succeed");

    // let sessions = cluster_fixture[0].list_sessions().await?; // TODO. Do once integrated into edgli

    Ok(())
}

#[rstest]
#[cfg(feature = "runtime-tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_update_session_config(#[future(awt)] cluster_fixture: &Vec<Hopr>) -> anyhow::Result<()> {
    use hopr_lib::SurbBalancerConfig;

    let node = &cluster_fixture[0];

    let session = node
        .connect_to(
            &cluster_fixture[1].me_onchain(),
            SessionTarget::UdpStream(":0"),
            SessionClientConfig {
                forward_path_options: hopr_lib::RoutingOptions::Hops(0_u32.try_into()?),
                return_path_options: hopr_lib::RoutingOptions::Hops(0_u32.try_into()?),
                capabilities: Capabilities::from(Capability::Segmentation),
                pseudonym: None,
                surb_management: None,
                always_max_out_surbs: true,
            },
        )
        .await
        .expect("creating a session must succeed");

    let config = node
        .get_session_surb_balancer_config(&session.id())
        .await
        .expect("should get session config");

    assert_eq!(config, None);

    let new_config = SurbBalancerConfig {
        target_surb_buffer_size: 5_000,
        max_surbs_per_sec: 2500,
        surb_decay: Some((Duration::from_millis(200), 0.05)),
    };

    node.update_session_surb_balancer_config(&session.id(), new_config.clone())
        .await
        .expect("should update session config");

    assert_eq!(
        config = node
            .get_session_surb_balancer_config(&session.id())
            .await
            .expect("should get session config"),
        Some(new_config)
    );

    Ok(())
}
