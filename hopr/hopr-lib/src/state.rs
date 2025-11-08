use crate::exports::transport::HoprTransportProcess;

/// An enum representing the current state of the HOPR node
#[atomic_enum::atomic_enum]
#[derive(PartialEq, Eq)]
pub enum HoprState {
    Uninitialized = 0,
    Initializing = 1,
    Indexing = 2,
    Starting = 3,
    Running = 4,
}

impl std::fmt::Display for HoprState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Enum differentiator for loop component futures.
///
/// Used to differentiate the type of the future that exits the loop premateruly
/// by tagging it as an enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, strum::Display)]
pub enum HoprLibProcesses {
    #[strum(to_string = "transport: {0}")]
    Transport(HoprTransportProcess),
    #[cfg(feature = "session-server")]
    #[strum(to_string = "session server providing the exit node session stream functionality")]
    SessionServer,
    #[strum(to_string = "tick wake up the strategies to perform an action")]
    StrategyTick,
    #[strum(to_string = "initial indexing operation into the DB")]
    Indexing,
    #[strum(to_string = "processing of indexed operations in internal components")]
    IndexReflection,
    #[strum(to_string = "on-chain transaction queue component for outgoing transactions")]
    OutgoingOnchainActionQueue,
    #[strum(to_string = "flush operation of outgoing ticket indices to the DB")]
    TicketIndexFlush,
    #[strum(to_string = "on received ticket event (winning or rejected)")]
    OnTicketEvent,
}

impl HoprLibProcesses {
    /// Identifies whether a loop is allowed to finish or should
    /// run indefinitely.
    pub fn can_finish(&self) -> bool {
        matches!(self, HoprLibProcesses::Indexing)
    }
}

impl From<HoprTransportProcess> for HoprLibProcesses {
    fn from(value: HoprTransportProcess) -> Self {
        HoprLibProcesses::Transport(value)
    }
}
