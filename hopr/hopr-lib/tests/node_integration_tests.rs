mod common;

use std::str::FromStr;
use std::time::Duration;

use common::TestChainEnv;
use hopr_lib::Hopr;

use alloy::primitives::U256;
use once_cell::sync::Lazy;
use rstest::rstest;
use tokio::sync::OnceCell;
use tracing::info;

const SNAPSHOT_BASE: &str = "tests/snapshots/node_snapshot_base";
const PATH_TO_PROTOCOL_CONFIG: &str = "tests/protocol-config-anvil.json";
const SWARM_N: usize = 1;

static CHAINENV_FIXTURE: Lazy<OnceCell<TestChainEnv>> = Lazy::new(|| OnceCell::const_new());
static SWARM_N_FIXTURE: Lazy<OnceCell<Vec<Hopr>>> = Lazy::new(|| OnceCell::const_new());

#[rstest::fixture]
async fn chainenv_fixture() -> &'static TestChainEnv {
    use common::deploy_test_environment;
    use hopr_chain_rpc::client::SnapshotRequestor;

    CHAINENV_FIXTURE
        .get_or_init(|| async {
            env_logger::init();

            let block_time = Duration::from_secs(1);
            let finality = 2;

            let requestor_base = SnapshotRequestor::new(SNAPSHOT_BASE)
                .with_ignore_snapshot(!hopr_crypto_random::is_rng_fixed())
                .load(true)
                .await;

            deploy_test_environment(requestor_base, block_time, finality).await
        })
        .await
}
#[rstest::fixture]
async fn swarm_fixture(#[future(awt)] chainenv_fixture: &TestChainEnv) -> &'static Vec<Hopr> {
    use common::{NodeSafeConfig, onboard_node};
    use hopr_crypto_types::prelude::{ChainKeypair, Keypair, OffchainKeypair};
    use hopr_lib::{
        ProtocolsConfig,
        config::{Chain, HoprLibConfig, SafeModule},
    };
    use std::fs::read_to_string;

    SWARM_N_FIXTURE
        .get_or_init(|| async {
            let onchain_keys = chainenv_fixture.node_chain_keys[0..SWARM_N]
                .iter()
                .map(|k| k.clone())
                .collect::<Vec<ChainKeypair>>();

            let offchain_keys: OffchainKeypair = OffchainKeypair::random();

            // Setup SWARM_N safes
            let mut safes = Vec::with_capacity(SWARM_N);
            for i in 0..SWARM_N {
                let safe: NodeSafeConfig = onboard_node(
                    &chainenv_fixture,
                    &onchain_keys[i],
                    U256::from(10_u32),
                    U256::from(10_000_u32),
                )
                .await;
                safes.push(safe);
            }

            let protocol_config = ProtocolsConfig::from_str(
                &read_to_string(PATH_TO_PROTOCOL_CONFIG).expect("failed to read protocol config file"),
            )
            .expect("failed to parse protocol config");

            // Setup SWARM_N nodes
            let mut hopr_instances = Vec::with_capacity(SWARM_N);
            for i in 0..SWARM_N {
                let hopr_instance = Hopr::new(
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
                            data: format!("/tmp/hopr-tests/node{}", i),
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
                    &offchain_keys,
                    &chainenv_fixture.node_chain_keys[i],
                )
                .expect("failed to create hopr instance");

                hopr_instances.push(hopr_instance);
            }

            // let (_a, _b) = hopr_instance.run().await?;

            hopr_instances
        })
        .await
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[cfg(feature = "runtime-tokio")]
async fn test_open_channel(#[future(awt)] swarm_fixture: &Vec<Hopr>) -> Result<(), Box<dyn std::error::Error>> {
    let channels = swarm_fixture[0].all_channels().await?;
    info!("Node has {} channels", channels.len());

    // let res = node_fixture
    //     .open_channel(
    //         &(node_chain_keys[1].public().to_address()),
    //         HoprBalance::from_str("1 wxHOPR")?,
    //     )
    //     .await
    //     .expect("failed to open channel");

    // println!("Opened channel: {:?}", res.channel_id);

    // println!("Node has {} channels", channels.len());

    // debug!("Created Hopr instances for both nodes");

    // finish the test
    Ok(())
}
