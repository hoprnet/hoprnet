use futures::pin_mut;
use futures::{future::Either, SinkExt};
use hopr_db_api::protocol::TransportPacketWithChainData;
use libp2p_identity::PeerId;
use tracing::{debug, error};

use core_path::path::{Path, TransportPath};
use hopr_async_runtime::prelude::sleep;
use hopr_crypto_packet::errors::{
    PacketError::{TagReplay, TransportError},
    Result,
};
use hopr_crypto_types::prelude::*;
use hopr_db_api::prelude::HoprDbProtocolOperations;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

use super::packet::OutgoingPacket;
use crate::bloom;
use crate::msg::mixer::MixerConfig;

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::{MultiCounter, SimpleCounter, SimpleGauge};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    // packet processing
    static ref METRIC_PACKET_COUNT: MultiCounter =
        MultiCounter::new(
        "hopr_packets_count",
        "Number of processed packets of different types (sent, received, forwarded)",
        &["type"]
    ).unwrap();
    static ref METRIC_PACKET_COUNT_PER_PEER: MultiCounter =
        MultiCounter::new(
        "hopr_packets_per_peer_count",
        "Number of processed packets to/from distinct peers",
        &["peer", "direction"]
    ).unwrap();
    static ref METRIC_REJECTED_TICKETS_COUNT: SimpleCounter =
        SimpleCounter::new("hopr_rejected_tickets_count", "Number of rejected tickets").unwrap();
    // mixer
    static ref METRIC_QUEUE_SIZE: SimpleGauge =
        SimpleGauge::new("hopr_mixer_queue_size", "Current mixer queue size").unwrap();
    static ref METRIC_MIXER_AVERAGE_DELAY: SimpleGauge = SimpleGauge::new(
        "hopr_mixer_average_packet_delay",
        "Average mixer packet delay averaged over a packet window"
    )
    .unwrap();
    static ref METRIC_REPLAYED_PACKET_COUNT: SimpleCounter = SimpleCounter::new(
        "hopr_replayed_packet_count",
        "The total count of replayed packets during the packet processing pipeline run",
    ).unwrap();
}

lazy_static::lazy_static! {
    /// Fixed price per packet to 0.01 HOPR
    static ref DEFAULT_PRICE_PER_PACKET: U256 = 10000000000000000u128.into();
}

#[async_trait::async_trait]
pub trait PacketWrapping {
    type Input;

    async fn send(&self, data: ApplicationData, path: TransportPath) -> Result<(PeerId, Box<[u8]>)>;
}

pub struct SendPkt {
    pub peer: PeerId,
    pub data: Box<[u8]>,
}

pub struct SendAck {
    pub peer: PeerId,
    pub ack: Acknowledgement,
}

pub enum RecvOperation {
    Receive { data: ApplicationData, ack: SendAck },
    Forward { msg: SendPkt, ack: SendAck },
}

#[async_trait::async_trait]
pub trait PacketUnwrapping {
    type Packet;

    async fn recv(&self, peer: &PeerId, data: Box<[u8]>) -> Result<Self::Packet>;
}

/// Implements protocol acknowledgement logic for msg packets
#[derive(Debug, Clone)]
pub struct PacketProcessor<Db>
where
    Db: HoprDbProtocolOperations + Send + Sync + std::fmt::Debug + Clone,
{
    db: Db,
    tbf: bloom::WrappedTagBloomFilter,
    cfg: PacketInteractionConfig,
}

#[async_trait::async_trait]
impl<Db> PacketWrapping for PacketProcessor<Db>
where
    Db: HoprDbProtocolOperations + Send + Sync + std::fmt::Debug + Clone,
{
    type Input = ApplicationData;

    #[tracing::instrument(level = "debug", skip(self, data))]
    async fn send(&self, data: ApplicationData, path: TransportPath) -> Result<(PeerId, Box<[u8]>)> {
        let path: std::result::Result<Vec<OffchainPublicKey>, hopr_primitive_types::errors::GeneralError> =
            path.hops().iter().map(OffchainPublicKey::try_from).collect();

        let packet = self
            .db
            .to_send(data.to_bytes(), self.cfg.chain_keypair.clone(), path?)
            .await
            .map_err(|e| hopr_crypto_packet::errors::PacketError::PacketConstructionError(e.to_string()))?;

        let packet: OutgoingPacket = packet.try_into().map_err(|e: crate::errors::ProtocolError| {
            hopr_crypto_packet::errors::PacketError::LogicError(e.to_string())
        })?;

        self.add_mixing_delay().await;

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            METRIC_PACKET_COUNT_PER_PEER.increment(&["out", &packet.next_hop.to_string()]);
            METRIC_PACKET_COUNT.increment(&["sent"]);
        }

        Ok((packet.next_hop, packet.data))
    }
}

