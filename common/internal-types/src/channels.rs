use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use std::{
    fmt::{Display, Formatter},
};

/// Describes status of a channel
#[derive(Copy, Clone, Debug, smart_default::SmartDefault, Serialize, Deserialize, strum::Display)]
#[strum(serialize_all = "PascalCase")]
pub enum ChannelStatus {
    /// Channel is closed.
    #[default]
    Closed,
    /// Channel is opened.
    Open,
    /// Channel is pending to be closed.
    /// The timestamp marks the *earliest* possible time when the channel can transition into the `Closed` state.
    #[strum(serialize = "PendingToClose")]
    PendingToClose(SystemTime),
}

// Cannot use #[repr(u8)] due to PendingToClose
impl From<ChannelStatus> for u8 {
    fn from(value: ChannelStatus) -> Self {
        match value {
            ChannelStatus::Closed => 0,
            ChannelStatus::Open => 1,
            ChannelStatus::PendingToClose(_) => 2,
        }
    }
}

// Manual implementation of PartialEq, because we need only precision up to seconds in PendingToClose
impl PartialEq for ChannelStatus {
    fn eq(&self, other: &Self) -> bool {
        // Use pattern matching to avoid recursion
        match (self, other) {
            (Self::Open, Self::Open) => true,
            (Self::Closed, Self::Closed) => true,
            (Self::PendingToClose(ct_1), Self::PendingToClose(ct_2)) => {
                let diff = ct_1.max(ct_2).duration_since(*ct_1.min(ct_2)).unwrap();
                diff.as_secs() == 0
            }
            _ => false,
        }
    }
}
impl Eq for ChannelStatus {}

/// Describes a direction of node's own channel.
/// The direction of a channel that is not own is undefined.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::Display, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum ChannelDirection {
    /// The other party is initiator of the channel.
    Incoming = 0,
    /// Our own node is the initiator of the channel.
    Outgoing = 1,
}

/// Overall description of a channel
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChannelEntry {
    pub source: Address,
    pub destination: Address,
    pub balance: Balance,
    pub ticket_index: U256,
    pub status: ChannelStatus,
    pub channel_epoch: U256,
    id: Hash,
}

impl ChannelEntry {
    pub fn new(
        source: Address,
        destination: Address,
        balance: Balance,
        ticket_index: U256,
        status: ChannelStatus,
        channel_epoch: U256,
    ) -> Self {
        assert_eq!(BalanceType::HOPR, balance.balance_type(), "invalid balance currency");
        ChannelEntry {
            source,
            destination,
            balance,
            ticket_index,
            status,
            channel_epoch,
            id: generate_channel_id(&source, &destination),
        }
    }

    /// Generates the channel ID using the source and destination address
    pub fn get_id(&self) -> Hash {
        self.id
    }

    /// Checks if the closure time of this channel has passed.
    /// Also returns `false` if the channel closure has not been initiated (it is in `Open` state).
    /// Returns also `true`, if the channel is in `Closed` state.
    pub fn closure_time_passed(&self, current_time: SystemTime) -> bool {
        match self.status {
            ChannelStatus::Open => false,
            ChannelStatus::PendingToClose(closure_time) => closure_time <= current_time,
            ChannelStatus::Closed => true,
        }
    }

    /// Calculates the remaining channel closure grace period.
    /// Returns `None` if the channel closure has not been initiated yet (channel is in `Open` state).
    pub fn remaining_closure_time(&self, current_time: SystemTime) -> Option<Duration> {
        match self.status {
            ChannelStatus::Open => None,
            ChannelStatus::PendingToClose(closure_time) => {
                Some(closure_time.duration_since(current_time).unwrap_or(Duration::ZERO))
            }
            ChannelStatus::Closed => Some(Duration::ZERO),
        }
    }

    /// Returns the earliest time the channel can transition from `PendingToClose` into `Closed`.
    /// If the channel is not in `PendingToClose` state, returns `None`.
    pub fn closure_time_at(&self) -> Option<SystemTime> {
        match self.status {
            ChannelStatus::PendingToClose(ct) => Some(ct),
            _ => None,
        }
    }

    /// Determines the channel direction given the self address.
    /// Returns `None` if neither source nor destination are equal to `me`.
    pub fn direction(&self, me: &Address) -> Option<ChannelDirection> {
        if self.source.eq(me) {
            Some(ChannelDirection::Outgoing)
        } else if self.destination.eq(me) {
            Some(ChannelDirection::Incoming)
        } else {
            None
        }
    }

    /// Determines the channel's direction and counterparty relative to `me`.
    /// Returns `None` if neither source nor destination are equal to `me`.
    pub fn orientation(&self, me: &Address) -> Option<(ChannelDirection, Address)> {
        if self.source.eq(me) {
            Some((ChannelDirection::Outgoing, self.destination))
        } else if self.destination.eq(me) {
            Some((ChannelDirection::Incoming, self.source))
        } else {
            None
        }
    }
}

impl Display for ChannelEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} channel {}", self.status, self.get_id(),)
    }
}

