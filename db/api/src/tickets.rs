use std::{
    collections::HashSet,
    fmt::{Display, Formatter},
    ops::{Bound, RangeBounds},
    sync::{Arc, atomic::AtomicU64},
};

use async_trait::async_trait;
use futures::stream::BoxStream;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

use crate::errors::Result;

/// Allows selecting a range of ticket indices in [TicketSelector].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TicketIndexSelector {
    /// Selects no ticket index specifically.
    /// This makes the [TicketSelector] less restrictive.
    #[default]
    None,
    /// Selects a single ticket with the given index.
    Single(u64),
    /// Selects multiple tickets with the given indices.
    Multiple(HashSet<u64>),
    /// Selects multiple tickets with indices within the given range.
    Range((Bound<u64>, Bound<u64>)),
}

impl Display for TicketIndexSelector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            TicketIndexSelector::None => write!(f, ""),
            TicketIndexSelector::Single(idx) => write!(f, "with index {idx}"),
            TicketIndexSelector::Multiple(indices) => write!(f, "with indices {indices:?}"),
            TicketIndexSelector::Range((lb, ub)) => write!(f, "with indices in {lb:?}..{ub:?}"),
        }
    }
}

/// Allows selecting multiple tickets (if `index` does not contain a single value)
/// or a single ticket (with unitary `index`) in the given channel and epoch.
/// The selection can be further restricted to select ticket only in the given `state`.
#[derive(Clone, Debug)]
pub struct TicketSelector {
    /// Channel ID and Epoch pairs.
    pub channel_identifiers: Vec<(Hash, U256)>,
    /// If given, will select ticket(s) with the given indices
    /// in the given channel and epoch.
    /// See [TicketIndexSelector] for possible options.
    pub index: TicketIndexSelector,
    /// If given, the tickets are further restricted to the ones with a winning probability
    /// in this range.
    pub win_prob: (Bound<WinningProbability>, Bound<WinningProbability>),
    /// If given, the tickets are further restricted to the ones with an amount
    /// in this range.
    pub amount: (Bound<HoprBalance>, Bound<HoprBalance>),
    /// Further restriction to tickets with the given state.
    pub state: Option<AcknowledgedTicketStatus>,
    /// Further restrict to only aggregated tickets.
    pub only_aggregated: bool,
}

impl Display for TicketSelector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let out = format!(
            "ticket selector in {:?} {}{}{}{}{}",
            self.channel_identifiers,
            self.index,
            self.state
                .map(|state| format!(" in state {state}"))
                .unwrap_or("".into()),
            if self.only_aggregated { " only aggregated" } else { "" },
            match &self.win_prob {
                (Bound::Unbounded, Bound::Unbounded) => "".to_string(),
                bounds => format!(" with winning probability in {bounds:?}"),
            },
            match &self.amount {
                (Bound::Unbounded, Bound::Unbounded) => "".to_string(),
                bounds => format!(" with amount in {bounds:?}"),
            },
        );
        write!(f, "{}", out.trim())
    }
}

impl TicketSelector {
    /// Create a new ticket selector given the `channel_id` and `epoch`.
    pub fn new<T: Into<U256>>(channel_id: Hash, epoch: T) -> Self {
        Self {
            channel_identifiers: vec![(channel_id, epoch.into())],
            index: TicketIndexSelector::None,
            win_prob: (Bound::Unbounded, Bound::Unbounded),
            amount: (Bound::Unbounded, Bound::Unbounded),
            state: None,
            only_aggregated: false,
        }
    }

    /// Allows matching tickets on multiple channels, by adding the given
    /// `channel_id` and `epoch` to the selector.
    ///
    /// This also nullifies any prior effect of any prior calls to [`TicketSelector::with_index`] or
    /// [`TicketSelector::with_index_range`]
    /// as ticket indices cannot be matched over multiple channels.
    pub fn also_on_channel<T: Into<U256>>(self, channel_id: Hash, epoch: T) -> Self {
        let mut ret = self.clone();
        ret.index = TicketIndexSelector::None;
        ret.channel_identifiers.push((channel_id, epoch.into()));
        ret
    }

    /// Sets the selector to match only tickets on the given `channel_id` and `epoch`.
    /// This nullifies any prior calls to [`TicketSelector::also_on_channel`].
    pub fn just_on_channel<T: Into<U256>>(self, channel_id: Hash, epoch: T) -> Self {
        let mut ret = self.clone();
        ret.channel_identifiers = vec![(channel_id, epoch.into())];
        ret
    }

    /// Checks if this selector operates only on a single channel.
    ///
    /// This will return `false` if [`TicketSelector::also_on_channel`] was called, and neither
    /// the [`TicketSelector::with_index`] nor [`TicketSelector::with_index_range`] were
    /// called subsequently.
    pub fn is_single_channel(&self) -> bool {
        self.channel_identifiers.len() == 1
    }

    /// If `false` is returned, the selector can fetch more than a single ticket.
    pub fn is_unique(&self) -> bool {
        self.is_single_channel()
            && (matches!(&self.index, TicketIndexSelector::Single(_))
                || matches!(&self.index, TicketIndexSelector::Multiple(indices) if indices.len() == 1))
    }

    /// Returns this instance with a ticket index set.
    /// This method can be called multiple times to select multiple tickets.
    /// If [`TicketSelector::with_index_range`] was previously called, it will be replaced.
    /// If [`TicketSelector::also_on_channel`] was previously called, its effect will be nullified.
    pub fn with_index(mut self, index: u64) -> Self {
        self.channel_identifiers.truncate(1);
        self.index = match self.index {
            TicketIndexSelector::None | TicketIndexSelector::Range(_) => TicketIndexSelector::Single(index),
            TicketIndexSelector::Single(existing) => {
                TicketIndexSelector::Multiple(HashSet::from_iter([existing, index]))
            }
            TicketIndexSelector::Multiple(mut existing) => {
                existing.insert(index);
                TicketIndexSelector::Multiple(existing)
            }
        };
        self
    }

