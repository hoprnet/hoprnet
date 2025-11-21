//! This module contains all the methods used for onchain interaction, especially with Safe instance, Mutlicall, and
//! Multisend contracts.
//!
//! [SafeTxOperation] corresponds to the `Operation` Enum used in Safe smart contract.
//!
//! [MultisendTransaction] struct is used for building transactions interacting with Multisend contract

#![allow(clippy::too_many_arguments)]

use std::{ops::Add, str::FromStr, sync::Arc};

use IMulticall3Extract::IMulticall3ExtractInstance;
use SafeSingleton::{SafeSingletonInstance, execTransactionCall, removeOwnerCall, setupCall};
use hex_literal::hex;
use hopr_bindings::{
    exports::alloy::{
        network::TransactionBuilder,
        primitives::{Address, B256, Bytes, U256, keccak256, utils::format_units},
        providers::{
            CallInfoTrait, CallItem, MULTICALL3_ADDRESS, MulticallBuilder, MulticallError, Provider, WalletProvider,
            bindings::IMulticall3::{Call3, Call3Value, aggregate3Call, aggregate3ValueCall},
        },
        rpc::types::TransactionRequest,
        signers::{Signer, local::PrivateKeySigner},
        sol,
        sol_types::{SolCall, SolValue},
    },
    hopr_node_management_module::HoprNodeManagementModule::{
        HoprNodeManagementModuleInstance, addChannelsAndTokenTargetCall, includeNodeCall, initializeCall,
        removeNodeCall, scopeTargetTokenCall,
    },
    hopr_node_safe_registry::HoprNodeSafeRegistry::{HoprNodeSafeRegistryInstance, deregisterNodeBySafeCall},
    hopr_node_stake_factory::HoprNodeStakeFactory::{HoprNodeStakeFactoryInstance, cloneCall},
    hopr_token::HoprToken::{HoprTokenInstance, approveCall},
};
use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
use tracing::{debug, info};

use crate::{
    constants::{
        DEFAULT_ANNOUNCEMENT_PERMISSIONS, DEFAULT_CAPABILITY_PERMISSIONS, DEFAULT_NODE_PERMISSIONS,
        DOMAIN_SEPARATOR_TYPEHASH, ERC_1967_PROXY_CREATION_CODE, ETH_VALUE_FOR_MULTICALL3_DEPLOYER,
        MULTICALL3_DEPLOYER, SAFE_COMPATIBILITYFALLBACKHANDLER_ADDRESS, SAFE_MULTISEND_ADDRESS, SAFE_SAFE_L2_ADDRESS,
        SAFE_SAFEPROXYFACTORY_ADDRESS, SAFE_TX_TYPEHASH, SENTINEL_OWNERS,
    },
    utils::{HelperErrors, get_create2_address},
};

sol!(
    #![sol(abi)]
    #![sol(rpc)]
    contract SafeSingleton {
        event ExecutionSuccess(bytes32 indexed txHash, uint256 payment);

        function setup(address[],uint256,address,bytes,address,address,uint256,address);
        function execTransaction(address to, uint256 value, bytes calldata data, uint8 operation, uint256 safeTxGas, uint256 baseGas, uint256 gasPrice, address gasToken, address payable refundReceiver, bytes memory signatures) public payable returns (bool);
        function removeOwner(address prevOwner, address owner, uint256 _threshold) public;
        function getThreshold() public view returns (uint256);
        function getOwners() public view returns (address[] memory);
        function nonce() public view returns (uint256);
        function domainSeparator() public view returns (bytes32);
        function encodeTransactionData(address to, uint256 value, bytes calldata data, uint8 operation, uint256 safeTxGas, uint256 baseGas, uint256 gasPrice, address gasToken, address refundReceiver, uint256 _nonce) public view returns (bytes memory);
        function getTransactionHash(address to, uint256 value, bytes calldata data, uint8 operation, uint256 safeTxGas, uint256 baseGas, uint256 gasPrice, address gasToken, address refundReceiver, uint256 _nonce) public view returns (bytes32);
        function isModuleEnabled(address module) public view returns (bool);
    }
);

sol!(
    #![sol(abi)]
    #![sol(rpc)]
    contract ModuleSingleton {
        function isNode(address) external view returns (bool);
        function getTargets() external view returns (uint256[] memory);
        function owner() public view returns (address);
    }
);

sol!(
    #![sol(abi)]
    #![sol(rpc)]
    function multiSend(bytes memory transactions) public payable;
);

sol!(
    #![sol(abi)]
    #![sol(rpc)]
    interface IMulticall3Extract {
        function getEthBalance(address addr) external view returns (uint256 balance);
    }
);

/// Enums of Safe transaction operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SafeTxOperation {
    Call,
    DelegateCall,
}
impl SafeTxOperation {
    /// convert the SafeTxOperation to exact one byte
    pub fn to_byte(&self) -> [u8; 1] {
        match self {
            SafeTxOperation::Call => hex!("00"),
            SafeTxOperation::DelegateCall => hex!("01"),
        }
    }
}

impl From<SafeTxOperation> for u8 {
    fn from(s: SafeTxOperation) -> u8 {
        s as u8
    }
}

/// Struct to make a multisend transaction, mainly used by safe instances
#[derive(Debug, Clone)]
pub struct MultisendTransaction {
    // data paylaod encoded with selector
    pub encoded_data: Bytes,
    // transaction type
    pub tx_operation: SafeTxOperation,
    // target address
    pub to: Address,
    // payable eth sending along the tx
    pub value: U256,
}

/// Methods for Multisend transaction
impl MultisendTransaction {
    /// encode a multisend transaction
    fn encode_packed(&self) -> Vec<u8> {
        let tx_operation_bytes: Bytes = self.tx_operation.to_byte().into();

        let value = (
            tx_operation_bytes,                  // 1 bytes
            self.to,                             // 20 bytes
            U256::from(self.value),              // 32 bytes
            U256::from(self.encoded_data.len()), // 32 bytes
            self.encoded_data.clone(),           // bytes
        );
        value.abi_encode_packed()
    }

    /// build a multisend transaction data payload
    fn build_multisend_tx(transactions: Vec<MultisendTransaction>) -> Vec<u8> {
        let mut payload: Vec<u8> = Vec::new();
        for transaction in transactions {
            payload = [payload, transaction.encode_packed()].concat();
        }
        debug!("payload {:?}", hex::encode(&payload));
        payload
    }
}

/// get the domain separator of a safe instance
/// contract_address should be safe address
fn get_domain_separator(chain_id: U256, contract_address: Address) -> [u8; 32] {
    keccak256(
        (
            B256::from_str(DOMAIN_SEPARATOR_TYPEHASH)
                .unwrap_or_else(|_| panic!("decode the DOMAIN_SEPARATOR_TYPEHASH")), // DOMAIN_SEPARATOR_TYPEHASH
            chain_id,         // getChainId
            contract_address, // this
        )
            .abi_encode(),
    )
    .into()
}

/// Implement getTransactionHash() function as in vendor/solidity/safe-contracts-1.4.1/contracts/Safe.sol
/// Note that `safeTxGas`, `baseGas`, and `gasPrice` are zero; `gasToken` is also address zero
fn get_safe_transaction_hash(
    to: Address,
    value: U256,
    data: Vec<u8>,
    operation: SafeTxOperation,
    refund_address: Address,
    nonce: U256,
    domain_separator: [u8; 32],
) -> [u8; 32] {
    // first encodeTransactionData()
    let data_hash = keccak256(data);

    let encoded = (
        B256::from_str(SAFE_TX_TYPEHASH).unwrap_or_else(|_| panic!("failed to decode the SAFE_TX_TYPEHASH")), // SAFE_TX_TYPEHASH
        to,                                                                                                   // to
        value,                                                                                                // value
        data_hash,                   // keccak256
        U256::from(operation as u8), // operation
        U256::ZERO,                  // safeTxGas
        U256::ZERO,                  // baseGas
        U256::ZERO,                  // gasPrice
        Address::ZERO,               // gasToken
        refund_address,              // refundReceiver
        nonce,                       // _nonce
    )
        .abi_encode();

    let safe_hash = keccak256(encoded);

    let encoded_transaction_data = (hex!("1901"), domain_separator, safe_hash).abi_encode_packed();

    let transaction_hash = keccak256(encoded_transaction_data);
    debug!("transaction_hash {:?}", hex::encode(transaction_hash));
    transaction_hash.0
}

/// Use safe to delegatecall to multisend contract
/// Note that when no additional signature is provided, the safe must have a threshold of one,
/// so that the transaction can be executed.
/// Note that the refund address is the caller (safe owner) wallet
pub async fn send_multisend_safe_transaction_with_threshold_one<P: WalletProvider + Provider>(
    safe: SafeSingletonInstance<Arc<P>>,
    signer_key: ChainKeypair,
    multisend_contract: Address,
    multisend_txns: Vec<MultisendTransaction>,
    chain_id: U256,
    nonce: U256,
) -> Result<(), HelperErrors> {
    // get signer
    let signer = safe.provider().default_signer_address();
    // let signer = safe.client().default_sender().expect("client must have a sender");
    let wallet = PrivateKeySigner::from_slice(signer_key.secret().as_ref()).expect("failed to construct wallet");

    // prepare a safe transaction:
    // 1. calculate total value
    let total_value = multisend_txns
        .clone()
        .into_iter()
        .fold(U256::ZERO, |acc, cur| acc.add(cur.value));
    // 2. prepare tx payload
    let tx_payload = MultisendTransaction::build_multisend_tx(multisend_txns);
    let multisend_payload = multiSendCall {
        transactions: tx_payload.into(),
    }
    .abi_encode();
    // 3. get domain separator
    let domain_separator = get_domain_separator(chain_id, *safe.address());

    debug!("multisend_payload {:?}", hex::encode(&multisend_payload));

    // get transaction hash
    let transaction_hash = get_safe_transaction_hash(
        multisend_contract,
        total_value,
        multisend_payload.clone(),
        SafeTxOperation::DelegateCall,
        signer,
        nonce,
        domain_separator,
    );

    // sign the transaction
    let signature = wallet
        .sign_hash(&B256::from_slice(&transaction_hash))
        .await
        .unwrap_or_else(|_| panic!("failed to sign a transaction hash"));
    debug!("signature {:?}", hex::encode(signature.as_bytes()));

    // execute the transaction
    let tx_receipt = safe
        .execTransaction(
            multisend_contract,
            total_value,
            multisend_payload.into(),
            SafeTxOperation::DelegateCall.into(),
            U256::ZERO,
            U256::ZERO,
            U256::ZERO,
            Address::ZERO,
            signer,
            Bytes::from(signature.as_bytes()),
        )
        .send()
        .await?
        // .unwrap_or_else(|_| panic!("failed to exeute a pending transaction"))
        .get_receipt()
        .await?;

    tx_receipt
        .decoded_log::<SafeSingleton::ExecutionSuccess>()
        .ok_or(HelperErrors::MultiSendError)?;
    Ok(())
}

/// Deploy a MULTICALL contract into Anvil local chain for testing
pub async fn deploy_multicall3_for_testing<P: Provider>(provider: Arc<P>) -> Result<(), HelperErrors> {
    // check if the multicall3 contract is already deployed. If deployed, skip all
    let code = provider.get_code_at(MULTICALL3_ADDRESS).await?;
    if code != Bytes::default() {
        info!(
            "Multicall3 contract is already deployed at address {:?}",
            MULTICALL3_ADDRESS
        );
        return Ok(());
    }

    // Fund Multicall3 deployer and deploy ERC1820Registry
    let tx = TransactionRequest::default()
        .with_to(MULTICALL3_DEPLOYER)
        .with_value(ETH_VALUE_FOR_MULTICALL3_DEPLOYER);

    provider
        .send_transaction(tx)
        .await?
        .watch()
        // .map_err(|e| ContractError::MiddlewareError { e })?
        .await?;

    // provider.send_raw_transaction(
    //     &hex!("f90f538085174876e800830f42408080b90f00608060405234801561001057600080fd5b50610ee0806100206000396000f3fe6080604052600436106100f35760003560e01c80634d2301cc1161008a578063a8b0574e11610059578063a8b0574e1461025a578063bce38bd714610275578063c3077fa914610288578063ee82ac5e1461029b57600080fd5b80634d2301cc146101ec57806372425d9d1461022157806382ad56cb1461023457806386d516e81461024757600080fd5b80633408e470116100c65780633408e47014610191578063399542e9146101a45780633e64a696146101c657806342cbb15c146101d957600080fd5b80630f28c97d146100f8578063174dea711461011a578063252dba421461013a57806327e86d6e1461015b575b600080fd5b34801561010457600080fd5b50425b6040519081526020015b60405180910390f35b61012d610128366004610a85565b6102ba565b6040516101119190610bbe565b61014d610148366004610a85565b6104ef565b604051610111929190610bd8565b34801561016757600080fd5b50437fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0140610107565b34801561019d57600080fd5b5046610107565b6101b76101b2366004610c60565b610690565b60405161011193929190610cba565b3480156101d257600080fd5b5048610107565b3480156101e557600080fd5b5043610107565b3480156101f857600080fd5b50610107610207366004610ce2565b73ffffffffffffffffffffffffffffffffffffffff163190565b34801561022d57600080fd5b5044610107565b61012d610242366004610a85565b6106ab565b34801561025357600080fd5b5045610107565b34801561026657600080fd5b50604051418152602001610111565b61012d610283366004610c60565b61085a565b6101b7610296366004610a85565b610a1a565b3480156102a757600080fd5b506101076102b6366004610d18565b4090565b60606000828067ffffffffffffffff8111156102d8576102d8610d31565b60405190808252806020026020018201604052801561031e57816020015b6040805180820190915260008152606060208201528152602001906001900390816102f65790505b5092503660005b8281101561047757600085828151811061034157610341610d60565b6020026020010151905087878381811061035d5761035d610d60565b905060200281019061036f9190610d8f565b6040810135958601959093506103886020850185610ce2565b73ffffffffffffffffffffffffffffffffffffffff16816103ac6060870187610dcd565b6040516103ba929190610e32565b60006040518083038185875af1925050503d80600081146103f7576040519150601f19603f3d011682016040523d82523d6000602084013e6103fc565b606091505b50602080850191909152901515808452908501351761046d577f08c379a000000000000000000000000000000000000000000000000000000000600052602060045260176024527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060445260846000fd5b5050600101610325565b508234146104e6576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601a60248201527f4d756c746963616c6c333a2076616c7565206d69736d6174636800000000000060448201526064015b60405180910390fd5b50505092915050565b436060828067ffffffffffffffff81111561050c5761050c610d31565b60405190808252806020026020018201604052801561053f57816020015b606081526020019060019003908161052a5790505b5091503660005b8281101561068657600087878381811061056257610562610d60565b90506020028101906105749190610e42565b92506105836020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff166105a66020850185610dcd565b6040516105b4929190610e32565b6000604051808303816000865af19150503d80600081146105f1576040519150601f19603f3d011682016040523d82523d6000602084013e6105f6565b606091505b5086848151811061060957610609610d60565b602090810291909101015290508061067d576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601760248201527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060448201526064016104dd565b50600101610546565b5050509250929050565b43804060606106a086868661085a565b905093509350939050565b6060818067ffffffffffffffff8111156106c7576106c7610d31565b60405190808252806020026020018201604052801561070d57816020015b6040805180820190915260008152606060208201528152602001906001900390816106e55790505b5091503660005b828110156104e657600084828151811061073057610730610d60565b6020026020010151905086868381811061074c5761074c610d60565b905060200281019061075e9190610e76565b925061076d6020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff166107906040850185610dcd565b60405161079e929190610e32565b6000604051808303816000865af19150503d80600081146107db576040519150601f19603f3d011682016040523d82523d6000602084013e6107e0565b606091505b506020808401919091529015158083529084013517610851577f08c379a000000000000000000000000000000000000000000000000000000000600052602060045260176024527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060445260646000fd5b50600101610714565b6060818067ffffffffffffffff81111561087657610876610d31565b6040519080825280602002602001820160405280156108bc57816020015b6040805180820190915260008152606060208201528152602001906001900390816108945790505b5091503660005b82811015610a105760008482815181106108df576108df610d60565b602002602001015190508686838181106108fb576108fb610d60565b905060200281019061090d9190610e42565b925061091c6020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff1661093f6020850185610dcd565b60405161094d929190610e32565b6000604051808303816000865af19150503d806000811461098a576040519150601f19603f3d011682016040523d82523d6000602084013e61098f565b606091505b506020830152151581528715610a07578051610a07576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601760248201527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060448201526064016104dd565b506001016108c3565b5050509392505050565b6000806060610a2b60018686610690565b919790965090945092505050565b60008083601f840112610a4b57600080fd5b50813567ffffffffffffffff811115610a6357600080fd5b6020830191508360208260051b8501011115610a7e57600080fd5b9250929050565b60008060208385031215610a9857600080fd5b823567ffffffffffffffff811115610aaf57600080fd5b610abb85828601610a39565b90969095509350505050565b6000815180845260005b81811015610aed57602081850181015186830182015201610ad1565b81811115610aff576000602083870101525b50601f017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0169290920160200192915050565b600082825180855260208086019550808260051b84010181860160005b84811015610bb1578583037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe001895281518051151584528401516040858501819052610b9d81860183610ac7565b9a86019a9450505090830190600101610b4f565b5090979650505050505050565b602081526000610bd16020830184610b32565b9392505050565b600060408201848352602060408185015281855180845260608601915060608160051b870101935082870160005b82811015610c52577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa0888703018452610c40868351610ac7565b95509284019290840190600101610c06565b509398975050505050505050565b600080600060408486031215610c7557600080fd5b83358015158114610c8557600080fd5b9250602084013567ffffffffffffffff811115610ca157600080fd5b610cad86828701610a39565b9497909650939450505050565b838152826020820152606060408201526000610cd96060830184610b32565b95945050505050565b600060208284031215610cf457600080fd5b813573ffffffffffffffffffffffffffffffffffffffff81168114610bd157600080fd5b600060208284031215610d2a57600080fd5b5035919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b7f4e487b7100000000000000000000000000000000000000000000000000000000600052603260045260246000fd5b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff81833603018112610dc357600080fd5b9190910192915050565b60008083357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe1843603018112610e0257600080fd5b83018035915067ffffffffffffffff821115610e1d57600080fd5b602001915036819003821315610a7e57600080fd5b8183823760009101908152919050565b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc1833603018112610dc357600080fd5b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa1833603018112610dc357600080fdfea2646970667358221220bb2b5c71a328032f97c676ae39a1ec2148d3e5d6f73d95e9b17910152d61f16264736f6c634300080c00331ca0edce47092c0f398cebf3ffc267f05c8e7076e3b89445e0fe50f6332273d4569ba01b0b9d000e19b24c5869b0fc3b22b0d6fa47cd63316875cbbd577d76e6fde086")
    //         ).await?
    //         .watch()
    //         // .map_err(|e| ContractError::MiddlewareError {e})?
    //         .await?;
    Ok(())
}

/// Get chain id and safe nonce
pub async fn get_chain_id_and_safe_nonce<P: Provider>(
    safe: SafeSingletonInstance<P>,
) -> Result<(U256, U256), HelperErrors> {
    let provider = safe.provider();
    let multicall = provider.multicall().get_chain_id().add(safe.nonce());
    let (get_chain_id_return, nonce_return) = multicall.aggregate().await?;

    Ok((get_chain_id_return, nonce_return))
}

