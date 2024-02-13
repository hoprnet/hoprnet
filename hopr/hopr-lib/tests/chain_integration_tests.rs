use async_std::sync::RwLock;
use async_std::task::JoinHandle;
use chain_actions::action_queue::{ActionQueue, ActionQueueConfig};
use chain_actions::action_state::{ActionState, IndexerActionTracker};
use chain_actions::channels::ChannelActions;
use chain_actions::node::NodeActions;
use chain_actions::payload::SafePayloadGenerator;
use chain_actions::redeem::TicketRedeemActions;
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
use chain_types::utils::{
    add_announcement_as_target, approve_channel_transfer_from_safe, create_anvil, include_node_to_module_by_safe,
};
use chain_types::{ContractAddresses, ContractInstances};
use core_transport::{ChainKeypair, Hash, Keypair, Multiaddr, OffchainKeypair};
use ethers::providers::Middleware;
use ethers::utils::AnvilInstance;
use futures::StreamExt;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use log::{debug, info};
use std::sync::Arc;
use std::time::Duration;
use utils_db::constants::ACKNOWLEDGED_TICKETS_PREFIX;
use utils_db::db::DB;
use utils_db::sqlite::SqliteShim;
use utils_db::CurrentDbShim;

// Helper function to generate the first acked ticket (channel_epoch 1, index 0, offset 0) of win prob 100%
async fn generate_the_first_ack_ticket<M: Middleware>(
    myself: &ChainNode,
    counterparty: &ChainKeypair,
    price: Balance,
    instances: &ContractInstances<M>,
) {
    let hk1 = HalfKey::random();
    let hk2 = HalfKey::random();

    let cp1: CurvePoint = hk1.to_challenge().into();
    let cp2: CurvePoint = hk2.to_challenge().into();
    let cp_sum = CurvePoint::combine(&[&cp1, &cp2]);

    let domain_separator: Hash = instances.channels.domain_separator().call().await.unwrap().into();

    let ticket = Ticket::new(
        &myself.chain_key.public().to_address(),
        &price,
        U256::zero(),
        U256::one(),
        1.0f64,
        U256::one(),
        &Challenge::from(cp_sum).to_ethereum_challenge(),
        counterparty,
        &domain_separator,
    );

    if let Err(e) = validate_ticket(&ticket, &myself.chain_key.public().to_address(), &domain_separator) {
        panic!("{:?}", e);
    }

    let unacked_ticket = UnacknowledgedTicket::new(ticket, hk1);

    assert!(validate_unacknowledged_ticket(&unacked_ticket, &Hash::default()).is_ok());
    let ack_ticket = unacked_ticket.acknowledge(&hk2).unwrap();

    assert!(validate_acknowledged_ticket(&ack_ticket).is_ok());

    let mut ack_key = Vec::new();
    ack_key.extend_from_slice(&ack_ticket.ticket.channel_id.to_bytes());
    ack_key.extend_from_slice(&ack_ticket.ticket.channel_epoch.to_be_bytes());
    ack_key.extend_from_slice(&ack_ticket.ticket.index.to_be_bytes());
    let key = utils_db::db::Key::new_bytes_with_prefix(&ack_key, ACKNOWLEDGED_TICKETS_PREFIX).unwrap();

    myself
        .db
        .write()
        .await
        .db
        .set(key, &ack_ticket)
        .await
        .expect("should store ack key");
}

