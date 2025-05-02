use async_std::task::sleep;
use ethers::utils::AnvilInstance;
use std::time::Duration;
use tracing::info;

use hopr_chain_rpc::client::surf_client::SurfRequestor;
use hopr_chain_rpc::client::{
    create_rpc_client_to_anvil, JsonRpcProviderClient, SimpleJsonRpcRetryPolicy, SnapshotRequestor,
};
use hopr_chain_types::utils::{
    add_announcement_as_target, approve_channel_transfer_from_safe, create_anvil, include_node_to_module_by_safe,
};
use hopr_chain_types::{ContractAddresses, ContractInstances};
use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;

pub type AnvilRpcClient<R> = ethers::middleware::SignerMiddleware<
    ethers::providers::Provider<JsonRpcProviderClient<R, SimpleJsonRpcRetryPolicy>>,
    ethers::signers::Wallet<ethers::core::k256::ecdsa::SigningKey>,
>;

/// Snapshot requestor used for testing.
pub type Requestor = SnapshotRequestor<SurfRequestor>;

/// Represents a HOPR environment deployment into Anvil.
#[allow(unused)]
pub struct TestChainEnv {
    /// Running Anvil instance
    pub anvil: AnvilInstance,
    /// Private key of smart contracts deployer
    pub contract_deployer: ChainKeypair,
    /// Chain keys of 9 possible HOPR nodes
    pub node_chain_keys: Vec<ChainKeypair>,
    /// Instances of deployed smart contracts
    pub contract_instances: ContractInstances<AnvilRpcClient<Requestor>>,
    /// Addresses of deployed smart contracts
    pub contract_addresses: ContractAddresses,
}

/// Deploys Anvil and all HOPR smart contracts as a testing environment
pub async fn deploy_test_environment(requestor: Requestor, block_time: Duration, finality: u32) -> TestChainEnv {
    let anvil = create_anvil(Some(block_time));
    let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();

    let client = create_rpc_client_to_anvil(requestor, &anvil, &contract_deployer);
    info!("Deploying SCs to Anvil...");
    let contract_instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
        .await
        .expect("failed to deploy");

    // Mint some tokens
    hopr_chain_types::utils::mint_tokens(contract_instances.token.clone(), 1_000_000_u128.into()).await;

    sleep((1 + finality) * block_time).await;

    TestChainEnv {
        contract_deployer,
        node_chain_keys: anvil
            .keys()
            .iter()
            .skip(1)
            .map(|k| ChainKeypair::from_secret(k.to_bytes().as_ref()).unwrap())
            .collect(),
        contract_addresses: ContractAddresses::from(&contract_instances),
        contract_instances,
        anvil,
    }
}

#[allow(unused)]
#[derive(Clone, Copy, Default)]
pub struct NodeSafeConfig {
    pub safe_address: Address,
    pub module_address: Address,
}

/// Onboards HOPR node by deploying its Safe and Module and funding them.
pub async fn onboard_node(
    chain_env: &TestChainEnv,
    node_chain_key: &ChainKeypair,
    fund_native: U256,
    fund_hopr: U256,
) -> NodeSafeConfig {
    let client = chain_env.contract_instances.token.client();

    // Deploy Safe and Module for node
    let (module, safe) = hopr_chain_types::utils::deploy_one_safe_one_module_and_setup_for_testing(
        &chain_env.contract_instances,
        client.clone(),
        &chain_env.contract_deployer,
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
        &chain_env.contract_deployer,
    )
    .await
    .expect("could not include node to module");

    // Add an announcement as target into the module
    add_announcement_as_target(
        client.clone(),
        safe,
        module,
        chain_env.contract_instances.announcements.address().into(),
        &chain_env.contract_deployer,
    )
    .await
    .expect("could not add announcement to module");

    // Fund the node's Safe with native tokens and HOPR token
    hopr_chain_types::utils::fund_node(safe, fund_native, fund_hopr, chain_env.contract_instances.token.clone()).await;

    // Fund node's address with 10 native tokens
    hopr_chain_types::utils::fund_node(
        node_chain_key.public().to_address(),
        fund_native,
        0.into(),
        chain_env.contract_instances.token.clone(),
    )
    .await;

    // Approve token transfer for HOPR Channels contract
    approve_channel_transfer_from_safe(
        client.clone(),
        safe,
        chain_env.contract_instances.token.address().into(),
        chain_env.contract_instances.channels.address().into(),
        &chain_env.contract_deployer,
    )
    .await
    .expect("could not approve channels to be a spender for safe");

    NodeSafeConfig {
        safe_address: safe,
        module_address: module,
    }
}
