use async_trait::async_trait;
use futures::{future, future::BoxFuture, future::FutureExt, future::TryFutureExt, stream, stream::StreamExt, Stream};
use hyper::header::{HeaderName, HeaderValue, CONTENT_TYPE};
use hyper::{service::Service, Body, Request, Response, Uri};
use percent_encoding::{utf8_percent_encode, AsciiSet};
use std::borrow::Cow;
use std::convert::TryInto;
use std::error::Error;
use std::fmt;
use std::future::Future;
use std::io::{ErrorKind, Read};
use std::marker::PhantomData;
use std::path::Path;
use std::str;
use std::str::FromStr;
use std::string::ToString;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use swagger::{ApiError, AuthData, BodyExt, Connector, DropContextService, Has, XSpanIdString};
use url::form_urlencoded;

use crate::header;
use crate::models;

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
#[allow(dead_code)]
const FRAGMENT_ENCODE_SET: &AsciiSet = &percent_encoding::CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'<')
    .add(b'>')
    .add(b'`');

/// This encode set is used for object IDs
///
/// Aside from the special characters defined in the `PATH_SEGMENT_ENCODE_SET`,
/// the vertical bar (|) is encoded.
#[allow(dead_code)]
const ID_ENCODE_SET: &AsciiSet = &FRAGMENT_ENCODE_SET.add(b'|');

use crate::{
    AccountGetAddressResponse, AccountGetAddressesResponse, AccountGetBalancesResponse, AccountWithdrawResponse,
    AliasesGetAliasResponse, AliasesGetAliasesResponse, AliasesRemoveAliasResponse, AliasesSetAliasResponse, Api,
    ChannelsAggregateTicketsResponse, ChannelsCloseChannelResponse, ChannelsFundChannelResponse,
    ChannelsGetChannelResponse, ChannelsGetChannelsResponse, ChannelsGetTicketsResponse, ChannelsOpenChannelResponse,
    ChannelsRedeemTicketsResponse, CheckNodeHealthyResponse, CheckNodeReadyResponse, CheckNodeStartedResponse,
    MessagesDeleteMessagesResponse, MessagesGetSizeResponse, MessagesPopAllMessageResponse, MessagesPopMessageResponse,
    MessagesSendMessageResponse, MessagesWebsocketResponse, NodeGetEntryNodesResponse, NodeGetInfoResponse,
    NodeGetMetricsResponse, NodeGetPeersResponse, NodeGetVersionResponse, PeerInfoGetPeerInfoResponse,
    PeersPingPeerResponse, SettingsGetSettingsResponse, SettingsSetSettingResponse, TicketsGetStatisticsResponse,
    TicketsGetTicketsResponse, TicketsRedeemTicketsResponse, TokensCreateResponse, TokensDeleteResponse,
    TokensGetTokenResponse,
};

/// Convert input into a base path, e.g. "http://example:123". Also checks the scheme as it goes.
fn into_base_path(
    input: impl TryInto<Uri, Error = hyper::http::uri::InvalidUri>,
    correct_scheme: Option<&'static str>,
) -> Result<String, ClientInitError> {
    // First convert to Uri, since a base path is a subset of Uri.
    let uri = input.try_into()?;

    let scheme = uri.scheme_str().ok_or(ClientInitError::InvalidScheme)?;

    // Check the scheme if necessary
    if let Some(correct_scheme) = correct_scheme {
        if scheme != correct_scheme {
            return Err(ClientInitError::InvalidScheme);
        }
    }

    let host = uri.host().ok_or(ClientInitError::MissingHost)?;
    let port = uri.port_u16().map(|x| format!(":{}", x)).unwrap_or_default();
    Ok(format!(
        "{}://{}{}{}",
        scheme,
        host,
        port,
        uri.path().trim_end_matches('/')
    ))
}

/// A client that implements the API by making HTTP calls out to a server.
pub struct Client<S, C>
where
    S: Service<(Request<Body>, C), Response = Response<Body>> + Clone + Sync + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<crate::ServiceError> + fmt::Display,
    C: Clone + Send + Sync + 'static,
{
    /// Inner service
    client_service: S,

    /// Base path of the API
    base_path: String,

    /// Marker
    marker: PhantomData<fn(C)>,
}

impl<S, C> fmt::Debug for Client<S, C>
where
    S: Service<(Request<Body>, C), Response = Response<Body>> + Clone + Sync + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<crate::ServiceError> + fmt::Display,
    C: Clone + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Client {{ base_path: {} }}", self.base_path)
    }
}

