//! Process HTTP connections on the server.

use async_io::Timer;
use futures_lite::io::{self, AsyncRead as Read, AsyncWrite as Write};
use futures_lite::prelude::*;
use http_types::headers::{CONNECTION, UPGRADE};
use http_types::upgrade::Connection;
use http_types::{Request, Response, StatusCode};
use std::{future::Future, marker::PhantomData, time::Duration};
mod body_reader;
mod decode;
mod encode;

pub use decode::decode;
pub use encode::Encoder;

/// Configure the server.
#[derive(Debug, Clone)]
pub struct ServerOptions {
    /// Timeout to handle headers. Defaults to 60s.
    headers_timeout: Option<Duration>,
}

impl Default for ServerOptions {
    fn default() -> Self {
        Self {
            headers_timeout: Some(Duration::from_secs(60)),
        }
    }
}

/// Accept a new incoming HTTP/1.1 connection.
///
/// Supports `KeepAlive` requests by default.
pub async fn accept<RW, F, Fut>(io: RW, endpoint: F) -> http_types::Result<()>
where
    RW: Read + Write + Clone + Send + Sync + Unpin + 'static,
    F: Fn(Request) -> Fut,
    Fut: Future<Output = http_types::Result<Response>>,
{
    Server::new(io, endpoint).accept().await
}

/// Accept a new incoming HTTP/1.1 connection.
///
/// Supports `KeepAlive` requests by default.
pub async fn accept_with_opts<RW, F, Fut>(
    io: RW,
    endpoint: F,
    opts: ServerOptions,
) -> http_types::Result<()>
where
    RW: Read + Write + Clone + Send + Sync + Unpin + 'static,
    F: Fn(Request) -> Fut,
    Fut: Future<Output = http_types::Result<Response>>,
{
    Server::new(io, endpoint).with_opts(opts).accept().await
}

/// struct for server
#[derive(Debug)]
pub struct Server<RW, F, Fut> {
    io: RW,
    endpoint: F,
    opts: ServerOptions,
    _phantom: PhantomData<Fut>,
}

/// An enum that represents whether the server should accept a subsequent request
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ConnectionStatus {
    /// The server should not accept another request
    Close,

    /// The server may accept another request
    KeepAlive,
}

impl<RW, F, Fut> Server<RW, F, Fut>
where
    RW: Read + Write + Clone + Send + Sync + Unpin + 'static,
    F: Fn(Request) -> Fut,
    Fut: Future<Output = http_types::Result<Response>>,
{
    /// builds a new server
    pub fn new(io: RW, endpoint: F) -> Self {
        Self {
            io,
            endpoint,
            opts: Default::default(),
            _phantom: PhantomData,
        }
    }

    /// with opts
    pub fn with_opts(mut self, opts: ServerOptions) -> Self {
        self.opts = opts;
        self
    }

    /// accept in a loop
    pub async fn accept(&mut self) -> http_types::Result<()> {
        while ConnectionStatus::KeepAlive == self.accept_one().await? {}
        Ok(())
    }

    /// accept one request
    pub async fn accept_one(&mut self) -> http_types::Result<ConnectionStatus>
    where
        RW: Read + Write + Clone + Send + Sync + Unpin + 'static,
        F: Fn(Request) -> Fut,
        Fut: Future<Output = http_types::Result<Response>>,
    {
        // Decode a new request, timing out if this takes longer than the timeout duration.
        let fut = decode(self.io.clone());

        let (req, mut body) = if let Some(timeout_duration) = self.opts.headers_timeout {
            match fut
                .or(async {
                    Timer::after(timeout_duration).await;
                    Ok(None)
                })
                .await
            {
                Ok(Some(r)) => r,
                Ok(None) => return Ok(ConnectionStatus::Close), /* EOF or timeout */
                Err(e) => return Err(e),
            }
        } else {
            match fut.await? {
                Some(r) => r,
                None => return Ok(ConnectionStatus::Close), /* EOF */
            }
        };

        let has_upgrade_header = req.header(UPGRADE).is_some();
        let connection_header_as_str = req
            .header(CONNECTION)
            .map(|connection| connection.as_str())
            .unwrap_or("");

        let connection_header_is_upgrade = connection_header_as_str
            .split(',')
            .any(|s| s.trim().eq_ignore_ascii_case("upgrade"));
        let mut close_connection = connection_header_as_str.eq_ignore_ascii_case("close");

        let upgrade_requested = has_upgrade_header && connection_header_is_upgrade;

        let method = req.method();

        // Pass the request to the endpoint and encode the response.
        let mut res = (self.endpoint)(req).await?;

        close_connection |= res
            .header(CONNECTION)
            .map(|c| c.as_str().eq_ignore_ascii_case("close"))
            .unwrap_or(false);

        let upgrade_provided = res.status() == StatusCode::SwitchingProtocols && res.has_upgrade();

        let upgrade_sender = if upgrade_requested && upgrade_provided {
            Some(res.send_upgrade())
        } else {
            None
        };

        let mut encoder = Encoder::new(res, method);

        let bytes_written = io::copy(&mut encoder, &mut self.io).await?;
        log::trace!("wrote {} response bytes", bytes_written);

        let body_bytes_discarded = io::copy(&mut body, &mut io::sink()).await?;
        log::trace!(
            "discarded {} unread request body bytes",
            body_bytes_discarded
        );

        if let Some(upgrade_sender) = upgrade_sender {
            upgrade_sender.send(Connection::new(self.io.clone())).await;
            Ok(ConnectionStatus::Close)
        } else if close_connection {
            Ok(ConnectionStatus::Close)
        } else {
            Ok(ConnectionStatus::KeepAlive)
        }
    }
}
