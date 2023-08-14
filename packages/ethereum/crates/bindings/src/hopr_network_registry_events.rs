pub use hopr_network_registry_events::*;
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
pub mod hopr_network_registry_events {
    #[rustfmt::skip]
    const __ABI: &str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"stakingAccount\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"nodeAddress\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"Deregistered\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"stakingAccount\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"nodeAddress\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"DeregisteredByManager\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"stakingAccount\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"bool\",\"name\":\"eligibility\",\"type\":\"bool\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"EligibilityUpdated\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"bool\",\"name\":\"isEnabled\",\"type\":\"bool\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"NetworkRegistryStatusUpdated\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"stakingAccount\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"nodeAddress\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"Registered\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"stakingAccount\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"nodeAddress\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"RegisteredByManager\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"requirementImplementation\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"RequirementUpdated\",\"outputs\":[],\"anonymous\":false}]";
    ///The parsed JSON ABI of the contract.
    pub static HOPRNETWORKREGISTRYEVENTS_ABI: ::ethers::contract::Lazy<
        ::ethers::core::abi::Abi,
    > = ::ethers::contract::Lazy::new(|| {
        ::ethers::core::utils::__serde_json::from_str(__ABI)
            .expect("ABI is always valid")
    });
    pub struct HoprNetworkRegistryEvents<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for HoprNetworkRegistryEvents<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for HoprNetworkRegistryEvents<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for HoprNetworkRegistryEvents<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for HoprNetworkRegistryEvents<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(stringify!(HoprNetworkRegistryEvents))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> HoprNetworkRegistryEvents<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    HOPRNETWORKREGISTRYEVENTS_ABI.clone(),
                    client,
                ),
            )
        }
        ///Gets the contract's `Deregistered` event
        pub fn deregistered_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            DeregisteredFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `DeregisteredByManager` event
        pub fn deregistered_by_manager_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            DeregisteredByManagerFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `EligibilityUpdated` event
        pub fn eligibility_updated_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            EligibilityUpdatedFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `NetworkRegistryStatusUpdated` event
        pub fn network_registry_status_updated_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            NetworkRegistryStatusUpdatedFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `Registered` event
        pub fn registered_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            RegisteredFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `RegisteredByManager` event
        pub fn registered_by_manager_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            RegisteredByManagerFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `RequirementUpdated` event
        pub fn requirement_updated_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            RequirementUpdatedFilter,
        > {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            HoprNetworkRegistryEventsEvents,
        > {
            self.0.event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for HoprNetworkRegistryEvents<M> {
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
    #[ethevent(name = "Deregistered", abi = "Deregistered(address,address)")]
    pub struct DeregisteredFilter {
        #[ethevent(indexed)]
        pub staking_account: ::ethers::core::types::Address,
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
    #[ethevent(
        name = "DeregisteredByManager",
        abi = "DeregisteredByManager(address,address)"
    )]
    pub struct DeregisteredByManagerFilter {
        #[ethevent(indexed)]
        pub staking_account: ::ethers::core::types::Address,
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
    #[ethevent(name = "EligibilityUpdated", abi = "EligibilityUpdated(address,bool)")]
    pub struct EligibilityUpdatedFilter {
        #[ethevent(indexed)]
        pub staking_account: ::ethers::core::types::Address,
        #[ethevent(indexed)]
        pub eligibility: bool,
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
        name = "NetworkRegistryStatusUpdated",
        abi = "NetworkRegistryStatusUpdated(bool)"
    )]
    pub struct NetworkRegistryStatusUpdatedFilter {
        #[ethevent(indexed)]
        pub is_enabled: bool,
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
    #[ethevent(name = "Registered", abi = "Registered(address,address)")]
    pub struct RegisteredFilter {
        #[ethevent(indexed)]
        pub staking_account: ::ethers::core::types::Address,
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
    #[ethevent(
        name = "RegisteredByManager",
        abi = "RegisteredByManager(address,address)"
    )]
    pub struct RegisteredByManagerFilter {
        #[ethevent(indexed)]
        pub staking_account: ::ethers::core::types::Address,
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
    #[ethevent(name = "RequirementUpdated", abi = "RequirementUpdated(address)")]
    pub struct RequirementUpdatedFilter {
        #[ethevent(indexed)]
        pub requirement_implementation: ::ethers::core::types::Address,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum HoprNetworkRegistryEventsEvents {
        DeregisteredFilter(DeregisteredFilter),
        DeregisteredByManagerFilter(DeregisteredByManagerFilter),
        EligibilityUpdatedFilter(EligibilityUpdatedFilter),
        NetworkRegistryStatusUpdatedFilter(NetworkRegistryStatusUpdatedFilter),
        RegisteredFilter(RegisteredFilter),
        RegisteredByManagerFilter(RegisteredByManagerFilter),
        RequirementUpdatedFilter(RequirementUpdatedFilter),
    }
    impl ::ethers::contract::EthLogDecode for HoprNetworkRegistryEventsEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = DeregisteredFilter::decode_log(log) {
                return Ok(HoprNetworkRegistryEventsEvents::DeregisteredFilter(decoded));
            }
            if let Ok(decoded) = DeregisteredByManagerFilter::decode_log(log) {
                return Ok(
                    HoprNetworkRegistryEventsEvents::DeregisteredByManagerFilter(decoded),
                );
            }
            if let Ok(decoded) = EligibilityUpdatedFilter::decode_log(log) {
                return Ok(
                    HoprNetworkRegistryEventsEvents::EligibilityUpdatedFilter(decoded),
                );
            }
            if let Ok(decoded) = NetworkRegistryStatusUpdatedFilter::decode_log(log) {
                return Ok(
                    HoprNetworkRegistryEventsEvents::NetworkRegistryStatusUpdatedFilter(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded) = RegisteredFilter::decode_log(log) {
                return Ok(HoprNetworkRegistryEventsEvents::RegisteredFilter(decoded));
            }
            if let Ok(decoded) = RegisteredByManagerFilter::decode_log(log) {
                return Ok(
                    HoprNetworkRegistryEventsEvents::RegisteredByManagerFilter(decoded),
                );
            }
            if let Ok(decoded) = RequirementUpdatedFilter::decode_log(log) {
                return Ok(
                    HoprNetworkRegistryEventsEvents::RequirementUpdatedFilter(decoded),
                );
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for HoprNetworkRegistryEventsEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::DeregisteredFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::DeregisteredByManagerFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::EligibilityUpdatedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NetworkRegistryStatusUpdatedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RegisteredFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::RegisteredByManagerFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RequirementUpdatedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<DeregisteredFilter> for HoprNetworkRegistryEventsEvents {
        fn from(value: DeregisteredFilter) -> Self {
            Self::DeregisteredFilter(value)
        }
    }
    impl ::core::convert::From<DeregisteredByManagerFilter>
    for HoprNetworkRegistryEventsEvents {
        fn from(value: DeregisteredByManagerFilter) -> Self {
            Self::DeregisteredByManagerFilter(value)
        }
    }
    impl ::core::convert::From<EligibilityUpdatedFilter>
    for HoprNetworkRegistryEventsEvents {
        fn from(value: EligibilityUpdatedFilter) -> Self {
            Self::EligibilityUpdatedFilter(value)
        }
    }
    impl ::core::convert::From<NetworkRegistryStatusUpdatedFilter>
    for HoprNetworkRegistryEventsEvents {
        fn from(value: NetworkRegistryStatusUpdatedFilter) -> Self {
            Self::NetworkRegistryStatusUpdatedFilter(value)
        }
    }
    impl ::core::convert::From<RegisteredFilter> for HoprNetworkRegistryEventsEvents {
        fn from(value: RegisteredFilter) -> Self {
            Self::RegisteredFilter(value)
        }
    }
    impl ::core::convert::From<RegisteredByManagerFilter>
    for HoprNetworkRegistryEventsEvents {
        fn from(value: RegisteredByManagerFilter) -> Self {
            Self::RegisteredByManagerFilter(value)
        }
    }
    impl ::core::convert::From<RequirementUpdatedFilter>
    for HoprNetworkRegistryEventsEvents {
        fn from(value: RequirementUpdatedFilter) -> Self {
            Self::RequirementUpdatedFilter(value)
        }
    }
}
