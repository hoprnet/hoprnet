use std::{
    fmt::{Display, Formatter},
    time::{Duration, SystemTime},
};

use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;

/// Describes status of a channel
#[derive(Copy, Clone, Debug, smart_default::SmartDefault, strum::Display, strum::EnumDiscriminants)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[strum_discriminants(vis(pub))]
#[strum_discriminants(derive(strum::FromRepr, strum::EnumCount), repr(i8))]
#[strum(serialize_all = "PascalCase")]
pub enum ChannelStatus {
    /// The channel is closed.
    #[default]
    Closed,
    /// The channel is opened.
    Open,
    /// The channel is pending to be closed.
    /// The timestamp marks the *earliest* possible time when the channel can transition into the `Closed` state.
    #[strum(serialize = "PendingToClose")]
    PendingToClose(SystemTime),
}

// Cannot use #[repr(u8)] due to PendingToClose
impl From<ChannelStatus> for i8 {
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
                let diff = ct_1.max(ct_2).saturating_sub(*ct_1.min(ct_2));
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

/// Represents an ID of a [`ChannelEntry`].
///
/// The ID is created as `Keccak256(source || destination)` where `source` and `destination`
/// are two **different** [addresses](Address) of the channel's endpoints.
///
/// An attempt to create a `ChannelId` with the same source and destination will result in a panic.
/// It is the caller's responsibility to ensure that the source and destination addresses are different.
///
/// ## Example
/// ```rust
/// # use hopr_primitive_types::prelude::*;
/// # use hopr_internal_types::prelude::*;
/// # use hopr_crypto_types::prelude::*;
/// let source = Address::new(&[0; 20]);
/// let destination = Address::new(&[1; 20]);
/// let id: ChannelId = (source, destination).into();
///
/// assert_eq!(id, Hash::create(&[source.as_ref(), destination.as_ref()]));
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChannelId(Hash);

impl ChannelId {
    pub const SIZE: usize = Hash::SIZE;

    /// Creates channel ID from two `source` and `destination` objects that can be converted into [`Address`]
    pub fn from_addrs<A1: Into<Address>, A2: Into<Address>>(source: A1, destination: A2) -> Self {
        (source.into(), destination.into()).into()
    }
}

impl From<(&Address, &Address)> for ChannelId {
    fn from((source, destination): (&Address, &Address)) -> Self {
        assert_ne!(
            source, destination,
            "channel id: source and destination must not be equal"
        );
        Self(Hash::create(&[source.as_ref(), destination.as_ref()]))
    }
}

impl From<(Address, Address)> for ChannelId {
    fn from((source, destination): (Address, Address)) -> Self {
        (&source, &destination).into()
    }
}

impl AsRef<[u8]> for ChannelId {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsRef<Hash> for ChannelId {
    fn as_ref(&self) -> &Hash {
        &self.0
    }
}

impl From<Hash> for ChannelId {
    fn from(value: Hash) -> Self {
        Self(value)
    }
}

impl PartialEq<Hash> for ChannelId {
    fn eq(&self, other: &Hash) -> bool {
        &self.0 == other
    }
}

impl Display for ChannelId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl ToHex for ChannelId {
    fn to_hex(&self) -> String {
        self.0.to_hex()
    }

