use hopr_lib::errors::HoprLibError;
use hopr_lib::{transfer_session, HoprOffchainKeypair};
use hopr_network_types::prelude::ForeignDataMode;
use hopr_network_types::udp::UdpStreamParallelism;
use hoprd_api::{HOPR_TCP_BUFFER_SIZE, HOPR_UDP_BUFFER_SIZE, HOPR_UDP_QUEUE_SIZE};
use serde_with::serde_as;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::time::Duration;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_ACTIVE_TARGETS: hopr_metrics::MultiGauge = hopr_metrics::MultiGauge::new(
        "hopr_session_hoprd_target_connections",
        "Number of currently active HOPR session target connections on this Exit node",
        &["type"]
    ).unwrap();
}

fn fifteen() -> u32 {
    15
}

fn default_target_retry_delay() -> Duration {
    Duration::from_secs(2)
}

/// Configuration of [`HoprServerIpForwardingReactor`].
#[serde_as]
#[derive(
    Clone, Debug, Eq, PartialEq, smart_default::SmartDefault, serde::Deserialize, serde::Serialize, validator::Validate,
)]
pub struct IpForwardingReactorConfig {
    /// If specified, enforces only the given target addresses (after DNS resolution).
    /// If `None` is specified, allows all targets.
    ///
    /// Defaults to `None`.
    #[serde(default)]
    #[default(None)]
    #[serde_as(as = "Option<HashSet<serde_with::DisplayFromStr>>")]
    pub target_allow_list: Option<HashSet<SocketAddr>>,

    /// Delay between retries in seconds to reach a TCP target.
    ///
    /// Defaults to 2 seconds.
    #[serde(default = "default_target_retry_delay")]
    #[default(Duration::from_secs(2))]
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    pub tcp_target_retry_delay: Duration,

    /// Maximum number of retries to reach a TCP target before giving up.
    ///
    /// Default is 10.
    #[serde(default = "fifteen")]
    #[default(10)]
    #[validate(range(min = 1))]
    pub max_tcp_target_retries: u32,
}

/// Implementation of [`hopr_lib::HoprSessionReactor`] that facilitates
/// bridging of TCP or UDP sockets from the Session Exit node to a destination.
#[derive(Debug, Clone)]
pub struct HoprServerIpForwardingReactor {
    keypair: HoprOffchainKeypair,
    cfg: IpForwardingReactorConfig,
}

impl HoprServerIpForwardingReactor {
    pub fn new(keypair: HoprOffchainKeypair, cfg: IpForwardingReactorConfig) -> Self {
        Self { keypair, cfg }
    }

    fn all_ips_allowed(&self, addrs: &[SocketAddr]) -> bool {
        addrs.iter().all(|addr| {
            self.cfg
                .target_allow_list
                .as_ref()
                .map_or(true, |list| list.contains(addr))
        })
    }
}

#[hopr_lib::async_trait]
impl hopr_lib::HoprSessionReactor for HoprServerIpForwardingReactor {
    #[tracing::instrument(level = "debug", skip(self, session))]
    async fn process(&self, mut session: hopr_lib::HoprIncomingSession) -> hopr_lib::errors::Result<()> {
        let session_id = *session.session.id();
        match session.target {
            hopr_lib::SessionTarget::UdpStream(udp_target) => {
                let udp_target = udp_target
                    .unseal(&self.keypair)
                    .map_err(|e| HoprLibError::GeneralError(format!("cannot unseal target: {e}")))?;

                tracing::debug!(
                    session_id = debug(session_id),
                    "binding socket to the UDP server {udp_target}..."
                );

                // In UDP, it is impossible to determine if the target is viable,
                // so we just take the first resolved address.
                let resolved_udp_target = udp_target
                    .clone()
                    .resolve_tokio()
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

                if !self.all_ips_allowed(&[resolved_udp_target]) {
                    return Err(HoprLibError::GeneralError(format!(
                        "denied target address {resolved_udp_target}"
                    )));
                }

                let mut udp_bridge = hopr_network_types::udp::ConnectedUdpStream::builder()
                    .with_buffer_size(HOPR_UDP_BUFFER_SIZE)
                    .with_counterparty(resolved_udp_target)
                    .with_foreign_data_mode(ForeignDataMode::Error)
                    .with_queue_size(HOPR_UDP_QUEUE_SIZE)
                    .with_receiver_parallelism(UdpStreamParallelism::Auto)
                    .build(("0.0.0.0", 0))
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

                    match transfer_session(&mut session.session, &mut udp_bridge, HOPR_UDP_BUFFER_SIZE).await {
                        Ok((session_to_stream_bytes, stream_to_session_bytes)) => tracing::info!(
                            session_id = debug(session_id),
                            session_to_stream_bytes,
                            stream_to_session_bytes,
                            "server bridged session to UDP {udp_target} ended"
                        ),
                        Err(e) => tracing::error!(
                            session_id = debug(session_id),
                            "UDP server stream ({udp_target}) is closed: {e:?}"
                        ),
                    }

                    #[cfg(all(feature = "prometheus", not(test)))]
                    METRIC_ACTIVE_TARGETS.decrement(&["tcp"], 1.0);
                });

                Ok(())
            }
            hopr_lib::SessionTarget::TcpStream(tcp_target) => {
                let tcp_target = tcp_target
                    .unseal(&self.keypair)
                    .map_err(|e| HoprLibError::GeneralError(format!("cannot unseal target: {e}")))?;

                tracing::debug!(
                    session_id = debug(session_id),
                    "creating a connection to the TCP server {tcp_target}..."
                );

                // TCP is able to determine which of the resolved multiple addresses is viable,
                // and therefore we can pass all of them.
                let resolved_tcp_targets =
                    tcp_target.clone().resolve_tokio().await.map_err(|e| {
                        HoprLibError::GeneralError(format!("failed to resolve DNS name {tcp_target}: {e}"))
                    })?;
                tracing::debug!(
                    session_id = debug(session_id),
                    "TCP target {tcp_target} resolved to {resolved_tcp_targets:?}"
                );

                if !self.all_ips_allowed(&resolved_tcp_targets) {
                    return Err(HoprLibError::GeneralError(format!(
                        "denied target address {resolved_tcp_targets:?}"
                    )));
                }

                let strategy = tokio_retry::strategy::FixedInterval::new(self.cfg.tcp_target_retry_delay)
                    .take(self.cfg.max_tcp_target_retries as usize);

                let mut tcp_bridge = tokio_retry::Retry::spawn(strategy, || {
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

                    match transfer_session(&mut session.session, &mut tcp_bridge, HOPR_TCP_BUFFER_SIZE).await {
                        Ok((session_to_stream_bytes, stream_to_session_bytes)) => tracing::info!(
                            session_id = debug(session_id),
                            session_to_stream_bytes,
                            stream_to_session_bytes,
                            "server bridged session to TCP {tcp_target} ended"
                        ),
                        Err(e) => tracing::error!(
                            session_id = debug(session_id),
                            "TCP server stream ({tcp_target}) is closed: {e:?}"
                        ),
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
