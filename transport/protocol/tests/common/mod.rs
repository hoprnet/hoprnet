use std::str::FromStr;

use anyhow::Context;
use async_std::prelude::FutureExt;
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use hex_literal::hex;
use hopr_crypto_random::{random_bytes, random_integer};
use lazy_static::lazy_static;
use libp2p::{Multiaddr, PeerId};

use core_path::{
    channel_graph::ChannelGraph,
    path::{Path, TransportPath},
};
use hopr_crypto_types::{
    keypairs::{ChainKeypair, Keypair, OffchainKeypair},
    types::{Hash, OffchainPublicKey},
};
use hopr_db_api::{info::DomainSeparator, resolver::HoprDbResolverOperations};
use hopr_db_sql::{
    accounts::HoprDbAccountOperations, channels::HoprDbChannelOperations, db::HoprDb, info::HoprDbInfoOperations,
};
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use hopr_transport_mixer::config::MixerConfig;
use hopr_transport_protocol::{
    msg::processor::{MsgSender, PacketInteractionConfig, PacketSendFinalizer},
    DEFAULT_PRICE_PER_PACKET,
};
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
}

lazy_static! {
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
}

fn create_dummy_channel(from: Address, to: Address) -> ChannelEntry {
    ChannelEntry::new(
        from,
        to,
        Balance::new(
            U256::from(1234u64) * U256::from(*DEFAULT_PRICE_PER_PACKET),
            BalanceType::HOPR,
        ),
        U256::zero(),
        ChannelStatus::Open,
        U256::zero(),
    )
}

pub async fn create_dbs(amount: usize) -> anyhow::Result<Vec<HoprDb>> {
    Ok(
        futures::future::join_all((0..amount).map(|i| HoprDb::new_in_memory(PEERS_CHAIN[i].clone())))
            .await
            .into_iter()
            .map(|v| v.map_err(|e| anyhow::anyhow!(e.to_string())))
            .collect::<anyhow::Result<Vec<HoprDb>>>()?,
    )
}

pub async fn create_minimal_topology(dbs: &mut Vec<HoprDb>) -> anyhow::Result<()> {
    let mut previous_channel: Option<ChannelEntry> = None;

    for index in 0..dbs.len() {
        dbs[index]
            .set_domain_separator(None, DomainSeparator::Channel, Hash::default())
            .await?;

        dbs[index]
            .update_ticket_price(None, Balance::new(100u128, BalanceType::HOPR))
            .await?;

        // Link all the node keys and chain keys from the simulated announcements
        for i in 0..PEERS.len() {
            let node_key = PEERS[i].public();
            let chain_key = PEERS_CHAIN[i].public();
            dbs[index]
                .insert_account(
                    None,
                    AccountEntry {
                        public_key: node_key.clone(),
                        chain_addr: chain_key.to_address(),
                        entry_type: AccountType::Announced {
                            multiaddr: Multiaddr::from_str("/ip4/127.0.0.1/tcp/4444")?,
                            updated_block: 1,
                        },
                    },
                )
                .await
                .map_err(|e| hopr_transport_protocol::errors::ProtocolError::Logic(e.to_string()))?;
        }

        let mut channel: Option<ChannelEntry> = None;
        let this_peer_chain = &PEERS_CHAIN[index];

        if index < PEERS.len() - 1 {
            channel = Some(create_dummy_channel(
                this_peer_chain.public().to_address(),
                PEERS_CHAIN[index + 1].public().to_address(),
            ));

            dbs[index]
                .upsert_channel(None, channel.context("channel should be present")?)
                .await
                .map_err(|e| hopr_transport_protocol::errors::ProtocolError::Logic(e.to_string()))?;
        }

        if index > 0 {
            dbs[index]
                .upsert_channel(None, previous_channel.context("channel should be present")?)
                .await
                .map_err(|e| hopr_transport_protocol::errors::ProtocolError::Logic(e.to_string()))?;
        }

        previous_channel = channel;
    }

    Ok(())
}

pub type WireChannels = (
    (
        futures::channel::mpsc::UnboundedSender<(PeerId, Acknowledgement)>,
        futures::channel::mpsc::UnboundedReceiver<(PeerId, Acknowledgement)>,
    ),
    (
        futures::channel::mpsc::UnboundedSender<(PeerId, Box<[u8]>)>,
        hopr_transport_mixer::channel::Receiver<(PeerId, Box<[u8]>)>,
    ),
);

