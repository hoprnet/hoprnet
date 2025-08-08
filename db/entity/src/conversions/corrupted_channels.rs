use hopr_internal_types::{channels::ChannelId, corrupted_channels::CorruptedChannelEntry};
use hopr_primitive_types::prelude::ToHex;
use sea_orm::Set;

use crate::{corrupted_channel, errors::DbEntityError};

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
