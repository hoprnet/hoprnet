pub use hopr_multi_sig::*;
/// This module was auto-generated with ethers-rs Abigen.
/// More information at: <https://github.com/gakonst/ethers-rs>
#[allow(
    clippy::enum_variant_names,
    clippy::too_many_arguments,
    clippy::upper_case_acronyms,
    clippy::type_complexity,
    dead_code,
    non_camel_case_types,
)]
pub mod hopr_multi_sig {
    #[rustfmt::skip]
    const __ABI: &str = "[{\"inputs\":[],\"type\":\"error\",\"name\":\"AlreadyInitialized\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"ContractNotResponsible\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InvalidSafeAddress\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"MultiSigUninitialized\",\"outputs\":[]}]";
    ///The parsed JSON ABI of the contract.
    pub static HOPRMULTISIG_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> = ::ethers::contract::Lazy::new(||
    ::ethers::core::utils::__serde_json::from_str(__ABI).expect("ABI is always valid"));
    pub struct HoprMultiSig<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for HoprMultiSig<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for HoprMultiSig<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for HoprMultiSig<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for HoprMultiSig<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(stringify!(HoprMultiSig)).field(&self.address()).finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> HoprMultiSig<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    HOPRMULTISIG_ABI.clone(),
                    client,
                ),
            )
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for HoprMultiSig<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Custom Error type `AlreadyInitialized` with signature `AlreadyInitialized()` and selector `0x0dc149f0`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "AlreadyInitialized", abi = "AlreadyInitialized()")]
    pub struct AlreadyInitialized;
    ///Custom Error type `ContractNotResponsible` with signature `ContractNotResponsible()` and selector `0xacd5a823`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "ContractNotResponsible", abi = "ContractNotResponsible()")]
    pub struct ContractNotResponsible;
    ///Custom Error type `InvalidSafeAddress` with signature `InvalidSafeAddress()` and selector `0x8e9d7c5e`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "InvalidSafeAddress", abi = "InvalidSafeAddress()")]
    pub struct InvalidSafeAddress;
    ///Custom Error type `MultiSigUninitialized` with signature `MultiSigUninitialized()` and selector `0x454a20c8`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "MultiSigUninitialized", abi = "MultiSigUninitialized()")]
    pub struct MultiSigUninitialized;
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum HoprMultiSigErrors {
        AlreadyInitialized(AlreadyInitialized),
        ContractNotResponsible(ContractNotResponsible),
        InvalidSafeAddress(InvalidSafeAddress),
        MultiSigUninitialized(MultiSigUninitialized),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for HoprMultiSigErrors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded)
                = <::std::string::String as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::RevertString(decoded));
            }
            if let Ok(decoded)
                = <AlreadyInitialized as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::AlreadyInitialized(decoded));
            }
            if let Ok(decoded)
                = <ContractNotResponsible as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::ContractNotResponsible(decoded));
            }
            if let Ok(decoded)
                = <InvalidSafeAddress as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::InvalidSafeAddress(decoded));
            }
            if let Ok(decoded)
                = <MultiSigUninitialized as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::MultiSigUninitialized(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for HoprMultiSigErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::AlreadyInitialized(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ContractNotResponsible(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidSafeAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MultiSigUninitialized(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for HoprMultiSigErrors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector
                    == <AlreadyInitialized as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <ContractNotResponsible as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidSafeAddress as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <MultiSigUninitialized as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for HoprMultiSigErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AlreadyInitialized(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ContractNotResponsible(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidSafeAddress(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::MultiSigUninitialized(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for HoprMultiSigErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<AlreadyInitialized> for HoprMultiSigErrors {
        fn from(value: AlreadyInitialized) -> Self {
            Self::AlreadyInitialized(value)
        }
    }
    impl ::core::convert::From<ContractNotResponsible> for HoprMultiSigErrors {
        fn from(value: ContractNotResponsible) -> Self {
            Self::ContractNotResponsible(value)
        }
    }
    impl ::core::convert::From<InvalidSafeAddress> for HoprMultiSigErrors {
        fn from(value: InvalidSafeAddress) -> Self {
            Self::InvalidSafeAddress(value)
        }
    }
    impl ::core::convert::From<MultiSigUninitialized> for HoprMultiSigErrors {
        fn from(value: MultiSigUninitialized) -> Self {
            Self::MultiSigUninitialized(value)
        }
    }
}
