// Test file to verify snapshot functionality independently
use std::path::Path;

use tempfile::TempDir;

use crate::{
    config::IndexerConfig,
    snapshot::{
        download::SnapshotDownloader, error::SnapshotError, extract::SnapshotExtractor, validate::SnapshotValidator,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing snapshot functionality...");

    // Test 1: Configuration validation
    test_config_validation().await?;

    // Test 2: Disk space checking
    test_disk_space_check().await?;

    // Test 3: Error message enhancement
    test_error_messages().await?;

    println!("All tests passed!");
    Ok(())
}

async fn test_config_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing configuration validation...");

    // Valid configuration
    let valid_config = IndexerConfig::new(
        100,
        true,
        true,
        "https://example.com/snapshot.tar.gz".to_string(),
        "/tmp/hopr_data".to_string(),
    );

    assert!(valid_config.is_valid());

    // Invalid configuration - bad URL
    let invalid_config = IndexerConfig::new(
        100,
        true,
        true,
        "ftp://example.com/snapshot.tar.gz".to_string(),
        "/tmp/hopr_data".to_string(),
    );

    assert!(!invalid_config.is_valid());

    // Invalid configuration - empty directory when snapshots enabled
    let empty_dir_config = IndexerConfig::new(
        100,
        true,
        true,
        "https://example.com/snapshot.tar.gz".to_string(),
        "".to_string(),
    );

    assert!(!empty_dir_config.is_valid());

    println!("✓ Configuration validation tests passed");
    Ok(())
}

async fn test_disk_space_check() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing disk space check...");

    let temp_dir = TempDir::new()?;
    let downloader = SnapshotDownloader::new();

    // Test with available disk space (should succeed)
    let result = downloader.check_disk_space(temp_dir.path()).await;
    assert!(result.is_ok());

    println!("✓ Disk space check tests passed");
    Ok(())
}

async fn test_error_messages() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing enhanced error messages...");

    // Test that errors contain suggestions
    let error = SnapshotError::InvalidData("Test error".to_string());
    let error_string = error.to_string();
    assert!(error_string.contains("Suggestion:"));

    let network_error = SnapshotError::Network(reqwest::Error::from(std::io::Error::new(
        std::io::ErrorKind::ConnectionRefused,
        "test",
    )));
    let network_error_string = network_error.to_string();
    assert!(network_error_string.contains("Suggestion:"));

    println!("✓ Error message enhancement tests passed");
    Ok(())
}
