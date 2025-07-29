//! Chain utilities used for testing.
//!
//! This used in unit and integration tests.

#![allow(clippy::too_many_arguments)]

use std::str::FromStr;

use SafeContract::SafeContractInstance;
use alloy::{
    contract::{Error as ContractError, Result as ContractResult},
    network::{ReceiptResponse, TransactionBuilder},
    primitives::{self, Bytes, U256, address, aliases, keccak256},
    rpc::types::TransactionRequest,
    signers::{Signer, local::PrivateKeySigner},
    sol,
    sol_types::{SolCall, SolValue},
};
use hopr_bindings::{
    hoprchannels::HoprChannels::HoprChannelsInstance,
    hoprnodemanagementmodule::HoprNodeManagementModule,
    hoprnodestakefactory::HoprNodeStakeFactory,
    hoprtoken::HoprToken::{self, HoprTokenInstance},
};
use hopr_crypto_types::prelude::*;
use hopr_primitive_types::primitives::Address;
use tracing::debug;

use crate::{
    ContractInstances, constants,
    errors::{ChainTypesError, Result as ChainTypesResult},
};

// define basic safe abi
sol!(
    #![sol(abi)]
    #![sol(rpc)]
    // #[allow(dead_code)]
    contract SafeContract {
        function nonce() view returns (uint256);
        function getTransactionHash( address to, uint256 value, bytes calldata data, uint8 operation, uint256 safeTxGas, uint256 baseGas, uint256 gasPrice, address gasToken, address refundReceiver, uint256 _nonce) public view returns (bytes32);
        function execTransaction(address to, uint256 value, bytes calldata data, uint8 operation, uint256 safeTxGas, uint256 baseGas, uint256 gasPrice, address gasToken, address payable refundReceiver, bytes memory signatures) public returns (bool);
    }
);

lazy_static::lazy_static! {
    static ref MINTER_ROLE_VALUE: primitives::FixedBytes<32> = keccak256("MINTER_ROLE");
}

/// Creates local Anvil instance.
///
/// Used for testing. When block time is given, new blocks are mined periodically.
/// Otherwise, a new block is mined per transaction.
///
/// Uses a fixed mnemonic to make generated accounts deterministic.
pub fn create_anvil(block_time: Option<std::time::Duration>) -> alloy::node_bindings::AnvilInstance {
    let mut anvil = alloy::node_bindings::Anvil::new()
        .mnemonic("gentle wisdom move brush express similar canal dune emotion series because parrot");

    if let Some(bt) = block_time {
        anvil = anvil.block_time(bt.as_secs());
    }

    anvil.spawn()
}

/// Mints specified amount of HOPR tokens to the contract deployer wallet.
/// Assumes that the `hopr_token` contract is associated with a RPC client that also deployed the contract.
/// Returns the block number at which the minting transaction was confirmed.
pub async fn mint_tokens<P, N>(hopr_token: HoprTokenInstance<P, N>, amount: U256) -> ContractResult<Option<u64>>
where
    P: alloy::contract::private::Provider<N>,
    N: alloy::providers::Network,
{
    let deployer = hopr_token
        .provider()
        .get_accounts()
        .await
        .expect("client must have a signer")[0];

    hopr_token
        .grantRole(*MINTER_ROLE_VALUE, deployer)
        .send()
        .await?
        .watch()
        .await?;

    let tx_receipt = hopr_token
        .mint(deployer, amount, Bytes::new(), Bytes::new())
        .send()
        .await?
        .get_receipt()
        .await?;

    Ok(tx_receipt.block_number())
}

/// Creates a transaction that transfers the given `amount` of native tokens to the
/// given destination.
pub fn create_native_transfer<N>(to: Address, amount: U256) -> N::TransactionRequest
where
    N: alloy::providers::Network,
{
    N::TransactionRequest::default().with_to(to.into()).with_value(amount)
}

/// Funds the given wallet address with specified amount of native tokens and HOPR tokens.
/// These must be present in the client's wallet.
pub async fn fund_node<P, N>(
    node: Address,
    native_token: U256,
    hopr_token: U256,
    hopr_token_contract: HoprTokenInstance<P, N>,
) -> ContractResult<()>
where
    P: alloy::contract::private::Provider<N>,
    N: alloy::providers::Network,
{
    let native_transfer_tx = N::TransactionRequest::default()
        .with_to(node.into())
        .with_value(native_token);

    // let native_transfer_tx = Eip1559TransactionRequest::new()
    //     .to(NameOrAddress::Address(node.into()))
    //     .value(native_token);

    let provider = hopr_token_contract.provider();

    provider.send_transaction(native_transfer_tx).await?.watch().await?;

    hopr_token_contract
        .transfer(node.into(), hopr_token)
        .send()
        .await?
        .watch()
        .await?;
    Ok(())
}

