use hopr_internal_types::{
    channels::{ChannelStatus, CorruptedChannelEntry},
    prelude::ChannelEntry,
};
use hopr_primitive_types::{
    balance::HoprBalance,
    prelude::{IntoEndian, ToHex, U256},
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
            return Err(DbEntityError::ConversionError(
                "cannot convert corrupted channel model to ChannelEntry".into(),
            ));
        }
        match model_to_channel_entry_unchecked(&value) {
            Ok(channel_entry) => Ok(channel_entry),
            Err(e) => Err(e),
        }
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
        match model_to_channel_entry_unchecked(&value) {
            Ok(channel_entry) => Ok(CorruptedChannelEntry::new(channel_entry)),
            Err(e) => Err(e),
        }
    }
}

impl TryFrom<channel::Model> for CorruptedChannelEntry {
    type Error = DbEntityError;

    fn try_from(value: channel::Model) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl From<CorruptedChannelEntry> for channel::ActiveModel {
    fn from(value: CorruptedChannelEntry) -> Self {
        let mut ret = channel::ActiveModel::from(value.channel().clone());
        ret.corrupted = Set(true);
        ret
    }
}

fn model_to_channel_entry_unchecked(model: &channel::Model) -> Result<ChannelEntry, DbEntityError> {
    Ok(ChannelEntry::new(
        model.source.parse()?,
        model.destination.parse()?,
        HoprBalance::from(U256::from_be_bytes(&model.balance)),
        U256::from_be_bytes(&model.ticket_index),
        model.try_into()?,
        U256::from_be_bytes(&model.epoch),
    ))
}
