use std::{pin::Pin, sync::Arc};

use async_lock::RwLock;
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::future::{poll_fn, Either};
use futures::{pin_mut, stream::Stream, StreamExt};
use libp2p_identity::PeerId;
use rust_stream_ext_concurrent::then_concurrent::StreamThenConcurrentExt;
use tracing::{debug, error, warn};

use core_path::path::{Path, TransportPath};
use hopr_crypto_packet::errors::{
    PacketError::{Retry, TagReplay, TransportError},
    Result,
};
use hopr_crypto_types::prelude::*;
use hopr_db_api::prelude::HoprDbProtocolOperations;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

use super::packet::{PacketConstructing, TransportPacket};
use crate::executor::{sleep, spawn};
use crate::msg::mixer::MixerConfig;

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::{MultiCounter, SimpleCounter, SimpleGauge, SimpleHistogram};

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
    static ref METRIC_RELAYED_PACKET_IN_MIXER_TIME: SimpleHistogram = SimpleHistogram::new(
        "hopr_relayed_packet_processing_time_with_mixing_sec",
        "Histogram of measured processing and mixing time for a relayed packet in seconds",
        vec![0.01, 0.025, 0.050, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    ).unwrap();
}

lazy_static::lazy_static! {
    /// Fixed price per packet to 0.01 HOPR
    static ref DEFAULT_PRICE_PER_PACKET: U256 = 10000000000000000u128.into();
}

// Default sizes of the packet queues
pub const PACKET_TX_QUEUE_SIZE: usize = 8192;
pub const PACKET_RX_QUEUE_SIZE: usize = 8192;

pub enum MsgToProcess {
    ToReceive(Box<[u8]>, PeerId),
    ToSend(ApplicationData, TransportPath, PacketSendFinalizer),
    ToForward(Box<[u8]>, PeerId),
}

// Custom implementation of Debug used by tracing, the data content
// itself should not be displayed for any case.
impl std::fmt::Debug for MsgToProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ToReceive(_, peer) => f.debug_tuple("ToReceive").field(peer).finish(),
            Self::ToSend(_, path, _) => f.debug_tuple("ToSend").field(path).finish(),
            Self::ToForward(_, peer) => f.debug_tuple("ToForward").field(peer).finish(),
        }
    }
}

pub enum MsgProcessed {
    Receive(PeerId, ApplicationData, Acknowledgement),
    Send(PeerId, Box<[u8]>),
    Forward(PeerId, Box<[u8]>, PeerId, Acknowledgement),
}

// Custom implementation of Debug used by tracing, the data content
// itself should not be displayed for any case.
impl std::fmt::Debug for MsgProcessed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Receive(peer, _, ack) => f.debug_tuple("Receive").field(peer).field(ack).finish(),
            Self::Send(peer, _) => f.debug_tuple("Send").field(peer).finish(),
            Self::Forward(source_peer, _, dest_peer, ack) => f
                .debug_tuple("Forward")
                .field(source_peer)
                .field(dest_peer)
                .field(ack)
                .finish(),
        }
    }
}

/// Implements protocol acknowledgement logic for msg packets
#[derive(Debug, Clone)]
pub struct PacketProcessor<Db>
where
    Db: HoprDbProtocolOperations + Send + Sync + std::fmt::Debug + Clone,
{
    db: Db,
    cfg: PacketInteractionConfig,
}