/// Funds the channel to the counterparty with the given amount of HOPR tokens.
/// The amount must be present in the wallet of the client.
pub async fn fund_channel<P, N>(
    counterparty: Address,
    hopr_token: HoprTokenInstance<P, N>,
    hopr_channels: HoprChannelsInstance<P, N>,
    amount: U256,
) -> ContractResult<()>
where
    P: alloy::contract::private::Provider<N>,
    N: alloy::providers::Network,
{
    hopr_token
        .approve(*hopr_channels.address(), amount)
        .send()
        .await?
        .watch()
        .await?;

    hopr_channels
        .fundChannel(counterparty.into(), aliases::U96::from(amount))
        .send()
        .await?
        .watch()
        .await?;

    Ok(())
}

/// Funds the channel to the counterparty with the given amount of HOPR tokens, from a different client
/// The amount must be present in the wallet of the client.
pub async fn fund_channel_from_different_client<P, N>(
    counterparty: Address,
    hopr_token_address: Address,
    hopr_channels_address: Address,
    amount: U256,
    new_client: P,
) -> ContractResult<()>
where
    P: alloy::contract::private::Provider<N> + Clone,
    N: alloy::providers::Network,
{
    let hopr_token_with_new_client: HoprTokenInstance<P, N> =
        HoprTokenInstance::new(hopr_token_address.into(), new_client.clone());
    let hopr_channels_with_new_client = HoprChannelsInstance::new(hopr_channels_address.into(), new_client.clone());
    hopr_token_with_new_client
        .approve(hopr_channels_address.into(), amount)
        .send()
        .await?
        .watch()
        .await?;

    hopr_channels_with_new_client
        .fundChannel(counterparty.into(), aliases::U96::from(amount))
        .send()
        .await?
        .watch()
        .await?;

    Ok(())
}

/// Prepare a safe transaction
pub async fn get_safe_tx<P, N>(
    safe_contract: SafeContractInstance<P, N>,
    target: Address,
    inner_tx_data: Bytes,
    wallet: PrivateKeySigner,
) -> ChainTypesResult<N::TransactionRequest>
where
    P: alloy::contract::private::Provider<N>,
    N: alloy::providers::Network,
{
    let nonce = safe_contract.nonce().call().await?;

    let data_hash = safe_contract
        .getTransactionHash(
            target.into(),
            U256::ZERO,
            inner_tx_data.clone(),
            0,
            U256::ZERO,
            U256::ZERO,
            U256::ZERO,
            primitives::Address::default(),
            wallet.address(),
            nonce,
        )
        .call()
        .await?;

    let signed_data_hash = wallet.sign_hash(&data_hash).await?;

    let safe_tx_data = SafeContract::execTransactionCall {
        to: target.into(),
        value: U256::ZERO,
        data: inner_tx_data,
        operation: 0,
        safeTxGas: U256::ZERO,
        baseGas: U256::ZERO,
        gasPrice: U256::ZERO,
        gasToken: primitives::Address::default(),
        refundReceiver: wallet.address(),
        signatures: Bytes::from(signed_data_hash.as_bytes()),
    }
    .abi_encode();

    // Outer tx payload: execute as safe tx
    let safe_tx = N::TransactionRequest::default()
        .with_to(*safe_contract.address())
        .with_input(safe_tx_data);

    Ok(safe_tx)
}

/// Send a Safe transaction to the module to include node to the module
pub async fn include_node_to_module_by_safe<P, N>(
    provider: P,
    safe_address: Address,
    module_address: Address,
    node_address: Address,
    deployer: &ChainKeypair, // also node address
) -> Result<(), ChainTypesError>
where
    P: alloy::contract::private::Provider<N> + Clone,
    N: alloy::providers::Network,
{
    // prepare default permission for node.
    // - Clearance: Function
    // - TargetType: SEND
    // - TargetPermission: allow all
    // - NodeDefaultPermission: None
    let node_target_permission = format!("{node_address:?}010203000000000000000000");

    // Inner tx payload: include node to the module
    let inner_tx_data = HoprNodeManagementModule::includeNodeCall {
        nodeDefaultTarget: U256::from_str(&node_target_permission).unwrap(),
    }
    .abi_encode();

    let safe_contract = SafeContract::new(safe_address.into(), provider.clone());
    let wallet = PrivateKeySigner::from_slice(deployer.secret().as_ref()).expect("failed to construct wallet");
    let safe_tx = get_safe_tx(safe_contract, module_address, inner_tx_data.into(), wallet).await?;

    provider
        .send_transaction(safe_tx)
        .await
        .map_err(|e| ChainTypesError::ContractError(e.into()))?
        .watch()
        .await
        .map_err(|e| ChainTypesError::ContractError(e.into()))?;

    Ok(())
}