impl<S, C> Clone for Client<S, C>
where
    S: Service<(Request<Body>, C), Response = Response<Body>> + Clone + Sync + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<crate::ServiceError> + fmt::Display,
    C: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            client_service: self.client_service.clone(),
            base_path: self.base_path.clone(),
            marker: PhantomData,
        }
    }
}

impl<Connector, C> Client<DropContextService<hyper::client::Client<Connector, Body>, C>, C>
where
    Connector: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    C: Clone + Send + Sync + 'static,
{
    /// Create a client with a custom implementation of hyper::client::Connect.
    ///
    /// Intended for use with custom implementations of connect for e.g. protocol logging
    /// or similar functionality which requires wrapping the transport layer. When wrapping a TCP connection,
    /// this function should be used in conjunction with `swagger::Connector::builder()`.
    ///
    /// For ordinary tcp connections, prefer the use of `try_new_http`, `try_new_https`
    /// and `try_new_https_mutual`, to avoid introducing a dependency on the underlying transport layer.
    ///
    /// # Arguments
    ///
    /// * `base_path` - base path of the client API, i.e. "http://www.my-api-implementation.com"
    /// * `protocol` - Which protocol to use when constructing the request url, e.g. `Some("http")`
    /// * `connector` - Implementation of `hyper::client::Connect` to use for the client
    pub fn try_new_with_connector(
        base_path: &str,
        protocol: Option<&'static str>,
        connector: Connector,
    ) -> Result<Self, ClientInitError> {
        let client_service = hyper::client::Client::builder().build(connector);
        let client_service = DropContextService::new(client_service);

        Ok(Self {
            client_service,
            base_path: into_base_path(base_path, protocol)?,
            marker: PhantomData,
        })
    }
}

#[derive(Debug, Clone)]
pub enum HyperClient {
    Http(hyper::client::Client<hyper::client::HttpConnector, Body>),
    Https(hyper::client::Client<HttpsConnector, Body>),
}

impl Service<Request<Body>> for HyperClient {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = hyper::client::ResponseFuture;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        match self {
            HyperClient::Http(client) => client.poll_ready(cx),
            HyperClient::Https(client) => client.poll_ready(cx),
        }
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        match self {
            HyperClient::Http(client) => client.call(req),
            HyperClient::Https(client) => client.call(req),
        }
    }
}

impl<C> Client<DropContextService<HyperClient, C>, C>
where
    C: Clone + Send + Sync + 'static,
{
    /// Create an HTTP client.
    ///
    /// # Arguments
    /// * `base_path` - base path of the client API, i.e. "http://www.my-api-implementation.com"
    pub fn try_new(base_path: &str) -> Result<Self, ClientInitError> {
        let uri = Uri::from_str(base_path)?;

        let scheme = uri.scheme_str().ok_or(ClientInitError::InvalidScheme)?;
        let scheme = scheme.to_ascii_lowercase();

        let connector = Connector::builder();

        let client_service = match scheme.as_str() {
            "http" => HyperClient::Http(hyper::client::Client::builder().build(connector.build())),
            "https" => {
                let connector = connector.https().build().map_err(ClientInitError::SslError)?;
                HyperClient::Https(hyper::client::Client::builder().build(connector))
            }
            _ => {
                return Err(ClientInitError::InvalidScheme);
            }
        };

        let client_service = DropContextService::new(client_service);

        Ok(Self {
            client_service,
            base_path: into_base_path(base_path, None)?,
            marker: PhantomData,
        })
    }
}

impl<C> Client<DropContextService<hyper::client::Client<hyper::client::HttpConnector, Body>, C>, C>
where
    C: Clone + Send + Sync + 'static,
{
    /// Create an HTTP client.
    ///
    /// # Arguments
    /// * `base_path` - base path of the client API, i.e. "http://www.my-api-implementation.com"
    pub fn try_new_http(base_path: &str) -> Result<Self, ClientInitError> {
        let http_connector = Connector::builder().build();

        Self::try_new_with_connector(base_path, Some("http"), http_connector)
    }
}

#[cfg(any(target_os = "macos", target_os = "windows", target_os = "ios"))]
type HttpsConnector = hyper_tls::HttpsConnector<hyper::client::HttpConnector>;

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
type HttpsConnector = hyper_openssl::HttpsConnector<hyper::client::HttpConnector>;

