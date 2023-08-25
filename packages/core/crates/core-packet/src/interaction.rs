use async_lock::RwLock;
use core_mixer::future_extensions::StreamThenConcurrentExt;
use futures::future::{poll_fn, Either};
use futures::channel::mpsc::{channel, UnboundedSender, Receiver, Sender};
use std::fmt::{Display, Formatter};
use std::pin::Pin;

use crate::errors::PacketError::{
    AcknowledgementValidation, ChannelNotFound, InvalidPacketState, OutOfFunds, PacketConstructionError,
    PacketDecodingError, PathError, Retry, TagReplay, TransportError,
};
use crate::errors::Result;
use crate::packet::{Packet, PacketState, PAYLOAD_SIZE};

use core_crypto::keypairs::{ChainKeypair, OffchainKeypair};
use core_crypto::types::{HalfKeyChallenge, Hash, OffchainPublicKey};
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_mixer::mixer::{Mixer, MixerConfig};
use core_path::errors::PathError::PathNotValid;
use core_path::path::Path;
use core_types::acknowledgement::{AcknowledgedTicket, Acknowledgement, PendingAcknowledgement, UnacknowledgedTicket};
use core_types::channels::Ticket;
use futures::{stream::Stream, pin_mut, StreamExt};
use libp2p_identity::PeerId;
use serde::{Deserialize, Serialize};
use std::ops::Mul;
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
use utils_types::errors::GeneralError::ParseError;

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

/// Fixed price per packet to 0.01 HOPR
pub const PRICE_PER_PACKET: &str = "10000000000000000";
/// Fixed inverse ticket winning probability
pub const INVERSE_TICKET_WIN_PROB: &str = "1";

/// Represents a payload (packet or acknowledgement) at the transport level.
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Clone)]
pub struct Payload {
    remote_peer: PeerId,
    data: Box<[u8]>,
}

impl Display for Payload {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Payload")
            .field("remote_peer", &self.remote_peer)
            .field("data", &hex::encode(&self.data))
            .finish()
    }
}

/// Tags are currently 16-bit unsigned integers
pub type Tag = u16;

/// Represent a default application tag if none is specified in `send_packet`.
pub const DEFAULT_APPLICATION_TAG: Tag = 0;

/// Represents the received decrypted packet carrying the application-layer data.
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApplicationData {
    pub application_tag: Option<Tag>,
    #[serde(with = "serde_bytes")]
    pub plain_text: Box<[u8]>,
}

impl Display for ApplicationData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}): {}",
            self.application_tag.unwrap_or(DEFAULT_APPLICATION_TAG),
            hex::encode(&self.plain_text)
        )
    }
}

impl AsRef<[u8]> for ApplicationData {
    fn as_ref(&self) -> &[u8] {
        &self.plain_text
    }
}

impl BinarySerializable for ApplicationData {
    const SIZE: usize = 2; // minimum size

