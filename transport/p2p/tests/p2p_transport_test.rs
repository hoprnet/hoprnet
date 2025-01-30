use std::{
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
    str::FromStr,
};

use anyhow::Context;
use futures::StreamExt;
use libp2p::{Multiaddr, PeerId};

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

pub(crate) struct Interface {
    pub me: PeerId,
    pub address: Multiaddr,
    pub update_from_network: futures::channel::mpsc::Sender<NetworkTriggeredEvent>,
    pub update_from_announcements: futures::channel::mpsc::UnboundedSender<PeerDiscovery>,
    pub send_heartbeat: futures::channel::mpsc::UnboundedSender<(PeerId, PingQueryReplier)>,
    pub send_ticket_aggregation: futures::channel::mpsc::UnboundedSender<TicketAggregationEvent>,
    // ---
    pub send_msg: futures::channel::mpsc::UnboundedSender<(PeerId, Box<[u8]>)>,
    pub recv_msg: futures::channel::mpsc::UnboundedReceiver<(PeerId, Box<[u8]>)>,
    pub send_ack: futures::channel::mpsc::UnboundedSender<(PeerId, Acknowledgement)>,
    pub recv_ack: futures::channel::mpsc::UnboundedReceiver<(PeerId, Acknowledgement)>,
}
pub(crate) enum Announcement {
    TCP,
    QUIC,
}

pub(crate) type TestSwarm = HoprSwarmWithProcessors<
    futures::channel::mpsc::UnboundedReceiver<(PeerId, Box<[u8]>)>,
    futures::channel::mpsc::UnboundedSender<(PeerId, Box<[u8]>)>,
    futures::channel::mpsc::UnboundedReceiver<(PeerId, Acknowledgement)>,
    futures::channel::mpsc::UnboundedSender<(PeerId, Acknowledgement)>,
>;

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
        Announcement::TCP => format!("/ip4/127.0.0.1/tcp/{random_port}"),
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

/// 500 characters long string of random gibberish
const RANDOM_GIBBERISH: &str = "abcdferjskdiq7LGuzjfXMEI2tTCUIZsCDsHnfycUbPcA1IvxsrbK3bNCevOMXYMqrhsVBXfmKy23K7ItgbuObTmqk0ndfceAhugLZveAhp4Xx1vHCAROY69sOTJiia3EBC2aXSBpUfb3WHSJDxHRMHwzCwd0BPj4WFi4Ig884Ph6altlFWzpL3ILsHmLxy9KoPCAtolb3YEegMCI4y9BsoWyCtcZdBHBrqXaSzuJivw5J1DBudj3Z6oORrEfRuFIQLi0l89Emc35WhSyzOdguC1x9PS8AiIAu7UoXlp3VIaqVUu4XGUZ21ABxI9DyMzxGbOOlsrRGFFN9G8di9hqIX1UOZpRgMNmtDwZoyoU2nGLoWGM58buwuvbNkLjGu2X9HamiiDsRIR4vxi5i61wIP6VueVOb68wvbz8csR88OhFsExjGBD9XXtJvUjy1nwdkikBOblNm2FUbyq8aHwHocoMqZk8elbYMHgbjme9d1CxZQKRwOR";

pub fn generate_packets_of_hopr_payload_size(count: usize) -> Vec<Box<[u8]>> {
    let mut packets = Vec::with_capacity(count);
    for i in 0..count {
        packets.push(Box::from(RANDOM_GIBBERISH.as_bytes()));
    }
    packets
}

pub struct SelfClosingJoinHandle {
    handle: JoinHandle<()>,
}

impl SelfClosingJoinHandle {
    pub fn new<F>(f: F) -> Self
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        Self { handle: spawn(f) }
    }
}

impl Drop for SelfClosingJoinHandle {
    fn drop(&mut self) {
        #[cfg(feature = "runtime-async-std")]
        block_on(self.handle.cancel());
        #[cfg(feature = "runtime-tokio")]
        self.handle.abort()
    }
}

#[cfg(feature = "runtime-async-std")]
use async_std::{
    future::time::timeout,
    task::{block_on, spawn, JoinHandle},
};

#[cfg(all(feature = "runtime-tokio", not(feature = "runtime-async-std")))]
use tokio::{
    task::{spawn, JoinHandle},
    time::timeout,
};

#[cfg_attr(feature = "runtime-async-std", test_log::test(async_std::test))]
#[cfg_attr(
    all(feature = "runtime-tokio", not(feature = "runtime-async-std")),
    test_log::test(tokio::test)
)]
async fn p2p_only_communication_tcp() -> anyhow::Result<()> {
    let (api1, swarm1) = build_p2p_swarm(Announcement::TCP).await?;
    let (mut api2, swarm2) = build_p2p_swarm(Announcement::TCP).await?;

    let sjh1 = SelfClosingJoinHandle::new(swarm1.run("1.0.0".into()));
    let sjh2 = SelfClosingJoinHandle::new(swarm2.run("1.0.0".into()));

    std::thread::sleep(std::time::Duration::from_secs(5));

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

    api1.send_msg
        .unbounded_send((api2.me, Box::from(RANDOM_GIBBERISH.as_bytes())))
        .context("failed to send message")?;

    timeout(std::time::Duration::from_secs(3), api2.recv_msg.next())
        .await?
        .context("failed to receive message")?
        .1
        .as_ref()
        .iter()
        .zip(RANDOM_GIBBERISH.as_bytes())
        .for_each(|(a, b)| assert_eq!(a, b));

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