async fn onboard_node<M: Middleware>(
    instances: &ContractInstances<M>,
    contract_deployer: &ChainKeypair,
    node_chain_key: &ChainKeypair,
) -> (Address, Address) {
    let client = instances.token.client();

    // Deploy Safe and Module for node
    let (module, safe) = chain_types::utils::deploy_one_safe_one_module_and_setup_for_testing(
        instances,
        client.clone(),
        contract_deployer,
    )
    .await
    .expect("could not deploy safe and module");

    // ----------------
    // Onboarding:
    // Include node to the module
    // Add announcement contract to be a target in the module
    // Mint HOPR tokens to the Safe
    // Approve token transfer for Channel contract

    // Include node to the module
    include_node_to_module_by_safe(
        client.clone(),
        safe,
        module,
        node_chain_key.public().to_address(),
        contract_deployer,
    )
    .await
    .expect("could not include node to module");

    // Add announcement as target into the module
    add_announcement_as_target(
        client.clone(),
        safe,
        module,
        instances.announcements.address().into(),
        contract_deployer,
    )
    .await
    .expect("could not add announcement to module");

    // Fund the node's Safe with 10 native token and 10 000 * 1e-18 HOPR token
    chain_types::utils::fund_node(safe, 10_u128.into(), 10_000_u128.into(), instances.token.clone()).await;

    // Fund node's address with 10 native token
    chain_types::utils::fund_node(
        node_chain_key.public().to_address(),
        10_u128.into(),
        0.into(),
        instances.token.clone(),
    )
    .await;

    // Approve token transfer for channels contract
    approve_channel_transfer_from_safe(
        client.clone(),
        safe,
        instances.token.address().into(),
        instances.channels.address().into(),
        contract_deployer,
    )
    .await
    .expect("could not approve channels to be a spender for safe");

    (module, safe)
}

type TestRpc = RpcOperations<JsonRpcProviderClient<SurfRequestor, SimpleJsonRpcRetryPolicy>>;

type TestDb = CoreEthereumDb<SqliteShim<'static>>;

struct ChainNode {
    chain_key: ChainKeypair,
    db: Arc<RwLock<TestDb>>,
    actions: CoreEthereumActions<TestDb>,
    _indexer: Indexer<TestRpc, ContractEventHandlers<TestDb>, TestDb>,
    node_tasks: Vec<JoinHandle<()>>,
}

#[allow(clippy::too_many_arguments)]
async fn start_node_chain_logic(
    chain_key: &ChainKeypair,
    anvil: &AnvilInstance,
    contract_addrs: ContractAddresses,
    module_addr: Address,
    safe_addr: Address,
    rpc_cfg: RpcOperationsConfig,
    actions_cfg: ActionQueueConfig,
    indexer_cfg: IndexerConfig,
) -> ChainNode {
    // DB
    let inner_db = DB::new(CurrentDbShim::new_in_memory().await);
    let db = Arc::new(RwLock::new(CoreEthereumDb::new(
        inner_db,
        chain_key.public().to_address(),
    )));

    // RPC
    let json_rpc_client = JsonRpcProviderClient::new(
        &anvil.endpoint(),
        SurfRequestor::default(),
        SimpleJsonRpcRetryPolicy::default(),
    );

    let rpc_ops = RpcOperations::new(json_rpc_client, chain_key, rpc_cfg).expect("failed to create RpcOperations");

    // Transaction executor
    let eth_client = RpcEthereumClient::new(rpc_ops.clone(), RpcEthereumClientConfig::default());
    let tx_exec = EthereumTransactionExecutor::new(
        eth_client,
        SafePayloadGenerator::new(chain_key, contract_addrs, module_addr),
    );

    // Actions
    let action_queue = ActionQueue::new(db.clone(), IndexerActionTracker::default(), tx_exec, actions_cfg);
    let action_state = action_queue.action_state();
    let actions = CoreEthereumActions::new(chain_key.clone(), db.clone(), action_queue.new_sender());

    let mut node_tasks = Vec::new();

    node_tasks.push(async_std::task::spawn(action_queue.action_loop()));

    // Action state tracking
    let (sce_tx, mut sce_rx) = futures::channel::mpsc::unbounded();
    node_tasks.push(async_std::task::spawn(async move {
        while let Some(sce) = sce_rx.next().await {
            let res = action_state.match_and_resolve(&sce).await;
            debug!("{:?}: expectations resolved {:?}", sce, res);
        }
    }));

    // Indexer
    let chain_log_handler =
        ContractEventHandlers::new(contract_addrs, safe_addr, chain_key.public().to_address(), db.clone());

    let mut indexer = Indexer::new(rpc_ops.clone(), chain_log_handler, db.clone(), indexer_cfg, sce_tx);
    indexer.start().await.expect("indexer should sync");

    ChainNode {
        chain_key: chain_key.clone(),
        db,
        actions,
        _indexer: indexer,
        node_tasks,
    }
}