    /// Returns this instance with a ticket index upper bound set.
    /// If [`TicketSelector::with_index`] was previously called, it will be replaced.
    /// If [`TicketSelector::also_on_channel`] was previously called, its effect will be nullified.
    pub fn with_index_range<T: RangeBounds<u64>>(mut self, index_bound: T) -> Self {
        self.channel_identifiers.truncate(1);
        self.index = TicketIndexSelector::Range((index_bound.start_bound().cloned(), index_bound.end_bound().cloned()));
        self
    }

    /// Returns this instance with a ticket state set.
    pub fn with_state(mut self, state: AcknowledgedTicketStatus) -> Self {
        self.state = Some(state);
        self
    }

    /// Returns this instance without a ticket state set.
    pub fn with_no_state(mut self) -> Self {
        self.state = None;
        self
    }

    /// Returns this instance with `only_aggregated` flag value.
    pub fn with_aggregated_only(mut self, only_aggregated: bool) -> Self {
        self.only_aggregated = only_aggregated;
        self
    }

    /// Returns this instance with a winning probability range bounds set.
    pub fn with_winning_probability<T: RangeBounds<WinningProbability>>(mut self, range: T) -> Self {
        self.win_prob = (range.start_bound().cloned(), range.end_bound().cloned());
        self
    }

    /// Returns this instance with the ticket amount range bounds set.
    pub fn with_amount<T: RangeBounds<HoprBalance>>(mut self, range: T) -> Self {
        self.amount = (range.start_bound().cloned(), range.end_bound().cloned());
        self
    }
}

impl From<&AcknowledgedTicket> for TicketSelector {
    fn from(value: &AcknowledgedTicket) -> Self {
        Self {
            channel_identifiers: vec![(
                value.verified_ticket().channel_id,
                value.verified_ticket().channel_epoch.into(),
            )],
            index: TicketIndexSelector::Single(value.verified_ticket().index),
            win_prob: (Bound::Unbounded, Bound::Unbounded),
            amount: (Bound::Unbounded, Bound::Unbounded),
            state: Some(value.status),
            only_aggregated: value.verified_ticket().index_offset > 1,
        }
    }
}

impl From<&RedeemableTicket> for TicketSelector {
    fn from(value: &RedeemableTicket) -> Self {
        Self {
            channel_identifiers: vec![(
                value.verified_ticket().channel_id,
                value.verified_ticket().channel_epoch.into(),
            )],
            index: TicketIndexSelector::Single(value.verified_ticket().index),
            win_prob: (Bound::Unbounded, Bound::Unbounded),
            amount: (Bound::Unbounded, Bound::Unbounded),
            state: None,
            only_aggregated: value.verified_ticket().index_offset > 1,
        }
    }
}

impl From<&ChannelEntry> for TicketSelector {
    fn from(value: &ChannelEntry) -> Self {
        Self {
            channel_identifiers: vec![(value.get_id(), value.channel_epoch)],
            index: TicketIndexSelector::None,
            win_prob: (Bound::Unbounded, Bound::Unbounded),
            amount: (Bound::Unbounded, Bound::Unbounded),
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

/// Different markers for unredeemed tickets.
/// See [`HoprDbTicketOperations::mark_tickets_as`] for usage.
#[derive(Debug, Copy, Clone, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum TicketMarker {
    Redeemed,
    Rejected,
    Neglected,
}

#[async_trait]
pub trait HoprDbTicketOperations {
    /// Retrieve acknowledged winning tickets, according to the given `selector`.
    ///
    /// The optional transaction `tx` must be in the database.
    async fn get_all_tickets(&self) -> Result<Vec<AcknowledgedTicket>>;

    /// Retrieve acknowledged winning tickets, according to the given `selector`.
    ///
    /// The optional transaction `tx` must be in the database.
    async fn get_tickets(&self, selector: TicketSelector) -> Result<Vec<AcknowledgedTicket>>;

    /// Marks tickets as the given [`TicketMarker`], removing them from the DB and updating the
    /// ticket statistics for each ticket's channel.
    ///
    /// Returns the number of marked tickets.
    async fn mark_tickets_as(&self, selector: TicketSelector, mark_as: TicketMarker) -> Result<usize>;

    /// Updates the ticket statistics according to the fact that the given ticket has
    /// been rejected by the packet processing pipeline.
    ///
    /// This ticket is not yet stored in the ticket DB;
    /// therefore, only the statistics in the corresponding channel are updated.
    async fn mark_unsaved_ticket_rejected(&self, ticket: &Ticket) -> Result<()>;

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

    /// Resets the ticket statistics about neglected, rejected, and redeemed tickets.
    async fn reset_ticket_statistics(&self) -> Result<()>;

    /// Counts the tickets matching the given `selector` and their total value.
    ///
    /// The optional transaction `tx` must be in the database.
    async fn get_tickets_value(&self, selector: TicketSelector) -> Result<(usize, HoprBalance)>;

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
}

/// Can contain ticket statistics for a channel or aggregated ticket statistics for all channels.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct ChannelTicketStatistics {
    pub winning_tickets: u128,
    pub neglected_value: HoprBalance,
    pub redeemed_value: HoprBalance,
    pub unredeemed_value: HoprBalance,
    pub rejected_value: HoprBalance,
}
