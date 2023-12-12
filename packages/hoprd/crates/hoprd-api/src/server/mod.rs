use futures::{future, future::BoxFuture, future::FutureExt, stream, stream::TryStreamExt, Stream};
use hyper::header::{HeaderName, HeaderValue, CONTENT_TYPE};
use hyper::{Body, HeaderMap, Request, Response, StatusCode};
use log::warn;
#[allow(unused_imports)]
use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::future::Future;
use std::marker::PhantomData;
use std::task::{Context, Poll};
pub use swagger::auth::Authorization;
use swagger::auth::Scopes;
use swagger::{ApiError, BodyExt, Has, RequestParser, XSpanIdString};
use url::form_urlencoded;

use crate::header;
#[allow(unused_imports)]
use crate::models;

pub use crate::context;

type ServiceFuture = BoxFuture<'static, Result<Response<Body>, crate::ServiceError>>;

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

mod paths {
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref GLOBAL_REGEX_SET: regex::RegexSet = regex::RegexSet::new(vec![
            r"^/api/v3/account/address$",
            r"^/api/v3/account/addresses$",
            r"^/api/v3/account/balances$",
            r"^/api/v3/account/withdraw$",
            r"^/api/v3/aliases/$",
            r"^/api/v3/aliases/(?P<alias>[^/?#]*)$",
            r"^/api/v3/channels/$",
            r"^/api/v3/channels/(?P<channelid>[^/?#]*)/$",
            r"^/api/v3/channels/(?P<channelid>[^/?#]*)/fund$",
            r"^/api/v3/channels/(?P<channelid>[^/?#]*)/tickets$",
            r"^/api/v3/channels/(?P<channelid>[^/?#]*)/tickets/aggregate$",
            r"^/api/v3/channels/(?P<channelid>[^/?#]*)/tickets/redeem$",
            r"^/api/v3/healthyz/$",
            r"^/api/v3/messages/$",
            r"^/api/v3/messages/pop$",
            r"^/api/v3/messages/pop-all$",
            r"^/api/v3/messages/size$",
            r"^/api/v3/messages/websocket$",
            r"^/api/v3/node/entryNodes$",
            r"^/api/v3/node/info$",
            r"^/api/v3/node/metrics$",
            r"^/api/v3/node/peers$",
            r"^/api/v3/node/version$",
            r"^/api/v3/peers/(?P<peerid>[^/?#]*)/$",
            r"^/api/v3/peers/(?P<peerid>[^/?#]*)/ping$",
            r"^/api/v3/readyz/$",
            r"^/api/v3/settings/$",
            r"^/api/v3/settings/(?P<setting>[^/?#]*)$",
            r"^/api/v3/startedz/$",
            r"^/api/v3/tickets/$",
            r"^/api/v3/tickets/redeem$",
            r"^/api/v3/tickets/statistics$",
            r"^/api/v3/token$",
            r"^/api/v3/tokens/$",
            r"^/api/v3/tokens/(?P<id>[^/?#]*)$"
        ])
        .expect("Unable to create global regex set");
    }
    pub(crate) static ID_ACCOUNT_ADDRESS: usize = 0;
    pub(crate) static ID_ACCOUNT_ADDRESSES: usize = 1;
    pub(crate) static ID_ACCOUNT_BALANCES: usize = 2;
    pub(crate) static ID_ACCOUNT_WITHDRAW: usize = 3;
    pub(crate) static ID_ALIASES_: usize = 4;
    pub(crate) static ID_ALIASES_ALIAS: usize = 5;
    lazy_static! {
        pub static ref REGEX_ALIASES_ALIAS: regex::Regex = #[allow(clippy::invalid_regex)]
        regex::Regex::new(r"^/api/v3/aliases/(?P<alias>[^/?#]*)$")
            .expect("Unable to create regex for ALIASES_ALIAS");
    }
    pub(crate) static ID_CHANNELS_: usize = 6;
    pub(crate) static ID_CHANNELS_CHANNELID_: usize = 7;
    lazy_static! {
        pub static ref REGEX_CHANNELS_CHANNELID_: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/api/v3/channels/(?P<channelid>[^/?#]*)/$")
                .expect("Unable to create regex for CHANNELS_CHANNELID_");
    }
    pub(crate) static ID_CHANNELS_CHANNELID_FUND: usize = 8;
    lazy_static! {
        pub static ref REGEX_CHANNELS_CHANNELID_FUND: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/api/v3/channels/(?P<channelid>[^/?#]*)/fund$")
                .expect("Unable to create regex for CHANNELS_CHANNELID_FUND");
    }
    pub(crate) static ID_CHANNELS_CHANNELID_TICKETS: usize = 9;
    lazy_static! {
        pub static ref REGEX_CHANNELS_CHANNELID_TICKETS: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/api/v3/channels/(?P<channelid>[^/?#]*)/tickets$")
                .expect("Unable to create regex for CHANNELS_CHANNELID_TICKETS");
    }
    pub(crate) static ID_CHANNELS_CHANNELID_TICKETS_AGGREGATE: usize = 10;
    lazy_static! {
        pub static ref REGEX_CHANNELS_CHANNELID_TICKETS_AGGREGATE: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/api/v3/channels/(?P<channelid>[^/?#]*)/tickets/aggregate$")
                .expect("Unable to create regex for CHANNELS_CHANNELID_TICKETS_AGGREGATE");
    }
    pub(crate) static ID_CHANNELS_CHANNELID_TICKETS_REDEEM: usize = 11;
    lazy_static! {
        pub static ref REGEX_CHANNELS_CHANNELID_TICKETS_REDEEM: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/api/v3/channels/(?P<channelid>[^/?#]*)/tickets/redeem$")
                .expect("Unable to create regex for CHANNELS_CHANNELID_TICKETS_REDEEM");
    }
    pub(crate) static ID_HEALTHYZ_: usize = 12;
    pub(crate) static ID_MESSAGES_: usize = 13;
    pub(crate) static ID_MESSAGES_POP: usize = 14;
    pub(crate) static ID_MESSAGES_POP_ALL: usize = 15;
    pub(crate) static ID_MESSAGES_SIZE: usize = 16;
    pub(crate) static ID_MESSAGES_WEBSOCKET: usize = 17;
    pub(crate) static ID_NODE_ENTRYNODES: usize = 18;
    pub(crate) static ID_NODE_INFO: usize = 19;
    pub(crate) static ID_NODE_METRICS: usize = 20;
    pub(crate) static ID_NODE_PEERS: usize = 21;
    pub(crate) static ID_NODE_VERSION: usize = 22;
    pub(crate) static ID_PEERS_PEERID_: usize = 23;
    lazy_static! {
        pub static ref REGEX_PEERS_PEERID_: regex::Regex = #[allow(clippy::invalid_regex)]
        regex::Regex::new(r"^/api/v3/peers/(?P<peerid>[^/?#]*)/$")
            .expect("Unable to create regex for PEERS_PEERID_");
    }
    pub(crate) static ID_PEERS_PEERID_PING: usize = 24;
    lazy_static! {
        pub static ref REGEX_PEERS_PEERID_PING: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/api/v3/peers/(?P<peerid>[^/?#]*)/ping$")
                .expect("Unable to create regex for PEERS_PEERID_PING");
    }
    pub(crate) static ID_READYZ_: usize = 25;
    pub(crate) static ID_SETTINGS_: usize = 26;
    pub(crate) static ID_SETTINGS_SETTING: usize = 27;
    lazy_static! {
        pub static ref REGEX_SETTINGS_SETTING: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/api/v3/settings/(?P<setting>[^/?#]*)$")
                .expect("Unable to create regex for SETTINGS_SETTING");
    }
    pub(crate) static ID_STARTEDZ_: usize = 28;
    pub(crate) static ID_TICKETS_: usize = 29;
    pub(crate) static ID_TICKETS_REDEEM: usize = 30;
    pub(crate) static ID_TICKETS_STATISTICS: usize = 31;
    pub(crate) static ID_TOKEN: usize = 32;
    pub(crate) static ID_TOKENS_: usize = 33;
    pub(crate) static ID_TOKENS_ID: usize = 34;
    lazy_static! {
        pub static ref REGEX_TOKENS_ID: regex::Regex = #[allow(clippy::invalid_regex)]
        regex::Regex::new(r"^/api/v3/tokens/(?P<id>[^/?#]*)$")
            .expect("Unable to create regex for TOKENS_ID");
    }
}

pub struct MakeService<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    api_impl: T,
    marker: PhantomData<C>,
}

impl<T, C> MakeService<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    pub fn new(api_impl: T) -> Self {
        MakeService {
            api_impl,
            marker: PhantomData,
        }
    }
}

impl<T, C, Target> hyper::service::Service<Target> for MakeService<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    type Response = Service<T, C>;
    type Error = crate::ServiceError;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, target: Target) -> Self::Future {
        futures::future::ok(Service::new(self.api_impl.clone()))
    }
}

fn method_not_allowed() -> Result<Response<Body>, crate::ServiceError> {
    Ok(Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .body(Body::empty())
        .expect("Unable to create Method Not Allowed response"))
}

pub struct Service<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    api_impl: T,
    marker: PhantomData<C>,
}

impl<T, C> Service<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    pub fn new(api_impl: T) -> Self {
        Service {
            api_impl,
            marker: PhantomData,
        }
    }
}

impl<T, C> Clone for Service<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Service {
            api_impl: self.api_impl.clone(),
            marker: self.marker,
        }
    }
}

impl<T, C> hyper::service::Service<(Request<Body>, C)> for Service<T, C>
where
    T: Api<C> + Clone + Send + Sync + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    type Response = Response<Body>;
    type Error = crate::ServiceError;
    type Future = ServiceFuture;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.api_impl.poll_ready(cx)
    }

    fn call(&mut self, req: (Request<Body>, C)) -> Self::Future {
        async fn run<T, C>(mut api_impl: T, req: (Request<Body>, C)) -> Result<Response<Body>, crate::ServiceError>
        where
            T: Api<C> + Clone + Send + 'static,
            C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
        {
            let (request, context) = req;
            let (parts, body) = request.into_parts();
            let (method, uri, headers) = (parts.method, parts.uri, parts.headers);
            let path = paths::GLOBAL_REGEX_SET.matches(uri.path());

            match method {
                // AccountGetAddress - GET /account/address
                hyper::Method::GET if path.matched(paths::ID_ACCOUNT_ADDRESS) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    let result = api_impl.account_get_address(&context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => {
                            match rsp {
                                AccountGetAddressResponse::AddressesFetchedSuccessfully(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ACCOUNT_GET_ADDRESS_ADDRESSES_FETCHED_SUCCESSFULLY"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                AccountGetAddressResponse::AuthenticationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ACCOUNT_GET_ADDRESS_AUTHENTICATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                AccountGetAddressResponse::AuthorizationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ACCOUNT_GET_ADDRESS_AUTHORIZATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                AccountGetAddressResponse::UnknownFailure(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ACCOUNT_GET_ADDRESS_UNKNOWN_FAILURE"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                            }
                        }
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // AccountGetAddresses - GET /account/addresses
                hyper::Method::GET if path.matched(paths::ID_ACCOUNT_ADDRESSES) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    let result = api_impl.account_get_addresses(&context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => match rsp {
                            AccountGetAddressesResponse::AddressesFetchedSuccessfully(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ACCOUNT_GET_ADDRESSES_ADDRESSES_FETCHED_SUCCESSFULLY"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            AccountGetAddressesResponse::AuthenticationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ACCOUNT_GET_ADDRESSES_AUTHENTICATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            AccountGetAddressesResponse::AuthorizationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ACCOUNT_GET_ADDRESSES_AUTHORIZATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            AccountGetAddressesResponse::UnknownFailure(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ACCOUNT_GET_ADDRESSES_UNKNOWN_FAILURE"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                        },
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // AccountGetBalances - GET /account/balances
                hyper::Method::GET if path.matched(paths::ID_ACCOUNT_BALANCES) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    let result = api_impl.account_get_balances(&context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => {
                            match rsp {
                                AccountGetBalancesResponse::BalancesFetchedSuccessfuly(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ACCOUNT_GET_BALANCES_BALANCES_FETCHED_SUCCESSFULY"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                AccountGetBalancesResponse::AuthenticationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ACCOUNT_GET_BALANCES_AUTHENTICATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                AccountGetBalancesResponse::AuthorizationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ACCOUNT_GET_BALANCES_AUTHORIZATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                AccountGetBalancesResponse::UnknownFailure(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ACCOUNT_GET_BALANCES_UNKNOWN_FAILURE"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                            }
                        }
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // AccountWithdraw - POST /account/withdraw
                hyper::Method::POST if path.matched(paths::ID_ACCOUNT_WITHDRAW) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_account_withdraw_request: Option<models::AccountWithdrawRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_account_withdraw_request) => param_account_withdraw_request,
                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                };

                                let result = api_impl.account_withdraw(
                                            param_account_withdraw_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                AccountWithdrawResponse::WithdrawSuccessful
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ACCOUNT_WITHDRAW_WITHDRAW_SUCCESSFUL"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                AccountWithdrawResponse::IncorrectDataInRequestBody
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ACCOUNT_WITHDRAW_INCORRECT_DATA_IN_REQUEST_BODY"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                AccountWithdrawResponse::AuthenticationFailed
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ACCOUNT_WITHDRAW_AUTHENTICATION_FAILED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                AccountWithdrawResponse::AuthorizationFailed
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ACCOUNT_WITHDRAW_AUTHORIZATION_FAILED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                AccountWithdrawResponse::WithdrawAmountExeedsCurrentBalanceOrUnknownError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ACCOUNT_WITHDRAW_WITHDRAW_AMOUNT_EXEEDS_CURRENT_BALANCE_OR_UNKNOWN_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter AccountWithdrawRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter AccountWithdrawRequest")),
                        }
                }

                // AliasesGetAlias - GET /aliases/{alias}
                hyper::Method::GET if path.matched(paths::ID_ALIASES_ALIAS) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    // Path parameters
                    let path: &str = uri.path();
                    let path_params = paths::REGEX_ALIASES_ALIAS.captures(path).unwrap_or_else(|| {
                        panic!(
                            "Path {} matched RE ALIASES_ALIAS in set but failed match against \"{}\"",
                            path,
                            paths::REGEX_ALIASES_ALIAS.as_str()
                        )
                    });

                    let param_alias =
                        match percent_encoding::percent_decode(path_params["alias"].as_bytes()).decode_utf8() {
                            Ok(param_alias) => match param_alias.parse::<String>() {
                                Ok(param_alias) => param_alias,
                                Err(e) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter alias: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter"))
                                }
                            },
                            Err(_) => {
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(Body::from(format!(
                                        "Couldn't percent-decode path parameter as UTF-8: {}",
                                        &path_params["alias"]
                                    )))
                                    .expect("Unable to create Bad Request response for invalid percent decode"))
                            }
                        };

                    let result = api_impl.aliases_get_alias(param_alias, &context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => {
                            match rsp {
                                AliasesGetAliasResponse::HOPRAddressWasFoundForTheProvidedAlias(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ALIASES_GET_ALIAS_HOPR_ADDRESS_WAS_FOUND_FOR_THE_PROVIDED_ALIAS"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                AliasesGetAliasResponse::AuthenticationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ALIASES_GET_ALIAS_AUTHENTICATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                AliasesGetAliasResponse::AuthorizationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ALIASES_GET_ALIAS_AUTHORIZATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                AliasesGetAliasResponse::ThisAliasWasNotAssignedToAnyPeerIdBefore(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ALIASES_GET_ALIAS_THIS_ALIAS_WAS_NOT_ASSIGNED_TO_ANY_PEER_ID_BEFORE"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                AliasesGetAliasResponse::UnknownFailure(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ALIASES_GET_ALIAS_UNKNOWN_FAILURE"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                            }
                        }
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // AliasesGetAliases - GET /aliases/
                hyper::Method::GET if path.matched(paths::ID_ALIASES_) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    let result = api_impl.aliases_get_aliases(&context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => {
                            match rsp {
                                AliasesGetAliasesResponse::ReturnsListOfAliasesAndCorrespondingPeerIds(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ALIASES_GET_ALIASES_RETURNS_LIST_OF_ALIASES_AND_CORRESPONDING_PEER_IDS"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                AliasesGetAliasesResponse::UnknownFailure(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ALIASES_GET_ALIASES_UNKNOWN_FAILURE"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                            }
                        }
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // AliasesRemoveAlias - DELETE /aliases/{alias}
                hyper::Method::DELETE if path.matched(paths::ID_ALIASES_ALIAS) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    // Path parameters
                    let path: &str = uri.path();
                    let path_params = paths::REGEX_ALIASES_ALIAS.captures(path).unwrap_or_else(|| {
                        panic!(
                            "Path {} matched RE ALIASES_ALIAS in set but failed match against \"{}\"",
                            path,
                            paths::REGEX_ALIASES_ALIAS.as_str()
                        )
                    });

                    let param_alias =
                        match percent_encoding::percent_decode(path_params["alias"].as_bytes()).decode_utf8() {
                            Ok(param_alias) => match param_alias.parse::<String>() {
                                Ok(param_alias) => param_alias,
                                Err(e) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter alias: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter"))
                                }
                            },
                            Err(_) => {
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(Body::from(format!(
                                        "Couldn't percent-decode path parameter as UTF-8: {}",
                                        &path_params["alias"]
                                    )))
                                    .expect("Unable to create Bad Request response for invalid percent decode"))
                            }
                        };

                    let result = api_impl.aliases_remove_alias(param_alias, &context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => {
                            match rsp {
                                AliasesRemoveAliasResponse::AliasRemovedSuccesfully => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                                }
                                AliasesRemoveAliasResponse::AuthenticationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ALIASES_REMOVE_ALIAS_AUTHENTICATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                AliasesRemoveAliasResponse::AuthorizationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ALIASES_REMOVE_ALIAS_AUTHORIZATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                AliasesRemoveAliasResponse::UnknownFailure(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ALIASES_REMOVE_ALIAS_UNKNOWN_FAILURE"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                            }
                        }
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // AliasesSetAlias - POST /aliases/
                hyper::Method::POST if path.matched(paths::ID_ALIASES_) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_aliases_set_alias_request: Option<models::AliasesSetAliasRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_aliases_set_alias_request) => param_aliases_set_alias_request,
                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                };

                                let result = api_impl.aliases_set_alias(
                                            param_aliases_set_alias_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                AliasesSetAliasResponse::AliasSetSuccesfully
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(201).expect("Unable to turn 201 into a StatusCode");
                                                },
                                                AliasesSetAliasResponse::InvalidPeerId
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ALIASES_SET_ALIAS_INVALID_PEER_ID"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                AliasesSetAliasResponse::AuthenticationFailed
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ALIASES_SET_ALIAS_AUTHENTICATION_FAILED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                AliasesSetAliasResponse::AuthorizationFailed
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ALIASES_SET_ALIAS_AUTHORIZATION_FAILED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                AliasesSetAliasResponse::UnknownFailure
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ALIASES_SET_ALIAS_UNKNOWN_FAILURE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter AliasesSetAliasRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter AliasesSetAliasRequest")),
                        }
                }

                // ChannelsAggregateTickets - POST /channels/{channelid}/tickets/aggregate
                hyper::Method::POST if path.matched(paths::ID_CHANNELS_CHANNELID_TICKETS_AGGREGATE) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    // Path parameters
                    let path: &str = uri.path();
                    let path_params =
                    paths::REGEX_CHANNELS_CHANNELID_TICKETS_AGGREGATE
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE CHANNELS_CHANNELID_TICKETS_AGGREGATE in set but failed match against \"{}\"", path, paths::REGEX_CHANNELS_CHANNELID_TICKETS_AGGREGATE.as_str())
                    );

                    let param_channelid =
                        match percent_encoding::percent_decode(path_params["channelid"].as_bytes()).decode_utf8() {
                            Ok(param_channelid) => match param_channelid.parse::<String>() {
                                Ok(param_channelid) => param_channelid,
                                Err(e) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter channelid: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter"))
                                }
                            },
                            Err(_) => {
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(Body::from(format!(
                                        "Couldn't percent-decode path parameter as UTF-8: {}",
                                        &path_params["channelid"]
                                    )))
                                    .expect("Unable to create Bad Request response for invalid percent decode"))
                            }
                        };

                    let result = api_impl.channels_aggregate_tickets(param_channelid, &context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => match rsp {
                            ChannelsAggregateTicketsResponse::TicketsSuccessfullyAggregated => {
                                *response.status_mut() =
                                    StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                            }
                            ChannelsAggregateTicketsResponse::InvalidChannelId(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_AGGREGATE_TICKETS_INVALID_CHANNEL_ID"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            ChannelsAggregateTicketsResponse::AuthenticationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_AGGREGATE_TICKETS_AUTHENTICATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            ChannelsAggregateTicketsResponse::AuthorizationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_AGGREGATE_TICKETS_AUTHORIZATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            ChannelsAggregateTicketsResponse::TheSpecifiedResourceWasNotFound => {
                                *response.status_mut() =
                                    StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                            }
                            ChannelsAggregateTicketsResponse::UnknownFailure(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_AGGREGATE_TICKETS_UNKNOWN_FAILURE"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                        },
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // ChannelsCloseChannel - DELETE /channels/{channelid}/
                hyper::Method::DELETE if path.matched(paths::ID_CHANNELS_CHANNELID_) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    // Path parameters
                    let path: &str = uri.path();
                    let path_params = paths::REGEX_CHANNELS_CHANNELID_.captures(path).unwrap_or_else(|| {
                        panic!(
                            "Path {} matched RE CHANNELS_CHANNELID_ in set but failed match against \"{}\"",
                            path,
                            paths::REGEX_CHANNELS_CHANNELID_.as_str()
                        )
                    });

                    let param_channelid =
                        match percent_encoding::percent_decode(path_params["channelid"].as_bytes()).decode_utf8() {
                            Ok(param_channelid) => match param_channelid.parse::<String>() {
                                Ok(param_channelid) => param_channelid,
                                Err(e) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter channelid: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter"))
                                }
                            },
                            Err(_) => {
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(Body::from(format!(
                                        "Couldn't percent-decode path parameter as UTF-8: {}",
                                        &path_params["channelid"]
                                    )))
                                    .expect("Unable to create Bad Request response for invalid percent decode"))
                            }
                        };

                    let result = api_impl.channels_close_channel(param_channelid, &context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => match rsp {
                            ChannelsCloseChannelResponse::ChannelClosedSuccesfully(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_CLOSE_CHANNEL_CHANNEL_CLOSED_SUCCESFULLY"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            ChannelsCloseChannelResponse::InvalidChannelId(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_CLOSE_CHANNEL_INVALID_CHANNEL_ID"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            ChannelsCloseChannelResponse::AuthenticationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_CLOSE_CHANNEL_AUTHENTICATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            ChannelsCloseChannelResponse::AuthorizationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_CLOSE_CHANNEL_AUTHORIZATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            ChannelsCloseChannelResponse::UnknownFailure(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_CLOSE_CHANNEL_UNKNOWN_FAILURE"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                        },
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // ChannelsFundChannel - POST /channels/{channelid}/fund
                hyper::Method::POST if path.matched(paths::ID_CHANNELS_CHANNELID_FUND) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    // Path parameters
                    let path: &str = uri.path();
                    let path_params = paths::REGEX_CHANNELS_CHANNELID_FUND.captures(path).unwrap_or_else(|| {
                        panic!(
                            "Path {} matched RE CHANNELS_CHANNELID_FUND in set but failed match against \"{}\"",
                            path,
                            paths::REGEX_CHANNELS_CHANNELID_FUND.as_str()
                        )
                    });

                    let param_channelid =
                        match percent_encoding::percent_decode(path_params["channelid"].as_bytes()).decode_utf8() {
                            Ok(param_channelid) => match param_channelid.parse::<String>() {
                                Ok(param_channelid) => param_channelid,
                                Err(e) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter channelid: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter"))
                                }
                            },
                            Err(_) => {
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(Body::from(format!(
                                        "Couldn't percent-decode path parameter as UTF-8: {}",
                                        &path_params["channelid"]
                                    )))
                                    .expect("Unable to create Bad Request response for invalid percent decode"))
                            }
                        };

                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_channels_fund_channel_request: Option<models::ChannelsFundChannelRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_channels_fund_channel_request) => param_channels_fund_channel_request,
                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                };

                                let result = api_impl.channels_fund_channel(
                                            param_channelid,
                                            param_channels_fund_channel_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                ChannelsFundChannelResponse::ChannelFundedSuccessfully
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_FUND_CHANNEL_CHANNEL_FUNDED_SUCCESSFULLY"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ChannelsFundChannelResponse::InvalidChannelId
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_FUND_CHANNEL_INVALID_CHANNEL_ID"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ChannelsFundChannelResponse::AuthenticationFailed
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_FUND_CHANNEL_AUTHENTICATION_FAILED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ChannelsFundChannelResponse::AuthorizationFailed
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_FUND_CHANNEL_AUTHORIZATION_FAILED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ChannelsFundChannelResponse::TheSpecifiedResourceWasNotFound
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                },
                                                ChannelsFundChannelResponse::UnknownFailure
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_FUND_CHANNEL_UNKNOWN_FAILURE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter ChannelsFundChannelRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter ChannelsFundChannelRequest")),
                        }
                }

                // ChannelsGetChannel - GET /channels/{channelid}/
                hyper::Method::GET if path.matched(paths::ID_CHANNELS_CHANNELID_) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    // Path parameters
                    let path: &str = uri.path();
                    let path_params = paths::REGEX_CHANNELS_CHANNELID_.captures(path).unwrap_or_else(|| {
                        panic!(
                            "Path {} matched RE CHANNELS_CHANNELID_ in set but failed match against \"{}\"",
                            path,
                            paths::REGEX_CHANNELS_CHANNELID_.as_str()
                        )
                    });

                    let param_channelid =
                        match percent_encoding::percent_decode(path_params["channelid"].as_bytes()).decode_utf8() {
                            Ok(param_channelid) => match param_channelid.parse::<serde_json::Value>() {
                                Ok(param_channelid) => param_channelid,
                                Err(e) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter channelid: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter"))
                                }
                            },
                            Err(_) => {
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(Body::from(format!(
                                        "Couldn't percent-decode path parameter as UTF-8: {}",
                                        &path_params["channelid"]
                                    )))
                                    .expect("Unable to create Bad Request response for invalid percent decode"))
                            }
                        };

                    let result = api_impl.channels_get_channel(param_channelid, &context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => {
                            match rsp {
                                ChannelsGetChannelResponse::ChannelFetchedSuccesfully(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_GET_CHANNEL_CHANNEL_FETCHED_SUCCESFULLY"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                ChannelsGetChannelResponse::InvalidChannelId(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_GET_CHANNEL_INVALID_CHANNEL_ID"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                ChannelsGetChannelResponse::AuthenticationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_GET_CHANNEL_AUTHENTICATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                ChannelsGetChannelResponse::AuthorizationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_GET_CHANNEL_AUTHORIZATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                ChannelsGetChannelResponse::TheSpecifiedResourceWasNotFound => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                }
                                ChannelsGetChannelResponse::UnknownFailure(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_GET_CHANNEL_UNKNOWN_FAILURE"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                            }
                        }
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // ChannelsGetChannels - GET /channels/
                hyper::Method::GET if path.matched(paths::ID_CHANNELS_) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes()).collect::<Vec<_>>();
                    let param_including_closed = query_params
                        .iter()
                        .filter(|e| e.0 == "includingClosed")
                        .map(|e| e.1.clone())
                        .next();
                    let param_including_closed = match param_including_closed {
                        Some(param_including_closed) => {
                            let param_including_closed =
                                <String as std::str::FromStr>::from_str(&param_including_closed);
                            match param_including_closed {
                            Ok(param_including_closed) => Some(param_including_closed),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter includingClosed - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter includingClosed")),
                        }
                        }
                        None => None,
                    };
                    let param_full_topology = query_params
                        .iter()
                        .filter(|e| e.0 == "fullTopology")
                        .map(|e| e.1.clone())
                        .next();
                    let param_full_topology = match param_full_topology {
                        Some(param_full_topology) => {
                            let param_full_topology = <String as std::str::FromStr>::from_str(&param_full_topology);
                            match param_full_topology {
                            Ok(param_full_topology) => Some(param_full_topology),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter fullTopology - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter fullTopology")),
                        }
                        }
                        None => None,
                    };

                    let result = api_impl
                        .channels_get_channels(param_including_closed, param_full_topology, &context)
                        .await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => match rsp {
                            ChannelsGetChannelsResponse::ChannelsFetchedSuccessfully(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_GET_CHANNELS_CHANNELS_FETCHED_SUCCESSFULLY"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            ChannelsGetChannelsResponse::AuthenticationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_GET_CHANNELS_AUTHENTICATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            ChannelsGetChannelsResponse::AuthorizationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_GET_CHANNELS_AUTHORIZATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            ChannelsGetChannelsResponse::UnknownFailure(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_GET_CHANNELS_UNKNOWN_FAILURE"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                        },
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // ChannelsGetTickets - GET /channels/{channelid}/tickets
                hyper::Method::GET if path.matched(paths::ID_CHANNELS_CHANNELID_TICKETS) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    // Path parameters
                    let path: &str = uri.path();
                    let path_params = paths::REGEX_CHANNELS_CHANNELID_TICKETS
                        .captures(path)
                        .unwrap_or_else(|| {
                            panic!(
                                "Path {} matched RE CHANNELS_CHANNELID_TICKETS in set but failed match against \"{}\"",
                                path,
                                paths::REGEX_CHANNELS_CHANNELID_TICKETS.as_str()
                            )
                        });

                    let param_channelid =
                        match percent_encoding::percent_decode(path_params["channelid"].as_bytes()).decode_utf8() {
                            Ok(param_channelid) => match param_channelid.parse::<String>() {
                                Ok(param_channelid) => param_channelid,
                                Err(e) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter channelid: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter"))
                                }
                            },
                            Err(_) => {
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(Body::from(format!(
                                        "Couldn't percent-decode path parameter as UTF-8: {}",
                                        &path_params["channelid"]
                                    )))
                                    .expect("Unable to create Bad Request response for invalid percent decode"))
                            }
                        };

                    let result = api_impl.channels_get_tickets(param_channelid, &context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => {
                            match rsp {
                                ChannelsGetTicketsResponse::TicketsFetchedSuccessfully(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_GET_TICKETS_TICKETS_FETCHED_SUCCESSFULLY"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                ChannelsGetTicketsResponse::InvalidPeerId(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_GET_TICKETS_INVALID_PEER_ID"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                ChannelsGetTicketsResponse::AuthenticationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_GET_TICKETS_AUTHENTICATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                ChannelsGetTicketsResponse::AuthorizationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_GET_TICKETS_AUTHORIZATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                ChannelsGetTicketsResponse::TicketsWereNotFoundForThatChannel(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_GET_TICKETS_TICKETS_WERE_NOT_FOUND_FOR_THAT_CHANNEL"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                ChannelsGetTicketsResponse::UnknownFailure(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_GET_TICKETS_UNKNOWN_FAILURE"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                            }
                        }
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // ChannelsOpenChannel - POST /channels/
                hyper::Method::POST if path.matched(paths::ID_CHANNELS_) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_channels_open_channel_request: Option<models::ChannelsOpenChannelRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_channels_open_channel_request) => param_channels_open_channel_request,
                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                };

                                let result = api_impl.channels_open_channel(
                                            param_channels_open_channel_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                ChannelsOpenChannelResponse::ChannelSuccesfullyOpened
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(201).expect("Unable to turn 201 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_OPEN_CHANNEL_CHANNEL_SUCCESFULLY_OPENED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ChannelsOpenChannelResponse::ProblemWithInputs
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_OPEN_CHANNEL_PROBLEM_WITH_INPUTS"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ChannelsOpenChannelResponse::AuthenticationFailed
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_OPEN_CHANNEL_AUTHENTICATION_FAILED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ChannelsOpenChannelResponse::FailedToOpenTheChannelBecauseOfInsufficientHOPRBalanceOrAllowance
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_OPEN_CHANNEL_FAILED_TO_OPEN_THE_CHANNEL_BECAUSE_OF_INSUFFICIENT_HOPR_BALANCE_OR_ALLOWANCE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ChannelsOpenChannelResponse::FailedToOpenTheChannelBecauseTheChannelBetweenThisNodesAlreadyExists
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(409).expect("Unable to turn 409 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_OPEN_CHANNEL_FAILED_TO_OPEN_THE_CHANNEL_BECAUSE_THE_CHANNEL_BETWEEN_THIS_NODES_ALREADY_EXISTS"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ChannelsOpenChannelResponse::UnknownFailure
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_OPEN_CHANNEL_UNKNOWN_FAILURE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter ChannelsOpenChannelRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter ChannelsOpenChannelRequest")),
                        }
                }

                // ChannelsRedeemTickets - POST /channels/{channelid}/tickets/redeem
                hyper::Method::POST if path.matched(paths::ID_CHANNELS_CHANNELID_TICKETS_REDEEM) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    // Path parameters
                    let path: &str = uri.path();
                    let path_params =
                    paths::REGEX_CHANNELS_CHANNELID_TICKETS_REDEEM
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE CHANNELS_CHANNELID_TICKETS_REDEEM in set but failed match against \"{}\"", path, paths::REGEX_CHANNELS_CHANNELID_TICKETS_REDEEM.as_str())
                    );

                    let param_channelid =
                        match percent_encoding::percent_decode(path_params["channelid"].as_bytes()).decode_utf8() {
                            Ok(param_channelid) => match param_channelid.parse::<String>() {
                                Ok(param_channelid) => param_channelid,
                                Err(e) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter channelid: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter"))
                                }
                            },
                            Err(_) => {
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(Body::from(format!(
                                        "Couldn't percent-decode path parameter as UTF-8: {}",
                                        &path_params["channelid"]
                                    )))
                                    .expect("Unable to create Bad Request response for invalid percent decode"))
                            }
                        };

                    let result = api_impl.channels_redeem_tickets(param_channelid, &context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => match rsp {
                            ChannelsRedeemTicketsResponse::TicketsRedeemedSuccessfully => {
                                *response.status_mut() =
                                    StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                            }
                            ChannelsRedeemTicketsResponse::InvalidChannelId(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_REDEEM_TICKETS_INVALID_CHANNEL_ID"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            ChannelsRedeemTicketsResponse::AuthenticationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_REDEEM_TICKETS_AUTHENTICATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            ChannelsRedeemTicketsResponse::AuthorizationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_REDEEM_TICKETS_AUTHORIZATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            ChannelsRedeemTicketsResponse::TicketsWereNotFoundForThatChannel(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_REDEEM_TICKETS_TICKETS_WERE_NOT_FOUND_FOR_THAT_CHANNEL"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            ChannelsRedeemTicketsResponse::UnknownFailure(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHANNELS_REDEEM_TICKETS_UNKNOWN_FAILURE"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                        },
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // CheckNodeHealthy - GET /healthyz/
                hyper::Method::GET if path.matched(paths::ID_HEALTHYZ_) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    let result = api_impl.check_node_healthy(&context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => {
                            match rsp {
                                CheckNodeHealthyResponse::TheNodeIsReady(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHECK_NODE_HEALTHY_THE_NODE_IS_READY"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                CheckNodeHealthyResponse::AuthenticationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHECK_NODE_HEALTHY_AUTHENTICATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                CheckNodeHealthyResponse::AuthorizationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHECK_NODE_HEALTHY_AUTHORIZATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                CheckNodeHealthyResponse::UnknownFailure(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHECK_NODE_HEALTHY_UNKNOWN_FAILURE"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                            }
                        }
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // CheckNodeReady - GET /readyz/
                hyper::Method::GET if path.matched(paths::ID_READYZ_) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    let result = api_impl.check_node_ready(&context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => {
                            match rsp {
                                CheckNodeReadyResponse::TheNodeIsReady(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHECK_NODE_READY_THE_NODE_IS_READY"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                CheckNodeReadyResponse::AuthenticationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHECK_NODE_READY_AUTHENTICATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                CheckNodeReadyResponse::AuthorizationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHECK_NODE_READY_AUTHORIZATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                CheckNodeReadyResponse::UnknownFailure(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                    response.headers_mut().insert(
                                        CONTENT_TYPE,
                                        HeaderValue::from_str("application/json").expect(
                                            "Unable to create Content-Type header for CHECK_NODE_READY_UNKNOWN_FAILURE",
                                        ),
                                    );
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                            }
                        }
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // CheckNodeStarted - GET /startedz/
                hyper::Method::GET if path.matched(paths::ID_STARTEDZ_) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    let result = api_impl.check_node_started(&context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => {
                            match rsp {
                                CheckNodeStartedResponse::TheNodeIsStarted(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHECK_NODE_STARTED_THE_NODE_IS_STARTED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                CheckNodeStartedResponse::AuthenticationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHECK_NODE_STARTED_AUTHENTICATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                CheckNodeStartedResponse::AuthorizationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHECK_NODE_STARTED_AUTHORIZATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                CheckNodeStartedResponse::UnknownFailure(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CHECK_NODE_STARTED_UNKNOWN_FAILURE"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                            }
                        }
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // MessagesDeleteMessages - DELETE /messages/
                hyper::Method::DELETE if path.matched(paths::ID_MESSAGES_) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes()).collect::<Vec<_>>();
                    let param_tag = query_params.iter().filter(|e| e.0 == "tag").map(|e| e.1.clone()).next();
                    let param_tag = match param_tag {
                        Some(param_tag) => {
                            let param_tag = <i32 as std::str::FromStr>::from_str(&param_tag);
                            match param_tag {
                                Ok(param_tag) => Some(param_tag),
                                Err(e) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!(
                                            "Couldn't parse query parameter tag - doesn't match schema: {}",
                                            e
                                        )))
                                        .expect(
                                            "Unable to create Bad Request response for invalid query parameter tag",
                                        ))
                                }
                            }
                        }
                        None => None,
                    };
                    let param_tag = match param_tag {
                        Some(param_tag) => param_tag,
                        None => {
                            return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from("Missing required query parameter tag"))
                                .expect("Unable to create Bad Request response for missing query parameter tag"))
                        }
                    };

                    let result = api_impl.messages_delete_messages(param_tag, &context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => match rsp {
                            MessagesDeleteMessagesResponse::MessagesSuccessfullyDeleted => {
                                *response.status_mut() =
                                    StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                            }
                            MessagesDeleteMessagesResponse::AuthenticationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MESSAGES_DELETE_MESSAGES_AUTHENTICATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            MessagesDeleteMessagesResponse::AuthorizationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MESSAGES_DELETE_MESSAGES_AUTHORIZATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                        },
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // MessagesGetSize - GET /messages/size
                hyper::Method::GET if path.matched(paths::ID_MESSAGES_SIZE) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes()).collect::<Vec<_>>();
                    let param_tag = query_params.iter().filter(|e| e.0 == "tag").map(|e| e.1.clone()).next();
                    let param_tag = match param_tag {
                        Some(param_tag) => {
                            let param_tag = <i32 as std::str::FromStr>::from_str(&param_tag);
                            match param_tag {
                                Ok(param_tag) => Some(param_tag),
                                Err(e) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!(
                                            "Couldn't parse query parameter tag - doesn't match schema: {}",
                                            e
                                        )))
                                        .expect(
                                            "Unable to create Bad Request response for invalid query parameter tag",
                                        ))
                                }
                            }
                        }
                        None => None,
                    };
                    let param_tag = match param_tag {
                        Some(param_tag) => param_tag,
                        None => {
                            return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from("Missing required query parameter tag"))
                                .expect("Unable to create Bad Request response for missing query parameter tag"))
                        }
                    };

                    let result = api_impl.messages_get_size(param_tag, &context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => {
                            match rsp {
                                MessagesGetSizeResponse::ReturnsTheMessageInboxSizeFilteredByTheGivenTag(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MESSAGES_GET_SIZE_RETURNS_THE_MESSAGE_INBOX_SIZE_FILTERED_BY_THE_GIVEN_TAG"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                MessagesGetSizeResponse::AuthenticationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MESSAGES_GET_SIZE_AUTHENTICATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                MessagesGetSizeResponse::AuthorizationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MESSAGES_GET_SIZE_AUTHORIZATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                MessagesGetSizeResponse::UnknownFailure(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MESSAGES_GET_SIZE_UNKNOWN_FAILURE"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                            }
                        }
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // MessagesPopAllMessage - POST /messages/pop-all
                hyper::Method::POST if path.matched(paths::ID_MESSAGES_POP_ALL) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_messages_pop_all_message_request: Option<models::MessagesPopAllMessageRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_messages_pop_all_message_request) => param_messages_pop_all_message_request,
                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                };

                                let result = api_impl.messages_pop_all_message(
                                            param_messages_pop_all_message_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                MessagesPopAllMessageResponse::ReturnsListOfMessages
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MESSAGES_POP_ALL_MESSAGE_RETURNS_LIST_OF_MESSAGES"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                MessagesPopAllMessageResponse::AuthenticationFailed
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MESSAGES_POP_ALL_MESSAGE_AUTHENTICATION_FAILED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                MessagesPopAllMessageResponse::AuthorizationFailed
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MESSAGES_POP_ALL_MESSAGE_AUTHORIZATION_FAILED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                MessagesPopAllMessageResponse::UnknownFailure
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MESSAGES_POP_ALL_MESSAGE_UNKNOWN_FAILURE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter MessagesPopAllMessageRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter MessagesPopAllMessageRequest")),
                        }
                }

                // MessagesPopMessage - POST /messages/pop
                hyper::Method::POST if path.matched(paths::ID_MESSAGES_POP) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_messages_pop_all_message_request: Option<models::MessagesPopAllMessageRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_messages_pop_all_message_request) => param_messages_pop_all_message_request,
                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                };

                                let result = api_impl.messages_pop_message(
                                            param_messages_pop_all_message_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                MessagesPopMessageResponse::ReturnsAMessage
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MESSAGES_POP_MESSAGE_RETURNS_A_MESSAGE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                MessagesPopMessageResponse::AuthenticationFailed
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MESSAGES_POP_MESSAGE_AUTHENTICATION_FAILED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                MessagesPopMessageResponse::AuthorizationFailed
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MESSAGES_POP_MESSAGE_AUTHORIZATION_FAILED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                MessagesPopMessageResponse::TheSpecifiedResourceWasNotFound
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                },
                                                MessagesPopMessageResponse::UnknownFailure
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MESSAGES_POP_MESSAGE_UNKNOWN_FAILURE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter MessagesPopAllMessageRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter MessagesPopAllMessageRequest")),
                        }
                }

                // MessagesSendMessage - POST /messages/
                hyper::Method::POST if path.matched(paths::ID_MESSAGES_) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_messages_send_message_request: Option<models::MessagesSendMessageRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_messages_send_message_request) => param_messages_send_message_request,
                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                };

                                let result = api_impl.messages_send_message(
                                            param_messages_send_message_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                MessagesSendMessageResponse::TheMessageWasSentSuccessfully
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(202).expect("Unable to turn 202 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MESSAGES_SEND_MESSAGE_THE_MESSAGE_WAS_SENT_SUCCESSFULLY"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                MessagesSendMessageResponse::AuthenticationFailed
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MESSAGES_SEND_MESSAGE_AUTHENTICATION_FAILED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                MessagesSendMessageResponse::AuthorizationFailed
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MESSAGES_SEND_MESSAGE_AUTHORIZATION_FAILED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                MessagesSendMessageResponse::UnknownFailure
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MESSAGES_SEND_MESSAGE_UNKNOWN_FAILURE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter MessagesSendMessageRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter MessagesSendMessageRequest")),
                        }
                }

                // MessagesWebsocket - GET /messages/websocket
                hyper::Method::GET if path.matched(paths::ID_MESSAGES_WEBSOCKET) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    let result = api_impl.messages_websocket(&context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => match rsp {
                            MessagesWebsocketResponse::SwitchingProtocols => {
                                *response.status_mut() =
                                    StatusCode::from_u16(101).expect("Unable to turn 101 into a StatusCode");
                            }
                            MessagesWebsocketResponse::IncomingData(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(206).expect("Unable to turn 206 into a StatusCode");
                                response.headers_mut().insert(
                                    CONTENT_TYPE,
                                    HeaderValue::from_str("application/text").expect(
                                        "Unable to create Content-Type header for MESSAGES_WEBSOCKET_INCOMING_DATA",
                                    ),
                                );
                                let body = body;
                                *response.body_mut() = Body::from(body);
                            }
                            MessagesWebsocketResponse::AuthenticationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MESSAGES_WEBSOCKET_AUTHENTICATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            MessagesWebsocketResponse::AuthorizationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MESSAGES_WEBSOCKET_AUTHORIZATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            MessagesWebsocketResponse::NotFound => {
                                *response.status_mut() =
                                    StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                            }
                        },
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // NodeGetEntryNodes - GET /node/entryNodes
                hyper::Method::GET if path.matched(paths::ID_NODE_ENTRYNODES) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    let result = api_impl.node_get_entry_nodes(&context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => {
                            match rsp {
                                NodeGetEntryNodesResponse::EntryNodeInformationFetchedSuccessfuly(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NODE_GET_ENTRY_NODES_ENTRY_NODE_INFORMATION_FETCHED_SUCCESSFULY"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                NodeGetEntryNodesResponse::AuthenticationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NODE_GET_ENTRY_NODES_AUTHENTICATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                NodeGetEntryNodesResponse::AuthorizationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NODE_GET_ENTRY_NODES_AUTHORIZATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                NodeGetEntryNodesResponse::UnknownFailure(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NODE_GET_ENTRY_NODES_UNKNOWN_FAILURE"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                            }
                        }
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // NodeGetInfo - GET /node/info
                hyper::Method::GET if path.matched(paths::ID_NODE_INFO) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    let result = api_impl.node_get_info(&context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => {
                            match rsp {
                                NodeGetInfoResponse::NodeInformationFetchedSuccessfuly(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NODE_GET_INFO_NODE_INFORMATION_FETCHED_SUCCESSFULY"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                NodeGetInfoResponse::AuthenticationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NODE_GET_INFO_AUTHENTICATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                NodeGetInfoResponse::AuthorizationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NODE_GET_INFO_AUTHORIZATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                NodeGetInfoResponse::UnknownFailure(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                    response.headers_mut().insert(
                                        CONTENT_TYPE,
                                        HeaderValue::from_str("application/json").expect(
                                            "Unable to create Content-Type header for NODE_GET_INFO_UNKNOWN_FAILURE",
                                        ),
                                    );
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                            }
                        }
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // NodeGetMetrics - GET /node/metrics
                hyper::Method::GET if path.matched(paths::ID_NODE_METRICS) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    let result = api_impl.node_get_metrics(&context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => match rsp {
                            NodeGetMetricsResponse::ReturnsTheEncodedSerializedMetrics(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("text/plain; version=0.0.4")
                                                            .expect("Unable to create Content-Type header for NODE_GET_METRICS_RETURNS_THE_ENCODED_SERIALIZED_METRICS"));
                                let body = body;
                                *response.body_mut() = Body::from(body);
                            }
                            NodeGetMetricsResponse::AuthenticationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NODE_GET_METRICS_AUTHENTICATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            NodeGetMetricsResponse::AuthorizationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NODE_GET_METRICS_AUTHORIZATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            NodeGetMetricsResponse::UnknownFailure(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                response.headers_mut().insert(
                                    CONTENT_TYPE,
                                    HeaderValue::from_str("application/json").expect(
                                        "Unable to create Content-Type header for NODE_GET_METRICS_UNKNOWN_FAILURE",
                                    ),
                                );
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                        },
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // NodeGetPeers - GET /node/peers
                hyper::Method::GET if path.matched(paths::ID_NODE_PEERS) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes()).collect::<Vec<_>>();
                    let param_quality = query_params
                        .iter()
                        .filter(|e| e.0 == "quality")
                        .map(|e| e.1.clone())
                        .next();
                    let param_quality =
                        match param_quality {
                            Some(param_quality) => {
                                let param_quality = <f64 as std::str::FromStr>::from_str(&param_quality);
                                match param_quality {
                                    Ok(param_quality) => Some(param_quality),
                                    Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!(
                                            "Couldn't parse query parameter quality - doesn't match schema: {}",
                                            e
                                        )))
                                        .expect(
                                            "Unable to create Bad Request response for invalid query parameter quality",
                                        )),
                                }
                            }
                            None => None,
                        };

                    let result = api_impl.node_get_peers(param_quality, &context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => {
                            match rsp {
                                NodeGetPeersResponse::PeersInformationFetchedSuccessfuly(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NODE_GET_PEERS_PEERS_INFORMATION_FETCHED_SUCCESSFULY"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                NodeGetPeersResponse::InvalidInput(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                    response.headers_mut().insert(
                                        CONTENT_TYPE,
                                        HeaderValue::from_str("application/json").expect(
                                            "Unable to create Content-Type header for NODE_GET_PEERS_INVALID_INPUT",
                                        ),
                                    );
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                NodeGetPeersResponse::AuthenticationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NODE_GET_PEERS_AUTHENTICATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                NodeGetPeersResponse::AuthorizationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NODE_GET_PEERS_AUTHORIZATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                NodeGetPeersResponse::UnknownFailure(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                    response.headers_mut().insert(
                                        CONTENT_TYPE,
                                        HeaderValue::from_str("application/json").expect(
                                            "Unable to create Content-Type header for NODE_GET_PEERS_UNKNOWN_FAILURE",
                                        ),
                                    );
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                            }
                        }
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // NodeGetVersion - GET /node/version
                hyper::Method::GET if path.matched(paths::ID_NODE_VERSION) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    let result = api_impl.node_get_version(&context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => match rsp {
                            NodeGetVersionResponse::ReturnsTheReleaseVersionOfTheRunningNode(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NODE_GET_VERSION_RETURNS_THE_RELEASE_VERSION_OF_THE_RUNNING_NODE"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            NodeGetVersionResponse::AuthenticationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NODE_GET_VERSION_AUTHENTICATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            NodeGetVersionResponse::AuthorizationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NODE_GET_VERSION_AUTHORIZATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            NodeGetVersionResponse::UnknownFailure(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                response.headers_mut().insert(
                                    CONTENT_TYPE,
                                    HeaderValue::from_str("application/json").expect(
                                        "Unable to create Content-Type header for NODE_GET_VERSION_UNKNOWN_FAILURE",
                                    ),
                                );
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                        },
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // PeerInfoGetPeerInfo - GET /peers/{peerid}/
                hyper::Method::GET if path.matched(paths::ID_PEERS_PEERID_) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    // Path parameters
                    let path: &str = uri.path();
                    let path_params = paths::REGEX_PEERS_PEERID_.captures(path).unwrap_or_else(|| {
                        panic!(
                            "Path {} matched RE PEERS_PEERID_ in set but failed match against \"{}\"",
                            path,
                            paths::REGEX_PEERS_PEERID_.as_str()
                        )
                    });

                    let param_peerid =
                        match percent_encoding::percent_decode(path_params["peerid"].as_bytes()).decode_utf8() {
                            Ok(param_peerid) => match param_peerid.parse::<String>() {
                                Ok(param_peerid) => param_peerid,
                                Err(e) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter peerid: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter"))
                                }
                            },
                            Err(_) => {
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(Body::from(format!(
                                        "Couldn't percent-decode path parameter as UTF-8: {}",
                                        &path_params["peerid"]
                                    )))
                                    .expect("Unable to create Bad Request response for invalid percent decode"))
                            }
                        };

                    let result = api_impl.peer_info_get_peer_info(param_peerid, &context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => match rsp {
                            PeerInfoGetPeerInfoResponse::PeerInformationFetchedSuccessfully(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for PEER_INFO_GET_PEER_INFO_PEER_INFORMATION_FETCHED_SUCCESSFULLY"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            PeerInfoGetPeerInfoResponse::AuthenticationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for PEER_INFO_GET_PEER_INFO_AUTHENTICATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            PeerInfoGetPeerInfoResponse::AuthorizationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for PEER_INFO_GET_PEER_INFO_AUTHORIZATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            PeerInfoGetPeerInfoResponse::UnknownFailure(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for PEER_INFO_GET_PEER_INFO_UNKNOWN_FAILURE"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                        },
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // PeersPingPeer - POST /peers/{peerid}/ping
                hyper::Method::POST if path.matched(paths::ID_PEERS_PEERID_PING) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    // Path parameters
                    let path: &str = uri.path();
                    let path_params = paths::REGEX_PEERS_PEERID_PING.captures(path).unwrap_or_else(|| {
                        panic!(
                            "Path {} matched RE PEERS_PEERID_PING in set but failed match against \"{}\"",
                            path,
                            paths::REGEX_PEERS_PEERID_PING.as_str()
                        )
                    });

                    let param_peerid =
                        match percent_encoding::percent_decode(path_params["peerid"].as_bytes()).decode_utf8() {
                            Ok(param_peerid) => match param_peerid.parse::<String>() {
                                Ok(param_peerid) => param_peerid,
                                Err(e) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter peerid: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter"))
                                }
                            },
                            Err(_) => {
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(Body::from(format!(
                                        "Couldn't percent-decode path parameter as UTF-8: {}",
                                        &path_params["peerid"]
                                    )))
                                    .expect("Unable to create Bad Request response for invalid percent decode"))
                            }
                        };

                    let result = api_impl.peers_ping_peer(param_peerid, &context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => {
                            match rsp {
                                PeersPingPeerResponse::PingSuccessful(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                    response.headers_mut().insert(
                                        CONTENT_TYPE,
                                        HeaderValue::from_str("application/json").expect(
                                            "Unable to create Content-Type header for PEERS_PING_PEER_PING_SUCCESSFUL",
                                        ),
                                    );
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                PeersPingPeerResponse::InvalidPeerId(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                    response.headers_mut().insert(
                                        CONTENT_TYPE,
                                        HeaderValue::from_str("application/json").expect(
                                            "Unable to create Content-Type header for PEERS_PING_PEER_INVALID_PEER_ID",
                                        ),
                                    );
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                PeersPingPeerResponse::AuthenticationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for PEERS_PING_PEER_AUTHENTICATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                PeersPingPeerResponse::AuthorizationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for PEERS_PING_PEER_AUTHORIZATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                PeersPingPeerResponse::AnErrorOccured(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                    response.headers_mut().insert(
                                        CONTENT_TYPE,
                                        HeaderValue::from_str("application/json").expect(
                                            "Unable to create Content-Type header for PEERS_PING_PEER_AN_ERROR_OCCURED",
                                        ),
                                    );
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                            }
                        }
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // SettingsGetSettings - GET /settings/
                hyper::Method::GET if path.matched(paths::ID_SETTINGS_) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    let result = api_impl.settings_get_settings(&context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => match rsp {
                            SettingsGetSettingsResponse::SettingsFetchedSuccesfully(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SETTINGS_GET_SETTINGS_SETTINGS_FETCHED_SUCCESFULLY"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SettingsGetSettingsResponse::AuthenticationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SETTINGS_GET_SETTINGS_AUTHENTICATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SettingsGetSettingsResponse::AuthorizationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SETTINGS_GET_SETTINGS_AUTHORIZATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SettingsGetSettingsResponse::UnknownFailure(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SETTINGS_GET_SETTINGS_UNKNOWN_FAILURE"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                        },
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // SettingsSetSetting - PUT /settings/{setting}
                hyper::Method::PUT if path.matched(paths::ID_SETTINGS_SETTING) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    // Path parameters
                    let path: &str = uri.path();
                    let path_params = paths::REGEX_SETTINGS_SETTING.captures(path).unwrap_or_else(|| {
                        panic!(
                            "Path {} matched RE SETTINGS_SETTING in set but failed match against \"{}\"",
                            path,
                            paths::REGEX_SETTINGS_SETTING.as_str()
                        )
                    });

                    let param_setting =
                        match percent_encoding::percent_decode(path_params["setting"].as_bytes()).decode_utf8() {
                            Ok(param_setting) => match param_setting.parse::<String>() {
                                Ok(param_setting) => param_setting,
                                Err(e) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter setting: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter"))
                                }
                            },
                            Err(_) => {
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(Body::from(format!(
                                        "Couldn't percent-decode path parameter as UTF-8: {}",
                                        &path_params["setting"]
                                    )))
                                    .expect("Unable to create Bad Request response for invalid percent decode"))
                            }
                        };

                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_settings_set_setting_request: Option<models::SettingsSetSettingRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_settings_set_setting_request) => param_settings_set_setting_request,
                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                };

                                let result = api_impl.settings_set_setting(
                                            param_setting,
                                            param_settings_set_setting_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                SettingsSetSettingResponse::SettingSetSuccesfully
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                                                },
                                                SettingsSetSettingResponse::InvalidInput
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SETTINGS_SET_SETTING_INVALID_INPUT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SettingsSetSettingResponse::AuthenticationFailed
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SETTINGS_SET_SETTING_AUTHENTICATION_FAILED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SettingsSetSettingResponse::AuthorizationFailed
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SETTINGS_SET_SETTING_AUTHORIZATION_FAILED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SettingsSetSettingResponse::UnknownFailure
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SETTINGS_SET_SETTING_UNKNOWN_FAILURE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter SettingsSetSettingRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter SettingsSetSettingRequest")),
                        }
                }

                // TicketsGetStatistics - GET /tickets/statistics
                hyper::Method::GET if path.matched(paths::ID_TICKETS_STATISTICS) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    let result = api_impl.tickets_get_statistics(&context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => match rsp {
                            TicketsGetStatisticsResponse::TicketsStatisticsFetchedSuccessfully(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TICKETS_GET_STATISTICS_TICKETS_STATISTICS_FETCHED_SUCCESSFULLY"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            TicketsGetStatisticsResponse::AuthenticationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TICKETS_GET_STATISTICS_AUTHENTICATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            TicketsGetStatisticsResponse::AuthorizationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TICKETS_GET_STATISTICS_AUTHORIZATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            TicketsGetStatisticsResponse::UnknownFailure(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TICKETS_GET_STATISTICS_UNKNOWN_FAILURE"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                        },
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // TicketsGetTickets - GET /tickets/
                hyper::Method::GET if path.matched(paths::ID_TICKETS_) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    let result = api_impl.tickets_get_tickets(&context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => {
                            match rsp {
                                TicketsGetTicketsResponse::TicketsFetchedSuccessfully(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TICKETS_GET_TICKETS_TICKETS_FETCHED_SUCCESSFULLY"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                TicketsGetTicketsResponse::AuthenticationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TICKETS_GET_TICKETS_AUTHENTICATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                TicketsGetTicketsResponse::AuthorizationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TICKETS_GET_TICKETS_AUTHORIZATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                TicketsGetTicketsResponse::UnknownFailure(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TICKETS_GET_TICKETS_UNKNOWN_FAILURE"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                            }
                        }
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // TicketsRedeemTickets - POST /tickets/redeem
                hyper::Method::POST if path.matched(paths::ID_TICKETS_REDEEM) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    let result = api_impl.tickets_redeem_tickets(&context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => match rsp {
                            TicketsRedeemTicketsResponse::TicketsRedeemedSuccesfully => {
                                *response.status_mut() =
                                    StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                            }
                            TicketsRedeemTicketsResponse::AuthenticationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TICKETS_REDEEM_TICKETS_AUTHENTICATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            TicketsRedeemTicketsResponse::AuthorizationFailed(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TICKETS_REDEEM_TICKETS_AUTHORIZATION_FAILED"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            TicketsRedeemTicketsResponse::UnknownFailure(body) => {
                                *response.status_mut() =
                                    StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TICKETS_REDEEM_TICKETS_UNKNOWN_FAILURE"));
                                let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                        },
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // TokensCreate - POST /tokens/
                hyper::Method::POST if path.matched(paths::ID_TOKENS_) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_tokens_create_request: Option<models::TokensCreateRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_tokens_create_request) => param_tokens_create_request,
                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                };

                                let result = api_impl.tokens_create(
                                            param_tokens_create_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                TokensCreateResponse::TokenSuccesfullyCreated
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(201).expect("Unable to turn 201 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TOKENS_CREATE_TOKEN_SUCCESFULLY_CREATED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TokensCreateResponse::ProblemWithInputs
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TOKENS_CREATE_PROBLEM_WITH_INPUTS"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TokensCreateResponse::MissingCapabilityToAccessEndpoint
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                },
                                                TokensCreateResponse::UnknownFailure
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TOKENS_CREATE_UNKNOWN_FAILURE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter TokensCreateRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter TokensCreateRequest")),
                        }
                }

                // TokensDelete - DELETE /tokens/{id}
                hyper::Method::DELETE if path.matched(paths::ID_TOKENS_ID) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    // Path parameters
                    let path: &str = uri.path();
                    let path_params = paths::REGEX_TOKENS_ID.captures(path).unwrap_or_else(|| {
                        panic!(
                            "Path {} matched RE TOKENS_ID in set but failed match against \"{}\"",
                            path,
                            paths::REGEX_TOKENS_ID.as_str()
                        )
                    });

                    let param_id = match percent_encoding::percent_decode(path_params["id"].as_bytes()).decode_utf8() {
                        Ok(param_id) => match param_id.parse::<String>() {
                            Ok(param_id) => param_id,
                            Err(e) => {
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(Body::from(format!("Couldn't parse path parameter id: {}", e)))
                                    .expect("Unable to create Bad Request response for invalid path parameter"))
                            }
                        },
                        Err(_) => {
                            return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!(
                                    "Couldn't percent-decode path parameter as UTF-8: {}",
                                    &path_params["id"]
                                )))
                                .expect("Unable to create Bad Request response for invalid percent decode"))
                        }
                    };

                    let result = api_impl.tokens_delete(param_id, &context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => {
                            match rsp {
                                TokensDeleteResponse::TokenSuccessfullyDeleted => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                                }
                                TokensDeleteResponse::AuthenticationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TOKENS_DELETE_AUTHENTICATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                TokensDeleteResponse::AuthorizationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TOKENS_DELETE_AUTHORIZATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                TokensDeleteResponse::TheSpecifiedResourceWasNotFound => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                }
                            }
                        }
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // TokensGetToken - GET /token
                hyper::Method::GET if path.matched(paths::ID_TOKEN) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };
                    }

                    let result = api_impl.tokens_get_token(&context).await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                            .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => {
                            match rsp {
                                TokensGetTokenResponse::TokenInformation(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TOKENS_GET_TOKEN_TOKEN_INFORMATION"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                TokensGetTokenResponse::AuthenticationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TOKENS_GET_TOKEN_AUTHENTICATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                TokensGetTokenResponse::AuthorizationFailed(body) => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TOKENS_GET_TOKEN_AUTHORIZATION_FAILED"));
                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                    *response.body_mut() = Body::from(body);
                                }
                                TokensGetTokenResponse::TheSpecifiedResourceWasNotFound => {
                                    *response.status_mut() =
                                        StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                }
                            }
                        }
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                _ if path.matched(paths::ID_ACCOUNT_ADDRESS) => method_not_allowed(),
                _ if path.matched(paths::ID_ACCOUNT_ADDRESSES) => method_not_allowed(),
                _ if path.matched(paths::ID_ACCOUNT_BALANCES) => method_not_allowed(),
                _ if path.matched(paths::ID_ACCOUNT_WITHDRAW) => method_not_allowed(),
                _ if path.matched(paths::ID_ALIASES_) => method_not_allowed(),
                _ if path.matched(paths::ID_ALIASES_ALIAS) => method_not_allowed(),
                _ if path.matched(paths::ID_CHANNELS_) => method_not_allowed(),
                _ if path.matched(paths::ID_CHANNELS_CHANNELID_) => method_not_allowed(),
                _ if path.matched(paths::ID_CHANNELS_CHANNELID_FUND) => method_not_allowed(),
                _ if path.matched(paths::ID_CHANNELS_CHANNELID_TICKETS) => method_not_allowed(),
                _ if path.matched(paths::ID_CHANNELS_CHANNELID_TICKETS_AGGREGATE) => method_not_allowed(),
                _ if path.matched(paths::ID_CHANNELS_CHANNELID_TICKETS_REDEEM) => method_not_allowed(),
                _ if path.matched(paths::ID_HEALTHYZ_) => method_not_allowed(),
                _ if path.matched(paths::ID_MESSAGES_) => method_not_allowed(),
                _ if path.matched(paths::ID_MESSAGES_POP) => method_not_allowed(),
                _ if path.matched(paths::ID_MESSAGES_POP_ALL) => method_not_allowed(),
                _ if path.matched(paths::ID_MESSAGES_SIZE) => method_not_allowed(),
                _ if path.matched(paths::ID_MESSAGES_WEBSOCKET) => method_not_allowed(),
                _ if path.matched(paths::ID_NODE_ENTRYNODES) => method_not_allowed(),
                _ if path.matched(paths::ID_NODE_INFO) => method_not_allowed(),
                _ if path.matched(paths::ID_NODE_METRICS) => method_not_allowed(),
                _ if path.matched(paths::ID_NODE_PEERS) => method_not_allowed(),
                _ if path.matched(paths::ID_NODE_VERSION) => method_not_allowed(),
                _ if path.matched(paths::ID_PEERS_PEERID_) => method_not_allowed(),
                _ if path.matched(paths::ID_PEERS_PEERID_PING) => method_not_allowed(),
                _ if path.matched(paths::ID_READYZ_) => method_not_allowed(),
                _ if path.matched(paths::ID_SETTINGS_) => method_not_allowed(),
                _ if path.matched(paths::ID_SETTINGS_SETTING) => method_not_allowed(),
                _ if path.matched(paths::ID_STARTEDZ_) => method_not_allowed(),
                _ if path.matched(paths::ID_TICKETS_) => method_not_allowed(),
                _ if path.matched(paths::ID_TICKETS_REDEEM) => method_not_allowed(),
                _ if path.matched(paths::ID_TICKETS_STATISTICS) => method_not_allowed(),
                _ if path.matched(paths::ID_TOKEN) => method_not_allowed(),
                _ if path.matched(paths::ID_TOKENS_) => method_not_allowed(),
                _ if path.matched(paths::ID_TOKENS_ID) => method_not_allowed(),
                _ => Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .expect("Unable to create Not Found response")),
            }
        }
        Box::pin(run(self.api_impl.clone(), req))
    }
}

