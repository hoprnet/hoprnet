use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::future::{poll_fn, Either};
use futures::{pin_mut, stream::Stream, StreamExt};
use libp2p_identity::PeerId;
use std::pin::Pin;
use std::sync::Arc;

use async_lock::RwLock;
use rust_stream_ext_concurrent::then_concurrent::StreamThenConcurrentExt;

use chain_db::traits::HoprCoreEthereumDbActions;
use core_packet::errors::PacketError::{
    self, ChannelNotFound, MissingDomainSeparator, OutOfFunds, PacketConstructionError, PacketDecodingError,
    PathPositionMismatch, Retry, TagReplay, TransportError,
};
use core_packet::errors::Result;
use core_packet::validation::validate_unacknowledged_ticket;
use core_path::path::{Path, TransportPath};
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

use tracing::{debug, error, trace, warn, Instrument};

use super::packet::{PacketConstructing, TransportPacket};
use crate::msg::{chain::ChainPacketComponents, mixer::MixerConfig};

use async_std::task::{sleep, spawn};

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
    static ref METRIC_TICKETS_COUNT: SimpleCounter =
        SimpleCounter::new("hopr_created_tickets_count", "Number of created tickets").unwrap();
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
const PACKET_TX_QUEUE_SIZE: usize = 2048;
const PACKET_RX_QUEUE_SIZE: usize = 2048;

#[derive(Debug)]
pub enum MsgToProcess {
    ToReceive(Box<[u8]>, PeerId),
    ToSend(ApplicationData, TransportPath, PacketSendFinalizer),
    ToForward(Box<[u8]>, PeerId),
}

#[derive(Debug)]
pub enum MsgProcessed {
    Receive(PeerId, ApplicationData, Acknowledgement),
    Send(PeerId, Box<[u8]>),
    Forward(PeerId, Box<[u8]>, PeerId, Acknowledgement),
}

/// Implements protocol acknowledgement logic for msg packets
#[derive(Debug)]
pub struct PacketProcessor<Db>
where
    Db: HoprCoreEthereumDbActions + std::marker::Send + std::marker::Sync + std::fmt::Debug,
{
    db: Arc<RwLock<Db>>,
    cfg: PacketInteractionConfig,
}

impl<Db> Clone for PacketProcessor<Db>
where
    Db: HoprCoreEthereumDbActions + std::marker::Send + std::marker::Sync + std::fmt::Debug,
{
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            cfg: self.cfg.clone(),
        }
    }
}

