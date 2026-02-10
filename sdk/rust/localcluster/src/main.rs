use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};

use anyhow::{Context, Result};
use clap::Parser;
use futures::future::try_join_all;
use tracing::{debug, error, info, warn};

mod cli;
mod helper;

use cli::Args;
use helper::HoprdApiClient;

const DEFAULT_WAIT_TIMEOUT: Duration = Duration::from_secs(60);
const INDEXING_WAIT_TIME: Duration = Duration::from_secs(10);

struct ChainHandle {
    name: String,
    child: Child,
}

impl ChainHandle {
    fn start(args: &Args, log_dir: &Path) -> Result<Self> {
        fs::create_dir_all(log_dir).context("failed to create log directory")?;
        let log_file = log_dir.join("chain.log");
        let log_file = File::create(&log_file).context("failed to create blokli log file")?;
        let log_err = log_file.try_clone().context("failed to clone blokli log file handle")?;
        let name = "hopr-chain";

        let mut cmd = Command::new("docker");
        cmd.arg("run")
            .arg("--rm")
            .arg("--name")
            .arg(name)
            .arg("-p")
            .arg("8081:8080")
            .arg(&args.chain_image)
            .stdout(Stdio::from(log_file))
            .stderr(Stdio::from(log_err));

        let child = cmd.spawn().context("failed to start blokli container")?;

        Ok(Self {
            name: name.to_string(),
            child,
        })
    }

    fn stop(&mut self) {
        let _ = self.child.kill();
        let _ = Command::new("docker").arg("rm").arg("-f").arg(&self.name).status();
    }
}

struct NodeProcess {
    id: usize,
    api_port: u16,
    p2p_port: u16,
    api: HoprdApiClient,
    child: Child,
    address: Option<String>,
}

#[derive(Default)]
struct Cleanup {
    nodes: Vec<NodeProcess>,
    chain: Option<ChainHandle>,
}

