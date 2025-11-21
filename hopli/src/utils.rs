//! This module contains errors produced in this crate

use std::str::FromStr;

use SafeContract::SafeContractInstance;
use hopr_bindings::{
    exports::alloy::{
        self,
        contract::{Error as ContractError, Result as ContractResult},
        hex::FromHexError,
        network::{ReceiptResponse, TransactionBuilder},
        primitives::{self, Address, Bytes, U256, address, aliases, keccak256},
        providers::{MULTICALL3_ADDRESS, MulticallError, PendingTransactionError},
        rpc::types::TransactionRequest,
        signers::{Signer, local::PrivateKeySigner},
        sol,
        sol_types::{SolCall, SolValue},
        transports::TransportErrorKind,
    },
    hopr_announcements::{HoprAnnouncements, HoprAnnouncements::HoprAnnouncementsInstance},
    hopr_announcements_proxy::HoprAnnouncementsProxy,
    hopr_channels::{HoprChannels, HoprChannels::HoprChannelsInstance},
    hopr_node_management_module::{
        HoprNodeManagementModule, HoprNodeManagementModule::HoprNodeManagementModuleInstance,
    },
    hopr_node_safe_registry::{HoprNodeSafeRegistry, HoprNodeSafeRegistry::HoprNodeSafeRegistryInstance},
    hopr_node_stake_factory::{HoprNodeStakeFactory, HoprNodeStakeFactory::HoprNodeStakeFactoryInstance},
    hopr_ticket_price_oracle::{HoprTicketPriceOracle, HoprTicketPriceOracle::HoprTicketPriceOracleInstance},
    hopr_token::{HoprToken, HoprToken::HoprTokenInstance},
    hopr_winning_probability_oracle::{
        HoprWinningProbabilityOracle, HoprWinningProbabilityOracle::HoprWinningProbabilityOracleInstance,
    },
};
use hopr_chain_types::{ContractAddresses, errors::ChainTypesError};
use hopr_crypto_keypair::errors::KeyPairError;
use hopr_crypto_types::{keypairs::ChainKeypair, prelude::Keypair};
use thiserror::Error;
use tracing::debug;

use crate::constants::{
    ERC_1820_DEPLOYER, ERC_1820_REGISTRY_DEPLOY_CODE, ETH_VALUE_FOR_ERC1820_DEPLOYER, INIT_KEY_BINDING_FEE,
    MULTICALL3_DEPLOY_CODE,
};

pub trait Cmd: clap::Parser + Sized {
    fn run(self) -> Result<(), HelperErrors>;
    fn async_run(self) -> impl std::future::Future<Output = Result<(), HelperErrors>> + Send;
}

