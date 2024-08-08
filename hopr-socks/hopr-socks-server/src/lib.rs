pub mod cli;

use cli::AuthMode;
use fast_socks5::server::{Config as ServerConfig, SimpleUserPassword, Socks5Socket};
use fast_socks5::{Result, SocksError};
use std::sync::Arc;

use tokio::{net::TcpListener, task};
use tracing::{error, info, warn};

pub struct SocksServer {
    config: Arc<ServerConfig<SimpleUserPassword>>,
    bind_address: String,
}

impl SocksServer {
    pub async fn new(bind_address: String, timeout: u64, auth: AuthMode) -> Result<Self, SocksError> {
        let mut config = ServerConfig::default();
        config.set_request_timeout(timeout);
        config = match auth {
            AuthMode::NoAuth => {
                warn!("No authentication has been set!");
                config
            }
            AuthMode::Password { username, password } => {
                info!("Simple auth system has been set.");
                config.with_authentication(SimpleUserPassword { username, password })
            }
        };

        Ok(SocksServer {
            config: Arc::new(config),
            bind_address,
        })
    }

    pub async fn run(self) -> Result<(), SocksError> {
        let listener = TcpListener::bind(&self.bind_address).await?;
        info!("Listen for socks connections @ {}", &self.bind_address);

        loop {
            match listener.accept().await {
                Ok((socket, _addr)) => {
                    info!("Connection from {}", socket.peer_addr()?);
                    let socket = Socks5Socket::new(socket, self.config.clone());

                    // TODO: we don't care about individual processing
                    task::spawn(async move {
                        if let Err(e) = socket.upgrade_to_socks5().await {
                            error!("{:#}", &e);
                        }
                    });
                }
                // TODO: consider use cases when there's a limit for incoming connections
                // and this can fail
                Err(err) => error!("accept error = {err:?}"),
            }
        }
    }
}
