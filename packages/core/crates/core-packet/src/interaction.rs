use crate::errors::PacketError::{
    AcknowledgementValidation, ChannelNotFound, InvalidPacketState, MissingDomainSeparator, OutOfFunds,
    PacketConstructionError, PacketDecodingError, PathError, PathPositionMismatch, Retry, TagReplay, TransportError,
};
use crate::errors::Result;
use crate::packet::{Packet, PacketState};
use async_lock::RwLock;
use core_crypto::keypairs::{ChainKeypair, Keypair, OffchainKeypair};
use core_crypto::types::{HalfKeyChallenge, Hash, OffchainPublicKey};
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_mixer::future_extensions::StreamThenConcurrentExt;
use core_mixer::mixer::{Mixer, MixerConfig};
use core_path::errors::PathError::PathNotValid;
use core_path::path::Path;
use core_types::acknowledgement::{AcknowledgedTicket, Acknowledgement, PendingAcknowledgement, UnacknowledgedTicket};
use core_types::channels::Ticket;
use futures::channel::mpsc::{channel, Receiver, Sender, UnboundedSender};
use futures::future::{poll_fn, Either};
use futures::{pin_mut, stream::Stream, StreamExt};
use libp2p_identity::PeerId;
use std::pin::Pin;
use std::sync::Arc;
use utils_log::{debug, error, info, warn};
use utils_types::primitives::{Address, Balance, BalanceType, U256};
use utils_types::traits::{BinarySerializable, PeerIdLike, ToHex};

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::spawn_local;

#[cfg(all(feature = "wasm", not(test)))]
use wasm_bindgen_futures::spawn_local;

use crate::validation::validate_unacknowledged_ticket;
#[cfg(all(feature = "prometheus", not(test)))]
use utils_metrics::metrics::SimpleCounter;
#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_RECEIVED_SUCCESSFUL_ACKS: SimpleCounter = SimpleCounter::new(
        "core_counter_received_successful_acks",
        "Number of received successful acknowledgements"
    )
    .unwrap();
    static ref METRIC_RECEIVED_FAILED_ACKS: SimpleCounter = SimpleCounter::new(
        "core_counter_received_failed_acks",
        "Number of received failed acknowledgements"
    )
    .unwrap();
    static ref METRIC_SENT_ACKS: SimpleCounter =
        SimpleCounter::new("core_counter_sent_acks", "Number of sent message acknowledgements").unwrap();
    static ref METRIC_ACKED_TICKETS: SimpleCounter =
        SimpleCounter::new("core_counter_acked_tickets", "Number of acknowledged tickets").unwrap();
    static ref METRIC_FWD_MESSAGE_COUNT: SimpleCounter =
        SimpleCounter::new("core_counter_forwarded_messages", "Number of forwarded messages").unwrap();
    static ref METRIC_RECV_MESSAGE_COUNT: SimpleCounter =
        SimpleCounter::new("core_counter_received_messages", "Number of received messages").unwrap();
    static ref METRIC_TICKETS_COUNT: SimpleCounter =
        SimpleCounter::new("core_counter_created_tickets", "Number of created tickets").unwrap();
    static ref METRIC_PACKETS_COUNT: SimpleCounter =
        SimpleCounter::new("core_counter_packets", "Number of created packets").unwrap();
}

lazy_static::lazy_static! {
    /// Fixed price per packet to 0.01 HOPR
    static ref DEFAULT_PRICE_PER_PACKET: U256 = 10000000000000000u128.into();
}

// Default sizes of the acknowledgement queues
pub const ACK_TX_QUEUE_SIZE: usize = 2048;
pub const ACK_RX_QUEUE_SIZE: usize = 2048;

/// Represents a payload (packet or acknowledgement) at the transport level.
#[derive(Debug, Clone)]
pub(crate) struct Payload {
    pub(crate) remote_peer: PeerId,
    pub(crate) data: Box<[u8]>,
}

impl Display for Payload {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Payload")
            .field("remote_peer", &self.remote_peer)
            .field("data", &hex::encode(&self.data))
            .finish()
    }
}

#[derive(Debug)]
pub enum AckToProcess {
    ToReceive(PeerId, Acknowledgement),
    ToSend(PeerId, Acknowledgement),
}

#[derive(Debug)]
pub enum AckProcessed {
    Receive(PeerId, Result<()>),
    Send(PeerId, Acknowledgement),
}

/// Implements protocol acknowledgement logic for acknowledgements
pub struct AcknowledgementProcessor<Db: HoprCoreEthereumDbActions> {
    db: Arc<RwLock<Db>>,
    chain_key: ChainKeypair,
    pub on_acknowledgement: Option<UnboundedSender<HalfKeyChallenge>>,
    pub on_acknowledged_ticket: Option<UnboundedSender<AcknowledgedTicket>>,
}

impl<Db: HoprCoreEthereumDbActions> Clone for AcknowledgementProcessor<Db> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            chain_key: self.chain_key.clone(),
            on_acknowledgement: self.on_acknowledgement.clone(),
            on_acknowledged_ticket: self.on_acknowledged_ticket.clone(),
        }
    }
}

impl<Db: HoprCoreEthereumDbActions> AcknowledgementProcessor<Db> {
    pub fn new(
        db: Arc<RwLock<Db>>,
        chain_key: &ChainKeypair,
        on_acknowledgement: Option<UnboundedSender<HalfKeyChallenge>>,
        on_acknowledged_ticket: Option<UnboundedSender<AcknowledgedTicket>>,
    ) -> Self {
        Self {
            db,
            on_acknowledgement,
            chain_key: chain_key.clone(),
            on_acknowledged_ticket,
        }
    }