#[async_trait::async_trait]
impl<Db> crate::msg::packet::PacketConstructing for PacketProcessor<Db>
where
    Db: HoprCoreEthereumDbActions + std::marker::Send + std::marker::Sync + std::fmt::Debug,
{
    type Input = ApplicationData;

    async fn into_outgoing(&self, data: Self::Input, path: &TransportPath) -> Result<TransportPacket> {
        let next_peer = self
            .db
            .read()
            .await
            .get_chain_key(&OffchainPublicKey::try_from(path.hops()[0])?)
            .await?
            .ok_or_else(|| {
                debug!("Could not retrieve on-chain key for {}", path.hops()[0]);
                PacketConstructionError
            })?;

        let domain_separator = self
            .db
            .read()
            .await
            .get_channels_domain_separator()
            .await?
            .ok_or_else(|| {
                warn!("Missing domain separator.");
                MissingDomainSeparator
            })?;

        // Decide whether to create 0-hop or multihop ticket
        let next_ticket = if path.length() == 1 {
            Ticket::new_zero_hop(&next_peer, &self.cfg.chain_keypair, &domain_separator)?
        } else {
            self.create_multihop_ticket(next_peer, path.length() as u8).await?
        };

        match ChainPacketComponents::into_outgoing(
            &data.to_bytes(),
            path,
            &self.cfg.chain_keypair,
            next_ticket,
            &domain_separator,
        ) {
            Ok(p) => match p {
                ChainPacketComponents::Final { .. } | ChainPacketComponents::Forwarded { .. } => {
                    Err(PacketError::LogicError("Must contain an outgoing packet type".into()))
                }
                ChainPacketComponents::Outgoing {
                    packet,
                    ticket,
                    next_hop,
                    ack_challenge,
                } => {
                    self.db
                        .write()
                        .instrument(tracing::debug_span!(
                            "db: outgoing packet (store pending acknowledgement)"
                        ))
                        .await
                        .store_pending_acknowledgment(ack_challenge, PendingAcknowledgement::WaitingAsSender)
                        .await?;

                    let mut payload = Vec::with_capacity(ChainPacketComponents::SIZE);
                    payload.extend_from_slice(packet.as_ref());
                    payload.extend_from_slice(&ticket.to_bytes());

                    Ok(TransportPacket::Outgoing {
                        next_hop: next_hop.into(),
                        ack_challenge,
                        data: payload.into_boxed_slice(),
                    })
                }
            },
            Err(e) => Err(e),
        }
    }

    async fn from_incoming(
        &self,
        data: Box<[u8]>,
        pkt_keypair: &OffchainKeypair,
        sender: &PeerId,
    ) -> Result<TransportPacket> {
        let components = ChainPacketComponents::from_incoming(&data, pkt_keypair, sender)?;

        match components {
            ChainPacketComponents::Final {
                packet_tag,
                ack_key,
                previous_hop,
                plain_text,
                ..
            } => {
                let ack = Acknowledgement::new(ack_key, pkt_keypair);

                Ok(TransportPacket::Final {
                    packet_tag,
                    previous_hop: previous_hop.into(),
                    plain_text,
                    ack,
                })
            }
            ChainPacketComponents::Forwarded {
                packet,
                ticket,
                ack_challenge,
                packet_tag,
                ack_key,
                previous_hop,
                own_key,
                next_hop,
                next_challenge,
                path_pos,
            } => {
                let domain_separator =
                    self.db
                        .read()
                        .await
                        .get_channels_domain_separator()
                        .await?
                        .ok_or_else(|| {
                            warn!("Missing domain separator");
                            MissingDomainSeparator
                        })?;

                let previous_peer = previous_hop.into();
                let next_peer = next_hop.into();

                // START: channel = get_channel_from_to(packet_key, packet_key)
                let previous_hop_addr =
                    self.db
                        .read()
                        .await
                        .get_chain_key(&previous_hop)
                        .await?
                        .ok_or(PacketDecodingError(format!(
                            "failed to find channel key for packet key {previous_peer} on previous hop"
                        )))?;

                let next_hop_addr = self
                    .db
                    .read()
                    .await
                    .get_chain_key(&next_hop)
                    .await?
                    .ok_or(PacketDecodingError(format!(
                        "failed to find channel key for packet key {next_peer} on next hop",
                    )))?;

                // Find the corresponding channel
                debug!("looking for channel with {previous_hop_addr} ({previous_peer})");
                let channel = self
                    .db
                    .read()
                    .await
                    .get_channel_from(&previous_hop_addr)
                    .await?
                    .ok_or(ChannelNotFound(previous_hop.to_string()))?;
                // END: channel = get_channel_from_to(packet_key, packet_key)

                // Validate the ticket first
                let price_per_packet = self
                    .db
                    .read()
                    .await
                    .get_ticket_price()
                    .await
                    .unwrap_or_else(|_| {
                        warn!(
                            "Error reading ticket price value from database, using default {:?}",
                            *DEFAULT_PRICE_PER_PACKET
                        );
                        Some(*DEFAULT_PRICE_PER_PACKET)
                    })
                    .unwrap_or_else(|| {
                        warn!(
                            "No ticket price value set in database yet, using default {:?}",
                            *DEFAULT_PRICE_PER_PACKET
                        );
                        *DEFAULT_PRICE_PER_PACKET
                    });

                debug!("price per packet is {price_per_packet}");

                let validation_res = validate_unacknowledged_ticket::<Db>(
                    &*self.db.read().await,
                    &ticket,
                    &channel,
                    &previous_hop_addr,
                    Balance::new(price_per_packet, BalanceType::HOPR),
                    TICKET_WIN_PROB,
                    self.cfg.check_unrealized_balance,
                    &domain_separator,
                )
                .await;

                if let Err(e) = validation_res {
                    // Mark as reject and passthrough the error
                    self.db
                        .write()
                        .instrument(tracing::debug_span!("db: forwarded packet (mark rejected)"))
                        .await
                        .mark_rejected(&ticket)
                        .await?;

                    #[cfg(all(feature = "prometheus", not(test)))]
                    METRIC_REJECTED_TICKETS_COUNT.increment();

                    return Err(e);
                }

                {
                    let mut g = self
                        .db
                        .write()
                        .instrument(tracing::debug_span!(
                            "db: forwarded packet (store pending acknowledgement)",
                            channel = channel.get_id().to_string()
                        ))
                        .await;
                    g.set_current_ticket_index(&channel.get_id().hash(), ticket.index.into())
                        .await?;

                    // Store the unacknowledged ticket
                    g.store_pending_acknowledgment(
                        ack_challenge,
                        PendingAcknowledgement::WaitingAsRelayer(UnacknowledgedTicket::new(
                            ticket.clone(),
                            own_key.clone(),
                            previous_hop_addr,
                        )),
                    )
                    .await?;
                }

                // Check that the calculated path position from the ticket matches value from the packet header
                let ticket_path_pos = ticket.get_path_position(price_per_packet)?;
                if !ticket_path_pos.eq(&path_pos) {
                    error!("path position mismatch: from ticket {ticket_path_pos}, from packet {path_pos}");
                    return Err(PathPositionMismatch);
                }

                // Create next ticket for the packet
                let mut ticket = if ticket_path_pos == 1 {
                    Ticket::new_zero_hop(&next_hop_addr, &self.cfg.chain_keypair, &domain_separator)?
                } else {
                    self.create_multihop_ticket(next_hop_addr, ticket_path_pos).await?
                };

                // forward packet
                ticket.challenge = next_challenge.to_ethereum_challenge();
                ticket.sign(&self.cfg.chain_keypair, &domain_separator);

                let ack = Acknowledgement::new(ack_key, pkt_keypair);

                let mut payload = Vec::with_capacity(ChainPacketComponents::SIZE);
                payload.extend_from_slice(packet.as_ref());
                payload.extend_from_slice(&ticket.to_bytes());

                Ok(TransportPacket::Forwarded {
                    packet_tag,
                    previous_hop: previous_peer,
                    next_hop: next_peer,
                    data: payload.into_boxed_slice(),
                    ack,
                })
            }
            ChainPacketComponents::Outgoing { .. } => {
                Err(PacketError::LogicError("Cannot receive an outgoing packet".into()))
            }
        }
    }
}

