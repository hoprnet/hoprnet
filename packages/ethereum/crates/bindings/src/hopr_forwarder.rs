pub use hopr_forwarder::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod hopr_forwarder {
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
    #[doc = "HoprForwarder was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"ERC1820_REGISTRY\",\"outputs\":[{\"internalType\":\"contract IERC1820Registry\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"HOPR_TOKEN\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"MULTISIG\",\"outputs\":[{\"internalType\":\"address payable\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"TOKENS_RECIPIENT_INTERFACE_HASH\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"token\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"recoverTokens\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"userData\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"tokensReceived\",\"outputs\":[]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static HOPRFORWARDER_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    #[doc = r" Bytecode of the #name contract"]
    pub static HOPRFORWARDER_BYTECODE: ethers::contract::Lazy<ethers::core::types::Bytes> =
        ethers::contract::Lazy::new(|| {
            "0x608060405234801561001057600080fd5b506040516329965a1d60e01b815230600482018190527fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b60248301526044820152731820a4b7618bde71dce8cdc73aab6c95905fad24906329965a1d90606401600060405180830381600087803b15801561008a57600080fd5b505af115801561009e573d6000803e3d6000fd5b5050505061086f806100b16000396000f3fe608060405234801561001057600080fd5b50600436106100615760003560e01c806223de2914610066578063013eb1771461007b57806316114acd146100b35780631ba6bac2146100c65780632530b145146100e157806372581cc0146100fc575b600080fd5b6100796100743660046106b9565b610131565b005b610096731820a4b7618bde71dce8cdc73aab6c95905fad2481565b6040516001600160a01b0390911681526020015b60405180910390f35b6100796100c1366004610764565b6102c9565b61009673f5581dfefd8fb0e4aec526be659cfab1f8c781da81565b610096734f50ab4e931289344a57f2fe4bbd10546a6fdc1781565b6101237fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b81565b6040519081526020016100aa565b3373f5581dfefd8fb0e4aec526be659cfab1f8c781da146101ab5760405162461bcd60e51b815260206004820152602960248201527f486f70724d696e746572577261707065723a204f6e6c7920616363657074204860448201526827a829103a37b5b2b760b91b60648201526084015b60405180910390fd5b6001600160a01b0387161561020e5760405162461bcd60e51b8152602060048201526024808201527f486f70724d696e746572577261707065723a204f6e6c792072656365697665206044820152631b5a5b9d60e21b60648201526084016101a2565b6001600160a01b038616301461028c5760405162461bcd60e51b815260206004820152603f60248201527f486f70724d696e746572577261707065723a204d7573742062652073656e646960448201527f6e6720746f6b656e7320746f20746865206d696e74657220777261707065720060648201526084016101a2565b6102bf73f5581dfefd8fb0e4aec526be659cfab1f8c781da734f50ab4e931289344a57f2fe4bbd10546a6fdc17876103b1565b5050505050505050565b6001600160a01b03811661031b57604051734f50ab4e931289344a57f2fe4bbd10546a6fdc17904780156108fc02916000818181858888f19350505050158015610317573d6000803e3d6000fd5b5050565b6040516370a0823160e01b81523060048201526103ae90734f50ab4e931289344a57f2fe4bbd10546a6fdc17906001600160a01b038416906370a0823190602401602060405180830381865afa158015610379573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061039d919061077f565b6001600160a01b03841691906103b1565b50565b604080516001600160a01b038416602482015260448082018490528251808303909101815260649091019091526020810180516001600160e01b031663a9059cbb60e01b179052610403908490610408565b505050565b600061045d826040518060400160405280602081526020017f5361666545524332303a206c6f772d6c6576656c2063616c6c206661696c6564815250856001600160a01b03166104da9092919063ffffffff16565b805190915015610403578080602001905181019061047b9190610798565b6104035760405162461bcd60e51b815260206004820152602a60248201527f5361666545524332303a204552433230206f7065726174696f6e20646964206e6044820152691bdd081cdd58d8d9595960b21b60648201526084016101a2565b60606104e984846000856104f3565b90505b9392505050565b6060824710156105545760405162461bcd60e51b815260206004820152602660248201527f416464726573733a20696e73756666696369656e742062616c616e636520666f6044820152651c8818d85b1b60d21b60648201526084016101a2565b843b6105a25760405162461bcd60e51b815260206004820152601d60248201527f416464726573733a2063616c6c20746f206e6f6e2d636f6e747261637400000060448201526064016101a2565b600080866001600160a01b031685876040516105be91906107ea565b60006040518083038185875af1925050503d80600081146105fb576040519150601f19603f3d011682016040523d82523d6000602084013e610600565b606091505b509150915061061082828661061b565b979650505050505050565b6060831561062a5750816104ec565b82511561063a5782518084602001fd5b8160405162461bcd60e51b81526004016101a29190610806565b80356001600160a01b038116811461066b57600080fd5b919050565b60008083601f84011261068257600080fd5b50813567ffffffffffffffff81111561069a57600080fd5b6020830191508360208285010111156106b257600080fd5b9250929050565b60008060008060008060008060c0898b0312156106d557600080fd5b6106de89610654565b97506106ec60208a01610654565b96506106fa60408a01610654565b955060608901359450608089013567ffffffffffffffff8082111561071e57600080fd5b61072a8c838d01610670565b909650945060a08b013591508082111561074357600080fd5b506107508b828c01610670565b999c989b5096995094979396929594505050565b60006020828403121561077657600080fd5b6104ec82610654565b60006020828403121561079157600080fd5b5051919050565b6000602082840312156107aa57600080fd5b815180151581146104ec57600080fd5b60005b838110156107d55781810151838201526020016107bd565b838111156107e4576000848401525b50505050565b600082516107fc8184602087016107ba565b9190910192915050565b60208152600082518060208401526108258160408501602087016107ba565b601f01601f1916919091016040019291505056fea2646970667358221220d428eef65d3e8b70cc9ed3e4f7b3ad1af4c8c4d7d2fcfe2887b773cebedc83f664736f6c634300080d0033" . parse () . expect ("invalid bytecode")
        });
    pub struct HoprForwarder<M>(ethers::contract::Contract<M>);
    impl<M> Clone for HoprForwarder<M> {
        fn clone(&self) -> Self {
            HoprForwarder(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for HoprForwarder<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for HoprForwarder<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(HoprForwarder))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> HoprForwarder<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), HOPRFORWARDER_ABI.clone(), client)
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
                HOPRFORWARDER_ABI.clone(),
                HOPRFORWARDER_BYTECODE.clone().into(),
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
        #[doc = "Calls the contract's `HOPR_TOKEN` (0x1ba6bac2) function"]
        pub fn hopr_token(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([27, 166, 186, 194], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `MULTISIG` (0x2530b145) function"]
        pub fn multisig(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([37, 48, 177, 69], ())
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
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for HoprForwarder<M> {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
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
    #[doc = "Container type for all input parameters for the `HOPR_TOKEN` function with signature `HOPR_TOKEN()` and selector `[27, 166, 186, 194]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "HOPR_TOKEN", abi = "HOPR_TOKEN()")]
    pub struct HoprTokenCall;
    #[doc = "Container type for all input parameters for the `MULTISIG` function with signature `MULTISIG()` and selector `[37, 48, 177, 69]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "MULTISIG", abi = "MULTISIG()")]
    pub struct MultisigCall;
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
    pub enum HoprForwarderCalls {
        Erc1820Registry(Erc1820RegistryCall),
        HoprToken(HoprTokenCall),
        Multisig(MultisigCall),
        TokensRecipientInterfaceHash(TokensRecipientInterfaceHashCall),
        RecoverTokens(RecoverTokensCall),
        TokensReceived(TokensReceivedCall),
    }
    impl ethers::core::abi::AbiDecode for HoprForwarderCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <Erc1820RegistryCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprForwarderCalls::Erc1820Registry(decoded));
            }
            if let Ok(decoded) =
                <HoprTokenCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprForwarderCalls::HoprToken(decoded));
            }
            if let Ok(decoded) =
                <MultisigCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprForwarderCalls::Multisig(decoded));
            }
            if let Ok(decoded) =
                <TokensRecipientInterfaceHashCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(HoprForwarderCalls::TokensRecipientInterfaceHash(decoded));
            }
            if let Ok(decoded) =
                <RecoverTokensCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprForwarderCalls::RecoverTokens(decoded));
            }
            if let Ok(decoded) =
                <TokensReceivedCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(HoprForwarderCalls::TokensReceived(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for HoprForwarderCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                HoprForwarderCalls::Erc1820Registry(element) => element.encode(),
                HoprForwarderCalls::HoprToken(element) => element.encode(),
                HoprForwarderCalls::Multisig(element) => element.encode(),
                HoprForwarderCalls::TokensRecipientInterfaceHash(element) => element.encode(),
                HoprForwarderCalls::RecoverTokens(element) => element.encode(),
                HoprForwarderCalls::TokensReceived(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for HoprForwarderCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                HoprForwarderCalls::Erc1820Registry(element) => element.fmt(f),
                HoprForwarderCalls::HoprToken(element) => element.fmt(f),
                HoprForwarderCalls::Multisig(element) => element.fmt(f),
                HoprForwarderCalls::TokensRecipientInterfaceHash(element) => element.fmt(f),
                HoprForwarderCalls::RecoverTokens(element) => element.fmt(f),
                HoprForwarderCalls::TokensReceived(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<Erc1820RegistryCall> for HoprForwarderCalls {
        fn from(var: Erc1820RegistryCall) -> Self {
            HoprForwarderCalls::Erc1820Registry(var)
        }
    }
    impl ::std::convert::From<HoprTokenCall> for HoprForwarderCalls {
        fn from(var: HoprTokenCall) -> Self {
            HoprForwarderCalls::HoprToken(var)
        }
    }
    impl ::std::convert::From<MultisigCall> for HoprForwarderCalls {
        fn from(var: MultisigCall) -> Self {
            HoprForwarderCalls::Multisig(var)
        }
    }
    impl ::std::convert::From<TokensRecipientInterfaceHashCall> for HoprForwarderCalls {
        fn from(var: TokensRecipientInterfaceHashCall) -> Self {
            HoprForwarderCalls::TokensRecipientInterfaceHash(var)
        }
    }
    impl ::std::convert::From<RecoverTokensCall> for HoprForwarderCalls {
        fn from(var: RecoverTokensCall) -> Self {
            HoprForwarderCalls::RecoverTokens(var)
        }
    }
    impl ::std::convert::From<TokensReceivedCall> for HoprForwarderCalls {
        fn from(var: TokensReceivedCall) -> Self {
            HoprForwarderCalls::TokensReceived(var)
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
    #[doc = "Container type for all return fields from the `HOPR_TOKEN` function with signature `HOPR_TOKEN()` and selector `[27, 166, 186, 194]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct HoprTokenReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `MULTISIG` function with signature `MULTISIG()` and selector `[37, 48, 177, 69]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct MultisigReturn(pub ethers::core::types::Address);
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
}