    fn from_bytes(data: &[u8]) -> utils_types::errors::Result<Self> {
        if data.len() <= PAYLOAD_SIZE && data.len() >= Self::SIZE {
            let tag = u16::from_be_bytes(data[0..2].try_into().map_err(|_| ParseError)?);
            Ok(Self {
                application_tag: if tag != DEFAULT_APPLICATION_TAG {
                    Some(tag)
                } else {
                    None
                },
                plain_text: (&data[2..]).into(),
            })
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut buf = Vec::with_capacity(Self::SIZE + self.plain_text.len());
        let tag = self.application_tag.unwrap_or(DEFAULT_APPLICATION_TAG);
        buf.extend_from_slice(&tag.to_be_bytes());
        buf.extend_from_slice(&self.plain_text);
        buf.into_boxed_slice()
    }
}

// Default sizes of the acknowledgement queues
pub const ACK_TX_QUEUE_SIZE: usize = 2048;
pub const ACK_RX_QUEUE_SIZE: usize = 2048;

#[derive(Debug)]
pub enum AckToProcess {
    ToReceive(PeerId, Acknowledgement),
    ToSend(PeerId, Acknowledgement),
}

#[derive(Debug)]
pub enum AckProcessed {
    Receive(PeerId, Result<()>),
    Send(PeerId, Acknowledgement)
}

/// Implements protocol acknowledgement logic for acknowledgements
pub struct AcknowledgementProcessor<Db: HoprCoreEthereumDbActions> {
    db: Arc<RwLock<Db>>,
    pub on_acknowledgement: Option<UnboundedSender<HalfKeyChallenge>>,
    pub on_acknowledged_ticket: Option<UnboundedSender<AcknowledgedTicket>>,
}

impl<Db: HoprCoreEthereumDbActions> Clone for AcknowledgementProcessor<Db> {
    fn clone(&self) -> Self {
        Self { db: self.db.clone(), on_acknowledgement: self.on_acknowledgement.clone(), on_acknowledged_ticket: self.on_acknowledged_ticket.clone() }
    }
}

impl<Db: HoprCoreEthereumDbActions> AcknowledgementProcessor<Db> {
    pub fn new(db: Arc<RwLock<Db>>, on_acknowledgement: Option<UnboundedSender<HalfKeyChallenge>>, on_acknowledged_ticket: Option<UnboundedSender<AcknowledgedTicket>>) -> Self {
        Self { db, on_acknowledgement, on_acknowledged_ticket }
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
                let response = unackowledged.get_response(&ack.ack_key_share)?;
                debug!("acknowledging ticket using response {}", response.to_hex());

                let ack_ticket = AcknowledgedTicket::new(unackowledged.ticket, response, unackowledged.signer);

                // replace the un-acked ticket with acked ticket.
                self.db
                    .write()
                    .await
                    .replace_unack_with_ack(&ack.ack_challenge(), ack_ticket.clone())
                    .await?;

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
    pub queue: Sender<AckToProcess>
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
        self.queue
            .try_send(event)
            .map_err(|e| {
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
        on_acknowledgement: Option<UnboundedSender<HalfKeyChallenge>>,
        on_acknowledged_ticket: Option<UnboundedSender<AcknowledgedTicket>>,
    ) -> Self {
        let (processing_in_tx, processing_in_rx) = channel::<AckToProcess>(ACK_RX_QUEUE_SIZE + ACK_TX_QUEUE_SIZE);
        let (processing_out_tx, processing_out_rx) = channel::<AckProcessed>(ACK_RX_QUEUE_SIZE + ACK_TX_QUEUE_SIZE);
        
        let processor = AcknowledgementProcessor::new(db, on_acknowledgement, on_acknowledged_ticket);

        let processing_stream = processing_in_rx
        .then_concurrent(move |event| {
            let mut processor = processor.clone();
            let mut processed_tx = processing_out_tx.clone();

            async move {
                let processed: Option<AckProcessed> = match event {
                    AckToProcess::ToReceive(peer, mut ack) => {
                        if let Ok(remote_pk) = OffchainPublicKey::from_peerid(&peer) {
                            debug!("validating incoming acknowledgement from {}", peer);
                            if ack.validate(&remote_pk) {
                                match processor.handle_acknowledgement(ack).await {
                                    Ok(_) => Some(AckProcessed::Receive(peer, Ok(()))),
                                    Err(e) => {
                                        error!("Encountered error while handling acknowledgement from peer '{}': {}", &peer, e);
                                        None
                                    }
                                }
                            } else {
                                error!(
                                    "failed to verify signature on acknowledgement from peer {}",
                                    peer
                                );
                                None
                            }
                        } else {
                            error!("invalid remote peer id {}", peer);
                            None
                        }
                    },
                    AckToProcess::ToSend(peer, ack) => Some(AckProcessed::Send(peer, ack))
                };

                if let Some(event) = processed {
                    match poll_fn(|cx| Pin::new(&mut processed_tx).poll_ready(cx)).await {
                        Ok(_) => {
                            match processed_tx.start_send(event) {
                                Ok(_) => {},
                                Err(e) => error!("Failed to pass a processed ack message: {}", e),
                            }
                        },
                        Err(e) => {
                            warn!("The receiver for processed ack no longer exists: {}", e);
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
            ack_event_queue: (processing_in_tx, processing_out_rx),
        }
    }

    pub fn writer(&self) -> AcknowledgementActions {
        AcknowledgementActions { queue: self.ack_event_queue.0.clone() }
    }
}

impl Stream for AcknowledgementInteraction {
    type Item = AckProcessed;

    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {   
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
    ToForward(Box<[u8]>, PeerId)
}

#[derive(Debug)]
pub enum MsgProcessed {
    Receive(PeerId, Box<[u8]>),
    Send(PeerId, Box<[u8]>),
    Forward(PeerId, Box<[u8]>,)
}

/// Implements protocol acknowledgement logic for msg packets
pub struct PacketProcessor<Db>
where
    Db: HoprCoreEthereumDbActions,
{
    db: Arc<RwLock<Db>>,
    cfg: PacketInteractionConfig,
}

impl<Db> Clone for PacketProcessor<Db>
where
    Db: HoprCoreEthereumDbActions
{
    fn clone(&self) -> Self {
        Self { db: self.db.clone(), cfg: self.cfg.clone() }
    }
}

pub enum PacketType {
    Final(Packet, Option<Acknowledgement>),
    Forward(Packet, Option<Acknowledgement>, PeerId, PeerId)
}

impl<Db> PacketProcessor<Db>
where
    Db: HoprCoreEthereumDbActions,
{
    /// Creates a new instance given the DB and configuration.
    pub fn new(db: Arc<RwLock<Db>>, cfg: PacketInteractionConfig) -> Self {
        Self {
            db,
            cfg,
        }
    }

    async fn bump_ticket_index(&self, channel_id: &Hash) -> Result<U256> {
        let current_ticket_index = self
            .db
            .read()
            .await
            .get_current_ticket_index(channel_id)
            .await?
            .unwrap_or(U256::one());

        self.db
            .write()
            .await
            .set_current_ticket_index(channel_id, current_ticket_index.addn(1))
            .await?;

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
        let amount = Balance::new(
            U256::new(PRICE_PER_PACKET)
                .mul(U256::new(INVERSE_TICKET_WIN_PROB))
                .muln(path_pos as u32 - 1),
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

        let ticket = Ticket::new(
            destination,
            current_index,
            amount,
            U256::from_inverse_probability(U256::new(INVERSE_TICKET_WIN_PROB))?,
            channel.channel_epoch,
            &self.cfg.chain_keypair,
        );

        self.db.write().await.mark_pending(&ticket).await?;

        debug!(
            "Creating ticket in channel {channel_id}. Ticket data: {}",
            ticket.to_hex()
        );

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_TICKETS_COUNT.increment();

        Ok(ticket)
    }

    pub async fn create_packet_from_me(&self, data: Box<[u8]>, path: Path) -> Result<(Payload, HalfKeyChallenge)> {
        let next_peer = self
            .db
            .read()
            .await
            .get_chain_key(&OffchainPublicKey::from_peerid(&path.hops()[0])?)
            .await?
            .ok_or(PacketConstructionError)?;

        // Decide whether to create 0-hop or multihop ticket
        let next_ticket = if path.length() == 1 {
            Ticket::new_zero_hop(next_peer, &self.cfg.chain_keypair)
        } else {
            self.create_multihop_ticket(next_peer, path.length() as u8).await?
        };

        // Create the packet
        let packet = Packet::new(&data, &path, &self.cfg.chain_keypair, next_ticket)?;
        debug!("packet state {}", packet.state());
        match packet.state() {
            PacketState::Outgoing { ack_challenge, .. } => {
                self.db
                    .write()
                    .await
                    .store_pending_acknowledgment(*ack_challenge, PendingAcknowledgement::WaitingAsSender)
                    .await?;

                Ok((Payload {
                    remote_peer: path.hops()[0],
                    data: packet.to_bytes(),
                }, ack_challenge.clone()))
            }
            _ => {
                debug!("invalid packet state {:?}", packet.state());
                Err(crate::errors::PacketError::Other(
                    utils_types::errors::GeneralError::Other("invalid packet state".into()),
                ))
            }
        }
    }

    pub fn create_packet_from_bytes(&self, data: &[u8], peer: &PeerId) -> Result<Packet> {
        Packet::from_bytes(data, &self.cfg.packet_keypair, peer)
    }

    pub async fn handle_mixed_packet(&self, mut packet: Packet) -> Result<PacketType> {
        let next_ticket;
        let previous_peer;
        let next_peer;

        match packet.state() {
            PacketState::Outgoing { .. } => return Err(InvalidPacketState),

            PacketState::Final {
                packet_tag,
                ..
            } => {
                // Validate if it's not a replayed packet
                if self.db.write().await.check_and_set_packet_tag(packet_tag).await? {
                    return Err(TagReplay);
                }

                let ack = packet.create_acknowledgement(&self.cfg.packet_keypair);
                return Ok(PacketType::Final(packet, ack))
            }

            PacketState::Forwarded {
                ack_challenge,
                previous_hop,
                own_key,
                next_hop,
                packet_tag,
                ..
            } => {
                // Validate if it's not a replayed packet
                if self.db.write().await.check_and_set_packet_tag(packet_tag).await? {
                    return Err(TagReplay);
                }

                let inverse_win_prob = U256::new(INVERSE_TICKET_WIN_PROB);

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
                if let Err(e) = validate_unacknowledged_ticket::<Db>(
                    &*self.db.read().await,
                    &packet.ticket,
                    &channel,
                    &previous_hop_addr,
                    Balance::from_str(PRICE_PER_PACKET, BalanceType::HOPR),
                    inverse_win_prob,
                    self.cfg.check_unrealized_balance,
                )
                .await
                {
                    // Mark as reject and passthrough the error
                    self.db.write().await.mark_rejected(&packet.ticket).await?;
                    return Err(e);
                }

                {
                    let mut g = self.db.write().await;
                    g.set_current_ticket_index(&channel.get_id().hash(), packet.ticket.index)
                        .await?;

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

                let path_pos = packet
                    .ticket
                    .get_path_position(U256::new(PRICE_PER_PACKET), inverse_win_prob);

                // Create next ticket for the packet
                next_ticket = if path_pos == 1 {
                    Ticket::new_zero_hop(next_hop_addr, &self.cfg.chain_keypair)
                } else {
                    self.create_multihop_ticket(next_hop_addr, path_pos).await?
                };
                previous_peer = previous_hop.to_peerid();
                next_peer = next_hop.to_peerid();
            }
        }

        // Transform the packet for forwarding using the next ticket
        packet.forward(&self.cfg.chain_keypair, next_ticket)?;

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
                Ok(_) => {},
                Err(_) => {
                    error!("Failed to notify the awaiter about the successful packet transmission")
                },
            }
        } else {
            error!("Sender for packet send signalization is already spent")
        }
    }
}

/// Await on future until the confirmation of packet reception is received
#[derive(Debug)]
pub struct PacketSendAwaiter {
    rx: Option<futures::channel::oneshot::Receiver<HalfKeyChallenge>>
}

impl From<futures::channel::oneshot::Receiver<HalfKeyChallenge>> for PacketSendAwaiter {
    fn from(value: futures::channel::oneshot::Receiver<HalfKeyChallenge>) -> Self {
        Self { rx: Some(value) }
    }
}

#[cfg(all(feature = "wasm", not(test)))]
use gloo_timers::future::sleep;
#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::sleep;

impl PacketSendAwaiter {
    pub async fn consume_and_wait(&mut self, until_timeout: std::time::Duration) -> Result<HalfKeyChallenge> {
        match self.rx.take() {
            Some(resolve) => {
                let timeout = sleep(until_timeout);
                pin_mut!(resolve, timeout);
                match futures::future::select(resolve, timeout).await {
                    Either::Left((challenge, _)) => {
                        challenge.map_err(|_| TransportError("Canceled".to_owned()))
                    },
                    Either::Right(_) => Err(TransportError("Timed out on sending a packet".to_owned())),
                }
            },
            None => Err(TransportError("Packet send process observation already consumed".to_owned())),
        }
    }
}

/// External API for feeding Packet actions into the Packet processor
#[derive(Debug, Clone)]
pub struct PacketActions {
    pub queue: Sender<MsgToProcess>
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
        self.queue
            .try_send(event)
            .map_err(|e| {
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
    pub fn new(packet_keypair: OffchainKeypair, chain_keypair: ChainKeypair) -> Self {
        Self {
            packet_keypair,
            chain_keypair,
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
    pub fn new<Db: HoprCoreEthereumDbActions + 'static>(db: Arc<RwLock<Db>>, mixer: Mixer, ack_interaction: AcknowledgementActions, on_final_packet: Option<Sender<ApplicationData>>, cfg: PacketInteractionConfig) -> Self {
        let (to_process_tx, to_process_rx) = channel::<MsgToProcess>(PACKET_RX_QUEUE_SIZE + PACKET_TX_QUEUE_SIZE);
        let (processed_tx, processed_rx) = channel::<MsgProcessed>(PACKET_RX_QUEUE_SIZE + PACKET_TX_QUEUE_SIZE);
        
        let processor = PacketProcessor::new(db, cfg);

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
                        match processor.create_packet_from_me(data, path).await {
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
        PacketActions { queue: self.ack_event_queue.0.clone() }
    }
}

impl Stream for PacketInteraction {
    type Item = MsgProcessed;

    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {   
        use futures_lite::stream::StreamExt;     
        return std::pin::Pin::new(self).ack_event_queue.1.poll_next(cx);
    }
}


#[cfg(feature = "wasm")]
mod wasm {
    use std::str::FromStr;

    use crate::interaction::Payload;

    use super::ApplicationData;
    use libp2p_identity::PeerId;
    use utils_misc::{utils::wasm::JsResult, ok_or_jserr};
    use utils_types::traits::BinarySerializable;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    impl ApplicationData {
        #[wasm_bindgen(constructor)]
        pub fn _new(tag: Option<u16>, data: Box<[u8]>) -> Self {
            Self {
                application_tag: tag,
                plain_text: data,
            }
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8]) -> JsResult<ApplicationData> {
            ok_or_jserr!(Self::from_bytes(data))
        }
    }

    #[wasm_bindgen]
    impl Payload {
        #[wasm_bindgen(constructor)]
        pub fn _new(peer_id: &str, packet_data: Box<[u8]>) -> JsResult<Payload> {
            Ok(Self {
                remote_peer: ok_or_jserr!(PeerId::from_str(peer_id))?,
                data: packet_data,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::interaction::{AcknowledgementInteraction, PacketInteraction, PacketInteractionConfig, PRICE_PER_PACKET, AckProcessed, MsgProcessed, ApplicationData};
    use crate::por::ProofOfRelayValues;
    use async_std::sync::RwLock;
    use core_crypto::derivation::derive_ack_key_share;
    use core_crypto::types::{PublicKey, HalfKeyChallenge, OffchainPublicKey};
    use core_ethereum_db::{db::CoreEthereumDb, traits::HoprCoreEthereumDbActions};
    use core_mixer::mixer::{MixerConfig, Mixer};
    use core_types::acknowledgement::{Acknowledgement, PendingAcknowledgement, AcknowledgedTicket};
    use core_crypto::keypairs::{ChainKeypair, Keypair, OffchainKeypair};
    use core_crypto::shared_keys::SharedSecret;
    use core_crypto::random::random_bytes;
    use core_path::path::Path;
    use core_types::channels::{ChannelEntry, ChannelStatus};
    use futures::channel::mpsc::{UnboundedSender, Sender};
    use futures::future::{select, Either};
    use futures::{pin_mut, StreamExt};
    use hex_literal::hex;
    use lazy_static::lazy_static;
    use libp2p_identity::PeerId;
    use serial_test::serial;
    use std::ops::Mul;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use utils_db::db::DB;
    use utils_db::leveldb::rusty::RustyLevelDbShim;
    use utils_log::debug;
    use utils_types::primitives::{Balance, BalanceType, Snapshot, U256};
    use utils_types::traits::{PeerIdLike, ToHex, BinarySerializable};

    use super::{PacketSendFinalizer, PacketSendAwaiter};

    #[async_std::test]
    pub async fn test_packet_send_finalizer_succeeds_with_a_stored_challenge() {
        let (tx, rx) = futures::channel::oneshot::channel::<HalfKeyChallenge>();

        let mut finalizer = PacketSendFinalizer::new(tx);
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

    lazy_static! {
        static ref PEERS: Vec<PeerId> = PEERS_PRIVS
            .iter()
            .map(|private| OffchainPublicKey::from_privkey(private).unwrap().to_peerid())
            .collect();
    }

    async fn create_dummy_channel(db: &CoreEthereumDb<RustyLevelDbShim>, from: &PeerId, to: &PeerId) -> ChannelEntry {
        let source = db
            .get_chain_key(&OffchainPublicKey::from_peerid(from).unwrap())
            .await
            .unwrap()
            .expect("failed to retrieve source address");
        let destination = db
            .get_chain_key(&OffchainPublicKey::from_peerid(to).unwrap())
            .await
            .unwrap()
            .expect("failed to retrieve destination address");

        ChannelEntry::new(
            source,
            destination,
            Balance::new(U256::new("1234").mul(U256::new(PRICE_PER_PACKET)), BalanceType::HOPR),
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
                    PublicKey::from_privkey(&PEERS_PRIVS[i]).unwrap().to_address(),
                )))
            })
            .collect::<Vec<_>>()
    }

    async fn create_minimal_topology(dbs: &Vec<Arc<Mutex<rusty_leveldb::DB>>>) -> crate::errors::Result<()> {
        let testing_snapshot = Snapshot::default();
        let mut previous_channel: Option<ChannelEntry> = None;

        for (index, peer_key) in PEERS_PRIVS.iter().enumerate().take(dbs.len()) {
            let mut db = CoreEthereumDb::new(
                DB::new(RustyLevelDbShim::new(dbs[index].clone())),
                PublicKey::from_privkey(peer_key)?.to_address(),
            );

            // Link all the node keys and chain keys from the simulated announcements
            for peer_key in PEERS_PRIVS {
                let node_key = OffchainPublicKey::from_privkey(&peer_key)?;
                let chain_key = PublicKey::from_privkey(&peer_key)?;
                db.link_chain_and_packet_keys(&chain_key.to_address(), &node_key, &Snapshot::default())
                    .await?;
            }

            let mut channel: Option<ChannelEntry> = None;
            let this_peer = OffchainPublicKey::from_privkey(peer_key)?.to_peerid();

            if index < PEERS.len() - 1 {
                channel = Some(create_dummy_channel(&db, &this_peer, &PEERS[index + 1]).await);

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
        debug!("peer 1 (sender)    = {}", PEERS[0]);
        debug!("peer 2 (recipient) = {}", PEERS[1]);

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
        let ack_interaction_sender = AcknowledgementInteraction::new(
            core_dbs[0].clone(),
            Some(done_tx),
            None,
        );

        // Peer 2: Recipient of the packet and sender of the acknowledgement
        let mut ack_interaction_counterparty = AcknowledgementInteraction::new(
            core_dbs[1].clone(),
            None,
            None,
        );

        // Peer 2: start sending out outgoing acknowledgement
        for (ack_key, _) in sent_challenges.clone() {
            ack_interaction_counterparty
                .writer()
                .send_acknowledgement(
                    PEERS[0].clone(),
                    Acknowledgement::new(ack_key, &OffchainKeypair::from_secret(&PEERS_PRIVS[1]).unwrap()),
                )
                .expect("failed to send ack");

            // emulate channel to another peer
            match ack_interaction_counterparty.next().await {
                Some(value) => match value {
                    AckProcessed::Send(_, ack) => {
                        ack_interaction_sender.writer().receive_acknowledgement(PEERS[1].clone(), ack).expect("Should succeed")
                    },
                    _ => panic!("Unexpected incoming acknowledgement detected")
                }
                None => panic!("There should have been an acknowledgment to send")
            }
        }

        let finish = async move {
            for i in 1..PENDING_ACKS + 1 {
                let ack = done_rx.next().await.expect("failed finalize ack");
                debug!("sender has received acknowledgement: {}", ack.to_hex());
                if let Some(_) = sent_challenges.iter().find(|(_, chal)| chal.eq(&ack)) {
                    // TODO: check solution of ack challenge
                    /*assert!(
                        ack_msg.solve(&ack_key.to_bytes()),
                        "acknowledgement key must solve acknowledgement challenge"
                    );*/

                    // If it matches, set a signal that the test has finished
                    debug!("peer 1 received expected ack");
                }
                debug!("done ack #{i} out of {PENDING_ACKS}");
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

    async fn peer_setup_for(count: usize, ack_tx: UnboundedSender<AcknowledgedTicket>, pkt_tx: Sender<ApplicationData>) -> Vec<(AcknowledgementInteraction, PacketInteraction)> {
        let peer_count = count;
        
        assert!(peer_count <= PEERS.len());
        assert!(peer_count >= 3);
        let dbs = create_dbs(peer_count);

        create_minimal_topology(&dbs)
            .await
            .expect("failed to create minimal channel topology");

        let core_dbs = create_core_dbs(&dbs);

        // Begin tests
        for i in 0..peer_count {
            let peer_type = {
                if i == 0 { "sender" } else if i == (peer_count - 1) { "recipient" } else { "relayer" }
            };

            debug!("peer {i} ({peer_type})    = {}", PEERS[i]);
        }

        core_dbs
            .into_iter()
            .enumerate()
            .map(|(i, db)| {
                let ack = AcknowledgementInteraction::new(
                    db.clone(),
                    None,
                    if i == peer_count - 2 { Some(ack_tx.clone()) } else { None }
                );
                let pkt = PacketInteraction::new(
                    db.clone(),
                    Mixer::new(MixerConfig::default()),
                    ack.writer(),
                    if i == peer_count - 1 { Some(pkt_tx.clone()) } else { None },
                    PacketInteractionConfig {
                        check_unrealized_balance: true,
                        packet_keypair: OffchainKeypair::from_secret(&PEERS_PRIVS[i]).unwrap(),
                        chain_keypair: ChainKeypair::from_secret(&PEERS_PRIVS[i]).unwrap(),
                        mixer: MixerConfig::default(),      // TODO: unnecessary, can be removed
                    },
                );

                (ack, pkt)
            })
            .collect::<Vec<_>>()
    }

    async fn emulate_channel_communication(pending_packet_count: usize, mut components: Vec<(AcknowledgementInteraction, PacketInteraction)>, expected_msg: ApplicationData) {
        let component_length = components.len();

        for _ in 0..pending_packet_count {
            match components[0].1.next().await.expect("pkt_sender should have sent a packet") {
                MsgProcessed::Send(peer, data) => {
                    assert_eq!(peer, PEERS[1]);
                    components[1].1.writer().receive_packet(data, PEERS[0]).expect("Send to relayer should succeed")
                },
                _ => panic!("Should have gotten a send request")
            }
        }

        for i in 1..components.len() {
            for _ in 0..pending_packet_count {
                match components[i].0.next().await.expect("ACK relayer should send an ack to the previous") {
                    AckProcessed::Send(peer, ack) => {
                        assert_eq!(peer, PEERS[i-1]);
                        components[i-1].0.writer().receive_acknowledgement(PEERS[i], ack).expect("Send of ack from relayer to sender should succeed")
                    },
                    _ => panic!("Should have gotten a send request")
                }
            }

            for _ in 0..pending_packet_count {
                match components[i].1.next().await.expect("MSG relayer should forward a msg to the next") {
                    MsgProcessed::Forward(peer, data) => {
                        assert_eq!(peer, PEERS[i+1]);
                        assert!(i != component_length - 1, "Only intermediate peers can serve as a forwarder");
                        components[i+1].1.writer().receive_packet(data, PEERS[i]).expect("Send of ack from relayer to receiver should succeed")
                    },
                    MsgProcessed::Receive(_peer, packet) => {
                        let recv_app_data = ApplicationData::from_bytes(&packet).expect("could not deserialize app data");
                        assert_eq!(i, component_length - 1, "Only the last peer can be a recepient");
                        assert_eq!(expected_msg, recv_app_data, "received packet payload must match");
                    }
                    _ => panic!("Should have gotten a send request or a final packet")
                }
            }
        }
    }

    #[serial]
    #[async_std::test]
    async fn test_packet_relayer_workflow_3_peers() {
        let _ = env_logger::builder().is_test(true).try_init();

        let (pkt_tx, mut pkt_rx) = futures::channel::mpsc::channel::<ApplicationData>(100);
        let (ack_tx, mut ack_rx) = futures::channel::mpsc::unbounded::<AcknowledgedTicket>();

        const PENDING_PACKETS: usize = 5;
        const TIMEOUT_SECONDS: u64 = 20;

        let test_msg = ApplicationData {
            application_tag: Some(10),
            plain_text: random_bytes::<300>().into()
        };

        let peer_count = 3;
        let components = peer_setup_for(peer_count, ack_tx, pkt_tx).await;

        // Peer 1: start sending out packets
        let packet_path = Path::new_valid(PEERS[1..peer_count].to_vec());
        assert_eq!(peer_count - 1, packet_path.length() as usize, "path has invalid length");

        for _ in 0..PENDING_PACKETS {
            components[0]
                .1
                .writer()
                .send_packet(test_msg.clone(), packet_path.clone())
                .expect("Packet should be sent successfully");
        }

        let channel = emulate_channel_communication(PENDING_PACKETS, components, test_msg.clone());
        let timeout = async_std::task::sleep(Duration::from_secs(TIMEOUT_SECONDS));
        pin_mut!(channel, timeout);

        let succeeded = match select(channel, timeout).await {
            Either::Left(_) => true,
            Either::Right(_) => false,
        };

        // Check that we received all acknowledgements and packets
        let finish = async move {
            let (mut acks, mut pkts) = (0, 0);
            for _ in 0..2 * PENDING_PACKETS {
                match select(ack_rx.next(), pkt_rx.next()).await {
                    Either::Left((_, _)) => {
                        acks += 1;
                    }
                    Either::Right((pkt, _)) => {
                        let msg = pkt.unwrap();
                        assert_eq!(test_msg, msg, "received packet payload must match");
                        pkts += 1;
                    }
                }
            }
            (acks, pkts)
        };

        let timeout = async_std::task::sleep(Duration::from_secs(TIMEOUT_SECONDS));
        pin_mut!(finish, timeout);

        let succeeded = match select(finish, timeout).await {
            Either::Left(((acks, pkts), _)) => {
                assert_eq!(acks, PENDING_PACKETS, "did not receive all acknowledgements");
                assert_eq!(pkts, PENDING_PACKETS, "did not receive all packets");
                succeeded && true
            }
            Either::Right(_) => false,
        };

        assert!(succeeded, "test timed out after {TIMEOUT_SECONDS} seconds");
    }

    #[serial]
    #[async_std::test]
    async fn test_packet_relayer_workflow_5_peers() {
        let _ = env_logger::builder().is_test(true).try_init();

        let (pkt_tx, mut pkt_rx) = futures::channel::mpsc::channel::<ApplicationData>(100);
        let (ack_tx, mut ack_rx) = futures::channel::mpsc::unbounded::<AcknowledgedTicket>();

        const PENDING_PACKETS: usize = 5;
        const TIMEOUT_SECONDS: u64 = 20;

        let test_msg = ApplicationData {
            application_tag: Some(10),
            plain_text: random_bytes::<300>().into()
        };

        let peer_count = 5;
        let components = peer_setup_for(peer_count, ack_tx, pkt_tx).await;

        // Peer 1: start sending out packets
        let packet_path = Path::new_valid(PEERS[1..peer_count].to_vec());
        assert_eq!(peer_count - 1, packet_path.length() as usize, "path has invalid length");

        for _ in 0..PENDING_PACKETS {
            components[0]
                .1
                .writer()
                .send_packet(test_msg.clone(), packet_path.clone())
                .expect("Packet should be sent successfully");
        }

        let channel = emulate_channel_communication(PENDING_PACKETS, components, test_msg.clone());
        let timeout = async_std::task::sleep(Duration::from_secs(TIMEOUT_SECONDS));
        pin_mut!(channel, timeout);

        let succeeded = match select(channel, timeout).await {
            Either::Left(_) => true,
            Either::Right(_) => false,
        };

        // Check that we received all acknowledgements and packets
        let finish = async move {
            let (mut acks, mut pkts) = (0, 0);
            for _ in 0..2 * PENDING_PACKETS {
                match select(ack_rx.next(), pkt_rx.next()).await {
                    Either::Left((_, _)) => {
                        acks += 1;
                    }
                    Either::Right((pkt, _)) => {
                        let msg = pkt.unwrap();
                        assert_eq!(test_msg, msg, "received packet payload must match");
                        pkts += 1;
                    }
                }
            }
            (acks, pkts)
        };

        let timeout = async_std::task::sleep(Duration::from_secs(TIMEOUT_SECONDS));
        pin_mut!(finish, timeout);

        let succeeded = match select(finish, timeout).await {
            Either::Left(((acks, pkts), _)) => {
                assert_eq!(acks, PENDING_PACKETS, "did not receive all acknowledgements");
                assert_eq!(pkts, PENDING_PACKETS, "did not receive all packets");
                succeeded && true
            }
            Either::Right(_) => false,
        };

        assert!(succeeded, "test timed out after {TIMEOUT_SECONDS} seconds");
    }
}
