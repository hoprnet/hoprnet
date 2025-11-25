use std::{
    collections::{HashMap, HashSet},
    fmt::{Display, Formatter},
    ops::{Bound, RangeBounds},
};

use futures::stream::BoxStream;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

/// Allows selecting a range of ticket indices in [`TicketSelector`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TicketIndexSelector {
    /// Selects no ticket index specifically.
    /// This makes the [`TicketSelector`] less restrictive.
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

/// Allows selecting tickets via [`HoprDbTicketOperations`].
///
/// The `TicketSelector` always allows selecting only tickets in a single channel.
/// To select tickets across multiple channels, multiple `TicketSelector`s must be used.
#[derive(Clone, Debug)]
pub struct TicketSelector {
    /// Channel ID and Epoch pair.
    pub channel_identifier: (ChannelId, U256),
    /// If given, will select ticket(s) with the given indices
    /// in the given channel and epoch.
    ///
    /// See [`TicketIndexSelector`] for possible options.
    pub index: TicketIndexSelector,
    /// If given, the tickets are further restricted to the ones with a winning probability
    /// in this range.
    pub win_prob: (Bound<WinningProbability>, Bound<WinningProbability>),
    /// If given, the tickets are further restricted to the ones with an amount
    /// in this range.
    pub amount: (Bound<HoprBalance>, Bound<HoprBalance>),
    /// Further restriction to tickets with the given state.
    pub state: Option<AcknowledgedTicketStatus>,
}

