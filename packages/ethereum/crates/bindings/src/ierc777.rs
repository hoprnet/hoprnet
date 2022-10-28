pub use ierc777::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod ierc777 {
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
    #[doc = "IERC777 was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"tokenHolder\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"AuthorizedOperator\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Burned\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Minted\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"tokenHolder\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"RevokedOperator\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Sent\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"authorizeOperator\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"owner\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"balanceOf\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"burn\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"defaultOperators\",\"outputs\":[{\"internalType\":\"address[]\",\"name\":\"\",\"type\":\"address[]\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"granularity\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"tokenHolder\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"isOperatorFor\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"name\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"operatorBurn\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"recipient\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"operatorData\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"operatorSend\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"revokeOperator\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"recipient\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"send\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"symbol\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"totalSupply\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static IERC777_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    pub struct IERC777<M>(ethers::contract::Contract<M>);
    impl<M> Clone for IERC777<M> {
        fn clone(&self) -> Self {
            IERC777(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for IERC777<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for IERC777<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(IERC777))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> IERC777<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), IERC777_ABI.clone(), client).into()
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
            owner: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([112, 160, 130, 49], owner)
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
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(&self) -> ethers::contract::builders::Event<M, IERC777Events> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for IERC777<M> {
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
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum IERC777Events {
        AuthorizedOperatorFilter(AuthorizedOperatorFilter),
        BurnedFilter(BurnedFilter),
        MintedFilter(MintedFilter),
        RevokedOperatorFilter(RevokedOperatorFilter),
        SentFilter(SentFilter),
    }
    impl ethers::contract::EthLogDecode for IERC777Events {
        fn decode_log(
            log: &ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = AuthorizedOperatorFilter::decode_log(log) {
                return Ok(IERC777Events::AuthorizedOperatorFilter(decoded));
            }
            if let Ok(decoded) = BurnedFilter::decode_log(log) {
                return Ok(IERC777Events::BurnedFilter(decoded));
            }
            if let Ok(decoded) = MintedFilter::decode_log(log) {
                return Ok(IERC777Events::MintedFilter(decoded));
            }
            if let Ok(decoded) = RevokedOperatorFilter::decode_log(log) {
                return Ok(IERC777Events::RevokedOperatorFilter(decoded));
            }
            if let Ok(decoded) = SentFilter::decode_log(log) {
                return Ok(IERC777Events::SentFilter(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for IERC777Events {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                IERC777Events::AuthorizedOperatorFilter(element) => element.fmt(f),
                IERC777Events::BurnedFilter(element) => element.fmt(f),
                IERC777Events::MintedFilter(element) => element.fmt(f),
                IERC777Events::RevokedOperatorFilter(element) => element.fmt(f),
                IERC777Events::SentFilter(element) => element.fmt(f),
            }
        }
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
        pub owner: ethers::core::types::Address,
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
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum IERC777Calls {
        AuthorizeOperator(AuthorizeOperatorCall),
        BalanceOf(BalanceOfCall),
        Burn(BurnCall),
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
    }
    impl ethers::core::abi::AbiDecode for IERC777Calls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <AuthorizeOperatorCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IERC777Calls::AuthorizeOperator(decoded));
            }
            if let Ok(decoded) =
                <BalanceOfCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IERC777Calls::BalanceOf(decoded));
            }
            if let Ok(decoded) = <BurnCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(IERC777Calls::Burn(decoded));
            }
            if let Ok(decoded) =
                <DefaultOperatorsCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IERC777Calls::DefaultOperators(decoded));
            }
            if let Ok(decoded) =
                <GranularityCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IERC777Calls::Granularity(decoded));
            }
            if let Ok(decoded) =
                <IsOperatorForCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IERC777Calls::IsOperatorFor(decoded));
            }
            if let Ok(decoded) = <NameCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(IERC777Calls::Name(decoded));
            }
            if let Ok(decoded) =
                <OperatorBurnCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IERC777Calls::OperatorBurn(decoded));
            }
            if let Ok(decoded) =
                <OperatorSendCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IERC777Calls::OperatorSend(decoded));
            }
            if let Ok(decoded) =
                <RevokeOperatorCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IERC777Calls::RevokeOperator(decoded));
            }
            if let Ok(decoded) = <SendCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(IERC777Calls::Send(decoded));
            }
            if let Ok(decoded) = <SymbolCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IERC777Calls::Symbol(decoded));
            }
            if let Ok(decoded) =
                <TotalSupplyCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(IERC777Calls::TotalSupply(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for IERC777Calls {
        fn encode(self) -> Vec<u8> {
            match self {
                IERC777Calls::AuthorizeOperator(element) => element.encode(),
                IERC777Calls::BalanceOf(element) => element.encode(),
                IERC777Calls::Burn(element) => element.encode(),
                IERC777Calls::DefaultOperators(element) => element.encode(),
                IERC777Calls::Granularity(element) => element.encode(),
                IERC777Calls::IsOperatorFor(element) => element.encode(),
                IERC777Calls::Name(element) => element.encode(),
                IERC777Calls::OperatorBurn(element) => element.encode(),
                IERC777Calls::OperatorSend(element) => element.encode(),
                IERC777Calls::RevokeOperator(element) => element.encode(),
                IERC777Calls::Send(element) => element.encode(),
                IERC777Calls::Symbol(element) => element.encode(),
                IERC777Calls::TotalSupply(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for IERC777Calls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                IERC777Calls::AuthorizeOperator(element) => element.fmt(f),
                IERC777Calls::BalanceOf(element) => element.fmt(f),
                IERC777Calls::Burn(element) => element.fmt(f),
                IERC777Calls::DefaultOperators(element) => element.fmt(f),
                IERC777Calls::Granularity(element) => element.fmt(f),
                IERC777Calls::IsOperatorFor(element) => element.fmt(f),
                IERC777Calls::Name(element) => element.fmt(f),
                IERC777Calls::OperatorBurn(element) => element.fmt(f),
                IERC777Calls::OperatorSend(element) => element.fmt(f),
                IERC777Calls::RevokeOperator(element) => element.fmt(f),
                IERC777Calls::Send(element) => element.fmt(f),
                IERC777Calls::Symbol(element) => element.fmt(f),
                IERC777Calls::TotalSupply(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<AuthorizeOperatorCall> for IERC777Calls {
        fn from(var: AuthorizeOperatorCall) -> Self {
            IERC777Calls::AuthorizeOperator(var)
        }
    }
    impl ::std::convert::From<BalanceOfCall> for IERC777Calls {
        fn from(var: BalanceOfCall) -> Self {
            IERC777Calls::BalanceOf(var)
        }
    }
    impl ::std::convert::From<BurnCall> for IERC777Calls {
        fn from(var: BurnCall) -> Self {
            IERC777Calls::Burn(var)
        }
    }
    impl ::std::convert::From<DefaultOperatorsCall> for IERC777Calls {
        fn from(var: DefaultOperatorsCall) -> Self {
            IERC777Calls::DefaultOperators(var)
        }
    }
    impl ::std::convert::From<GranularityCall> for IERC777Calls {
        fn from(var: GranularityCall) -> Self {
            IERC777Calls::Granularity(var)
        }
    }
    impl ::std::convert::From<IsOperatorForCall> for IERC777Calls {
        fn from(var: IsOperatorForCall) -> Self {
            IERC777Calls::IsOperatorFor(var)
        }
    }
    impl ::std::convert::From<NameCall> for IERC777Calls {
        fn from(var: NameCall) -> Self {
            IERC777Calls::Name(var)
        }
    }
    impl ::std::convert::From<OperatorBurnCall> for IERC777Calls {
        fn from(var: OperatorBurnCall) -> Self {
            IERC777Calls::OperatorBurn(var)
        }
    }
    impl ::std::convert::From<OperatorSendCall> for IERC777Calls {
        fn from(var: OperatorSendCall) -> Self {
            IERC777Calls::OperatorSend(var)
        }
    }
    impl ::std::convert::From<RevokeOperatorCall> for IERC777Calls {
        fn from(var: RevokeOperatorCall) -> Self {
            IERC777Calls::RevokeOperator(var)
        }
    }
    impl ::std::convert::From<SendCall> for IERC777Calls {
        fn from(var: SendCall) -> Self {
            IERC777Calls::Send(var)
        }
    }
    impl ::std::convert::From<SymbolCall> for IERC777Calls {
        fn from(var: SymbolCall) -> Self {
            IERC777Calls::Symbol(var)
        }
    }
    impl ::std::convert::From<TotalSupplyCall> for IERC777Calls {
        fn from(var: TotalSupplyCall) -> Self {
            IERC777Calls::TotalSupply(var)
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
}