#[async_trait::async_trait]
impl<Db> PacketUnwrapping for PacketProcessor<Db>
where
    Db: HoprDbProtocolOperations + Send + Sync + std::fmt::Debug + Clone,
{
    type Packet = RecvOperation;

    #[tracing::instrument(level = "debug", skip(self, data))]
    async fn recv(&self, peer: &PeerId, data: Box<[u8]>) -> Result<RecvOperation> {
        #[cfg(all(feature = "prometheus", not(test)))]
        let mut metadata = PacketMetadata::default();

        let previous_hop = OffchainPublicKey::try_from(peer).map_err(|e| {
            hopr_crypto_packet::errors::PacketError::LogicError(format!(
                "failed to convert '{peer}' into the public key: {e}"
            ))
        })?;

        let packet = self
            .db
            .from_recv(
                data,
                self.cfg.chain_keypair.clone(),
                &self.cfg.packet_keypair,
                previous_hop,
            )
            .await
            .map_err(|e| {
                #[cfg(all(feature = "prometheus", not(test)))]
                if let hopr_db_api::errors::DbError::TicketValidationError(_) = e {
                    METRIC_REJECTED_TICKETS_COUNT.increment();
                }

                hopr_crypto_packet::errors::PacketError::PacketConstructionError(e.to_string())
            })?;

        if let TransportPacketWithChainData::Final { packet_tag, .. }
        | TransportPacketWithChainData::Forwarded { packet_tag, .. } = &packet
        {
            self.is_tag_replay(packet_tag).await.then_some(()).ok_or(TagReplay)?;
        };

        Ok(match packet {
            TransportPacketWithChainData::Final {
                previous_hop,
                plain_text,
                ack,
                ..
            } => {
                let app_data = ApplicationData::from_bytes(plain_text.as_ref())?;
                #[cfg(all(feature = "prometheus", not(test)))]
                {
                    METRIC_PACKET_COUNT_PER_PEER.increment(&["in", &previous_hop.to_string()]);
                    METRIC_PACKET_COUNT.increment(&["received"]);
                }
                RecvOperation::Receive {
                    data: app_data,
                    ack: SendAck {
                        peer: previous_hop.into(),
                        ack,
                    },
                }
            }
            TransportPacketWithChainData::Forwarded {
                previous_hop,
                next_hop,
                data,
                ack,
                ..
            } => {
                #[cfg(all(feature = "prometheus", not(test)))]
                {
                    metadata.start_time = hopr_platform::time::native::current_time();
                    METRIC_PACKET_COUNT_PER_PEER.increment(&["in", &previous_hop.to_string()]);
                    METRIC_PACKET_COUNT_PER_PEER.increment(&["out", &next_hop.to_string()]);
                    METRIC_PACKET_COUNT.increment(&["forwarded"]);
                }

                RecvOperation::Forward {
                    msg: SendPkt {
                        peer: next_hop.into(),
                        data,
                    },
                    ack: SendAck {
                        peer: previous_hop.into(),
                        ack,
                    },
                }
            }
            TransportPacketWithChainData::Outgoing { .. } => {
                return Err(hopr_crypto_packet::errors::PacketError::LogicError(
                    "Attempting to process an outgoing packet in the incoming pipeline".into(),
                ))
            }
        })
    }
}

