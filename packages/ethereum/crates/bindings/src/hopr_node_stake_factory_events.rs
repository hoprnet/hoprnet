pub use hopr_node_stake_factory_events::*;
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
pub mod hopr_node_stake_factory_events {
    #[rustfmt::skip]
    const __ABI: &str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"moduleImplementation\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"instance\",\"type\":\"address\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"NewHoprNodeStakeModule\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"instance\",\"type\":\"address\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"NewHoprNodeStakeSafe\",\"outputs\":[],\"anonymous\":false}]";
    ///The parsed JSON ABI of the contract.
    pub static HOPRNODESTAKEFACTORYEVENTS_ABI: ::ethers::contract::Lazy<
        ::ethers::core::abi::Abi,
    > = ::ethers::contract::Lazy::new(|| {
        ::ethers::core::utils::__serde_json::from_str(__ABI)
            .expect("ABI is always valid")
    });
    pub struct HoprNodeStakeFactoryEvents<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for HoprNodeStakeFactoryEvents<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for HoprNodeStakeFactoryEvents<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for HoprNodeStakeFactoryEvents<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for HoprNodeStakeFactoryEvents<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(stringify!(HoprNodeStakeFactoryEvents))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> HoprNodeStakeFactoryEvents<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    HOPRNODESTAKEFACTORYEVENTS_ABI.clone(),
                    client,
                ),
            )
        }
        ///Gets the contract's `NewHoprNodeStakeModule` event
        pub fn new_hopr_node_stake_module_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            NewHoprNodeStakeModuleFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `NewHoprNodeStakeSafe` event
        pub fn new_hopr_node_stake_safe_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            NewHoprNodeStakeSafeFilter,
        > {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            HoprNodeStakeFactoryEventsEvents,
        > {
            self.0.event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for HoprNodeStakeFactoryEvents<M> {
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
        name = "NewHoprNodeStakeModule",
        abi = "NewHoprNodeStakeModule(address,address)"
    )]
    pub struct NewHoprNodeStakeModuleFilter {
        #[ethevent(indexed)]
        pub module_implementation: ::ethers::core::types::Address,
        pub instance: ::ethers::core::types::Address,
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
    #[ethevent(name = "NewHoprNodeStakeSafe", abi = "NewHoprNodeStakeSafe(address)")]
    pub struct NewHoprNodeStakeSafeFilter {
        pub instance: ::ethers::core::types::Address,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum HoprNodeStakeFactoryEventsEvents {
        NewHoprNodeStakeModuleFilter(NewHoprNodeStakeModuleFilter),
        NewHoprNodeStakeSafeFilter(NewHoprNodeStakeSafeFilter),
    }
    impl ::ethers::contract::EthLogDecode for HoprNodeStakeFactoryEventsEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = NewHoprNodeStakeModuleFilter::decode_log(log) {
                return Ok(
                    HoprNodeStakeFactoryEventsEvents::NewHoprNodeStakeModuleFilter(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded) = NewHoprNodeStakeSafeFilter::decode_log(log) {
                return Ok(
                    HoprNodeStakeFactoryEventsEvents::NewHoprNodeStakeSafeFilter(decoded),
                );
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for HoprNodeStakeFactoryEventsEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::NewHoprNodeStakeModuleFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NewHoprNodeStakeSafeFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<NewHoprNodeStakeModuleFilter>
    for HoprNodeStakeFactoryEventsEvents {
        fn from(value: NewHoprNodeStakeModuleFilter) -> Self {
            Self::NewHoprNodeStakeModuleFilter(value)
        }
    }
    impl ::core::convert::From<NewHoprNodeStakeSafeFilter>
    for HoprNodeStakeFactoryEventsEvents {
        fn from(value: NewHoprNodeStakeSafeFilter) -> Self {
            Self::NewHoprNodeStakeSafeFilter(value)
        }
    }
}
