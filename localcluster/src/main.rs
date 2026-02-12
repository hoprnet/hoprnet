use std::{
    fs::{self, File},
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};

use anyhow::{Context, Result};
use clap::Parser;
use hoprd_localcluster::{
    blokli_helper, cli, client_helper,
    identity::{self, DEFAULT_BLOKLI_URL},
};
use tracing::{debug, error, info, warn};

const DEFAULT_WAIT_TIMEOUT: Duration = Duration::from_secs(60);
const INDEXING_WAIT_TIME: Duration = Duration::from_secs(10);

#[derive(Default)]
struct Cleanup {
    nodes: Vec<client_helper::NodeProcess>,
    chain: Option<blokli_helper::ChainHandle>,
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
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    let args = cli::Args::parse();

    let data_dir = args.data_dir.clone();
    fs::create_dir_all(&data_dir).context("failed to create data directory")?;
    let log_dir = data_dir.join("logs");
    fs::create_dir_all(&log_dir).context("failed to create log directory")?;

    let blokli_url = args.chain_url.clone().unwrap_or_else(|| DEFAULT_BLOKLI_URL.to_string());
    let blokli_url = blokli_url.trim_end_matches('/').to_string();
    let config = identity::GenerationConfig {
        blokli_url: blokli_url.to_string(),
        num_nodes: args.size,
        config_home: data_dir.to_path_buf(),
        identity_password: args.identity_password.clone(),
        random_identities: true,
        ..Default::default()
    };

    let mut cleanup = Cleanup::default();

    let result: Result<()> = async {
        if args.chain_url.is_some() {
            info!("using external chain services at {blokli_url}");
        } else {
            let chain_image = args
                .chain_image
                .as_deref()
                .context("missing chain image (set --chain-image or HOPRD_CHAIN_IMAGE)")?;
            info!("starting chain services (anvil + blokli)");
            cleanup.chain = Some(blokli_helper::ChainHandle::start(chain_image, &log_dir)?);
        }

        // TODO: replace with a proper healthcheck call to blokli once available
        {
            wait_for_http_response(&format!("{blokli_url}/graphql"), DEFAULT_WAIT_TIMEOUT).await?;
            info!(
                "chain services are up. Waiting {} seconds for the indexer to catch up",
                INDEXING_WAIT_TIME.as_secs()
            );
            tokio::time::sleep(INDEXING_WAIT_TIME).await;
        }

        info!("generating identities and configs via hoprd-gen-test library");
        identity::generate(&config).await?;

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
            info!("opening channels to every other node");
            client_helper::open_full_mesh_channels(&cleanup.nodes, &args.funding_amount).await?;
        }

        node_summary(&cleanup.nodes, &args);

        info!("localcluster running; press Ctrl+C to stop");
        tokio::signal::ctrl_c().await.context("failed to await Ctrl+C")?;
        info!("shutdown requested");

        Ok(())
    }
    .await;

    cleanup.shutdown();

    if let Err(err) = result {
        error!(error = %err, "localcluster failed");
        return Err(err);
    }

    Ok(())
}

async fn start_hoprd_nodes(
    args: &cli::Args,
    data_dir: &Path,
    log_dir: &Path,
) -> Result<Vec<client_helper::NodeProcess>> {
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
            .arg(format!("{}:{}", &args.p2p_host, p2p_port))
            .arg("--password")
            .arg(&args.identity_password)
            .stdout(Stdio::from(log_file))
            .stderr(Stdio::from(log_err));

        if let Some(token) = &args.api_token {
            cmd.arg("--apiToken").arg(token);
        }

        debug!("starting hoprd node {} with command: {:?}", id, cmd);
        let child = cmd.spawn().context("failed to start hoprd")?;
        let api = client_helper::HoprdApiClient::new(
            format!("http://{}:{}", api_client_host, api_port),
            args.api_token.clone(),
        )?;

        nodes.push(client_helper::NodeProcess {
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

fn node_summary(nodes: &[client_helper::NodeProcess], args: &cli::Args) {
    println!();

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

        let rows = [
            ("Address", addr),
            ("P2P", format!("{}:{}", &args.p2p_host, node.p2p_port)),
            ("API host", api),
            ("API token", token),
            ("Node admin", node_admin),
            ("PID", node.child.id().to_string()),
        ];
        let label_width = rows.iter().map(|(label, _)| label.len()).max().unwrap_or(0);

        println!("Node {}", node.id);
        for (label, value) in rows {
            println!("\t{label:<width$}: {value}", width = label_width);
        }
        println!();
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
