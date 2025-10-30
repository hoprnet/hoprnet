mod common;

use std::{env, path::Path, time::Duration};

use alloy::primitives::{B256, U256};
use common::create_rpc_client_to_anvil_with_snapshot;
use futures::{StreamExt, pin_mut};
use hex_literal::hex;
use hopr_api::db::{HoprDbTicketOperations, TicketSelector};
use hopr_async_runtime::{AbortHandle, prelude::sleep, spawn_as_abortable};
use hopr_chain_actions::{
    ChainActions,
    action_queue::{ActionQueue, ActionQueueConfig},
    action_state::{ActionState, IndexerActionTracker},
    channels::ChannelActions,
    node::NodeActions,
    payload::SafePayloadGenerator,
    redeem::TicketRedeemActions,
};
use hopr_chain_api::{
    DefaultHttpRequestor,
    executors::{EthereumTransactionExecutor, RpcEthereumClient, RpcEthereumClientConfig},
};
use hopr_chain_indexer::{IndexerConfig, block::Indexer, handlers::ContractEventHandlers};
use hopr_chain_rpc::{
    client::SnapshotRequestor,
    rpc::{RpcOperations, RpcOperationsConfig},
};
use hopr_chain_types::{ContractAddresses, chain_events::ChainEvent, utils::create_anvil};
use hopr_crypto_types::prelude::*;
use hopr_db_node::HoprNodeDb;
use hopr_db_sql::{logs::HoprDbLogOperations, prelude::*};
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use hopr_transport::{ChainKeypair, Hash, Keypair, Multiaddr, OffchainKeypair};
use tokio::fs;
use tracing::info;

use crate::common::{NodeSafeConfig, TestChainEnv, deploy_test_environment, onboard_node};

// Helper function to generate the first acked ticket (channel_epoch 1, index 0, offset 0) of win prob 100%
async fn generate_the_first_ack_ticket(
    myself: &ChainNode,
    counterparty: &ChainKeypair,
    price: HoprBalance,
    domain_separator: Hash,
) -> anyhow::Result<()> {
    let hk1 = HalfKey::try_from(hex!("16e1d5a405315958b7db2d70ed797d858c9e6ba979783cf5110c13e0200ab0d0").as_ref())?;
    let hk2 = HalfKey::try_from(hex!("bc580f2aad36f35419d5936cc3256e2eb4a7a5f42c934b91a94305da8c4f7e81").as_ref())?;

    let challenge = Response::from_half_keys(&hk1, &hk2)?.to_challenge()?;

    let ack_ticket = TicketBuilder::default()
        .addresses(counterparty, &myself.chain_key)
        .balance(price)
        .index(0)
        .index_offset(1)
        .win_prob(WinningProbability::ALWAYS)
        .channel_epoch(1)
        .challenge(challenge)
        .build_signed(counterparty, &domain_separator)?
        .into_acknowledged(Response::from_half_keys(&hk1, &hk2)?);

    myself.node_db.upsert_ticket(ack_ticket).await?;

    Ok(())
}

type TestRpc = RpcOperations<DefaultHttpRequestor>;

struct ChainNode {
    chain_key: ChainKeypair,
    offchain_key: OffchainKeypair,
    index_db: HoprIndexerDb,
    node_db: HoprNodeDb,
    actions: ChainActions<HoprNodeDb>,
    _indexer: Indexer<TestRpc, ContractEventHandlers<TestRpc, HoprNodeDb>>,
    node_tasks: Vec<AbortHandle>,
}