impl<Db> PacketProcessor<Db>
where
    Db: HoprCoreEthereumDbActions + std::marker::Send + std::marker::Sync + std::fmt::Debug,
{
    /// Creates a new instance given the DB and configuration.
    pub fn new(db: Arc<RwLock<Db>>, cfg: PacketInteractionConfig) -> Self {
        Self { db, cfg }
    }

    #[tracing::instrument(level = "debug")]
    async fn create_multihop_ticket(&self, destination: Address, path_pos: u8) -> Result<Ticket> {
        trace!("begin creating multihop ticket for destination {destination}");

        let (channel, channel_id, current_ticket_index) = {
            let db = self.db.read().await;

            let channel = db
                .get_channel_to(&destination)
                .await?
                .ok_or(ChannelNotFound(destination.to_string()))?;

            let channel_id = channel.get_id();

            let current_index = db.get_current_ticket_index(&channel_id).await?.unwrap_or(U256::one());

            (channel, channel_id, current_index)
        };

        self.db
            .write()
            .await
            .set_current_ticket_index(&channel_id, current_ticket_index + 1u32)
            .await?;

        let ticket = {
            let db = self.db.read().await;

            let price_per_packet = db
                .get_ticket_price()
                .await
                .unwrap_or_else(|_| {
                    warn!(
                        "Error reading ticket price value from database, using default {:?}",
                        *DEFAULT_PRICE_PER_PACKET
                    );
                    Some(*DEFAULT_PRICE_PER_PACKET)
                })
                .unwrap_or_else(|| {
                    warn!(
                        "No ticket price value set in database yet, using default {:?}",
                        *DEFAULT_PRICE_PER_PACKET
                    );
                    *DEFAULT_PRICE_PER_PACKET
                });

            let amount = Balance::new(
                price_per_packet
                    .div_f64(TICKET_WIN_PROB)
                    .expect("winning probability outside of allowed interval (0.0, 1.0]")
                    * U256::from(path_pos - 1),
                BalanceType::HOPR,
            );

            if channel.balance.lt(&amount) {
                return Err(OutOfFunds(format!("{channel_id} with counterparty {destination}")));
            }

            Ticket::new_partial(
                &self.cfg.chain_keypair.public().to_address(),
                &destination,
                &amount,
                current_ticket_index,
                U256::one(),     // unaggregated always have index_offset == 1
                TICKET_WIN_PROB, // 100% winning probability
                channel.channel_epoch,
            )
        }?;

        debug!("Creating ticket in channel {channel_id}.",);

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_TICKETS_COUNT.increment();

        Ok(ticket)
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
    #[default(std::time::UNIX_EPOCH)]
    #[cfg(all(feature = "prometheus", not(test)))]
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
    ack_event_queue: (Sender<MsgToProcess>, Receiver<MsgProcessed>),
}

impl PacketInteraction {
    /// Creates a new instance given the DB and our public key used to verify the acknowledgements.
    pub fn new<Db: HoprCoreEthereumDbActions + Send + Sync + std::fmt::Debug + 'static>(
        db: Arc<RwLock<Db>>,
        tbf: Arc<RwLock<TagBloomFilter>>,
        cfg: PacketInteractionConfig,
    ) -> Self {
        let (to_process_tx, to_process_rx) = channel::<MsgToProcess>(PACKET_RX_QUEUE_SIZE + PACKET_TX_QUEUE_SIZE);
        let (processed_tx, processed_rx) = channel::<MsgProcessed>(PACKET_RX_QUEUE_SIZE + PACKET_TX_QUEUE_SIZE);

        let mixer_cfg = cfg.mixer;
        let pkt_keypair = cfg.packet_keypair.clone();
        let processor = PacketProcessor::new(db, cfg);

        let mut processing_stream = to_process_rx
            .then_concurrent(move |event| {
                let processor = processor.clone();
                let pkt_keypair = pkt_keypair.clone();

                async move {
                    let mut metadata = PacketMetadata::default();

                    let packet = match event {
                        MsgToProcess::ToReceive(data, peer) | MsgToProcess::ToForward(data, peer) => {
                            processor.from_incoming(data, &pkt_keypair, &peer).await
                        }
                        MsgToProcess::ToSend(data, path, finalizer) => {
                            metadata.send_finalizer.replace(finalizer);

                            processor.into_outgoing(data, &path).await
                        }
                    };

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
                    if let Ok(p) = &packet {
                        let packet_tag = match p {
                            TransportPacket::Final { packet_tag, .. } => Some(packet_tag),
                            TransportPacket::Forwarded { packet_tag, .. } => Some(packet_tag),
                            _ => None,
                        };

                        if let Some(tag) = packet_tag {
                            // There is a 0.1% chance that the positive result is not a replay
                            // because a Bloom filter is used
                            if tbf
                                .write()
                                .instrument(tracing::debug_span!("tbf: check tag replay"))
                                .await
                                .check_and_set(tag)
                            {
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
                            debug!("Mixer created a random packet delay of {}ms", random_delay.as_millis());

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
                        #[cfg_attr(test, allow(unused_variables))]
                        Ok((processed_msg, metadata)) => {
                            #[cfg(all(feature = "prometheus", not(test)))]
                            if let MsgProcessed::Forward(_, _, _, _) = &processed_msg {
                                METRIC_RELAYED_PACKET_IN_MIXER_TIME.observe(
                                    hopr_platform::time::native::current_time()
                                        .duration_since(metadata.start_time)
                                        .unwrap_or_default()
                                        .as_secs_f64(),
                                )
                            };

                            match poll_fn(|cx| Pin::new(&mut processed_tx).poll_ready(cx)).await {
                                Ok(_) => match processed_tx.start_send(processed_msg) {
                                    Ok(_) => {}
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

        spawn(async move {
            // poll the stream until it's done
            while processing_stream.next().await.is_some() {}
        });

        Self {
            ack_event_queue: (to_process_tx, processed_rx),
        }
    }

    pub fn writer(&self) -> PacketActions {
        PacketActions {
            queue: self.ack_event_queue.0.clone(),
        }
    }
}

impl Stream for PacketInteraction {
    type Item = MsgProcessed;

    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        use futures_lite::stream::StreamExt;
        Pin::new(self).ack_event_queue.1.poll_next(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::{ApplicationData, MsgProcessed, PacketInteraction, PacketInteractionConfig, DEFAULT_PRICE_PER_PACKET};
    use crate::{
        ack::processor::{AckProcessed, AcknowledgementInteraction, Reply},
        msg::mixer::MixerConfig,
    };
    use async_lock::RwLock;
    use async_trait::async_trait;
    use chain_db::{db::CoreEthereumDb, traits::HoprCoreEthereumDbActions};
    use core_packet::por::ProofOfRelayValues;
    use core_path::channel_graph::ChannelGraph;
    use core_path::path::{Path, TransportPath};
    use futures::{
        future::{select, Either},
        pin_mut, StreamExt,
    };
    use hex_literal::hex;
    use hopr_crypto_random::{random_bytes, random_integer};
    use hopr_crypto_sphinx::{derivation::derive_ack_key_share, shared_keys::SharedSecret};
    use hopr_crypto_types::prelude::*;
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use lazy_static::lazy_static;
    use libp2p_identity::PeerId;
    use serial_test::serial;
    use std::{sync::Arc, time::Duration};
    use tracing::debug;
    use utils_db::{db::DB, CurrentDbShim};

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

    async fn create_dbs(amount: usize) -> Vec<CurrentDbShim> {
        futures::future::join_all((0..amount).map(|_| CurrentDbShim::new_in_memory())).await
    }

    fn create_core_dbs(dbs: &Vec<CurrentDbShim>) -> Vec<Arc<RwLock<CoreEthereumDb<CurrentDbShim>>>> {
        dbs.iter()
            .enumerate()
            .map(|(i, db)| {
                Arc::new(RwLock::new(CoreEthereumDb::new(
                    DB::new(db.clone()),
                    (&PEERS_CHAIN[i]).into(),
                )))
            })
            .collect::<Vec<_>>()
    }

    async fn create_minimal_topology(dbs: &Vec<CurrentDbShim>) -> crate::errors::Result<()> {
        let testing_snapshot = Snapshot::default();
        let mut previous_channel: Option<ChannelEntry> = None;

        for index in 0..dbs.len() {
            let mut db = CoreEthereumDb::new(DB::new(dbs[index].clone()), PEERS_CHAIN[index].public().to_address());

            // Link all the node keys and chain keys from the simulated announcements
            for i in 0..PEERS.len() {
                let node_key = PEERS[i].public();
                let chain_key = PEERS_CHAIN[i].public();
                db.link_chain_and_packet_keys(&chain_key.to_address(), node_key, &Snapshot::default())
                    .await?;
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

                db.update_channel_and_snapshot(&channel.unwrap().get_id(), &channel.unwrap(), &testing_snapshot)
                    .await?;
            }

            if index > 0 {
                db.update_channel_and_snapshot(
                    &previous_channel.unwrap().get_id(),
                    &previous_channel.unwrap(),
                    &testing_snapshot,
                )
                .await?;
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

    #[serial]
    #[async_std::test]
    pub async fn test_packet_acknowledgement_sender_workflow() {
        let _ = env_logger::builder().is_test(true).try_init();
        const TIMEOUT_SECONDS: u64 = 10;

        // let (done_tx, mut done_rx) = futures::channel::mpsc::unbounded();

        let dbs = create_dbs(2).await;

        create_minimal_topology(&dbs)
            .await
            .expect("failed to create minimal channel topology");

        let core_dbs = create_core_dbs(&dbs);

        // Begin test
        debug!("peer 1 (sender)    = {}", PEERS[0].public().to_peerid_str());
        debug!("peer 2 (recipient) = {}", PEERS[1].public().to_peerid_str());

        const PENDING_ACKS: usize = 5;
        let mut sent_challenges = Vec::with_capacity(PENDING_ACKS);
        for _ in 0..PENDING_ACKS {
            let secrets = (0..2).map(|_| SharedSecret::random()).collect::<Vec<_>>();
            let porv = ProofOfRelayValues::new(&secrets[0], Some(&secrets[1]));

            // Mimics that the packet sender has sent a packet and now it has a pending acknowledgement in it's DB
            core_dbs[0]
                .write()
                .await
                .store_pending_acknowledgment(porv.ack_challenge, PendingAcknowledgement::WaitingAsSender)
                .await
                .expect("failed to store pending ack");

            // This is what counterparty derives and sends back to solve the challenge
            let ack_key = derive_ack_key_share(&secrets[0]);

            sent_challenges.push((ack_key, porv.ack_challenge));
        }

        // Peer 1: ACK interaction of the packet sender, hookup receiving of acknowledgements and start processing them
        let mut ack_interaction_sender = AcknowledgementInteraction::new(core_dbs[0].clone(), &PEERS_CHAIN[0]);

        // Peer 2: Recipient of the packet and sender of the acknowledgement
        let mut ack_interaction_counterparty = AcknowledgementInteraction::new(core_dbs[1].clone(), &PEERS_CHAIN[1]);

        // Peer 2: start sending out outgoing acknowledgement
        for (ack_key, _) in sent_challenges.clone() {
            ack_interaction_counterparty
                .writer()
                .send_acknowledgement(PEERS[0].public().into(), Acknowledgement::new(ack_key, &PEERS[1]))
                .expect("failed to send ack");

            // emulate channel to another peer
            match ack_interaction_counterparty.next().await {
                Some(value) => match value {
                    AckProcessed::Send(_, ack) => ack_interaction_sender
                        .writer()
                        .receive_acknowledgement(PEERS[1].public().into(), ack)
                        .expect("failed to receive ack"),
                    _ => panic!("Unexpected incoming acknowledgement detected"),
                },
                None => panic!("There should have been an acknowledgment to send"),
            }
        }

        let finish = async move {
            for i in 1..PENDING_ACKS + 1 {
                if let Some(a) = ack_interaction_sender.next().await {
                    match a {
                        AckProcessed::Receive(_, Ok(Reply::Sender(ack))) => {
                            debug!("sender has received acknowledgement {i}: {ack}");
                            assert!(
                                sent_challenges.iter().any(|(_, c)| ack.eq(c)),
                                "received invalid challenge {ack}"
                            );
                        }
                        _ => assert!(false, "Should only receive as a Sender"),
                    }
                }
            }
        };
        let timeout = async_std::task::sleep(Duration::from_secs(TIMEOUT_SECONDS));
        pin_mut!(finish, timeout);

        let succeeded = match select(finish, timeout).await {
            Either::Left(_) => true,
            Either::Right(_) => false,
        };

        assert!(succeeded, "test timed out after {TIMEOUT_SECONDS} seconds");
    }

    async fn peer_setup_for(count: usize) -> Vec<(AcknowledgementInteraction, PacketInteraction)> {
        let peer_count = count;

        assert!(peer_count <= PEERS.len());
        assert!(peer_count >= 3);
        let dbs = create_dbs(peer_count).await;

        create_minimal_topology(&dbs)
            .await
            .expect("failed to create minimal channel topology");

        let core_dbs = create_core_dbs(&dbs);

        for core_db in &core_dbs {
            core_db
                .write()
                .await
                .set_channels_domain_separator(&Hash::default(), &Snapshot::default())
                .await
                .unwrap();
        }

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

        core_dbs
            .into_iter()
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
                            Reply::Sender(hkc) => {
                                assert_eq!(i - 1, 0, "Only the sender can receive a half key challenge");
                                received_challenges.push(hkc);
                            }
                            Reply::RelayerWinning(tkt) => {
                                // choose the last relayer before the receiver
                                if i - 1 == components.len() - 2 {
                                    received_tickets.push(tkt)
                                }
                            }
                            Reply::RelayerLosing => {
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
        impl PeerAddressResolver for TestResolver {
            async fn resolve_packet_key(&self, onchain_key: &Address) -> Option<OffchainPublicKey> {
                self.0.iter().find(|(_, addr)| addr.eq(onchain_key)).map(|(pk, _)| *pk)
            }

            async fn resolve_chain_key(&self, offchain_key: &OffchainPublicKey) -> Option<Address> {
                self.0.iter().find(|(pk, _)| pk.eq(offchain_key)).map(|(_, addr)| *addr)
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
    async fn test_packet_relayer_workflow_3_peers() {
        let _ = env_logger::builder().is_test(true).try_init();
        packet_relayer_workflow_n_peers(3, 5).await;
    }

    #[serial]
    #[async_std::test]
    async fn test_packet_relayer_workflow_5_peers() {
        let _ = env_logger::builder().is_test(true).try_init();
        packet_relayer_workflow_n_peers(5, 5).await;
    }
}