#[async_trait::async_trait]
impl<Db> crate::msg::packet::PacketConstructing for PacketProcessor<Db>
where
    Db: HoprDbProtocolOperations + Send + Sync + std::fmt::Debug + Clone,
{
    type Input = ApplicationData;
    type Packet = TransportPacket;

    async fn to_send(&self, data: Self::Input, path: Vec<OffchainPublicKey>) -> Result<Self::Packet> {
        Ok(self
            .db
            .to_send(data.to_bytes(), self.cfg.chain_keypair.clone(), path.clone())
            .await
            .map_err(|e| hopr_crypto_packet::errors::PacketError::PacketConstructionError(e.to_string()))?
            .into())
    }

    async fn from_recv(
        &self,
        data: Box<[u8]>,
        pkt_keypair: &OffchainKeypair,
        sender: OffchainPublicKey,
    ) -> Result<Self::Packet> {
        match self
            .db
            .from_recv(data, self.cfg.chain_keypair.clone(), pkt_keypair, sender)
            .await
        {
            Ok(v) => Ok(v.into()),
            Err(e) => {
                #[cfg(all(feature = "prometheus", not(test)))]
                if let hopr_db_api::errors::DbError::TicketValidationError(_) = e {
                    METRIC_REJECTED_TICKETS_COUNT.increment();
                }

                Err(hopr_crypto_packet::errors::PacketError::PacketConstructionError(
                    e.to_string(),
                ))
            }
        }
    }
}