/// Get native balance and hopr token balance for given addresses
pub async fn get_native_and_token_balances<P: Provider>(
    hopr_token: HoprTokenInstance<P>,
    addresses: Vec<Address>,
) -> Result<(Vec<U256>, Vec<U256>), MulticallError> {
    let provider = hopr_token.provider();
    let multicall3_instance = IMulticall3ExtractInstance::new(MULTICALL3_ADDRESS, provider);

    // if there is less than two addresses, use multicall3 on each address
    // otherwise, make multicall on all addresses
    if addresses.is_empty() {
        Ok((vec![], vec![]))
    } else if addresses.len() == 1 {
        let address = addresses[0];
        let multicall = provider
            .multicall()
            .get_eth_balance(address)
            .add(hopr_token.balanceOf(address));

        let (native_balance, token_balance) = multicall.aggregate().await?;
        Ok((vec![native_balance], vec![token_balance]))
    } else {
        let mut native_balances_multicall = MulticallBuilder::new_dynamic(provider);
        let mut token_balances_multicall = MulticallBuilder::new_dynamic(provider);

        for address in addresses {
            native_balances_multicall =
                native_balances_multicall.add_dynamic(multicall3_instance.getEthBalance(address));
            token_balances_multicall = token_balances_multicall.add_dynamic(hopr_token.balanceOf(address));
            // balances_multicall.add_call(hopr_token.balanceOf(address));
        }

        let native_balances_return = native_balances_multicall.aggregate().await?;
        let token_balances_return = token_balances_multicall.aggregate().await?;

        Ok((native_balances_return, token_balances_return))
    }
}

/// Transfer some HOPR tokens from the caller to the list of addresses
/// Address_i receives amounts_i HOPR tokens.
/// When there's not enough token in caller's balance, if the caller is
/// a minter, mint the missing tokens. If not, returns error
///
/// Attention! Do not use this function to distribute large amount of tokens
///
/// Note that to save gas in batch funding, we use multicall to facilitate token distribution via `transferFrom`
/// To use this functionality, caller must grant Multicall3 contract the exact allowance equal to the sum of tokens
/// to be transferred. As it's a separate function, there is a window between granting the allowance and executing
/// the transactin. Attacker may take advantage of this window and steal tokens from the caller's account.
///
/// TODO: To mitigate this risk, create a MulticallErc777Recipient contract to enable receiption of tokens
/// on the multicall contract and purposely re-entrance with forwarded payload
pub async fn transfer_or_mint_tokens<P: Provider + WalletProvider>(
    hopr_token: HoprTokenInstance<Arc<P>>,
    addresses: Vec<Address>,
    amounts: Vec<U256>,
) -> Result<U256, HelperErrors> {
    let provider = hopr_token.provider();
    let caller = hopr_token.provider().default_signer_address();

    // check if two vectors have the same length
    assert_eq!(
        addresses.len(),
        amounts.len(),
        "addresses and amounts are of different lengths in transfer_or_mint_tokens"
    );

    // early return if no recipient is provided
    if addresses.is_empty() {
        return Ok(U256::ZERO);
    }

    // calculate the sum of tokens to be sent
    let total = amounts.iter().fold(U256::ZERO, |acc, cur| acc.add(cur));
    info!("total amount of HOPR tokens to be transferred {:?}", total.to_string());

    // get caller balance and its role
    let encoded_minter_role = keccak256(b"MINTER_ROLE");
    let multicall = provider
        .multicall()
        .add(
            hopr_token.balanceOf(caller), /* .method::<_, U256>("balanceOf", caller)
                                           * .map_err(|e| HelperErrors::MulticallError(e.to_string()))?,
                                           * false, */
        )
        .add(
            hopr_token.hasRole(encoded_minter_role, caller), /* hopr_token
                                                              *     .method::<_, bool>("hasRole",
                                                              * (encoded_minter_role, caller))
                                                              *     .map_err(|e|
                                                              * HelperErrors::MulticallError(e.to_string()))?,
                                                              * false, */
        );
    let (token_balance_return, has_role_return) = multicall.aggregate().await?;

    // compare the total with caller's current balance. If caller doens't have enough balance, try to mint some.
    // Otherwise, revert
    if total.gt(&token_balance_return) {
        info!("caller does not have enough balance to transfer tokens to recipients.");
        if has_role_return {
            info!("caller tries to mint tokens");
            hopr_token
                .mint(caller, total, Bytes::default(), Bytes::default())
                .send()
                .await?
                // .unwrap_or_else(|_| panic!("failed to exeute a pending transaction"))
                .watch()
                .await?;
            // .unwrap_or_else(|_| panic!("failed to resolve a transaction receipt"));
        } else {
            return Err(HelperErrors::NotAMinter);
        }
    }

    // when there are multiple recipients, use multicall; when single recipient, direct transfer
    if addresses.len() == 1 {
        info!("doing direct transfer...");

        // direct transfer
        hopr_token
            .transfer(addresses[0], amounts[0])
            .send()
            .await?
            // .unwrap_or_else(|_| panic!("failed to exeute a pending transaction"))
            .watch()
            .await?;
        // .unwrap_or_else(|_| panic!("failed to resolve a transaction receipt"));
    } else {
        info!("using multicall...");
        // use multicall
        // TODO: introduce a new ERC777Recipient contract and batch the following separated steps into one, to mitigate
        // the attack vector approve the multicall to be able to transfer from caller's wallet
        hopr_token
            .approve(MULTICALL3_ADDRESS, total)
            .send()
            .await?
            // .unwrap_or_else(|_| panic!("failed to exeute a pending transaction"))
            .watch()
            .await?;

        let calls: Vec<Call3> = addresses
            .clone()
            .into_iter()
            .enumerate()
            .map(|(i, addr)| {
                let calldata = hopr_token.transferFrom(caller, addr, amounts[i]);
                let call = Call3 {
                    target: *hopr_token.address(),
                    allowFailure: false,
                    callData: calldata.calldata().clone(),
                };
                call
            })
            .collect::<Vec<_>>();
        let aggregate3_payload = aggregate3Call { calls }.abi_encode();
        let tx = TransactionRequest::default()
            .with_to(MULTICALL3_ADDRESS)
            .with_input(aggregate3_payload);
        provider.send_transaction(tx).await?.watch().await?;
    }

    Ok(total)
}

/// Transfer some native tokens from the caller to the list of addresses
/// Address_i receives amounts_i native tokens.
pub async fn transfer_native_tokens<P: Provider + WalletProvider>(
    provider: Arc<P>,
    addresses: Vec<Address>,
    amounts: Vec<U256>,
) -> Result<U256, HelperErrors> {
    // check if two vectors have the same length
    assert_eq!(
        addresses.len(),
        amounts.len(),
        "addresses and amounts are of different lengths in transfer_native_tokens"
    );
    // calculate the sum of tokens to be sent
    let total = amounts.iter().fold(U256::ZERO, |acc, cur| acc.add(cur));
    info!(
        "total amount of native tokens to be transferred {:?}",
        total.to_string()
    );

    let calls: Vec<Call3Value> = addresses
        .clone()
        .into_iter()
        .enumerate()
        .map(|(i, addr)| Call3Value {
            target: addr,
            allowFailure: false,
            value: amounts[i],
            callData: Bytes::default(),
        })
        .collect::<Vec<_>>();
    let aggregate3_value_payload = aggregate3ValueCall { calls }.abi_encode();
    let tx = TransactionRequest::default()
        .with_to(MULTICALL3_ADDRESS)
        .with_input(aggregate3_value_payload)
        .with_value(total);
    provider.send_transaction(tx).await?.watch().await?;
    Ok(total)
}

/// Helper function to predict module address. Note that here the caller is the contract deployer
/// FIXME: The result mismatch from predicted module address from smart contract
pub fn predict_module_address(
    caller: Address,
    nonce: B256,
    safe_address: Address,
    announcement_address: Address,
    factory_address: Address,
    default_target: U256,
    implementation_address: Address,
) -> Result<Address, HelperErrors> {
    let module_salt = keccak256((caller, nonce).abi_encode_packed());
    // debug!("module_salt {:?}", module_salt);

    let default_announcement_target =
        U256::from_str(format!("{announcement_address:?}{DEFAULT_ANNOUNCEMENT_PERMISSIONS}").as_str()).unwrap();

    let initialize_parameters = (
        safe_address,
        SAFE_MULTISEND_ADDRESS,
        default_announcement_target,
        default_target,
    )
        .abi_encode();

    let encode_initialization = initializeCall {
        initParams: initialize_parameters.into(),
    }
    .abi_encode();

    let erc1967_initialize_code = (implementation_address, encode_initialization).abi_encode()[32..].to_vec();
    debug!("erc1967_initialize_code {:?}", hex::encode(&erc1967_initialize_code));

    let module_creation_code = (
        Bytes::copy_from_slice(ERC_1967_PROXY_CREATION_CODE),
        erc1967_initialize_code,
    )
        .abi_encode_packed();
    debug!("module_creation_code {:?}", hex::encode(&module_creation_code));
    debug!(
        "module_creation_code_hash {:?}",
        hex::encode(keccak256(&module_creation_code))
    );

    let predict_module_addr = get_create2_address(factory_address, module_salt, keccak256(&module_creation_code));
    debug!("predict_module_addr {:?}", predict_module_addr);

    Ok(predict_module_addr)
}

/// Helper function to predict safe address
pub fn predict_safe_address(
    stake_factory: Address,
    admins: Vec<Address>,
    nonce: B256,
    safe_fallback: Address,
    safe_singleton: Address,
    safe_factory: Address,
) -> Result<Address, HelperErrors> {
    let mut temp_admins = admins.clone();
    temp_admins.push(stake_factory);

    let initializer = setupCall {
        _0: temp_admins,
        _1: U256::ONE,
        _2: Address::ZERO,
        _3: Bytes::from(hex!("00")),
        _4: safe_fallback,
        _5: Address::ZERO,
        _6: U256::ZERO,
        _7: Address::ZERO,
    }
    .abi_encode();

    let safe_salt = get_salt_from_salt_nonce(initializer, nonce)?;
    debug!("safe_salt {:?}", hex::encode(safe_salt));

    let predict_safe_addr = deploy_proxy(safe_singleton, safe_salt, safe_factory)?;
    debug!("predict_safe_addr {:?}", hex::encode(predict_safe_addr));

    Ok(predict_safe_addr)
}

/// helper function to get salt nonce
fn get_salt_from_salt_nonce(initializer: Vec<u8>, salt_nonce: B256) -> Result<[u8; 32], HelperErrors> {
    let hashed_initializer = keccak256(initializer);
    let encoded = (hashed_initializer, salt_nonce).abi_encode_packed();

    Ok(keccak256(encoded).into())
}

/// helper function to compute create2 safe proxy address
fn deploy_proxy(safe_singleton: Address, safe_salt: [u8; 32], safe_factory: Address) -> Result<Address, HelperErrors> {
    let safe_creation_code = (
        Bytes::from_static(&hex!("608060405234801561001057600080fd5b506040516101e63803806101e68339818101604052602081101561003357600080fd5b8101908080519060200190929190505050600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614156100ca576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260228152602001806101c46022913960400191505060405180910390fd5b806000806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055505060ab806101196000396000f3fe608060405273ffffffffffffffffffffffffffffffffffffffff600054167fa619486e0000000000000000000000000000000000000000000000000000000060003514156050578060005260206000f35b3660008037600080366000845af43d6000803e60008114156070573d6000fd5b3d6000f3fea264697066735822122003d1488ee65e08fa41e58e888a9865554c535f2c77126a82cb4c0f917f31441364736f6c63430007060033496e76616c69642073696e676c65746f6e20616464726573732070726f7669646564")),
        Bytes::from_static(&hex!("000000000000000000000000")),    // pad address to bytes32
        safe_singleton,
    ).abi_encode_packed();
    debug!("safe_creation_code {:?}", hex::encode(safe_creation_code.clone()));

    let predict_safe_addr = get_create2_address(safe_factory, safe_salt, safe_creation_code);
    debug!("predict_safe_addr {:?}", hex::encode(predict_safe_addr));

    Ok(predict_safe_addr.0.into())
}

pub fn prepare_safe_tx_multicall_payload_from_owner_contract(
    deployed_safe: Address,
    target: Address,
    refund_address: Address,
    tx_payload: Vec<u8>,
) -> CallItem<execTransactionCall> {
    let approval_hash_sig = (
        Bytes::from_static(&hex!("000000000000000000000000")),
        MULTICALL3_ADDRESS,
        Bytes::from_static(&hex!(
            "0000000000000000000000000000000000000000000000000000000000000000"
        )),
        Bytes::from_static(&hex!("01")),
    )
        .abi_encode_packed();

    let input = execTransactionCall {
        to: target,
        value: U256::ZERO,
        data: Bytes::from(tx_payload),
        operation: 0u8,
        safeTxGas: U256::ZERO,
        baseGas: U256::ZERO,
        gasPrice: U256::ZERO,
        gasToken: Address::ZERO,
        refundReceiver: refund_address,
        signatures: Bytes::from(approval_hash_sig),
    }
    .abi_encode();

    CallItem::<execTransactionCall>::new(deployed_safe, input.into())
}

/// Deploy a safe and a module proxies via v4 HoprStakeFactory contract with default permissions and announcement
/// targets With the multicall contract, it deploys a safe proxy instance and a module proxy instance with multicall as
/// an owner, and completes necessary setup.
/// Then the multicall includes some additional steps:
/// 1. if node addresses are known, include nodes to the module by safe
/// 2. transfer safe ownership to actual admins
/// 3. set desired threshold
///
/// Returns safe proxy address and module proxy address
#[allow(clippy::too_many_arguments)]
pub async fn deploy_safe_module_with_targets_and_nodes<P: WalletProvider + Provider>(
    hopr_node_stake_factory: HoprNodeStakeFactoryInstance<Arc<P>>,
    hopr_channels_address: Address,
    node_addresses: Option<Vec<Address>>,
    admins: Vec<Address>,
    threshold: U256,
) -> Result<(SafeSingletonInstance<Arc<P>>, HoprNodeManagementModuleInstance<Arc<P>>), HelperErrors> {
    let caller = hopr_node_stake_factory.provider().default_signer_address();
    let provider = hopr_node_stake_factory.provider();

    // check safes owners are provided and threshold is valid
    assert!(!admins.is_empty(), "safe must have valid admin(s)");
    assert!(
        threshold.ge(&U256::ONE) && threshold.le(&U256::from(admins.len())),
        "safe threshold must be at least one and not greater than the total number of admins"
    );
    assert!(
        !admins.contains(&MULTICALL3_ADDRESS),
        "multicall contract cannot be an admin"
    );

    // build a new temporary admin
    let mut temporary_admins: Vec<Address> = admins.clone();
    temporary_admins.insert(0, MULTICALL3_ADDRESS);
    info!(
        "temporary_admins expends from admin from {:?} addresses to {:?}",
        admins.len(),
        temporary_admins.len()
    );

    // build the default permissions of capabilities
    let default_target =
    // let default_target: [u8; 32] =
        U256::from_str(format!("{hopr_channels_address:?}{DEFAULT_CAPABILITY_PERMISSIONS}").as_str())
            .unwrap();
    debug!("default target {:?}", default_target);
    // salt nonce
    let curr_nonce = provider
        .get_transaction_count(caller)
        .pending()
        .await
        .map_err(|e| HelperErrors::MiddlewareError(e.to_string()))?;
    let nonce = keccak256((caller, U256::from(curr_nonce)).abi_encode_packed());

    debug!("curr_nonce {} and nonce {:?}", curr_nonce, nonce);

    let safe_address = predict_safe_address(
        *hopr_node_stake_factory.address(),
        temporary_admins.clone(),
        nonce,
        SAFE_COMPATIBILITYFALLBACKHANDLER_ADDRESS,
        SAFE_SAFE_L2_ADDRESS,
        SAFE_SAFEPROXYFACTORY_ADDRESS,
    )?;
    debug!("predicted safe address {:?}", safe_address.to_string());

    let module_address = hopr_node_stake_factory
        .predictModuleAddress_1(MULTICALL3_ADDRESS, nonce.into(), safe_address, default_target.into())
        .call()
        .await?;
    debug!("predicted module address {:?}", module_address.to_string());

    let deployed_module = HoprNodeManagementModuleInstance::new(module_address, provider.clone());
    let deployed_safe = SafeSingleton::new(safe_address, provider.clone());

    // Use multicall to deploy a safe proxy instance and a module proxy instance with multicall as an owner
    let mut multicall_payloads: Vec<Call3> = vec![];
    let safe_address = *deployed_safe.address();
    multicall_payloads.push(Call3 {
        target: *hopr_node_stake_factory.address(),
        allowFailure: false,
        callData: cloneCall {
            nonce: nonce.into(),
            defaultTarget: default_target.into(),
            admins: temporary_admins,
        }
        .abi_encode()
        .into(),
    });
    info!("Safe and module deployment multicall payload is created");

    // if node addresses are known, include nodes to the module by safe
    if let Some(nodes) = node_addresses {
        for node in nodes {
            let node_target =
                U256::from_str(&format!("{node:?}{DEFAULT_NODE_PERMISSIONS}")).expect("Invalid node_target format");

            let encoded_call = includeNodeCall {
                nodeDefaultTarget: node_target,
            }
            .abi_encode();

            let payload = prepare_safe_tx_multicall_payload_from_owner_contract(
                safe_address,
                module_address,
                caller,
                encoded_call,
            );

            multicall_payloads.push(payload.to_call3());
        }

        info!("Nodes inclusion multicall payload is created");
    } else {
        info!("No node has been provided. Skip node inclusion action for multicall payload generation");
    }

    // renounce ownership granted to multicall so that only actual admins are included. Set the threshold.
    let remove_owner_tx_payload = removeOwnerCall {
        prevOwner: Address::from_str(SENTINEL_OWNERS).unwrap(),
        owner: MULTICALL3_ADDRESS,
        _threshold: threshold,
    }
    .abi_encode();

    let multicall_payload_5 = prepare_safe_tx_multicall_payload_from_owner_contract(
        safe_address,
        safe_address,
        caller,
        remove_owner_tx_payload,
    );
    // let multicall = multicall.add_call(multicall_payload_5);

    multicall_payloads.push(multicall_payload_5.to_call3());
    info!("Admins and threshold setting multicall payload is created");

    // build multicall transaction
    let aggregate3_payload = aggregate3Call {
        calls: multicall_payloads,
    }
    .abi_encode();
    let tx = TransactionRequest::default()
        .with_to(MULTICALL3_ADDRESS)
        .with_input(aggregate3_payload);
    let tx_receipt = provider.send_transaction(tx).await?.get_receipt().await?;
    info!("multicall is sent {:?}", tx_receipt.transaction_hash.to_string());

    let safe_address_from_log = tx_receipt
        .decoded_log::<hopr_bindings::hopr_node_stake_factory::HoprNodeStakeFactory::NewHoprNodeStakeSafe>()
        .ok_or_else(|| HelperErrors::ContractNotDeployed("cannot find safe from log".into()))?
        .instance;
    let module_address_from_log = tx_receipt
        .decoded_log::<hopr_bindings::hopr_node_stake_factory::HoprNodeStakeFactory::NewHoprNodeStakeModule>()
        .ok_or_else(|| HelperErrors::ContractNotDeployed("cannot find module from log".into()))?
        .instance;
    info!("tx_receipt {:?}", tx_receipt);

    assert_eq!(
        safe_address,
        safe_address_from_log,
        "safe address mismatch: predicted {:?} actual {:?}",
        safe_address.to_string(),
        safe_address_from_log.to_string(),
    );
    assert_eq!(
        module_address,
        module_address_from_log,
        "module address mismatch: predicted {:?} actual {:?}",
        module_address.to_string(),
        module_address_from_log.to_string(),
    );
    Ok((deployed_safe, deployed_module))
}

/// Get registered safes for given nodes on the node-safe registry
pub async fn get_registered_safes_for_nodes_on_node_safe_registry<P: Provider>(
    node_safe_registry: HoprNodeSafeRegistryInstance<P>,
    node_addresses: Vec<Address>,
) -> Result<Vec<Address>, MulticallError> {
    let provider = node_safe_registry.provider();
    let mut dyn_multicall = MulticallBuilder::new_dynamic(provider);

    for node in node_addresses {
        dyn_multicall = dyn_multicall.add_dynamic(node_safe_registry.nodeToSafe(node));
    }

    let native_balances_return = dyn_multicall.aggregate().await?;

    Ok(native_balances_return)
}

