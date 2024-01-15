use hopr_internal_types::acknowledgement::AcknowledgedTicket;
use hopr_internal_types::announcement::AnnouncementData;
use hopr_internal_types::channels::{ChannelDirection, ChannelEntry};
use hopr_primitive_types::primitives::{Address, Balance};
use std::fmt::{Display, Formatter};

/// Enumerates all possible on-chain state change requests
#[derive(Clone, PartialEq, Debug, strum::EnumVariantNames, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Action {
    /// Redeem the given acknowledged ticket
    RedeemTicket(AcknowledgedTicket),

    /// Open channel to the given destination with the given stake
    OpenChannel(Address, Balance),

    /// Fund channel with the given ID and amount
    FundChannel(ChannelEntry, Balance),

    /// Close channel with the given source and destination
    CloseChannel(ChannelEntry, ChannelDirection),

    /// Withdraw given balance to the given address
    Withdraw(Address, Balance),

    /// Announce node on-chain
    Announce(AnnouncementData),

    /// Register safe address with this node
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
            Action::Announce(data) => write!(f, "announce action of {}", data.to_multiaddress_str()),
            Action::RegisterSafe(safe_address) => write!(f, "register safe action {safe_address}"),
        }
    }
}