impl<C> Client<DropContextService<hyper::client::Client<HttpsConnector, Body>, C>, C>
where
    C: Clone + Send + Sync + 'static,
{
    /// Create a client with a TLS connection to the server
    ///
    /// # Arguments
    /// * `base_path` - base path of the client API, i.e. "https://www.my-api-implementation.com"
    pub fn try_new_https(base_path: &str) -> Result<Self, ClientInitError> {
        let https_connector = Connector::builder()
            .https()
            .build()
            .map_err(ClientInitError::SslError)?;
        Self::try_new_with_connector(base_path, Some("https"), https_connector)
    }

    /// Create a client with a TLS connection to the server using a pinned certificate
    ///
    /// # Arguments
    /// * `base_path` - base path of the client API, i.e. "https://www.my-api-implementation.com"
    /// * `ca_certificate` - Path to CA certificate used to authenticate the server
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    pub fn try_new_https_pinned<CA>(base_path: &str, ca_certificate: CA) -> Result<Self, ClientInitError>
    where
        CA: AsRef<Path>,
    {
        let https_connector = Connector::builder()
            .https()
            .pin_server_certificate(ca_certificate)
            .build()
            .map_err(ClientInitError::SslError)?;
        Self::try_new_with_connector(base_path, Some("https"), https_connector)
    }

    /// Create a client with a mutually authenticated TLS connection to the server.
    ///
    /// # Arguments
    /// * `base_path` - base path of the client API, i.e. "https://www.my-api-implementation.com"
    /// * `ca_certificate` - Path to CA certificate used to authenticate the server
    /// * `client_key` - Path to the client private key
    /// * `client_certificate` - Path to the client's public certificate associated with the private key
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    pub fn try_new_https_mutual<CA, K, D>(
        base_path: &str,
        ca_certificate: CA,
        client_key: K,
        client_certificate: D,
    ) -> Result<Self, ClientInitError>
    where
        CA: AsRef<Path>,
        K: AsRef<Path>,
        D: AsRef<Path>,
    {
        let https_connector = Connector::builder()
            .https()
            .pin_server_certificate(ca_certificate)
            .client_authentication(client_key, client_certificate)
            .build()
            .map_err(ClientInitError::SslError)?;
        Self::try_new_with_connector(base_path, Some("https"), https_connector)
    }
}

impl<S, C> Client<S, C>
where
    S: Service<(Request<Body>, C), Response = Response<Body>> + Clone + Sync + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<crate::ServiceError> + fmt::Display,
    C: Clone + Send + Sync + 'static,
{
    /// Constructor for creating a `Client` by passing in a pre-made `hyper::service::Service` /
    /// `tower::Service`
    ///
    /// This allows adding custom wrappers around the underlying transport, for example for logging.
    pub fn try_new_with_client_service(client_service: S, base_path: &str) -> Result<Self, ClientInitError> {
        Ok(Self {
            client_service,
            base_path: into_base_path(base_path, None)?,
            marker: PhantomData,
        })
    }
}

/// Error type failing to create a Client
#[derive(Debug)]
pub enum ClientInitError {
    /// Invalid URL Scheme
    InvalidScheme,

    /// Invalid URI
    InvalidUri(hyper::http::uri::InvalidUri),

    /// Missing Hostname
    MissingHost,

    /// SSL Connection Error
    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "ios"))]
    SslError(native_tls::Error),

    /// SSL Connection Error
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    SslError(openssl::error::ErrorStack),
}

impl From<hyper::http::uri::InvalidUri> for ClientInitError {
    fn from(err: hyper::http::uri::InvalidUri) -> ClientInitError {
        ClientInitError::InvalidUri(err)
    }
}

impl fmt::Display for ClientInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: &dyn fmt::Debug = self;
        s.fmt(f)
    }
}

impl Error for ClientInitError {
    fn description(&self) -> &str {
        "Failed to produce a hyper client."
    }
}

