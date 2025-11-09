use std::sync::Arc;

use anyhow::Context;
use bimap::BiHashMap;
use futures::{SinkExt, StreamExt};
use hex_literal::hex;
use hopr_api::chain::*;
use hopr_chain_connector::create_trustful_hopr_blokli_connector;
use hopr_crypto_random::{Randomizable, random_bytes, random_integer};
use hopr_crypto_types::prelude::*;
use hopr_db_node::HoprNodeDb;
use hopr_internal_types::prelude::*;
use hopr_network_types::prelude::ResolvedTransportRouting;
use hopr_primitive_types::prelude::*;
use hopr_protocol_app::prelude::*;
use hopr_transport_mixer::config::MixerConfig;
use hopr_transport_protocol::processor::PacketInteractionConfig;
use lazy_static::lazy_static;
use libp2p::PeerId;
use tokio::time::timeout;
use tracing::debug;

lazy_static! {
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

    /// Fixed price per packet to 0.01 HOPR
    pub static ref DEFAULT_PRICE_PER_PACKET: U256 = 10000000000000000u128.into();

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

    static ref MAPPER: bimap::BiHashMap<KeyIdent, OffchainPublicKey> = PEERS
        .iter()
        .enumerate()
        .map(|(i, k)| (KeyIdent::from(i as u32), *k.public()))
        .collect::<BiHashMap<_, _>>();

    static ref CHAIN_DATA: hopr_chain_connector::testing::BlokliTestStateBuilder = hopr_chain_connector::testing::BlokliTestStateBuilder::default()
        .with_accounts(PEERS.iter().enumerate().map(|(i, kp)| {
            let node_key = kp.public();
            let chain_key = PEERS_CHAIN[i].public();
            AccountEntry {
                public_key: *node_key,
                chain_addr: chain_key.to_address(),
                entry_type: AccountType::Announced("/ip4/127.0.0.1/tcp/4444".parse().unwrap()),
                safe_address: None,
                key_id: (i as u32).into(),
            }
        }))
        .with_channels(PEERS_CHAIN.iter().take(PEERS_CHAIN.len()-1).enumerate().map(|(i, cp)| {
            create_dummy_channel(cp.public().to_address(), PEERS_CHAIN[i+1].public().to_address())
        }));
}

fn create_dummy_channel(from: Address, to: Address) -> ChannelEntry {
    ChannelEntry::new(
        from,
        to,
        (U256::from(1234u64) * U256::from(*DEFAULT_PRICE_PER_PACKET)).into(),
        U256::zero(),
        ChannelStatus::Open,
        U256::zero(),
    )
}

pub async fn create_dbs(amount: usize) -> anyhow::Result<Vec<HoprNodeDb>> {
    let node_dbs = futures::future::join_all((0..amount).map(|i| HoprNodeDb::new_in_memory(PEERS_CHAIN[i].clone())))
        .await
        .into_iter()
        .map(|v| v.map_err(|e| anyhow::anyhow!(e.to_string())))
        .collect::<anyhow::Result<Vec<HoprNodeDb>>>()?;

    Ok(node_dbs)
}

#[allow(dead_code)]
pub type WireChannels = (
    futures::channel::mpsc::UnboundedSender<(PeerId, Box<[u8]>)>,
    hopr_transport_mixer::channel::Receiver<(PeerId, Box<[u8]>)>,
);

#[allow(dead_code)]
pub type LogicalChannels = (
    futures::channel::mpsc::UnboundedSender<(ResolvedTransportRouting, ApplicationDataOut)>,
    futures::channel::mpsc::UnboundedReceiver<(HoprPseudonym, ApplicationDataIn)>,
);

#[allow(dead_code)]
pub type TicketChannel = futures::channel::mpsc::UnboundedReceiver<AcknowledgedTicket>;

#[allow(dead_code)]
pub async fn peer_setup_for(
    count: usize,
) -> anyhow::Result<(Vec<WireChannels>, Vec<LogicalChannels>, Vec<TicketChannel>)> {
    let peer_count = count;

    assert!(peer_count <= PEERS.len());
    assert!(peer_count >= 3);
    let node_dbs = create_dbs(peer_count).await?;

    // Begin tests
    for i in 0..peer_count {
        let peer_type = {
            if i == 0 {
                "sender"
            } else if i == (peer_count - 1) {
                "recipient"
            } else {
                "relayer"
            }
        };

        debug!("peer {i} ({peer_type})    = {}", PEERS[i].public().to_peerid_str());
    }

    let mut wire_channels = Vec::new();
    let mut logical_channels = Vec::new();
    let mut ticket_channels = Vec::new();

    for (i, node_db) in node_dbs.into_iter().enumerate() {
        let (received_ack_tickets_tx, received_ack_tickets_rx) =
            futures::channel::mpsc::unbounded::<AcknowledgedTicket>();

        let (wire_msg_send_tx, wire_msg_send_rx) = futures::channel::mpsc::unbounded::<(PeerId, Box<[u8]>)>();
        let (mixer_channel_tx, mixer_channel_rx) =
            hopr_transport_mixer::channel::<(PeerId, Box<[u8]>)>(MixerConfig::default());

        let (api_send_tx, api_send_rx) =
            futures::channel::mpsc::unbounded::<(ResolvedTransportRouting, ApplicationDataOut)>();
        let (api_recv_tx, api_recv_rx) = futures::channel::mpsc::unbounded::<(HoprPseudonym, ApplicationDataIn)>();

        let opk: &OffchainKeypair = &PEERS[i];
        let packet_cfg = PacketInteractionConfig {
            packet_keypair: opk.clone(),
            outgoing_ticket_win_prob: Some(WinningProbability::ALWAYS),
            outgoing_ticket_price: Some((*DEFAULT_PRICE_PER_PACKET).into()),
        };

        node_db.start_ticket_processing(Some(received_ack_tickets_tx))?;
        let connector =
            create_trustful_hopr_blokli_connector(&PEERS_CHAIN[0], CHAIN_DATA.clone().build_static_client(), Default::default()).await?;

        hopr_transport_protocol::run_msg_ack_protocol(
            packet_cfg,
            node_db,
            Arc::new(connector),
            (mixer_channel_tx, wire_msg_send_rx),
            (api_recv_tx, api_send_rx),
        )
        .await;

        wire_channels.push((wire_msg_send_tx, mixer_channel_rx));

        logical_channels.push((api_send_tx, api_recv_rx));
        ticket_channels.push(received_ack_tickets_rx)
    }

    Ok((wire_channels, logical_channels, ticket_channels))
}

