// Emulated multi-peer harness shared across hopr-transport integration tests and benches.
//
// Provides:
//  - `PEERS` / `PEERS_CHAIN` — fixed keypair sets for deterministic test topology.
//  - `CHAIN_DATA` — pre-seeded Blokli emulator state (channels, balances, ticket price).
//  - Payload generators: `random_packets_of_count`, `random_packet_of_size`.
//  - Routing helpers: `resolve_mock_path`, `make_routing`, `make_outgoing_packets`.
//  - Per-peer pipeline wiring: `peer_setup_for`, `peer_setup_for_with_cfg`,
//    `peer_setup_for_with_counters`.
//  - In-process software transport: `emulate_channel_communication` — routes `(PeerId, Bytes)`
//    between peers without any real network sockets.
//  - Convenience combined harness: `send_and_receive_packets`, `send_relay_receive_channel_of_n_peers`.
//
// Originally hosted in `tests/protocol/common/mod.rs`; moved here so both benches and
// integration tests can consume it without the `#[path=...]` hack.
#![allow(dead_code)]

use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, LazyLock},
    time::Duration,
};

use anyhow::Context;
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use futures_concurrency::stream::StreamGroup;
use futures_time::future::FutureExt;
use hex_literal::hex;
use hopr_api::{
    chain::*,
    node::TicketEvent,
    types::{
        crypto::prelude::*,
        crypto_random::{random_bytes, random_integer},
        internal::{errors::PathError, prelude::*, routing::ResolvedTransportRouting},
        primitive::prelude::*,
    },
};
use hopr_chain_connector::create_trustful_hopr_blokli_connector;
use hopr_crypto_packet::HoprSurb;
use hopr_protocol_app::prelude::*;
use hopr_protocol_hopr::{
    HoprCodecConfig, HoprDecoder, HoprEncoder, HoprUnacknowledgedTicketProcessor,
    HoprUnacknowledgedTicketProcessorConfig, MemorySurbStore, SurbStoreConfig,
};
use hopr_ticket_manager::{HoprTicketFactory, RedbStore};
use hopr_transport_mixer::config::MixerConfig;
use hopr_utils::runtime::AbortableList;
use lazy_static::lazy_static;
use libp2p::PeerId;
use tracing::debug;

use crate::protocol::{PacketPipelineBuilder, PacketPipelineConfig, PeerProtocolCounterRegistry};

lazy_static! {
    static ref DEFAULT_PRICE_PER_PACKET: HoprBalance = HoprBalance::from_str("0.1 wxHOPR").unwrap();
    pub static ref PEERS: Vec<OffchainKeypair> = [
        hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"),
        hex!("5bf21ea8cccd69aa784346b07bf79c84dac606e00eecaa68bf8c31aff397b1ca"),
        hex!("3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa"),
        hex!("db7e3e8fcac4c817aa4cecee1d6e2b4d53da51f9881592c0e1cc303d8a012b92"),
        hex!("0726a9704d56a013980a9077d195520a61b5aed28f92d89c50bca6e0e0c48cfc")
    ]
    .iter()
    .map(|private| OffchainKeypair::from_secret(private).expect("lazy static keypair should be valid"))
    .collect();
    pub static ref PEERS_CHAIN: Vec<ChainKeypair> = [
        hex!("4db3ac225fdcc7e20bf887cd90bbd62dc6bd41ce8ba5c23cc9ae0bf56e20d056"),
        hex!("1d40c69c179528bbdf49c2254e93400b485f47d7d2fa84aae280af5a31c1918b"),
        hex!("99facd2cd33664d65826ad220920a6b356e31d18c1ce1734303b70a962664d71"),
        hex!("62b362fd3295caf8657b8cf4f65d6e2cbb1ef81754f7bdff65e510220611afc2"),
        hex!("40ed717eb285dea3921a8346155d988b7ed5bf751bc4eee3cd3a64f4c692396f")
    ]
    .iter()
    .map(|private| ChainKeypair::from_secret(private).expect("lazy static keypair should be valid"))
    .collect();
    pub static ref CHANNELS: Vec<ChannelEntry> = PEERS_CHAIN
        .iter()
        .take(PEERS_CHAIN.len() - 1)
        .enumerate()
        .map(|(i, cp)| { create_dummy_channel(cp.public().to_address(), PEERS_CHAIN[i + 1].public().to_address()) })
        .collect::<Vec<_>>();
    pub static ref CHAIN_DATA: hopr_chain_connector::testing::BlokliTestStateBuilder =
        hopr_chain_connector::testing::BlokliTestStateBuilder::default()
            .with_hopr_network_chain_info("anvil-localhost")
            .with_accounts(PEERS.iter().enumerate().map(|(i, kp)| {
                let node_key = kp.public();
                let chain_key = PEERS_CHAIN[i].public();
                (
                    AccountEntry {
                        public_key: *node_key,
                        chain_addr: chain_key.to_address(),
                        entry_type: AccountType::Announced(vec![format!("/ip4/127.0.0.1/tcp/444{i}").parse().unwrap()]),
                        safe_address: None,
                        key_id: ((i + 1) as u32).into(),
                    },
                    HoprBalance::new_base(1000),
                    XDaiBalance::new_base(1),
                )
            }))
            .with_channels(CHANNELS.iter().cloned())
            .with_ticket_price(*DEFAULT_PRICE_PER_PACKET);
}

