use std::fmt::{Display, Formatter};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use async_trait::async_trait;
use futures::stream::BoxStream;

use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

use crate::errors::Result;

/// Allows to select multiple tickets (if `index` is `None`)
/// or a single ticket (with given `index`) in the given channel and epoch.
///
/// The selection can be further restricted to select ticket only in the given `state`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TicketSelector {
    /// Channel ID
    pub channel_id: Hash,
    /// Channel epoch
    pub epoch: U256,
    /// If given, will select single ticket with the given index
    /// in the given channel and epoch.
    pub index: Option<u64>,
    /// Further restriction to tickets with the given state.
    pub state: Option<AcknowledgedTicketStatus>,
    /// Further restrict to only aggregated tickets.
    pub only_aggregated: bool,
}

impl Display for TicketSelector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ticket selector in {} epoch {}{}{}{}",
            self.channel_id,
            self.epoch,
            self.index.map(|idx| format!(" with index {idx}")).unwrap_or("".into()),
            self.state
                .map(|state| format!(" in state {state}"))
                .unwrap_or("".into()),
            if self.only_aggregated { " only aggregated" } else { "" }
        )
    }
}

impl TicketSelector {
    /// Create a new ticket selector given the `channel_id` and `epoch`.
    pub fn new<T: Into<U256>>(channel_id: Hash, epoch: T) -> Self {
        Self {
            channel_id,
            epoch: epoch.into(),
            index: None,
            state: None,
            only_aggregated: false,
        }
    }

    /// If `false` is returned, the selector can fetch more than a single ticket.
    pub fn is_unique(&self) -> bool {
        self.index.is_some()
    }

    /// Returns this instance with ticket index set.
    pub fn with_index(mut self, index: u64) -> Self {
        self.index = Some(index);
        self
    }

    /// Returns this instance with ticket state set.
    pub fn with_state(mut self, state: AcknowledgedTicketStatus) -> Self {
        self.state = Some(state);
        self
    }

    /// Returns this instance without ticket state set.
    pub fn with_no_state(mut self) -> Self {
        self.state = None;
        self
    }

    /// Returns this instance with `only_aggregated` flag value.
    pub fn with_aggregated_only(mut self, only_aggregated: bool) -> Self {
        self.only_aggregated = only_aggregated;
        self
    }
}

impl From<&AcknowledgedTicket> for TicketSelector {
    fn from(value: &AcknowledgedTicket) -> Self {
        Self {
            channel_id: value.verified_ticket().channel_id,
            epoch: value.verified_ticket().channel_epoch.into(),
            index: Some(value.verified_ticket().index),
            state: Some(value.status),
            only_aggregated: value.verified_ticket().index_offset > 1,
        }
    }
}

impl From<&RedeemableTicket> for TicketSelector {
    fn from(value: &RedeemableTicket) -> Self {
        Self {
            channel_id: value.verified_ticket().channel_id,
            epoch: value.verified_ticket().channel_epoch.into(),
            index: Some(value.verified_ticket().index),
            state: None,
            only_aggregated: value.verified_ticket().index_offset > 1,
        }
    }
}

impl From<&ChannelEntry> for TicketSelector {
    fn from(value: &ChannelEntry) -> Self {
        Self {
            channel_id: value.get_id(),
            epoch: value.channel_epoch,
            index: None,
            state: None,
            only_aggregated: false,
        }
    }
}

impl From<ChannelEntry> for TicketSelector {
    fn from(value: ChannelEntry) -> Self {
        TicketSelector::from(&value)
    }
}

/// Prerequisites for the ticket aggregator.
/// The prerequisites are **independent** of each other.
/// If none of the prerequisites are given, they are considered satisfied.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AggregationPrerequisites {
    /// Minimum number of tickets in the channel.
    pub min_ticket_count: Option<usize>,
    /// Minimum ratio of balance of unaggregated messages and channel stake.
    /// I.e. the condition is met if sum of unaggregated ticket amounts divided by
    /// the total channel stake is greater than `min_unaggregated_ratio`.
    pub min_unaggregated_ratio: Option<f64>,
}

#[async_trait]
pub trait HoprDbTicketOperations {
    /// Retrieve acknowledged winning tickets according to the given `selector`.
    ///
    /// The optional transaction `tx` must be in the [tickets database](TargetDb::Tickets).
    async fn get_all_tickets(&self) -> Result<Vec<AcknowledgedTicket>>;

    /// Retrieve acknowledged winning tickets according to the given `selector`.
    ///
    /// The optional transaction `tx` must be in the [tickets database](TargetDb::Tickets).
    async fn get_tickets(&self, selector: TicketSelector) -> Result<Vec<AcknowledgedTicket>>;

    /// Marks tickets as redeemed (removing them from the DB) and updating the statistics.\
    /// Returns the number of tickets that were redeemed.
    async fn mark_tickets_redeemed(&self, selector: TicketSelector) -> Result<usize>;