impl<Db> PacketProcessor<Db>
where
    Db: HoprDbProtocolOperations + Send + Sync + std::fmt::Debug + Clone,
{
    /// Creates a new instance given the DB and configuration.
    pub fn new(db: Db, tbf: bloom::WrappedTagBloomFilter, cfg: PacketInteractionConfig) -> Self {
        Self { db, tbf, cfg }
    }

    #[tracing::instrument(level = "trace", name = "check_tag_replay", skip(self, tag))]
    /// Check whether the packet is replayed using a packet tag.
    ///
    /// There is a 0.1% chance that the positive result is not a replay because a Bloom filter is used.
    pub async fn is_tag_replay(&self, tag: &PacketTag) -> bool {
        let is_replay_attempt = self
            .tbf
            .with_write_lock(|inner: &mut TagBloomFilter| inner.check_and_set(tag))
            .await;

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            METRIC_REPLAYED_PACKET_COUNT.increment();
        }

        is_replay_attempt
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn add_mixing_delay(&self) {
        let random_delay = self.cfg.mixer.random_delay();
        debug!(
            delay_in_ms = random_delay.as_millis(),
            "Mixer created a random packet delay",
        );

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_QUEUE_SIZE.increment(1.0f64);

        sleep(random_delay).await;

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            METRIC_QUEUE_SIZE.decrement(1.0f64);

            let weight = 1.0f64 / self.cfg.mixer.metric_delay_window as f64;
            METRIC_MIXER_AVERAGE_DELAY.set(
                (weight * random_delay.as_millis() as f64) + ((1.0f64 - weight) * METRIC_MIXER_AVERAGE_DELAY.get()),
            );
        }
    }
}

/// Packet send finalizer notifying the awaiting future once the send has been acknowledged.
///
/// This is a remnant of the original logic that assumed that the p2p transport is invokable
/// and its result can be directly polled. As the `send_packet` logic is the only part visible
/// outside the communication loop from the protocol side, it is retained pending a larger
/// architectural overhaul of the hopr daemon.
#[derive(Debug)]
pub struct PacketSendFinalizer {
    tx: futures::channel::oneshot::Sender<()>,
}

impl PacketSendFinalizer {
    pub fn new(tx: futures::channel::oneshot::Sender<()>) -> Self {
        Self { tx }
    }

    pub fn finalize(self) {
        if self.tx.send(()).is_err() {
            error!("Failed to notify the awaiter about the successful packet transmission")
        }
    }
}

/// Await on future until the confirmation of packet reception is received
#[derive(Debug)]
pub struct PacketSendAwaiter {
    rx: futures::channel::oneshot::Receiver<()>,
}

impl From<futures::channel::oneshot::Receiver<()>> for PacketSendAwaiter {
    fn from(value: futures::channel::oneshot::Receiver<()>) -> Self {
        Self { rx: value }
    }
}

impl PacketSendAwaiter {
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn consume_and_wait(self, until_timeout: std::time::Duration) -> Result<()> {
        let timeout = sleep(until_timeout);
        let rx = self.rx;
        pin_mut!(rx, timeout);
        match futures::future::select(rx, timeout).await {
            Either::Left((challenge, _)) => challenge.map_err(|_| TransportError("Canceled".to_owned())),
            Either::Right(_) => Err(TransportError("Timed out on sending a packet".to_owned())),
        }
    }
}

#[derive(Debug)]
pub struct MsgSender {
    tx: futures::channel::mpsc::UnboundedSender<(ApplicationData, TransportPath, PacketSendFinalizer)>,
}

impl MsgSender {
    pub fn new(
        tx: futures::channel::mpsc::UnboundedSender<(ApplicationData, TransportPath, PacketSendFinalizer)>,
    ) -> Self {
        Self { tx }
    }

    /// Pushes a new packet into processing.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn send_packet(&self, data: ApplicationData, path: TransportPath) -> Result<PacketSendAwaiter> {
        let (tx, rx) = futures::channel::oneshot::channel::<()>();

        self.tx
            .clone()
            .send((data, path, PacketSendFinalizer::new(tx)))
            .await
            .map_err(|_| TransportError("Failed to send a message".to_owned()))
            .map(move |_| {
                let awaiter: PacketSendAwaiter = rx.into();
                awaiter
            })
    }
}

/// Configuration parameters for the packet interaction.
#[derive(Clone, Debug)]
pub struct PacketInteractionConfig {
    pub check_unrealized_balance: bool,
    pub packet_keypair: OffchainKeypair,
    pub chain_keypair: ChainKeypair,
    pub mixer: MixerConfig,
}