#[tracing::instrument(level = "debug", skip(components))]
pub async fn emulate_channel_communication(pending_packet_count: usize, mut components: Vec<WireChannels>) {
    for i in 0..components.len() {
        for j in 0..pending_packet_count {
            debug!("Component: {i} on packet {j}");

            let count = if i == 0 || i == components.len() - 1 { 1 } else { 2 };

            for _i in 0..count {
                let (dest, payload) = components[i]
                    .1
                    .next()
                    .await
                    .expect("MSG relayer should forward a msg to the next");

                let destination = if i == 0 {
                    assert_eq!(
                        dest,
                        PEERS[i + 1].public().into(),
                        "first peer should send only data to the next one"
                    );
                    i + 1
                } else if i == components.len() - 1 {
                    assert_eq!(
                        dest,
                        PEERS[i - 1].public().into(),
                        "last peer should send only ack to the previous one"
                    );
                    i - 1
                } else if dest == PEERS[i + 1].public().into() {
                    debug!(%dest, "sending data to next");
                    i + 1
                } else if dest == PEERS[i - 1].public().into() {
                    debug!(%dest, "sending ack to previous");
                    i - 1
                } else {
                    panic!("Unexpected destination");
                };

                components[destination]
                    .0
                    .send((PEERS[i].public().into(), payload))
                    .await
                    .expect("Sending of payload to the peer failed");
            }
        }
    }

    futures::future::pending::<()>().await;
}

pub async fn resolve_mock_path(me: Address, peers_onchain: Vec<Address>) -> anyhow::Result<ValidatedPath> {
    let connector =
        create_trustful_hopr_blokli_connector(&PEERS_CHAIN[0], CHAIN_DATA.clone().build_static_client(), Default::default()).await?;
    let resolver = connector.as_path_resolver();
    Ok(ValidatedPath::new(me, ChainPath::new(peers_onchain)?, &resolver).await?)
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

#[allow(dead_code)]
pub async fn send_relay_receive_channel_of_n_peers(
    peer_count: usize,
    mut test_msgs: Vec<ApplicationData>,
) -> anyhow::Result<()> {
    let packet_count = test_msgs.len();

    assert!(peer_count >= 3, "invalid peer count given");
    assert!(!test_msgs.is_empty(), "at least one packet must be given");

    const TIMEOUT_SECONDS: std::time::Duration = std::time::Duration::from_secs(10);

    let (wire_apis, mut apis, ticket_channels) = peer_setup_for(peer_count).await?;

    // Peer 1: start sending out packets
    let packet_path = resolve_mock_path(
        PEERS_CHAIN[0].public().to_address(),
        PEERS_CHAIN[1..peer_count]
            .iter()
            .map(|key| key.public().to_address())
            .collect(),
    )
    .await?;

    assert_eq!(peer_count - 1, packet_path.num_hops(), "path has invalid length");

    tokio::task::spawn(emulate_channel_communication(packet_count, wire_apis));

    let pseudonym = HoprPseudonym::random();
    let mut sent_packet_count = 0;
    for test_msg in test_msgs.iter().take(packet_count) {
        let mut sender = apis[0].0.clone();
        let routing = ResolvedTransportRouting::Forward {
            pseudonym,
            forward_path: packet_path.clone(),
            return_paths: vec![],
        };

        sender
            .send((routing, ApplicationDataOut::with_no_packet_info(test_msg.clone())))
            .await?;

        sent_packet_count += 1;
    }

    assert_eq!(
        sent_packet_count, packet_count,
        "not all packets were successfully sent"
    );

    let compare_packets = async move {
        let last_node_recv = apis.remove(peer_count - 1).1;

        let mut recv_packets = last_node_recv
            .take(packet_count)
            .map(|packet| packet)
            .collect::<Vec<_>>()
            .await;

        assert_eq!(recv_packets.len(), test_msgs.len());

        test_msgs.sort_by(|a, b| a.plain_text.cmp(&b.plain_text));
        recv_packets.sort_by(|(_, a), (_, b)| a.data.plain_text.cmp(&b.data.plain_text));

        assert_eq!(
            recv_packets.into_iter().map(|(_, b)| b.data).collect::<Vec<_>>(),
            test_msgs
        );
    };

    let res = timeout(TIMEOUT_SECONDS, compare_packets).await;

    assert!(
        res.is_ok(),
        "test timed out after {} seconds",
        TIMEOUT_SECONDS.as_secs()
    );

    assert_eq!(ticket_channels.len(), peer_count);

    for (i, rx) in ticket_channels.into_iter().enumerate() {
        let expected_tickets = if i != 0 && i != peer_count - 1 { packet_count } else { 0 };

        assert_eq!(
            timeout(std::time::Duration::from_secs(1), rx.take(expected_tickets).count())
                .await
                .context("peer should be able to extract expected tickets")?,
            expected_tickets,
            "peer {i} did not receive the expected amount of tickets",
        );
    }

    Ok(())
}