fn create_dummy_channel(from: Address, to: Address) -> ChannelEntry {
    ChannelEntry::builder()
        .between(from, to)
        .balance(*DEFAULT_PRICE_PER_PACKET * 100)
        .ticket_index(0)
        .status(ChannelStatus::Open)
        .epoch(0)
        .build()
        .unwrap()
}

pub type WireChannels = (
    futures::channel::mpsc::UnboundedSender<(PeerId, Bytes)>,
    hopr_transport_mixer::channel::Receiver<(PeerId, Bytes)>,
);

pub type LogicalChannels = (
    futures::channel::mpsc::UnboundedSender<(ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)>,
    futures::channel::mpsc::UnboundedReceiver<(HoprPseudonym, ApplicationDataIn)>,
);

pub type TicketChannel = futures::channel::mpsc::UnboundedReceiver<TicketEvent>;

pub async fn peer_setup_for(
    count: usize,
) -> anyhow::Result<(
    HashMap<PeerId, WireChannels>,
    Vec<LogicalChannels>,
    Vec<TicketChannel>,
    AbortableList<usize>,
)> {
    let (w, l, t, p, _) = peer_setup_for_with_all(count, Default::default()).await?;
    Ok((w, l, t, p))
}

#[tracing::instrument(level = "debug", skip(components))]
pub async fn emulate_channel_communication(components: HashMap<PeerId, WireChannels>) {
    let (mut senders, streams): (HashMap<_, _>, Vec<_>) = components
        .into_iter()
        .map(|(peer, (tx, rx))| ((peer, tx), rx.map(move |(target, msg)| (peer, target, msg))))
        .unzip();

    let mut stream_group = StreamGroup::from_iter(streams);
    while let Some((sender, target, msg)) = stream_group.next().await {
        let target_sender = senders
            .get_mut(&target)
            .unwrap_or_else(|| panic!("peer {target} should be part of the test setup"));

        tracing::trace!(%sender, %target, "transporting packet");

        target_sender
            .send((sender, msg))
            .await
            .expect("failed to send packet to peer");
    }
}

struct MockPathResolver;

#[async_trait::async_trait]
impl PathAddressResolver for MockPathResolver {
    async fn resolve_transport_address(&self, address: &Address) -> Result<Option<OffchainPublicKey>, PathError> {
        let index = PEERS_CHAIN
            .iter()
            .enumerate()
            .find(|(_, key)| key.public().to_address() == *address)
            .map(|(i, _)| i);

        Ok(index.map(|i| *PEERS[i].public()))
    }

    async fn resolve_chain_address(&self, key: &OffchainPublicKey) -> Result<Option<Address>, PathError> {
        let index = PEERS
            .iter()
            .enumerate()
            .find(|(_, ockey)| ockey.public() == key)
            .map(|(i, _)| i);

        Ok(index.map(|i| PEERS_CHAIN[i].public().to_address()))
    }