    pub async fn handle_acknowledgement(&mut self, ack: Acknowledgement) -> Result<()> {
        /*
            There are three cases:
            1. There is an unacknowledged ticket and we are
                awaiting a half key.
            2. We were the creator of the packet, hence we
                do not wait for any half key
            3. The acknowledgement is unexpected and stems from
                a protocol bug or an attacker
        */

        let pending = self
            .db
            .read()
            .await
            .get_pending_acknowledgement(&ack.ack_challenge())
            .await?
            .ok_or_else(|| {
                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_RECEIVED_FAILED_ACKS.increment();

                AcknowledgementValidation(format!(
                    "received unexpected acknowledgement for half key challenge {} - half key {}",
                    ack.ack_challenge().to_hex(),
                    ack.ack_key_share.to_hex()
                ))
            })?;

        match pending {
            PendingAcknowledgement::WaitingAsSender => {
                // No pending ticket, nothing to do.
                debug!("received acknowledgement as sender: first relayer has processed the packet.");
                if let Some(emitter) = &mut self.on_acknowledgement {
                    if let Err(e) = emitter.unbounded_send(ack.ack_challenge()) {
                        error!("failed to emit received acknowledgement: {e}")
                    }
                }

                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_RECEIVED_SUCCESSFUL_ACKS.increment();
            }

            PendingAcknowledgement::WaitingAsRelayer(unackowledged) => {
                // Try to unlock our incentive
                unackowledged.verify_challenge(&ack.ack_key_share).map_err(|e| {
                    #[cfg(all(feature = "prometheus", not(test)))]
                    METRIC_RECEIVED_FAILED_ACKS.increment();

                    AcknowledgementValidation(format!(
                        "the acknowledgement is not sufficient to solve the embedded challenge, {e}"
                    ))
                })?;

                self.db
                    .read()
                    .await
                    .get_channel_from(&unackowledged.signer)
                    .await
                    .map_err(|e| {
                        #[cfg(all(feature = "prometheus", not(test)))]
                        METRIC_RECEIVED_FAILED_ACKS.increment();

                        AcknowledgementValidation(format!(
                            "acknowledgement received for channel that does not exist, {e}"
                        ))
                    })?;

                let domain_separator = self
                    .db
                    .read()
                    .await
                    .get_channels_domain_separator()
                    .await
                    .unwrap()
                    .ok_or(MissingDomainSeparator)?;

                let ack_ticket = unackowledged.acknowledge(&ack.ack_key_share, &self.chain_key, &domain_separator)?;

                // replace the un-acked ticket with acked ticket.
                debug!(">>> WRITE replacing unack with ack");
                self.db
                    .write()
                    .await
                    .replace_unack_with_ack(&ack.ack_challenge(), ack_ticket.clone())
                    .await?;
                debug!("<<< WRITE replacing unack with ack");

                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_ACKED_TICKETS.increment();

                if let Some(emitter) = &mut self.on_acknowledged_ticket {
                    if let Err(e) = emitter.unbounded_send(ack_ticket) {
                        error!("failed to emit acknowledged ticket: {e}");
                    }
                }
            }
        }
        Ok(())
    }
}

/// External API for feeding Acknowledgement actions into the Acknowledgement processor
/// processing the elements independently in the background.
#[derive(Debug, Clone)]
pub struct AcknowledgementActions {
    pub queue: Sender<AckToProcess>,
}

impl AcknowledgementActions {
    /// Pushes the acknowledgement received from the transport layer into processing.
    pub fn receive_acknowledgement(&mut self, source: PeerId, acknowledgement: Acknowledgement) -> Result<()> {
        self.process(AckToProcess::ToReceive(source, acknowledgement))
    }

    /// Pushes a new outgoing acknowledgement into the processing.
    pub fn send_acknowledgement(&mut self, destination: PeerId, acknowledgement: Acknowledgement) -> Result<()> {
        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_SENT_ACKS.increment();

        self.process(AckToProcess::ToSend(destination, acknowledgement))
    }

