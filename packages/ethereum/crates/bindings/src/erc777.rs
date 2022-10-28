pub use erc777::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod erc777 {
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
    #[doc = "ERC777 was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"string\",\"name\":\"name_\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"symbol_\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"defaultOperators_\",\"type\":\"address[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"owner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"spender\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Approval\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"tokenHolder\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"AuthorizedOperator\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Burned\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Minted\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"tokenHolder\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"RevokedOperator\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Sent\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Transfer\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"holder\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"spender\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"allowance\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"spender\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"approve\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"authorizeOperator\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"tokenHolder\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"balanceOf\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"burn\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"pure\",\"type\":\"function\",\"name\":\"decimals\",\"outputs\":[{\"internalType\":\"uint8\",\"name\":\"\",\"type\":\"uint8\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"defaultOperators\",\"outputs\":[{\"internalType\":\"address[]\",\"name\":\"\",\"type\":\"address[]\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"granularity\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"tokenHolder\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"isOperatorFor\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"name\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"operatorBurn\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"recipient\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"operatorSend\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"revokeOperator\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"recipient\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"send\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"symbol\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"totalSupply\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"recipient\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transfer\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"holder\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"recipient\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferFrom\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static ERC777_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static ERC777_BYTECODE: ethers::contract::Lazy<ethers::core::types::Bytes> =
        ethers::contract::Lazy::new(|| {
            "0x60806040523480156200001157600080fd5b50604051620023b0380380620023b0833981810160405260608110156200003757600080fd5b81019080805160405193929190846401000000008211156200005857600080fd5b9083019060208201858111156200006e57600080fd5b82516401000000008111828201881017156200008957600080fd5b82525081516020918201929091019080838360005b83811015620000b85781810151838201526020016200009e565b50505050905090810190601f168015620000e65780820380516001836020036101000a031916815260200191505b50604052602001805160405193929190846401000000008211156200010a57600080fd5b9083019060208201858111156200012057600080fd5b82516401000000008111828201881017156200013b57600080fd5b82525081516020918201929091019080838360005b838110156200016a57818101518382015260200162000150565b50505050905090810190601f168015620001985780820380516001836020036101000a031916815260200191505b5060405260200180516040519392919084640100000000821115620001bc57600080fd5b908301906020820185811115620001d257600080fd5b8251866020820283011164010000000082111715620001f057600080fd5b82525081516020918201928201910280838360005b838110156200021f57818101518382015260200162000205565b5050505091909101604052505084516200024392506002915060208601906200040b565b508151620002599060039060208501906200040b565b5080516200026f90600490602084019062000490565b5060005b600454811015620002cf57600160056000600484815481106200029257fe5b6000918252602080832091909101546001600160a01b031683528201929092526040019020805460ff191691151591909117905560010162000273565b50604080516329965a1d60e01b815230600482018190527fac7fbab5f54a3ca8194167523c6753bfeb96a445279294b6125b68cce2177054602483015260448201529051731820a4b7618bde71dce8cdc73aab6c95905fad24916329965a1d91606480830192600092919082900301818387803b1580156200035057600080fd5b505af115801562000365573d6000803e3d6000fd5b5050604080516329965a1d60e01b815230600482018190527faea199e31a596269b42cdafd93407f14436db6e4cad65417994c2eb37381e05a602483015260448201529051731820a4b7618bde71dce8cdc73aab6c95905fad2493506329965a1d9250606480830192600092919082900301818387803b158015620003e957600080fd5b505af1158015620003fe573d6000803e3d6000fd5b505050505050506200052e565b828054600181600116156101000203166002900490600052602060002090601f016020900481019282601f106200044e57805160ff19168380011785556200047e565b828001600101855582156200047e579182015b828111156200047e57825182559160200191906001019062000461565b506200048c929150620004f6565b5090565b828054828255906000526020600020908101928215620004e8579160200282015b82811115620004e857825182546001600160a01b0319166001600160a01b03909116178255602090920191600190910190620004b1565b506200048c9291506200050d565b5b808211156200048c5760008155600101620004f7565b5b808211156200048c5780546001600160a01b03191681556001016200050e565b611e72806200053e6000396000f3fe608060405234801561001057600080fd5b50600436106101165760003560e01c8063959b8c3f116100a2578063d95b637111610071578063d95b63711461052a578063dd62ed3e14610558578063fad8b32a14610586578063fc673c4f146105ac578063fe9d9303146106ea57610116565b8063959b8c3f1461041757806395d89b411461043d5780639bd9bbc614610445578063a9059cbb146104fe57610116565b806323b872dd116100e957806323b872dd1461024a578063313ce56714610280578063556f0dc71461029e57806362ad1b83146102a657806370a08231146103f157610116565b806306e485381461011b57806306fdde0314610173578063095ea7b3146101f057806318160ddd14610230575b600080fd5b610123610795565b60408051602080825283518183015283519192839290830191858101910280838360005b8381101561015f578181015183820152602001610147565b505050509050019250505060405180910390f35b61017b6107f7565b6040805160208082528351818301528351919283929083019185019080838360005b838110156101b557818101518382015260200161019d565b50505050905090810190601f1680156101e25780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b61021c6004803603604081101561020657600080fd5b506001600160a01b038135169060200135610881565b604080519115158252519081900360200190f35b6102386108a3565b60408051918252519081900360200190f35b61021c6004803603606081101561026057600080fd5b506001600160a01b038135811691602081013590911690604001356108a9565b610288610a26565b6040805160ff9092168252519081900360200190f35b610238610a2b565b6103ef600480360360a08110156102bc57600080fd5b6001600160a01b03823581169260208101359091169160408201359190810190608081016060820135600160201b8111156102f657600080fd5b82018360208201111561030857600080fd5b803590602001918460018302840111600160201b8311171561032957600080fd5b91908080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920191909152509295949360208101935035915050600160201b81111561037b57600080fd5b82018360208201111561038d57600080fd5b803590602001918460018302840111600160201b831117156103ae57600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600092019190915250929550610a30945050505050565b005b6102386004803603602081101561040757600080fd5b50356001600160a01b0316610a92565b6103ef6004803603602081101561042d57600080fd5b50356001600160a01b0316610aad565b61017b610bf9565b6103ef6004803603606081101561045b57600080fd5b6001600160a01b0382351691602081013591810190606081016040820135600160201b81111561048a57600080fd5b82018360208201111561049c57600080fd5b803590602001918460018302840111600160201b831117156104bd57600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600092019190915250929550610c5a945050505050565b61021c6004803603604081101561051457600080fd5b506001600160a01b038135169060200135610c84565b61021c6004803603604081101561054057600080fd5b506001600160a01b0381358116916020013516610d5d565b6102386004803603604081101561056e57600080fd5b506001600160a01b0381358116916020013516610dff565b6103ef6004803603602081101561059c57600080fd5b50356001600160a01b0316610e2a565b6103ef600480360360808110156105c257600080fd5b6001600160a01b0382351691602081013591810190606081016040820135600160201b8111156105f157600080fd5b82018360208201111561060357600080fd5b803590602001918460018302840111600160201b8311171561062457600080fd5b91908080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920191909152509295949360208101935035915050600160201b81111561067657600080fd5b82018360208201111561068857600080fd5b803590602001918460018302840111600160201b831117156106a957600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600092019190915250929550610f76945050505050565b6103ef6004803603604081101561070057600080fd5b81359190810190604081016020820135600160201b81111561072157600080fd5b82018360208201111561073357600080fd5b803590602001918460018302840111600160201b8311171561075457600080fd5b91908080601f016020809104026020016040519081016040528093929190818152602001838380828437600092019190915250929550610fd4945050505050565b606060048054806020026020016040519081016040528092919081815260200182805480156107ed57602002820191906000526020600020905b81546001600160a01b031681526001909101906020018083116107cf575b5050505050905090565b60028054604080516020601f60001961010060018716150201909416859004938401819004810282018101909252828152606093909290918301828280156107ed5780601f10610855576101008083540402835291602001916107ed565b820191906000526020600020905b81548152906001019060200180831161086357509395945050505050565b60008061088c610ffa565b9050610899818585610ffe565b5060019392505050565b60015490565b60006001600160a01b0383166108f05760405162461bcd60e51b8152600401808060200182810382526024815260200180611d586024913960400191505060405180910390fd5b6001600160a01b0384166109355760405162461bcd60e51b8152600401808060200182810382526026815260200180611dd16026913960400191505060405180910390fd5b600061093f610ffa565b905061096d8186868660405180602001604052806000815250604051806020016040528060008152506110ea565b610999818686866040518060200160405280600081525060405180602001604052806000815250611317565b6109ed85826109e886604051806060016040528060298152602001611da8602991396001600160a01b03808c166000908152600860209081526040808320938b16835292905220549190611530565b610ffe565b610a1b81868686604051806020016040528060008152506040518060200160405280600081525060006115c7565b506001949350505050565b601290565b600190565b610a41610a3b610ffa565b86610d5d565b610a7c5760405162461bcd60e51b815260040180806020018281038252602c815260200180611d7c602c913960400191505060405180910390fd5b610a8b8585858585600161184c565b5050505050565b6001600160a01b031660009081526020819052604090205490565b806001600160a01b0316610abf610ffa565b6001600160a01b03161415610b055760405162461bcd60e51b8152600401808060200182810382526024815260200180611cc66024913960400191505060405180910390fd5b6001600160a01b03811660009081526005602052604090205460ff1615610b685760076000610b32610ffa565b6001600160a01b03908116825260208083019390935260409182016000908120918516815292529020805460ff19169055610baf565b600160066000610b76610ffa565b6001600160a01b03908116825260208083019390935260409182016000908120918616815292529020805460ff19169115159190911790555b610bb7610ffa565b6001600160a01b0316816001600160a01b03167ff4caeb2d6ca8932a215a353d0703c326ec2d81fc68170f320eb2ab49e9df61f960405160405180910390a350565b60038054604080516020601f60026000196101006001881615020190951694909404938401819004810282018101909252828152606093909290918301828280156107ed5780601f10610855576101008083540402835291602001916107ed565b610c7f610c65610ffa565b84848460405180602001604052806000815250600161184c565b505050565b60006001600160a01b038316610ccb5760405162461bcd60e51b8152600401808060200182810382526024815260200180611d586024913960400191505060405180910390fd5b6000610cd5610ffa565b9050610d038182868660405180602001604052806000815250604051806020016040528060008152506110ea565b610d2f818286866040518060200160405280600081525060405180602001604052806000815250611317565b61089981828686604051806020016040528060008152506040518060200160405280600081525060006115c7565b6000816001600160a01b0316836001600160a01b03161480610dc857506001600160a01b03831660009081526005602052604090205460ff168015610dc857506001600160a01b0380831660009081526007602090815260408083209387168352929052205460ff16155b80610df857506001600160a01b0380831660009081526006602090815260408083209387168352929052205460ff165b9392505050565b6001600160a01b03918216600090815260086020908152604080832093909416825291909152205490565b610e32610ffa565b6001600160a01b0316816001600160a01b03161415610e825760405162461bcd60e51b8152600401808060200182810382526021815260200180611cea6021913960400191505060405180910390fd5b6001600160a01b03811660009081526005602052604090205460ff1615610eee57600160076000610eb1610ffa565b6001600160a01b03908116825260208083019390935260409182016000908120918616815292529020805460ff1916911515919091179055610f2c565b60066000610efa610ffa565b6001600160a01b03908116825260208083019390935260409182016000908120918516815292529020805460ff191690555b610f34610ffa565b6001600160a01b0316816001600160a01b03167f50546e66e5f44d728365dc3908c63bc5cfeeab470722c1677e3073a6ac294aa160405160405180910390a350565b610f87610f81610ffa565b85610d5d565b610fc25760405162461bcd60e51b815260040180806020018281038252602c815260200180611d7c602c913960400191505060405180910390fd5b610fce84848484611923565b50505050565b610ff6610fdf610ffa565b838360405180602001604052806000815250611923565b5050565b3390565b6001600160a01b0383166110435760405162461bcd60e51b8152600401808060200182810382526025815260200180611c366025913960400191505060405180910390fd5b6001600160a01b0382166110885760405162461bcd60e51b8152600401808060200182810382526023815260200180611e1a6023913960400191505060405180910390fd5b6001600160a01b03808416600081815260086020908152604080832094871680845294825291829020859055815185815291517f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b9259281900390910190a3505050565b6040805163555ddc6560e11b81526001600160a01b03871660048201527f29ddb589b1fb5fc7cf394961c1adf5f8c6454761adf795e67fe149f658abe89560248201529051600091731820a4b7618bde71dce8cdc73aab6c95905fad249163aabbb8ca91604480820192602092909190829003018186803b15801561116e57600080fd5b505afa158015611182573d6000803e3d6000fd5b505050506040513d602081101561119857600080fd5b505190506001600160a01b0381161561130e57806001600160a01b03166375ab97828888888888886040518763ffffffff1660e01b815260040180876001600160a01b03168152602001866001600160a01b03168152602001856001600160a01b031681526020018481526020018060200180602001838103835285818151815260200191508051906020019080838360005b8381101561124357818101518382015260200161122b565b50505050905090810190601f1680156112705780820380516001836020036101000a031916815260200191505b50838103825284518152845160209182019186019080838360005b838110156112a357818101518382015260200161128b565b50505050905090810190601f1680156112d05780820380516001836020036101000a031916815260200191505b5098505050505050505050600060405180830381600087803b1580156112f557600080fd5b505af1158015611309573d6000803e3d6000fd5b505050505b50505050505050565b61132386868686610fce565b61136083604051806060016040528060278152602001611c7d602791396001600160a01b0388166000908152602081905260409020549190611530565b6001600160a01b03808716600090815260208190526040808220939093559086168152205461138f9084611b5d565b600080866001600160a01b03166001600160a01b0316815260200190815260200160002081905550836001600160a01b0316856001600160a01b0316876001600160a01b03167f06b541ddaa720db2b10a4d0cdac39b8d360425fc073085fac19bc82614677987868686604051808481526020018060200180602001838103835285818151815260200191508051906020019080838360005b83811015611440578181015183820152602001611428565b50505050905090810190601f16801561146d5780820380516001836020036101000a031916815260200191505b50838103825284518152845160209182019186019080838360005b838110156114a0578181015183820152602001611488565b50505050905090810190601f1680156114cd5780820380516001836020036101000a031916815260200191505b509550505050505060405180910390a4836001600160a01b0316856001600160a01b03167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef856040518082815260200191505060405180910390a3505050505050565b600081848411156115bf5760405162461bcd60e51b81526004018080602001828103825283818151815260200191508051906020019080838360005b8381101561158457818101518382015260200161156c565b50505050905090810190601f1680156115b15780820380516001836020036101000a031916815260200191505b509250505060405180910390fd5b505050900390565b6040805163555ddc6560e11b81526001600160a01b03871660048201527fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b60248201529051600091731820a4b7618bde71dce8cdc73aab6c95905fad249163aabbb8ca91604480820192602092909190829003018186803b15801561164b57600080fd5b505afa15801561165f573d6000803e3d6000fd5b505050506040513d602081101561167557600080fd5b505190506001600160a01b038116156117ee57806001600160a01b03166223de298989898989896040518763ffffffff1660e01b815260040180876001600160a01b03168152602001866001600160a01b03168152602001856001600160a01b031681526020018481526020018060200180602001838103835285818151815260200191508051906020019080838360005b8381101561171f578181015183820152602001611707565b50505050905090810190601f16801561174c5780820380516001836020036101000a031916815260200191505b50838103825284518152845160209182019186019080838360005b8381101561177f578181015183820152602001611767565b50505050905090810190601f1680156117ac5780820380516001836020036101000a031916815260200191505b5098505050505050505050600060405180830381600087803b1580156117d157600080fd5b505af11580156117e5573d6000803e3d6000fd5b50505050611842565b811561184257611806866001600160a01b0316611bb7565b156118425760405162461bcd60e51b815260040180806020018281038252604d815260200180611d0b604d913960600191505060405180910390fd5b5050505050505050565b6001600160a01b0386166118915760405162461bcd60e51b8152600401808060200182810382526022815260200180611c5b6022913960400191505060405180910390fd5b6001600160a01b0385166118ec576040805162461bcd60e51b815260206004820181905260248201527f4552433737373a2073656e6420746f20746865207a65726f2061646472657373604482015290519081900360640190fd5b60006118f6610ffa565b90506119068188888888886110ea565b611914818888888888611317565b61130e818888888888886115c7565b6001600160a01b0384166119685760405162461bcd60e51b8152600401808060200182810382526022815260200180611ca46022913960400191505060405180910390fd5b6000611972610ffa565b9050611983818660008787876110ea565b6119908186600087610fce565b6119cd84604051806060016040528060238152602001611df7602391396001600160a01b0388166000908152602081905260409020549190611530565b6001600160a01b0386166000908152602081905260409020556001546119f39085611bf3565b600181905550846001600160a01b0316816001600160a01b03167fa78a9be3a7b862d26933ad85fb11d80ef66b8f972d7cbba06621d583943a4098868686604051808481526020018060200180602001838103835285818151815260200191508051906020019080838360005b83811015611a78578181015183820152602001611a60565b50505050905090810190601f168015611aa55780820380516001836020036101000a031916815260200191505b50838103825284518152845160209182019186019080838360005b83811015611ad8578181015183820152602001611ac0565b50505050905090810190601f168015611b055780820380516001836020036101000a031916815260200191505b509550505050505060405180910390a36040805185815290516000916001600160a01b038816917fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef9181900360200190a35050505050565b600082820183811015610df8576040805162461bcd60e51b815260206004820152601b60248201527f536166654d6174683a206164646974696f6e206f766572666c6f770000000000604482015290519081900360640190fd5b6000813f7fc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470818114801590611beb57508115155b949350505050565b6000610df883836040518060400160405280601e81526020017f536166654d6174683a207375627472616374696f6e206f766572666c6f77000081525061153056fe4552433737373a20617070726f76652066726f6d20746865207a65726f20616464726573734552433737373a2073656e642066726f6d20746865207a65726f20616464726573734552433737373a207472616e7366657220616d6f756e7420657863656564732062616c616e63654552433737373a206275726e2066726f6d20746865207a65726f20616464726573734552433737373a20617574686f72697a696e672073656c66206173206f70657261746f724552433737373a207265766f6b696e672073656c66206173206f70657261746f724552433737373a20746f6b656e20726563697069656e7420636f6e747261637420686173206e6f20696d706c656d656e74657220666f7220455243373737546f6b656e73526563697069656e744552433737373a207472616e7366657220746f20746865207a65726f20616464726573734552433737373a2063616c6c6572206973206e6f7420616e206f70657261746f7220666f7220686f6c6465724552433737373a207472616e7366657220616d6f756e74206578636565647320616c6c6f77616e63654552433737373a207472616e736665722066726f6d20746865207a65726f20616464726573734552433737373a206275726e20616d6f756e7420657863656564732062616c616e63654552433737373a20617070726f766520746f20746865207a65726f2061646472657373a2646970667358221220fbe5b48c0667cb952158e047531487ca5d6d18883c2d3d91fb2f5d5f3c2d536364736f6c634300060c0033" . parse () . expect ("invalid bytecode")
        });
    pub struct ERC777<M>(ethers::contract::Contract<M>);
    impl<M> Clone for ERC777<M> {
        fn clone(&self) -> Self {
            ERC777(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for ERC777<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for ERC777<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(ERC777))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> ERC777<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), ERC777_ABI.clone(), client).into()
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
                ERC777_ABI.clone(),
                ERC777_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        #[doc = "Calls the contract's `allowance` (0xdd62ed3e) function"]
        pub fn allowance(
            &self,
            holder: ethers::core::types::Address,
            spender: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([221, 98, 237, 62], (holder, spender))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `approve` (0x095ea7b3) function"]
        pub fn approve(
            &self,
            spender: ethers::core::types::Address,
            value: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([9, 94, 167, 179], (spender, value))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `authorizeOperator` (0x959b8c3f) function"]
        pub fn authorize_operator(
            &self,
            operator: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([149, 155, 140, 63], operator)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `balanceOf` (0x70a08231) function"]
        pub fn balance_of(
            &self,
            token_holder: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([112, 160, 130, 49], token_holder)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `burn` (0xfe9d9303) function"]
        pub fn burn(
            &self,
            amount: ethers::core::types::U256,
            data: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([254, 157, 147, 3], (amount, data))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `decimals` (0x313ce567) function"]
        pub fn decimals(&self) -> ethers::contract::builders::ContractCall<M, u8> {
            self.0
                .method_hash([49, 60, 229, 103], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `defaultOperators` (0x06e48538) function"]
        pub fn default_operators(
            &self,
        ) -> ethers::contract::builders::ContractCall<
            M,
            ::std::vec::Vec<ethers::core::types::Address>,
        > {
            self.0
                .method_hash([6, 228, 133, 56], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `granularity` (0x556f0dc7) function"]
        pub fn granularity(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([85, 111, 13, 199], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `isOperatorFor` (0xd95b6371) function"]
        pub fn is_operator_for(
            &self,
            operator: ethers::core::types::Address,
            token_holder: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([217, 91, 99, 113], (operator, token_holder))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `name` (0x06fdde03) function"]
        pub fn name(&self) -> ethers::contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([6, 253, 222, 3], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `operatorBurn` (0xfc673c4f) function"]
        pub fn operator_burn(
            &self,
            account: ethers::core::types::Address,
            amount: ethers::core::types::U256,
            data: ethers::core::types::Bytes,
            operator_data: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([252, 103, 60, 79], (account, amount, data, operator_data))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `operatorSend` (0x62ad1b83) function"]
        pub fn operator_send(
            &self,
            sender: ethers::core::types::Address,
            recipient: ethers::core::types::Address,
            amount: ethers::core::types::U256,
            data: ethers::core::types::Bytes,
            operator_data: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [98, 173, 27, 131],
                    (sender, recipient, amount, data, operator_data),
                )
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `revokeOperator` (0xfad8b32a) function"]
        pub fn revoke_operator(
            &self,
            operator: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([250, 216, 179, 42], operator)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `send` (0x9bd9bbc6) function"]
        pub fn send(
            &self,
            recipient: ethers::core::types::Address,
            amount: ethers::core::types::U256,
            data: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([155, 217, 187, 198], (recipient, amount, data))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `symbol` (0x95d89b41) function"]
        pub fn symbol(&self) -> ethers::contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([149, 216, 155, 65], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `totalSupply` (0x18160ddd) function"]
        pub fn total_supply(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([24, 22, 13, 221], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `transfer` (0xa9059cbb) function"]
        pub fn transfer(
            &self,
            recipient: ethers::core::types::Address,
            amount: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([169, 5, 156, 187], (recipient, amount))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `transferFrom` (0x23b872dd) function"]
        pub fn transfer_from(
            &self,
            holder: ethers::core::types::Address,
            recipient: ethers::core::types::Address,
            amount: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([35, 184, 114, 221], (holder, recipient, amount))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Gets the contract's `Approval` event"]
        pub fn approval_filter(&self) -> ethers::contract::builders::Event<M, ApprovalFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `AuthorizedOperator` event"]
        pub fn authorized_operator_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, AuthorizedOperatorFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Burned` event"]
        pub fn burned_filter(&self) -> ethers::contract::builders::Event<M, BurnedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Minted` event"]
        pub fn minted_filter(&self) -> ethers::contract::builders::Event<M, MintedFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `RevokedOperator` event"]
        pub fn revoked_operator_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, RevokedOperatorFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Sent` event"]
        pub fn sent_filter(&self) -> ethers::contract::builders::Event<M, SentFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Transfer` event"]
        pub fn transfer_filter(&self) -> ethers::contract::builders::Event<M, TransferFilter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(&self) -> ethers::contract::builders::Event<M, ERC777Events> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for ERC777<M> {
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
    #[ethevent(name = "Approval", abi = "Approval(address,address,uint256)")]
    pub struct ApprovalFilter {
        #[ethevent(indexed)]
        pub owner: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub spender: ethers::core::types::Address,
        pub value: ethers::core::types::U256,
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
        name = "AuthorizedOperator",
        abi = "AuthorizedOperator(address,address)"
    )]
    pub struct AuthorizedOperatorFilter {
        #[ethevent(indexed)]
        pub operator: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub token_holder: ethers::core::types::Address,
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
    #[ethevent(name = "Burned", abi = "Burned(address,address,uint256,bytes,bytes)")]
    pub struct BurnedFilter {
        #[ethevent(indexed)]
        pub operator: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub from: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
        pub operator_data: ethers::core::types::Bytes,
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
    #[ethevent(name = "Minted", abi = "Minted(address,address,uint256,bytes,bytes)")]
    pub struct MintedFilter {
        #[ethevent(indexed)]
        pub operator: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub to: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
        pub operator_data: ethers::core::types::Bytes,
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
    #[ethevent(name = "RevokedOperator", abi = "RevokedOperator(address,address)")]
    pub struct RevokedOperatorFilter {
        #[ethevent(indexed)]
        pub operator: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub token_holder: ethers::core::types::Address,
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
        name = "Sent",
        abi = "Sent(address,address,address,uint256,bytes,bytes)"
    )]
    pub struct SentFilter {
        #[ethevent(indexed)]
        pub operator: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub from: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub to: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
        pub operator_data: ethers::core::types::Bytes,
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
    #[ethevent(name = "Transfer", abi = "Transfer(address,address,uint256)")]
    pub struct TransferFilter {
        #[ethevent(indexed)]
        pub from: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub to: ethers::core::types::Address,
        pub value: ethers::core::types::U256,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum ERC777Events {
        ApprovalFilter(ApprovalFilter),
        AuthorizedOperatorFilter(AuthorizedOperatorFilter),
        BurnedFilter(BurnedFilter),
        MintedFilter(MintedFilter),
        RevokedOperatorFilter(RevokedOperatorFilter),
        SentFilter(SentFilter),
        TransferFilter(TransferFilter),
    }
    impl ethers::contract::EthLogDecode for ERC777Events {
        fn decode_log(
            log: &ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = ApprovalFilter::decode_log(log) {
                return Ok(ERC777Events::ApprovalFilter(decoded));
            }
            if let Ok(decoded) = AuthorizedOperatorFilter::decode_log(log) {
                return Ok(ERC777Events::AuthorizedOperatorFilter(decoded));
            }
            if let Ok(decoded) = BurnedFilter::decode_log(log) {
                return Ok(ERC777Events::BurnedFilter(decoded));
            }
            if let Ok(decoded) = MintedFilter::decode_log(log) {
                return Ok(ERC777Events::MintedFilter(decoded));
            }
            if let Ok(decoded) = RevokedOperatorFilter::decode_log(log) {
                return Ok(ERC777Events::RevokedOperatorFilter(decoded));
            }
            if let Ok(decoded) = SentFilter::decode_log(log) {
                return Ok(ERC777Events::SentFilter(decoded));
            }
            if let Ok(decoded) = TransferFilter::decode_log(log) {
                return Ok(ERC777Events::TransferFilter(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for ERC777Events {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                ERC777Events::ApprovalFilter(element) => element.fmt(f),
                ERC777Events::AuthorizedOperatorFilter(element) => element.fmt(f),
                ERC777Events::BurnedFilter(element) => element.fmt(f),
                ERC777Events::MintedFilter(element) => element.fmt(f),
                ERC777Events::RevokedOperatorFilter(element) => element.fmt(f),
                ERC777Events::SentFilter(element) => element.fmt(f),
                ERC777Events::TransferFilter(element) => element.fmt(f),
            }
        }
    }
    #[doc = "Container type for all input parameters for the `allowance` function with signature `allowance(address,address)` and selector `[221, 98, 237, 62]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "allowance", abi = "allowance(address,address)")]
    pub struct AllowanceCall {
        pub holder: ethers::core::types::Address,
        pub spender: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `approve` function with signature `approve(address,uint256)` and selector `[9, 94, 167, 179]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "approve", abi = "approve(address,uint256)")]
    pub struct ApproveCall {
        pub spender: ethers::core::types::Address,
        pub value: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `authorizeOperator` function with signature `authorizeOperator(address)` and selector `[149, 155, 140, 63]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "authorizeOperator", abi = "authorizeOperator(address)")]
    pub struct AuthorizeOperatorCall {
        pub operator: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `balanceOf` function with signature `balanceOf(address)` and selector `[112, 160, 130, 49]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "balanceOf", abi = "balanceOf(address)")]
    pub struct BalanceOfCall {
        pub token_holder: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `burn` function with signature `burn(uint256,bytes)` and selector `[254, 157, 147, 3]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "burn", abi = "burn(uint256,bytes)")]
    pub struct BurnCall {
        pub amount: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
    }
    #[doc = "Container type for all input parameters for the `decimals` function with signature `decimals()` and selector `[49, 60, 229, 103]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "decimals", abi = "decimals()")]
    pub struct DecimalsCall;
    #[doc = "Container type for all input parameters for the `defaultOperators` function with signature `defaultOperators()` and selector `[6, 228, 133, 56]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "defaultOperators", abi = "defaultOperators()")]
    pub struct DefaultOperatorsCall;
    #[doc = "Container type for all input parameters for the `granularity` function with signature `granularity()` and selector `[85, 111, 13, 199]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "granularity", abi = "granularity()")]
    pub struct GranularityCall;
    #[doc = "Container type for all input parameters for the `isOperatorFor` function with signature `isOperatorFor(address,address)` and selector `[217, 91, 99, 113]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "isOperatorFor", abi = "isOperatorFor(address,address)")]
    pub struct IsOperatorForCall {
        pub operator: ethers::core::types::Address,
        pub token_holder: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `name` function with signature `name()` and selector `[6, 253, 222, 3]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "name", abi = "name()")]
    pub struct NameCall;
    #[doc = "Container type for all input parameters for the `operatorBurn` function with signature `operatorBurn(address,uint256,bytes,bytes)` and selector `[252, 103, 60, 79]`"]
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
        name = "operatorBurn",
        abi = "operatorBurn(address,uint256,bytes,bytes)"
    )]
    pub struct OperatorBurnCall {
        pub account: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
        pub operator_data: ethers::core::types::Bytes,
    }
    #[doc = "Container type for all input parameters for the `operatorSend` function with signature `operatorSend(address,address,uint256,bytes,bytes)` and selector `[98, 173, 27, 131]`"]
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
        name = "operatorSend",
        abi = "operatorSend(address,address,uint256,bytes,bytes)"
    )]
    pub struct OperatorSendCall {
        pub sender: ethers::core::types::Address,
        pub recipient: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
        pub operator_data: ethers::core::types::Bytes,
    }
    #[doc = "Container type for all input parameters for the `revokeOperator` function with signature `revokeOperator(address)` and selector `[250, 216, 179, 42]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "revokeOperator", abi = "revokeOperator(address)")]
    pub struct RevokeOperatorCall {
        pub operator: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `send` function with signature `send(address,uint256,bytes)` and selector `[155, 217, 187, 198]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "send", abi = "send(address,uint256,bytes)")]
    pub struct SendCall {
        pub recipient: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
    }
    #[doc = "Container type for all input parameters for the `symbol` function with signature `symbol()` and selector `[149, 216, 155, 65]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "symbol", abi = "symbol()")]
    pub struct SymbolCall;
    #[doc = "Container type for all input parameters for the `totalSupply` function with signature `totalSupply()` and selector `[24, 22, 13, 221]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "totalSupply", abi = "totalSupply()")]
    pub struct TotalSupplyCall;
    #[doc = "Container type for all input parameters for the `transfer` function with signature `transfer(address,uint256)` and selector `[169, 5, 156, 187]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "transfer", abi = "transfer(address,uint256)")]
    pub struct TransferCall {
        pub recipient: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `transferFrom` function with signature `transferFrom(address,address,uint256)` and selector `[35, 184, 114, 221]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "transferFrom", abi = "transferFrom(address,address,uint256)")]
    pub struct TransferFromCall {
        pub holder: ethers::core::types::Address,
        pub recipient: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum ERC777Calls {
        Allowance(AllowanceCall),
        Approve(ApproveCall),
        AuthorizeOperator(AuthorizeOperatorCall),
        BalanceOf(BalanceOfCall),
        Burn(BurnCall),
        Decimals(DecimalsCall),
        DefaultOperators(DefaultOperatorsCall),
        Granularity(GranularityCall),
        IsOperatorFor(IsOperatorForCall),
        Name(NameCall),
        OperatorBurn(OperatorBurnCall),
        OperatorSend(OperatorSendCall),
        RevokeOperator(RevokeOperatorCall),
        Send(SendCall),
        Symbol(SymbolCall),
        TotalSupply(TotalSupplyCall),
        Transfer(TransferCall),
        TransferFrom(TransferFromCall),
    }
    impl ethers::core::abi::AbiDecode for ERC777Calls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <AllowanceCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777Calls::Allowance(decoded));
            }
            if let Ok(decoded) =
                <ApproveCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777Calls::Approve(decoded));
            }
            if let Ok(decoded) =
                <AuthorizeOperatorCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777Calls::AuthorizeOperator(decoded));
            }
            if let Ok(decoded) =
                <BalanceOfCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777Calls::BalanceOf(decoded));
            }
            if let Ok(decoded) = <BurnCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(ERC777Calls::Burn(decoded));
            }
            if let Ok(decoded) =
                <DecimalsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777Calls::Decimals(decoded));
            }
            if let Ok(decoded) =
                <DefaultOperatorsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777Calls::DefaultOperators(decoded));
            }
            if let Ok(decoded) =
                <GranularityCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777Calls::Granularity(decoded));
            }
            if let Ok(decoded) =
                <IsOperatorForCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777Calls::IsOperatorFor(decoded));
            }
            if let Ok(decoded) = <NameCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(ERC777Calls::Name(decoded));
            }
            if let Ok(decoded) =
                <OperatorBurnCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777Calls::OperatorBurn(decoded));
            }
            if let Ok(decoded) =
                <OperatorSendCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777Calls::OperatorSend(decoded));
            }
            if let Ok(decoded) =
                <RevokeOperatorCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777Calls::RevokeOperator(decoded));
            }
            if let Ok(decoded) = <SendCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(ERC777Calls::Send(decoded));
            }
            if let Ok(decoded) = <SymbolCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777Calls::Symbol(decoded));
            }
            if let Ok(decoded) =
                <TotalSupplyCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777Calls::TotalSupply(decoded));
            }
            if let Ok(decoded) =
                <TransferCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777Calls::Transfer(decoded));
            }
            if let Ok(decoded) =
                <TransferFromCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777Calls::TransferFrom(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for ERC777Calls {
        fn encode(self) -> Vec<u8> {
            match self {
                ERC777Calls::Allowance(element) => element.encode(),
                ERC777Calls::Approve(element) => element.encode(),
                ERC777Calls::AuthorizeOperator(element) => element.encode(),
                ERC777Calls::BalanceOf(element) => element.encode(),
                ERC777Calls::Burn(element) => element.encode(),
                ERC777Calls::Decimals(element) => element.encode(),
                ERC777Calls::DefaultOperators(element) => element.encode(),
                ERC777Calls::Granularity(element) => element.encode(),
                ERC777Calls::IsOperatorFor(element) => element.encode(),
                ERC777Calls::Name(element) => element.encode(),
                ERC777Calls::OperatorBurn(element) => element.encode(),
                ERC777Calls::OperatorSend(element) => element.encode(),
                ERC777Calls::RevokeOperator(element) => element.encode(),
                ERC777Calls::Send(element) => element.encode(),
                ERC777Calls::Symbol(element) => element.encode(),
                ERC777Calls::TotalSupply(element) => element.encode(),
                ERC777Calls::Transfer(element) => element.encode(),
                ERC777Calls::TransferFrom(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for ERC777Calls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                ERC777Calls::Allowance(element) => element.fmt(f),
                ERC777Calls::Approve(element) => element.fmt(f),
                ERC777Calls::AuthorizeOperator(element) => element.fmt(f),
                ERC777Calls::BalanceOf(element) => element.fmt(f),
                ERC777Calls::Burn(element) => element.fmt(f),
                ERC777Calls::Decimals(element) => element.fmt(f),
                ERC777Calls::DefaultOperators(element) => element.fmt(f),
                ERC777Calls::Granularity(element) => element.fmt(f),
                ERC777Calls::IsOperatorFor(element) => element.fmt(f),
                ERC777Calls::Name(element) => element.fmt(f),
                ERC777Calls::OperatorBurn(element) => element.fmt(f),
                ERC777Calls::OperatorSend(element) => element.fmt(f),
                ERC777Calls::RevokeOperator(element) => element.fmt(f),
                ERC777Calls::Send(element) => element.fmt(f),
                ERC777Calls::Symbol(element) => element.fmt(f),
                ERC777Calls::TotalSupply(element) => element.fmt(f),
                ERC777Calls::Transfer(element) => element.fmt(f),
                ERC777Calls::TransferFrom(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<AllowanceCall> for ERC777Calls {
        fn from(var: AllowanceCall) -> Self {
            ERC777Calls::Allowance(var)
        }
    }
    impl ::std::convert::From<ApproveCall> for ERC777Calls {
        fn from(var: ApproveCall) -> Self {
            ERC777Calls::Approve(var)
        }
    }
    impl ::std::convert::From<AuthorizeOperatorCall> for ERC777Calls {
        fn from(var: AuthorizeOperatorCall) -> Self {
            ERC777Calls::AuthorizeOperator(var)
        }
    }
    impl ::std::convert::From<BalanceOfCall> for ERC777Calls {
        fn from(var: BalanceOfCall) -> Self {
            ERC777Calls::BalanceOf(var)
        }
    }
    impl ::std::convert::From<BurnCall> for ERC777Calls {
        fn from(var: BurnCall) -> Self {
            ERC777Calls::Burn(var)
        }
    }
    impl ::std::convert::From<DecimalsCall> for ERC777Calls {
        fn from(var: DecimalsCall) -> Self {
            ERC777Calls::Decimals(var)
        }
    }
    impl ::std::convert::From<DefaultOperatorsCall> for ERC777Calls {
        fn from(var: DefaultOperatorsCall) -> Self {
            ERC777Calls::DefaultOperators(var)
        }
    }
    impl ::std::convert::From<GranularityCall> for ERC777Calls {
        fn from(var: GranularityCall) -> Self {
            ERC777Calls::Granularity(var)
        }
    }
    impl ::std::convert::From<IsOperatorForCall> for ERC777Calls {
        fn from(var: IsOperatorForCall) -> Self {
            ERC777Calls::IsOperatorFor(var)
        }
    }
    impl ::std::convert::From<NameCall> for ERC777Calls {
        fn from(var: NameCall) -> Self {
            ERC777Calls::Name(var)
        }
    }
    impl ::std::convert::From<OperatorBurnCall> for ERC777Calls {
        fn from(var: OperatorBurnCall) -> Self {
            ERC777Calls::OperatorBurn(var)
        }
    }
    impl ::std::convert::From<OperatorSendCall> for ERC777Calls {
        fn from(var: OperatorSendCall) -> Self {
            ERC777Calls::OperatorSend(var)
        }
    }
    impl ::std::convert::From<RevokeOperatorCall> for ERC777Calls {
        fn from(var: RevokeOperatorCall) -> Self {
            ERC777Calls::RevokeOperator(var)
        }
    }
    impl ::std::convert::From<SendCall> for ERC777Calls {
        fn from(var: SendCall) -> Self {
            ERC777Calls::Send(var)
        }
    }
    impl ::std::convert::From<SymbolCall> for ERC777Calls {
        fn from(var: SymbolCall) -> Self {
            ERC777Calls::Symbol(var)
        }
    }
    impl ::std::convert::From<TotalSupplyCall> for ERC777Calls {
        fn from(var: TotalSupplyCall) -> Self {
            ERC777Calls::TotalSupply(var)
        }
    }
    impl ::std::convert::From<TransferCall> for ERC777Calls {
        fn from(var: TransferCall) -> Self {
            ERC777Calls::Transfer(var)
        }
    }
    impl ::std::convert::From<TransferFromCall> for ERC777Calls {
        fn from(var: TransferFromCall) -> Self {
            ERC777Calls::TransferFrom(var)
        }
    }
    #[doc = "Container type for all return fields from the `allowance` function with signature `allowance(address,address)` and selector `[221, 98, 237, 62]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct AllowanceReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `approve` function with signature `approve(address,uint256)` and selector `[9, 94, 167, 179]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ApproveReturn(pub bool);
    #[doc = "Container type for all return fields from the `balanceOf` function with signature `balanceOf(address)` and selector `[112, 160, 130, 49]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct BalanceOfReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `decimals` function with signature `decimals()` and selector `[49, 60, 229, 103]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct DecimalsReturn(pub u8);
    #[doc = "Container type for all return fields from the `defaultOperators` function with signature `defaultOperators()` and selector `[6, 228, 133, 56]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct DefaultOperatorsReturn(pub ::std::vec::Vec<ethers::core::types::Address>);
    #[doc = "Container type for all return fields from the `granularity` function with signature `granularity()` and selector `[85, 111, 13, 199]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GranularityReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `isOperatorFor` function with signature `isOperatorFor(address,address)` and selector `[217, 91, 99, 113]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct IsOperatorForReturn(pub bool);
    #[doc = "Container type for all return fields from the `name` function with signature `name()` and selector `[6, 253, 222, 3]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct NameReturn(pub String);
    #[doc = "Container type for all return fields from the `symbol` function with signature `symbol()` and selector `[149, 216, 155, 65]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct SymbolReturn(pub String);
    #[doc = "Container type for all return fields from the `totalSupply` function with signature `totalSupply()` and selector `[24, 22, 13, 221]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct TotalSupplyReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `transfer` function with signature `transfer(address,uint256)` and selector `[169, 5, 156, 187]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct TransferReturn(pub bool);
    #[doc = "Container type for all return fields from the `transferFrom` function with signature `transferFrom(address,address,uint256)` and selector `[35, 184, 114, 221]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct TransferFromReturn(pub bool);
}