#[allow(clippy::too_many_arguments)]
async fn start_node_chain_logic(
    chain_key: &ChainKeypair,
    offchain_key: &OffchainKeypair,
    requestor_in: SnapshotRequestor,
    requestor_out: SnapshotRequestor,
    chain_env: &TestChainEnv,
    safe_cfg: NodeSafeConfig,
    mut rpc_cfg: RpcOperationsConfig,
    actions_cfg: ActionQueueConfig,
    indexer_cfg: IndexerConfig,
) -> anyhow::Result<ChainNode> {
    // DB
    let index_db = HoprIndexerDb::new_in_memory(chain_key.clone()).await?;
    let node_db = HoprNodeDb::new_in_memory(chain_key.clone()).await?;
    let self_db = index_db.clone();
    index_db
        .begin_transaction()
        .await?
        .perform(|tx| {
            Box::pin(async move {
                self_db
                    .set_domain_separator(Some(tx), DomainSeparator::Channel, Hash::default())
                    .await
            })
        })
        .await?;

    // RPC
    rpc_cfg.safe_address = safe_cfg.safe_address;
    rpc_cfg.module_address = safe_cfg.module_address;

    let http_requestor_in = DefaultHttpRequestor::new();
    let json_rpc_client = create_rpc_client_to_anvil_with_snapshot(requestor_in.clone(), &chain_env.anvil);
    let rpc_ops_in = RpcOperations::new(
        json_rpc_client,
        http_requestor_in.clone(),
        chain_key,
        rpc_cfg.clone(),
        None,
    )?;

    let http_requestor_out = DefaultHttpRequestor::new();
    let json_rpc_client = create_rpc_client_to_anvil_with_snapshot(requestor_out.clone(), &chain_env.anvil);
    let rpc_ops_out = RpcOperations::new(
        json_rpc_client,
        http_requestor_out.clone(),
        chain_key,
        rpc_cfg.clone(),
        None,
    )?;

    // Transaction executor
    let eth_client = RpcEthereumClient::new(rpc_ops_out, RpcEthereumClientConfig::default());
    let tx_exec = EthereumTransactionExecutor::new(
        eth_client,
        SafePayloadGenerator::new(chain_key, chain_env.contract_addresses, safe_cfg.module_address),
    );

    // Actions
    let action_queue = ActionQueue::new(node_db.clone(), IndexerActionTracker::default(), tx_exec, actions_cfg);
    let action_state = action_queue.action_state();
    let actions = ChainActions::new(chain_key, index_db.clone(), node_db.clone(), action_queue.new_sender());

    let mut node_tasks = Vec::new();

    node_tasks.push(spawn_as_abortable!(async move {
        action_queue.start().await;
    }));

    // Action state tracking
    let (sce_tx, sce_rx) = futures::channel::mpsc::channel(10_000);
    node_tasks.push(spawn_as_abortable!(async move {
        pin_mut!(sce_rx);

        while let Some(sce) = sce_rx.next().await {
            let _ = action_state.match_and_resolve(&sce).await;
            // debug!("{:?}: expectations resolved {:?}", sce, res);
        }
    }));

    // Indexer
    let chain_log_handler = ContractEventHandlers::new(
        chain_env.contract_addresses,
        safe_cfg.safe_address,
        chain_key.clone(),
        index_db.clone(),
        node_db.clone(),
        rpc_ops_in.clone(),
    );

    let indexer = Indexer::new(rpc_ops_in, chain_log_handler, index_db.clone(), indexer_cfg, sce_tx);
    let _indexer = indexer.clone();
    indexer.start().await?;

    Ok(ChainNode {
        offchain_key: offchain_key.clone(),
        chain_key: chain_key.clone(),
        index_db,
        node_db,
        actions,
        _indexer,
        node_tasks,
    })
}

const SNAPSHOT_BASE: &str = "tests/snapshots/indexer_snapshot_base";
const SNAPSHOT_ALICE_TX: &str = "tests/snapshots/indexer_snapshot_alice_out";
const SNAPSHOT_ALICE_RX: &str = "tests/snapshots/indexer_snapshot_alice_in";
const SNAPSHOT_BOB_TX: &str = "tests/snapshots/indexer_snapshot_bob_out";
const SNAPSHOT_BOB_RX: &str = "tests/snapshots/indexer_snapshot_bob_in";

