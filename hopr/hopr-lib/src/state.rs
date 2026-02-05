use std::hash::Hash;

use crate::exports::transport::HoprTransportProcess;

/// An enum representing the current state of the HOPR node.
///
/// The states represent granular steps in the node initialization and lifecycle,
/// providing detailed progress information suitable for UI applications.
#[atomic_enum::atomic_enum]
#[derive(PartialEq, Eq, strum::Display, strum::EnumIter)]
pub enum HoprState {
    /// Node instance created but not yet started
    #[strum(to_string = "Node is not yet initialized")]
    Uninitialized = 0,

    /// Waiting for the node wallet to receive initial funding
    #[strum(to_string = "Waiting for initial wallet funding")]
    WaitingForFunds = 1,

    /// Verifying the wallet has sufficient balance to operate
    #[strum(to_string = "Verifying wallet balance")]
    CheckingBalance = 2,

    /// Validating network parameters like ticket price and winning probability
    #[strum(to_string = "Validating network configuration")]
    ValidatingNetworkConfig = 3,

    /// Subscribing to on-chain account announcements
    #[strum(to_string = "Subscribing to network announcements")]
    SubscribingToAnnouncements = 4,

    /// Registering the Safe contract with this node
    #[strum(to_string = "Registering Safe contract")]
    RegisteringSafe = 5,

    /// Announcing node multiaddresses on the blockchain
    #[strum(to_string = "Announcing node on chain")]
    AnnouncingNode = 6,

    /// Waiting for the node's key binding to appear on-chain
    #[strum(to_string = "Waiting for on-chain key binding confirmation")]
    AwaitingKeyBinding = 7,

    /// Initializing internal services (sessions, tickets, transport, channels)
    #[strum(to_string = "Initializing internal services")]
    InitializingServices = 8,

    /// Node is fully operational and ready for use
    #[strum(to_string = "Node is running")]
    Running = 9,

    /// Node has been shut down
    #[strum(to_string = "Node has been terminated")]
    Terminated = 10,
}

/// Long-running tasks that are spawned by the HOPR node.
#[derive(Debug, Clone, PartialEq, Eq, Hash, strum::Display, strum::EnumCount)]
pub enum HoprLibProcess {
    #[strum(to_string = "transport: {0}")]
    Transport(HoprTransportProcess),
    #[cfg(feature = "session-server")]
    #[strum(to_string = "session server providing the exit node session stream functionality")]
    SessionServer,
    #[strum(to_string = "ticket redemption queue driver")]
    TicketRedemptions,
    #[strum(to_string = "subscription for on-chain account announcements")]
    AccountAnnouncements,
    #[strum(to_string = "subscription for on-chain channel updates")]
    ChannelEvents,
    #[strum(to_string = "on received ticket event (winning or rejected)")]
    TicketEvents,
}
