use std::sync::Arc;

use async_lock::RwLock;
use hopr_internal_types::protocol::TagBloomFilter;
use hopr_platform::file::native::{read_file, write};
use tracing::{debug, error, info};

#[derive(Debug, Clone)]
pub struct WrappedTagBloomFilter {
    path: String,
    tbf: Arc<RwLock<TagBloomFilter>>,
}

impl WrappedTagBloomFilter {
    pub fn new(path: String) -> Self {
        info!("Creating the Bloom filter storage at: {path}");
        let tbf = read_file(&path)
            .and_then(|data| {
                TagBloomFilter::from_bytes(&data)
                    .map_err(|e| hopr_platform::error::PlatformError::GeneralError(e.to_string()))
            })
            .unwrap_or_else(|_| {
                debug!("No tag Bloom filter found, using empty");
                TagBloomFilter::default()
            });

        Self {
            path,
            tbf: Arc::new(RwLock::new(tbf)),
        }
    }

    pub async fn with_write_lock<T>(&self, f: impl FnOnce(&mut TagBloomFilter) -> T) -> T {
        let mut tbf = self.tbf.write().await;
        f(&mut tbf)
    }

    pub async fn save(&self) {
        let bloom = self.tbf.read().await.clone(); // Clone to immediately release the lock

        if let Err(e) = write(&self.path, bloom.to_bytes()) {
            error!("Tag Bloom filter save failed: {e}")
        } else {
            info!("Tag Bloom filter saved successfully")
        };
    }
}
