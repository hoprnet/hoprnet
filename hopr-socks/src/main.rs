use anyhow::Context;
use clap::{Parser, Subcommand};
use fast_socks5::client::{Config as ClientConfig, Socks5Stream};
use fast_socks5::server::{Authentication, Config as ServerConfig, SimpleUserPassword, Socks5Socket};
use fast_socks5::Result;
use std::future::Future;
use std::sync::Arc;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpListener,
    task,
};
use tracing::{debug, error, info, warn};
use tracing_subscriber::layer::SubscriberExt;

/// # How to use it:
///
/// ## Run in server mode:
///
/// $ hopr-socks --host 127.0.0.1 --port 1337 server no-auth
/// $ hopr-socks --host 127.0.0.1 --port 1337 server password --username admin --password password
///
/// ## Run in client mode:
///
/// $ hopr-socks --host 127.0.0.1 --port 1337 client --target-host example.com no-auth
///
/// $ hopr-socks --host 127.0.0.1 --port 1337 client --target-host example.com password --username admin --password password
///
#[derive(Debug, Parser)]
#[clap(name = "hopr-socks", about = "A simple SOCKS5 server implementation.")]
struct Opt {
    #[clap(help = "Bind on address host. eg. `127.0.0.1`", long)]
    host: String,

    #[clap(help = "Bind on address port", long)]
    port: String,

    /// Choose running mode
    #[clap(subcommand)]
    pub mode: RunModeOpt,
}

#[derive(Debug, Subcommand)]
enum RunModeOpt {
    Client {
        /// Target address server (not the socks server)
        #[clap(short = 'a', long)]
        target_host: String,

        /// Target port server (not the socks server)
        #[clap(short = 'p', long, default_value = "80")]
        target_port: u16,

        /// Choose authentication type
        #[clap(subcommand)]
        auth: AuthMode,
    },
    Server {
        /// Request timeout
        #[clap(short = 't', long, default_value = "10")]
        request_timeout: u64,

        /// Choose authentication type
        #[clap(subcommand)]
        auth: AuthMode,
    },
}

/// Choose the authentication type
#[derive(Subcommand, Debug)]
enum AuthMode {
    NoAuth,
    Password {
        #[clap(short, long)]
        username: String,

        #[clap(short, long)]
        password: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let format = tracing_subscriber::fmt::layer()
        .with_level(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(false);

    let subscriber = tracing_subscriber::Registry::default().with(env_filter).with(format);

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    let opt: Opt = Opt::parse();
    let socks_domain: String = [opt.host, opt.port].join(":");

    return match opt.mode {
        RunModeOpt::Client {
            target_host,
            target_port,
            auth,
        } => spawn_socks_client(&socks_domain, target_host, target_port, auth).await,
        RunModeOpt::Server { request_timeout, auth } => spawn_socks_server(&socks_domain, request_timeout, auth).await,
    };
}

async fn spawn_socks_client(
    socks_domain: &String,
    target_host: String,
    target_port: u16,
    auth: AuthMode,
) -> Result<()> {
    let remote_domain: String = [target_host.clone(), target_port.to_string()].join(":");
    let config = ClientConfig::default();

    // Creating a SOCKS stream to the target address through the socks server
    let mut socks = match auth {
        AuthMode::NoAuth => Socks5Stream::connect(socks_domain.clone(), target_host, target_port, config).await?,
        AuthMode::Password { username, password } => {
            Socks5Stream::connect_with_password(
                socks_domain.clone(),
                target_host,
                target_port,
                username,
                password,
                config,
            )
            .await?
        }
    };

    // Once connection is completed, can start to communicate with the server
    http_request(&mut socks, remote_domain).await?;

    Ok(())
}
async fn spawn_socks_server(socks_domain: &String, timeout: u64, auth: AuthMode) -> Result<()> {
    let mut config = ServerConfig::default();
    config.set_request_timeout(timeout);

    let config = match auth {
        AuthMode::NoAuth => {
            warn!("No authentication has been set!");
            config
        }
        AuthMode::Password { username, password } => {
            info!("Simple auth system has been set.");
            config.with_authentication(SimpleUserPassword { username, password })
        }
    };

    let config = Arc::new(config);
    let listener = TcpListener::bind(&socks_domain).await?;

    info!("Listen for socks connections @ {}", &socks_domain);

    // Standard TCP loop
    loop {
        match listener.accept().await {
            Ok((socket, _addr)) => {
                info!("Connection from {}", socket.peer_addr()?);
                let socket = Socks5Socket::new(socket, config.clone());

                spawn_and_log_error(socket.upgrade_to_socks5());
            }
            Err(err) => error!("accept error = {:?}", err),
        }
    }
}

fn spawn_and_log_error<F, T, A>(fut: F) -> task::JoinHandle<()>
where
    F: Future<Output = Result<Socks5Socket<T, A>>> + Send + 'static,
    T: AsyncRead + AsyncWrite + Unpin,
    A: Authentication,
{
    task::spawn(async move {
        if let Err(e) = fut.await {
            error!("{:#}", &e);
        }
    })
}

async fn http_request<T: AsyncRead + AsyncWrite + Unpin>(stream: &mut T, domain: String) -> Result<()> {
    debug!("Requesting body...");

    // construct our request, with a dynamic domain
    let mut headers = vec![];
    headers.extend_from_slice("GET / HTTP/1.1\n".as_bytes());
    headers.extend_from_slice(format!("Host: {domain}\n").as_bytes());
    headers.extend_from_slice("User-Agent: hopr-socks/0.1.0\n".as_bytes());
    headers.extend_from_slice("Accept: */*\n\n".as_bytes());

    // flush headers
    stream.write_all(&headers).await.context("Can't write HTTP Headers")?;

    debug!("Reading body response...");
    let mut result = [0u8; 1024];
    stream.read(&mut result).await.context("Can't read HTTP Response")?;

    info!("Response: {}", String::from_utf8_lossy(&result));

    if result.starts_with(b"HTTP/1.1") {
        info!("HTTP/1.1 Response detected!");
    }

    Ok(())
}