/// Deregister safes and nodes from the node-safe registry.
/// It returns the number of removed nodes
/// - If nodes have been registered to a safe, remove the node
/// - If nodes have not been registered to any safe, no op
///
/// When deregsitering one node, also remove the node from the module
pub async fn deregister_nodes_from_node_safe_registry_and_remove_from_module<P: WalletProvider + Provider>(
    node_safe_registry: HoprNodeSafeRegistryInstance<Arc<P>>,
    node_addresses: Vec<Address>,
    module_addresses: Vec<Address>,
    owner_chain_key: ChainKeypair,
) -> Result<u32, HelperErrors> {
    let provider = node_safe_registry.provider();
    // check registered safes of given node addresses
    let registered_safes =
        get_registered_safes_for_nodes_on_node_safe_registry(node_safe_registry.clone(), node_addresses.clone())
            .await
            .unwrap();

    let mut nodes_to_remove_counter = 0u32;

    for (i, registered_safe) in registered_safes.iter().enumerate() {
        if registered_safe.ne(&Address::ZERO) {
            // connect to safe
            let safe = SafeSingleton::new(registered_safe.to_owned(), provider.clone());
            // update counter
            nodes_to_remove_counter += 1;
            // get chain id and nonce
            let (chain_id, safe_nonce) = get_chain_id_and_safe_nonce(safe.clone()).await?;

            // for each safe, prepare a multisend transaction to dergister node from safe and remove node from module
            let multisend_txns: Vec<MultisendTransaction> = vec![
                MultisendTransaction {
                    // build multisend tx payload
                    encoded_data: deregisterNodeBySafeCall {
                        nodeAddr: node_addresses[i],
                    }
                    .abi_encode()
                    .into(),
                    tx_operation: SafeTxOperation::Call,
                    to: *node_safe_registry.address(),
                    value: U256::ZERO,
                },
                MultisendTransaction {
                    // build multisend tx payload
                    encoded_data: removeNodeCall {
                        nodeAddress: node_addresses[i],
                    }
                    .abi_encode()
                    .into(),
                    tx_operation: SafeTxOperation::Call,
                    to: module_addresses[i],
                    value: U256::ZERO,
                },
            ];

            // send safe transaction
            send_multisend_safe_transaction_with_threshold_one(
                safe,
                owner_chain_key.clone(),
                SAFE_MULTISEND_ADDRESS,
                multisend_txns,
                chain_id,
                safe_nonce,
            )
            .await?;
        }
    }

    Ok(nodes_to_remove_counter)
}

/// Include nodes to the module
pub async fn include_nodes_to_module<P: WalletProvider + Provider>(
    safe: SafeSingletonInstance<Arc<P>>,
    node_addresses: Vec<Address>,
    module_address: Address,
    owner_chain_key: ChainKeypair,
) -> Result<(), HelperErrors> {
    // get chain id and nonce
    let (chain_id, safe_nonce) = get_chain_id_and_safe_nonce(safe.clone()).await?;

    // prepare a multisend transaction to include each node to the  module
    let mut multisend_txns: Vec<MultisendTransaction> = Vec::new();
    for node_address in node_addresses {
        let node_target = U256::from_str(format!("{node_address:?}{DEFAULT_NODE_PERMISSIONS}").as_str()).unwrap();
        multisend_txns.push(MultisendTransaction {
            encoded_data: includeNodeCall {
                nodeDefaultTarget: node_target,
            }
            .abi_encode()
            .into(),
            tx_operation: SafeTxOperation::Call,
            to: module_address,
            value: U256::ZERO,
        });
    }

    // send safe transaction
    send_multisend_safe_transaction_with_threshold_one(
        safe,
        owner_chain_key.clone(),
        SAFE_MULTISEND_ADDRESS,
        multisend_txns,
        chain_id,
        safe_nonce,
    )
    .await?;

    Ok(())
}

/// Migrate nodes to be able to run in a new network.
// - scope the Channel contract of the new network to the module as target and set default permissions.
// - scope the Announcement contract as target to the module
// - approve HOPR tokens of the Safe proxy to be transferred by the new Channels contract
pub async fn migrate_nodes<P: WalletProvider + Provider>(
    safe: SafeSingletonInstance<Arc<P>>,
    module_addresses: Address,
    channels_address: Address,
    token_address: Address,
    announcement_address: Address,
    allowance: U256,
    owner_chain_key: ChainKeypair,
) -> Result<(), HelperErrors> {
    let (chain_id, safe_nonce) = get_chain_id_and_safe_nonce(safe.clone()).await?;

    let mut multisend_txns: Vec<MultisendTransaction> = Vec::new();

    // scope channels and tokens contract of the network
    let default_target =
        U256::from_str(format!("{channels_address:?}{DEFAULT_CAPABILITY_PERMISSIONS}").as_str()).unwrap();
    debug!("default target {:?}", default_target);

    multisend_txns.push(MultisendTransaction {
        // build multisend tx payload
        encoded_data: addChannelsAndTokenTargetCall {
            defaultTarget: default_target,
        }
        .abi_encode()
        .into(),
        tx_operation: SafeTxOperation::Call,
        to: module_addresses,
        value: U256::ZERO,
    });

    // scope announcement contract of the new network
    let announcement_target =
        U256::from_str(format!("{announcement_address:?}{DEFAULT_ANNOUNCEMENT_PERMISSIONS}").as_str()).unwrap();

    multisend_txns.push(MultisendTransaction {
        // build multisend tx payload
        encoded_data: scopeTargetTokenCall {
            defaultTarget: announcement_target,
        }
        .abi_encode()
        .into(),
        tx_operation: SafeTxOperation::Call,
        to: module_addresses,
        value: U256::ZERO,
    });

    // approve token transfer by the new Channels contract
    multisend_txns.push(MultisendTransaction {
        // build multisend tx payload
        encoded_data: approveCall {
            spender: channels_address,
            value: allowance,
        }
        .abi_encode()
        .into(),
        tx_operation: SafeTxOperation::Call,
        to: token_address,
        value: U256::ZERO,
    });

    // send safe transaction
    send_multisend_safe_transaction_with_threshold_one(
        safe,
        owner_chain_key.clone(),
        SAFE_MULTISEND_ADDRESS,
        multisend_txns,
        chain_id,
        safe_nonce,
    )
    .await?;

    Ok(())
}

/// Quick check if the following values are correct, for one single node:
/// 1. node xDAI balance
/// 2. If node and safe are associated on Node Safe Registry
pub async fn debug_node_safe_module_setup_on_balance_and_registries<P: Provider>(
    node_safe_registry: HoprNodeSafeRegistryInstance<Arc<P>>,
    node_address: &Address,
) -> Result<Address, MulticallError> {
    let provider = node_safe_registry.provider();
    // let mut multicall = Multicall::new(provider.clone(), Some(MULTICALL_ADDRESS))
    //     .await
    //     .expect("cannot create multicall");

    info!("checking for node {:?}", node_address);
    let multicall = provider
        .multicall()
        // 1. node xDAI balance
        .get_eth_balance(*node_address)
        // 2. get the safe address from the Node Safe Registry
        .add(node_safe_registry.nodeToSafe(*node_address));

    let (node_native_balance, safe_in_nodesafe_registry) = multicall.aggregate().await?;

    info!(
        "node does{:?} have xDAI balance {:?}",
        if node_native_balance.ge(
            &U256::from_str("100000000000000000").unwrap() // 0.1 ether
        ) {
            ""
        } else {
            " NOT"
        },
        format_units(node_native_balance, "ether").unwrap_or("Unknown balance".into())
    );

    if safe_in_nodesafe_registry.eq(&Address::ZERO) {
        info!("Please start the node. It will auto-register to node-safe registry");
    } else {
        info!("safe in node-safe registry {:?}", safe_in_nodesafe_registry);
    }

    Ok(safe_in_nodesafe_registry)
}

