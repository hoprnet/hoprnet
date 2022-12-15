pub use hopr_stake::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod hopr_stake {
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
    #[doc = "HoprStake was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_nftAddress\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"_newOwner\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"_lockToken\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"_rewardToken\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"rewardAmount\",\"type\":\"uint256\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"Claimed\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"previousOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"OwnershipTransferred\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"boostTokenId\",\"type\":\"uint256\",\"components\":[],\"indexed\":true},{\"internalType\":\"bool\",\"name\":\"factorRegistered\",\"type\":\"bool\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"Redeemed\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"actualAmount\",\"type\":\"uint256\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"virtualAmount\",\"type\":\"uint256\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"Released\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"RewardFueled\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"actualAmount\",\"type\":\"uint256\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"virtualAmount\",\"type\":\"uint256\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"Staked\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"increment\",\"type\":\"uint256\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"Sync\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"BASIC_FACTOR_NUMERATOR\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"BASIC_START\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"BOOST_CAP\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"FACTOR_DENOMINATOR\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"LOCK_TOKEN\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"PROGRAM_END\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"REWARD_TOKEN\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"SEED_FACTOR_NUMERATOR\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"SEED_START\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"accounts\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"actualLockedTokenAmount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"virtualLockedTokenAmount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"lastSyncTimestamp\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"cumulatedRewards\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"claimedRewards\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"availableReward\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"claimRewards\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getCumulatedRewardsIncrement\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address[]\",\"name\":\"investors\",\"type\":\"address[]\",\"components\":[]},{\"internalType\":\"uint256[]\",\"name\":\"caps\",\"type\":\"uint256[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"lock\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"nftContract\",\"outputs\":[{\"internalType\":\"contract IHoprBoost\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"tokenId\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"onERC721Received\",\"outputs\":[{\"internalType\":\"bytes4\",\"name\":\"\",\"type\":\"bytes4\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_from\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"_value\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"_data\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"onTokenTransfer\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"owner\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"tokenAddress\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"reclaimErc20Tokens\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"tokenAddress\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"tokenId\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"reclaimErc721Tokens\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"redeemedFactor\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"redeemedFactorIndex\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"redeemedNft\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"redeemedNftIndex\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"renounceOwnership\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"sync\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"userData\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"tokensReceived\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"totalLocked\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferOwnership\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"unlock\",\"outputs\":[]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static HOPRSTAKE_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static HOPRSTAKE_BYTECODE: ethers::contract::Lazy<ethers::core::types::Bytes> =
        ethers::contract::Lazy::new(|| {
            "0x6080604052600280546001600160a01b031990811673d057604a14982fe8d88c5fc25aac3267ea142a08179091556003805490911673d4fdec44db9d44b8f2b6d529620f9c0c7066a2c11790553480156200005957600080fd5b50604051620023bd380380620023bd8339810160408190526200007c91620002cc565b62000087336200018a565b600180554660648114620000c657600280546001600160a01b038086166001600160a01b03199283161790925560038054928516929091169190911790555b600480546001600160a01b0319166001600160a01b038716179055620000ec84620001da565b6040516329965a1d60e01b815230600482018190527fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b60248301526044820152731820a4b7618bde71dce8cdc73aab6c95905fad24906329965a1d90606401600060405180830381600087803b1580156200016657600080fd5b505af11580156200017b573d6000803e3d6000fd5b50505050505050505062000329565b600080546001600160a01b038381166001600160a01b0319831681178455604051919092169283917f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09190a35050565b6000546001600160a01b031633146200023a5760405162461bcd60e51b815260206004820181905260248201527f4f776e61626c653a2063616c6c6572206973206e6f7420746865206f776e657260448201526064015b60405180910390fd5b6001600160a01b038116620002a15760405162461bcd60e51b815260206004820152602660248201527f4f776e61626c653a206e6577206f776e657220697320746865207a65726f206160448201526564647265737360d01b606482015260840162000231565b620002ac816200018a565b50565b80516001600160a01b0381168114620002c757600080fd5b919050565b60008060008060808587031215620002e357600080fd5b620002ee85620002af565b9350620002fe60208601620002af565b92506200030e60408601620002af565b91506200031e60608601620002af565b905092959194509250565b61208480620003396000396000f3fe608060405234801561001057600080fd5b50600436106101d95760003560e01c806370af093411610104578063cbffa3c7116100a2578063ef0526a211610071578063ef0526a21461046e578063ef5cfb8c1461047f578063f20c912414610492578063f2fde38b1461049d57600080fd5b8063cbffa3c71461041d578063d0c02d6314610428578063d0da680114610448578063d56d229d1461045b57600080fd5b80638da5cb5b116100de5780638da5cb5b146103af57806399248ea7146103d4578063a4c0ed36146103e7578063a58411941461040a57600080fd5b806370af093414610369578063715018a61461037c57806381128c1d1461038457600080fd5b806348c64e411161017c5780635e5c06e21161014b5780635e5c06e2146102dc5780635ef73d58146103415780636067bc151461034d5780636aa8d4cf1461036057600080fd5b806348c64e41146102a45780634ad84b34146102b757806356891412146102c05780635c3c71f4146102c957600080fd5b8063150b7a02116101b8578063150b7a021461023c5780631f014d83146102685780632f6c493c146102885780632f9984681461029b57600080fd5b806223de29146101de5780630a1a257a146101f3578063112376dc14610231575b600080fd5b6101f16101ec366004611b46565b6104b0565b005b61021e610201366004611bf1565b600560209081526000928352604080842090915290825290205481565b6040519081526020015b60405180910390f35b61021e636128d3c081565b61024f61024a366004611c1b565b61065d565b6040516001600160e01b03199091168152602001610228565b61021e610276366004611c8a565b60086020526000908152604090205481565b6101f1610296366004611c8a565b610b3d565b61021e611b7881565b6101f16102b2366004611bf1565b610d2b565b61021e600b5481565b61021e600a5481565b6101f16102d7366004611cea565b610e3d565b6103196102ea366004611c8a565b600960205260009081526040902080546001820154600283015460038401546004909401549293919290919085565b604080519586526020860194909452928401919091526060830152608082015260a001610228565b61021e64e8d4a5100081565b6101f161035b366004611c8a565b610fda565b61021e61169b81565b61021e610377366004611c8a565b61118d565b6101f161119e565b61021e610392366004611bf1565b600760209081526000928352604080842090915290825290205481565b6000546001600160a01b03165b6040516001600160a01b039091168152602001610228565b6003546103bc906001600160a01b031681565b6103fa6103f5366004611d6c565b6111d4565b6040519015158152602001610228565b6101f1610418366004611c8a565b6112fc565b61021e6361e5685081565b61021e610436366004611c8a565b60066020526000908152604090205481565b6002546103bc906001600160a01b031681565b6004546103bc906001600160a01b031681565b61021e69d3c21bcecceda100000081565b6101f161048d366004611c8a565b611308565b61021e6360fff54081565b6101f16104ab366004611c8a565b61131a565b6003546001600160a01b0316331461051e5760405162461bcd60e51b815260206004820152602660248201527f486f70725374616b653a2053656e646572206d757374206265207778484f5052604482015265103a37b5b2b760d11b60648201526084015b60405180910390fd5b6001600160a01b038616301461059c5760405162461bcd60e51b815260206004820152603760248201527f486f70725374616b653a204d7573742062652073656e64696e6720746f6b656e60448201527f7320746f20486f70725374616b6520636f6e74726163740000000000000000006064820152608401610515565b6000546001600160a01b038881169116146106115760405162461bcd60e51b815260206004820152602f60248201527f486f70725374616b653a204f6e6c7920616363657074206f776e657220746f2060448201526e70726f76696465207265776172647360881b6064820152608401610515565b84600b60008282546106239190611e4d565b909155505060405185907f2bf52bcae319602514e02ff69bbe4b89a19718b96e7867044128ec872419437c90600090a25050505050505050565b6004546000906001600160a01b0316336001600160a01b0316146106e95760405162461bcd60e51b815260206004820152603f60248201527f486f70725374616b653a2043616e6e6f7420536166655472616e73666572467260448201527f6f6d20746f6b656e73206f74686572207468616e20486f7072426f6f73742e006064820152608401610515565b6361e568504211156107555760405162461bcd60e51b815260206004820152602f60248201527f486f70725374616b653a2050726f6772616d20656e6465642c2063616e6e6f7460448201526e103932b232b2b6903137b7b9ba399760891b6064820152608401610515565b61075e856113b2565b6001600160a01b03851660008181526005602090815260408083206006808452828520805486529184529184208990559383529052815460019291906107a5908490611e4d565b90915550506004805460405163562317c560e01b81529182018690526000916001600160a01b039091169063562317c590602401602060405180830381865afa1580156107f6573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061081a9190611e65565b6004805460405163225b377d60e21b815291820188905291925060009182916001600160a01b039091169063896cddf4906024016040805180830381865afa15801561086a573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061088e9190611e7e565b91509150804211156108f55760405162461bcd60e51b815260206004820152602a60248201527f486f70725374616b653a2043616e6e6f742072656465656d20616e20657870696044820152693932b2103137b7b9ba1760b11b6064820152608401610515565b6001600160a01b038816600090815260086020526040812054905b81811015610a9f576001600160a01b038a811660009081526007602090815260408083208584529091528082205460048054925163225b377d60e21b81529081018290529093919091169063896cddf4906024016040805180830381865afa158015610980573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906109a49190611e7e565b506004805460405163562317c560e01b815291820185905291925088916001600160a01b03169063562317c590602401602060405180830381865afa1580156109f1573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610a159190611e65565b03610a8a5785811015610a49576001600160a01b038c16600090815260076020908152604080832086845290915290208b90555b604051868210908c906001600160a01b038f16907f066d96cb280fccf3a0a3a5686966a801b2690a32ec98fd2711a4e09f345d935d90600090a45050610a9f565b50508080610a9790611ea2565b915050610910565b818103610b26576001600160a01b038a16600081815260076020908152604080832086845282528083208d905592825260089052908120805460019290610ae7908490611e4d565b90915550506040516001908a906001600160a01b038d16907f066d96cb280fccf3a0a3a5686966a801b2690a32ec98fd2711a4e09f345d935d90600090a45b50630a85bd0160e11b9a9950505050505050505050565b6361e568504211610bac5760405162461bcd60e51b815260206004820152603360248201527f486f70725374616b653a2050726f6772616d206973206f6e676f696e672c206360448201527230b73737ba103ab73637b1b59039ba30b5b29760691b6064820152608401610515565b6001600160a01b03811660009081526009602052604090208054600190910154610bd5836113b2565b6001600160a01b0383166000908152600960205260408120818155600101819055600a8054849290610c08908490611ebb565b90915550610c1790508361143d565b600254610c2e906001600160a01b031684846115dd565b60005b6001600160a01b038416600090815260066020526040902054811015610ced57600480546001600160a01b038681166000818152600560209081526040808320888452909152908190205490516323b872dd60e01b815230958101959095526024850191909152604484015216906323b872dd90606401600060405180830381600087803b158015610cc257600080fd5b505af1158015610cd6573d6000803e3d6000fd5b505050508080610ce590611ea2565b915050610c31565b508082846001600160a01b03167f82e416ba72d10e709b5de7ac16f5f49ff1d94f22d55bf582d353d3c313a1e8dd60405160405180910390a4505050565b6000546001600160a01b03163314610d555760405162461bcd60e51b815260040161051590611ed2565b600260015403610da75760405162461bcd60e51b815260206004820152601f60248201527f5265656e7472616e637947756172643a207265656e7472616e742063616c6c006044820152606401610515565b60026001556001600160a01b0382166323b872dd30610dce6000546001600160a01b031690565b6040516001600160e01b031960e085901b1681526001600160a01b0392831660048201529116602482015260448101849052606401600060405180830381600087803b158015610e1d57600080fd5b505af1158015610e31573d6000803e3d6000fd5b50506001805550505050565b6000546001600160a01b03163314610e675760405162461bcd60e51b815260040161051590611ed2565b6361e56850421115610e8b5760405162461bcd60e51b815260040161051590611f07565b828114610eda5760405162461bcd60e51b815260206004820181905260248201527f486f70725374616b653a204c656e67746820646f6573206e6f74206d617463686044820152606401610515565b60005b83811015610fd3576000858583818110610ef957610ef9611f56565b9050602002016020810190610f0e9190611c8a565b9050610f19816113b2565b838383818110610f2b57610f2b611f56565b9050602002013560096000836001600160a01b03166001600160a01b031681526020019081526020016000206001016000828254610f699190611e4d565b909155508490508383818110610f8157610f81611f56565b905060200201356000826001600160a01b03167f1449c6dd7851abc30abf37f57715f492010519147cc2652fbc38202c18a6ee9060405160405180910390a45080610fcb81611ea2565b915050610edd565b5050505050565b6000546001600160a01b031633146110045760405162461bcd60e51b815260040161051590611ed2565b6002600154036110565760405162461bcd60e51b815260206004820152601f60248201527f5265656e7472616e637947756172643a207265656e7472616e742063616c6c006044820152606401610515565b60026001819055546000906001600160a01b03908116908316036110f457600a546002546040516370a0823160e01b81523060048201526001600160a01b03909116906370a0823190602401602060405180830381865afa1580156110bf573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906110e39190611e65565b6110ed9190611ebb565b905061115f565b6040516370a0823160e01b81523060048201526001600160a01b038316906370a0823190602401602060405180830381865afa158015611138573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061115c9190611e65565b90505b6111856111746000546001600160a01b031690565b6001600160a01b03841690836115dd565b505060018055565b600061119882611634565b92915050565b6000546001600160a01b031633146111c85760405162461bcd60e51b815260040161051590611ed2565b6111d2600061188e565b565b6002546000906001600160a01b031633146112465760405162461bcd60e51b815260206004820152602c60248201527f486f70725374616b653a204f6e6c7920616363657074204c4f434b5f544f4b4560448201526b4e20696e207374616b696e6760a01b6064820152608401610515565b6361e5685042111561126a5760405162461bcd60e51b815260040161051590611f07565b611273846113b2565b6001600160a01b0384166000908152600960205260408120805485929061129b908490611e4d565b9250508190555082600a60008282546112b49190611e4d565b909155505060405160009084906001600160a01b038716907f1449c6dd7851abc30abf37f57715f492010519147cc2652fbc38202c18a6ee90908490a45060015b9392505050565b611305816113b2565b50565b611311816113b2565b6113058161143d565b6000546001600160a01b031633146113445760405162461bcd60e51b815260040161051590611ed2565b6001600160a01b0381166113a95760405162461bcd60e51b815260206004820152602660248201527f4f776e61626c653a206e6577206f776e657220697320746865207a65726f206160448201526564647265737360d01b6064820152608401610515565b6113058161188e565b60006113bd82611634565b6001600160a01b0383166000908152600960205260408120600301805492935083929091906113ed908490611e4d565b90915550506001600160a01b03821660008181526009602052604080822042600290910155518392917f99869d968ca3581a661f31abb3a6aa70ccec5cdc49855eab174cf9e00a2462db91a35050565b6001600160a01b0381166000908152600960209081526040808320815160a08101835281548152600182015493810193909352600281015491830191909152600381015460608301819052600490910154608083018190529192916114a191611ebb565b9050600081116114f35760405162461bcd60e51b815260206004820152601b60248201527f486f70725374616b653a204e6f7468696e6720746f20636c61696d00000000006044820152606401610515565b6001600160a01b03831660009081526009602052604090206003810154600490910155600b548111156115745760405162461bcd60e51b8152602060048201526024808201527f486f70725374616b653a20496e73756666696369656e7420726577617264207060448201526337b7b61760e11b6064820152608401610515565b80600b60008282546115869190611ebb565b90915550506003546115a2906001600160a01b031684836115dd565b60405181906001600160a01b038516907fd8138f8a3f377c5259ca548e70e4c2de94f129f5a11036a15b69513cba2b426a90600090a3505050565b604080516001600160a01b038416602482015260448082018490528251808303909101815260649091019091526020810180516001600160e01b031663a9059cbb60e01b17905261162f9084906118de565b505050565b6001600160a01b0381166000908152600960209081526040808320815160a08101835281548152600182015493810193909352600281015491830191909152600381015460608301526004015460808201526360fff540421115806116a157506361e56850816040015110155b156116af5750600092915050565b80516000906116c19061169b90611f6c565b905060005b6001600160a01b0385166000908152600860205260409020548110156117b9576001600160a01b0385811660009081526007602090815260408083208584529091528082205460048054925163225b377d60e21b81529081018290529093919091169063896cddf4906024016040805180830381865afa15801561174e573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906117729190611e7e565b508551909150819061178e9069d3c21bcecceda10000006119b0565b6117989190611f6c565b6117a29085611e4d565b9350505080806117b190611ea2565b9150506116c6565b5064e8d4a510006117e96361e568506117e3636128d3c086604001516119c690919063ffffffff16565b906119b0565b6117ff6361e568506117e342636128d3c06119c6565b6118099190611ebb565b611b78846020015161181b9190611f6c565b6118259190611f6c565b6118486361e568506117e36360fff54087604001516119c690919063ffffffff16565b61185e6361e568506117e3426360fff5406119c6565b6118689190611ebb565b6118729084611f6c565b61187c9190611e4d565b6118869190611f8b565b949350505050565b600080546001600160a01b038381166001600160a01b0319831681178455604051919092169283917f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09190a35050565b6000611933826040518060400160405280602081526020017f5361666545524332303a206c6f772d6c6576656c2063616c6c206661696c6564815250856001600160a01b03166119d69092919063ffffffff16565b80519091501561162f57808060200190518101906119519190611fad565b61162f5760405162461bcd60e51b815260206004820152602a60248201527f5361666545524332303a204552433230206f7065726174696f6e20646964206e6044820152691bdd081cdd58d8d9595960b21b6064820152608401610515565b60008183106119bf57816112f5565b5090919050565b6000818310156119bf57816112f5565b6060611886848460008585843b611a2f5760405162461bcd60e51b815260206004820152601d60248201527f416464726573733a2063616c6c20746f206e6f6e2d636f6e74726163740000006044820152606401610515565b600080866001600160a01b03168587604051611a4b9190611fff565b60006040518083038185875af1925050503d8060008114611a88576040519150601f19603f3d011682016040523d82523d6000602084013e611a8d565b606091505b5091509150611a9d828286611aa8565b979650505050505050565b60608315611ab75750816112f5565b825115611ac75782518084602001fd5b8160405162461bcd60e51b8152600401610515919061201b565b80356001600160a01b0381168114611af857600080fd5b919050565b60008083601f840112611b0f57600080fd5b50813567ffffffffffffffff811115611b2757600080fd5b602083019150836020828501011115611b3f57600080fd5b9250929050565b60008060008060008060008060c0898b031215611b6257600080fd5b611b6b89611ae1565b9750611b7960208a01611ae1565b9650611b8760408a01611ae1565b955060608901359450608089013567ffffffffffffffff80821115611bab57600080fd5b611bb78c838d01611afd565b909650945060a08b0135915080821115611bd057600080fd5b50611bdd8b828c01611afd565b999c989b5096995094979396929594505050565b60008060408385031215611c0457600080fd5b611c0d83611ae1565b946020939093013593505050565b600080600080600060808688031215611c3357600080fd5b611c3c86611ae1565b9450611c4a60208701611ae1565b935060408601359250606086013567ffffffffffffffff811115611c6d57600080fd5b611c7988828901611afd565b969995985093965092949392505050565b600060208284031215611c9c57600080fd5b6112f582611ae1565b60008083601f840112611cb757600080fd5b50813567ffffffffffffffff811115611ccf57600080fd5b6020830191508360208260051b8501011115611b3f57600080fd5b60008060008060408587031215611d0057600080fd5b843567ffffffffffffffff80821115611d1857600080fd5b611d2488838901611ca5565b90965094506020870135915080821115611d3d57600080fd5b50611d4a87828801611ca5565b95989497509550505050565b634e487b7160e01b600052604160045260246000fd5b600080600060608486031215611d8157600080fd5b611d8a84611ae1565b925060208401359150604084013567ffffffffffffffff80821115611dae57600080fd5b818601915086601f830112611dc257600080fd5b813581811115611dd457611dd4611d56565b604051601f8201601f19908116603f01168101908382118183101715611dfc57611dfc611d56565b81604052828152896020848701011115611e1557600080fd5b8260208601602083013760006020848301015280955050505050509250925092565b634e487b7160e01b600052601160045260246000fd5b60008219821115611e6057611e60611e37565b500190565b600060208284031215611e7757600080fd5b5051919050565b60008060408385031215611e9157600080fd5b505080516020909101519092909150565b600060018201611eb457611eb4611e37565b5060010190565b600082821015611ecd57611ecd611e37565b500390565b6020808252818101527f4f776e61626c653a2063616c6c6572206973206e6f7420746865206f776e6572604082015260600190565b6020808252602f908201527f486f70725374616b653a2050726f6772616d20656e6465642c2063616e6e6f7460408201526e1039ba30b5b29030b73cb6b7b9329760891b606082015260800190565b634e487b7160e01b600052603260045260246000fd5b6000816000190483118215151615611f8657611f86611e37565b500290565b600082611fa857634e487b7160e01b600052601260045260246000fd5b500490565b600060208284031215611fbf57600080fd5b815180151581146112f557600080fd5b60005b83811015611fea578181015183820152602001611fd2565b83811115611ff9576000848401525b50505050565b60008251612011818460208701611fcf565b9190910192915050565b602081526000825180602084015261203a816040850160208701611fcf565b601f01601f1916919091016040019291505056fea26469706673582212200889b558ecc8fdb671da0d87c14909591b5c3f2380e05e616f3951850f8da6e164736f6c634300080d0033" . parse () . expect ("invalid bytecode")
        });
    pub struct HoprStake<M>(ethers::contract::Contract<M>);
    impl<M> Clone for HoprStake<M> {
        fn clone(&self) -> Self {
            HoprStake(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for HoprStake<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for HoprStake<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(HoprStake))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> HoprStake<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), HOPRSTAKE_ABI.clone(), client).into()
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
                HOPRSTAKE_ABI.clone(),
                HOPRSTAKE_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        #[doc = "Calls the contract's `BASIC_FACTOR_NUMERATOR` (0x6aa8d4cf) function"]
        pub fn basic_factor_numerator(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([106, 168, 212, 207], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `BASIC_START` (0xf20c9124) function"]
        pub fn basic_start(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([242, 12, 145, 36], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `BOOST_CAP` (0xef0526a2) function"]
        pub fn boost_cap(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([239, 5, 38, 162], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `FACTOR_DENOMINATOR` (0x5ef73d58) function"]
        pub fn factor_denominator(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([94, 247, 61, 88], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `LOCK_TOKEN` (0xd0da6801) function"]
        pub fn lock_token(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([208, 218, 104, 1], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `PROGRAM_END` (0xcbffa3c7) function"]
        pub fn program_end(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([203, 255, 163, 199], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `REWARD_TOKEN` (0x99248ea7) function"]
        pub fn reward_token(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([153, 36, 142, 167], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `SEED_FACTOR_NUMERATOR` (0x2f998468) function"]
        pub fn seed_factor_numerator(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([47, 153, 132, 104], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `SEED_START` (0x112376dc) function"]
        pub fn seed_start(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([17, 35, 118, 220], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `accounts` (0x5e5c06e2) function"]
        pub fn accounts(
            &self,
            p0: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<
            M,
            (
                ethers::core::types::U256,
                ethers::core::types::U256,
                ethers::core::types::U256,
                ethers::core::types::U256,
                ethers::core::types::U256,
            ),
        > {
            self.0
                .method_hash([94, 92, 6, 226], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `availableReward` (0x4ad84b34) function"]
        pub fn available_reward(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([74, 216, 75, 52], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `claimRewards` (0xef5cfb8c) function"]
        pub fn claim_rewards(
            &self,
            account: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([239, 92, 251, 140], account)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getCumulatedRewardsIncrement` (0x70af0934) function"]
        pub fn get_cumulated_rewards_increment(
            &self,
            account: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([112, 175, 9, 52], account)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `lock` (0x5c3c71f4) function"]
        pub fn lock(
            &self,
            investors: ::std::vec::Vec<ethers::core::types::Address>,
            caps: ::std::vec::Vec<ethers::core::types::U256>,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([92, 60, 113, 244], (investors, caps))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `nftContract` (0xd56d229d) function"]
        pub fn nft_contract(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([213, 109, 34, 157], ())
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
        #[doc = "Calls the contract's `redeemedFactor` (0x81128c1d) function"]
        pub fn redeemed_factor(
            &self,
            p0: ethers::core::types::Address,
            p1: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([129, 18, 140, 29], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `redeemedFactorIndex` (0x1f014d83) function"]
        pub fn redeemed_factor_index(
            &self,
            p0: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([31, 1, 77, 131], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `redeemedNft` (0x0a1a257a) function"]
        pub fn redeemed_nft(
            &self,
            p0: ethers::core::types::Address,
            p1: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([10, 26, 37, 122], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `redeemedNftIndex` (0xd0c02d63) function"]
        pub fn redeemed_nft_index(
            &self,
            p0: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([208, 192, 45, 99], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `renounceOwnership` (0x715018a6) function"]
        pub fn renounce_ownership(&self) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([113, 80, 24, 166], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `sync` (0xa5841194) function"]
        pub fn sync(
            &self,
            account: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([165, 132, 17, 148], account)
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
        #[doc = "Calls the contract's `totalLocked` (0x56891412) function"]
        pub fn total_locked(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([86, 137, 20, 18], ())
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
        #[doc = "Calls the contract's `unlock` (0x2f6c493c) function"]
        pub fn unlock(
            &self,
            account: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([47, 108, 73, 60], account)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Gets the contract's `Claimed` event"]
        pub fn claimed_filter(&self) -> ethers::contract::builders::Event<M, ClaimedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `OwnershipTransferred` event"]
        pub fn ownership_transferred_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, OwnershipTransferredFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Redeemed` event"]
        pub fn redeemed_filter(&self) -> ethers::contract::builders::Event<M, RedeemedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Released` event"]
        pub fn released_filter(&self) -> ethers::contract::builders::Event<M, ReleasedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `RewardFueled` event"]
        pub fn reward_fueled_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, RewardFueledFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Staked` event"]
        pub fn staked_filter(&self) -> ethers::contract::builders::Event<M, StakedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Sync` event"]
        pub fn sync_filter(&self) -> ethers::contract::builders::Event<M, SyncFilter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(&self) -> ethers::contract::builders::Event<M, HoprStakeEvents> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for HoprStake<M> {
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
    #[ethevent(name = "Claimed", abi = "Claimed(address,uint256)")]
    pub struct ClaimedFilter {
        #[ethevent(indexed)]
        pub account: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub reward_amount: ethers::core::types::U256,
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
    #[ethevent(name = "Redeemed", abi = "Redeemed(address,uint256,bool)")]
    pub struct RedeemedFilter {
        #[ethevent(indexed)]
        pub account: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub boost_token_id: ethers::core::types::U256,
        #[ethevent(indexed)]
        pub factor_registered: bool,
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
    #[ethevent(name = "Released", abi = "Released(address,uint256,uint256)")]
    pub struct ReleasedFilter {
        #[ethevent(indexed)]
        pub account: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub actual_amount: ethers::core::types::U256,
        #[ethevent(indexed)]
        pub virtual_amount: ethers::core::types::U256,
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
    #[ethevent(name = "RewardFueled", abi = "RewardFueled(uint256)")]
    pub struct RewardFueledFilter {
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
    #[ethevent(name = "Staked", abi = "Staked(address,uint256,uint256)")]
    pub struct StakedFilter {
        #[ethevent(indexed)]
        pub account: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub actual_amount: ethers::core::types::U256,
        #[ethevent(indexed)]
        pub virtual_amount: ethers::core::types::U256,
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
    #[ethevent(name = "Sync", abi = "Sync(address,uint256)")]
    pub struct SyncFilter {
        #[ethevent(indexed)]
        pub account: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub increment: ethers::core::types::U256,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum HoprStakeEvents {
        ClaimedFilter(ClaimedFilter),
        OwnershipTransferredFilter(OwnershipTransferredFilter),
        RedeemedFilter(RedeemedFilter),
        ReleasedFilter(ReleasedFilter),
        RewardFueledFilter(RewardFueledFilter),
        StakedFilter(StakedFilter),
        SyncFilter(SyncFilter),
    }
    impl ethers::contract::EthLogDecode for HoprStakeEvents {
        fn decode_log(
            log: &ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = ClaimedFilter::decode_log(log) {
                return Ok(HoprStakeEvents::ClaimedFilter(decoded));
            }
            if let Ok(decoded) = OwnershipTransferredFilter::decode_log(log) {
                return Ok(HoprStakeEvents::OwnershipTransferredFilter(decoded));
            }
            if let Ok(decoded) = RedeemedFilter::decode_log(log) {
                return Ok(HoprStakeEvents::RedeemedFilter(decoded));
            }
            if let Ok(decoded) = ReleasedFilter::decode_log(log) {
                return Ok(HoprStakeEvents::ReleasedFilter(decoded));
            }
            if let Ok(decoded) = RewardFueledFilter::decode_log(log) {
                return Ok(HoprStakeEvents::RewardFueledFilter(decoded));
            }
            if let Ok(decoded) = StakedFilter::decode_log(log) {
                return Ok(HoprStakeEvents::StakedFilter(decoded));
            }
            if let Ok(decoded) = SyncFilter::decode_log(log) {
                return Ok(HoprStakeEvents::SyncFilter(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for HoprStakeEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                HoprStakeEvents::ClaimedFilter(element) => element.fmt(f),
                HoprStakeEvents::OwnershipTransferredFilter(element) => element.fmt(f),
                HoprStakeEvents::RedeemedFilter(element) => element.fmt(f),
                HoprStakeEvents::ReleasedFilter(element) => element.fmt(f),
                HoprStakeEvents::RewardFueledFilter(element) => element.fmt(f),
                HoprStakeEvents::StakedFilter(element) => element.fmt(f),
                HoprStakeEvents::SyncFilter(element) => element.fmt(f),
            }
        }
    }
    #[doc = "Container type for all input parameters for the `BASIC_FACTOR_NUMERATOR` function with signature `BASIC_FACTOR_NUMERATOR()` and selector `[106, 168, 212, 207]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "BASIC_FACTOR_NUMERATOR", abi = "BASIC_FACTOR_NUMERATOR()")]
    pub struct BasicFactorNumeratorCall;
    #[doc = "Container type for all input parameters for the `BASIC_START` function with signature `BASIC_START()` and selector `[242, 12, 145, 36]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "BASIC_START", abi = "BASIC_START()")]
    pub struct BasicStartCall;
    #[doc = "Container type for all input parameters for the `BOOST_CAP` function with signature `BOOST_CAP()` and selector `[239, 5, 38, 162]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "BOOST_CAP", abi = "BOOST_CAP()")]
    pub struct BoostCapCall;
    #[doc = "Container type for all input parameters for the `FACTOR_DENOMINATOR` function with signature `FACTOR_DENOMINATOR()` and selector `[94, 247, 61, 88]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "FACTOR_DENOMINATOR", abi = "FACTOR_DENOMINATOR()")]
    pub struct FactorDenominatorCall;
    #[doc = "Container type for all input parameters for the `LOCK_TOKEN` function with signature `LOCK_TOKEN()` and selector `[208, 218, 104, 1]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "LOCK_TOKEN", abi = "LOCK_TOKEN()")]
    pub struct LockTokenCall;
    #[doc = "Container type for all input parameters for the `PROGRAM_END` function with signature `PROGRAM_END()` and selector `[203, 255, 163, 199]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "PROGRAM_END", abi = "PROGRAM_END()")]
    pub struct ProgramEndCall;
    #[doc = "Container type for all input parameters for the `REWARD_TOKEN` function with signature `REWARD_TOKEN()` and selector `[153, 36, 142, 167]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "REWARD_TOKEN", abi = "REWARD_TOKEN()")]
    pub struct RewardTokenCall;
    #[doc = "Container type for all input parameters for the `SEED_FACTOR_NUMERATOR` function with signature `SEED_FACTOR_NUMERATOR()` and selector `[47, 153, 132, 104]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "SEED_FACTOR_NUMERATOR", abi = "SEED_FACTOR_NUMERATOR()")]
    pub struct SeedFactorNumeratorCall;
    #[doc = "Container type for all input parameters for the `SEED_START` function with signature `SEED_START()` and selector `[17, 35, 118, 220]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "SEED_START", abi = "SEED_START()")]
    pub struct SeedStartCall;
    #[doc = "Container type for all input parameters for the `accounts` function with signature `accounts(address)` and selector `[94, 92, 6, 226]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "accounts", abi = "accounts(address)")]
    pub struct AccountsCall(pub ethers::core::types::Address);
    #[doc = "Container type for all input parameters for the `availableReward` function with signature `availableReward()` and selector `[74, 216, 75, 52]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "availableReward", abi = "availableReward()")]
    pub struct AvailableRewardCall;
    #[doc = "Container type for all input parameters for the `claimRewards` function with signature `claimRewards(address)` and selector `[239, 92, 251, 140]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "claimRewards", abi = "claimRewards(address)")]
    pub struct ClaimRewardsCall {
        pub account: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `getCumulatedRewardsIncrement` function with signature `getCumulatedRewardsIncrement(address)` and selector `[112, 175, 9, 52]`"]
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
        name = "getCumulatedRewardsIncrement",
        abi = "getCumulatedRewardsIncrement(address)"
    )]
    pub struct GetCumulatedRewardsIncrementCall {
        pub account: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `lock` function with signature `lock(address[],uint256[])` and selector `[92, 60, 113, 244]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "lock", abi = "lock(address[],uint256[])")]
    pub struct LockCall {
        pub investors: ::std::vec::Vec<ethers::core::types::Address>,
        pub caps: ::std::vec::Vec<ethers::core::types::U256>,
    }
    #[doc = "Container type for all input parameters for the `nftContract` function with signature `nftContract()` and selector `[213, 109, 34, 157]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "nftContract", abi = "nftContract()")]
    pub struct NftContractCall;
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
    #[doc = "Container type for all input parameters for the `redeemedFactor` function with signature `redeemedFactor(address,uint256)` and selector `[129, 18, 140, 29]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "redeemedFactor", abi = "redeemedFactor(address,uint256)")]
    pub struct RedeemedFactorCall(
        pub ethers::core::types::Address,
        pub ethers::core::types::U256,
    );
    #[doc = "Container type for all input parameters for the `redeemedFactorIndex` function with signature `redeemedFactorIndex(address)` and selector `[31, 1, 77, 131]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "redeemedFactorIndex", abi = "redeemedFactorIndex(address)")]
    pub struct RedeemedFactorIndexCall(pub ethers::core::types::Address);
    #[doc = "Container type for all input parameters for the `redeemedNft` function with signature `redeemedNft(address,uint256)` and selector `[10, 26, 37, 122]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "redeemedNft", abi = "redeemedNft(address,uint256)")]
    pub struct RedeemedNftCall(
        pub ethers::core::types::Address,
        pub ethers::core::types::U256,
    );
    #[doc = "Container type for all input parameters for the `redeemedNftIndex` function with signature `redeemedNftIndex(address)` and selector `[208, 192, 45, 99]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "redeemedNftIndex", abi = "redeemedNftIndex(address)")]
    pub struct RedeemedNftIndexCall(pub ethers::core::types::Address);
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
    #[doc = "Container type for all input parameters for the `sync` function with signature `sync(address)` and selector `[165, 132, 17, 148]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "sync", abi = "sync(address)")]
    pub struct SyncCall {
        pub account: ethers::core::types::Address,
    }
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
    #[doc = "Container type for all input parameters for the `totalLocked` function with signature `totalLocked()` and selector `[86, 137, 20, 18]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "totalLocked", abi = "totalLocked()")]
    pub struct TotalLockedCall;
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
    #[doc = "Container type for all input parameters for the `unlock` function with signature `unlock(address)` and selector `[47, 108, 73, 60]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "unlock", abi = "unlock(address)")]
    pub struct UnlockCall {
        pub account: ethers::core::types::Address,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum HoprStakeCalls {
        BasicFactorNumerator(BasicFactorNumeratorCall),
        BasicStart(BasicStartCall),
        BoostCap(BoostCapCall),
        FactorDenominator(FactorDenominatorCall),
        LockToken(LockTokenCall),
        ProgramEnd(ProgramEndCall),
        RewardToken(RewardTokenCall),
        SeedFactorNumerator(SeedFactorNumeratorCall),
        SeedStart(SeedStartCall),
        Accounts(AccountsCall),
        AvailableReward(AvailableRewardCall),
        ClaimRewards(ClaimRewardsCall),
        GetCumulatedRewardsIncrement(GetCumulatedRewardsIncrementCall),
        Lock(LockCall),
        NftContract(NftContractCall),
        OnERC721Received(OnERC721ReceivedCall),
        OnTokenTransfer(OnTokenTransferCall),
        Owner(OwnerCall),
        ReclaimErc20Tokens(ReclaimErc20TokensCall),
        ReclaimErc721Tokens(ReclaimErc721TokensCall),
        RedeemedFactor(RedeemedFactorCall),
        RedeemedFactorIndex(RedeemedFactorIndexCall),
        RedeemedNft(RedeemedNftCall),
        RedeemedNftIndex(RedeemedNftIndexCall),
        RenounceOwnership(RenounceOwnershipCall),
        Sync(SyncCall),
        TokensReceived(TokensReceivedCall),
        TotalLocked(TotalLockedCall),
        TransferOwnership(TransferOwnershipCall),
        Unlock(UnlockCall),
    }
    impl ethers::core::abi::AbiDecode for HoprStakeCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <BasicFactorNumeratorCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::BasicFactorNumerator(decoded));
            }
            if let Ok(decoded) =
                <BasicStartCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::BasicStart(decoded));
            }
            if let Ok(decoded) =
                <BoostCapCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::BoostCap(decoded));
            }
            if let Ok(decoded) =
                <FactorDenominatorCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::FactorDenominator(decoded));
            }
            if let Ok(decoded) =
                <LockTokenCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::LockToken(decoded));
            }
            if let Ok(decoded) =
                <ProgramEndCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::ProgramEnd(decoded));
            }
            if let Ok(decoded) =
                <RewardTokenCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::RewardToken(decoded));
            }
            if let Ok(decoded) =
                <SeedFactorNumeratorCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::SeedFactorNumerator(decoded));
            }
            if let Ok(decoded) =
                <SeedStartCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::SeedStart(decoded));
            }
            if let Ok(decoded) =
                <AccountsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::Accounts(decoded));
            }
            if let Ok(decoded) =
                <AvailableRewardCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::AvailableReward(decoded));
            }
            if let Ok(decoded) =
                <ClaimRewardsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::ClaimRewards(decoded));
            }
            if let Ok(decoded) =
                <GetCumulatedRewardsIncrementCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(HoprStakeCalls::GetCumulatedRewardsIncrement(decoded));
            }
            if let Ok(decoded) = <LockCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(HoprStakeCalls::Lock(decoded));
            }
            if let Ok(decoded) =
                <NftContractCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::NftContract(decoded));
            }
            if let Ok(decoded) =
                <OnERC721ReceivedCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::OnERC721Received(decoded));
            }
            if let Ok(decoded) =
                <OnTokenTransferCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::OnTokenTransfer(decoded));
            }
            if let Ok(decoded) = <OwnerCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::Owner(decoded));
            }
            if let Ok(decoded) =
                <ReclaimErc20TokensCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::ReclaimErc20Tokens(decoded));
            }
            if let Ok(decoded) =
                <ReclaimErc721TokensCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::ReclaimErc721Tokens(decoded));
            }
            if let Ok(decoded) =
                <RedeemedFactorCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::RedeemedFactor(decoded));
            }
            if let Ok(decoded) =
                <RedeemedFactorIndexCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::RedeemedFactorIndex(decoded));
            }
            if let Ok(decoded) =
                <RedeemedNftCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::RedeemedNft(decoded));
            }
            if let Ok(decoded) =
                <RedeemedNftIndexCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::RedeemedNftIndex(decoded));
            }
            if let Ok(decoded) =
                <RenounceOwnershipCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::RenounceOwnership(decoded));
            }
            if let Ok(decoded) = <SyncCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(HoprStakeCalls::Sync(decoded));
            }
            if let Ok(decoded) =
                <TokensReceivedCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::TokensReceived(decoded));
            }
            if let Ok(decoded) =
                <TotalLockedCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::TotalLocked(decoded));
            }
            if let Ok(decoded) =
                <TransferOwnershipCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::TransferOwnership(decoded));
            }
            if let Ok(decoded) = <UnlockCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakeCalls::Unlock(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for HoprStakeCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                HoprStakeCalls::BasicFactorNumerator(element) => element.encode(),
                HoprStakeCalls::BasicStart(element) => element.encode(),
                HoprStakeCalls::BoostCap(element) => element.encode(),
                HoprStakeCalls::FactorDenominator(element) => element.encode(),
                HoprStakeCalls::LockToken(element) => element.encode(),
                HoprStakeCalls::ProgramEnd(element) => element.encode(),
                HoprStakeCalls::RewardToken(element) => element.encode(),
                HoprStakeCalls::SeedFactorNumerator(element) => element.encode(),
                HoprStakeCalls::SeedStart(element) => element.encode(),
                HoprStakeCalls::Accounts(element) => element.encode(),
                HoprStakeCalls::AvailableReward(element) => element.encode(),
                HoprStakeCalls::ClaimRewards(element) => element.encode(),
                HoprStakeCalls::GetCumulatedRewardsIncrement(element) => element.encode(),
                HoprStakeCalls::Lock(element) => element.encode(),
                HoprStakeCalls::NftContract(element) => element.encode(),
                HoprStakeCalls::OnERC721Received(element) => element.encode(),
                HoprStakeCalls::OnTokenTransfer(element) => element.encode(),
                HoprStakeCalls::Owner(element) => element.encode(),
                HoprStakeCalls::ReclaimErc20Tokens(element) => element.encode(),
                HoprStakeCalls::ReclaimErc721Tokens(element) => element.encode(),
                HoprStakeCalls::RedeemedFactor(element) => element.encode(),
                HoprStakeCalls::RedeemedFactorIndex(element) => element.encode(),
                HoprStakeCalls::RedeemedNft(element) => element.encode(),
                HoprStakeCalls::RedeemedNftIndex(element) => element.encode(),
                HoprStakeCalls::RenounceOwnership(element) => element.encode(),
                HoprStakeCalls::Sync(element) => element.encode(),
                HoprStakeCalls::TokensReceived(element) => element.encode(),
                HoprStakeCalls::TotalLocked(element) => element.encode(),
                HoprStakeCalls::TransferOwnership(element) => element.encode(),
                HoprStakeCalls::Unlock(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for HoprStakeCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                HoprStakeCalls::BasicFactorNumerator(element) => element.fmt(f),
                HoprStakeCalls::BasicStart(element) => element.fmt(f),
                HoprStakeCalls::BoostCap(element) => element.fmt(f),
                HoprStakeCalls::FactorDenominator(element) => element.fmt(f),
                HoprStakeCalls::LockToken(element) => element.fmt(f),
                HoprStakeCalls::ProgramEnd(element) => element.fmt(f),
                HoprStakeCalls::RewardToken(element) => element.fmt(f),
                HoprStakeCalls::SeedFactorNumerator(element) => element.fmt(f),
                HoprStakeCalls::SeedStart(element) => element.fmt(f),
                HoprStakeCalls::Accounts(element) => element.fmt(f),
                HoprStakeCalls::AvailableReward(element) => element.fmt(f),
                HoprStakeCalls::ClaimRewards(element) => element.fmt(f),
                HoprStakeCalls::GetCumulatedRewardsIncrement(element) => element.fmt(f),
                HoprStakeCalls::Lock(element) => element.fmt(f),
                HoprStakeCalls::NftContract(element) => element.fmt(f),
                HoprStakeCalls::OnERC721Received(element) => element.fmt(f),
                HoprStakeCalls::OnTokenTransfer(element) => element.fmt(f),
                HoprStakeCalls::Owner(element) => element.fmt(f),
                HoprStakeCalls::ReclaimErc20Tokens(element) => element.fmt(f),
                HoprStakeCalls::ReclaimErc721Tokens(element) => element.fmt(f),
                HoprStakeCalls::RedeemedFactor(element) => element.fmt(f),
                HoprStakeCalls::RedeemedFactorIndex(element) => element.fmt(f),
                HoprStakeCalls::RedeemedNft(element) => element.fmt(f),
                HoprStakeCalls::RedeemedNftIndex(element) => element.fmt(f),
                HoprStakeCalls::RenounceOwnership(element) => element.fmt(f),
                HoprStakeCalls::Sync(element) => element.fmt(f),
                HoprStakeCalls::TokensReceived(element) => element.fmt(f),
                HoprStakeCalls::TotalLocked(element) => element.fmt(f),
                HoprStakeCalls::TransferOwnership(element) => element.fmt(f),
                HoprStakeCalls::Unlock(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<BasicFactorNumeratorCall> for HoprStakeCalls {
        fn from(var: BasicFactorNumeratorCall) -> Self {
            HoprStakeCalls::BasicFactorNumerator(var)
        }
    }
    impl ::std::convert::From<BasicStartCall> for HoprStakeCalls {
        fn from(var: BasicStartCall) -> Self {
            HoprStakeCalls::BasicStart(var)
        }
    }
    impl ::std::convert::From<BoostCapCall> for HoprStakeCalls {
        fn from(var: BoostCapCall) -> Self {
            HoprStakeCalls::BoostCap(var)
        }
    }
    impl ::std::convert::From<FactorDenominatorCall> for HoprStakeCalls {
        fn from(var: FactorDenominatorCall) -> Self {
            HoprStakeCalls::FactorDenominator(var)
        }
    }
    impl ::std::convert::From<LockTokenCall> for HoprStakeCalls {
        fn from(var: LockTokenCall) -> Self {
            HoprStakeCalls::LockToken(var)
        }
    }
    impl ::std::convert::From<ProgramEndCall> for HoprStakeCalls {
        fn from(var: ProgramEndCall) -> Self {
            HoprStakeCalls::ProgramEnd(var)
        }
    }
    impl ::std::convert::From<RewardTokenCall> for HoprStakeCalls {
        fn from(var: RewardTokenCall) -> Self {
            HoprStakeCalls::RewardToken(var)
        }
    }
    impl ::std::convert::From<SeedFactorNumeratorCall> for HoprStakeCalls {
        fn from(var: SeedFactorNumeratorCall) -> Self {
            HoprStakeCalls::SeedFactorNumerator(var)
        }
    }
    impl ::std::convert::From<SeedStartCall> for HoprStakeCalls {
        fn from(var: SeedStartCall) -> Self {
            HoprStakeCalls::SeedStart(var)
        }
    }
    impl ::std::convert::From<AccountsCall> for HoprStakeCalls {
        fn from(var: AccountsCall) -> Self {
            HoprStakeCalls::Accounts(var)
        }
    }
    impl ::std::convert::From<AvailableRewardCall> for HoprStakeCalls {
        fn from(var: AvailableRewardCall) -> Self {
            HoprStakeCalls::AvailableReward(var)
        }
    }
    impl ::std::convert::From<ClaimRewardsCall> for HoprStakeCalls {
        fn from(var: ClaimRewardsCall) -> Self {
            HoprStakeCalls::ClaimRewards(var)
        }
    }
    impl ::std::convert::From<GetCumulatedRewardsIncrementCall> for HoprStakeCalls {
        fn from(var: GetCumulatedRewardsIncrementCall) -> Self {
            HoprStakeCalls::GetCumulatedRewardsIncrement(var)
        }
    }
    impl ::std::convert::From<LockCall> for HoprStakeCalls {
        fn from(var: LockCall) -> Self {
            HoprStakeCalls::Lock(var)
        }
    }
    impl ::std::convert::From<NftContractCall> for HoprStakeCalls {
        fn from(var: NftContractCall) -> Self {
            HoprStakeCalls::NftContract(var)
        }
    }
    impl ::std::convert::From<OnERC721ReceivedCall> for HoprStakeCalls {
        fn from(var: OnERC721ReceivedCall) -> Self {
            HoprStakeCalls::OnERC721Received(var)
        }
    }
    impl ::std::convert::From<OnTokenTransferCall> for HoprStakeCalls {
        fn from(var: OnTokenTransferCall) -> Self {
            HoprStakeCalls::OnTokenTransfer(var)
        }
    }
    impl ::std::convert::From<OwnerCall> for HoprStakeCalls {
        fn from(var: OwnerCall) -> Self {
            HoprStakeCalls::Owner(var)
        }
    }
    impl ::std::convert::From<ReclaimErc20TokensCall> for HoprStakeCalls {
        fn from(var: ReclaimErc20TokensCall) -> Self {
            HoprStakeCalls::ReclaimErc20Tokens(var)
        }
    }
    impl ::std::convert::From<ReclaimErc721TokensCall> for HoprStakeCalls {
        fn from(var: ReclaimErc721TokensCall) -> Self {
            HoprStakeCalls::ReclaimErc721Tokens(var)
        }
    }
    impl ::std::convert::From<RedeemedFactorCall> for HoprStakeCalls {
        fn from(var: RedeemedFactorCall) -> Self {
            HoprStakeCalls::RedeemedFactor(var)
        }
    }
    impl ::std::convert::From<RedeemedFactorIndexCall> for HoprStakeCalls {
        fn from(var: RedeemedFactorIndexCall) -> Self {
            HoprStakeCalls::RedeemedFactorIndex(var)
        }
    }
    impl ::std::convert::From<RedeemedNftCall> for HoprStakeCalls {
        fn from(var: RedeemedNftCall) -> Self {
            HoprStakeCalls::RedeemedNft(var)
        }
    }
    impl ::std::convert::From<RedeemedNftIndexCall> for HoprStakeCalls {
        fn from(var: RedeemedNftIndexCall) -> Self {
            HoprStakeCalls::RedeemedNftIndex(var)
        }
    }
    impl ::std::convert::From<RenounceOwnershipCall> for HoprStakeCalls {
        fn from(var: RenounceOwnershipCall) -> Self {
            HoprStakeCalls::RenounceOwnership(var)
        }
    }
    impl ::std::convert::From<SyncCall> for HoprStakeCalls {
        fn from(var: SyncCall) -> Self {
            HoprStakeCalls::Sync(var)
        }
    }
    impl ::std::convert::From<TokensReceivedCall> for HoprStakeCalls {
        fn from(var: TokensReceivedCall) -> Self {
            HoprStakeCalls::TokensReceived(var)
        }
    }
    impl ::std::convert::From<TotalLockedCall> for HoprStakeCalls {
        fn from(var: TotalLockedCall) -> Self {
            HoprStakeCalls::TotalLocked(var)
        }
    }
    impl ::std::convert::From<TransferOwnershipCall> for HoprStakeCalls {
        fn from(var: TransferOwnershipCall) -> Self {
            HoprStakeCalls::TransferOwnership(var)
        }
    }
    impl ::std::convert::From<UnlockCall> for HoprStakeCalls {
        fn from(var: UnlockCall) -> Self {
            HoprStakeCalls::Unlock(var)
        }
    }
    #[doc = "Container type for all return fields from the `BASIC_FACTOR_NUMERATOR` function with signature `BASIC_FACTOR_NUMERATOR()` and selector `[106, 168, 212, 207]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct BasicFactorNumeratorReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `BASIC_START` function with signature `BASIC_START()` and selector `[242, 12, 145, 36]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct BasicStartReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `BOOST_CAP` function with signature `BOOST_CAP()` and selector `[239, 5, 38, 162]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct BoostCapReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `FACTOR_DENOMINATOR` function with signature `FACTOR_DENOMINATOR()` and selector `[94, 247, 61, 88]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct FactorDenominatorReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `LOCK_TOKEN` function with signature `LOCK_TOKEN()` and selector `[208, 218, 104, 1]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct LockTokenReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `PROGRAM_END` function with signature `PROGRAM_END()` and selector `[203, 255, 163, 199]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ProgramEndReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `REWARD_TOKEN` function with signature `REWARD_TOKEN()` and selector `[153, 36, 142, 167]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct RewardTokenReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `SEED_FACTOR_NUMERATOR` function with signature `SEED_FACTOR_NUMERATOR()` and selector `[47, 153, 132, 104]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct SeedFactorNumeratorReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `SEED_START` function with signature `SEED_START()` and selector `[17, 35, 118, 220]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct SeedStartReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `accounts` function with signature `accounts(address)` and selector `[94, 92, 6, 226]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct AccountsReturn {
        pub actual_locked_token_amount: ethers::core::types::U256,
        pub virtual_locked_token_amount: ethers::core::types::U256,
        pub last_sync_timestamp: ethers::core::types::U256,
        pub cumulated_rewards: ethers::core::types::U256,
        pub claimed_rewards: ethers::core::types::U256,
    }
    #[doc = "Container type for all return fields from the `availableReward` function with signature `availableReward()` and selector `[74, 216, 75, 52]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct AvailableRewardReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `getCumulatedRewardsIncrement` function with signature `getCumulatedRewardsIncrement(address)` and selector `[112, 175, 9, 52]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetCumulatedRewardsIncrementReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `nftContract` function with signature `nftContract()` and selector `[213, 109, 34, 157]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct NftContractReturn(pub ethers::core::types::Address);
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
    #[doc = "Container type for all return fields from the `redeemedFactor` function with signature `redeemedFactor(address,uint256)` and selector `[129, 18, 140, 29]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct RedeemedFactorReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `redeemedFactorIndex` function with signature `redeemedFactorIndex(address)` and selector `[31, 1, 77, 131]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct RedeemedFactorIndexReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `redeemedNft` function with signature `redeemedNft(address,uint256)` and selector `[10, 26, 37, 122]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct RedeemedNftReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `redeemedNftIndex` function with signature `redeemedNftIndex(address)` and selector `[208, 192, 45, 99]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct RedeemedNftIndexReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `totalLocked` function with signature `totalLocked()` and selector `[86, 137, 20, 18]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct TotalLockedReturn(pub ethers::core::types::U256);
}
