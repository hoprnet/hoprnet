use std::{net::ToSocketAddrs, str::FromStr};

use anyhow::{Context, Result};
use clap::Parser;
use hopr_lib::{
    HoprBalance,
    config::HostConfig,
    testing::fixtures::{ClusterGuard, SWARM_N, cluster_fixture},
};
use hopr_utils_session::ListenerJoinHandles;
use hoprd_api::{
    RestApiParameters,
    config::{Api as ApiConfig, Auth},
    serve_api,
};
use serde_json::json;
use tokio::net::TcpListener;
use tracing::{error, info, warn};

mod cli;
mod helper;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let args = cli::Args::parse();
    if args.size == 0 || args.size > SWARM_N {
        anyhow::bail!("size must be between 1 and {SWARM_N}");
    }

    info!(size = args.size, "starting local cluster");
    let cluster = cluster_fixture(args.size);

    info!(size = cluster.size(), "cluster ready");

    if args.skip_channels {
        warn!("skipping channel creation");
    } else {
        let funding: HoprBalance = args.funding_amount.parse()?;
        let channel_guards = helper::open_full_mesh_channels(&cluster, funding)
            .await
            .context("failed to open full-mesh channels")?;

        if channel_guards.len() != args.size * (args.size - 1) {
            warn!(
                "expected {} channels, but only {} were opened",
                args.size * (args.size - 1),
                channel_guards.len()
            );
        } else {
            info!(count = channel_guards.len(), "full mesh channels opened");
        }
    }

    let api_handles = start_rest_api(&cluster, &args).await?;
    helper::print_node_summary(&cluster, &api_handles);

    info!("localcluster running; press Ctrl+C to stop");
    tokio::signal::ctrl_c().await.context("failed to await Ctrl+C")?;
    info!("shutdown requested");

    for handle in api_handles {
        handle.join.abort();
    }

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    Ok(())
}

fn init_tracing() {
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

async fn start_rest_api(cluster: &ClusterGuard, args: &cli::Args) -> Result<Vec<helper::ApiHandle>> {
    let mut handles = Vec::new();

    for (idx, node) in cluster.iter().enumerate() {
        let port = args.api_port_base + idx as u16;

        let auth = Auth::Token(args.api_token.clone());
        let host = HostConfig::from_str(&format!("{}:{}", args.api_host, port)).map_err(|e| anyhow::anyhow!(e))?;
        let api_cfg = ApiConfig {
            enable: true,
            auth,
            host,
        };

        let listener = TcpListener::bind(format!("{}:{}", args.api_host, port))
            .await
            .context("failed to bind REST API listener")?;

        let hoprd_cfg = json!({
            "api": {
                "host": format!("{}:{}", args.api_host, port),
                "auth": json!({"token": args.api_token})
            }
        });

        let host = args.api_host.clone();
        let port = 0; // We only need the port for resolving the default session listen host, and it doesn't have to be the actual API port

        let mut addrs = (host.clone(), port).to_socket_addrs()?;
        let default_session_listen_host = addrs
            .next()
            .ok_or_else(|| anyhow::anyhow!("could not resolve {host}:{port}"))?;

        let params = RestApiParameters {
            listener,
            hoprd_cfg,
            cfg: api_cfg,
            hopr: node.instance.clone(),
            session_listener_sockets: std::sync::Arc::new(ListenerJoinHandles::default()),
            default_session_listen_host,
        };

        let handle = tokio::spawn(async move {
            if let Err(e) = serve_api(params).await {
                error!(error = %e, "REST API server stopped");
            }
        });

        handles.push(helper::ApiHandle {
            host: args.api_host.clone(),
            port,
            token: args.api_token.clone(),
            join: handle,
        });
    }

    Ok(handles)
}