impl PacketInteractionConfig {
    pub fn new(packet_keypair: &OffchainKeypair, chain_keypair: &ChainKeypair) -> Self {
        Self {
            packet_keypair: packet_keypair.clone(),
            chain_keypair: chain_keypair.clone(),
            check_unrealized_balance: true,
            mixer: MixerConfig::default(),
        }
    }
}

#[derive(Debug, smart_default::SmartDefault)]
pub struct PacketMetadata {
    #[cfg(all(feature = "prometheus", not(test)))]
    #[default(std::time::UNIX_EPOCH)]
    pub start_time: std::time::SystemTime,
}

#[cfg(test)]
mod tests {
    use super::{ApplicationData, MsgProcessed, PacketInteraction, PacketInteractionConfig, DEFAULT_PRICE_PER_PACKET};
    use crate::{
        ack::processor::{AckProcessed, AckResult, AcknowledgementInteraction},
        bloom::WrappedTagBloomFilter,
        msg::mixer::MixerConfig,
    };

    use async_trait::async_trait;
    use core_path::channel_graph::ChannelGraph;
    use core_path::path::{Path, TransportPath};
    use futures::{
        future::{select, Either},
        pin_mut, StreamExt,
    };
    use hex_literal::hex;
    use hopr_crypto_random::{random_bytes, random_integer};
    use hopr_crypto_types::prelude::*;
    use hopr_db_api::{info::DomainSeparator, resolver::HoprDbResolverOperations};
    use hopr_db_sql::{
        accounts::HoprDbAccountOperations, channels::HoprDbChannelOperations, db::HoprDb, info::HoprDbInfoOperations,
    };
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use lazy_static::lazy_static;
    use libp2p::Multiaddr;
    use libp2p_identity::PeerId;
    use serial_test::serial;
    use std::{str::FromStr, time::Duration};
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

    async fn create_dummy_channel(from: Address, to: Address) -> ChannelEntry {
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

    async fn create_minimal_topology(dbs: &mut Vec<HoprDb>) -> crate::errors::Result<()> {
        let mut previous_channel: Option<ChannelEntry> = None;

        for index in 0..dbs.len() {
            dbs[index]
                .set_domain_separator(None, DomainSeparator::Channel, Hash::default())
                .await
                .map_err(|e| crate::errors::ProtocolError::Logic(e.to_string()))?;

            dbs[index]
                .update_ticket_price(None, Balance::new(100u128, BalanceType::HOPR))
                .await
                .map_err(|e| crate::errors::ProtocolError::Logic(e.to_string()))?;

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
                    .map_err(|e| crate::errors::ProtocolError::Logic(e.to_string()))?;
            }

            let mut channel: Option<ChannelEntry> = None;
            let this_peer_chain = &PEERS_CHAIN[index];

            if index < PEERS.len() - 1 {
                channel = Some(
                    create_dummy_channel(
                        this_peer_chain.public().to_address(),
                        PEERS_CHAIN[index + 1].public().to_address(),
                    )
                    .await,
                );

                dbs[index]
                    .upsert_channel(None, channel.unwrap())
                    .await
                    .map_err(|e| crate::errors::ProtocolError::Logic(e.to_string()))?;
            }

            if index > 0 {
                dbs[index]
                    .upsert_channel(None, previous_channel.unwrap())
                    .await
                    .map_err(|e| crate::errors::ProtocolError::Logic(e.to_string()))?;
            }

            previous_channel = channel;
        }

        Ok(())
    }

    #[async_std::test]
    pub async fn test_packet_send_finalizer_succeeds() {
        let (tx, rx) = futures::channel::oneshot::channel::<()>();

        let finalizer = super::PacketSendFinalizer::new(tx);
        let mut awaiter: super::PacketSendAwaiter = rx.into();

        finalizer.finalize();

        let result = awaiter.consume_and_wait(Duration::from_millis(20)).await;

        assert!(result.is_ok());
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

    #[serial]
    #[async_std::test]
    // #[tracing_test::traced_test]
    async fn test_packet_relayer_workflow_3_peers() {
        packet_relayer_workflow_n_peers(3, 5).await;
    }

    #[serial]
    #[async_std::test]
    // #[tracing_test::traced_test]
    async fn test_packet_relayer_workflow_5_peers() {
        packet_relayer_workflow_n_peers(5, 5).await;
    }
}
