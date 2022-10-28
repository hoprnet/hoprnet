pub use hopr_whitehat::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod hopr_whitehat {
    #![allow(clippy::enum_variant_names)]
    #![allow(dead_code)]
    #![allow(clippy::type_complexity)]
    #![allow(unused_imports)]
    use ethers::contract::{
        builders::{ContractCall, Event},
        Contract, Lazy,
    };
    use ethers::core::{
        abi::{Abi, Detokenize, InvalidOutputType, Token, Tokenizable},
        types::*,
    };
    use ethers::providers::Middleware;
    #[doc = "HoprWhitehat was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_newOwner\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"_myHoprBoost\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"_myHoprStake\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"_xHopr\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"_wxHopr\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"contractAddress\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"Called777Hook\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"contractAddress\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"Called777HookForFunding\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"previousOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"OwnershipTransferred\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"contractAddress\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"Received677\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"tokenId\",\"type\":\"uint256\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"ReclaimedBoost\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"entitledReward\",\"type\":\"uint256\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"RequestedGimme\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"activate\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"interfaceHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"canImplementInterfaceForAddress\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"currentCaller\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"deactivate\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"gimmeToken\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"staker\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"gimmeTokenFor\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"isActive\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"myHoprBoost\",\"outputs\":[{\"internalType\":\"contract HoprBoost\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"myHoprStake\",\"outputs\":[{\"internalType\":\"contract HoprStake\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"tokenId\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"onERC721Received\",\"outputs\":[{\"internalType\":\"bytes4\",\"name\":\"\",\"type\":\"bytes4\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_from\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"_value\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"_data\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"onTokenTransfer\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"owner\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"stakerAddress\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"tokenId\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"ownerRescueBoosterNft\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"stakerAddress\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"ownerRescueBoosterNftInBatch\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"tokenAddress\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"reclaimErc20Tokens\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"tokenAddress\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"tokenId\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"reclaimErc721Tokens\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"renounceOwnership\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"rescuedXHoprAmount\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"userData\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"tokensReceived\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"multisig\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferBackOwnership\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferOwnership\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"wxHopr\",\"outputs\":[{\"internalType\":\"contract ERC777Mock\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"xHopr\",\"outputs\":[{\"internalType\":\"contract ERC677Mock\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static HOPRWHITEHAT_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static HOPRWHITEHAT_BYTECODE: ethers::contract::Lazy<ethers::core::types::Bytes> =
        ethers::contract::Lazy::new(|| {
            "0x6080604052600580546001600160a01b03199081167343d13d7b83607f14335cf2cb75e87da369d056c71790915560068054821673912f4d6607160256787a2ad40da098ac2afe57ac17905560078054821673d4fdec44db9d44b8f2b6d529620f9c0c7066a2c11790556008805490911673d057604a14982fe8d88c5fc25aac3267ea142a081790553480156200009557600080fd5b50604051620021e1380380620021e1833981016040819052620000b8916200031c565b620000c333620001da565b600160025546606481146200011d57600580546001600160a01b038088166001600160a01b031992831617909255600680548784169083161790556008805486841690831617905560078054928516929091169190911790555b6003805460ff60a01b191690556040516329965a1d60e01b815230600482018190527fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b60248301526044820152731820a4b7618bde71dce8cdc73aab6c95905fad24906329965a1d90606401600060405180830381600087803b158015620001a457600080fd5b505af1158015620001b9573d6000803e3d6000fd5b50505050620001ce866200022a60201b60201c565b5050505050506200038c565b600080546001600160a01b038381166001600160a01b0319831681178455604051919092169283917f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09190a35050565b6000546001600160a01b031633146200028a5760405162461bcd60e51b815260206004820181905260248201527f4f776e61626c653a2063616c6c6572206973206e6f7420746865206f776e657260448201526064015b60405180910390fd5b6001600160a01b038116620002f15760405162461bcd60e51b815260206004820152602660248201527f4f776e61626c653a206e6577206f776e657220697320746865207a65726f206160448201526564647265737360d01b606482015260840162000281565b620002fc81620001da565b50565b80516001600160a01b03811681146200031757600080fd5b919050565b600080600080600060a086880312156200033557600080fd5b6200034086620002ff565b94506200035060208701620002ff565b93506200036060408701620002ff565b92506200037060608701620002ff565b91506200038060808701620002ff565b90509295509295909350565b611e45806200039c6000396000f3fe608060405234801561001057600080fd5b506004361061014c5760003560e01c80636067bc15116100c3578063a188cbbc1161007c578063a188cbbc146102c9578063a4c0ed36146102dc578063b7955631146102ef578063bb04fb69146102f8578063de9118c51461030b578063f2fde38b1461031e57600080fd5b80636067bc1514610264578063715018a614610277578063770f07e21461027f5780638da5cb5b1461029257806393fd97c9146102a35780639b011c6f146102b657600080fd5b806322f3e2d41161011557806322f3e2d4146101de578063249cb3fa146102025780632dcdad7d14610223578063314503d21461023657806348c64e411461024957806351b42b001461025c57600080fd5b806223de2914610151578063058e7326146101665780630d8dbd4c146101965780630f15f4c01461019e578063150b7a02146101a6575b600080fd5b61016461015f366004611918565b610331565b005b600654610179906001600160a01b031681565b6040516001600160a01b0390911681526020015b60405180910390f35b610164610600565b610164610a13565b6101c56101b43660046119c9565b630a85bd0160e11b95945050505050565b6040516001600160e01b0319909116815260200161018d565b6003546101f290600160a01b900460ff1681565b604051901515815260200161018d565b610215610210366004611a3c565b610aac565b60405190815260200161018d565b600754610179906001600160a01b031681565b600354610179906001600160a01b031681565b610164610257366004611a6c565b610b05565b610164610bbd565b610164610272366004611a98565b610c5a565b610164610d1b565b61016461028d366004611a98565b610d51565b6000546001600160a01b0316610179565b6101646102b1366004611a6c565b610f92565b6101646102c4366004611a98565b611099565b600854610179906001600160a01b031681565b6101f26102ea366004611acb565b611438565b61021560045481565b610164610306366004611a98565b6114a7565b600554610179906001600160a01b031681565b61016461032c366004611a98565b611533565b600354600160a01b900460ff16156105f6576007546001600160a01b031633146103a25760405162461bcd60e51b815260206004820152601e60248201527f63616e206f6e6c792062652063616c6c65642066726f6d207778484f5052000060448201526064015b60405180910390fd5b6006546001600160a01b03908116908816036105bd576003546001600160a01b038781169116146104315760405162461bcd60e51b815260206004820152603360248201527f6d7573742073656e642045524337373720746f6b656e7320746f20746865206360448201527230b63632b91037b31033b4b6b6b2aa37b5b2b760691b6064820152608401610399565b60405185906001600160a01b0389169033907fa0b858f3caaa729015e5994b98e49454e2e751fc55022580ca2e9cbd92d3d6a690600090a4600660009054906101000a90046001600160a01b03166001600160a01b031663568914126040518163ffffffff1660e01b8152600401602060405180830381865afa1580156104bc573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906104e09190611b98565b6008546006546040516370a0823160e01b81526001600160a01b0391821660048201529116906370a0823190602401602060405180830381865afa15801561052c573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906105509190611b98565b11156105b857600654600854604051636067bc1560e01b81526001600160a01b039182166004820152911690636067bc1590602401600060405180830381600087803b15801561059f57600080fd5b505af11580156105b3573d6000803e3d6000fd5b505050505b6105f6565b60405185906001600160a01b0389169033907fd360e50741bd90efe8c6807dbba710bdde93731888dd3c3affffe933f2164d3f90600090a45b5050505050505050565b60028054036106515760405162461bcd60e51b815260206004820152601f60248201527f5265656e7472616e637947756172643a207265656e7472616e742063616c6c006044820152606401610399565b60028055600354600160a01b900460ff166106a75760405162461bcd60e51b81526020600482015260166024820152755768697465686174206973206e6f742061637469766560501b6044820152606401610399565b60065460408051638da5cb5b60e01b8152905130926001600160a01b031691638da5cb5b9160048083019260209291908290030181865afa1580156106f0573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906107149190611bb1565b6001600160a01b03161461073a5760405162461bcd60e51b815260040161039990611bce565b60405163555ddc6560e11b81523360048201527fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b60248201523090731820a4b7618bde71dce8cdc73aab6c95905fad249063aabbb8ca90604401602060405180830381865afa1580156107b1573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906107d59190611bb1565b6001600160a01b0316146107fb5760405162461bcd60e51b815260040161039990611c13565b600380546001600160a01b03191633908117909155600654604051632961046560e21b815260048101929092526001600160a01b03169063a584119490602401600060405180830381600087803b15801561085557600080fd5b505af1158015610869573d6000803e3d6000fd5b5050600654600354604051632f2e037160e11b81526001600160a01b03918216600482015260009450849350839283928392911690635e5c06e29060240160a060405180830381865afa1580156108c4573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906108e89190611c67565b94509450945094509450600081836109009190611cbd565b60035460405191925082916001600160a01b03909116907f9ea37a571f2bde6d09e9bded2d6df972a77cf44b91b6dee9e00e60e2221b9fd390600090a3600754600654604051634decdde360e11b81526001600160a01b0392831692639bd9bbc692610973929116908590600401611cd4565b600060405180830381600087803b15801561098d57600080fd5b505af11580156109a1573d6000803e3d6000fd5b5050600654600354604051630bdb124f60e21b81526001600160a01b03918216600482015291169250632f6c493c9150602401600060405180830381600087803b1580156109ee57600080fd5b505af1158015610a02573d6000803e3d6000fd5b505060016002555050505050505050565b6000546001600160a01b03163314610a3d5760405162461bcd60e51b815260040161039990611d08565b600354600160a01b900460ff1615610a975760405162461bcd60e51b815260206004820152601e60248201527f486f7072576869746568617420697320616c72656164792061637469766500006044820152606401610399565b6003805460ff60a01b1916600160a01b179055565b60007fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b8314610adc576000610afe565b7fa2ef4600d742022d532d4747cb3547474667d6f13804902513b2ec01c848f4b45b9392505050565b6000546001600160a01b03163314610b2f5760405162461bcd60e51b815260040161039990611d08565b816001600160a01b03166323b872dd30610b516000546001600160a01b031690565b6040516001600160e01b031960e085901b1681526001600160a01b03928316600482015291166024820152604481018490526064015b600060405180830381600087803b158015610ba157600080fd5b505af1158015610bb5573d6000803e3d6000fd5b505050505050565b6000546001600160a01b03163314610be75760405162461bcd60e51b815260040161039990611d08565b600354600160a01b900460ff16610c4b5760405162461bcd60e51b815260206004820152602260248201527f486f7072576869746568617420697320616c7265616479206e6f742061637469604482015261766560f01b6064820152608401610399565b6003805460ff60a01b19169055565b6000546001600160a01b03163314610c845760405162461bcd60e51b815260040161039990611d08565b6040516370a0823160e01b81523060048201526000906001600160a01b038316906370a0823190602401602060405180830381865afa158015610ccb573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610cef9190611b98565b9050610d17610d066000546001600160a01b031690565b6001600160a01b03841690836115ce565b5050565b6000546001600160a01b03163314610d455760405162461bcd60e51b815260040161039990611d08565b610d4f6000611620565b565b6000546001600160a01b03163314610d7b5760405162461bcd60e51b815260040161039990611d08565b60065460405163d0c02d6360e01b81526001600160a01b038381166004830152600092169063d0c02d6390602401602060405180830381865afa158015610dc6573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610dea9190611b98565b905060005b81811015610f8d5760065460405163050d12bd60e11b81526001600160a01b038581166004830152602482018490526000921690630a1a257a90604401602060405180830381865afa158015610e49573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610e6d9190611b98565b6006546005546040516348c64e4160e01b81526001600160a01b0391821660048201526024810184905292935016906348c64e4190604401600060405180830381600087803b158015610ebf57600080fd5b505af1158015610ed3573d6000803e3d6000fd5b50506040518392506001600160a01b03871691507f591d6a09dd56239a87549c1ea1b09c163e95e438042207ca0947730f169586f590600090a3600554604051632142170760e11b81523060048201526001600160a01b03868116602483015260448201849052909116906342842e0e90606401600060405180830381600087803b158015610f6157600080fd5b505af1158015610f75573d6000803e3d6000fd5b50505050508080610f8590611d3d565b915050610def565b505050565b6000546001600160a01b03163314610fbc5760405162461bcd60e51b815260040161039990611d08565b6006546005546040516348c64e4160e01b81526001600160a01b039182166004820152602481018490529116906348c64e4190604401600060405180830381600087803b15801561100c57600080fd5b505af1158015611020573d6000803e3d6000fd5b50506040518392506001600160a01b03851691507f591d6a09dd56239a87549c1ea1b09c163e95e438042207ca0947730f169586f590600090a3600554604051632142170760e11b81523060048201526001600160a01b03848116602483015260448201849052909116906342842e0e90606401610b87565b6000546001600160a01b031633146110c35760405162461bcd60e51b815260040161039990611d08565b60065460408051638da5cb5b60e01b8152905130926001600160a01b031691638da5cb5b9160048083019260209291908290030181865afa15801561110c573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906111309190611bb1565b6001600160a01b0316146111565760405162461bcd60e51b815260040161039990611bce565b60405163555ddc6560e11b81526001600160a01b03821660048201527fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b60248201523090731820a4b7618bde71dce8cdc73aab6c95905fad249063aabbb8ca90604401602060405180830381865afa1580156111d6573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906111fa9190611bb1565b6001600160a01b0316146112205760405162461bcd60e51b815260040161039990611c13565b600380546001600160a01b0319166001600160a01b03838116918217909255600654604051632961046560e21b815260048101929092529091169063a584119490602401600060405180830381600087803b15801561127e57600080fd5b505af1158015611292573d6000803e3d6000fd5b5050600654600354604051632f2e037160e11b81526001600160a01b03918216600482015260009450849350839283928392911690635e5c06e29060240160a060405180830381865afa1580156112ed573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906113119190611c67565b94509450945094509450600081836113299190611cbd565b60035460405191925082916001600160a01b03909116907f9ea37a571f2bde6d09e9bded2d6df972a77cf44b91b6dee9e00e60e2221b9fd390600090a3600754600654604051634decdde360e11b81526001600160a01b0392831692639bd9bbc69261139c929116908590600401611cd4565b600060405180830381600087803b1580156113b657600080fd5b505af11580156113ca573d6000803e3d6000fd5b5050600654600354604051630bdb124f60e21b81526001600160a01b03918216600482015291169250632f6c493c9150602401600060405180830381600087803b15801561141757600080fd5b505af115801561142b573d6000803e3d6000fd5b5050505050505050505050565b6008546000906001600160a01b0316330361146557826004600082825461145f9190611d56565b90915550505b60405183906001600160a01b0386169033907f350a7c854b33230aeadc9ca5c8e747896586e434150cb87c5505be09f3f3f99090600090a45060019392505050565b6000546001600160a01b031633146114d15760405162461bcd60e51b815260040161039990611d08565b60065460405163f2fde38b60e01b81526001600160a01b0383811660048301529091169063f2fde38b90602401600060405180830381600087803b15801561151857600080fd5b505af115801561152c573d6000803e3d6000fd5b5050505050565b6000546001600160a01b0316331461155d5760405162461bcd60e51b815260040161039990611d08565b6001600160a01b0381166115c25760405162461bcd60e51b815260206004820152602660248201527f4f776e61626c653a206e6577206f776e657220697320746865207a65726f206160448201526564647265737360d01b6064820152608401610399565b6115cb81611620565b50565b604080516001600160a01b038416602482015260448082018490528251808303909101815260649091019091526020810180516001600160e01b031663a9059cbb60e01b179052610f8d908490611670565b600080546001600160a01b038381166001600160a01b0319831681178455604051919092169283917f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09190a35050565b60006116c5826040518060400160405280602081526020017f5361666545524332303a206c6f772d6c6576656c2063616c6c206661696c6564815250856001600160a01b03166117429092919063ffffffff16565b805190915015610f8d57808060200190518101906116e39190611d6e565b610f8d5760405162461bcd60e51b815260206004820152602a60248201527f5361666545524332303a204552433230206f7065726174696f6e20646964206e6044820152691bdd081cdd58d8d9595960b21b6064820152608401610399565b60606117518484600085611759565b949350505050565b6060824710156117ba5760405162461bcd60e51b815260206004820152602660248201527f416464726573733a20696e73756666696369656e742062616c616e636520666f6044820152651c8818d85b1b60d21b6064820152608401610399565b843b6118085760405162461bcd60e51b815260206004820152601d60248201527f416464726573733a2063616c6c20746f206e6f6e2d636f6e74726163740000006044820152606401610399565b600080866001600160a01b031685876040516118249190611dc0565b60006040518083038185875af1925050503d8060008114611861576040519150601f19603f3d011682016040523d82523d6000602084013e611866565b606091505b5091509150611876828286611881565b979650505050505050565b60608315611890575081610afe565b8251156118a05782518084602001fd5b8160405162461bcd60e51b81526004016103999190611ddc565b6001600160a01b03811681146115cb57600080fd5b60008083601f8401126118e157600080fd5b50813567ffffffffffffffff8111156118f957600080fd5b60208301915083602082850101111561191157600080fd5b9250929050565b60008060008060008060008060c0898b03121561193457600080fd5b883561193f816118ba565b9750602089013561194f816118ba565b9650604089013561195f816118ba565b955060608901359450608089013567ffffffffffffffff8082111561198357600080fd5b61198f8c838d016118cf565b909650945060a08b01359150808211156119a857600080fd5b506119b58b828c016118cf565b999c989b5096995094979396929594505050565b6000806000806000608086880312156119e157600080fd5b85356119ec816118ba565b945060208601356119fc816118ba565b935060408601359250606086013567ffffffffffffffff811115611a1f57600080fd5b611a2b888289016118cf565b969995985093965092949392505050565b60008060408385031215611a4f57600080fd5b823591506020830135611a61816118ba565b809150509250929050565b60008060408385031215611a7f57600080fd5b8235611a8a816118ba565b946020939093013593505050565b600060208284031215611aaa57600080fd5b8135610afe816118ba565b634e487b7160e01b600052604160045260246000fd5b600080600060608486031215611ae057600080fd5b8335611aeb816118ba565b925060208401359150604084013567ffffffffffffffff80821115611b0f57600080fd5b818601915086601f830112611b2357600080fd5b813581811115611b3557611b35611ab5565b604051601f8201601f19908116603f01168101908382118183101715611b5d57611b5d611ab5565b81604052828152896020848701011115611b7657600080fd5b8260208601602083013760006020848301015280955050505050509250925092565b600060208284031215611baa57600080fd5b5051919050565b600060208284031215611bc357600080fd5b8151610afe816118ba565b60208082526025908201527f486f70725374616b65206e6565647320746f207472616e73666572206f776e65604082015264072736869760dc1b606082015260800190565b60208082526034908201527f43616c6c65722068617320746f20736574207468697320636f6e7472616374206040820152736173204552433138323020696e7465726661636560601b606082015260800190565b600080600080600060a08688031215611c7f57600080fd5b5050835160208501516040860151606087015160809097015192989197509594509092509050565b634e487b7160e01b600052601160045260246000fd5b600082821015611ccf57611ccf611ca7565b500390565b6001600160a01b0392909216825260208201526060604082018190526003908201526203078360ec1b608082015260a00190565b6020808252818101527f4f776e61626c653a2063616c6c6572206973206e6f7420746865206f776e6572604082015260600190565b600060018201611d4f57611d4f611ca7565b5060010190565b60008219821115611d6957611d69611ca7565b500190565b600060208284031215611d8057600080fd5b81518015158114610afe57600080fd5b60005b83811015611dab578181015183820152602001611d93565b83811115611dba576000848401525b50505050565b60008251611dd2818460208701611d90565b9190910192915050565b6020815260008251806020840152611dfb816040850160208701611d90565b601f01601f1916919091016040019291505056fea2646970667358221220ead9da3c5ad5b7488f4e1389c3eddb408d1ef1f7828ed012a103f1bbe4a55d8864736f6c634300080d0033" . parse () . expect ("invalid bytecode")
        });
    pub struct HoprWhitehat<M>(ethers::contract::Contract<M>);
    impl<M> Clone for HoprWhitehat<M> {
        fn clone(&self) -> Self {
            HoprWhitehat(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for HoprWhitehat<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for HoprWhitehat<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(HoprWhitehat))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> HoprWhitehat<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), HOPRWHITEHAT_ABI.clone(), client).into()
        }
        #[doc = r" Constructs the general purpose `Deployer` instance based on the provided constructor arguments and sends it."]
        #[doc = r" Returns a new instance of a deployer that returns an instance of this contract after sending the transaction"]
        #[doc = r""]
        #[doc = r" Notes:"]
        #[doc = r" 1. If there are no constructor arguments, you should pass `()` as the argument."]
        #[doc = r" 1. The default poll duration is 7 seconds."]
        #[doc = r" 1. The default number of confirmations is 1 block."]
        #[doc = r""]
        #[doc = r""]
        #[doc = r" # Example"]
        #[doc = r""]
        #[doc = r" Generate contract bindings with `abigen!` and deploy a new contract instance."]
        #[doc = r""]
        #[doc = r" *Note*: this requires a `bytecode` and `abi` object in the `greeter.json` artifact."]
        #[doc = r""]
        #[doc = r" ```ignore"]
        #[doc = r" # async fn deploy<M: ethers::providers::Middleware>(client: ::std::sync::Arc<M>) {"]
        #[doc = r#"     abigen!(Greeter,"../greeter.json");"#]
        #[doc = r""]
        #[doc = r#"    let greeter_contract = Greeter::deploy(client, "Hello world!".to_string()).unwrap().send().await.unwrap();"#]
        #[doc = r"    let msg = greeter_contract.greet().call().await.unwrap();"]
        #[doc = r" # }"]
        #[doc = r" ```"]
        pub fn deploy<T: ethers::core::abi::Tokenize>(
            client: ::std::sync::Arc<M>,
            constructor_args: T,
        ) -> ::std::result::Result<
            ethers::contract::builders::ContractDeployer<M, Self>,
            ethers::contract::ContractError<M>,
        > {
            let factory = ethers::contract::ContractFactory::new(
                HOPRWHITEHAT_ABI.clone(),
                HOPRWHITEHAT_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        #[doc = "Calls the contract's `activate` (0x0f15f4c0) function"]
        pub fn activate(&self) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([15, 21, 244, 192], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `canImplementInterfaceForAddress` (0x249cb3fa) function"]
        pub fn can_implement_interface_for_address(
            &self,
            interface_hash: [u8; 32],
            account: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([36, 156, 179, 250], (interface_hash, account))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `currentCaller` (0x314503d2) function"]
        pub fn current_caller(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([49, 69, 3, 210], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `deactivate` (0x51b42b00) function"]
        pub fn deactivate(&self) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([81, 180, 43, 0], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `gimmeToken` (0x0d8dbd4c) function"]
        pub fn gimme_token(&self) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([13, 141, 189, 76], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `gimmeTokenFor` (0x9b011c6f) function"]
        pub fn gimme_token_for(
            &self,
            staker: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([155, 1, 28, 111], staker)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `isActive` (0x22f3e2d4) function"]
        pub fn is_active(&self) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([34, 243, 226, 212], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `myHoprBoost` (0xde9118c5) function"]
        pub fn my_hopr_boost(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([222, 145, 24, 197], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `myHoprStake` (0x058e7326) function"]
        pub fn my_hopr_stake(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([5, 142, 115, 38], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `onERC721Received` (0x150b7a02) function"]
        pub fn on_erc721_received(
            &self,
            operator: ethers::core::types::Address,
            from: ethers::core::types::Address,
            token_id: ethers::core::types::U256,
            data: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, [u8; 4]> {
            self.0
                .method_hash([21, 11, 122, 2], (operator, from, token_id, data))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `onTokenTransfer` (0xa4c0ed36) function"]
        pub fn on_token_transfer(
            &self,
            from: ethers::core::types::Address,
            value: ethers::core::types::U256,
            data: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([164, 192, 237, 54], (from, value, data))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `owner` (0x8da5cb5b) function"]
        pub fn owner(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([141, 165, 203, 91], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `ownerRescueBoosterNft` (0x93fd97c9) function"]
        pub fn owner_rescue_booster_nft(
            &self,
            staker_address: ethers::core::types::Address,
            token_id: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([147, 253, 151, 201], (staker_address, token_id))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `ownerRescueBoosterNftInBatch` (0x770f07e2) function"]
        pub fn owner_rescue_booster_nft_in_batch(
            &self,
            staker_address: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([119, 15, 7, 226], staker_address)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `reclaimErc20Tokens` (0x6067bc15) function"]
        pub fn reclaim_erc_20_tokens(
            &self,
            token_address: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([96, 103, 188, 21], token_address)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `reclaimErc721Tokens` (0x48c64e41) function"]
        pub fn reclaim_erc_721_tokens(
            &self,
            token_address: ethers::core::types::Address,
            token_id: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([72, 198, 78, 65], (token_address, token_id))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `renounceOwnership` (0x715018a6) function"]
        pub fn renounce_ownership(&self) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([113, 80, 24, 166], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `rescuedXHoprAmount` (0xb7955631) function"]
        pub fn rescued_x_hopr_amount(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([183, 149, 86, 49], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `tokensReceived` (0x0023de29) function"]
        pub fn tokens_received(
            &self,
            operator: ethers::core::types::Address,
            from: ethers::core::types::Address,
            to: ethers::core::types::Address,
            amount: ethers::core::types::U256,
            user_data: ethers::core::types::Bytes,
            operator_data: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [0, 35, 222, 41],
                    (operator, from, to, amount, user_data, operator_data),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `transferBackOwnership` (0xbb04fb69) function"]
        pub fn transfer_back_ownership(
            &self,
            multisig: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([187, 4, 251, 105], multisig)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `transferOwnership` (0xf2fde38b) function"]
        pub fn transfer_ownership(
            &self,
            new_owner: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([242, 253, 227, 139], new_owner)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `wxHopr` (0x2dcdad7d) function"]
        pub fn wx_hopr(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([45, 205, 173, 125], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `xHopr` (0xa188cbbc) function"]
        pub fn x_hopr(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([161, 136, 203, 188], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Gets the contract's `Called777Hook` event"]
        pub fn called_777_hook_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, Called777HookFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Called777HookForFunding` event"]
        pub fn called_777_hook_for_funding_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, Called777HookForFundingFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `OwnershipTransferred` event"]
        pub fn ownership_transferred_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, OwnershipTransferredFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Received677` event"]
        pub fn received_677_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, Received677Filter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `ReclaimedBoost` event"]
        pub fn reclaimed_boost_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, ReclaimedBoostFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `RequestedGimme` event"]
        pub fn requested_gimme_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, RequestedGimmeFilter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(&self) -> ethers::contract::builders::Event<M, HoprWhitehatEvents> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for HoprWhitehat<M> {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "Called777Hook", abi = "Called777Hook(address,address,uint256)")]
    pub struct Called777HookFilter {
        #[ethevent(indexed)]
        pub contract_address: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub from: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub amount: ethers::core::types::U256,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(
        name = "Called777HookForFunding",
        abi = "Called777HookForFunding(address,address,uint256)"
    )]
    pub struct Called777HookForFundingFilter {
        #[ethevent(indexed)]
        pub contract_address: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub from: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub amount: ethers::core::types::U256,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(
        name = "OwnershipTransferred",
        abi = "OwnershipTransferred(address,address)"
    )]
    pub struct OwnershipTransferredFilter {
        #[ethevent(indexed)]
        pub previous_owner: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub new_owner: ethers::core::types::Address,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "Received677", abi = "Received677(address,address,uint256)")]
    pub struct Received677Filter {
        #[ethevent(indexed)]
        pub contract_address: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub from: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub amount: ethers::core::types::U256,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "ReclaimedBoost", abi = "ReclaimedBoost(address,uint256)")]
    pub struct ReclaimedBoostFilter {
        #[ethevent(indexed)]
        pub account: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub token_id: ethers::core::types::U256,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "RequestedGimme", abi = "RequestedGimme(address,uint256)")]
    pub struct RequestedGimmeFilter {
        #[ethevent(indexed)]
        pub account: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub entitled_reward: ethers::core::types::U256,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum HoprWhitehatEvents {
        Called777HookFilter(Called777HookFilter),
        Called777HookForFundingFilter(Called777HookForFundingFilter),
        OwnershipTransferredFilter(OwnershipTransferredFilter),
        Received677Filter(Received677Filter),
        ReclaimedBoostFilter(ReclaimedBoostFilter),
        RequestedGimmeFilter(RequestedGimmeFilter),
    }
    impl ethers::contract::EthLogDecode for HoprWhitehatEvents {
        fn decode_log(
            log: &ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = Called777HookFilter::decode_log(log) {
                return Ok(HoprWhitehatEvents::Called777HookFilter(decoded));
            }
            if let Ok(decoded) = Called777HookForFundingFilter::decode_log(log) {
                return Ok(HoprWhitehatEvents::Called777HookForFundingFilter(decoded));
            }
            if let Ok(decoded) = OwnershipTransferredFilter::decode_log(log) {
                return Ok(HoprWhitehatEvents::OwnershipTransferredFilter(decoded));
            }
            if let Ok(decoded) = Received677Filter::decode_log(log) {
                return Ok(HoprWhitehatEvents::Received677Filter(decoded));
            }
            if let Ok(decoded) = ReclaimedBoostFilter::decode_log(log) {
                return Ok(HoprWhitehatEvents::ReclaimedBoostFilter(decoded));
            }
            if let Ok(decoded) = RequestedGimmeFilter::decode_log(log) {
                return Ok(HoprWhitehatEvents::RequestedGimmeFilter(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for HoprWhitehatEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                HoprWhitehatEvents::Called777HookFilter(element) => element.fmt(f),
                HoprWhitehatEvents::Called777HookForFundingFilter(element) => element.fmt(f),
                HoprWhitehatEvents::OwnershipTransferredFilter(element) => element.fmt(f),
                HoprWhitehatEvents::Received677Filter(element) => element.fmt(f),
                HoprWhitehatEvents::ReclaimedBoostFilter(element) => element.fmt(f),
                HoprWhitehatEvents::RequestedGimmeFilter(element) => element.fmt(f),
            }
        }
    }
    #[doc = "Container type for all input parameters for the `activate` function with signature `activate()` and selector `[15, 21, 244, 192]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "activate", abi = "activate()")]
    pub struct ActivateCall;
    #[doc = "Container type for all input parameters for the `canImplementInterfaceForAddress` function with signature `canImplementInterfaceForAddress(bytes32,address)` and selector `[36, 156, 179, 250]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "canImplementInterfaceForAddress",
        abi = "canImplementInterfaceForAddress(bytes32,address)"
    )]
    pub struct CanImplementInterfaceForAddressCall {
        pub interface_hash: [u8; 32],
        pub account: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `currentCaller` function with signature `currentCaller()` and selector `[49, 69, 3, 210]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "currentCaller", abi = "currentCaller()")]
    pub struct CurrentCallerCall;
    #[doc = "Container type for all input parameters for the `deactivate` function with signature `deactivate()` and selector `[81, 180, 43, 0]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "deactivate", abi = "deactivate()")]
    pub struct DeactivateCall;
    #[doc = "Container type for all input parameters for the `gimmeToken` function with signature `gimmeToken()` and selector `[13, 141, 189, 76]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "gimmeToken", abi = "gimmeToken()")]
    pub struct GimmeTokenCall;
    #[doc = "Container type for all input parameters for the `gimmeTokenFor` function with signature `gimmeTokenFor(address)` and selector `[155, 1, 28, 111]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "gimmeTokenFor", abi = "gimmeTokenFor(address)")]
    pub struct GimmeTokenForCall {
        pub staker: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `isActive` function with signature `isActive()` and selector `[34, 243, 226, 212]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "isActive", abi = "isActive()")]
    pub struct IsActiveCall;
    #[doc = "Container type for all input parameters for the `myHoprBoost` function with signature `myHoprBoost()` and selector `[222, 145, 24, 197]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "myHoprBoost", abi = "myHoprBoost()")]
    pub struct MyHoprBoostCall;
    #[doc = "Container type for all input parameters for the `myHoprStake` function with signature `myHoprStake()` and selector `[5, 142, 115, 38]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "myHoprStake", abi = "myHoprStake()")]
    pub struct MyHoprStakeCall;
    #[doc = "Container type for all input parameters for the `onERC721Received` function with signature `onERC721Received(address,address,uint256,bytes)` and selector `[21, 11, 122, 2]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "onERC721Received",
        abi = "onERC721Received(address,address,uint256,bytes)"
    )]
    pub struct OnERC721ReceivedCall {
        pub operator: ethers::core::types::Address,
        pub from: ethers::core::types::Address,
        pub token_id: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
    }
    #[doc = "Container type for all input parameters for the `onTokenTransfer` function with signature `onTokenTransfer(address,uint256,bytes)` and selector `[164, 192, 237, 54]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "onTokenTransfer",
        abi = "onTokenTransfer(address,uint256,bytes)"
    )]
    pub struct OnTokenTransferCall {
        pub from: ethers::core::types::Address,
        pub value: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
    }
    #[doc = "Container type for all input parameters for the `owner` function with signature `owner()` and selector `[141, 165, 203, 91]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "owner", abi = "owner()")]
    pub struct OwnerCall;
    #[doc = "Container type for all input parameters for the `ownerRescueBoosterNft` function with signature `ownerRescueBoosterNft(address,uint256)` and selector `[147, 253, 151, 201]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "ownerRescueBoosterNft",
        abi = "ownerRescueBoosterNft(address,uint256)"
    )]
    pub struct OwnerRescueBoosterNftCall {
        pub staker_address: ethers::core::types::Address,
        pub token_id: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `ownerRescueBoosterNftInBatch` function with signature `ownerRescueBoosterNftInBatch(address)` and selector `[119, 15, 7, 226]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "ownerRescueBoosterNftInBatch",
        abi = "ownerRescueBoosterNftInBatch(address)"
    )]
    pub struct OwnerRescueBoosterNftInBatchCall {
        pub staker_address: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `reclaimErc20Tokens` function with signature `reclaimErc20Tokens(address)` and selector `[96, 103, 188, 21]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "reclaimErc20Tokens", abi = "reclaimErc20Tokens(address)")]
    pub struct ReclaimErc20TokensCall {
        pub token_address: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `reclaimErc721Tokens` function with signature `reclaimErc721Tokens(address,uint256)` and selector `[72, 198, 78, 65]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "reclaimErc721Tokens",
        abi = "reclaimErc721Tokens(address,uint256)"
    )]
    pub struct ReclaimErc721TokensCall {
        pub token_address: ethers::core::types::Address,
        pub token_id: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `renounceOwnership` function with signature `renounceOwnership()` and selector `[113, 80, 24, 166]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "renounceOwnership", abi = "renounceOwnership()")]
    pub struct RenounceOwnershipCall;
    #[doc = "Container type for all input parameters for the `rescuedXHoprAmount` function with signature `rescuedXHoprAmount()` and selector `[183, 149, 86, 49]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "rescuedXHoprAmount", abi = "rescuedXHoprAmount()")]
    pub struct RescuedXHoprAmountCall;
    #[doc = "Container type for all input parameters for the `tokensReceived` function with signature `tokensReceived(address,address,address,uint256,bytes,bytes)` and selector `[0, 35, 222, 41]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "tokensReceived",
        abi = "tokensReceived(address,address,address,uint256,bytes,bytes)"
    )]
    pub struct TokensReceivedCall {
        pub operator: ethers::core::types::Address,
        pub from: ethers::core::types::Address,
        pub to: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
        pub user_data: ethers::core::types::Bytes,
        pub operator_data: ethers::core::types::Bytes,
    }
    #[doc = "Container type for all input parameters for the `transferBackOwnership` function with signature `transferBackOwnership(address)` and selector `[187, 4, 251, 105]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "transferBackOwnership", abi = "transferBackOwnership(address)")]
    pub struct TransferBackOwnershipCall {
        pub multisig: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `transferOwnership` function with signature `transferOwnership(address)` and selector `[242, 253, 227, 139]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "transferOwnership", abi = "transferOwnership(address)")]
    pub struct TransferOwnershipCall {
        pub new_owner: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `wxHopr` function with signature `wxHopr()` and selector `[45, 205, 173, 125]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "wxHopr", abi = "wxHopr()")]
    pub struct WxHoprCall;
    #[doc = "Container type for all input parameters for the `xHopr` function with signature `xHopr()` and selector `[161, 136, 203, 188]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "xHopr", abi = "xHopr()")]
    pub struct XhoprCall;
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum HoprWhitehatCalls {
        Activate(ActivateCall),
        CanImplementInterfaceForAddress(CanImplementInterfaceForAddressCall),
        CurrentCaller(CurrentCallerCall),
        Deactivate(DeactivateCall),
        GimmeToken(GimmeTokenCall),
        GimmeTokenFor(GimmeTokenForCall),
        IsActive(IsActiveCall),
        MyHoprBoost(MyHoprBoostCall),
        MyHoprStake(MyHoprStakeCall),
        OnERC721Received(OnERC721ReceivedCall),
        OnTokenTransfer(OnTokenTransferCall),
        Owner(OwnerCall),
        OwnerRescueBoosterNft(OwnerRescueBoosterNftCall),
        OwnerRescueBoosterNftInBatch(OwnerRescueBoosterNftInBatchCall),
        ReclaimErc20Tokens(ReclaimErc20TokensCall),
        ReclaimErc721Tokens(ReclaimErc721TokensCall),
        RenounceOwnership(RenounceOwnershipCall),
        RescuedXHoprAmount(RescuedXHoprAmountCall),
        TokensReceived(TokensReceivedCall),
        TransferBackOwnership(TransferBackOwnershipCall),
        TransferOwnership(TransferOwnershipCall),
        WxHopr(WxHoprCall),
        Xhopr(XhoprCall),
    }
    impl ethers::core::abi::AbiDecode for HoprWhitehatCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <ActivateCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWhitehatCalls::Activate(decoded));
            }
            if let Ok(decoded) =
                <CanImplementInterfaceForAddressCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(HoprWhitehatCalls::CanImplementInterfaceForAddress(decoded));
            }
            if let Ok(decoded) =
                <CurrentCallerCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWhitehatCalls::CurrentCaller(decoded));
            }
            if let Ok(decoded) =
                <DeactivateCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWhitehatCalls::Deactivate(decoded));
            }
            if let Ok(decoded) =
                <GimmeTokenCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWhitehatCalls::GimmeToken(decoded));
            }
            if let Ok(decoded) =
                <GimmeTokenForCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWhitehatCalls::GimmeTokenFor(decoded));
            }
            if let Ok(decoded) =
                <IsActiveCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWhitehatCalls::IsActive(decoded));
            }
            if let Ok(decoded) =
                <MyHoprBoostCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWhitehatCalls::MyHoprBoost(decoded));
            }
            if let Ok(decoded) =
                <MyHoprStakeCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWhitehatCalls::MyHoprStake(decoded));
            }
            if let Ok(decoded) =
                <OnERC721ReceivedCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWhitehatCalls::OnERC721Received(decoded));
            }
            if let Ok(decoded) =
                <OnTokenTransferCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWhitehatCalls::OnTokenTransfer(decoded));
            }
            if let Ok(decoded) = <OwnerCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWhitehatCalls::Owner(decoded));
            }
            if let Ok(decoded) =
                <OwnerRescueBoosterNftCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWhitehatCalls::OwnerRescueBoosterNft(decoded));
            }
            if let Ok(decoded) =
                <OwnerRescueBoosterNftInBatchCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(HoprWhitehatCalls::OwnerRescueBoosterNftInBatch(decoded));
            }
            if let Ok(decoded) =
                <ReclaimErc20TokensCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWhitehatCalls::ReclaimErc20Tokens(decoded));
            }
            if let Ok(decoded) =
                <ReclaimErc721TokensCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWhitehatCalls::ReclaimErc721Tokens(decoded));
            }
            if let Ok(decoded) =
                <RenounceOwnershipCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWhitehatCalls::RenounceOwnership(decoded));
            }
            if let Ok(decoded) =
                <RescuedXHoprAmountCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWhitehatCalls::RescuedXHoprAmount(decoded));
            }
            if let Ok(decoded) =
                <TokensReceivedCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWhitehatCalls::TokensReceived(decoded));
            }
            if let Ok(decoded) =
                <TransferBackOwnershipCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWhitehatCalls::TransferBackOwnership(decoded));
            }
            if let Ok(decoded) =
                <TransferOwnershipCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWhitehatCalls::TransferOwnership(decoded));
            }
            if let Ok(decoded) = <WxHoprCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWhitehatCalls::WxHopr(decoded));
            }
            if let Ok(decoded) = <XhoprCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWhitehatCalls::Xhopr(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for HoprWhitehatCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                HoprWhitehatCalls::Activate(element) => element.encode(),
                HoprWhitehatCalls::CanImplementInterfaceForAddress(element) => element.encode(),
                HoprWhitehatCalls::CurrentCaller(element) => element.encode(),
                HoprWhitehatCalls::Deactivate(element) => element.encode(),
                HoprWhitehatCalls::GimmeToken(element) => element.encode(),
                HoprWhitehatCalls::GimmeTokenFor(element) => element.encode(),
                HoprWhitehatCalls::IsActive(element) => element.encode(),
                HoprWhitehatCalls::MyHoprBoost(element) => element.encode(),
                HoprWhitehatCalls::MyHoprStake(element) => element.encode(),
                HoprWhitehatCalls::OnERC721Received(element) => element.encode(),
                HoprWhitehatCalls::OnTokenTransfer(element) => element.encode(),
                HoprWhitehatCalls::Owner(element) => element.encode(),
                HoprWhitehatCalls::OwnerRescueBoosterNft(element) => element.encode(),
                HoprWhitehatCalls::OwnerRescueBoosterNftInBatch(element) => element.encode(),
                HoprWhitehatCalls::ReclaimErc20Tokens(element) => element.encode(),
                HoprWhitehatCalls::ReclaimErc721Tokens(element) => element.encode(),
                HoprWhitehatCalls::RenounceOwnership(element) => element.encode(),
                HoprWhitehatCalls::RescuedXHoprAmount(element) => element.encode(),
                HoprWhitehatCalls::TokensReceived(element) => element.encode(),
                HoprWhitehatCalls::TransferBackOwnership(element) => element.encode(),
                HoprWhitehatCalls::TransferOwnership(element) => element.encode(),
                HoprWhitehatCalls::WxHopr(element) => element.encode(),
                HoprWhitehatCalls::Xhopr(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for HoprWhitehatCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                HoprWhitehatCalls::Activate(element) => element.fmt(f),
                HoprWhitehatCalls::CanImplementInterfaceForAddress(element) => element.fmt(f),
                HoprWhitehatCalls::CurrentCaller(element) => element.fmt(f),
                HoprWhitehatCalls::Deactivate(element) => element.fmt(f),
                HoprWhitehatCalls::GimmeToken(element) => element.fmt(f),
                HoprWhitehatCalls::GimmeTokenFor(element) => element.fmt(f),
                HoprWhitehatCalls::IsActive(element) => element.fmt(f),
                HoprWhitehatCalls::MyHoprBoost(element) => element.fmt(f),
                HoprWhitehatCalls::MyHoprStake(element) => element.fmt(f),
                HoprWhitehatCalls::OnERC721Received(element) => element.fmt(f),
                HoprWhitehatCalls::OnTokenTransfer(element) => element.fmt(f),
                HoprWhitehatCalls::Owner(element) => element.fmt(f),
                HoprWhitehatCalls::OwnerRescueBoosterNft(element) => element.fmt(f),
                HoprWhitehatCalls::OwnerRescueBoosterNftInBatch(element) => element.fmt(f),
                HoprWhitehatCalls::ReclaimErc20Tokens(element) => element.fmt(f),
                HoprWhitehatCalls::ReclaimErc721Tokens(element) => element.fmt(f),
                HoprWhitehatCalls::RenounceOwnership(element) => element.fmt(f),
                HoprWhitehatCalls::RescuedXHoprAmount(element) => element.fmt(f),
                HoprWhitehatCalls::TokensReceived(element) => element.fmt(f),
                HoprWhitehatCalls::TransferBackOwnership(element) => element.fmt(f),
                HoprWhitehatCalls::TransferOwnership(element) => element.fmt(f),
                HoprWhitehatCalls::WxHopr(element) => element.fmt(f),
                HoprWhitehatCalls::Xhopr(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<ActivateCall> for HoprWhitehatCalls {
        fn from(var: ActivateCall) -> Self {
            HoprWhitehatCalls::Activate(var)
        }
    }
    impl ::std::convert::From<CanImplementInterfaceForAddressCall> for HoprWhitehatCalls {
        fn from(var: CanImplementInterfaceForAddressCall) -> Self {
            HoprWhitehatCalls::CanImplementInterfaceForAddress(var)
        }
    }
    impl ::std::convert::From<CurrentCallerCall> for HoprWhitehatCalls {
        fn from(var: CurrentCallerCall) -> Self {
            HoprWhitehatCalls::CurrentCaller(var)
        }
    }
    impl ::std::convert::From<DeactivateCall> for HoprWhitehatCalls {
        fn from(var: DeactivateCall) -> Self {
            HoprWhitehatCalls::Deactivate(var)
        }
    }
    impl ::std::convert::From<GimmeTokenCall> for HoprWhitehatCalls {
        fn from(var: GimmeTokenCall) -> Self {
            HoprWhitehatCalls::GimmeToken(var)
        }
    }
    impl ::std::convert::From<GimmeTokenForCall> for HoprWhitehatCalls {
        fn from(var: GimmeTokenForCall) -> Self {
            HoprWhitehatCalls::GimmeTokenFor(var)
        }
    }
    impl ::std::convert::From<IsActiveCall> for HoprWhitehatCalls {
        fn from(var: IsActiveCall) -> Self {
            HoprWhitehatCalls::IsActive(var)
        }
    }
    impl ::std::convert::From<MyHoprBoostCall> for HoprWhitehatCalls {
        fn from(var: MyHoprBoostCall) -> Self {
            HoprWhitehatCalls::MyHoprBoost(var)
        }
    }
    impl ::std::convert::From<MyHoprStakeCall> for HoprWhitehatCalls {
        fn from(var: MyHoprStakeCall) -> Self {
            HoprWhitehatCalls::MyHoprStake(var)
        }
    }
    impl ::std::convert::From<OnERC721ReceivedCall> for HoprWhitehatCalls {
        fn from(var: OnERC721ReceivedCall) -> Self {
            HoprWhitehatCalls::OnERC721Received(var)
        }
    }
    impl ::std::convert::From<OnTokenTransferCall> for HoprWhitehatCalls {
        fn from(var: OnTokenTransferCall) -> Self {
            HoprWhitehatCalls::OnTokenTransfer(var)
        }
    }
    impl ::std::convert::From<OwnerCall> for HoprWhitehatCalls {
        fn from(var: OwnerCall) -> Self {
            HoprWhitehatCalls::Owner(var)
        }
    }
    impl ::std::convert::From<OwnerRescueBoosterNftCall> for HoprWhitehatCalls {
        fn from(var: OwnerRescueBoosterNftCall) -> Self {
            HoprWhitehatCalls::OwnerRescueBoosterNft(var)
        }
    }
    impl ::std::convert::From<OwnerRescueBoosterNftInBatchCall> for HoprWhitehatCalls {
        fn from(var: OwnerRescueBoosterNftInBatchCall) -> Self {
            HoprWhitehatCalls::OwnerRescueBoosterNftInBatch(var)
        }
    }
    impl ::std::convert::From<ReclaimErc20TokensCall> for HoprWhitehatCalls {
        fn from(var: ReclaimErc20TokensCall) -> Self {
            HoprWhitehatCalls::ReclaimErc20Tokens(var)
        }
    }
    impl ::std::convert::From<ReclaimErc721TokensCall> for HoprWhitehatCalls {
        fn from(var: ReclaimErc721TokensCall) -> Self {
            HoprWhitehatCalls::ReclaimErc721Tokens(var)
        }
    }
    impl ::std::convert::From<RenounceOwnershipCall> for HoprWhitehatCalls {
        fn from(var: RenounceOwnershipCall) -> Self {
            HoprWhitehatCalls::RenounceOwnership(var)
        }
    }
    impl ::std::convert::From<RescuedXHoprAmountCall> for HoprWhitehatCalls {
        fn from(var: RescuedXHoprAmountCall) -> Self {
            HoprWhitehatCalls::RescuedXHoprAmount(var)
        }
    }
    impl ::std::convert::From<TokensReceivedCall> for HoprWhitehatCalls {
        fn from(var: TokensReceivedCall) -> Self {
            HoprWhitehatCalls::TokensReceived(var)
        }
    }
    impl ::std::convert::From<TransferBackOwnershipCall> for HoprWhitehatCalls {
        fn from(var: TransferBackOwnershipCall) -> Self {
            HoprWhitehatCalls::TransferBackOwnership(var)
        }
    }
    impl ::std::convert::From<TransferOwnershipCall> for HoprWhitehatCalls {
        fn from(var: TransferOwnershipCall) -> Self {
            HoprWhitehatCalls::TransferOwnership(var)
        }
    }
    impl ::std::convert::From<WxHoprCall> for HoprWhitehatCalls {
        fn from(var: WxHoprCall) -> Self {
            HoprWhitehatCalls::WxHopr(var)
        }
    }
    impl ::std::convert::From<XhoprCall> for HoprWhitehatCalls {
        fn from(var: XhoprCall) -> Self {
            HoprWhitehatCalls::Xhopr(var)
        }
    }
    #[doc = "Container type for all return fields from the `canImplementInterfaceForAddress` function with signature `canImplementInterfaceForAddress(bytes32,address)` and selector `[36, 156, 179, 250]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct CanImplementInterfaceForAddressReturn(pub [u8; 32]);
    #[doc = "Container type for all return fields from the `currentCaller` function with signature `currentCaller()` and selector `[49, 69, 3, 210]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct CurrentCallerReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `isActive` function with signature `isActive()` and selector `[34, 243, 226, 212]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct IsActiveReturn(pub bool);
    #[doc = "Container type for all return fields from the `myHoprBoost` function with signature `myHoprBoost()` and selector `[222, 145, 24, 197]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct MyHoprBoostReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `myHoprStake` function with signature `myHoprStake()` and selector `[5, 142, 115, 38]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct MyHoprStakeReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `onERC721Received` function with signature `onERC721Received(address,address,uint256,bytes)` and selector `[21, 11, 122, 2]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct OnERC721ReceivedReturn(pub [u8; 4]);
    #[doc = "Container type for all return fields from the `onTokenTransfer` function with signature `onTokenTransfer(address,uint256,bytes)` and selector `[164, 192, 237, 54]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct OnTokenTransferReturn(pub bool);
    #[doc = "Container type for all return fields from the `owner` function with signature `owner()` and selector `[141, 165, 203, 91]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct OwnerReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `rescuedXHoprAmount` function with signature `rescuedXHoprAmount()` and selector `[183, 149, 86, 49]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct RescuedXHoprAmountReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `wxHopr` function with signature `wxHopr()` and selector `[45, 205, 173, 125]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct WxHoprReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `xHopr` function with signature `xHopr()` and selector `[161, 136, 203, 188]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct XhoprReturn(pub ethers::core::types::Address);
}