/// Generates channel ID hash from `source` and `destination` addresses.
pub fn generate_channel_id(source: &Address, destination: &Address) -> Hash {
    Hash::create(&[source.as_ref(), destination.as_ref()])
}

/// Enumerates possible changes on a channel entry update
#[derive(Clone, Copy, Debug)]
pub enum ChannelChange {
    /// Channel status has changed
    Status { left: ChannelStatus, right: ChannelStatus },

    /// Channel balance has changed
    CurrentBalance { left: Balance, right: Balance },

    /// Channel epoch has changed
    Epoch { left: u32, right: u32 },

    /// Ticket index has changed
    TicketIndex { left: u64, right: u64 },
}

impl Display for ChannelChange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelChange::Status { left, right } => {
                write!(f, "Status: {left} -> {right}")
            }

            ChannelChange::CurrentBalance { left, right } => {
                write!(f, "Balance: {left} -> {right}")
            }

            ChannelChange::Epoch { left, right } => {
                write!(f, "Epoch: {left} -> {right}")
            }

            ChannelChange::TicketIndex { left, right } => {
                write!(f, "TicketIndex: {left} -> {right}")
            }
        }
    }
}

impl ChannelChange {
    /// Compares the two given channels and returns a vector of `ChannelChange`s
    /// Both channels must have the same ID (source,destination and direction) to be comparable using this function.
    /// The function panics if `left` and `right` do not have equal ids.
    /// Note that only some fields are tracked for changes, and therefore an empty vector returned
    /// does not imply the two `ChannelEntry` instances are equal.
    pub fn diff_channels(left: &ChannelEntry, right: &ChannelEntry) -> Vec<Self> {
        assert_eq!(left.id, right.id, "must have equal ids"); // misuse
        let mut ret = Vec::with_capacity(4);
        if left.status != right.status {
            ret.push(ChannelChange::Status {
                left: left.status,
                right: right.status,
            });
        }

        if left.balance != right.balance {
            ret.push(ChannelChange::CurrentBalance {
                left: left.balance,
                right: right.balance,
            });
        }

        if left.channel_epoch != right.channel_epoch {
            ret.push(ChannelChange::Epoch {
                left: left.channel_epoch.as_u32(),
                right: right.channel_epoch.as_u32(),
            });
        }

        if left.ticket_index != right.ticket_index {
            ret.push(ChannelChange::TicketIndex {
                left: left.ticket_index.as_u64(),
                right: right.ticket_index.as_u64(),
            })
        }

        ret
    }
}

#[cfg(test)]
pub mod tests {
    use crate::channels::{generate_channel_id, ChannelEntry, ChannelStatus};
    use hex_literal::hex;
    use hopr_crypto_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use std::ops::Add;
    use std::str::FromStr;
    use std::time::{Duration, SystemTime};
    use crate::tickets::{f64_to_win_prob, Ticket};

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).unwrap();
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).unwrap();

        static ref ADDRESS_1: Address = "3829b806aea42200c623c4d6b9311670577480ed".parse().unwrap();
        static ref ADDRESS_2: Address = "1a34729c69e95d6e11c3a9b9be3ea0c62c6dc5b1".parse().unwrap();
    }

    #[test]
    pub fn test_generate_id() {
        let from = Address::from_str("0xa460f2e47c641b64535f5f4beeb9ac6f36f9d27c").unwrap();
        let to = Address::from_str("0xb8b75fef7efdf4530cf1688c933d94e4e519ccd1").unwrap();
        let id = generate_channel_id(&from, &to).to_string();
        assert_eq!("0x1a410210ce7265f3070bf0e8885705dce452efcfbd90a5467525d136fcefc64a", id);
    }

    #[test]
    fn channel_status_names() {
        assert_eq!("Open", ChannelStatus::Open.to_string());
        assert_eq!("Closed", ChannelStatus::Closed.to_string());
        assert_eq!(
            "PendingToClose",
            ChannelStatus::PendingToClose(SystemTime::now()).to_string()
        );
    }

    #[test]
    pub fn channel_entry_closure_time() {
        let mut ce = ChannelEntry::new(
            *ADDRESS_1,
            *ADDRESS_2,
            Balance::new(10_u64, BalanceType::HOPR),
            23u64.into(),
            ChannelStatus::Open,
            3u64.into(),
        );

        assert!(
            !ce.closure_time_passed(SystemTime::now()),
            "opened channel cannot pass closure time"
        );
        assert!(
            ce.remaining_closure_time(SystemTime::now()).is_none(),
            "opened channel cannot have remaining closure time"
        );

        let current_time = SystemTime::now();
        ce.status = ChannelStatus::PendingToClose(current_time.add(Duration::from_secs(60)));

        assert!(
            !ce.closure_time_passed(current_time),
            "must not have passed closure time"
        );
        assert_eq!(
            60,
            ce.remaining_closure_time(current_time)
                .expect("must have closure time")
                .as_secs()
        );

        let current_time = current_time.add(Duration::from_secs(120));

        assert!(ce.closure_time_passed(current_time), "must have passed closure time");
        assert_eq!(
            Duration::ZERO,
            ce.remaining_closure_time(current_time).expect("must have closure time")
        );
    }
}