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
use alloy::{
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
};
use hex_literal::hex;
use hopr_bindings::{
    hoprnetworkregistry::HoprNetworkRegistry::HoprNetworkRegistryInstance,
    hoprnodemanagementmodule::HoprNodeManagementModule::{
        HoprNodeManagementModuleInstance, addChannelsAndTokenTargetCall, includeNodeCall, removeNodeCall,
        scopeTargetTokenCall,
    },
    hoprnodesaferegistry::HoprNodeSafeRegistry::{HoprNodeSafeRegistryInstance, deregisterNodeBySafeCall},
    hoprnodestakefactory::HoprNodeStakeFactory::{HoprNodeStakeFactoryInstance, cloneCall},
    hoprtoken::HoprToken::{HoprTokenInstance, approveCall},
};
use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
use tracing::{debug, info};

use crate::utils::{
    DEFAULT_ANNOUNCEMENT_PERMISSIONS, DEFAULT_CAPABILITY_PERMISSIONS, DEFAULT_NODE_PERMISSIONS,
    DOMAIN_SEPARATOR_TYPEHASH, HelperErrors, SAFE_COMPATIBILITYFALLBACKHANDLER_ADDRESS, SAFE_MULTISEND_ADDRESS,
    SAFE_SAFE_ADDRESS, SAFE_SAFEPROXYFACTORY_ADDRESS, SAFE_TX_TYPEHASH, SENTINEL_OWNERS, get_create2_address,
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
    // Fund Multicall3 deployer and deploy ERC1820Registry
    let tx = TransactionRequest::default()
        .with_to(Address::from_str(crate::utils::MULTICALL3_DEPLOYER).expect("failed to parse MULTICALL3_DEPLOYER"))
        .with_value(U256::from(crate::utils::ETH_VALUE_FOR_MULTICALL3_DEPLOYER));

    provider
        .send_transaction(tx)
        .await?
        .watch()
        // .map_err(|e| ContractError::MiddlewareError { e })?
        .await?;

    provider.send_raw_transaction(
        &hex!("f90f538085174876e800830f42408080b90f00608060405234801561001057600080fd5b50610ee0806100206000396000f3fe6080604052600436106100f35760003560e01c80634d2301cc1161008a578063a8b0574e11610059578063a8b0574e1461025a578063bce38bd714610275578063c3077fa914610288578063ee82ac5e1461029b57600080fd5b80634d2301cc146101ec57806372425d9d1461022157806382ad56cb1461023457806386d516e81461024757600080fd5b80633408e470116100c65780633408e47014610191578063399542e9146101a45780633e64a696146101c657806342cbb15c146101d957600080fd5b80630f28c97d146100f8578063174dea711461011a578063252dba421461013a57806327e86d6e1461015b575b600080fd5b34801561010457600080fd5b50425b6040519081526020015b60405180910390f35b61012d610128366004610a85565b6102ba565b6040516101119190610bbe565b61014d610148366004610a85565b6104ef565b604051610111929190610bd8565b34801561016757600080fd5b50437fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0140610107565b34801561019d57600080fd5b5046610107565b6101b76101b2366004610c60565b610690565b60405161011193929190610cba565b3480156101d257600080fd5b5048610107565b3480156101e557600080fd5b5043610107565b3480156101f857600080fd5b50610107610207366004610ce2565b73ffffffffffffffffffffffffffffffffffffffff163190565b34801561022d57600080fd5b5044610107565b61012d610242366004610a85565b6106ab565b34801561025357600080fd5b5045610107565b34801561026657600080fd5b50604051418152602001610111565b61012d610283366004610c60565b61085a565b6101b7610296366004610a85565b610a1a565b3480156102a757600080fd5b506101076102b6366004610d18565b4090565b60606000828067ffffffffffffffff8111156102d8576102d8610d31565b60405190808252806020026020018201604052801561031e57816020015b6040805180820190915260008152606060208201528152602001906001900390816102f65790505b5092503660005b8281101561047757600085828151811061034157610341610d60565b6020026020010151905087878381811061035d5761035d610d60565b905060200281019061036f9190610d8f565b6040810135958601959093506103886020850185610ce2565b73ffffffffffffffffffffffffffffffffffffffff16816103ac6060870187610dcd565b6040516103ba929190610e32565b60006040518083038185875af1925050503d80600081146103f7576040519150601f19603f3d011682016040523d82523d6000602084013e6103fc565b606091505b50602080850191909152901515808452908501351761046d577f08c379a000000000000000000000000000000000000000000000000000000000600052602060045260176024527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060445260846000fd5b5050600101610325565b508234146104e6576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601a60248201527f4d756c746963616c6c333a2076616c7565206d69736d6174636800000000000060448201526064015b60405180910390fd5b50505092915050565b436060828067ffffffffffffffff81111561050c5761050c610d31565b60405190808252806020026020018201604052801561053f57816020015b606081526020019060019003908161052a5790505b5091503660005b8281101561068657600087878381811061056257610562610d60565b90506020028101906105749190610e42565b92506105836020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff166105a66020850185610dcd565b6040516105b4929190610e32565b6000604051808303816000865af19150503d80600081146105f1576040519150601f19603f3d011682016040523d82523d6000602084013e6105f6565b606091505b5086848151811061060957610609610d60565b602090810291909101015290508061067d576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601760248201527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060448201526064016104dd565b50600101610546565b5050509250929050565b43804060606106a086868661085a565b905093509350939050565b6060818067ffffffffffffffff8111156106c7576106c7610d31565b60405190808252806020026020018201604052801561070d57816020015b6040805180820190915260008152606060208201528152602001906001900390816106e55790505b5091503660005b828110156104e657600084828151811061073057610730610d60565b6020026020010151905086868381811061074c5761074c610d60565b905060200281019061075e9190610e76565b925061076d6020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff166107906040850185610dcd565b60405161079e929190610e32565b6000604051808303816000865af19150503d80600081146107db576040519150601f19603f3d011682016040523d82523d6000602084013e6107e0565b606091505b506020808401919091529015158083529084013517610851577f08c379a000000000000000000000000000000000000000000000000000000000600052602060045260176024527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060445260646000fd5b50600101610714565b6060818067ffffffffffffffff81111561087657610876610d31565b6040519080825280602002602001820160405280156108bc57816020015b6040805180820190915260008152606060208201528152602001906001900390816108945790505b5091503660005b82811015610a105760008482815181106108df576108df610d60565b602002602001015190508686838181106108fb576108fb610d60565b905060200281019061090d9190610e42565b925061091c6020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff1661093f6020850185610dcd565b60405161094d929190610e32565b6000604051808303816000865af19150503d806000811461098a576040519150601f19603f3d011682016040523d82523d6000602084013e61098f565b606091505b506020830152151581528715610a07578051610a07576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601760248201527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060448201526064016104dd565b506001016108c3565b5050509392505050565b6000806060610a2b60018686610690565b919790965090945092505050565b60008083601f840112610a4b57600080fd5b50813567ffffffffffffffff811115610a6357600080fd5b6020830191508360208260051b8501011115610a7e57600080fd5b9250929050565b60008060208385031215610a9857600080fd5b823567ffffffffffffffff811115610aaf57600080fd5b610abb85828601610a39565b90969095509350505050565b6000815180845260005b81811015610aed57602081850181015186830182015201610ad1565b81811115610aff576000602083870101525b50601f017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0169290920160200192915050565b600082825180855260208086019550808260051b84010181860160005b84811015610bb1578583037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe001895281518051151584528401516040858501819052610b9d81860183610ac7565b9a86019a9450505090830190600101610b4f565b5090979650505050505050565b602081526000610bd16020830184610b32565b9392505050565b600060408201848352602060408185015281855180845260608601915060608160051b870101935082870160005b82811015610c52577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa0888703018452610c40868351610ac7565b95509284019290840190600101610c06565b509398975050505050505050565b600080600060408486031215610c7557600080fd5b83358015158114610c8557600080fd5b9250602084013567ffffffffffffffff811115610ca157600080fd5b610cad86828701610a39565b9497909650939450505050565b838152826020820152606060408201526000610cd96060830184610b32565b95945050505050565b600060208284031215610cf457600080fd5b813573ffffffffffffffffffffffffffffffffffffffff81168114610bd157600080fd5b600060208284031215610d2a57600080fd5b5035919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b7f4e487b7100000000000000000000000000000000000000000000000000000000600052603260045260246000fd5b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff81833603018112610dc357600080fd5b9190910192915050565b60008083357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe1843603018112610e0257600080fd5b83018035915067ffffffffffffffff821115610e1d57600080fd5b602001915036819003821315610a7e57600080fd5b8183823760009101908152919050565b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc1833603018112610dc357600080fd5b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa1833603018112610dc357600080fdfea2646970667358221220bb2b5c71a328032f97c676ae39a1ec2148d3e5d6f73d95e9b17910152d61f16264736f6c634300080c00331ca0edce47092c0f398cebf3ffc267f05c8e7076e3b89445e0fe50f6332273d4569ba01b0b9d000e19b24c5869b0fc3b22b0d6fa47cd63316875cbbd577d76e6fde086")
            ).await?
            .watch()
            // .map_err(|e| ContractError::MiddlewareError {e})?
            .await?;
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

/// Get registered safes for given nodes on the network registry
pub async fn get_registered_safes_for_nodes_on_network_registry<P: Provider + WalletProvider>(
    network_registry: HoprNetworkRegistryInstance<Arc<P>>,
    node_addresses: Vec<Address>,
) -> Result<Vec<Address>, MulticallError> {
    let provider = network_registry.provider();

    let mut dynamic_multicall = MulticallBuilder::new_dynamic(provider.clone());

    for node in node_addresses {
        dynamic_multicall = dynamic_multicall.add_dynamic(network_registry.nodeRegisterdToAccount(node));
    }

    let response = dynamic_multicall.aggregate().await?;

    Ok(response)
}

/// Register safes and nodes to the network registry, and force-sync the eligibility to true.
/// It returns the number of removed nodes and nodes being added.
/// - If nodes have been registered to a different safe, overwrite it (remove the old safe and regsiter with the new
///   safe)
/// - If ndoes have been registered to the same safe, no op
/// - If nodes have not been registered to any safe, register it
///
/// After all the nodes have been added to the network registry, force-sync the eligibility of all the added safes to
/// true
pub async fn register_safes_and_nodes_on_network_registry<P: Provider + WalletProvider + Clone>(
    network_registry: HoprNetworkRegistryInstance<Arc<P>>,
    safe_addresses: Vec<Address>,
    node_addresses: Vec<Address>,
) -> Result<(usize, usize), HelperErrors> {
    assert_eq!(
        safe_addresses.len(),
        node_addresses.len(),
        "unmatched lengths of safes and nodes"
    );

    // check registered safes of given node addresses
    let registered_safes =
        get_registered_safes_for_nodes_on_network_registry(network_registry.clone(), node_addresses.clone()).await?;

    let mut nodes_to_remove: Vec<Address> = Vec::new();
    let mut safes_to_add: Vec<Address> = Vec::new();
    let mut nodes_to_add: Vec<Address> = Vec::new();

    for (i, registered_safe) in registered_safes.iter().enumerate() {
        if registered_safe.eq(&Address::ZERO) {
            // no entry, add to network registry
            safes_to_add.push(safe_addresses[i]);
            nodes_to_add.push(node_addresses[i]);
        } else if registered_safe.ne(&safe_addresses[i]) {
            // remove first then add
            nodes_to_remove.push(node_addresses[i]);
            safes_to_add.push(safe_addresses[i]);
            nodes_to_add.push(node_addresses[i]);
        } else {
            // no-op
        }
    }

    if !nodes_to_remove.is_empty() {
        // need to remove some nodes
        network_registry
            .managerDeregister(nodes_to_remove.clone())
            .send()
            .await?
            .watch()
            .await?;
    }

    network_registry
        .managerRegister(safes_to_add.clone(), nodes_to_add.clone())
        .send()
        .await?
        .watch()
        .await?;

    // force sync their eligibility
    network_registry
        .managerForceSync(safes_to_add.clone(), vec![true; safes_to_add.len()])
        .send()
        .await?
        .watch()
        .await?;

    Ok((nodes_to_remove.len(), nodes_to_add.len()))
}

/// Deregister safes and nodes from the network registry. Does not do any action on the eligibility.
/// It returns the number of removed nodes
/// - If nodes have been registered to a safe, remove the node
/// - If nodes have not been registered to any safe, no op
pub async fn deregister_nodes_from_network_registry<P: Provider + WalletProvider + Clone>(
    network_registry: HoprNetworkRegistryInstance<Arc<P>>,
    node_addresses: Vec<Address>,
) -> Result<usize, HelperErrors> {
    // check registered safes of given node addresses
    let registered_safes =
        get_registered_safes_for_nodes_on_network_registry(network_registry.clone(), node_addresses.clone()).await?;

    let mut nodes_to_remove: Vec<Address> = Vec::new();

    for (i, registered_safe) in registered_safes.iter().enumerate() {
        if registered_safe.ne(&Address::ZERO) {
            // remove the node
            nodes_to_remove.push(node_addresses[i]);
        }
    }

    if !nodes_to_remove.is_empty() {
        // need to remove some nodes
        network_registry
            .managerDeregister(nodes_to_remove.clone())
            .send()
            .await?
            .watch()
            .await?;
    }
    Ok(nodes_to_remove.len())
}

/// Force-sync the eligibility to given values. This can only be called with a manager account
pub async fn force_sync_safes_on_network_registry<P: Provider>(
    network_registry: HoprNetworkRegistryInstance<Arc<P>>,
    safe_addresses: Vec<Address>,
    eligibilities: Vec<bool>,
) -> Result<(), HelperErrors> {
    assert_eq!(
        safe_addresses.len(),
        eligibilities.len(),
        "unmatched lengths of safes and eligibilities"
    );

    // force sync their eligibility
    network_registry
        .managerForceSync(safe_addresses, eligibilities)
        .send()
        .await?
        .watch()
        .await?;

    Ok(())
}

pub async fn toggle_network_registry_status<P: Provider>(
    network_registry: HoprNetworkRegistryInstance<Arc<P>>,
    status: bool,
) -> Result<(), HelperErrors> {
    let current_status = network_registry.enabled().call().await?;

    info!(
        current_status = ?current_status,
        desired_status = ?status,
        "Toggling network registry status",
    );

    if current_status == status {
        info!("Network registry is already in the desired state: {:?}", status);
        return Ok(());
    }

    if status {
        info!("Enabling the network registry");
        // enable network registry
        network_registry.enableRegistry().send().await?.watch().await?;
    } else {
        info!("Disabling the network registry");
        // disable network registry
        network_registry.disableRegistry().send().await?.watch().await?;
    }
    Ok(())
}

/// Helper function to predict module address. Note that here the caller is the contract deployer
pub fn predict_module_address(
    caller: Address,
    nonce: B256,
    factory_address: Address,
    implementation_address: Address,
) -> Result<Address, HelperErrors> {
    let module_salt = keccak256((caller, nonce).abi_encode_packed());
    debug!("module_salt {:?}", module_salt);

    let module_creation_code = (
        Bytes::copy_from_slice(&hex!("3d602d80600a3d3981f3363d3d373d3d3d363d73")),
        implementation_address,
        Bytes::copy_from_slice(&hex!("5af43d82803e903d91602b57fd5bf3")),
    )
        .abi_encode_packed();
    debug!("module_creation_code {:?}", module_creation_code);

    let predict_module_addr = get_create2_address(factory_address, module_salt, module_creation_code);
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
    temp_admins[0] = stake_factory;

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

/// Deploy a safe and a module proxies via HoprStakeFactory contract with default permissions and announcement targets
/// Within one multicall, as an owner of the safe:
/// - deploy a safe proxy instance and a module proxy instance with multicall as an owner
/// - add announcement as a permitted target in the deployed module proxy
/// - approve token transfer to be done for the safe by channel contracts
/// - if node addresses are known, include nodes to the module by safe
/// - transfer safe ownership to actual admins
/// - set desired threshold
/// - if node addresses are known, include nodes and safes to the network registry.
///
/// Returns safe proxy address and module proxy address
#[allow(clippy::too_many_arguments)]
pub async fn deploy_safe_module_with_targets_and_nodes<P: WalletProvider + Provider>(
    hopr_node_stake_factory: HoprNodeStakeFactoryInstance<Arc<P>>,
    hopr_token_address: Address,
    hopr_channels_address: Address,
    hopr_module_implementation_address: Address,
    hopr_announcement_address: Address,
    allowance: U256,
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

    // predict module and safe address
    let module_address = predict_module_address(
        MULTICALL3_ADDRESS,
        nonce,
        *hopr_node_stake_factory.address(),
        hopr_module_implementation_address,
    )?;
    info!("predicted module address {:?}", module_address.to_string());

    let safe_address = predict_safe_address(
        *hopr_node_stake_factory.address(),
        temporary_admins.clone(),
        nonce,
        Address::from_str(SAFE_COMPATIBILITYFALLBACKHANDLER_ADDRESS).unwrap(),
        Address::from_str(SAFE_SAFE_ADDRESS).unwrap(),
        Address::from_str(SAFE_SAFEPROXYFACTORY_ADDRESS).unwrap(),
    )?;
    info!("predicted safe address {:?}", safe_address.to_string());

    let deployed_module = HoprNodeManagementModuleInstance::new(module_address, provider.clone());
    let deployed_safe = SafeSingleton::new(safe_address, provider.clone());

    // Use multicall to deploy a safe proxy instance and a module proxy instance with multicall as an owner
    let mut multicall_payloads: Vec<Call3> = vec![];
    let safe_address = *deployed_safe.address();
    multicall_payloads.push(Call3 {
        target: *hopr_node_stake_factory.address(),
        allowFailure: false,
        callData: cloneCall {
            moduleSingletonAddress: hopr_module_implementation_address,
            admins: temporary_admins,
            nonce: nonce.into(),
            defaultTarget: default_target.into(),
        }
        .abi_encode()
        .into(),
    });
    info!("Safe and module deployment multicall payload is created");

    // add announcement as a permitted target in the deployed module proxy
    let announcement_target =
        U256::from_str(format!("{hopr_announcement_address:?}{DEFAULT_ANNOUNCEMENT_PERMISSIONS}").as_str()).unwrap();
    let scope_announcement_tx_payload = scopeTargetTokenCall {
        defaultTarget: announcement_target,
    }
    .abi_encode();

    let multicall_payload_2 = prepare_safe_tx_multicall_payload_from_owner_contract(
        // multicall.clone(),
        safe_address,
        module_address,
        caller,
        scope_announcement_tx_payload,
    );
    // let multicall = multicall.add_call(multicall_payload_2);
    multicall_payloads.push(multicall_payload_2.to_call3());
    info!("Announcement contract scoping multicall payload is created");

    // approve token transfer to be done for the safe by channel contracts
    let approve_tx_payload = approveCall {
        spender: hopr_channels_address,
        value: allowance,
    }
    .abi_encode();

    let multicall_payload_3 = prepare_safe_tx_multicall_payload_from_owner_contract(
        safe_address,
        hopr_token_address,
        caller,
        approve_tx_payload,
    );
    // let multicall = multicall.add_call(multicall_payload_3);
    multicall_payloads.push(multicall_payload_3.to_call3());

    info!("Token transfer approval multicall payload is created");

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
        .decoded_log::<hopr_bindings::hoprnodestakefactory::HoprNodeStakeFactory::NewHoprNodeStakeSafe>()
        .ok_or_else(|| HelperErrors::ContractNotDeployed("cannot find safe from log".into()))?
        .instance;
    let module_address_from_log = tx_receipt
        .decoded_log::<hopr_bindings::hoprnodestakefactory::HoprNodeStakeFactory::NewHoprNodeStakeModule>()
        .ok_or_else(|| HelperErrors::ContractNotDeployed("cannot find module from log".into()))?
        .instance;

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
                Address::from_str(SAFE_MULTISEND_ADDRESS).unwrap(),
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
        Address::from_str(SAFE_MULTISEND_ADDRESS).unwrap(),
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
        Address::from_str(SAFE_MULTISEND_ADDRESS).unwrap(),
        multisend_txns,
        chain_id,
        safe_nonce,
    )
    .await?;

    Ok(())
}

/// Quick check if the following values are correct, for one single node:
/// 1. node xDAI balance
/// 2. If node has been included on Network Registry
/// 3. If node and safe are associated on Node Safe Registry
pub async fn debug_node_safe_module_setup_on_balance_and_registries<P: Provider>(
    network_registry: HoprNetworkRegistryInstance<Arc<P>>,
    node_safe_registry: HoprNodeSafeRegistryInstance<Arc<P>>,
    node_address: &Address,
) -> Result<Address, MulticallError> {
    let provider = network_registry.provider();
    // let mut multicall = Multicall::new(provider.clone(), Some(MULTICALL_ADDRESS))
    //     .await
    //     .expect("cannot create multicall");

    info!("checking for node {:?}", node_address);
    let multicall = provider
        .multicall()
        // 1. node xDAI balance
        .get_eth_balance(*node_address)
        // 2. get safe address from the Network Registry
        .add(network_registry.nodeRegisterdToAccount(*node_address))
        // 3. get the safe address from the Node Safe Registry
        .add(node_safe_registry.nodeToSafe(*node_address));

    let (node_native_balance, safe_in_network_registry, safe_in_nodesafe_registry) = multicall.aggregate().await?;

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

    if safe_in_network_registry.eq(&Address::ZERO) {
        info!("Please register the node to the network registry");
    } else {
        info!("safe in network registry {:?}", safe_in_network_registry);
    }

    if safe_in_nodesafe_registry.eq(&Address::ZERO) {
        info!("Please start the node. It will auto-register to node-safe registry");
    } else {
        info!("safe in node-safe registry {:?}", safe_in_nodesafe_registry);
    }
    info!(
        "Safes in both registies should match: {:?}",
        safe_in_network_registry.eq(&safe_in_nodesafe_registry)
    );

    Ok(safe_in_network_registry)
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

    use alloy::{
        contract::Result as ContractResult,
        network::{EthereumWallet, TransactionBuilder},
        primitives::address,
        providers::{Identity, RootProvider, fillers::*},
        rpc::types::TransactionRequest,
        sol_types::SolValue,
    };
    use hopr_bindings::{
        hoprannouncements::HoprAnnouncements, hoprchannels::HoprChannels, hoprnetworkregistry::HoprNetworkRegistry,
        hoprnodesaferegistry::HoprNodeSafeRegistry, hoprnodestakefactory::HoprNodeStakeFactory, hoprtoken::HoprToken,
    };
    use hopr_chain_types::ContractInstances;
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
    use hopr_primitive_types::prelude::BytesRepresentable;

    use super::*;

    pub type AnvilRpcClient = FillProvider<
        JoinFill<
            JoinFill<Identity, JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>>,
            WalletFiller<EthereumWallet>,
        >,
        RootProvider,
    >;
    /// Used for testing. Creates RPC client to the local Anvil instance.
    pub fn create_rpc_client_to_anvil(
        anvil: &alloy::node_bindings::AnvilInstance,
        signer: &hopr_crypto_types::keypairs::ChainKeypair,
    ) -> Arc<AnvilRpcClient> {
        use alloy::{
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
            let _tx_safe_compatibility_fallback_handler = TransactionRequest::default().with_to(safe_diamond_proxy_address).with_input(
            hex!("0000000000000000000000000000000000000000000000000000000000000000608060405234801561001057600080fd5b5061073f806100206000396000f3fe608060405234801561001057600080fd5b50600436106100565760003560e01c806223de291461005b57806301ffc9a714610193578063150b7a02146101f6578063bc197c81146102ec578063f23a6e6114610482575b600080fd5b610191600480360360c081101561007157600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190803590602001906401000000008111156100f857600080fd5b82018360208201111561010a57600080fd5b8035906020019184600183028401116401000000008311171561012c57600080fd5b90919293919293908035906020019064010000000081111561014d57600080fd5b82018360208201111561015f57600080fd5b8035906020019184600183028401116401000000008311171561018157600080fd5b9091929391929390505050610582565b005b6101de600480360360208110156101a957600080fd5b8101908080357bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916906020019092919050505061058c565b60405180821515815260200191505060405180910390f35b6102b76004803603608081101561020c57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803590602001909291908035906020019064010000000081111561027357600080fd5b82018360208201111561028557600080fd5b803590602001918460018302840111640100000000831117156102a757600080fd5b90919293919293905050506106c6565b60405180827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916815260200191505060405180910390f35b61044d600480360360a081101561030257600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff1690602001909291908035906020019064010000000081111561035f57600080fd5b82018360208201111561037157600080fd5b8035906020019184602083028401116401000000008311171561039357600080fd5b9091929391929390803590602001906401000000008111156103b457600080fd5b8201836020820111156103c657600080fd5b803590602001918460208302840111640100000000831117156103e857600080fd5b90919293919293908035906020019064010000000081111561040957600080fd5b82018360208201111561041b57600080fd5b8035906020019184600183028401116401000000008311171561043d57600080fd5b90919293919293905050506106db565b60405180827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916815260200191505060405180910390f35b61054d600480360360a081101561049857600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190803590602001909291908035906020019064010000000081111561050957600080fd5b82018360208201111561051b57600080fd5b8035906020019184600183028401116401000000008311171561053d57600080fd5b90919293919293905050506106f3565b60405180827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916815260200191505060405180910390f35b5050505050505050565b60007f4e2312e0000000000000000000000000000000000000000000000000000000007bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916148061065757507f150b7a02000000000000000000000000000000000000000000000000000000007bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916145b806106bf57507f01ffc9a7000000000000000000000000000000000000000000000000000000007bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916145b9050919050565b600063150b7a0260e01b905095945050505050565b600063bc197c8160e01b905098975050505050505050565b600063f23a6e6160e01b9050969550505050505056fea2646970667358221220f62cd059f3672bb04062df149e7ae71534a8512cca0172e695d98a43cff0c53564736f6c63430007060033")
        );
            // 3. Library: only MultiSendCallOnly and omit MultiSendCall
            let _tx_safe_multisend_call_only = TransactionRequest::default().with_to(safe_diamond_proxy_address).with_input(
            hex!("0000000000000000000000000000000000000000000000000000000000000000608060405234801561001057600080fd5b5061019a806100206000396000f3fe60806040526004361061001e5760003560e01c80638d80ff0a14610023575b600080fd5b6100dc6004803603602081101561003957600080fd5b810190808035906020019064010000000081111561005657600080fd5b82018360208201111561006857600080fd5b8035906020019184600183028401116401000000008311171561008a57600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f8201169050808301925050505050505091929192905050506100de565b005b805160205b8181101561015f578083015160f81c6001820184015160601c60158301850151603584018601516055850187016000856000811461012857600181146101385761013d565b6000808585888a5af1915061013d565b600080fd5b50600081141561014c57600080fd5b82605501870196505050505050506100e3565b50505056fea26469706673582212208d297bb003abee230b5dfb38774688f37a6fbb97a82a21728e8049b2acb9b73564736f6c63430007060033")
        );
            // 4. Safe singleton deploySafe();
            let _tx_safe_singleton = TransactionRequest::default().with_to(safe_diamond_proxy_address).with_input(
            hex!("0000000000000000000000000000000000000000000000000000000000000000608060405234801561001057600080fd5b506001600481905550615c1b80620000296000396000f3fe6080604052600436106101d15760003560e01c8063affed0e0116100f7578063e19a9dd911610095578063f08a032311610064578063f08a03231461156b578063f698da25146115bc578063f8dc5dd9146115e7578063ffa1ad741461166257610226565b8063e19a9dd9146112bf578063e318b52b14611310578063e75235b8146113a1578063e86637db146113cc57610226565b8063cc2f8452116100d1578063cc2f84521461100c578063d4d9bdcd146110d9578063d8d11f7814611114578063e009cfde1461124e57610226565b8063affed0e014610d89578063b4faba0914610db4578063b63e800d14610e9c57610226565b80635624b25b1161016f5780636a7612021161013e5780636a761202146109895780637d83297414610b45578063934f3a1114610bb4578063a0e67e2b14610d1d57610226565b80635624b25b146107f05780635ae6bd37146108ae578063610b5925146108fd578063694e80c31461094e57610226565b80632f54bf6e116101ab5780632f54bf6e146104c85780633408e4701461052f578063468721a71461055a5780635229073f1461066f57610226565b80630d582f131461029357806312fb68e0146102ee5780632d9ad53d1461046157610226565b36610226573373ffffffffffffffffffffffffffffffffffffffff167f3d0ce9bfc3ed7d6862dbb28b2dea94561fe714a1b4d019aa8af39730d1ad7c3d346040518082815260200191505060405180910390a2005b34801561023257600080fd5b5060007f6c9a6c4a39284e37ed1cf53d337577d14212a4870fb976a4366c693b939918d560001b905080548061026757600080f35b36600080373360601b365260008060143601600080855af13d6000803e8061028e573d6000fd5b3d6000f35b34801561029f57600080fd5b506102ec600480360360408110156102b657600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803590602001909291905050506116f2565b005b3480156102fa57600080fd5b5061045f6004803603608081101561031157600080fd5b81019080803590602001909291908035906020019064010000000081111561033857600080fd5b82018360208201111561034a57600080fd5b8035906020019184600183028401116401000000008311171561036c57600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f820116905080830192505050505050509192919290803590602001906401000000008111156103cf57600080fd5b8201836020820111156103e157600080fd5b8035906020019184600183028401116401000000008311171561040357600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f82011690508083019250505050505050919291929080359060200190929190505050611ad8565b005b34801561046d57600080fd5b506104b06004803603602081101561048457600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff1690602001909291905050506123d6565b60405180821515815260200191505060405180910390f35b3480156104d457600080fd5b50610517600480360360208110156104eb57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff1690602001909291905050506124a8565b60405180821515815260200191505060405180910390f35b34801561053b57600080fd5b5061054461257a565b6040518082815260200191505060405180910390f35b34801561056657600080fd5b506106576004803603608081101561057d57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190803590602001906401000000008111156105c457600080fd5b8201836020820111156105d657600080fd5b803590602001918460018302840111640100000000831117156105f857600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f820116905080830192505050505050509192919290803560ff169060200190929190505050612587565b60405180821515815260200191505060405180910390f35b34801561067b57600080fd5b5061076c6004803603608081101561069257600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190803590602001906401000000008111156106d957600080fd5b8201836020820111156106eb57600080fd5b8035906020019184600183028401116401000000008311171561070d57600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f820116905080830192505050505050509192919290803560ff16906020019092919050505061278d565b60405180831515815260200180602001828103825283818151815260200191508051906020019080838360005b838110156107b4578082015181840152602081019050610799565b50505050905090810190601f1680156107e15780820380516001836020036101000a031916815260200191505b50935050505060405180910390f35b3480156107fc57600080fd5b506108336004803603604081101561081357600080fd5b8101908080359060200190929190803590602001909291905050506127c3565b6040518080602001828103825283818151815260200191508051906020019080838360005b83811015610873578082015181840152602081019050610858565b50505050905090810190601f1680156108a05780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b3480156108ba57600080fd5b506108e7600480360360208110156108d157600080fd5b810190808035906020019092919050505061284a565b6040518082815260200191505060405180910390f35b34801561090957600080fd5b5061094c6004803603602081101561092057600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190505050612862565b005b34801561095a57600080fd5b506109876004803603602081101561097157600080fd5b8101908080359060200190929190505050612bea565b005b610b2d60048036036101408110156109a057600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190803590602001906401000000008111156109e757600080fd5b8201836020820111156109f957600080fd5b80359060200191846001830284011164010000000083111715610a1b57600080fd5b9091929391929390803560ff169060200190929190803590602001909291908035906020019092919080359060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190640100000000811115610aa757600080fd5b820183602082011115610ab957600080fd5b80359060200191846001830284011164010000000083111715610adb57600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f820116905080830192505050505050509192919290505050612d24565b60405180821515815260200191505060405180910390f35b348015610b5157600080fd5b50610b9e60048036036040811015610b6857600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190505050613253565b6040518082815260200191505060405180910390f35b348015610bc057600080fd5b50610d1b60048036036060811015610bd757600080fd5b810190808035906020019092919080359060200190640100000000811115610bfe57600080fd5b820183602082011115610c1057600080fd5b80359060200191846001830284011164010000000083111715610c3257600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f82011690508083019250505050505050919291929080359060200190640100000000811115610c9557600080fd5b820183602082011115610ca757600080fd5b80359060200191846001830284011164010000000083111715610cc957600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f820116905080830192505050505050509192919290505050613278565b005b348015610d2957600080fd5b50610d32613307565b6040518080602001828103825283818151815260200191508051906020019060200280838360005b83811015610d75578082015181840152602081019050610d5a565b505050509050019250505060405180910390f35b348015610d9557600080fd5b50610d9e6134b0565b6040518082815260200191505060405180910390f35b348015610dc057600080fd5b50610e9a60048036036040811015610dd757600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190640100000000811115610e1457600080fd5b820183602082011115610e2657600080fd5b80359060200191846001830284011164010000000083111715610e4857600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f8201169050808301925050505050505091929192905050506134b6565b005b348015610ea857600080fd5b5061100a6004803603610100811015610ec057600080fd5b8101908080359060200190640100000000811115610edd57600080fd5b820183602082011115610eef57600080fd5b80359060200191846020830284011164010000000083111715610f1157600080fd5b909192939192939080359060200190929190803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190640100000000811115610f5c57600080fd5b820183602082011115610f6e57600080fd5b80359060200191846001830284011164010000000083111715610f9057600080fd5b9091929391929390803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190803573ffffffffffffffffffffffffffffffffffffffff1690602001909291905050506134d8565b005b34801561101857600080fd5b506110656004803603604081101561102f57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190505050613696565b60405180806020018373ffffffffffffffffffffffffffffffffffffffff168152602001828103825284818151815260200191508051906020019060200280838360005b838110156110c45780820151818401526020810190506110a9565b50505050905001935050505060405180910390f35b3480156110e557600080fd5b50611112600480360360208110156110fc57600080fd5b81019080803590602001909291905050506139f9565b005b34801561112057600080fd5b50611238600480360361014081101561113857600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803590602001909291908035906020019064010000000081111561117f57600080fd5b82018360208201111561119157600080fd5b803590602001918460018302840111640100000000831117156111b357600080fd5b9091929391929390803560ff169060200190929190803590602001909291908035906020019092919080359060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190505050613b98565b6040518082815260200191505060405180910390f35b34801561125a57600080fd5b506112bd6004803603604081101561127157600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190505050613bc5565b005b3480156112cb57600080fd5b5061130e600480360360208110156112e257600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190505050613f4c565b005b34801561131c57600080fd5b5061139f6004803603606081101561133357600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190505050614138565b005b3480156113ad57600080fd5b506113b6614796565b6040518082815260200191505060405180910390f35b3480156113d857600080fd5b506114f060048036036101408110156113f057600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803590602001909291908035906020019064010000000081111561143757600080fd5b82018360208201111561144957600080fd5b8035906020019184600183028401116401000000008311171561146b57600080fd5b9091929391929390803560ff169060200190929190803590602001909291908035906020019092919080359060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803590602001909291905050506147a0565b6040518080602001828103825283818151815260200191508051906020019080838360005b83811015611530578082015181840152602081019050611515565b50505050905090810190601f16801561155d5780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b34801561157757600080fd5b506115ba6004803603602081101561158e57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190505050614948565b005b3480156115c857600080fd5b506115d161499f565b6040518082815260200191505060405180910390f35b3480156115f357600080fd5b506116606004803603606081101561160a57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190505050614a1d565b005b34801561166e57600080fd5b50611677614e46565b6040518080602001828103825283818151815260200191508051906020019080838360005b838110156116b757808201518184015260208101905061169c565b50505050905090810190601f1680156116e45780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b6116fa614e7f565b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff16141580156117645750600173ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614155b801561179c57503073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614155b61180e576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303300000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff16600260008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff161461190f576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303400000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60026000600173ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16600260008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508160026000600173ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055506003600081548092919060010191905055508173ffffffffffffffffffffffffffffffffffffffff167f9465fa0c962cc76958e6373a993326400c1c94f8be2fe3a952adfa7f60b2ea2660405160405180910390a28060045414611ad457611ad381612bea565b5b5050565b611aec604182614f2290919063ffffffff16565b82511015611b62576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330323000000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b6000808060008060005b868110156123ca57611b7e8882614f5c565b80945081955082965050505060008460ff1614156120035789898051906020012014611c12576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330323700000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b8260001c9450611c2c604188614f2290919063ffffffff16565b8260001c1015611ca4576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330323100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b8751611cbd60208460001c614f8b90919063ffffffff16565b1115611d31576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330323200000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60006020838a01015190508851611d6782611d5960208760001c614f8b90919063ffffffff16565b614f8b90919063ffffffff16565b1115611ddb576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330323300000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60606020848b010190506320c13b0b60e01b7bffffffffffffffffffffffffffffffffffffffffffffffffffffffff19168773ffffffffffffffffffffffffffffffffffffffff166320c13b0b8d846040518363ffffffff1660e01b8152600401808060200180602001838103835285818151815260200191508051906020019080838360005b83811015611e7d578082015181840152602081019050611e62565b50505050905090810190601f168015611eaa5780820380516001836020036101000a031916815260200191505b50838103825284818151815260200191508051906020019080838360005b83811015611ee3578082015181840152602081019050611ec8565b50505050905090810190601f168015611f105780820380516001836020036101000a031916815260200191505b5094505050505060206040518083038186803b158015611f2f57600080fd5b505afa158015611f43573d6000803e3d6000fd5b505050506040513d6020811015611f5957600080fd5b81019080805190602001909291905050507bffffffffffffffffffffffffffffffffffffffffffffffffffffffff191614611ffc576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330323400000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b5050612248565b60018460ff161415612117578260001c94508473ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff1614806120a057506000600860008773ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008c81526020019081526020016000205414155b612112576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330323500000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b612247565b601e8460ff1611156121df5760018a60405160200180807f19457468657265756d205369676e6564204d6573736167653a0a333200000000815250601c018281526020019150506040516020818303038152906040528051906020012060048603858560405160008152602001604052604051808581526020018460ff1681526020018381526020018281526020019450505050506020604051602081039080840390855afa1580156121ce573d6000803e3d6000fd5b505050602060405103519450612246565b60018a85858560405160008152602001604052604051808581526020018460ff1681526020018381526020018281526020019450505050506020604051602081039080840390855afa158015612239573d6000803e3d6000fd5b5050506020604051035194505b5b5b8573ffffffffffffffffffffffffffffffffffffffff168573ffffffffffffffffffffffffffffffffffffffff1611801561230f5750600073ffffffffffffffffffffffffffffffffffffffff16600260008773ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614155b80156123485750600173ffffffffffffffffffffffffffffffffffffffff168573ffffffffffffffffffffffffffffffffffffffff1614155b6123ba576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330323600000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b8495508080600101915050611b6c565b50505050505050505050565b60008173ffffffffffffffffffffffffffffffffffffffff16600173ffffffffffffffffffffffffffffffffffffffff16141580156124a15750600073ffffffffffffffffffffffffffffffffffffffff16600160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614155b9050919050565b6000600173ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff16141580156125735750600073ffffffffffffffffffffffffffffffffffffffff16600260008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614155b9050919050565b6000804690508091505090565b6000600173ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff16141580156126525750600073ffffffffffffffffffffffffffffffffffffffff16600160003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614155b6126c4576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475331303400000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b6126f1858585857fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff614faa565b90508015612741573373ffffffffffffffffffffffffffffffffffffffff167f6895c13664aa4f67288b25d7a21d7aaa34916e355fb9b6fae0a139a9085becb860405160405180910390a2612785565b3373ffffffffffffffffffffffffffffffffffffffff167facd2c8702804128fdb0db2bb49f6d127dd0181c13fd45dbfe16de0930e2bd37560405160405180910390a25b949350505050565b6000606061279d86868686612587565b915060405160203d0181016040523d81523d6000602083013e8091505094509492505050565b606060006020830267ffffffffffffffff811180156127e157600080fd5b506040519080825280601f01601f1916602001820160405280156128145781602001600182028036833780820191505090505b50905060005b8381101561283f5780850154806020830260208501015250808060010191505061281a565b508091505092915050565b60076020528060005260406000206000915090505481565b61286a614e7f565b600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16141580156128d45750600173ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614155b612946576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475331303100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff16600160008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614612a47576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475331303200000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60016000600173ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16600160008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508060016000600173ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508073ffffffffffffffffffffffffffffffffffffffff167fecdf3a3effea5783a3c4c2140e677577666428d44ed9d474a0b3a4c9943f844060405160405180910390a250565b612bf2614e7f565b600354811115612c6a576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b6001811015612ce1576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303200000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b806004819055507f610f7ff2b304ae8903c3de74c60c6ab1f7d6226b3f52c5161905bb5ad4039c936004546040518082815260200191505060405180910390a150565b6000806000612d3e8e8e8e8e8e8e8e8e8e8e6005546147a0565b905060056000815480929190600101919050555080805190602001209150612d67828286613278565b506000612d72614ff6565b9050600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614612f58578073ffffffffffffffffffffffffffffffffffffffff166375f0bb528f8f8f8f8f8f8f8f8f8f8f336040518d63ffffffff1660e01b8152600401808d73ffffffffffffffffffffffffffffffffffffffff1681526020018c8152602001806020018a6001811115612e1557fe5b81526020018981526020018881526020018781526020018673ffffffffffffffffffffffffffffffffffffffff1681526020018573ffffffffffffffffffffffffffffffffffffffff168152602001806020018473ffffffffffffffffffffffffffffffffffffffff16815260200183810383528d8d82818152602001925080828437600081840152601f19601f820116905080830192505050838103825285818151815260200191508051906020019080838360005b83811015612ee7578082015181840152602081019050612ecc565b50505050905090810190601f168015612f145780820380516001836020036101000a031916815260200191505b509e505050505050505050505050505050600060405180830381600087803b158015612f3f57600080fd5b505af1158015612f53573d6000803e3d6000fd5b505050505b6101f4612f7f6109c48b01603f60408d0281612f7057fe5b0461502790919063ffffffff16565b015a1015612ff5576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330313000000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60005a905061305e8f8f8f8f8080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f820116905080830192505050505050508e60008d14613053578e613059565b6109c45a035b614faa565b93506130735a8261504190919063ffffffff16565b90508380613082575060008a14155b8061308e575060008814155b613100576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330313300000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60008089111561311a57613117828b8b8b8b615061565b90505b841561315d57837f442e715f626346e8c54381002da614f62bee8d27386535b2521ec8540898556e826040518082815260200191505060405180910390a2613196565b837f23428b18acfb3ea64b08dc0c1d296ea9c09702c09083ca5272e64d115b687d23826040518082815260200191505060405180910390a25b5050600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614613242578073ffffffffffffffffffffffffffffffffffffffff16639327136883856040518363ffffffff1660e01b815260040180838152602001821515815260200192505050600060405180830381600087803b15801561322957600080fd5b505af115801561323d573d6000803e3d6000fd5b505050505b50509b9a5050505050505050505050565b6008602052816000526040600020602052806000526040600020600091509150505481565b60006004549050600081116132f5576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330303100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b61330184848484611ad8565b50505050565b6060600060035467ffffffffffffffff8111801561332457600080fd5b506040519080825280602002602001820160405280156133535781602001602082028036833780820191505090505b50905060008060026000600173ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1690505b600173ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16146134a757808383815181106133fe57fe5b602002602001019073ffffffffffffffffffffffffffffffffffffffff16908173ffffffffffffffffffffffffffffffffffffffff1681525050600260008273ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16905081806001019250506133bd565b82935050505090565b60055481565b600080825160208401855af4806000523d6020523d600060403e60403d016000fd5b6135238a8a80806020026020016040519081016040528093929190818152602001838360200280828437600081840152601f19601f8201169050808301925050505050505089615267565b600073ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff16146135615761356084615767565b5b6135af8787878080601f016020809104026020016040519081016040528093929190818152602001838380828437600081840152601f19601f82011690508083019250505050505050615838565b60008211156135c9576135c782600060018685615061565b505b3373ffffffffffffffffffffffffffffffffffffffff167f141df868a6331af528e38c83b7aa03edc19be66e37ae67f9285bf4f8e3c6a1a88b8b8b8b8960405180806020018581526020018473ffffffffffffffffffffffffffffffffffffffff1681526020018373ffffffffffffffffffffffffffffffffffffffff1681526020018281038252878782818152602001925060200280828437600081840152601f19601f820116905080830192505050965050505050505060405180910390a250505050505050505050565b60606000600173ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff1614806136da57506136d9846123d6565b5b61374c576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475331303500000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600083116137c2576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475331303600000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b8267ffffffffffffffff811180156137d957600080fd5b506040519080825280602002602001820160405280156138085781602001602082028036833780820191505090505b5091506000600160008673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1691505b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff16141580156138da5750600173ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614155b80156138e557508381105b156139a057818382815181106138f757fe5b602002602001019073ffffffffffffffffffffffffffffffffffffffff16908173ffffffffffffffffffffffffffffffffffffffff1681525050600160008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1691508080600101915050613870565b600173ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff16146139ee578260018203815181106139e357fe5b602002602001015191505b808352509250929050565b600073ffffffffffffffffffffffffffffffffffffffff16600260003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff161415613afb576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330333000000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b6001600860003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000206000838152602001908152602001600020819055503373ffffffffffffffffffffffffffffffffffffffff16817ff2a0eb156472d1440255b0d7c1e19cc07115d1051fe605b0dce69acfec884d9c60405160405180910390a350565b6000613bad8c8c8c8c8c8c8c8c8c8c8c6147a0565b8051906020012090509b9a5050505050505050505050565b613bcd614e7f565b600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614158015613c375750600173ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614155b613ca9576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475331303100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b8073ffffffffffffffffffffffffffffffffffffffff16600160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614613da9576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475331303300000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600160008273ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16600160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055506000600160008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508073ffffffffffffffffffffffffffffffffffffffff167faab4fa2b463f581b2b32cb3b7e3b704b9ce37cc209b5fb4d77e593ace405427660405160405180910390a25050565b613f54614e7f565b600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16146140c6578073ffffffffffffffffffffffffffffffffffffffff166301ffc9a77fe6d7a83a000000000000000000000000000000000000000000000000000000006040518263ffffffff1660e01b815260040180827bffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916815260200191505060206040518083038186803b15801561401857600080fd5b505afa15801561402c573d6000803e3d6000fd5b505050506040513d602081101561404257600080fd5b81019080805190602001909291905050506140c5576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475333303000000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b5b60007f4a204f620c8c5ccdca3fd54d003badd85ba500436a431f0cbda4f558c93c34c860001b90508181558173ffffffffffffffffffffffffffffffffffffffff167f1151116914515bc0891ff9047a6cb32cf902546f83066499bcf8ba33d2353fa260405160405180910390a25050565b614140614e7f565b600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16141580156141aa5750600173ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614155b80156141e257503073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614155b614254576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303300000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff16600260008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614614355576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303400000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff16141580156143bf5750600173ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614155b614431576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303300000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b8173ffffffffffffffffffffffffffffffffffffffff16600260008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614614531576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303500000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600260008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16600260008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff16021790555080600260008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055506000600260008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508173ffffffffffffffffffffffffffffffffffffffff167ff8d49fc529812e9a7c5c50e69c20f0dccc0db8fa95c98bc58cc9a4f1c1299eaf60405160405180910390a28073ffffffffffffffffffffffffffffffffffffffff167f9465fa0c962cc76958e6373a993326400c1c94f8be2fe3a952adfa7f60b2ea2660405160405180910390a2505050565b6000600454905090565b606060007fbb8310d486368db6bd6f849402fdd73ad53d316b5a4b2644ad6efe0f941286d860001b8d8d8d8d60405180838380828437808301925050509250505060405180910390208c8c8c8c8c8c8c604051602001808c81526020018b73ffffffffffffffffffffffffffffffffffffffff1681526020018a815260200189815260200188600181111561483157fe5b81526020018781526020018681526020018581526020018473ffffffffffffffffffffffffffffffffffffffff1681526020018373ffffffffffffffffffffffffffffffffffffffff1681526020018281526020019b505050505050505050505050604051602081830303815290604052805190602001209050601960f81b600160f81b6148bd61499f565b8360405160200180857effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff19168152600101847effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff191681526001018381526020018281526020019450505050506040516020818303038152906040529150509b9a5050505050505050505050565b614950614e7f565b61495981615767565b8073ffffffffffffffffffffffffffffffffffffffff167f5ac6c46c93c8d0e53714ba3b53db3e7c046da994313d7ed0d192028bc7c228b060405160405180910390a250565b60007f47e79534a245952e8b16893a336b85a3d9ea9fa8c573f3d803afb92a7946921860001b6149cd61257a565b30604051602001808481526020018381526020018273ffffffffffffffffffffffffffffffffffffffff168152602001935050505060405160208183030381529060405280519060200120905090565b614a25614e7f565b806001600354031015614aa0576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614158015614b0a5750600173ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614155b614b7c576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303300000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b8173ffffffffffffffffffffffffffffffffffffffff16600260008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614614c7c576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303500000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600260008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16600260008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055506000600260008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff160217905550600360008154809291906001900391905055508173ffffffffffffffffffffffffffffffffffffffff167ff8d49fc529812e9a7c5c50e69c20f0dccc0db8fa95c98bc58cc9a4f1c1299eaf60405160405180910390a28060045414614e4157614e4081612bea565b5b505050565b6040518060400160405280600581526020017f312e342e3100000000000000000000000000000000000000000000000000000081525081565b3073ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff1614614f20576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330333100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b565b600080831415614f355760009050614f56565b6000828402905082848281614f4657fe5b0414614f5157600080fd5b809150505b92915050565b60008060008360410260208101860151925060408101860151915060ff60418201870151169350509250925092565b600080828401905083811015614fa057600080fd5b8091505092915050565b6000600180811115614fb857fe5b836001811115614fc457fe5b1415614fdd576000808551602087018986f49050614fed565b600080855160208701888a87f190505b95945050505050565b6000807f4a204f620c8c5ccdca3fd54d003badd85ba500436a431f0cbda4f558c93c34c860001b9050805491505090565b6000818310156150375781615039565b825b905092915050565b60008282111561505057600080fd5b600082840390508091505092915050565b600080600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff161461509e57826150a0565b325b9050600073ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff1614156151b85761510a3a86106150e7573a6150e9565b855b6150fc888a614f8b90919063ffffffff16565b614f2290919063ffffffff16565b91508073ffffffffffffffffffffffffffffffffffffffff166108fc839081150290604051600060405180830381858888f193505050506151b3576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330313100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b61525d565b6151dd856151cf888a614f8b90919063ffffffff16565b614f2290919063ffffffff16565b91506151ea848284615b0e565b61525c576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330313200000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b5b5095945050505050565b6000600454146152df576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303000000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b8151811115615356576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303100000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60018110156153cd576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303200000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60006001905060005b83518110156156d35760008482815181106153ed57fe5b60200260200101519050600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16141580156154615750600173ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614155b801561549957503073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614155b80156154d157508073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff1614155b615543576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303300000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff16600260008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614615644576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475332303400000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b80600260008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508092505080806001019150506153d6565b506001600260008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff160217905550825160038190555081600481905550505050565b3073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff161415615809576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475334303000000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b60007f6c9a6c4a39284e37ed1cf53d337577d14212a4870fb976a4366c693b939918d560001b90508181555050565b600073ffffffffffffffffffffffffffffffffffffffff1660016000600173ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff161461593a576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475331303000000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b6001806000600173ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff160217905550600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614615b0a576159f682615bd2565b615a68576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330303200000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b615a978260008360017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff614faa565b615b09576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260058152602001807f475330303000000000000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b5b5050565b60008063a9059cbb8484604051602401808373ffffffffffffffffffffffffffffffffffffffff168152602001828152602001925050506040516020818303038152906040529060e01b6020820180517bffffffffffffffffffffffffffffffffffffffffffffffffffffffff83818316178352505050509050602060008251602084016000896127105a03f13d60008114615bb55760208114615bbd5760009350615bc8565b819350615bc8565b600051158215171593505b5050509392505050565b600080823b90506000811191505091905056fea264697066735822122057398fa72884cf9a6cb78aab2fb58a6b927f0e9d97d75b015daaee0959a153bf64736f6c63430007060033")
        );
            // 5. Safe multisend:
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
        let anvil = hopr_chain_types::utils::create_anvil(None);
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
        let anvil = hopr_chain_types::utils::create_anvil(None);
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
            .grantRole(encoded_minter_role, contract_deployer.public().to_address().into())
            .send()
            .await?
            .watch()
            .await?;

        // test the deployer has minter role now
        let check_minter_role = instances
            .token
            .hasRole(encoded_minter_role, contract_deployer.public().to_address().into())
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
        let anvil = hopr_chain_types::utils::create_anvil(None);
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
            .grantRole(encoded_minter_role, contract_deployer.public().to_address().into())
            .send()
            .await?
            .watch()
            .await?;

        // test the deployer has minter role now
        let check_minter_role = instances
            .token
            .hasRole(encoded_minter_role, contract_deployer.public().to_address().into())
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
        let anvil = hopr_chain_types::utils::create_anvil(None);
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
        let anvil = hopr_chain_types::utils::create_anvil(None);
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
    async fn test_register_safes_and_nodes_then_deregister_nodes_in_anvil_with_multicall() -> anyhow::Result<()> {
        let mut safe_addresses: Vec<Address> = Vec::new();
        let mut node_addresses: Vec<Address> = Vec::new();
        for _ in 0..4 {
            safe_addresses.push(get_random_address_for_testing());
            node_addresses.push(get_random_address_for_testing());
        }

        // launch local anvil instance
        let anvil = hopr_chain_types::utils::create_anvil(None);
        let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let client = create_rpc_client_to_anvil(&anvil, &contract_deployer);
        let instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
            .await
            .expect("failed to deploy");
        // deploy multicall contract
        deploy_multicall3_for_testing(client.clone()).await?;

        // register some nodes
        let (removed_amt, added_amt) = register_safes_and_nodes_on_network_registry(
            instances.network_registry.clone(),
            safe_addresses.clone(),
            node_addresses.clone(),
        )
        .await?;

        assert_eq!(removed_amt, 0, "should not remove any safe");
        assert_eq!(added_amt, 4, "there should be 4 additions");

        // get registered safes from nodes
        let registered_safes = get_registered_safes_for_nodes_on_network_registry(
            instances.network_registry.clone(),
            node_addresses.clone(),
        )
        .await?;

        assert_eq!(safe_addresses.len(), registered_safes.len(), "returned length unmatch");
        for (i, safe_addr) in safe_addresses.iter().enumerate() {
            assert_eq!(safe_addr, &registered_safes[i], "registered safe addresses unmatch");
        }

        // deregister 3 of them
        let deregisterd_nodes = deregister_nodes_from_network_registry(
            instances.network_registry.clone(),
            node_addresses.split_at(3).0.to_vec(),
        )
        .await?;
        assert_eq!(deregisterd_nodes, 3, "cannot deregister all the nodes");

        // re-register 4 of them
        let (removed_amt_2, added_amt_2) = register_safes_and_nodes_on_network_registry(
            instances.network_registry.clone(),
            safe_addresses.clone(),
            node_addresses.clone(),
        )
        .await?;

        assert_eq!(removed_amt_2, 0, "should not remove any safe");
        assert_eq!(added_amt_2, 3, "there should be 3 additions");
        Ok(())
    }

    #[tokio::test]
    async fn test_deploy_proxy() -> anyhow::Result<()> {
        let prediction = deploy_proxy(
            address!("41675c099f32341bf84bfc5382af534df5c7461a"),
            hex!("09e458584ce79e57b65cb303dc136c5d53e17b676599b9b7bc03815e0eef5172"),
            Address::from_str(SAFE_SAFEPROXYFACTORY_ADDRESS)?,
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
        // testing value extracted from https://dashboard.tenderly.co/tx/xdai/0x510e3ac3dc7939cae2525a0b0f096ad709b23d94169e0fbf2e1154fdd6911c49?trace=0
        let _ = env_logger::builder().is_test(true).try_init();

        // prepare some input data
        let mut admin_addresses: Vec<Address> = Vec::new();
        for _ in 0..2 {
            admin_addresses.push(get_random_address_for_testing());
        }

        // launch local anvil instance
        let anvil = hopr_chain_types::utils::create_anvil(None);
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

        let module_address = predict_module_address(
            caller,
            nonce,
            *instances.stake_factory.address(),
            *instances.module_implementation.address(),
        )?;
        debug!("predict_module_addr {:?}", module_address);

        let safe_address = predict_safe_address(
            *instances.stake_factory.address(),
            vec![address!("9062a96b9e947b2b57283e705c97b871f5eb6561")],
            nonce,
            Address::from_str(SAFE_COMPATIBILITYFALLBACKHANDLER_ADDRESS)?,
            Address::from_str(SAFE_SAFE_ADDRESS)?,
            Address::from_str(SAFE_SAFEPROXYFACTORY_ADDRESS)?,
        )?;

        debug!("predict_safe_address {:?}", safe_address);

        // deploy a safe proxy instance and a module proxy instance with multicall as an owner
        let deployment_receipt = instances
            .stake_factory
            .clone(
                *instances.module_implementation.address(),
                vec![caller],
                nonce.into(),
                U256::from_str(&default_target)?.into(),
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
            .ok_or_else(|| anyhow::anyhow!("Module log not found"))?;

        let module_addr = module_log.instance;
        let safe_addr = safe_log.instance;

        assert_eq!(module_addr, module_address, "module prediction does not match");
        assert_eq!(safe_addr, safe_address, "safe prediction does not match");
        Ok(())
    }

    #[tokio::test]
    async fn test_deploy_safe_and_module() -> anyhow::Result<()> {
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
        let anvil = hopr_chain_types::utils::create_anvil(None);
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
            *instances.token.address(),
            *instances.channels.address(),
            *instances.module_implementation.address(),
            *instances.announcements.address(),
            U256::MAX,
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

        assert_eq!(allowance, U256::MAX, "allowance is not set");

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
        let anvil = hopr_chain_types::utils::create_anvil(None);
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
            *instances.token.address(),
            *instances.channels.address(),
            *instances.module_implementation.address(),
            *instances.announcements.address(),
            U256::MAX,
            None,
            vec![contract_deployer.public().to_address().into()],
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

        assert_eq!(allowance, U256::MAX, "allowance initiation is wrong");

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
            Address::from_str(SAFE_MULTISEND_ADDRESS)?,
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
        let anvil = hopr_chain_types::utils::create_anvil(None);
        let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let client = create_rpc_client_to_anvil(&anvil, &contract_deployer);
        let instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
            .await
            .expect("failed to deploy");
        // deploy multicall contract
        deploy_multicall3_for_testing(client.clone()).await?;
        // deploy safe suits
        deploy_safe_suites(client.clone()).await?;

        let deployer_vec: Vec<Address> = vec![contract_deployer.public().to_address().into()];

        // create a safe
        let (safe, node_module) = deploy_safe_module_with_targets_and_nodes(
            instances.stake_factory,
            *instances.token.address(),
            *instances.channels.address(),
            *instances.module_implementation.address(),
            *instances.announcements.address(),
            U256::MAX,
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
            .isNode(contract_deployer.public().to_address().into())
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
        let anvil = hopr_chain_types::utils::create_anvil(None);
        let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let client = create_rpc_client_to_anvil(&anvil, &contract_deployer);
        let instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
            .await
            .expect("failed to deploy");
        // deploy multicall contract
        deploy_multicall3_for_testing(client.clone()).await?;
        // deploy safe suits
        deploy_safe_suites(client.clone()).await?;

        let deployer_vec: Vec<Address> = vec![contract_deployer.public().to_address().into()];

        // create a safe
        let (safe, node_module) = deploy_safe_module_with_targets_and_nodes(
            instances.stake_factory,
            *instances.token.address(),
            *instances.channels.address(),
            *instances.module_implementation.address(),
            *instances.announcements.address(),
            U256::MAX,
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
        let anvil = hopr_chain_types::utils::create_anvil(None);
        let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let self_address: Address = contract_deployer.public().to_address().into();
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
        let new_announcements = HoprAnnouncements::deploy(client.clone(), *new_safe_registry.address()).await?;
        let _new_network_registry = HoprNetworkRegistry::deploy(
            client.clone(),
            instances.network_registry_proxy.address().into(),
            self_address,
            self_address,
        )
        .await?;

        let deployer_vec: Vec<Address> = vec![self_address];

        // create a safe
        let (safe, node_module) = deploy_safe_module_with_targets_and_nodes(
            instances.stake_factory,
            *instances.token.address(),
            *instances.channels.address(),
            *instances.module_implementation.address(),
            *instances.announcements.address(),
            U256::MAX,
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
    async fn test_debug_node_safe_module_setup_on_balance_and_registries() -> anyhow::Result<()> {
        // set allowance for token transfer for the safe multiple times
        let _ = env_logger::builder().is_test(true).try_init();

        let mut node_addresses: Vec<Address> = Vec::new();
        for _ in 0..2 {
            node_addresses.push(get_random_address_for_testing());
        }

        // launch local anvil instance
        let anvil = hopr_chain_types::utils::create_anvil(None);
        let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let client = create_rpc_client_to_anvil(&anvil, &contract_deployer);
        let instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
            .await
            .expect("failed to deploy");
        // deploy multicall contract
        deploy_multicall3_for_testing(client.clone()).await?;
        // deploy safe suits
        deploy_safe_suites(client.clone()).await?;

        let deployer_vec: Vec<Address> = vec![contract_deployer.public().to_address().into()];

        // create a safe
        let (safe, _) = deploy_safe_module_with_targets_and_nodes(
            instances.stake_factory,
            *instances.token.address(),
            *instances.channels.address(),
            *instances.module_implementation.address(),
            *instances.announcements.address(),
            U256::MAX,
            None,
            deployer_vec.clone(),
            U256::from(1),
        )
        .await?;
        let registered_safe_before_registration = debug_node_safe_module_setup_on_balance_and_registries(
            instances.network_registry.clone(),
            instances.safe_registry.clone(),
            &node_addresses[0],
        )
        .await?;

        assert_eq!(
            registered_safe_before_registration,
            Address::ZERO,
            "safe is already registered"
        );

        // register some nodes
        let (..) = register_safes_and_nodes_on_network_registry(
            instances.network_registry.clone(),
            vec![*safe.address()],
            vec![node_addresses[0]],
        )
        .await?;

        let registered_safe_after_registration = debug_node_safe_module_setup_on_balance_and_registries(
            instances.network_registry.clone(),
            instances.safe_registry.clone(),
            &node_addresses[0],
        )
        .await?;

        assert_eq!(
            &registered_safe_after_registration,
            safe.address(),
            "safe is not registered"
        );
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
        let anvil = hopr_chain_types::utils::create_anvil(None);
        let contract_deployer = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let client = create_rpc_client_to_anvil(&anvil, &contract_deployer);
        let instances = ContractInstances::deploy_for_testing(client.clone(), &contract_deployer)
            .await
            .expect("failed to deploy");
        // deploy multicall contract
        deploy_multicall3_for_testing(client.clone()).await?;
        // deploy safe suits
        deploy_safe_suites(client.clone()).await?;

        let deployer_vec: Vec<Address> = vec![contract_deployer.public().to_address().into()];

        // create a safe
        let (safe, node_module) = deploy_safe_module_with_targets_and_nodes(
            instances.stake_factory,
            *instances.token.address(),
            *instances.channels.address(),
            *instances.module_implementation.address(),
            *instances.announcements.address(),
            U256::MAX,
            None,
            deployer_vec.clone(),
            U256::from(1),
        )
        .await?;

        // register some nodes
        let (..) = register_safes_and_nodes_on_network_registry(
            instances.network_registry.clone(),
            vec![*safe.address()],
            vec![node_addresses[0]],
        )
        .await?;

        debug_node_safe_module_setup_main(
            instances.token.clone(),
            node_module.address(),
            &node_addresses[0],
            safe.address(),
            instances.channels.address(),
            instances.announcements.address(),
        )
        .await?;
        Ok(())
    }
}
