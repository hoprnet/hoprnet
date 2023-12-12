//! Main library entry point for openapi_client implementation.

#![allow(unused_imports)]

use async_trait::async_trait;
use futures::{future, Stream, StreamExt, TryFutureExt, TryStreamExt};
use hyper::server::conn::Http;
use hyper::service::Service;
use log::info;
use std::future::Future;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use swagger::auth::MakeAllowAllAuthenticator;
use swagger::EmptyContext;
use swagger::{Has, XSpanIdString};
use tokio::net::TcpListener;

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
use openssl::ssl::{Ssl, SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};

use hoprd_api::models;

/// Builds an SSL implementation for Simple HTTPS from some hard-coded file names
pub async fn create(addr: &str, https: bool) {
    let addr = addr.parse().expect("Failed to parse bind address");

    let server = Server::new();

    let service = MakeService::new(server);

    let service = MakeAllowAllAuthenticator::new(service, "cosmo");

    #[allow(unused_mut)]
    let mut service = hoprd_api::server::context::MakeAddContext::<_, EmptyContext>::new(service);

    if https {
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "ios"))]
        {
            unimplemented!("SSL is not implemented for the examples on MacOS, Windows or iOS");
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
        {
            let mut ssl =
                SslAcceptor::mozilla_intermediate_v5(SslMethod::tls()).expect("Failed to create SSL Acceptor");

            // Server authentication
            ssl.set_private_key_file("examples/server-key.pem", SslFiletype::PEM)
                .expect("Failed to set private key");
            ssl.set_certificate_chain_file("examples/server-chain.pem")
                .expect("Failed to set certificate chain");
            ssl.check_private_key().expect("Failed to check private key");

            let tls_acceptor = ssl.build();
            let tcp_listener = TcpListener::bind(&addr).await.unwrap();

            loop {
                if let Ok((tcp, _)) = tcp_listener.accept().await {
                    let ssl = Ssl::new(tls_acceptor.context()).unwrap();
                    let addr = tcp.peer_addr().expect("Unable to get remote address");
                    let service = service.call(addr);

                    tokio::spawn(async move {
                        let tls = tokio_openssl::SslStream::new(ssl, tcp).map_err(|_| ())?;
                        let service = service.await.map_err(|_| ())?;

                        Http::new().serve_connection(tls, service).await.map_err(|_| ())
                    });
                }
            }
        }
    } else {
        // Using HTTP
        hyper::server::Server::bind(&addr).serve(service).await.unwrap()
    }
}

#[derive(Copy, Clone)]
pub struct Server<C> {
    marker: PhantomData<C>,
}

impl<C> Server<C> {
    pub fn new() -> Self {
        Server { marker: PhantomData }
    }
}

