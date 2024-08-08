use std::{str::FromStr, time::Duration};

use async_trait::async_trait;
use futures::{future::Either, pin_mut, stream::select};
use hex_literal::hex;
use hopr_crypto_random::{random_bytes, random_integer};
use lazy_static::lazy_static;
use libp2p::{Multiaddr, PeerId};
use serial_test::serial;

use core_path::{
    channel_graph::ChannelGraph,
    path::{Path, TransportPath},
};
use core_protocol::{
    bloom::WrappedTagBloomFilter,
    msg::{mixer::MixerConfig, processor::PacketInteractionConfig},
    DEFAULT_PRICE_PER_PACKET,
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
use tracing::debug;

lazy_static! {
    static ref PEERS: Vec<OffchainKeypair> = [
        hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"),
        hex!("5bf21ea8cccd69aa784346b07bf79c84dac606e00eecaa68bf8c31aff397b1ca"),
        hex!("3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa"),
        hex!("db7e3e8fcac4c817aa4cecee1d6e2b4d53da51f9881592c0e1cc303d8a012b92"),
        hex!("0726a9704d56a013980a9077d195520a61b5aed28f92d89c50bca6e0e0c48cfc")
    ]
    .iter()
    .map(|private| OffchainKeypair::from_secret(private).unwrap())
    .collect();
}

lazy_static! {
    static ref PEERS_CHAIN: Vec<ChainKeypair> = [
        hex!("4db3ac225fdcc7e20bf887cd90bbd62dc6bd41ce8ba5c23cc9ae0bf56e20d056"),
        hex!("1d40c69c179528bbdf49c2254e93400b485f47d7d2fa84aae280af5a31c1918b"),
        hex!("99facd2cd33664d65826ad220920a6b356e31d18c1ce1734303b70a962664d71"),
        hex!("62b362fd3295caf8657b8cf4f65d6e2cbb1ef81754f7bdff65e510220611afc2"),
        hex!("40ed717eb285dea3921a8346155d988b7ed5bf751bc4eee3cd3a64f4c692396f")
    ]
    .iter()
    .map(|private| ChainKeypair::from_secret(private).unwrap())
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

async fn create_dbs(amount: usize) -> Vec<HoprDb> {
    futures::future::join_all((0..amount).map(|i| HoprDb::new_in_memory(PEERS_CHAIN[i].clone()))).await
}

async fn create_minimal_topology(dbs: &mut Vec<HoprDb>) -> core_protocol::errors::Result<()> {
    let mut previous_channel: Option<ChannelEntry> = None;

    for index in 0..dbs.len() {
        dbs[index]
            .set_domain_separator(None, DomainSeparator::Channel, Hash::default())
            .await
            .map_err(|e| core_protocol::errors::ProtocolError::Logic(e.to_string()))?;

        dbs[index]
            .update_ticket_price(None, Balance::new(100u128, BalanceType::HOPR))
            .await
            .map_err(|e| core_protocol::errors::ProtocolError::Logic(e.to_string()))?;

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
                            multiaddr: Multiaddr::from_str("/ip4/127.0.0.1/tcp/4444").unwrap(),
                            updated_block: 1,
                        },
                    },
                )
                .await
                .map_err(|e| core_protocol::errors::ProtocolError::Logic(e.to_string()))?;
        }

        let mut channel: Option<ChannelEntry> = None;
        let this_peer_chain = &PEERS_CHAIN[index];

        if index < PEERS.len() - 1 {
            channel = Some(create_dummy_channel(
                this_peer_chain.public().to_address(),
                PEERS_CHAIN[index + 1].public().to_address(),
            ));

            dbs[index]
                .upsert_channel(None, channel.unwrap())
                .await
                .map_err(|e| core_protocol::errors::ProtocolError::Logic(e.to_string()))?;
        }

        if index > 0 {
            dbs[index]
                .upsert_channel(None, previous_channel.unwrap())
                .await
                .map_err(|e| core_protocol::errors::ProtocolError::Logic(e.to_string()))?;
        }

        previous_channel = channel;
    }

    Ok(())
}

async fn peer_setup_for(count: usize) -> Vec<(AcknowledgementInteraction, PacketInteraction)> {
    let peer_count = count;

    assert!(peer_count <= PEERS.len());
    assert!(peer_count >= 3);
    let mut dbs = create_dbs(peer_count).await;

    create_minimal_topology(&mut dbs)
        .await
        .expect("failed to create minimal channel topology");

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

    dbs.into_iter()
        .enumerate()
        .map(|(i, db)| {
            let ack = AcknowledgementInteraction::new(db.clone(), &PEERS_CHAIN[i]);
            let pkt = PacketInteraction::new(
                db.clone(),
                WrappedTagBloomFilter::new("/ratata/invalid_path".into()),
                PacketInteractionConfig {
                    check_unrealized_balance: true,
                    packet_keypair: PEERS[i].clone(),
                    chain_keypair: PEERS_CHAIN[i].clone(),
                    mixer: MixerConfig::default(), // TODO: unnecessary, can be removed
                },
            );

            (ack, pkt)
        })
        .collect::<Vec<_>>()
}

