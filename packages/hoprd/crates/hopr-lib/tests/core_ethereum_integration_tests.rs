use std::sync::Arc;
use std::time::Duration;
use async_std::sync::RwLock;
use futures::StreamExt;
use core_ethereum_actions::action_queue::{ActionQueue, ActionQueueConfig};
use core_ethereum_actions::action_state::{ActionState, IndexerActionTracker};
use core_ethereum_actions::channels::ChannelActions;
use core_ethereum_actions::CoreEthereumActions;
use core_ethereum_actions::payload::SafePayloadGenerator;
use core_ethereum_api::executors::{EthereumTransactionExecutor, RpcEthereumClient, RpcEthereumClientConfig};
use core_ethereum_db::db::CoreEthereumDb;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_ethereum_indexer::block::{Indexer, IndexerConfig};
use core_ethereum_indexer::handlers::ContractEventHandlers;
use core_ethereum_rpc::client::{create_rpc_client_to_anvil, JsonRpcProviderClient, SimpleJsonRpcRetryPolicy};
use core_ethereum_rpc::client::native::SurfRequestor;
use core_ethereum_rpc::rpc::{RpcOperations, RpcOperationsConfig};
use core_ethereum_types::{ContractAddresses, ContractInstances};
use core_ethereum_types::chain_events::ChainEventType;
use core_transport::{ChainKeypair, Keypair};
use utils_db::db::DB;
use utils_db::rusty::RustyLevelDbShim;
use utils_log::debug;
use utils_types::primitives::{Address, Balance, BalanceType, U256};

#[async_std::test]
async fn integration_test_indexer() {
    let anvil = core_ethereum_types::utils::create_anvil(Some(Duration::from_secs(4)));
    let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();
    let node_chain_key = ChainKeypair::from_secret(anvil.keys()[1].to_bytes().as_ref()).unwrap();
    let bob_chain_key = ChainKeypair::from_secret(anvil.keys()[2].to_bytes().as_ref()).unwrap();

    let contract_addrs = {
        let client = create_rpc_client_to_anvil(SurfRequestor::default(), &anvil, &contract_deployer);
        let instances =
            ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
                .await
                .expect("failed to deploy");

        core_ethereum_types::utils::mint_tokens(instances.token.clone(), 1000_u128.into()).await;
        core_ethereum_types::utils::fund_node(node_chain_key.public().to_address(), 10_u128.into(), 100_u128.into(), instances.token.clone()).await;

        ContractAddresses::from(&instances)
    };

    // TODO: deploy module and safe
    let module_addr = Address::random();
    let safe_addr = Address::random();

    let json_rpc_client = JsonRpcProviderClient::new(&anvil.endpoint(), SurfRequestor::default());

    let rpc_ops = RpcOperations::new(json_rpc_client, &node_chain_key, RpcOperationsConfig::default(), SimpleJsonRpcRetryPolicy)
        .expect("failed to create RpcOperations");

    let eth_client = RpcEthereumClient::new(rpc_ops.clone(), RpcEthereumClientConfig::default());
    let tx_exec = EthereumTransactionExecutor::new(eth_client, SafePayloadGenerator::new(&node_chain_key, contract_addrs.clone(), module_addr));

    let db = Arc::new(RwLock::new(CoreEthereumDb::new(DB::new(RustyLevelDbShim::new_in_memory()), node_chain_key.public().to_address())));
    let chain_log_handler = ContractEventHandlers::new(contract_addrs, safe_addr, node_chain_key.public().to_address(), db.clone());

    let action_queue = ActionQueue::new(db.clone(), IndexerActionTracker::default(), tx_exec, ActionQueueConfig::default());

    let (sce_tx, mut sce_rx) = futures::channel::mpsc::unbounded();
    let mut indexer = Indexer::new(rpc_ops.clone(), chain_log_handler, db.clone(), IndexerConfig::default(), sce_tx);
    let action_tracker = action_queue.action_state();
    async_std::task::spawn_local(async move {
        while let Some(sce) = sce_rx.next().await {
            let res = action_tracker.match_and_resolve(&sce).await;
            debug!("expectations resolved {:?}", res);
        }
    });

    let actions = CoreEthereumActions::new(node_chain_key.public().to_address(), db.clone(), action_queue.new_sender());
    async_std::task::spawn_local(action_queue.transaction_loop());

    indexer.start().await.expect("indexer should sync");

    // ----------------

    let confirmation_resolver = actions.open_channel(bob_chain_key.public().to_address(), Balance::new(U256::one(), BalanceType::HOPR))
        .await
        .expect("should submit channel open tx");

    let confirmation = confirmation_resolver.await.expect("open channel should be resolved");

    match confirmation.event {
        Some(ChainEventType::ChannelOpened(channel)) => {
            let new_channel_in_db = db.read().await.get_channel_to(&bob_chain_key.public().to_address())
                .await
                .expect("db call should not fail")
                .expect("should contain a channel to Bob");

            assert_eq!(channel.get_id(), new_channel_in_db.get_id(), "channel in the DB must match the confirmed action");
        },
        _ => panic!("invalid confirmation")
    }
}