/// Request parser for `Api`.
pub struct ApiRequestParser;
impl<T> RequestParser<T> for ApiRequestParser {
    fn parse_operation_id(request: &Request<T>) -> Option<&'static str> {
        let path = paths::GLOBAL_REGEX_SET.matches(request.uri().path());
        match *request.method() {
            // AccountGetAddress - GET /account/address
            hyper::Method::GET if path.matched(paths::ID_ACCOUNT_ADDRESS) => Some("AccountGetAddress"),
            // AccountGetAddresses - GET /account/addresses
            hyper::Method::GET if path.matched(paths::ID_ACCOUNT_ADDRESSES) => Some("AccountGetAddresses"),
            // AccountGetBalances - GET /account/balances
            hyper::Method::GET if path.matched(paths::ID_ACCOUNT_BALANCES) => Some("AccountGetBalances"),
            // AccountWithdraw - POST /account/withdraw
            hyper::Method::POST if path.matched(paths::ID_ACCOUNT_WITHDRAW) => Some("AccountWithdraw"),
            // AliasesGetAlias - GET /aliases/{alias}
            hyper::Method::GET if path.matched(paths::ID_ALIASES_ALIAS) => Some("AliasesGetAlias"),
            // AliasesGetAliases - GET /aliases/
            hyper::Method::GET if path.matched(paths::ID_ALIASES_) => Some("AliasesGetAliases"),
            // AliasesRemoveAlias - DELETE /aliases/{alias}
            hyper::Method::DELETE if path.matched(paths::ID_ALIASES_ALIAS) => Some("AliasesRemoveAlias"),
            // AliasesSetAlias - POST /aliases/
            hyper::Method::POST if path.matched(paths::ID_ALIASES_) => Some("AliasesSetAlias"),
            // ChannelsAggregateTickets - POST /channels/{channelid}/tickets/aggregate
            hyper::Method::POST if path.matched(paths::ID_CHANNELS_CHANNELID_TICKETS_AGGREGATE) => {
                Some("ChannelsAggregateTickets")
            }
            // ChannelsCloseChannel - DELETE /channels/{channelid}/
            hyper::Method::DELETE if path.matched(paths::ID_CHANNELS_CHANNELID_) => Some("ChannelsCloseChannel"),
            // ChannelsFundChannel - POST /channels/{channelid}/fund
            hyper::Method::POST if path.matched(paths::ID_CHANNELS_CHANNELID_FUND) => Some("ChannelsFundChannel"),
            // ChannelsGetChannel - GET /channels/{channelid}/
            hyper::Method::GET if path.matched(paths::ID_CHANNELS_CHANNELID_) => Some("ChannelsGetChannel"),
            // ChannelsGetChannels - GET /channels/
            hyper::Method::GET if path.matched(paths::ID_CHANNELS_) => Some("ChannelsGetChannels"),
            // ChannelsGetTickets - GET /channels/{channelid}/tickets
            hyper::Method::GET if path.matched(paths::ID_CHANNELS_CHANNELID_TICKETS) => Some("ChannelsGetTickets"),
            // ChannelsOpenChannel - POST /channels/
            hyper::Method::POST if path.matched(paths::ID_CHANNELS_) => Some("ChannelsOpenChannel"),
            // ChannelsRedeemTickets - POST /channels/{channelid}/tickets/redeem
            hyper::Method::POST if path.matched(paths::ID_CHANNELS_CHANNELID_TICKETS_REDEEM) => {
                Some("ChannelsRedeemTickets")
            }
            // CheckNodeHealthy - GET /healthyz/
            hyper::Method::GET if path.matched(paths::ID_HEALTHYZ_) => Some("CheckNodeHealthy"),
            // CheckNodeReady - GET /readyz/
            hyper::Method::GET if path.matched(paths::ID_READYZ_) => Some("CheckNodeReady"),
            // CheckNodeStarted - GET /startedz/
            hyper::Method::GET if path.matched(paths::ID_STARTEDZ_) => Some("CheckNodeStarted"),
            // MessagesDeleteMessages - DELETE /messages/
            hyper::Method::DELETE if path.matched(paths::ID_MESSAGES_) => Some("MessagesDeleteMessages"),
            // MessagesGetSize - GET /messages/size
            hyper::Method::GET if path.matched(paths::ID_MESSAGES_SIZE) => Some("MessagesGetSize"),
            // MessagesPopAllMessage - POST /messages/pop-all
            hyper::Method::POST if path.matched(paths::ID_MESSAGES_POP_ALL) => Some("MessagesPopAllMessage"),
            // MessagesPopMessage - POST /messages/pop
            hyper::Method::POST if path.matched(paths::ID_MESSAGES_POP) => Some("MessagesPopMessage"),
            // MessagesSendMessage - POST /messages/
            hyper::Method::POST if path.matched(paths::ID_MESSAGES_) => Some("MessagesSendMessage"),
            // MessagesWebsocket - GET /messages/websocket
            hyper::Method::GET if path.matched(paths::ID_MESSAGES_WEBSOCKET) => Some("MessagesWebsocket"),
            // NodeGetEntryNodes - GET /node/entryNodes
            hyper::Method::GET if path.matched(paths::ID_NODE_ENTRYNODES) => Some("NodeGetEntryNodes"),
            // NodeGetInfo - GET /node/info
            hyper::Method::GET if path.matched(paths::ID_NODE_INFO) => Some("NodeGetInfo"),
            // NodeGetMetrics - GET /node/metrics
            hyper::Method::GET if path.matched(paths::ID_NODE_METRICS) => Some("NodeGetMetrics"),
            // NodeGetPeers - GET /node/peers
            hyper::Method::GET if path.matched(paths::ID_NODE_PEERS) => Some("NodeGetPeers"),
            // NodeGetVersion - GET /node/version
            hyper::Method::GET if path.matched(paths::ID_NODE_VERSION) => Some("NodeGetVersion"),
            // PeerInfoGetPeerInfo - GET /peers/{peerid}/
            hyper::Method::GET if path.matched(paths::ID_PEERS_PEERID_) => Some("PeerInfoGetPeerInfo"),
            // PeersPingPeer - POST /peers/{peerid}/ping
            hyper::Method::POST if path.matched(paths::ID_PEERS_PEERID_PING) => Some("PeersPingPeer"),
            // SettingsGetSettings - GET /settings/
            hyper::Method::GET if path.matched(paths::ID_SETTINGS_) => Some("SettingsGetSettings"),
            // SettingsSetSetting - PUT /settings/{setting}
            hyper::Method::PUT if path.matched(paths::ID_SETTINGS_SETTING) => Some("SettingsSetSetting"),
            // TicketsGetStatistics - GET /tickets/statistics
            hyper::Method::GET if path.matched(paths::ID_TICKETS_STATISTICS) => Some("TicketsGetStatistics"),
            // TicketsGetTickets - GET /tickets/
            hyper::Method::GET if path.matched(paths::ID_TICKETS_) => Some("TicketsGetTickets"),
            // TicketsRedeemTickets - POST /tickets/redeem
            hyper::Method::POST if path.matched(paths::ID_TICKETS_REDEEM) => Some("TicketsRedeemTickets"),
            // TokensCreate - POST /tokens/
            hyper::Method::POST if path.matched(paths::ID_TOKENS_) => Some("TokensCreate"),
            // TokensDelete - DELETE /tokens/{id}
            hyper::Method::DELETE if path.matched(paths::ID_TOKENS_ID) => Some("TokensDelete"),
            // TokensGetToken - GET /token
            hyper::Method::GET if path.matched(paths::ID_TOKEN) => Some("TokensGetToken"),
            _ => None,
        }
    }
}