    /// Marks tickets as redeemed (removing them from the DB) and updating the statistics.
    ///
    /// Returns the number of tickets that were neglected.
    async fn mark_tickets_neglected(&self, selector: TicketSelector) -> Result<usize>;

    /// Updates the ticket statistics according to the fact that the given ticket has
    /// been rejected by the packet processing pipeline.
    async fn mark_ticket_rejected(&self, ticket: &Ticket) -> Result<()>;

    /// Updates [state](AcknowledgedTicketStatus) of the tickets matching the given `selector`.
    ///
    /// Returns the updated tickets in the new state.
    async fn update_ticket_states_and_fetch<'a>(
        &'a self,
        selector: TicketSelector,
        new_state: AcknowledgedTicketStatus,
    ) -> Result<BoxStream<'a, AcknowledgedTicket>>;

    /// Updates [state](AcknowledgedTicketStatus) of the tickets matching the given `selector`.
    async fn update_ticket_states(
        &self,
        selector: TicketSelector,
        new_state: AcknowledgedTicketStatus,
    ) -> Result<usize>;

    /// Retrieves the ticket statistics for the given channel.
    ///
    /// If no channel is given, it retrieves aggregate ticket statistics for all channels.
    async fn get_ticket_statistics(&self, channel_id: Option<Hash>) -> Result<ChannelTicketStatistics>;

    /// Counts the tickets matching the given `selector` and their total value.
    ///
    /// The optional transaction `tx` must be in the [tickets database](TargetDb::Tickets).
    async fn get_tickets_value(&self, selector: TicketSelector) -> Result<(usize, Balance)>;

    /// Sets the stored outgoing ticket index to `index`, only if the currently stored value
    /// is less than `index`. This ensures the stored value can only be growing.
    ///
    /// Returns the old value.
    ///
    /// If the entry is not yet present for the given ID, it is initialized to 0.
    async fn compare_and_set_outgoing_ticket_index(&self, channel_id: Hash, index: u64) -> Result<u64>;

    /// Resets the outgoing ticket index to 0 for the given channel id.
    ///
    /// Returns the old value before reset.
    ///
    /// If the entry is not yet present for the given ID, it is initialized to 0.
    async fn reset_outgoing_ticket_index(&self, channel_id: Hash) -> Result<u64>;

    /// Increments the outgoing ticket index in the given channel ID and returns the value before incrementing.
    ///
    /// If the entry is not yet present for the given ID, it is initialized to 0 and incremented.
    async fn increment_outgoing_ticket_index(&self, channel_id: Hash) -> Result<u64>;

    /// Gets the current outgoing ticket index for the given channel id.
    ///
    /// If the entry is not yet present for the given ID, it is initialized to 0.
    async fn get_outgoing_ticket_index(&self, channel_id: Hash) -> Result<Arc<AtomicU64>>;

    /// Compares outgoing ticket indices in the cache with the stored values
    /// and updates the stored value where changed.
    ///
    /// Returns the number of updated ticket indices.
    async fn persist_outgoing_ticket_indices(&self) -> Result<usize>;

    /// Prepare a viable collection of tickets to be aggregated.
    ///
    /// Some preconditions for tickets apply. This callback will collect the aggregatable
    /// tickets and marks them as being aggregated.
    async fn prepare_aggregation_in_channel(
        &self,
        channel: &Hash,
        prerequisites: AggregationPrerequisites,
    ) -> Result<Option<(OffchainPublicKey, Vec<TransferableWinningTicket>, Hash)>>;

    /// Perform a ticket aggregation rollback in the channel.
    ///
    /// If a ticket aggregation fails, this callback can be invoked to make sure that
    /// resources are properly restored and cleaned up in the database, allowing further
    /// aggregations.
    async fn rollback_aggregation_in_channel(&self, channel: Hash) -> Result<()>;

    /// Replace the aggregated tickets locally with an aggregated ticket from the counterparty.
    async fn process_received_aggregated_ticket(
        &self,
        aggregated_ticket: Ticket,
        chain_keypair: &ChainKeypair,
    ) -> Result<AcknowledgedTicket>;

    /// Performs ticket aggregation as an issuing party of the given tickets.
    async fn aggregate_tickets(
        &self,
        destination: OffchainPublicKey,
        acked_tickets: Vec<TransferableWinningTicket>,
        me: &ChainKeypair,
    ) -> Result<VerifiedTicket>;
}

/// Can contains ticket statistics for a channel or aggregate ticket statistics for all channels.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ChannelTicketStatistics {
    pub winning_tickets: u128,
    pub neglected_value: Balance,
    pub redeemed_value: Balance,
    pub unredeemed_value: Balance,
    pub rejected_value: Balance,
}

impl Default for ChannelTicketStatistics {
    fn default() -> Self {
        Self {
            winning_tickets: 0,
            neglected_value: BalanceType::HOPR.zero(),
            redeemed_value: BalanceType::HOPR.zero(),
            unredeemed_value: BalanceType::HOPR.zero(),
            rejected_value: BalanceType::HOPR.zero(),
        }
    }
}
