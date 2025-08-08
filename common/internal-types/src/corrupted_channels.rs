use crate::prelude::ChannelId;

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
