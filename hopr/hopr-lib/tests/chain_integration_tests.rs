use async_std::sync::RwLock;
use chain_actions::action_queue::{ActionQueue, ActionQueueConfig};
use chain_actions::action_state::{ActionState, IndexerActionTracker};
use chain_actions::channels::ChannelActions;
use chain_actions::node::NodeActions;
use chain_actions::payload::SafePayloadGenerator;
use chain_actions::CoreEthereumActions;
use chain_api::executors::{EthereumTransactionExecutor, RpcEthereumClient, RpcEthereumClientConfig};
use chain_db::db::CoreEthereumDb;
use chain_db::traits::HoprCoreEthereumDbActions;
use chain_indexer::block::{Indexer, IndexerConfig};
use chain_indexer::handlers::ContractEventHandlers;
use chain_rpc::client::native::SurfRequestor;
use chain_rpc::client::{create_rpc_client_to_anvil, JsonRpcProviderClient, SimpleJsonRpcRetryPolicy};
use chain_rpc::rpc::{RpcOperations, RpcOperationsConfig};
use chain_types::chain_events::ChainEventType;
use chain_types::utils::create_anvil;
use chain_types::{ContractAddresses, ContractInstances};
use core_transport::{ChainKeypair, Keypair, Multiaddr, OffchainKeypair};
use futures::StreamExt;
use hopr_internal_types::channels::{ChannelDirection, ChannelStatus};
use hopr_primitive_types::prelude::*;
use log::debug;
use std::sync::Arc;
use std::time::Duration;
use utils_db::db::DB;
use utils_db::CurrentDbShim;

