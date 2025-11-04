use strum::EnumCount;

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

/// Long-running tasks that are spawned by the HOPR node.
#[derive(Debug, Clone, PartialEq, Eq, Hash, strum::Display, strum::EnumCount)]
pub enum HoprLibProcess {
    #[strum(to_string = "transport: {0}")]
    Transport(HoprTransportProcess),
    #[cfg(feature = "session-server")]
    #[strum(to_string = "session server providing the exit node session stream functionality")]
    SessionServer,
    #[strum(to_string = "streaming of events to strategies")]
    Strategies,
    #[strum(to_string = "flush operation of outgoing ticket indices to the DB")]
    TicketIndexFlush,
}

impl From<HoprTransportProcess> for HoprLibProcess {
    fn from(value: HoprTransportProcess) -> Self {
        HoprLibProcess::Transport(value)
    }
}

/// Object containing abort handles to all [`HoprLibProcess`].
///
/// The abort handles are aborted once the object is dropped.
pub struct HoprLibProcessList(std::collections::HashMap<HoprLibProcess, hopr_async_runtime::prelude::AbortHandle>);

impl HoprLibProcessList {
    pub(crate) fn new() -> Self {
        Self(std::collections::HashMap::with_capacity(HoprLibProcess::COUNT))
    }

    pub(crate) fn insert(&mut self, process: HoprLibProcess, handle: hopr_async_runtime::prelude::AbortHandle) {
        self.0.insert(process, handle);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&HoprLibProcess, &hopr_async_runtime::prelude::AbortHandle)> {
        self.0.iter()
    }
}

impl Drop for HoprLibProcessList {
    fn drop(&mut self) {
        for (_, handle) in self.0.iter() {
            handle.abort();
        }
    }
}
