use hopr_crypto_types::types::Hash;
use hopr_internal_types::{
    channels::{ChannelId, ChannelStatus, CorruptedChannelEntry},
    prelude::ChannelEntry,
};
use hopr_primitive_types::{
    balance::HoprBalance,
    prelude::{Address, IntoEndian, ToHex, U256},
};
use sea_orm::Set;

use crate::{channel, errors::DbEntityError};

/// Extension trait for updating [ChannelStatus] inside [channel::ActiveModel].
/// This is needed as `status` maps to two model members.
pub trait ChannelStatusUpdate {
    /// Update [ChannelStatus] of this active model.
    fn set_status(&mut self, new_status: ChannelStatus);
}

impl ChannelStatusUpdate for channel::ActiveModel {
    fn set_status(&mut self, new_status: ChannelStatus) {
        self.status = Set(i8::from(new_status));
        if let ChannelStatus::PendingToClose(t) = new_status {
            self.closure_time = Set(Some(chrono::DateTime::<chrono::Utc>::from(t)))
        }
    }
}

impl TryFrom<&channel::Model> for ChannelStatus {
    type Error = DbEntityError;

    fn try_from(value: &channel::Model) -> Result<Self, Self::Error> {
        match value.status {
            0 => Ok(ChannelStatus::Closed),
            1 => Ok(ChannelStatus::Open),
            2 => value
                .closure_time
                .ok_or(DbEntityError::ConversionError(
                    "channel is pending to close but without closure time".into(),
                ))
                .map(|time| ChannelStatus::PendingToClose(time.into())),
            _ => Err(DbEntityError::ConversionError("invalid channel status value".into())),
        }
    }
}

impl TryFrom<&channel::Model> for ChannelEntry {
    type Error = DbEntityError;

    fn try_from(value: &channel::Model) -> Result<Self, Self::Error> {
        if value.corrupted {
            return Err(DbEntityError::InvalidCorruptionFlag(
                "cannot convert corrupted channel model to ChannelEntry".into(),
            ));
        }

        Ok(ChannelEntry::new(
            value.source.parse()?,
            value.destination.parse()?,
            HoprBalance::from(U256::from_be_bytes(&value.balance)),
            U256::from_be_bytes(&value.ticket_index),
            value.try_into()?,
            U256::from_be_bytes(&value.epoch),
        ))
    }
}

impl TryFrom<channel::Model> for ChannelEntry {
    type Error = DbEntityError;

    fn try_from(value: channel::Model) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl From<ChannelEntry> for channel::ActiveModel {
    fn from(value: ChannelEntry) -> Self {
        let mut ret = channel::ActiveModel {
            channel_id: Set(value.get_id().to_hex()),
            source: Set(value.source.to_hex()),
            destination: Set(value.destination.to_hex()),
            balance: Set(value.balance.amount().to_be_bytes().into()),
            epoch: Set(value.channel_epoch.to_be_bytes().into()),
            ticket_index: Set(value.ticket_index.to_be_bytes().into()),
            corrupted: Set(false),
            ..Default::default()
        };
        ret.set_status(value.status);
        ret
    }
}

impl TryFrom<&channel::Model> for CorruptedChannelEntry {
    type Error = DbEntityError;

    fn try_from(value: &channel::Model) -> Result<Self, Self::Error> {
        if !value.corrupted {
            return Err(DbEntityError::ConversionError(
                "cannot convert non-corrupted channel model to CorruptedChannelEntry".into(),
            ));
        }

        let channel_id = Hash::from_hex(value.channel_id.as_str())
            .map_err(|_| DbEntityError::ConversionError("invalid channel ID".into()))?;

        Ok(channel_id.into())
    }
}

impl TryFrom<channel::Model> for CorruptedChannelEntry {
    type Error = DbEntityError;

    fn try_from(value: channel::Model) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl From<ChannelId> for channel::ActiveModel {
    fn from(value: ChannelId) -> Self {
        channel::ActiveModel {
            channel_id: Set(value.to_hex()),
            source: Set(Address::default().to_hex()), // Default to empty address
            destination: Set(Address::default().to_hex()), // Default to empty address
            balance: Set(HoprBalance::default().amount().to_be_bytes().into()), //
            status: Set(i8::from(ChannelStatus::Open)), // Default to Open status
            ..Default::default()
        }
    }
}

impl From<CorruptedChannelEntry> for channel::ActiveModel {
    fn from(value: CorruptedChannelEntry) -> Self {
        let mut ret = channel::ActiveModel::from(value.channel_id().clone());
        ret.corrupted = Set(true);
        ret
    }
}