impl Display for TicketSelector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let out = format!(
            "ticket selector in {:?} {}{}{}{}",
            self.channel_identifier,
            self.index,
            self.state
                .map(|state| format!(" in state {state}"))
                .unwrap_or("".into()),
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

fn approx_cmp_bounds(b1: Bound<WinningProbability>, b2: Bound<WinningProbability>) -> bool {
    match (b1, b2) {
        (Bound::Unbounded, Bound::Unbounded) => true,
        (Bound::Included(a), Bound::Included(b)) => b.approx_eq(&a),
        (Bound::Excluded(a), Bound::Excluded(b)) => b.approx_eq(&a),
        _ => false,
    }
}

impl PartialEq for TicketSelector {
    fn eq(&self, other: &Self) -> bool {
        self.channel_identifier == other.channel_identifier
            && self.index == other.index
            && self.state == other.state
            && self.amount == other.amount
            && approx_cmp_bounds(self.win_prob.0, other.win_prob.0)
            && approx_cmp_bounds(self.win_prob.1, other.win_prob.1)
    }
}

impl TicketSelector {
    /// Create a new ticket selector given the `channel_id` and `epoch`.
    pub fn new<T: Into<U256>>(channel_id: ChannelId, epoch: T) -> Self {
        Self {
            channel_identifier: (channel_id, epoch.into()),
            index: TicketIndexSelector::None,
            win_prob: (Bound::Unbounded, Bound::Unbounded),
            amount: (Bound::Unbounded, Bound::Unbounded),
            state: None,
        }
    }

    /// If `false` is returned, the selector can fetch more than a single ticket.
    pub fn is_unique(&self) -> bool {
        matches!(&self.index, TicketIndexSelector::Single(_))
            || matches!(&self.index, TicketIndexSelector::Multiple(indices) if indices.len() == 1)
    }

    /// Returns this instance with a ticket index set.
    ///
    /// This method can be called multiple times to select multiple tickets.
    /// If [`TicketSelector::with_index_range`] was previously called, it will be replaced.
    #[must_use]
    pub fn with_index(mut self, index: u64) -> Self {
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
    #[must_use]
    pub fn with_index_range<T: RangeBounds<u64>>(mut self, index_bound: T) -> Self {
        self.index = TicketIndexSelector::Range((index_bound.start_bound().cloned(), index_bound.end_bound().cloned()));
        self
    }

    /// Returns this instance with a ticket state set.
    #[must_use]
    pub fn with_state(mut self, state: AcknowledgedTicketStatus) -> Self {
        self.state = Some(state);
        self
    }

    /// Returns this instance without a ticket state set.
    #[must_use]
    pub fn with_no_state(mut self) -> Self {
        self.state = None;
        self
    }

    /// Returns this instance with a winning probability range bounds set.
    #[must_use]
    pub fn with_winning_probability<T: RangeBounds<WinningProbability>>(mut self, range: T) -> Self {
        self.win_prob = (range.start_bound().cloned(), range.end_bound().cloned());
        self
    }

    /// Returns this instance with the ticket amount range bounds set.
    #[must_use]
    pub fn with_amount<T: RangeBounds<HoprBalance>>(mut self, range: T) -> Self {
        self.amount = (range.start_bound().cloned(), range.end_bound().cloned());
        self
    }
}

impl From<&AcknowledgedTicket> for TicketSelector {
    fn from(value: &AcknowledgedTicket) -> Self {
        Self::from(value.verified_ticket())
    }
}

impl From<&RedeemableTicket> for TicketSelector {
    fn from(value: &RedeemableTicket) -> Self {
        Self::from(value.verified_ticket())
    }
}

impl From<&VerifiedTicket> for TicketSelector {
    fn from(value: &VerifiedTicket) -> Self {
        Self::from(value.verified_ticket())
    }
}

impl From<&Ticket> for TicketSelector {
    fn from(value: &Ticket) -> Self {
        Self {
            channel_identifier: (value.channel_id, value.channel_epoch.into()),
            index: TicketIndexSelector::Single(value.index),
            win_prob: (Bound::Unbounded, Bound::Unbounded),
            amount: (Bound::Unbounded, Bound::Unbounded),
            state: None,
        }
    }
}

impl From<&ChannelEntry> for TicketSelector {
    fn from(value: &ChannelEntry) -> Self {
        Self {
            channel_identifier: (*value.get_id(), value.channel_epoch),
            index: TicketIndexSelector::None,
            win_prob: (Bound::Unbounded, Bound::Unbounded),
            amount: (Bound::Unbounded, Bound::Unbounded),
            state: None,
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, strum::Display, Hash)]
#[strum(serialize_all = "lowercase")]
pub enum TicketMarker {
    /// Ticket has been successfully redeemed on-chain.
    Redeemed,
    /// An invalid ticket has been rejected by the packet processing pipeline.
    Rejected,
    /// A winning ticket that was not redeemed on-chain (e.g.: due to the channel being closed)
    Neglected,
}

// TODO: refactor this trait further so that caching responsibility does not lie in the DB (#7575)
/// Database operations for tickets.
///
/// The redeemable winning tickets enter the DB via [`HoprDb::insert_ticket`] and can only leave the DB
/// when [marked](TicketMarker) via [`HoprDbTicketOperations::mark_tickets_as`]
///
/// The overall value of tickets in the DB and of those that left the DB is tracked
/// via the [`ChannelTicketStatistics`] by calling the [`HoprDbTicketOperations::get_ticket_statistics`].
///
/// The statistics can also track tickets that were rejected before entering the DB,
/// which can be done via [`HoprDbTicketOperations::mark_unsaved_ticket_rejected`].
///
/// NOTE: tickets that are not winning are NOT considered as rejected. Non-winning tickets
/// are therefore not tracked in any statistics, also for performance reasons.
#[async_trait::async_trait]
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait HoprDbTicketOperations {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Retrieve acknowledged winning tickets, according to the given `selectors`.
    ///
    /// If no selector is given, streams tickets in all channels.
    async fn stream_tickets<'c, S: Into<TicketSelector>, I: IntoIterator<Item = S> + Send>(
        &'c self,
        selectors: I,
    ) -> Result<BoxStream<'c, RedeemableTicket>, Self::Error>;

    /// Inserts a new winning ticket into the DB.
    ///
    /// Returns an error if the ticket already exists.
    async fn insert_ticket(&self, ticket: RedeemableTicket) -> Result<(), Self::Error>;

    /// Marks tickets as the given [`TicketMarker`], removing them from the DB and updating the
    /// ticket statistics for each ticket's channel.
    ///
    /// Returns the number of marked tickets.
    async fn mark_tickets_as<S: Into<TicketSelector> + Send, I: IntoIterator<Item = S> + Send>(
        &self,
        selectors: I,
        mark_as: TicketMarker,
    ) -> Result<usize, Self::Error>;

    /// Updates the ticket statistics according to the fact that the given ticket has
    /// been rejected by the packet processing pipeline.
    ///
    /// This ticket is not yet stored in the ticket DB;
    /// therefore, only the statistics in the corresponding channel are updated.
    async fn mark_unsaved_ticket_rejected(&self, ticket: &Ticket) -> Result<(), Self::Error>;

    /// Updates the [state](AcknowledgedTicketStatus) of the tickets matching the given `selectors`.
    ///
    /// Returns the updated tickets in the new state.
    async fn update_ticket_states_and_fetch<'a, S: Into<TicketSelector>, I: IntoIterator<Item = S> + Send>(
        &'a self,
        selectors: I,
        new_state: AcknowledgedTicketStatus,
    ) -> Result<BoxStream<'a, RedeemableTicket>, Self::Error>;

    /// Updates [state](AcknowledgedTicketStatus) of the tickets matching the given `selector`.
    async fn update_ticket_states<S: Into<TicketSelector>, I: IntoIterator<Item = S> + Send>(
        &self,
        selectors: I,
        new_state: AcknowledgedTicketStatus,
    ) -> Result<usize, Self::Error>;

    /// Retrieves the ticket statistics for the given channel.
    ///
    /// If no channel is given, it retrieves aggregate ticket statistics for all channels.
    async fn get_ticket_statistics(
        &self,
        channel_id: Option<ChannelId>,
    ) -> Result<ChannelTicketStatistics, Self::Error>;

    /// Resets the ticket statistics about neglected, rejected, and redeemed tickets.
    async fn reset_ticket_statistics(&self) -> Result<(), Self::Error>;

    /// Counts the total value of tickets matching the given `selector` on a single channel.
    ///
    /// Returns the count of tickets and the total ticket value.
    async fn get_tickets_value(&self, selector: TicketSelector) -> Result<(usize, HoprBalance), Self::Error>;

    /// Gets the index of the next outgoing ticket for the given channel.
    ///
    /// If such an entry does not exist, it is initialized with 0 and `None` is returned.
    async fn get_or_create_outgoing_ticket_index(
        &self,
        channel_id: &ChannelId,
        epoch: u32,
    ) -> Result<Option<u64>, Self::Error>;

    /// Stores the ticket index of the next outgoing ticket for the given channel.
    ///
    /// Does nothing if the entry for the given channel and epoch does not exist.
    /// Returns an error if the given `index` is less than the current index in the DB.
    async fn update_outgoing_ticket_index(
        &self,
        channel_id: &ChannelId,
        epoch: u32,
        index: u64,
    ) -> Result<(), Self::Error>;

    /// Removes the outgoing ticket index for the given channel and epoch.
    ///
    /// Does nothing if the value did not exist
    async fn remove_outgoing_ticket_index(&self, channel_id: &ChannelId, epoch: u32) -> Result<(), Self::Error>;
}

/// Contains ticket statistics for one or more channels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChannelTicketStatistics {
    /// Total number of winning tickets.
    pub winning_tickets: u128,
    /// Values of tickets that were finalized and removed from the DB.
    pub finalized_values: HashMap<TicketMarker, HoprBalance>,
    /// The total value in unredeemed winning tickets still in the DB.
    pub unredeemed_value: HoprBalance,
}

impl Default for ChannelTicketStatistics {
    fn default() -> Self {
        Self {
            winning_tickets: 0,
            finalized_values: HashMap::from([
                (TicketMarker::Neglected, HoprBalance::zero()),
                (TicketMarker::Rejected, HoprBalance::zero()),
                (TicketMarker::Redeemed, HoprBalance::zero()),
            ]),
            unredeemed_value: HoprBalance::zero(),
        }
    }
}

impl ChannelTicketStatistics {
    /// The total value of neglected tickets.
    pub fn neglected_value(&self) -> HoprBalance {
        self.finalized_values
            .get(&TicketMarker::Neglected)
            .copied()
            .unwrap_or_default()
    }

    /// The total value of rejected tickets.
    pub fn rejected_value(&self) -> HoprBalance {
        self.finalized_values
            .get(&TicketMarker::Rejected)
            .copied()
            .unwrap_or_default()
    }

    /// The total value of redeemed tickets.
    pub fn redeemed_value(&self) -> HoprBalance {
        self.finalized_values
            .get(&TicketMarker::Redeemed)
            .copied()
            .unwrap_or_default()
    }
}
