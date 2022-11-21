pub use hopr_network_registry::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod hopr_network_registry {
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
    #[doc = "HoprNetworkRegistry was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_requirementImplementation\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"_newOwner\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"peerId\",\"type\":\"string\",\"components\":[]}],\"type\":\"error\",\"name\":\"InvalidPeerId\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"string\",\"name\":\"hoprPeerId\",\"type\":\"string\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Deregistered\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"string\",\"name\":\"hoprPeerId\",\"type\":\"string\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"DeregisteredByOwner\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"bool\",\"name\":\"eligibility\",\"type\":\"bool\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"EligibilityUpdated\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"bool\",\"name\":\"isEnabled\",\"type\":\"bool\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"EnabledNetworkRegistry\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"previousOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"OwnershipTransferred\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"string\",\"name\":\"hoprPeerId\",\"type\":\"string\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Registered\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"string\",\"name\":\"hoprPeerId\",\"type\":\"string\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"RegisteredByOwner\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"requirementImplementation\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"RequirementUpdated\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"countRegisterdNodesPerAccount\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"disableRegistry\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"enableRegistry\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"enabled\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"isAccountRegisteredAndEligible\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"hoprPeerId\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"isNodeRegisteredAndEligible\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"nodePeerIdToAccount\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"owner\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string[]\",\"name\":\"hoprPeerIds\",\"type\":\"string[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"ownerDeregister\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address[]\",\"name\":\"accounts\",\"type\":\"address[]\",\"components\":[]},{\"internalType\":\"bool[]\",\"name\":\"eligibility\",\"type\":\"bool[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"ownerForceEligibility\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address[]\",\"name\":\"accounts\",\"type\":\"address[]\",\"components\":[]},{\"internalType\":\"string[]\",\"name\":\"hoprPeerIds\",\"type\":\"string[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"ownerRegister\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"renounceOwnership\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"requirementImplementation\",\"outputs\":[{\"internalType\":\"contract IHoprNetworkRegistryRequirement\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string[]\",\"name\":\"hoprPeerIds\",\"type\":\"string[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"selfDeregister\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"string[]\",\"name\":\"hoprPeerIds\",\"type\":\"string[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"selfRegister\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"string[]\",\"name\":\"hoprPeerIds\",\"type\":\"string[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"sync\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferOwnership\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_requirementImplementation\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"updateRequirementImplementation\",\"outputs\":[]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static HOPRNETWORKREGISTRY_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static HOPRNETWORKREGISTRY_BYTECODE: ethers::contract::Lazy<ethers::core::types::Bytes> =
        ethers::contract::Lazy::new(|| {
            "0x60806040523480156200001157600080fd5b506040516200190938038062001909833981016040819052620000349162000147565b6200003f33620000da565b600180546001600160a01b0319166001600160a01b0384161781556004805460ff191690911790556200007281620000da565b6040516001600160a01b038316907f8ac4b2eb7749f75c5b99b898e547fd615dd7a424e68356ea196b7dae742d6c3290600090a26040516001907f3f749856d23f89c3cefe0f1055d280fc302c5d6adb048bc20f5d975239c3049290600090a250506200017f565b600080546001600160a01b038381166001600160a01b0319831681178455604051919092169283917f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09190a35050565b80516001600160a01b03811681146200014257600080fd5b919050565b600080604083850312156200015b57600080fd5b62000166836200012a565b915062000176602084016200012a565b90509250929050565b61177a806200018f6000396000f3fe608060405234801561001057600080fd5b50600436106101165760003560e01c80638da5cb5b116100a2578063cfb3735f11610071578063cfb3735f1461025f578063d86a517014610267578063dd41c7461461027a578063e3d2a0711461028d578063f2fde38b146102a057600080fd5b80638da5cb5b14610220578063a3c676dd14610231578063af4c4cf714610244578063cbebafe01461025757600080fd5b806336e1aa80116100e957806336e1aa80146101b15780633e38949c146101c45780633fa58457146101d75780635d1c396414610205578063715018a61461021857600080fd5b8063036167ae1461011b578063238dafe01461013057806327b040a1146101525780632e6e031814610165575b600080fd5b61012e6101293660046112c2565b6102b3565b005b60045461013d9060ff1681565b60405190151581526020015b60405180910390f35b61012e610160366004611304565b610439565b61019961017336600461134a565b80516020818301810180516003825292820191909301209152546001600160a01b031681565b6040516001600160a01b039091168152602001610149565b61012e6101bf3660046113fb565b6104ad565b61012e6101d23660046112c2565b61071f565b6101f76101e5366004611304565b60026020526000908152604090205481565b604051908152602001610149565b61013d610213366004611304565b610943565b61012e610974565b6000546001600160a01b0316610199565b61012e61023f3660046112c2565b6109aa565b61013d610252366004611467565b610cbc565b61012e610d0c565b61012e610d8f565b61012e6102753660046112c2565b610e58565b61012e6102883660046113fb565b610fb6565b600154610199906001600160a01b031681565b61012e6102ae366004611304565b6110e5565b6000546001600160a01b031633146102e65760405162461bcd60e51b81526004016102dd906114d9565b60405180910390fd5b60005b818110156104345760008383838181106103055761030561150e565b90506020028101906103179190611524565b8080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920182905250604051949550936003935061035f925085915061159b565b908152604051908190036020019020546001600160a01b03169050801561041f57600382604051610390919061159b565b908152604080516020928190038301902080546001600160a01b03191690556001600160a01b0383166000908152600290925281208054600192906103d69084906115cd565b92505081905550806001600160a01b03167f7303b218be576befd7afef7d2f9088c49eb7c26c155dba016527f71ee2f4ad0c8360405161041691906115e4565b60405180910390a25b5050808061042c90611617565b9150506102e9565b505050565b6000546001600160a01b031633146104635760405162461bcd60e51b81526004016102dd906114d9565b600180546001600160a01b0319166001600160a01b0383169081179091556040517f8ac4b2eb7749f75c5b99b898e547fd615dd7a424e68356ea196b7dae742d6c3290600090a250565b6000546001600160a01b031633146104d75760405162461bcd60e51b81526004016102dd906114d9565b80831461054c5760405162461bcd60e51b815260206004820152603f60248201527f486f70724e6574776f726b52656769737472793a20686f70725065657249646560448201527f7320616e64206163636f756e7473206c656e67746873206d69736d617463680060648201526084016102dd565b60005b83811015610718578282828181106105695761056961150e565b905060200281019061057b9190611524565b905060351480156105d157508282828181106105995761059961150e565b90506020028101906105ab9190611524565b6105ba91600891600091611630565b6105c39161165a565b67313655697532484160c01b145b156107065760008383838181106105ea576105ea61150e565b90506020028101906105fc9190611524565b8080601f01602080910402602001604051908101604052809392919081815260200183838082843760009201829052509394508992508891508590508181106106475761064761150e565b905060200201602081019061065c9190611304565b90508060038360405161066f919061159b565b908152604080516020928190038301902080546001600160a01b0319166001600160a01b03948516179055918316600090815260029091529081208054600192906106bb908490611678565b92505081905550806001600160a01b03167f87b2f82f8766cb6651342bc0a77cfb41521b857c0dd7f38e751c2dfd21820c23836040516106fb91906115e4565b60405180910390a250505b8061071081611617565b91505061054f565b5050505050565b60045460ff166107415760405162461bcd60e51b81526004016102dd90611690565b33600090815260026020526040812080548392906107609084906115cd565b9091555061076f905033611180565b1561079557604051600190339060008051602061172583398151915290600090a36107b1565b6040516000903390600080516020611725833981519152908390a35b60005b818110156104345760008383838181106107d0576107d061150e565b90506020028101906107e29190611524565b8080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920191909152505060405192935033926003925061082c9150849061159b565b908152604051908190036020019020546001600160a01b0316146108cc5760405162461bcd60e51b815260206004820152604b60248201527f486f70724e6574776f726b52656769737472793a2043616e6e6f742064656c6560448201527f746520616e20656e747279206e6f74206173736f63696174656420776974682060648201526a3a34329031b0b63632b91760a91b608482015260a4016102dd565b6003816040516108dc919061159b565b90815260405190819003602001812080546001600160a01b031916905533907fed392d6c60bfee7cf61b9bc8bbcf48abb41bd5884565367943576fcd56a058e1906109289084906115e4565b60405180910390a2508061093b81611617565b9150506107b4565b6001600160a01b0381166000908152600260205260408120541580159061096e575061096e82611180565b92915050565b6000546001600160a01b0316331461099e5760405162461bcd60e51b81526004016102dd906114d9565b6109a86000611226565b565b60045460ff166109cc5760405162461bcd60e51b81526004016102dd90611690565b33600090815260026020526040812080548392906109eb908490611678565b909155506109fa905033611180565b610a865760405162461bcd60e51b815260206004820152605160248201527f486f70724e6574776f726b52656769737472793a2073656c665265676973746560448201527f722072656163686573206c696d69742c2063616e6e6f74207265676973746572606482015270103932b8bab2b9ba32b2103737b232b99760791b608482015260a4016102dd565b604051600190339060008051602061172583398151915290600090a360005b81811015610434576000838383818110610ac157610ac161150e565b9050602002810190610ad39190611524565b8080601f01602080910402602001604051908101604052809392919081815260200183838082843760009201919091525050825192935050506035141580610b615750838383818110610b2857610b2861150e565b9050602002810190610b3a9190611524565b610b4991600891600091611630565b610b529161165a565b67313655697532484160c01b14155b15610b81578060405163754c8e5960e11b81526004016102dd91906115e4565b6000600382604051610b93919061159b565b908152604051908190036020019020546001600160a01b03169050338103610bbc575050610caa565b6001600160a01b03811615610c2f5760405162461bcd60e51b815260206004820152603360248201527f486f70724e6574776f726b52656769737472793a2043616e6e6f74206c696e6b6044820152721030903932b3b4b9ba32b932b2103737b2329760691b60648201526084016102dd565b33600383604051610c40919061159b565b90815260405190819003602001812080546001600160a01b03939093166001600160a01b03199093169290921790915533907fb3eccf73f39b1c07947c780b2b39df2a1bb058b4037b0a42d0881ca1a028a13290610c9f9085906115e4565b60405180910390a250505b80610cb481611617565b915050610aa5565b60008060038484604051610cd19291906116d9565b908152604051908190036020019020546001600160a01b0316905080610cfb57600091505061096e565b610d0481611180565b949350505050565b6000546001600160a01b03163314610d365760405162461bcd60e51b81526004016102dd906114d9565b60045460ff16610d585760405162461bcd60e51b81526004016102dd90611690565b6004805460ff191690556040516000907f3f749856d23f89c3cefe0f1055d280fc302c5d6adb048bc20f5d975239c30492908290a2565b6000546001600160a01b03163314610db95760405162461bcd60e51b81526004016102dd906114d9565b60045460ff1615610e1d5760405162461bcd60e51b815260206004820152602860248201527f486f70724e6574776f726b52656769737472793a20526567697374727920697360448201526708195b98589b195960c21b60648201526084016102dd565b6004805460ff191660019081179091556040517f3f749856d23f89c3cefe0f1055d280fc302c5d6adb048bc20f5d975239c3049290600090a2565b6000546001600160a01b03163314610e825760405162461bcd60e51b81526004016102dd906114d9565b60045460ff16610ea45760405162461bcd60e51b81526004016102dd90611690565b60005b81811015610434576000838383818110610ec357610ec361150e565b9050602002810190610ed59190611524565b8080601f01602080910402602001604051908101604052809392919081815260200183838082843760009201829052506040519495509360039350610f1d925085915061159b565b908152604051908190036020019020546001600160a01b0316905080610f44575050610fa4565b610f4d81611180565b15610f7c576040516001906001600160a01b0383169060008051602061172583398151915290600090a3610fa1565b6040516000906001600160a01b03831690600080516020611725833981519152908390a35b50505b80610fae81611617565b915050610ea7565b6000546001600160a01b03163314610fe05760405162461bcd60e51b81526004016102dd906114d9565b8281146110555760405162461bcd60e51b815260206004820152603e60248201527f486f70724e6574776f726b52656769737472793a206163636f756e747320616e60448201527f6420656c69676962696c697479206c656e67746873206d69736d61746368000060648201526084016102dd565b60005b83811015610718578282828181106110725761107261150e565b905060200201602081019061108791906116e9565b151585858381811061109b5761109b61150e565b90506020020160208101906110b09190611304565b6001600160a01b031660008051602061172583398151915260405160405180910390a3806110dd81611617565b915050611058565b6000546001600160a01b0316331461110f5760405162461bcd60e51b81526004016102dd906114d9565b6001600160a01b0381166111745760405162461bcd60e51b815260206004820152602660248201527f4f776e61626c653a206e6577206f776e657220697320746865207a65726f206160448201526564647265737360d01b60648201526084016102dd565b61117d81611226565b50565b6001546040516359aa274160e11b81526001600160a01b038381166004830152600092839291169063b3544e8290602401602060405180830381865afa1580156111ce573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906111f2919061170b565b6001600160a01b038416600090815260026020526040902054909150811061121d5750600192915050565b50600092915050565b600080546001600160a01b038381166001600160a01b0319831681178455604051919092169283917f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e09190a35050565b60008083601f84011261128857600080fd5b50813567ffffffffffffffff8111156112a057600080fd5b6020830191508360208260051b85010111156112bb57600080fd5b9250929050565b600080602083850312156112d557600080fd5b823567ffffffffffffffff8111156112ec57600080fd5b6112f885828601611276565b90969095509350505050565b60006020828403121561131657600080fd5b81356001600160a01b038116811461132d57600080fd5b9392505050565b634e487b7160e01b600052604160045260246000fd5b60006020828403121561135c57600080fd5b813567ffffffffffffffff8082111561137457600080fd5b818401915084601f83011261138857600080fd5b81358181111561139a5761139a611334565b604051601f8201601f19908116603f011681019083821181831017156113c2576113c2611334565b816040528281528760208487010111156113db57600080fd5b826020860160208301376000928101602001929092525095945050505050565b6000806000806040858703121561141157600080fd5b843567ffffffffffffffff8082111561142957600080fd5b61143588838901611276565b9096509450602087013591508082111561144e57600080fd5b5061145b87828801611276565b95989497509550505050565b6000806020838503121561147a57600080fd5b823567ffffffffffffffff8082111561149257600080fd5b818501915085601f8301126114a657600080fd5b8135818111156114b557600080fd5b8660208285010111156114c757600080fd5b60209290920196919550909350505050565b6020808252818101527f4f776e61626c653a2063616c6c6572206973206e6f7420746865206f776e6572604082015260600190565b634e487b7160e01b600052603260045260246000fd5b6000808335601e1984360301811261153b57600080fd5b83018035915067ffffffffffffffff82111561155657600080fd5b6020019150368190038213156112bb57600080fd5b60005b8381101561158657818101518382015260200161156e565b83811115611595576000848401525b50505050565b600082516115ad81846020870161156b565b9190910192915050565b634e487b7160e01b600052601160045260246000fd5b6000828210156115df576115df6115b7565b500390565b602081526000825180602084015261160381604085016020870161156b565b601f01601f19169190910160400192915050565b600060018201611629576116296115b7565b5060010190565b6000808585111561164057600080fd5b8386111561164d57600080fd5b5050820193919092039150565b8035602083101561096e57600019602084900360031b1b1692915050565b6000821982111561168b5761168b6115b7565b500190565b60208082526029908201527f486f70724e6574776f726b52656769737472793a20526567697374727920697360408201526808191a5cd8589b195960ba1b606082015260800190565b8183823760009101908152919050565b6000602082840312156116fb57600080fd5b8135801515811461132d57600080fd5b60006020828403121561171d57600080fd5b505191905056fee2994f8d6f600ad473dba82c0a890ab7affacb860d3365f474baa3dc04a2e557a26469706673582212207b7b594594b98d04fb7ba125ebe6869db03fe6b2b6ac107ec692cb44b274e94864736f6c634300080d0033" . parse () . expect ("invalid bytecode")
        });
    pub struct HoprNetworkRegistry<M>(ethers::contract::Contract<M>);
    impl<M> Clone for HoprNetworkRegistry<M> {
        fn clone(&self) -> Self {
            HoprNetworkRegistry(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for HoprNetworkRegistry<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for HoprNetworkRegistry<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(HoprNetworkRegistry))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> HoprNetworkRegistry<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), HOPRNETWORKREGISTRY_ABI.clone(), client)
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
                HOPRNETWORKREGISTRY_ABI.clone(),
                HOPRNETWORKREGISTRY_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        #[doc = "Calls the contract's `countRegisterdNodesPerAccount` (0x3fa58457) function"]
        pub fn count_registerd_nodes_per_account(
            &self,
            p0: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([63, 165, 132, 87], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `disableRegistry` (0xcbebafe0) function"]
        pub fn disable_registry(&self) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([203, 235, 175, 224], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `enableRegistry` (0xcfb3735f) function"]
        pub fn enable_registry(&self) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([207, 179, 115, 95], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `enabled` (0x238dafe0) function"]
        pub fn enabled(&self) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([35, 141, 175, 224], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `isAccountRegisteredAndEligible` (0x5d1c3964) function"]
        pub fn is_account_registered_and_eligible(
            &self,
            account: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([93, 28, 57, 100], account)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `isNodeRegisteredAndEligible` (0xaf4c4cf7) function"]
        pub fn is_node_registered_and_eligible(
            &self,
            hopr_peer_id: String,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([175, 76, 76, 247], hopr_peer_id)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `nodePeerIdToAccount` (0x2e6e0318) function"]
        pub fn node_peer_id_to_account(
            &self,
            p0: String,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([46, 110, 3, 24], p0)
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
        #[doc = "Calls the contract's `ownerDeregister` (0x036167ae) function"]
        pub fn owner_deregister(
            &self,
            hopr_peer_ids: ::std::vec::Vec<String>,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([3, 97, 103, 174], hopr_peer_ids)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `ownerForceEligibility` (0xdd41c746) function"]
        pub fn owner_force_eligibility(
            &self,
            accounts: ::std::vec::Vec<ethers::core::types::Address>,
            eligibility: ::std::vec::Vec<bool>,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([221, 65, 199, 70], (accounts, eligibility))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `ownerRegister` (0x36e1aa80) function"]
        pub fn owner_register(
            &self,
            accounts: ::std::vec::Vec<ethers::core::types::Address>,
            hopr_peer_ids: ::std::vec::Vec<String>,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([54, 225, 170, 128], (accounts, hopr_peer_ids))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `renounceOwnership` (0x715018a6) function"]
        pub fn renounce_ownership(&self) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([113, 80, 24, 166], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `requirementImplementation` (0xe3d2a071) function"]
        pub fn requirement_implementation(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([227, 210, 160, 113], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `selfDeregister` (0x3e38949c) function"]
        pub fn self_deregister(
            &self,
            hopr_peer_ids: ::std::vec::Vec<String>,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([62, 56, 148, 156], hopr_peer_ids)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `selfRegister` (0xa3c676dd) function"]
        pub fn self_register(
            &self,
            hopr_peer_ids: ::std::vec::Vec<String>,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([163, 198, 118, 221], hopr_peer_ids)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `sync` (0xd86a5170) function"]
        pub fn sync(
            &self,
            hopr_peer_ids: ::std::vec::Vec<String>,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([216, 106, 81, 112], hopr_peer_ids)
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
        #[doc = "Calls the contract's `updateRequirementImplementation` (0x27b040a1) function"]
        pub fn update_requirement_implementation(
            &self,
            requirement_implementation: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([39, 176, 64, 161], requirement_implementation)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Gets the contract's `Deregistered` event"]
        pub fn deregistered_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, DeregisteredFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `DeregisteredByOwner` event"]
        pub fn deregistered_by_owner_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, DeregisteredByOwnerFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `EligibilityUpdated` event"]
        pub fn eligibility_updated_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, EligibilityUpdatedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `EnabledNetworkRegistry` event"]
        pub fn enabled_network_registry_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, EnabledNetworkRegistryFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `OwnershipTransferred` event"]
        pub fn ownership_transferred_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, OwnershipTransferredFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Registered` event"]
        pub fn registered_filter(&self) -> ethers::contract::builders::Event<M, RegisteredFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `RegisteredByOwner` event"]
        pub fn registered_by_owner_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, RegisteredByOwnerFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `RequirementUpdated` event"]
        pub fn requirement_updated_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, RequirementUpdatedFilter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(&self) -> ethers::contract::builders::Event<M, HoprNetworkRegistryEvents> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>>
        for HoprNetworkRegistry<M>
    {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
    #[doc = "Custom Error type `InvalidPeerId` with signature `InvalidPeerId(string)` and selector `[234, 153, 28, 178]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthError,
        ethers :: contract :: EthDisplay,
    )]
    #[etherror(name = "InvalidPeerId", abi = "InvalidPeerId(string)")]
    pub struct InvalidPeerId {
        pub peer_id: String,
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
    #[ethevent(name = "Deregistered", abi = "Deregistered(address,string)")]
    pub struct DeregisteredFilter {
        #[ethevent(indexed)]
        pub account: ethers::core::types::Address,
        pub hopr_peer_id: String,
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
        name = "DeregisteredByOwner",
        abi = "DeregisteredByOwner(address,string)"
    )]
    pub struct DeregisteredByOwnerFilter {
        #[ethevent(indexed)]
        pub account: ethers::core::types::Address,
        pub hopr_peer_id: String,
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
    #[ethevent(name = "EligibilityUpdated", abi = "EligibilityUpdated(address,bool)")]
    pub struct EligibilityUpdatedFilter {
        #[ethevent(indexed)]
        pub account: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub eligibility: bool,
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
    #[ethevent(name = "EnabledNetworkRegistry", abi = "EnabledNetworkRegistry(bool)")]
    pub struct EnabledNetworkRegistryFilter {
        #[ethevent(indexed)]
        pub is_enabled: bool,
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
    #[ethevent(name = "Registered", abi = "Registered(address,string)")]
    pub struct RegisteredFilter {
        #[ethevent(indexed)]
        pub account: ethers::core::types::Address,
        pub hopr_peer_id: String,
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
    #[ethevent(name = "RegisteredByOwner", abi = "RegisteredByOwner(address,string)")]
    pub struct RegisteredByOwnerFilter {
        #[ethevent(indexed)]
        pub account: ethers::core::types::Address,
        pub hopr_peer_id: String,
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
    #[ethevent(name = "RequirementUpdated", abi = "RequirementUpdated(address)")]
    pub struct RequirementUpdatedFilter {
        #[ethevent(indexed)]
        pub requirement_implementation: ethers::core::types::Address,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum HoprNetworkRegistryEvents {
        DeregisteredFilter(DeregisteredFilter),
        DeregisteredByOwnerFilter(DeregisteredByOwnerFilter),
        EligibilityUpdatedFilter(EligibilityUpdatedFilter),
        EnabledNetworkRegistryFilter(EnabledNetworkRegistryFilter),
        OwnershipTransferredFilter(OwnershipTransferredFilter),
        RegisteredFilter(RegisteredFilter),
        RegisteredByOwnerFilter(RegisteredByOwnerFilter),
        RequirementUpdatedFilter(RequirementUpdatedFilter),
    }
    impl ethers::contract::EthLogDecode for HoprNetworkRegistryEvents {
        fn decode_log(
            log: &ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = DeregisteredFilter::decode_log(log) {
                return Ok(HoprNetworkRegistryEvents::DeregisteredFilter(decoded));
            }
            if let Ok(decoded) = DeregisteredByOwnerFilter::decode_log(log) {
                return Ok(HoprNetworkRegistryEvents::DeregisteredByOwnerFilter(
                    decoded,
                ));
            }
            if let Ok(decoded) = EligibilityUpdatedFilter::decode_log(log) {
                return Ok(HoprNetworkRegistryEvents::EligibilityUpdatedFilter(decoded));
            }
            if let Ok(decoded) = EnabledNetworkRegistryFilter::decode_log(log) {
                return Ok(HoprNetworkRegistryEvents::EnabledNetworkRegistryFilter(
                    decoded,
                ));
            }
            if let Ok(decoded) = OwnershipTransferredFilter::decode_log(log) {
                return Ok(HoprNetworkRegistryEvents::OwnershipTransferredFilter(
                    decoded,
                ));
            }
            if let Ok(decoded) = RegisteredFilter::decode_log(log) {
                return Ok(HoprNetworkRegistryEvents::RegisteredFilter(decoded));
            }
            if let Ok(decoded) = RegisteredByOwnerFilter::decode_log(log) {
                return Ok(HoprNetworkRegistryEvents::RegisteredByOwnerFilter(decoded));
            }
            if let Ok(decoded) = RequirementUpdatedFilter::decode_log(log) {
                return Ok(HoprNetworkRegistryEvents::RequirementUpdatedFilter(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for HoprNetworkRegistryEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                HoprNetworkRegistryEvents::DeregisteredFilter(element) => element.fmt(f),
                HoprNetworkRegistryEvents::DeregisteredByOwnerFilter(element) => element.fmt(f),
                HoprNetworkRegistryEvents::EligibilityUpdatedFilter(element) => element.fmt(f),
                HoprNetworkRegistryEvents::EnabledNetworkRegistryFilter(element) => element.fmt(f),
                HoprNetworkRegistryEvents::OwnershipTransferredFilter(element) => element.fmt(f),
                HoprNetworkRegistryEvents::RegisteredFilter(element) => element.fmt(f),
                HoprNetworkRegistryEvents::RegisteredByOwnerFilter(element) => element.fmt(f),
                HoprNetworkRegistryEvents::RequirementUpdatedFilter(element) => element.fmt(f),
            }
        }
    }
    #[doc = "Container type for all input parameters for the `countRegisterdNodesPerAccount` function with signature `countRegisterdNodesPerAccount(address)` and selector `[63, 165, 132, 87]`"]
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
        name = "countRegisterdNodesPerAccount",
        abi = "countRegisterdNodesPerAccount(address)"
    )]
    pub struct CountRegisterdNodesPerAccountCall(pub ethers::core::types::Address);
    #[doc = "Container type for all input parameters for the `disableRegistry` function with signature `disableRegistry()` and selector `[203, 235, 175, 224]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "disableRegistry", abi = "disableRegistry()")]
    pub struct DisableRegistryCall;
    #[doc = "Container type for all input parameters for the `enableRegistry` function with signature `enableRegistry()` and selector `[207, 179, 115, 95]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "enableRegistry", abi = "enableRegistry()")]
    pub struct EnableRegistryCall;
    #[doc = "Container type for all input parameters for the `enabled` function with signature `enabled()` and selector `[35, 141, 175, 224]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "enabled", abi = "enabled()")]
    pub struct EnabledCall;
    #[doc = "Container type for all input parameters for the `isAccountRegisteredAndEligible` function with signature `isAccountRegisteredAndEligible(address)` and selector `[93, 28, 57, 100]`"]
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
        name = "isAccountRegisteredAndEligible",
        abi = "isAccountRegisteredAndEligible(address)"
    )]
    pub struct IsAccountRegisteredAndEligibleCall {
        pub account: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `isNodeRegisteredAndEligible` function with signature `isNodeRegisteredAndEligible(string)` and selector `[175, 76, 76, 247]`"]
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
        name = "isNodeRegisteredAndEligible",
        abi = "isNodeRegisteredAndEligible(string)"
    )]
    pub struct IsNodeRegisteredAndEligibleCall {
        pub hopr_peer_id: String,
    }
    #[doc = "Container type for all input parameters for the `nodePeerIdToAccount` function with signature `nodePeerIdToAccount(string)` and selector `[46, 110, 3, 24]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "nodePeerIdToAccount", abi = "nodePeerIdToAccount(string)")]
    pub struct NodePeerIdToAccountCall(pub String);
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
    #[doc = "Container type for all input parameters for the `ownerDeregister` function with signature `ownerDeregister(string[])` and selector `[3, 97, 103, 174]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "ownerDeregister", abi = "ownerDeregister(string[])")]
    pub struct OwnerDeregisterCall {
        pub hopr_peer_ids: ::std::vec::Vec<String>,
    }
    #[doc = "Container type for all input parameters for the `ownerForceEligibility` function with signature `ownerForceEligibility(address[],bool[])` and selector `[221, 65, 199, 70]`"]
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
        name = "ownerForceEligibility",
        abi = "ownerForceEligibility(address[],bool[])"
    )]
    pub struct OwnerForceEligibilityCall {
        pub accounts: ::std::vec::Vec<ethers::core::types::Address>,
        pub eligibility: ::std::vec::Vec<bool>,
    }
    #[doc = "Container type for all input parameters for the `ownerRegister` function with signature `ownerRegister(address[],string[])` and selector `[54, 225, 170, 128]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "ownerRegister", abi = "ownerRegister(address[],string[])")]
    pub struct OwnerRegisterCall {
        pub accounts: ::std::vec::Vec<ethers::core::types::Address>,
        pub hopr_peer_ids: ::std::vec::Vec<String>,
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
    #[doc = "Container type for all input parameters for the `requirementImplementation` function with signature `requirementImplementation()` and selector `[227, 210, 160, 113]`"]
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
        name = "requirementImplementation",
        abi = "requirementImplementation()"
    )]
    pub struct RequirementImplementationCall;
    #[doc = "Container type for all input parameters for the `selfDeregister` function with signature `selfDeregister(string[])` and selector `[62, 56, 148, 156]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "selfDeregister", abi = "selfDeregister(string[])")]
    pub struct SelfDeregisterCall {
        pub hopr_peer_ids: ::std::vec::Vec<String>,
    }
    #[doc = "Container type for all input parameters for the `selfRegister` function with signature `selfRegister(string[])` and selector `[163, 198, 118, 221]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "selfRegister", abi = "selfRegister(string[])")]
    pub struct SelfRegisterCall {
        pub hopr_peer_ids: ::std::vec::Vec<String>,
    }
    #[doc = "Container type for all input parameters for the `sync` function with signature `sync(string[])` and selector `[216, 106, 81, 112]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "sync", abi = "sync(string[])")]
    pub struct SyncCall {
        pub hopr_peer_ids: ::std::vec::Vec<String>,
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
    #[doc = "Container type for all input parameters for the `updateRequirementImplementation` function with signature `updateRequirementImplementation(address)` and selector `[39, 176, 64, 161]`"]
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
        name = "updateRequirementImplementation",
        abi = "updateRequirementImplementation(address)"
    )]
    pub struct UpdateRequirementImplementationCall {
        pub requirement_implementation: ethers::core::types::Address,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum HoprNetworkRegistryCalls {
        CountRegisterdNodesPerAccount(CountRegisterdNodesPerAccountCall),
        DisableRegistry(DisableRegistryCall),
        EnableRegistry(EnableRegistryCall),
        Enabled(EnabledCall),
        IsAccountRegisteredAndEligible(IsAccountRegisteredAndEligibleCall),
        IsNodeRegisteredAndEligible(IsNodeRegisteredAndEligibleCall),
        NodePeerIdToAccount(NodePeerIdToAccountCall),
        Owner(OwnerCall),
        OwnerDeregister(OwnerDeregisterCall),
        OwnerForceEligibility(OwnerForceEligibilityCall),
        OwnerRegister(OwnerRegisterCall),
        RenounceOwnership(RenounceOwnershipCall),
        RequirementImplementation(RequirementImplementationCall),
        SelfDeregister(SelfDeregisterCall),
        SelfRegister(SelfRegisterCall),
        Sync(SyncCall),
        TransferOwnership(TransferOwnershipCall),
        UpdateRequirementImplementation(UpdateRequirementImplementationCall),
    }
    impl ethers::core::abi::AbiDecode for HoprNetworkRegistryCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <CountRegisterdNodesPerAccountCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(HoprNetworkRegistryCalls::CountRegisterdNodesPerAccount(
                    decoded,
                ));
            }
            if let Ok(decoded) =
                <DisableRegistryCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprNetworkRegistryCalls::DisableRegistry(decoded));
            }
            if let Ok(decoded) =
                <EnableRegistryCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprNetworkRegistryCalls::EnableRegistry(decoded));
            }
            if let Ok(decoded) =
                <EnabledCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprNetworkRegistryCalls::Enabled(decoded));
            }
            if let Ok(decoded) =
                <IsAccountRegisteredAndEligibleCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(HoprNetworkRegistryCalls::IsAccountRegisteredAndEligible(
                    decoded,
                ));
            }
            if let Ok(decoded) =
                <IsNodeRegisteredAndEligibleCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(HoprNetworkRegistryCalls::IsNodeRegisteredAndEligible(
                    decoded,
                ));
            }
            if let Ok(decoded) =
                <NodePeerIdToAccountCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprNetworkRegistryCalls::NodePeerIdToAccount(decoded));
            }
            if let Ok(decoded) = <OwnerCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprNetworkRegistryCalls::Owner(decoded));
            }
            if let Ok(decoded) =
                <OwnerDeregisterCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprNetworkRegistryCalls::OwnerDeregister(decoded));
            }
            if let Ok(decoded) =
                <OwnerForceEligibilityCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprNetworkRegistryCalls::OwnerForceEligibility(decoded));
            }
            if let Ok(decoded) =
                <OwnerRegisterCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprNetworkRegistryCalls::OwnerRegister(decoded));
            }
            if let Ok(decoded) =
                <RenounceOwnershipCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprNetworkRegistryCalls::RenounceOwnership(decoded));
            }
            if let Ok(decoded) =
                <RequirementImplementationCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(HoprNetworkRegistryCalls::RequirementImplementation(decoded));
            }
            if let Ok(decoded) =
                <SelfDeregisterCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprNetworkRegistryCalls::SelfDeregister(decoded));
            }
            if let Ok(decoded) =
                <SelfRegisterCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprNetworkRegistryCalls::SelfRegister(decoded));
            }
            if let Ok(decoded) = <SyncCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(HoprNetworkRegistryCalls::Sync(decoded));
            }
            if let Ok(decoded) =
                <TransferOwnershipCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprNetworkRegistryCalls::TransferOwnership(decoded));
            }
            if let Ok(decoded) =
                <UpdateRequirementImplementationCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(HoprNetworkRegistryCalls::UpdateRequirementImplementation(
                    decoded,
                ));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for HoprNetworkRegistryCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                HoprNetworkRegistryCalls::CountRegisterdNodesPerAccount(element) => {
                    element.encode()
                }
                HoprNetworkRegistryCalls::DisableRegistry(element) => element.encode(),
                HoprNetworkRegistryCalls::EnableRegistry(element) => element.encode(),
                HoprNetworkRegistryCalls::Enabled(element) => element.encode(),
                HoprNetworkRegistryCalls::IsAccountRegisteredAndEligible(element) => {
                    element.encode()
                }
                HoprNetworkRegistryCalls::IsNodeRegisteredAndEligible(element) => element.encode(),
                HoprNetworkRegistryCalls::NodePeerIdToAccount(element) => element.encode(),
                HoprNetworkRegistryCalls::Owner(element) => element.encode(),
                HoprNetworkRegistryCalls::OwnerDeregister(element) => element.encode(),
                HoprNetworkRegistryCalls::OwnerForceEligibility(element) => element.encode(),
                HoprNetworkRegistryCalls::OwnerRegister(element) => element.encode(),
                HoprNetworkRegistryCalls::RenounceOwnership(element) => element.encode(),
                HoprNetworkRegistryCalls::RequirementImplementation(element) => element.encode(),
                HoprNetworkRegistryCalls::SelfDeregister(element) => element.encode(),
                HoprNetworkRegistryCalls::SelfRegister(element) => element.encode(),
                HoprNetworkRegistryCalls::Sync(element) => element.encode(),
                HoprNetworkRegistryCalls::TransferOwnership(element) => element.encode(),
                HoprNetworkRegistryCalls::UpdateRequirementImplementation(element) => {
                    element.encode()
                }
            }
        }
    }
    impl ::std::fmt::Display for HoprNetworkRegistryCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                HoprNetworkRegistryCalls::CountRegisterdNodesPerAccount(element) => element.fmt(f),
                HoprNetworkRegistryCalls::DisableRegistry(element) => element.fmt(f),
                HoprNetworkRegistryCalls::EnableRegistry(element) => element.fmt(f),
                HoprNetworkRegistryCalls::Enabled(element) => element.fmt(f),
                HoprNetworkRegistryCalls::IsAccountRegisteredAndEligible(element) => element.fmt(f),
                HoprNetworkRegistryCalls::IsNodeRegisteredAndEligible(element) => element.fmt(f),
                HoprNetworkRegistryCalls::NodePeerIdToAccount(element) => element.fmt(f),
                HoprNetworkRegistryCalls::Owner(element) => element.fmt(f),
                HoprNetworkRegistryCalls::OwnerDeregister(element) => element.fmt(f),
                HoprNetworkRegistryCalls::OwnerForceEligibility(element) => element.fmt(f),
                HoprNetworkRegistryCalls::OwnerRegister(element) => element.fmt(f),
                HoprNetworkRegistryCalls::RenounceOwnership(element) => element.fmt(f),
                HoprNetworkRegistryCalls::RequirementImplementation(element) => element.fmt(f),
                HoprNetworkRegistryCalls::SelfDeregister(element) => element.fmt(f),
                HoprNetworkRegistryCalls::SelfRegister(element) => element.fmt(f),
                HoprNetworkRegistryCalls::Sync(element) => element.fmt(f),
                HoprNetworkRegistryCalls::TransferOwnership(element) => element.fmt(f),
                HoprNetworkRegistryCalls::UpdateRequirementImplementation(element) => {
                    element.fmt(f)
                }
            }
        }
    }
    impl ::std::convert::From<CountRegisterdNodesPerAccountCall> for HoprNetworkRegistryCalls {
        fn from(var: CountRegisterdNodesPerAccountCall) -> Self {
            HoprNetworkRegistryCalls::CountRegisterdNodesPerAccount(var)
        }
    }
    impl ::std::convert::From<DisableRegistryCall> for HoprNetworkRegistryCalls {
        fn from(var: DisableRegistryCall) -> Self {
            HoprNetworkRegistryCalls::DisableRegistry(var)
        }
    }
    impl ::std::convert::From<EnableRegistryCall> for HoprNetworkRegistryCalls {
        fn from(var: EnableRegistryCall) -> Self {
            HoprNetworkRegistryCalls::EnableRegistry(var)
        }
    }
    impl ::std::convert::From<EnabledCall> for HoprNetworkRegistryCalls {
        fn from(var: EnabledCall) -> Self {
            HoprNetworkRegistryCalls::Enabled(var)
        }
    }
    impl ::std::convert::From<IsAccountRegisteredAndEligibleCall> for HoprNetworkRegistryCalls {
        fn from(var: IsAccountRegisteredAndEligibleCall) -> Self {
            HoprNetworkRegistryCalls::IsAccountRegisteredAndEligible(var)
        }
    }
    impl ::std::convert::From<IsNodeRegisteredAndEligibleCall> for HoprNetworkRegistryCalls {
        fn from(var: IsNodeRegisteredAndEligibleCall) -> Self {
            HoprNetworkRegistryCalls::IsNodeRegisteredAndEligible(var)
        }
    }
    impl ::std::convert::From<NodePeerIdToAccountCall> for HoprNetworkRegistryCalls {
        fn from(var: NodePeerIdToAccountCall) -> Self {
            HoprNetworkRegistryCalls::NodePeerIdToAccount(var)
        }
    }
    impl ::std::convert::From<OwnerCall> for HoprNetworkRegistryCalls {
        fn from(var: OwnerCall) -> Self {
            HoprNetworkRegistryCalls::Owner(var)
        }
    }
    impl ::std::convert::From<OwnerDeregisterCall> for HoprNetworkRegistryCalls {
        fn from(var: OwnerDeregisterCall) -> Self {
            HoprNetworkRegistryCalls::OwnerDeregister(var)
        }
    }
    impl ::std::convert::From<OwnerForceEligibilityCall> for HoprNetworkRegistryCalls {
        fn from(var: OwnerForceEligibilityCall) -> Self {
            HoprNetworkRegistryCalls::OwnerForceEligibility(var)
        }
    }
    impl ::std::convert::From<OwnerRegisterCall> for HoprNetworkRegistryCalls {
        fn from(var: OwnerRegisterCall) -> Self {
            HoprNetworkRegistryCalls::OwnerRegister(var)
        }
    }
    impl ::std::convert::From<RenounceOwnershipCall> for HoprNetworkRegistryCalls {
        fn from(var: RenounceOwnershipCall) -> Self {
            HoprNetworkRegistryCalls::RenounceOwnership(var)
        }
    }
    impl ::std::convert::From<RequirementImplementationCall> for HoprNetworkRegistryCalls {
        fn from(var: RequirementImplementationCall) -> Self {
            HoprNetworkRegistryCalls::RequirementImplementation(var)
        }
    }
    impl ::std::convert::From<SelfDeregisterCall> for HoprNetworkRegistryCalls {
        fn from(var: SelfDeregisterCall) -> Self {
            HoprNetworkRegistryCalls::SelfDeregister(var)
        }
    }
    impl ::std::convert::From<SelfRegisterCall> for HoprNetworkRegistryCalls {
        fn from(var: SelfRegisterCall) -> Self {
            HoprNetworkRegistryCalls::SelfRegister(var)
        }
    }
    impl ::std::convert::From<SyncCall> for HoprNetworkRegistryCalls {
        fn from(var: SyncCall) -> Self {
            HoprNetworkRegistryCalls::Sync(var)
        }
    }
    impl ::std::convert::From<TransferOwnershipCall> for HoprNetworkRegistryCalls {
        fn from(var: TransferOwnershipCall) -> Self {
            HoprNetworkRegistryCalls::TransferOwnership(var)
        }
    }
    impl ::std::convert::From<UpdateRequirementImplementationCall> for HoprNetworkRegistryCalls {
        fn from(var: UpdateRequirementImplementationCall) -> Self {
            HoprNetworkRegistryCalls::UpdateRequirementImplementation(var)
        }
    }
    #[doc = "Container type for all return fields from the `countRegisterdNodesPerAccount` function with signature `countRegisterdNodesPerAccount(address)` and selector `[63, 165, 132, 87]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct CountRegisterdNodesPerAccountReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `enabled` function with signature `enabled()` and selector `[35, 141, 175, 224]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct EnabledReturn(pub bool);
    #[doc = "Container type for all return fields from the `isAccountRegisteredAndEligible` function with signature `isAccountRegisteredAndEligible(address)` and selector `[93, 28, 57, 100]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct IsAccountRegisteredAndEligibleReturn(pub bool);
    #[doc = "Container type for all return fields from the `isNodeRegisteredAndEligible` function with signature `isNodeRegisteredAndEligible(string)` and selector `[175, 76, 76, 247]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct IsNodeRegisteredAndEligibleReturn(pub bool);
    #[doc = "Container type for all return fields from the `nodePeerIdToAccount` function with signature `nodePeerIdToAccount(string)` and selector `[46, 110, 3, 24]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct NodePeerIdToAccountReturn(pub ethers::core::types::Address);
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
    #[doc = "Container type for all return fields from the `requirementImplementation` function with signature `requirementImplementation()` and selector `[227, 210, 160, 113]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct RequirementImplementationReturn(pub ethers::core::types::Address);
}
