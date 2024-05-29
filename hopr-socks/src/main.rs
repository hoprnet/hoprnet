use fast_socks5::server::{Authentication, Config, SimpleUserPassword, Socks5Socket};
use fast_socks5::Result;
use log::{error, info, warn};
use std::future::Future;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::task;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpListener,
};

/// # How to use it:
///
/// Listen on a local address, authentication-free:
///     `$ RUST_LOG=debug cargo run --bin hopr-socks -- --host 127.0.0.1 --port 1337 no-auth`
///
/// Listen on a local address, with basic username/password requirement:
///     `$ RUST_LOG=debug cargo run --bin hopr-socks -- --host 127.0.0.1 --port 1337 password --username admin --password password`
///
#[derive(Debug, StructOpt)]
#[structopt(name = "hopr-socks", about = "A simple SOCKS5 server implementation.")]
struct Opt {
    /// Bind on address address. eg. `127.0.0.1`
    #[structopt(long)]
    pub host: String,

    /// Bind on address address. eg. `1337`
    #[structopt(long)]
    pub port: String,

    /// Request timeout
    #[structopt(short = "t", long, default_value = "10")]
    pub request_timeout: u64,

    /// Choose authentication type
    #[structopt(subcommand, name = "auth")]
    pub auth: AuthMode,
}

/// Choose the authentication type
#[derive(StructOpt, Debug)]
enum AuthMode {
    NoAuth,
    Password {
        #[structopt(short, long)]
        username: String,

        #[structopt(short, long)]
        password: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    spawn_socks_server().await
}

async fn spawn_socks_server() -> Result<()> {
    let opt: Opt = Opt::from_args();
    let mut config = Config::default();
    config.set_request_timeout(opt.request_timeout);

    let config = match opt.auth {
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

    let domain = [opt.host, opt.port].join(":");
    let listener = TcpListener::bind(&domain).await?;
    //    listener.set_config(config);

    info!("Listen for socks connections @ {}", &domain);

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