impl<Db> PacketProcessor<Db>
where
    Db: HoprDbProtocolOperations + Send + Sync + std::fmt::Debug + Clone,
{
    /// Creates a new instance given the DB and configuration.
    pub fn new(db: Db, cfg: PacketInteractionConfig) -> Self {
        Self { db, cfg }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn to_transport_packet_with_metadata(
        &self,
        event: MsgToProcess,
    ) -> (Result<TransportPacket>, PacketMetadata) {
        let mut metadata = PacketMetadata::default();

        let packet = match event {
            MsgToProcess::ToReceive(data, peer) | MsgToProcess::ToForward(data, peer) => {
                let previous_hop = OffchainPublicKey::try_from(&peer).map_err(|e| {
                    hopr_crypto_packet::errors::PacketError::LogicError(format!(
                        "failed to convert '{peer}' into the public key {e}"
                    ))
                });

                match previous_hop {
                    Ok(previous_hop) => self.from_recv(data, &self.cfg.packet_keypair, previous_hop).await,
                    Err(e) => Err(e),
                }
            }
            MsgToProcess::ToSend(data, path, finalizer) => {
                metadata.send_finalizer.replace(finalizer);

                let path: std::result::Result<Vec<OffchainPublicKey>, hopr_primitive_types::errors::GeneralError> =
                    path.hops().iter().map(OffchainPublicKey::try_from).collect();
                match path {
                    Ok(path) => self.to_send(data, path).await,
                    Err(e) => Err(hopr_crypto_packet::errors::PacketError::PacketConstructionError(
                        e.to_string(),
                    )),
                }
            }
        };

        (packet, metadata)
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
    tx: Option<futures::channel::oneshot::Sender<HalfKeyChallenge>>,
}

impl PacketSendFinalizer {
    pub fn new(tx: futures::channel::oneshot::Sender<HalfKeyChallenge>) -> Self {
        Self { tx: Some(tx) }
    }

    pub fn finalize(mut self, challenge: HalfKeyChallenge) {
        if let Some(sender) = self.tx.take() {
            match sender.send(challenge) {
                Ok(_) => {}
                Err(_) => {
                    error!("Failed to notify the awaiter about the successful packet transmission")
                }
            }
        } else {
            error!("Sender for packet send signalization is already spent")
        }
    }
}

/// Await on future until the confirmation of packet reception is received
#[derive(Debug)]
pub struct PacketSendAwaiter {
    rx: Option<futures::channel::oneshot::Receiver<HalfKeyChallenge>>,
}

impl From<futures::channel::oneshot::Receiver<HalfKeyChallenge>> for PacketSendAwaiter {
    fn from(value: futures::channel::oneshot::Receiver<HalfKeyChallenge>) -> Self {
        Self { rx: Some(value) }
    }
}

impl PacketSendAwaiter {
    pub async fn consume_and_wait(&mut self, until_timeout: std::time::Duration) -> Result<HalfKeyChallenge> {
        match self.rx.take() {
            Some(resolve) => {
                let timeout = sleep(until_timeout);
                pin_mut!(resolve, timeout);
                match futures::future::select(resolve, timeout).await {
                    Either::Left((challenge, _)) => challenge.map_err(|_| TransportError("Canceled".to_owned())),
                    Either::Right(_) => Err(TransportError("Timed out on sending a packet".to_owned())),
                }
            }
            None => Err(TransportError(
                "Packet send process observation already consumed".to_owned(),
            )),
        }
    }
}

/// External API for feeding Packet actions into the Packet processor
#[derive(Debug, Clone)]
pub struct PacketActions {
    pub queue: Sender<MsgToProcess>,
}

/// Pushes the packet with the given payload for sending via the given valid path.
impl PacketActions {
    /// Pushes a new packet from this node into processing.
    pub fn send_packet(&mut self, data: ApplicationData, path: TransportPath) -> Result<PacketSendAwaiter> {
        let (tx, rx) = futures::channel::oneshot::channel::<HalfKeyChallenge>();

        self.process(MsgToProcess::ToSend(data, path, PacketSendFinalizer::new(tx)))
            .map(move |_| {
                let awaiter: PacketSendAwaiter = rx.into();
                awaiter
            })
    }

    /// Pushes the packet received from the transport layer into processing.
    pub fn receive_packet(&mut self, payload: Box<[u8]>, source: PeerId) -> Result<()> {
        self.process(MsgToProcess::ToReceive(payload, source))
    }

    fn process(&mut self, event: MsgToProcess) -> Result<()> {
        self.queue.try_send(event).map_err(|e| {
            if e.is_full() {
                Retry
            } else if e.is_disconnected() {
                TransportError("queue is closed".to_string())
            } else {
                TransportError(format!("Unknown error: {}", e))
            }
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
    #[default(None)]
    pub send_finalizer: Option<PacketSendFinalizer>,
    #[cfg(all(feature = "prometheus", not(test)))]
    #[default(std::time::UNIX_EPOCH)]
    pub start_time: std::time::SystemTime,
}

/// Sets up processing of packet interactions and returns relevant read and write mechanism.
///
/// Packet processing logic:
/// * When a new packet is delivered from the transport the `receive_packet` method is used
/// to push it into the processing queue of incoming packets.
/// * When a new packet is delivered from the transport and is designated for forwarding,
/// the `forward_packet` method is used.
/// * When a packet is generated to be sent over the network the `send_packet` is used to
/// push it into the processing queue.
///
/// The result of packet processing can be extracted as a stream.
pub struct PacketInteraction {
    msg_event_queue: (Sender<MsgToProcess>, Receiver<MsgProcessed>),
}

impl PacketInteraction {
    /// Creates a new instance given the DB and our public key used to verify the acknowledgements.
    pub fn new<Db>(db: Db, tbf: Arc<RwLock<TagBloomFilter>>, cfg: PacketInteractionConfig) -> Self
    where
        Db: HoprDbProtocolOperations + Send + Sync + std::fmt::Debug + Clone + 'static,
    {
        let (to_process_tx, to_process_rx) = channel::<MsgToProcess>(PACKET_RX_QUEUE_SIZE + PACKET_TX_QUEUE_SIZE);
        let (processed_tx, processed_rx) = channel::<MsgProcessed>(PACKET_RX_QUEUE_SIZE + PACKET_TX_QUEUE_SIZE);

        let mixer_cfg = cfg.mixer;
        let processor = PacketProcessor::new(db, cfg);

        let mut processing_stream = to_process_rx
            .then_concurrent(move |event| {
                let processor = processor.clone();

                async move {
                    #[cfg_attr(not(all(feature = "prometheus", not(test))), allow(unused_mut))]
                    let (packet, mut metadata) = processor.to_transport_packet_with_metadata(event).await;

                    #[cfg(all(feature = "prometheus", not(test)))]
                    if let Ok(TransportPacket::Forwarded { .. }) = &packet {
                        metadata.start_time = hopr_platform::time::native::current_time();
                    }

                    (packet, metadata)
                }
            })
            // check tag replay
            .then_concurrent(move |(packet, metadata)| {
                let tbf = tbf.clone();

                async move {
                    tracing::debug!("tbf: check tag replay");

                    if let Ok(p) = &packet {
                        let packet_tag = match p {
                            TransportPacket::Final { packet_tag, .. } => Some(packet_tag),
                            TransportPacket::Forwarded { packet_tag, .. } => Some(packet_tag),
                            _ => None,
                        };

                        if let Some(tag) = packet_tag {
                            // There is a 0.1% chance that the positive result is not a replay
                            // because a Bloom filter is used
                            if tbf.write().await.check_and_set(tag) {
                                return (Err(TagReplay), metadata);
                            }
                        }
                    };

                    (packet, metadata)
                }
            })
            // process packet operation
            .then_concurrent(move |(packet, mut metadata)| async move {
                match packet {
                    Err(e) => Err(e),
                    Ok(packet) => match packet {
                        TransportPacket::Outgoing {
                            next_hop,
                            ack_challenge,
                            data,
                        } => {
                            #[cfg(all(feature = "prometheus", not(test)))]
                            {
                                METRIC_PACKET_COUNT_PER_PEER.increment(&["out", &next_hop.to_string()]);
                                METRIC_PACKET_COUNT.increment(&["sent"]);
                            }

                            if let Some(finalizer) = metadata.send_finalizer.take() {
                                finalizer.finalize(ack_challenge);
                            }
                            Ok((MsgProcessed::Send(next_hop, data), metadata))
                        }

                        TransportPacket::Final {
                            previous_hop,
                            plain_text,
                            ack,
                            ..
                        } => match ApplicationData::from_bytes(plain_text.as_ref()) {
                            Ok(app_data) => {
                                #[cfg(all(feature = "prometheus", not(test)))]
                                {
                                    METRIC_PACKET_COUNT_PER_PEER.increment(&["in", &previous_hop.to_string()]);
                                    METRIC_PACKET_COUNT.increment(&["received"]);
                                }

                                Ok((MsgProcessed::Receive(previous_hop, app_data, ack), metadata))
                            }
                            Err(e) => Err(e.into()),
                        },

                        TransportPacket::Forwarded {
                            previous_hop,
                            next_hop,
                            data,
                            ack,
                            ..
                        } => {
                            #[cfg(all(feature = "prometheus", not(test)))]
                            {
                                METRIC_PACKET_COUNT_PER_PEER.increment(&["in", &previous_hop.to_string()]);
                                METRIC_PACKET_COUNT_PER_PEER.increment(&["out", &next_hop.to_string()]);
                                METRIC_PACKET_COUNT.increment(&["forwarded"]);
                            }

                            Ok((MsgProcessed::Forward(next_hop, data, previous_hop, ack), metadata))
                        }
                    },
                }
            })
            // introduce random timeout to mix packets asynchrounously
            .then_concurrent(move |event| async move {
                match event {
                    Ok((processed, metadata)) => match processed {
                        MsgProcessed::Send(..) | MsgProcessed::Forward(..) => {
                            let random_delay = mixer_cfg.random_delay();
                            debug!("Mixer created a random packet delay {}ms", random_delay.as_millis());

                            #[cfg(all(feature = "prometheus", not(test)))]
                            METRIC_QUEUE_SIZE.increment(1.0f64);

                            sleep(random_delay).await;

                            #[cfg(all(feature = "prometheus", not(test)))]
                            {
                                METRIC_QUEUE_SIZE.decrement(1.0f64);

                                let weight = 1.0f64 / mixer_cfg.metric_delay_window as f64;
                                METRIC_MIXER_AVERAGE_DELAY.set(
                                    (weight * random_delay.as_millis() as f64)
                                        + ((1.0f64 - weight) * METRIC_MIXER_AVERAGE_DELAY.get()),
                                );
                            }

                            Ok((processed, metadata))
                        }
                        MsgProcessed::Receive(..) => Ok((processed, metadata)),
                    },
                    Err(e) => Err(e),
                }
            })
            // output processed packet into the event mechanism
            .then_concurrent(move |processed| {
                let mut processed_tx = processed_tx.clone();

                async move {
                    match processed {
                        Ok((processed_msg, _metadata)) => {
                            #[cfg(all(feature = "prometheus", not(test)))]
                            if let MsgProcessed::Forward(_, _, _, _) = &processed_msg {
                                METRIC_RELAYED_PACKET_IN_MIXER_TIME.observe(
                                    hopr_platform::time::native::current_time()
                                        .saturating_sub(_metadata.start_time)
                                        .as_secs_f64(),
                                )
                            };

                            match poll_fn(|cx| Pin::new(&mut processed_tx).poll_ready(cx)).await {
                                Ok(_) => match processed_tx.start_send(processed_msg) {
                                    Ok(_) => debug!("Pipeline resulted in a processed msg"),
                                    Err(e) => error!("Failed to pass a processed ack message: {}", e),
                                },
                                Err(e) => {
                                    warn!("The receiver for processed packets no longer exists: {}", e);
                                }
                            };
                        }
                        Err(e) => error!("Packet processing error: {}", e),
                    }
                }
            });

        // NOTE: This spawned task does not need to be explicitly canceled, since it will
        // be automatically dropped when the event sender object is dropped.
        spawn(async move {
            // poll the stream until it's done
            while processing_stream.next().await.is_some() {}
        });

        Self {
            msg_event_queue: (to_process_tx, processed_rx),
        }
    }

    pub fn writer(&self) -> PacketActions {
        PacketActions {
            queue: self.msg_event_queue.0.clone(),
        }
    }
}

impl Stream for PacketInteraction {
    type Item = MsgProcessed;

    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        use futures::stream::StreamExt;
        Pin::new(self).msg_event_queue.1.poll_next_unpin(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::{ApplicationData, MsgProcessed, PacketInteraction, PacketInteractionConfig, DEFAULT_PRICE_PER_PACKET};
    use crate::{
        ack::processor::{AckProcessed, AckResult, AcknowledgementInteraction},
        msg::mixer::MixerConfig,
    };
    use async_lock::RwLock;
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
    use std::{str::FromStr, sync::Arc, time::Duration};
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
    pub async fn test_packet_send_finalizer_succeeds_with_a_stored_challenge() {
        let (tx, rx) = futures::channel::oneshot::channel::<HalfKeyChallenge>();

        let finalizer = super::PacketSendFinalizer::new(tx);
        let challenge = HalfKeyChallenge::default();
        let mut awaiter: super::PacketSendAwaiter = rx.into();

        finalizer.finalize(challenge);

        let result = awaiter.consume_and_wait(Duration::from_millis(20)).await;

        assert_eq!(challenge, result.expect("HalfKeyChallange should be transmitted"));
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
                    Arc::new(RwLock::new(TagBloomFilter::default())),
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
    ) -> (Vec<ApplicationData>, Vec<HalfKeyChallenge>, Vec<AcknowledgedTicket>) {
        let component_length = components.len();
        let mut received_packets: Vec<ApplicationData> = vec![];
        let mut received_challenges: Vec<HalfKeyChallenge> = vec![];
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
                        assert!(reply.is_ok());

                        match reply.unwrap() {
                            AckResult::Sender(hkc) => {
                                assert_eq!(i - 1, 0, "Only the sender can receive a half key challenge");
                                received_challenges.push(hkc);
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

                assert_eq!(acks.len(), pending_packets, "did not receive all acknowledgements");
                assert!(
                    packet_challenges.iter().all(|c| acks.contains(c)),
                    "received some unknown acknowledgement"
                );

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