    async fn get_channel(&self, src: &Address, dst: &Address) -> Result<Option<ChannelEntry>, PathError> {
        Ok(CHANNELS
            .iter()
            .find(|c| &c.source == src && &c.destination == dst)
            .cloned())
    }
}

pub async fn resolve_mock_path(me: Address, peers_onchain: Vec<Address>) -> anyhow::Result<ValidatedPath> {
    let path = ValidatedPath::new(me, ChainPath::new(peers_onchain)?, &MockPathResolver).await?;
    debug!(%path, "resolved path");
    Ok(path)
}

pub fn random_packets_of_count(size: usize) -> Vec<ApplicationData> {
    (0..size)
        .map(|i| {
            ApplicationData::new(
                if i == 0 {
                    random_integer(16u64, Some(65535u64))
                } else {
                    0u64
                },
                &random_bytes::<300>(),
            )
        })
        .collect::<Result<Vec<_>, _>>()
        .expect("data generation must not fail")
}

static RANDOM_BYTE_POOL: LazyLock<[u8; 4096]> = LazyLock::new(random_bytes::<4096>);

pub fn random_packet_of_size(payload_size: usize) -> ApplicationData {
    let pool = &*RANDOM_BYTE_POOL;
    let start = random_integer(0u64, Some(pool.len() as u64)) as usize;
    let data: Vec<u8> = (0..payload_size).map(|i| pool[(start + i) % pool.len()]).collect();
    ApplicationData::new(random_integer(16u64, Some(65535u64)), data).expect("data generation must not fail")
}

pub fn make_routing(path: ValidatedPath) -> ResolvedTransportRouting<HoprSurb> {
    ResolvedTransportRouting::Forward {
        pseudonym: SimplePseudonym::from([0xde_u8, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe, 0x11, 0x22]),
        forward_path: path,
        return_paths: vec![],
    }
}

pub fn make_outgoing_packets(
    packets: &[ApplicationData],
    path: ValidatedPath,
) -> Vec<(ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)> {
    packets
        .iter()
        .map(|msg| {
            (
                make_routing(path.clone()),
                ApplicationDataOut::with_no_packet_info(msg.clone()),
            )
        })
        .collect()
}

pub async fn send_and_receive_packets(
    peer_count: usize,
    test_msgs: &[ApplicationData],
) -> anyhow::Result<(
    Vec<(HoprPseudonym, ApplicationDataIn)>,
    Vec<TicketChannel>,
    AbortableList<usize>,
)> {
    assert!(peer_count >= 3, "invalid peer count given");
    assert!(!test_msgs.is_empty(), "at least one packet must be given");

    const TIMEOUT_SECONDS: Duration = Duration::from_secs(10);

    let (wire_apis, mut apis, ticket_channels, processes) = peer_setup_for(peer_count)
        .await
        .context("failed to setup peers for test")?;

    let packet_path = resolve_mock_path(
        PEERS_CHAIN[0].public().to_address(),
        PEERS_CHAIN[1..peer_count]
            .iter()
            .map(|key| key.public().to_address())
            .collect(),
    )
    .await
    .context("failed to resolve path")?;

    tokio::task::spawn(emulate_channel_communication(wire_apis));

    let out_msgs = make_outgoing_packets(test_msgs, packet_path);
    apis[0].0.send_all(&mut futures::stream::iter(out_msgs).map(Ok)).await?;

    let (_apis_send, mut apis_recv): (Vec<_>, Vec<_>) = apis.into_iter().unzip();

    let last_node_recv = apis_recv.remove(peer_count - 1);
    let recv_packets = last_node_recv
        .take(test_msgs.len())
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TIMEOUT_SECONDS))
        .await?;

    Ok((recv_packets, ticket_channels, processes))
}

