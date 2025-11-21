use std::hash::Hash;

use crate::exports::transport::HoprTransportProcess;

/// An enum representing the current state of the HOPR node
#[atomic_enum::atomic_enum]
#[derive(PartialEq, Eq)]
pub enum HoprState {
    Uninitialized = 0,
    Initializing = 1,
    Running = 2,
    Terminated = 3,
}

impl std::fmt::Display for HoprState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Long-running tasks that are spawned by the HOPR node.
#[derive(Debug, Clone, PartialEq, Eq, Hash, strum::Display, strum::EnumCount)]
pub enum HoprLibProcess {
    #[strum(to_string = "transport: {0}")]
    Transport(HoprTransportProcess),
    #[cfg(feature = "session-server")]
    #[strum(to_string = "session server providing the exit node session stream functionality")]
    SessionServer,
    #[strum(to_string = "flush operation of outgoing ticket indices to the DB")]
    TicketIndexFlush,
    #[strum(to_string = "ticket redemption queue driver")]
    TicketRedemptions,
    #[strum(to_string = "subscription for on-chain account announcements")]
    AccountAnnouncements,
    #[strum(to_string = "subscription for on-chain channel updates")]
    ChannelEvents,
    #[strum(to_string = "on received ticket event (winning or rejected)")]
    OnTicketEvent,
}
