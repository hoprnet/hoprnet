//! This module contains errors produced in this crate
// use ethers::providers::{Http, JsonRpcClient, ProviderError};
use hoprd_keypair::errors::KeyPairError;
use thiserror::Error;

pub trait Cmd: clap::Parser + Sized {
    fn run(self) -> Result<(), HelperErrors>;
    fn async_run(self) -> impl std::future::Future<Output = Result<(), HelperErrors>> + Send;
}

/// Enumerates different errors produced by this crate.
#[derive(Error, Debug)]
pub enum HelperErrors {
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
    UnableToReadPrivateKey(#[from] std::env::VarError),

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
    RpcError(#[from] chain_rpc::errors::RpcError),

    /// Error with signer wallet error
    #[error(transparent)]
    WalletError(#[from] ethers::signers::WalletError),

    /// Fail to make a multicall
    #[error("multicall Error: {0}")]
    MulticallError(String),

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

    // encode packed error
    #[error(transparent)]
    EncodePackedError(#[from] ethers::abi::EncodePackedError),
}

/// Multicall3 deployer wallet
pub const MULTICALL3_DEPLOYER: &str = "05f32b3cc3888453ff71b01135b34ff8e41263f2";

/// Amount of ETH to fund MULTICALL3_DEPLOYER, to deployer Multicall3 contract
pub const ETH_VALUE_FOR_MULTICALL3_DEPLOYER: u128 = 100_000_000_000_000_000; // 0.1 (anvil) ETH

/// Contract creation code for mullticall3, as in <https://etherscan.io/address/0xcA11bde05977b3631167028862bE2a173976CA11>
pub const MULTICALL3_CONTRACT_CREATION_CODE: &str = "608060405234801561001057600080fd5b50610ee0806100206000396000f3fe6080604052600436106100f35760003560e01c80634d2301cc1161008a578063a8b0574e11610059578063a8b0574e1461025a578063bce38bd714610275578063c3077fa914610288578063ee82ac5e1461029b57600080fd5b80634d2301cc146101ec57806372425d9d1461022157806382ad56cb1461023457806386d516e81461024757600080fd5b80633408e470116100c65780633408e47014610191578063399542e9146101a45780633e64a696146101c657806342cbb15c146101d957600080fd5b80630f28c97d146100f8578063174dea711461011a578063252dba421461013a57806327e86d6e1461015b575b600080fd5b34801561010457600080fd5b50425b6040519081526020015b60405180910390f35b61012d610128366004610a85565b6102ba565b6040516101119190610bbe565b61014d610148366004610a85565b6104ef565b604051610111929190610bd8565b34801561016757600080fd5b50437fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0140610107565b34801561019d57600080fd5b5046610107565b6101b76101b2366004610c60565b610690565b60405161011193929190610cba565b3480156101d257600080fd5b5048610107565b3480156101e557600080fd5b5043610107565b3480156101f857600080fd5b50610107610207366004610ce2565b73ffffffffffffffffffffffffffffffffffffffff163190565b34801561022d57600080fd5b5044610107565b61012d610242366004610a85565b6106ab565b34801561025357600080fd5b5045610107565b34801561026657600080fd5b50604051418152602001610111565b61012d610283366004610c60565b61085a565b6101b7610296366004610a85565b610a1a565b3480156102a757600080fd5b506101076102b6366004610d18565b4090565b60606000828067ffffffffffffffff8111156102d8576102d8610d31565b60405190808252806020026020018201604052801561031e57816020015b6040805180820190915260008152606060208201528152602001906001900390816102f65790505b5092503660005b8281101561047757600085828151811061034157610341610d60565b6020026020010151905087878381811061035d5761035d610d60565b905060200281019061036f9190610d8f565b6040810135958601959093506103886020850185610ce2565b73ffffffffffffffffffffffffffffffffffffffff16816103ac6060870187610dcd565b6040516103ba929190610e32565b60006040518083038185875af1925050503d80600081146103f7576040519150601f19603f3d011682016040523d82523d6000602084013e6103fc565b606091505b50602080850191909152901515808452908501351761046d577f08c379a000000000000000000000000000000000000000000000000000000000600052602060045260176024527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060445260846000fd5b5050600101610325565b508234146104e6576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601a60248201527f4d756c746963616c6c333a2076616c7565206d69736d6174636800000000000060448201526064015b60405180910390fd5b50505092915050565b436060828067ffffffffffffffff81111561050c5761050c610d31565b60405190808252806020026020018201604052801561053f57816020015b606081526020019060019003908161052a5790505b5091503660005b8281101561068657600087878381811061056257610562610d60565b90506020028101906105749190610e42565b92506105836020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff166105a66020850185610dcd565b6040516105b4929190610e32565b6000604051808303816000865af19150503d80600081146105f1576040519150601f19603f3d011682016040523d82523d6000602084013e6105f6565b606091505b5086848151811061060957610609610d60565b602090810291909101015290508061067d576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601760248201527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060448201526064016104dd565b50600101610546565b5050509250929050565b43804060606106a086868661085a565b905093509350939050565b6060818067ffffffffffffffff8111156106c7576106c7610d31565b60405190808252806020026020018201604052801561070d57816020015b6040805180820190915260008152606060208201528152602001906001900390816106e55790505b5091503660005b828110156104e657600084828151811061073057610730610d60565b6020026020010151905086868381811061074c5761074c610d60565b905060200281019061075e9190610e76565b925061076d6020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff166107906040850185610dcd565b60405161079e929190610e32565b6000604051808303816000865af19150503d80600081146107db576040519150601f19603f3d011682016040523d82523d6000602084013e6107e0565b606091505b506020808401919091529015158083529084013517610851577f08c379a000000000000000000000000000000000000000000000000000000000600052602060045260176024527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060445260646000fd5b50600101610714565b6060818067ffffffffffffffff81111561087657610876610d31565b6040519080825280602002602001820160405280156108bc57816020015b6040805180820190915260008152606060208201528152602001906001900390816108945790505b5091503660005b82811015610a105760008482815181106108df576108df610d60565b602002602001015190508686838181106108fb576108fb610d60565b905060200281019061090d9190610e42565b925061091c6020840184610ce2565b73ffffffffffffffffffffffffffffffffffffffff1661093f6020850185610dcd565b60405161094d929190610e32565b6000604051808303816000865af19150503d806000811461098a576040519150601f19603f3d011682016040523d82523d6000602084013e61098f565b606091505b506020830152151581528715610a07578051610a07576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601760248201527f4d756c746963616c6c333a2063616c6c206661696c656400000000000000000060448201526064016104dd565b506001016108c3565b5050509392505050565b6000806060610a2b60018686610690565b919790965090945092505050565b60008083601f840112610a4b57600080fd5b50813567ffffffffffffffff811115610a6357600080fd5b6020830191508360208260051b8501011115610a7e57600080fd5b9250929050565b60008060208385031215610a9857600080fd5b823567ffffffffffffffff811115610aaf57600080fd5b610abb85828601610a39565b90969095509350505050565b6000815180845260005b81811015610aed57602081850181015186830182015201610ad1565b81811115610aff576000602083870101525b50601f017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0169290920160200192915050565b600082825180855260208086019550808260051b84010181860160005b84811015610bb1578583037fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe001895281518051151584528401516040858501819052610b9d81860183610ac7565b9a86019a9450505090830190600101610b4f565b5090979650505050505050565b602081526000610bd16020830184610b32565b9392505050565b600060408201848352602060408185015281855180845260608601915060608160051b870101935082870160005b82811015610c52577fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa0888703018452610c40868351610ac7565b95509284019290840190600101610c06565b509398975050505050505050565b600080600060408486031215610c7557600080fd5b83358015158114610c8557600080fd5b9250602084013567ffffffffffffffff811115610ca157600080fd5b610cad86828701610a39565b9497909650939450505050565b838152826020820152606060408201526000610cd96060830184610b32565b95945050505050565b600060208284031215610cf457600080fd5b813573ffffffffffffffffffffffffffffffffffffffff81168114610bd157600080fd5b600060208284031215610d2a57600080fd5b5035919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b7f4e487b7100000000000000000000000000000000000000000000000000000000600052603260045260246000fd5b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff81833603018112610dc357600080fd5b9190910192915050565b60008083357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe1843603018112610e0257600080fd5b83018035915067ffffffffffffffff821115610e1d57600080fd5b602001915036819003821315610a7e57600080fd5b8183823760009101908152919050565b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc1833603018112610dc357600080fd5b600082357fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa1833603018112610dc357600080fdfea2646970667358221220bb2b5c71a328032f97c676ae39a1ec2148d3e5d6f73d95e9b17910152d61f16264736f6c634300080c0033";

/// Default capability permissions, as in smart contract
/// ```text
///     [
///       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultRedeemTicketSafeFunctionPermisson
///       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // RESERVED
///       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultCloseIncomingChannelSafeFunctionPermisson
///       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultInitiateOutgoingChannelClosureSafeFunctionPermisson
///       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultFinalizeOutgoingChannelClosureSafeFunctionPermisson
///       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultFundChannelMultiFunctionPermisson
///       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultSetCommitmentSafeFunctionPermisson
///       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultApproveFunctionPermisson
///       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW  // defaultSendFunctionPermisson
///     ]
/// ```
pub const DEFAULT_CAPABILITY_PERMISSIONS: &str = "010103030303030303030303";

/// Default announcement permissions, where nothing is specified and falls back to the default, as in smart contract
/// ```text
///     [
///       CapabilityPermission.NONE, // defaultRedeemTicketSafeFunctionPermisson
///       CapabilityPermission.NONE, // RESERVED
///       CapabilityPermission.NONE, // defaultCloseIncomingChannelSafeFunctionPermisson
///       CapabilityPermission.NONE, // defaultInitiateOutgoingChannelClosureSafeFunctionPermisson
///       CapabilityPermission.NONE, // defaultFinalizeOutgoingChannelClosureSafeFunctionPermisson
///       CapabilityPermission.NONE, // defaultFundChannelMultiFunctionPermisson
///       CapabilityPermission.NONE, // defaultSetCommitmentSafeFunctionPermisson
///       CapabilityPermission.NONE, // defaultApproveFunctionPermisson
///       CapabilityPermission.NONE  // defaultSendFunctionPermisson
///     ]
/// ```
pub const DEFAULT_ANNOUNCEMENT_PERMISSIONS: &str = "010003000000000000000000";

/// Default node permissions, where nothing is specified and falls back to the default, as in smart contract
/// ```text
///     [
///       CapabilityPermission.NONE, // defaultRedeemTicketSafeFunctionPermisson
///       CapabilityPermission.NONE, // RESERVED
///       CapabilityPermission.NONE, // defaultCloseIncomingChannelSafeFunctionPermisson
///       CapabilityPermission.NONE, // defaultInitiateOutgoingChannelClosureSafeFunctionPermisson
///       CapabilityPermission.NONE, // defaultFinalizeOutgoingChannelClosureSafeFunctionPermisson
///       CapabilityPermission.NONE, // defaultFundChannelMultiFunctionPermisson
///       CapabilityPermission.NONE, // defaultSetCommitmentSafeFunctionPermisson
///       CapabilityPermission.NONE, // defaultApproveFunctionPermisson
///       CapabilityPermission.NONE  // defaultSendFunctionPermisson
///     ]
/// ```
pub const DEFAULT_NODE_PERMISSIONS: &str = "010303000000000000000000";

/// Safe compatibility fallback handler contract deployed for v1.4.0, as in <https://github.com/safe-global/safe-deployments/blob/b707e5e2994e6f86d76ff7ffade0445c4e49ae9a/src/assets/v1.4.0/compatibility_fallback_handler.json>
pub const SAFE_COMPATIBILITYFALLBACKHANDLER_ADDRESS: &str = "2a15DE4410d4c8af0A7b6c12803120f43C42B820";
/// Safe implementation contract deployed for v1.4.1, as in <https://github.com/safe-global/safe-deployments/blob/8c504f44d148f4c898fee02749c88372bae6609a/src/assets/v1.4.1/safe.json>
pub const SAFE_SAFE_ADDRESS: &str = "41675C099F32341bf84BFc5382aF534df5C7461a";
/// Safe proxy factory contract deployed for v1.4.0, <https://github.com/safe-global/safe-deployments/blob/b707e5e2994e6f86d76ff7ffade0445c4e49ae9a/src/assets/v1.4.0/safe_proxy_factory.json>
pub const SAFE_SAFEPROXYFACTORY_ADDRESS: &str = "4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67";
/// Safe multisend contract deployed for v1.4.0,  <https://github.com/safe-global/safe-deployments/blob/b707e5e2994e6f86d76ff7ffade0445c4e49ae9a/src/assets/v1.4.0/multi_send.json>
pub const SAFE_MULTISEND_ADDRESS: &str = "38869bf66a61cF6bDB996A6aE40D5853Fd43B526";

/// Topic hash for `NewHoprNodeStakeModule` event
pub const NEW_HOPR_NODE_STAKE_MODULE_TOPIC: &str = "0x813d391dc490d6c1dae7d3fdd555f337533d1da2c908c6efd36d4cf557a63206";
/// Topic hash for `NewHoprNodeStakeSafe` event
pub const NEW_HOPR_NODE_STAKE_SAFE_TOPIC: &str = "0x8231d169f416b666ae7fa43faa24a18899738075a53f32c97617d173b189e386";

/// Starting point for a linked list in safe contract, as in <https://github.com/safe-global/safe-smart-account/blob/2278f7ccd502878feb5cec21dd6255b82df374b5/contracts/base/OwnerManager.sol#L14>
pub const SENTINEL_OWNERS: &str = "0x0000000000000000000000000000000000000001";
/// Safe transaction type hash, as in <https://github.com/safe-global/safe-smart-account/blob/2278f7ccd502878feb5cec21dd6255b82df374b5/contracts/Safe.sol#L59>
pub const SAFE_TX_TYPEHASH: &str = "bb8310d486368db6bd6f849402fdd73ad53d316b5a4b2644ad6efe0f941286d8";

/// Safe domain separator typehash, as in <https://github.com/safe-global/safe-smart-account/blob/2278f7ccd502878feb5cec21dd6255b82df374b5/contracts/Safe.sol#L54>
pub const DOMAIN_SEPARATOR_TYPEHASH: &str = "47e79534a245952e8b16893a336b85a3d9ea9fa8c573f3d803afb92a79469218";

/// Topic hash for `ExecutionSuccess` event, as in <https://github.com/safe-global/safe-smart-account/blob/2278f7ccd502878feb5cec21dd6255b82df374b5/contracts/interfaces/ISafe.sol#L18>
pub const SAFE_EXECUTION_SUCCESS: &str = "0x442e715f626346e8c54381002da614f62bee8d27386535b2521ec8540898556e";
