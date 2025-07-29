use std::sync::Arc;

use async_lock::RwLock;
use hopr_crypto_types::types::PacketTag;
use hopr_platform::file::native::{read_file, write};
use tracing::{debug, error};

use crate::raw::TagBloomFilter;

#[derive(Debug, Clone)]
pub struct WrappedTagBloomFilter {
    path: String,
    tbf: Arc<RwLock<TagBloomFilter>>,
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
                debug!(path = &path, "No tag Bloom filter found, using empty");
                TagBloomFilter::default()
            });

        Self {
            path,
            tbf: Arc::new(RwLock::new(tbf)),
        }
    }

    /// Check whether the packet is replayed using a packet tag.
    ///
    /// There is a 0.1% chance that the positive result is not a replay because a Bloom filter is used.
    #[tracing::instrument(level = "trace", skip(self, tag))]
    pub async fn is_tag_replay(&self, tag: &PacketTag) -> bool {
        self.with_write_lock(|inner: &mut TagBloomFilter| inner.check_and_set(tag))
            .await
    }

    pub async fn with_write_lock<T>(&self, f: impl FnOnce(&mut TagBloomFilter) -> T) -> T {
        let mut tbf = self.tbf.write_arc().await;
        f(&mut tbf)
    }

    pub async fn save(&self) {
        let bloom = self.tbf.read_arc().await.clone(); // Clone to immediately release the lock

        if let Err(e) = bincode::serde::encode_to_vec(&bloom, Self::TAGBLOOM_BINCODE_CONFIGURATION)
            .map_err(|e| hopr_platform::error::PlatformError::GeneralError(e.to_string()))
            .and_then(|d| write(&self.path, &d))
        {
            error!(error = %e, "Tag Bloom filter save failed")
        } else {
            debug!("Tag Bloom filter saved successfully")
        };
    }
}
