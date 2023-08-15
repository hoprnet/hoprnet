pub use hopr_announcements_events::*;
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
pub mod hopr_announcements_events {
    #[rustfmt::skip]
    const __ABI: &str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"node\",\"type\":\"address\",\"components\":[],\"indexed\":false},{\"internalType\":\"string\",\"name\":\"baseMultiaddr\",\"type\":\"string\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"AddressAnnouncement\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"ed25519_sig_0\",\"type\":\"bytes32\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes32\",\"name\":\"ed25519_sig_1\",\"type\":\"bytes32\",\"components\":[],\"indexed\":false},{\"internalType\":\"bytes32\",\"name\":\"ed25519_pub_key\",\"type\":\"bytes32\",\"components\":[],\"indexed\":false},{\"internalType\":\"address\",\"name\":\"chain_key\",\"type\":\"address\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"KeyBinding\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"node\",\"type\":\"address\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"RevokeAnnouncement\",\"outputs\":[],\"anonymous\":false}]";
    ///The parsed JSON ABI of the contract.
    pub static HOPRANNOUNCEMENTSEVENTS_ABI: ::ethers::contract::Lazy<
        ::ethers::core::abi::Abi,
    > = ::ethers::contract::Lazy::new(|| {
        ::ethers::core::utils::__serde_json::from_str(__ABI)
            .expect("ABI is always valid")
    });
    pub struct HoprAnnouncementsEvents<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for HoprAnnouncementsEvents<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for HoprAnnouncementsEvents<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for HoprAnnouncementsEvents<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for HoprAnnouncementsEvents<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(stringify!(HoprAnnouncementsEvents))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> HoprAnnouncementsEvents<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    HOPRANNOUNCEMENTSEVENTS_ABI.clone(),
                    client,
                ),
            )
        }
        ///Gets the contract's `AddressAnnouncement` event
        pub fn address_announcement_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            AddressAnnouncementFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `KeyBinding` event
        pub fn key_binding_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            KeyBindingFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `RevokeAnnouncement` event
        pub fn revoke_announcement_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            RevokeAnnouncementFilter,
        > {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            HoprAnnouncementsEventsEvents,
        > {
            self.0.event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for HoprAnnouncementsEvents<M> {
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
        name = "AddressAnnouncement",
        abi = "AddressAnnouncement(address,string)"
    )]
    pub struct AddressAnnouncementFilter {
        pub node: ::ethers::core::types::Address,
        pub base_multiaddr: ::std::string::String,
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
    #[ethevent(name = "KeyBinding", abi = "KeyBinding(bytes32,bytes32,bytes32,address)")]
    pub struct KeyBindingFilter {
        pub ed_25519_sig_0: [u8; 32],
        pub ed_25519_sig_1: [u8; 32],
        pub ed_25519_pub_key: [u8; 32],
        pub chain_key: ::ethers::core::types::Address,
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
    #[ethevent(name = "RevokeAnnouncement", abi = "RevokeAnnouncement(address)")]
    pub struct RevokeAnnouncementFilter {
        pub node: ::ethers::core::types::Address,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum HoprAnnouncementsEventsEvents {
        AddressAnnouncementFilter(AddressAnnouncementFilter),
        KeyBindingFilter(KeyBindingFilter),
        RevokeAnnouncementFilter(RevokeAnnouncementFilter),
    }
    impl ::ethers::contract::EthLogDecode for HoprAnnouncementsEventsEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = AddressAnnouncementFilter::decode_log(log) {
                return Ok(
                    HoprAnnouncementsEventsEvents::AddressAnnouncementFilter(decoded),
                );
            }
            if let Ok(decoded) = KeyBindingFilter::decode_log(log) {
                return Ok(HoprAnnouncementsEventsEvents::KeyBindingFilter(decoded));
            }
            if let Ok(decoded) = RevokeAnnouncementFilter::decode_log(log) {
                return Ok(
                    HoprAnnouncementsEventsEvents::RevokeAnnouncementFilter(decoded),
                );
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for HoprAnnouncementsEventsEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AddressAnnouncementFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::KeyBindingFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::RevokeAnnouncementFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<AddressAnnouncementFilter>
    for HoprAnnouncementsEventsEvents {
        fn from(value: AddressAnnouncementFilter) -> Self {
            Self::AddressAnnouncementFilter(value)
        }
    }
    impl ::core::convert::From<KeyBindingFilter> for HoprAnnouncementsEventsEvents {
        fn from(value: KeyBindingFilter) -> Self {
            Self::KeyBindingFilter(value)
        }
    }
    impl ::core::convert::From<RevokeAnnouncementFilter>
    for HoprAnnouncementsEventsEvents {
        fn from(value: RevokeAnnouncementFilter) -> Self {
            Self::RevokeAnnouncementFilter(value)
        }
    }
}
