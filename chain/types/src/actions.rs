//! Types related to high-level actions that lead to Ethereum transactions.
//!
//! Defines an enumeration of action that can be done by a HOPR node.
//!
//! The actions eventually lead to an on-chain transaction.
//!
//! See the `chain-actions` crate for details.
use std::fmt::{Display, Formatter};

use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

/// Enumerates all possible on-chain state change requests.
///
/// An `Action` is an operation done by the HOPR node that leads
/// to an on-chain transaction or a contract call. An `Action` is considered complete
/// until the corresponding [SignificantChainEvent](crate::chain_events::SignificantChainEvent)
/// is registered by the Indexer or a timeout.
#[allow(clippy::large_enum_variant)] // TODO: Refactor the large enum variant
#[derive(Clone, PartialEq, Debug, strum::VariantNames, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Action {
    /// Redeem the given acknowledged ticket.
    RedeemTicket(RedeemableTicket),

    /// Open a channel to the given destination with the given stake
    OpenChannel(Address, HoprBalance),

    /// Fund channel with the given ID and amount
    FundChannel(ChannelEntry, HoprBalance),

    /// Close channel with the given source and destination
    CloseChannel(ChannelEntry, ChannelDirection),

    /// Withdraw the given balance to the given address
    Withdraw(Address, HoprBalance),

    /// Withdraw the given native balance to the given address
    WithdrawNative(Address, XDaiBalance),

    /// Announce node on-chain
    Announce(AnnouncementData, U256),

    /// Register a safe address with this node
    RegisterSafe(Address),
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::RedeemTicket(ack) => write!(f, "redeem action of {ack}"),
            Action::OpenChannel(dst, amount) => write!(f, "open channel action to {dst} with {amount}"),
            Action::FundChannel(channel, amount) => write!(
                f,
                "fund channel action for channel from {} to {} with {amount}",
                channel.source, channel.destination
            ),
            Action::CloseChannel(channel, direction) => write!(
                f,
                "closure action of {} channel from {} to {}",
                direction, channel.source, channel.destination
            ),
            Action::Withdraw(destination, amount) => write!(f, "withdraw action of {amount} to {destination}"),
            Action::WithdrawNative(destination, amount) => {
                write!(f, "withdraw native action of {amount} to {destination}")
            }
            Action::Announce(data, key_binding_fee) => write!(
                f,
                "announce action of {} with key bound fee {}",
                data.multiaddress(),
                key_binding_fee
            ),
            Action::RegisterSafe(safe_address) => write!(f, "register safe action {safe_address}"),
        }
    }
}
