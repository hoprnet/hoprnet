//! Types that are related to events that are raised on-chain and extracted from chain logs.
//!
//! These events happen in response to actions (transactions, smart contract calls) done by a HOPR node on chain.
//!
//! See `chain-actions` and `chain-indexer` crates for details.
use std::fmt::{Display, Formatter};

use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

/// Enumeration of HOPR chain events.
#[derive(Debug, Clone, PartialEq, strum::EnumTryAs, strum::EnumIs)]
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
    /// Ticket redemption on the node's own channel failed.
    RedeemFailed(ChannelEntry, String, Box<VerifiedTicket>),
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
            ChainEvent::RedeemFailed(c, r, _) => write!(f, "ticket redeem failed in channel {} due to {r}", c.get_id()),
        }
    }
}
