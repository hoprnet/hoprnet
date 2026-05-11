//! Implements complete logic of ticket management in the HOPR protocol.
//!
//! There are two major parts in the architecture of the ticket management:
//! - [`HoprTicketManager`] is responsible for managing the incoming ticket queues and ticket redemption.
//! - [`HoprTicketFactory`] is responsible for managing outgoing ticket indices and creating outgoing tickets.
//!
//! Usually, they are created together via [`HoprTicketManager::new_with_factory`].
//!
//! See the [`HoprTicketManager`] and [`HoprTicketFactory`] documentation for complete details.

mod backend;
mod errors;
mod factory;
mod manager;
mod traits;
mod utils;

#[cfg(feature = "redb")]
pub use crate::backend::{RedbStore, RedbTicketQueue};
pub use crate::{
    backend::{MemoryStore, MemoryTicketQueue},
    errors::TicketManagerError,
    factory::HoprTicketFactory,
    manager::HoprTicketManager,
    traits::{OutgoingIndexStore, TicketQueue, TicketQueueStore},
};
