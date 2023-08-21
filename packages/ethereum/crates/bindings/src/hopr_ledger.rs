pub use hopr_ledger::*;
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
pub mod hopr_ledger {
    #[rustfmt::skip]
    const __ABI: &str = "[{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"ledgerDomainSeparator\",\"type\":\"bytes32\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"LedgerDomainSeparatorUpdated\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"LEDGER_VERSION\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"ledgerDomainSeparator\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"updateLedgerDomainSeparator\",\"outputs\":[]}]";
    ///The parsed JSON ABI of the contract.
    pub static HOPRLEDGER_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> = ::ethers::contract::Lazy::new(||
    ::ethers::core::utils::__serde_json::from_str(__ABI).expect("ABI is always valid"));
    pub struct HoprLedger<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for HoprLedger<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for HoprLedger<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for HoprLedger<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for HoprLedger<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(stringify!(HoprLedger)).field(&self.address()).finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> HoprLedger<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    HOPRLEDGER_ABI.clone(),
                    client,
                ),
            )
        }
        ///Calls the contract's `LEDGER_VERSION` (0xddad1902) function
        pub fn ledger_version(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([221, 173, 25, 2], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `ledgerDomainSeparator` (0xc966c4fe) function
        pub fn ledger_domain_separator(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([201, 102, 196, 254], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `updateLedgerDomainSeparator` (0xdc96fd50) function
        pub fn update_ledger_domain_separator(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([220, 150, 253, 80], ())
                .expect("method not found (this should never happen)")
        }
        ///Gets the contract's `LedgerDomainSeparatorUpdated` event
        pub fn ledger_domain_separator_updated_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            LedgerDomainSeparatorUpdatedFilter,
        > {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            LedgerDomainSeparatorUpdatedFilter,
        > {
            self.0.event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for HoprLedger<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "LedgerDomainSeparatorUpdated",
        abi = "LedgerDomainSeparatorUpdated(bytes32)"
    )]
    pub struct LedgerDomainSeparatorUpdatedFilter {
        #[ethevent(indexed)]
        pub ledger_domain_separator: [u8; 32],
    }
    ///Container type for all input parameters for the `LEDGER_VERSION` function with signature `LEDGER_VERSION()` and selector `0xddad1902`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "LEDGER_VERSION", abi = "LEDGER_VERSION()")]
    pub struct LedgerVersionCall;
    ///Container type for all input parameters for the `ledgerDomainSeparator` function with signature `ledgerDomainSeparator()` and selector `0xc966c4fe`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "ledgerDomainSeparator", abi = "ledgerDomainSeparator()")]
    pub struct LedgerDomainSeparatorCall;
    ///Container type for all input parameters for the `updateLedgerDomainSeparator` function with signature `updateLedgerDomainSeparator()` and selector `0xdc96fd50`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "updateLedgerDomainSeparator",
        abi = "updateLedgerDomainSeparator()"
    )]
    pub struct UpdateLedgerDomainSeparatorCall;
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum HoprLedgerCalls {
        LedgerVersion(LedgerVersionCall),
        LedgerDomainSeparator(LedgerDomainSeparatorCall),
        UpdateLedgerDomainSeparator(UpdateLedgerDomainSeparatorCall),
    }
    impl ::ethers::core::abi::AbiDecode for HoprLedgerCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded)
                = <LedgerVersionCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::LedgerVersion(decoded));
            }
            if let Ok(decoded)
                = <LedgerDomainSeparatorCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::LedgerDomainSeparator(decoded));
            }
            if let Ok(decoded)
                = <UpdateLedgerDomainSeparatorCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::UpdateLedgerDomainSeparator(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for HoprLedgerCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::LedgerVersion(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::LedgerDomainSeparator(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::UpdateLedgerDomainSeparator(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for HoprLedgerCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::LedgerVersion(element) => ::core::fmt::Display::fmt(element, f),
                Self::LedgerDomainSeparator(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::UpdateLedgerDomainSeparator(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<LedgerVersionCall> for HoprLedgerCalls {
        fn from(value: LedgerVersionCall) -> Self {
            Self::LedgerVersion(value)
        }
    }
    impl ::core::convert::From<LedgerDomainSeparatorCall> for HoprLedgerCalls {
        fn from(value: LedgerDomainSeparatorCall) -> Self {
            Self::LedgerDomainSeparator(value)
        }
    }
    impl ::core::convert::From<UpdateLedgerDomainSeparatorCall> for HoprLedgerCalls {
        fn from(value: UpdateLedgerDomainSeparatorCall) -> Self {
            Self::UpdateLedgerDomainSeparator(value)
        }
    }
    ///Container type for all return fields from the `LEDGER_VERSION` function with signature `LEDGER_VERSION()` and selector `0xddad1902`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct LedgerVersionReturn(pub ::std::string::String);
    ///Container type for all return fields from the `ledgerDomainSeparator` function with signature `ledgerDomainSeparator()` and selector `0xc966c4fe`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct LedgerDomainSeparatorReturn(pub [u8; 32]);
}