/// Enumerates different errors produced by this crate.
#[derive(Error, Debug)]
pub enum HelperErrors {
    /// Error of contract error
    #[error(transparent)]
    FromHexError(#[from] FromHexError),

    /// Error of contract error
    #[error(transparent)]
    ContractError(#[from] ContractError),

    /// Error propagated by pending transctions
    #[error(transparent)]
    PendingTransactionError(#[from] PendingTransactionError),

    /// Error propagated by IO operations
    #[error(transparent)]
    UnableToReadFromPath(#[from] std::io::Error),

    /// Error in parsing provided comma-separated addresses
    #[error("error parsing address: {0:?}")]
    UnableToParseAddress(String),

    /// System time rrror
    #[error(transparent)]
    SystemTime(#[from] std::time::SystemTimeError),

    /// Error when identity cannot be created
    #[error("unable to create identity")]
    UnableToCreateIdentity,

    #[error("unable to update identity password")]
    UnableToUpdateIdentityPassword,

    /// Error due to supplying a non-existing file name
    #[error("incorrect filename: {0}")]
    IncorrectFilename(String),

    /// Error when identity existed
    #[error("identity file exists: {0}")]
    IdentityFileExists(String),

    /// Fail to read identity
    #[error("unable to read identity")]
    UnableToReadIdentity,

    /// Fail to find the identity directory
    #[error("unable to read identity directory")]
    MissingIdentityDirectory,

    /// Fail to delete an identity
    #[error("unable to delete identity")]
    UnableToDeleteIdentity,

    /// Provided environement does not match with that in the `ethereum/contracts/contracts-addresses.json`
    #[error("environment info mismatch")]
    EnvironmentInfoMismatch,

    /// Wrong foundry contract root is provided
    #[error("unable to set foundry root")]
    UnableToSetFoundryRoot,

    /// Fail to run foundry
    #[error("unable to run foundry")]
    ErrorInRunningFoundry,

    /// Fail to read password
    #[error("unable read password")]
    UnableToReadPassword,

    /// Fail to read private key
    #[error("cannot read private key error: {0}")]
    UnableToReadPrivateKey(String),

    /// Paramters are missing
    #[error("missing parameter: {0}")]
    MissingParameter(String),

    /// Error with the keystore file
    #[error(transparent)]
    KeyStoreError(#[from] KeyPairError),

    #[error("deserialization Error: {0}")]
    /// Serde JSON Error
    SerdeJson(#[from] serde_json::Error),

    /// Cannot find network details from the given network name
    #[error("unable to find network details from the given network name ")]
    UnknownNetwork,

    /// Error with HTTP Json RPC provider
    #[error(transparent)]
    RpcTransportError(#[from] alloy::rpc::json_rpc::RpcError<TransportErrorKind>),

    /// Fail to make a multicall
    #[error(transparent)]
    MulticallError(#[from] MulticallError),

    /// Fail to make a multisend call
    #[error("internal transaction failure in multisend")]
    MultiSendError,

    /// Txn caller does not have the minter role
    #[error("caller does not have the privilege to mint tokens")]
    NotAMinter,

    /// Error with middleware
    #[error("middleware Error: {0}")]
    MiddlewareError(String),

    /// A required smart contract (Safe or module proxy instance) is not deployed
    #[error("contract not deployed: {0}")]
    ContractNotDeployed(String),

    // error of parsing addresses
    #[error("Cannot parse address: {0}")]
    InvalidAddress(String),

    // general error of parsing
    #[error("Cannot parse: {0}")]
    ParseError(String),
}

// Used instead of From implementation to avoid alloy being a dependency of the primitive crates
/// Converts [`alloy::primitives::Address`] into [`hopr_primitive_types::prelude::Address`]
pub fn h2a(h: alloy::primitives::Address) -> hopr_primitive_types::prelude::Address {
    hopr_primitive_types::prelude::Address::from(h.0.0)
}

/// Converts [`hopr_primitive_types::prelude::Address`] into [`alloy::primitives::Address`]
pub fn a2h(a: hopr_primitive_types::prelude::Address) -> alloy::primitives::Address {
    alloy::primitives::Address::from_slice(a.as_ref())
}

/// Holds instances to contracts.
#[derive(Debug, Clone)]
pub struct ContractInstances<P> {
    pub token: HoprTokenInstance<P>,
    pub channels: HoprChannelsInstance<P>,
    pub announcements: HoprAnnouncementsInstance<P>,
    pub safe_registry: HoprNodeSafeRegistryInstance<P>,
    pub price_oracle: HoprTicketPriceOracleInstance<P>,
    pub win_prob_oracle: HoprWinningProbabilityOracleInstance<P>,
    pub stake_factory: HoprNodeStakeFactoryInstance<P>,
    pub module_implementation: HoprNodeManagementModuleInstance<P>,
}

impl<P> ContractInstances<P>
where
    P: alloy::providers::Provider + Clone,
{
    pub fn new(contract_addresses: &ContractAddresses, provider: P, _use_dummy_nr: bool) -> Self {
        Self {
            token: HoprTokenInstance::new(a2h(contract_addresses.token), provider.clone()),
            channels: HoprChannelsInstance::new(a2h(contract_addresses.channels), provider.clone()),
            announcements: HoprAnnouncementsInstance::new(a2h(contract_addresses.announcements), provider.clone()),
            safe_registry: HoprNodeSafeRegistryInstance::new(
                a2h(contract_addresses.node_safe_registry),
                provider.clone(),
            ),
            price_oracle: HoprTicketPriceOracleInstance::new(
                a2h(contract_addresses.ticket_price_oracle),
                provider.clone(),
            ),
            win_prob_oracle: HoprWinningProbabilityOracleInstance::new(
                a2h(contract_addresses.winning_probability_oracle),
                provider.clone(),
            ),
            stake_factory: HoprNodeStakeFactoryInstance::new(
                a2h(contract_addresses.node_stake_factory),
                provider.clone(),
            ),
            module_implementation: HoprNodeManagementModuleInstance::new(
                a2h(contract_addresses.module_implementation),
                provider.clone(),
            ),
        }
    }

    /// Deploys testing environment (with dummy network registry proxy) via the given provider.
    async fn inner_deploy_common_contracts_for_testing(provider: P, deployer: &ChainKeypair) -> ContractResult<Self> {
        {
            debug!("deploying ERC1820 registry...");
            // Fund 1820 deployer and deploy ERC1820Registry
            let tx = TransactionRequest::default()
                .with_to(ERC_1820_DEPLOYER)
                .with_value(ETH_VALUE_FOR_ERC1820_DEPLOYER);

            // Sequentially executing the following transactions:
            // 1. Fund the deployer wallet
            provider.send_transaction(tx.clone()).await?.watch().await?;
            // 2. Use the funded deployer wallet to deploy ERC1820Registry with a signed txn
            provider
                .send_raw_transaction(&ERC_1820_REGISTRY_DEPLOY_CODE)
                .await?
                .watch()
                .await?;
        }

        {
            debug!("deploying Multicall3...");
            // Fund Multicall3 deployer and deploy Multicall3
            let multicall3_code = provider.get_code_at(MULTICALL3_ADDRESS).await?;
            if multicall3_code.is_empty() {
                // Fund Multicall3 deployer and deploy ERC1820Registry
                let tx = TransactionRequest::default()
                    .with_to(crate::constants::MULTICALL3_DEPLOYER)
                    .with_value(crate::constants::ETH_VALUE_FOR_MULTICALL3_DEPLOYER);
                // Sequentially executing the following transactions:
                // 1. Fund the deployer wallet
                provider.send_transaction(tx.clone()).await?.watch().await?;
                // 2. Use the funded deployer wallet to deploy Multicall3 with a signed txn
                provider
                    .send_raw_transaction(MULTICALL3_DEPLOY_CODE)
                    .await?
                    .watch()
                    .await?;
            }
        }

        debug!("deploying contracts...");

        // Get deployer address
        let self_address = a2h(deployer.public().to_address());

        let module_implementation = HoprNodeManagementModule::deploy(provider.clone()).await?;
        let safe_registry = HoprNodeSafeRegistry::deploy(provider.clone()).await?;
        let price_oracle = HoprTicketPriceOracle::deploy(
            provider.clone(),
            self_address,
            primitives::U256::from(100000000000000000_u128), // U256::from(100000000000000000_u128),
        )
        .await?;
        let win_prob_oracle = HoprWinningProbabilityOracle::deploy(
            provider.clone(),
            self_address,
            primitives::aliases::U56::from(0xFFFFFFFFFFFFFF_u64), /* 0xFFFFFFFFFFFFFF in hex or 72057594037927935 in
                                                                   * decimal values */
        )
        .await?;
        let token = HoprToken::deploy(provider.clone()).await?;
        let channels = HoprChannels::deploy(
            provider.clone(),
            primitives::Address::from(token.address().as_ref()),
            1_u32,
            primitives::Address::from(safe_registry.address().as_ref()),
        )
        .await?;
        let announcements_implementation = HoprAnnouncements::deploy(provider.clone()).await?;
        let announcement_initialize_parameters = (
            *token.address(),
            *safe_registry.address(),
            INIT_KEY_BINDING_FEE,
            self_address,
        )
            .abi_encode();
        let encode_initialization = HoprAnnouncements::initializeCall {
            initParams: announcement_initialize_parameters.into(),
        }
        .abi_encode();

        let announcements_proxy = HoprAnnouncementsProxy::deploy(
            provider.clone(),
            primitives::Address::from(announcements_implementation.address().as_ref()),
            encode_initialization.into(),
        )
        .await?;

        let stake_factory = HoprNodeStakeFactory::deploy(
            provider.clone(),
            primitives::Address::from(module_implementation.address().as_ref()),
            primitives::Address::from(announcements_proxy.address().as_ref()),
            self_address,
        )
        .await?;

        // get the defaultHoprNetwork from the stake factory
        let default_hopr_network = stake_factory.defaultHoprNetwork().call().await?;
        let new_default_hopr_network = HoprNodeStakeFactory::HoprNetwork {
            tokenAddress: *token.address(),
            defaultTokenAllowance: default_hopr_network.defaultTokenAllowance,
            defaultAnnouncementTarget: default_hopr_network.defaultAnnouncementTarget,
        };
        // Update the `defaultHoprNetwork` in the factory contract, to update the token address
        stake_factory
            .updateHoprNetwork(new_default_hopr_network)
            .send()
            .await?
            .watch()
            .await?;

        Ok(Self {
            token,
            channels,
            announcements: HoprAnnouncementsInstance::new(*announcements_proxy.address(), provider.clone()),
            safe_registry,
            price_oracle,
            win_prob_oracle,
            stake_factory,
            module_implementation,
        })
    }

    /// Deploys testing environment (with dummy network registry proxy) via the given provider.
    pub async fn deploy_for_testing(provider: P, deployer: &ChainKeypair) -> ContractResult<Self> {
        let instances = Self::inner_deploy_common_contracts_for_testing(provider.clone(), deployer).await?;

        Ok(Self { ..instances })
    }
}

impl<P> From<&ContractInstances<P>> for ContractAddresses
where
    P: alloy::providers::Provider + Clone,
{
    fn from(instances: &ContractInstances<P>) -> Self {
        Self {
            token: h2a(*instances.token.address()),
            channels: h2a(*instances.channels.address()),
            announcements: h2a(*instances.announcements.address()),
            node_safe_registry: h2a(*instances.safe_registry.address()),
            ticket_price_oracle: h2a(*instances.price_oracle.address()),
            winning_probability_oracle: h2a(*instances.win_prob_oracle.address()),
            node_safe_migration: hopr_primitive_types::prelude::Address::default(),
            node_stake_factory: h2a(*instances.stake_factory.address()),
            module_implementation: h2a(*instances.module_implementation.address()),
        }
    }
}

/// Implement ethers-rs `get_create2_address` function
/// Returns the CREATE2 address of a smart contract as specified in
/// [EIP1014](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1014.md)
///
/// keccak256( 0xff ++ senderAddress ++ salt ++ keccak256(init_code))[12..]
pub fn get_create2_address(from: Address, salt: impl AsRef<[u8]>, init_code: impl AsRef<[u8]>) -> Address {
    let salt = salt.as_ref();
    let init_code_hash = keccak256(init_code.as_ref());

    let mut bytes = Vec::with_capacity(1 + 20 + salt.len() + init_code_hash.len());
    bytes.push(0xff);
    bytes.extend_from_slice(from.as_slice());
    bytes.extend_from_slice(salt);
    bytes.extend_from_slice(&init_code_hash.0);

    let hash = keccak256(bytes);

    let mut bytes = [0u8; 20];
    bytes.copy_from_slice(&hash[12..]);
    Address::from(bytes)
}

// define basic safe abi
sol!(
    #![sol(abi)]
    #![sol(rpc)]
    #[allow(clippy::too_many_arguments)]
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
pub fn create_native_transfer<N>(to: hopr_primitive_types::prelude::Address, amount: U256) -> N::TransactionRequest
where
    N: alloy::providers::Network,
{
    N::TransactionRequest::default().with_to(a2h(to)).with_value(amount)
}

/// Funds the given wallet address with specified amount of native tokens and HOPR tokens.
/// These must be present in the client's wallet.
pub async fn fund_node<P, N>(
    node: hopr_primitive_types::prelude::Address,
    native_token: U256,
    hopr_token: U256,
    hopr_token_contract: HoprTokenInstance<P, N>,
) -> ContractResult<()>
where
    P: alloy::contract::private::Provider<N>,
    N: alloy::providers::Network,
{
    let native_transfer_tx = N::TransactionRequest::default()
        .with_to(a2h(node))
        .with_value(native_token);

    // let native_transfer_tx = Eip1559TransactionRequest::new()
    //     .to(NameOrAddress::Address(node.into()))
    //     .value(native_token);

    let provider = hopr_token_contract.provider();

    provider.send_transaction(native_transfer_tx).await?.watch().await?;

    hopr_token_contract
        .transfer(a2h(node), hopr_token)
        .send()
        .await?
        .watch()
        .await?;
    Ok(())
}

/// Funds the channel to the counterparty with the given amount of HOPR tokens.
/// The amount must be present in the wallet of the client.
pub async fn fund_channel<P, N>(
    counterparty: hopr_primitive_types::prelude::Address,
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
        .fundChannel(a2h(counterparty), aliases::U96::from(amount))
        .send()
        .await?
        .watch()
        .await?;

    Ok(())
}

/// Funds the channel to the counterparty with the given amount of HOPR tokens, from a different client
/// The amount must be present in the wallet of the client.
pub async fn fund_channel_from_different_client<P, N>(
    counterparty: hopr_primitive_types::prelude::Address,
    hopr_token_address: hopr_primitive_types::prelude::Address,
    hopr_channels_address: hopr_primitive_types::prelude::Address,
    amount: U256,
    new_client: P,
) -> ContractResult<()>
where
    P: alloy::contract::private::Provider<N> + Clone,
    N: alloy::providers::Network,
{
    let hopr_token_with_new_client: HoprTokenInstance<P, N> =
        HoprTokenInstance::new(a2h(hopr_token_address), new_client.clone());
    let hopr_channels_with_new_client = HoprChannelsInstance::new(a2h(hopr_channels_address), new_client.clone());
    hopr_token_with_new_client
        .approve(a2h(hopr_channels_address), amount)
        .send()
        .await?
        .watch()
        .await?;

    hopr_channels_with_new_client
        .fundChannel(a2h(counterparty), aliases::U96::from(amount))
        .send()
        .await?
        .watch()
        .await?;

    Ok(())
}

/// Prepare a safe transaction
pub async fn get_safe_tx<P, N>(
    safe_contract: SafeContractInstance<P, N>,
    target: hopr_primitive_types::prelude::Address,
    inner_tx_data: Bytes,
    wallet: PrivateKeySigner,
) -> anyhow::Result<N::TransactionRequest>
where
    P: alloy::contract::private::Provider<N>,
    N: alloy::providers::Network,
{
    let nonce = safe_contract.nonce().call().await?;

    let data_hash = safe_contract
        .getTransactionHash(
            a2h(target),
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
        to: a2h(target),
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
    safe_address: hopr_primitive_types::prelude::Address,
    module_address: hopr_primitive_types::prelude::Address,
    node_address: hopr_primitive_types::prelude::Address,
    deployer: &ChainKeypair, // also node address
) -> anyhow::Result<()>
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
        nodeDefaultTarget: U256::from_str(&node_target_permission)?,
    }
    .abi_encode();

    let safe_contract = SafeContract::new(a2h(safe_address), provider.clone());
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
    safe_address: hopr_primitive_types::prelude::Address,
    module_address: hopr_primitive_types::prelude::Address,
    announcement_contract_address: hopr_primitive_types::prelude::Address,
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

    let safe_contract = SafeContract::new(a2h(safe_address), provider.clone());
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
    safe_address: hopr_primitive_types::prelude::Address,
    token_address: hopr_primitive_types::prelude::Address,
    channel_address: hopr_primitive_types::prelude::Address,
    deployer: &ChainKeypair, // also node address
) -> ContractResult<()>
where
    P: alloy::contract::private::Provider<N> + Clone,
    N: alloy::providers::Network,
{
    // Inner tx payload: include node to the module
    let inner_tx_data = HoprToken::approveCall {
        spender: a2h(channel_address),
        value: U256::MAX,
    }
    .abi_encode();

    let safe_contract = SafeContract::new(a2h(safe_address), provider.clone());
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
) -> ContractResult<(
    hopr_primitive_types::prelude::Address,
    hopr_primitive_types::prelude::Address,
)>
where
    P: alloy::providers::Provider + Clone,
{
    // Get deployer address
    let self_address: hopr_primitive_types::prelude::Address = deployer.public().to_address();

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
                .send_raw_transaction(&crate::constants::SAFE_DIAMOND_PROXY_SINGLETON_DEPLOY_CODE)
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
                .with_input(crate::constants::SAFE_PROXY_FACTORY_DEPLOY_CODE);

            // 2. Handler: only CompatibilityFallbackHandler and omit TokenCallbackHandler as it's not used now
            let _tx_safe_compatibility_fallback_handler = TransactionRequest::default()
                .with_to(safe_diamond_proxy_address)
                .with_input(crate::constants::SAFE_COMPATIBILITY_FALLBACK_HANDLER_DEPLOY_CODE);
            // 3. Library: only MultiSendCallOnly and omit MultiSendCall
            let _tx_safe_multisend_call_only = TransactionRequest::default()
                .with_to(safe_diamond_proxy_address)
                .with_input(crate::constants::SAFE_MULTISEND_CALL_ONLY_DEPLOY_CODE);
            // 4. Safe singleton deploySafe();
            let _tx_safe_singleton = TransactionRequest::default()
                .with_to(safe_diamond_proxy_address)
                .with_input(crate::constants::SAFE_SINGLETON_DEPLOY_CODE);
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
        .get_transaction_count(a2h(self_address))
        .pending()
        //  Some(BlockNumber::Pending.into()))
        .await
        .unwrap();
    debug!(%curr_nonce, "curr_nonce");

    // FIXME: Check if this is the correct way to get the nonce
    let nonce = keccak256((a2h(self_address), U256::from(curr_nonce)).abi_encode_packed());
    let default_target = format!("{:?}010103020202020202020202", instances.channels.address());

    debug!(%self_address, "self_address");
    debug!("nonce {:?}", U256::from_be_bytes(nonce.0).to_string());
    debug!("default_target in bytes {:?}", default_target.bytes());
    debug!("default_target in u256 {:?}", U256::from_str(&default_target).unwrap());

    let typed_tx = HoprNodeStakeFactory::cloneCall {
        // moduleSingletonAddress: *instances.module_implementation.address(),
        admins: vec![a2h(self_address)],
        nonce: nonce.into(),
        defaultTarget: U256::from_str(&default_target).unwrap().into(),
    }
    .abi_encode();

    debug!("typed_tx {:?}", typed_tx);

    // deploy one safe and one module
    let instance_deployment_tx_receipt = instances
        .stake_factory
        .clone(
            //*instances.module_implementation.address(),
            nonce.into(),
            U256::from_str(&default_target).unwrap().into(),
            vec![a2h(self_address)],
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

    Ok((h2a(deployed_module_address), h2a(deployed_safe_address)))
}
