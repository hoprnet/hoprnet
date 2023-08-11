pub use hopr_channels_events::*;
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
pub mod hopr_channels_events {
    #[rustfmt::skip]
    const __ABI: &str = "[{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"channelId\",\"type\":\"bytes32\",\"components\":[],\"indexed\":true},{\"internalType\":\"HoprChannels.Balance\",\"name\":\"newBalance\",\"type\":\"uint96\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"ChannelBalanceDecreased\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"channelId\",\"type\":\"bytes32\",\"components\":[],\"indexed\":true},{\"internalType\":\"HoprChannels.Balance\",\"name\":\"newBalance\",\"type\":\"uint96\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"ChannelBalanceIncreased\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"channelId\",\"type\":\"bytes32\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"ChannelClosed\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"source\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"destination\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"HoprChannels.Balance\",\"name\":\"amount\",\"type\":\"uint96\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"ChannelOpened\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"channelId\",\"type\":\"bytes32\",\"components\":[],\"indexed\":true},{\"internalType\":\"HoprChannels.ChannelEpoch\",\"name\":\"epoch\",\"type\":\"uint24\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"CommitmentSet\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"channelId\",\"type\":\"bytes32\",\"components\":[],\"indexed\":true},{\"internalType\":\"HoprChannels.Timestamp\",\"name\":\"closureInitiationTime\",\"type\":\"uint32\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"OutgoingChannelClosureInitiated\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"channelId\",\"type\":\"bytes32\",\"components\":[],\"indexed\":true},{\"internalType\":\"HoprChannels.TicketIndex\",\"name\":\"newTicketIndex\",\"type\":\"uint48\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"TicketRedeemed\",\"outputs\":[],\"anonymous\":false}]";
    ///The parsed JSON ABI of the contract.
    pub static HOPRCHANNELSEVENTS_ABI: ::ethers::contract::Lazy<
        ::ethers::core::abi::Abi,
    > = ::ethers::contract::Lazy::new(|| {
        ::ethers::core::utils::__serde_json::from_str(__ABI)
            .expect("ABI is always valid")
    });
    pub struct HoprChannelsEvents<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for HoprChannelsEvents<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for HoprChannelsEvents<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for HoprChannelsEvents<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for HoprChannelsEvents<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(stringify!(HoprChannelsEvents)).field(&self.address()).finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> HoprChannelsEvents<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    HOPRCHANNELSEVENTS_ABI.clone(),
                    client,
                ),
            )
        }
        ///Gets the contract's `ChannelBalanceDecreased` event
        pub fn channel_balance_decreased_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ChannelBalanceDecreasedFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ChannelBalanceIncreased` event
        pub fn channel_balance_increased_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ChannelBalanceIncreasedFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ChannelClosed` event
        pub fn channel_closed_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ChannelClosedFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ChannelOpened` event
        pub fn channel_opened_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ChannelOpenedFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `CommitmentSet` event
        pub fn commitment_set_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            CommitmentSetFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `OutgoingChannelClosureInitiated` event
        pub fn outgoing_channel_closure_initiated_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            OutgoingChannelClosureInitiatedFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `TicketRedeemed` event
        pub fn ticket_redeemed_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            TicketRedeemedFilter,
        > {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            HoprChannelsEventsEvents,
        > {
            self.0.event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for HoprChannelsEvents<M> {
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
        name = "ChannelBalanceDecreased",
        abi = "ChannelBalanceDecreased(bytes32,uint96)"
    )]
    pub struct ChannelBalanceDecreasedFilter {
        #[ethevent(indexed)]
        pub channel_id: [u8; 32],
        pub new_balance: u128,
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
        name = "ChannelBalanceIncreased",
        abi = "ChannelBalanceIncreased(bytes32,uint96)"
    )]
    pub struct ChannelBalanceIncreasedFilter {
        #[ethevent(indexed)]
        pub channel_id: [u8; 32],
        pub new_balance: u128,
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
    #[ethevent(name = "ChannelClosed", abi = "ChannelClosed(bytes32)")]
    pub struct ChannelClosedFilter {
        #[ethevent(indexed)]
        pub channel_id: [u8; 32],
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
    #[ethevent(name = "ChannelOpened", abi = "ChannelOpened(address,address,uint96)")]
    pub struct ChannelOpenedFilter {
        #[ethevent(indexed)]
        pub source: ::ethers::core::types::Address,
        #[ethevent(indexed)]
        pub destination: ::ethers::core::types::Address,
        pub amount: u128,
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
    #[ethevent(name = "CommitmentSet", abi = "CommitmentSet(bytes32,uint24)")]
    pub struct CommitmentSetFilter {
        #[ethevent(indexed)]
        pub channel_id: [u8; 32],
        pub epoch: u32,
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
        name = "OutgoingChannelClosureInitiated",
        abi = "OutgoingChannelClosureInitiated(bytes32,uint32)"
    )]
    pub struct OutgoingChannelClosureInitiatedFilter {
        #[ethevent(indexed)]
        pub channel_id: [u8; 32],
        pub closure_initiation_time: u32,
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
    #[ethevent(name = "TicketRedeemed", abi = "TicketRedeemed(bytes32,uint48)")]
    pub struct TicketRedeemedFilter {
        #[ethevent(indexed)]
        pub channel_id: [u8; 32],
        pub new_ticket_index: u64,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum HoprChannelsEventsEvents {
        ChannelBalanceDecreasedFilter(ChannelBalanceDecreasedFilter),
        ChannelBalanceIncreasedFilter(ChannelBalanceIncreasedFilter),
        ChannelClosedFilter(ChannelClosedFilter),
        ChannelOpenedFilter(ChannelOpenedFilter),
        CommitmentSetFilter(CommitmentSetFilter),
        OutgoingChannelClosureInitiatedFilter(OutgoingChannelClosureInitiatedFilter),
        TicketRedeemedFilter(TicketRedeemedFilter),
    }
    impl ::ethers::contract::EthLogDecode for HoprChannelsEventsEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = ChannelBalanceDecreasedFilter::decode_log(log) {
                return Ok(
                    HoprChannelsEventsEvents::ChannelBalanceDecreasedFilter(decoded),
                );
            }
            if let Ok(decoded) = ChannelBalanceIncreasedFilter::decode_log(log) {
                return Ok(
                    HoprChannelsEventsEvents::ChannelBalanceIncreasedFilter(decoded),
                );
            }
            if let Ok(decoded) = ChannelClosedFilter::decode_log(log) {
                return Ok(HoprChannelsEventsEvents::ChannelClosedFilter(decoded));
            }
            if let Ok(decoded) = ChannelOpenedFilter::decode_log(log) {
                return Ok(HoprChannelsEventsEvents::ChannelOpenedFilter(decoded));
            }
            if let Ok(decoded) = CommitmentSetFilter::decode_log(log) {
                return Ok(HoprChannelsEventsEvents::CommitmentSetFilter(decoded));
            }
            if let Ok(decoded) = OutgoingChannelClosureInitiatedFilter::decode_log(log) {
                return Ok(
                    HoprChannelsEventsEvents::OutgoingChannelClosureInitiatedFilter(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded) = TicketRedeemedFilter::decode_log(log) {
                return Ok(HoprChannelsEventsEvents::TicketRedeemedFilter(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for HoprChannelsEventsEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::ChannelBalanceDecreasedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ChannelBalanceIncreasedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ChannelClosedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ChannelOpenedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CommitmentSetFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OutgoingChannelClosureInitiatedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::TicketRedeemedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<ChannelBalanceDecreasedFilter>
    for HoprChannelsEventsEvents {
        fn from(value: ChannelBalanceDecreasedFilter) -> Self {
            Self::ChannelBalanceDecreasedFilter(value)
        }
    }
    impl ::core::convert::From<ChannelBalanceIncreasedFilter>
    for HoprChannelsEventsEvents {
        fn from(value: ChannelBalanceIncreasedFilter) -> Self {
            Self::ChannelBalanceIncreasedFilter(value)
        }
    }
    impl ::core::convert::From<ChannelClosedFilter> for HoprChannelsEventsEvents {
        fn from(value: ChannelClosedFilter) -> Self {
            Self::ChannelClosedFilter(value)
        }
    }
    impl ::core::convert::From<ChannelOpenedFilter> for HoprChannelsEventsEvents {
        fn from(value: ChannelOpenedFilter) -> Self {
            Self::ChannelOpenedFilter(value)
        }
    }
    impl ::core::convert::From<CommitmentSetFilter> for HoprChannelsEventsEvents {
        fn from(value: CommitmentSetFilter) -> Self {
            Self::CommitmentSetFilter(value)
        }
    }
    impl ::core::convert::From<OutgoingChannelClosureInitiatedFilter>
    for HoprChannelsEventsEvents {
        fn from(value: OutgoingChannelClosureInitiatedFilter) -> Self {
            Self::OutgoingChannelClosureInitiatedFilter(value)
        }
    }
    impl ::core::convert::From<TicketRedeemedFilter> for HoprChannelsEventsEvents {
        fn from(value: TicketRedeemedFilter) -> Self {
            Self::TicketRedeemedFilter(value)
        }
    }
}