    fn from_hex(str: &str) -> hopr_primitive_types::errors::Result<Self>
    where
        Self: Sized,
    {
        str.parse().map(Self)
    }
}

/// Builder for [`ChannelEntry`].
///
/// The builder contains a channel entry that is always valid, therefore
/// the [`ChannelBuilder::build`] never fails.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ChannelBuilder(ChannelEntry);

impl ChannelBuilder {
    /// Creates a new builder for [`ChannelEntry`] from `source` to `destination`.
    ///
    /// ## Default values
    /// The remaining members of [`ChannelEntry`] are set as follows (unless modified by the builder):
    /// ```rust
    /// # use hopr_primitive_types::prelude::*;
    /// # use hopr_internal_types::prelude::*;
    ///
    /// let entry = ChannelBuilder::new(Address::new(&[0; 20]), Address::new(&[1; 20])).build();
    ///
    /// assert_eq!(entry.balance, HoprBalance::zero());
    /// assert_eq!(entry.status, ChannelStatus::Closed);
    /// assert_eq!(entry.ticket_index, 0);
    /// assert_eq!(entry.epoch, 1);
    /// ```
    pub fn new<U: Into<Address>, V: Into<Address>>(source: U, destination: V) -> Self {
        let source = source.into();
        let destination = destination.into();
        Self(ChannelEntry {
            id: (source, destination).into(),
            source,
            destination,
            balance: HoprBalance::zero(),
            ticket_index: 0,
            status: ChannelStatus::Closed,
            epoch: 1,
        })
    }

    /// Sets the balance via stake amount.
    ///
    /// See [`ChannelBuilder::with_balance`].
    #[must_use]
    pub fn with_stake<T: Into<U256>>(self, stake: T) -> Self {
        self.with_balance(HoprBalance::from(stake.into()))
    }

    /// Set the balance of the channel.
    ///
    /// Value is coerced between `0` and [`ChannelEntry::MAX_CHANNEL_BALANCE`].
    #[must_use]
    pub fn with_balance(mut self, balance: HoprBalance) -> Self {
        // Coerce to the maximum possible balance
        self.0.balance = balance.min(ChannelEntry::MAX_CHANNEL_BALANCE.into());
        self
    }

    /// Sets the index of the next ticket to be redeemed in the channel.
    ///
    /// Value is coerced between `0` and `2^48-1`.
    #[must_use]
    pub fn with_ticket_index(mut self, ticket_index: u64) -> Self {
        // Coerce to 48-bit unsigned integer
        self.0.ticket_index = ticket_index & 0x1ffffffffffff_u64;
        self
    }

    /// Sets the [status](ChannelStatus) of the channel.
    #[must_use]
    pub fn with_status(mut self, status: ChannelStatus) -> Self {
        self.0.status = status;
        self
    }

    /// Sets the epoch value of the channel.
    ///
    /// Value is coerced between `0` and `2^24-1`.
    #[must_use]
    pub fn with_epoch(mut self, epoch: u32) -> Self {
        // Coerce to 24-bit unsigned integer
        self.0.epoch = epoch.max(1) & 0x1ffffff_u32;
        self
    }

    /// Builds th [`ChannelEntry`].
    #[must_use]
    pub fn build(self) -> ChannelEntry {
        self.0
    }
}

impl From<ChannelEntry> for ChannelBuilder {
    fn from(entry: ChannelEntry) -> Self {
        Self(entry)
    }
}

impl From<ChannelBuilder> for ChannelEntry {
    fn from(builder: ChannelBuilder) -> Self {
        builder.0
    }
}

/// Overall description of a payment channel.
///
/// Use [`ChannelBuilder`] to create a new instance.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChannelEntry {
    /// Source [`Address`] of the channel.
    pub source: Address,
    /// Destination [`Address`] of the channel.
    pub destination: Address,
    /// Current balance of the channel.
    ///
    /// The maximum possible balance of a channel is 10^25 wxHOPR.
    pub balance: HoprBalance,
    /// Next ticket index to be redeemed from the channel.
    ///
    /// Starts always at 0, the maximum value is `2^48 - 1`.
    pub ticket_index: u64,
    /// Current [`ChannelStatus`] of this channel.
    pub status: ChannelStatus,
    /// Epoch (generation) of the channel.
    ///
    /// It is a counter of how many times the channel has been (re)opened.
    /// Since each channel's lifecycle starts by opening,
    /// the lowest value is 1, the maximum value is `2^24 - 1`.
    pub epoch: u32,
    id: ChannelId,
}

impl ChannelEntry {
    /// Maximum possible balance of a channel: 10^25 wxHOPR
    pub const MAX_CHANNEL_BALANCE: u128 = 10_u128.pow(25);

