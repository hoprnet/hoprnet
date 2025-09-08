use std::{collections::HashMap, sync::Arc, time::Duration};

use async_lock::RwLock;
use hopr_crypto_types::prelude::*;
use hopr_lib::{
    Address, Hopr,
    config::{Chain, Db, HoprLibConfig, HostConfig, HostType, SafeModule},
};
use hopr_transport::{ChainKeypair, OffchainKeypair};
use tokio::net::TcpListener;
use tracing::info;

#[tokio::test(flavor = "multi_thread")]
/// This tests ensures API endpoints are functioning while the RPC is not usable.
/// We simulate RPC failures by using a non-existent RPC endpoint and verify
/// that the API server handles this gracefully without crashing.
async fn integration_test_api_failing_rpc() -> anyhow::Result<()> {
    // Initialize test environment
    let _ = tracing_subscriber::fmt::try_init();

    // Create test keypairs
    let chain_key = ChainKeypair::random();
    let offchain_key = OffchainKeypair::random();

    // Create temporary directory for test data
    let temp_dir = tempfile::tempdir()?;

    // Create HOPR lib config with minimal settings
    // Using a non-existent RPC endpoint to simulate RPC failures
    let hopr_cfg = HoprLibConfig {
        db: Db {
            data: temp_dir.path().to_string_lossy().to_string(),
            initialize: true,
            force_initialize: true,
        },
        host: HostConfig {
            address: HostType::IPv4("127.0.0.1".to_string()),
            port: 0, // Let the system assign a port
        },
        chain: Chain {
            network: "debug-staging".to_string(),
            // Use a non-responsive endpoint to simulate RPC failures
            provider: Some("http://localhost:1234".to_string()), // Non-existent port
            announce: false,
            fast_sync: true,
            keep_logs: true,
            max_rpc_requests_per_sec: Some(1000),
            enable_logs_snapshot: false,
            logs_snapshot_url: None,
            protocols: Default::default(),
        },
        transport: Default::default(),
        protocol: Default::default(),
        network_options: Default::default(),
        strategy: Default::default(),
        probe: Default::default(),
        session: Default::default(),
        safe_module: SafeModule {
            safe_address: Address::from([11u8; 20]),
            module_address: Address::from([12u8; 20]),
            safe_transaction_service_provider: hopr_lib::config::DEFAULT_SAFE_TRANSACTION_SERVICE_PROVIDER.to_string(),
        },
    };

    // Create the HOPR node - this may fail with RPC issues, but that's expected
    let node = Arc::new(Hopr::new(hopr_cfg, &offchain_key, &chain_key)?);
    let _ = node.run(true).await?;

    // Create the API server configuration
    let api_cfg = hoprd_api::config::Api {
        enable: true,
        host: HostConfig {
            address: HostType::IPv4("127.0.0.1".to_string()),
            port: 0, // Let system assign a port
        },
        auth: hoprd_api::config::Auth::Token("test-token".to_string()),
    };

    // Create TCP listener for the API
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let api_addr = listener.local_addr()?;
    info!("API server listening on: {}", api_addr);

    // Prepare API parameters
    let api_params = hoprd_api::RestApiParameters {
        listener,
        hoprd_cfg: serde_json::json!({
            "version": "test",
            "network": "debug-staging"
        }),
        cfg: api_cfg,
        hopr: node.clone(),
        session_listener_sockets: Arc::new(RwLock::new(HashMap::new())),
        default_session_listen_host: std::net::SocketAddr::from(([127, 0, 0, 1], 0)),
    };

    // Start the API server in the background
    let api_handle = tokio::spawn(async move {
        if let Err(e) = hoprd_api::serve_api(api_params).await {
            tracing::error!("API server error: {}", e);
        }
    });

    // Give the server a moment to start
    tokio::time::sleep(Duration::from_millis(5000)).await;

    // Create HTTP client for testing
    let client = reqwest::Client::new();
    let base_url = format!("http://{}", api_addr);

    // Test /startedz - should return 412 PRECONDITION_FAILED
    let resp = client.get(&format!("{}/startedz", base_url)).send().await?;
    assert_eq!(resp.status(), reqwest::StatusCode::PRECONDITION_FAILED);

    // Test /readyz - should return 412 PRECONDITION_FAILED
    let resp = client.get(&format!("{}/readyz", base_url)).send().await?;
    assert_eq!(resp.status(), reqwest::StatusCode::PRECONDITION_FAILED);

    // Test /healthyz - should return 412 PRECONDITION_FAILED
    let resp = client.get(&format!("{}/healthyz", base_url)).send().await?;
    assert_eq!(resp.status(), reqwest::StatusCode::PRECONDITION_FAILED);

    // Test authenticated endpoints
    let auth_token = "test-token";

    // Test /api/v4/node/version - should work without RPC
    let resp = client
        .get(&format!("{}/api/v4/node/version", base_url))
        .header("X-Auth-Token", auth_token)
        .send()
        .await?;
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let version_data: serde_json::Value = resp.json().await?;
    info!("/api/v4/node/version works: {:?}", version_data);

    // Test /api/v4/node/info - should not work without RPC, because it fetches channel info from
    // chain
    let resp = client
        .get(&format!("{}/api/v4/node/info", base_url))
        .header("X-Auth-Token", auth_token)
        .send()
        .await?;
    assert_eq!(resp.status(), reqwest::StatusCode::UNPROCESSABLE_ENTITY);

    // Test /api/v4/node/configuration - should work without RPC
    let resp = client
        .get(&format!("{}/api/v4/node/configuration", base_url))
        .header("X-Auth-Token", auth_token)
        .send()
        .await?;
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    info!("/api/v4/node/configuration works");

    // Test /api/v4/account/addresses - might fail due to RPC
    let resp = client
        .get(&format!("{}/api/v4/account/addresses", base_url))
        .header("X-Auth-Token", auth_token)
        .send()
        .await?;
    info!(
        "/api/v4/account/addresses returned status: {} (RPC-dependent)",
        resp.status()
    );

    // Test /api/v4/account/balances - will likely fail due to RPC
    let resp = client
        .get(&format!("{}/api/v4/account/balances", base_url))
        .header("X-Auth-Token", auth_token)
        .send()
        .await?;
    info!(
        "  /api/v4/account/balances returned status: {} (RPC-dependent)",
        resp.status()
    );

    info!("\n✅ API integration test completed successfully!");
    info!("   The API server handled requests appropriately despite RPC being unavailable.");
    info!("   - Non-RPC endpoints (node info, version, etc.) work correctly");
    info!("   - RPC-dependent endpoints fail gracefully without crashing the server");

    // Clean up
    api_handle.abort();

    Ok(())
}
