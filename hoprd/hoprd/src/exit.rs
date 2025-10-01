use std::{net::SocketAddr, num::NonZeroUsize};

use hopr_lib::{
    HoprOffchainKeypair, ServiceId,
    errors::HoprLibError,
    prelude::{ConnectedUdpStream, ForeignDataMode, UdpStreamParallelism},
    transfer_session,
};
use hoprd_api::{HOPR_TCP_BUFFER_SIZE, HOPR_UDP_BUFFER_SIZE, HOPR_UDP_QUEUE_SIZE};

use crate::config::SessionIpForwardingConfig;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_ACTIVE_TARGETS: hopr_metrics::MultiGauge = hopr_metrics::MultiGauge::new(
        "hopr_session_hoprd_target_connections",
        "Number of currently active HOPR session target connections on this Exit node",
        &["type"]
    ).unwrap();
}

/// Implementation of [`hopr_lib::HoprSessionReactor`] that facilitates
/// bridging of TCP or UDP sockets from the Session Exit node to a destination.
#[derive(Debug, Clone)]
pub struct HoprServerIpForwardingReactor {
    keypair: HoprOffchainKeypair,
    cfg: SessionIpForwardingConfig,
}

impl HoprServerIpForwardingReactor {
    pub fn new(keypair: HoprOffchainKeypair, cfg: SessionIpForwardingConfig) -> Self {
        Self { keypair, cfg }
    }

    fn all_ips_allowed(&self, addrs: &[SocketAddr]) -> bool {
        if self.cfg.use_target_allow_list {
            for addr in addrs {
                if !self.cfg.target_allow_list.contains(addr) {
                    tracing::error!(%addr, "address not allowed by the target allow list, denying the target");
                    return false;
                }
                tracing::debug!(%addr, "address allowed by the target allow list, accepting the target");
            }
        }
        true
    }
}

pub const SERVICE_ID_LOOPBACK: ServiceId = 0;