async fn emulate_channel_communication(
    pending_packet_count: usize,
    mut components: Vec<(AcknowledgementInteraction, PacketInteraction)>,
) -> (Vec<ApplicationData>, usize, Vec<AcknowledgedTicket>) {
    let component_length = components.len();
    let mut received_packets: Vec<ApplicationData> = vec![];
    let mut received_challenges: usize = 0;
    let mut received_tickets: Vec<AcknowledgedTicket> = vec![];

    for _ in 0..pending_packet_count {
        match components[0]
            .1
            .next()
            .await
            .expect("pkt_sender should have sent a packet")
        {
            MsgProcessed::Send(peer, data) => {
                assert_eq!(peer, PEERS[1].public().into());
                components[1]
                    .1
                    .writer()
                    .receive_packet(data, PEERS[0].public().into())
                    .expect("Send to relayer should succeed")
            }
            _ => panic!("Should have gotten a send request"),
        }
    }

    for i in 1..components.len() {
        for _ in 0..pending_packet_count {
            match components[i]
                .1
                .next()
                .await
                .expect("MSG relayer should forward a msg to the next")
            {
                MsgProcessed::Forward(peer, data, previous_peer, ack) => {
                    assert_eq!(peer, PEERS[i + 1].public().into());
                    assert_eq!(previous_peer, PEERS[i - 1].public().into());
                    assert_ne!(
                        i,
                        component_length - 1,
                        "Only intermediate peers can serve as a forwarder"
                    );
                    components[i + 1]
                        .1
                        .writer()
                        .receive_packet(data, PEERS[i].public().into())
                        .expect("Send of ack from relayer to receiver should succeed");
                    assert!(components[i - 1]
                        .0
                        .writer()
                        .receive_acknowledgement(PEERS[i].public().into(), ack)
                        .is_ok());
                }
                MsgProcessed::Receive(_peer, packet, ack) => {
                    received_packets.push(packet);
                    assert_eq!(i, component_length - 1, "Only the last peer can be a recipient");
                    assert!(components[i - 1]
                        .0
                        .writer()
                        .receive_acknowledgement(PEERS[i].public().into(), ack)
                        .is_ok());
                }
                _ => panic!("Should have gotten a send request or a final packet"),
            }

            match components[i - 1]
                .0
                .next()
                .await
                .expect("MSG relayer should forward a msg to the next")
            {
                AckProcessed::Receive(peer, reply) => {
                    assert_eq!(peer, PEERS[i].public().into());

                    match reply {
                        AckResult::Sender(_hkc) => {
                            assert_eq!(i - 1, 0, "Only the sender can receive a half key challenge");
                            received_challenges += 1;
                        }
                        AckResult::RelayerWinning(tkt) => {
                            // choose the last relayer before the receiver
                            if i - 1 == components.len() - 2 {
                                received_tickets.push(tkt)
                            }
                        }
                        AckResult::RelayerLosing => {
                            assert!(false);
                        }
                    }
                }
                _ => panic!("Should have gotten a send request or a final packet"),
            }
        }
    }

    (received_packets, received_challenges, received_tickets)
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

async fn resolve_mock_path(me: Address, peers_offchain: Vec<PeerId>, peers_onchain: Vec<Address>) -> TransportPath {
    let peers_addrs = peers_offchain
        .iter()
        .zip(peers_onchain)
        .map(|(peer_id, addr)| (OffchainPublicKey::try_from(peer_id).unwrap(), addr))
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
    TransportPath::resolve(peers_offchain, &TestResolver(peers_addrs), &cg)
        .await
        .unwrap()
        .0
}

async fn packet_relayer_workflow_n_peers(peer_count: usize, pending_packets: usize) {
    assert!(peer_count >= 3, "invalid peer count given");
    assert!(pending_packets >= 1, "at least one packet must be given");

    const TIMEOUT_SECONDS: u64 = 10;

    let test_msgs = (0..pending_packets)
        .map(|i| ApplicationData {
            application_tag: (i == 0).then(|| random_integer(1, Some(65535)) as Tag),
            plain_text: random_bytes::<300>().into(),
        })
        .collect::<Vec<_>>();

    let components = peer_setup_for(peer_count).await;

    // Peer 1: start sending out packets
    let packet_path = resolve_mock_path(
        PEERS_CHAIN[0].public().to_address(),
        PEERS[1..peer_count].iter().map(|p| p.public().into()).collect(),
        PEERS_CHAIN[1..peer_count]
            .iter()
            .map(|key| key.public().to_address())
            .collect(),
    )
    .await;
    assert_eq!(peer_count - 1, packet_path.length() as usize, "path has invalid length");

    let mut packet_challenges = Vec::new();
    for i in 0..pending_packets {
        let awaiter = components[0]
            .1
            .writer()
            .send_packet(test_msgs[i].clone(), packet_path.clone())
            .expect("Packet should be sent successfully");
        let challenge = awaiter.rx.unwrap().await.expect("missing packet send challenge");
        packet_challenges.push(challenge);
    }

    let channel = emulate_channel_communication(pending_packets, components);
    let timeout = async_std::task::sleep(Duration::from_secs(TIMEOUT_SECONDS));
    pin_mut!(channel, timeout);

    let succeeded = match select(channel, timeout).await {
        Either::Left(((pkts, acks, ack_tkts), _)) => {
            assert_eq!(pkts.len(), pending_packets, "did not receive all packets");
            assert!(
                test_msgs.iter().all(|m| pkts.contains(m)),
                "some received packet data does not match"
            );

            assert_eq!(acks, pending_packets, "did not receive all acknowledgements");

            assert_eq!(
                ack_tkts.len(),
                pending_packets,
                "did not receive all acknowledgement tickets"
            );

            true
        }
        Either::Right(_) => false,
    };

    assert!(succeeded, "test timed out after {TIMEOUT_SECONDS} seconds");
}

//     #[serial]
//     #[async_std::test]
//     // #[tracing_test::traced_test]
//     async fn test_packet_relayer_workflow_3_peers() {
//         packet_relayer_workflow_n_peers(3, 5).await;
//     }

//     #[serial]
//     #[async_std::test]
//     // #[tracing_test::traced_test]
//     async fn test_packet_relayer_workflow_5_peers() {
//         packet_relayer_workflow_n_peers(5, 5).await;
//     }
