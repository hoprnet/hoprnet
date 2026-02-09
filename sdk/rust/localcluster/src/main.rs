use anyhow::{Context, Result};
use clap::Parser;
use futures::future::try_join_all;
use hopr_lib::testing::fixtures::{ClusterGuard, SWARM_N, cluster_fixture};
use hopr_lib::testing::hopr::ChannelGuard;
use hopr_lib::{HoprBalance, config::HostConfig};
use hopr_utils_session::ListenerJoinHandles;
use hoprd_api::{
    RestApiParameters,
    config::{Api as ApiConfig, Auth},
    serve_api,
};
use serde_json::json;
use std::net::{SocketAddr, ToSocketAddrs};
use std::str::FromStr;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(name = "hopr-localcluster", about = "Run an in-process local HOPR cluster")]
struct Args {
    /// Number of nodes to start (max: SWARM_N)
    #[arg(long, default_value_t = SWARM_N)]
    size: usize,

    /// Channel funding amount in base units (per channel)
    #[arg(long, default_value = "1 wxHOPR")]
    funding_amount: String,

    /// Skip channel creation
    #[arg(long, default_value_t = false)]
    skip_channels: bool,

    /// REST API host to bind
    #[arg(long, default_value = "127.0.0.1")]
    api_host: String,

    /// REST API base port (node index is added)
    #[arg(long, default_value_t = 7000)]
    api_port_base: u16,

    /// Disable REST API authentication
    #[arg(long, default_value = "e2e-API-token^^")]
    api_token: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let args = Args::parse();
    if args.size == 0 || args.size > SWARM_N {
        anyhow::bail!("size must be between 1 and {SWARM_N}");
    }

    info!(size = args.size, "starting local cluster");
    let cluster = cluster_fixture(args.size);

    info!(size = cluster.size(), "cluster ready");
    print_nodes(&cluster);

    if args.skip_channels {
        warn!("skipping channel creation");
    } else {
        let funding: HoprBalance = args.funding_amount.parse()?;
        let channel_guards = open_full_mesh_channels(&cluster, funding)
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
    print_node_summary(&cluster, &api_handles);

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

fn print_nodes(cluster: &ClusterGuard) {
    for (idx, node) in cluster.iter().enumerate() {
        let port = node.config().host.port;
        info!(
            index = idx,
            address = %node.address(),
            peer_id = %node.peer_id(),
            port = port,
            "node ready"
        );
    }
}

fn print_node_summary(cluster: &ClusterGuard, api_handles: &[ApiHandle]) {
    println!("\n\n");

    for (idx, node) in cluster.iter().enumerate() {
        let api = api_handles.get(idx);
        let api_host = api.map(|h| h.host_with_port()).unwrap_or_else(|| "N/A".to_string());
        let api_key = api.map(|h| h.token.clone()).unwrap_or_else(|| "N/A".to_string());
        let node_admin = format!(
            "http://localhost:4677/node/info?apiToken={}&apiEndpoint=http://{}",
            api_key, api_host
        );

        println!(
            "Node {}:\n\tAddress: {}\n\tPeer ID: {}\n\tAPI Host: {}\n\tAPI Key: {}\n\tNode admin: {}\n\n",
            idx,
            node.address(),
            node.peer_id(),
            api_host,
            api_key,
            node_admin
        );
    }
}

async fn open_full_mesh_channels(cluster: &ClusterGuard, funding: HoprBalance) -> Result<Vec<ChannelGuard>> {
    let mut futures = Vec::new();

    for (src_idx, src) in cluster.iter().enumerate() {
        for (dst_idx, dst) in cluster.iter().enumerate() {
            if src_idx == dst_idx {
                continue;
            }

            let src = src.instance.clone();
            let dst = dst.instance.clone();
            futures.push(async move { ChannelGuard::open_channel_between_nodes(src, dst, funding).await });
        }
    }

    try_join_all(futures).await.map_err(Into::into)
}

struct ApiHandle {
    host: String,
    port: u16,
    token: String,
    join: JoinHandle<()>,
}

impl ApiHandle {
    fn host_with_port(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

async fn start_rest_api(cluster: &ClusterGuard, args: &Args) -> Result<Vec<ApiHandle>> {
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

        let default_session_listen_host =
            resolve_socket_addr(&args.api_host, 0).context("failed to resolve default session listen host")?;

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

        handles.push(ApiHandle {
            host: args.api_host.clone(),
            port,
            token: args.api_token.clone(),
            join: handle,
        });
    }

    Ok(handles)
}

fn resolve_socket_addr(host: &str, port: u16) -> Result<SocketAddr> {
    let mut addrs = (host, port).to_socket_addrs()?;
    addrs
        .next()
        .ok_or_else(|| anyhow::anyhow!("could not resolve {host}:{port}"))
}