#[async_std::test]
async fn integration_test_indexer() {
    let anvil = create_anvil(Some(Duration::from_secs(4)));
    let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();
    let node_chain_key = ChainKeypair::from_secret(anvil.keys()[1].to_bytes().as_ref()).unwrap();
    let bob_chain_key = ChainKeypair::from_secret(anvil.keys()[2].to_bytes().as_ref()).unwrap();

    // Deploy contracts
    let (contract_addrs, module_addr, safe_addr) = {
        let client = create_rpc_client_to_anvil(SurfRequestor::default(), &anvil, &contract_deployer);
        let instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
            .await
            .expect("failed to deploy");

        // Mint some tokens
        chain_types::utils::mint_tokens(instances.token.clone(), 1000_u128.into()).await;

        // Fund the node address
        chain_types::utils::fund_node(
            node_chain_key.public().to_address(),
            10_u128.into(),
            100_u128.into(),
            instances.token.clone(),
        )
        .await;

        let (module, safe) = chain_types::utils::deploy_one_safe_one_module_and_setup_for_testing(
            &instances,
            client.clone(),
            &contract_deployer,
        )
        .await
        .expect("could not deploy safe and module");

        (ContractAddresses::from(&instances), module, safe)
    };

    // DB
    let db = Arc::new(RwLock::new(CoreEthereumDb::new(
        DB::new(CurrentDbShim::new_in_memory().await),
        node_chain_key.public().to_address(),
    )));

    // RPC
    let json_rpc_client = JsonRpcProviderClient::new(
        &anvil.endpoint(),
        SurfRequestor::default(),
        SimpleJsonRpcRetryPolicy::default(),
    );
    let rpc_ops = RpcOperations::new(json_rpc_client, &node_chain_key, RpcOperationsConfig::default())
        .expect("failed to create RpcOperations");

    // Transaction executor
    let eth_client = RpcEthereumClient::new(rpc_ops.clone(), RpcEthereumClientConfig::default());
    let tx_exec = EthereumTransactionExecutor::new(
        eth_client,
        SafePayloadGenerator::new(&node_chain_key, contract_addrs.clone(), module_addr),
    );

    // Actions
    let actions_cfg = ActionQueueConfig {
        max_action_confirmation_wait: Duration::from_secs(60),
    }; // use lower action confirmation limit
    let action_queue = ActionQueue::new(db.clone(), IndexerActionTracker::default(), tx_exec, actions_cfg);
    let action_state = action_queue.action_state();
    let actions = CoreEthereumActions::new(
        node_chain_key.public().to_address(),
        db.clone(),
        action_queue.new_sender(),
    );
    async_std::task::spawn_local(action_queue.action_loop());

    // Action state tracking
    let (sce_tx, mut sce_rx) = futures::channel::mpsc::unbounded();
    async_std::task::spawn_local(async move {
        while let Some(sce) = sce_rx.next().await {
            let res = action_state.match_and_resolve(&sce).await;
            debug!("{:?}: expectations resolved {:?}", sce, res);
        }
    });

    // Indexer
    let chain_log_handler = ContractEventHandlers::new(
        contract_addrs,
        safe_addr,
        node_chain_key.public().to_address(),
        db.clone(),
    );
    let mut indexer = Indexer::new(
        rpc_ops.clone(),
        chain_log_handler,
        db.clone(),
        IndexerConfig::default(),
        sce_tx,
    );
    indexer.start().await.expect("indexer should sync");

    // ----------------
    // Test plan:
    // Register with Safe
    // Announce
    // Open channel to Bob
    // Redeem ticket in the channel
    // Close channel to Bob

    // Register Safe
    let confirmation = actions
        .register_safe_by_node(safe_addr)
        .await
        .expect("should submit safe registration tx")
        .await
        .expect("should confirm safe registration");

    assert!(
        matches!(confirmation.event, Some(ChainEventType::NodeSafeRegistered(reg_safe)) if reg_safe == safe_addr),
        "confirmed safe address must match"
    );

    // Announce the node
    let maddr: Multiaddr = "/ip4/127.0.0.1/tcp/10000".parse().unwrap();
    let offchain_key = OffchainKeypair::random();
    let confirmation = actions
        .announce(&maddr, &offchain_key)
        .await
        .expect("should submit announcement tx")
        .await
        .expect("should confirm announcement");

    assert!(
        matches!(confirmation.event,
            Some(ChainEventType::Announcement{ peer, address, multiaddresses })
            if peer == offchain_key.public().into() &&
            address == node_chain_key.public().to_address() &&
            multiaddresses.contains(&maddr)
        ),
        "confirmed announcement must match"
    );

    // Open channel
    let confirmation = actions
        .open_channel(
            bob_chain_key.public().to_address(),
            Balance::new(U256::one(), BalanceType::HOPR),
        )
        .await
        .expect("should submit channel open tx")
        .await
        .expect("should confirm open channel");

    match confirmation.event {
        Some(ChainEventType::ChannelOpened(channel)) => {
            let new_channel_in_db = db
                .read()
                .await
                .get_channel_to(&bob_chain_key.public().to_address())
                .await
                .expect("db call should not fail")
                .expect("should contain a channel to Bob");

            assert_eq!(
                channel.get_id(),
                new_channel_in_db.get_id(),
                "channel in the DB must match the confirmed action"
            );
        }
        _ => panic!("invalid confirmation"),
    }

    let channel = db
        .read()
        .await
        .get_channel_to(&bob_chain_key.public().to_address())
        .await
        .expect("must get channel")
        .expect("channel to bob must exist");

    assert_eq!(ChannelStatus::Open, channel.status, "channel must be opened");

    // TODO: create acknowledged ticket from Bob and store it in the DB, so Alice can redeem it here

    // Close channel
    let confirmation = actions
        .close_channel(bob_chain_key.public().to_address(), ChannelDirection::Outgoing, false)
        .await
        .expect("should submit channel close tx")
        .await
        .expect("should confirm close channel");

    match confirmation.event {
        Some(ChainEventType::ChannelClosureInitiated(channel)) => {
            let closing_channel_in_db = db
                .read()
                .await
                .get_channel_to(&bob_chain_key.public().to_address())
                .await
                .expect("db call should not fail")
                .expect("should contain a channel to Bob");

            assert_eq!(
                channel.get_id(),
                closing_channel_in_db.get_id(),
                "channel in the DB must match the confirmed action"
            );
        }
        _ => panic!("invalid confirmation"),
    }

    let channel = db
        .read()
        .await
        .get_channel_to(&bob_chain_key.public().to_address())
        .await
        .expect("must get channel")
        .expect("channel to bob must exist");

    assert_eq!(
        ChannelStatus::PendingToClose,
        channel.status,
        "channel must be pending to close"
    );
}
