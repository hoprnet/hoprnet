use std::{
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
    str::FromStr,
};

use anyhow::Context;
use futures::{
    channel::mpsc::{Receiver, Sender},
    SinkExt, StreamExt,
};
use lazy_static::lazy_static;
use libp2p::{Multiaddr, PeerId};

use hopr_crypto_packet::chain::ChainPacketComponents;
use hopr_crypto_types::{keypairs::Keypair, prelude::OffchainKeypair};
use hopr_internal_types::protocol::Acknowledgement;
use hopr_platform::time::native::current_time;
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

pub(crate) struct Interface {
    pub me: PeerId,
    pub address: Multiaddr,
    #[allow(dead_code)]
    pub update_from_network: futures::channel::mpsc::Sender<NetworkTriggeredEvent>,
    pub update_from_announcements: futures::channel::mpsc::UnboundedSender<PeerDiscovery>,
    #[allow(dead_code)]
    pub send_heartbeat: futures::channel::mpsc::UnboundedSender<(PeerId, PingQueryReplier)>,
    #[allow(dead_code)]
    pub send_ticket_aggregation: futures::channel::mpsc::UnboundedSender<TicketAggregationEvent>,
    // ---
    pub send_msg: Sender<(PeerId, Box<[u8]>)>,
    pub recv_msg: Receiver<(PeerId, Box<[u8]>)>,
    #[allow(dead_code)]
    pub send_ack: Sender<(PeerId, Acknowledgement)>,
    #[allow(dead_code)]
    pub recv_ack: Receiver<(PeerId, Acknowledgement)>,
}
pub(crate) enum Announcement {
    QUIC,
}

pub(crate) type TestSwarm = HoprSwarmWithProcessors;

async fn build_p2p_swarm(announcement: Announcement) -> anyhow::Result<(Interface, TestSwarm)> {
    let random_port = random_free_local_ipv4_port().context("could not find a free port")?;
    let random_keypair = OffchainKeypair::random();
    let identity: libp2p::identity::Keypair = (&random_keypair).into();
    let peer_id: PeerId = identity.public().into();

    let (network_events_tx, network_events_rx) = futures::channel::mpsc::channel::<NetworkTriggeredEvent>(100);
    let (transport_updates_tx, transport_updates_rx) = futures::channel::mpsc::unbounded::<PeerDiscovery>();
    let (heartbeat_requests_tx, heartbeat_requests_rx) =
        futures::channel::mpsc::unbounded::<(PeerId, PingQueryReplier)>();
    let (ticket_aggregation_req_tx, ticket_aggregation_req_rx) =
        futures::channel::mpsc::unbounded::<TicketAggregationEvent>();

    let multiaddress = match announcement {
        Announcement::QUIC => format!("/ip4/127.0.0.1/udp/{random_port}/quic-v1"),
    };
    let multiaddress = Multiaddr::from_str(&multiaddress).context("failed to create a valid multiaddress")?;

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

    let msg_proto_control = swarm.build_protocol_control(hopr_transport_protocol::msg::CURRENT_HOPR_MSG_PROTOCOL);
    let msg_codec = hopr_transport_protocol::msg::MsgCodec;
    let (wire_msg_tx, wire_msg_rx) =
        hopr_transport_protocol::stream::process_stream_protocol(msg_codec, msg_proto_control).await?;

    let ack_proto_control = swarm.build_protocol_control(hopr_transport_protocol::ack::CURRENT_HOPR_ACK_PROTOCOL);
    let ack_codec = hopr_transport_protocol::ack::AckCodec::new();
    let (wire_ack_tx, wire_ack_rx) =
        hopr_transport_protocol::stream::process_stream_protocol(ack_codec, ack_proto_control).await?;

    let (taa_tx, _taa_rx) = futures::channel::mpsc::channel::<
        TicketAggregationToProcess<TicketAggregationResponseType, TicketAggregationRequestType>,
    >(100);
    let _taa =
        TicketAggregationActions::<TicketAggregationResponseType, TicketAggregationRequestType> { queue: taa_tx };

    let swarm = swarm.with_processors(_taa);

    let api = Interface {
        me: peer_id,
        address: multiaddress,
        update_from_network: network_events_tx,
        update_from_announcements: transport_updates_tx,
        send_heartbeat: heartbeat_requests_tx,
        send_ticket_aggregation: ticket_aggregation_req_tx,
        send_msg: wire_msg_tx,
        recv_msg: wire_msg_rx,
        send_ack: wire_ack_tx,
        recv_ack: wire_ack_rx,
    };

    Ok((api, swarm))
}

