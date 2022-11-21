pub use hopr_staking_proxy_for_network_registry::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod hopr_staking_proxy_for_network_registry {
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
    #[doc = "HoprStakingProxyForNetworkRegistry was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_stakeContract\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"_newOwner\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"_minStake\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"nftType\",\"type\":\"uint256\",\"components\":[],\"indexed\":true},{\"internalType\":\"string\",\"name\":\"nftRank\",\"type\":\"string\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"NftTypeAndRankAdded\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"nftType\",\"type\":\"uint256\",\"components\":[],\"indexed\":true},{\"internalType\":\"string\",\"name\":\"nftRank\",\"type\":\"string\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"NftTypeAndRankRemoved\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"previousOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"OwnershipTransferred\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"nftType\",\"type\":\"uint256\",\"components\":[],\"indexed\":true},{\"internalType\":\"string\",\"name\":\"nftRank\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"maxRegistration\",\"type\":\"uint256\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"SpecialNftTypeAndRankAdded\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"nftType\",\"type\":\"uint256\",\"components\":[],\"indexed\":true},{\"internalType\":\"string\",\"name\":\"nftRank\",\"type\":\"string\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"SpecialNftTypeAndRankRemoved\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"stakeContract\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"StakeContractUpdated\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"threshold\",\"type\":\"uint256\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"ThresholdUpdated\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"eligibleNftTypeAndRank\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"nftType\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"nftRank\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"maxAllowedRegistrations\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"maxRegistrationsPerSpecialNft\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"owner\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"nftType\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"nftRank\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"ownerAddNftTypeAndRank\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256[]\",\"name\":\"nftTypes\",\"type\":\"uint256[]\",\"components\":[]},{\"internalType\":\"string[]\",\"name\":\"nftRanks\",\"type\":\"string[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"ownerBatchAddNftTypeAndRank\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256[]\",\"name\":\"nftTypes\",\"type\":\"uint256[]\",\"components\":[]},{\"internalType\":\"string[]\",\"name\":\"nftRanks\",\"type\":\"string[]\",\"components\":[]},{\"internalType\":\"uint256[]\",\"name\":\"maxRegistrations\",\"type\":\"uint256[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"ownerBatchAddSpecialNftTypeAndRank\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256[]\",\"name\":\"nftTypes\",\"type\":\"uint256[]\",\"components\":[]},{\"internalType\":\"string[]\",\"name\":\"nftRanks\",\"type\":\"string[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"ownerBatchRemoveNftTypeAndRank\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256[]\",\"name\":\"nftTypes\",\"type\":\"uint256[]\",\"components\":[]},{\"internalType\":\"string[]\",\"name\":\"nftRanks\",\"type\":\"string[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"ownerBatchRemoveSpecialNftTypeAndRank\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"nftType\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"nftRank\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"ownerRemoveNftTypeAndRank\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"newThreshold\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"ownerUpdateThreshold\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"renounceOwnership\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"specialNftTypeAndRank\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"nftType\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"nftRank\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"stakeContract\",\"outputs\":[{\"internalType\":\"contract IHoprStake\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"stakeThreshold\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferOwnership\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_stakeContract\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"updateStakeContract\",\"outputs\":[]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static HOPRSTAKINGPROXYFORNETWORKREGISTRY_ABI: ethers::contract::Lazy<
        ethers::core::abi::Abi,
    > = ethers::contract::Lazy::new(|| {
        ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
    });
    #[doc = r" Bytecode of the #name contract"]
    pub static HOPRSTAKINGPROXYFORNETWORKREGISTRY_BYTECODE: ethers::contract::Lazy<
        ethers::core::types::Bytes,
    > = ethers::contract::Lazy::new(|| {
        "0x60806040523480156200001157600080fd5b5060405162001f3b38038062001f3b833981016040819052620000349162000145565b6200003f336200008e565b6200004a83620000de565b62000055826200008e565b600281905560405181907fadfa8ecb21b6962ebcd0adbd9ab985b7b4c5b5eb3b0dead683171565c7bfe17190600090a250505062000186565b600080546001600160a01b038381166001600160a01b0319831681178455604051919092169283917f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09190a35050565b600180546001600160a01b0319166001600160a01b0383169081179091556040517f573bbfa679af6fdcdbd9cf191c5ef3e526599ac2bf75e9177d47adb8530b9c6990600090a250565b80516001600160a01b03811681146200014057600080fd5b919050565b6000806000606084860312156200015b57600080fd5b620001668462000128565b9250620001766020850162000128565b9150604084015190509250925092565b611da580620001966000396000f3fe608060405234801561001057600080fd5b506004361061010b5760003560e01c80639b97076f116100a2578063de626c0e11610071578063de626c0e14610222578063ee50c7c414610235578063f11f77f914610248578063f2fde38b14610251578063fb66ac571461026457600080fd5b80639b97076f146101c8578063b05e8ba9146101db578063b3544e82146101ee578063ba1cef231461020f57600080fd5b80636a3b64b6116100de5780636a3b64b614610189578063715018a61461019c578063830c6cc2146101a45780638da5cb5b146101b757600080fd5b80631a186227146101105780632c3ec80b14610140578063506472cc14610161578063654251eb14610176575b600080fd5b600154610123906001600160a01b031681565b6040516001600160a01b0390911681526020015b60405180910390f35b61015361014e366004611818565b610277565b60405161013792919061187e565b61017461016f3660046118eb565b61032f565b005b61017461018436600461196d565b61047c565b610174610197366004611a28565b6104b4565b6101746106df565b6101746101b2366004611ac2565b610715565b6000546001600160a01b0316610123565b6101746101d636600461196d565b61074b565b6101746101e93660046118eb565b61077f565b6102016101fc366004611ac2565b6108c9565b604051908152602001610137565b61020161021d366004611818565b610c7e565b610153610230366004611818565b610c9f565b610174610243366004611818565b610caf565b61020160025481565b61017461025f366004611ac2565b610d8b565b6101746102723660046118eb565b610e23565b6005818154811061028757600080fd5b600091825260209091206002909102018054600182018054919350906102ac90611aeb565b80601f01602080910402602001604051908101604052809291908181526020018280546102d890611aeb565b80156103255780601f106102fa57610100808354040283529160200191610325565b820191906000526020600020905b81548152906001019060200180831161030857829003601f168201915b5050505050905082565b6000546001600160a01b031633146103625760405162461bcd60e51b815260040161035990611b25565b60405180910390fd5b8281146103de5760405162461bcd60e51b81526020600482015260506024820152600080516020611d5083398151915260448201527f72793a206f776e657242617463684164644e667454797065416e6452616e6b2060648201526f0d8cadccee8d0e640dad2e6dac2e8c6d60831b608482015260a401610359565b60005b83811015610475576104638585838181106103fe576103fe611b5a565b9050602002013584848481811061041757610417611b5a565b90506020028101906104299190611b70565b8080601f016020809104026020016040519081016040528093929190818152602001838380828437600092019190915250610f6392505050565b8061046d81611bcd565b9150506103e1565b5050505050565b6000546001600160a01b031633146104a65760405162461bcd60e51b815260040161035990611b25565b6104b082826110c4565b5050565b6000546001600160a01b031633146104de5760405162461bcd60e51b815260040161035990611b25565b84831461057d5760405162461bcd60e51b815260206004820152606d6024820152600080516020611d5083398151915260448201527f72793a206f776e657242617463684164645370656369616c4e6674547970654160648201527f6e6452616e6b206e6674547970657320616e64206e667452616e6b73206c656e60848201526c0cee8d0e640dad2e6dac2e8c6d609b1b60a482015260c401610359565b8481146106245760405162461bcd60e51b81526020600482015260756024820152600080516020611d5083398151915260448201527f72793a206f776e657242617463684164645370656369616c4e6674547970654160648201527f6e6452616e6b206e6674547970657320616e64206d61785265676973747261746084820152740d2dedce640d8cadccee8d0e640dad2e6dac2e8c6d605b1b60a482015260c401610359565b60005b858110156106d6576106c487878381811061064457610644611b5a565b9050602002013586868481811061065d5761065d611b5a565b905060200281019061066f9190611b70565b8080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920191909152508892508791508690508181106106b8576106b8611b5a565b90506020020135611245565b806106ce81611bcd565b915050610627565b50505050505050565b6000546001600160a01b031633146107095760405162461bcd60e51b815260040161035990611b25565b610713600061142e565b565b6000546001600160a01b0316331461073f5760405162461bcd60e51b815260040161035990611b25565b6107488161147e565b50565b6000546001600160a01b031633146107755760405162461bcd60e51b815260040161035990611b25565b6104b08282610f63565b6000546001600160a01b031633146107a95760405162461bcd60e51b815260040161035990611b25565b8281146108325760405162461bcd60e51b815260206004820152605a6024820152600080516020611d5083398151915260448201527f72793a206f776e6572426174636852656d6f76655370656369616c4e6674547960648201527f7065416e6452616e6b206c656e67746873206d69736d61746368000000000000608482015260a401610359565b60005b83811015610475576108b785858381811061085257610852611b5a565b9050602002013584848481811061086b5761086b611b5a565b905060200281019061087d9190611b70565b8080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920191909152506114c892505050565b806108c181611bcd565b915050610835565b60008060005b600554811015610a6b576000600582815481106108ee576108ee611b5a565b90600052602060002090600202016040518060400160405290816000820154815260200160018201805461092190611aeb565b80601f016020809104026020016040519081016040528092919081815260200182805461094d90611aeb565b801561099a5780601f1061096f5761010080835404028352916020019161099a565b820191906000526020600020905b81548152906001019060200180831161097d57829003601f168201915b505050919092525050600154825160208401516040516396a9cd7d60e01b81529495506001600160a01b03909216936396a9cd7d93506109df92908a90600401611be6565b602060405180830381865afa1580156109fc573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610a209190611c17565b15610a5857610a5560048381548110610a3b57610a3b611b5a565b9060005260206000200154846116b590919063ffffffff16565b92505b5080610a6381611bcd565b9150506108cf565b5060015460405163f978fff160e01b81526001600160a01b038581166004830152600092169063f978fff190602401602060405180830381865afa158015610ab7573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610adb9190611c39565b9050600254811015610aee575092915050565b60005b600354811015610c7557600060038281548110610b1057610b10611b5a565b906000526020600020906002020160405180604001604052908160008201548152602001600182018054610b4390611aeb565b80601f0160208091040260200160405190810160405280929190818152602001828054610b6f90611aeb565b8015610bbc5780601f10610b9157610100808354040283529160200191610bbc565b820191906000526020600020905b815481529060010190602001808311610b9f57829003601f168201915b505050919092525050600154825160208401516040516396a9cd7d60e01b81529495506001600160a01b03909216936396a9cd7d9350610c0192908b90600401611be6565b602060405180830381865afa158015610c1e573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610c429190611c17565b15610c6257610c5f60025484610c589190611c52565b85906116b5565b93505b5080610c6d81611bcd565b915050610af1565b50909392505050565b60048181548110610c8e57600080fd5b600091825260209091200154905081565b6003818154811061028757600080fd5b6000546001600160a01b03163314610cd95760405162461bcd60e51b815260040161035990611b25565b8060025403610d585760405162461bcd60e51b81526020600482015260516024820152600080516020611d5083398151915260448201527f72793a2074727920746f207570646174652077697468207468652073616d65206064820152701cdd185ada5b99c81d1a1c995cda1bdb19607a1b608482015260a401610359565b600281905560405181907fadfa8ecb21b6962ebcd0adbd9ab985b7b4c5b5eb3b0dead683171565c7bfe17190600090a250565b6000546001600160a01b03163314610db55760405162461bcd60e51b815260040161035990611b25565b6001600160a01b038116610e1a5760405162461bcd60e51b815260206004820152602660248201527f4f776e61626c653a206e6577206f776e657220697320746865207a65726f206160448201526564647265737360d01b6064820152608401610359565b6107488161142e565b6000546001600160a01b03163314610e4d5760405162461bcd60e51b815260040161035990611b25565b828114610ecc5760405162461bcd60e51b81526020600482015260536024820152600080516020611d5083398151915260448201527f72793a206f776e6572426174636852656d6f76654e667454797065416e6452616064820152720dcd640d8cadccee8d0e640dad2e6dac2e8c6d606b1b608482015260a401610359565b60005b8381101561047557610f51858583818110610eec57610eec611b5a565b90506020020135848484818110610f0557610f05611b5a565b9050602002810190610f179190611b70565b8080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920191909152506110c492505050565b80610f5b81611bcd565b915050610ecf565b60005b600354811015610ffd578260038281548110610f8457610f84611b5a565b906000526020600020906002020160000154148015610fe15750818051906020012060038281548110610fb957610fb9611b5a565b9060005260206000209060020201600101604051610fd79190611c74565b6040518091039020145b15610feb57505050565b80610ff581611bcd565b915050610f66565b60408051808201909152838152602080820184815260038054600181018255600091909152835160029091027fc2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85b8101918255915180519193611084937fc2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85c01929101906116ce565b505050827f2360ee3e77485441cfa07e30e8dc5b031fac38455647c89a763434f58733fcc1836040516110b79190611d0f565b60405180910390a2505050565b60005b6003548110156112405782600382815481106110e5576110e5611b5a565b906000526020600020906002020160000154148015611142575081805190602001206003828154811061111a5761111a611b5a565b90600052602060002090600202016001016040516111389190611c74565b6040518091039020145b1561122e576003805461115790600190611d22565b8154811061116757611167611b5a565b90600052602060002090600202016003828154811061118857611188611b5a565b90600052602060002090600202016000820154816000015560018201816001019080546111b490611aeb565b6111bf929190611752565b5090505060038054806111d4576111d4611d39565b600082815260208120600260001990930192830201818155906111fa60018301826117cd565b50509055827fb1323e42d97b2b3d45f9d4641bf4b6b3f9d0d01e90832ae7b7413109b7a5d347836040516110b79190611d0f565b8061123881611bcd565b9150506110c7565b505050565b60005b60055481101561133b57836005828154811061126657611266611b5a565b9060005260206000209060020201600001541480156112c3575082805190602001206005828154811061129b5761129b611b5a565b90600052602060002090600202016001016040516112b99190611c74565b6040518091039020145b156113295781600482815481106112dc576112dc611b5a565b906000526020600020018190555081847fe43bf5f5f8a1211930e5726ba0abceacb1748f97b2966db30a818ba10961cbcc8560405161131b9190611d0f565b60405180910390a350505050565b8061133381611bcd565b915050611248565b60408051808201909152848152602080820185815260058054600181018255600091909152835160029091027f036b6384b5eca791c62761152d0c79bb0604c104a5fb6f4eb0703f3154bb3db081019182559151805191936113c2937f036b6384b5eca791c62761152d0c79bb0604c104a5fb6f4eb0703f3154bb3db101929101906116ce565b5050600480546001810182556000919091527f8a35acfbc15ff81a39ae7d344fd709f28e8600b4aa8c65c6b64bfe7fe36bd19b0183905550604051829085907fe43bf5f5f8a1211930e5726ba0abceacb1748f97b2966db30a818ba10961cbcc9061131b908790611d0f565b600080546001600160a01b038381166001600160a01b0319831681178455604051919092169283917f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09190a35050565b600180546001600160a01b0319166001600160a01b0383169081179091556040517f573bbfa679af6fdcdbd9cf191c5ef3e526599ac2bf75e9177d47adb8530b9c6990600090a250565b60005b6005548110156112405782600582815481106114e9576114e9611b5a565b906000526020600020906002020160000154148015611546575081805190602001206005828154811061151e5761151e611b5a565b906000526020600020906002020160010160405161153c9190611c74565b6040518091039020145b156116a3576005805461155b90600190611d22565b8154811061156b5761156b611b5a565b90600052602060002090600202016005828154811061158c5761158c611b5a565b90600052602060002090600202016000820154816000015560018201816001019080546115b890611aeb565b6115c3929190611752565b5090505060058054806115d8576115d8611d39565b600082815260208120600260001990930192830201818155906115fe60018301826117cd565b505090556004805461161290600190611d22565b8154811061162257611622611b5a565b90600052602060002001546004828154811061164057611640611b5a565b600091825260209091200155600480548061165d5761165d611d39565b60019003818190600052602060002001600090559055827fdb75199103504bd1d3653de758d4295bf00d4587e1d53dfc114464cc47ed97b7836040516110b79190611d0f565b806116ad81611bcd565b9150506114cb565b6000818310156116c557816116c7565b825b9392505050565b8280546116da90611aeb565b90600052602060002090601f0160209004810192826116fc5760008555611742565b82601f1061171557805160ff1916838001178555611742565b82800160010185558215611742579182015b82811115611742578251825591602001919060010190611727565b5061174e929150611803565b5090565b82805461175e90611aeb565b90600052602060002090601f0160209004810192826117805760008555611742565b82601f106117915780548555611742565b8280016001018555821561174257600052602060002091601f016020900482015b828111156117425782548255916001019190600101906117b2565b5080546117d990611aeb565b6000825580601f106117e9575050565b601f01602090049060005260206000209081019061074891905b5b8082111561174e5760008155600101611804565b60006020828403121561182a57600080fd5b5035919050565b6000815180845260005b818110156118575760208185018101518683018201520161183b565b81811115611869576000602083870101525b50601f01601f19169290920160200192915050565b8281526040602082015260006118976040830184611831565b949350505050565b60008083601f8401126118b157600080fd5b50813567ffffffffffffffff8111156118c957600080fd5b6020830191508360208260051b85010111156118e457600080fd5b9250929050565b6000806000806040858703121561190157600080fd5b843567ffffffffffffffff8082111561191957600080fd5b6119258883890161189f565b9096509450602087013591508082111561193e57600080fd5b5061194b8782880161189f565b95989497509550505050565b634e487b7160e01b600052604160045260246000fd5b6000806040838503121561198057600080fd5b82359150602083013567ffffffffffffffff8082111561199f57600080fd5b818501915085601f8301126119b357600080fd5b8135818111156119c5576119c5611957565b604051601f8201601f19908116603f011681019083821181831017156119ed576119ed611957565b81604052828152886020848701011115611a0657600080fd5b8260208601602083013760006020848301015280955050505050509250929050565b60008060008060008060608789031215611a4157600080fd5b863567ffffffffffffffff80821115611a5957600080fd5b611a658a838b0161189f565b90985096506020890135915080821115611a7e57600080fd5b611a8a8a838b0161189f565b90965094506040890135915080821115611aa357600080fd5b50611ab089828a0161189f565b979a9699509497509295939492505050565b600060208284031215611ad457600080fd5b81356001600160a01b03811681146116c757600080fd5b600181811c90821680611aff57607f821691505b602082108103611b1f57634e487b7160e01b600052602260045260246000fd5b50919050565b6020808252818101527f4f776e61626c653a2063616c6c6572206973206e6f7420746865206f776e6572604082015260600190565b634e487b7160e01b600052603260045260246000fd5b6000808335601e19843603018112611b8757600080fd5b83018035915067ffffffffffffffff821115611ba257600080fd5b6020019150368190038213156118e457600080fd5b634e487b7160e01b600052601160045260246000fd5b600060018201611bdf57611bdf611bb7565b5060010190565b838152606060208201526000611bff6060830185611831565b905060018060a01b0383166040830152949350505050565b600060208284031215611c2957600080fd5b815180151581146116c757600080fd5b600060208284031215611c4b57600080fd5b5051919050565b600082611c6f57634e487b7160e01b600052601260045260246000fd5b500490565b600080835481600182811c915080831680611c9057607f831692505b60208084108203611caf57634e487b7160e01b86526022600452602486fd5b818015611cc35760018114611cd457611d01565b60ff19861689528489019650611d01565b60008a81526020902060005b86811015611cf95781548b820152908501908301611ce0565b505084890196505b509498975050505050505050565b6020815260006116c76020830184611831565b600082821015611d3457611d34611bb7565b500390565b634e487b7160e01b600052603160045260246000fdfe486f70725374616b696e6750726f7879466f724e6574776f726b526567697374a26469706673582212202f9662b3370f1f33db53cefe9215163fc70f7186bbb83947b2e716b0b98e732764736f6c634300080d0033" . parse () . expect ("invalid bytecode")
    });
    pub struct HoprStakingProxyForNetworkRegistry<M>(ethers::contract::Contract<M>);
    impl<M> Clone for HoprStakingProxyForNetworkRegistry<M> {
        fn clone(&self) -> Self {
            HoprStakingProxyForNetworkRegistry(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for HoprStakingProxyForNetworkRegistry<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for HoprStakingProxyForNetworkRegistry<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(HoprStakingProxyForNetworkRegistry))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> HoprStakingProxyForNetworkRegistry<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(
                address.into(),
                HOPRSTAKINGPROXYFORNETWORKREGISTRY_ABI.clone(),
                client,
            )
            .into()
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
                HOPRSTAKINGPROXYFORNETWORKREGISTRY_ABI.clone(),
                HOPRSTAKINGPROXYFORNETWORKREGISTRY_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        #[doc = "Calls the contract's `eligibleNftTypeAndRank` (0xde626c0e) function"]
        pub fn eligible_nft_type_and_rank(
            &self,
            p0: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, (ethers::core::types::U256, String)>
        {
            self.0
                .method_hash([222, 98, 108, 14], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `maxAllowedRegistrations` (0xb3544e82) function"]
        pub fn max_allowed_registrations(
            &self,
            account: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([179, 84, 78, 130], account)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `maxRegistrationsPerSpecialNft` (0xba1cef23) function"]
        pub fn max_registrations_per_special_nft(
            &self,
            p0: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([186, 28, 239, 35], p0)
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
        #[doc = "Calls the contract's `ownerAddNftTypeAndRank` (0x9b97076f) function"]
        pub fn owner_add_nft_type_and_rank(
            &self,
            nft_type: ethers::core::types::U256,
            nft_rank: String,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([155, 151, 7, 111], (nft_type, nft_rank))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `ownerBatchAddNftTypeAndRank` (0x506472cc) function"]
        pub fn owner_batch_add_nft_type_and_rank(
            &self,
            nft_types: ::std::vec::Vec<ethers::core::types::U256>,
            nft_ranks: ::std::vec::Vec<String>,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([80, 100, 114, 204], (nft_types, nft_ranks))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `ownerBatchAddSpecialNftTypeAndRank` (0x6a3b64b6) function"]
        pub fn owner_batch_add_special_nft_type_and_rank(
            &self,
            nft_types: ::std::vec::Vec<ethers::core::types::U256>,
            nft_ranks: ::std::vec::Vec<String>,
            max_registrations: ::std::vec::Vec<ethers::core::types::U256>,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [106, 59, 100, 182],
                    (nft_types, nft_ranks, max_registrations),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `ownerBatchRemoveNftTypeAndRank` (0xfb66ac57) function"]
        pub fn owner_batch_remove_nft_type_and_rank(
            &self,
            nft_types: ::std::vec::Vec<ethers::core::types::U256>,
            nft_ranks: ::std::vec::Vec<String>,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([251, 102, 172, 87], (nft_types, nft_ranks))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `ownerBatchRemoveSpecialNftTypeAndRank` (0xb05e8ba9) function"]
        pub fn owner_batch_remove_special_nft_type_and_rank(
            &self,
            nft_types: ::std::vec::Vec<ethers::core::types::U256>,
            nft_ranks: ::std::vec::Vec<String>,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([176, 94, 139, 169], (nft_types, nft_ranks))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `ownerRemoveNftTypeAndRank` (0x654251eb) function"]
        pub fn owner_remove_nft_type_and_rank(
            &self,
            nft_type: ethers::core::types::U256,
            nft_rank: String,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([101, 66, 81, 235], (nft_type, nft_rank))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `ownerUpdateThreshold` (0xee50c7c4) function"]
        pub fn owner_update_threshold(
            &self,
            new_threshold: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([238, 80, 199, 196], new_threshold)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `renounceOwnership` (0x715018a6) function"]
        pub fn renounce_ownership(&self) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([113, 80, 24, 166], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `specialNftTypeAndRank` (0x2c3ec80b) function"]
        pub fn special_nft_type_and_rank(
            &self,
            p0: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, (ethers::core::types::U256, String)>
        {
            self.0
                .method_hash([44, 62, 200, 11], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `stakeContract` (0x1a186227) function"]
        pub fn stake_contract(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([26, 24, 98, 39], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `stakeThreshold` (0xf11f77f9) function"]
        pub fn stake_threshold(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([241, 31, 119, 249], ())
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
        #[doc = "Calls the contract's `updateStakeContract` (0x830c6cc2) function"]
        pub fn update_stake_contract(
            &self,
            stake_contract: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([131, 12, 108, 194], stake_contract)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Gets the contract's `NftTypeAndRankAdded` event"]
        pub fn nft_type_and_rank_added_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, NftTypeAndRankAddedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `NftTypeAndRankRemoved` event"]
        pub fn nft_type_and_rank_removed_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, NftTypeAndRankRemovedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `OwnershipTransferred` event"]
        pub fn ownership_transferred_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, OwnershipTransferredFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `SpecialNftTypeAndRankAdded` event"]
        pub fn special_nft_type_and_rank_added_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, SpecialNftTypeAndRankAddedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `SpecialNftTypeAndRankRemoved` event"]
        pub fn special_nft_type_and_rank_removed_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, SpecialNftTypeAndRankRemovedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `StakeContractUpdated` event"]
        pub fn stake_contract_updated_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, StakeContractUpdatedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `ThresholdUpdated` event"]
        pub fn threshold_updated_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, ThresholdUpdatedFilter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(
            &self,
        ) -> ethers::contract::builders::Event<M, HoprStakingProxyForNetworkRegistryEvents>
        {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>>
        for HoprStakingProxyForNetworkRegistry<M>
    {
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
    #[ethevent(
        name = "NftTypeAndRankAdded",
        abi = "NftTypeAndRankAdded(uint256,string)"
    )]
    pub struct NftTypeAndRankAddedFilter {
        #[ethevent(indexed)]
        pub nft_type: ethers::core::types::U256,
        pub nft_rank: String,
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
        name = "NftTypeAndRankRemoved",
        abi = "NftTypeAndRankRemoved(uint256,string)"
    )]
    pub struct NftTypeAndRankRemovedFilter {
        #[ethevent(indexed)]
        pub nft_type: ethers::core::types::U256,
        pub nft_rank: String,
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
    #[ethevent(
        name = "SpecialNftTypeAndRankAdded",
        abi = "SpecialNftTypeAndRankAdded(uint256,string,uint256)"
    )]
    pub struct SpecialNftTypeAndRankAddedFilter {
        #[ethevent(indexed)]
        pub nft_type: ethers::core::types::U256,
        pub nft_rank: String,
        #[ethevent(indexed)]
        pub max_registration: ethers::core::types::U256,
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
        name = "SpecialNftTypeAndRankRemoved",
        abi = "SpecialNftTypeAndRankRemoved(uint256,string)"
    )]
    pub struct SpecialNftTypeAndRankRemovedFilter {
        #[ethevent(indexed)]
        pub nft_type: ethers::core::types::U256,
        pub nft_rank: String,
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
    #[ethevent(name = "StakeContractUpdated", abi = "StakeContractUpdated(address)")]
    pub struct StakeContractUpdatedFilter {
        #[ethevent(indexed)]
        pub stake_contract: ethers::core::types::Address,
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
    #[ethevent(name = "ThresholdUpdated", abi = "ThresholdUpdated(uint256)")]
    pub struct ThresholdUpdatedFilter {
        #[ethevent(indexed)]
        pub threshold: ethers::core::types::U256,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum HoprStakingProxyForNetworkRegistryEvents {
        NftTypeAndRankAddedFilter(NftTypeAndRankAddedFilter),
        NftTypeAndRankRemovedFilter(NftTypeAndRankRemovedFilter),
        OwnershipTransferredFilter(OwnershipTransferredFilter),
        SpecialNftTypeAndRankAddedFilter(SpecialNftTypeAndRankAddedFilter),
        SpecialNftTypeAndRankRemovedFilter(SpecialNftTypeAndRankRemovedFilter),
        StakeContractUpdatedFilter(StakeContractUpdatedFilter),
        ThresholdUpdatedFilter(ThresholdUpdatedFilter),
    }
    impl ethers::contract::EthLogDecode for HoprStakingProxyForNetworkRegistryEvents {
        fn decode_log(
            log: &ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = NftTypeAndRankAddedFilter::decode_log(log) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryEvents::NftTypeAndRankAddedFilter(decoded),
                );
            }
            if let Ok(decoded) = NftTypeAndRankRemovedFilter::decode_log(log) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryEvents::NftTypeAndRankRemovedFilter(decoded),
                );
            }
            if let Ok(decoded) = OwnershipTransferredFilter::decode_log(log) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryEvents::OwnershipTransferredFilter(decoded),
                );
            }
            if let Ok(decoded) = SpecialNftTypeAndRankAddedFilter::decode_log(log) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryEvents::SpecialNftTypeAndRankAddedFilter(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded) = SpecialNftTypeAndRankRemovedFilter::decode_log(log) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryEvents::SpecialNftTypeAndRankRemovedFilter(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded) = StakeContractUpdatedFilter::decode_log(log) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryEvents::StakeContractUpdatedFilter(decoded),
                );
            }
            if let Ok(decoded) = ThresholdUpdatedFilter::decode_log(log) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryEvents::ThresholdUpdatedFilter(decoded),
                );
            }
            Err(ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for HoprStakingProxyForNetworkRegistryEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                HoprStakingProxyForNetworkRegistryEvents::NftTypeAndRankAddedFilter(element) => {
                    element.fmt(f)
                }
                HoprStakingProxyForNetworkRegistryEvents::NftTypeAndRankRemovedFilter(element) => {
                    element.fmt(f)
                }
                HoprStakingProxyForNetworkRegistryEvents::OwnershipTransferredFilter(element) => {
                    element.fmt(f)
                }
                HoprStakingProxyForNetworkRegistryEvents::SpecialNftTypeAndRankAddedFilter(
                    element,
                ) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryEvents::SpecialNftTypeAndRankRemovedFilter(
                    element,
                ) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryEvents::StakeContractUpdatedFilter(element) => {
                    element.fmt(f)
                }
                HoprStakingProxyForNetworkRegistryEvents::ThresholdUpdatedFilter(element) => {
                    element.fmt(f)
                }
            }
        }
    }
    #[doc = "Container type for all input parameters for the `eligibleNftTypeAndRank` function with signature `eligibleNftTypeAndRank(uint256)` and selector `[222, 98, 108, 14]`"]
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
        name = "eligibleNftTypeAndRank",
        abi = "eligibleNftTypeAndRank(uint256)"
    )]
    pub struct EligibleNftTypeAndRankCall(pub ethers::core::types::U256);
    #[doc = "Container type for all input parameters for the `maxAllowedRegistrations` function with signature `maxAllowedRegistrations(address)` and selector `[179, 84, 78, 130]`"]
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
        name = "maxAllowedRegistrations",
        abi = "maxAllowedRegistrations(address)"
    )]
    pub struct MaxAllowedRegistrationsCall {
        pub account: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `maxRegistrationsPerSpecialNft` function with signature `maxRegistrationsPerSpecialNft(uint256)` and selector `[186, 28, 239, 35]`"]
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
        name = "maxRegistrationsPerSpecialNft",
        abi = "maxRegistrationsPerSpecialNft(uint256)"
    )]
    pub struct MaxRegistrationsPerSpecialNftCall(pub ethers::core::types::U256);
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
    #[doc = "Container type for all input parameters for the `ownerAddNftTypeAndRank` function with signature `ownerAddNftTypeAndRank(uint256,string)` and selector `[155, 151, 7, 111]`"]
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
        name = "ownerAddNftTypeAndRank",
        abi = "ownerAddNftTypeAndRank(uint256,string)"
    )]
    pub struct OwnerAddNftTypeAndRankCall {
        pub nft_type: ethers::core::types::U256,
        pub nft_rank: String,
    }
    #[doc = "Container type for all input parameters for the `ownerBatchAddNftTypeAndRank` function with signature `ownerBatchAddNftTypeAndRank(uint256[],string[])` and selector `[80, 100, 114, 204]`"]
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
        name = "ownerBatchAddNftTypeAndRank",
        abi = "ownerBatchAddNftTypeAndRank(uint256[],string[])"
    )]
    pub struct OwnerBatchAddNftTypeAndRankCall {
        pub nft_types: ::std::vec::Vec<ethers::core::types::U256>,
        pub nft_ranks: ::std::vec::Vec<String>,
    }
    #[doc = "Container type for all input parameters for the `ownerBatchAddSpecialNftTypeAndRank` function with signature `ownerBatchAddSpecialNftTypeAndRank(uint256[],string[],uint256[])` and selector `[106, 59, 100, 182]`"]
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
        name = "ownerBatchAddSpecialNftTypeAndRank",
        abi = "ownerBatchAddSpecialNftTypeAndRank(uint256[],string[],uint256[])"
    )]
    pub struct OwnerBatchAddSpecialNftTypeAndRankCall {
        pub nft_types: ::std::vec::Vec<ethers::core::types::U256>,
        pub nft_ranks: ::std::vec::Vec<String>,
        pub max_registrations: ::std::vec::Vec<ethers::core::types::U256>,
    }
    #[doc = "Container type for all input parameters for the `ownerBatchRemoveNftTypeAndRank` function with signature `ownerBatchRemoveNftTypeAndRank(uint256[],string[])` and selector `[251, 102, 172, 87]`"]
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
        name = "ownerBatchRemoveNftTypeAndRank",
        abi = "ownerBatchRemoveNftTypeAndRank(uint256[],string[])"
    )]
    pub struct OwnerBatchRemoveNftTypeAndRankCall {
        pub nft_types: ::std::vec::Vec<ethers::core::types::U256>,
        pub nft_ranks: ::std::vec::Vec<String>,
    }
    #[doc = "Container type for all input parameters for the `ownerBatchRemoveSpecialNftTypeAndRank` function with signature `ownerBatchRemoveSpecialNftTypeAndRank(uint256[],string[])` and selector `[176, 94, 139, 169]`"]
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
        name = "ownerBatchRemoveSpecialNftTypeAndRank",
        abi = "ownerBatchRemoveSpecialNftTypeAndRank(uint256[],string[])"
    )]
    pub struct OwnerBatchRemoveSpecialNftTypeAndRankCall {
        pub nft_types: ::std::vec::Vec<ethers::core::types::U256>,
        pub nft_ranks: ::std::vec::Vec<String>,
    }
    #[doc = "Container type for all input parameters for the `ownerRemoveNftTypeAndRank` function with signature `ownerRemoveNftTypeAndRank(uint256,string)` and selector `[101, 66, 81, 235]`"]
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
        name = "ownerRemoveNftTypeAndRank",
        abi = "ownerRemoveNftTypeAndRank(uint256,string)"
    )]
    pub struct OwnerRemoveNftTypeAndRankCall {
        pub nft_type: ethers::core::types::U256,
        pub nft_rank: String,
    }
    #[doc = "Container type for all input parameters for the `ownerUpdateThreshold` function with signature `ownerUpdateThreshold(uint256)` and selector `[238, 80, 199, 196]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "ownerUpdateThreshold", abi = "ownerUpdateThreshold(uint256)")]
    pub struct OwnerUpdateThresholdCall {
        pub new_threshold: ethers::core::types::U256,
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
    #[doc = "Container type for all input parameters for the `specialNftTypeAndRank` function with signature `specialNftTypeAndRank(uint256)` and selector `[44, 62, 200, 11]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "specialNftTypeAndRank", abi = "specialNftTypeAndRank(uint256)")]
    pub struct SpecialNftTypeAndRankCall(pub ethers::core::types::U256);
    #[doc = "Container type for all input parameters for the `stakeContract` function with signature `stakeContract()` and selector `[26, 24, 98, 39]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "stakeContract", abi = "stakeContract()")]
    pub struct StakeContractCall;
    #[doc = "Container type for all input parameters for the `stakeThreshold` function with signature `stakeThreshold()` and selector `[241, 31, 119, 249]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "stakeThreshold", abi = "stakeThreshold()")]
    pub struct StakeThresholdCall;
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
    #[doc = "Container type for all input parameters for the `updateStakeContract` function with signature `updateStakeContract(address)` and selector `[131, 12, 108, 194]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "updateStakeContract", abi = "updateStakeContract(address)")]
    pub struct UpdateStakeContractCall {
        pub stake_contract: ethers::core::types::Address,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum HoprStakingProxyForNetworkRegistryCalls {
        EligibleNftTypeAndRank(EligibleNftTypeAndRankCall),
        MaxAllowedRegistrations(MaxAllowedRegistrationsCall),
        MaxRegistrationsPerSpecialNft(MaxRegistrationsPerSpecialNftCall),
        Owner(OwnerCall),
        OwnerAddNftTypeAndRank(OwnerAddNftTypeAndRankCall),
        OwnerBatchAddNftTypeAndRank(OwnerBatchAddNftTypeAndRankCall),
        OwnerBatchAddSpecialNftTypeAndRank(OwnerBatchAddSpecialNftTypeAndRankCall),
        OwnerBatchRemoveNftTypeAndRank(OwnerBatchRemoveNftTypeAndRankCall),
        OwnerBatchRemoveSpecialNftTypeAndRank(OwnerBatchRemoveSpecialNftTypeAndRankCall),
        OwnerRemoveNftTypeAndRank(OwnerRemoveNftTypeAndRankCall),
        OwnerUpdateThreshold(OwnerUpdateThresholdCall),
        RenounceOwnership(RenounceOwnershipCall),
        SpecialNftTypeAndRank(SpecialNftTypeAndRankCall),
        StakeContract(StakeContractCall),
        StakeThreshold(StakeThresholdCall),
        TransferOwnership(TransferOwnershipCall),
        UpdateStakeContract(UpdateStakeContractCall),
    }
    impl ethers::core::abi::AbiDecode for HoprStakingProxyForNetworkRegistryCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <EligibleNftTypeAndRankCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakingProxyForNetworkRegistryCalls::EligibleNftTypeAndRank(decoded));
            }
            if let Ok(decoded) =
                <MaxAllowedRegistrationsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(
                    HoprStakingProxyForNetworkRegistryCalls::MaxAllowedRegistrations(decoded),
                );
            }
            if let Ok(decoded) =
                <MaxRegistrationsPerSpecialNftCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(
                    HoprStakingProxyForNetworkRegistryCalls::MaxRegistrationsPerSpecialNft(decoded),
                );
            }
            if let Ok(decoded) = <OwnerCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakingProxyForNetworkRegistryCalls::Owner(decoded));
            }
            if let Ok(decoded) =
                <OwnerAddNftTypeAndRankCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakingProxyForNetworkRegistryCalls::OwnerAddNftTypeAndRank(decoded));
            }
            if let Ok(decoded) =
                <OwnerBatchAddNftTypeAndRankCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(
                    HoprStakingProxyForNetworkRegistryCalls::OwnerBatchAddNftTypeAndRank(decoded),
                );
            }
            if let Ok(decoded) =
                <OwnerBatchAddSpecialNftTypeAndRankCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(
                    HoprStakingProxyForNetworkRegistryCalls::OwnerBatchAddSpecialNftTypeAndRank(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded) =
                <OwnerBatchRemoveNftTypeAndRankCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(
                    HoprStakingProxyForNetworkRegistryCalls::OwnerBatchRemoveNftTypeAndRank(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded) =
                <OwnerBatchRemoveSpecialNftTypeAndRankCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(
                    HoprStakingProxyForNetworkRegistryCalls::OwnerBatchRemoveSpecialNftTypeAndRank(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded) =
                <OwnerRemoveNftTypeAndRankCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(
                    HoprStakingProxyForNetworkRegistryCalls::OwnerRemoveNftTypeAndRank(decoded),
                );
            }
            if let Ok(decoded) =
                <OwnerUpdateThresholdCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakingProxyForNetworkRegistryCalls::OwnerUpdateThreshold(decoded));
            }
            if let Ok(decoded) =
                <RenounceOwnershipCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakingProxyForNetworkRegistryCalls::RenounceOwnership(
                    decoded,
                ));
            }
            if let Ok(decoded) =
                <SpecialNftTypeAndRankCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakingProxyForNetworkRegistryCalls::SpecialNftTypeAndRank(decoded));
            }
            if let Ok(decoded) =
                <StakeContractCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakingProxyForNetworkRegistryCalls::StakeContract(
                    decoded,
                ));
            }
            if let Ok(decoded) =
                <StakeThresholdCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakingProxyForNetworkRegistryCalls::StakeThreshold(
                    decoded,
                ));
            }
            if let Ok(decoded) =
                <TransferOwnershipCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakingProxyForNetworkRegistryCalls::TransferOwnership(
                    decoded,
                ));
            }
            if let Ok(decoded) =
                <UpdateStakeContractCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprStakingProxyForNetworkRegistryCalls::UpdateStakeContract(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for HoprStakingProxyForNetworkRegistryCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                HoprStakingProxyForNetworkRegistryCalls::EligibleNftTypeAndRank(element) => {
                    element.encode()
                }
                HoprStakingProxyForNetworkRegistryCalls::MaxAllowedRegistrations(element) => {
                    element.encode()
                }
                HoprStakingProxyForNetworkRegistryCalls::MaxRegistrationsPerSpecialNft(element) => {
                    element.encode()
                }
                HoprStakingProxyForNetworkRegistryCalls::Owner(element) => element.encode(),
                HoprStakingProxyForNetworkRegistryCalls::OwnerAddNftTypeAndRank(element) => {
                    element.encode()
                }
                HoprStakingProxyForNetworkRegistryCalls::OwnerBatchAddNftTypeAndRank(element) => {
                    element.encode()
                }
                HoprStakingProxyForNetworkRegistryCalls::OwnerBatchAddSpecialNftTypeAndRank(
                    element,
                ) => element.encode(),
                HoprStakingProxyForNetworkRegistryCalls::OwnerBatchRemoveNftTypeAndRank(
                    element,
                ) => element.encode(),
                HoprStakingProxyForNetworkRegistryCalls::OwnerBatchRemoveSpecialNftTypeAndRank(
                    element,
                ) => element.encode(),
                HoprStakingProxyForNetworkRegistryCalls::OwnerRemoveNftTypeAndRank(element) => {
                    element.encode()
                }
                HoprStakingProxyForNetworkRegistryCalls::OwnerUpdateThreshold(element) => {
                    element.encode()
                }
                HoprStakingProxyForNetworkRegistryCalls::RenounceOwnership(element) => {
                    element.encode()
                }
                HoprStakingProxyForNetworkRegistryCalls::SpecialNftTypeAndRank(element) => {
                    element.encode()
                }
                HoprStakingProxyForNetworkRegistryCalls::StakeContract(element) => element.encode(),
                HoprStakingProxyForNetworkRegistryCalls::StakeThreshold(element) => {
                    element.encode()
                }
                HoprStakingProxyForNetworkRegistryCalls::TransferOwnership(element) => {
                    element.encode()
                }
                HoprStakingProxyForNetworkRegistryCalls::UpdateStakeContract(element) => {
                    element.encode()
                }
            }
        }
    }
    impl ::std::fmt::Display for HoprStakingProxyForNetworkRegistryCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                HoprStakingProxyForNetworkRegistryCalls::EligibleNftTypeAndRank(element) => {
                    element.fmt(f)
                }
                HoprStakingProxyForNetworkRegistryCalls::MaxAllowedRegistrations(element) => {
                    element.fmt(f)
                }
                HoprStakingProxyForNetworkRegistryCalls::MaxRegistrationsPerSpecialNft(element) => {
                    element.fmt(f)
                }
                HoprStakingProxyForNetworkRegistryCalls::Owner(element) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryCalls::OwnerAddNftTypeAndRank(element) => {
                    element.fmt(f)
                }
                HoprStakingProxyForNetworkRegistryCalls::OwnerBatchAddNftTypeAndRank(element) => {
                    element.fmt(f)
                }
                HoprStakingProxyForNetworkRegistryCalls::OwnerBatchAddSpecialNftTypeAndRank(
                    element,
                ) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryCalls::OwnerBatchRemoveNftTypeAndRank(
                    element,
                ) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryCalls::OwnerBatchRemoveSpecialNftTypeAndRank(
                    element,
                ) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryCalls::OwnerRemoveNftTypeAndRank(element) => {
                    element.fmt(f)
                }
                HoprStakingProxyForNetworkRegistryCalls::OwnerUpdateThreshold(element) => {
                    element.fmt(f)
                }
                HoprStakingProxyForNetworkRegistryCalls::RenounceOwnership(element) => {
                    element.fmt(f)
                }
                HoprStakingProxyForNetworkRegistryCalls::SpecialNftTypeAndRank(element) => {
                    element.fmt(f)
                }
                HoprStakingProxyForNetworkRegistryCalls::StakeContract(element) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryCalls::StakeThreshold(element) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryCalls::TransferOwnership(element) => {
                    element.fmt(f)
                }
                HoprStakingProxyForNetworkRegistryCalls::UpdateStakeContract(element) => {
                    element.fmt(f)
                }
            }
        }
    }
    impl ::std::convert::From<EligibleNftTypeAndRankCall> for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: EligibleNftTypeAndRankCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::EligibleNftTypeAndRank(var)
        }
    }
    impl ::std::convert::From<MaxAllowedRegistrationsCall> for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: MaxAllowedRegistrationsCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::MaxAllowedRegistrations(var)
        }
    }
    impl ::std::convert::From<MaxRegistrationsPerSpecialNftCall>
        for HoprStakingProxyForNetworkRegistryCalls
    {
        fn from(var: MaxRegistrationsPerSpecialNftCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::MaxRegistrationsPerSpecialNft(var)
        }
    }
    impl ::std::convert::From<OwnerCall> for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: OwnerCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::Owner(var)
        }
    }
    impl ::std::convert::From<OwnerAddNftTypeAndRankCall> for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: OwnerAddNftTypeAndRankCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::OwnerAddNftTypeAndRank(var)
        }
    }
    impl ::std::convert::From<OwnerBatchAddNftTypeAndRankCall>
        for HoprStakingProxyForNetworkRegistryCalls
    {
        fn from(var: OwnerBatchAddNftTypeAndRankCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::OwnerBatchAddNftTypeAndRank(var)
        }
    }
    impl ::std::convert::From<OwnerBatchAddSpecialNftTypeAndRankCall>
        for HoprStakingProxyForNetworkRegistryCalls
    {
        fn from(var: OwnerBatchAddSpecialNftTypeAndRankCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::OwnerBatchAddSpecialNftTypeAndRank(var)
        }
    }
    impl ::std::convert::From<OwnerBatchRemoveNftTypeAndRankCall>
        for HoprStakingProxyForNetworkRegistryCalls
    {
        fn from(var: OwnerBatchRemoveNftTypeAndRankCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::OwnerBatchRemoveNftTypeAndRank(var)
        }
    }
    impl ::std::convert::From<OwnerBatchRemoveSpecialNftTypeAndRankCall>
        for HoprStakingProxyForNetworkRegistryCalls
    {
        fn from(var: OwnerBatchRemoveSpecialNftTypeAndRankCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::OwnerBatchRemoveSpecialNftTypeAndRank(var)
        }
    }
    impl ::std::convert::From<OwnerRemoveNftTypeAndRankCall>
        for HoprStakingProxyForNetworkRegistryCalls
    {
        fn from(var: OwnerRemoveNftTypeAndRankCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::OwnerRemoveNftTypeAndRank(var)
        }
    }
    impl ::std::convert::From<OwnerUpdateThresholdCall> for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: OwnerUpdateThresholdCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::OwnerUpdateThreshold(var)
        }
    }
    impl ::std::convert::From<RenounceOwnershipCall> for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: RenounceOwnershipCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::RenounceOwnership(var)
        }
    }
    impl ::std::convert::From<SpecialNftTypeAndRankCall> for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: SpecialNftTypeAndRankCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::SpecialNftTypeAndRank(var)
        }
    }
    impl ::std::convert::From<StakeContractCall> for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: StakeContractCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::StakeContract(var)
        }
    }
    impl ::std::convert::From<StakeThresholdCall> for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: StakeThresholdCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::StakeThreshold(var)
        }
    }
    impl ::std::convert::From<TransferOwnershipCall> for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: TransferOwnershipCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::TransferOwnership(var)
        }
    }
    impl ::std::convert::From<UpdateStakeContractCall> for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: UpdateStakeContractCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::UpdateStakeContract(var)
        }
    }
    #[doc = "Container type for all return fields from the `eligibleNftTypeAndRank` function with signature `eligibleNftTypeAndRank(uint256)` and selector `[222, 98, 108, 14]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct EligibleNftTypeAndRankReturn {
        pub nft_type: ethers::core::types::U256,
        pub nft_rank: String,
    }
    #[doc = "Container type for all return fields from the `maxAllowedRegistrations` function with signature `maxAllowedRegistrations(address)` and selector `[179, 84, 78, 130]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct MaxAllowedRegistrationsReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `maxRegistrationsPerSpecialNft` function with signature `maxRegistrationsPerSpecialNft(uint256)` and selector `[186, 28, 239, 35]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct MaxRegistrationsPerSpecialNftReturn(pub ethers::core::types::U256);
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
    #[doc = "Container type for all return fields from the `specialNftTypeAndRank` function with signature `specialNftTypeAndRank(uint256)` and selector `[44, 62, 200, 11]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct SpecialNftTypeAndRankReturn {
        pub nft_type: ethers::core::types::U256,
        pub nft_rank: String,
    }
    #[doc = "Container type for all return fields from the `stakeContract` function with signature `stakeContract()` and selector `[26, 24, 98, 39]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct StakeContractReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `stakeThreshold` function with signature `stakeThreshold()` and selector `[241, 31, 119, 249]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct StakeThresholdReturn(pub ethers::core::types::U256);
}