    fn process(&mut self, event: AckToProcess) -> Result<()> {
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

/// Sets up processing of acknowledgement interactions and returns relevant read and write mechanism.
///
/// When a new acknowledgement is delivered from the transport the `receive_acknowledgement`
/// method is used to push it into the processing queue of incoming acknowledgements.
///
/// Acknowledgments issued by this node are generated using the `send_acknowledgement` method.
///
/// The result of processing the acknowledgements can be extracted as a stream.
pub struct AcknowledgementInteraction {
    ack_event_queue: (Sender<AckToProcess>, Receiver<AckProcessed>),
}

impl AcknowledgementInteraction {
    /// Creates a new instance given the DB and our public key used to verify the acknowledgements.
    pub fn new<Db: HoprCoreEthereumDbActions + 'static>(
        db: Arc<RwLock<Db>>,
        chain_key: &ChainKeypair,
        on_acknowledgement: Option<UnboundedSender<HalfKeyChallenge>>,
        on_acknowledged_ticket: Option<UnboundedSender<AcknowledgedTicket>>,
    ) -> Self {
        let (processing_in_tx, processing_in_rx) = channel::<AckToProcess>(ACK_RX_QUEUE_SIZE + ACK_TX_QUEUE_SIZE);
        let (processing_out_tx, processing_out_rx) = channel::<AckProcessed>(ACK_RX_QUEUE_SIZE + ACK_TX_QUEUE_SIZE);

        let processor = AcknowledgementProcessor::new(db, chain_key, on_acknowledgement, on_acknowledged_ticket);

        let processing_stream = processing_in_rx.then_concurrent(move |event| {
            let mut processor = processor.clone();
            let mut processed_tx = processing_out_tx.clone();

            async move {
                let processed: Option<AckProcessed> = match event {
                    AckToProcess::ToReceive(peer, mut ack) => {
                        if let Ok(remote_pk) = OffchainPublicKey::from_peerid(&peer) {
                            debug!("validating incoming acknowledgement from {}", peer);
                            if ack.validate(&remote_pk) {
                                debug!("going to handle incoming ack");
                                match processor.handle_acknowledgement(ack).await {
                                    Ok(_) => Some(AckProcessed::Receive(peer, Ok(()))),
                                    Err(e) => {
                                        error!(
                                            "Encountered error while handling acknowledgement from peer '{}': {}",
                                            &peer, e
                                        );
                                        None
                                    }
                                }
                            } else {
                                error!("failed to verify signature on acknowledgement from peer {}", peer);
                                None
                            }
                        } else {
                            error!("invalid remote peer id {}", peer);
                            None
                        }
                    }
                    AckToProcess::ToSend(peer, ack) => Some(AckProcessed::Send(peer, ack)),
                };

                if let Some(event) = processed {
                    match poll_fn(|cx| Pin::new(&mut processed_tx).poll_ready(cx)).await {
                        Ok(_) => match processed_tx.start_send(event) {
                            Ok(_) => {}
                            Err(e) => error!("Failed to pass a processed ack message: {}", e),
                        },
                        Err(e) => {
                            warn!("The receiver for processed ack no longer exists: {}", e);
                        }
                    };
                }
            }
        });

        spawn_local(async move {
            processing_stream
                .map(|x| Ok(x))
                .forward(futures::sink::drain())
                .await
                .unwrap();
        });

        Self {
            ack_event_queue: (processing_in_tx, processing_out_rx),
        }
    }

    pub fn writer(&self) -> AcknowledgementActions {
        AcknowledgementActions {
            queue: self.ack_event_queue.0.clone(),
        }
    }
}

impl Stream for AcknowledgementInteraction {
    type Item = AckProcessed;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use futures_lite::stream::StreamExt;
        return std::pin::Pin::new(self).ack_event_queue.1.poll_next(cx);
    }
}

// Default sizes of the packet queues
const PACKET_TX_QUEUE_SIZE: usize = 2048;
const PACKET_RX_QUEUE_SIZE: usize = 2048;

#[derive(Debug)]
pub enum MsgToProcess {
    ToReceive(Box<[u8]>, PeerId),
    ToSend(Box<[u8]>, Path, PacketSendFinalizer),
    ToForward(Box<[u8]>, PeerId),
}

#[derive(Debug)]
pub enum MsgProcessed {
    Receive(PeerId, Box<[u8]>),
    Send(PeerId, Box<[u8]>),
    Forward(PeerId, Box<[u8]>),
}

/// Implements protocol acknowledgement logic for msg packets
pub struct PacketProcessor<Db>
where
    Db: HoprCoreEthereumDbActions,
{
    db: Arc<RwLock<Db>>,
    pub tbf: Arc<RwLock<TagBloomFilter>>,
    cfg: PacketInteractionConfig,
}

impl<Db> Clone for PacketProcessor<Db>
where
    Db: HoprCoreEthereumDbActions,
{
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            tbf: self.tbf.clone(),
            cfg: self.cfg.clone(),
        }
    }
}

pub enum PacketType {
    Final(Packet, Option<Acknowledgement>),
    Forward(Packet, Option<Acknowledgement>, PeerId, PeerId),
}

