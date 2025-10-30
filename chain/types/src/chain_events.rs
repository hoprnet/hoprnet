//! Types that are related to events that are raised on-chain and extracted from chain logs.
//!
//! These events happen in response to actions (transactions, smart contract calls) done by a HOPR node on chain.
//!
//! See `chain-actions` and `chain-indexer` crates for details.
use std::fmt::{Display, Formatter};

use hopr_crypto_types::types::Hash;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use libp2p_identity::PeerId;
use multiaddr::Multiaddr;

/// Enumeration of HOPR chain events.
#[derive(Debug, Clone, PartialEq, strum::EnumTryAs)]
pub enum ChainEvent {
    /// Peer on-chain announcement event.
    Announcement {
        /// Announced peer id
        peer: PeerId,
        /// Announced on-chain address
        address: Address,
        /// Multiaddresses
        multiaddresses: Vec<Multiaddr>,
    },
    /// A new channel has been opened
    ChannelOpened(ChannelEntry),
    /// Channel closure has been initiated.
    ChannelClosureInitiated(ChannelEntry),
    /// Channel closure has been finalized.
    ChannelClosed(ChannelEntry),
    /// Channel balance has increased by an amount.
    ChannelBalanceIncreased(ChannelEntry, HoprBalance),
    /// Channel balance has decreased by an amount.
    ChannelBalanceDecreased(ChannelEntry, HoprBalance),
    /// Ticket has been redeemed on a channel.
    ///
    /// If the channel is a node's own, it also contains the ticket that has been redeemed.
    TicketRedeemed(ChannelEntry, Option<Box<VerifiedTicket>>),
    /// Ticket redemption on the node's own channel failed.
    RedeemFailed(ChannelEntry, Box<VerifiedTicket>),
    /// Safe has been registered with the node.
    NodeSafeRegistered(Address),
}

impl Display for ChainEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainEvent::Announcement {
                peer,
                address,
                multiaddresses,
            } => write!(f, "announcement event of {peer} ({address}): {multiaddresses:?}"),
            ChainEvent::ChannelOpened(c) => write!(f, "open channel event {}", c.get_id()),
            ChainEvent::ChannelClosureInitiated(c) => write!(f, "close channel initiation event {}", c.get_id()),
            ChainEvent::ChannelClosed(c) => write!(f, "close channel event {}", c.get_id()),
            ChainEvent::ChannelBalanceIncreased(c, _) => write!(f, "channel increase balance event {}", c.get_id()),
            ChainEvent::ChannelBalanceDecreased(c, _) => write!(f, "channel decrease balance event {}", c.get_id()),
            ChainEvent::TicketRedeemed(c, _) => write!(f, "ticket redeem event in channel {}", c.get_id()),
            ChainEvent::NodeSafeRegistered(s) => write!(f, "safe registered event {s}"),
            ChainEvent::RedeemFailed(c, _) => write!(f, "ticket redeem failed in channel {}", c.get_id()),
        }
    }
}