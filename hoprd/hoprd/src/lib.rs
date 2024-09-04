//! HOPR daemon application providing a higher level interface for creating a HOPRd with or without
//! a dedicated REST API.
//!
//! When the Rest API is enabled, the node serves a Swagger UI to inspect and test
//! the Rest API v3 at: http://localhost:3001/scalar
//!
//! NOTE: Hostname and port can be different, since they depend on the settings `--apiHost` and `--apiPort`.
//!
//! ## Usage
//! See `hoprd --help` for full list.

//! ```shell
//! $ hoprd --help
//! Contains the main entry point of HOPR daemon applicatio
//!
//! Usage: hoprd [OPTIONS]
//!
//! Options:
//!       --network <NETWORK>
//!           ID of the network the node will attempt to connect to [env: HOPRD_NETWORK=]
//!       --identity <IDENTITY>
//!           The path to the identity file [env: HOPRD_IDENTITY=]
//!       --data <DATA>
//!           Specifies the directory to hold all the data [env: HOPRD_DATA=]
//!       --host <HOST>
//!           Host to listen on for P2P connections [env: HOPRD_HOST=]
//!       --announce
//!           Announce the node on chain with a public address [env: HOPRD_ANNOUNCE=]
//!       --api
//!           Expose the API on localhost:3001 [env: HOPRD_API=]
//!       --apiHost <HOST>
//!           Set host IP to which the API server will bind [env: HOPRD_API_HOST=]
//!       --apiPort <PORT>
//!           Set port to which the API server will bind [env: HOPRD_API_PORT=]
//!       --apiToken <TOKEN>
//!           A REST API token and for user authentication [env: HOPRD_API_TOKEN=]
//!       --password <PASSWORD>
//!           A password to encrypt your keys [env: HOPRD_PASSWORD=]
//!       --defaultStrategy <DEFAULT_STRATEGY>
//!           Default channel strategy to use after node starts up [env: HOPRD_DEFAULT_STRATEGY=] [possible values: promiscuous, aggregating, auto_redeeming, auto_funding, multi, passive]
//!       --maxAutoChannels <MAX_AUTO_CHANNELS>
//!           Maximum number of channel a strategy can open. If not specified, square root of number of available peers is used. [env: HOPRD_MAX_AUTO_CHANNELS=]
//!       --disableTicketAutoRedeem
//!           Disables automatic redeeming of winning tickets. [env: HOPRD_DISABLE_AUTO_REDEEEM_TICKETS=]
//!       --disableUnrealizedBalanceCheck
//!           Disables checking of unrealized balance before validating unacknowledged tickets. [env: HOPRD_DISABLE_UNREALIZED_BALANCE_CHECK=]
//!       --provider <PROVIDER>
//!           A custom RPC provider to be used for the node to connect to blockchain [env: HOPRD_PROVIDER=]
//!       --init
//!           initialize a database if it doesn't already exist [env: HOPRD_INIT=]
//!       --forceInit
//!           initialize a database, even if it already exists [env: HOPRD_FORCE_INIT=]
//!       --inbox-capacity <INBOX_CAPACITY>
//!           Set maximum capacity of the HOPRd inbox [env: HOPRD_INBOX_CAPACITY=]
//!       --testAnnounceLocalAddresses
//!           For testing local testnets. Announce local addresses [env: HOPRD_TEST_ANNOUNCE_LOCAL_ADDRESSES=]
//!       --heartbeatInterval <MILLISECONDS>
//!           Interval in milliseconds in which the availability of other nodes get measured [env: HOPRD_HEARTBEAT_INTERVAL=]
//!       --heartbeatThreshold <MILLISECONDS>
//!           Timeframe in milliseconds after which a heartbeat to another peer is performed, if it hasn't been seen since [env: HOPRD_HEARTBEAT_THRESHOLD=]
//!       --heartbeatVariance <MILLISECONDS>
//!           Upper bound for variance applied to heartbeat interval in milliseconds [env: HOPRD_HEARTBEAT_VARIANCE=]
//!       --networkQualityThreshold <THRESHOLD>
//!           Minimum quality of a peer connection to be considered usable [env: HOPRD_NETWORK_QUALITY_THRESHOLD=]
//!       --configurationFilePath <CONFIG_FILE_PATH>
//!           Path to a file containing the entire HOPRd configuration [env: HOPRD_CONFIGURATION_FILE_PATH=]
//!       --safeTransactionServiceProvider <HOPRD_SAFE_TX_SERVICE_PROVIDER>
//!           Base URL for safe transaction service [env: HOPRD_SAFE_TRANSACTION_SERVICE_PROVIDER=]
//!       --safeAddress <HOPRD_SAFE_ADDR>
//!           Address of Safe that safeguards tokens [env: HOPRD_SAFE_ADDRESS=]
//!       --moduleAddress <HOPRD_MODULE_ADDR>
//!           Address of the node mangement module [env: HOPRD_MODULE_ADDRESS=]
//!       --protocolConfig <HOPRD_PROTOCOL_CONFIG_PATH>
//!           Path to the protocol-config.json file [env: HOPRD_PROTOCOL_CONFIG_PATH=]
//!       --dryRun
//!           DEPRECATED [env: HOPRD_DRY_RUN=]
//!       --healthCheck
//!           DEPRECATED
//!       --healthCheckHost <HEALTH_CHECK_HOST>
//!           DEPRECATED
//!       --healthCheckPort <HEALTH_CHECK_PORT>
//!           DEPRECATED
//!   -h, --help
//!           Print help
//!   -V, --version
//!           Print version
//! ```