    /// Generates the channel ID using the source and destination address
    pub fn get_id(&self) -> &ChannelId {
        &self.id
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
            ChannelStatus::PendingToClose(closure_time) => Some(closure_time.saturating_sub(current_time)),
            ChannelStatus::Closed => Some(Duration::ZERO),
        }
    }

    /// Returns the earliest time the channel can transition from `PendingToClose` into `Closed`.
    /// If the channel is not in the ` PendingToClose ` state, it returns `None`.
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

/// Lists possible changes on a channel entry update
#[derive(Clone, Copy, Debug)]
pub enum ChannelChange {
    /// Channel status has changed
    Status { left: ChannelStatus, right: ChannelStatus },

    /// Channel balance has changed
    CurrentBalance { left: HoprBalance, right: HoprBalance },

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

        if left.epoch != right.epoch {
            ret.push(ChannelChange::Epoch {
                left: left.epoch,
                right: right.epoch,
            });
        }

        if left.ticket_index != right.ticket_index {
            ret.push(ChannelChange::TicketIndex {
                left: left.ticket_index,
                right: right.ticket_index,
            })
        }

        ret
    }
}

/// A wrapper around [`ChannelId`] representing a Channel that is corrupted.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CorruptedChannelEntry(ChannelId);

impl From<ChannelId> for CorruptedChannelEntry {
    fn from(value: ChannelId) -> Self {
        CorruptedChannelEntry(value)
    }
}

impl CorruptedChannelEntry {
    /// Returns the channel ID of the corrupted channel.
    pub fn channel_id(&self) -> &ChannelId {
        &self.0
    }
}

/// A pair of source and destination addresses representing a channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SrcDstPair(Address, Address);

impl From<ChannelEntry> for SrcDstPair {
    fn from(channel: ChannelEntry) -> Self {
        SrcDstPair(channel.source, channel.destination)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        ops::Add,
        str::FromStr,
        time::{Duration, SystemTime},
    };

    use hex_literal::hex;

    use super::*;

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).expect("lazy static keypair should be constructible");
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).expect("lazy static keypair should be constructible");

        static ref ADDRESS_1: Address = "3829b806aea42200c623c4d6b9311670577480ed".parse().expect("lazy static address should be constructible");
        static ref ADDRESS_2: Address = "1a34729c69e95d6e11c3a9b9be3ea0c62c6dc5b1".parse().expect("lazy static address should be constructible");
    }

    #[test]
    pub fn test_generate_id() -> anyhow::Result<()> {
        let from = Address::from_str("0xa460f2e47c641b64535f5f4beeb9ac6f36f9d27c")?;
        let to = Address::from_str("0xb8b75fef7efdf4530cf1688c933d94e4e519ccd1")?;
        let id: ChannelId = (&from, &to).into();
        assert_eq!(
            "0x1a410210ce7265f3070bf0e8885705dce452efcfbd90a5467525d136fcefc64a",
            id.to_string()
        );

        Ok(())
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
    fn channel_status_repr_compat() {
        assert_eq!(ChannelStatusDiscriminants::Open as i8, i8::from(ChannelStatus::Open));
        assert_eq!(
            ChannelStatusDiscriminants::Closed as i8,
            i8::from(ChannelStatus::Closed)
        );
        assert_eq!(
            ChannelStatusDiscriminants::PendingToClose as i8,
            i8::from(ChannelStatus::PendingToClose(SystemTime::now()))
        );
    }

    #[test]
    pub fn channel_entry_closure_time() {
        let mut ce = ChannelBuilder::new(*ADDRESS_1, *ADDRESS_2)
            .with_stake(10)
            .with_ticket_index(23)
            .with_status(ChannelStatus::Open)
            .with_epoch(3u32)
            .build();

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
