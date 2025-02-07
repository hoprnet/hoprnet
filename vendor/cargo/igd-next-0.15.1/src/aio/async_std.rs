//! Async-std abstraction for the aio [`Gateway`].

use std::collections::HashMap;
use std::net::SocketAddr;

use async_std::{future::timeout, net::UdpSocket};
use async_trait::async_trait;
use futures::prelude::*;
use log::debug;

use super::{Provider, HEADER_NAME, MAX_RESPONSE_SIZE};
use crate::aio::Gateway;
use crate::common::options::{DEFAULT_TIMEOUT, RESPONSE_TIMEOUT};
use crate::common::{messages, parsing, SearchOptions};
use crate::errors::SearchError;
use crate::RequestError;

/// Async-std provider for the [`Gateway`].
#[derive(Debug, Clone)]
pub struct AsyncStd;

#[async_trait]
impl Provider for AsyncStd {
    async fn send_async(url: &str, action: &str, body: &str) -> Result<String, RequestError> {
        Ok(surf::post(url)
            .header(HEADER_NAME, action)
            .content_type("text/xml")
            .body(body)
            .recv_string()
            .await?)
    }
}

/// Search for a gateway with the provided options.
pub async fn search_gateway(options: SearchOptions) -> Result<Gateway<AsyncStd>, SearchError> {
    // Create socket for future calls.
    let mut socket = UdpSocket::bind(&options.bind_addr).await?;

    send_search_request(&mut socket, options.broadcast_address).await?;

    let max_search_time = options.timeout.unwrap_or(DEFAULT_TIMEOUT);
    let start_search_time = std::time::Instant::now();

    while start_search_time.elapsed() < max_search_time {
        let search_response = receive_search_response(&mut socket);

        // Receive search response
        let (response_body, from) = match timeout(RESPONSE_TIMEOUT, search_response).await {
            Ok(Ok(v)) => v,
            Ok(Err(err)) => {
                debug!("error while receiving broadcast response: {err}");
                continue;
            }
            Err(_) => {
                debug!("timeout while receiving broadcast response");
                continue;
            }
        };

        let (addr, root_url) = match handle_broadcast_resp(&from, &response_body) {
            Ok(v) => v,
            Err(e) => {
                debug!("error handling broadcast response: {}", e);
                continue;
            }
        };

        let (control_schema_url, control_url) = match get_control_urls(&addr, &root_url).await {
            Ok(v) => v,
            Err(e) => {
                debug!("error getting control URLs: {}", e);
                continue;
            }
        };

        let control_schema = match get_control_schemas(&addr, &control_schema_url).await {
            Ok(v) => v,
            Err(e) => {
                debug!("error getting control schemas: {}", e);
                continue;
            }
        };

        return Ok(Gateway {
            addr,
            root_url,
            control_url,
            control_schema_url,
            control_schema,
            provider: AsyncStd,
        });
    }

    Err(SearchError::NoResponseWithinTimeout)
}

// Create a new search.
async fn send_search_request(socket: &mut UdpSocket, addr: SocketAddr) -> Result<(), SearchError> {
    debug!(
        "sending broadcast request to: {} on interface: {:?}",
        addr,
        socket.local_addr()
    );
    socket
        .send_to(messages::SEARCH_REQUEST.as_bytes(), &addr)
        .map_ok(|_| ())
        .map_err(SearchError::from)
        .await
}

async fn receive_search_response(socket: &mut UdpSocket) -> Result<(Vec<u8>, SocketAddr), SearchError> {
    let mut buff = [0u8; MAX_RESPONSE_SIZE];
    let (n, from) = socket.recv_from(&mut buff).map_err(SearchError::from).await?;
    debug!("received broadcast response from: {}", from);
    Ok((buff[..n].to_vec(), from))
}

// Handle a UDP response message.
fn handle_broadcast_resp(from: &SocketAddr, data: &[u8]) -> Result<(SocketAddr, String), SearchError> {
    debug!("handling broadcast response from: {}", from);

    // Convert response to text.
    let text = std::str::from_utf8(data).map_err(SearchError::from)?;

    // Parse socket address and path.
    let (addr, root_url) = parsing::parse_search_result(text)?;

    Ok((addr, root_url))
}

async fn get_control_urls(addr: &SocketAddr, path: &str) -> Result<(String, String), SearchError> {
    let uri = format!("http://{addr}{path}");

    debug!("requesting control url from: {}", uri);

    let resp = surf::get(uri).recv_bytes().await?;

    debug!("handling control response from: {}", addr);
    let c = std::io::Cursor::new(&resp);
    parsing::parse_control_urls(c)
}

async fn get_control_schemas(
    addr: &SocketAddr,
    control_schema_url: &str,
) -> Result<HashMap<String, Vec<String>>, SearchError> {
    let uri = format!("http://{addr}{control_schema_url}");

    debug!("requesting control schema from: {uri}");
    let resp = surf::get(uri).recv_bytes().await?;

    debug!("handling schema response from: {addr}");
    let c = std::io::Cursor::new(&resp);
    parsing::parse_schemas(c)
}