// Sets up N peers with the given `PacketPipelineConfig` and returns per-peer counter registries.
async fn peer_setup_for_with_all(
    count: usize,
    cfg: PacketPipelineConfig,
) -> anyhow::Result<(
    HashMap<PeerId, WireChannels>,
    Vec<LogicalChannels>,
    Vec<TicketChannel>,
    AbortableList<usize>,
    Vec<PeerProtocolCounterRegistry>,
)> {
    let peer_count = count;

    assert!(peer_count <= PEERS.len());
    assert!(peer_count >= 3);

    for i in 0..peer_count {
        let peer_type = if i == 0 {
            "sender"
        } else if i == peer_count - 1 {
            "recipient"
        } else {
            "relayer"
        };
        debug!(
            "peer {i} ({peer_type})    = {} ({})",
            PEERS[i].public().to_peerid_str(),
            PEERS_CHAIN[i].public().to_address()
        );
    }

    let mut wire_channels = HashMap::new();
    let mut logical_channels = Vec::new();
    let mut ticket_channels = Vec::new();
    let mut processes = AbortableList::default();
    let mut counter_registries = Vec::new();

    for i in 0..peer_count {
        let (received_ack_tickets_tx, received_ack_tickets_rx) = futures::channel::mpsc::unbounded::<TicketEvent>();

        let (wire_msg_send_tx, wire_msg_send_rx) = futures::channel::mpsc::unbounded::<(PeerId, Bytes)>();
        let (mixer_channel_tx, mixer_channel_rx) =
            hopr_transport_mixer::channel::<(PeerId, Bytes)>(MixerConfig::default());

        let (api_send_tx, api_send_rx) =
            futures::channel::mpsc::unbounded::<(ResolvedTransportRouting<HoprSurb>, ApplicationDataOut)>();
        let (api_recv_tx, api_recv_rx) = futures::channel::mpsc::unbounded::<(HoprPseudonym, ApplicationDataIn)>();

        let mut connector = create_trustful_hopr_blokli_connector(
            &PEERS_CHAIN[i],
            Default::default(),
            CHAIN_DATA.clone().build_static_client(),
            Default::default(),
        )
        .await?;
        connector.connect().await?;

        let connector = Arc::new(connector);
        let surb_store = MemorySurbStore::new(SurbStoreConfig::default());
        let channels_dst = connector.domain_separators().await?.channel;

        let codec_config = HoprCodecConfig {
            outgoing_ticket_price: Some(*DEFAULT_PRICE_PER_PACKET),
            min_incoming_ticket_price: None,
            outgoing_win_prob: Some(WinningProbability::ALWAYS),
        };

        let ticket_proc = HoprUnacknowledgedTicketProcessor::new(
            connector.clone(),
            PEERS_CHAIN[i].clone(),
            channels_dst,
            HoprUnacknowledgedTicketProcessorConfig::default(),
        );

        let ticket_factory = Arc::new(HoprTicketFactory::new(RedbStore::new_temp()?));

        let encoder = HoprEncoder::new(
            PEERS_CHAIN[i].clone(),
            connector.clone(),
            surb_store.clone(),
            ticket_factory.clone(),
            channels_dst,
            codec_config,
        );

        let decoder = HoprDecoder::new(
            (PEERS[i].clone(), PEERS_CHAIN[i].clone()),
            connector.clone(),
            surb_store,
            ticket_factory.clone(),
            channels_dst,
            codec_config,
        );

        let counters = PeerProtocolCounterRegistry::default();

        let node_processes = PacketPipelineBuilder::new(PEERS[i].clone())
            .transport((mixer_channel_tx, wire_msg_send_rx))
            .codec((encoder, decoder))
            .api((api_recv_tx, api_send_rx))
            .with_counters(counters.clone())
            .with_config(cfg)
            .with_ticket_processing(ticket_proc, received_ack_tickets_tx)
            .build_for_relay();

        wire_channels.insert(PeerId::from(*PEERS[i].public()), (wire_msg_send_tx, mixer_channel_rx));
        logical_channels.push((api_send_tx, api_recv_rx));
        ticket_channels.push(received_ack_tickets_rx);
        counter_registries.push(counters);
        processes.insert(i, node_processes);
    }

    Ok((
        wire_channels,
        logical_channels,
        ticket_channels,
        processes,
        counter_registries,
    ))
}