/// Send a Safe transaction to the module to include annoucement to the module
pub async fn add_announcement_as_target<P, N>(
    provider: P,
    safe_address: Address,
    module_address: Address,
    announcement_contract_address: Address,
    deployer: &ChainKeypair, // also node address
) -> ContractResult<()>
where
    P: alloy::contract::private::Provider<N> + Clone,
    N: alloy::providers::Network,
{
    // prepare default permission for announcement.
    // - Clearance: Function
    // - TargetType: TOKEN
    // - TargetPermission: allow all
    // - NodeDefaultPermission: None
    let announcement_target_permission = format!("{announcement_contract_address:?}010003000000000000000000");

    // Inner tx payload: include node to the module
    let inner_tx_data = HoprNodeManagementModule::scopeTargetTokenCall {
        defaultTarget: U256::from_str(&announcement_target_permission).unwrap(),
    }
    .abi_encode();

    let safe_contract = SafeContract::new(safe_address.into(), provider.clone());
    let wallet = PrivateKeySigner::from_slice(deployer.secret().as_ref()).expect("failed to construct wallet");
    let safe_tx = get_safe_tx(safe_contract, module_address, inner_tx_data.into(), wallet)
        .await
        .unwrap();

    provider.send_transaction(safe_tx).await?.watch().await?;

    Ok(())
}

/// Send a Safe transaction to the token contract, to approve channels on behalf of safe.
pub async fn approve_channel_transfer_from_safe<P, N>(
    provider: P,
    safe_address: Address,
    token_address: Address,
    channel_address: Address,
    deployer: &ChainKeypair, // also node address
) -> ContractResult<()>
where
    P: alloy::contract::private::Provider<N> + Clone,
    N: alloy::providers::Network,
{
    // Inner tx payload: include node to the module
    let inner_tx_data = HoprToken::approveCall {
        spender: channel_address.into(),
        value: U256::MAX,
    }
    .abi_encode();

    let safe_contract = SafeContract::new(safe_address.into(), provider.clone());
    let wallet = PrivateKeySigner::from_slice(deployer.secret().as_ref()).expect("failed to construct wallet");
    let safe_tx = get_safe_tx(safe_contract, token_address, inner_tx_data.into(), wallet)
        .await
        .unwrap();

    provider.send_transaction(safe_tx).await?.watch().await?;

    Ok(())
}

