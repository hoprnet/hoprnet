use hopr_internal_types::{
    channels::{ChannelId, ChannelStatus, CorruptedChannelEntry},
    prelude::{ChannelBuilder, ChannelEntry},
};
use hopr_primitive_types::prelude::{Address, IntoEndian, ToHex, U256};
use sea_orm::Set;

use crate::{channel, corrupted_channel, errors::DbEntityError};

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
        Ok(
            ChannelBuilder::new(value.source.parse::<Address>()?, value.destination.parse::<Address>()?)
                .with_stake(U256::from_be_bytes(&value.balance))
                .with_ticket_index(u64::from_be_bytes(
                    value.ticket_index[0..size_of::<u64>()]
                        .try_into()
                        .map_err(|_| DbEntityError::ConversionError("invalid ticket index".into()))?,
                ))
                .with_epoch(u32::from_be_bytes(
                    value.epoch[0..size_of::<u32>()]
                        .try_into()
                        .map_err(|_| DbEntityError::ConversionError("invalid epoch".into()))?,
                ))
                .with_status(value.try_into()?)
                .build(),
        )
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
            epoch: Set(value.epoch.to_be_bytes().into()),
            ticket_index: Set(value.ticket_index.to_be_bytes().into()),
            ..Default::default()
        };
        ret.set_status(value.status);
        ret
    }
}

impl TryFrom<&corrupted_channel::Model> for CorruptedChannelEntry {
    type Error = DbEntityError;

    fn try_from(value: &corrupted_channel::Model) -> Result<Self, Self::Error> {
        let channel_id = ChannelId::from_hex(value.channel_id.as_str())
            .map_err(|_| DbEntityError::ConversionError("invalid channel ID".into()))?;

        Ok(channel_id.into())
    }
}

impl TryFrom<corrupted_channel::Model> for CorruptedChannelEntry {
    type Error = DbEntityError;

    fn try_from(value: corrupted_channel::Model) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl From<CorruptedChannelEntry> for corrupted_channel::ActiveModel {
    fn from(value: CorruptedChannelEntry) -> Self {
        corrupted_channel::ActiveModel {
            channel_id: Set(value.channel_id().to_hex()),
            ..Default::default()
        }
    }
}