impl Cleanup {
    fn shutdown(&mut self) {
        for node in self.nodes.iter_mut() {
            let _ = node.child.kill();
        }
        if let Some(chain) = self.chain.as_mut() {
            chain.stop();
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let args = Args::parse();
    validate_args(&args)?;

    let data_dir = PathBuf::from(&args.data_dir);
    fs::create_dir_all(&data_dir).context("failed to create data directory")?;
    let log_dir = data_dir.join("logs");
    fs::create_dir_all(&log_dir).context("failed to create log directory")?;

    let mut cleanup = Cleanup::default();

    let result: Result<()> = async {
        info!("starting chain services (anvil + blokli)");
        cleanup.chain = Some(ChainHandle::start(&args, &log_dir)?);
        let blokli_url = "http://127.0.0.1:8081".to_string();

        wait_for_http_response(&format!("{blokli_url}/graphql"), DEFAULT_WAIT_TIMEOUT).await?;

        info!(
            "chain services are up. Waiting {} seconds for the indexer to catch up",
            INDEXING_WAIT_TIME.as_secs()
        );
        tokio::time::sleep(INDEXING_WAIT_TIME).await;

        info!("generating identities and configs via hoprd-gen-test");
        run_hoprd_gen_test(&args, &data_dir, &blokli_url)?;

        info!("starting hoprd nodes");
        cleanup.nodes = start_hoprd_nodes(&args, &data_dir, &log_dir).await?;

        info!("waiting for nodes to start");
        for node in cleanup.nodes.iter() {
            node.api.wait_started(2 * DEFAULT_WAIT_TIMEOUT).await?;
        }
        info!("waiting for nodes to be ready");
        for node in cleanup.nodes.iter() {
            node.api.wait_ready(DEFAULT_WAIT_TIMEOUT).await?;
        }

        info!("fetching node addresses");
        for node in cleanup.nodes.iter_mut() {
            node.address = Some(node.api.addresses().await?);
        }

        if args.skip_channels {
            warn!("skipping channel creation");
        } else {
            info!("opening full-mesh channels");
            open_full_mesh_channels(&cleanup.nodes, &args.funding_amount).await?;
        }

        print_node_summary(&cleanup.nodes, &args);

        info!("localcluster running; press Ctrl+C to stop");
        tokio::signal::ctrl_c().await.context("failed to await Ctrl+C")?;
        info!("shutdown requested");

        Ok(())
    }
    .await;

    if let Err(err) = result {
        error!(error = %err, "localcluster failed");
        cleanup.shutdown();
        return Err(err);
    }

    cleanup.shutdown();
    Ok(())
}

fn validate_args(args: &Args) -> Result<()> {
    if args.size == 0 {
        anyhow::bail!("size must be at least 1");
    }
    Ok(())
}

fn init_tracing() {
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

fn run_hoprd_gen_test(args: &Args, data_dir: &Path, blokli_url: &str) -> Result<()> {
    let mut cmd = Command::new(&args.hoprd_gen_test_bin);
    cmd.arg("--blokli-url")
        .arg(blokli_url)
        .arg("--num-nodes")
        .arg(args.size.to_string())
        .arg("--config-home")
        .arg(data_dir)
        .arg("--identity-password")
        .arg(&args.identity_password)
        .arg("--random-identities");

    let status = cmd.status().context("failed to run hoprd-gen-test")?;
    if !status.success() {
        anyhow::bail!("hoprd-gen-test failed with status {status}");
    }

    Ok(())
}

async fn start_hoprd_nodes(args: &Args, data_dir: &Path, log_dir: &Path) -> Result<Vec<NodeProcess>> {
    let mut nodes = Vec::new();
    let api_host = &args.api_host;
    let api_client_host = if api_host == "0.0.0.0" { "127.0.0.1" } else { api_host };

    for id in 0..args.size {
        let api_port = args.api_port_base + id as u16;
        let p2p_port = args.p2p_port_base + id as u16;
        let cfg_file = data_dir.join(format!("hoprd_cfg_{id}.yaml"));
        if !cfg_file.exists() {
            anyhow::bail!("missing hoprd config file: {}", cfg_file.display());
        }
        let db_dir = data_dir.join(format!("db_{id}"));
        fs::create_dir_all(db_dir.join("node_db"))
            .with_context(|| format!("failed to create db directory {}", db_dir.join("node_db").display()))?;
        let log_file = log_dir.join(format!("hoprd_{id}.log"));

        let log_file = File::create(&log_file).context("failed to create hoprd log file")?;
        let log_err = log_file.try_clone().context("failed to clone hoprd log file handle")?;

        let mut cmd = Command::new(&args.hoprd_bin);
        cmd.arg("--configurationFilePath")
            .arg(cfg_file)
            .arg("--api")
            .arg("--apiHost")
            .arg(api_host)
            .arg("--apiPort")
            .arg(api_port.to_string())
            .arg("--host")
            .arg(format!("{}:{}", args.p2p_host, p2p_port))
            .arg("--password")
            .arg(&args.identity_password)
            .stdout(Stdio::from(log_file))
            .stderr(Stdio::from(log_err));

        if let Some(token) = &args.api_token {
            cmd.arg("--apiToken").arg(token);
        }

        debug!("starting hoprd node {} with command: {:?}", id, cmd);
        let child = cmd.spawn().context("failed to start hoprd")?;
        let api = HoprdApiClient::new(
            format!("http://{}:{}", api_client_host, api_port),
            args.api_token.clone(),
        )?;

        nodes.push(NodeProcess {
            id,
            api_port,
            p2p_port,
            api,
            child,
            address: None,
        });
    }

    Ok(nodes)
}

async fn open_full_mesh_channels(nodes: &[NodeProcess], amount: &str) -> Result<()> {
    let mut tasks = Vec::new();
    for src in nodes {
        let Some(src_addr) = src.address.clone() else {
            anyhow::bail!("node {} address missing", src.id);
        };
        for dst in nodes {
            let Some(dst_addr) = dst.address.clone() else {
                anyhow::bail!("node {} address missing", dst.id);
            };
            if src_addr == dst_addr {
                continue;
            }
            let api = src.api.clone();
            let amount = amount.to_string();
            tasks.push(async move { api.open_channel(&dst_addr, &amount).await });
        }
    }

    try_join_all(tasks).await.context("failed to open channels")?;
    Ok(())
}

fn print_node_summary(nodes: &[NodeProcess], args: &Args) {
    println!("\n\n");

    for node in nodes {
        let addr = node.address.clone().unwrap_or_else(|| "N/A".to_string());
        let api_host = if args.api_host == "0.0.0.0" {
            "127.0.0.1"
        } else {
            &args.api_host
        };
        let api = format!("http://{}:{}", api_host, node.api_port);
        let token = args.api_token.clone().unwrap_or_else(|| "N/A".to_string());
        let mut node_admin = format!("http://localhost:4677/node/info?apiEndpoint={api}");
        if let Some(token) = &args.api_token {
            node_admin.push_str(&format!("&apiToken={token}"));
        }

        println!(
            "Node {}:\n\tAddress: {}\n\tP2P: {}:{}\n\tAPI host {}\n\tAPI token: {}\n\tNode admin: {}\n\tPID: {}\n\n",
            node.id,
            addr,
            args.p2p_host,
            node.p2p_port,
            api,
            token,
            node_admin,
            node.child.id()
        );
    }
}

async fn wait_for_http_response(url: &str, timeout: Duration) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .context("failed to build http client")?;
    let start = std::time::Instant::now();

    loop {
        if client.get(url).send().await.is_ok() {
            return Ok(());
        }

        if start.elapsed() > timeout {
            anyhow::bail!("timeout waiting for {url}");
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
