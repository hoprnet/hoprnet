//! Chain utilities used for testing.
//!
//! This used in unit and integration tests.
use crate::{constants, errors::Result as ChainTypesResult, ContractAddresses, ContractInstances};
use alloy::{
    contract::{Error as ContractError, Result as ContractResult},
    network::{ReceiptResponse, TransactionBuilder},
    primitives::{self, address, aliases, keccak256, Bytes, U256},
    rpc::types::{TransactionReceipt, TransactionRequest},
    signers::{local::PrivateKeySigner, Signer},
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
use std::str::FromStr;
use tracing::debug;
use SafeContract::SafeContractInstance;

/// convert hopr_primitive_types::primitives::Address to alloy::primitives::Address
pub fn address_to_alloy_primitive(a: hopr_primitive_types::primitives::Address) -> alloy::primitives::Address {
    // alloy::primitives::Address::from(a.into_array())
    alloy::primitives::Address::from_slice(a.as_ref())
}

/// convert alloy::primitives::Address to hopr_primitive_types::primitives::Address
pub fn address_from_alloy_primitive(b: alloy::primitives::Address) -> hopr_primitive_types::primitives::Address {
    hopr_primitive_types::primitives::Address::from(b.0 .0)
}

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

// pub fn create_anvil(block_time: Option<std::time::Duration>) -> ethers::utils::AnvilInstance {
//     let mut anvil = ethers::utils::Anvil::new()
//         .mnemonic("gentle wisdom move brush express similar canal dune emotion series because parrot");
//     if let Some(bt) = block_time {
//         anvil = anvil.block_time(bt.as_secs());
//     }

//     anvil.spawn()
// }

/// Mints specified amount of HOPR tokens to the contract deployer wallet.
/// Assumes that the `hopr_token` contract is associated with a RPC client that also deployed the contract.
/// Returns the block number at which the minting transaction was confirmed.
pub async fn mint_tokens<T, P, N>(hopr_token: HoprTokenInstance<T, P, N>, amount: U256) -> ContractResult<Option<u64>>
where
    T: alloy::contract::private::Transport + Clone,
    P: alloy::contract::private::Provider<T, N>,
    N: alloy::providers::Network,
{
    // pub async fn mint_tokens<M: Middleware + 'static>(hopr_token: HoprTokenInstance<M>, amount: U256) -> u64 {
    let deployer = hopr_token
        .provider()
        .get_accounts()
        .await
        .expect("client must have a signer")[0];
    // let accounts = provider.get_accounts().await.unwrap();
    //     let sender = accounts[0];
    // let deployer = hopr_token.client().default_sender().expect("client must have a signer");

    hopr_token
        .grantRole(*MINTER_ROLE_VALUE, deployer)
        .send()
        .await?
        .watch()
        .await?;
    // .send()
    // .await
    // .unwrap()
    // .await
    // .unwrap();

    let tx_receipt = hopr_token
        .mint(deployer, amount, Bytes::new(), Bytes::new())
        .send()
        .await?
        .get_receipt()
        .await?;

    Ok(tx_receipt.block_number())
    // .unwrap()
    // .unwrap()
    // .block_number
    // .unwrap()
    // .as_u64()
}

/// Creates a transaction that transfers the given `amount` of native tokens to the
/// given destination.
pub fn create_native_transfer<N>(to: Address, amount: U256) -> N::TransactionRequest
where
    N: alloy::providers::Network,
{
    let tx = N::TransactionRequest::default()
        .with_to(address_to_alloy_primitive(to))
        .with_value(amount);

    tx
}

/// Funds the given wallet address with specified amount of native tokens and HOPR tokens.
/// These must be present in the client's wallet.
pub async fn fund_node<T, P, N>(
    node: Address,
    native_token: U256,
    hopr_token: U256,
    hopr_token_contract: HoprTokenInstance<T, P, N>,
) -> ContractResult<()>
where
    T: alloy::contract::private::Transport + Clone,
    P: alloy::contract::private::Provider<T, N>,
    N: alloy::providers::Network,
{
    let native_transfer_tx = N::TransactionRequest::default()
        .with_to(address_to_alloy_primitive(node))
        .with_value(native_token);

    // let native_transfer_tx = Eip1559TransactionRequest::new()
    //     .to(NameOrAddress::Address(node.into()))
    //     .value(native_token);

    let provider = hopr_token_contract.provider();

    provider.send_transaction(native_transfer_tx).await?.watch().await?;

    hopr_token_contract
        .transfer(address_to_alloy_primitive(node), hopr_token)
        .send()
        .await?
        .watch()
        .await?;
    Ok(())
}

/// Funds the channel to the counterparty with the given amount of HOPR tokens.
/// The amount must be present in the wallet of the client.
pub async fn fund_channel<T, P, N>(
    counterparty: Address,
    hopr_token: HoprTokenInstance<T, P, N>,
    hopr_channels: HoprChannelsInstance<T, P, N>,
    amount: U256,
) -> ContractResult<()>
where
    T: alloy::contract::private::Transport + Clone,
    P: alloy::contract::private::Provider<T, N>,
    N: alloy::providers::Network,
{
    hopr_token
        .approve(*hopr_channels.address(), amount)
        .send()
        .await?
        .watch()
        .await?;

    hopr_channels
        .fundChannel(address_to_alloy_primitive(counterparty), aliases::U96::from(amount))
        .send()
        .await?
        .watch()
        .await?;

    Ok(())
}

/// Funds the channel to the counterparty with the given amount of HOPR tokens, from a different client
/// The amount must be present in the wallet of the client.
pub async fn fund_channel_from_different_client<T, P, N>(
    counterparty: Address,
    hopr_token_address: Address,
    hopr_channels_address: Address,
    amount: U256,
    new_client: P,
    // new_client: Arc<M>,
) -> ContractResult<()>
where
    T: alloy::contract::private::Transport + Clone,
    P: alloy::contract::private::Provider<T, N> + Clone,
    N: alloy::providers::Network,
{
    let hopr_token_with_new_client =
        HoprTokenInstance::new(address_to_alloy_primitive(hopr_token_address), new_client.clone());
    let hopr_channels_with_new_client =
        HoprChannelsInstance::new(address_to_alloy_primitive(hopr_channels_address), new_client.clone());
    hopr_token_with_new_client
        .approve(address_to_alloy_primitive(hopr_channels_address), amount)
        .send()
        .await?
        .watch()
        .await?;

    hopr_channels_with_new_client
        .fundChannel(address_to_alloy_primitive(counterparty), aliases::U96::from(amount))
        .send()
        .await?
        .watch()
        .await?;

    Ok(())
}

/// Prepare a safe transaction
pub async fn get_safe_tx<T, P, N>(
    safe_contract: SafeContractInstance<T, P, N>,
    target: Address,
    inner_tx_data: Bytes,
    wallet: PrivateKeySigner,
) -> ChainTypesResult<N::TransactionRequest>
where
    T: alloy::contract::private::Transport + Clone,
    P: alloy::contract::private::Provider<T, N>,
    N: alloy::providers::Network,
{
    // ) -> Result<TypedTransaction, ContractError<M>> {
    let nonce = safe_contract.nonce().call().await?;

    let data_hash = safe_contract
        .getTransactionHash(
            address_to_alloy_primitive(target),
            U256::ZERO,
            inner_tx_data.clone(),
            0,
            U256::ZERO,
            U256::ZERO,
            U256::ZERO,
            primitives::Address::default(),
            wallet.address(),
            nonce._0,
        )
        .call()
        .await?;

    let signed_data_hash = wallet.sign_hash(&data_hash._0).await?;

    // safe_tx.set_data(
    let safe_tx_data = SafeContract::execTransactionCall {
        to: address_to_alloy_primitive(target),
        value: U256::ZERO,
        data: inner_tx_data,
        operation: 0,
        safeTxGas: U256::ZERO,
        baseGas: U256::ZERO,
        gasPrice: U256::ZERO,
        gasToken: primitives::Address::default(),
        refundReceiver: wallet.address(),
        signatures: Bytes::from(signed_data_hash.as_bytes()),
        // signatures: Bytes::from(signed_data_hash),
    }
    .abi_encode();

    // Outer tx payload: execute as safe tx
    let safe_tx = N::TransactionRequest::default()
        .with_to(*safe_contract.address())
        .with_input(safe_tx_data);

    // // SafeContractCalls::execTransaction(()).
    // ExecTransactionCall {
    //     to: target.into(),
    //     value: U256::ZERO,
    //     data: inner_tx_data,
    //     operation: 0,
    //     safe_tx_gas: U256::ZERO,
    //     base_gas: U256::ZERO,
    //     gas_price: U256::ZERO,
    //     gas_token: Address::default().into(),
    //     refund_receiver: wallet.address(),
    //     signatures: encode_packed(&[Token::Bytes(signed_data_hash.to_vec())])
    //         .unwrap()
    //         .into(),
    // }
    // .encode()
    // .into();
    // );
    // safe_tx.set_to(NameOrAddress::Address(safe_contract.address()));

    Ok(safe_tx)
}

/// Send a Safe transaction to the module to include node to the module
pub async fn include_node_to_module_by_safe<T, P, N>(
    provider: P,
    safe_address: Address,
    module_address: Address,
    node_address: Address,
    deployer: &ChainKeypair, // also node address
) -> ContractResult<()>
where
    T: alloy::contract::private::Transport + Clone,
    P: alloy::contract::private::Provider<T, N> + Clone,
    N: alloy::providers::Network,
{
    // prepare default permission for node.
    // - Clearance: Function
    // - TargetType: SEND
    // - TargetPermission: allow all
    // - NodeDefaultPermission: None
    let node_target_permission = format!("{:?}010203000000000000000000", node_address);

    // Inner tx payload: include node to the module
    let inner_tx_data = HoprNodeManagementModule::includeNodeCall {
        nodeDefaultTarget: U256::from_str(&node_target_permission).unwrap(),
    }
    .abi_encode();
    // let inner_tx_data: ethers::types::Bytes = IncludeNodeCall {
    //     node_default_target: U256::from_str(&node_target_permission).unwrap(),
    // };

    let safe_contract = SafeContract::new(address_to_alloy_primitive(safe_address), provider.clone());
    let wallet = PrivateKeySigner::from_slice(deployer.secret().as_ref()).expect("failed to construct wallet");
    let safe_tx = get_safe_tx(safe_contract, module_address, inner_tx_data.into(), wallet)
        .await
        .unwrap();

    provider
        .send_transaction(safe_tx)
        .await?
        .watch()
        // .map_err(|e| ContractError::MiddlewareError { e })?
        .await?;

    Ok(())
}

/// Send a Safe transaction to the module to include annoucement to the module
pub async fn add_announcement_as_target<T, P, N>(
    provider: P,
    safe_address: Address,
    module_address: Address,
    announcement_contract_address: Address,
    deployer: &ChainKeypair, // also node address
) -> ContractResult<()>
where
    T: alloy::contract::private::Transport + Clone,
    P: alloy::contract::private::Provider<T, N> + Clone,
    N: alloy::providers::Network,
{
    // prepare default permission for announcement.
    // - Clearance: Function
    // - TargetType: TOKEN
    // - TargetPermission: allow all
    // - NodeDefaultPermission: None
    let announcement_target_permission = format!("{:?}010003000000000000000000", announcement_contract_address);

    // Inner tx payload: include node to the module
    let inner_tx_data = HoprNodeManagementModule::scopeTargetTokenCall {
        defaultTarget: U256::from_str(&announcement_target_permission).unwrap(),
    }
    .abi_encode();
    // let inner_tx_data: ethers::types::Bytes = ScopeTargetTokenCall {
    //     default_target: U256::from_str(&announcement_target_permission).unwrap(),
    // }
    // .encode()
    // .into();

    let safe_contract = SafeContract::new(address_to_alloy_primitive(safe_address), provider.clone());
    let wallet = PrivateKeySigner::from_slice(deployer.secret().as_ref()).expect("failed to construct wallet");
    let safe_tx = get_safe_tx(safe_contract, module_address, inner_tx_data.into(), wallet)
        .await
        .unwrap();

    provider.send_transaction(safe_tx).await?.watch().await?;

    Ok(())
}

/// Send a Safe transaction to the token contract, to approve channels on behalf of safe.
pub async fn approve_channel_transfer_from_safe<T, P, N>(
    provider: P,
    safe_address: Address,
    token_address: Address,
    channel_address: Address,
    deployer: &ChainKeypair, // also node address
) -> ContractResult<()>
where
    T: alloy::contract::private::Transport + Clone,
    P: alloy::contract::private::Provider<T, N> + Clone,
    N: alloy::providers::Network,
{
    // Inner tx payload: include node to the module
    let inner_tx_data = HoprToken::approveCall {
        spender: address_to_alloy_primitive(channel_address),
        value: U256::MAX,
    }
    .abi_encode();

    let safe_contract = SafeContract::new(address_to_alloy_primitive(safe_address), provider.clone());
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
/// Notice that to be able to open channels, the deployed safe should have HOPR tokens and approve token transfer for Channels contract on the token contract.
///
/// Returns (module address, safe address)
pub async fn deploy_one_safe_one_module_and_setup_for_testing<T, P, N>(
    instances: &ContractInstances<T, P, N>,
    provider: P,
    deployer: &ChainKeypair,
) -> ContractResult<(Address, Address)>
where
    T: alloy::contract::private::Transport + Clone,
    P: alloy::providers::Provider + alloy::contract::private::Provider<T, N>,
    N: alloy::providers::Network,
    // N: alloy::network::Ethereum,
{
    // Get deployer address
    let self_address: Address = deployer.public().to_address().into();

    // Check if safe suite has been deployed. If so, skip this step
    let code = provider
        .get_code_at(address!("914d7Fec6aaC8cd542e72Bca78B30650d45643d7"))
        .await?;

    // only deploy contracts when needed
    if code.is_empty() {
        // // FIXME: debug log
        // debug!("deploying safe code");
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
            // let tx = provider
            //     .send_raw_transaction(Bytes::from_static(&*SAFE_DIAMOND_PROXY_SINGLETON_DEPLOY_CODE))
            //     .await
            //     .map_err(|e| ContractError::MiddlewareError { e })?
            //     .await?
            //     .unwrap();
            tx_receipt.contract_address().unwrap()
            // .ok_or_else(|| ContractError::ContractNotDeployed)?;
            // )?;
            // tx.contract_address.unwrap()
        };
        // // FIXME: debug log
        // debug!(%safe_diamond_proxy_address, "Safe diamond proxy singleton");

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
        .get_transaction_count(address_to_alloy_primitive(self_address))
        .pending()
        //  Some(BlockNumber::Pending.into()))
        .await
        .unwrap();
    // // FIXME: debug log
    // debug!(%curr_nonce, "curr_nonce");

    // FIXME: Check if this is the correct way to get the nonce
    let nonce = keccak256((address_to_alloy_primitive(self_address), U256::from(curr_nonce)).abi_encode_packed());
    // let nonce = keccak256(sol_types::abi::abi_encode_packed(&[Token::Address(self_address), Token::Uint(curr_nonce)]).unwrap());
    let default_target = format!("{:?}010103020202020202020202", instances.channels.address());

    let contract_addrs = ContractAddresses::from(instances);

    // // FIXME: debug log
    // debug!(%self_address, "self_address");
    // debug!("nonce {:?}", U256::from_be_bytes(nonce.0).to_string());
    // debug!("default_target in bytes {:?}", default_target.encode_hex());
    // debug!("default_target in u256 {:?}", U256::from_str(&default_target).unwrap());

    // let typed_tx = instances
    //     .stake_factory
    //     .clone(
    //         *instances.module_implementation.address(),
    //         vec![*address_to_alloy_primitive(self_address).into()],
    //         nonce.into(),
    //         U256::from_str(&default_target).unwrap().into(),
    //     );
    // .tx;
    let typed_tx = HoprNodeStakeFactory::cloneCall {
        moduleSingletonAddress: *instances.module_implementation.address(),
        admins: vec![*address_to_alloy_primitive(self_address).into()],
        nonce: nonce.into(),
        defaultTarget: U256::from_str(&default_target).unwrap().into(),
    }
    .abi_encode();
    // // FIXME: debug log
    // debug!("typed_tx {:?}", typed_tx);

    // deploy one safe and one module
    let instance_deployment_tx_receipt = instances
        .stake_factory
        .clone(
            *instances.module_implementation.address(),
            vec![*address_to_alloy_primitive(self_address).into()],
            nonce.into(),
            U256::from_str(&default_target).unwrap().into(),
        )
        .send()
        .await?
        .get_receipt()
        .await?;

    // decode logs
    let maybe_module_tx_log =
        instance_deployment_tx_receipt.decoded_log::<HoprNodeStakeFactory::NewHoprNodeStakeModule>()?;
    let deployed_module_address: primitives::Address = if let Some(module_tx_log) = maybe_module_tx_log {
        let HoprNodeStakeFactory::NewHoprNodeStakeModule { instance, .. } = module_tx_log.data;
        instance
    } else {
        return Err(ContractError::ContractNotDeployed);
    };

    let maybe_safe_tx_log =
        instance_deployment_tx_receipt.decoded_log::<HoprNodeStakeFactory::NewHoprNodeStakeSafe>()?;
    let deployed_safe_address: primitives::Address = if let Some(safe_tx_log) = maybe_safe_tx_log {
        let HoprNodeStakeFactory::NewHoprNodeStakeSafe { instance } = safe_tx_log.data;
        instance
    } else {
        return Err(ContractError::ContractNotDeployed);
    };
    // let module_log = instance_deployment_tx_logs
    //     .iter()
    //     .find(|log| {
    //         log.topics[0].eq(&"0x813d391dc490d6c1dae7d3fdd555f337533d1da2c908c6efd36d4cf557a63206"
    //             .parse::<H256>()
    //             .unwrap())
    //     })
    //     .ok_or(ContractError::ContractNotDeployed)?; // "NewHoprNodeStakeModule(address,address)"

    // let safe_log = instance_deployment_tx_logs
    //     .iter()
    //     .find(|log| {
    //         log.topics[0].eq(&"0x8231d169f416b666ae7fa43faa24a18899738075a53f32c97617d173b189e386"
    //             .parse::<H256>()
    //             .unwrap())
    //     })
    //     .ok_or(ContractError::ContractNotDeployed)?; // "NewHoprNodeStakeSafe(address)"

    // // get address of deployed instances
    // let deployed_module_address: Address = instances
    //     .stake_factory
    //     .decode_event::<NewHoprNodeStakeModuleFilter>(
    //         "NewHoprNodeStakeModule",
    //         module_log.topics.clone(),
    //         module_log.data.clone(),
    //     )
    //     .unwrap()
    //     .instance
    //     .into();

    // note that using the following snippet would cause error
    // "DetokenizationError(InvalidOutputType("Expected Tuple, got Address(0x8819c5bab7d63c61d72f65b19b05a6772f55391b)"))"
    // ```
    // let decoded_safe_log: NewHoprNodeStakeSafeFilter = instances
    // .stake_factory
    // .decode_event("NewHoprNodeStakeSafe", safe_log.topics.clone(), safe_log.data.clone())
    // .unwrap();
    // ```
    // let deployed_safe_address: Address = instances
    //     .stake_factory
    //     .abi()
    //     .event("NewHoprNodeStakeSafe")
    //     .unwrap()
    //     .parse_log(RawLog {
    //         topics: safe_log.topics.clone(),
    //         data: safe_log.data.clone().to_vec(),
    //     })?
    //     .params
    //     .into_iter()
    //     .map(|param| param.value)
    //     .collect::<Vec<_>>()
    //     .pop()
    //     .unwrap()
    //     .into_address()
    //     .unwrap()
    //     .into();

    // // FIXME: debug log
    // debug!("instance_deployment_tx module instance {:?}", deployed_module_address);
    // debug!("instance_deployment_tx safe instance {:?}", deployed_safe_address);

    Ok((
        address_from_alloy_primitive(deployed_module_address),
        address_from_alloy_primitive(deployed_safe_address),
    ))
}