pub type LogicalChannels = (
    futures::channel::mpsc::UnboundedSender<(ApplicationData, TransportPath, PacketSendFinalizer)>,
    futures::channel::mpsc::UnboundedReceiver<ApplicationData>,
);

pub type TicketChannel = futures::channel::mpsc::UnboundedReceiver<AcknowledgedTicket>;

pub async fn peer_setup_for(
    count: usize,
) -> anyhow::Result<(Vec<WireChannels>, Vec<LogicalChannels>, Vec<TicketChannel>)> {
    let peer_count = count;

    assert!(peer_count <= PEERS.len());
    assert!(peer_count >= 3);
    let mut dbs = create_dbs(peer_count).await?;

    create_minimal_topology(&mut dbs).await?;

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

    for (i, db) in dbs.into_iter().enumerate().collect::<Vec<(usize, HoprDb)>>() {
        let (received_ack_tickets_tx, received_ack_tickets_rx) =
            futures::channel::mpsc::unbounded::<AcknowledgedTicket>();

        let (wire_ack_send_tx, wire_ack_send_rx) = futures::channel::mpsc::unbounded::<(PeerId, Acknowledgement)>();
        let (wire_ack_recv_tx, wire_ack_recv_rx) = futures::channel::mpsc::unbounded::<(PeerId, Acknowledgement)>();

        let (wire_msg_send_tx, wire_msg_send_rx) = futures::channel::mpsc::unbounded::<(PeerId, Box<[u8]>)>();
        let (mixer_channel_tx, mixer_channel_rx) =
            hopr_transport_mixer::channel::<(PeerId, Box<[u8]>)>(MixerConfig::default());

        let (api_send_tx, api_send_rx) =
            futures::channel::mpsc::unbounded::<(ApplicationData, TransportPath, PacketSendFinalizer)>();
        let (api_recv_tx, api_recv_rx) = futures::channel::mpsc::unbounded::<ApplicationData>();

        let opk: &OffchainKeypair = &PEERS[i];
        let ock: &ChainKeypair = &PEERS_CHAIN[i];
        let packet_cfg = PacketInteractionConfig {
            check_unrealized_balance: true,
            packet_keypair: opk.clone(),
            chain_keypair: ock.clone(),
            outgoing_ticket_win_prob: 1.0,
        };

        db.start_ticket_processing(Some(received_ack_tickets_tx))?;

        hopr_transport_protocol::run_msg_ack_protocol(
            packet_cfg,
            db,
            None,
            (wire_ack_recv_tx, wire_ack_send_rx),
            (mixer_channel_tx, wire_msg_send_rx),
            (api_recv_tx, api_send_rx),
        )
        .await;

        wire_channels.push((
            (wire_ack_send_tx, wire_ack_recv_rx),
            (wire_msg_send_tx, mixer_channel_rx),
        ));

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

            if i != components.len() - 1 {
                debug!("Resending message to the next");
                let (peer, data) = components[i]
                    .1
                     .1
                    .next()
                    .await
                    .expect("MSG relayer should forward a msg to the next");

                assert_eq!(peer, PEERS[i + 1].public().into());

                debug!(from = i, to = i + 1, "relaying packet");
                components[i + 1]
                    .1
                     .0
                    .send((PEERS[i].public().into(), data))
                    .await
                    .expect("Send to relayer should succeed");
            }

            if i != 0 {
                debug!("Peeking into the ack queue");
                let (peer, ack) = components[i]
                    .0
                     .1
                    .next()
                    .await
                    .expect("MSG relayer should ack the forwarded packet back");

                assert_eq!(peer, PEERS[i - 1].public().into());

                debug!(from = i, to = i - 1, "sending ack back");
                components[i - 1]
                    .0
                     .0
                    .send((PEERS[i].public().into(), ack))
                    .await
                    .expect("ACK send to originator should succeed");
            }
        }
    }
}

struct TestResolver(Vec<(OffchainPublicKey, Address)>);

