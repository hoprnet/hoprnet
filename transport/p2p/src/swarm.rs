use std::{net::Ipv4Addr, num::NonZeroU8};
use tracing::{error, info};

// #[cfg(any(feature = "runtime-async-std", test))]
// use async_std::task::spawn;

// #[cfg(all(feature = "runtime-tokio", not(test)))]
// use tokio::task::spawn;

use crate::{
    constants,
    errors::Result,
    multiaddrs::{Multiaddr, Protocol},
    HoprNetworkBehavior,
};
use core_protocol::config::ProtocolConfig;

use std::net::ToSocketAddrs;

/// Replaces the IPv4 and IPv6 from the network layer with a unspecified interface in any multiaddress.
fn replace_transport_with_unspecified(ma: &Multiaddr) -> Result<Multiaddr> {
    let mut out = Multiaddr::empty();

    for proto in ma.iter() {
        match proto {
            Protocol::Ip4(_) => out.push(std::net::IpAddr::V4(Ipv4Addr::UNSPECIFIED).into()),
            Protocol::Ip6(_) => out.push(std::net::IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED).into()),
            _ => out.push(proto),
        }
    }

    Ok(out)
}

/// Resolves the DNS parts of a multiaddress and replaces it with the resolved IP address.
fn resolve_dns_if_any(ma: &Multiaddr) -> Result<Multiaddr> {
    let mut out = Multiaddr::empty();

    for proto in ma.iter() {
        match proto {
            Protocol::Dns4(domain) => {
                let ip = format!("{domain}:443") // dummy port, irrevelant at this point
                    .to_socket_addrs()
                    .map_err(|e| crate::errors::P2PError::Logic(e.to_string()))?
                    .filter(|sa| sa.is_ipv4())
                    .collect::<Vec<_>>()
                    .first()
                    .ok_or(crate::errors::P2PError::Logic(format!(
                        "Failed to resolve {domain} to an IPv4 address. Does the DNS entry has an A record?"
                    )))?
                    .ip();

                out.push(ip.into())
            }
            Protocol::Dns6(domain) => {
                let ip = format!("{domain}:443") // dummy port, irrevelant at this point
                    .to_socket_addrs()
                    .map_err(|e| crate::errors::P2PError::Logic(e.to_string()))?
                    .filter(|sa| sa.is_ipv6())
                    .collect::<Vec<_>>()
                    .first()
                    .ok_or(crate::errors::P2PError::Logic(format!(
                        "Failed to resolve {domain} to an IPv6 address. Does the DNS entry has an AAAA record?"
                    )))?
                    .ip();

                out.push(ip.into())
            }
            _ => out.push(proto),
        }
    }

    Ok(out)
}