#[async_std::test]
async fn integration_test_indexer() {
    let _ = env_logger::builder().is_test(true).try_init();

    let block_time = Duration::from_secs(1);
    let anvil = create_anvil(Some(block_time));
    let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();

    let alice_chain_key = ChainKeypair::from_secret(anvil.keys()[1].to_bytes().as_ref()).unwrap();
    let bob_chain_key = ChainKeypair::from_secret(anvil.keys()[2].to_bytes().as_ref()).unwrap();

    let client = create_rpc_client_to_anvil(SurfRequestor::default(), &anvil, &contract_deployer);
    let instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
        .await
        .expect("failed to deploy");

    // Mint some tokens
    chain_types::utils::mint_tokens(instances.token.clone(), 1_000_000_u128.into()).await;

    let contract_addrs = ContractAddresses::from(&instances);

    let finality = 2;
    async_std::task::sleep((1 + finality) * block_time).await;

    // ----------------------------------------

    let mut rpc_cfg = RpcOperationsConfig {
        chain_id: anvil.chain_id(),
        finality,
        contract_addrs,
        module_address: Address::default(),
        expected_block_time: block_time,
        tx_polling_interval: Duration::from_millis(100),
        max_block_range_fetch_size: 100,
        min_block_range_fetch_size: 3,
    };

    let actions_cfg = ActionQueueConfig {
        max_action_confirmation_wait: Duration::from_secs(60), // lower action confirmation limit
    };

    let indexer_cfg = IndexerConfig {
        start_block_number: 1,
        fetch_token_transactions: true,
    };

    // Setup ALICE
    info!("Setting up ALICE");
    let (alice_module_addr, alice_safe_addr) = onboard_node(&instances, &contract_deployer, &alice_chain_key).await;

    rpc_cfg.module_address = alice_module_addr;

    let alice_node = start_node_chain_logic(
        &alice_chain_key,
        &anvil,
        contract_addrs,
        alice_module_addr,
        alice_safe_addr,
        rpc_cfg,
        actions_cfg,
        indexer_cfg,
    )
    .await;

    // Setup BOB
    info!("Setting up BOB");
    let (bob_module_addr, bob_safe_addr) = onboard_node(&instances, &contract_deployer, &bob_chain_key).await;

    rpc_cfg.module_address = bob_module_addr;

    let bob_node = start_node_chain_logic(
        &bob_chain_key,
        &anvil,
        contract_addrs,
        bob_module_addr,
        bob_safe_addr,
        rpc_cfg,
        actions_cfg,
        indexer_cfg,
    )
    .await;
    // Bob fund channel with 100 HOPR
    let incoming_funding_amount = BalanceType::HOPR.balance(100);
    bob_node
        .actions
        .register_safe_by_node(bob_safe_addr)
        .await
        .expect("should submit safe registration tx")
        .await
        .expect("should confirm safe registration");
    bob_node
        .actions
        .open_channel(alice_chain_key.public().to_address(), incoming_funding_amount)
        .await
        .expect("should submit incoming channel open tx")
        .await
        .expect("should confirm open incoming channel");

    info!("======== STARTING TEST ========");

    // ----------------
    // Test plan:
    // Register with Safe
    // Announce
    // Open channel to Bob
    // Redeem ticket in the channel
    // Close channel to Bob

    // Register Safe
    let confirmation = alice_node
        .actions
        .register_safe_by_node(alice_safe_addr)
        .await
        .expect("should submit safe registration tx")
        .await
        .expect("should confirm safe registration");

    assert!(
        matches!(confirmation.event, Some(ChainEventType::NodeSafeRegistered(reg_safe)) if reg_safe == alice_safe_addr),
        "confirmed safe address must match"
    );

    // Announce the node
    let maddr: Multiaddr = "/ip4/127.0.0.1/tcp/10000".parse().unwrap();
    let offchain_key = OffchainKeypair::random();
    let confirmation = alice_node
        .actions
        .announce(&[maddr.clone()], &offchain_key)
        .await
        .expect("should submit announcement tx")
        .await
        .expect("should confirm announcement");

    assert!(
        matches!(confirmation.event,
            Some(ChainEventType::Announcement{ peer, address, multiaddresses })
            if peer == offchain_key.public().into() &&
            address == alice_chain_key.public().to_address() &&
            multiaddresses.contains(&maddr)
        ),
        "confirmed announcement must match"
    );

    // Open channel (from Alice to Bob) with 1 HOPR
    let initial_channel_funds = BalanceType::HOPR.balance(1);
    let confirmation = alice_node
        .actions
        .open_channel(bob_chain_key.public().to_address(), initial_channel_funds)
        .await
        .expect("should submit channel open tx")
        .await
        .expect("should confirm open channel");

    // Delay the fetch, so that channel increase can be processed first
    async_std::task::sleep(Duration::from_millis(100)).await;

    let channel_alice_bob = alice_node
        .db
        .read()
        .await
        .get_channel_to(&bob_chain_key.public().to_address())
        .await
        .expect("db call should not fail")
        .expect("should contain a channel to Bob");

    match confirmation.event {
        Some(ChainEventType::ChannelOpened(channel)) => {
            assert_eq!(
                channel.get_id(),
                channel_alice_bob.get_id(),
                "channel in the DB must match the confirmed action"
            );
        }
        _ => panic!("invalid confirmation"),
    };

    assert_eq!(ChannelStatus::Open, channel_alice_bob.status, "channel must be opened");
    assert_eq!(
        U256::from(1),
        channel_alice_bob.balance.amount(),
        "channel must have the correct balance"
    );

    // Fund the channel from Alice to Bob with additional 99 HOPR
    let funding_amount = BalanceType::HOPR.balance(99);
    let confirmation = alice_node
        .actions
        .fund_channel(channel_alice_bob.get_id(), funding_amount)
        .await
        .expect("should submit fund channel tx")
        .await
        .expect("should confirm fund channel");

    match confirmation.event {
        Some(ChainEventType::ChannelBalanceIncreased(channel, amount)) => {
            assert_eq!(
                channel.get_id(),
                channel_alice_bob.get_id(),
                "channel in the DB must match the confirmed action"
            );
            assert_eq!(funding_amount, amount, "invalid balance increase");
        }
        _ => panic!("invalid confirmation"),
    };

    // after the second funding, read channel_alice_bob again and compare its balance
    let channel_alice_bob = alice_node
        .db
        .read()
        .await
        .get_channel_to(&bob_chain_key.public().to_address())
        .await
        .expect("db call should not fail")
        .expect("should contain a channel to Bob");

    let channel_alice_bob_seen_by_bob = bob_node
        .db
        .read()
        .await
        .get_channel_from(&alice_chain_key.public().to_address())
        .await
        .expect("db call should not fail")
        .expect("should contain a channel from Alice");

    assert_eq!(
        channel_alice_bob.get_id(),
        channel_alice_bob_seen_by_bob.get_id(),
        "channel ids must match"
    );
    assert_eq!(
        channel_alice_bob.balance, channel_alice_bob_seen_by_bob.balance,
        "channel balance must match"
    );

    let channel_bob_alice = bob_node
        .db
        .read()
        .await
        .get_channel_to(&alice_chain_key.public().to_address())
        .await
        .expect("db call should not fail")
        .expect("should contain a channel to Alice from Bob");
    let ticket_price = Balance::new(1, BalanceType::HOPR);

    // Create ticket from Bob in Alice's DB
    generate_the_first_ack_ticket(&alice_node, &bob_chain_key, ticket_price, &instances).await;

    let alice_ack_tickets = alice_node
        .db
        .read()
        .await
        .get_acknowledged_tickets(Some(channel_bob_alice))
        .await
        .expect("get ack ticket call on Alice's db must not fail");

    assert_eq!(
        1,
        alice_ack_tickets.len(),
        "Alice must have a single acknowledged ticket"
    );

    let channel_balance_before_redeem = channel_alice_bob.balance;
    let channel_bob_alice = alice_node
        .db
        .read()
        .await
        .get_channel_from(&bob_chain_key.public().to_address())
        .await
        .expect("db call should not fail")
        .expect("should contain a channel from Bob");

    let (on_chain_channel_bob_alice_balance, _, _, _, _) = instances
        .channels
        .channels(channel_bob_alice.get_id().into())
        .call()
        .await
        .unwrap();
    let (on_chain_channel_alice_bob_balance, _, _, _, _) = instances
        .channels
        .channels(channel_alice_bob.get_id().into())
        .call()
        .await
        .unwrap();

    assert_eq!(
        channel_alice_bob.balance.amount(),
        on_chain_channel_alice_bob_balance.into(),
        "channel alice->bob balance (before ticket redemption) must match"
    );
    assert_eq!(
        100, on_chain_channel_alice_bob_balance,
        "channel alice->bob balance (before ticket redemption) must be 100"
    );
    assert_eq!(
        100, on_chain_channel_bob_alice_balance,
        "channel bob->alice balance (before ticket redemption) must be 100"
    );

    // Alice redeems ticket
    let confirmations = futures::future::try_join_all(
        alice_node
            .actions
            .redeem_tickets_with_counterparty(&bob_chain_key.public().to_address(), false)
            .await
            .expect("should submit redeem action"),
    )
    .await
    .expect("should redeem all tickets");

    assert_eq!(1, confirmations.len(), "Alice should redeem a single ticket");

    let alice_ack_tickets = alice_node
        .db
        .read()
        .await
        .get_acknowledged_tickets(Some(channel_bob_alice))
        .await
        .expect("get ack ticket call on Alice's db must not fail");

    assert_eq!(
        0,
        alice_ack_tickets.len(),
        "Alice must have no acknowledged tickets after redeeming"
    );

    let channel_bob_alice = alice_node
        .db
        .read()
        .await
        .get_channel_from(&bob_chain_key.public().to_address())
        .await
        .expect("db call should not fail")
        .expect("should contain a channel from Bob");

    let (on_chain_channel_bob_alice_balance, _, _, _, _) = instances
        .channels
        .channels(channel_bob_alice.get_id().into())
        .call()
        .await
        .unwrap();
    let (on_chain_channel_alice_bob_balance, _, _, _, _) = instances
        .channels
        .channels(channel_alice_bob.get_id().into())
        .call()
        .await
        .unwrap();

    assert_eq!(
        101, on_chain_channel_alice_bob_balance,
        "channel alice->bob balance (before ticket redemption) must be 101"
    );
    assert_eq!(
        99, on_chain_channel_bob_alice_balance,
        "channel bob->alice balance (after ticket redemption) must be 99"
    );
    assert_eq!(
        channel_bob_alice.balance.amount(),
        on_chain_channel_bob_alice_balance.into(),
        "channel bob->alice balance (after ticket redemption) must match"
    );
    assert_eq!(
        channel_balance_before_redeem - ticket_price,
        channel_bob_alice.balance,
        "channel balance must decrease after redeem"
    );

    // Close channel
    let confirmation = alice_node
        .actions
        .close_channel(bob_chain_key.public().to_address(), ChannelDirection::Outgoing, true)
        .await
        .expect("should submit channel close tx")
        .await
        .expect("should confirm close channel");

    match confirmation.event {
        Some(ChainEventType::ChannelClosureInitiated(channel)) => {
            let closing_channel_in_db = alice_node
                .db
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

    let channel_alice_bob = alice_node
        .db
        .read()
        .await
        .get_channel_to(&bob_chain_key.public().to_address())
        .await
        .expect("must get channel")
        .expect("channel to bob must exist");

    assert_eq!(
        ChannelStatus::PendingToClose,
        channel_alice_bob.status,
        "channel must be pending to close"
    );

    futures::future::join_all(alice_node.node_tasks.into_iter().map(|t| t.cancel())).await;
}