impl<Db> PacketProcessor<Db>
where
    Db: HoprCoreEthereumDbActions,
{
    /// Creates a new instance given the DB and configuration.
    pub fn new(db: Arc<RwLock<Db>>, tbf: Arc<RwLock<TagBloomFilter>>, cfg: PacketInteractionConfig) -> Self {
        Self { db, cfg, tbf }
    }

    async fn bump_ticket_index(&self, channel_id: &Hash) -> Result<U256> {
        let current_ticket_index = self
            .db
            .read()
            .await
            .get_current_ticket_index(channel_id)
            .await?
            .unwrap_or(U256::one());

        debug!(">>> WRITE set_current_ticket_index");
        self.db
            .write()
            .await
            .set_current_ticket_index(channel_id, current_ticket_index + 1u64.into())
            .await?;
        debug!("<<< WRITE set_current_ticket_index");

        Ok(current_ticket_index)
    }

    async fn create_multihop_ticket(&self, destination: Address, path_pos: u8) -> Result<Ticket> {
        debug!("begin creating multihop ticket for destination {destination}");
        let channel = self
            .db
            .read()
            .await
            .get_channel_to(&destination)
            .await?
            .ok_or(ChannelNotFound(destination.to_string()))?;

        let channel_id = channel.get_id();
        debug!("going to bump ticket index for channel id {channel_id}");
        let current_index = self.bump_ticket_index(&channel_id).await?;
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
        let amount = Balance::new(
            price_per_packet
                .divide_f64(TICKET_WIN_PROB)
                .expect("winning probability outside of allowed interval (0.0, 1.0]")
                * U256::from(path_pos - 1),
            BalanceType::HOPR,
        );

        debug!("retrieving pending balance to {destination}");
        let outstanding_balance = self.db.read().await.get_pending_balance_to(&destination).await?;

        let channel_balance = channel.balance.sub(&outstanding_balance);

        info!(
            "balances {} - {outstanding_balance} = {channel_balance} should >= {amount} in channel open to {}",
            channel.balance, channel.destination
        );

        if channel_balance.lt(&amount) {
            return Err(OutOfFunds(format!("{channel_id} with counterparty {destination}")));
        }

        let ticket = Ticket::new_partial(
            &self.cfg.chain_keypair.public().to_address(),
            &destination,
            &amount,
            current_index,
            U256::one(),     // unaggregated always have index_offset == 1
            TICKET_WIN_PROB, // 100% winning probability
            channel.channel_epoch,
        )?;

        debug!(">>> WRITE mark_pending");
        self.db.write().await.mark_pending(&destination, &ticket).await?;
        debug!("<<< WRITE mark_pending");

        debug!("Creating ticket in channel {channel_id}.",);

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_TICKETS_COUNT.increment();

        Ok(ticket)
    }

    async fn create_outgoing_packet(&self, data: Box<[u8]>, path: Path) -> Result<(Payload, HalfKeyChallenge)> {
        let next_peer = self
            .db
            .read()
            .await
            .get_chain_key(&OffchainPublicKey::from_peerid(&path.hops()[0])?)
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
                debug!("Missing domain separator.");
                MissingDomainSeparator
            })?;

        // Decide whether to create 0-hop or multihop ticket
        let next_ticket = if path.length() == 1 {
            Ticket::new_zero_hop(&next_peer, &self.cfg.chain_keypair, &domain_separator)
        } else {
            self.create_multihop_ticket(next_peer, path.length() as u8).await?
        };

        // Create the packet
        let packet = Packet::new(&data, &path, &self.cfg.chain_keypair, next_ticket, &domain_separator)?;
        debug!("packet state {}", packet.state());
        match packet.state() {
            PacketState::Outgoing { ack_challenge, .. } => {
                debug!(">>> WRITE store_pending_ack");
                self.db
                    .write()
                    .await
                    .store_pending_acknowledgment(*ack_challenge, PendingAcknowledgement::WaitingAsSender)
                    .await?;
                debug!("<<< WRITE store_pending_ack");

                Ok((
                    Payload {
                        remote_peer: path.hops()[0],
                        data: packet.to_bytes(),
                    },
                    ack_challenge.clone(),
                ))
            }
            _ => {
                debug!("invalid packet state {:?}", packet.state());
                Err(crate::errors::PacketError::Other(
                    utils_types::errors::GeneralError::Other("invalid packet state".into()),
                ))
            }
        }
    }

    fn create_packet_from_bytes(&self, data: &[u8], peer: &PeerId) -> Result<Packet> {
        Packet::from_bytes(data, &self.cfg.packet_keypair, peer)
    }

    pub async fn handle_mixed_packet(&self, mut packet: Packet) -> Result<PacketType> {
        let next_ticket;
        let previous_peer;
        let next_peer;

        let domain_separator = self
            .db
            .read()
            .await
            .get_channels_domain_separator()
            .await?
            .ok_or_else(|| {
                debug!("Missing domain separator");
                MissingDomainSeparator
            })?;

        match packet.state() {
            PacketState::Outgoing { .. } => return Err(InvalidPacketState),

            PacketState::Final { packet_tag, .. } => {
                // Validate if it's not a replayed packet
                if self.tbf.write().await.check_and_set(packet_tag) {
                    // This could be a false positive (0.1% chance) due to the use of Bloom filter
                    return Err(TagReplay);
                }

                debug!("packet is not replayed, creating acknowledgement...");
                let ack = packet.create_acknowledgement(&self.cfg.packet_keypair);
                return Ok(PacketType::Final(packet, ack));
            }

            PacketState::Forwarded {
                ack_challenge,
                previous_hop,
                own_key,
                next_hop,
                packet_tag,
                path_pos,
                ..
            } => {
                // Validate if it's not a replayed packet
                if self.tbf.write().await.check_and_set(packet_tag) {
                    // This could be a false positive due to the use of Bloom filter
                    return Err(TagReplay);
                }
                debug!("forwarded packet is not replayed");

                let previous_hop_addr =
                    self.db
                        .read()
                        .await
                        .get_chain_key(previous_hop)
                        .await?
                        .ok_or(PacketDecodingError(format!(
                            "failed to find channel key for packet key {previous_hop} on previous hop"
                        )))?;

                let next_hop_addr = self
                    .db
                    .read()
                    .await
                    .get_chain_key(next_hop)
                    .await?
                    .ok_or(PacketDecodingError(format!(
                        "failed to find channel key for packet key {next_hop} on next hop"
                    )))?;

                // Find the corresponding channel
                debug!("looking for channel with {previous_hop}");
                let channel = self
                    .db
                    .read()
                    .await
                    .get_channel_from(&previous_hop_addr)
                    .await?
                    .ok_or(ChannelNotFound(previous_hop.to_string()))?;

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

                if let Err(e) = validate_unacknowledged_ticket::<Db>(
                    &*self.db.read().await,
                    &packet.ticket,
                    &channel,
                    &previous_hop_addr,
                    Balance::new(price_per_packet, BalanceType::HOPR),
                    TICKET_WIN_PROB,
                    self.cfg.check_unrealized_balance,
                    &domain_separator,
                )
                .await
                {
                    // Mark as reject and passthrough the error
                    debug!(">>> WRITE mark_rejected");
                    self.db.write().await.mark_rejected(&packet.ticket).await?;
                    debug!("<<< WRITE mark_rejected");
                    return Err(e);
                }

                {
                    debug!(">>> WRITE store_pending_ack");
                    let mut g = self.db.write().await;
                    g.set_current_ticket_index(&channel.get_id().hash(), packet.ticket.index.into())
                        .await?;

                    debug!("storing pending acknowledgement");
                    // Store the unacknowledged ticket
                    g.store_pending_acknowledgment(
                        *ack_challenge,
                        PendingAcknowledgement::WaitingAsRelayer(UnacknowledgedTicket::new(
                            packet.ticket.clone(),
                            own_key.clone(),
                            previous_hop_addr,
                        )),
                    )
                    .await?;
                }
                debug!("<<< WRITE store_pending_ack");

                // Check that the calculated path position from the ticket matches value from the packet header
                let ticket_path_pos = packet.ticket.get_path_position(U256::from(price_per_packet))?;
                if !ticket_path_pos.eq(path_pos) {
                    error!("path position mismatch: from ticket {ticket_path_pos}, from packet {path_pos}");
                    return Err(PathPositionMismatch);
                }

                // Create next ticket for the packet
                next_ticket = if ticket_path_pos == 1 {
                    Ticket::new_zero_hop(&next_hop_addr, &self.cfg.chain_keypair, &domain_separator)
                } else {
                    self.create_multihop_ticket(next_hop_addr, ticket_path_pos).await?
                };
                previous_peer = previous_hop.to_peerid();
                next_peer = next_hop.to_peerid();
            }
        }

        // Transform the packet for forwarding using the next ticket
        packet.forward(&self.cfg.chain_keypair, next_ticket, &domain_separator)?;

        let ack = packet.create_acknowledgement(&self.cfg.packet_keypair);
        Ok(PacketType::Forward(packet, ack, previous_peer, next_peer))
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

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::sleep;
use core_types::protocol::{ApplicationData, TagBloomFilter, TICKET_WIN_PROB};
#[cfg(all(feature = "wasm", not(test)))]
use gloo_timers::future::sleep;
use std::fmt::{Display, Formatter};

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
    pub fn send_packet(&mut self, msg: ApplicationData, path: Path) -> Result<PacketSendAwaiter> {
        // Check if the path is valid
        if !path.valid() {
            return Err(PathError(PathNotValid));
        }

        let (tx, rx) = futures::channel::oneshot::channel::<HalfKeyChallenge>();

        self.process(MsgToProcess::ToSend(msg.to_bytes(), path, PacketSendFinalizer::new(tx)))
            .map(move |_| {
                let awaiter: PacketSendAwaiter = rx.into();
                awaiter
            })
    }

    /// Pushes a packet received from the transport designated for forwarding.
    pub fn forward_packet(&mut self, msg: Box<[u8]>, peer: PeerId) -> Result<()> {
        self.process(MsgToProcess::ToForward(msg, peer))
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
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
#[derive(Clone, Debug)]
pub struct PacketInteractionConfig {
    pub check_unrealized_balance: bool,
    pub packet_keypair: OffchainKeypair,
    pub chain_keypair: ChainKeypair,
    pub mixer: MixerConfig,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl PacketInteractionConfig {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new(packet_keypair: &OffchainKeypair, chain_keypair: &ChainKeypair) -> Self {
        Self {
            packet_keypair: packet_keypair.clone(),
            chain_keypair: chain_keypair.clone(),
            check_unrealized_balance: true,
            mixer: MixerConfig::default(),
        }
    }
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
    pub fn new<Db: HoprCoreEthereumDbActions + 'static>(
        db: Arc<RwLock<Db>>,
        tbf: Arc<RwLock<TagBloomFilter>>,
        mixer: Mixer,
        ack_interaction: AcknowledgementActions,
        on_final_packet: Option<Sender<ApplicationData>>,
        cfg: PacketInteractionConfig,
    ) -> Self {
        let (to_process_tx, to_process_rx) = channel::<MsgToProcess>(PACKET_RX_QUEUE_SIZE + PACKET_TX_QUEUE_SIZE);
        let (processed_tx, processed_rx) = channel::<MsgProcessed>(PACKET_RX_QUEUE_SIZE + PACKET_TX_QUEUE_SIZE);

        let processor = PacketProcessor::new(db, tbf, cfg);

        let processing_stream = to_process_rx
            .then_concurrent(move |event| async move {
                match event {
                    MsgToProcess::ToSend(_, _, _)
                    | MsgToProcess::ToForward(_, _) => { mixer.mix(event).await },
                    MsgToProcess::ToReceive(_, _) => { event },
                }
            })
            .then_concurrent(move |event| {
                let processor = processor.clone();
                let mut processed_tx = processed_tx.clone();
                let mut on_final_packet = on_final_packet.clone();
                let mut ack_interaction = ack_interaction.clone();

                async move {
                    let processed: Option<MsgProcessed> = match event {
                    MsgToProcess::ToReceive(data, peer) |
                    MsgToProcess::ToForward(data, peer) => {
                        match processor.create_packet_from_bytes(&data, &peer) {
                            Ok(packet) => {
                                match processor.handle_mixed_packet(packet).await {
                                    Ok(value) => match value {
                                        PacketType::Final(packet, ack) => {
                                            // We're the destination of the packet, so emit the packet contents
                                            match packet.state() {
                                                PacketState::Final { plain_text, previous_hop, .. } => {
                                                    if let Some(emitter) = &mut on_final_packet {
                                                        match ApplicationData::from_bytes(&plain_text) {
                                                            Ok(app_data) => {
                                                                if let Err(e) = emitter.try_send(app_data) {
                                                                    error!("failed to emit received final packet: {e}");
                                                                }
                                                            }
                                                            Err(e) => error!("failed to reconstruct application data from final packet: {e}")
                                                        }
                                                    }

                                                    if let Some(ack) = ack {
                                                        if let Err(e) = ack_interaction.send_acknowledgement(previous_hop.to_peerid(), ack) {
                                                            error!("failed to acknowledge relayed packet: {e}");
                                                        }
                                                    }

                                                    #[cfg(all(feature = "prometheus", not(test)))]
                                                    METRIC_RECV_MESSAGE_COUNT.increment();

                                                    Some(MsgProcessed::Receive(previous_hop.to_peerid(), plain_text.clone()))
                                                },
                                                _ => {
                                                    error!("A presumed final packet was not in fact final");
                                                    None
                                                }
                                            }
                                        },
                                        PacketType::Forward(packet, ack, previous_peer, next_peer) => {
                                            if let Some(ack) = ack {
                                                if let Err(e) = ack_interaction.send_acknowledgement(previous_peer, ack) {
                                                    error!("failed to acknowledge relayed packet: {e}");
                                                }
                                            }

                                            #[cfg(all(feature = "prometheus", not(test)))]
                                            METRIC_FWD_MESSAGE_COUNT.increment();

                                            Some(MsgProcessed::Forward(next_peer, packet.to_bytes()))
                                        },
                                    },
                                    Err(e) => {
                                        error!("Failed to mix a packet: {}", e);
                                        None
                                    },
                                }
                            },
                            Err(e) => {
                                error!("Failed to construct a proper packet: {:?}", e);
                                None
                            },
                        }
                    },
                    MsgToProcess::ToSend(data, path, finalizer) => {
                        match processor.create_outgoing_packet(data, path).await {
                            Ok((payload, challenge)) => {
                                #[cfg(all(feature = "prometheus", not(test)))]
                                METRIC_PACKETS_COUNT.increment();

                                finalizer.finalize(challenge);
                                Some(MsgProcessed::Send(payload.remote_peer, payload.data))
                            },
                            Err(e) => {
                                error!("Encountered error creating a packet to send: {}", e);
                                None
                            },
                        }
                    },
                };

                if processed.is_some() {
                    match poll_fn(|cx| Pin::new(&mut processed_tx).poll_ready(cx)).await {
                        Ok(_) => {
                            match processed_tx.start_send(processed.unwrap()) {
                                Ok(_) => {},
                                Err(e) => error!("Failed to pass a processed ack message: {}", e),
                            }
                        },
                        Err(e) => {
                            warn!("The receiver for processed packets no longer exists: {}", e);
                        }
                    };
                }
            }});

        spawn_local(async move {
            processing_stream
                .map(|x| Ok(x))
                .forward(futures::sink::drain())
                .await
                .unwrap();
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

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use futures_lite::stream::StreamExt;
        return std::pin::Pin::new(self).ack_event_queue.1.poll_next(cx);
    }
}

#[cfg(test)]
mod tests {
    use crate::interaction::DEFAULT_PRICE_PER_PACKET;
    use crate::{
        interaction::{
            AckProcessed, AcknowledgementInteraction, ApplicationData, MsgProcessed, PacketInteraction,
            PacketInteractionConfig,
        },
        por::ProofOfRelayValues,
    };
    use async_std::sync::RwLock;
    use core_crypto::random::random_integer;
    use core_crypto::{
        derivation::derive_ack_key_share,
        keypairs::{ChainKeypair, Keypair, OffchainKeypair},
        random::random_bytes,
        shared_keys::SharedSecret,
        types::{HalfKeyChallenge, Hash, PublicKey},
    };
    use core_ethereum_db::{db::CoreEthereumDb, traits::HoprCoreEthereumDbActions};
    use core_mixer::mixer::{Mixer, MixerConfig};
    use core_path::path::Path;
    use core_types::protocol::{Tag, TagBloomFilter};
    use core_types::{
        acknowledgement::{AcknowledgedTicket, Acknowledgement, PendingAcknowledgement},
        channels::{ChannelEntry, ChannelStatus},
    };
    use futures::channel::mpsc::{Sender, UnboundedSender};
    use futures::future::{join3, select, Either};
    use futures::{pin_mut, StreamExt};
    use hex_literal::hex;
    use lazy_static::lazy_static;
    use libp2p_identity::PeerId;
    use serial_test::serial;
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };
    use utils_db::{db::DB, leveldb::rusty::RustyLevelDbShim};
    use utils_log::debug;
    use utils_types::{
        primitives::{Address, Balance, BalanceType, Snapshot, U256},
        traits::{BinarySerializable, PeerIdLike},
    };

    use super::{PacketSendAwaiter, PacketSendFinalizer};

    #[async_std::test]
    pub async fn test_packet_send_finalizer_succeeds_with_a_stored_challenge() {
        let (tx, rx) = futures::channel::oneshot::channel::<HalfKeyChallenge>();

        let finalizer = PacketSendFinalizer::new(tx);
        let challenge = HalfKeyChallenge::default();
        let mut awaiter: PacketSendAwaiter = rx.into();

        finalizer.finalize(challenge.clone());

        let result = awaiter.consume_and_wait(Duration::from_millis(20)).await;

        assert_eq!(challenge, result.expect("HalfKeyChallange should be transmitted"));
    }

    const PEERS_PRIVS: [[u8; 32]; 5] = [
        hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"),
        hex!("5bf21ea8cccd69aa784346b07bf79c84dac606e00eecaa68bf8c31aff397b1ca"),
        hex!("3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa"),
        hex!("db7e3e8fcac4c817aa4cecee1d6e2b4d53da51f9881592c0e1cc303d8a012b92"),
        hex!("0726a9704d56a013980a9077d195520a61b5aed28f92d89c50bca6e0e0c48cfc"),
    ];

    const PEERS_CHAIN_PRIVS: [[u8; 32]; 5] = [
        hex!("4db3ac225fdcc7e20bf887cd90bbd62dc6bd41ce8ba5c23cc9ae0bf56e20d056"),
        hex!("1d40c69c179528bbdf49c2254e93400b485f47d7d2fa84aae280af5a31c1918b"),
        hex!("99facd2cd33664d65826ad220920a6b356e31d18c1ce1734303b70a962664d71"),
        hex!("62b362fd3295caf8657b8cf4f65d6e2cbb1ef81754f7bdff65e510220611afc2"),
        hex!("40ed717eb285dea3921a8346155d988b7ed5bf751bc4eee3cd3a64f4c692396f"),
    ];

    lazy_static! {
        static ref PEERS: Vec<OffchainKeypair> = PEERS_PRIVS
            .iter()
            .map(|private| OffchainKeypair::from_secret(private).unwrap())
            .collect();
    }

    lazy_static! {
        static ref PEERS_CHAIN: Vec<ChainKeypair> = PEERS_CHAIN_PRIVS
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
            U256::zero(),
        )
    }

    fn create_dbs(amount: usize) -> Vec<Arc<Mutex<rusty_leveldb::DB>>> {
        (0..amount)
            .map(|i| {
                Arc::new(Mutex::new(
                    rusty_leveldb::DB::open(format!("test_db_{i}"), rusty_leveldb::in_memory()).unwrap(),
                ))
            })
            .collect()
    }

    fn create_core_dbs(dbs: &Vec<Arc<Mutex<rusty_leveldb::DB>>>) -> Vec<Arc<RwLock<CoreEthereumDb<RustyLevelDbShim>>>> {
        dbs.iter()
            .enumerate()
            .map(|(i, db)| {
                Arc::new(RwLock::new(CoreEthereumDb::new(
                    DB::new(RustyLevelDbShim::new(db.clone())),
                    PublicKey::from_privkey(&PEERS_CHAIN_PRIVS[i]).unwrap().to_address(),
                )))
            })
            .collect::<Vec<_>>()
    }

    async fn create_minimal_topology(dbs: &Vec<Arc<Mutex<rusty_leveldb::DB>>>) -> crate::errors::Result<()> {
        let testing_snapshot = Snapshot::default();
        let mut previous_channel: Option<ChannelEntry> = None;

        for index in 0..dbs.len() {
            let mut db = CoreEthereumDb::new(
                DB::new(RustyLevelDbShim::new(dbs[index].clone())),
                PEERS_CHAIN[index].public().to_address(),
            );

            // Link all the node keys and chain keys from the simulated announcements
            for i in 0..PEERS_PRIVS.len() {
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

                db.update_channel_and_snapshot(
                    &channel.clone().unwrap().get_id(),
                    &channel.clone().unwrap(),
                    &testing_snapshot,
                )
                .await?;
            }

            if index > 0 {
                db.update_channel_and_snapshot(
                    &previous_channel.clone().unwrap().get_id(),
                    &previous_channel.clone().unwrap(),
                    &testing_snapshot,
                )
                .await?;
            }

            previous_channel = channel;
        }

        Ok(())
    }

    #[serial]
    #[async_std::test]
    pub async fn test_packet_acknowledgement_sender_workflow() {
        let _ = env_logger::builder().is_test(true).try_init();
        const TIMEOUT_SECONDS: u64 = 10;

        let (done_tx, mut done_rx) = futures::channel::mpsc::unbounded();

        let dbs = create_dbs(2);

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
            let secrets = (0..2).into_iter().map(|_| SharedSecret::random()).collect::<Vec<_>>();
            let porv = ProofOfRelayValues::new(&secrets[0], Some(&secrets[1]));

            // Mimics that the packet sender has sent a packet and now it has a pending acknowledgement in it's DB
            core_dbs[0]
                .write()
                .await
                .store_pending_acknowledgment(porv.ack_challenge.clone(), PendingAcknowledgement::WaitingAsSender)
                .await
                .expect("failed to store pending ack");

            // This is what counterparty derives and sends back to solve the challenge
            let ack_key = derive_ack_key_share(&secrets[0]);

            sent_challenges.push((ack_key, porv.ack_challenge));
        }

        // Peer 1: ACK interaction of the packet sender, hookup receiving of acknowledgements and start processing them
        let ack_interaction_sender =
            AcknowledgementInteraction::new(core_dbs[0].clone(), &PEERS_CHAIN[0], Some(done_tx), None);

        // Peer 2: Recipient of the packet and sender of the acknowledgement
        let mut ack_interaction_counterparty =
            AcknowledgementInteraction::new(core_dbs[1].clone(), &PEERS_CHAIN[1], None, None);

        // Peer 2: start sending out outgoing acknowledgement
        for (ack_key, _) in sent_challenges.clone() {
            ack_interaction_counterparty
                .writer()
                .send_acknowledgement(
                    PEERS[0].public().to_peerid(),
                    Acknowledgement::new(ack_key, &OffchainKeypair::from_secret(&PEERS_PRIVS[1]).unwrap()),
                )
                .expect("failed to send ack");

            // emulate channel to another peer
            match ack_interaction_counterparty.next().await {
                Some(value) => match value {
                    AckProcessed::Send(_, ack) => ack_interaction_sender
                        .writer()
                        .receive_acknowledgement(PEERS[1].public().to_peerid(), ack)
                        .expect("failed to receive ack"),
                    _ => panic!("Unexpected incoming acknowledgement detected"),
                },
                None => panic!("There should have been an acknowledgment to send"),
            }
        }

        let finish = async move {
            for i in 1..PENDING_ACKS + 1 {
                let ack = done_rx.next().await.expect("failed finalize ack");
                debug!("sender has received acknowledgement {i}: {ack}");
                assert!(
                    sent_challenges.iter().find(|(_, c)| ack.eq(c)).is_some(),
                    "received invalid challenge {ack}"
                );
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

    async fn peer_setup_for(
        count: usize,
        ack_tkt_tx: UnboundedSender<AcknowledgedTicket>,
        ack_tx: UnboundedSender<HalfKeyChallenge>,
        pkt_tx: Sender<ApplicationData>,
    ) -> Vec<(AcknowledgementInteraction, PacketInteraction)> {
        let peer_count = count;

        assert!(peer_count <= PEERS.len());
        assert!(peer_count >= 3);
        let dbs = create_dbs(peer_count);

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
                let ack = AcknowledgementInteraction::new(
                    db.clone(),
                    &PEERS_CHAIN[i],
                    if i == 0 { Some(ack_tx.clone()) } else { None },
                    if i == peer_count - 2 {
                        Some(ack_tkt_tx.clone())
                    } else {
                        None
                    },
                );
                let pkt = PacketInteraction::new(
                    db.clone(),
                    Arc::new(RwLock::new(TagBloomFilter::default())),
                    Mixer::new(MixerConfig::default()),
                    ack.writer(),
                    if i == peer_count - 1 {
                        Some(pkt_tx.clone())
                    } else {
                        None
                    },
                    PacketInteractionConfig {
                        check_unrealized_balance: true,
                        packet_keypair: OffchainKeypair::from_secret(&PEERS_PRIVS[i]).unwrap(),
                        chain_keypair: ChainKeypair::from_secret(&PEERS_CHAIN_PRIVS[i]).unwrap(),
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
    ) {
        let component_length = components.len();

        for _ in 0..pending_packet_count {
            match components[0]
                .1
                .next()
                .await
                .expect("pkt_sender should have sent a packet")
            {
                MsgProcessed::Send(peer, data) => {
                    assert_eq!(peer, PEERS[1].public().to_peerid());
                    components[1]
                        .1
                        .writer()
                        .receive_packet(data, PEERS[0].public().to_peerid())
                        .expect("Send to relayer should succeed")
                }
                _ => panic!("Should have gotten a send request"),
            }
        }

        for i in 1..components.len() {
            for _ in 0..pending_packet_count {
                match components[i]
                    .0
                    .next()
                    .await
                    .expect("ACK relayer should send an ack to the previous")
                {
                    AckProcessed::Send(peer, ack) => {
                        assert_eq!(peer, PEERS[i - 1].public().to_peerid());
                        components[i - 1]
                            .0
                            .writer()
                            .receive_acknowledgement(PEERS[i].public().to_peerid(), ack)
                            .expect("Send of ack from relayer to sender should succeed")
                    }
                    _ => panic!("Should have gotten a send request"),
                }
            }

            for _ in 0..pending_packet_count {
                match components[i]
                    .1
                    .next()
                    .await
                    .expect("MSG relayer should forward a msg to the next")
                {
                    MsgProcessed::Forward(peer, data) => {
                        assert_eq!(peer, PEERS[i + 1].public().to_peerid());
                        assert_ne!(
                            i,
                            component_length - 1,
                            "Only intermediate peers can serve as a forwarder"
                        );
                        components[i + 1]
                            .1
                            .writer()
                            .receive_packet(data, PEERS[i].public().to_peerid())
                            .expect("Send of ack from relayer to receiver should succeed")
                    }
                    MsgProcessed::Receive(_peer, packet) => {
                        ApplicationData::from_bytes(&packet).expect("could not deserialize app data");
                        assert_eq!(i, component_length - 1, "Only the last peer can be a recipient");
                    }
                    _ => panic!("Should have gotten a send request or a final packet"),
                }
            }
        }
    }

    async fn packet_relayer_workflow_n_peers(peer_count: usize, pending_packets: usize) {
        assert!(peer_count >= 3, "invalid peer count given");
        assert!(pending_packets >= 1, "at least one packet must be given");

        let (pkt_tx, pkt_rx) = futures::channel::mpsc::channel::<ApplicationData>(100);
        let (ack_tkt_tx, ack_tkt_rx) = futures::channel::mpsc::unbounded::<AcknowledgedTicket>();
        let (ack_tx, ack_rx) = futures::channel::mpsc::unbounded::<HalfKeyChallenge>();

        const TIMEOUT_SECONDS: u64 = 10;

        let test_msgs = (0..pending_packets)
            .map(|i| ApplicationData {
                application_tag: (i == 0).then(|| random_integer(1, Some(65535)).unwrap() as Tag),
                plain_text: random_bytes::<300>().into(),
            })
            .collect::<Vec<_>>();

        let components = peer_setup_for(peer_count, ack_tkt_tx, ack_tx, pkt_tx).await;

        // Peer 1: start sending out packets
        let packet_path = Path::new_valid(
            PEERS[1..peer_count]
                .iter()
                .map(|p| p.public().to_peerid())
                .collect::<Vec<PeerId>>(),
        );
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
            Either::Left(_) => true,
            Either::Right(_) => false,
        };

        // Check that we received all acknowledgements and packets
        let finish = join3(
            ack_rx.collect::<Vec<_>>(),
            ack_tkt_rx.collect::<Vec<_>>(),
            pkt_rx.collect::<Vec<_>>(),
        );
        let timeout = async_std::task::sleep(Duration::from_secs(TIMEOUT_SECONDS));
        pin_mut!(finish, timeout);

        let succeeded = succeeded
            && match select(finish, timeout).await {
                Either::Left(((acks, ack_tkts, pkts), _)) => {
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

                    assert_eq!(pkts.len(), pending_packets, "did not receive all packets");
                    assert!(
                        test_msgs.iter().all(|m| pkts.contains(m)),
                        "some received packet data does not match"
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
