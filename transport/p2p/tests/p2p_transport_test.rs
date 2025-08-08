use std::{
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
    str::FromStr,
};

use anyhow::Context;
use futures::{
    SinkExt, StreamExt,
    channel::mpsc::{Receiver, Sender},
};
use hopr_crypto_types::{keypairs::Keypair, prelude::OffchainKeypair};
use hopr_platform::time::native::current_time;
use hopr_transport_p2p::HoprSwarm;
use hopr_transport_probe::ping::PingQueryReplier;
use hopr_transport_protocol::PeerDiscovery;
use lazy_static::lazy_static;

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
    pub update_from_announcements: futures::channel::mpsc::UnboundedSender<PeerDiscovery>,
    #[allow(dead_code)]
    pub send_heartbeat: futures::channel::mpsc::UnboundedSender<(PeerId, PingQueryReplier)>,
    // ---
    pub send_msg: Sender<(PeerId, Box<[u8]>)>,
    pub recv_msg: Receiver<(PeerId, Box<[u8]>)>,
}
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum Announcement {
    QUIC,
}

pub(crate) type TestSwarm = HoprSwarm;

async fn build_p2p_swarm(announcement: Announcement) -> anyhow::Result<(Interface, TestSwarm)> {
    let random_port = random_free_local_ipv4_port().context("could not find a free port")?;
    let random_keypair = OffchainKeypair::random();
    let identity: libp2p::identity::Keypair = (&random_keypair).into();
    let peer_id: PeerId = identity.public().into();

    let (transport_updates_tx, transport_updates_rx) = futures::channel::mpsc::unbounded::<PeerDiscovery>();
    let (heartbeat_requests_tx, _heartbeat_requests_rx) =
        futures::channel::mpsc::unbounded::<(PeerId, PingQueryReplier)>();

    let multiaddress = match announcement {
        Announcement::QUIC => format!("/ip4/127.0.0.1/udp/{random_port}/quic-v1"),
    };
    let multiaddress = Multiaddr::from_str(&multiaddress).context("failed to create a valid multiaddress")?;

    let swarm = HoprSwarm::new(identity, transport_updates_rx, vec![multiaddress.clone()]).await;

    let msg_proto_control = swarm.build_protocol_control(hopr_transport_protocol::CURRENT_HOPR_MSG_PROTOCOL);
    let msg_codec = hopr_transport_protocol::HoprBinaryCodec {};
    let (wire_msg_tx, wire_msg_rx) =
        hopr_transport_protocol::stream::process_stream_protocol(msg_codec, msg_proto_control).await?;

    let api = Interface {
        me: peer_id,
        address: multiaddress,
        update_from_announcements: transport_updates_tx,
        send_heartbeat: heartbeat_requests_tx,
        send_msg: wire_msg_tx,
        recv_msg: wire_msg_rx,
    };

    Ok((api, swarm))
}

const TRANSPORT_PAYLOAD_SIZE: usize = HoprPacket::SIZE;

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

impl Drop for SelfClosingJoinHandle {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

use hopr_crypto_packet::prelude::HoprPacket;
use libp2p::{Multiaddr, PeerId};
use more_asserts::assert_gt;
use tokio::{
    task::{JoinHandle, spawn},
    time::{sleep, timeout},
};

#[ignore]
#[tokio::test]
async fn p2p_only_communication_quic() -> anyhow::Result<()> {
    let (mut api1, swarm1) = build_p2p_swarm(Announcement::QUIC).await?;
    let (api2, swarm2) = build_p2p_swarm(Announcement::QUIC).await?;

    let (tx, _rx) = futures::channel::mpsc::channel::<hopr_transport_p2p::DiscoveryEvent>(1000);

    let _sjh1 = SelfClosingJoinHandle::new(swarm1.run(tx.clone()));
    let _sjh2 = SelfClosingJoinHandle::new(swarm2.run(tx));

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

    assert_gt!(
        speed_in_mbytes_s,
        100.0f64,
        "The measured speed for data transfer is ~{}MB/s",
        speed_in_mbytes_s
    );

    Ok(())
}
