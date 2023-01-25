pub use hopr_staking_proxy_for_network_registry::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod hopr_staking_proxy_for_network_registry {
    #![allow(clippy::enum_variant_names)]
    #![allow(dead_code)]
    #![allow(clippy::type_complexity)]
    #![allow(unused_imports)]
    ///HoprStakingProxyForNetworkRegistry was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs
    use std::sync::Arc;
    use ::ethers::core::{
        abi::{Abi, Token, Detokenize, InvalidOutputType, Tokenizable},
        types::*,
    };
    use ::ethers::contract::{
        Contract, builders::{ContractCall, Event},
        Lazy,
    };
    use ::ethers::providers::Middleware;
    #[rustfmt::skip]
    const __ABI: &str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_stakeContract\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"_newOwner\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"_minStake\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"nftType\",\"type\":\"uint256\",\"components\":[],\"indexed\":true},{\"internalType\":\"string\",\"name\":\"nftRank\",\"type\":\"string\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"NftTypeAndRankAdded\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"nftType\",\"type\":\"uint256\",\"components\":[],\"indexed\":true},{\"internalType\":\"string\",\"name\":\"nftRank\",\"type\":\"string\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"NftTypeAndRankRemoved\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"previousOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"OwnershipTransferred\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"nftType\",\"type\":\"uint256\",\"components\":[],\"indexed\":true},{\"internalType\":\"string\",\"name\":\"nftRank\",\"type\":\"string\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"maxRegistration\",\"type\":\"uint256\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"SpecialNftTypeAndRankAdded\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"nftType\",\"type\":\"uint256\",\"components\":[],\"indexed\":true},{\"internalType\":\"string\",\"name\":\"nftRank\",\"type\":\"string\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"SpecialNftTypeAndRankRemoved\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"stakeContract\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"StakeContractUpdated\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"threshold\",\"type\":\"uint256\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"ThresholdUpdated\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"eligibleNftTypeAndRank\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"nftType\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"nftRank\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"maxAllowedRegistrations\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"maxRegistrationsPerSpecialNft\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"owner\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"nftType\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"nftRank\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"ownerAddNftTypeAndRank\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256[]\",\"name\":\"nftTypes\",\"type\":\"uint256[]\",\"components\":[]},{\"internalType\":\"string[]\",\"name\":\"nftRanks\",\"type\":\"string[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"ownerBatchAddNftTypeAndRank\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256[]\",\"name\":\"nftTypes\",\"type\":\"uint256[]\",\"components\":[]},{\"internalType\":\"string[]\",\"name\":\"nftRanks\",\"type\":\"string[]\",\"components\":[]},{\"internalType\":\"uint256[]\",\"name\":\"maxRegistrations\",\"type\":\"uint256[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"ownerBatchAddSpecialNftTypeAndRank\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256[]\",\"name\":\"nftTypes\",\"type\":\"uint256[]\",\"components\":[]},{\"internalType\":\"string[]\",\"name\":\"nftRanks\",\"type\":\"string[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"ownerBatchRemoveNftTypeAndRank\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256[]\",\"name\":\"nftTypes\",\"type\":\"uint256[]\",\"components\":[]},{\"internalType\":\"string[]\",\"name\":\"nftRanks\",\"type\":\"string[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"ownerBatchRemoveSpecialNftTypeAndRank\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"nftType\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"nftRank\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"ownerRemoveNftTypeAndRank\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"newThreshold\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"ownerUpdateThreshold\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"renounceOwnership\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"specialNftTypeAndRank\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"nftType\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"nftRank\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"stakeContract\",\"outputs\":[{\"internalType\":\"contract IHoprStake\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"stakeThreshold\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferOwnership\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_stakeContract\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"updateStakeContract\",\"outputs\":[]}]";
    /// The parsed JSON-ABI of the contract.
    pub static HOPRSTAKINGPROXYFORNETWORKREGISTRY_ABI: ::ethers::contract::Lazy<
        ::ethers::core::abi::Abi,
    > = ::ethers::contract::Lazy::new(|| {
        ::ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
    });
    /// Bytecode of the #name contract
    pub static HOPRSTAKINGPROXYFORNETWORKREGISTRY_BYTECODE: ::ethers::contract::Lazy<
        ::ethers::core::types::Bytes,
    > = ::ethers::contract::Lazy::new(|| {
        "0x60806040523480156200001157600080fd5b5060405162001fdc38038062001fdc833981016040819052620000349162000145565b6200003f336200008e565b6200004a83620000de565b62000055826200008e565b600281905560405181907fadfa8ecb21b6962ebcd0adbd9ab985b7b4c5b5eb3b0dead683171565c7bfe17190600090a250505062000186565b600080546001600160a01b038381166001600160a01b0319831681178455604051919092169283917f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09190a35050565b600180546001600160a01b0319166001600160a01b0383169081179091556040517f573bbfa679af6fdcdbd9cf191c5ef3e526599ac2bf75e9177d47adb8530b9c6990600090a250565b80516001600160a01b03811681146200014057600080fd5b919050565b6000806000606084860312156200015b57600080fd5b620001668462000128565b9250620001766020850162000128565b9150604084015190509250925092565b611e4680620001966000396000f3fe608060405234801561001057600080fd5b506004361061010b5760003560e01c80639b97076f116100a2578063de626c0e11610071578063de626c0e14610222578063ee50c7c414610235578063f11f77f914610248578063f2fde38b14610251578063fb66ac571461026457600080fd5b80639b97076f146101c8578063b05e8ba9146101db578063b3544e82146101ee578063ba1cef231461020f57600080fd5b80636a3b64b6116100de5780636a3b64b614610189578063715018a61461019c578063830c6cc2146101a45780638da5cb5b146101b757600080fd5b80631a186227146101105780632c3ec80b14610140578063506472cc14610161578063654251eb14610176575b600080fd5b600154610123906001600160a01b031681565b6040516001600160a01b0390911681526020015b60405180910390f35b61015361014e366004611701565b610277565b604051610137929190611760565b61017461016f3660046117cd565b61032f565b005b61017461018436600461184f565b61047c565b61017461019736600461190a565b6104b4565b6101746106df565b6101746101b23660046119a4565b610715565b6000546001600160a01b0316610123565b6101746101d636600461184f565b61074b565b6101746101e93660046117cd565b61077f565b6102016101fc3660046119a4565b6108c9565b604051908152602001610137565b61020161021d366004611701565b610c7e565b610153610230366004611701565b610c9f565b610174610243366004611701565b610caf565b61020160025481565b61017461025f3660046119a4565b610d8b565b6101746102723660046117cd565b610e23565b6005818154811061028757600080fd5b600091825260209091206002909102018054600182018054919350906102ac906119d4565b80601f01602080910402602001604051908101604052809291908181526020018280546102d8906119d4565b80156103255780601f106102fa57610100808354040283529160200191610325565b820191906000526020600020905b81548152906001019060200180831161030857829003601f168201915b5050505050905082565b6000546001600160a01b031633146103625760405162461bcd60e51b815260040161035990611a0e565b60405180910390fd5b8281146103de5760405162461bcd60e51b81526020600482015260506024820152600080516020611df183398151915260448201527f72793a206f776e657242617463684164644e667454797065416e6452616e6b2060648201526f0d8cadccee8d0e640dad2e6dac2e8c6d60831b608482015260a401610359565b60005b83811015610475576104638585838181106103fe576103fe611a43565b9050602002013584848481811061041757610417611a43565b90506020028101906104299190611a59565b8080601f016020809104026020016040519081016040528093929190818152602001838380828437600092019190915250610f6392505050565b8061046d81611ab6565b9150506103e1565b5050505050565b6000546001600160a01b031633146104a65760405162461bcd60e51b815260040161035990611a0e565b6104b082826110bf565b5050565b6000546001600160a01b031633146104de5760405162461bcd60e51b815260040161035990611a0e565b84831461057d5760405162461bcd60e51b815260206004820152606d6024820152600080516020611df183398151915260448201527f72793a206f776e657242617463684164645370656369616c4e6674547970654160648201527f6e6452616e6b206e6674547970657320616e64206e667452616e6b73206c656e60848201526c0cee8d0e640dad2e6dac2e8c6d609b1b60a482015260c401610359565b8481146106245760405162461bcd60e51b81526020600482015260756024820152600080516020611df183398151915260448201527f72793a206f776e657242617463684164645370656369616c4e6674547970654160648201527f6e6452616e6b206e6674547970657320616e64206d61785265676973747261746084820152740d2dedce640d8cadccee8d0e640dad2e6dac2e8c6d605b1b60a482015260c401610359565b60005b858110156106d6576106c487878381811061064457610644611a43565b9050602002013586868481811061065d5761065d611a43565b905060200281019061066f9190611a59565b8080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920191909152508892508791508690508181106106b8576106b8611a43565b9050602002013561123d565b806106ce81611ab6565b915050610627565b50505050505050565b6000546001600160a01b031633146107095760405162461bcd60e51b815260040161035990611a0e565b6107136000611421565b565b6000546001600160a01b0316331461073f5760405162461bcd60e51b815260040161035990611a0e565b61074881611471565b50565b6000546001600160a01b031633146107755760405162461bcd60e51b815260040161035990611a0e565b6104b08282610f63565b6000546001600160a01b031633146107a95760405162461bcd60e51b815260040161035990611a0e565b8281146108325760405162461bcd60e51b815260206004820152605a6024820152600080516020611df183398151915260448201527f72793a206f776e6572426174636852656d6f76655370656369616c4e6674547960648201527f7065416e6452616e6b206c656e67746873206d69736d61746368000000000000608482015260a401610359565b60005b83811015610475576108b785858381811061085257610852611a43565b9050602002013584848481811061086b5761086b611a43565b905060200281019061087d9190611a59565b8080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920191909152506114bb92505050565b806108c181611ab6565b915050610835565b60008060005b600554811015610a6b576000600582815481106108ee576108ee611a43565b906000526020600020906002020160405180604001604052908160008201548152602001600182018054610921906119d4565b80601f016020809104026020016040519081016040528092919081815260200182805461094d906119d4565b801561099a5780601f1061096f5761010080835404028352916020019161099a565b820191906000526020600020905b81548152906001019060200180831161097d57829003601f168201915b505050919092525050600154825160208401516040516396a9cd7d60e01b81529495506001600160a01b03909216936396a9cd7d93506109df92908a90600401611acf565b602060405180830381865afa1580156109fc573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610a209190611b00565b15610a5857610a5560048381548110610a3b57610a3b611a43565b90600052602060002001548461169890919063ffffffff16565b92505b5080610a6381611ab6565b9150506108cf565b5060015460405163f978fff160e01b81526001600160a01b038581166004830152600092169063f978fff190602401602060405180830381865afa158015610ab7573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610adb9190611b22565b9050600254811015610aee575092915050565b60005b600354811015610c7557600060038281548110610b1057610b10611a43565b906000526020600020906002020160405180604001604052908160008201548152602001600182018054610b43906119d4565b80601f0160208091040260200160405190810160405280929190818152602001828054610b6f906119d4565b8015610bbc5780601f10610b9157610100808354040283529160200191610bbc565b820191906000526020600020905b815481529060010190602001808311610b9f57829003601f168201915b505050919092525050600154825160208401516040516396a9cd7d60e01b81529495506001600160a01b03909216936396a9cd7d9350610c0192908b90600401611acf565b602060405180830381865afa158015610c1e573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610c429190611b00565b15610c6257610c5f60025484610c589190611b3b565b8590611698565b93505b5080610c6d81611ab6565b915050610af1565b50909392505050565b60048181548110610c8e57600080fd5b600091825260209091200154905081565b6003818154811061028757600080fd5b6000546001600160a01b03163314610cd95760405162461bcd60e51b815260040161035990611a0e565b8060025403610d585760405162461bcd60e51b81526020600482015260516024820152600080516020611df183398151915260448201527f72793a2074727920746f207570646174652077697468207468652073616d65206064820152701cdd185ada5b99c81d1a1c995cda1bdb19607a1b608482015260a401610359565b600281905560405181907fadfa8ecb21b6962ebcd0adbd9ab985b7b4c5b5eb3b0dead683171565c7bfe17190600090a250565b6000546001600160a01b03163314610db55760405162461bcd60e51b815260040161035990611a0e565b6001600160a01b038116610e1a5760405162461bcd60e51b815260206004820152602660248201527f4f776e61626c653a206e6577206f776e657220697320746865207a65726f206160448201526564647265737360d01b6064820152608401610359565b61074881611421565b6000546001600160a01b03163314610e4d5760405162461bcd60e51b815260040161035990611a0e565b828114610ecc5760405162461bcd60e51b81526020600482015260536024820152600080516020611df183398151915260448201527f72793a206f776e6572426174636852656d6f76654e667454797065416e6452616064820152720dcd640d8cadccee8d0e640dad2e6dac2e8c6d606b1b608482015260a401610359565b60005b8381101561047557610f51858583818110610eec57610eec611a43565b90506020020135848484818110610f0557610f05611a43565b9050602002810190610f179190611a59565b8080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920191909152506110bf92505050565b80610f5b81611ab6565b915050610ecf565b60005b600354811015610ffd578260038281548110610f8457610f84611a43565b906000526020600020906002020160000154148015610fe15750818051906020012060038281548110610fb957610fb9611a43565b9060005260206000209060020201600101604051610fd79190611b5d565b6040518091039020145b15610feb57505050565b80610ff581611ab6565b915050610f66565b50604080518082019091528281526020810182815260038054600181018255600091909152825160029091027fc2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85b8101918255915190917fc2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85c01906110809082611c21565b505050817f2360ee3e77485441cfa07e30e8dc5b031fac38455647c89a763434f58733fcc1826040516110b39190611ce1565b60405180910390a25050565b60005b6003548110156112385782600382815481106110e0576110e0611a43565b90600052602060002090600202016000015414801561113d575081805190602001206003828154811061111557611115611a43565b90600052602060002090600202016001016040516111339190611b5d565b6040518091039020145b15611226576003805461115290600190611cf4565b8154811061116257611162611a43565b90600052602060002090600202016003828154811061118357611183611a43565b6000918252602090912082546002909202019081556001808201906111aa90840182611d07565b5090505060038054806111bf576111bf611dda565b600082815260208120600260001990930192830201818155906111e560018301826116b3565b50509055827fb1323e42d97b2b3d45f9d4641bf4b6b3f9d0d01e90832ae7b7413109b7a5d347836040516112199190611ce1565b60405180910390a2505050565b8061123081611ab6565b9150506110c2565b505050565b60005b60055481101561133357836005828154811061125e5761125e611a43565b9060005260206000209060020201600001541480156112bb575082805190602001206005828154811061129357611293611a43565b90600052602060002090600202016001016040516112b19190611b5d565b6040518091039020145b156113215781600482815481106112d4576112d4611a43565b906000526020600020018190555081847fe43bf5f5f8a1211930e5726ba0abceacb1748f97b2966db30a818ba10961cbcc856040516113139190611ce1565b60405180910390a350505050565b8061132b81611ab6565b915050611240565b604080518082019091528481526020810184815260058054600181018255600091909152825160029091027f036b6384b5eca791c62761152d0c79bb0604c104a5fb6f4eb0703f3154bb3db08101918255915190917f036b6384b5eca791c62761152d0c79bb0604c104a5fb6f4eb0703f3154bb3db101906113b59082611c21565b5050600480546001810182556000919091527f8a35acfbc15ff81a39ae7d344fd709f28e8600b4aa8c65c6b64bfe7fe36bd19b0183905550604051829085907fe43bf5f5f8a1211930e5726ba0abceacb1748f97b2966db30a818ba10961cbcc90611313908790611ce1565b600080546001600160a01b038381166001600160a01b0319831681178455604051919092169283917f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09190a35050565b600180546001600160a01b0319166001600160a01b0383169081179091556040517f573bbfa679af6fdcdbd9cf191c5ef3e526599ac2bf75e9177d47adb8530b9c6990600090a250565b60005b6005548110156112385782600582815481106114dc576114dc611a43565b906000526020600020906002020160000154148015611539575081805190602001206005828154811061151157611511611a43565b906000526020600020906002020160010160405161152f9190611b5d565b6040518091039020145b15611686576005805461154e90600190611cf4565b8154811061155e5761155e611a43565b90600052602060002090600202016005828154811061157f5761157f611a43565b6000918252602090912082546002909202019081556001808201906115a690840182611d07565b5090505060058054806115bb576115bb611dda565b600082815260208120600260001990930192830201818155906115e160018301826116b3565b50509055600480546115f590600190611cf4565b8154811061160557611605611a43565b90600052602060002001546004828154811061162357611623611a43565b600091825260209091200155600480548061164057611640611dda565b60019003818190600052602060002001600090559055827fdb75199103504bd1d3653de758d4295bf00d4587e1d53dfc114464cc47ed97b7836040516112199190611ce1565b8061169081611ab6565b9150506114be565b6000818310156116a857816116aa565b825b90505b92915050565b5080546116bf906119d4565b6000825580601f106116cf575050565b601f01602090049060005260206000209081019061074891905b808211156116fd57600081556001016116e9565b5090565b60006020828403121561171357600080fd5b5035919050565b6000815180845260005b8181101561174057602081850181015186830182015201611724565b506000602082860101526020601f19601f83011685010191505092915050565b828152604060208201526000611779604083018461171a565b949350505050565b60008083601f84011261179357600080fd5b50813567ffffffffffffffff8111156117ab57600080fd5b6020830191508360208260051b85010111156117c657600080fd5b9250929050565b600080600080604085870312156117e357600080fd5b843567ffffffffffffffff808211156117fb57600080fd5b61180788838901611781565b9096509450602087013591508082111561182057600080fd5b5061182d87828801611781565b95989497509550505050565b634e487b7160e01b600052604160045260246000fd5b6000806040838503121561186257600080fd5b82359150602083013567ffffffffffffffff8082111561188157600080fd5b818501915085601f83011261189557600080fd5b8135818111156118a7576118a7611839565b604051601f8201601f19908116603f011681019083821181831017156118cf576118cf611839565b816040528281528860208487010111156118e857600080fd5b8260208601602083013760006020848301015280955050505050509250929050565b6000806000806000806060878903121561192357600080fd5b863567ffffffffffffffff8082111561193b57600080fd5b6119478a838b01611781565b9098509650602089013591508082111561196057600080fd5b61196c8a838b01611781565b9096509450604089013591508082111561198557600080fd5b5061199289828a01611781565b979a9699509497509295939492505050565b6000602082840312156119b657600080fd5b81356001600160a01b03811681146119cd57600080fd5b9392505050565b600181811c908216806119e857607f821691505b602082108103611a0857634e487b7160e01b600052602260045260246000fd5b50919050565b6020808252818101527f4f776e61626c653a2063616c6c6572206973206e6f7420746865206f776e6572604082015260600190565b634e487b7160e01b600052603260045260246000fd5b6000808335601e19843603018112611a7057600080fd5b83018035915067ffffffffffffffff821115611a8b57600080fd5b6020019150368190038213156117c657600080fd5b634e487b7160e01b600052601160045260246000fd5b600060018201611ac857611ac8611aa0565b5060010190565b838152606060208201526000611ae8606083018561171a565b905060018060a01b0383166040830152949350505050565b600060208284031215611b1257600080fd5b815180151581146119cd57600080fd5b600060208284031215611b3457600080fd5b5051919050565b600082611b5857634e487b7160e01b600052601260045260246000fd5b500490565b6000808354611b6b816119d4565b60018281168015611b835760018114611b9857611bc7565b60ff1984168752821515830287019450611bc7565b8760005260208060002060005b85811015611bbe5781548a820152908401908201611ba5565b50505082870194505b50929695505050505050565b601f82111561123857600081815260208120601f850160051c81016020861015611bfa5750805b601f850160051c820191505b81811015611c1957828155600101611c06565b505050505050565b815167ffffffffffffffff811115611c3b57611c3b611839565b611c4f81611c4984546119d4565b84611bd3565b602080601f831160018114611c845760008415611c6c5750858301515b600019600386901b1c1916600185901b178555611c19565b600085815260208120601f198616915b82811015611cb357888601518255948401946001909101908401611c94565b5085821015611cd15787850151600019600388901b60f8161c191681555b5050505050600190811b01905550565b6020815260006116aa602083018461171a565b818103818111156116ad576116ad611aa0565b818103611d12575050565b611d1c82546119d4565b67ffffffffffffffff811115611d3457611d34611839565b611d4281611c4984546119d4565b6000601f821160018114611d765760008315611d5e5750848201545b600019600385901b1c1916600184901b178455610475565b600085815260209020601f19841690600086815260209020845b83811015611db05782860154825560019586019590910190602001611d90565b5085831015611cd15793015460001960f8600387901b161c19169092555050600190811b01905550565b634e487b7160e01b600052603160045260246000fdfe486f70725374616b696e6750726f7879466f724e6574776f726b526567697374a264697066735822122005f048532106093e784acf53b5b6fb31d79769cc4bd7cb56af5726e02a7eacb764736f6c63430008110033"
            .parse()
            .expect("invalid bytecode")
    });
    pub struct HoprStakingProxyForNetworkRegistry<M>(::ethers::contract::Contract<M>);
    impl<M> Clone for HoprStakingProxyForNetworkRegistry<M> {
        fn clone(&self) -> Self {
            HoprStakingProxyForNetworkRegistry(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for HoprStakingProxyForNetworkRegistry<M> {
        type Target = ::ethers::contract::Contract<M>;
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
    impl<M: ::ethers::providers::Middleware> HoprStakingProxyForNetworkRegistry<M> {
        /// Creates a new contract instance with the specified `ethers`
        /// client at the given `Address`. The contract derefs to a `ethers::Contract`
        /// object
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    HOPRSTAKINGPROXYFORNETWORKREGISTRY_ABI.clone(),
                    client,
                ),
            )
        }
        /// Constructs the general purpose `Deployer` instance based on the provided constructor arguments and sends it.
        /// Returns a new instance of a deployer that returns an instance of this contract after sending the transaction
        ///
        /// Notes:
        /// 1. If there are no constructor arguments, you should pass `()` as the argument.
        /// 1. The default poll duration is 7 seconds.
        /// 1. The default number of confirmations is 1 block.
        ///
        ///
        /// # Example
        ///
        /// Generate contract bindings with `abigen!` and deploy a new contract instance.
        ///
        /// *Note*: this requires a `bytecode` and `abi` object in the `greeter.json` artifact.
        ///
        /// ```ignore
        /// # async fn deploy<M: ethers::providers::Middleware>(client: ::std::sync::Arc<M>) {
        ///     abigen!(Greeter,"../greeter.json");
        ///
        ///    let greeter_contract = Greeter::deploy(client, "Hello world!".to_string()).unwrap().send().await.unwrap();
        ///    let msg = greeter_contract.greet().call().await.unwrap();
        /// # }
        /// ```
        pub fn deploy<T: ::ethers::core::abi::Tokenize>(
            client: ::std::sync::Arc<M>,
            constructor_args: T,
        ) -> ::std::result::Result<
            ::ethers::contract::builders::ContractDeployer<M, Self>,
            ::ethers::contract::ContractError<M>,
        > {
            let factory = ::ethers::contract::ContractFactory::new(
                HOPRSTAKINGPROXYFORNETWORKREGISTRY_ABI.clone(),
                HOPRSTAKINGPROXYFORNETWORKREGISTRY_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ::ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        ///Calls the contract's `eligibleNftTypeAndRank` (0xde626c0e) function
        pub fn eligible_nft_type_and_rank(
            &self,
            p0: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (::ethers::core::types::U256, String),
        > {
            self.0
                .method_hash([222, 98, 108, 14], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `maxAllowedRegistrations` (0xb3544e82) function
        pub fn max_allowed_registrations(
            &self,
            account: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([179, 84, 78, 130], account)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `maxRegistrationsPerSpecialNft` (0xba1cef23) function
        pub fn max_registrations_per_special_nft(
            &self,
            p0: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([186, 28, 239, 35], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `owner` (0x8da5cb5b) function
        pub fn owner(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([141, 165, 203, 91], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `ownerAddNftTypeAndRank` (0x9b97076f) function
        pub fn owner_add_nft_type_and_rank(
            &self,
            nft_type: ::ethers::core::types::U256,
            nft_rank: String,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([155, 151, 7, 111], (nft_type, nft_rank))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `ownerBatchAddNftTypeAndRank` (0x506472cc) function
        pub fn owner_batch_add_nft_type_and_rank(
            &self,
            nft_types: ::std::vec::Vec<::ethers::core::types::U256>,
            nft_ranks: ::std::vec::Vec<String>,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([80, 100, 114, 204], (nft_types, nft_ranks))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `ownerBatchAddSpecialNftTypeAndRank` (0x6a3b64b6) function
        pub fn owner_batch_add_special_nft_type_and_rank(
            &self,
            nft_types: ::std::vec::Vec<::ethers::core::types::U256>,
            nft_ranks: ::std::vec::Vec<String>,
            max_registrations: ::std::vec::Vec<::ethers::core::types::U256>,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [106, 59, 100, 182],
                    (nft_types, nft_ranks, max_registrations),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `ownerBatchRemoveNftTypeAndRank` (0xfb66ac57) function
        pub fn owner_batch_remove_nft_type_and_rank(
            &self,
            nft_types: ::std::vec::Vec<::ethers::core::types::U256>,
            nft_ranks: ::std::vec::Vec<String>,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([251, 102, 172, 87], (nft_types, nft_ranks))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `ownerBatchRemoveSpecialNftTypeAndRank` (0xb05e8ba9) function
        pub fn owner_batch_remove_special_nft_type_and_rank(
            &self,
            nft_types: ::std::vec::Vec<::ethers::core::types::U256>,
            nft_ranks: ::std::vec::Vec<String>,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([176, 94, 139, 169], (nft_types, nft_ranks))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `ownerRemoveNftTypeAndRank` (0x654251eb) function
        pub fn owner_remove_nft_type_and_rank(
            &self,
            nft_type: ::ethers::core::types::U256,
            nft_rank: String,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([101, 66, 81, 235], (nft_type, nft_rank))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `ownerUpdateThreshold` (0xee50c7c4) function
        pub fn owner_update_threshold(
            &self,
            new_threshold: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([238, 80, 199, 196], new_threshold)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `renounceOwnership` (0x715018a6) function
        pub fn renounce_ownership(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([113, 80, 24, 166], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `specialNftTypeAndRank` (0x2c3ec80b) function
        pub fn special_nft_type_and_rank(
            &self,
            p0: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (::ethers::core::types::U256, String),
        > {
            self.0
                .method_hash([44, 62, 200, 11], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `stakeContract` (0x1a186227) function
        pub fn stake_contract(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([26, 24, 98, 39], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `stakeThreshold` (0xf11f77f9) function
        pub fn stake_threshold(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([241, 31, 119, 249], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `transferOwnership` (0xf2fde38b) function
        pub fn transfer_ownership(
            &self,
            new_owner: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([242, 253, 227, 139], new_owner)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `updateStakeContract` (0x830c6cc2) function
        pub fn update_stake_contract(
            &self,
            stake_contract: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([131, 12, 108, 194], stake_contract)
                .expect("method not found (this should never happen)")
        }
        ///Gets the contract's `NftTypeAndRankAdded` event
        pub fn nft_type_and_rank_added_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<M, NftTypeAndRankAddedFilter> {
            self.0.event()
        }
        ///Gets the contract's `NftTypeAndRankRemoved` event
        pub fn nft_type_and_rank_removed_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<M, NftTypeAndRankRemovedFilter> {
            self.0.event()
        }
        ///Gets the contract's `OwnershipTransferred` event
        pub fn ownership_transferred_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<M, OwnershipTransferredFilter> {
            self.0.event()
        }
        ///Gets the contract's `SpecialNftTypeAndRankAdded` event
        pub fn special_nft_type_and_rank_added_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<M, SpecialNftTypeAndRankAddedFilter> {
            self.0.event()
        }
        ///Gets the contract's `SpecialNftTypeAndRankRemoved` event
        pub fn special_nft_type_and_rank_removed_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<M, SpecialNftTypeAndRankRemovedFilter> {
            self.0.event()
        }
        ///Gets the contract's `StakeContractUpdated` event
        pub fn stake_contract_updated_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<M, StakeContractUpdatedFilter> {
            self.0.event()
        }
        ///Gets the contract's `ThresholdUpdated` event
        pub fn threshold_updated_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<M, ThresholdUpdatedFilter> {
            self.0.event()
        }
        /// Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<
            M,
            HoprStakingProxyForNetworkRegistryEvents,
        > {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for HoprStakingProxyForNetworkRegistry<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethevent(
        name = "NftTypeAndRankAdded",
        abi = "NftTypeAndRankAdded(uint256,string)"
    )]
    pub struct NftTypeAndRankAddedFilter {
        #[ethevent(indexed)]
        pub nft_type: ::ethers::core::types::U256,
        pub nft_rank: String,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethevent(
        name = "NftTypeAndRankRemoved",
        abi = "NftTypeAndRankRemoved(uint256,string)"
    )]
    pub struct NftTypeAndRankRemovedFilter {
        #[ethevent(indexed)]
        pub nft_type: ::ethers::core::types::U256,
        pub nft_rank: String,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethevent(
        name = "OwnershipTransferred",
        abi = "OwnershipTransferred(address,address)"
    )]
    pub struct OwnershipTransferredFilter {
        #[ethevent(indexed)]
        pub previous_owner: ::ethers::core::types::Address,
        #[ethevent(indexed)]
        pub new_owner: ::ethers::core::types::Address,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethevent(
        name = "SpecialNftTypeAndRankAdded",
        abi = "SpecialNftTypeAndRankAdded(uint256,string,uint256)"
    )]
    pub struct SpecialNftTypeAndRankAddedFilter {
        #[ethevent(indexed)]
        pub nft_type: ::ethers::core::types::U256,
        pub nft_rank: String,
        #[ethevent(indexed)]
        pub max_registration: ::ethers::core::types::U256,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethevent(
        name = "SpecialNftTypeAndRankRemoved",
        abi = "SpecialNftTypeAndRankRemoved(uint256,string)"
    )]
    pub struct SpecialNftTypeAndRankRemovedFilter {
        #[ethevent(indexed)]
        pub nft_type: ::ethers::core::types::U256,
        pub nft_rank: String,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethevent(name = "StakeContractUpdated", abi = "StakeContractUpdated(address)")]
    pub struct StakeContractUpdatedFilter {
        #[ethevent(indexed)]
        pub stake_contract: ::ethers::core::types::Address,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethevent(name = "ThresholdUpdated", abi = "ThresholdUpdated(uint256)")]
    pub struct ThresholdUpdatedFilter {
        #[ethevent(indexed)]
        pub threshold: ::ethers::core::types::U256,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ::ethers::contract::EthAbiType)]
    pub enum HoprStakingProxyForNetworkRegistryEvents {
        NftTypeAndRankAddedFilter(NftTypeAndRankAddedFilter),
        NftTypeAndRankRemovedFilter(NftTypeAndRankRemovedFilter),
        OwnershipTransferredFilter(OwnershipTransferredFilter),
        SpecialNftTypeAndRankAddedFilter(SpecialNftTypeAndRankAddedFilter),
        SpecialNftTypeAndRankRemovedFilter(SpecialNftTypeAndRankRemovedFilter),
        StakeContractUpdatedFilter(StakeContractUpdatedFilter),
        ThresholdUpdatedFilter(ThresholdUpdatedFilter),
    }
    impl ::ethers::contract::EthLogDecode for HoprStakingProxyForNetworkRegistryEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ::ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = NftTypeAndRankAddedFilter::decode_log(log) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryEvents::NftTypeAndRankAddedFilter(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded) = NftTypeAndRankRemovedFilter::decode_log(log) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryEvents::NftTypeAndRankRemovedFilter(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded) = OwnershipTransferredFilter::decode_log(log) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryEvents::OwnershipTransferredFilter(
                        decoded,
                    ),
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
                    HoprStakingProxyForNetworkRegistryEvents::StakeContractUpdatedFilter(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded) = ThresholdUpdatedFilter::decode_log(log) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryEvents::ThresholdUpdatedFilter(
                        decoded,
                    ),
                );
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for HoprStakingProxyForNetworkRegistryEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                HoprStakingProxyForNetworkRegistryEvents::NftTypeAndRankAddedFilter(
                    element,
                ) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryEvents::NftTypeAndRankRemovedFilter(
                    element,
                ) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryEvents::OwnershipTransferredFilter(
                    element,
                ) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryEvents::SpecialNftTypeAndRankAddedFilter(
                    element,
                ) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryEvents::SpecialNftTypeAndRankRemovedFilter(
                    element,
                ) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryEvents::StakeContractUpdatedFilter(
                    element,
                ) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryEvents::ThresholdUpdatedFilter(
                    element,
                ) => element.fmt(f),
            }
        }
    }
    ///Container type for all input parameters for the `eligibleNftTypeAndRank` function with signature `eligibleNftTypeAndRank(uint256)` and selector `0xde626c0e`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "eligibleNftTypeAndRank", abi = "eligibleNftTypeAndRank(uint256)")]
    pub struct EligibleNftTypeAndRankCall(pub ::ethers::core::types::U256);
    ///Container type for all input parameters for the `maxAllowedRegistrations` function with signature `maxAllowedRegistrations(address)` and selector `0xb3544e82`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(
        name = "maxAllowedRegistrations",
        abi = "maxAllowedRegistrations(address)"
    )]
    pub struct MaxAllowedRegistrationsCall {
        pub account: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `maxRegistrationsPerSpecialNft` function with signature `maxRegistrationsPerSpecialNft(uint256)` and selector `0xba1cef23`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(
        name = "maxRegistrationsPerSpecialNft",
        abi = "maxRegistrationsPerSpecialNft(uint256)"
    )]
    pub struct MaxRegistrationsPerSpecialNftCall(pub ::ethers::core::types::U256);
    ///Container type for all input parameters for the `owner` function with signature `owner()` and selector `0x8da5cb5b`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "owner", abi = "owner()")]
    pub struct OwnerCall;
    ///Container type for all input parameters for the `ownerAddNftTypeAndRank` function with signature `ownerAddNftTypeAndRank(uint256,string)` and selector `0x9b97076f`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(
        name = "ownerAddNftTypeAndRank",
        abi = "ownerAddNftTypeAndRank(uint256,string)"
    )]
    pub struct OwnerAddNftTypeAndRankCall {
        pub nft_type: ::ethers::core::types::U256,
        pub nft_rank: String,
    }
    ///Container type for all input parameters for the `ownerBatchAddNftTypeAndRank` function with signature `ownerBatchAddNftTypeAndRank(uint256[],string[])` and selector `0x506472cc`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(
        name = "ownerBatchAddNftTypeAndRank",
        abi = "ownerBatchAddNftTypeAndRank(uint256[],string[])"
    )]
    pub struct OwnerBatchAddNftTypeAndRankCall {
        pub nft_types: ::std::vec::Vec<::ethers::core::types::U256>,
        pub nft_ranks: ::std::vec::Vec<String>,
    }
    ///Container type for all input parameters for the `ownerBatchAddSpecialNftTypeAndRank` function with signature `ownerBatchAddSpecialNftTypeAndRank(uint256[],string[],uint256[])` and selector `0x6a3b64b6`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(
        name = "ownerBatchAddSpecialNftTypeAndRank",
        abi = "ownerBatchAddSpecialNftTypeAndRank(uint256[],string[],uint256[])"
    )]
    pub struct OwnerBatchAddSpecialNftTypeAndRankCall {
        pub nft_types: ::std::vec::Vec<::ethers::core::types::U256>,
        pub nft_ranks: ::std::vec::Vec<String>,
        pub max_registrations: ::std::vec::Vec<::ethers::core::types::U256>,
    }
    ///Container type for all input parameters for the `ownerBatchRemoveNftTypeAndRank` function with signature `ownerBatchRemoveNftTypeAndRank(uint256[],string[])` and selector `0xfb66ac57`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(
        name = "ownerBatchRemoveNftTypeAndRank",
        abi = "ownerBatchRemoveNftTypeAndRank(uint256[],string[])"
    )]
    pub struct OwnerBatchRemoveNftTypeAndRankCall {
        pub nft_types: ::std::vec::Vec<::ethers::core::types::U256>,
        pub nft_ranks: ::std::vec::Vec<String>,
    }
    ///Container type for all input parameters for the `ownerBatchRemoveSpecialNftTypeAndRank` function with signature `ownerBatchRemoveSpecialNftTypeAndRank(uint256[],string[])` and selector `0xb05e8ba9`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(
        name = "ownerBatchRemoveSpecialNftTypeAndRank",
        abi = "ownerBatchRemoveSpecialNftTypeAndRank(uint256[],string[])"
    )]
    pub struct OwnerBatchRemoveSpecialNftTypeAndRankCall {
        pub nft_types: ::std::vec::Vec<::ethers::core::types::U256>,
        pub nft_ranks: ::std::vec::Vec<String>,
    }
    ///Container type for all input parameters for the `ownerRemoveNftTypeAndRank` function with signature `ownerRemoveNftTypeAndRank(uint256,string)` and selector `0x654251eb`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(
        name = "ownerRemoveNftTypeAndRank",
        abi = "ownerRemoveNftTypeAndRank(uint256,string)"
    )]
    pub struct OwnerRemoveNftTypeAndRankCall {
        pub nft_type: ::ethers::core::types::U256,
        pub nft_rank: String,
    }
    ///Container type for all input parameters for the `ownerUpdateThreshold` function with signature `ownerUpdateThreshold(uint256)` and selector `0xee50c7c4`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "ownerUpdateThreshold", abi = "ownerUpdateThreshold(uint256)")]
    pub struct OwnerUpdateThresholdCall {
        pub new_threshold: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `renounceOwnership` function with signature `renounceOwnership()` and selector `0x715018a6`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "renounceOwnership", abi = "renounceOwnership()")]
    pub struct RenounceOwnershipCall;
    ///Container type for all input parameters for the `specialNftTypeAndRank` function with signature `specialNftTypeAndRank(uint256)` and selector `0x2c3ec80b`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "specialNftTypeAndRank", abi = "specialNftTypeAndRank(uint256)")]
    pub struct SpecialNftTypeAndRankCall(pub ::ethers::core::types::U256);
    ///Container type for all input parameters for the `stakeContract` function with signature `stakeContract()` and selector `0x1a186227`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "stakeContract", abi = "stakeContract()")]
    pub struct StakeContractCall;
    ///Container type for all input parameters for the `stakeThreshold` function with signature `stakeThreshold()` and selector `0xf11f77f9`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "stakeThreshold", abi = "stakeThreshold()")]
    pub struct StakeThresholdCall;
    ///Container type for all input parameters for the `transferOwnership` function with signature `transferOwnership(address)` and selector `0xf2fde38b`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "transferOwnership", abi = "transferOwnership(address)")]
    pub struct TransferOwnershipCall {
        pub new_owner: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `updateStakeContract` function with signature `updateStakeContract(address)` and selector `0x830c6cc2`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "updateStakeContract", abi = "updateStakeContract(address)")]
    pub struct UpdateStakeContractCall {
        pub stake_contract: ::ethers::core::types::Address,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ::ethers::contract::EthAbiType)]
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
    impl ::ethers::core::abi::AbiDecode for HoprStakingProxyForNetworkRegistryCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ::ethers::core::abi::AbiError> {
            if let Ok(decoded)
                = <EligibleNftTypeAndRankCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryCalls::EligibleNftTypeAndRank(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded)
                = <MaxAllowedRegistrationsCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryCalls::MaxAllowedRegistrations(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded)
                = <MaxRegistrationsPerSpecialNftCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryCalls::MaxRegistrationsPerSpecialNft(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded)
                = <OwnerCall as ::ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(HoprStakingProxyForNetworkRegistryCalls::Owner(decoded));
            }
            if let Ok(decoded)
                = <OwnerAddNftTypeAndRankCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryCalls::OwnerAddNftTypeAndRank(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded)
                = <OwnerBatchAddNftTypeAndRankCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryCalls::OwnerBatchAddNftTypeAndRank(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded)
                = <OwnerBatchAddSpecialNftTypeAndRankCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryCalls::OwnerBatchAddSpecialNftTypeAndRank(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded)
                = <OwnerBatchRemoveNftTypeAndRankCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryCalls::OwnerBatchRemoveNftTypeAndRank(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded)
                = <OwnerBatchRemoveSpecialNftTypeAndRankCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryCalls::OwnerBatchRemoveSpecialNftTypeAndRank(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded)
                = <OwnerRemoveNftTypeAndRankCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryCalls::OwnerRemoveNftTypeAndRank(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded)
                = <OwnerUpdateThresholdCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryCalls::OwnerUpdateThreshold(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded)
                = <RenounceOwnershipCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryCalls::RenounceOwnership(decoded),
                );
            }
            if let Ok(decoded)
                = <SpecialNftTypeAndRankCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryCalls::SpecialNftTypeAndRank(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded)
                = <StakeContractCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryCalls::StakeContract(decoded),
                );
            }
            if let Ok(decoded)
                = <StakeThresholdCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryCalls::StakeThreshold(decoded),
                );
            }
            if let Ok(decoded)
                = <TransferOwnershipCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryCalls::TransferOwnership(decoded),
                );
            }
            if let Ok(decoded)
                = <UpdateStakeContractCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(
                    HoprStakingProxyForNetworkRegistryCalls::UpdateStakeContract(decoded),
                );
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for HoprStakingProxyForNetworkRegistryCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                HoprStakingProxyForNetworkRegistryCalls::EligibleNftTypeAndRank(
                    element,
                ) => element.encode(),
                HoprStakingProxyForNetworkRegistryCalls::MaxAllowedRegistrations(
                    element,
                ) => element.encode(),
                HoprStakingProxyForNetworkRegistryCalls::MaxRegistrationsPerSpecialNft(
                    element,
                ) => element.encode(),
                HoprStakingProxyForNetworkRegistryCalls::Owner(element) => {
                    element.encode()
                }
                HoprStakingProxyForNetworkRegistryCalls::OwnerAddNftTypeAndRank(
                    element,
                ) => element.encode(),
                HoprStakingProxyForNetworkRegistryCalls::OwnerBatchAddNftTypeAndRank(
                    element,
                ) => element.encode(),
                HoprStakingProxyForNetworkRegistryCalls::OwnerBatchAddSpecialNftTypeAndRank(
                    element,
                ) => element.encode(),
                HoprStakingProxyForNetworkRegistryCalls::OwnerBatchRemoveNftTypeAndRank(
                    element,
                ) => element.encode(),
                HoprStakingProxyForNetworkRegistryCalls::OwnerBatchRemoveSpecialNftTypeAndRank(
                    element,
                ) => element.encode(),
                HoprStakingProxyForNetworkRegistryCalls::OwnerRemoveNftTypeAndRank(
                    element,
                ) => element.encode(),
                HoprStakingProxyForNetworkRegistryCalls::OwnerUpdateThreshold(
                    element,
                ) => element.encode(),
                HoprStakingProxyForNetworkRegistryCalls::RenounceOwnership(element) => {
                    element.encode()
                }
                HoprStakingProxyForNetworkRegistryCalls::SpecialNftTypeAndRank(
                    element,
                ) => element.encode(),
                HoprStakingProxyForNetworkRegistryCalls::StakeContract(element) => {
                    element.encode()
                }
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
                HoprStakingProxyForNetworkRegistryCalls::EligibleNftTypeAndRank(
                    element,
                ) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryCalls::MaxAllowedRegistrations(
                    element,
                ) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryCalls::MaxRegistrationsPerSpecialNft(
                    element,
                ) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryCalls::Owner(element) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryCalls::OwnerAddNftTypeAndRank(
                    element,
                ) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryCalls::OwnerBatchAddNftTypeAndRank(
                    element,
                ) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryCalls::OwnerBatchAddSpecialNftTypeAndRank(
                    element,
                ) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryCalls::OwnerBatchRemoveNftTypeAndRank(
                    element,
                ) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryCalls::OwnerBatchRemoveSpecialNftTypeAndRank(
                    element,
                ) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryCalls::OwnerRemoveNftTypeAndRank(
                    element,
                ) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryCalls::OwnerUpdateThreshold(
                    element,
                ) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryCalls::RenounceOwnership(element) => {
                    element.fmt(f)
                }
                HoprStakingProxyForNetworkRegistryCalls::SpecialNftTypeAndRank(
                    element,
                ) => element.fmt(f),
                HoprStakingProxyForNetworkRegistryCalls::StakeContract(element) => {
                    element.fmt(f)
                }
                HoprStakingProxyForNetworkRegistryCalls::StakeThreshold(element) => {
                    element.fmt(f)
                }
                HoprStakingProxyForNetworkRegistryCalls::TransferOwnership(element) => {
                    element.fmt(f)
                }
                HoprStakingProxyForNetworkRegistryCalls::UpdateStakeContract(element) => {
                    element.fmt(f)
                }
            }
        }
    }
    impl ::std::convert::From<EligibleNftTypeAndRankCall>
    for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: EligibleNftTypeAndRankCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::EligibleNftTypeAndRank(var)
        }
    }
    impl ::std::convert::From<MaxAllowedRegistrationsCall>
    for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: MaxAllowedRegistrationsCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::MaxAllowedRegistrations(var)
        }
    }
    impl ::std::convert::From<MaxRegistrationsPerSpecialNftCall>
    for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: MaxRegistrationsPerSpecialNftCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::MaxRegistrationsPerSpecialNft(var)
        }
    }
    impl ::std::convert::From<OwnerCall> for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: OwnerCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::Owner(var)
        }
    }
    impl ::std::convert::From<OwnerAddNftTypeAndRankCall>
    for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: OwnerAddNftTypeAndRankCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::OwnerAddNftTypeAndRank(var)
        }
    }
    impl ::std::convert::From<OwnerBatchAddNftTypeAndRankCall>
    for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: OwnerBatchAddNftTypeAndRankCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::OwnerBatchAddNftTypeAndRank(var)
        }
    }
    impl ::std::convert::From<OwnerBatchAddSpecialNftTypeAndRankCall>
    for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: OwnerBatchAddSpecialNftTypeAndRankCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::OwnerBatchAddSpecialNftTypeAndRank(
                var,
            )
        }
    }
    impl ::std::convert::From<OwnerBatchRemoveNftTypeAndRankCall>
    for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: OwnerBatchRemoveNftTypeAndRankCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::OwnerBatchRemoveNftTypeAndRank(var)
        }
    }
    impl ::std::convert::From<OwnerBatchRemoveSpecialNftTypeAndRankCall>
    for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: OwnerBatchRemoveSpecialNftTypeAndRankCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::OwnerBatchRemoveSpecialNftTypeAndRank(
                var,
            )
        }
    }
    impl ::std::convert::From<OwnerRemoveNftTypeAndRankCall>
    for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: OwnerRemoveNftTypeAndRankCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::OwnerRemoveNftTypeAndRank(var)
        }
    }
    impl ::std::convert::From<OwnerUpdateThresholdCall>
    for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: OwnerUpdateThresholdCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::OwnerUpdateThreshold(var)
        }
    }
    impl ::std::convert::From<RenounceOwnershipCall>
    for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: RenounceOwnershipCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::RenounceOwnership(var)
        }
    }
    impl ::std::convert::From<SpecialNftTypeAndRankCall>
    for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: SpecialNftTypeAndRankCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::SpecialNftTypeAndRank(var)
        }
    }
    impl ::std::convert::From<StakeContractCall>
    for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: StakeContractCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::StakeContract(var)
        }
    }
    impl ::std::convert::From<StakeThresholdCall>
    for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: StakeThresholdCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::StakeThreshold(var)
        }
    }
    impl ::std::convert::From<TransferOwnershipCall>
    for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: TransferOwnershipCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::TransferOwnership(var)
        }
    }
    impl ::std::convert::From<UpdateStakeContractCall>
    for HoprStakingProxyForNetworkRegistryCalls {
        fn from(var: UpdateStakeContractCall) -> Self {
            HoprStakingProxyForNetworkRegistryCalls::UpdateStakeContract(var)
        }
    }
    ///Container type for all return fields from the `eligibleNftTypeAndRank` function with signature `eligibleNftTypeAndRank(uint256)` and selector `0xde626c0e`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
    )]
    #[derive(Default)]
    pub struct EligibleNftTypeAndRankReturn {
        pub nft_type: ::ethers::core::types::U256,
        pub nft_rank: String,
    }
    ///Container type for all return fields from the `maxAllowedRegistrations` function with signature `maxAllowedRegistrations(address)` and selector `0xb3544e82`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
    )]
    #[derive(Default)]
    pub struct MaxAllowedRegistrationsReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `maxRegistrationsPerSpecialNft` function with signature `maxRegistrationsPerSpecialNft(uint256)` and selector `0xba1cef23`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
    )]
    #[derive(Default)]
    pub struct MaxRegistrationsPerSpecialNftReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `owner` function with signature `owner()` and selector `0x8da5cb5b`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
    )]
    #[derive(Default)]
    pub struct OwnerReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `specialNftTypeAndRank` function with signature `specialNftTypeAndRank(uint256)` and selector `0x2c3ec80b`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
    )]
    #[derive(Default)]
    pub struct SpecialNftTypeAndRankReturn {
        pub nft_type: ::ethers::core::types::U256,
        pub nft_rank: String,
    }
    ///Container type for all return fields from the `stakeContract` function with signature `stakeContract()` and selector `0x1a186227`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
    )]
    #[derive(Default)]
    pub struct StakeContractReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `stakeThreshold` function with signature `stakeThreshold()` and selector `0xf11f77f9`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
    )]
    #[derive(Default)]
    pub struct StakeThresholdReturn(pub ::ethers::core::types::U256);
}
