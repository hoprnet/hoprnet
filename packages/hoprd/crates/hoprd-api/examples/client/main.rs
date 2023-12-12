#![allow(missing_docs, unused_variables, trivial_casts)]

use clap::{App, Arg};
#[allow(unused_imports)]
use futures::{future, stream, Stream};
#[allow(unused_imports)]
use hoprd_api::{
    models, AccountGetAddressResponse, AccountGetAddressesResponse, AccountGetBalancesResponse,
    AccountWithdrawResponse, AliasesGetAliasResponse, AliasesGetAliasesResponse, AliasesRemoveAliasResponse,
    AliasesSetAliasResponse, Api, ApiNoContext, ChannelsAggregateTicketsResponse, ChannelsCloseChannelResponse,
    ChannelsFundChannelResponse, ChannelsGetChannelResponse, ChannelsGetChannelsResponse, ChannelsGetTicketsResponse,
    ChannelsOpenChannelResponse, ChannelsRedeemTicketsResponse, CheckNodeHealthyResponse, CheckNodeReadyResponse,
    CheckNodeStartedResponse, Client, ContextWrapperExt, MessagesDeleteMessagesResponse, MessagesGetSizeResponse,
    MessagesPopAllMessageResponse, MessagesPopMessageResponse, MessagesSendMessageResponse, MessagesWebsocketResponse,
    NodeGetEntryNodesResponse, NodeGetInfoResponse, NodeGetMetricsResponse, NodeGetPeersResponse,
    NodeGetVersionResponse, PeerInfoGetPeerInfoResponse, PeersPingPeerResponse, SettingsGetSettingsResponse,
    SettingsSetSettingResponse, TicketsGetStatisticsResponse, TicketsGetTicketsResponse, TicketsRedeemTicketsResponse,
    TokensCreateResponse, TokensDeleteResponse, TokensGetTokenResponse,
};

#[allow(unused_imports)]
use log::info;

// swagger::Has may be unused if there are no examples
#[allow(unused_imports)]
use swagger::{AuthData, ContextBuilder, EmptyContext, Has, Push, XSpanIdString};

type ClientContext = swagger::make_context_ty!(ContextBuilder, EmptyContext, Option<AuthData>, XSpanIdString);

