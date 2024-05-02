pub mod topics {
    use bindings::{
        hopr_announcements::{AddressAnnouncementFilter, KeyBindingFilter, RevokeAnnouncementFilter},
        hopr_boost::{ApprovalFilter, TransferFilter},
        hopr_channels::{
            ChannelBalanceDecreasedFilter, ChannelBalanceIncreasedFilter, ChannelClosedFilter, ChannelOpenedFilter,
            DomainSeparatorUpdatedFilter, LedgerDomainSeparatorUpdatedFilter, OutgoingChannelClosureInitiatedFilter,
            TicketRedeemedFilter,
        },
        hopr_network_registry::{
            DeregisteredByManagerFilter, DeregisteredFilter, EligibilityUpdatedFilter,
            NetworkRegistryStatusUpdatedFilter, RegisteredByManagerFilter, RegisteredFilter, RequirementUpdatedFilter,
        },
        hopr_node_safe_registry::{DergisteredNodeSafeFilter, RegisteredNodeSafeFilter},
        hopr_ticket_price_oracle::TicketPriceUpdatedFilter,
    };
    use ethers::{contract::EthEvent, types::TxHash};

    pub fn channel() -> Vec<TxHash> {
        vec![
            ChannelBalanceDecreasedFilter::signature(),
            ChannelBalanceIncreasedFilter::signature(),
            ChannelClosedFilter::signature(),
            ChannelOpenedFilter::signature(),
            OutgoingChannelClosureInitiatedFilter::signature(),
            TicketRedeemedFilter::signature(),
            DomainSeparatorUpdatedFilter::signature(),
            LedgerDomainSeparatorUpdatedFilter::signature(),
        ]
    }

    pub fn token() -> Vec<TxHash> {
        vec![TransferFilter::signature(), ApprovalFilter::signature()]
    }

    pub fn network_registry() -> Vec<TxHash> {
        vec![
            DeregisteredByManagerFilter::signature(),
            DeregisteredFilter::signature(),
            EligibilityUpdatedFilter::signature(),
            NetworkRegistryStatusUpdatedFilter::signature(),
            RegisteredByManagerFilter::signature(),
            RegisteredFilter::signature(),
            RequirementUpdatedFilter::signature(),
        ]
    }

    pub fn announcement() -> Vec<TxHash> {
        vec![
            AddressAnnouncementFilter::signature(),
            KeyBindingFilter::signature(),
            RevokeAnnouncementFilter::signature(),
        ]
    }

    pub fn node_safe_registry() -> Vec<TxHash> {
        vec![
            RegisteredNodeSafeFilter::signature(),
            DergisteredNodeSafeFilter::signature(),
            bindings::hopr_node_safe_registry::DomainSeparatorUpdatedFilter::signature(),
        ]
    }

    pub fn ticket_price_oracle() -> Vec<TxHash> {
        vec![TicketPriceUpdatedFilter::signature()]
    }

    pub fn module_implementation() -> Vec<TxHash> {
        vec![]
    }
}