use hoprd_api::server::MakeService;
use hoprd_api::{
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
use std::error::Error;
use swagger::ApiError;

#[async_trait]
impl<C> Api<C> for Server<C>
where
    C: Has<XSpanIdString> + Send + Sync,
{
    async fn account_get_address(&self, context: &C) -> Result<AccountGetAddressResponse, ApiError> {
        info!("account_get_address() - X-Span-ID: {:?}", context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn account_get_addresses(&self, context: &C) -> Result<AccountGetAddressesResponse, ApiError> {
        info!("account_get_addresses() - X-Span-ID: {:?}", context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn account_get_balances(&self, context: &C) -> Result<AccountGetBalancesResponse, ApiError> {
        info!("account_get_balances() - X-Span-ID: {:?}", context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn account_withdraw(
        &self,
        account_withdraw_request: Option<models::AccountWithdrawRequest>,
        context: &C,
    ) -> Result<AccountWithdrawResponse, ApiError> {
        info!(
            "account_withdraw({:?}) - X-Span-ID: {:?}",
            account_withdraw_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    async fn aliases_get_alias(&self, alias: String, context: &C) -> Result<AliasesGetAliasResponse, ApiError> {
        info!(
            "aliases_get_alias(\"{}\") - X-Span-ID: {:?}",
            alias,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    async fn aliases_get_aliases(&self, context: &C) -> Result<AliasesGetAliasesResponse, ApiError> {
        info!("aliases_get_aliases() - X-Span-ID: {:?}", context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn aliases_remove_alias(&self, alias: String, context: &C) -> Result<AliasesRemoveAliasResponse, ApiError> {
        info!(
            "aliases_remove_alias(\"{}\") - X-Span-ID: {:?}",
            alias,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    async fn aliases_set_alias(
        &self,
        aliases_set_alias_request: Option<models::AliasesSetAliasRequest>,
        context: &C,
    ) -> Result<AliasesSetAliasResponse, ApiError> {
        info!(
            "aliases_set_alias({:?}) - X-Span-ID: {:?}",
            aliases_set_alias_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    async fn channels_aggregate_tickets(
        &self,
        channelid: String,
        context: &C,
    ) -> Result<ChannelsAggregateTicketsResponse, ApiError> {
        info!(
            "channels_aggregate_tickets(\"{}\") - X-Span-ID: {:?}",
            channelid,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    async fn channels_close_channel(
        &self,
        channelid: String,
        context: &C,
    ) -> Result<ChannelsCloseChannelResponse, ApiError> {
        info!(
            "channels_close_channel(\"{}\") - X-Span-ID: {:?}",
            channelid,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    async fn channels_fund_channel(
        &self,
        channelid: String,
        channels_fund_channel_request: Option<models::ChannelsFundChannelRequest>,
        context: &C,
    ) -> Result<ChannelsFundChannelResponse, ApiError> {
        info!(
            "channels_fund_channel(\"{}\", {:?}) - X-Span-ID: {:?}",
            channelid,
            channels_fund_channel_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    async fn channels_get_channel(
        &self,
        channelid: serde_json::Value,
        context: &C,
    ) -> Result<ChannelsGetChannelResponse, ApiError> {
        info!(
            "channels_get_channel({:?}) - X-Span-ID: {:?}",
            channelid,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    async fn channels_get_channels(
        &self,
        including_closed: Option<String>,
        full_topology: Option<String>,
        context: &C,
    ) -> Result<ChannelsGetChannelsResponse, ApiError> {
        info!(
            "channels_get_channels({:?}, {:?}) - X-Span-ID: {:?}",
            including_closed,
            full_topology,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    async fn channels_get_tickets(
        &self,
        channelid: String,
        context: &C,
    ) -> Result<ChannelsGetTicketsResponse, ApiError> {
        info!(
            "channels_get_tickets(\"{}\") - X-Span-ID: {:?}",
            channelid,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    async fn channels_open_channel(
        &self,
        channels_open_channel_request: Option<models::ChannelsOpenChannelRequest>,
        context: &C,
    ) -> Result<ChannelsOpenChannelResponse, ApiError> {
        info!(
            "channels_open_channel({:?}) - X-Span-ID: {:?}",
            channels_open_channel_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    async fn channels_redeem_tickets(
        &self,
        channelid: String,
        context: &C,
    ) -> Result<ChannelsRedeemTicketsResponse, ApiError> {
        info!(
            "channels_redeem_tickets(\"{}\") - X-Span-ID: {:?}",
            channelid,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    async fn check_node_healthy(&self, context: &C) -> Result<CheckNodeHealthyResponse, ApiError> {
        info!("check_node_healthy() - X-Span-ID: {:?}", context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn check_node_ready(&self, context: &C) -> Result<CheckNodeReadyResponse, ApiError> {
        info!("check_node_ready() - X-Span-ID: {:?}", context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn check_node_started(&self, context: &C) -> Result<CheckNodeStartedResponse, ApiError> {
        info!("check_node_started() - X-Span-ID: {:?}", context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn messages_delete_messages(
        &self,
        tag: i32,
        context: &C,
    ) -> Result<MessagesDeleteMessagesResponse, ApiError> {
        info!(
            "messages_delete_messages({}) - X-Span-ID: {:?}",
            tag,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    async fn messages_get_size(&self, tag: i32, context: &C) -> Result<MessagesGetSizeResponse, ApiError> {
        info!("messages_get_size({}) - X-Span-ID: {:?}", tag, context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn messages_pop_all_message(
        &self,
        messages_pop_all_message_request: Option<models::MessagesPopAllMessageRequest>,
        context: &C,
    ) -> Result<MessagesPopAllMessageResponse, ApiError> {
        info!(
            "messages_pop_all_message({:?}) - X-Span-ID: {:?}",
            messages_pop_all_message_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    async fn messages_pop_message(
        &self,
        messages_pop_all_message_request: Option<models::MessagesPopAllMessageRequest>,
        context: &C,
    ) -> Result<MessagesPopMessageResponse, ApiError> {
        info!(
            "messages_pop_message({:?}) - X-Span-ID: {:?}",
            messages_pop_all_message_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    async fn messages_send_message(
        &self,
        messages_send_message_request: Option<models::MessagesSendMessageRequest>,
        context: &C,
    ) -> Result<MessagesSendMessageResponse, ApiError> {
        info!(
            "messages_send_message({:?}) - X-Span-ID: {:?}",
            messages_send_message_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    async fn messages_websocket(&self, context: &C) -> Result<MessagesWebsocketResponse, ApiError> {
        info!("messages_websocket() - X-Span-ID: {:?}", context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn node_get_entry_nodes(&self, context: &C) -> Result<NodeGetEntryNodesResponse, ApiError> {
        info!("node_get_entry_nodes() - X-Span-ID: {:?}", context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn node_get_info(&self, context: &C) -> Result<NodeGetInfoResponse, ApiError> {
        info!("node_get_info() - X-Span-ID: {:?}", context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn node_get_metrics(&self, context: &C) -> Result<NodeGetMetricsResponse, ApiError> {
        info!("node_get_metrics() - X-Span-ID: {:?}", context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn node_get_peers(&self, quality: Option<f64>, context: &C) -> Result<NodeGetPeersResponse, ApiError> {
        info!(
            "node_get_peers({:?}) - X-Span-ID: {:?}",
            quality,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    async fn node_get_version(&self, context: &C) -> Result<NodeGetVersionResponse, ApiError> {
        info!("node_get_version() - X-Span-ID: {:?}", context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn peer_info_get_peer_info(
        &self,
        peerid: String,
        context: &C,
    ) -> Result<PeerInfoGetPeerInfoResponse, ApiError> {
        info!(
            "peer_info_get_peer_info(\"{}\") - X-Span-ID: {:?}",
            peerid,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    async fn peers_ping_peer(&self, peerid: String, context: &C) -> Result<PeersPingPeerResponse, ApiError> {
        info!(
            "peers_ping_peer(\"{}\") - X-Span-ID: {:?}",
            peerid,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    async fn settings_get_settings(&self, context: &C) -> Result<SettingsGetSettingsResponse, ApiError> {
        info!("settings_get_settings() - X-Span-ID: {:?}", context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn settings_set_setting(
        &self,
        setting: String,
        settings_set_setting_request: Option<models::SettingsSetSettingRequest>,
        context: &C,
    ) -> Result<SettingsSetSettingResponse, ApiError> {
        info!(
            "settings_set_setting(\"{}\", {:?}) - X-Span-ID: {:?}",
            setting,
            settings_set_setting_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    async fn tickets_get_statistics(&self, context: &C) -> Result<TicketsGetStatisticsResponse, ApiError> {
        info!("tickets_get_statistics() - X-Span-ID: {:?}", context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn tickets_get_tickets(&self, context: &C) -> Result<TicketsGetTicketsResponse, ApiError> {
        info!("tickets_get_tickets() - X-Span-ID: {:?}", context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn tickets_redeem_tickets(&self, context: &C) -> Result<TicketsRedeemTicketsResponse, ApiError> {
        info!("tickets_redeem_tickets() - X-Span-ID: {:?}", context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn tokens_create(
        &self,
        tokens_create_request: Option<models::TokensCreateRequest>,
        context: &C,
    ) -> Result<TokensCreateResponse, ApiError> {
        info!(
            "tokens_create({:?}) - X-Span-ID: {:?}",
            tokens_create_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    async fn tokens_delete(&self, id: String, context: &C) -> Result<TokensDeleteResponse, ApiError> {
        info!("tokens_delete(\"{}\") - X-Span-ID: {:?}", id, context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    async fn tokens_get_token(&self, context: &C) -> Result<TokensGetTokenResponse, ApiError> {
        info!("tokens_get_token() - X-Span-ID: {:?}", context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }
}
