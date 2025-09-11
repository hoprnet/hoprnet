use std::{ops::Deref, sync::Arc};

use hopr_crypto_types::types::PacketTag;
use hopr_platform::file::native::{read_file, write};
use tracing::{debug, error};

use crate::raw::TagBloomFilter;

#[derive(Debug, Clone)]
pub struct WrappedTagBloomFilter {
    path: String,
    tbf: Arc<std::sync::RwLock<TagBloomFilter>>,
}

impl WrappedTagBloomFilter {
    const TAGBLOOM_BINCODE_CONFIGURATION: bincode::config::Configuration = bincode::config::standard()
        .with_little_endian()
        .with_variable_int_encoding();

    pub fn new(path: String) -> Self {
        let tbf = read_file(&path)
            .and_then(|data| {
                debug!(path = &path, "Found and loading a tag Bloom filter");
                bincode::serde::decode_from_slice(&data, Self::TAGBLOOM_BINCODE_CONFIGURATION)
                    .map(|(f, _)| f)
                    .map_err(|e| hopr_platform::error::PlatformError::GeneralError(e.to_string()))
            })
            .unwrap_or_else(|_| {
                debug!(path = &path, "No tag bloom filter found, using empty");
                TagBloomFilter::default()
            });

        Self {
            path,
            tbf: Arc::new(std::sync::RwLock::new(tbf)),
        }
    }

    /// Check whether the packet is replayed using a packet tag.
    ///
    /// There is a 0.1% chance that the positive result is not a replay because a Bloom filter is used.
    #[tracing::instrument(level = "trace", skip(self, tag))]
    pub fn is_tag_replay(&self, tag: &PacketTag) -> bool {
        if let Ok(mut tbf) = self.tbf.write() {
            tbf.check_and_set(tag)
        } else {
            // If the locking failed and the lock is poisoned,
            // we will always indicate a replay.
            error!("failed to acquire write lock on tag bloom filter");
            true
        }
    }

    pub fn save(&self) {
        // Lock the filter, clone it to release the lock immediately.
        let bloom = self.tbf.read().map(|tbf| tbf.deref().clone()).ok();

        if let Some(bloom) = bloom {
            if let Err(error) = bincode::serde::encode_to_vec(&bloom, Self::TAGBLOOM_BINCODE_CONFIGURATION)
                .map_err(|e| hopr_platform::error::PlatformError::GeneralError(e.to_string()))
                .and_then(|d| write(&self.path, &d))
            {
                error!(%error, "tag bloom filter save failed")
            } else {
                debug!("tag bloom filter saved successfully")
            };
        } else {
            error!("failed to acquire read lock on tag bloom filter");
        }
    }
}