use hopr_lib::errors::HoprLibError;
use hopr_network_types::prelude::ForeignDataMode;
use hopr_network_types::utils::copy_duplex;

pub mod cli;
pub mod config;
pub mod errors;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_ACTIVE_TARGETS: hopr_metrics::MultiGauge = hopr_metrics::MultiGauge::new(
        "hopr_session_hoprd_target_connections",
        "Number of currently active HOPR session target connections on this Exit node",
        &["type"]
    ).unwrap();
}

#[derive(Debug, Clone)]
pub struct HoprServerIpForwardingReactor;

#[hopr_lib::async_trait]
impl hopr_lib::HoprSessionReactor for HoprServerIpForwardingReactor {
    #[tracing::instrument(level = "debug", skip(self, session))]
    async fn process(&self, mut session: hopr_lib::HoprIncomingSession) -> hopr_lib::errors::Result<()> {
        let session_id = *session.session.id();
        match session.target {
            hopr_lib::SessionTarget::UdpStream(udp_target) => {
                tracing::debug!(
                    session_id = debug(session_id),
                    "binding socket to the UDP server {udp_target}..."
                );

                // In UDP, it is impossible to determine if the target is viable,
                // so we just take the first resolved address.
                let resolved_udp_target = udp_target
                    .clone()
                    .resolve()
                    .await
                    .map_err(|e| HoprLibError::GeneralError(format!("failed to resolve DNS name {udp_target}: {e}")))?
                    .first()
                    .ok_or(HoprLibError::GeneralError(format!(
                        "failed to resolve DNS name {udp_target}"
                    )))?
                    .to_owned();
                tracing::debug!(
                    session_id = debug(session_id),
                    "UDP target {udp_target} resolved to {resolved_udp_target}"
                );

                let udp_bridge = hopr_network_types::udp::ConnectedUdpStream::bind(("0.0.0.0", 0))
                    .await
                    .and_then(|s| s.with_counterparty(resolved_udp_target))
                    .map(|s| s.with_foreign_data_mode(ForeignDataMode::Error))
                    .map_err(|e| {
                        HoprLibError::GeneralError(format!(
                            "could not bridge the incoming session to {udp_target}: {e}"
                        ))
                    })?;

                tracing::debug!(
                    session_id = debug(session_id),
                    "bridging the session to the UDP server {udp_target} ..."
                );
                tokio::task::spawn(async move {
                    #[cfg(all(feature = "prometheus", not(test)))]
                    METRIC_ACTIVE_TARGETS.increment(&["tcp"], 1.0);

                    match copy_duplex(&mut session.session, &mut tokio_util::compat::TokioAsyncReadCompatExt::compat(udp_bridge), hopr_lib::SESSION_USABLE_MTU_SIZE, hopr_lib::SESSION_USABLE_MTU_SIZE).await {
                        Ok(bound_stream_finished) => tracing::info!(
                            session_id = debug(session_id),
                            "server bridged session through UDP {udp_target} ended with {bound_stream_finished:?} bytes transferred in both directions."
                        ),
                        Err(e) => tracing::error!(session_id = debug(session_id),
                            "UDP server stream ({udp_target}) is closed: {e:?}"
                        )
                    }

                    #[cfg(all(feature = "prometheus", not(test)))]
                    METRIC_ACTIVE_TARGETS.decrement(&["tcp"], 1.0);
                });

                Ok(())
            }
            hopr_lib::SessionTarget::TcpStream(tcp_target) => {
                tracing::debug!(
                    session_id = debug(session_id),
                    "creating a connection to the TCP server {tcp_target}..."
                );

                // TCP is able to determine which of the resolved multiple addresses is viable,
                // and therefore we can pass all of them.
                let resolved_tcp_targets =
                    tcp_target.clone().resolve().await.map_err(|e| {
                        HoprLibError::GeneralError(format!("failed to resolve DNS name {tcp_target}: {e}"))
                    })?;
                tracing::debug!(
                    session_id = debug(session_id),
                    "TCP target {tcp_target} resolved to {resolved_tcp_targets:?}"
                );

                // TODO: make TCP connection retry strategy configurable either by the server or the client
                let strategy = tokio_retry::strategy::FixedInterval::from_millis(1500).take(15);

                let tcp_bridge = tokio_retry::Retry::spawn(strategy, || {
                    tokio::net::TcpStream::connect(resolved_tcp_targets.as_slice())
                })
                .await
                .map_err(|e| {
                    HoprLibError::GeneralError(format!("could not bridge the incoming session to {tcp_target}: {e}"))
                })?;

                tcp_bridge.set_nodelay(true).map_err(|e| {
                    HoprLibError::GeneralError(format!(
                        "could not set the TCP_NODELAY option for the bridged session to {tcp_target}: {e}",
                    ))
                })?;

                tracing::debug!(
                    session_id = debug(session_id),
                    "bridging the session to the TCP server {tcp_target} ..."
                );
                tokio::task::spawn(async move {
                    #[cfg(all(feature = "prometheus", not(test)))]
                    METRIC_ACTIVE_TARGETS.increment(&["udp"], 1.0);

                    match copy_duplex(&mut session.session, &mut tokio_util::compat::TokioAsyncReadCompatExt::compat(tcp_bridge), hopr_lib::SESSION_USABLE_MTU_SIZE, hopr_lib::SESSION_USABLE_MTU_SIZE).await {
                        Ok(bound_stream_finished) => tracing::info!(
                            session_id = debug(session_id),
                            "server bridged session through TCP {tcp_target} ended with {bound_stream_finished:?} bytes transferred in both directions."
                        ),
                        Err(e) => tracing::error!(
                            session_id = debug(session_id),
                            "TCP server stream ({tcp_target}) is closed: {e:?}"
                        )
                    }

                    #[cfg(all(feature = "prometheus", not(test)))]
                    METRIC_ACTIVE_TARGETS.decrement(&["udp"], 1.0);
                });

                Ok(())
            }
            hopr_lib::SessionTarget::ExitNode(_) => Err(HoprLibError::GeneralError(
                "server does not support internal session processing".into(),
            )),
        }
    }
}
