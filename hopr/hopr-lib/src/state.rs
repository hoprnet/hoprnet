use std::hash::Hash;

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

pub trait Abortable {
    fn abort_process(&self);
}

impl Abortable for hopr_async_runtime::prelude::AbortHandle {
    fn abort_process(&self) {
        self.abort();
    }
}

impl Abortable for Box<dyn Abortable> {
    fn abort_process(&self) {
        self.as_ref().abort_process();
    }
}

impl Abortable for std::sync::Arc<dyn Abortable> {
    fn abort_process(&self) {
        self.as_ref().abort_process();
    }
}

/// An object containing abort handles to some spawned processes (such as [`HoprLibProcess`])
///
/// The abort handles are aborted once the object is dropped.
pub struct ProcessList<T, A: Abortable>(std::collections::HashMap<T, A>);

pub type HoprLibProcessList = ProcessList<HoprLibProcess, hopr_async_runtime::prelude::AbortHandle>;

impl<T, A: Abortable> Default for ProcessList<T, A> {
    fn default() -> Self {
        Self(std::collections::HashMap::new())
    }
}

impl<T: Clone + Hash + Eq, A: Abortable + Clone> ProcessList<T, A> {
    pub fn insert(&mut self, process: T, handle: A) {
        self.0.insert(process, handle);
    }

    pub fn extend(&mut self, other: impl Iterator<Item = (T, A)>) {
        for (k, v) in other {
            self.0.insert(k, v);
        }
    }

    pub fn into_iter(mut self) -> impl Iterator<Item = (T, A)> {
        let inner = self.0.clone();
        self.0.clear(); // Clear to avoid dropping the abort handles
        inner.into_iter()
    }
}

impl<T, A: Abortable> Abortable for ProcessList<T, A> {
    fn abort_process(&self) {
        for (_, handle) in self.0.iter() {
            handle.abort_process();
        }
    }
}

impl<T, A: Abortable> Drop for ProcessList<T, A> {
    fn drop(&mut self) {
        self.abort_process();
    }
}
