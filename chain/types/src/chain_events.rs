//! Types that are related to events that are raised on-chain and extracted from chain logs.
//!
//! These events happen in response to actions (transactions, smart contract calls) done by a HOPR node on chain.
//!
//! See `chain-actions` and `chain-indexer` crates for details.
use std::fmt::{Display, Formatter};

use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

/// Enumeration of HOPR chain events.
#[derive(Debug, Clone, strum::EnumTryAs, strum::EnumIs, strum::EnumDiscriminants)]
pub enum ChainEvent {
    /// Peer on-chain announcement event.
    ///
    /// The [`AccountEntry`] is guaranteed to be [announced](AccountEntry::has_announced).
    Announcement(AccountEntry),
    /// A new channel has been opened
    ///
    /// The [`ChannelEntry`] is guaranteed to be [opened](ChannelStatus::Open).
    ChannelOpened(ChannelEntry),
    /// Channel closure has been initiated.
    ///
    /// The [`ChannelEntry`] is guaranteed to be [pending to close](ChannelStatus::PendingToClose).
    ChannelClosureInitiated(ChannelEntry),
    /// Channel closure has been finalized.
    ///
    /// The [`ChannelEntry`] is guaranteed to be [closed](ChannelStatus::Closed).
    ChannelClosed(ChannelEntry),
    /// Channel balance has increased by an amount.
    ///
    /// The [`HoprBalance`] is never `0` and represents the difference from the current new balance on the
    /// [`ChannelEntry`].
    ChannelBalanceIncreased(ChannelEntry, HoprBalance),
    /// Channel balance has decreased by an amount.
    ///
    /// The [`HoprBalance`] is never `0` and represents the difference from the current new balance on the
    /// [`ChannelEntry`].
    ChannelBalanceDecreased(ChannelEntry, HoprBalance),
    /// Ticket has been redeemed on a channel.
    ///
    /// If the channel is a node's own, it also contains the ticket that has been redeemed.
    TicketRedeemed(ChannelEntry, Option<Box<VerifiedTicket>>),

    /// The minimum winning probability has been increased.
    WinningProbabilityIncreased(WinningProbability),

    /// The minimum winning probability has been decreased.
    WinningProbabilityDecreased(WinningProbability),

    /// A new ticket price has been set.
    TicketPriceChanged(HoprBalance),
}

impl Display for ChainEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainEvent::Announcement(a) => write!(f, "announcement event of {a}"),
            ChainEvent::ChannelOpened(c) => write!(f, "open channel event {}", c.get_id()),
            ChainEvent::ChannelClosureInitiated(c) => write!(f, "close channel initiation event {}", c.get_id()),
            ChainEvent::ChannelClosed(c) => write!(f, "close channel event {}", c.get_id()),
            ChainEvent::ChannelBalanceIncreased(c, _) => write!(f, "channel increase balance event {}", c.get_id()),
            ChainEvent::ChannelBalanceDecreased(c, _) => write!(f, "channel decrease balance event {}", c.get_id()),
            ChainEvent::TicketRedeemed(c, _) => write!(f, "ticket redeem event in channel {}", c.get_id()),
            ChainEvent::WinningProbabilityIncreased(p) => write!(f, "winning probability increased to {p}"),
            ChainEvent::WinningProbabilityDecreased(p) => write!(f, "winning probability decreased to {p}"),
            ChainEvent::TicketPriceChanged(p) => write!(f, "ticket price changed to {p}"),
        }
    }
}
