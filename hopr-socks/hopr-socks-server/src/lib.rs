use cli::AuthMode;
use fast_socks5::server::{Authentication, Config as ServerConfig, SimpleUserPassword, Socks5Socket};
use fast_socks5::Result;
use std::{future::Future, sync::Arc};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpListener,
    task,
};
use tracing::{error, info, warn};

pub mod cli;

pub async fn spawn_server(socks_domain: &String, timeout: u64, auth: AuthMode) -> Result<()> {
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