// #[tracing_test::traced_test]
#[tokio::test]
async fn integration_test_indexer() -> anyhow::Result<()> {
    let block_time = Duration::from_secs(1);
    let anvil = create_anvil(Some(block_time));

    let alice_chain_key = ChainKeypair::from_secret(anvil.keys()[1].to_bytes().as_ref())?;
    let bob_chain_key = ChainKeypair::from_secret(anvil.keys()[2].to_bytes().as_ref())?;

    let alice_offchain_key = OffchainKeypair::from_secret(&hex!(
        "2ba8cd4c723083159c00464fe0e6d7dbb1f931383cec4a04d21b63e49f8a18cf"
    ))?;
    let bob_offchain_key = OffchainKeypair::from_secret(&hex!(
        "4166d6b8455a6be8aa0f41e6b1d0446ad95e744de1eb3e4e6e5af30ca27d7af5"
    ))?;

    if !hopr_crypto_random::is_rng_fixed() {
        tracing::warn!("snapshot based tests require fixed RNG")
    }

    let requestor_base = SnapshotRequestor::new(SNAPSHOT_BASE)
        .with_ignore_snapshot(!hopr_crypto_random::is_rng_fixed())
        .load(true)
        .await;
    let requestor_alice_rx = SnapshotRequestor::new(SNAPSHOT_ALICE_RX)
        .with_ignore_snapshot(!hopr_crypto_random::is_rng_fixed())
        .with_aggresive_save()
        .load(true)
        .await;
    let requestor_alice_tx = SnapshotRequestor::new(SNAPSHOT_ALICE_TX)
        .with_ignore_snapshot(!hopr_crypto_random::is_rng_fixed())
        .with_aggresive_save()
        .load(true)
        .await;
    let requestor_bob_rx = SnapshotRequestor::new(SNAPSHOT_BOB_RX)
        .with_ignore_snapshot(!hopr_crypto_random::is_rng_fixed())
        .with_aggresive_save()
        .load(true)
        .await;
    let requestor_bob_tx = SnapshotRequestor::new(SNAPSHOT_BOB_TX)
        .with_ignore_snapshot(!hopr_crypto_random::is_rng_fixed())
        .with_aggresive_save()
        .load(true)
        .await;

    let finality = 2;

    let chain_env = deploy_test_environment(requestor_base, block_time, finality).await;

    let mut safe_cfgs = [NodeSafeConfig::default(); 2];
    safe_cfgs[0] = onboard_node(&chain_env, &alice_chain_key, U256::from(10_u32), U256::from(10_000_u32)).await;
    safe_cfgs[1] = onboard_node(&chain_env, &bob_chain_key, U256::from(10_u32), U256::from(10_000_u32)).await;

    sleep((1 + finality) * block_time).await;

    let domain_separator: Hash = (*chain_env.contract_instances.channels.domainSeparator().call().await?).into();

    let rpc_cfg = RpcOperationsConfig {
        chain_id: chain_env.anvil.chain_id(),
        finality,
        contract_addrs: chain_env.contract_addresses,
        expected_block_time: block_time,
        tx_polling_interval: Duration::from_millis(100),
        max_block_range_fetch_size: 100,
        gas_oracle_url: None,
        ..RpcOperationsConfig::default()
    };

    let actions_cfg = ActionQueueConfig {
        max_action_confirmation_wait: Duration::from_secs(60), // lower action confirmation limit
    };

    let indexer_cfg = IndexerConfig::new(1, false, false, None, "/tmp/test_hopr_data".to_string());

    // Setup ALICE
    info!("Starting up ALICE");

    let alice_node = start_node_chain_logic(
        &alice_chain_key,
        &alice_offchain_key,
        requestor_alice_rx,
        requestor_alice_tx,
        &chain_env,
        safe_cfgs[0],
        rpc_cfg.clone(),
        actions_cfg,
        indexer_cfg.clone(),
    )
    .await?;

    // Setup BOB
    info!("Starting up BOB");

    let bob_node = start_node_chain_logic(
        &bob_chain_key,
        &bob_offchain_key,
        requestor_bob_rx,
        requestor_bob_tx,
        &chain_env,
        safe_cfgs[1],
        rpc_cfg,
        actions_cfg,
        indexer_cfg,
    )
    .await?;

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
        .register_safe_by_node(safe_cfgs[0].safe_address)
        .await
        .expect("should submit safe registration tx")
        .await
        .expect("should confirm safe registration");

    assert!(
        matches!(confirmation.event, Some(ChainEvent::NodeSafeRegistered(reg_safe)) if reg_safe == safe_cfgs[0].safe_address),
        "confirmed safe address must match"
    );
    info!("--> Alice's Safe has been registered");

    // Register Safe Bob
    let confirmation = bob_node
        .actions
        .register_safe_by_node(safe_cfgs[1].safe_address)
        .await
        .expect("should submit safe registration tx")
        .await
        .expect("should confirm safe registration");

    assert!(
        matches!(confirmation.event, Some(ChainEvent::NodeSafeRegistered(reg_safe)) if reg_safe == safe_cfgs[1].safe_address),
        "confirmed safe address must match"
    );
    info!("--> Bob's Safe has been registered");

    // Announce the node by Alice
    let maddr: Multiaddr = "/ip4/127.0.0.1/tcp/10000".parse()?;
    let confirmation = alice_node
        .actions
        .announce(&[maddr.clone()], &alice_node.offchain_key)
        .await
        .expect("should submit announcement tx")
        .await
        .expect("should confirm announcement");

    assert!(
        matches!(confirmation.event,
            Some(ChainEvent::Announcement{ peer, address, multiaddresses })
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
    let maddr: Multiaddr = "/ip4/127.0.0.1/tcp/20000".parse()?;
    let confirmation = bob_node
        .actions
        .announce(&[maddr.clone()], &bob_node.offchain_key)
        .await
        .expect("should submit announcement tx")
        .await
        .expect("should confirm announcement");

    assert!(
        matches!(confirmation.event,
            Some(ChainEvent::Announcement{ peer, address, multiaddresses })
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
            .index_db
            .resolve_chain_key(alice_node.offchain_key.public())
            .await?,
        "bob must resolve alice's chain key correctly"
    );

    assert_eq!(
        Some(*alice_node.offchain_key.public()),
        bob_node
            .index_db
            .resolve_packet_key(&alice_node.chain_key.public().to_address())
            .await?,
        "bob must resolve alice's offchain key correctly"
    );

    assert_eq!(
        Some(bob_node.chain_key.public().to_address()),
        alice_node
            .index_db
            .resolve_chain_key(bob_node.offchain_key.public())
            .await?,
        "alice must resolve bob's chain key correctly"
    );

    assert_eq!(
        Some(*bob_node.offchain_key.public()),
        alice_node
            .index_db
            .resolve_packet_key(&bob_node.chain_key.public().to_address())
            .await?,
        "alice must resolve bob's offchain key correctly"
    );

    // Open channel (from Alice to Bob) with 1 HOPR
    let initial_channel_funds = HoprBalance::from(1);
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
        .index_db
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
        Some(ChainEvent::ChannelOpened(channel)) => {
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
        U256::from_be_bytes(channel_alice_bob.balance.amount().to_be_bytes()),
        "channel must have the correct balance"
    );

    // Fund the channel from Alice to Bob with an additional 99 HOPR
    let funding_amount = HoprBalance::from(99);
    let confirmation = alice_node
        .actions
        .fund_channel(channel_alice_bob.get_id(), funding_amount)
        .await
        .expect("should submit fund channel tx")
        .await
        .expect("should confirm fund channel");

    match confirmation.event {
        Some(ChainEvent::ChannelBalanceIncreased(channel, amount)) => {
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

    sleep(Duration::from_millis(2000)).await;

    // After the funding, read channel_alice_bob again and compare its balance
    let channel_alice_bob = alice_node
        .index_db
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
        .index_db
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
    let incoming_funding_amount = HoprBalance::from(100);

    let confirmation = bob_node
        .actions
        .open_channel(alice_chain_key.public().to_address(), incoming_funding_amount)
        .await
        .expect("should submit incoming channel open tx")
        .await
        .expect("should confirm open incoming channel");

    match confirmation.event {
        Some(ChainEvent::ChannelOpened(channel)) => {
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
        .index_db
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

    let ticket_price = HoprBalance::from(1);

    // Create a ticket from Alice in Bob's DB
    generate_the_first_ack_ticket(&bob_node, &alice_chain_key, ticket_price, domain_separator).await?;

    let bob_ack_tickets = bob_node
        .node_db
        .stream_tickets(Some(channel_alice_bob_seen_by_bob.into()))
        .await
        .expect("get ack ticket call on Alice's db must not fail")
        .collect::<Vec<_>>()
        .await;

    assert_eq!(1, bob_ack_tickets.len(), "Bob must have a single acknowledged ticket");
    info!("--> successfully created acknowledged winning ticket by Alice for Bob");

    let channel_alice_bob_balance_before_redeem = channel_alice_bob.balance;
    let channel_alice_bob = alice_node
        .index_db
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

    let on_chain_channel_bob_alice_balance = chain_env
        .contract_instances
        .channels
        .channels(B256::from_slice(channel_bob_alice.get_id().as_ref()))
        .call()
        .await?
        .balance;
    let on_chain_channel_alice_bob_balance = chain_env
        .contract_instances
        .channels
        .channels(B256::from_slice(channel_alice_bob.get_id().as_ref()))
        .call()
        .await?
        .balance;

    assert_eq!(
        U256::from_be_bytes(channel_alice_bob.balance.amount().to_be_bytes()),
        on_chain_channel_alice_bob_balance.to::<U256>(),
        "channel alice->bob balance (before ticket redemption) must match"
    );

    assert_eq!(
        U256::from(100_u32),
        on_chain_channel_alice_bob_balance.to::<U256>(),
        "channel alice->bob balance (before ticket redemption) must be 100"
    );

    assert_eq!(
        U256::from(100_u32),
        on_chain_channel_bob_alice_balance.to::<U256>(),
        "channel bob->alice balance (before ticket redemption) must be 100"
    );

    // Bob redeems ticket
    let confirmations = futures::future::try_join_all(
        bob_node
            .actions
            .redeem_tickets(TicketSelector::from(&channel_alice_bob))
            .await
            .expect("should submit redeem action"),
    )
    .await
    .expect("should redeem all tickets");

    assert_eq!(1, confirmations.len(), "Bob should redeem a single ticket");

    match &confirmations.first().unwrap().event {
        Some(ChainEvent::TicketRedeemed(channel, ack_ticket)) => {
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
        .node_db
        .stream_tickets(Some(channel_bob_alice.into()))
        .await
        .expect("get ack ticket call on Alice's db must not fail")
        .collect::<Vec<_>>()
        .await;

    assert_eq!(
        0,
        bob_ack_tickets.len(),
        "Bob must have no acknowledged tickets after redeeming"
    );

    let channel_bob_alice = alice_node
        .index_db
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
        .index_db
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

    let on_chain_channel_bob_alice_balance = chain_env
        .contract_instances
        .channels
        .channels(B256::from_slice(channel_bob_alice.get_id().as_ref())) // .channels(channel_bob_alice.get_id().into())
        .call()
        .await?
        .balance;

    let on_chain_channel_alice_bob_balance = chain_env
        .contract_instances
        .channels
        .channels(B256::from_slice(channel_alice_bob.get_id().as_ref()))
        // .channels(channel_alice_bob.get_id().into())
        .call()
        .await?
        .balance;

    assert_eq!(
        U256::from_be_bytes(channel_alice_bob.balance.amount().to_be_bytes()),
        on_chain_channel_alice_bob_balance.to::<U256>(),
        // channel_alice_bob.balance.amount().to_be_bytes(),
        // on_chain_channel_alice_bob_balance.to_be_bytes(),
        "channel alice->bob balance (after ticket redemption) must match"
    );
    assert_eq!(
        U256::from_be_bytes(
            (channel_alice_bob_balance_before_redeem - ticket_price)
                .amount()
                .to_be_bytes()
        ),
        on_chain_channel_alice_bob_balance.to::<U256>(),
        "channel alice->bob balance (after ticket redemption) must be decreased"
    );
    // Channel balances were the same on both channels before redeeming
    assert_eq!(
        U256::from_be_bytes(
            (channel_alice_bob_balance_before_redeem + ticket_price)
                .amount()
                .to_be_bytes()
        ),
        on_chain_channel_bob_alice_balance.to::<U256>(),
        "channel bob->alice balance (after ticket redemption) must be increase"
    );

    info!("--> successfully passed all tests after redemption");

    // Close channel
    let confirmation = alice_node
        .actions
        .close_channel(channel_alice_bob.clone())
        .await
        .expect("should submit channel close tx")
        .await
        .expect("should confirm close channel");

    match confirmation.event {
        Some(ChainEvent::ChannelClosureInitiated(channel)) => {
            let closing_channel_in_db = alice_node
                .index_db
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
        .index_db
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

    let alice_checksum = alice_node
        .index_db
        .get_last_checksummed_log()
        .await?
        .ok_or_else(|| anyhow::anyhow!("alice must have a checksum"))?;
    let bob_checksum = bob_node
        .index_db
        .get_last_checksummed_log()
        .await?
        .ok_or_else(|| anyhow::anyhow!("bob must have a checksum"))?;
    info!("alice completed at {:?}", alice_checksum);
    info!("bob completed at {:?}", bob_checksum);

    assert_eq!(
        alice_checksum.checksum, bob_checksum.checksum,
        "alice and bob must be at the same checksum"
    );

    futures::future::join_all(alice_node.node_tasks.into_iter().map(|ah| async move { ah.abort() })).await;
    futures::future::join_all(bob_node.node_tasks.into_iter().map(|ah| async move { ah.abort() })).await;

    Ok(())
}

#[test_log::test(tokio::test)]
async fn integration_test_indexer_logs_snapshot_by_file() -> anyhow::Result<()> {
    // Setup test environment
    let id = hopr_crypto_random::random_integer(100_000, None);
    let temp_dir = env::temp_dir().join(format!("hopr_indexer_snapshot_test_{}", id));
    let data_directory = temp_dir.join("hopr_data");
    fs::create_dir_all(&data_directory).await?;

    let chain_key = ChainKeypair::random();
    let db = HoprIndexerDb::new(
        &data_directory.join("db"),
        chain_key.clone(),
        HoprIndexerDbConfig::default(),
    )
    .await?;
    let node_db = HoprNodeDb::new_in_memory(chain_key.clone()).await?;

    // Verify snapshot file exists
    let snapshot_file_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/log-snapshots/logs-snapshot.tar.xz");
    if !snapshot_file_path.exists() {
        anyhow::bail!("Snapshot test file not found at: {}", snapshot_file_path.display());
    }

    let logs_snapshot_url = format!("file://{}", snapshot_file_path.display());

    let indexer_cfg = IndexerConfig::new(
        0,
        true,
        true,
        Some(logs_snapshot_url),
        data_directory.to_string_lossy().to_string(),
    );

    let requestor_in = SnapshotRequestor::new(SNAPSHOT_ALICE_RX)
        .with_ignore_snapshot(!hopr_crypto_random::is_rng_fixed())
        .with_aggresive_save()
        .load(true)
        .await;

    let http_requestor_in = DefaultHttpRequestor::new();
    let anvil = alloy::node_bindings::Anvil::new().spawn();
    let json_rpc_client = create_rpc_client_to_anvil_with_snapshot(requestor_in.clone(), &anvil);
    let rpc = RpcOperations::new(
        json_rpc_client,
        http_requestor_in.clone(),
        &chain_key,
        RpcOperationsConfig::default(),
        None,
    )?;

    let handlers = ContractEventHandlers::new(
        ContractAddresses::default(),
        chain_key.public().to_address(),
        chain_key.clone(),
        db.clone(),
        node_db.clone(),
        rpc.clone(),
    );

    let indexer = Indexer::new(
        rpc,
        handlers,
        db.clone(),
        indexer_cfg,
        futures::channel::mpsc::channel(1000).0,
    );

    // Verify database is initially empty (as expected for snapshot test)
    let initial_logs_count = db.get_logs_count(None, None).await.unwrap_or(0);
    let initial_index_empty = db.index_is_empty().await?;

    // These conditions should be true for a fresh database that would benefit from snapshot
    assert_eq!(initial_logs_count, 0, "Fresh database should have no logs");
    assert!(initial_index_empty, "Fresh database index should be empty");

    // run snapshot fetch
    indexer.pre_start().await?;

    // now we can check if the logs were imported
    let logs_count = db.get_logs_count(None, None).await.unwrap_or(0);
    let index_empty = db.index_is_empty().await?;

    assert_eq!(logs_count, 72, "Imported database should have logs");
    assert!(index_empty, "Imported database index should be empty");

    Ok(())
}

#[test_log::test(tokio::test)]
async fn integration_test_indexer_logs_snapshot_by_http() -> anyhow::Result<()> {
    // Setup test environment
    let id = hopr_crypto_random::random_integer(100_000, None);
    let temp_dir = env::temp_dir().join(format!("hopr_indexer_snapshot_test_{}", id));
    let data_directory = temp_dir.join("hopr_data");
    fs::create_dir_all(&data_directory).await?;

    let chain_key = ChainKeypair::random();
    let db = HoprIndexerDb::new(
        &data_directory.join("db"),
        chain_key.clone(),
        HoprIndexerDbConfig::default(),
    )
    .await?;
    let node_db = HoprNodeDb::new_in_memory(chain_key.clone()).await?;

    // Verify snapshot file exists
    let snapshot_file_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/log-snapshots/logs-snapshot.tar.xz");
    if !snapshot_file_path.exists() {
        anyhow::bail!("Snapshot test file not found at: {}", snapshot_file_path.display());
    }

    let mut server = mockito::Server::new_async().await;
    let server_mock = server
        .mock("GET", "/logs-snapshot.tar.xz")
        .with_status(200)
        .with_body_from_file(snapshot_file_path.to_string_lossy().to_string())
        .expect(1)
        .create();

    let logs_snapshot_url = url::Url::parse(format!("{}/logs-snapshot.tar.xz", server.url()).as_str())?;

    let indexer_cfg = IndexerConfig::new(
        0,
        true,
        true,
        Some(logs_snapshot_url.into()),
        data_directory.to_string_lossy().to_string(),
    );

    let requestor_in = SnapshotRequestor::new(SNAPSHOT_ALICE_RX)
        .with_ignore_snapshot(!hopr_crypto_random::is_rng_fixed())
        .with_aggresive_save()
        .load(true)
        .await;

    let http_requestor_in = DefaultHttpRequestor::new();
    let anvil = alloy::node_bindings::Anvil::new().spawn();
    let json_rpc_client = create_rpc_client_to_anvil_with_snapshot(requestor_in.clone(), &anvil);
    let rpc = RpcOperations::new(
        json_rpc_client,
        http_requestor_in.clone(),
        &chain_key,
        RpcOperationsConfig::default(),
        None,
    )?;

    let handlers = ContractEventHandlers::new(
        ContractAddresses::default(),
        chain_key.public().to_address(),
        chain_key.clone(),
        db.clone(),
        node_db.clone(),
        rpc.clone(),
    );

    let indexer = Indexer::new(
        rpc,
        handlers,
        db.clone(),
        indexer_cfg,
        futures::channel::mpsc::channel(1000).0,
    );

    // Verify database is initially empty (as expected for snapshot test)
    let initial_logs_count = db.get_logs_count(None, None).await.unwrap_or(0);
    let initial_index_empty = db.index_is_empty().await?;

    // These conditions should be true for a fresh database that would benefit from snapshot
    assert_eq!(initial_logs_count, 0, "Fresh database should have no logs");
    assert!(initial_index_empty, "Fresh database index should be empty");

    // run snapshot fetch
    indexer.pre_start().await?;

    // now we can check if the logs were imported
    let logs_count = db.get_logs_count(None, None).await.unwrap_or(0);
    let index_empty = db.index_is_empty().await?;

    server_mock.assert();

    assert_eq!(logs_count, 72, "Imported database should have logs");
    assert!(index_empty, "Imported database index should be empty");

    Ok(())
}