#[async_trait]
impl<S, C> Api<C> for Client<S, C>
where
    S: Service<(Request<Body>, C), Response = Response<Body>> + Clone + Sync + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<crate::ServiceError> + fmt::Display,
    C: Has<XSpanIdString> + Has<Option<AuthData>> + Clone + Send + Sync + 'static,
{
    fn poll_ready(&self, cx: &mut Context) -> Poll<Result<(), crate::ServiceError>> {
        match self.client_service.clone().poll_ready(cx) {
            Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
            Poll::Ready(Ok(o)) => Poll::Ready(Ok(o)),
            Poll::Pending => Poll::Pending,
        }
    }

    async fn account_get_address(&self, context: &C) -> Result<AccountGetAddressResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/account/address", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("GET").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::AccountGetAddresses200Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AccountGetAddressResponse::AddressesFetchedSuccessfully(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AccountGetAddressResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AccountGetAddressResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AccountGetAddressResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn account_get_addresses(&self, context: &C) -> Result<AccountGetAddressesResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/account/addresses", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("GET").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::AccountGetAddresses200Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AccountGetAddressesResponse::AddressesFetchedSuccessfully(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AccountGetAddressesResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AccountGetAddressesResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AccountGetAddressesResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn account_get_balances(&self, context: &C) -> Result<AccountGetBalancesResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/account/balances", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("GET").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::AccountGetBalances200Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AccountGetBalancesResponse::BalancesFetchedSuccessfuly(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AccountGetBalancesResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AccountGetBalancesResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AccountGetBalancesResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn account_withdraw(
        &self,
        param_account_withdraw_request: Option<models::AccountWithdrawRequest>,
        context: &C,
    ) -> Result<AccountWithdrawResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/account/withdraw", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("POST").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let body = param_account_withdraw_request
            .map(|ref body| serde_json::to_string(body).expect("impossible to fail to serialize"));

        if let Some(body) = body {
            *request.body_mut() = Body::from(body);
        }

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create header: {} - {}", header, e))),
            },
        );

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::AccountWithdraw200Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AccountWithdrawResponse::WithdrawSuccessful(body))
            }
            400 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RequestStatus>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AccountWithdrawResponse::IncorrectDataInRequestBody(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AccountWithdrawResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AccountWithdrawResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::AccountWithdraw422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AccountWithdrawResponse::WithdrawAmountExeedsCurrentBalanceOrUnknownError(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn aliases_get_alias(&self, param_alias: String, context: &C) -> Result<AliasesGetAliasResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/api/v3/aliases/{alias}",
            self.base_path,
            alias = utf8_percent_encode(&param_alias.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("GET").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::AliasesGetAlias200Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AliasesGetAliasResponse::HOPRAddressWasFoundForTheProvidedAlias(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AliasesGetAliasResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AliasesGetAliasResponse::AuthorizationFailed(body))
            }
            404 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RequestStatus>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AliasesGetAliasResponse::ThisAliasWasNotAssignedToAnyPeerIdBefore(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AliasesGetAliasResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn aliases_get_aliases(&self, context: &C) -> Result<AliasesGetAliasesResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/aliases/", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("GET").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::AliasesGetAliases200Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AliasesGetAliasesResponse::ReturnsListOfAliasesAndCorrespondingPeerIds(
                    body,
                ))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AliasesGetAliasesResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn aliases_remove_alias(
        &self,
        param_alias: String,
        context: &C,
    ) -> Result<AliasesRemoveAliasResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/api/v3/aliases/{alias}",
            self.base_path,
            alias = utf8_percent_encode(&param_alias.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("DELETE").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            204 => Ok(AliasesRemoveAliasResponse::AliasRemovedSuccesfully),
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AliasesRemoveAliasResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AliasesRemoveAliasResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AliasesRemoveAliasResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn aliases_set_alias(
        &self,
        param_aliases_set_alias_request: Option<models::AliasesSetAliasRequest>,
        context: &C,
    ) -> Result<AliasesSetAliasResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/aliases/", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("POST").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let body = param_aliases_set_alias_request
            .map(|ref body| serde_json::to_string(body).expect("impossible to fail to serialize"));

        if let Some(body) = body {
            *request.body_mut() = Body::from(body);
        }

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create header: {} - {}", header, e))),
            },
        );

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            201 => Ok(AliasesSetAliasResponse::AliasSetSuccesfully),
            400 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RequestStatus>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AliasesSetAliasResponse::InvalidPeerId(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AliasesSetAliasResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AliasesSetAliasResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(AliasesSetAliasResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn channels_aggregate_tickets(
        &self,
        param_channelid: String,
        context: &C,
    ) -> Result<ChannelsAggregateTicketsResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/api/v3/channels/{channelid}/tickets/aggregate",
            self.base_path,
            channelid = utf8_percent_encode(&param_channelid.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("POST").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            204 => Ok(ChannelsAggregateTicketsResponse::TicketsSuccessfullyAggregated),
            400 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RequestStatus>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsAggregateTicketsResponse::InvalidChannelId(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsAggregateTicketsResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsAggregateTicketsResponse::AuthorizationFailed(body))
            }
            404 => Ok(ChannelsAggregateTicketsResponse::TheSpecifiedResourceWasNotFound),
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsAggregateTicketsResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn channels_close_channel(
        &self,
        param_channelid: String,
        context: &C,
    ) -> Result<ChannelsCloseChannelResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/api/v3/channels/{channelid}/",
            self.base_path,
            channelid = utf8_percent_encode(&param_channelid.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("DELETE").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ChannelsCloseChannel200Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsCloseChannelResponse::ChannelClosedSuccesfully(body))
            }
            400 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RequestStatus>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsCloseChannelResponse::InvalidChannelId(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsCloseChannelResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsCloseChannelResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsCloseChannelResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn channels_fund_channel(
        &self,
        param_channelid: String,
        param_channels_fund_channel_request: Option<models::ChannelsFundChannelRequest>,
        context: &C,
    ) -> Result<ChannelsFundChannelResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/api/v3/channels/{channelid}/fund",
            self.base_path,
            channelid = utf8_percent_encode(&param_channelid.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("POST").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let body = param_channels_fund_channel_request
            .map(|ref body| serde_json::to_string(body).expect("impossible to fail to serialize"));
        if let Some(body) = body {
            *request.body_mut() = Body::from(body);
        }

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create header: {} - {}", header, e))),
            },
        );
        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ChannelsFundChannel200Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsFundChannelResponse::ChannelFundedSuccessfully(body))
            }
            400 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RequestStatus>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsFundChannelResponse::InvalidChannelId(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsFundChannelResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsFundChannelResponse::AuthorizationFailed(body))
            }
            404 => Ok(ChannelsFundChannelResponse::TheSpecifiedResourceWasNotFound),
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsFundChannelResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn channels_get_channel(
        &self,
        param_channelid: serde_json::Value,
        context: &C,
    ) -> Result<ChannelsGetChannelResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/api/v3/channels/{channelid}/",
            self.base_path,
            channelid = utf8_percent_encode(&param_channelid.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("GET").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ChannelTopology>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsGetChannelResponse::ChannelFetchedSuccesfully(body))
            }
            400 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RequestStatus>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsGetChannelResponse::InvalidChannelId(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsGetChannelResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsGetChannelResponse::AuthorizationFailed(body))
            }
            404 => Ok(ChannelsGetChannelResponse::TheSpecifiedResourceWasNotFound),
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsGetChannelResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn channels_get_channels(
        &self,
        param_including_closed: Option<String>,
        param_full_topology: Option<String>,
        context: &C,
    ) -> Result<ChannelsGetChannelsResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/channels/", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_including_closed) = param_including_closed {
                query_string.append_pair("includingClosed", &param_including_closed);
            }
            if let Some(param_full_topology) = param_full_topology {
                query_string.append_pair("fullTopology", &param_full_topology);
            }
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("GET").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ChannelsGetChannels200Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsGetChannelsResponse::ChannelsFetchedSuccessfully(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsGetChannelsResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsGetChannelsResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsGetChannelsResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn channels_get_tickets(
        &self,
        param_channelid: String,
        context: &C,
    ) -> Result<ChannelsGetTicketsResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/api/v3/channels/{channelid}/tickets",
            self.base_path,
            channelid = utf8_percent_encode(&param_channelid.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("GET").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<Vec<models::Ticket>>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsGetTicketsResponse::TicketsFetchedSuccessfully(body))
            }
            400 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RequestStatus>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsGetTicketsResponse::InvalidPeerId(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsGetTicketsResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsGetTicketsResponse::AuthorizationFailed(body))
            }
            404 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RequestStatus>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsGetTicketsResponse::TicketsWereNotFoundForThatChannel(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsGetTicketsResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn channels_open_channel(
        &self,
        param_channels_open_channel_request: Option<models::ChannelsOpenChannelRequest>,
        context: &C,
    ) -> Result<ChannelsOpenChannelResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/channels/", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("POST").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let body = param_channels_open_channel_request
            .map(|ref body| serde_json::to_string(body).expect("impossible to fail to serialize"));
        if let Some(body) = body {
            *request.body_mut() = Body::from(body);
        }

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create header: {} - {}", header, e))),
            },
        );
        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            201 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ChannelsOpenChannel201Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsOpenChannelResponse::ChannelSuccesfullyOpened(body))
            }
            400 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RequestStatus>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsOpenChannelResponse::ProblemWithInputs(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsOpenChannelResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ChannelsOpenChannel403Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(
                    ChannelsOpenChannelResponse::FailedToOpenTheChannelBecauseOfInsufficientHOPRBalanceOrAllowance(
                        body,
                    ),
                )
            }
            409 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ChannelsOpenChannel409Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(
                    ChannelsOpenChannelResponse::FailedToOpenTheChannelBecauseTheChannelBetweenThisNodesAlreadyExists(
                        body,
                    ),
                )
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsOpenChannelResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn channels_redeem_tickets(
        &self,
        param_channelid: String,
        context: &C,
    ) -> Result<ChannelsRedeemTicketsResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/api/v3/channels/{channelid}/tickets/redeem",
            self.base_path,
            channelid = utf8_percent_encode(&param_channelid.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("POST").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            204 => Ok(ChannelsRedeemTicketsResponse::TicketsRedeemedSuccessfully),
            400 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RequestStatus>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsRedeemTicketsResponse::InvalidChannelId(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsRedeemTicketsResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsRedeemTicketsResponse::AuthorizationFailed(body))
            }
            404 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RequestStatus>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsRedeemTicketsResponse::TicketsWereNotFoundForThatChannel(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(ChannelsRedeemTicketsResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn check_node_healthy(&self, context: &C) -> Result<CheckNodeHealthyResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/healthyz/", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("GET").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<serde_json::Value>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(CheckNodeHealthyResponse::TheNodeIsReady(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(CheckNodeHealthyResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(CheckNodeHealthyResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(CheckNodeHealthyResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn check_node_ready(&self, context: &C) -> Result<CheckNodeReadyResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/readyz/", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("GET").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<serde_json::Value>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(CheckNodeReadyResponse::TheNodeIsReady(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(CheckNodeReadyResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(CheckNodeReadyResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(CheckNodeReadyResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn check_node_started(&self, context: &C) -> Result<CheckNodeStartedResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/startedz/", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("GET").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<serde_json::Value>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(CheckNodeStartedResponse::TheNodeIsStarted(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(CheckNodeStartedResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(CheckNodeStartedResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(CheckNodeStartedResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn messages_delete_messages(
        &self,
        param_tag: i32,
        context: &C,
    ) -> Result<MessagesDeleteMessagesResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/messages/", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.append_pair("tag", &param_tag.to_string());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("DELETE").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            204 => Ok(MessagesDeleteMessagesResponse::MessagesSuccessfullyDeleted),
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(MessagesDeleteMessagesResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(MessagesDeleteMessagesResponse::AuthorizationFailed(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn messages_get_size(&self, param_tag: i32, context: &C) -> Result<MessagesGetSizeResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/messages/size", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.append_pair("tag", &param_tag.to_string());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("GET").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::MessagesGetSize200Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(MessagesGetSizeResponse::ReturnsTheMessageInboxSizeFilteredByTheGivenTag(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(MessagesGetSizeResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(MessagesGetSizeResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(MessagesGetSizeResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn messages_pop_all_message(
        &self,
        param_messages_pop_all_message_request: Option<models::MessagesPopAllMessageRequest>,
        context: &C,
    ) -> Result<MessagesPopAllMessageResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/messages/pop-all", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("POST").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let body = param_messages_pop_all_message_request
            .map(|ref body| serde_json::to_string(body).expect("impossible to fail to serialize"));
        if let Some(body) = body {
            *request.body_mut() = Body::from(body);
        }

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create header: {} - {}", header, e))),
            },
        );
        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::MessagesPopAllMessage200Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(MessagesPopAllMessageResponse::ReturnsListOfMessages(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(MessagesPopAllMessageResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(MessagesPopAllMessageResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(MessagesPopAllMessageResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn messages_pop_message(
        &self,
        param_messages_pop_all_message_request: Option<models::MessagesPopAllMessageRequest>,
        context: &C,
    ) -> Result<MessagesPopMessageResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/messages/pop", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("POST").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let body = param_messages_pop_all_message_request
            .map(|ref body| serde_json::to_string(body).expect("impossible to fail to serialize"));
        if let Some(body) = body {
            *request.body_mut() = Body::from(body);
        }

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create header: {} - {}", header, e))),
            },
        );
        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ReceivedMessage>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(MessagesPopMessageResponse::ReturnsAMessage(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(MessagesPopMessageResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(MessagesPopMessageResponse::AuthorizationFailed(body))
            }
            404 => Ok(MessagesPopMessageResponse::TheSpecifiedResourceWasNotFound),
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(MessagesPopMessageResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn messages_send_message(
        &self,
        param_messages_send_message_request: Option<models::MessagesSendMessageRequest>,
        context: &C,
    ) -> Result<MessagesSendMessageResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/messages/", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("POST").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let body = param_messages_send_message_request
            .map(|ref body| serde_json::to_string(body).expect("impossible to fail to serialize"));
        if let Some(body) = body {
            *request.body_mut() = Body::from(body);
        }

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create header: {} - {}", header, e))),
            },
        );
        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            202 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<String>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(MessagesSendMessageResponse::TheMessageWasSentSuccessfully(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(MessagesSendMessageResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(MessagesSendMessageResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(MessagesSendMessageResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn messages_websocket(&self, context: &C) -> Result<MessagesWebsocketResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/messages/websocket", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("GET").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            101 => Ok(MessagesWebsocketResponse::SwitchingProtocols),
            206 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = body.to_string();
                Ok(MessagesWebsocketResponse::IncomingData(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(MessagesWebsocketResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(MessagesWebsocketResponse::AuthorizationFailed(body))
            }
            404 => Ok(MessagesWebsocketResponse::NotFound),
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn node_get_entry_nodes(&self, context: &C) -> Result<NodeGetEntryNodesResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/node/entryNodes", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("GET").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<
                    std::collections::HashMap<String, models::NodeGetEntryNodes200ResponseValue>,
                >(body)
                .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(NodeGetEntryNodesResponse::EntryNodeInformationFetchedSuccessfuly(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(NodeGetEntryNodesResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(NodeGetEntryNodesResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(NodeGetEntryNodesResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn node_get_info(&self, context: &C) -> Result<NodeGetInfoResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/node/info", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("GET").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::NodeGetInfo200Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(NodeGetInfoResponse::NodeInformationFetchedSuccessfuly(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(NodeGetInfoResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(NodeGetInfoResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(NodeGetInfoResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn node_get_metrics(&self, context: &C) -> Result<NodeGetMetricsResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/node/metrics", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("GET").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = body.to_string();
                Ok(NodeGetMetricsResponse::ReturnsTheEncodedSerializedMetrics(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(NodeGetMetricsResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(NodeGetMetricsResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(NodeGetMetricsResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn node_get_peers(&self, param_quality: Option<f64>, context: &C) -> Result<NodeGetPeersResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/node/peers", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_quality) = param_quality {
                query_string.append_pair("quality", &param_quality.to_string());
            }
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("GET").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::NodeGetPeers200Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(NodeGetPeersResponse::PeersInformationFetchedSuccessfuly(body))
            }
            400 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RequestStatus>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(NodeGetPeersResponse::InvalidInput(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(NodeGetPeersResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(NodeGetPeersResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(NodeGetPeersResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn node_get_version(&self, context: &C) -> Result<NodeGetVersionResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/node/version", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("GET").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<String>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(NodeGetVersionResponse::ReturnsTheReleaseVersionOfTheRunningNode(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(NodeGetVersionResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(NodeGetVersionResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(NodeGetVersionResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn peer_info_get_peer_info(
        &self,
        param_peerid: String,
        context: &C,
    ) -> Result<PeerInfoGetPeerInfoResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/api/v3/peers/{peerid}/",
            self.base_path,
            peerid = utf8_percent_encode(&param_peerid.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("GET").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::PeerInfoGetPeerInfo200Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(PeerInfoGetPeerInfoResponse::PeerInformationFetchedSuccessfully(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(PeerInfoGetPeerInfoResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(PeerInfoGetPeerInfoResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(PeerInfoGetPeerInfoResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn peers_ping_peer(&self, param_peerid: String, context: &C) -> Result<PeersPingPeerResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/api/v3/peers/{peerid}/ping",
            self.base_path,
            peerid = utf8_percent_encode(&param_peerid.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("POST").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::PeersPingPeer200Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(PeersPingPeerResponse::PingSuccessful(body))
            }
            400 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RequestStatus>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(PeersPingPeerResponse::InvalidPeerId(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(PeersPingPeerResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(PeersPingPeerResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RequestStatus>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(PeersPingPeerResponse::AnErrorOccured(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn settings_get_settings(&self, context: &C) -> Result<SettingsGetSettingsResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/settings/", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("GET").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Settings>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(SettingsGetSettingsResponse::SettingsFetchedSuccesfully(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(SettingsGetSettingsResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(SettingsGetSettingsResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(SettingsGetSettingsResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn settings_set_setting(
        &self,
        param_setting: String,
        param_settings_set_setting_request: Option<models::SettingsSetSettingRequest>,
        context: &C,
    ) -> Result<SettingsSetSettingResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/api/v3/settings/{setting}",
            self.base_path,
            setting = utf8_percent_encode(&param_setting.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("PUT").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let body = param_settings_set_setting_request
            .map(|ref body| serde_json::to_string(body).expect("impossible to fail to serialize"));

        if let Some(body) = body {
            *request.body_mut() = Body::from(body);
        }

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create header: {} - {}", header, e))),
            },
        );

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            204 => Ok(SettingsSetSettingResponse::SettingSetSuccesfully),
            400 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RequestStatus>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(SettingsSetSettingResponse::InvalidInput(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(SettingsSetSettingResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(SettingsSetSettingResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(SettingsSetSettingResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn tickets_get_statistics(&self, context: &C) -> Result<TicketsGetStatisticsResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/tickets/statistics", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("GET").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TicketsGetStatistics200Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(TicketsGetStatisticsResponse::TicketsStatisticsFetchedSuccessfully(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(TicketsGetStatisticsResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(TicketsGetStatisticsResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(TicketsGetStatisticsResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn tickets_get_tickets(&self, context: &C) -> Result<TicketsGetTicketsResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/tickets/", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("GET").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<Vec<models::Ticket>>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(TicketsGetTicketsResponse::TicketsFetchedSuccessfully(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(TicketsGetTicketsResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(TicketsGetTicketsResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(TicketsGetTicketsResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn tickets_redeem_tickets(&self, context: &C) -> Result<TicketsRedeemTicketsResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/tickets/redeem", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("POST").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            204 => Ok(TicketsRedeemTicketsResponse::TicketsRedeemedSuccesfully),
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(TicketsRedeemTicketsResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(TicketsRedeemTicketsResponse::AuthorizationFailed(body))
            }
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(TicketsRedeemTicketsResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn tokens_create(
        &self,
        param_tokens_create_request: Option<models::TokensCreateRequest>,
        context: &C,
    ) -> Result<TokensCreateResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/tokens/", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("POST").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        // Body parameter
        let body = param_tokens_create_request
            .map(|ref body| serde_json::to_string(body).expect("impossible to fail to serialize"));
        if let Some(body) = body {
            *request.body_mut() = Body::from(body);
        }

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create header: {} - {}", header, e))),
            },
        );
        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            201 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate201Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(TokensCreateResponse::TokenSuccesfullyCreated(body))
            }
            400 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RequestStatus>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(TokensCreateResponse::ProblemWithInputs(body))
            }
            403 => Ok(TokensCreateResponse::MissingCapabilityToAccessEndpoint),
            422 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::TokensCreate422Response>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(TokensCreateResponse::UnknownFailure(body))
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn tokens_delete(&self, param_id: String, context: &C) -> Result<TokensDeleteResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/api/v3/tokens/{id}",
            self.base_path,
            id = utf8_percent_encode(&param_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("DELETE").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            204 => Ok(TokensDeleteResponse::TokenSuccessfullyDeleted),
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(TokensDeleteResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(TokensDeleteResponse::AuthorizationFailed(body))
            }
            404 => Ok(TokensDeleteResponse::TheSpecifiedResourceWasNotFound),
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }

    async fn tokens_get_token(&self, context: &C) -> Result<TokensGetTokenResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/api/v3/token", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder().method("GET").uri(uri).body(Body::empty()) {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {}", e))),
            },
        );

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Basic(ref basic_header) => {
                    let auth = swagger::auth::Header(basic_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {}", e))),
                    };
                    request.headers_mut().insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Token>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(TokensGetTokenResponse::TokenInformation(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(TokensGetTokenResponse::AuthenticationFailed(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body =
                    str::from_utf8(&body).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(TokensGetTokenResponse::AuthorizationFailed(body))
            }
            404 => Ok(TokensGetTokenResponse::TheSpecifiedResourceWasNotFound),
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }
}