/// Build objects comprising the p2p network.
///
/// Returns a built [libp2p::Swarm] object implementing the HoprNetworkBehavior functionality.
async fn build_p2p_network(
    me: libp2p::identity::Keypair,
    protocol_cfg: ProtocolConfig,
) -> Result<libp2p::Swarm<HoprNetworkBehavior>> {
    let tcp_upgrade = libp2p::core::upgrade::SelectUpgrade::new(
        libp2p::yamux::Config::default(),
        libp2p_mplex::MplexConfig::new()
            .set_max_num_streams(1024)
            .set_max_buffer_size(32)
            .set_split_send_size(8 * 1024)
            .set_max_buffer_behaviour(libp2p_mplex::MaxBufferBehaviour::Block)
            .clone(),
    );

    #[cfg(feature = "runtime-async-std")]
    let swarm = libp2p::SwarmBuilder::with_existing_identity(me)
        .with_async_std()
        .with_tcp(Default::default(), libp2p::noise::Config::new, move || tcp_upgrade)
        .map_err(|e| crate::errors::P2PError::Libp2p(e.to_string()))?
        .with_quic()
        .with_dns()
        .await;

    #[cfg(feature = "runtime-tokio")]
    let swarm = libp2p::SwarmBuilder::with_existing_identity(me)
        .with_tokio()
        .with_tcp(Default::default(), libp2p::noise::Config::new, || tcp_upgrade)
        .map_err(|e| crate::errors::P2PError::Libp2p(e.to_string()))?
        .with_quic()
        .with_dns();

    Ok(swarm
        .map_err(|e| crate::errors::P2PError::Libp2p(e.to_string()))?
        .with_behaviour(|_key| {
            HoprNetworkBehavior::new(
                protocol_cfg.msg,
                protocol_cfg.ack,
                protocol_cfg.heartbeat,
                protocol_cfg.ticket_aggregation,
            )
        })
        .map_err(|e| crate::errors::P2PError::Libp2p(e.to_string()))?
        .with_swarm_config(|cfg| {
            cfg.with_dial_concurrency_factor(
                NonZeroU8::new(
                    std::env::var("HOPR_INTERNAL_LIBP2P_MAX_CONCURRENTLY_DIALED_PEER_COUNT")
                        .map(|v| v.trim().parse::<u8>().unwrap_or(u8::MAX))
                        .unwrap_or(constants::HOPR_SWARM_CONCURRENTLY_DIALED_PEER_COUNT),
                )
                .expect("concurrently dialed peer count must be > 0"),
            )
            .with_max_negotiating_inbound_streams(
                std::env::var("HOPR_INTERNAL_LIBP2P_MAX_NEGOTIATING_INBOUND_STREAM_COUNT")
                    .map(|v| v.trim().parse::<usize>().unwrap_or(128))
                    .unwrap_or(constants::HOPR_SWARM_CONCURRENTLY_NEGOTIATING_INBOUND_PEER_COUNT),
            )
            .with_idle_connection_timeout(constants::HOPR_SWARM_IDLE_CONNECTION_TIMEOUT)
        })
        .build())
}

pub struct HoprSwarm {
    pub(crate) swarm: libp2p::Swarm<HoprNetworkBehavior>,
}

impl HoprSwarm {
    pub async fn new(
        identity: libp2p::identity::Keypair,
        my_multiaddresses: Vec<Multiaddr>,
        protocol_cfg: ProtocolConfig,
    ) -> Self {
        let mut swarm = build_p2p_network(identity, protocol_cfg)
            .await
            .expect("swarm must be constructible");

        for multiaddress in my_multiaddresses.iter() {
            match resolve_dns_if_any(multiaddress) {
                Ok(ma) => {
                    if let Err(e) = swarm.listen_on(ma.clone()) {
                        error!("Failed to listen_on using '{multiaddress}': {e}");

                        match replace_transport_with_unspecified(&ma) {
                            Ok(ma) => {
                                if let Err(e) = swarm.listen_on(ma.clone()) {
                                    error!("Failed to listen_on also using the unspecified multiaddress '{ma}': {e}",);
                                } else {
                                    info!("Successfully started listening on {ma} (from {multiaddress})");
                                    swarm.add_external_address(multiaddress.clone());
                                }
                            }
                            Err(e) => {
                                error!("Failed to transform the multiaddress '{ma}' to unspecified: {e}")
                            }
                        }
                    } else {
                        info!("Successfully started listening on {ma} (from {multiaddress})");
                        swarm.add_external_address(multiaddress.clone());
                    }
                }
                Err(_) => error!("Failed to transform the multiaddress '{multiaddress}' - skipping"),
            }
        }

        // NOTE: This would be a valid check but is not immediate
        // assert!(
        //     swarm.listeners().count() > 0,
        //     "The node failed to listen on at least one of the specified interfaces"
        // );

        Self { swarm }
    }
}

impl From<HoprSwarm> for libp2p::Swarm<HoprNetworkBehavior> {
    fn from(value: HoprSwarm) -> Self {
        value.swarm
    }
}
