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

pub mod cli;
pub mod config;
pub mod errors;

pub const LISTENING_SESSION_RETRANSMISSION_SERVER_PORT: u16 = 4677;

#[derive(Debug, Clone)]
pub struct HoprServerIpForwardingReactor;

#[hopr_lib::async_trait]
impl hopr_lib::HoprSessionReactor for HoprServerIpForwardingReactor {
    #[tracing::instrument(level = "debug", skip(self, session))]
    async fn process(
        &self,
        session: hopr_lib::HoprSession,
        target: hopr_lib::SessionTarget,
    ) -> hopr_lib::errors::Result<()> {
        match target {
            hopr_lib::SessionTarget::UdpStream(udp_target) => {
                tracing::debug!("Binding socket to the UDP server {udp_target}...");
                let bind_addr = std::net::SocketAddr::new(std::net::Ipv4Addr::new(127, 0, 0, 1).into(), 0);
                let mut udp_bridge =
                    hopr_network_types::udp::ConnectedUdpStream::bind_and_connect(bind_addr, udp_target)
                        .await
                        .map_err(|e| {
                            hopr_lib::errors::HoprLibError::GeneralError(format!(
                                "Could not bridge the incoming session to {udp_target}: {e}"
                            ))
                        })?;

                tracing::debug!("Bridging the session to the UDP server {udp_target} ...");
                tokio::task::spawn(async move {
                    match tokio::io::copy_bidirectional_with_sizes(&mut tokio_util::compat::FuturesAsyncReadCompatExt::compat(session), &mut udp_bridge, hopr_lib::SESSION_USABLE_MTU_SIZE, hopr_lib::SESSION_USABLE_MTU_SIZE).await {
                        Ok(bound_stream_finished) => tracing::info!("Server bridged session through UDP {udp_target} ended with {bound_stream_finished:?} bytes transferred in both directions."),
                        Err(e) => tracing::error!("The UDP server stream ({udp_target}) is closed: {e:?}")
                    }
                });

                Ok(())
            }
            hopr_lib::SessionTarget::TcpStream(tcp_target) => {
                tracing::debug!("Creating a connection to the TCP server {tcp_target}...");
                let mut tcp_bridge = tokio::net::TcpStream::connect(tcp_target).await.map_err(|e| {
                    hopr_lib::errors::HoprLibError::GeneralError(format!(
                        "Could not bridge the incoming session to {tcp_target}: {e}"
                    ))
                })?;

                tcp_bridge.set_nodelay(true).map_err(|e| {
                    hopr_lib::errors::HoprLibError::GeneralError(format!(
                        "Could not set the TCP_NODELAY option for the bridged session to {tcp_target}: {e}",
                    ))
                })?;

                tracing::debug!("Bridging the session to the TCP server {tcp_target} ...");
                tokio::task::spawn(async move {
                    match tokio::io::copy_bidirectional_with_sizes(&mut tokio_util::compat::FuturesAsyncReadCompatExt::compat(session), &mut tcp_bridge, hopr_lib::SESSION_USABLE_MTU_SIZE, hopr_lib::SESSION_USABLE_MTU_SIZE).await {
                        Ok(bound_stream_finished) => tracing::info!("Server bridged session through TCP {tcp_target} ended with {bound_stream_finished:?} bytes transferred in both directions."),
                        Err(e) => tracing::error!("The TCP server stream ({tcp_target}) is closed: {e:?}")
                    }
                });

                Ok(())
            }
            hopr_lib::SessionTarget::ExitNode(_) => Err(HoprLibError::GeneralError(
                "server does not support internal session processing".into(),
            )),
        }
    }
}
