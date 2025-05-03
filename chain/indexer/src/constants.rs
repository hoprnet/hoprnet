pub mod topics {
    use alloy::{primitives::B256, sol_types::SolEvent};
    use hopr_bindings::{
        hoprannouncementsevents::HoprAnnouncementsEvents::{AddressAnnouncement, KeyBinding, RevokeAnnouncement},
        hoprchannels::HoprChannels::LedgerDomainSeparatorUpdated,
        hoprchannelsevents::HoprChannelsEvents::{
            ChannelBalanceDecreased, ChannelBalanceIncreased, ChannelClosed, ChannelOpened, DomainSeparatorUpdated,
            OutgoingChannelClosureInitiated, TicketRedeemed,
        },
        hoprnetworkregistryevents::HoprNetworkRegistryEvents::{
            Deregistered, DeregisteredByManager, EligibilityUpdated, NetworkRegistryStatusUpdated, Registered,
            RegisteredByManager, RequirementUpdated,
        },
        hoprnodesaferegistryevents::HoprNodeSafeRegistryEvents::{DergisteredNodeSafe, RegisteredNodeSafe},
        hoprticketpriceoracleevents::HoprTicketPriceOracleEvents::TicketPriceUpdated,
        hoprtoken::HoprToken::{Approval, Transfer},
        hoprwinningprobabilityoracleevents::HoprWinningProbabilityOracleEvents::WinProbUpdated,
    };

    pub fn channel() -> Vec<B256> {
        vec![
            ChannelBalanceDecreased::SIGNATURE_HASH,
            ChannelBalanceIncreased::SIGNATURE_HASH,
            ChannelClosed::SIGNATURE_HASH,
            ChannelOpened::SIGNATURE_HASH,
            OutgoingChannelClosureInitiated::SIGNATURE_HASH,
            TicketRedeemed::SIGNATURE_HASH,
            DomainSeparatorUpdated::SIGNATURE_HASH,
            LedgerDomainSeparatorUpdated::SIGNATURE_HASH,
        ]
    }

    pub fn token() -> Vec<B256> {
        vec![Transfer::SIGNATURE_HASH, Approval::SIGNATURE_HASH]
    }

    pub fn network_registry() -> Vec<B256> {
        vec![
            DeregisteredByManager::SIGNATURE_HASH,
            Deregistered::SIGNATURE_HASH,
            EligibilityUpdated::SIGNATURE_HASH,
            NetworkRegistryStatusUpdated::SIGNATURE_HASH,
            RegisteredByManager::SIGNATURE_HASH,
            Registered::SIGNATURE_HASH,
            RequirementUpdated::SIGNATURE_HASH,
        ]
    }

    pub fn announcement() -> Vec<B256> {
        vec![
            AddressAnnouncement::SIGNATURE_HASH,
            KeyBinding::SIGNATURE_HASH,
            RevokeAnnouncement::SIGNATURE_HASH,
        ]
    }

    pub fn node_safe_registry() -> Vec<B256> {
        vec![
            RegisteredNodeSafe::SIGNATURE_HASH,
            DergisteredNodeSafe::SIGNATURE_HASH,
            hopr_bindings::hoprnodesaferegistryevents::HoprNodeSafeRegistryEvents::DomainSeparatorUpdated::SIGNATURE_HASH,
        ]
    }

    pub fn ticket_price_oracle() -> Vec<B256> {
        vec![TicketPriceUpdated::SIGNATURE_HASH]
    }

    pub fn winning_prob_oracle() -> Vec<B256> {
        vec![WinProbUpdated::SIGNATURE_HASH]
    }

    pub fn module_implementation() -> Vec<B256> {
        vec![]
    }
}