/// Quick check if the following values are correct, for one single node:
/// 4. If Safe is owned by the correct owner(s)
/// 5. Safe’s wxHOPR balance and allowance
/// 6. if the module is enabled
/// 7. if node is included in the module
/// 8. Get all the targets of the safe (then check if channel and announcement are there)
/// 9. Get the owner of the module
pub async fn debug_node_safe_module_setup_main<P: Provider>(
    hopr_token: HoprTokenInstance<Arc<P>>,
    module_address: &Address,
    node_address: &Address,
    safe_address: &Address,
    channel_address: &Address,
    announce_address: &Address,
) -> Result<(), MulticallError> {
    let provider = hopr_token.provider();

    let safe = SafeSingleton::new(safe_address.to_owned(), provider.clone());
    let module = ModuleSingleton::new(module_address.to_owned(), provider.clone());

    info!("checking for safe {:?} module {:?}", safe_address, module_address);
    let multicall = provider
        .multicall()
        // 4. get owners of the safe
        .add(safe.getOwners())
        // 5.a. get the wxHOPR balance for the safe address
        .add(hopr_token.balanceOf(*safe_address))
        // 5.b. get the wxHOPR balance for the safe address
        .add(hopr_token.allowance(*safe_address, *channel_address))
        // 6. if the module is enabled
        .add(safe.isModuleEnabled(*module_address))
        // 7. if node is included in the module
        .add(module.isNode(*node_address))
        // 7. get targets of the safe
        .add(module.getTargets())
        // 8. get owner of the module
        .add(module.owner());

    let (
        safe_owners,
        safe_wxhopr_balance,
        safe_wxhopr_allownace,
        is_module_enabled,
        is_node_included,
        module_targets,
        module_owner,
    ) = multicall.aggregate().await?;

    info!("safe has owners: {:?}", safe_owners);
    info!(
        "safe has wxHOPR balance: {:?}",
        format_units(safe_wxhopr_balance, "ether").unwrap_or("Unknown balance".into())
    );
    info!(
        "safe has wxHOPR allowance: {:?}",
        format_units(safe_wxhopr_allownace, "ether").unwrap_or("Unknown balance".into())
    );
    info!("module is enabled: {:?}", is_module_enabled);
    info!("node is included in the module: {:?}", is_node_included);
    info!("module has targets:");
    for target in module_targets {
        let target_address = format!("{target:#x}");
        let has_channels = target_address.contains(&format!("{channel_address:#x}"));
        let has_announcement = target_address.contains(&format!("{announce_address:#x}"));
        // check if it contains channel and announcement
        info!(
            "Target {:?} has channels {:?} has announcement {:?}",
            target_address, has_channels, has_announcement
        );
    }

    info!(
        "module owner: {:?} same as safe address: {:?}",
        module_owner,
        module_owner.eq(safe_address)
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::vec;

    use hopr_bindings::{
        exports::alloy::{
            contract::Result as ContractResult,
            network::{EthereumWallet, TransactionBuilder},
            primitives::address,
            providers::{Identity, RootProvider, fillers::*},
            rpc::types::TransactionRequest,
            sol_types::SolValue,
        },
        hopr_announcements::HoprAnnouncements,
        hopr_channels::HoprChannels,
        hopr_node_safe_registry::HoprNodeSafeRegistry,
        hopr_node_stake_factory::HoprNodeStakeFactory,
        hopr_token::HoprToken,
    };
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
    use hopr_primitive_types::prelude::BytesRepresentable;
    use tracing_subscriber::{EnvFilter, Registry, fmt, prelude::*};

    use super::*;
    use crate::utils::{ContractInstances, a2h, create_anvil};

    pub type AnvilRpcClient = FillProvider<
        JoinFill<
            JoinFill<Identity, JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>>,
            WalletFiller<EthereumWallet>,
        >,
        RootProvider,
    >;

    fn init_tracing() {
        // Use RUST_LOG if set, otherwise default to "debug" for verbose test output
        let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));

        // Match main.rs formatting style
        let format = fmt::layer()
            .with_level(true)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(false)
            .with_test_writer(); // ensures logs show up in `cargo test`

        // Set the global subscriber (harmless no-op if already initialized)
        let _ = Registry::default().with(env_filter).with(format).try_init();
    }

    /// Used for testing. Creates RPC client to the local Anvil instance.
    pub fn create_rpc_client_to_anvil(
        anvil: &hopr_bindings::exports::alloy::node_bindings::AnvilInstance,
        signer: &hopr_crypto_types::keypairs::ChainKeypair,
    ) -> Arc<AnvilRpcClient> {
        use hopr_bindings::exports::alloy::{
            providers::ProviderBuilder, rpc::client::ClientBuilder, signers::local::PrivateKeySigner,
            transports::http::ReqwestTransport,
        };
        use hopr_crypto_types::keypairs::Keypair;

        let wallet = PrivateKeySigner::from_slice(signer.secret().as_ref()).expect("failed to construct wallet");

        let transport_client = ReqwestTransport::new(anvil.endpoint_url());

        let rpc_client = ClientBuilder::default().transport(transport_client.clone(), transport_client.guess_local());

        let provider = ProviderBuilder::new().wallet(wallet).connect_client(rpc_client);

        Arc::new(provider)
    }

    fn get_random_address_for_testing() -> Address {
        // Creates a random Ethereum address, only used for testing
        Address::new(hopr_crypto_random::random_bytes::<
            { hopr_primitive_types::primitives::Address::SIZE },
        >())
    }

    async fn deploy_safe_suites<P: Provider>(provider: Arc<P>) -> ContractResult<()> {
        // Check if safe suite has been deployed. If so, skip this step
        let code = provider
            .get_code_at(address!("0x914d7Fec6aaC8cd542e72Bca78B30650d45643d7"))
            .await?;
        // .map_err(|e| ContractError::MiddlewareError { e })?;

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

                let tx = provider.send_raw_transaction(
            &hex!("f8a78085174876e800830186a08080b853604580600e600039806000f350fe7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe03601600081602082378035828234f58015156039578182fd5b8082525050506014600cf382f4f5a00dc4d1d21b308094a30f5f93da35e4d72e99115378f135f2295bea47301a3165a0636b822daad40aa8c52dd5132f378c0c0e6d83b4898228c7e21c84e631a0b891")
                ).await?.get_receipt()
                .await?;
                // .unwrap();
                tx.contract_address.unwrap()
            };
            debug!("Safe diamond proxy singleton {:?}", safe_diamond_proxy_address);

            // Deploy minimum Safe suite
            // 1. Safe proxy factory deploySafeProxyFactory();
            let _tx_safe_proxy_factory = TransactionRequest::default().with_to(safe_diamond_proxy_address).with_input(
            hex!("0000000000000000000000000000000000000000000000000000000000000000608060405234801561001057600080fd5b50610bee806100206000396000f3fe608060405234801561001057600080fd5b50600436106100575760003560e01c80631688f0b91461005c5780633408e4701461016b57806353e5d93514610189578063d18af54d1461020c578063ec9e80bb1461033b575b600080fd5b61013f6004803603606081101561007257600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803590602001906401000000008111156100af57600080fd5b8201836020820111156100c157600080fd5b803590602001918460018302840111640100000000831117156100e357600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f8201169050808301925050505050505091929192908035906020019092919050505061044a565b604051808273ffffffffffffffffffffffffffffffffffffffff16815260200191505060405180910390f35b6101736104fe565b6040518082815260200191505060405180910390f35b61019161050b565b6040518080602001828103825283818151815260200191508051906020019080838360005b838110156101d15780820151818401526020810190506101b6565b50505050905090810190601f1680156101fe5780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b61030f6004803603608081101561022257600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff1690602001909291908035906020019064010000000081111561025f57600080fd5b82018360208201111561027157600080fd5b8035906020019184600183028401116401000000008311171561029357600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f82011690508083019250505050505050919291929080359060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190505050610536565b604051808273ffffffffffffffffffffffffffffffffffffffff16815260200191505060405180910390f35b61041e6004803603606081101561035157600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff1690602001909291908035906020019064010000000081111561038e57600080fd5b8201836020820111156103a057600080fd5b803590602001918460018302840111640100000000831117156103c257600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f820116905080830192505050505050509192919290803590602001909291905050506106e5565b604051808273ffffffffffffffffffffffffffffffffffffffff16815260200191505060405180910390f35b60008083805190602001208360405160200180838152602001828152602001925050506040516020818303038152906040528051906020012090506104908585836107a8565b91508173ffffffffffffffffffffffffffffffffffffffff167f4f51faf6c4561ff95f067657e43439f0f856d97c04d9ec9070a6199ad418e23586604051808273ffffffffffffffffffffffffffffffffffffffff16815260200191505060405180910390a2509392505050565b6000804690508091505090565b60606040518060200161051d906109c5565b6020820181038252601f19601f82011660405250905090565b6000808383604051602001808381526020018273ffffffffffffffffffffffffffffffffffffffff1660601b8152601401925050506040516020818303038152906040528051906020012060001c905061059186868361044a565b9150600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff16146106dc578273ffffffffffffffffffffffffffffffffffffffff16631e52b518838888886040518563ffffffff1660e01b8152600401808573ffffffffffffffffffffffffffffffffffffffff1681526020018473ffffffffffffffffffffffffffffffffffffffff16815260200180602001838152602001828103825284818151815260200191508051906020019080838360005b83811015610674578082015181840152602081019050610659565b50505050905090810190601f1680156106a15780820380516001836020036101000a031916815260200191505b5095505050505050600060405180830381600087803b1580156106c357600080fd5b505af11580156106d7573d6000803e3d6000fd5b505050505b50949350505050565b6000808380519060200120836106f96104fe565b60405160200180848152602001838152602001828152602001935050505060405160208183030381529060405280519060200120905061073a8585836107a8565b91508173ffffffffffffffffffffffffffffffffffffffff167f4f51faf6c4561ff95f067657e43439f0f856d97c04d9ec9070a6199ad418e23586604051808273ffffffffffffffffffffffffffffffffffffffff16815260200191505060405180910390a2509392505050565b60006107b3846109b2565b610825576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040180806020018281038252601f8152602001807f53696e676c65746f6e20636f6e7472616374206e6f74206465706c6f7965640081525060200191505060405180910390fd5b600060405180602001610837906109c5565b6020820181038252601f19601f820116604052508573ffffffffffffffffffffffffffffffffffffffff166040516020018083805190602001908083835b602083106108985780518252602082019150602081019050602083039250610875565b6001836020036101000a038019825116818451168082178552505050505050905001828152602001925050506040516020818303038152906040529050828151826020016000f59150600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff161415610984576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260138152602001807f437265617465322063616c6c206661696c65640000000000000000000000000081525060200191505060405180910390fd5b6000845111156109aa5760008060008651602088016000875af114156109a957600080fd5b5b509392505050565b600080823b905060008111915050919050565b6101e6806109d38339019056fe608060405234801561001057600080fd5b506040516101e63803806101e68339818101604052602081101561003357600080fd5b8101908080519060200190929190505050600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614156100ca576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260228152602001806101c46022913960400191505060405180910390fd5b806000806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055505060ab806101196000396000f3fe608060405273ffffffffffffffffffffffffffffffffffffffff600054167fa619486e0000000000000000000000000000000000000000000000000000000060003514156050578060005260206000f35b3660008037600080366000845af43d6000803e60008114156070573d6000fd5b3d6000f3fea264697066735822122003d1488ee65e08fa41e58e888a9865554c535f2c77126a82cb4c0f917f31441364736f6c63430007060033496e76616c69642073696e676c65746f6e20616464726573732070726f7669646564a26469706673582212200fd975ca8e62d9bf08aa3d09c74b9bdc9d7acba7621835be4187989ddd0e54b164736f6c63430007060033")
        );
            // 2. Handler: only CompatibilityFallbackHandler and omit TokenCallbackHandler as it's not used now
            // 2. Hanlder: deploy Safe ExtensibleFallbackHandler, v1.5.0
            let _tx_safe_compatibility_fallback_handler = TransactionRequest::default().with_to(safe_diamond_proxy_address).with_input(
            hex!("0000000000000000000000000000000000000000000000000000000000000000608060405234801561001057600080fd5b50612d8b806100206000396000f3fe608060405234801561001057600080fd5b50600436106100d35760003560e01c806351cad5ee1161008c5780637f73528b116100665780637f73528b14610ac5578063b435a13b14610b1e578063bc197c8114610ba1578063f23a6e6114610d37576100d4565b806351cad5ee1461095457806361f5401b146109cc57806364f95acc14610a6e576100d4565b806301ffc9a7146105d45780630a3fe05414610637578063150b7a02146106b85780631626ba7e146107ae578063327b533c146108645780633365582c14610906576100d4565b5b6000366060601860003690501015610154576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260178152602001807f696e76616c6964206d6574686f642073656c6563746f7200000000000000000081525060200191505060405180910390fd5b600080600080610162610e37565b9350935093509350600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16141561020d576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260168152602001807f6d6574686f642068616e646c6572206e6f74207365740000000000000000000081525060200191505060405180910390fd5b81156103ed578073ffffffffffffffffffffffffffffffffffffffff166325d6803f85856000803660009060146000369050039261024d93929190612d22565b6040518663ffffffff1660e01b8152600401808673ffffffffffffffffffffffffffffffffffffffff1681526020018573ffffffffffffffffffffffffffffffffffffffff168152602001848152602001806020018281038252848482818152602001925080828437600081840152601f19601f820116905080830192505050965050505050505060006040518083038186803b1580156102ed57600080fd5b505afa158015610301573d6000803e3d6000fd5b505050506040513d6000823e3d601f19601f82011682018060405250602081101561032b57600080fd5b810190808051604051939291908464010000000082111561034b57600080fd5b8382019150602082018581111561036157600080fd5b825186600182028301116401000000008211171561037e57600080fd5b8083526020830192505050908051906020019080838360005b838110156103b2578082015181840152602081019050610397565b50505050905090810190601f1680156103df5780820380516001836020036101000a031916815260200191505b5060405250505094506105c5565b8073ffffffffffffffffffffffffffffffffffffffff166325d6803f85856000803660009060146000369050039261042793929190612d22565b6040518663ffffffff1660e01b8152600401808673ffffffffffffffffffffffffffffffffffffffff1681526020018573ffffffffffffffffffffffffffffffffffffffff168152602001848152602001806020018281038252848482818152602001925080828437600081840152601f19601f8201169050808301925050509650505050505050600060405180830381600087803b1580156104c957600080fd5b505af11580156104dd573d6000803e3d6000fd5b505050506040513d6000823e3d601f19601f82011682018060405250602081101561050757600080fd5b810190808051604051939291908464010000000082111561052757600080fd5b8382019150602082018581111561053d57600080fd5b825186600182028301116401000000008211171561055a57600080fd5b8083526020830192505050908051906020019080838360005b8381101561058e578082015181840152602081019050610573565b50505050905090810190601f1680156105bb5780820380516001836020036101000a031916815260200191505b5060405250505094505b50505050915050805190602001f35b61061f600480360360208110156105ea57600080fd5b8101908080357bffffffffffffffffffffffffffffffffffffffffffffffffffffffff19169060200190929190505050610f14565b60405180821515815260200191505060405180910390f35b6106a26004803603604081101561064d57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080357bffffffffffffffffffffffffffffffffffffffffffffffffffffffff191690602001909291905050506110a0565b6040518082815260200191505060405180910390f35b610779600480360360808110156106ce57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803590602001909291908035906020019064010000000081111561073557600080fd5b82018360208201111561074757600080fd5b8035906020019184600183028401116401000000008311171561076957600080fd5b90919293919293905050506110c5565b60405180827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916815260200191505060405180910390f35b61082f600480360360408110156107c457600080fd5b8101908080359060200190929190803590602001906401000000008111156107eb57600080fd5b8201836020820111156107fd57600080fd5b8035906020019184600183028401116401000000008311171561081f57600080fd5b90919293919293905050506110e2565b60405180827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916815260200191505060405180910390f35b6109046004803603604081101561087a57600080fd5b8101908080357bffffffffffffffffffffffffffffffffffffffffffffffffffffffff19169060200190929190803590602001906401000000008111156108c057600080fd5b8201836020820111156108d257600080fd5b803590602001918460208302840111640100000000831117156108f457600080fd5b90919293919293905050506115ff565b005b6109526004803603604081101561091c57600080fd5b8101908080359060200190929190803573ffffffffffffffffffffffffffffffffffffffff1690602001909291905050506117e9565b005b6109a06004803603604081101561096a57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190505050611a38565b604051808273ffffffffffffffffffffffffffffffffffffffff16815260200191505060405180910390f35b610a6c600480360360408110156109e257600080fd5b8101908080357bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916906020019092919080359060200190640100000000811115610a2857600080fd5b820183602082011115610a3a57600080fd5b80359060200191846020830284011164010000000083111715610a5c57600080fd5b9091929391929390505050611a7a565b005b610ac360048036036040811015610a8457600080fd5b8101908080357bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916906020019092919080359060200190929190505050611c97565b005b610b1c60048036036040811015610adb57600080fd5b8101908080357bffffffffffffffffffffffffffffffffffffffffffffffffffffffff19169060200190929190803515159060200190929190505050611d5c565b005b610b8960048036036040811015610b3457600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080357bffffffffffffffffffffffffffffffffffffffffffffffffffffffff191690602001909291905050506120b9565b60405180821515815260200191505060405180910390f35b610d02600480360360a0811015610bb757600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190640100000000811115610c1457600080fd5b820183602082011115610c2657600080fd5b80359060200191846020830284011164010000000083111715610c4857600080fd5b909192939192939080359060200190640100000000811115610c6957600080fd5b820183602082011115610c7b57600080fd5b80359060200191846020830284011164010000000083111715610c9d57600080fd5b909192939192939080359060200190640100000000811115610cbe57600080fd5b820183602082011115610cd057600080fd5b80359060200191846001830284011164010000000083111715610cf257600080fd5b90919293919293905050506120e8565b60405180827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916815260200191505060405180910390f35b610e02600480360360a0811015610d4d57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803590602001909291908035906020019092919080359060200190640100000000811115610dbe57600080fd5b820183602082011115610dd057600080fd5b80359060200191846001830284011164010000000083111715610df257600080fd5b9091929391929390505050612108565b60405180827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916815260200191505060405180910390f35b600080600080610e45612126565b8094508195505050610f066000808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020600080357fffffffff00000000000000000000000000000000000000000000000000000000167bffffffffffffffffffffffffffffffffffffffffffffffffffffffff19167bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916815260200190815260200160002054612141565b809250819350505090919293565b60007f01ffc9a7000000000000000000000000000000000000000000000000000000007bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff19161480610fdf57507f98c8e097000000000000000000000000000000000000000000000000000000007bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916145b80610fef5750610fee82612169565b5b806110995750600260006110016123db565b73ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000206000837bffffffffffffffffffffffffffffffffffffffffffffffffffffffff19167bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916815260200190815260200160002060009054906101000a900460ff165b9050919050565b6000602052816000526040600020602052806000526040600020600091509150505481565b60006110cf6123e3565b63150b7a0260e01b905095945050505050565b60008060006110ef612126565b91509150600485859050106115a457600085359050635fd7e97d60e01b7bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916817bffffffffffffffffffffffffffffffffffffffffffffffffffffffff191614801561115c575060448686905010155b156115a257600080878760049060449261117893929190612d22565b604081101561118657600080fd5b810190808035906020019092919080359060200190929190505050915091506000600160008773ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020600084815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff169050600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff161461159e576000808a8a600490809261126493929190612d22565b608081101561127257600080fd5b810190808035906020019092919080359060200190929190803590602001906401000000008111156112a357600080fd5b8201836020820111156112b557600080fd5b803590602001918460018302840111640100000000831117156112d757600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f8201169050808301925050505050505091929192908035906020019064010000000081111561133a57600080fd5b82018360208201111561134c57600080fd5b8035906020019184600183028401116401000000008311171561136e57600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f8201169050808301925050505050505091929192905050509350935050508b6113cd86868561262f565b80519060200120141561159b578273ffffffffffffffffffffffffffffffffffffffff166353f00b1489898f898988886040518863ffffffff1660e01b8152600401808873ffffffffffffffffffffffffffffffffffffffff1681526020018773ffffffffffffffffffffffffffffffffffffffff1681526020018681526020018581526020018481526020018060200180602001838103835285818151815260200191508051906020019080838360005b8381101561149a57808201518184015260208101905061147f565b50505050905090810190601f1680156114c75780820380516001836020036101000a031916815260200191505b50838103825284818151815260200191508051906020019080838360005b838110156115005780820151818401526020810190506114e5565b50505050905090810190601f16801561152d5780820380516001836020036101000a031916815260200191505b50995050505050505050505060206040518083038186803b15801561155157600080fd5b505afa158015611565573d6000803e3d6000fd5b505050506040513d602081101561157b57600080fd5b8101908080519060200190929190505050985050505050505050506115f8565b50505b5050505b505b6115f3828787878080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f82011690508083019250505050505050612737565b925050505b9392505050565b6116076123db565b73ffffffffffffffffffffffffffffffffffffffff16611625612a55565b73ffffffffffffffffffffffffffffffffffffffff16146116ae576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040180806020018281038252601e8152602001807f6f6e6c7920736166652063616e2063616c6c2074686973206d6574686f64000081525060200191505060405180910390fd5b60006116b8612a55565b905060008060e01b9050600084849050905060005b818110156117225760008060006116f58989868181106116e957fe5b90506020020135612adf565b92509250925061170f878361170a8685612b14565b612b67565b81861895505050508060010190506116cd565b50857bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916146117d6576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260158152602001807f696e74657266616365206964206d69736d61746368000000000000000000000081525060200191505060405180910390fd5b6117e1866001611d5c565b505050505050565b6117f16123db565b73ffffffffffffffffffffffffffffffffffffffff1661180f612a55565b73ffffffffffffffffffffffffffffffffffffffff1614611898576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040180806020018281038252601e8152602001807f6f6e6c7920736166652063616e2063616c6c2074686973206d6574686f64000081525060200191505060405180910390fd5b60006118a2612a55565b90506000600160008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020600085815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16905082600160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020600086815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508173ffffffffffffffffffffffffffffffffffffffff167f06341ac2f62eb79165de8c8b54ab89a8825b12746c2b98e01dff1007837d2ba8858386604051808481526020018373ffffffffffffffffffffffffffffffffffffffff1681526020018273ffffffffffffffffffffffffffffffffffffffff168152602001935050505060405180910390a250505050565b60016020528160005260406000206020528060005260406000206000915091509054906101000a900473ffffffffffffffffffffffffffffffffffffffff1681565b611a826123db565b73ffffffffffffffffffffffffffffffffffffffff16611aa0612a55565b73ffffffffffffffffffffffffffffffffffffffff1614611b29576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040180806020018281038252601e8152602001807f6f6e6c7920736166652063616e2063616c6c2074686973206d6574686f64000081525060200191505060405180910390fd5b6000611b33612a55565b905060008060e01b9050600084849050905060005b81811015611bd057611b8f84878784818110611b6057fe5b905060200201357bffffffffffffffffffffffffffffffffffffffffffffffffffffffff19166000801b612b67565b858582818110611b9b57fe5b905060200201357bffffffffffffffffffffffffffffffffffffffffffffffffffffffff191683189250806001019050611b48565b50857bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff191614611c84576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260158152602001807f696e74657266616365206964206d69736d61746368000000000000000000000081525060200191505060405180910390fd5b611c8f866000611d5c565b505050505050565b611c9f6123db565b73ffffffffffffffffffffffffffffffffffffffff16611cbd612a55565b73ffffffffffffffffffffffffffffffffffffffff1614611d46576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040180806020018281038252601e8152602001807f6f6e6c7920736166652063616e2063616c6c2074686973206d6574686f64000081525060200191505060405180910390fd5b611d58611d51612a55565b8383612b67565b5050565b611d646123db565b73ffffffffffffffffffffffffffffffffffffffff16611d82612a55565b73ffffffffffffffffffffffffffffffffffffffff1614611e0b576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040180806020018281038252601e8152602001807f6f6e6c7920736166652063616e2063616c6c2074686973206d6574686f64000081525060200191505060405180910390fd5b6000611e156123db565b905063ffffffff60e01b837bffffffffffffffffffffffffffffffffffffffffffffffffffffffff19161415611eb3576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260148152602001807f696e76616c696420696e7465726661636520696400000000000000000000000081525060200191505060405180910390fd5b6000600260008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002090506000816000867bffffffffffffffffffffffffffffffffffffffffffffffffffffffff19167bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916815260200190815260200160002060009054906101000a900460ff169050801515841515146120b25783826000877bffffffffffffffffffffffffffffffffffffffffffffffffffffffff19167bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916815260200190815260200160002060006101000a81548160ff0219169083151502179055508315612043578273ffffffffffffffffffffffffffffffffffffffff167f3d5024c13f12fa602dbf52b1979058940c224ebf170c83a4e358725ae50db36d8660405180827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916815260200191505060405180910390a26120b1565b8273ffffffffffffffffffffffffffffffffffffffff167ff159d834f487747c1b3f17e2107743e42e6eecff444d894e511c18943072b29f8660405180827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916815260200191505060405180910390a25b5b5050505050565b60026020528160005260406000206020528060005260406000206000915091509054906101000a900460ff1681565b60006120f26123e3565b63bc197c8160e01b905098975050505050505050565b60006121126123e3565b63f23a6e6160e01b90509695505050505050565b6000806121316123db565b915061213b612a55565b90509091565b6000808260f81c15915073ffffffffffffffffffffffffffffffffffffffff83169050915091565b60007f150b7a02000000000000000000000000000000000000000000000000000000007bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916148061223457507f4e2312e0000000000000000000000000000000000000000000000000000000007bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916145b8061229c57507f1626ba7e000000000000000000000000000000000000000000000000000000007bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916145b8061230457507f62af8dc2000000000000000000000000000000000000000000000000000000007bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916145b8061236c57507f99372930000000000000000000000000000000000000000000000000000000007bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916145b806123d457507f64f95acc000000000000000000000000000000000000000000000000000000007bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916145b9050919050565b600033905090565b60003373ffffffffffffffffffffffffffffffffffffffff16635624b25b7f6c9a6c4a39284e37ed1cf53d337577d14212a4870fb976a4366c693b939918d560001b60001c60016040518363ffffffff1660e01b8152600401808381526020018281526020019250505060006040518083038186803b15801561246557600080fd5b505afa158015612479573d6000803e3d6000fd5b505050506040513d6000823e3d601f19601f8201168201806040525060208110156124a357600080fd5b81019080805160405193929190846401000000008211156124c357600080fd5b838201915060208201858111156124d957600080fd5b82518660018202830111640100000000821117156124f657600080fd5b8083526020830192505050908051906020019080838360005b8381101561252a57808201518184015260208101905061250f565b50505050905090810190601f1680156125575780820380516001836020036101000a031916815260200191505b506040525050509050600081806020019051602081101561257757600080fd5b810190808051906020019092919050505090503073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff161461262b576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260138152602001807f6e6f7420612066616c6c6261636b2063616c6c0000000000000000000000000081525060200191505060405180910390fd5b5050565b6060601960f81b600160f81b8585856040516020018083815260200182805190602001908083835b6020831061267a5780518252602082019150602081019050602083039250612657565b6001836020036101000a038019825116818451168082178552505050505050905001925050506040516020818303038152906040528051906020012060405160200180857effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff19168152600101847effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916815260010183815260200182815260200194505050505060405160208183030381529060405290509392505050565b60008061282d8573ffffffffffffffffffffffffffffffffffffffff1663f698da256040518163ffffffff1660e01b815260040160206040518083038186803b15801561278357600080fd5b505afa158015612797573d6000803e3d6000fd5b505050506040513d60208110156127ad57600080fd5b81019080805190602001909291905050507f60b3cbf8b4a223d68d641b3b6ddf9a298e7f33710cf3d3a9d1146b5a6150fbca60001b8660405160200180828152602001915050604051602081830303815290604052805190602001206040516020018082815260200191505060405160208183030381529060405261262f565b905060008180519060200120905060008451141561294c5760008673ffffffffffffffffffffffffffffffffffffffff16635ae6bd37836040518263ffffffff1660e01b81526004018082815260200191505060206040518083038186803b15801561289857600080fd5b505afa1580156128ac573d6000803e3d6000fd5b505050506040513d60208110156128c257600080fd5b81019080805190602001909291905050501415612947576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260118152602001807f48617368206e6f7420617070726f76656400000000000000000000000000000081525060200191505060405180910390fd5b612a42565b8573ffffffffffffffffffffffffffffffffffffffff1663f855438b600083876040518463ffffffff1660e01b8152600401808473ffffffffffffffffffffffffffffffffffffffff16815260200183815260200180602001828103825283818151815260200191508051906020019080838360005b838110156129dd5780820151818401526020810190506129c2565b50505050905090810190601f168015612a0a5780820380516001836020036101000a031916815260200191505b5094505050505060006040518083038186803b158015612a2957600080fd5b505afa158015612a3d573d6000803e3d6000fd5b505050505b631626ba7e60e01b925050509392505050565b6000601460003690501015612ad2576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260178152602001807f496e76616c69642063616c6c64617461206c656e67746800000000000000000081525060200191505060405180910390fd5b601436033560601c905090565b60008060008360f81c15925073ffffffffffffffffffffffffffffffffffffffff841690508360a01c60a81b91509193909250565b600082612b41577f0100000000000000000000000000000000000000000000000000000000000000612b44565b60005b8273ffffffffffffffffffffffffffffffffffffffff161760001b905092915050565b60008060008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002090506000816000857bffffffffffffffffffffffffffffffffffffffffffffffffffffffff19167bffffffffffffffffffffffffffffffffffffffffffffffffffffffff191681526020019081526020016000205490506000612c0984612141565b915050600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff161415612c48576000801b93505b83836000877bffffffffffffffffffffffffffffffffffffffffffffffffffffffff19167bffffffffffffffffffffffffffffffffffffffffffffffffffffffff19168152602001908152602001600020819055508573ffffffffffffffffffffffffffffffffffffffff167fe6e8ad7e5547dc860775f9f0638e195a4751a4cfbb7fd2ab60a52eb6c07082ec86848760405180847bffffffffffffffffffffffffffffffffffffffffffffffffffffffff19168152602001838152602001828152602001935050505060405180910390a2505050505050565b60008085851115612d3257600080fd5b83861115612d3f57600080fd5b600185028301915084860390509450949250505056fea26469706673582212205ef3d30ba0400c51cffe4d66020db6f8e0785fe148ba70591b7ba7b501e647ac64736f6c63430007060033")
        );
            // 3. Library: only MultiSendCallOnly and omit MultiSendCall
            let _tx_safe_multisend_call_only = TransactionRequest::default().with_to(safe_diamond_proxy_address).with_input(
            hex!("0000000000000000000000000000000000000000000000000000000000000000608060405234801561001057600080fd5b5061019a806100206000396000f3fe60806040526004361061001e5760003560e01c80638d80ff0a14610023575b600080fd5b6100dc6004803603602081101561003957600080fd5b810190808035906020019064010000000081111561005657600080fd5b82018360208201111561006857600080fd5b8035906020019184600183028401116401000000008311171561008a57600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f8201169050808301925050505050505091929192905050506100de565b005b805160205b8181101561015f578083015160f81c6001820184015160601c60158301850151603584018601516055850187016000856000811461012857600181146101385761013d565b6000808585888a5af1915061013d565b600080fd5b50600081141561014c57600080fd5b82605501870196505050505050506100e3565b50505056fea26469706673582212208d297bb003abee230b5dfb38774688f37a6fbb97a82a21728e8049b2acb9b73564736f6c63430007060033")
        );
            // 4. Safe singleton deploySafe();
            let _tx_safe_singleton = TransactionRequest::default().with_to(safe_diamond_proxy_address).with_input(
            hex!("0000000000000000000000000000000000000000000000000000000000000000608060405234801561001057600080fd5b506001600481905550615c1b80620000296000396000f3fe6080604052600436106101d15760003560e01c8063affed0e0116100f7578063e19a9dd911610095578063f08a032311610064578063f08a03231461156b578063f698da25146115bc578063f8dc5dd9146115e7578063ffa1ad741461166257610226565b8063e19a9dd9146112bf578063e318b52b14611310578063e75235b8146113a1578063e86637db146113cc57610226565b8063cc2f8452116100d1578063cc2f84521461100c578063d4d9bdcd146110d9578063d8d11f7814611114578063e009cfde1461124e57610226565b8063affed0e014610d89578063b4faba0914610db4578063b63e800d14610e9c57610226565b80635624b25b1161016f5780636a7612021161013e5780636a761202146109895780637d83297414610b45578063934f3a1114610bb4578063a0e67e2b14610d1d57610226565b80635624b25b146107f05780635ae6bd37146108ae578063610b5925146108fd578063694e80c31461094e57610226565b80632f54bf6e116101ab5780632f54bf6e146104c85780633408e4701461052f578063468721a71461055a5780635229073f1461066f57610226565b80630d582f131461029357806312fb68e0146102ee5780632d9ad53d1461046157610226565b36610226573373ffffffffffffffffffffffffffffffffffffffff167f3d0ce9bfc3ed7d6862dbb28b2dea94561fe714a1b4d019aa8af39730d1ad7c3d346040518082815260200191505060405180910390a2005b34801561023257600080fd5b5060007f6c9a6c4a39284e37ed1cf53d337577d14212a4870fb976a4366c693b939918d560001b905080548061026757600080f35b36600080373360601b365260008060143601600080855af13d6000803e8061028e573d6000fd5b3d6000f35b34801561029f57600080fd5b506102ec600480360360408110156102b657600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803590602001909291905050506116f2565b005b3480156102fa57600080fd5b5061045f6004803603608081101561031157600080fd5b81019080803590602001909291908035906020019064010000000081111561033857600080fd5b82018360208201111561034a57600080fd5b8035906020019184600183028401116401000000008311171561036c57600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f820116905080830192505050505050509192919290803590602001906401000000008111156103cf57600080fd5b8201836020820111156103e157600080fd5b8035906020019184600183028401116401000000008311171561040357600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f82011690508083019250505050505050919291929080359060200190929190505050611ad8565b005b34801561046d57600080fd5b506104b06004803603602081101561048457600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff1690602001909291905050506123d6565b60405180821515815260200191505060405180910390f35b3480156104d457600080fd5b50610517600480360360208110156104eb57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff1690602001909291905050506124a8565b60405180821515815260200191505060405180910390f35b34801561053b57600080fd5b5061054461257a565b6040518082815260200191505060405180910390f35b34801561056657600080fd5b506106576004803603608081101561057d57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190803590602001906401000000008111156105c457600080fd5b8201836020820111156105d657600080fd5b803590602001918460018302840111640100000000831117156105f857600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f820116905080830192505050505050509192919290803560ff169060200190929190505050612587565b60405180821515815260200191505060405180910390f35b34801561067b57600080fd5b5061076c6004803603608081101561069257600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190803590602001906401000000008111156106d957600080fd5b8201836020820111156106eb57600080fd5b8035906020019184600183028401116401000000008311171561070d57600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f820116905080830192505050505050509192919290803560ff16906020019092919050505061278d565b60405180831515815260200180602001828103825283818151815260200191508051906020019080838360005b838110156107b4578082015181840152602081019050610799565b50505050905090810190601f1680156107e15780820380516001836020036101000a031916815260200191505b50935050505060405180910390f35b3480156107fc57600080fd5b506108336004803603604081101561081357600080fd5b8101908080359060200190929190803590602001909291905050506127c3565b6040518080602001828103825283818151815260200191508051906020019080838360005b83811015610873578082015181840152602081019050610858565b50505050905090810190601f1680156108a05780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b3480156108ba57600080fd5b506108e7600480360360208110156108d157600080fd5b810190808035906020019092919050505061284a565b6040518082815260200191505060405180910390f35b34801561090957600080fd5b5061094c6004803603602081101561092057600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190505050612862565b005b34801561095a57600080fd5b506109876004803603602081101561097157600080fd5b8101908080359060200190929190505050612bea565b005b610b2d60048036036101408110156109a057600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190803590602001906401000000008111156109e757600080fd5b8201836020820111156109f957600080fd5b80359060200191846001830284011164010000000083111715610a1b57600080fd5b9091929391929390803560ff169060200190929190803590602001909291908035906020019092919080359060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190640100000000811115610aa757600080fd5b820183602082011115610ab957600080fd5b80359060200191846001830284011164010000000083111715610adb57600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f820116905080830192505050505050509192919290505050612d24565b60405180821515815260200191505060405180910390f35b348015610b5157600080fd5b50610b9e60048036036040811015610b6857600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190505050613253565b6040518082815260200191505060405180910390f35b348015610bc057600080fd5b50610d1b60048036036060811015610bd757600080fd5b810190808035906020019092919080359060200190640100000000811115610bfe57600080fd5b820183602082011115610c1057600080fd5b80359060200191846001830284011164010000000083111715610c3257600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f82011690508083019250505050505050919291929080359060200190640100000000811115610c9557600080fd5b820183602082011115610ca757600080fd5b80359060200191846001830284011164010000000083111715610cc957600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f820116905080830192505050505050509192919290505050613278565b005b348015610d2957600080fd5b50610d32613307565b6040518080602001828103825283818151815260200191508051906020019060200280838360005b83811015610d75578082015181840152602081019050610d5a565b505050509050019250505060405180910390f35b348015610d9557600080fd5b50610d9e6134b0565b6040518082815260200191505060405180910390f35b348015610dc057600080fd5b50610e9a60048036036040811015610dd757600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190640100000000811115610e1457600080fd5b820183602082011115610e2657600080fd5b80359060200191846001830284011164010000000083111715610e4857600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f8201169050808301925050505050505091929192905050506134b6565b005b348015610ea857600080fd5b5061100a6004803603610100811015610ec057600080fd5b8101908080359060200190640100000000811115610edd57600080fd5b820183602082011115610eef57600080fd5b80359060200191846020830284011164010000000083111715610f1157600080fd5b909192939192939080359060200190929190803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190640100000000811115610f5c57600080fd5b820183602082011115610f6e57600080fd5b80359060200191846001830284011164010000000083111715610f9057600080fd5b9091929391929390803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190803573ffffffffffffffffffffffffffffffffffffffff1690602001909291905050506134d8565b005b34801561101857600080fd5b506110656004803603604081101561102f57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190505050613696565b60405180806020018373ffffffffffffffffffffffffffffffffffffffff168152602001828103825284818151815260200191508051906020019060200280838360005b838110156110c45780820151818401526020810190506110a9565b50505050905001935050505060405180910390f35b3480156110e557600080fd5b50611112600480360360208110156110fc57600080fd5b81019080803590602001909291905050506139f9565b005b34801561112057600080fd5b50611238600480360361014081101561113857600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803590602001909291908035906020019064010000000081111561117f57600080fd5b82018360208201111561119157600080fd5b803590602001918460018302840111640100000000831117156111b357600080fd5b9091929391929390803560ff169060200190929190803590602001909291908035906020019092919080359060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190505050613b98565b6040518082815260200191505060405180910390f35b34801561125a57600080fd5b506112bd6004803603604081101561127157600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190505050613bc5565b005b3480156112cb57600080fd5b5061130e600480360360208110156112e257600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190505050613f4c565b005b34801561131c57600080fd5b5061139f6004803603606081101561133357600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190505050614138565b005b3480156113ad57600080fd5b506113b6614796565b6040518082815260200191505060405180910390f35b3480156113d857600080fd5b506114f060048036036101408110156113f057600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803590602001909291908035906020019064010000000081111561143757600080fd5b82018360208201111561144957600080fd5b8035906020019184600183028401116401000000008311171561146b57600080fd5b9091929391929390803560ff169060200190929190803590602001909291908035906020019092919080359060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803590602001909291905050506147a0565b6040518080602001828103825283818151815260200191508051906020019080838360005b83811015611530578082015181840152602081019050611515565b50505050905090810190601f16801561155d5780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b34801561157757600080fd5b506115ba6004803603602081101561158e57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190505050614948565b005b3480156115c857600080fd5b506115d161499f565b6040518082815260200191505060405180910390f35b3480156115f357600080fd5b506116606004803603606081101561160a57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190505050614a1d565b005b34801561166e57600080fd5b50611677614e46565b6040518080602001828103825283818151815260200191508051906020019080838360005b838110156116b757808201518184015260208101905061169c565b50505050905090810190601f1680156116e45780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b6116fa614e7f565b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff16141580156117645750600173ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614155b801561179c57503073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614155b61180e576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303300000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff16600260008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff161461190f576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303400000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60026000600173ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16600260008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508160026000600173ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055506003600081548092919060010191905055508173ffffffffffffffffffffffffffffffffffffffff167f9465fa0c962cc76958e6373a993326400c1c94f8be2fe3a952adfa7f60b2ea2660405160405180910390a28060045414611ad457611ad381612bea565b5b5050565b611aec604182614f2290919063ffffffff16565b82511015611b62576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330323000000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b6000808060008060005b868110156123ca57611b7e8882614f5c565b80945081955082965050505060008460ff1614156120035789898051906020012014611c12576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330323700000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b8260001c9450611c2c604188614f2290919063ffffffff16565b8260001c1015611ca4576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330323100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b8751611cbd60208460001c614f8b90919063ffffffff16565b1115611d31576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330323200000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60006020838a01015190508851611d6782611d5960208760001c614f8b90919063ffffffff16565b614f8b90919063ffffffff16565b1115611ddb576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330323300000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60606020848b010190506320c13b0b60e01b7bffffffffffffffffffffffffffffffffffffffffffffffffffffffff19168773ffffffffffffffffffffffffffffffffffffffff166320c13b0b8d846040518363ffffffff1660e01b8152600401808060200180602001838103835285818151815260200191508051906020019080838360005b83811015611e7d578082015181840152602081019050611e62565b50505050905090810190601f168015611eaa5780820380516001836020036101000a031916815260200191505b50838103825284818151815260200191508051906020019080838360005b83811015611ee3578082015181840152602081019050611ec8565b50505050905090810190601f168015611f105780820380516001836020036101000a031916815260200191505b5094505050505060206040518083038186803b158015611f2f57600080fd5b505afa158015611f43573d6000803e3d6000fd5b505050506040513d6020811015611f5957600080fd5b81019080805190602001909291905050507bffffffffffffffffffffffffffffffffffffffffffffffffffffffff191614611ffc576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330323400000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b5050612248565b60018460ff161415612117578260001c94508473ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff1614806120a057506000600860008773ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008c81526020019081526020016000205414155b612112576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330323500000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b612247565b601e8460ff1611156121df5760018a60405160200180807f19457468657265756d205369676e6564204d6573736167653a0a333200000000815250601c018281526020019150506040516020818303038152906040528051906020012060048603858560405160008152602001604052604051808581526020018460ff1681526020018381526020018281526020019450505050506020604051602081039080840390855afa1580156121ce573d6000803e3d6000fd5b505050602060405103519450612246565b60018a85858560405160008152602001604052604051808581526020018460ff1681526020018381526020018281526020019450505050506020604051602081039080840390855afa158015612239573d6000803e3d6000fd5b5050506020604051035194505b5b5b8573ffffffffffffffffffffffffffffffffffffffff168573ffffffffffffffffffffffffffffffffffffffff1611801561230f5750600073ffffffffffffffffffffffffffffffffffffffff16600260008773ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614155b80156123485750600173ffffffffffffffffffffffffffffffffffffffff168573ffffffffffffffffffffffffffffffffffffffff1614155b6123ba576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330323600000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b8495508080600101915050611b6c565b50505050505050505050565b60008173ffffffffffffffffffffffffffffffffffffffff16600173ffffffffffffffffffffffffffffffffffffffff16141580156124a15750600073ffffffffffffffffffffffffffffffffffffffff16600160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614155b9050919050565b6000600173ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff16141580156125735750600073ffffffffffffffffffffffffffffffffffffffff16600260008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614155b9050919050565b6000804690508091505090565b6000600173ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff16141580156126525750600073ffffffffffffffffffffffffffffffffffffffff16600160003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614155b6126c4576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475331303400000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b6126f1858585857fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff614faa565b90508015612741573373ffffffffffffffffffffffffffffffffffffffff167f6895c13664aa4f67288b25d7a21d7aaa34916e355fb9b6fae0a139a9085becb860405160405180910390a2612785565b3373ffffffffffffffffffffffffffffffffffffffff167facd2c8702804128fdb0db2bb49f6d127dd0181c13fd45dbfe16de0930e2bd37560405160405180910390a25b949350505050565b6000606061279d86868686612587565b915060405160203d0181016040523d81523d6000602083013e8091505094509492505050565b606060006020830267ffffffffffffffff811180156127e157600080fd5b506040519080825280601f01601f1916602001820160405280156128145781602001600182028036833780820191505090505b50905060005b8381101561283f5780850154806020830260208501015250808060010191505061281a565b508091505092915050565b60076020528060005260406000206000915090505481565b61286a614e7f565b600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16141580156128d45750600173ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614155b612946576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475331303100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff16600160008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614612a47576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475331303200000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60016000600173ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16600160008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508060016000600173ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508073ffffffffffffffffffffffffffffffffffffffff167fecdf3a3effea5783a3c4c2140e677577666428d44ed9d474a0b3a4c9943f844060405160405180910390a250565b612bf2614e7f565b600354811115612c6a576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b6001811015612ce1576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303200000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b806004819055507f610f7ff2b304ae8903c3de74c60c6ab1f7d6226b3f52c5161905bb5ad4039c936004546040518082815260200191505060405180910390a150565b6000806000612d3e8e8e8e8e8e8e8e8e8e8e6005546147a0565b905060056000815480929190600101919050555080805190602001209150612d67828286613278565b506000612d72614ff6565b9050600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614612f58578073ffffffffffffffffffffffffffffffffffffffff166375f0bb528f8f8f8f8f8f8f8f8f8f8f336040518d63ffffffff1660e01b8152600401808d73ffffffffffffffffffffffffffffffffffffffff1681526020018c8152602001806020018a6001811115612e1557fe5b81526020018981526020018881526020018781526020018673ffffffffffffffffffffffffffffffffffffffff1681526020018573ffffffffffffffffffffffffffffffffffffffff168152602001806020018473ffffffffffffffffffffffffffffffffffffffff16815260200183810383528d8d82818152602001925080828437600081840152601f19601f820116905080830192505050838103825285818151815260200191508051906020019080838360005b83811015612ee7578082015181840152602081019050612ecc565b50505050905090810190601f168015612f145780820380516001836020036101000a031916815260200191505b509e505050505050505050505050505050600060405180830381600087803b158015612f3f57600080fd5b505af1158015612f53573d6000803e3d6000fd5b505050505b6101f4612f7f6109c48b01603f60408d0281612f7057fe5b0461502790919063ffffffff16565b015a1015612ff5576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330313000000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60005a905061305e8f8f8f8f8080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f820116905080830192505050505050508e60008d14613053578e613059565b6109c45a035b614faa565b93506130735a8261504190919063ffffffff16565b90508380613082575060008a14155b8061308e575060008814155b613100576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330313300000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60008089111561311a57613117828b8b8b8b615061565b90505b841561315d57837f442e715f626346e8c54381002da614f62bee8d27386535b2521ec8540898556e826040518082815260200191505060405180910390a2613196565b837f23428b18acfb3ea64b08dc0c1d296ea9c09702c09083ca5272e64d115b687d23826040518082815260200191505060405180910390a25b5050600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614613242578073ffffffffffffffffffffffffffffffffffffffff16639327136883856040518363ffffffff1660e01b815260040180838152602001821515815260200192505050600060405180830381600087803b15801561322957600080fd5b505af115801561323d573d6000803e3d6000fd5b505050505b50509b9a5050505050505050505050565b6008602052816000526040600020602052806000526040600020600091509150505481565b60006004549050600081116132f5576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330303100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b61330184848484611ad8565b50505050565b6060600060035467ffffffffffffffff8111801561332457600080fd5b506040519080825280602002602001820160405280156133535781602001602082028036833780820191505090505b50905060008060026000600173ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1690505b600173ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16146134a757808383815181106133fe57fe5b602002602001019073ffffffffffffffffffffffffffffffffffffffff16908173ffffffffffffffffffffffffffffffffffffffff1681525050600260008273ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16905081806001019250506133bd565b82935050505090565b60055481565b600080825160208401855af4806000523d6020523d600060403e60403d016000fd5b6135238a8a80806020026020016040519081016040528093929190818152602001838360200280828437600081840152601f19601f8201169050808301925050505050505089615267565b600073ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff16146135615761356084615767565b5b6135af8787878080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f82011690508083019250505050505050615838565b60008211156135c9576135c782600060018685615061565b505b3373ffffffffffffffffffffffffffffffffffffffff167f141df868a6331af528e38c83b7aa03edc19be66e37ae67f9285bf4f8e3c6a1a88b8b8b8b8960405180806020018581526020018473ffffffffffffffffffffffffffffffffffffffff1681526020018373ffffffffffffffffffffffffffffffffffffffff1681526020018281038252878782818152602001925060200280828437600081840152601f19601f820116905080830192505050965050505050505060405180910390a250505050505050505050565b60606000600173ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff1614806136da57506136d9846123d6565b5b61374c576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475331303500000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600083116137c2576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475331303600000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b8267ffffffffffffffff811180156137d957600080fd5b506040519080825280602002602001820160405280156138085781602001602082028036833780820191505090505b5091506000600160008673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1691505b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff16141580156138da5750600173ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614155b80156138e557508381105b156139a057818382815181106138f757fe5b602002602001019073ffffffffffffffffffffffffffffffffffffffff16908173ffffffffffffffffffffffffffffffffffffffff1681525050600160008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1691508080600101915050613870565b600173ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff16146139ee578260018203815181106139e357fe5b602002602001015191505b808352509250929050565b600073ffffffffffffffffffffffffffffffffffffffff16600260003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff161415613afb576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330333000000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b6001600860003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000206000838152602001908152602001600020819055503373ffffffffffffffffffffffffffffffffffffffff16817ff2a0eb156472d1440255b0d7c1e19cc07115d1051fe605b0dce69acfec884d9c60405160405180910390a350565b6000613bad8c8c8c8c8c8c8c8c8c8c8c6147a0565b8051906020012090509b9a5050505050505050505050565b613bcd614e7f565b600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614158015613c375750600173ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614155b613ca9576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475331303100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b8073ffffffffffffffffffffffffffffffffffffffff16600160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614613da9576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475331303300000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600160008273ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16600160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055506000600160008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508073ffffffffffffffffffffffffffffffffffffffff167faab4fa2b463f581b2b32cb3b7e3b704b9ce37cc209b5fb4d77e593ace405427660405160405180910390a25050565b613f54614e7f565b600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16146140c6578073ffffffffffffffffffffffffffffffffffffffff166301ffc9a77fe6d7a83a000000000000000000000000000000000000000000000000000000006040518263ffffffff1660e01b815260040180827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916815260200191505060206040518083038186803b15801561401857600080fd5b505afa15801561402c573d6000803e3d6000fd5b505050506040513d602081101561404257600080fd5b81019080805190602001909291905050506140c5576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475333303000000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b5b60007f4a204f620c8c5ccdca3fd54d003badd85ba500436a431f0cbda4f558c93c34c860001b90508181558173ffffffffffffffffffffffffffffffffffffffff167f1151116914515bc0891ff9047a6cb32cf902546f83066499bcf8ba33d2353fa260405160405180910390a25050565b614140614e7f565b600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16141580156141aa5750600173ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614155b80156141e257503073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614155b614254576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303300000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff16600260008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614614355576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303400000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff16141580156143bf5750600173ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614155b614431576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303300000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b8173ffffffffffffffffffffffffffffffffffffffff16600260008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614614531576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303500000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600260008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16600260008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff16021790555080600260008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055506000600260008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508173ffffffffffffffffffffffffffffffffffffffff167ff8d49fc529812e9a7c5c50e69c20f0dccc0db8fa95c98bc58cc9a4f1c1299eaf60405160405180910390a28073ffffffffffffffffffffffffffffffffffffffff167f9465fa0c962cc76958e6373a993326400c1c94f8be2fe3a952adfa7f60b2ea2660405160405180910390a2505050565b6000600454905090565b606060007fbb8310d486368db6bd6f849402fdd73ad53d316b5a4b2644ad6efe0f941286d860001b8d8d8d8d60405180838380828437808301925050509250505060405180910390208c8c8c8c8c8c8c604051602001808c81526020018b73ffffffffffffffffffffffffffffffffffffffff1681526020018a815260200189815260200188600181111561483157fe5b81526020018781526020018681526020018581526020018473ffffffffffffffffffffffffffffffffffffffff1681526020018373ffffffffffffffffffffffffffffffffffffffff1681526020018281526020019b505050505050505050505050604051602081830303815290604052805190602001209050601960f81b600160f81b6148bd61499f565b8360405160200180857effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff19168152600101847effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff191681526001018381526020018281526020019450505050506040516020818303038152906040529150509b9a5050505050505050505050565b614950614e7f565b61495981615767565b8073ffffffffffffffffffffffffffffffffffffffff167f5ac6c46c93c8d0e53714ba3b53db3e7c046da994313d7ed0d192028bc7c228b060405160405180910390a250565b60007f47e79534a245952e8b16893a336b85a3d9ea9fa8c573f3d803afb92a7946921860001b6149cd61257a565b30604051602001808481526020018381526020018273ffffffffffffffffffffffffffffffffffffffff168152602001935050505060405160208183030381529060405280519060200120905090565b614a25614e7f565b806001600354031015614aa0576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614158015614b0a5750600173ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614155b614b7c576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303300000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b8173ffffffffffffffffffffffffffffffffffffffff16600260008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614614c7c576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303500000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600260008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16600260008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055506000600260008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff160217905550600360008154809291906001900391905055508173ffffffffffffffffffffffffffffffffffffffff167ff8d49fc529812e9a7c5c50e69c20f0dccc0db8fa95c98bc58cc9a4f1c1299eaf60405160405180910390a28060045414614e4157614e4081612bea565b5b505050565b6040518060400160405280600581526020017f312e342e3100000000000000000000000000000000000000000000000000000081525081565b3073ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff1614614f20576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330333100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b565b600080831415614f355760009050614f56565b6000828402905082848281614f4657fe5b0414614f5157600080fd5b809150505b92915050565b60008060008360410260208101860151925060408101860151915060ff60418201870151169350509250925092565b600080828401905083811015614fa057600080fd5b8091505092915050565b6000600180811115614fb857fe5b836001811115614fc457fe5b1415614fdd576000808551602087018986f49050614fed565b600080855160208701888a87f190505b95945050505050565b6000807f4a204f620c8c5ccdca3fd54d003badd85ba500436a431f0cbda4f558c93c34c860001b9050805491505090565b6000818310156150375781615039565b825b905092915050565b60008282111561505057600080fd5b600082840390508091505092915050565b600080600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff161461509e57826150a0565b325b9050600073ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff1614156151b85761510a3a86106150e7573a6150e9565b855b6150fc888a614f8b90919063ffffffff16565b614f2290919063ffffffff16565b91508073ffffffffffffffffffffffffffffffffffffffff166108fc839081150290604051600060405180830381858888f193505050506151b3576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330313100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b61525d565b6151dd856151cf888a614f8b90919063ffffffff16565b614f2290919063ffffffff16565b91506151ea848284615b0e565b61525c576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330313200000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b5b5095945050505050565b6000600454146152df576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303000000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b8151811115615356576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60018110156153cd576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303200000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60006001905060005b83518110156156d35760008482815181106153ed57fe5b60200260200101519050600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16141580156154615750600173ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614155b801561549957503073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614155b80156154d157508073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff1614155b615543576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303300000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff16600260008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614615644576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303400000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b80600260008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508092505080806001019150506153d6565b506001600260008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff160217905550825160038190555081600481905550505050565b3073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff161415615809576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475334303000000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60007f6c9a6c4a39284e37ed1cf53d337577d14212a4870fb976a4366c693b939918d560001b90508181555050565b600073ffffffffffffffffffffffffffffffffffffffff1660016000600173ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff161461593a576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475331303000000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b6001806000600173ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff160217905550600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614615b0a576159f682615bd2565b615a68576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330303200000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b615a978260008360017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff614faa565b615b09576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330303000000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b5b5050565b60008063a9059cbb8484604051602401808373ffffffffffffffffffffffffffffffffffffffff168152602001828152602001925050506040516020818303038152906040529060e01b6020820180517bffffffffffffffffffffffffffffffffffffffffffffffffffffffff83818316178352505050509050602060008251602084016000896127105a03f13d60008114615bb55760208114615bbd5760009350615bc8565b819350615bc8565b600051158215171593505b5050509392505050565b600080823b90506000811191505091905056fea264697066735822122057398fa72884cf9a6cb78aab2fb58a6b927f0e9d97d75b015daaee0959a153bf64736f6c63430007060033")
        );
            // 5. Safe L2 singleton deploySafe();
            let _tx_safe_l2_singleton = TransactionRequest::default().with_to(safe_diamond_proxy_address).with_input(
            hex!("0000000000000000000000000000000000000000000000000000000000000000608060405234801561001057600080fd5b506001600481905550615f6580620000296000396000f3fe6080604052600436106101d15760003560e01c8063affed0e0116100f7578063e19a9dd911610095578063f08a032311610064578063f08a03231461156b578063f698da25146115bc578063f8dc5dd9146115e7578063ffa1ad741461166257610226565b8063e19a9dd9146112bf578063e318b52b14611310578063e75235b8146113a1578063e86637db146113cc57610226565b8063cc2f8452116100d1578063cc2f84521461100c578063d4d9bdcd146110d9578063d8d11f7814611114578063e009cfde1461124e57610226565b8063affed0e014610d89578063b4faba0914610db4578063b63e800d14610e9c57610226565b80635624b25b1161016f5780636a7612021161013e5780636a761202146109895780637d83297414610b45578063934f3a1114610bb4578063a0e67e2b14610d1d57610226565b80635624b25b146107f05780635ae6bd37146108ae578063610b5925146108fd578063694e80c31461094e57610226565b80632f54bf6e116101ab5780632f54bf6e146104c85780633408e4701461052f578063468721a71461055a5780635229073f1461066f57610226565b80630d582f131461029357806312fb68e0146102ee5780632d9ad53d1461046157610226565b36610226573373ffffffffffffffffffffffffffffffffffffffff167f3d0ce9bfc3ed7d6862dbb28b2dea94561fe714a1b4d019aa8af39730d1ad7c3d346040518082815260200191505060405180910390a2005b34801561023257600080fd5b5060007f6c9a6c4a39284e37ed1cf53d337577d14212a4870fb976a4366c693b939918d560001b905080548061026757600080f35b36600080373360601b365260008060143601600080855af13d6000803e8061028e573d6000fd5b3d6000f35b34801561029f57600080fd5b506102ec600480360360408110156102b657600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803590602001909291905050506116f2565b005b3480156102fa57600080fd5b5061045f6004803603608081101561031157600080fd5b81019080803590602001909291908035906020019064010000000081111561033857600080fd5b82018360208201111561034a57600080fd5b8035906020019184600183028401116401000000008311171561036c57600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f820116905080830192505050505050509192919290803590602001906401000000008111156103cf57600080fd5b8201836020820111156103e157600080fd5b8035906020019184600183028401116401000000008311171561040357600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f82011690508083019250505050505050919291929080359060200190929190505050611ad8565b005b34801561046d57600080fd5b506104b06004803603602081101561048457600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff1690602001909291905050506123d6565b60405180821515815260200191505060405180910390f35b3480156104d457600080fd5b50610517600480360360208110156104eb57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff1690602001909291905050506124a8565b60405180821515815260200191505060405180910390f35b34801561053b57600080fd5b5061054461257a565b6040518082815260200191505060405180910390f35b34801561056657600080fd5b506106576004803603608081101561057d57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190803590602001906401000000008111156105c457600080fd5b8201836020820111156105d657600080fd5b803590602001918460018302840111640100000000831117156105f857600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f820116905080830192505050505050509192919290803560ff169060200190929190505050612587565b60405180821515815260200191505060405180910390f35b34801561067b57600080fd5b5061076c6004803603608081101561069257600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190803590602001906401000000008111156106d957600080fd5b8201836020820111156106eb57600080fd5b8035906020019184600183028401116401000000008311171561070d57600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f820116905080830192505050505050509192919290803560ff169060200190929190505050612692565b60405180831515815260200180602001828103825283818151815260200191508051906020019080838360005b838110156107b4578082015181840152602081019050610799565b50505050905090810190601f1680156107e15780820380516001836020036101000a031916815260200191505b50935050505060405180910390f35b3480156107fc57600080fd5b506108336004803603604081101561081357600080fd5b8101908080359060200190929190803590602001909291905050506126c8565b6040518080602001828103825283818151815260200191508051906020019080838360005b83811015610873578082015181840152602081019050610858565b50505050905090810190601f1680156108a05780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b3480156108ba57600080fd5b506108e7600480360360208110156108d157600080fd5b810190808035906020019092919050505061274f565b6040518082815260200191505060405180910390f35b34801561090957600080fd5b5061094c6004803603602081101561092057600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190505050612767565b005b34801561095a57600080fd5b506109876004803603602081101561097157600080fd5b8101908080359060200190929190505050612aef565b005b610b2d60048036036101408110156109a057600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190803590602001906401000000008111156109e757600080fd5b8201836020820111156109f957600080fd5b80359060200191846001830284011164010000000083111715610a1b57600080fd5b9091929391929390803560ff169060200190929190803590602001909291908035906020019092919080359060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190640100000000811115610aa757600080fd5b820183602082011115610ab957600080fd5b80359060200191846001830284011164010000000083111715610adb57600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f820116905080830192505050505050509192919290505050612c29565b60405180821515815260200191505060405180910390f35b348015610b5157600080fd5b50610b9e60048036036040811015610b6857600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190505050612e68565b6040518082815260200191505060405180910390f35b348015610bc057600080fd5b50610d1b60048036036060811015610bd757600080fd5b810190808035906020019092919080359060200190640100000000811115610bfe57600080fd5b820183602082011115610c1057600080fd5b80359060200191846001830284011164010000000083111715610c3257600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f82011690508083019250505050505050919291929080359060200190640100000000811115610c9557600080fd5b820183602082011115610ca757600080fd5b80359060200191846001830284011164010000000083111715610cc957600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f820116905080830192505050505050509192919290505050612e8d565b005b348015610d2957600080fd5b50610d32612f1c565b6040518080602001828103825283818151815260200191508051906020019060200280838360005b83811015610d75578082015181840152602081019050610d5a565b505050509050019250505060405180910390f35b348015610d9557600080fd5b50610d9e6130c5565b6040518082815260200191505060405180910390f35b348015610dc057600080fd5b50610e9a60048036036040811015610dd757600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190640100000000811115610e1457600080fd5b820183602082011115610e2657600080fd5b80359060200191846001830284011164010000000083111715610e4857600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f8201169050808301925050505050505091929192905050506130cb565b005b348015610ea857600080fd5b5061100a6004803603610100811015610ec057600080fd5b8101908080359060200190640100000000811115610edd57600080fd5b820183602082011115610eef57600080fd5b80359060200191846020830284011164010000000083111715610f1157600080fd5b909192939192939080359060200190929190803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190640100000000811115610f5c57600080fd5b820183602082011115610f6e57600080fd5b80359060200191846001830284011164010000000083111715610f9057600080fd5b9091929391929390803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190803573ffffffffffffffffffffffffffffffffffffffff1690602001909291905050506130ed565b005b34801561101857600080fd5b506110656004803603604081101561102f57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803590602001909291905050506132ab565b60405180806020018373ffffffffffffffffffffffffffffffffffffffff168152602001828103825284818151815260200191508051906020019060200280838360005b838110156110c45780820151818401526020810190506110a9565b50505050905001935050505060405180910390f35b3480156110e557600080fd5b50611112600480360360208110156110fc57600080fd5b810190808035906020019092919050505061360e565b005b34801561112057600080fd5b50611238600480360361014081101561113857600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803590602001909291908035906020019064010000000081111561117f57600080fd5b82018360208201111561119157600080fd5b803590602001918460018302840111640100000000831117156111b357600080fd5b9091929391929390803560ff169060200190929190803590602001909291908035906020019092919080359060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803590602001909291905050506137ad565b6040518082815260200191505060405180910390f35b34801561125a57600080fd5b506112bd6004803603604081101561127157600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff1690602001909291905050506137da565b005b3480156112cb57600080fd5b5061130e600480360360208110156112e257600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190505050613b61565b005b34801561131c57600080fd5b5061139f6004803603606081101561133357600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190505050613d4d565b005b3480156113ad57600080fd5b506113b66143ab565b6040518082815260200191505060405180910390f35b3480156113d857600080fd5b506114f060048036036101408110156113f057600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803590602001909291908035906020019064010000000081111561143757600080fd5b82018360208201111561144957600080fd5b8035906020019184600183028401116401000000008311171561146b57600080fd5b9091929391929390803560ff169060200190929190803590602001909291908035906020019092919080359060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803590602001909291905050506143b5565b6040518080602001828103825283818151815260200191508051906020019080838360005b83811015611530578082015181840152602081019050611515565b50505050905090810190601f16801561155d5780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b34801561157757600080fd5b506115ba6004803603602081101561158e57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919050505061455d565b005b3480156115c857600080fd5b506115d16145b4565b6040518082815260200191505060405180910390f35b3480156115f357600080fd5b506116606004803603606081101561160a57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190505050614632565b005b34801561166e57600080fd5b50611677614a5b565b6040518080602001828103825283818151815260200191508051906020019080838360005b838110156116b757808201518184015260208101905061169c565b50505050905090810190601f1680156116e45780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b6116fa614a94565b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff16141580156117645750600173ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614155b801561179c57503073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614155b61180e576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303300000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff16600260008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff161461190f576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303400000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60026000600173ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16600260008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508160026000600173ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055506003600081548092919060010191905055508173ffffffffffffffffffffffffffffffffffffffff167f9465fa0c962cc76958e6373a993326400c1c94f8be2fe3a952adfa7f60b2ea2660405160405180910390a28060045414611ad457611ad381612aef565b5b5050565b611aec604182614b3790919063ffffffff16565b82511015611b62576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330323000000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b6000808060008060005b868110156123ca57611b7e8882614b71565b80945081955082965050505060008460ff1614156120035789898051906020012014611c12576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330323700000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b8260001c9450611c2c604188614b3790919063ffffffff16565b8260001c1015611ca4576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330323100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b8751611cbd60208460001c614ba090919063ffffffff16565b1115611d31576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330323200000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60006020838a01015190508851611d6782611d5960208760001c614ba090919063ffffffff16565b614ba090919063ffffffff16565b1115611ddb576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330323300000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60606020848b010190506320c13b0b60e01b7bffffffffffffffffffffffffffffffffffffffffffffffffffffffff19168773ffffffffffffffffffffffffffffffffffffffff166320c13b0b8d846040518363ffffffff1660e01b8152600401808060200180602001838103835285818151815260200191508051906020019080838360005b83811015611e7d578082015181840152602081019050611e62565b50505050905090810190601f168015611eaa5780820380516001836020036101000a031916815260200191505b50838103825284818151815260200191508051906020019080838360005b83811015611ee3578082015181840152602081019050611ec8565b50505050905090810190601f168015611f105780820380516001836020036101000a031916815260200191505b5094505050505060206040518083038186803b158015611f2f57600080fd5b505afa158015611f43573d6000803e3d6000fd5b505050506040513d6020811015611f5957600080fd5b81019080805190602001909291905050507bffffffffffffffffffffffffffffffffffffffffffffffffffffffff191614611ffc576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330323400000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b5050612248565b60018460ff161415612117578260001c94508473ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff1614806120a057506000600860008773ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008c81526020019081526020016000205414155b612112576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330323500000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b612247565b601e8460ff1611156121df5760018a60405160200180807f19457468657265756d205369676e6564204d6573736167653a0a333200000000815250601c018281526020019150506040516020818303038152906040528051906020012060048603858560405160008152602001604052604051808581526020018460ff1681526020018381526020018281526020019450505050506020604051602081039080840390855afa1580156121ce573d6000803e3d6000fd5b505050602060405103519450612246565b60018a85858560405160008152602001604052604051808581526020018460ff1681526020018381526020018281526020019450505050506020604051602081039080840390855afa158015612239573d6000803e3d6000fd5b5050506020604051035194505b5b5b8573ffffffffffffffffffffffffffffffffffffffff168573ffffffffffffffffffffffffffffffffffffffff1611801561230f5750600073ffffffffffffffffffffffffffffffffffffffff16600260008773ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614155b80156123485750600173ffffffffffffffffffffffffffffffffffffffff168573ffffffffffffffffffffffffffffffffffffffff1614155b6123ba576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330323600000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b8495508080600101915050611b6c565b50505050505050505050565b60008173ffffffffffffffffffffffffffffffffffffffff16600173ffffffffffffffffffffffffffffffffffffffff16141580156124a15750600073ffffffffffffffffffffffffffffffffffffffff16600160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614155b9050919050565b6000600173ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff16141580156125735750600073ffffffffffffffffffffffffffffffffffffffff16600260008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614155b9050919050565b6000804690508091505090565b60007fb648d3644f584ed1c2232d53c46d87e693586486ad0d1175f8656013110b714e3386868686604051808673ffffffffffffffffffffffffffffffffffffffff1681526020018573ffffffffffffffffffffffffffffffffffffffff1681526020018481526020018060200183600181111561260157fe5b8152602001828103825284818151815260200191508051906020019080838360005b8381101561263e578082015181840152602081019050612623565b50505050905090810190601f16801561266b5780820380516001836020036101000a031916815260200191505b50965050505050505060405180910390a161268885858585614bbf565b9050949350505050565b600060606126a286868686612587565b915060405160203d0181016040523d81523d6000602083013e8091505094509492505050565b606060006020830267ffffffffffffffff811180156126e657600080fd5b506040519080825280601f01601f1916602001820160405280156127195781602001600182028036833780820191505090505b50905060005b838110156127445780850154806020830260208501015250808060010191505061271f565b508091505092915050565b60076020528060005260406000206000915090505481565b61276f614a94565b600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16141580156127d95750600173ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614155b61284b576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475331303100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff16600160008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff161461294c576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475331303200000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60016000600173ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16600160008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508060016000600173ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508073ffffffffffffffffffffffffffffffffffffffff167fecdf3a3effea5783a3c4c2140e677577666428d44ed9d474a0b3a4c9943f844060405160405180910390a250565b612af7614a94565b600354811115612b6f576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b6001811015612be6576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303200000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b806004819055507f610f7ff2b304ae8903c3de74c60c6ab1f7d6226b3f52c5161905bb5ad4039c936004546040518082815260200191505060405180910390a150565b6000606060055433600454604051602001808481526020018373ffffffffffffffffffffffffffffffffffffffff168152602001828152602001935050505060405160208183030381529060405290507f66753cd2356569ee081232e3be8909b950e0a76c1f8460c3a5e3c2be32b11bed8d8d8d8d8d8d8d8d8d8d8d8c604051808d73ffffffffffffffffffffffffffffffffffffffff1681526020018c8152602001806020018a6001811115612cdc57fe5b81526020018981526020018881526020018781526020018673ffffffffffffffffffffffffffffffffffffffff1681526020018573ffffffffffffffffffffffffffffffffffffffff168152602001806020018060200184810384528e8e82818152602001925080828437600081840152601f19601f820116905080830192505050848103835286818151815260200191508051906020019080838360005b83811015612d96578082015181840152602081019050612d7b565b50505050905090810190601f168015612dc35780820380516001836020036101000a031916815260200191505b50848103825285818151815260200191508051906020019080838360005b83811015612dfc578082015181840152602081019050612de1565b50505050905090810190601f168015612e295780820380516001836020036101000a031916815260200191505b509f5050505050505050505050505050505060405180910390a1612e568d8d8d8d8d8d8d8d8d8d8d614dc5565b9150509b9a5050505050505050505050565b6008602052816000526040600020602052806000526040600020600091509150505481565b6000600454905060008111612f0a576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330303100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b612f1684848484611ad8565b50505050565b6060600060035467ffffffffffffffff81118015612f3957600080fd5b50604051908082528060200260200182016040528015612f685781602001602082028036833780820191505090505b50905060008060026000600173ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1690505b600173ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16146130bc578083838151811061301357fe5b602002602001019073ffffffffffffffffffffffffffffffffffffffff16908173ffffffffffffffffffffffffffffffffffffffff1681525050600260008273ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1690508180600101925050612fd2565b82935050505090565b60055481565b600080825160208401855af4806000523d6020523d600060403e60403d016000fd5b6131388a8a80806020026020016040519081016040528093929190818152602001838360200280828437600081840152601f19601f82011690508083019250505050505050896152f4565b600073ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff161461317657613175846157f4565b5b6131c48787878080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f820116905080830192505050505050506158c5565b60008211156131de576131dc82600060018685615b9b565b505b3373ffffffffffffffffffffffffffffffffffffffff167f141df868a6331af528e38c83b7aa03edc19be66e37ae67f9285bf4f8e3c6a1a88b8b8b8b8960405180806020018581526020018473ffffffffffffffffffffffffffffffffffffffff1681526020018373ffffffffffffffffffffffffffffffffffffffff1681526020018281038252878782818152602001925060200280828437600081840152601f19601f820116905080830192505050965050505050505060405180910390a250505050505050505050565b60606000600173ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff1614806132ef57506132ee846123d6565b5b613361576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475331303500000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600083116133d7576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475331303600000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b8267ffffffffffffffff811180156133ee57600080fd5b5060405190808252806020026020018201604052801561341d5781602001602082028036833780820191505090505b5091506000600160008673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1691505b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff16141580156134ef5750600173ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614155b80156134fa57508381105b156135b5578183828151811061350c57fe5b602002602001019073ffffffffffffffffffffffffffffffffffffffff16908173ffffffffffffffffffffffffffffffffffffffff1681525050600160008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1691508080600101915050613485565b600173ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614613603578260018203815181106135f857fe5b602002602001015191505b808352509250929050565b600073ffffffffffffffffffffffffffffffffffffffff16600260003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff161415613710576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330333000000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b6001600860003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000206000838152602001908152602001600020819055503373ffffffffffffffffffffffffffffffffffffffff16817ff2a0eb156472d1440255b0d7c1e19cc07115d1051fe605b0dce69acfec884d9c60405160405180910390a350565b60006137c28c8c8c8c8c8c8c8c8c8c8c6143b5565b8051906020012090509b9a5050505050505050505050565b6137e2614a94565b600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff161415801561384c5750600173ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614155b6138be576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475331303100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b8073ffffffffffffffffffffffffffffffffffffffff16600160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16146139be576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475331303300000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600160008273ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16600160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055506000600160008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508073ffffffffffffffffffffffffffffffffffffffff167faab4fa2b463f581b2b32cb3b7e3b704b9ce37cc209b5fb4d77e593ace405427660405160405180910390a25050565b613b69614a94565b600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614613cdb578073ffffffffffffffffffffffffffffffffffffffff166301ffc9a77fe6d7a83a000000000000000000000000000000000000000000000000000000006040518263ffffffff1660e01b815260040180827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916815260200191505060206040518083038186803b158015613c2d57600080fd5b505afa158015613c41573d6000803e3d6000fd5b505050506040513d6020811015613c5757600080fd5b8101908080519060200190929190505050613cda576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475333303000000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b5b60007f4a204f620c8c5ccdca3fd54d003badd85ba500436a431f0cbda4f558c93c34c860001b90508181558173ffffffffffffffffffffffffffffffffffffffff167f1151116914515bc0891ff9047a6cb32cf902546f83066499bcf8ba33d2353fa260405160405180910390a25050565b613d55614a94565b600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614158015613dbf5750600173ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614155b8015613df757503073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614155b613e69576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303300000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff16600260008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614613f6a576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303400000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614158015613fd45750600173ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614155b614046576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303300000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b8173ffffffffffffffffffffffffffffffffffffffff16600260008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614614146576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303500000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600260008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16600260008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff16021790555080600260008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055506000600260008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508173ffffffffffffffffffffffffffffffffffffffff167ff8d49fc529812e9a7c5c50e69c20f0dccc0db8fa95c98bc58cc9a4f1c1299eaf60405160405180910390a28073ffffffffffffffffffffffffffffffffffffffff167f9465fa0c962cc76958e6373a993326400c1c94f8be2fe3a952adfa7f60b2ea2660405160405180910390a2505050565b6000600454905090565b606060007fbb8310d486368db6bd6f849402fdd73ad53d316b5a4b2644ad6efe0f941286d860001b8d8d8d8d60405180838380828437808301925050509250505060405180910390208c8c8c8c8c8c8c604051602001808c81526020018b73ffffffffffffffffffffffffffffffffffffffff1681526020018a815260200189815260200188600181111561444657fe5b81526020018781526020018681526020018581526020018473ffffffffffffffffffffffffffffffffffffffff1681526020018373ffffffffffffffffffffffffffffffffffffffff1681526020018281526020019b505050505050505050505050604051602081830303815290604052805190602001209050601960f81b600160f81b6144d26145b4565b8360405160200180857effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff19168152600101847effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff191681526001018381526020018281526020019450505050506040516020818303038152906040529150509b9a5050505050505050505050565b614565614a94565b61456e816157f4565b8073ffffffffffffffffffffffffffffffffffffffff167f5ac6c46c93c8d0e53714ba3b53db3e7c046da994313d7ed0d192028bc7c228b060405160405180910390a250565b60007f47e79534a245952e8b16893a336b85a3d9ea9fa8c573f3d803afb92a7946921860001b6145e261257a565b30604051602001808481526020018381526020018273ffffffffffffffffffffffffffffffffffffffff168152602001935050505060405160208183030381529060405280519060200120905090565b61463a614a94565b8060016003540310156146b5576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff161415801561471f5750600173ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614155b614791576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303300000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b8173ffffffffffffffffffffffffffffffffffffffff16600260008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614614891576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303500000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600260008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16600260008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055506000600260008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff160217905550600360008154809291906001900391905055508173ffffffffffffffffffffffffffffffffffffffff167ff8d49fc529812e9a7c5c50e69c20f0dccc0db8fa95c98bc58cc9a4f1c1299eaf60405160405180910390a28060045414614a5657614a5581612aef565b5b505050565b6040518060400160405280600581526020017f312e342e3100000000000000000000000000000000000000000000000000000081525081565b3073ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff1614614b35576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330333100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b565b600080831415614b4a5760009050614b6b565b6000828402905082848281614b5b57fe5b0414614b6657600080fd5b809150505b92915050565b60008060008360410260208101860151925060408101860151915060ff60418201870151169350509250925092565b600080828401905083811015614bb557600080fd5b8091505092915050565b6000600173ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff1614158015614c8a5750600073ffffffffffffffffffffffffffffffffffffffff16600160003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614155b614cfc576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475331303400000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b614d29858585857fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff615da1565b90508015614d79573373ffffffffffffffffffffffffffffffffffffffff167f6895c13664aa4f67288b25d7a21d7aaa34916e355fb9b6fae0a139a9085becb860405160405180910390a2614dbd565b3373ffffffffffffffffffffffffffffffffffffffff167facd2c8702804128fdb0db2bb49f6d127dd0181c13fd45dbfe16de0930e2bd37560405160405180910390a25b949350505050565b6000806000614ddf8e8e8e8e8e8e8e8e8e8e6005546143b5565b905060056000815480929190600101919050555080805190602001209150614e08828286612e8d565b506000614e13615ded565b9050600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614614ff9578073ffffffffffffffffffffffffffffffffffffffff166375f0bb528f8f8f8f8f8f8f8f8f8f8f336040518d63ffffffff1660e01b8152600401808d73ffffffffffffffffffffffffffffffffffffffff1681526020018c8152602001806020018a6001811115614eb657fe5b81526020018981526020018881526020018781526020018673ffffffffffffffffffffffffffffffffffffffff1681526020018573ffffffffffffffffffffffffffffffffffffffff168152602001806020018473ffffffffffffffffffffffffffffffffffffffff16815260200183810383528d8d82818152602001925080828437600081840152601f19601f820116905080830192505050838103825285818151815260200191508051906020019080838360005b83811015614f88578082015181840152602081019050614f6d565b50505050905090810190601f168015614fb55780820380516001836020036101000a031916815260200191505b509e505050505050505050505050505050600060405180830381600087803b158015614fe057600080fd5b505af1158015614ff4573d6000803e3d6000fd5b505050505b6101f46150206109c48b01603f60408d028161501157fe5b04615e1e90919063ffffffff16565b015a1015615096576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330313000000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60005a90506150ff8f8f8f8f8080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f820116905080830192505050505050508e60008d146150f4578e6150fa565b6109c45a035b615da1565b93506151145a82615e3890919063ffffffff16565b90508380615123575060008a14155b8061512f575060008814155b6151a1576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330313300000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b6000808911156151bb576151b8828b8b8b8b615b9b565b90505b84156151fe57837f442e715f626346e8c54381002da614f62bee8d27386535b2521ec8540898556e826040518082815260200191505060405180910390a2615237565b837f23428b18acfb3ea64b08dc0c1d296ea9c09702c09083ca5272e64d115b687d23826040518082815260200191505060405180910390a25b5050600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16146152e3578073ffffffffffffffffffffffffffffffffffffffff16639327136883856040518363ffffffff1660e01b815260040180838152602001821515815260200192505050600060405180830381600087803b1580156152ca57600080fd5b505af11580156152de573d6000803e3d6000fd5b505050505b50509b9a5050505050505050505050565b60006004541461536c576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303000000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b81518111156153e3576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600181101561545a576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303200000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60006001905060005b835181101561576057600084828151811061547a57fe5b60200260200101519050600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16141580156154ee5750600173ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614155b801561552657503073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614155b801561555e57508073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff1614155b6155d0576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303300000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff16600260008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16146156d1576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303400000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b80600260008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff160217905550809250508080600101915050615463565b506001600260008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff160217905550825160038190555081600481905550505050565b3073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff161415615896576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475334303000000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60007f6c9a6c4a39284e37ed1cf53d337577d14212a4870fb976a4366c693b939918d560001b90508181555050565b600073ffffffffffffffffffffffffffffffffffffffff1660016000600173ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16146159c7576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475331303000000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b6001806000600173ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff160217905550600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614615b9757615a8382615e58565b615af5576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330303200000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b615b248260008360017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff615da1565b615b96576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330303000000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b5b5050565b600080600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff1614615bd85782615bda565b325b9050600073ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff161415615cf257615c443a8610615c21573a615c23565b855b615c36888a614ba090919063ffffffff16565b614b3790919063ffffffff16565b91508073ffffffffffffffffffffffffffffffffffffffff166108fc839081150290604051600060405180830381858888f19350505050615ced576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330313100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b615d97565b615d1785615d09888a614ba090919063ffffffff16565b614b3790919063ffffffff16565b9150615d24848284615e6b565b615d96576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330313200000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b5b5095945050505050565b6000600180811115615daf57fe5b836001811115615dbb57fe5b1415615dd4576000808551602087018986f49050615de4565b600080855160208701888a87f190505b95945050505050565b6000807f4a204f620c8c5ccdca3fd54d003badd85ba500436a431f0cbda4f558c93c34c860001b9050805491505090565b600081831015615e2e5781615e30565b825b905092915050565b600082821115615e4757600080fd5b600082840390508091505092915050565b600080823b905060008111915050919050565b60008063a9059cbb8484604051602401808373ffffffffffffffffffffffffffffffffffffffff168152602001828152602001925050506040516020818303038152906040529060e01b6020820180517bffffffffffffffffffffffffffffffffffffffffffffffffffffffff83818316178352505050509050602060008251602084016000896127105a03f13d60008114615f125760208114615f1a5760009350615f25565b819350615f25565b600051158215171593505b505050939250505056fea2646970667358221220cd2bdb262f44c0636136d3c6bfed2c2458921f82c3bf476053bd2e9ac618b2da64736f6c63430007060033")
        );
            // 6. Safe multisend:
            let _tx_safe_multisend = TransactionRequest::default().with_to(safe_diamond_proxy_address).with_input(
                hex!("000000000000000000000000000000000000000000000000000000000000000060a060405234801561001057600080fd5b503073ffffffffffffffffffffffffffffffffffffffff1660808173ffffffffffffffffffffffffffffffffffffffff1660601b8152505060805160601c6102756100646000398060e052506102756000f3fe60806040526004361061001e5760003560e01c80638d80ff0a14610023575b600080fd5b6100dc6004803603602081101561003957600080fd5b810190808035906020019064010000000081111561005657600080fd5b82018360208201111561006857600080fd5b8035906020019184600183028401116401000000008311171561008a57600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f8201169050808301925050505050505091929192905050506100de565b005b7f000000000000000000000000000000000000000000000000000000000000000073ffffffffffffffffffffffffffffffffffffffff163073ffffffffffffffffffffffffffffffffffffffff161415610183576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260308152602001806102106030913960400191505060405180910390fd5b805160205b8181101561020a578083015160f81c6001820184015160601c6015830185015160358401860151605585018701600085600081146101cd57600181146101dd576101e8565b6000808585888a5af191506101e8565b6000808585895af491505b5060008114156101f757600080fd5b8260550187019650505050505050610188565b50505056fe4d756c746953656e642073686f756c64206f6e6c792062652063616c6c6564207669612064656c656761746563616c6ca264697066735822122021102e6d5bc1da75411b41fe2792a1748bf5c49c794e51e81405ccd2399da13564736f6c63430007060033")
            );
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
            provider.send_transaction(_tx_safe_l2_singleton).await?.watch().await?;
            provider.send_transaction(_tx_safe_multisend).await?.watch().await?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_native_and_token_balances_in_anvil_with_multicall() -> anyhow::Result<()> {
        // create a keypair
        let kp = ChainKeypair::random();
        let kp_address = Address::from(&kp.public().to_address().into());

        // launch local anvil instance
        let anvil = create_anvil(None);
        let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let client = create_rpc_client_to_anvil(&anvil, &contract_deployer);
        // deploy hopr contracts
        let instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
            .await
            .expect("failed to deploy");
        // deploy multicall contract
        deploy_multicall3_for_testing(client.clone()).await?;

        // get native and token balances
        let (native_balance, token_balance) = get_native_and_token_balances(instances.token, vec![kp_address]).await?;
        assert_eq!(native_balance.len(), 1, "invalid native balance lens");
        assert_eq!(token_balance.len(), 1, "invalid token balance lens");
        assert_eq!(native_balance[0].to::<u64>(), 0u64, "wrong native balance");
        assert_eq!(token_balance[0].to::<u64>(), 0u64, "wrong token balance");
        drop(anvil);
        Ok(())
    }

    #[tokio::test]
    async fn test_transfer_or_mint_tokens_in_anvil_with_multicall() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let mut addresses: Vec<Address> = Vec::new();
        for _ in 0..4 {
            addresses.push(get_random_address_for_testing());
        }
        let desired_amount = vec![U256::from(1), U256::from(2), U256::from(3), U256::from(4)];

        // launch local anvil instance
        let anvil = create_anvil(None);
        let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let client = create_rpc_client_to_anvil(&anvil, &contract_deployer);
        // deploy hopr contracts
        let instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
            .await
            .expect("failed to deploy");

        // deploy multicall contract
        deploy_multicall3_for_testing(client.clone()).await?;
        // grant deployer token minter role
        let encoded_minter_role = keccak256(b"MINTER_ROLE");
        instances
            .token
            .grantRole(encoded_minter_role, a2h(contract_deployer.public().to_address()))
            .send()
            .await?
            .watch()
            .await?;

        // test the deployer has minter role now
        let check_minter_role = instances
            .token
            .hasRole(encoded_minter_role, a2h(contract_deployer.public().to_address()))
            .call()
            .await?;
        assert!(check_minter_role, "deployer does not have minter role yet");

        // transfer or mint tokens to addresses
        let total_transferred_amount =
            transfer_or_mint_tokens(instances.token.clone(), addresses.clone(), desired_amount.clone()).await?;

        // get native and token balances
        let (native_balance, token_balance) = get_native_and_token_balances(instances.token, addresses.clone()).await?;

        assert_eq!(native_balance.len(), 4, "invalid native balance lens");
        assert_eq!(token_balance.len(), 4, "invalid token balance lens");
        for (i, amount) in desired_amount.iter().enumerate() {
            assert_eq!(&token_balance[i], amount, "token balance unmatch");
        }

        assert_eq!(
            total_transferred_amount,
            U256::from(10),
            "amount transferred does not equal to the desired amount"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_transfer_or_mint_tokens_in_anvil_with_one_recipient() -> anyhow::Result<()> {
        let addresses: Vec<Address> = vec![get_random_address_for_testing()];
        let desired_amount = vec![U256::from(42)];

        // launch local anvil instance
        let anvil = create_anvil(None);
        let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let client = create_rpc_client_to_anvil(&anvil, &contract_deployer);
        // deploy hopr contracts
        let instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
            .await
            .expect("failed to deploy");

        // deploy multicall contract
        deploy_multicall3_for_testing(client.clone()).await?;
        // grant deployer token minter role
        let encoded_minter_role = keccak256(b"MINTER_ROLE");
        instances
            .token
            .grantRole(encoded_minter_role, a2h(contract_deployer.public().to_address()))
            .send()
            .await?
            .watch()
            .await?;

        // test the deployer has minter role now
        let check_minter_role = instances
            .token
            .hasRole(encoded_minter_role, a2h(contract_deployer.public().to_address()))
            .call()
            .await?;
        assert!(check_minter_role, "deployer does not have minter role yet");

        // transfer or mint tokens to addresses
        let total_transferred_amount =
            transfer_or_mint_tokens(instances.token.clone(), addresses.clone(), desired_amount.clone()).await?;

        // get native and token balances
        let (native_balance, token_balance) = get_native_and_token_balances(instances.token, addresses.clone()).await?;
        assert_eq!(native_balance.len(), 1, "invalid native balance lens");
        assert_eq!(token_balance.len(), 1, "invalid token balance lens");
        for (i, amount) in desired_amount.iter().enumerate() {
            assert_eq!(&token_balance[i], amount, "token balance unmatch");
        }

        assert_eq!(
            total_transferred_amount, desired_amount[0],
            "amount transferred does not equal to the desired amount"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_transfer_or_mint_tokens_in_anvil_without_recipient() -> anyhow::Result<()> {
        let addresses: Vec<Address> = Vec::new();
        let desired_amount: Vec<U256> = Vec::new();

        // launch local anvil instance
        let anvil = create_anvil(None);
        let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let client = create_rpc_client_to_anvil(&anvil, &contract_deployer);
        // deploy hopr contracts
        let instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
            .await
            .expect("failed to deploy");

        // deploy multicall contract
        deploy_multicall3_for_testing(client.clone()).await?;

        // transfer or mint tokens to addresses
        let total_transferred_amount =
            transfer_or_mint_tokens(instances.token.clone(), addresses.clone(), desired_amount.clone()).await?;

        // get native and token balances
        let (native_balance, token_balance) = get_native_and_token_balances(instances.token, addresses.clone()).await?;
        assert_eq!(native_balance.len(), 0, "invalid native balance lens");
        assert_eq!(token_balance.len(), 0, "invalid token balance lens");
        // for (i, amount) in desired_amount.iter().enumerate() {
        //     assert_eq!(token_balance[i].as_u64(), amount.as_u64(), "token balance unmatch");
        // }

        assert_eq!(
            total_transferred_amount,
            U256::from(0),
            "amount transferred does not equal to the desired amount"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_transfer_native_tokens_in_anvil_with_multicall() -> anyhow::Result<()> {
        let mut addresses: Vec<Address> = Vec::new();
        for _ in 0..4 {
            addresses.push(get_random_address_for_testing());
        }
        let desired_amount = vec![U256::from(1), U256::from(2), U256::from(3), U256::from(4)];

        // launch local anvil instance
        let anvil = create_anvil(None);
        let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let client = create_rpc_client_to_anvil(&anvil, &contract_deployer);
        let instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
            .await
            .expect("failed to deploy");

        // deploy multicall contract
        deploy_multicall3_for_testing(client.clone()).await?;

        // transfer native tokens to addresses
        let total_transferred_amount =
            transfer_native_tokens(client.clone(), addresses.clone(), desired_amount.clone()).await?;

        // get native and token balances
        let (native_balance, token_balance) = get_native_and_token_balances(instances.token, addresses.clone()).await?;
        assert_eq!(native_balance.len(), 4, "invalid native balance lens");
        assert_eq!(token_balance.len(), 4, "invalid token balance lens");
        for (i, amount) in desired_amount.iter().enumerate() {
            assert_eq!(&native_balance[i], amount, "native balance unmatch");
        }

        assert_eq!(
            total_transferred_amount,
            U256::from(10),
            "amount transferred does not equal to the desired amount"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_deploy_proxy() -> anyhow::Result<()> {
        let prediction = deploy_proxy(
            address!("41675c099f32341bf84bfc5382af534df5c7461a"),
            hex!("09e458584ce79e57b65cb303dc136c5d53e17b676599b9b7bc03815e0eef5172"),
            SAFE_SAFEPROXYFACTORY_ADDRESS,
        )?;

        assert_eq!(
            prediction,
            address!("ec5c8d045dfa1f93785125c26e187e9439f67105"),
            "cannot reproduce proxy"
        );
        Ok(())
    }
    #[tokio::test]
    async fn test_get_salt_from_salt_nonce() -> anyhow::Result<()> {
        let salt = get_salt_from_salt_nonce(
            hex!("b63e800d00000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001400000000000000000000000002a15de4410d4c8af0a7b6c12803120f43c42b8200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000098b275485c406573d042848d66eb9d63fca311c00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000").into(),
            B256::from_str("E5EAFDE6416CCB48925026B6313D62A98C0997E03591E29EB4CF1EA968D6BC8F")?// &U256::from_str("103994836888229670573364883831672511342967953907147914065931589108526220754063")?.into(),
        )?;

        assert_eq!(
            salt.to_vec(),
            Bytes::from_str("09e458584ce79e57b65cb303dc136c5d53e17b676599b9b7bc03815e0eef5172")?,
            "cannot reproduce salt"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_safe_and_module_address_prediction() -> anyhow::Result<()> {
        init_tracing();
        // testing value extracted from https://dashboard.tenderly.co/tx/xdai/0x510e3ac3dc7939cae2525a0b0f096ad709b23d94169e0fbf2e1154fdd6911c49?trace=0
        let _ = env_logger::builder().is_test(true).try_init();

        // prepare some input data
        let mut admin_addresses: Vec<Address> = Vec::new();
        for _ in 0..2 {
            admin_addresses.push(get_random_address_for_testing());
        }

        // launch local anvil instance
        let anvil = create_anvil(None);
        let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let client = create_rpc_client_to_anvil(&anvil, &contract_deployer);
        let instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
            .await
            .expect("failed to deploy");
        // deploy multicall contract
        deploy_multicall3_for_testing(client.clone()).await?;
        // deploy safe suits
        deploy_safe_suites(client.clone()).await?;

        let caller = client.default_signer_address();

        // build the default permissions of capabilities
        let default_target = format!("{:?}{}", instances.channels.address(), DEFAULT_CAPABILITY_PERMISSIONS);
        debug!("default target {:?}", default_target);
        // salt nonce
        let curr_nonce = client.get_transaction_count(caller).pending().await?;
        let nonce = keccak256((caller, U256::from(curr_nonce)).abi_encode_packed());

        let safe_address = predict_safe_address(
            *instances.stake_factory.address(),
            vec![caller],
            nonce,
            SAFE_COMPATIBILITYFALLBACKHANDLER_ADDRESS,
            SAFE_SAFE_L2_ADDRESS,
            SAFE_SAFEPROXYFACTORY_ADDRESS,
        )?;

        debug!("predict_safe_address {:?}", safe_address);

        let safe_address_predicted_from_sc = instances
            .stake_factory
            .predictSafeAddress(vec![caller], nonce.into())
            .call()
            .await?;
        debug!(
            "predicted safe address from smart contract {:?}",
            safe_address_predicted_from_sc.to_string()
        );

        assert_eq!(
            safe_address, safe_address_predicted_from_sc,
            "safe address prediction local vs smart contract does not match"
        );

        let module_address_predicted_from_sc = instances
            .stake_factory
            .predictModuleAddress_1(
                caller,
                nonce.into(),
                safe_address,
                U256::from_str(default_target.as_str())?.into(),
            )
            .call()
            .await?;
        info!(
            "predicted module address from smart contract {:?}",
            module_address_predicted_from_sc.to_string()
        );

        // deploy a safe proxy instance and a module proxy instance with multicall as an owner
        let deployment_receipt = instances
            .stake_factory
            .clone(
                //*instances.module_implementation.address(),
                nonce.into(),
                U256::from_str(&default_target)?.into(),
                vec![caller],
            )
            .send()
            .await?
            .get_receipt()
            .await?;

        // parse the safe and module addresses
        let module_log = deployment_receipt
            .decoded_log::<HoprNodeStakeFactory::NewHoprNodeStakeModule>()
            .ok_or_else(|| anyhow::anyhow!("Module log not found"))?;

        let safe_log = deployment_receipt
            .decoded_log::<HoprNodeStakeFactory::NewHoprNodeStakeSafe>()
            .ok_or_else(|| anyhow::anyhow!("Safe log not found"))?;

        let module_addr = module_log.instance;
        let safe_addr = safe_log.instance;

        info!("deployed module address {:?}", module_addr);

        assert_eq!(safe_addr, safe_address, "safe prediction does not match");
        assert_eq!(
            module_addr, module_address_predicted_from_sc,
            "module prediction does not match"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_deploy_safe_and_module() -> anyhow::Result<()> {
        init_tracing();
        let _ = env_logger::builder().is_test(true).try_init();

        // prepare some input data
        let mut admin_addresses: Vec<Address> = Vec::new();
        for _ in 0..2 {
            admin_addresses.push(get_random_address_for_testing());
        }
        let mut node_addresses: Vec<Address> = Vec::new();
        for _ in 0..2 {
            node_addresses.push(get_random_address_for_testing());
        }

        // launch local anvil instance
        let anvil = create_anvil(None);
        let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let client = create_rpc_client_to_anvil(&anvil, &contract_deployer);
        let instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
            .await
            .expect("failed to deploy");
        // deploy multicall contract
        deploy_multicall3_for_testing(client.clone()).await?;
        // deploy safe suits
        deploy_safe_suites(client.clone()).await?;

        // register some nodes
        let (safe, node_module) = deploy_safe_module_with_targets_and_nodes(
            instances.stake_factory,
            *instances.channels.address(),
            Some(node_addresses.clone()),
            admin_addresses.clone(),
            U256::from(2),
        )
        .await?;

        // check announcement is a target
        let try_get_announcement_target = node_module
            .tryGetTarget(*instances.announcements.address())
            .call()
            .await?;

        assert!(try_get_announcement_target._0, "announcement is not a target");

        // check allowance for channel contract has increased
        let allowance = instances
            .token
            .allowance(*safe.address(), *instances.channels.address())
            .call()
            .await?;

        assert_eq!(
            allowance,
            U256::from(1_000_000_000_000_000_000_000_u128),
            "allowance is not set"
        );

        // check nodes have been included in the module
        for node_address in node_addresses {
            let is_node_included = node_module.isNode(node_address).call().await?;
            assert!(is_node_included, "failed to include a node");
        }

        // check owners are provided admins
        let owners = safe.getOwners().call().await?;
        let thresold = safe.getThreshold().call().await?;

        assert_eq!(owners.len(), 2, "should have 2 owners");
        for (i, owner) in owners.iter().enumerate() {
            assert_eq!(owner, &admin_addresses[i], "admin is wrong");
        }
        assert_eq!(thresold, U256::from(2), "threshold should be two");
        Ok(())
    }

    #[tokio::test]
    async fn test_safe_tx_via_multisend() -> anyhow::Result<()> {
        // set allowance for token transfer for the safe multiple times
        let _ = env_logger::builder().is_test(true).try_init();

        // prepare some input data
        let desired_amount = vec![U256::from(1), U256::from(2), U256::from(3), U256::from(4)];

        // launch local anvil instance
        let anvil = create_anvil(None);
        let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let client = create_rpc_client_to_anvil(&anvil, &contract_deployer);
        let instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
            .await
            .expect("failed to deploy");
        // deploy multicall contract
        deploy_multicall3_for_testing(client.clone()).await?;
        // deploy safe suits
        deploy_safe_suites(client.clone()).await?;

        // create a safe
        let (safe, _node_module) = deploy_safe_module_with_targets_and_nodes(
            instances.stake_factory,
            *instances.channels.address(),
            None,
            vec![a2h(contract_deployer.public().to_address())],
            U256::from(1),
        )
        .await?;

        // check owner of safe
        let is_owner = safe.getOwners().call().await?;
        assert_eq!(is_owner.len(), 1, "safe has too many owners");
        assert_eq!(
            is_owner[0].0.0,
            contract_deployer.public().to_address().as_ref(),
            "safe wrong owner"
        );

        // check allowance for channel contract is zero
        let allowance = instances
            .token
            .allowance(*safe.address(), *instances.channels.address())
            .call()
            .await?;

        assert_eq!(
            allowance,
            U256::from(1_000_000_000_000_000_000_000_u128),
            "allowance is not set"
        );

        let mut multisend_txns: Vec<MultisendTransaction> = Vec::new();
        for val in desired_amount {
            multisend_txns.push(MultisendTransaction {
                // build multisend tx payload
                encoded_data: approveCall {
                    spender: *instances.channels.address(),
                    value: val,
                }
                .abi_encode()
                .into(),
                tx_operation: SafeTxOperation::Call,
                to: *instances.token.address(),
                value: U256::ZERO,
            });
        }

        // get chain_id and safe_nonce
        let chain_id = client.get_chain_id().await?;
        let safe_nonce = safe.nonce().call().await?;
        debug!("safe address {:?}", safe.address());
        debug!("chain_id {:?}", chain_id);
        debug!("safe_nonce {:?}", safe_nonce);

        // send safe transaction
        send_multisend_safe_transaction_with_threshold_one(
            safe.clone(),
            contract_deployer,
            SAFE_MULTISEND_ADDRESS,
            multisend_txns,
            U256::from(chain_id),
            safe_nonce,
        )
        .await?;

        // check allowance for channel contract is 4
        let new_allowance = instances
            .token
            .allowance(*safe.address(), *instances.channels.address())
            .call()
            .await?;

        assert_eq!(new_allowance, U256::from(4), "final allowance is not desired");
        Ok(())
    }

    #[tokio::test]
    async fn test_register_nodes_to_node_safe_registry() -> anyhow::Result<()> {
        // set allowance for token transfer for the safe multiple times
        let _ = env_logger::builder().is_test(true).try_init();

        // launch local anvil instance
        let anvil = create_anvil(None);
        let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let client = create_rpc_client_to_anvil(&anvil, &contract_deployer);
        let instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
            .await
            .expect("failed to deploy");
        // deploy multicall contract
        deploy_multicall3_for_testing(client.clone()).await?;
        // deploy safe suits
        deploy_safe_suites(client.clone()).await?;

        let deployer_vec: Vec<Address> = vec![a2h(contract_deployer.public().to_address())];

        // create a safe
        let (safe, node_module) = deploy_safe_module_with_targets_and_nodes(
            instances.stake_factory,
            *instances.channels.address(),
            Some(deployer_vec.clone()),
            deployer_vec.clone(),
            U256::from(1),
        )
        .await?;

        // register one node to safe
        instances
            .safe_registry
            .registerSafeByNode(*safe.address())
            .send()
            .await?
            .watch()
            .await?;

        // get registration info
        let get_registered_safe =
            get_registered_safes_for_nodes_on_node_safe_registry(instances.safe_registry.clone(), deployer_vec.clone())
                .await?;

        assert_eq!(get_registered_safe.len(), 1, "cannot read registered safe");
        assert_eq!(&get_registered_safe[0], safe.address(), "registered safe is wrong");

        // deregister the node from safe
        deregister_nodes_from_node_safe_registry_and_remove_from_module(
            instances.safe_registry.clone(),
            deployer_vec.clone(),
            vec![*node_module.address()],
            contract_deployer.clone(),
        )
        .await?;

        // get registration info (updated)
        let get_registered_safe =
            get_registered_safes_for_nodes_on_node_safe_registry(instances.safe_registry.clone(), deployer_vec.clone())
                .await?;

        assert_eq!(get_registered_safe.len(), 1, "cannot read registered safe");
        assert_eq!(get_registered_safe[0], Address::ZERO, "node is still registered");

        // node is removed
        let is_removed = node_module
            .isNode(a2h(contract_deployer.public().to_address()))
            .call()
            .await?;
        assert!(!is_removed, "node is not removed");
        Ok(())
    }

    #[tokio::test]
    async fn test_include_nodes_to_module() -> anyhow::Result<()> {
        // set allowance for token transfer for the safe multiple times
        let _ = env_logger::builder().is_test(true).try_init();

        let mut node_addresses: Vec<Address> = Vec::new();
        for _ in 0..2 {
            node_addresses.push(get_random_address_for_testing());
        }

        // launch local anvil instance
        let anvil = create_anvil(None);
        let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let client = create_rpc_client_to_anvil(&anvil, &contract_deployer);
        let instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
            .await
            .expect("failed to deploy");
        // deploy multicall contract
        deploy_multicall3_for_testing(client.clone()).await?;
        // deploy safe suits
        deploy_safe_suites(client.clone()).await?;

        let deployer_vec: Vec<Address> = vec![a2h(contract_deployer.public().to_address())];

        // create a safe
        let (safe, node_module) = deploy_safe_module_with_targets_and_nodes(
            instances.stake_factory,
            *instances.channels.address(),
            None,
            deployer_vec.clone(),
            U256::from(1),
        )
        .await?;

        // check ndoes are not included
        for node_addr in node_addresses.clone() {
            // node is removed
            let node_is_not_included = node_module.isNode(node_addr).call().await?;
            assert!(!node_is_not_included, "node should not be included");
        }

        // include nodes to safe
        include_nodes_to_module(safe, node_addresses.clone(), *node_module.address(), contract_deployer).await?;

        // check nodes are included
        // check nodes are not included
        for node_addr in node_addresses {
            // node is removed
            let node_is_included = node_module.isNode(node_addr).call().await?;
            assert!(node_is_included, "node should be included");
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_migrate_nodes_to_new_network() -> anyhow::Result<()> {
        // set allowance for token transfer for the safe multiple times
        let _ = env_logger::builder().is_test(true).try_init();

        let mut node_addresses: Vec<Address> = Vec::new();
        for _ in 0..2 {
            node_addresses.push(get_random_address_for_testing());
        }

        // launch local anvil instance
        let anvil = create_anvil(None);
        let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let self_address: Address = a2h(contract_deployer.public().to_address());
        let client = create_rpc_client_to_anvil(&anvil, &contract_deployer);
        let instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
            .await
            .expect("failed to deploy");
        // deploy multicall contract
        deploy_multicall3_for_testing(client.clone()).await?;
        // deploy safe suits
        deploy_safe_suites(client.clone()).await?;

        // deploy some new contracts for the new network
        let new_safe_registry = HoprNodeSafeRegistry::deploy(client.clone()).await?;
        let new_token = HoprToken::deploy(client.clone()).await?;
        let new_channels = HoprChannels::deploy(
            client.clone(),
            *new_token.address(),
            1_u32,
            *new_safe_registry.address(),
        )
        .await?;
        let new_announcements = HoprAnnouncements::deploy(client.clone()).await?;

        let deployer_vec: Vec<Address> = vec![self_address];

        // create a safe
        let (safe, node_module) = deploy_safe_module_with_targets_and_nodes(
            instances.stake_factory,
            *instances.channels.address(),
            None,
            deployer_vec.clone(),
            U256::from(1),
        )
        .await?;

        // check new network is not included
        let old_channels_inclusion = node_module.tryGetTarget(*instances.channels.address()).call().await?;
        assert!(old_channels_inclusion._0, "old channel should be included");
        let new_channels_inclusion = node_module.tryGetTarget(*new_channels.address()).call().await?;
        assert!(!new_channels_inclusion._0, "new channel should not be included");

        // migrate nodes
        migrate_nodes(
            safe,
            *node_module.address(),
            *new_channels.address(),
            *new_token.address(),
            *new_announcements.address(),
            U256::MAX,
            contract_deployer,
        )
        .await?;

        // check new network is included
        let old_channels_inclusion = node_module.tryGetTarget(*instances.channels.address()).call().await?;
        assert!(old_channels_inclusion._0, "old channel should still be included");
        let new_channels_inclusion = node_module.tryGetTarget(*new_channels.address()).call().await?;
        assert!(new_channels_inclusion._0, "new channel should now be included");
        Ok(())
    }

    #[tokio::test]
    async fn test_debug_node_safe_module_setup_main() -> anyhow::Result<()> {
        // set allowance for token transfer for the safe multiple times
        let _ = env_logger::builder().is_test(true).try_init();

        let mut node_addresses: Vec<Address> = Vec::new();
        for _ in 0..2 {
            node_addresses.push(get_random_address_for_testing());
        }

        // launch local anvil instance
        let anvil = create_anvil(None);
        let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let client = create_rpc_client_to_anvil(&anvil, &contract_deployer);
        let instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
            .await
            .expect("failed to deploy");
        // deploy multicall contract
        deploy_multicall3_for_testing(client.clone()).await?;
        // deploy safe suits
        deploy_safe_suites(client.clone()).await?;

        let deployer_vec: Vec<Address> = vec![a2h(contract_deployer.public().to_address())];

        // create a safe
        let (_safe, _node_module) = deploy_safe_module_with_targets_and_nodes(
            instances.stake_factory,
            *instances.channels.address(),
            None,
            deployer_vec.clone(),
            U256::from(1),
        )
        .await?;

        Ok(())
    }
}
