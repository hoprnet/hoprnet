use std::path::Path;

/// Configuration for the chain indexer functionality
#[derive(Debug, Clone, smart_default::SmartDefault)]
pub struct IndexerConfig {
    /// The block at which the indexer should start
    ///
    /// It typically makes little sense to start indexing from the beginning
    /// of the chain; all that is sufficient is to start indexing since the
    /// relevant smart contracts were introduced into the chain.
    ///
    /// This value makes sure that indexing is relevant and as minimal as possible.
    ///
    /// Default is `0`.
    pub start_block_number: u64,

    /// Whether to use fast synchronization during indexing.
    /// When enabled, it allows for quicker indexing of existing logs during node startup.
    ///
    /// Default is `true`.
    #[default(true)]
    pub fast_sync: bool,

    /// Whether to perform logs snapshot download on startup.
    /// When enabled, it allows for quicker indexing from scratch.
    ///
    /// Default is `false`.
    #[default(false)]
    pub enable_logs_snapshot: bool,

    /// URL to download logs snapshot from.
    /// This should point to a publicly accessible tar.xz file containing
    /// the SQLite logs database files.
    ///
    /// Default is None
    #[default(None)]
    pub logs_snapshot_url: Option<String>,

    /// Path to the data directory where databases are stored.
    /// This is used for snapshot installation and database state checking.
    ///
    /// Default is empty string (must be set by application).
    pub data_directory: String,
}

impl IndexerConfig {
    /// Creates a new indexer configuration.
    ///
    /// # Arguments
    ///
    /// * `start_block_number` - The block number from which to start indexing
    /// * `fast_sync` - Whether to enable fast synchronization during startup
    /// * `enable_logs_snapshot` - Whether to enable logs snapshot download
    /// * `logs_snapshot_url` - URL to download logs snapshot from
    /// * `data_directory` - Path to the data directory where databases are stored
    ///
    /// # Returns
    ///
    /// A new instance of `IndexerConfig`
    pub fn new(
        start_block_number: u64,
        fast_sync: bool,
        enable_logs_snapshot: bool,
        logs_snapshot_url: Option<String>,
        data_directory: String,
    ) -> Self {
        Self {
            start_block_number,
            fast_sync,
            enable_logs_snapshot,
            logs_snapshot_url,
            data_directory,
        }
    }

    /// Validates the configuration and returns any validation errors.
    ///
    /// Performs comprehensive validation of configuration parameters including:
    /// - URL format and protocol validation (HTTP/HTTPS/file:// supported)
    /// - File extension validation (.tar.xz required)
    /// - Data directory path validation
    /// - Dependency validation (data directory required when snapshots enabled)
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all validation passes
    /// - `Err(String)` with a descriptive error message if validation fails
    ///
    /// # Example
    ///
    /// ```
    /// use hopr_chain_indexer::IndexerConfig;
    ///
    /// let config = IndexerConfig::new(
    ///     100,
    ///     true,
    ///     true,
    ///     Some("https://example.com/snapshot.tar.xz".to_string()),
    ///     "/tmp/hopr_data".to_string(),
    /// );
    ///
    /// assert!(config.validate().is_ok());
    /// ```
    pub fn validate(&self) -> Result<(), String> {
        // Validate URL format if snapshot URL is enabled
        if self.enable_logs_snapshot {
            if self.logs_snapshot_url.is_none() || self.logs_snapshot_url.as_ref().unwrap().is_empty() {
                return Err("Logs snapshot URL must be specified when logs snapshots are enabled".to_string());
            }

            if let Some(url) = &self.logs_snapshot_url {
                if url.is_empty() {
                    return Err("Logs snapshot URL must not be empty".to_string());
                }

                // Basic URL validation (allow file:// for testing)
                if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("file://") {
                    return Err("Logs snapshot URL must be a valid HTTP, HTTPS, or file:// URL".to_string());
                }

                // Check if URL ends with .tar.xz
                if !url.ends_with(".tar.xz") {
                    return Err(format!(
                        "Logs snapshot URL must point to a .tar.xz file. {url} is incorrect"
                    ));
                }
            }
        }

        // Validate data directory if snapshot is enabled
        if self.enable_logs_snapshot && self.data_directory.is_empty() {
            return Err("Data directory must be specified when logs snapshots are enabled".to_string());
        }

        // Validate data directory path
        if !self.data_directory.is_empty() {
            let path = Path::new(&self.data_directory);
            if path.is_file() {
                return Err("Data directory path points to a file, not a directory".to_string());
            }
        }

        Ok(())
    }

    /// Convenience method to check if the configuration is valid.
    ///
    /// This is a simple wrapper around `validate()` that returns a boolean
    /// instead of a `Result`, making it easier to use in conditional expressions.
    ///
    /// # Returns
    ///
    /// `true` if all validation passes, `false` otherwise
    ///
    /// # Example
    ///
    /// ```
    /// use hopr_chain_indexer::IndexerConfig;
    ///
    /// let config = IndexerConfig::default();
    /// if !config.is_valid() {
    ///     // Handle invalid configuration
    /// }
    /// ```
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_valid_config() {
        let data_directory = "/tmp/hopr_test_data";
        let logs_snapshot_url = format!("file:///tmp/snapshot.tar.xz");

        let cfg = IndexerConfig::new(0, true, true, Some(logs_snapshot_url), data_directory.into());

        cfg.validate().expect("Failed to validate snapshot configuration");
        assert!(cfg.is_valid(), "Valid configuration should return true for is_valid()");
    }

    #[test]
    fn test_invalid_url_config() {
        let data_directory = "/tmp/hopr_test_data";
        let logs_snapshot_url = format!("ftp://invalid.url/snapshot.tar.xz");

        let cfg = IndexerConfig::new(0, true, true, Some(logs_snapshot_url), data_directory.into());

        assert!(cfg.validate().is_err(), "Invalid URL should fail validation");
        assert!(
            !cfg.is_valid(),
            "Invalid configuration should return false for is_valid()"
        );
    }

    #[test]
    fn test_empty_dir_config() {
        let logs_snapshot_url = format!("ftp://invalid.url/snapshot.tar.xz");

        let cfg = IndexerConfig::new(0, true, true, Some(logs_snapshot_url), "".to_string());

        assert!(
            cfg.validate().is_err(),
            "Empty data directory should fail validation when snapshots enabled"
        );
    }

    #[test]
    fn test_disabled_snapshot_config() {
        let cfg = IndexerConfig::new(0, true, false, Some("".to_string()), "".to_string());

        assert!(
            cfg.validate().is_ok(),
            "Configuration should be valid when snapshots disabled"
        );
    }

    #[test]
    fn test_missing_snapshot_url() {
        let cfg = IndexerConfig::new(0, true, true, None, "".to_string());

        assert!(
            cfg.validate().is_err(),
            "Configuration should be invalid when snapshots is enabled but URL is missing"
        );
    }

    #[test]
    fn test_emoty_snapshot_url() {
        let cfg = IndexerConfig::new(0, true, true, Some("".to_string()), "".to_string());

        assert!(
            cfg.validate().is_err(),
            "Configuration should be invalid when snapshots is enabled but URL is empty"
        );
    }
}