pub async fn peer_setup_for_with_cfg(
    count: usize,
    cfg: PacketPipelineConfig,
) -> anyhow::Result<(
    HashMap<PeerId, WireChannels>,
    Vec<LogicalChannels>,
    Vec<TicketChannel>,
    AbortableList<usize>,
)> {
    let (w, l, t, p, _) = peer_setup_for_with_all(count, cfg).await?;
    Ok((w, l, t, p))
}

pub async fn peer_setup_for_with_counters(
    count: usize,
) -> anyhow::Result<(
    HashMap<PeerId, WireChannels>,
    Vec<LogicalChannels>,
    Vec<TicketChannel>,
    AbortableList<usize>,
    Vec<PeerProtocolCounterRegistry>,
)> {
    peer_setup_for_with_all(count, Default::default()).await
}

pub fn inject_raw_wire(
    wire_tx: &futures::channel::mpsc::UnboundedSender<(PeerId, Bytes)>,
    sender: PeerId,
    bytes: impl Into<Bytes>,
) -> anyhow::Result<()> {
    wire_tx
        .unbounded_send((sender, bytes.into()))
        .map_err(|e| anyhow::anyhow!("inject_raw_wire: channel closed: {e}"))
}

pub fn corrupt_bytes(bytes: &[u8], byte_idx: usize) -> Box<[u8]> {
    let mut out: Box<[u8]> = bytes.into();
    if let Some(b) = out.get_mut(byte_idx) {
        *b ^= 0xFF;
    }
    out
}

pub async fn send_relay_receive_channel_of_n_peers(
    peer_count: usize,
    mut test_msgs: Vec<ApplicationData>,
) -> anyhow::Result<()> {
    let packet_count = test_msgs.len();

    assert!(peer_count >= 3, "invalid peer count given");
    assert!(!test_msgs.is_empty(), "at least one packet must be given");

    const TIMEOUT_SECONDS: Duration = Duration::from_secs(10);

    let (wire_apis, mut apis, ticket_channels, processes) = peer_setup_for(peer_count)
        .await
        .context("failed to setup peers for test")?;

    let packet_path = resolve_mock_path(
        PEERS_CHAIN[0].public().to_address(),
        PEERS_CHAIN[1..peer_count]
            .iter()
            .map(|key| key.public().to_address())
            .collect(),
    )
    .await
    .context("failed to resolve path")?;

    assert_eq!(peer_count - 1, packet_path.num_hops(), "path has invalid length");

    tokio::task::spawn(emulate_channel_communication(wire_apis));

    let out_msgs = make_outgoing_packets(&test_msgs, packet_path);
    apis[0].0.send_all(&mut futures::stream::iter(out_msgs).map(Ok)).await?;

    let (_apis_send, mut apis_recv): (Vec<_>, Vec<_>) = apis.into_iter().unzip();

    let last_node_recv = apis_recv.remove(peer_count - 1);
    let mut recv_packets = last_node_recv
        .take(packet_count)
        .collect::<Vec<_>>()
        .timeout(futures_time::time::Duration::from(TIMEOUT_SECONDS))
        .await?;

    assert_eq!(recv_packets.len(), test_msgs.len());

    test_msgs.sort_by(|a, b| a.plain_text.cmp(&b.plain_text));
    recv_packets.sort_by(|(_, a), (_, b)| a.data.plain_text.cmp(&b.data.plain_text));

    assert_eq!(
        recv_packets.into_iter().map(|(_, b)| b.data).collect::<Vec<_>>(),
        test_msgs
    );

    assert_eq!(ticket_channels.len(), peer_count);

    for (i, rx) in ticket_channels.into_iter().enumerate() {
        let expected_tickets = if i != 0 && i != peer_count - 1 { packet_count } else { 0 };

        assert_eq!(
            rx.take(expected_tickets)
                .filter(|e| futures::future::ready(e.is_winning_ticket()))
                .count()
                .timeout(futures_time::time::Duration::from(TIMEOUT_SECONDS))
                .await
                .context("peer should be able to extract expected tickets")?,
            expected_tickets,
            "peer {i} did not receive the expected amount of tickets",
        );
    }

    tracing::trace!("all peers finished");
    processes.abort_all();
    Ok(())
}
