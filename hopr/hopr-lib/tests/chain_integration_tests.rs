#[cfg(feature = "runtime-async-std")]
use async_std::task::{sleep, spawn, JoinHandle};

#[cfg(feature = "runtime-tokio")]
use tokio::{
    task::{spawn, JoinHandle},
    time::sleep,
};

use chain_actions::action_queue::{ActionQueue, ActionQueueConfig};
use chain_actions::action_state::{ActionState, IndexerActionTracker};
use chain_actions::channels::ChannelActions;
use chain_actions::node::NodeActions;
use chain_actions::payload::SafePayloadGenerator;
use chain_actions::redeem::TicketRedeemActions;
use chain_actions::ChainActions;
use chain_api::executors::{EthereumTransactionExecutor, RpcEthereumClient, RpcEthereumClientConfig};
use chain_indexer::{block::Indexer, handlers::ContractEventHandlers, IndexerConfig};
use chain_rpc::client::surf_client::SurfRequestor;
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
use futures::{pin_mut, StreamExt};
use hopr_crypto_types::prelude::*;
use hopr_db_api::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use std::time::Duration;
use tracing::info;

#[cfg(feature = "runtime-async-std")]
async fn cancel_join_handle<T>(handle: JoinHandle<T>) {
    handle.cancel().await
}

#[cfg(feature = "runtime-tokio")]
async fn cancel_join_handle<T>(handle: JoinHandle<T>) {
    handle.abort()
}

