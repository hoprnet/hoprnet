//! Integration tests for the session stats REST API endpoint.
//!
//! These tests verify the behavior of the `/api/v4/session/stats/{session_id}` endpoint,
//! including successful stat retrieval, error handling, and response structure validation.
//! Tests use insta snapshots with redactions for non-deterministic fields (timestamps, durations).

#![cfg(feature = "test-fixtures")]

use std::{str::FromStr, sync::Arc, time::Duration};

use hopr_lib::{HoprBalance, config::HostConfig, testing::fixtures::cluster_fixture};
use hopr_utils_session::ListenerJoinHandles;
use hoprd_api::{
    RestApiParameters,
    config::{Api, Auth},
    serve_api,
};
use insta::assert_yaml_snapshot;
use reqwest::StatusCode;
use serde_json::Value;
use tokio::net::TcpListener;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn session_stats_endpoint_returns_snapshot() -> anyhow::Result<()> {
    let cluster = cluster_fixture(2);
    let entry = &cluster[0];
    let exit = &cluster[1];

    let (session, _fw_channels, _bw_channels) =
        cluster.create_session(&[entry, exit], HoprBalance::new_base(1)).await?;

    let session_id = session.id().to_string();

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let base_url = format!("http://{addr}");

    let params = RestApiParameters {
        listener,
        hoprd_cfg: serde_json::json!({}),
        cfg: Api {
            enable: true,
            auth: Auth::None,
            host: HostConfig::from_str("127.0.0.1:0").map_err(|e| anyhow::anyhow!("{}", e))?,
        },
        hopr: entry.instance.clone(),
        session_listener_sockets: Arc::new(ListenerJoinHandles::default()),
        default_session_listen_host: "127.0.0.1:0".parse()?,
    };

    let server_handle = tokio::spawn(async move { serve_api(params).await });
    tokio::time::sleep(Duration::from_millis(200)).await;

    let client = reqwest::Client::new();
    let url = format!("{base_url}/api/v4/session/stats/{session_id}");
    let response = client.get(url).send().await?;
    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = response.json().await?;
    assert_yaml_snapshot!(body, {
        ".sessionId" => "[session_id]",
        ".snapshotAtMs" => "[timestamp]",
        ".lifetime.createdAtMs" => "[timestamp]",
        ".lifetime.terminatedAtMs" => "[timestamp]",
        ".lifetime.lastActivityAtMs" => "[timestamp]",
        ".lifetime.idleMs" => "[duration]",
        ".lifetime.uptimeMs" => "[duration]",
    });

    server_handle.abort();
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn session_stats_invalid_id_returns_400() -> anyhow::Result<()> {
    let cluster = cluster_fixture(1);
    let entry = &cluster[0];

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let base_url = format!("http://{addr}");

    let params = RestApiParameters {
        listener,
        hoprd_cfg: serde_json::json!({}),
        cfg: Api {
            enable: true,
            auth: Auth::None,
            host: HostConfig::from_str("127.0.0.1:0").map_err(|e| anyhow::anyhow!("{}", e))?,
        },
        hopr: entry.instance.clone(),
        session_listener_sockets: Arc::new(ListenerJoinHandles::default()),
        default_session_listen_host: "127.0.0.1:0".parse()?,
    };

    let server_handle = tokio::spawn(async move { serve_api(params).await });
    tokio::time::sleep(Duration::from_millis(200)).await;

    let client = reqwest::Client::new();
    let url = format!("{base_url}/api/v4/session/stats/not-a-session-id");
    let response = client.get(url).send().await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    server_handle.abort();
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn session_stats_invalid_session_id_format() -> anyhow::Result<()> {
    let cluster = cluster_fixture(1);
    let entry = &cluster[0];

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let base_url = format!("http://{addr}");

    let params = RestApiParameters {
        listener,
        hoprd_cfg: serde_json::json!({}),
        cfg: Api {
            enable: true,
            auth: Auth::None,
            host: HostConfig::from_str("127.0.0.1:0").map_err(|e| anyhow::anyhow!("{}", e))?,
        },
        hopr: entry.instance.clone(),
        session_listener_sockets: Arc::new(ListenerJoinHandles::default()),
        default_session_listen_host: "127.0.0.1:0".parse()?,
    };

    let server_handle = tokio::spawn(async move { serve_api(params).await });
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Valid format but nonexistent session ID returns same error as invalid format
    let nonexistent_session_id = "0x0000000000000000000000000000000000000000:65535";

    let client = reqwest::Client::new();
    let url = format!("{base_url}/api/v4/session/stats/{nonexistent_session_id}");
    let response = client.get(url).send().await?;
    // API returns 400 for both invalid format and nonexistent sessions
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    server_handle.abort();
    Ok(())
}
