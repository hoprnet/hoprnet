pub use hopr_wrapper_proxy::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod hopr_wrapper_proxy {
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
    #[doc = "HoprWrapperProxy was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"FowardedFrom\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"FowardedTo\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"ERC1820_REGISTRY\",\"outputs\":[{\"internalType\":\"contract IERC1820Registry\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"TOKENS_RECIPIENT_INTERFACE_HASH\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"WRAPPER\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"WXHOPR_TOKEN\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"XDAI_MULTISIG\",\"outputs\":[{\"internalType\":\"address payable\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"XHOPR_TOKEN\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_from\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"_value\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"_data\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"onTokenTransfer\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"token\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"recoverTokens\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"userData\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"tokensReceived\",\"outputs\":[]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static HOPRWRAPPERPROXY_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static HOPRWRAPPERPROXY_BYTECODE: ethers::contract::Lazy<ethers::core::types::Bytes> =
        ethers::contract::Lazy::new(|| {
            "0x608060405234801561001057600080fd5b506040516329965a1d60e01b815230600482018190527fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b60248301526044820152731820a4b7618bde71dce8cdc73aab6c95905fad24906329965a1d90606401600060405180830381600087803b15801561008a57600080fd5b505af115801561009e573d6000803e3d6000fd5b50505050610bb8806100b16000396000f3fe608060405234801561001057600080fd5b50600436106100925760003560e01c8063a4c0ed3611610066578063a4c0ed361461012c578063b861fa9e1461014f578063e1eb13c11461016a578063e28f56f314610185578063fe26277b146101a057600080fd5b806223de2914610097578063013eb177146100ac57806316114acd146100e457806372581cc0146100f7575b600080fd5b6100aa6100a5366004610921565b6101bb565b005b6100c7731820a4b7618bde71dce8cdc73aab6c95905fad2481565b6040516001600160a01b0390911681526020015b60405180910390f35b6100aa6100f23660046109cc565b610332565b61011e7fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b81565b6040519081526020016100db565b61013f61013a3660046109fd565b61041a565b60405190151581526020016100db565b6100c773d057604a14982fe8d88c5fc25aac3267ea142a0881565b6100c773097707143e01318734535676cfe2e5cf8b656ae881565b6100c7735e1c4e7004b7411ba27dc354330fab31147dfef181565b6100c773d4fdec44db9d44b8f2b6d529620f9c0c7066a2c181565b3373d4fdec44db9d44b8f2b6d529620f9c0c7066a2c11461023c5760405162461bcd60e51b815260206004820152603060248201527f486f70725772617070657250726f78793a204f6e6c792061636365707420575860448201526f2427a8292faa27a5a2a7103a37b5b2b760811b60648201526084015b60405180910390fd5b6001600160a01b03861630146102b35760405162461bcd60e51b815260206004820152603660248201527f486f70725772617070657250726f78793a204d7573742062652073656e64696e6044820152756720746f6b656e7320746f20746869732070726f787960501b6064820152608401610233565b604080516001600160a01b0389168152602081018790527f7c6d66a12116d23472c9d07a15684954389e3cecd458a973e2d52340dcc40077910160405180910390a161032873d4fdec44db9d44b8f2b6d529620f9c0c7066a2c1735e1c4e7004b7411ba27dc354330fab31147dfef18761061b565b5050505050505050565b6001600160a01b03811661038457604051735e1c4e7004b7411ba27dc354330fab31147dfef1904780156108fc02916000818181858888f19350505050158015610380573d6000803e3d6000fd5b5050565b6040516370a0823160e01b815230600482015261041790735e1c4e7004b7411ba27dc354330fab31147dfef1906001600160a01b038416906370a0823190602401602060405180830381865afa1580156103e2573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906104069190610ac8565b6001600160a01b038416919061061b565b50565b60003373d057604a14982fe8d88c5fc25aac3267ea142a08146104915760405162461bcd60e51b815260206004820152602960248201527f486f70725772617070657250726f78793a204f6e6c792061636365707420784860448201526827a829103a37b5b2b760b91b6064820152608401610233565b6001600160a01b038416735e1c4e7004b7411ba27dc354330fab31147dfef1146105235760405162461bcd60e51b815260206004820152603a60248201527f486f70725772617070657250726f78793a204f6e6c792061636365707420784860448201527f4f50522066726f6d20746865204173736f204d756c74695369670000000000006064820152608401610233565b6040805173097707143e01318734535676cfe2e5cf8b656ae88152602081018590527f136ea539e14badd3720f65f9ff6414b0e6291b05e7c191cc3f1c81fa9d2dd569910160405180910390a1604051630200057560e51b815273097707143e01318734535676cfe2e5cf8b656ae8600482015260248101849052606060448201526000606482015273d057604a14982fe8d88c5fc25aac3267ea142a0890634000aea0906084016020604051808303816000875af11580156105ea573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061060e9190610ae1565b50600190505b9392505050565b604080516001600160a01b038416602482015260448082018490528251808303909101815260649091019091526020810180516001600160e01b031663a9059cbb60e01b17905261066d908490610672565b505050565b60006106c7826040518060400160405280602081526020017f5361666545524332303a206c6f772d6c6576656c2063616c6c206661696c6564815250856001600160a01b03166107449092919063ffffffff16565b80519091501561066d57808060200190518101906106e59190610ae1565b61066d5760405162461bcd60e51b815260206004820152602a60248201527f5361666545524332303a204552433230206f7065726174696f6e20646964206e6044820152691bdd081cdd58d8d9595960b21b6064820152608401610233565b6060610753848460008561075b565b949350505050565b6060824710156107bc5760405162461bcd60e51b815260206004820152602660248201527f416464726573733a20696e73756666696369656e742062616c616e636520666f6044820152651c8818d85b1b60d21b6064820152608401610233565b843b61080a5760405162461bcd60e51b815260206004820152601d60248201527f416464726573733a2063616c6c20746f206e6f6e2d636f6e74726163740000006044820152606401610233565b600080866001600160a01b031685876040516108269190610b33565b60006040518083038185875af1925050503d8060008114610863576040519150601f19603f3d011682016040523d82523d6000602084013e610868565b606091505b5091509150610878828286610883565b979650505050505050565b60608315610892575081610614565b8251156108a25782518084602001fd5b8160405162461bcd60e51b81526004016102339190610b4f565b80356001600160a01b03811681146108d357600080fd5b919050565b60008083601f8401126108ea57600080fd5b50813567ffffffffffffffff81111561090257600080fd5b60208301915083602082850101111561091a57600080fd5b9250929050565b60008060008060008060008060c0898b03121561093d57600080fd5b610946896108bc565b975061095460208a016108bc565b965061096260408a016108bc565b955060608901359450608089013567ffffffffffffffff8082111561098657600080fd5b6109928c838d016108d8565b909650945060a08b01359150808211156109ab57600080fd5b506109b88b828c016108d8565b999c989b5096995094979396929594505050565b6000602082840312156109de57600080fd5b610614826108bc565b634e487b7160e01b600052604160045260246000fd5b600080600060608486031215610a1257600080fd5b610a1b846108bc565b925060208401359150604084013567ffffffffffffffff80821115610a3f57600080fd5b818601915086601f830112610a5357600080fd5b813581811115610a6557610a656109e7565b604051601f8201601f19908116603f01168101908382118183101715610a8d57610a8d6109e7565b81604052828152896020848701011115610aa657600080fd5b8260208601602083013760006020848301015280955050505050509250925092565b600060208284031215610ada57600080fd5b5051919050565b600060208284031215610af357600080fd5b8151801515811461061457600080fd5b60005b83811015610b1e578181015183820152602001610b06565b83811115610b2d576000848401525b50505050565b60008251610b45818460208701610b03565b9190910192915050565b6020815260008251806020840152610b6e816040850160208701610b03565b601f01601f1916919091016040019291505056fea26469706673582212206d716334e0a1c660cd28fd75d10934c3d1f50d664afcdaafd86ab189d3f5968f64736f6c634300080d0033" . parse () . expect ("invalid bytecode")
        });
    pub struct HoprWrapperProxy<M>(ethers::contract::Contract<M>);
    impl<M> Clone for HoprWrapperProxy<M> {
        fn clone(&self) -> Self {
            HoprWrapperProxy(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for HoprWrapperProxy<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for HoprWrapperProxy<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(HoprWrapperProxy))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> HoprWrapperProxy<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), HOPRWRAPPERPROXY_ABI.clone(), client)
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
                HOPRWRAPPERPROXY_ABI.clone(),
                HOPRWRAPPERPROXY_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        #[doc = "Calls the contract's `ERC1820_REGISTRY` (0x013eb177) function"]
        pub fn erc1820_registry(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([1, 62, 177, 119], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `TOKENS_RECIPIENT_INTERFACE_HASH` (0x72581cc0) function"]
        pub fn tokens_recipient_interface_hash(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([114, 88, 28, 192], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `WRAPPER` (0xe1eb13c1) function"]
        pub fn wrapper(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([225, 235, 19, 193], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `WXHOPR_TOKEN` (0xfe26277b) function"]
        pub fn wxhopr_token(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([254, 38, 39, 123], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `XDAI_MULTISIG` (0xe28f56f3) function"]
        pub fn xdai_multisig(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([226, 143, 86, 243], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `XHOPR_TOKEN` (0xb861fa9e) function"]
        pub fn xhopr_token(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([184, 97, 250, 158], ())
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
        #[doc = "Calls the contract's `recoverTokens` (0x16114acd) function"]
        pub fn recover_tokens(
            &self,
            token: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([22, 17, 74, 205], token)
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
        #[doc = "Gets the contract's `FowardedFrom` event"]
        pub fn fowarded_from_filter(
            &self,
        ) -> ethers::contract::builders::Event<M, FowardedFromFilter> {
            self.0.event()
        }
        #[doc = "Gets the contract's `FowardedTo` event"]
        pub fn fowarded_to_filter(&self) -> ethers::contract::builders::Event<M, FowardedToFilter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(&self) -> ethers::contract::builders::Event<M, HoprWrapperProxyEvents> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for HoprWrapperProxy<M> {
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
    #[ethevent(name = "FowardedFrom", abi = "FowardedFrom(address,uint256)")]
    pub struct FowardedFromFilter {
        pub from: ethers::core::types::Address,
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
    #[ethevent(name = "FowardedTo", abi = "FowardedTo(address,uint256)")]
    pub struct FowardedToFilter {
        pub to: ethers::core::types::Address,
        pub amount: ethers::core::types::U256,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum HoprWrapperProxyEvents {
        FowardedFromFilter(FowardedFromFilter),
        FowardedToFilter(FowardedToFilter),
    }
    impl ethers::contract::EthLogDecode for HoprWrapperProxyEvents {
        fn decode_log(
            log: &ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = FowardedFromFilter::decode_log(log) {
                return Ok(HoprWrapperProxyEvents::FowardedFromFilter(decoded));
            }
            if let Ok(decoded) = FowardedToFilter::decode_log(log) {
                return Ok(HoprWrapperProxyEvents::FowardedToFilter(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for HoprWrapperProxyEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                HoprWrapperProxyEvents::FowardedFromFilter(element) => element.fmt(f),
                HoprWrapperProxyEvents::FowardedToFilter(element) => element.fmt(f),
            }
        }
    }
    #[doc = "Container type for all input parameters for the `ERC1820_REGISTRY` function with signature `ERC1820_REGISTRY()` and selector `[1, 62, 177, 119]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "ERC1820_REGISTRY", abi = "ERC1820_REGISTRY()")]
    pub struct Erc1820RegistryCall;
    #[doc = "Container type for all input parameters for the `TOKENS_RECIPIENT_INTERFACE_HASH` function with signature `TOKENS_RECIPIENT_INTERFACE_HASH()` and selector `[114, 88, 28, 192]`"]
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
        name = "TOKENS_RECIPIENT_INTERFACE_HASH",
        abi = "TOKENS_RECIPIENT_INTERFACE_HASH()"
    )]
    pub struct TokensRecipientInterfaceHashCall;
    #[doc = "Container type for all input parameters for the `WRAPPER` function with signature `WRAPPER()` and selector `[225, 235, 19, 193]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "WRAPPER", abi = "WRAPPER()")]
    pub struct WrapperCall;
    #[doc = "Container type for all input parameters for the `WXHOPR_TOKEN` function with signature `WXHOPR_TOKEN()` and selector `[254, 38, 39, 123]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "WXHOPR_TOKEN", abi = "WXHOPR_TOKEN()")]
    pub struct WxhoprTokenCall;
    #[doc = "Container type for all input parameters for the `XDAI_MULTISIG` function with signature `XDAI_MULTISIG()` and selector `[226, 143, 86, 243]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "XDAI_MULTISIG", abi = "XDAI_MULTISIG()")]
    pub struct XdaiMultisigCall;
    #[doc = "Container type for all input parameters for the `XHOPR_TOKEN` function with signature `XHOPR_TOKEN()` and selector `[184, 97, 250, 158]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "XHOPR_TOKEN", abi = "XHOPR_TOKEN()")]
    pub struct XhoprTokenCall;
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
    #[doc = "Container type for all input parameters for the `recoverTokens` function with signature `recoverTokens(address)` and selector `[22, 17, 74, 205]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "recoverTokens", abi = "recoverTokens(address)")]
    pub struct RecoverTokensCall {
        pub token: ethers::core::types::Address,
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
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum HoprWrapperProxyCalls {
        Erc1820Registry(Erc1820RegistryCall),
        TokensRecipientInterfaceHash(TokensRecipientInterfaceHashCall),
        Wrapper(WrapperCall),
        WxhoprToken(WxhoprTokenCall),
        XdaiMultisig(XdaiMultisigCall),
        XhoprToken(XhoprTokenCall),
        OnTokenTransfer(OnTokenTransferCall),
        RecoverTokens(RecoverTokensCall),
        TokensReceived(TokensReceivedCall),
    }
    impl ethers::core::abi::AbiDecode for HoprWrapperProxyCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <Erc1820RegistryCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWrapperProxyCalls::Erc1820Registry(decoded));
            }
            if let Ok(decoded) =
                <TokensRecipientInterfaceHashCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(HoprWrapperProxyCalls::TokensRecipientInterfaceHash(decoded));
            }
            if let Ok(decoded) =
                <WrapperCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWrapperProxyCalls::Wrapper(decoded));
            }
            if let Ok(decoded) =
                <WxhoprTokenCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWrapperProxyCalls::WxhoprToken(decoded));
            }
            if let Ok(decoded) =
                <XdaiMultisigCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWrapperProxyCalls::XdaiMultisig(decoded));
            }
            if let Ok(decoded) =
                <XhoprTokenCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWrapperProxyCalls::XhoprToken(decoded));
            }
            if let Ok(decoded) =
                <OnTokenTransferCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWrapperProxyCalls::OnTokenTransfer(decoded));
            }
            if let Ok(decoded) =
                <RecoverTokensCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWrapperProxyCalls::RecoverTokens(decoded));
            }
            if let Ok(decoded) =
                <TokensReceivedCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprWrapperProxyCalls::TokensReceived(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for HoprWrapperProxyCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                HoprWrapperProxyCalls::Erc1820Registry(element) => element.encode(),
                HoprWrapperProxyCalls::TokensRecipientInterfaceHash(element) => element.encode(),
                HoprWrapperProxyCalls::Wrapper(element) => element.encode(),
                HoprWrapperProxyCalls::WxhoprToken(element) => element.encode(),
                HoprWrapperProxyCalls::XdaiMultisig(element) => element.encode(),
                HoprWrapperProxyCalls::XhoprToken(element) => element.encode(),
                HoprWrapperProxyCalls::OnTokenTransfer(element) => element.encode(),
                HoprWrapperProxyCalls::RecoverTokens(element) => element.encode(),
                HoprWrapperProxyCalls::TokensReceived(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for HoprWrapperProxyCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                HoprWrapperProxyCalls::Erc1820Registry(element) => element.fmt(f),
                HoprWrapperProxyCalls::TokensRecipientInterfaceHash(element) => element.fmt(f),
                HoprWrapperProxyCalls::Wrapper(element) => element.fmt(f),
                HoprWrapperProxyCalls::WxhoprToken(element) => element.fmt(f),
                HoprWrapperProxyCalls::XdaiMultisig(element) => element.fmt(f),
                HoprWrapperProxyCalls::XhoprToken(element) => element.fmt(f),
                HoprWrapperProxyCalls::OnTokenTransfer(element) => element.fmt(f),
                HoprWrapperProxyCalls::RecoverTokens(element) => element.fmt(f),
                HoprWrapperProxyCalls::TokensReceived(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<Erc1820RegistryCall> for HoprWrapperProxyCalls {
        fn from(var: Erc1820RegistryCall) -> Self {
            HoprWrapperProxyCalls::Erc1820Registry(var)
        }
    }
    impl ::std::convert::From<TokensRecipientInterfaceHashCall> for HoprWrapperProxyCalls {
        fn from(var: TokensRecipientInterfaceHashCall) -> Self {
            HoprWrapperProxyCalls::TokensRecipientInterfaceHash(var)
        }
    }
    impl ::std::convert::From<WrapperCall> for HoprWrapperProxyCalls {
        fn from(var: WrapperCall) -> Self {
            HoprWrapperProxyCalls::Wrapper(var)
        }
    }
    impl ::std::convert::From<WxhoprTokenCall> for HoprWrapperProxyCalls {
        fn from(var: WxhoprTokenCall) -> Self {
            HoprWrapperProxyCalls::WxhoprToken(var)
        }
    }
    impl ::std::convert::From<XdaiMultisigCall> for HoprWrapperProxyCalls {
        fn from(var: XdaiMultisigCall) -> Self {
            HoprWrapperProxyCalls::XdaiMultisig(var)
        }
    }
    impl ::std::convert::From<XhoprTokenCall> for HoprWrapperProxyCalls {
        fn from(var: XhoprTokenCall) -> Self {
            HoprWrapperProxyCalls::XhoprToken(var)
        }
    }
    impl ::std::convert::From<OnTokenTransferCall> for HoprWrapperProxyCalls {
        fn from(var: OnTokenTransferCall) -> Self {
            HoprWrapperProxyCalls::OnTokenTransfer(var)
        }
    }
    impl ::std::convert::From<RecoverTokensCall> for HoprWrapperProxyCalls {
        fn from(var: RecoverTokensCall) -> Self {
            HoprWrapperProxyCalls::RecoverTokens(var)
        }
    }
    impl ::std::convert::From<TokensReceivedCall> for HoprWrapperProxyCalls {
        fn from(var: TokensReceivedCall) -> Self {
            HoprWrapperProxyCalls::TokensReceived(var)
        }
    }
    #[doc = "Container type for all return fields from the `ERC1820_REGISTRY` function with signature `ERC1820_REGISTRY()` and selector `[1, 62, 177, 119]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct Erc1820RegistryReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `TOKENS_RECIPIENT_INTERFACE_HASH` function with signature `TOKENS_RECIPIENT_INTERFACE_HASH()` and selector `[114, 88, 28, 192]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct TokensRecipientInterfaceHashReturn(pub [u8; 32]);
    #[doc = "Container type for all return fields from the `WRAPPER` function with signature `WRAPPER()` and selector `[225, 235, 19, 193]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct WrapperReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `WXHOPR_TOKEN` function with signature `WXHOPR_TOKEN()` and selector `[254, 38, 39, 123]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct WxhoprTokenReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `XDAI_MULTISIG` function with signature `XDAI_MULTISIG()` and selector `[226, 143, 86, 243]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct XdaiMultisigReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `XHOPR_TOKEN` function with signature `XHOPR_TOKEN()` and selector `[184, 97, 250, 158]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct XhoprTokenReturn(pub ethers::core::types::Address);
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
}
