pub use erc721_mock::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod erc721_mock {
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
    #[doc = "ERC721Mock was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"owner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"approved\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"tokenId\",\"type\":\"uint256\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"Approval\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"owner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"bool\",\"name\":\"approved\",\"type\":\"bool\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"ApprovalForAll\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"tokenId\",\"type\":\"uint256\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"Transfer\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"tokenId\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"approve\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"owner\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"balanceOf\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"baseURI\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"tokenId\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"burn\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"tokenId\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"exists\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"tokenId\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getApproved\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"owner\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"isApprovedForAll\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"tokenId\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"mint\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"name\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"tokenId\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"ownerOf\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"tokenId\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"_data\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"safeMint\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"tokenId\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"safeMint\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"tokenId\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"safeTransferFrom\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"tokenId\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"_data\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"safeTransferFrom\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"bool\",\"name\":\"approved\",\"type\":\"bool\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"setApprovalForAll\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bytes4\",\"name\":\"interfaceId\",\"type\":\"bytes4\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"supportsInterface\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"symbol\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"tokenId\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"tokenURI\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"tokenId\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferFrom\",\"outputs\":[]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static ERC721MOCK_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static ERC721MOCK_BYTECODE: ethers::contract::Lazy<ethers::core::types::Bytes> =
        ethers::contract::Lazy::new(|| {
            "0x60806040523480156200001157600080fd5b50604080518082018252600b81526a455243373231204d6f636b60a81b6020808301918252835180850190945260048452634d37323160e01b908401528151919291620000619160009162000080565b5080516200007790600190602084019062000080565b50505062000162565b8280546200008e9062000126565b90600052602060002090601f016020900481019282620000b25760008555620000fd565b82601f10620000cd57805160ff1916838001178555620000fd565b82800160010185558215620000fd579182015b82811115620000fd578251825591602001919060010190620000e0565b506200010b9291506200010f565b5090565b5b808211156200010b576000815560010162000110565b600181811c908216806200013b57607f821691505b6020821081036200015c57634e487b7160e01b600052602260045260246000fd5b50919050565b61160780620001726000396000f3fe608060405234801561001057600080fd5b50600436106101215760003560e01c80636352211e116100ad578063a144819411610071578063a144819414610259578063a22cb4651461026c578063b88d4fde1461027f578063c87b56dd14610292578063e985e9c5146102a557600080fd5b80636352211e146102025780636c0360eb1461021557806370a082311461021d5780638832e6e31461023e57806395d89b411461025157600080fd5b806323b872dd116100f457806323b872dd146101a357806340c10f19146101b657806342842e0e146101c957806342966c68146101dc5780634f558e79146101ef57600080fd5b806301ffc9a71461012657806306fdde031461014e578063081812fc14610163578063095ea7b31461018e575b600080fd5b6101396101343660046110aa565b6102e1565b60405190151581526020015b60405180910390f35b610156610333565b604051610145919061111f565b610176610171366004611132565b6103c5565b6040516001600160a01b039091168152602001610145565b6101a161019c366004611167565b61045f565b005b6101a16101b1366004611191565b610574565b6101a16101c4366004611167565b6105a5565b6101a16101d7366004611191565b6105b3565b6101a16101ea366004611132565b6105ce565b6101396101fd366004611132565b6105da565b610176610210366004611132565b6105f9565b610156610670565b61023061022b3660046111cd565b61068c565b604051908152602001610145565b6101a161024c36600461128b565b610713565b61015661071e565b6101a1610267366004611167565b61072d565b6101a161027a3660046112e2565b610737565b6101a161028d36600461131e565b610742565b6101566102a0366004611132565b61077a565b6101396102b3366004611386565b6001600160a01b03918216600090815260056020908152604080832093909416825291909152205460ff1690565b60006001600160e01b031982166380ac58cd60e01b148061031257506001600160e01b03198216635b5e139f60e01b145b8061032d57506301ffc9a760e01b6001600160e01b03198316145b92915050565b606060008054610342906113b9565b80601f016020809104026020016040519081016040528092919081815260200182805461036e906113b9565b80156103bb5780601f10610390576101008083540402835291602001916103bb565b820191906000526020600020905b81548152906001019060200180831161039e57829003601f168201915b5050505050905090565b6000818152600260205260408120546001600160a01b03166104435760405162461bcd60e51b815260206004820152602c60248201527f4552433732313a20617070726f76656420717565727920666f72206e6f6e657860448201526b34b9ba32b73a103a37b5b2b760a11b60648201526084015b60405180910390fd5b506000908152600460205260409020546001600160a01b031690565b600061046a826105f9565b9050806001600160a01b0316836001600160a01b0316036104d75760405162461bcd60e51b815260206004820152602160248201527f4552433732313a20617070726f76616c20746f2063757272656e74206f776e656044820152603960f91b606482015260840161043a565b336001600160a01b03821614806104f357506104f381336102b3565b6105655760405162461bcd60e51b815260206004820152603860248201527f4552433732313a20617070726f76652063616c6c6572206973206e6f74206f7760448201527f6e6572206e6f7220617070726f76656420666f7220616c6c0000000000000000606482015260840161043a565b61056f8383610862565b505050565b61057e33826108d0565b61059a5760405162461bcd60e51b815260040161043a906113f3565b61056f8383836109c7565b6105af8282610b67565b5050565b61056f83838360405180602001604052806000815250610742565b6105d781610ca9565b50565b6000818152600260205260408120546001600160a01b0316151561032d565b6000818152600260205260408120546001600160a01b03168061032d5760405162461bcd60e51b815260206004820152602960248201527f4552433732313a206f776e657220717565727920666f72206e6f6e657869737460448201526832b73a103a37b5b2b760b91b606482015260840161043a565b606061068760408051602081019091526000815290565b905090565b60006001600160a01b0382166106f75760405162461bcd60e51b815260206004820152602a60248201527f4552433732313a2062616c616e636520717565727920666f7220746865207a65604482015269726f206164647265737360b01b606482015260840161043a565b506001600160a01b031660009081526003602052604090205490565b61056f838383610d44565b606060018054610342906113b9565b6105af8282610d77565b6105af338383610d91565b61074c33836108d0565b6107685760405162461bcd60e51b815260040161043a906113f3565b61077484848484610e5f565b50505050565b6000818152600260205260409020546060906001600160a01b03166107f95760405162461bcd60e51b815260206004820152602f60248201527f4552433732314d657461646174613a2055524920717565727920666f72206e6f60448201526e3732bc34b9ba32b73a103a37b5b2b760891b606482015260840161043a565b600061081060408051602081019091526000815290565b90506000815111610830576040518060200160405280600081525061085b565b8061083a84610e92565b60405160200161084b929190611444565b6040516020818303038152906040525b9392505050565b600081815260046020526040902080546001600160a01b0319166001600160a01b0384169081179091558190610897826105f9565b6001600160a01b03167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b92560405160405180910390a45050565b6000818152600260205260408120546001600160a01b03166109495760405162461bcd60e51b815260206004820152602c60248201527f4552433732313a206f70657261746f7220717565727920666f72206e6f6e657860448201526b34b9ba32b73a103a37b5b2b760a11b606482015260840161043a565b6000610954836105f9565b9050806001600160a01b0316846001600160a01b0316148061098f5750836001600160a01b0316610984846103c5565b6001600160a01b0316145b806109bf57506001600160a01b0380821660009081526005602090815260408083209388168352929052205460ff165b949350505050565b826001600160a01b03166109da826105f9565b6001600160a01b031614610a425760405162461bcd60e51b815260206004820152602960248201527f4552433732313a207472616e73666572206f6620746f6b656e2074686174206960448201526839903737ba1037bbb760b91b606482015260840161043a565b6001600160a01b038216610aa45760405162461bcd60e51b8152602060048201526024808201527f4552433732313a207472616e7366657220746f20746865207a65726f206164646044820152637265737360e01b606482015260840161043a565b610aaf600082610862565b6001600160a01b0383166000908152600360205260408120805460019290610ad8908490611489565b90915550506001600160a01b0382166000908152600360205260408120805460019290610b069084906114a0565b909155505060008181526002602052604080822080546001600160a01b0319166001600160a01b0386811691821790925591518493918716917fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef91a4505050565b6001600160a01b038216610bbd5760405162461bcd60e51b815260206004820181905260248201527f4552433732313a206d696e7420746f20746865207a65726f2061646472657373604482015260640161043a565b6000818152600260205260409020546001600160a01b031615610c225760405162461bcd60e51b815260206004820152601c60248201527f4552433732313a20746f6b656e20616c7265616479206d696e74656400000000604482015260640161043a565b6001600160a01b0382166000908152600360205260408120805460019290610c4b9084906114a0565b909155505060008181526002602052604080822080546001600160a01b0319166001600160a01b03861690811790915590518392907fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef908290a45050565b6000610cb4826105f9565b9050610cc1600083610862565b6001600160a01b0381166000908152600360205260408120805460019290610cea908490611489565b909155505060008281526002602052604080822080546001600160a01b0319169055518391906001600160a01b038416907fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef908390a45050565b610d4e8383610b67565b610d5b6000848484610f93565b61056f5760405162461bcd60e51b815260040161043a906114b8565b6105af828260405180602001604052806000815250610d44565b816001600160a01b0316836001600160a01b031603610df25760405162461bcd60e51b815260206004820152601960248201527f4552433732313a20617070726f766520746f2063616c6c657200000000000000604482015260640161043a565b6001600160a01b03838116600081815260056020908152604080832094871680845294825291829020805460ff191686151590811790915591519182527f17307eab39ab6107e8899845ad3d59bd9653f200f220920489ca2b5937696c31910160405180910390a3505050565b610e6a8484846109c7565b610e7684848484610f93565b6107745760405162461bcd60e51b815260040161043a906114b8565b606081600003610eb95750506040805180820190915260018152600360fc1b602082015290565b8160005b8115610ee35780610ecd8161150a565b9150610edc9050600a83611539565b9150610ebd565b60008167ffffffffffffffff811115610efe57610efe6111e8565b6040519080825280601f01601f191660200182016040528015610f28576020820181803683370190505b5090505b84156109bf57610f3d600183611489565b9150610f4a600a8661154d565b610f559060306114a0565b60f81b818381518110610f6a57610f6a611561565b60200101906001600160f81b031916908160001a905350610f8c600a86611539565b9450610f2c565b60006001600160a01b0384163b1561108957604051630a85bd0160e11b81526001600160a01b0385169063150b7a0290610fd7903390899088908890600401611577565b6020604051808303816000875af1925050508015611012575060408051601f3d908101601f1916820190925261100f918101906115b4565b60015b61106f573d808015611040576040519150601f19603f3d011682016040523d82523d6000602084013e611045565b606091505b5080516000036110675760405162461bcd60e51b815260040161043a906114b8565b805181602001fd5b6001600160e01b031916630a85bd0160e11b1490506109bf565b506001949350505050565b6001600160e01b0319811681146105d757600080fd5b6000602082840312156110bc57600080fd5b813561085b81611094565b60005b838110156110e25781810151838201526020016110ca565b838111156107745750506000910152565b6000815180845261110b8160208601602086016110c7565b601f01601f19169290920160200192915050565b60208152600061085b60208301846110f3565b60006020828403121561114457600080fd5b5035919050565b80356001600160a01b038116811461116257600080fd5b919050565b6000806040838503121561117a57600080fd5b6111838361114b565b946020939093013593505050565b6000806000606084860312156111a657600080fd5b6111af8461114b565b92506111bd6020850161114b565b9150604084013590509250925092565b6000602082840312156111df57600080fd5b61085b8261114b565b634e487b7160e01b600052604160045260246000fd5b600082601f83011261120f57600080fd5b813567ffffffffffffffff8082111561122a5761122a6111e8565b604051601f8301601f19908116603f01168101908282118183101715611252576112526111e8565b8160405283815286602085880101111561126b57600080fd5b836020870160208301376000602085830101528094505050505092915050565b6000806000606084860312156112a057600080fd5b6112a98461114b565b925060208401359150604084013567ffffffffffffffff8111156112cc57600080fd5b6112d8868287016111fe565b9150509250925092565b600080604083850312156112f557600080fd5b6112fe8361114b565b91506020830135801515811461131357600080fd5b809150509250929050565b6000806000806080858703121561133457600080fd5b61133d8561114b565b935061134b6020860161114b565b925060408501359150606085013567ffffffffffffffff81111561136e57600080fd5b61137a878288016111fe565b91505092959194509250565b6000806040838503121561139957600080fd5b6113a28361114b565b91506113b06020840161114b565b90509250929050565b600181811c908216806113cd57607f821691505b6020821081036113ed57634e487b7160e01b600052602260045260246000fd5b50919050565b60208082526031908201527f4552433732313a207472616e736665722063616c6c6572206973206e6f74206f6040820152701ddb995c881b9bdc88185c1c1c9bdd9959607a1b606082015260800190565b600083516114568184602088016110c7565b83519083019061146a8183602088016110c7565b01949350505050565b634e487b7160e01b600052601160045260246000fd5b60008282101561149b5761149b611473565b500390565b600082198211156114b3576114b3611473565b500190565b60208082526032908201527f4552433732313a207472616e7366657220746f206e6f6e20455243373231526560408201527131b2b4bb32b91034b6b83632b6b2b73a32b960711b606082015260800190565b60006001820161151c5761151c611473565b5060010190565b634e487b7160e01b600052601260045260246000fd5b60008261154857611548611523565b500490565b60008261155c5761155c611523565b500690565b634e487b7160e01b600052603260045260246000fd5b6001600160a01b03858116825284166020820152604081018390526080606082018190526000906115aa908301846110f3565b9695505050505050565b6000602082840312156115c657600080fd5b815161085b8161109456fea26469706673582212208ca05dec68c7888146f0448dd74a9367a27f9bb2e73ee747fc9a8dff6d13342d64736f6c634300080d0033" . parse () . expect ("invalid bytecode")
        });
    pub struct ERC721Mock<M>(ethers::contract::Contract<M>);
    impl<M> Clone for ERC721Mock<M> {
        fn clone(&self) -> Self {
            ERC721Mock(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for ERC721Mock<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for ERC721Mock<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(ERC721Mock))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> ERC721Mock<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), ERC721MOCK_ABI.clone(), client).into()
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
                ERC721MOCK_ABI.clone(),
                ERC721MOCK_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        #[doc = "Calls the contract's `approve` (0x095ea7b3) function"]
        pub fn approve(
            &self,
            to: ethers::core::types::Address,
            token_id: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([9, 94, 167, 179], (to, token_id))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `balanceOf` (0x70a08231) function"]
        pub fn balance_of(
            &self,
            owner: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([112, 160, 130, 49], owner)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `baseURI` (0x6c0360eb) function"]
        pub fn base_uri(&self) -> ethers::contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([108, 3, 96, 235], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `burn` (0x42966c68) function"]
        pub fn burn(
            &self,
            token_id: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([66, 150, 108, 104], token_id)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `exists` (0x4f558e79) function"]
        pub fn exists(
            &self,
            token_id: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([79, 85, 142, 121], token_id)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getApproved` (0x081812fc) function"]
        pub fn get_approved(
            &self,
            token_id: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([8, 24, 18, 252], token_id)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `isApprovedForAll` (0xe985e9c5) function"]
        pub fn is_approved_for_all(
            &self,
            owner: ethers::core::types::Address,
            operator: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([233, 133, 233, 197], (owner, operator))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `mint` (0x40c10f19) function"]
        pub fn mint(
            &self,
            to: ethers::core::types::Address,
            token_id: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([64, 193, 15, 25], (to, token_id))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `name` (0x06fdde03) function"]
        pub fn name(&self) -> ethers::contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([6, 253, 222, 3], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `ownerOf` (0x6352211e) function"]
        pub fn owner_of(
            &self,
            token_id: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([99, 82, 33, 30], token_id)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `safeMint` (0x8832e6e3) function"]
        pub fn safe_mint_with_data(
            &self,
            to: ethers::core::types::Address,
            token_id: ethers::core::types::U256,
            data: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([136, 50, 230, 227], (to, token_id, data))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `safeMint` (0xa1448194) function"]
        pub fn safe_mint(
            &self,
            to: ethers::core::types::Address,
            token_id: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([161, 68, 129, 148], (to, token_id))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `safeTransferFrom` (0x42842e0e) function"]
        pub fn safe_transfer_from(
            &self,
            from: ethers::core::types::Address,
            to: ethers::core::types::Address,
            token_id: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([66, 132, 46, 14], (from, to, token_id))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `safeTransferFrom` (0xb88d4fde) function"]
        pub fn safe_transfer_from_with_data(
            &self,
            from: ethers::core::types::Address,
            to: ethers::core::types::Address,
            token_id: ethers::core::types::U256,
            data: ethers::core::types::Bytes,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([184, 141, 79, 222], (from, to, token_id, data))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `setApprovalForAll` (0xa22cb465) function"]
        pub fn set_approval_for_all(
            &self,
            operator: ethers::core::types::Address,
            approved: bool,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([162, 44, 180, 101], (operator, approved))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `supportsInterface` (0x01ffc9a7) function"]
        pub fn supports_interface(
            &self,
            interface_id: [u8; 4],
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([1, 255, 201, 167], interface_id)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `symbol` (0x95d89b41) function"]
        pub fn symbol(&self) -> ethers::contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([149, 216, 155, 65], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `tokenURI` (0xc87b56dd) function"]
        pub fn token_uri(
            &self,
            token_id: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([200, 123, 86, 221], token_id)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `transferFrom` (0x23b872dd) function"]
        pub fn transfer_from(
            &self,
            from: ethers::core::types::Address,
            to: ethers::core::types::Address,
            token_id: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([35, 184, 114, 221], (from, to, token_id))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Gets the contract's `Approval` event"]
        pub fn approval_filter(&self) -> ethers::contract::builders::Event<M, ApprovalFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `ApprovalForAll` event"]
        pub fn approval_for_all_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, ApprovalForAllFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `Transfer` event"]
        pub fn transfer_filter(&self) -> ethers::contract::builders::Event<M, TransferFilter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(&self) -> ethers::contract::builders::Event<M, ERC721MockEvents> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for ERC721Mock<M> {
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
        pub approved: ethers::core::types::Address,
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
    #[ethevent(name = "ApprovalForAll", abi = "ApprovalForAll(address,address,bool)")]
    pub struct ApprovalForAllFilter {
        #[ethevent(indexed)]
        pub owner: ethers::core::types::Address,
        #[ethevent(indexed)]
        pub operator: ethers::core::types::Address,
        pub approved: bool,
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
        #[ethevent(indexed)]
        pub token_id: ethers::core::types::U256,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum ERC721MockEvents {
        ApprovalFilter(ApprovalFilter),
        ApprovalForAllFilter(ApprovalForAllFilter),
        TransferFilter(TransferFilter),
    }
    impl ethers::contract::EthLogDecode for ERC721MockEvents {
        fn decode_log(
            log: &ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = ApprovalFilter::decode_log(log) {
                return Ok(ERC721MockEvents::ApprovalFilter(decoded));
            }
            if let Ok(decoded) = ApprovalForAllFilter::decode_log(log) {
                return Ok(ERC721MockEvents::ApprovalForAllFilter(decoded));
            }
            if let Ok(decoded) = TransferFilter::decode_log(log) {
                return Ok(ERC721MockEvents::TransferFilter(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for ERC721MockEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                ERC721MockEvents::ApprovalFilter(element) => element.fmt(f),
                ERC721MockEvents::ApprovalForAllFilter(element) => element.fmt(f),
                ERC721MockEvents::TransferFilter(element) => element.fmt(f),
            }
        }
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
        pub to: ethers::core::types::Address,
        pub token_id: ethers::core::types::U256,
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
        pub owner: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `baseURI` function with signature `baseURI()` and selector `[108, 3, 96, 235]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "baseURI", abi = "baseURI()")]
    pub struct BaseURICall;
    #[doc = "Container type for all input parameters for the `burn` function with signature `burn(uint256)` and selector `[66, 150, 108, 104]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "burn", abi = "burn(uint256)")]
    pub struct BurnCall {
        pub token_id: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `exists` function with signature `exists(uint256)` and selector `[79, 85, 142, 121]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "exists", abi = "exists(uint256)")]
    pub struct ExistsCall {
        pub token_id: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `getApproved` function with signature `getApproved(uint256)` and selector `[8, 24, 18, 252]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "getApproved", abi = "getApproved(uint256)")]
    pub struct GetApprovedCall {
        pub token_id: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `isApprovedForAll` function with signature `isApprovedForAll(address,address)` and selector `[233, 133, 233, 197]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "isApprovedForAll", abi = "isApprovedForAll(address,address)")]
    pub struct IsApprovedForAllCall {
        pub owner: ethers::core::types::Address,
        pub operator: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `mint` function with signature `mint(address,uint256)` and selector `[64, 193, 15, 25]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "mint", abi = "mint(address,uint256)")]
    pub struct MintCall {
        pub to: ethers::core::types::Address,
        pub token_id: ethers::core::types::U256,
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
    #[doc = "Container type for all input parameters for the `ownerOf` function with signature `ownerOf(uint256)` and selector `[99, 82, 33, 30]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "ownerOf", abi = "ownerOf(uint256)")]
    pub struct OwnerOfCall {
        pub token_id: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `safeMint` function with signature `safeMint(address,uint256,bytes)` and selector `[136, 50, 230, 227]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "safeMint", abi = "safeMint(address,uint256,bytes)")]
    pub struct SafeMintWithDataCall {
        pub to: ethers::core::types::Address,
        pub token_id: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
    }
    #[doc = "Container type for all input parameters for the `safeMint` function with signature `safeMint(address,uint256)` and selector `[161, 68, 129, 148]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "safeMint", abi = "safeMint(address,uint256)")]
    pub struct SafeMintCall {
        pub to: ethers::core::types::Address,
        pub token_id: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `safeTransferFrom` function with signature `safeTransferFrom(address,address,uint256)` and selector `[66, 132, 46, 14]`"]
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
        name = "safeTransferFrom",
        abi = "safeTransferFrom(address,address,uint256)"
    )]
    pub struct SafeTransferFromCall {
        pub from: ethers::core::types::Address,
        pub to: ethers::core::types::Address,
        pub token_id: ethers::core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `safeTransferFrom` function with signature `safeTransferFrom(address,address,uint256,bytes)` and selector `[184, 141, 79, 222]`"]
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
        name = "safeTransferFrom",
        abi = "safeTransferFrom(address,address,uint256,bytes)"
    )]
    pub struct SafeTransferFromWithDataCall {
        pub from: ethers::core::types::Address,
        pub to: ethers::core::types::Address,
        pub token_id: ethers::core::types::U256,
        pub data: ethers::core::types::Bytes,
    }
    #[doc = "Container type for all input parameters for the `setApprovalForAll` function with signature `setApprovalForAll(address,bool)` and selector `[162, 44, 180, 101]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "setApprovalForAll", abi = "setApprovalForAll(address,bool)")]
    pub struct SetApprovalForAllCall {
        pub operator: ethers::core::types::Address,
        pub approved: bool,
    }
    #[doc = "Container type for all input parameters for the `supportsInterface` function with signature `supportsInterface(bytes4)` and selector `[1, 255, 201, 167]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "supportsInterface", abi = "supportsInterface(bytes4)")]
    pub struct SupportsInterfaceCall {
        pub interface_id: [u8; 4],
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
    #[doc = "Container type for all input parameters for the `tokenURI` function with signature `tokenURI(uint256)` and selector `[200, 123, 86, 221]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "tokenURI", abi = "tokenURI(uint256)")]
    pub struct TokenURICall {
        pub token_id: ethers::core::types::U256,
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
        pub from: ethers::core::types::Address,
        pub to: ethers::core::types::Address,
        pub token_id: ethers::core::types::U256,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum ERC721MockCalls {
        Approve(ApproveCall),
        BalanceOf(BalanceOfCall),
        BaseURI(BaseURICall),
        Burn(BurnCall),
        Exists(ExistsCall),
        GetApproved(GetApprovedCall),
        IsApprovedForAll(IsApprovedForAllCall),
        Mint(MintCall),
        Name(NameCall),
        OwnerOf(OwnerOfCall),
        SafeMintWithData(SafeMintWithDataCall),
        SafeMint(SafeMintCall),
        SafeTransferFrom(SafeTransferFromCall),
        SafeTransferFromWithData(SafeTransferFromWithDataCall),
        SetApprovalForAll(SetApprovalForAllCall),
        SupportsInterface(SupportsInterfaceCall),
        Symbol(SymbolCall),
        TokenURI(TokenURICall),
        TransferFrom(TransferFromCall),
    }
    impl ethers::core::abi::AbiDecode for ERC721MockCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <ApproveCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC721MockCalls::Approve(decoded));
            }
            if let Ok(decoded) =
                <BalanceOfCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC721MockCalls::BalanceOf(decoded));
            }
            if let Ok(decoded) =
                <BaseURICall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC721MockCalls::BaseURI(decoded));
            }
            if let Ok(decoded) = <BurnCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(ERC721MockCalls::Burn(decoded));
            }
            if let Ok(decoded) = <ExistsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC721MockCalls::Exists(decoded));
            }
            if let Ok(decoded) =
                <GetApprovedCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC721MockCalls::GetApproved(decoded));
            }
            if let Ok(decoded) =
                <IsApprovedForAllCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC721MockCalls::IsApprovedForAll(decoded));
            }
            if let Ok(decoded) = <MintCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(ERC721MockCalls::Mint(decoded));
            }
            if let Ok(decoded) = <NameCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(ERC721MockCalls::Name(decoded));
            }
            if let Ok(decoded) =
                <OwnerOfCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC721MockCalls::OwnerOf(decoded));
            }
            if let Ok(decoded) =
                <SafeMintWithDataCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC721MockCalls::SafeMintWithData(decoded));
            }
            if let Ok(decoded) =
                <SafeMintCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC721MockCalls::SafeMint(decoded));
            }
            if let Ok(decoded) =
                <SafeTransferFromCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC721MockCalls::SafeTransferFrom(decoded));
            }
            if let Ok(decoded) =
                <SafeTransferFromWithDataCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(ERC721MockCalls::SafeTransferFromWithData(decoded));
            }
            if let Ok(decoded) =
                <SetApprovalForAllCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC721MockCalls::SetApprovalForAll(decoded));
            }
            if let Ok(decoded) =
                <SupportsInterfaceCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC721MockCalls::SupportsInterface(decoded));
            }
            if let Ok(decoded) = <SymbolCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC721MockCalls::Symbol(decoded));
            }
            if let Ok(decoded) =
                <TokenURICall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC721MockCalls::TokenURI(decoded));
            }
            if let Ok(decoded) =
                <TransferFromCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ERC721MockCalls::TransferFrom(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for ERC721MockCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                ERC721MockCalls::Approve(element) => element.encode(),
                ERC721MockCalls::BalanceOf(element) => element.encode(),
                ERC721MockCalls::BaseURI(element) => element.encode(),
                ERC721MockCalls::Burn(element) => element.encode(),
                ERC721MockCalls::Exists(element) => element.encode(),
                ERC721MockCalls::GetApproved(element) => element.encode(),
                ERC721MockCalls::IsApprovedForAll(element) => element.encode(),
                ERC721MockCalls::Mint(element) => element.encode(),
                ERC721MockCalls::Name(element) => element.encode(),
                ERC721MockCalls::OwnerOf(element) => element.encode(),
                ERC721MockCalls::SafeMintWithData(element) => element.encode(),
                ERC721MockCalls::SafeMint(element) => element.encode(),
                ERC721MockCalls::SafeTransferFrom(element) => element.encode(),
                ERC721MockCalls::SafeTransferFromWithData(element) => element.encode(),
                ERC721MockCalls::SetApprovalForAll(element) => element.encode(),
                ERC721MockCalls::SupportsInterface(element) => element.encode(),
                ERC721MockCalls::Symbol(element) => element.encode(),
                ERC721MockCalls::TokenURI(element) => element.encode(),
                ERC721MockCalls::TransferFrom(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for ERC721MockCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                ERC721MockCalls::Approve(element) => element.fmt(f),
                ERC721MockCalls::BalanceOf(element) => element.fmt(f),
                ERC721MockCalls::BaseURI(element) => element.fmt(f),
                ERC721MockCalls::Burn(element) => element.fmt(f),
                ERC721MockCalls::Exists(element) => element.fmt(f),
                ERC721MockCalls::GetApproved(element) => element.fmt(f),
                ERC721MockCalls::IsApprovedForAll(element) => element.fmt(f),
                ERC721MockCalls::Mint(element) => element.fmt(f),
                ERC721MockCalls::Name(element) => element.fmt(f),
                ERC721MockCalls::OwnerOf(element) => element.fmt(f),
                ERC721MockCalls::SafeMintWithData(element) => element.fmt(f),
                ERC721MockCalls::SafeMint(element) => element.fmt(f),
                ERC721MockCalls::SafeTransferFrom(element) => element.fmt(f),
                ERC721MockCalls::SafeTransferFromWithData(element) => element.fmt(f),
                ERC721MockCalls::SetApprovalForAll(element) => element.fmt(f),
                ERC721MockCalls::SupportsInterface(element) => element.fmt(f),
                ERC721MockCalls::Symbol(element) => element.fmt(f),
                ERC721MockCalls::TokenURI(element) => element.fmt(f),
                ERC721MockCalls::TransferFrom(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<ApproveCall> for ERC721MockCalls {
        fn from(var: ApproveCall) -> Self {
            ERC721MockCalls::Approve(var)
        }
    }
    impl ::std::convert::From<BalanceOfCall> for ERC721MockCalls {
        fn from(var: BalanceOfCall) -> Self {
            ERC721MockCalls::BalanceOf(var)
        }
    }
    impl ::std::convert::From<BaseURICall> for ERC721MockCalls {
        fn from(var: BaseURICall) -> Self {
            ERC721MockCalls::BaseURI(var)
        }
    }
    impl ::std::convert::From<BurnCall> for ERC721MockCalls {
        fn from(var: BurnCall) -> Self {
            ERC721MockCalls::Burn(var)
        }
    }
    impl ::std::convert::From<ExistsCall> for ERC721MockCalls {
        fn from(var: ExistsCall) -> Self {
            ERC721MockCalls::Exists(var)
        }
    }
    impl ::std::convert::From<GetApprovedCall> for ERC721MockCalls {
        fn from(var: GetApprovedCall) -> Self {
            ERC721MockCalls::GetApproved(var)
        }
    }
    impl ::std::convert::From<IsApprovedForAllCall> for ERC721MockCalls {
        fn from(var: IsApprovedForAllCall) -> Self {
            ERC721MockCalls::IsApprovedForAll(var)
        }
    }
    impl ::std::convert::From<MintCall> for ERC721MockCalls {
        fn from(var: MintCall) -> Self {
            ERC721MockCalls::Mint(var)
        }
    }
    impl ::std::convert::From<NameCall> for ERC721MockCalls {
        fn from(var: NameCall) -> Self {
            ERC721MockCalls::Name(var)
        }
    }
    impl ::std::convert::From<OwnerOfCall> for ERC721MockCalls {
        fn from(var: OwnerOfCall) -> Self {
            ERC721MockCalls::OwnerOf(var)
        }
    }
    impl ::std::convert::From<SafeMintWithDataCall> for ERC721MockCalls {
        fn from(var: SafeMintWithDataCall) -> Self {
            ERC721MockCalls::SafeMintWithData(var)
        }
    }
    impl ::std::convert::From<SafeMintCall> for ERC721MockCalls {
        fn from(var: SafeMintCall) -> Self {
            ERC721MockCalls::SafeMint(var)
        }
    }
    impl ::std::convert::From<SafeTransferFromCall> for ERC721MockCalls {
        fn from(var: SafeTransferFromCall) -> Self {
            ERC721MockCalls::SafeTransferFrom(var)
        }
    }
    impl ::std::convert::From<SafeTransferFromWithDataCall> for ERC721MockCalls {
        fn from(var: SafeTransferFromWithDataCall) -> Self {
            ERC721MockCalls::SafeTransferFromWithData(var)
        }
    }
    impl ::std::convert::From<SetApprovalForAllCall> for ERC721MockCalls {
        fn from(var: SetApprovalForAllCall) -> Self {
            ERC721MockCalls::SetApprovalForAll(var)
        }
    }
    impl ::std::convert::From<SupportsInterfaceCall> for ERC721MockCalls {
        fn from(var: SupportsInterfaceCall) -> Self {
            ERC721MockCalls::SupportsInterface(var)
        }
    }
    impl ::std::convert::From<SymbolCall> for ERC721MockCalls {
        fn from(var: SymbolCall) -> Self {
            ERC721MockCalls::Symbol(var)
        }
    }
    impl ::std::convert::From<TokenURICall> for ERC721MockCalls {
        fn from(var: TokenURICall) -> Self {
            ERC721MockCalls::TokenURI(var)
        }
    }
    impl ::std::convert::From<TransferFromCall> for ERC721MockCalls {
        fn from(var: TransferFromCall) -> Self {
            ERC721MockCalls::TransferFrom(var)
        }
    }
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
    #[doc = "Container type for all return fields from the `baseURI` function with signature `baseURI()` and selector `[108, 3, 96, 235]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct BaseURIReturn(pub String);
    #[doc = "Container type for all return fields from the `exists` function with signature `exists(uint256)` and selector `[79, 85, 142, 121]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct ExistsReturn(pub bool);
    #[doc = "Container type for all return fields from the `getApproved` function with signature `getApproved(uint256)` and selector `[8, 24, 18, 252]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetApprovedReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `isApprovedForAll` function with signature `isApprovedForAll(address,address)` and selector `[233, 133, 233, 197]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct IsApprovedForAllReturn(pub bool);
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
    #[doc = "Container type for all return fields from the `ownerOf` function with signature `ownerOf(uint256)` and selector `[99, 82, 33, 30]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct OwnerOfReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `supportsInterface` function with signature `supportsInterface(bytes4)` and selector `[1, 255, 201, 167]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct SupportsInterfaceReturn(pub bool);
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
    #[doc = "Container type for all return fields from the `tokenURI` function with signature `tokenURI(uint256)` and selector `[200, 123, 86, 221]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct TokenURIReturn(pub String);
}
