pub use erc777_mock::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod erc777_mock {
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
    #[doc = "ERC777Mock was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"initialHolder\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"initialBalance\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"name\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"symbol\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"defaultOperators\",\"type\":\"address[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"owner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"spender\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Approval\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"tokenHolder\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"AuthorizedOperator\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Burned\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Minted\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"tokenHolder\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"RevokedOperator\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Sent\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Transfer\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"holder\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"spender\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"allowance\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"spender\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"approve\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"holder\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"spender\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"approveInternal\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"authorizeOperator\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"tokenHolder\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"balanceOf\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"burn\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"pure\",\"type\":\"function\",\"name\":\"decimals\",\"outputs\":[{\"internalType\":\"uint8\",\"name\":\"\",\"type\":\"uint8\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"defaultOperators\",\"outputs\":[{\"internalType\":\"address[]\",\"name\":\"\",\"type\":\"address[]\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"granularity\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"tokenHolder\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"isOperatorFor\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"userData\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"mintInternal\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"name\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"operatorBurn\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"recipient\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"operatorSend\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"revokeOperator\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"recipient\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"send\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"symbol\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"totalSupply\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"recipient\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transfer\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"holder\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"recipient\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferFrom\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static ERC777MOCK_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static ERC777MOCK_BYTECODE: ethers::contract::Lazy<ethers::core::types::Bytes> =
        ethers::contract::Lazy::new(|| {
            "0x60806040523480156200001157600080fd5b50604051620023c5380380620023c58339810160408190526200003491620007b0565b82828282600290805190602001906200004f929190620005a9565b50815162000065906003906020850190620005a9565b5080516200007b90600490602084019062000638565b5060005b8151811015620000eb57600160056000848481518110620000a457620000a4620008d9565b6020908102919091018101516001600160a01b03168252810191909152604001600020805460ff191691151591909117905580620000e28162000905565b9150506200007f565b506040516329965a1d60e01b815230600482018190527fac7fbab5f54a3ca8194167523c6753bfeb96a445279294b6125b68cce217705460248301526044820152731820a4b7618bde71dce8cdc73aab6c95905fad24906329965a1d90606401600060405180830381600087803b1580156200016657600080fd5b505af11580156200017b573d6000803e3d6000fd5b50506040516329965a1d60e01b815230600482018190527faea199e31a596269b42cdafd93407f14436db6e4cad65417994c2eb37381e05a60248301526044820152731820a4b7618bde71dce8cdc73aab6c95905fad2492506329965a1d9150606401600060405180830381600087803b158015620001f957600080fd5b505af11580156200020e573d6000803e3d6000fd5b5050505050505062000247858560405180602001604052806000815250604051806020016040528060008152506200025260201b60201c565b505050505062000a62565b6200026284848484600162000268565b50505050565b6001600160a01b038516620002c45760405162461bcd60e51b815260206004820181905260248201527f4552433737373a206d696e7420746f20746865207a65726f206164647265737360448201526064015b60405180910390fd5b60003390508460016000828254620002dd919062000921565b90915550506001600160a01b038616600090815260208190526040812080548792906200030c90849062000921565b909155506200032490508160008888888888620003be565b856001600160a01b0316816001600160a01b03167f2fe5be0146f74c5bce36c0b80911af6c7d86ff27e89d5cfa61fc681327954e5d8787876040516200036d939291906200096a565b60405180910390a36040518581526001600160a01b038716906000907fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef9060200160405180910390a3505050505050565b60405163555ddc6560e11b81526001600160a01b03861660048201527fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b6024820152600090731820a4b7618bde71dce8cdc73aab6c95905fad249063aabbb8ca90604401602060405180830381865afa15801562000440573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190620004669190620009a3565b90506001600160a01b03811615620004e8576040516223de2960e01b81526001600160a01b038216906223de2990620004ae908b908b908b908b908b908b90600401620009c8565b600060405180830381600087803b158015620004c957600080fd5b505af1158015620004de573d6000803e3d6000fd5b5050505062000599565b811562000599576200050e866001600160a01b0316620005a360201b62000a211760201c565b15620005995760405162461bcd60e51b815260206004820152604d60248201527f4552433737373a20746f6b656e20726563697069656e7420636f6e747261637460448201527f20686173206e6f20696d706c656d656e74657220666f7220455243373737546f60648201526c1ad95b9cd49958da5c1a595b9d609a1b608482015260a401620002bb565b5050505050505050565b3b151590565b828054620005b79062000a26565b90600052602060002090601f016020900481019282620005db576000855562000626565b82601f10620005f657805160ff191683800117855562000626565b8280016001018555821562000626579182015b828111156200062657825182559160200191906001019062000609565b506200063492915062000690565b5090565b82805482825590600052602060002090810192821562000626579160200282015b828111156200062657825182546001600160a01b0319166001600160a01b0390911617825560209092019160019091019062000659565b5b8082111562000634576000815560010162000691565b80516001600160a01b0381168114620006bf57600080fd5b919050565b634e487b7160e01b600052604160045260246000fd5b604051601f8201601f191681016001600160401b0381118282101715620007055762000705620006c4565b604052919050565b60005b838110156200072a57818101518382015260200162000710565b83811115620002625750506000910152565b600082601f8301126200074e57600080fd5b81516001600160401b038111156200076a576200076a620006c4565b6200077f601f8201601f1916602001620006da565b8181528460208386010111156200079557600080fd5b620007a88260208301602087016200070d565b949350505050565b600080600080600060a08688031215620007c957600080fd5b620007d486620006a7565b60208781015160408901519297509550906001600160401b0380821115620007fb57600080fd5b620008098a838b016200073c565b955060608901519150808211156200082057600080fd5b6200082e8a838b016200073c565b945060808901519150808211156200084557600080fd5b818901915089601f8301126200085a57600080fd5b8151818111156200086f576200086f620006c4565b8060051b915062000882848301620006da565b818152918301840191848101908c8411156200089d57600080fd5b938501935b83851015620008c657620008b685620006a7565b82529385019390850190620008a2565b8096505050505050509295509295909350565b634e487b7160e01b600052603260045260246000fd5b634e487b7160e01b600052601160045260246000fd5b6000600182016200091a576200091a620008ef565b5060010190565b60008219821115620009375762000937620008ef565b500190565b60008151808452620009568160208601602086016200070d565b601f01601f19169290920160200192915050565b8381526060602082015260006200098560608301856200093c565b82810360408401526200099981856200093c565b9695505050505050565b600060208284031215620009b657600080fd5b620009c182620006a7565b9392505050565b6001600160a01b0387811682528681166020830152851660408201526060810184905260c06080820181905260009062000a05908301856200093c565b82810360a084015262000a1981856200093c565b9998505050505050505050565b600181811c9082168062000a3b57607f821691505b60208210810362000a5c57634e487b7160e01b600052602260045260246000fd5b50919050565b6119538062000a726000396000f3fe608060405234801561001057600080fd5b506004361061012c5760003560e01c8063959b8c3f116100ad578063d95b637111610071578063d95b637114610267578063dd62ed3e1461027a578063fad8b32a146102b3578063fc673c4f146102c6578063fe9d9303146102d957600080fd5b8063959b8c3f1461021357806395d89b41146102265780639bd9bbc61461022e578063a9059cbb14610241578063b1f0b5be1461025457600080fd5b8063313ce567116100f4578063313ce567146101ac578063556f0dc7146101bb57806356189cb4146101c257806362ad1b83146101d757806370a08231146101ea57600080fd5b806306e485381461013157806306fdde031461014f578063095ea7b31461016457806318160ddd1461018757806323b872dd14610199575b600080fd5b6101396102ec565b6040516101469190611384565b60405180910390f35b61015761034e565b604051610146919061141e565b610177610172366004611449565b6103d7565b6040519015158152602001610146565b6001545b604051908152602001610146565b6101776101a7366004611475565b6103ef565b60405160128152602001610146565b600161018b565b6101d56101d0366004611475565b6105b8565b005b6101d56101e5366004611559565b6105c8565b61018b6101f83660046115ec565b6001600160a01b031660009081526020819052604090205490565b6101d56102213660046115ec565b610604565b610157610721565b6101d561023c366004611609565b610730565b61017761024f366004611449565b61074e565b6101d5610262366004611662565b610801565b6101776102753660046116e2565b610813565b61018b6102883660046116e2565b6001600160a01b03918216600090815260086020908152604080832093909416825291909152205490565b6101d56102c13660046115ec565b6108b5565b6101d56102d4366004611662565b6109d0565b6101d56102e736600461171b565b610a02565b6060600480548060200260200160405190810160405280929190818152602001828054801561034457602002820191906000526020600020905b81546001600160a01b03168152600190910190602001808311610326575b5050505050905090565b60606002805461035d90611762565b80601f016020809104026020016040519081016040528092919081815260200182805461038990611762565b80156103445780601f106103ab57610100808354040283529160200191610344565b820191906000526020600020905b8154815290600101906020018083116103b957509395945050505050565b6000336103e5818585610a27565b5060019392505050565b60006001600160a01b0383166104205760405162461bcd60e51b81526004016104179061179c565b60405180910390fd5b6001600160a01b0384166104855760405162461bcd60e51b815260206004820152602660248201527f4552433737373a207472616e736665722066726f6d20746865207a65726f206160448201526564647265737360d01b6064820152608401610417565b60003390506104b6818686866040518060200160405280600081525060405180602001604052806000815250610b4e565b6104e2818686866040518060200160405280600081525060405180602001604052806000815250610c76565b6001600160a01b038086166000908152600860209081526040808320938516835292905220548381101561056a5760405162461bcd60e51b815260206004820152602960248201527f4552433737373a207472616e7366657220616d6f756e74206578636565647320604482015268616c6c6f77616e636560b81b6064820152608401610417565b61057e868361057987856117f6565b610a27565b6105ac8287878760405180602001604052806000815250604051806020016040528060008152506000610ddc565b50600195945050505050565b6105c3838383610a27565b505050565b6105d23386610813565b6105ee5760405162461bcd60e51b81526004016104179061180d565b6105fd85858585856001610fa1565b5050505050565b6001600160a01b03811633036106685760405162461bcd60e51b8152602060048201526024808201527f4552433737373a20617574686f72697a696e672073656c66206173206f70657260448201526330ba37b960e11b6064820152608401610417565b6001600160a01b03811660009081526005602052604090205460ff16156106b9573360009081526007602090815260408083206001600160a01b03851684529091529020805460ff191690556106e8565b3360009081526006602090815260408083206001600160a01b03851684529091529020805460ff191660011790555b60405133906001600160a01b038316907ff4caeb2d6ca8932a215a353d0703c326ec2d81fc68170f320eb2ab49e9df61f990600090a350565b60606003805461035d90611762565b6105c333848484604051806020016040528060008152506001610fa1565b60006001600160a01b0383166107765760405162461bcd60e51b81526004016104179061179c565b60003390506107a7818286866040518060200160405280600081525060405180602001604052806000815250610b4e565b6107d3818286866040518060200160405280600081525060405180602001604052806000815250610c76565b6103e58182868660405180602001604052806000815250604051806020016040528060008152506000610ddc565b61080d84848484611084565b50505050565b6000816001600160a01b0316836001600160a01b0316148061087e57506001600160a01b03831660009081526005602052604090205460ff16801561087e57506001600160a01b0380831660009081526007602090815260408083209387168352929052205460ff16155b806108ae57506001600160a01b0380831660009081526006602090815260408083209387168352929052205460ff165b9392505050565b336001600160a01b038216036109175760405162461bcd60e51b815260206004820152602160248201527f4552433737373a207265766f6b696e672073656c66206173206f70657261746f6044820152603960f91b6064820152608401610417565b6001600160a01b03811660009081526005602052604090205460ff161561096b573360009081526007602090815260408083206001600160a01b03851684529091529020805460ff19166001179055610997565b3360009081526006602090815260408083206001600160a01b03851684529091529020805460ff191690555b60405133906001600160a01b038316907f50546e66e5f44d728365dc3908c63bc5cfeeab470722c1677e3073a6ac294aa190600090a350565b6109da3385610813565b6109f65760405162461bcd60e51b81526004016104179061180d565b61080d84848484611092565b610a1d33838360405180602001604052806000815250611092565b5050565b3b151590565b6001600160a01b038316610a8b5760405162461bcd60e51b815260206004820152602560248201527f4552433737373a20617070726f76652066726f6d20746865207a65726f206164604482015264647265737360d81b6064820152608401610417565b6001600160a01b038216610aed5760405162461bcd60e51b815260206004820152602360248201527f4552433737373a20617070726f766520746f20746865207a65726f206164647260448201526265737360e81b6064820152608401610417565b6001600160a01b0383811660008181526008602090815260408083209487168084529482529182902085905590518481527f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925910160405180910390a3505050565b60405163555ddc6560e11b81526001600160a01b03861660048201527f29ddb589b1fb5fc7cf394961c1adf5f8c6454761adf795e67fe149f658abe8956024820152600090731820a4b7618bde71dce8cdc73aab6c95905fad249063aabbb8ca90604401602060405180830381865afa158015610bcf573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610bf39190611859565b90506001600160a01b03811615610c6d57604051633ad5cbc160e11b81526001600160a01b038216906375ab978290610c3a908a908a908a908a908a908a90600401611876565b600060405180830381600087803b158015610c5457600080fd5b505af1158015610c68573d6000803e3d6000fd5b505050505b50505050505050565b6001600160a01b03851660009081526020819052604090205483811015610cef5760405162461bcd60e51b815260206004820152602760248201527f4552433737373a207472616e7366657220616d6f756e7420657863656564732060448201526662616c616e636560c81b6064820152608401610417565b6001600160a01b03808716600090815260208190526040808220878503905591871681529081208054869290610d269084906118d0565b92505081905550846001600160a01b0316866001600160a01b0316886001600160a01b03167f06b541ddaa720db2b10a4d0cdac39b8d360425fc073085fac19bc82614677987878787604051610d7e939291906118e8565b60405180910390a4846001600160a01b0316866001600160a01b03167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef86604051610dcb91815260200190565b60405180910390a350505050505050565b60405163555ddc6560e11b81526001600160a01b03861660048201527fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b6024820152600090731820a4b7618bde71dce8cdc73aab6c95905fad249063aabbb8ca90604401602060405180830381865afa158015610e5d573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610e819190611859565b90506001600160a01b03811615610efd576040516223de2960e01b81526001600160a01b038216906223de2990610ec6908b908b908b908b908b908b90600401611876565b600060405180830381600087803b158015610ee057600080fd5b505af1158015610ef4573d6000803e3d6000fd5b50505050610f97565b8115610f97576001600160a01b0386163b15610f975760405162461bcd60e51b815260206004820152604d60248201527f4552433737373a20746f6b656e20726563697069656e7420636f6e747261637460448201527f20686173206e6f20696d706c656d656e74657220666f7220455243373737546f60648201526c1ad95b9cd49958da5c1a595b9d609a1b608482015260a401610417565b5050505050505050565b6001600160a01b0386166110025760405162461bcd60e51b815260206004820152602260248201527f4552433737373a2073656e642066726f6d20746865207a65726f206164647265604482015261737360f01b6064820152608401610417565b6001600160a01b0385166110585760405162461bcd60e51b815260206004820181905260248201527f4552433737373a2073656e6420746f20746865207a65726f20616464726573736044820152606401610417565b33611067818888888888610b4e565b611075818888888888610c76565b610c6d81888888888888610ddc565b61080d848484846001611247565b6001600160a01b0384166110f35760405162461bcd60e51b815260206004820152602260248201527f4552433737373a206275726e2066726f6d20746865207a65726f206164647265604482015261737360f01b6064820152608401610417565b3361110381866000878787610b4e565b6001600160a01b038516600090815260208190526040902054848110156111785760405162461bcd60e51b815260206004820152602360248201527f4552433737373a206275726e20616d6f756e7420657863656564732062616c616044820152626e636560e81b6064820152608401610417565b6001600160a01b03861660009081526020819052604081208683039055600180548792906111a79084906117f6565b92505081905550856001600160a01b0316826001600160a01b03167fa78a9be3a7b862d26933ad85fb11d80ef66b8f972d7cbba06621d583943a40988787876040516111f5939291906118e8565b60405180910390a36040518581526000906001600160a01b038816907fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef906020015b60405180910390a3505050505050565b6001600160a01b03851661129d5760405162461bcd60e51b815260206004820181905260248201527f4552433737373a206d696e7420746f20746865207a65726f20616464726573736044820152606401610417565b600033905084600160008282546112b491906118d0565b90915550506001600160a01b038616600090815260208190526040812080548792906112e19084906118d0565b909155506112f790508160008888888888610ddc565b856001600160a01b0316816001600160a01b03167f2fe5be0146f74c5bce36c0b80911af6c7d86ff27e89d5cfa61fc681327954e5d87878760405161133e939291906118e8565b60405180910390a36040518581526001600160a01b038716906000907fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef90602001611237565b6020808252825182820181905260009190848201906040850190845b818110156113c55783516001600160a01b0316835292840192918401916001016113a0565b50909695505050505050565b6000815180845260005b818110156113f7576020818501810151868301820152016113db565b81811115611409576000602083870101525b50601f01601f19169290920160200192915050565b6020815260006108ae60208301846113d1565b6001600160a01b038116811461144657600080fd5b50565b6000806040838503121561145c57600080fd5b823561146781611431565b946020939093013593505050565b60008060006060848603121561148a57600080fd5b833561149581611431565b925060208401356114a581611431565b929592945050506040919091013590565b634e487b7160e01b600052604160045260246000fd5b600082601f8301126114dd57600080fd5b813567ffffffffffffffff808211156114f8576114f86114b6565b604051601f8301601f19908116603f01168101908282118183101715611520576115206114b6565b8160405283815286602085880101111561153957600080fd5b836020870160208301376000602085830101528094505050505092915050565b600080600080600060a0868803121561157157600080fd5b853561157c81611431565b9450602086013561158c81611431565b935060408601359250606086013567ffffffffffffffff808211156115b057600080fd5b6115bc89838a016114cc565b935060808801359150808211156115d257600080fd5b506115df888289016114cc565b9150509295509295909350565b6000602082840312156115fe57600080fd5b81356108ae81611431565b60008060006060848603121561161e57600080fd5b833561162981611431565b925060208401359150604084013567ffffffffffffffff81111561164c57600080fd5b611658868287016114cc565b9150509250925092565b6000806000806080858703121561167857600080fd5b843561168381611431565b935060208501359250604085013567ffffffffffffffff808211156116a757600080fd5b6116b3888389016114cc565b935060608701359150808211156116c957600080fd5b506116d6878288016114cc565b91505092959194509250565b600080604083850312156116f557600080fd5b823561170081611431565b9150602083013561171081611431565b809150509250929050565b6000806040838503121561172e57600080fd5b82359150602083013567ffffffffffffffff81111561174c57600080fd5b611758858286016114cc565b9150509250929050565b600181811c9082168061177657607f821691505b60208210810361179657634e487b7160e01b600052602260045260246000fd5b50919050565b60208082526024908201527f4552433737373a207472616e7366657220746f20746865207a65726f206164646040820152637265737360e01b606082015260800190565b634e487b7160e01b600052601160045260246000fd5b600082821015611808576118086117e0565b500390565b6020808252602c908201527f4552433737373a2063616c6c6572206973206e6f7420616e206f70657261746f60408201526b39103337b9103437b63232b960a11b606082015260800190565b60006020828403121561186b57600080fd5b81516108ae81611431565b6001600160a01b0387811682528681166020830152851660408201526060810184905260c0608082018190526000906118b1908301856113d1565b82810360a08401526118c381856113d1565b9998505050505050505050565b600082198211156118e3576118e36117e0565b500190565b83815260606020820152600061190160608301856113d1565b828103604084015261191381856113d1565b969550505050505056fea2646970667358221220a3fdac4da913bfd56beec4aad5f442e006ce7992c3d28bda2b8ad38e5a9d087764736f6c634300080d0033" . parse () . expect ("invalid bytecode")
        });
    pub struct ERC777Mock<M>(ethers::contract::Contract<M>);
    impl<M> Clone for ERC777Mock<M> {
        fn clone(&self) -> Self {
            ERC777Mock(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for ERC777Mock<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for ERC777Mock<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(ERC777Mock))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> ERC777Mock<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), ERC777MOCK_ABI.clone(), client).into()
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
                ERC777MOCK_ABI.clone(),
                ERC777MOCK_BYTECODE.clone().into(),
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
        #[doc = "Calls the contract's `approveInternal` (0x56189cb4) function"]
        pub fn approve_internal(
            &self,
            holder: ethers::core::types::Address,
            spender: ethers::core::types::Address,
            value: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([86, 24, 156, 180], (holder, spender, value))
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
        #[doc = "Calls the contract's `mintInternal` (0xb1f0b5be) function"]
        pub fn mint_internal(
            &self,
            to: ethers::core::types::Address,
            amount: ethers::core::types::U256,
            user_data: ethers::core::types::Bytes,
            operator_data: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([177, 240, 181, 190], (to, amount, user_data, operator_data))
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
        pub fn events(&self) -> ethers::contract::builders::Event<M, ERC777MockEvents> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for ERC777Mock<M> {
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
    pub enum ERC777MockEvents {
        ApprovalFilter(ApprovalFilter),
        AuthorizedOperatorFilter(AuthorizedOperatorFilter),
        BurnedFilter(BurnedFilter),
        MintedFilter(MintedFilter),
        RevokedOperatorFilter(RevokedOperatorFilter),
        SentFilter(SentFilter),
        TransferFilter(TransferFilter),
    }
    impl ethers::contract::EthLogDecode for ERC777MockEvents {
        fn decode_log(
            log: &ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = ApprovalFilter::decode_log(log) {
                return Ok(ERC777MockEvents::ApprovalFilter(decoded));
            }
            if let Ok(decoded) = AuthorizedOperatorFilter::decode_log(log) {
                return Ok(ERC777MockEvents::AuthorizedOperatorFilter(decoded));
            }
            if let Ok(decoded) = BurnedFilter::decode_log(log) {
                return Ok(ERC777MockEvents::BurnedFilter(decoded));
            }
            if let Ok(decoded) = MintedFilter::decode_log(log) {
                return Ok(ERC777MockEvents::MintedFilter(decoded));
            }
            if let Ok(decoded) = RevokedOperatorFilter::decode_log(log) {
                return Ok(ERC777MockEvents::RevokedOperatorFilter(decoded));
            }
            if let Ok(decoded) = SentFilter::decode_log(log) {
                return Ok(ERC777MockEvents::SentFilter(decoded));
            }
            if let Ok(decoded) = TransferFilter::decode_log(log) {
                return Ok(ERC777MockEvents::TransferFilter(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for ERC777MockEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                ERC777MockEvents::ApprovalFilter(element) => element.fmt(f),
                ERC777MockEvents::AuthorizedOperatorFilter(element) => element.fmt(f),
                ERC777MockEvents::BurnedFilter(element) => element.fmt(f),
                ERC777MockEvents::MintedFilter(element) => element.fmt(f),
                ERC777MockEvents::RevokedOperatorFilter(element) => element.fmt(f),
                ERC777MockEvents::SentFilter(element) => element.fmt(f),
                ERC777MockEvents::TransferFilter(element) => element.fmt(f),
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
    #[doc = "Container type for all input parameters for the `approveInternal` function with signature `approveInternal(address,address,uint256)` and selector `[86, 24, 156, 180]`"]
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
        name = "approveInternal",
        abi = "approveInternal(address,address,uint256)"
    )]
    pub struct ApproveInternalCall {
        pub holder: ethers::core::types::Address,
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
    #[doc = "Container type for all input parameters for the `mintInternal` function with signature `mintInternal(address,uint256,bytes,bytes)` and selector `[177, 240, 181, 190]`"]
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
        name = "mintInternal",
        abi = "mintInternal(address,uint256,bytes,bytes)"
    )]
    pub struct MintInternalCall {
        pub to: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
        pub user_data: ethers::core::types::Bytes,
        pub operator_data: ethers::core::types::Bytes,
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
    pub enum ERC777MockCalls {
        Allowance(AllowanceCall),
        Approve(ApproveCall),
        ApproveInternal(ApproveInternalCall),
        AuthorizeOperator(AuthorizeOperatorCall),
        BalanceOf(BalanceOfCall),
        Burn(BurnCall),
        Decimals(DecimalsCall),
        DefaultOperators(DefaultOperatorsCall),
        Granularity(GranularityCall),
        IsOperatorFor(IsOperatorForCall),
        MintInternal(MintInternalCall),
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
    impl ethers::core::abi::AbiDecode for ERC777MockCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <AllowanceCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777MockCalls::Allowance(decoded));
            }
            if let Ok(decoded) =
                <ApproveCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777MockCalls::Approve(decoded));
            }
            if let Ok(decoded) =
                <ApproveInternalCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777MockCalls::ApproveInternal(decoded));
            }
            if let Ok(decoded) =
                <AuthorizeOperatorCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777MockCalls::AuthorizeOperator(decoded));
            }
            if let Ok(decoded) =
                <BalanceOfCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777MockCalls::BalanceOf(decoded));
            }
            if let Ok(decoded) = <BurnCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(ERC777MockCalls::Burn(decoded));
            }
            if let Ok(decoded) =
                <DecimalsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777MockCalls::Decimals(decoded));
            }
            if let Ok(decoded) =
                <DefaultOperatorsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777MockCalls::DefaultOperators(decoded));
            }
            if let Ok(decoded) =
                <GranularityCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777MockCalls::Granularity(decoded));
            }
            if let Ok(decoded) =
                <IsOperatorForCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777MockCalls::IsOperatorFor(decoded));
            }
            if let Ok(decoded) =
                <MintInternalCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777MockCalls::MintInternal(decoded));
            }
            if let Ok(decoded) = <NameCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(ERC777MockCalls::Name(decoded));
            }
            if let Ok(decoded) =
                <OperatorBurnCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777MockCalls::OperatorBurn(decoded));
            }
            if let Ok(decoded) =
                <OperatorSendCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777MockCalls::OperatorSend(decoded));
            }
            if let Ok(decoded) =
                <RevokeOperatorCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777MockCalls::RevokeOperator(decoded));
            }
            if let Ok(decoded) = <SendCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(ERC777MockCalls::Send(decoded));
            }
            if let Ok(decoded) = <SymbolCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777MockCalls::Symbol(decoded));
            }
            if let Ok(decoded) =
                <TotalSupplyCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777MockCalls::TotalSupply(decoded));
            }
            if let Ok(decoded) =
                <TransferCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777MockCalls::Transfer(decoded));
            }
            if let Ok(decoded) =
                <TransferFromCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC777MockCalls::TransferFrom(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for ERC777MockCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                ERC777MockCalls::Allowance(element) => element.encode(),
                ERC777MockCalls::Approve(element) => element.encode(),
                ERC777MockCalls::ApproveInternal(element) => element.encode(),
                ERC777MockCalls::AuthorizeOperator(element) => element.encode(),
                ERC777MockCalls::BalanceOf(element) => element.encode(),
                ERC777MockCalls::Burn(element) => element.encode(),
                ERC777MockCalls::Decimals(element) => element.encode(),
                ERC777MockCalls::DefaultOperators(element) => element.encode(),
                ERC777MockCalls::Granularity(element) => element.encode(),
                ERC777MockCalls::IsOperatorFor(element) => element.encode(),
                ERC777MockCalls::MintInternal(element) => element.encode(),
                ERC777MockCalls::Name(element) => element.encode(),
                ERC777MockCalls::OperatorBurn(element) => element.encode(),
                ERC777MockCalls::OperatorSend(element) => element.encode(),
                ERC777MockCalls::RevokeOperator(element) => element.encode(),
                ERC777MockCalls::Send(element) => element.encode(),
                ERC777MockCalls::Symbol(element) => element.encode(),
                ERC777MockCalls::TotalSupply(element) => element.encode(),
                ERC777MockCalls::Transfer(element) => element.encode(),
                ERC777MockCalls::TransferFrom(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for ERC777MockCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                ERC777MockCalls::Allowance(element) => element.fmt(f),
                ERC777MockCalls::Approve(element) => element.fmt(f),
                ERC777MockCalls::ApproveInternal(element) => element.fmt(f),
                ERC777MockCalls::AuthorizeOperator(element) => element.fmt(f),
                ERC777MockCalls::BalanceOf(element) => element.fmt(f),
                ERC777MockCalls::Burn(element) => element.fmt(f),
                ERC777MockCalls::Decimals(element) => element.fmt(f),
                ERC777MockCalls::DefaultOperators(element) => element.fmt(f),
                ERC777MockCalls::Granularity(element) => element.fmt(f),
                ERC777MockCalls::IsOperatorFor(element) => element.fmt(f),
                ERC777MockCalls::MintInternal(element) => element.fmt(f),
                ERC777MockCalls::Name(element) => element.fmt(f),
                ERC777MockCalls::OperatorBurn(element) => element.fmt(f),
                ERC777MockCalls::OperatorSend(element) => element.fmt(f),
                ERC777MockCalls::RevokeOperator(element) => element.fmt(f),
                ERC777MockCalls::Send(element) => element.fmt(f),
                ERC777MockCalls::Symbol(element) => element.fmt(f),
                ERC777MockCalls::TotalSupply(element) => element.fmt(f),
                ERC777MockCalls::Transfer(element) => element.fmt(f),
                ERC777MockCalls::TransferFrom(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<AllowanceCall> for ERC777MockCalls {
        fn from(var: AllowanceCall) -> Self {
            ERC777MockCalls::Allowance(var)
        }
    }
    impl ::std::convert::From<ApproveCall> for ERC777MockCalls {
        fn from(var: ApproveCall) -> Self {
            ERC777MockCalls::Approve(var)
        }
    }
    impl ::std::convert::From<ApproveInternalCall> for ERC777MockCalls {
        fn from(var: ApproveInternalCall) -> Self {
            ERC777MockCalls::ApproveInternal(var)
        }
    }
    impl ::std::convert::From<AuthorizeOperatorCall> for ERC777MockCalls {
        fn from(var: AuthorizeOperatorCall) -> Self {
            ERC777MockCalls::AuthorizeOperator(var)
        }
    }
    impl ::std::convert::From<BalanceOfCall> for ERC777MockCalls {
        fn from(var: BalanceOfCall) -> Self {
            ERC777MockCalls::BalanceOf(var)
        }
    }
    impl ::std::convert::From<BurnCall> for ERC777MockCalls {
        fn from(var: BurnCall) -> Self {
            ERC777MockCalls::Burn(var)
        }
    }
    impl ::std::convert::From<DecimalsCall> for ERC777MockCalls {
        fn from(var: DecimalsCall) -> Self {
            ERC777MockCalls::Decimals(var)
        }
    }
    impl ::std::convert::From<DefaultOperatorsCall> for ERC777MockCalls {
        fn from(var: DefaultOperatorsCall) -> Self {
            ERC777MockCalls::DefaultOperators(var)
        }
    }
    impl ::std::convert::From<GranularityCall> for ERC777MockCalls {
        fn from(var: GranularityCall) -> Self {
            ERC777MockCalls::Granularity(var)
        }
    }
    impl ::std::convert::From<IsOperatorForCall> for ERC777MockCalls {
        fn from(var: IsOperatorForCall) -> Self {
            ERC777MockCalls::IsOperatorFor(var)
        }
    }
    impl ::std::convert::From<MintInternalCall> for ERC777MockCalls {
        fn from(var: MintInternalCall) -> Self {
            ERC777MockCalls::MintInternal(var)
        }
    }
    impl ::std::convert::From<NameCall> for ERC777MockCalls {
        fn from(var: NameCall) -> Self {
            ERC777MockCalls::Name(var)
        }
    }
    impl ::std::convert::From<OperatorBurnCall> for ERC777MockCalls {
        fn from(var: OperatorBurnCall) -> Self {
            ERC777MockCalls::OperatorBurn(var)
        }
    }
    impl ::std::convert::From<OperatorSendCall> for ERC777MockCalls {
        fn from(var: OperatorSendCall) -> Self {
            ERC777MockCalls::OperatorSend(var)
        }
    }
    impl ::std::convert::From<RevokeOperatorCall> for ERC777MockCalls {
        fn from(var: RevokeOperatorCall) -> Self {
            ERC777MockCalls::RevokeOperator(var)
        }
    }
    impl ::std::convert::From<SendCall> for ERC777MockCalls {
        fn from(var: SendCall) -> Self {
            ERC777MockCalls::Send(var)
        }
    }
    impl ::std::convert::From<SymbolCall> for ERC777MockCalls {
        fn from(var: SymbolCall) -> Self {
            ERC777MockCalls::Symbol(var)
        }
    }
    impl ::std::convert::From<TotalSupplyCall> for ERC777MockCalls {
        fn from(var: TotalSupplyCall) -> Self {
            ERC777MockCalls::TotalSupply(var)
        }
    }
    impl ::std::convert::From<TransferCall> for ERC777MockCalls {
        fn from(var: TransferCall) -> Self {
            ERC777MockCalls::Transfer(var)
        }
    }
    impl ::std::convert::From<TransferFromCall> for ERC777MockCalls {
        fn from(var: TransferFromCall) -> Self {
            ERC777MockCalls::TransferFrom(var)
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