// rt may be unused if there are no examples
#[allow(unused_mut)]
fn main() {
    env_logger::init();

    let matches = App::new("client")
        .arg(
            Arg::with_name("operation")
                .help("Sets the operation to run")
                .possible_values(&[
                    "AccountGetAddress",
                    "AccountGetAddresses",
                    "AccountGetBalances",
                    "AccountWithdraw",
                    "AliasesGetAlias",
                    "AliasesGetAliases",
                    "AliasesRemoveAlias",
                    "AliasesSetAlias",
                    "ChannelsAggregateTickets",
                    "ChannelsCloseChannel",
                    "ChannelsFundChannel",
                    "ChannelsGetChannels",
                    "ChannelsGetTickets",
                    "ChannelsOpenChannel",
                    "ChannelsRedeemTickets",
                    "CheckNodeHealthy",
                    "CheckNodeReady",
                    "CheckNodeStarted",
                    "MessagesDeleteMessages",
                    "MessagesGetSize",
                    "MessagesPopAllMessage",
                    "MessagesPopMessage",
                    "MessagesSendMessage",
                    "MessagesWebsocket",
                    "NodeGetEntryNodes",
                    "NodeGetInfo",
                    "NodeGetMetrics",
                    "NodeGetPeers",
                    "NodeGetVersion",
                    "PeerInfoGetPeerInfo",
                    "PeersPingPeer",
                    "SettingsGetSettings",
                    "SettingsSetSetting",
                    "TicketsGetStatistics",
                    "TicketsGetTickets",
                    "TicketsRedeemTickets",
                    "TokensCreate",
                    "TokensDelete",
                    "TokensGetToken",
                ])
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("https")
                .long("https")
                .help("Whether to use HTTPS or not"),
        )
        .arg(
            Arg::with_name("host")
                .long("host")
                .takes_value(true)
                .default_value("localhost")
                .help("Hostname to contact"),
        )
        .arg(
            Arg::with_name("port")
                .long("port")
                .takes_value(true)
                .default_value("8080")
                .help("Port to contact"),
        )
        .get_matches();

    let is_https = matches.is_present("https");
    let base_url = format!(
        "{}://{}:{}",
        if is_https { "https" } else { "http" },
        matches.value_of("host").unwrap(),
        matches.value_of("port").unwrap()
    );

    let context: ClientContext = swagger::make_context!(
        ContextBuilder,
        EmptyContext,
        None as Option<AuthData>,
        XSpanIdString::default()
    );

    let mut client: Box<dyn ApiNoContext<ClientContext>> = if matches.is_present("https") {
        // Using Simple HTTPS
        let client = Box::new(Client::try_new_https(&base_url).expect("Failed to create HTTPS client"));
        Box::new(client.with_context(context))
    } else {
        // Using HTTP
        let client = Box::new(Client::try_new_http(&base_url).expect("Failed to create HTTP client"));
        Box::new(client.with_context(context))
    };

    let mut rt = tokio::runtime::Runtime::new().unwrap();

    match matches.value_of("operation") {
        Some("AccountGetAddress") => {
            let result = rt.block_on(client.account_get_address());
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("AccountGetAddresses") => {
            let result = rt.block_on(client.account_get_addresses());
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("AccountGetBalances") => {
            let result = rt.block_on(client.account_get_balances());
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("AccountWithdraw") => {
            let result = rt.block_on(client.account_withdraw(None));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("AliasesGetAlias") => {
            let result = rt.block_on(client.aliases_get_alias("Alice".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("AliasesGetAliases") => {
            let result = rt.block_on(client.aliases_get_aliases());
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("AliasesRemoveAlias") => {
            let result = rt.block_on(client.aliases_remove_alias("Alice".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("AliasesSetAlias") => {
            let result = rt.block_on(client.aliases_set_alias(None));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("ChannelsAggregateTickets") => {
            let result = rt.block_on(client.channels_aggregate_tickets("channelid_example".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("ChannelsCloseChannel") => {
            let result = rt.block_on(client.channels_close_channel("channelid_example".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("ChannelsFundChannel") => {
            let result = rt.block_on(client.channels_fund_channel("channelid_example".to_string(), None));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        /* Disabled because there's no example.
        Some("ChannelsGetChannel") => {
            let result = rt.block_on(client.channels_get_channel(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        Some("ChannelsGetChannels") => {
            let result =
                rt.block_on(client.channels_get_channels(Some("false".to_string()), Some("false".to_string())));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("ChannelsGetTickets") => {
            let result = rt.block_on(client.channels_get_tickets("channelid_example".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("ChannelsOpenChannel") => {
            let result = rt.block_on(client.channels_open_channel(None));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("ChannelsRedeemTickets") => {
            let result = rt.block_on(client.channels_redeem_tickets("channelid_example".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("CheckNodeHealthy") => {
            let result = rt.block_on(client.check_node_healthy());
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("CheckNodeReady") => {
            let result = rt.block_on(client.check_node_ready());
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("CheckNodeStarted") => {
            let result = rt.block_on(client.check_node_started());
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("MessagesDeleteMessages") => {
            let result = rt.block_on(client.messages_delete_messages(56));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("MessagesGetSize") => {
            let result = rt.block_on(client.messages_get_size(56));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("MessagesPopAllMessage") => {
            let result = rt.block_on(client.messages_pop_all_message(None));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("MessagesPopMessage") => {
            let result = rt.block_on(client.messages_pop_message(None));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("MessagesSendMessage") => {
            let result = rt.block_on(client.messages_send_message(None));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("MessagesWebsocket") => {
            let result = rt.block_on(client.messages_websocket());
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("NodeGetEntryNodes") => {
            let result = rt.block_on(client.node_get_entry_nodes());
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("NodeGetInfo") => {
            let result = rt.block_on(client.node_get_info());
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("NodeGetMetrics") => {
            let result = rt.block_on(client.node_get_metrics());
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("NodeGetPeers") => {
            let result = rt.block_on(client.node_get_peers(Some(0.5)));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("NodeGetVersion") => {
            let result = rt.block_on(client.node_get_version());
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("PeerInfoGetPeerInfo") => {
            let result = rt.block_on(client.peer_info_get_peer_info("peerid_example".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("PeersPingPeer") => {
            let result = rt.block_on(client.peers_ping_peer("peerid_example".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("SettingsGetSettings") => {
            let result = rt.block_on(client.settings_get_settings());
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("SettingsSetSetting") => {
            let result = rt.block_on(client.settings_set_setting("includeRecipient".to_string(), None));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("TicketsGetStatistics") => {
            let result = rt.block_on(client.tickets_get_statistics());
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("TicketsGetTickets") => {
            let result = rt.block_on(client.tickets_get_tickets());
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("TicketsRedeemTickets") => {
            let result = rt.block_on(client.tickets_redeem_tickets());
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("TokensCreate") => {
            let result = rt.block_on(client.tokens_create(None));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("TokensDelete") => {
            let result = rt.block_on(client.tokens_delete("someTOKENid1234".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("TokensGetToken") => {
            let result = rt.block_on(client.tokens_get_token());
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        _ => {
            panic!("Invalid operation provided")
        }
    }
}