#[async_trait]
impl HoprDbResolverOperations for TestResolver {
    async fn resolve_packet_key(
        &self,
        onchain_key: &Address,
    ) -> hopr_db_api::errors::Result<Option<OffchainPublicKey>> {
        Ok(self.0.iter().find(|(_, addr)| addr.eq(onchain_key)).map(|(pk, _)| *pk))
    }

    async fn resolve_chain_key(
        &self,
        offchain_key: &OffchainPublicKey,
    ) -> hopr_db_api::errors::Result<Option<Address>> {
        Ok(self.0.iter().find(|(pk, _)| pk.eq(offchain_key)).map(|(_, addr)| *addr))
    }
}

pub async fn resolve_mock_path(
    me: Address,
    peers_offchain: Vec<PeerId>,
    peers_onchain: Vec<Address>,
) -> anyhow::Result<TransportPath> {
    let peers_addrs = peers_offchain
        .iter()
        .zip(peers_onchain)
        .map(|(peer_id, addr)| {
            (
                OffchainPublicKey::try_from(peer_id).expect("should be valid PeerId"),
                addr,
            )
        })
        .collect::<Vec<_>>();

    let mut cg = ChannelGraph::new(me);
    let mut last_addr = cg.my_address();
    for (_, addr) in peers_addrs.iter() {
        let c = ChannelEntry::new(
            last_addr,
            *addr,
            Balance::new(1000_u32, BalanceType::HOPR),
            0u32.into(),
            ChannelStatus::Open,
            0u32.into(),
        );
        cg.update_channel(c);
        last_addr = *addr;
    }
    Ok(TransportPath::resolve(peers_offchain, &TestResolver(peers_addrs), &cg)
        .await?
        .0)
}

pub fn random_packets_of_count(size: usize) -> Vec<ApplicationData> {
    (0..size)
        .map(|i| ApplicationData {
            application_tag: (i == 0).then(|| random_integer(1, Some(65535)) as Tag),
            plain_text: random_bytes::<300>().into(),
        })
        .collect::<Vec<_>>()
}

pub async fn send_relay_receive_channel_of_n_peers(
    peer_count: usize,
    mut test_msgs: Vec<ApplicationData>,
) -> anyhow::Result<()> {
    let packet_count = test_msgs.len();

    assert!(peer_count >= 3, "invalid peer count given");
    assert!(test_msgs.len() >= 1, "at least one packet must be given");

    const TIMEOUT_SECONDS: std::time::Duration = std::time::Duration::from_secs(10);

    let (wire_apis, mut apis, ticket_channels) = peer_setup_for(peer_count).await?;

    // Peer 1: start sending out packets
    let packet_path = resolve_mock_path(
        PEERS_CHAIN[0].public().to_address(),
        PEERS[1..peer_count].iter().map(|p| p.public().into()).collect(),
        PEERS_CHAIN[1..peer_count]
            .iter()
            .map(|key| key.public().to_address())
            .collect(),
    )
    .await?;

    assert_eq!(peer_count - 1, packet_path.length(), "path has invalid length");

    async_std::task::spawn(emulate_channel_communication(packet_count, wire_apis));

    let mut sent_packet_count = 0;
    for i in 0..packet_count {
        let sender = MsgSender::new(apis[0].0.clone());

        let awaiter = sender.send_packet(test_msgs[i].clone(), packet_path.clone()).await?;

        if awaiter
            .consume_and_wait(std::time::Duration::from_millis(500))
            .await
            .is_ok()
        {
            sent_packet_count += 1;
        }
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
        recv_packets.sort_by(|a, b| a.plain_text.cmp(&b.plain_text));

        assert_eq!(recv_packets, test_msgs);
    };

    let res = compare_packets.timeout(TIMEOUT_SECONDS).await;

    assert!(
        res.is_ok(),
        "test timed out after {} seconds",
        TIMEOUT_SECONDS.as_secs()
    );

    assert_eq!(ticket_channels.len(), peer_count);

    for (i, rx) in ticket_channels.into_iter().enumerate() {
        let expected_tickets = if i != 0 && i != peer_count - 1 { packet_count } else { 0 };

        assert_eq!(
            rx.take(expected_tickets)
                .count()
                .timeout(std::time::Duration::from_secs(1))
                .await
                .context("peer should be able to extract expected tickets")?,
            expected_tickets,
            "peer {} did not receive the expected amount of tickets",
            i,
        );
    }

    Ok(())
}