#[hopr_lib::async_trait]
impl hopr_lib::traits::session::HoprSessionServer for HoprServerIpForwardingReactor {
    #[tracing::instrument(level = "debug", skip(self, session))]
    async fn process(
        &self,
        mut session: hopr_lib::exports::transport::IncomingSession,
    ) -> hopr_lib::errors::Result<()> {
        let session_id = *session.session.id();
        match session.target {
            hopr_lib::SessionTarget::UdpStream(udp_target) => {
                let kp = self.keypair.clone();
                let udp_target = hopr_lib::utils::parallelize::cpu::spawn_blocking(move || udp_target.unseal(&kp))
                    .await
                    .map_err(|e| HoprLibError::GeneralError(format!("cannot unseal target: {e}")))?;

                tracing::debug!(
                    session_id = ?session_id,
                    %udp_target,
                    "binding socket to the UDP server"
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
                    ?session_id,
                    %udp_target,
                    resolution = ?resolved_udp_target,
                    "UDP target resolved"
                );

                if !self.all_ips_allowed(&[resolved_udp_target]) {
                    return Err(HoprLibError::GeneralError(format!(
                        "denied target address {resolved_udp_target}"
                    )));
                }

                let mut udp_bridge = ConnectedUdpStream::builder()
                    .with_buffer_size(HOPR_UDP_BUFFER_SIZE)
                    .with_counterparty(resolved_udp_target)
                    .with_foreign_data_mode(ForeignDataMode::Error)
                    .with_queue_size(HOPR_UDP_QUEUE_SIZE)
                    .with_receiver_parallelism(
                        std::env::var("HOPRD_SESSION_EXIT_UDP_RX_PARALLELISM")
                            .ok()
                            .and_then(|s| s.parse::<NonZeroUsize>().ok())
                            .map(UdpStreamParallelism::Specific)
                            .unwrap_or(UdpStreamParallelism::Auto),
                    )
                    .build(("0.0.0.0", 0))
                    .map_err(|e| {
                        HoprLibError::GeneralError(format!(
                            "could not bridge the incoming session to {udp_target}: {e}"
                        ))
                    })?;

                tracing::debug!(
                    ?session_id,
                    %udp_target,
                    "bridging the session to the UDP server"
                );

                tokio::task::spawn(async move {
                    #[cfg(all(feature = "prometheus", not(test)))]
                    let _g = hopr_metrics::MultiGaugeGuard::new(&*METRIC_ACTIVE_TARGETS, &["udp"], 1.0);

                    // The Session forwards the termination to the udp_bridge, terminating
                    // the UDP socket.
                    match transfer_session(&mut session.session, &mut udp_bridge, HOPR_UDP_BUFFER_SIZE, None).await {
                        Ok((session_to_stream_bytes, stream_to_session_bytes)) => tracing::info!(
                            ?session_id,
                            session_to_stream_bytes,
                            stream_to_session_bytes,
                            %udp_target,
                            "server bridged session to UDP ended"
                        ),
                        Err(e) => tracing::error!(
                            ?session_id,
                            %udp_target,
                            error = %e,
                            "UDP server stream is closed"
                        ),
                    }
                });

                Ok(())
            }
            hopr_lib::SessionTarget::TcpStream(tcp_target) => {
                let kp = self.keypair.clone();
                let tcp_target = hopr_lib::utils::parallelize::cpu::spawn_blocking(move || tcp_target.unseal(&kp))
                    .await
                    .map_err(|e| HoprLibError::GeneralError(format!("cannot unseal target: {e}")))?;

                tracing::debug!(?session_id, %tcp_target, "creating a connection to the TCP server");

                // TCP is able to determine which of the resolved multiple addresses is viable,
                // and therefore we can pass all of them.
                let resolved_tcp_targets =
                    tcp_target.clone().resolve_tokio().await.map_err(|e| {
                        HoprLibError::GeneralError(format!("failed to resolve DNS name {tcp_target}: {e}"))
                    })?;
                tracing::debug!(
                    ?session_id,
                    %tcp_target,
                    resolution = ?resolved_tcp_targets,
                    "TCP target resolved"
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
                    ?session_id,
                    %tcp_target,
                    "bridging the session to the TCP server"
                );

                tokio::task::spawn(async move {
                    #[cfg(all(feature = "prometheus", not(test)))]
                    let _g = hopr_metrics::MultiGaugeGuard::new(&*METRIC_ACTIVE_TARGETS, &["tcp"], 1.0);

                    match transfer_session(&mut session.session, &mut tcp_bridge, HOPR_TCP_BUFFER_SIZE, None).await {
                        Ok((session_to_stream_bytes, stream_to_session_bytes)) => tracing::info!(
                            ?session_id,
                            session_to_stream_bytes,
                            stream_to_session_bytes,
                            %tcp_target,
                            "server bridged session to TCP ended"
                        ),
                        Err(error) => tracing::error!(
                            ?session_id,
                            %tcp_target,
                            %error,
                            "TCP server stream is closed"
                        ),
                    }
                });

                Ok(())
            }
            hopr_lib::SessionTarget::ExitNode(SERVICE_ID_LOOPBACK) => {
                tracing::debug!(?session_id, "bridging the session to the loopback service");
                let (mut reader, mut writer) = tokio::io::split(session.session);

                #[cfg(all(feature = "prometheus", not(test)))]
                let _g = hopr_metrics::MultiGaugeGuard::new(&*METRIC_ACTIVE_TARGETS, &["udp"], 1.0);

                // Uses 4 kB buffer for copying
                match tokio::io::copy(&mut reader, &mut writer).await {
                    Ok(copied) => tracing::info!(?session_id, copied, "server loopback session service ended"),
                    Err(error) => tracing::error!(
                        ?session_id,
                        %error,
                        "server loopback session service ended with an error"
                    ),
                }

                Ok(())
            }
            hopr_lib::SessionTarget::ExitNode(_) => Err(HoprLibError::GeneralError(
                "server does not support internal session processing".into(),
            )),
        }
    }
}
