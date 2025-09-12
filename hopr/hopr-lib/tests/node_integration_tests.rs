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
async fn test_open_channel(
    #[future(awt)] cluster_fixture: &Vec<Hopr>,
    random_int_pair: (usize, usize),
) -> anyhow::Result<()> {
    use hopr_lib::HoprBalance;

    let (src, dst) = random_int_pair;

    assert_eq!((cluster_fixture[src].all_channels().await?).len(), 0);

    cluster_fixture[src]
        .open_channel(&(cluster_fixture[dst].me_onchain()), HoprBalance::from_str("1 wxHOPR")?)
        .await
        .expect("failed to open channel");

    assert_eq!((cluster_fixture[src].all_channels().await?).len(), 1);

    Ok(())
}

#[rstest]
#[cfg(feature = "runtime-tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_cluster_connectivity(#[future(awt)] cluster_fixture: &Vec<Hopr>) -> anyhow::Result<()> {
    let results = futures::future::join_all(cluster_fixture.iter().map(|node| node.network_connected_peers()))
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("failed to get connected peers");

    for peers in &results {
        assert_eq!(peers.len(), SWARM_N - 1);
    }

    Ok(())
}

#[rstest]
#[cfg(feature = "runtime-tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_send_0_hop_without_open_channels(
    #[future(awt)] cluster_fixture: &Vec<Hopr>,
    random_int_pair: (usize, usize),
) -> anyhow::Result<()> {
    use hopr_lib::{DestinationRouting, Tag};
    use hopr_primitive_types::bounded::BoundedSize;

    let (src, dst) = random_int_pair;

    cluster_fixture[src]
        .send_message(
            b"Hello, HOPR!".to_vec().into(),
            DestinationRouting::forward_only(
                cluster_fixture[dst].me_onchain(),
                hopr_lib::RoutingOptions::Hops(BoundedSize::default()),
            ),
            Tag::Application(1024),
        )
        .await
        .expect("failed to send 0-hop message");

    Ok(())
}

#[rstest]
#[cfg(feature = "runtime-tokio")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_get_ticket_price(#[future(awt)] cluster_fixture: &Vec<Hopr>) -> anyhow::Result<()> {
    use hopr_lib::{Balance, WxHOPR};

    let ticket_price = cluster_fixture[0]
        .get_ticket_price()
        .await?
        .expect("ticket price should not be None");

    assert!(ticket_price > Balance::<WxHOPR>::zero());
    Ok(())
}
