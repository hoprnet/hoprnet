use std::future::Future;
use std::marker::{PhantomData, Send};

use crate::async_tungstenite::WebSocketStream;
use crate::tungstenite::protocol::Role;
use crate::WebSocketConnection;

use async_dup::Arc;
use async_std::task;
use sha1::{Digest, Sha1};

use tide::http::format_err;
use tide::http::headers::{HeaderName, CONNECTION, UPGRADE};
use tide::{Middleware, Request, Response, Result, StatusCode};

const WEBSOCKET_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// # endpoint/middleware handler for websockets in tide
///
/// This can either be used as a middleware or as an
/// endpoint. Regardless of which approach is taken, the handler
/// function provided to [`WebSocket::new`] is only called if the
/// request correctly negotiates an upgrade to the websocket protocol.
///
/// ## As a middleware
///
/// If used as a middleware, the endpoint will be executed if the
/// request is not a websocket upgrade.
///
/// ### Example
///
/// ```rust
/// use async_std::prelude::*;
/// use tide_websockets::{Message, WebSocket};
///
/// #[async_std::main]
/// async fn main() -> Result<(), std::io::Error> {
///     let mut app = tide::new();
///
///     app.at("/ws")
///         .with(WebSocket::new(|_request, mut stream| async move {
///             while let Some(Ok(Message::Text(input))) = stream.next().await {
///                 let output: String = input.chars().rev().collect();
///
///                 stream
///                     .send_string(format!("{} | {}", &input, &output))
///                     .await?;
///             }
///
///             Ok(())
///         }))
///        .get(|_| async move { Ok("this was not a websocket request") });
///
/// # if false {
///     app.listen("127.0.0.1:8080").await?;
/// # }
///     Ok(())
/// }
/// ```
///
/// ## As an endpoint
///
/// If used as an endpoint but the request is
/// not a websocket request, tide will reply with a `426 Upgrade
/// Required` status code.
///
/// ### example
///
/// ```rust
/// use async_std::prelude::*;
/// use tide_websockets::{Message, WebSocket};
///
/// #[async_std::main]
/// async fn main() -> Result<(), std::io::Error> {
///     let mut app = tide::new();
///
///     app.at("/ws")
///         .get(WebSocket::new(|_request, mut stream| async move {
///             while let Some(Ok(Message::Text(input))) = stream.next().await {
///                 let output: String = input.chars().rev().collect();
///
///                 stream
///                     .send_string(format!("{} | {}", &input, &output))
///                     .await?;
///             }
///
///             Ok(())
///         }));
///
/// # if false {
///     app.listen("127.0.0.1:8080").await?;
/// # }
///     Ok(())
/// }
/// ```
///
#[derive(Debug)]
pub struct WebSocket<S, H> {
    handler: Arc<H>,
    ghostly_apparition: PhantomData<S>,
    protocols: Vec<String>,
}

enum UpgradeStatus<S> {
    Upgraded(Result<Response>),
    NotUpgraded(Request<S>),
}
use UpgradeStatus::{NotUpgraded, Upgraded};

fn header_contains_ignore_case<T>(req: &Request<T>, header_name: HeaderName, value: &str) -> bool {
    req.header(header_name)
        .map(|h| {
            h.as_str()
                .split(',')
                .any(|s| s.trim().eq_ignore_ascii_case(value.trim()))
        })
        .unwrap_or(false)
}

impl<S, H, Fut> WebSocket<S, H>
where
    S: Send + Sync + Clone + 'static,
    H: Fn(Request<S>, WebSocketConnection) -> Fut + Sync + Send + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    /// Build a new WebSocket with a handler function that
    pub fn new(handler: H) -> Self {
        Self {
            handler: Arc::new(handler),
            ghostly_apparition: PhantomData,
            protocols: Default::default(),
        }
    }

    /// `protocols` is a sequence of known protocols. On successful handshake,
    /// the returned response headers contain the first protocol in this list
    /// which the server also knows.
    pub fn with_protocols(self, protocols: &[&str]) -> Self {
        Self {
            protocols: protocols.iter().map(ToString::to_string).collect(),
            ..self
        }
    }

    async fn handle_upgrade(&self, req: Request<S>) -> UpgradeStatus<S> {
        let connection_upgrade = header_contains_ignore_case(&req, CONNECTION, "upgrade");
        let upgrade_to_websocket = header_contains_ignore_case(&req, UPGRADE, "websocket");
        let upgrade_requested = connection_upgrade && upgrade_to_websocket;

        if !upgrade_requested {
            return NotUpgraded(req);
        }

        let header = match req.header("Sec-Websocket-Key") {
            Some(h) => h.as_str(),
            None => return Upgraded(Err(format_err!("expected sec-websocket-key"))),
        };

        let protocol = req.header("Sec-Websocket-Protocol").and_then(|value| {
            value
                .as_str()
                .split(',')
                .map(str::trim)
                .find(|req_p| self.protocols.iter().any(|p| p == req_p))
        });

        let mut response = Response::new(StatusCode::SwitchingProtocols);

        response.insert_header(UPGRADE, "websocket");
        response.insert_header(CONNECTION, "Upgrade");
        let hash = Sha1::new().chain(header).chain(WEBSOCKET_GUID).finalize();
        response.insert_header("Sec-Websocket-Accept", base64::encode(&hash[..]));
        response.insert_header("Sec-Websocket-Version", "13");

        if let Some(protocol) = protocol {
            response.insert_header("Sec-Websocket-Protocol", protocol);
        }

        let http_res: &mut tide::http::Response = response.as_mut();
        let upgrade_receiver = http_res.recv_upgrade().await;
        let handler = self.handler.clone();

        task::spawn(async move {
            if let Some(stream) = upgrade_receiver.await {
                let stream = WebSocketStream::from_raw_socket(stream, Role::Server, None).await;
                handler(req, stream.into()).await
            } else {
                Err(format_err!("never received an upgrade!"))
            }
        });

        Upgraded(Ok(response))
    }
}

#[tide::utils::async_trait]
impl<H, S, Fut> tide::Endpoint<S> for WebSocket<S, H>
where
    H: Fn(Request<S>, WebSocketConnection) -> Fut + Sync + Send + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
    S: Send + Sync + Clone + 'static,
{
    async fn call(&self, req: Request<S>) -> Result {
        match self.handle_upgrade(req).await {
            Upgraded(result) => result,
            NotUpgraded(_) => Ok(Response::new(StatusCode::UpgradeRequired)),
        }
    }
}

#[tide::utils::async_trait]
impl<H, S, Fut> Middleware<S> for WebSocket<S, H>
where
    H: Fn(Request<S>, WebSocketConnection) -> Fut + Sync + Send + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
    S: Send + Sync + Clone + 'static,
{
    async fn handle(&self, req: Request<S>, next: tide::Next<'_, S>) -> Result {
        match self.handle_upgrade(req).await {
            Upgraded(result) => result,
            NotUpgraded(req) => Ok(next.run(req).await),
        }
    }
}