// Helper function to generate the first acked ticket (channel_epoch 1, index 0, offset 0) of win prob 100%
async fn generate_the_first_ack_ticket<M: Middleware>(
    myself: &ChainNode,
    counterparty: &ChainKeypair,
    price: Balance,
    instances: &ContractInstances<M>,
) {
    let hk1 = HalfKey::random();
    let hk2 = HalfKey::random();

    let cp1: CurvePoint = hk1.to_challenge().try_into().unwrap();
    let cp2: CurvePoint = hk2.to_challenge().try_into().unwrap();
    let cp_sum = CurvePoint::combine(&[&cp1, &cp2]);

    let domain_separator: Hash = instances.channels.domain_separator().call().await.unwrap().into();

    let ack_ticket = TicketBuilder::default()
        .addresses(counterparty, &myself.chain_key)
        .balance(price)
        .index(0)
        .index_offset(1)
        .win_prob(1.0)
        .channel_epoch(1)
        .challenge(Challenge::from(cp_sum).into())
        .build_signed(counterparty, &domain_separator)
        .unwrap()
        .into_acknowledged(Response::from_half_keys(&hk1, &hk2).unwrap());

    myself
        .db
        .upsert_ticket(None, ack_ticket)
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

struct ChainNode {
    chain_key: ChainKeypair,
    offchain_key: OffchainKeypair,
    db: HoprDb,
    actions: ChainActions<HoprDb>,
    _indexer: Indexer<TestRpc, ContractEventHandlers<HoprDb>, HoprDb>,
    node_tasks: Vec<JoinHandle<()>>,
}

#[allow(clippy::too_many_arguments)]
async fn start_node_chain_logic(
    chain_key: &ChainKeypair,
    offchain_key: &OffchainKeypair,
    anvil: &AnvilInstance,
    contract_addrs: ContractAddresses,
    module_addr: Address,
    safe_addr: Address,
    rpc_cfg: RpcOperationsConfig,
    actions_cfg: ActionQueueConfig,
    indexer_cfg: IndexerConfig,
) -> ChainNode {
    // DB
    let db = HoprDb::new_in_memory(chain_key.clone()).await;
    let self_db = db.clone();
    let ock = offchain_key.public().clone();
    let ckp = chain_key.public().to_address().clone();
    db.begin_transaction()
        .await
        .unwrap()
        .perform(|tx| {
            Box::pin(async move {
                self_db
                    .set_domain_separator(Some(tx), DomainSeparator::Channel, Hash::default())
                    .await?;
                self_db
                    .insert_account(Some(tx), AccountEntry::new(ock, ckp, AccountType::NotAnnounced))
                    .await
            })
        })
        .await
        .unwrap();

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
    let actions = ChainActions::new(chain_key, db.clone(), action_queue.new_sender());

    let mut node_tasks = Vec::new();

    node_tasks.push(spawn(async move {
        action_queue.start().await;
    }));

    // Action state tracking
    let (sce_tx, sce_rx) = async_channel::unbounded();
    node_tasks.push(spawn(async move {
        let rx = sce_rx.clone();
        pin_mut!(rx);

        while let Some(sce) = rx.next().await {
            let _ = action_state.match_and_resolve(&sce).await;
            //debug!("{:?}: expectations resolved {:?}", sce, res);
        }
    }));

    // Indexer
    let chain_log_handler = ContractEventHandlers::new(contract_addrs, safe_addr, chain_key.clone(), db.clone());

    let mut indexer = Indexer::new(rpc_ops.clone(), chain_log_handler, db.clone(), indexer_cfg, sce_tx);
    indexer.start().await.expect("indexer should sync");

    ChainNode {
        offchain_key: offchain_key.clone(),
        chain_key: chain_key.clone(),
        db,
        actions,
        _indexer: indexer,
        node_tasks,
    }
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn integration_test_indexer() {
    let _ = env_logger::builder().is_test(true).try_init();

    let block_time = Duration::from_secs(1);
    let anvil = create_anvil(Some(block_time));
    let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();

    let alice_chain_key = ChainKeypair::from_secret(anvil.keys()[1].to_bytes().as_ref()).unwrap();
    let bob_chain_key = ChainKeypair::from_secret(anvil.keys()[2].to_bytes().as_ref()).unwrap();

    let alice_offchain_key = OffchainKeypair::random();
    let bob_offchain_key = OffchainKeypair::random();

    let client = create_rpc_client_to_anvil(SurfRequestor::default(), &anvil, &contract_deployer);
    info!("Deploying SCs to Anvil...");
    let instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
        .await
        .expect("failed to deploy");

    // Mint some tokens
    chain_types::utils::mint_tokens(instances.token.clone(), 1_000_000_u128.into()).await;

    let contract_addrs = ContractAddresses::from(&instances);

    let finality = 2;
    sleep((1 + finality) * block_time).await;

    // ----------------------------------------

    let mut rpc_cfg = RpcOperationsConfig {
        chain_id: anvil.chain_id(),
        finality,
        contract_addrs,
        module_address: Address::default(),
        expected_block_time: block_time,
        tx_polling_interval: Duration::from_millis(100),
        max_block_range_fetch_size: 100,
    };

    let actions_cfg = ActionQueueConfig {
        max_action_confirmation_wait: Duration::from_secs(60), // lower action confirmation limit
    };

    let indexer_cfg = IndexerConfig { start_block_number: 1 };

    // Setup ALICE
    info!("Setting up ALICE");
    let (alice_module_addr, alice_safe_addr) = onboard_node(&instances, &contract_deployer, &alice_chain_key).await;

    rpc_cfg.module_address = alice_module_addr;

    let alice_node = start_node_chain_logic(
        &alice_chain_key,
        &alice_offchain_key,
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
        &bob_offchain_key,
        &anvil,
        contract_addrs,
        bob_module_addr,
        bob_safe_addr,
        rpc_cfg,
        actions_cfg,
        indexer_cfg,
    )
    .await;

    info!("======== STARTING TEST ========");

    // ----------------
    // Test plan:
    // Register Safe for both nodes
    // Announce both nodes
    // Open channel to Bob
    // Fund channel to Bob
    // Open channel to Alice
    // Redeem ticket in the channel
    // Close channel to Bob

    // Register Safe Alice
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
    info!("--> Alice's Safe has been registered");

    // Register Safe Bob
    let confirmation = bob_node
        .actions
        .register_safe_by_node(bob_safe_addr)
        .await
        .expect("should submit safe registration tx")
        .await
        .expect("should confirm safe registration");

    assert!(
        matches!(confirmation.event, Some(ChainEventType::NodeSafeRegistered(reg_safe)) if reg_safe == bob_safe_addr),
        "confirmed safe address must match"
    );
    info!("--> Bob's Safe has been registered");

    // Announce the node by Alice
    let maddr: Multiaddr = "/ip4/127.0.0.1/tcp/10000".parse().unwrap();
    let confirmation = alice_node
        .actions
        .announce(&[maddr.clone()], &alice_node.offchain_key)
        .await
        .expect("should submit announcement tx")
        .await
        .expect("should confirm announcement");

    assert!(
        matches!(confirmation.event,
            Some(ChainEventType::Announcement{ peer, address, multiaddresses })
            if peer == alice_node.offchain_key.public().into() &&
            address == alice_chain_key.public().to_address() &&
            multiaddresses.contains(&maddr)
        ),
        "confirmed announcement must match"
    );
    info!(
        "--> Alice's node {} has been announced as {maddr}",
        alice_chain_key.public().to_address()
    );

    // Announce the node by Bob
    let maddr: Multiaddr = "/ip4/127.0.0.1/tcp/20000".parse().unwrap();
    let confirmation = bob_node
        .actions
        .announce(&[maddr.clone()], &bob_node.offchain_key)
        .await
        .expect("should submit announcement tx")
        .await
        .expect("should confirm announcement");

    assert!(
        matches!(confirmation.event,
            Some(ChainEventType::Announcement{ peer, address, multiaddresses })
            if peer == bob_node.offchain_key.public().into() &&
            address == bob_chain_key.public().to_address() &&
            multiaddresses.contains(&maddr)
        ),
        "confirmed announcement must match"
    );
    info!(
        "--> Bob's node {} has been announced as {maddr}",
        bob_chain_key.public().to_address()
    );

    sleep(Duration::from_millis(1000)).await;

    assert_eq!(
        Some(alice_node.chain_key.public().to_address()),
        bob_node
            .db
            .resolve_chain_key(alice_node.offchain_key.public())
            .await
            .expect("resolve should not fail"),
        "bob must resolve alice's chain key correctly"
    );

    assert_eq!(
        Some(*alice_node.offchain_key.public()),
        bob_node
            .db
            .resolve_packet_key(&alice_node.chain_key.public().to_address())
            .await
            .expect("resolve should not fail"),
        "bob must resolve alice's offchain key correctly"
    );

    assert_eq!(
        Some(bob_node.chain_key.public().to_address()),
        alice_node
            .db
            .resolve_chain_key(bob_node.offchain_key.public())
            .await
            .expect("resolve should not fail"),
        "alice must resolve bob's chain key correctly"
    );

    assert_eq!(
        Some(*bob_node.offchain_key.public()),
        alice_node
            .db
            .resolve_packet_key(&bob_node.chain_key.public().to_address())
            .await
            .expect("resolve should not fail"),
        "alice must resolve bob's offchain key correctly"
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
    sleep(Duration::from_millis(100)).await;

    let channel_alice_bob = alice_node
        .db
        .get_channel_by_id(
            None,
            &generate_channel_id(
                &alice_chain_key.public().to_address(),
                &bob_chain_key.public().to_address(),
            ),
        )
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
            info!("--> successfully opened channel Alice -> Bob: {channel}");
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
            info!("--> successfully opened channel Alice -> Bob with 99 HOPR: {channel}");
        }
        _ => panic!("invalid confirmation"),
    };

    sleep(Duration::from_millis(1000)).await;

    // After the funding, read channel_alice_bob again and compare its balance
    let channel_alice_bob = alice_node
        .db
        .get_channel_by_id(
            None,
            &generate_channel_id(
                &alice_chain_key.public().to_address(),
                &bob_chain_key.public().to_address(),
            ),
        )
        .await
        .expect("db call should not fail")
        .expect("should contain a channel to Bob");

    let channel_alice_bob_seen_by_bob = bob_node
        .db
        .get_channel_by_id(
            None,
            &generate_channel_id(
                &alice_chain_key.public().to_address(),
                &bob_chain_key.public().to_address(),
            ),
        )
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

    // Bob fund channel with 100 HOPR
    let incoming_funding_amount = BalanceType::HOPR.balance(100);

    let confirmation = bob_node
        .actions
        .open_channel(alice_chain_key.public().to_address(), incoming_funding_amount)
        .await
        .expect("should submit incoming channel open tx")
        .await
        .expect("should confirm open incoming channel");

    match confirmation.event {
        Some(ChainEventType::ChannelOpened(channel)) => {
            assert_eq!(
                channel.get_id(),
                generate_channel_id(
                    &bob_chain_key.public().to_address(),
                    &alice_chain_key.public().to_address(),
                ),
                "channel in the DB must match the confirmed action"
            );
            info!("--> successfully opened channel Bob -> Alice: {channel}");
        }
        _ => panic!("invalid confirmation"),
    };

    sleep(Duration::from_millis(1000)).await;

    let channel_bob_alice = alice_node
        .db
        .get_channel_by_id(
            None,
            &generate_channel_id(
                &bob_chain_key.public().to_address(),
                &alice_chain_key.public().to_address(),
            ),
        )
        .await
        .expect("db call should not fail")
        .expect("should contain a channel to Bob");

    let ticket_price = Balance::new(1, BalanceType::HOPR);

    // Create ticket from Alice in Bob's DB
    generate_the_first_ack_ticket(&bob_node, &alice_chain_key, ticket_price, &instances).await;

    let bob_ack_tickets = bob_node
        .db
        .get_tickets(None, channel_alice_bob_seen_by_bob.into())
        .await
        .expect("get ack ticket call on Alice's db must not fail");

    assert_eq!(1, bob_ack_tickets.len(), "Bob must have a single acknowledged ticket");
    info!("--> successfully created acknowledged winning ticket by Alice for Bob");

    let channel_alice_bob_balance_before_redeem = channel_alice_bob.balance;
    let channel_alice_bob = alice_node
        .db
        .get_channel_by_id(
            None,
            &generate_channel_id(
                &alice_chain_key.public().to_address(),
                &bob_chain_key.public().to_address(),
            ),
        )
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

    // Bob redeems ticket
    let confirmations = futures::future::try_join_all(
        bob_node
            .actions
            .redeem_tickets_with_counterparty(&alice_chain_key.public().to_address(), false)
            .await
            .expect("should submit redeem action"),
    )
    .await
    .expect("should redeem all tickets");

    assert_eq!(1, confirmations.len(), "Bob should redeem a single ticket");

    match &confirmations.first().unwrap().event {
        Some(ChainEventType::TicketRedeemed(channel, ack_ticket)) => {
            assert_eq!(
                channel.get_id(),
                channel_alice_bob.get_id(),
                "channel in the DB must match the confirmed action"
            );
            let ack_ticket = ack_ticket.clone().expect("event must contain ack ticket");
            assert_eq!(
                ack_ticket.verified_ticket().channel_id,
                channel_alice_bob.get_id(),
                "channel id on ticket must match"
            );
            assert_eq!(0, ack_ticket.verified_ticket().index, "ticket index must match");

            info!("--> Bob successfully redeemed {ack_ticket}");
        }
        _ => panic!("invalid confirmation"),
    };

    sleep(Duration::from_millis(1000)).await;

    let bob_ack_tickets = alice_node
        .db
        .get_tickets(None, channel_bob_alice.into())
        .await
        .expect("get ack ticket call on Alice's db must not fail");

    assert_eq!(
        0,
        bob_ack_tickets.len(),
        "Bob must have no acknowledged tickets after redeeming"
    );

    let channel_bob_alice = alice_node
        .db
        .get_channel_by_id(
            None,
            &generate_channel_id(
                &bob_chain_key.public().to_address(),
                &alice_chain_key.public().to_address(),
            ),
        )
        .await
        .expect("db call should not fail")
        .expect("should contain a channel from Alice");

    let channel_alice_bob = alice_node
        .db
        .get_channel_by_id(
            None,
            &generate_channel_id(
                &alice_chain_key.public().to_address(),
                &bob_chain_key.public().to_address(),
            ),
        )
        .await
        .expect("db call should not fail")
        .expect("should contain a channel from Alice");

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
        "channel alice->bob balance (after ticket redemption) must match"
    );

    assert_eq!(
        channel_alice_bob_balance_before_redeem - ticket_price,
        BalanceType::HOPR.balance(on_chain_channel_alice_bob_balance),
        "channel alice->bob balance (after ticket redemption) must be decreased"
    );

    // Channel balances were the same on both channels before redeeming
    assert_eq!(
        channel_alice_bob_balance_before_redeem + ticket_price,
        BalanceType::HOPR.balance(on_chain_channel_bob_alice_balance),
        "channel bob->alice balance (after ticket redemption) must be increase"
    );

    info!("--> successfully passed all tests after redemption");

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
                .get_channel_by_id(
                    None,
                    &generate_channel_id(
                        &alice_chain_key.public().to_address(),
                        &bob_chain_key.public().to_address(),
                    ),
                )
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

    sleep(Duration::from_millis(1000)).await;

    let channel_alice_bob = alice_node
        .db
        .get_channel_by_id(
            None,
            &generate_channel_id(
                &alice_chain_key.public().to_address(),
                &bob_chain_key.public().to_address(),
            ),
        )
        .await
        .expect("must get channel")
        .expect("channel to bob must exist");

    assert!(
        matches!(channel_alice_bob.status, ChannelStatus::PendingToClose(_)),
        "channel must be pending to close"
    );

    info!("--> successfully initiated channel closure for Alice -> Bob");

    let alice_checksum = alice_node.db.get_last_indexed_block(None).await.unwrap();
    let bob_checksum = bob_node.db.get_last_indexed_block(None).await.unwrap();
    info!("alice completed at {:?}", alice_checksum);
    info!("bob completed at {:?}", bob_checksum);

    assert_eq!(
        alice_checksum.1, bob_checksum.1,
        "alice and bob must be at the same checksum"
    );

    futures::future::join_all(alice_node.node_tasks.into_iter().map(|t| cancel_join_handle(t))).await;
}
