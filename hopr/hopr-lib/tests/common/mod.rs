use std::{sync::Arc, time::Duration};

use alloy::{
    node_bindings::AnvilInstance, primitives::U256, rpc::client::RpcClient, transports::http::ReqwestTransport,
};
use hopr_chain_rpc::client::{AnvilRpcClient, SnapshotRequestor};
use hopr_chain_types::{
    ContractAddresses, ContractInstances,
    utils::{
        add_announcement_as_target, approve_channel_transfer_from_safe, create_anvil, include_node_to_module_by_safe,
    },
};
use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;
use tokio::time::sleep;
use tracing::info;

/// Used for testing. Creates RPC client to the local Anvil instance.
#[allow(dead_code)]
#[cfg(not(target_arch = "wasm32"))]
pub fn create_rpc_client_to_anvil_with_snapshot(
    snapshot_requestor: SnapshotRequestor,
    anvil: &alloy::node_bindings::AnvilInstance,
) -> RpcClient {
    use alloy::rpc::client::ClientBuilder;
    use hopr_chain_rpc::client::SnapshotRequestorLayer;

    let transport_client = ReqwestTransport::new(anvil.endpoint_url());

    ClientBuilder::default()
        .layer(SnapshotRequestorLayer::from_requestor(snapshot_requestor))
        .transport(transport_client.clone(), transport_client.guess_local())
}

/// Used for testing. Creates an RPC client to the local Anvil instance.
#[cfg(not(target_arch = "wasm32"))]
pub fn create_provider_to_anvil_with_snapshot(
    snapshot_requestor: SnapshotRequestor,
    anvil: &alloy::node_bindings::AnvilInstance,
    signer: &hopr_crypto_types::keypairs::ChainKeypair,
) -> Arc<AnvilRpcClient> {
    use alloy::{providers::ProviderBuilder, rpc::client::ClientBuilder, signers::local::PrivateKeySigner};
    use hopr_chain_rpc::client::SnapshotRequestorLayer;
    use hopr_crypto_types::keypairs::Keypair;

    let wallet = PrivateKeySigner::from_slice(signer.secret().as_ref()).expect("failed to construct wallet");

    let transport_client = ReqwestTransport::new(anvil.endpoint_url());

    let rpc_client = ClientBuilder::default()
        .layer(SnapshotRequestorLayer::from_requestor(snapshot_requestor))
        .transport(transport_client.clone(), transport_client.guess_local());

    let provider = ProviderBuilder::new().wallet(wallet).connect_client(rpc_client);

    Arc::new(provider)
}

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
    pub contract_instances: ContractInstances<Arc<AnvilRpcClient>>,
    /// Addresses of deployed smart contracts
    pub contract_addresses: ContractAddresses,
}

/// Deploys Anvil and all HOPR smart contracts as a testing environment
pub async fn deploy_test_environment(
    requestor: SnapshotRequestor,
    block_time: Duration,
    finality: u32,
) -> TestChainEnv {
    let anvil: AnvilInstance = create_anvil(Some(block_time));
    info!("Anvil started at {}", anvil.endpoint_url());

    let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();

    let provider = create_provider_to_anvil_with_snapshot(requestor, &anvil, &contract_deployer);
    info!("Deploying SCs to Anvil...");
    let contract_instances = ContractInstances::deploy_for_testing(provider.clone(), &contract_deployer)
        .await
        .expect("failed to deploy");

    // Mint some tokens
    let _ = hopr_chain_types::utils::mint_tokens(contract_instances.token.clone(), U256::from(1_000_000_u128)).await;

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
    let provider = chain_env.contract_instances.token.provider();

    // Deploy Safe and Module for node
    let (module, safe) =
        hopr_chain_types::utils::deploy_one_safe_one_module_and_setup_for_testing::<Arc<AnvilRpcClient>>(
            &chain_env.contract_instances,
            provider.clone(),
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
        provider.clone(),
        safe,
        module,
        node_chain_key.public().to_address(),
        &chain_env.contract_deployer,
    )
    .await
    .expect("could not include node to module");

    // Add an announcement as target into the module
    add_announcement_as_target(
        provider.clone(),
        safe,
        module,
        chain_env.contract_instances.announcements.address().0.0.into(),
        &chain_env.contract_deployer,
    )
    .await
    .expect("could not add announcement to module");

    // Fund the node's Safe with native tokens and HOPR token
    let _ =
        hopr_chain_types::utils::fund_node(safe, fund_native, fund_hopr, chain_env.contract_instances.token.clone())
            .await;

    // Fund node's address with 10 native tokens
    let _ = hopr_chain_types::utils::fund_node(
        node_chain_key.public().to_address(),
        fund_native,
        U256::from(0_u32),
        chain_env.contract_instances.token.clone(),
    )
    .await;

    // Approve token transfer for HOPR Channels contract
    approve_channel_transfer_from_safe(
        provider.clone(),
        safe,
        chain_env.contract_instances.token.address().0.0.into(),
        chain_env.contract_instances.channels.address().0.0.into(),
        &chain_env.contract_deployer,
    )
    .await
    .expect("could not approve channels to be a spender for safe");

    NodeSafeConfig {
        safe_address: safe,
        module_address: module,
    }
}
