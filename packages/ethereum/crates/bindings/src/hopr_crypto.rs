pub use hopr_crypto::*;
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
pub mod hopr_crypto {
    #[rustfmt::skip]
    const __ABI: &str = "[{\"inputs\":[],\"type\":\"error\",\"name\":\"InvalidCurvePoint\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InvalidFieldElement\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InvalidPointWitness\",\"outputs\":[]}]";
    ///The parsed JSON ABI of the contract.
    pub static HOPRCRYPTO_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> = ::ethers::contract::Lazy::new(||
    ::ethers::core::utils::__serde_json::from_str(__ABI).expect("ABI is always valid"));
    pub struct HoprCrypto<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for HoprCrypto<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for HoprCrypto<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for HoprCrypto<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for HoprCrypto<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(stringify!(HoprCrypto)).field(&self.address()).finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> HoprCrypto<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    HOPRCRYPTO_ABI.clone(),
                    client,
                ),
            )
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for HoprCrypto<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Custom Error type `InvalidCurvePoint` with signature `InvalidCurvePoint()` and selector `0x72454a82`
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
    #[etherror(name = "InvalidCurvePoint", abi = "InvalidCurvePoint()")]
    pub struct InvalidCurvePoint;
    ///Custom Error type `InvalidFieldElement` with signature `InvalidFieldElement()` and selector `0x3ae4ed6b`
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
    #[etherror(name = "InvalidFieldElement", abi = "InvalidFieldElement()")]
    pub struct InvalidFieldElement;
    ///Custom Error type `InvalidPointWitness` with signature `InvalidPointWitness()` and selector `0xedfdcd98`
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
    #[etherror(name = "InvalidPointWitness", abi = "InvalidPointWitness()")]
    pub struct InvalidPointWitness;
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum HoprCryptoErrors {
        InvalidCurvePoint(InvalidCurvePoint),
        InvalidFieldElement(InvalidFieldElement),
        InvalidPointWitness(InvalidPointWitness),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for HoprCryptoErrors {
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
                = <InvalidCurvePoint as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::InvalidCurvePoint(decoded));
            }
            if let Ok(decoded)
                = <InvalidFieldElement as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::InvalidFieldElement(decoded));
            }
            if let Ok(decoded)
                = <InvalidPointWitness as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::InvalidPointWitness(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for HoprCryptoErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::InvalidCurvePoint(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidFieldElement(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidPointWitness(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for HoprCryptoErrors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector
                    == <InvalidCurvePoint as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidFieldElement as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidPointWitness as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for HoprCryptoErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::InvalidCurvePoint(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidFieldElement(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidPointWitness(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for HoprCryptoErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<InvalidCurvePoint> for HoprCryptoErrors {
        fn from(value: InvalidCurvePoint) -> Self {
            Self::InvalidCurvePoint(value)
        }
    }
    impl ::core::convert::From<InvalidFieldElement> for HoprCryptoErrors {
        fn from(value: InvalidFieldElement) -> Self {
            Self::InvalidFieldElement(value)
        }
    }
    impl ::core::convert::From<InvalidPointWitness> for HoprCryptoErrors {
        fn from(value: InvalidPointWitness) -> Self {
            Self::InvalidPointWitness(value)
        }
    }
}