const TRANSPORT_PAYLOAD_SIZE: usize = ChainPacketComponents::SIZE;

lazy_static! {
    pub static ref RANDOM_GIBBERISH: Box<[u8]> =
        Box::from(hopr_crypto_random::random_bytes::<TRANSPORT_PAYLOAD_SIZE>());
}

pub fn generate_packets_of_hopr_payload_size(count: usize) -> Vec<Box<[u8]>> {
    let mut packets = Vec::with_capacity(count);
    for _ in 0..count {
        packets.push(RANDOM_GIBBERISH.clone());
    }
    packets
}

pub struct SelfClosingJoinHandle {
    handle: Option<JoinHandle<()>>,
}

impl SelfClosingJoinHandle {
    pub fn new<F>(f: F) -> Self
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        Self { handle: Some(spawn(f)) }
    }
}

#[cfg(feature = "runtime-async-std")]
impl Drop for SelfClosingJoinHandle {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            block_on(handle.cancel());
        }
    }
}

#[cfg(feature = "runtime-async-std")]
use async_std::{
    future::timeout,
    task::{block_on, sleep, spawn, JoinHandle},
};

#[ignore]
#[cfg_attr(feature = "runtime-async-std", async_std::test)]
// #[cfg_attr(feature = "runtime-async-std", tracing_test::traced_test)]
async fn p2p_only_communication_quic() -> anyhow::Result<()> {
    let (mut api1, swarm1) = build_p2p_swarm(Announcement::QUIC).await?;
    let (api2, swarm2) = build_p2p_swarm(Announcement::QUIC).await?;

    let _sjh1 = SelfClosingJoinHandle::new(swarm1.run("1.0.0".into()));
    let _sjh2 = SelfClosingJoinHandle::new(swarm2.run("1.0.0".into()));

    // Announce nodes to each other
    api1.update_from_announcements
        .unbounded_send(PeerDiscovery::Announce(api2.me, vec![api2.address.clone()]))
        .context("failed to send announcement")?;
    api1.update_from_announcements
        .unbounded_send(PeerDiscovery::Allow(api2.me))
        .context("failed to send announcement")?;
    api2.update_from_announcements
        .unbounded_send(PeerDiscovery::Announce(api1.me, vec![api1.address.clone()]))
        .context("failed to send announcement")?;
    api2.update_from_announcements
        .unbounded_send(PeerDiscovery::Allow(api1.me))
        .context("failed to send announcement")?;

    // Wait for node listen_on and announcements
    sleep(std::time::Duration::from_secs(3)).await;

    let start = current_time();

    // ~10MB of data
    let packet_count: usize = 2 * 1024 * 10;
    for _ in 0..packet_count {
        api1.send_msg
            .send((api2.me, RANDOM_GIBBERISH.clone()))
            .await
            .context("failed to send message")?;
    }

    timeout(
        std::time::Duration::from_secs(30),
        api2.recv_msg.take(packet_count).collect::<Vec<_>>(),
    )
    .await?;

    let speed_in_mbytes_s =
        (RANDOM_GIBBERISH.len() * packet_count) as f64 / (start.elapsed()?.as_millis() as f64 * 1000f64);
    tracing::info!("The measured speed for data transfer is ~{speed_in_mbytes_s}MB/s",);

    assert!(speed_in_mbytes_s > 10.0f64);

    Ok(())
}