/// Deploy a safe instance and a module instance.
///
/// Notice that to complete the on-boarding process,
/// 1) node should be included to the module
/// 2) announcement contract should be a target in the module
///
/// Notice that to be able to open channels, the deployed safe should have HOPR tokens and approve token transfer for
/// Channels contract on the token contract.
///
/// Returns (module address, safe address)
pub async fn deploy_one_safe_one_module_and_setup_for_testing<P>(
    instances: &ContractInstances<P>,
    provider: P,
    deployer: &ChainKeypair,
) -> ContractResult<(Address, Address)>
where
    P: alloy::providers::Provider + Clone,
{
    // Get deployer address
    let self_address: Address = deployer.public().to_address();

    // Check if safe suite has been deployed. If so, skip this step
    let code = provider
        .get_code_at(address!("914d7Fec6aaC8cd542e72Bca78B30650d45643d7"))
        .await?;

    // only deploy contracts when needed
    if code.is_empty() {
        debug!("deploying safe code");
        // Deploy Safe diamond deployment proxy singleton
        let safe_diamond_proxy_address = {
            // Fund Safe singleton deployer 0.01 anvil-eth and deploy Safe singleton
            let tx = TransactionRequest::default()
                .with_to(address!("E1CB04A0fA36DdD16a06ea828007E35e1a3cBC37"))
                .with_value(U256::from(10000000000000000u128));

            provider.send_transaction(tx).await?.watch().await?;

            let tx_receipt = provider
                .send_raw_transaction(&constants::SAFE_DIAMOND_PROXY_SINGLETON_DEPLOY_CODE)
                .await?
                .get_receipt()
                .await?;
            tx_receipt.contract_address().unwrap()
        };
        debug!(%safe_diamond_proxy_address, "Safe diamond proxy singleton");

        // Deploy minimum Safe suite
        {
            // 1. Safe proxy factory deploySafeProxyFactory();
            let _tx_safe_proxy_factory = TransactionRequest::default()
                .with_to(safe_diamond_proxy_address)
                .with_input(constants::SAFE_PROXY_FACTORY_DEPLOY_CODE);

            // 2. Handler: only CompatibilityFallbackHandler and omit TokenCallbackHandler as it's not used now
            let _tx_safe_compatibility_fallback_handler = TransactionRequest::default()
                .with_to(safe_diamond_proxy_address)
                .with_input(constants::SAFE_COMPATIBILITY_FALLBACK_HANDLER_DEPLOY_CODE);
            // 3. Library: only MultiSendCallOnly and omit MultiSendCall
            let _tx_safe_multisend_call_only = TransactionRequest::default()
                .with_to(safe_diamond_proxy_address)
                .with_input(constants::SAFE_MULTISEND_CALL_ONLY_DEPLOY_CODE);
            // 4. Safe singleton deploySafe();
            let _tx_safe_singleton = TransactionRequest::default()
                .with_to(safe_diamond_proxy_address)
                .with_input(constants::SAFE_SINGLETON_DEPLOY_CODE);
            // other omitted libs: SimulateTxAccessor, CreateCall, and SignMessageLib
            // broadcast those transactions
            provider.send_transaction(_tx_safe_proxy_factory).await?.watch().await?;
            provider
                .send_transaction(_tx_safe_compatibility_fallback_handler)
                .await?
                .watch()
                .await?;
            provider
                .send_transaction(_tx_safe_multisend_call_only)
                .await?
                .watch()
                .await?;
            provider.send_transaction(_tx_safe_singleton).await?.watch().await?;
        }
    }

    // create a salt from the nonce
    let curr_nonce = provider
        .get_transaction_count(self_address.into())
        .pending()
        //  Some(BlockNumber::Pending.into()))
        .await
        .unwrap();
    debug!(%curr_nonce, "curr_nonce");

    // FIXME: Check if this is the correct way to get the nonce
    let nonce =
        keccak256((Into::<primitives::Address>::into(self_address), U256::from(curr_nonce)).abi_encode_packed());
    let default_target = format!("{:?}010103020202020202020202", instances.channels.address());

    debug!(%self_address, "self_address");
    debug!("nonce {:?}", U256::from_be_bytes(nonce.0).to_string());
    debug!("default_target in bytes {:?}", default_target.bytes());
    debug!("default_target in u256 {:?}", U256::from_str(&default_target).unwrap());

    let typed_tx = HoprNodeStakeFactory::cloneCall {
        moduleSingletonAddress: *instances.module_implementation.address(),
        admins: vec![self_address.into()],
        nonce: nonce.into(),
        defaultTarget: U256::from_str(&default_target).unwrap().into(),
    }
    .abi_encode();

    debug!("typed_tx {:?}", typed_tx);

    // deploy one safe and one module
    let instance_deployment_tx_receipt = instances
        .stake_factory
        .clone(
            *instances.module_implementation.address(),
            vec![self_address.into()],
            nonce.into(),
            U256::from_str(&default_target).unwrap().into(),
        )
        .send()
        .await?
        .get_receipt()
        .await?;

    // decode logs
    let maybe_module_tx_log =
        instance_deployment_tx_receipt.decoded_log::<HoprNodeStakeFactory::NewHoprNodeStakeModule>();
    let deployed_module_address: primitives::Address = if let Some(module_tx_log) = maybe_module_tx_log {
        let HoprNodeStakeFactory::NewHoprNodeStakeModule { instance, .. } = module_tx_log.data;
        instance
    } else {
        return Err(ContractError::ContractNotDeployed);
    };

    let maybe_safe_tx_log = instance_deployment_tx_receipt.decoded_log::<HoprNodeStakeFactory::NewHoprNodeStakeSafe>();
    let deployed_safe_address: primitives::Address = if let Some(safe_tx_log) = maybe_safe_tx_log {
        let HoprNodeStakeFactory::NewHoprNodeStakeSafe { instance } = safe_tx_log.data;
        instance
    } else {
        return Err(ContractError::ContractNotDeployed);
    };

    debug!("instance_deployment_tx module instance {:?}", deployed_module_address);
    debug!("instance_deployment_tx safe instance {:?}", deployed_safe_address);

    Ok((deployed_module_address.into(), deployed_safe_address.into()))
}
