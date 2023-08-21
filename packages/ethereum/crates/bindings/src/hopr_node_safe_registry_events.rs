pub use hopr_node_safe_registry_events::*;
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
pub mod hopr_node_safe_registry_events {
    #[rustfmt::skip]
    const __ABI: &str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"safeAddress\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"nodeAddress\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"DergisteredNodeSafe\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"domainSeparator\",\"type\":\"bytes32\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"DomainSeparatorUpdated\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"safeAddress\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"nodeAddress\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"RegisteredNodeSafe\",\"outputs\":[],\"anonymous\":false}]";
    ///The parsed JSON ABI of the contract.
    pub static HOPRNODESAFEREGISTRYEVENTS_ABI: ::ethers::contract::Lazy<
        ::ethers::core::abi::Abi,
    > = ::ethers::contract::Lazy::new(|| {
        ::ethers::core::utils::__serde_json::from_str(__ABI)
            .expect("ABI is always valid")
    });
    pub struct HoprNodeSafeRegistryEvents<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for HoprNodeSafeRegistryEvents<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for HoprNodeSafeRegistryEvents<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for HoprNodeSafeRegistryEvents<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for HoprNodeSafeRegistryEvents<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(stringify!(HoprNodeSafeRegistryEvents))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> HoprNodeSafeRegistryEvents<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    HOPRNODESAFEREGISTRYEVENTS_ABI.clone(),
                    client,
                ),
            )
        }
        ///Gets the contract's `DergisteredNodeSafe` event
        pub fn dergistered_node_safe_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            DergisteredNodeSafeFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `DomainSeparatorUpdated` event
        pub fn domain_separator_updated_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            DomainSeparatorUpdatedFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `RegisteredNodeSafe` event
        pub fn registered_node_safe_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            RegisteredNodeSafeFilter,
        > {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            HoprNodeSafeRegistryEventsEvents,
        > {
            self.0.event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for HoprNodeSafeRegistryEvents<M> {
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
        name = "DergisteredNodeSafe",
        abi = "DergisteredNodeSafe(address,address)"
    )]
    pub struct DergisteredNodeSafeFilter {
        #[ethevent(indexed)]
        pub safe_address: ::ethers::core::types::Address,
        #[ethevent(indexed)]
        pub node_address: ::ethers::core::types::Address,
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
    #[ethevent(name = "DomainSeparatorUpdated", abi = "DomainSeparatorUpdated(bytes32)")]
    pub struct DomainSeparatorUpdatedFilter {
        #[ethevent(indexed)]
        pub domain_separator: [u8; 32],
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
    #[ethevent(name = "RegisteredNodeSafe", abi = "RegisteredNodeSafe(address,address)")]
    pub struct RegisteredNodeSafeFilter {
        #[ethevent(indexed)]
        pub safe_address: ::ethers::core::types::Address,
        #[ethevent(indexed)]
        pub node_address: ::ethers::core::types::Address,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum HoprNodeSafeRegistryEventsEvents {
        DergisteredNodeSafeFilter(DergisteredNodeSafeFilter),
        DomainSeparatorUpdatedFilter(DomainSeparatorUpdatedFilter),
        RegisteredNodeSafeFilter(RegisteredNodeSafeFilter),
    }
    impl ::ethers::contract::EthLogDecode for HoprNodeSafeRegistryEventsEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = DergisteredNodeSafeFilter::decode_log(log) {
                return Ok(
                    HoprNodeSafeRegistryEventsEvents::DergisteredNodeSafeFilter(decoded),
                );
            }
            if let Ok(decoded) = DomainSeparatorUpdatedFilter::decode_log(log) {
                return Ok(
                    HoprNodeSafeRegistryEventsEvents::DomainSeparatorUpdatedFilter(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded) = RegisteredNodeSafeFilter::decode_log(log) {
                return Ok(
                    HoprNodeSafeRegistryEventsEvents::RegisteredNodeSafeFilter(decoded),
                );
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for HoprNodeSafeRegistryEventsEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::DergisteredNodeSafeFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::DomainSeparatorUpdatedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RegisteredNodeSafeFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<DergisteredNodeSafeFilter>
    for HoprNodeSafeRegistryEventsEvents {
        fn from(value: DergisteredNodeSafeFilter) -> Self {
            Self::DergisteredNodeSafeFilter(value)
        }
    }
    impl ::core::convert::From<DomainSeparatorUpdatedFilter>
    for HoprNodeSafeRegistryEventsEvents {
        fn from(value: DomainSeparatorUpdatedFilter) -> Self {
            Self::DomainSeparatorUpdatedFilter(value)
        }
    }
    impl ::core::convert::From<RegisteredNodeSafeFilter>
    for HoprNodeSafeRegistryEventsEvents {
        fn from(value: RegisteredNodeSafeFilter) -> Self {
            Self::RegisteredNodeSafeFilter(value)
        }
    }
}
