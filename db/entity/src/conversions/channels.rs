use std::str::FromStr;
use sea_orm::{Set};
use hopr_internal_types::channels::ChannelStatus;
use hopr_internal_types::prelude::ChannelEntry;
use hopr_primitive_types::prelude::{BalanceType, IntoEndian, ToHex, U256};
use crate::channel;
use crate::errors::DbEntityError;

/// Extension trait for updating [ChannelStatus] inside [channel::ActiveModel].
/// This is needed as `status` maps to two model members.
pub trait ChannelStatusUpdate {
    /// Update [ChannelStatus] of this active model.
    fn set_status(&mut self, new_status: ChannelStatus);
}

impl ChannelStatusUpdate for channel::ActiveModel {
    fn set_status(&mut self, new_status: ChannelStatus) {
        self.status = Set(u8::from(new_status) as i32);
        if let ChannelStatus::PendingToClose(t) = new_status {
            self.closure_time = Set(Some(chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339()))
        }
    }
}

impl TryFrom<&channel::Model> for ChannelStatus {
    type Error = DbEntityError;

    fn try_from(value: &channel::Model) -> Result<Self, Self::Error> {
        match value.status {
            0 => Ok(ChannelStatus::Closed),
            1 => Ok(ChannelStatus::Open),
            2 => {
                if let Some(ct) = &value.closure_time {
                    let time = chrono::DateTime::<chrono::Utc>::from_str(ct).map_err(|_| DbEntityError::ConversionError("channel closure time".into()))?;
                    Ok(ChannelStatus::PendingToClose(time.into()))
                } else {
                    Err(DbEntityError::ConversionError("channel is pending to close but without closure time".into()))
                }
            }
            _ => Err(DbEntityError::ConversionError("invalid channel status value".into())),
        }
    }
}

impl TryFrom<&channel::Model> for ChannelEntry {
    type Error = DbEntityError;

    fn try_from(value: &channel::Model) -> Result<Self, Self::Error> {
        let status = value.try_into()?;
        Ok(ChannelEntry::new(
            value.source.parse()?,
            value.destination.parse()?,
            BalanceType::HOPR.balance_bytes(&value.balance),
            U256::from_be_bytes(&value.ticket_index),
            status,
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
            ..Default::default()
        };
        ret.set_status(value.status);
        ret
    }
}