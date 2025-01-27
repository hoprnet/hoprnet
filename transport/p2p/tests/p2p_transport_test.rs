use std::{
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
    str::FromStr,
};

use anyhow::Context;
use libp2p::{identity, Multiaddr, PeerId};

use hopr_crypto_types::{keypairs::Keypair, prelude::OffchainKeypair};
use hopr_internal_types::protocol::Acknowledgement;
use hopr_transport_network::{network::NetworkTriggeredEvent, ping::PingQueryReplier};
use hopr_transport_p2p::{
    swarm::{
        HoprSwarmWithProcessors, TicketAggregationEvent, TicketAggregationRequestType, TicketAggregationResponseType,
    },
    HoprSwarm,
};
use hopr_transport_protocol::{
    config::ProtocolConfig,
    ticket_aggregation::processor::{TicketAggregationActions, TicketAggregationToProcess},
    PeerDiscovery,
};

pub fn random_free_local_ipv4_port() -> Option<u16> {
    let socket = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
    TcpListener::bind(socket)
        .and_then(|listener| listener.local_addr())
        .map(|addr| addr.port())
        .ok()
}

struct Interface {
    me: PeerId,
    address: Multiaddr,
    update_from_network: futures::channel::mpsc::Sender<NetworkTriggeredEvent>,
    update_from_announcements: futures::channel::mpsc::UnboundedSender<PeerDiscovery>,
    send_heartbeat: futures::channel::mpsc::UnboundedSender<(PeerId, PingQueryReplier)>,
    send_ticket_aggregation: futures::channel::mpsc::UnboundedSender<TicketAggregationEvent>,
    // ---
    send_msg: futures::channel::mpsc::UnboundedSender<(PeerId, Box<[u8]>)>,
    recv_msg: futures::channel::mpsc::UnboundedReceiver<(PeerId, Box<[u8]>)>,
    send_ack: futures::channel::mpsc::UnboundedSender<(PeerId, Acknowledgement)>,
    recv_ack: futures::channel::mpsc::UnboundedReceiver<(PeerId, Acknowledgement)>,
}
enum Announcement {
    TCP,
    QUIC,
}

type TestSwarm = HoprSwarmWithProcessors<
    futures::channel::mpsc::UnboundedReceiver<(PeerId, Box<[u8]>)>,
    futures::channel::mpsc::UnboundedSender<(PeerId, Box<[u8]>)>,
    futures::channel::mpsc::UnboundedReceiver<(PeerId, Acknowledgement)>,
    futures::channel::mpsc::UnboundedSender<(PeerId, Acknowledgement)>,
>;

async fn build_p2p_swarm(announcement: Announcement) -> anyhow::Result<(Interface, TestSwarm)> {
    let random_port = random_free_local_ipv4_port().context("could not find a free port")?;
    let random_key = OffchainKeypair::random();
    let identity = hopr_transport_identity::Keypair::generate_ed25519();
    let peer_id: PeerId = identity.public().into();

    let (network_events_tx, network_events_rx) = futures::channel::mpsc::channel::<NetworkTriggeredEvent>(100);
    let (transport_updates_tx, transport_updates_rx) = futures::channel::mpsc::unbounded::<PeerDiscovery>();
    let (heartbeat_requests_tx, heartbeat_requests_rx) =
        futures::channel::mpsc::unbounded::<(PeerId, PingQueryReplier)>();
    let (ticket_aggregation_req_tx, ticket_aggregation_req_rx) =
        futures::channel::mpsc::unbounded::<TicketAggregationEvent>();

    let multiaddress = Multiaddr::from_str(format!("/ip4/127.0.0.1/tcp/{random_port}").as_str())
        .context("failed to create a valid multiaddress")?;

    let swarm = HoprSwarm::new(
        identity,
        network_events_rx,
        transport_updates_rx,
        heartbeat_requests_rx,
        ticket_aggregation_req_rx,
        vec![multiaddress.clone()],
        ProtocolConfig::default(),
    )
    .await;

    let (msg_send_tx, msg_send_rx) = futures::channel::mpsc::unbounded::<(PeerId, Box<[u8]>)>();
    let (msg_recv_tx, msg_recv_rx) = futures::channel::mpsc::unbounded::<(PeerId, Box<[u8]>)>();
    let (ack_send_tx, ack_send_rx) = futures::channel::mpsc::unbounded::<(PeerId, Acknowledgement)>();
    let (ack_recv_tx, ack_recv_rx) = futures::channel::mpsc::unbounded::<(PeerId, Acknowledgement)>();

    let (taa_tx, taa_rx) = futures::channel::mpsc::channel::<
        TicketAggregationToProcess<TicketAggregationResponseType, TicketAggregationRequestType>,
    >(100);
    let _taa =
        TicketAggregationActions::<TicketAggregationResponseType, TicketAggregationRequestType> { queue: taa_tx };

    let swarm = swarm.with_processors(ack_send_rx, ack_recv_tx, msg_send_rx, msg_recv_tx, _taa);

    let api = Interface {
        me: peer_id,
        address: multiaddress,
        update_from_network: network_events_tx,
        update_from_announcements: transport_updates_tx,
        send_heartbeat: heartbeat_requests_tx,
        send_ticket_aggregation: ticket_aggregation_req_tx,
        send_msg: msg_send_tx,
        recv_msg: msg_recv_rx,
        send_ack: ack_send_tx,
        recv_ack: ack_recv_rx,
    };

    Ok((api, swarm))
}

#[cfg_attr(feature = "runtime-async-std", test_log::test(async_std::test))]
#[cfg_attr(
    all(feature = "runtime-tokio", not(feature = "runtime-async-std")),
    test_log::test(tokio::test)
)]
async fn p2p_only_communication_tcp() -> anyhow::Result<()> {
    let (api1, swarm1) = build_p2p_swarm(Announcement::TCP).await?;
    let (api2, swarm2) = build_p2p_swarm(Announcement::TCP).await?;

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", test_log::test(async_std::test))]
#[cfg_attr(
    all(feature = "runtime-tokio", not(feature = "runtime-async-std")),
    test_log::test(tokio::test)
)]
async fn p2p_only_communication_quic() -> anyhow::Result<()> {
    let (api1, swarm1) = build_p2p_swarm(Announcement::QUIC).await?;
    let (api2, swarm2) = build_p2p_swarm(Announcement::QUIC).await?;

    Ok(())
}
