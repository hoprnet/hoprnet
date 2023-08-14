use async_lock::RwLock;
use std::fmt::{Display, Formatter};

use crate::errors::PacketError::{
    AcknowledgementValidation, ChannelNotFound, InvalidPacketState, OutOfFunds, PacketConstructionError,
    PacketDecodingError, PathError, Retry, TagReplay, Timeout, TransportError,
};
use crate::errors::Result;
use crate::packet::{Packet, PacketState, PAYLOAD_SIZE};
use async_std::channel::{bounded, Receiver, Sender, TrySendError};
use core_crypto::keypairs::{ChainKeypair, OffchainKeypair};
use core_crypto::types::{HalfKeyChallenge, Hash, OffchainPublicKey};
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_mixer::mixer::{Mixer, MixerConfig};
use core_path::errors::PathError::PathNotValid;
use core_path::path::Path;
use core_types::acknowledgement::{AcknowledgedTicket, Acknowledgement, PendingAcknowledgement, UnacknowledgedTicket};
use core_types::channels::Ticket;
use futures::future::{select, Either};
use futures::pin_mut;
use libp2p_identity::PeerId;
use serde::{Deserialize, Serialize};
use std::ops::Mul;
use std::sync::Arc;
use std::time::Duration;
use utils_log::{debug, error, info};
use utils_types::primitives::{Address, Balance, BalanceType, U256};
use utils_types::traits::{BinarySerializable, PeerIdLike, ToHex};

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

const PREIMAGE_PLACE_HOLDER: [u8; Hash::SIZE] = [0xffu8; Hash::SIZE];

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
const ACK_TX_QUEUE_SIZE: usize = 2048;
const ACK_RX_QUEUE_SIZE: usize = 2048;

/// Implements protocol acknowledgement logic
/// Maintains TX and RX queues of `Payload` with the serialized `Acknowledgement` type.
/// Processing of each queue can be executed using `handle_incoming_acknowledgement` and
/// `handle_outgoing_acknowledgements` methods.
/// When a new acknowledgement is delivered from the transport the `received_acknowledgement`
/// method is used to push it into the processing queue of incoming acknowledgements.
/// Whan a new acknowledgement is about to be sent, the `send_acknowledgement` method is used
/// to push it into the processing queue of outgoing acknowledgements.
/// When no more processing needs to be done, the instance should be stopped via the `stop` method.
/// Once the instance is stopped, it cannot be restarted.
pub struct AcknowledgementInteraction<Db: HoprCoreEthereumDbActions> {
    db: Arc<RwLock<Db>>,
    pub on_acknowledgement: Option<Sender<HalfKeyChallenge>>,
    pub on_acknowledged_ticket: Option<Sender<AcknowledgedTicket>>,
    incoming_channel: (Sender<Payload>, Receiver<Payload>),
    outgoing_channel: (Sender<Payload>, Receiver<Payload>),
}

impl<Db: HoprCoreEthereumDbActions> AcknowledgementInteraction<Db> {
    /// Creates a new instance given the DB and our public key used to verify the acknowledgements.
    pub fn new(
        db: Arc<RwLock<Db>>,
        on_acknowledgement: Option<Sender<HalfKeyChallenge>>,
        on_acknowledged_ticket: Option<Sender<AcknowledgedTicket>>,
    ) -> Self {
        Self {
            db,
            incoming_channel: bounded(ACK_RX_QUEUE_SIZE),
            outgoing_channel: bounded(ACK_TX_QUEUE_SIZE),
            on_acknowledgement,
            on_acknowledged_ticket,
        }
    }

    /// Pushes the `Payload` received from the transport layer into processing.
    /// If `wait` is `true`, the method waits indefinitely if the RX queue is full until there's space.
    /// If `wait` is `false` and the RX queue is full, the method fails with `Err(Retry)`. At this point, the
    /// caller can decide whether to discard the acknowledgement.
    pub async fn received_acknowledgement(&self, payload: Payload, wait: bool) -> Result<()> {
        if wait {
            self.incoming_channel
                .0
                .send(payload)
                .await
                .map_err(|e| TransportError(e.to_string()))
        } else {
            self.incoming_channel.0.try_send(payload).map_err(|e| match e {
                TrySendError::Full(_) => Retry,
                TrySendError::Closed(_) => TransportError("queue is closed".to_string()),
            })
        }
    }

    /// Pushes a new outgoing acknowledgement into the processing given its destination.
    /// If `timeout` is given, the method waits the given time if the TX queue is full until there's space.
    /// If the `timeout` is zero, the method waits indefinitely.
    /// If `timeout` is `None` and the TX queue is full, the method fails with `Err(Retry)`
    pub async fn send_acknowledgement(
        &self,
        acknowledgement: Acknowledgement,
        destination: PeerId,
        timeout: Option<Duration>,
    ) -> Result<()> {
        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_SENT_ACKS.increment();

        if let Some(timeout_val) = timeout {
            let push = self.outgoing_channel.0.send(Payload {
                remote_peer: destination,
                data: acknowledgement.to_bytes(),
            });
            if timeout_val.is_zero() {
                let timeout = async_std::task::sleep(timeout_val);
                pin_mut!(push, timeout);
                match select(push, timeout).await {
                    Either::Left((res, _)) => res.map_err(|_| TransportError("queue is closed".to_string())),
                    Either::Right(_) => Err(Timeout(timeout_val.as_secs())),
                }
            } else {
                push.await.map_err(|_| TransportError("queue is closed".to_string()))
            }
        } else {
            self.outgoing_channel
                .0
                .try_send(Payload {
                    remote_peer: destination,
                    data: acknowledgement.to_bytes(),
                })
                .map_err(|e| match e {
                    TrySendError::Full(_) => Retry,
                    TrySendError::Closed(_) => TransportError("queue is closed".to_string()),
                })
        }
    }

    /// Start processing the incoming acknowledgement queue.
    pub async fn handle_incoming_acknowledgements(&self) {
        while let Ok(payload) = self.incoming_channel.1.recv().await {
            debug!("handle incoming acknowledgement loop iteration {payload}");

            match Acknowledgement::from_bytes(&payload.data) {
                Ok(mut ack) => {
                    if let Ok(remote_pk) = OffchainPublicKey::from_peerid(&payload.remote_peer) {
                        debug!("validating incoming acknowledgement from {}", payload.remote_peer);
                        if ack.validate(&remote_pk) {
                            if let Err(e) = self.handle_acknowledgement(ack).await {
                                error!(
                                    "failed to process incoming acknowledgement from {}: {e}",
                                    payload.remote_peer
                                );
                            }
                        } else {
                            error!(
                                "failed to verify signature on acknowledgement from peer {}",
                                payload.remote_peer
                            )
                        }
                    } else {
                        error!("invalid remote peer id {}", payload.remote_peer)
                    }
                }
                Err(e) => {
                    error!("received unreadable acknowledgement from {}: {e}", payload.remote_peer);
                }
            }
        }
        info!("done processing incoming acknowledgements");
    }

    /// Start processing the outgoing acknowledgement queue given the transport function.
    pub async fn handle_outgoing_acknowledgements<T, F>(&self, message_transport: &T)
    where
        T: Fn(Box<[u8]>, String) -> F,
        F: futures::Future<Output = core::result::Result<(), String>>,
    {
        while let Ok(payload) = self.outgoing_channel.1.recv().await {
            debug!("handle incoming acknowledgement loop iteration {payload}");

            if let Err(e) = message_transport(payload.data, payload.remote_peer.to_string()).await {
                error!("failed to send acknowledgement to {}: {e}", payload.remote_peer);
            }
        }
        info!("done processing outgoing acknowledgements")
    }

    /// Stop processing of TX and RQ queues.
    /// Cannot be restarted once stopped.
    pub fn stop(&self) {
        self.incoming_channel.0.close();
        self.incoming_channel.1.close();
        self.outgoing_channel.0.close();
        self.outgoing_channel.1.close();
    }

    async fn handle_acknowledgement(&self, ack: Acknowledgement) -> Result<()> {
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
                if let Some(emitter) = &self.on_acknowledgement {
                    if let Err(e) = emitter.try_send(ack.ack_challenge()) {
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

                let ack_ticket = AcknowledgedTicket::new(
                    unackowledged.ticket,
                    response,
                    Hash::new(&PREIMAGE_PLACE_HOLDER),
                    unackowledged.signer,
                );

                // replace the un-acked ticket with acked ticket.
                //debug!(">>> WRITE replacing unack with acked");
                self.db
                    .write()
                    .await
                    .replace_unack_with_ack(&ack.ack_challenge(), ack_ticket.clone())
                    .await?;
                //debug!("<<< WRITE replacing unack with acked");

                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_ACKED_TICKETS.increment();

                if let Some(emitter) = &self.on_acknowledged_ticket {
                    if let Err(e) = emitter.try_send(ack_ticket) {
                        error!("failed to emit acknowledged ticket: {e}");
                    }
                }
            }
        }
        Ok(())
    }
}

/// Configuration parameters for the packet interaction.
#[derive(Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
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

// Default sizes of the packet queues
const PACKET_TX_QUEUE_SIZE: usize = 2048;
const PACKET_RX_QUEUE_SIZE: usize = 2048;

/// Implements packet processing logic
/// Maintains TX and RX queues of `Payload` with the serialized `Packet` type.
/// Processing of each queue can be executed using `handle_incoming_packets` and
/// `handle_outgoing_packets` methods.
/// When a new packet is delivered from the transport the `received_packet`
/// method is used to push it into the processing queue of incoming packets.
/// Whan a new acknowledgement is about to be sent, the `send_packet` method is used
/// to push it into the processing queue of outgoing packets.
/// When no more processing needs to be done, the instance should be stopped via the `stop` method.
/// Once the instance is stopped, it cannot be restarted.
pub struct PacketInteraction<Db>
where
    Db: HoprCoreEthereumDbActions,
{
    db: Arc<RwLock<Db>>,
    incoming_packets: (Sender<Payload>, Receiver<Payload>),
    outgoing_packets: (Sender<Payload>, Receiver<Payload>),
    pub on_final_packet: Option<Sender<ApplicationData>>,
    pub mixer: Mixer<Payload>,
    cfg: PacketInteractionConfig,
}

impl<Db> PacketInteraction<Db>
where
    Db: HoprCoreEthereumDbActions,
{
    /// Creates a new instance given the DB and configuration.
    pub fn new(
        db: Arc<RwLock<Db>>,
        on_final_packet: Option<Sender<ApplicationData>>,
        cfg: PacketInteractionConfig,
    ) -> Self {
        Self {
            db,
            incoming_packets: bounded(PACKET_RX_QUEUE_SIZE),
            outgoing_packets: bounded(PACKET_TX_QUEUE_SIZE),
            on_final_packet,
            mixer: Mixer::new(cfg.mixer),
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

        //debug!(">>> WRITE bumping ticket index");
        self.db
            .write()
            .await
            .set_current_ticket_index(channel_id, current_ticket_index.addn(1))
            .await?;
        //debug!("<<< WRITE bumping ticket index");

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
            channel.ticket_epoch,
            current_index,
            amount,
            U256::from_inverse_probability(U256::new(INVERSE_TICKET_WIN_PROB))?,
            channel.channel_epoch,
            &self.cfg.chain_keypair,
        );

        //debug!(">>> WRITE mark_pending lock");
        self.db.write().await.mark_pending(&ticket).await?;
        //debug!("<<< WRITE mark_pending lock");

        debug!(
            "Creating ticket in channel {channel_id}. Ticket data: {}",
            ticket.to_hex()
        );

        // #[cfg(all(feature = "prometheus", not(test)))]
        // METRIC_TICKETS_COUNT.increment();

        Ok(ticket)
    }

    /// Pushes the packet with the given payload for sending via the given valid path.
    /// The application tag acts as an optional distinguisher for different application level protocols.
    /// If no `app_tag` is specified, the `DEFAULT_APPLICATION_TAG` is used.
    /// If `timeout` is given, the method waits the given time if the TX queue is full until there's space.
    /// If `timeout` is zero, the method waits indefinitely.
    /// If `timeout` is `None` and the TX queue is full, the method fails with `Err(Retry)`
    pub async fn send_packet(
        &self,
        msg: &[u8],
        app_tag: Option<Tag>,
        path: Path,
        timeout: Option<Duration>,
    ) -> Result<HalfKeyChallenge> {
        // Check if the path is valid
        if !path.valid() {
            return Err(PathError(PathNotValid));
        }

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

        // Create the packet with application tag
        let app_data = ApplicationData {
            application_tag: app_tag,
            plain_text: msg.into(),
        };

        let packet = Packet::new(&app_data.to_bytes(), &path, &self.cfg.chain_keypair, next_ticket)?;
        debug!("packet state {}", packet.state());
        match packet.state() {
            PacketState::Outgoing { ack_challenge, .. } => {
                //debug!(">>> WRITE store_pending_lock");
                self.db
                    .write()
                    .await
                    .store_pending_acknowledgment(*ack_challenge, PendingAcknowledgement::WaitingAsSender)
                    .await?;

                //debug!("<<< WRITE store_pending_lock");

                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_PACKETS_COUNT.increment();

                if let Some(timeout_value) = timeout {
                    let push = self.outgoing_packets.0.send(Payload {
                        remote_peer: path.hops()[0],
                        data: packet.to_bytes(),
                    });
                    if !timeout_value.is_zero() {
                        let timeout = async_std::task::sleep(timeout_value);
                        pin_mut!(push, timeout);
                        match select(push, timeout).await {
                            Either::Left((res, _)) => res.map_err(|_| TransportError("queue is closed".to_string())),
                            Either::Right(_) => Err(Timeout(timeout_value.as_secs())),
                        }
                    } else {
                        push.await.map_err(|_| TransportError("queue is closed".to_string()))
                    }
                } else {
                    self.outgoing_packets
                        .0
                        .try_send(Payload {
                            remote_peer: path.hops()[0],
                            data: packet.to_bytes(),
                        })
                        .map_err(|e| match e {
                            TrySendError::Full(_) => Retry,
                            TrySendError::Closed(_) => TransportError("queue is closed".to_string()),
                        })
                }?;

                Ok(*ack_challenge)
            }
            _ => {
                debug!("invalid packet state {:?}", packet.state());
                Err(crate::errors::PacketError::Other(
                    utils_types::errors::GeneralError::Other("invalid packet state".into()),
                ))
            }
        }
    }

    async fn handle_mixed_packet<T, F>(
        &self,
        mut packet: Packet,
        ack_interaction: Arc<AcknowledgementInteraction<Db>>,
        message_transport: &T,
    ) -> Result<()>
    where
        T: Fn(Box<[u8]>, String) -> F,
        F: futures::Future<Output = core::result::Result<(), String>>,
    {
        let next_ticket;
        let previous_peer;
        let next_peer;

        match packet.state() {
            PacketState::Outgoing { .. } => return Err(InvalidPacketState),

            PacketState::Final {
                plain_text,
                previous_hop,
                packet_tag,
                ..
            } => {
                // Validate if it's not a replayed packet
                //debug!(">>> WRITE check_and_set_packet_tag final lock");
                if self.db.write().await.check_and_set_packet_tag(packet_tag).await? {
                    return Err(TagReplay);
                }
                //debug!("<<< WRITE check_and_set_packet_tag final lock");

                // We're the destination of the packet, so emit the packet contents
                if let Some(emitter) = &self.on_final_packet {
                    let fd = ApplicationData::from_bytes(plain_text).map_err(|_| {
                        PacketDecodingError(format!("final plaintext is malformed: {}", hex::encode(plain_text)))
                    })?;
                    debug!("emitting final packet: {fd}");

                    if let Err(e) = emitter.try_send(fd) {
                        error!("failed to emit received final packet: {e}");
                    }
                }

                // And create acknowledgement, but do not fail completely if it fails
                let ack = packet.create_acknowledgement(&self.cfg.packet_keypair).unwrap();
                if let Err(e) = ack_interaction
                    .send_acknowledgement(ack, previous_hop.to_peerid(), None)
                    .await
                {
                    error!("failed acknowledge the final packet: {e}");
                }

                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_RECV_MESSAGE_COUNT.increment();

                return Ok(());
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
                //debug!(">>> WRITE check_and_set_packet_tag forwarded lock");
                if self.db.write().await.check_and_set_packet_tag(packet_tag).await? {
                    return Err(TagReplay);
                }
                //debug!("<<< WRITE check_and_set_packet_tag forwarded lock");

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
                    //debug!(">>> WRITE mark_rejected forwarded lock");
                    self.db.write().await.mark_rejected(&packet.ticket).await?;
                    //debug!("<<< WRITE mark_rejected forwarded lock");
                    return Err(e);
                }

                //debug!(">>> WRITE storing pending ack");
                {
                    let mut g = self.db.write().await;
                    g.set_current_ticket_index(&channel.get_id().hash(), packet.ticket.index)
                        .await?;

                    //debug!(">>> <<< updated current ticket index");

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
                //debug!("<<< WRITE storing pending ack");

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

        // Forward the packet to the next hop
        message_transport(packet.to_bytes(), next_peer.to_string())
            .await
            .map_err(TransportError)?;

        // Acknowledge to the previous hop that we forwarded the packet, but do not fail if the acknowledgement fails
        let ack = packet.create_acknowledgement(&self.cfg.packet_keypair).unwrap();
        if let Err(e) = ack_interaction.send_acknowledgement(ack, previous_peer, None).await {
            error!("failed to acknowledge relayed packet: {e}");
        }

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_FWD_MESSAGE_COUNT.increment();

        Ok(())
    }

    /// Starts handling of the outgoing packets using the given transport.
    pub async fn handle_outgoing_packets<T, F>(&self, message_transport: &T)
    where
        T: Fn(Box<[u8]>, String) -> F,
        F: futures::Future<Output = core::result::Result<(), String>>,
    {
        while let Ok(payload) = self.outgoing_packets.1.recv().await {
            debug!("handle outgoing packet loop iteration {payload}");
            // Send the packet
            if let Err(e) = message_transport(payload.data, payload.remote_peer.to_string()).await {
                error!("failed to send packet to {}: {e}", payload.remote_peer);
            }
        }
        info!("done sending packets")
    }

    /// Starts handling of the incoming packets using the given transport.
    /// The given acknowledgement interaction is used perform acknowledgements.
    pub async fn handle_incoming_packets<T, F>(
        &self,
        ack_interaction: Arc<AcknowledgementInteraction<Db>>,
        message_transport: &T,
    ) where
        T: Fn(Box<[u8]>, String) -> F,
        F: futures::Future<Output = core::result::Result<(), String>>,
    {
        while let Ok(payload) = self.incoming_packets.1.recv().await {
            debug!("handle incoming packet loop iteration {payload}");
            // Add some random delay via mixer
            let mixed_packet = self.mixer.mix(payload).await;
            match Packet::from_bytes(&mixed_packet.data, &self.cfg.packet_keypair, &mixed_packet.remote_peer) {
                Ok(packet) => {
                    if let Err(e) = self
                        .handle_mixed_packet(packet, ack_interaction.clone(), message_transport)
                        .await
                    {
                        error!("failed to handle packet from {}: {e}", mixed_packet.remote_peer);
                    }
                }
                Err(e) => {
                    error!("received unreadable packet from {}: {e}", mixed_packet.remote_peer);
                }
            }
        }
        info!("done processing packets")
    }

    /// Pushes the `Payload` received from the transport layer into processing.
    /// If `wait` is `true`, the method waits indefinitely if the RX queue is full until there's space.
    /// If `wait` is `false` and the RX queue is full, the method fails with `Err(Retry)`. At this point, the
    /// caller can decide whether to discard the packet.
    pub async fn received_packet(&self, payload: Payload, wait: bool) -> Result<()> {
        if wait {
            self.incoming_packets
                .0
                .send(payload)
                .await
                .map_err(|e| TransportError(e.to_string()))
        } else {
            self.incoming_packets.0.try_send(payload).map_err(|e| match e {
                TrySendError::Full(_) => Retry,
                TrySendError::Closed(_) => TransportError("queue is closed".to_string()),
            })
        }
    }

    /// Stop processing of TX and RQ queues.
    /// Cannot be restarted once stopped.
    pub fn stop(&self) {
        self.incoming_packets.0.close();
        self.incoming_packets.1.close();
        self.outgoing_packets.0.close();
        self.outgoing_packets.1.close();
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::PacketError::PacketDbError;
    use crate::interaction::{
        AcknowledgementInteraction, ApplicationData, PacketInteraction, PacketInteractionConfig, Payload,
        PRICE_PER_PACKET,
    };
    use crate::por::ProofOfRelayValues;
    use async_std::sync::RwLock;
    use core_crypto::derivation::derive_ack_key_share;
    use core_crypto::keypairs::{ChainKeypair, Keypair, OffchainKeypair};
    use core_crypto::random::random_bytes;
    use core_crypto::shared_keys::SharedSecret;
    use core_crypto::types::{Hash, OffchainPublicKey, PublicKey};
    use core_ethereum_db::db::CoreEthereumDb;
    use core_ethereum_db::traits::HoprCoreEthereumDbActions;
    use core_ethereum_misc::commitment::{initialize_commitment, ChannelCommitmentInfo};
    use core_mixer::mixer::MixerConfig;
    use core_path::path::Path;
    use core_types::acknowledgement::{Acknowledgement, PendingAcknowledgement};
    use core_types::channels::{ChannelEntry, ChannelStatus};
    use futures::future::{select, Either};
    use futures::pin_mut;
    use hex_literal::hex;
    use lazy_static::lazy_static;
    use libp2p_identity::PeerId;
    use serial_test::serial;
    use std::collections::HashMap;
    use std::ops::Mul;
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use utils_db::db::DB;
    use utils_db::errors::DbError;
    use utils_db::leveldb::rusty::RustyLevelDbShim;
    use utils_log::debug;
    use utils_types::primitives::{Balance, BalanceType, Snapshot, U256};
    use utils_types::traits::{BinarySerializable, PeerIdLike, ToHex};

    const PEERS_PRIVS: [[u8; 32]; 5] = [
        hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"),
        hex!("5bf21ea8cccd69aa784346b07bf79c84dac606e00eecaa68bf8c31aff397b1ca"),
        hex!("3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa"),
        hex!("db7e3e8fcac4c817aa4cecee1d6e2b4d53da51f9881592c0e1cc303d8a012b92"),
        hex!("0726a9704d56a013980a9077d195520a61b5aed28f92d89c50bca6e0e0c48cfc"),
    ];

    const ACK_PROTOCOL: usize = 0;
    const MSG_PROTOCOL: usize = 1;

    const TEST_MESSAGE: [u8; 8] = hex!["deadbeefcafebabe"];

    lazy_static! {
        static ref PEERS: Vec<PeerId> = PEERS_PRIVS
            .iter()
            .map(|private| OffchainPublicKey::from_privkey(private).unwrap().to_peerid())
            .collect();
        static ref MESSAGES: Mutex<[HashMap<PeerId, Vec<Msg<PeerId>>>; 2]> =
            Mutex::new([HashMap::new(), HashMap::new()]);
    }

    #[derive(Clone, Eq, PartialEq, Debug)]
    struct Msg<T> {
        pub from: T,
        pub to: T,
        pub data: Box<[u8]>,
    }

    fn init_transport() {
        let mut g = MESSAGES.lock().unwrap();
        g[ACK_PROTOCOL].clear();
        g[MSG_PROTOCOL].clear();

        for peer in PEERS.iter() {
            g[ACK_PROTOCOL].insert(peer.clone(), Vec::new());
            g[MSG_PROTOCOL].insert(peer.clone(), Vec::new());
        }
    }

    fn terminate_transport() {
        let mut g = MESSAGES.lock().unwrap();
        g[ACK_PROTOCOL].clear();
        g[MSG_PROTOCOL].clear();
    }

    async fn send_transport_as_peer<const PROTO: usize, const PEER_NUM: usize>(
        data: Box<[u8]>,
        dst: String,
    ) -> Result<(), String> {
        let from = PEERS[PEER_NUM];
        let to = PeerId::from_str(&dst).expect(&format!("invalid peer id: {dst}"));
        MESSAGES.lock().unwrap()[PROTO]
            .get_mut(&to)
            .expect(&format!("non existent channel: {to}"))
            .push(Msg { from, to, data });
        Ok(())
    }

    fn retrieve_transport_msgs_as_peer<const PROTO: usize, const PEER_NUM: usize>() -> Option<Vec<Msg<PeerId>>> {
        let for_peer = PEERS[PEER_NUM];
        Some(MESSAGES.lock().unwrap()[PROTO].get_mut(&for_peer)?.drain(..).collect())
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
            Hash::new(&random_bytes::<32>()),
            U256::zero(),
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

                let channel_info = ChannelCommitmentInfo {
                    chain_id: 1,
                    contract_address: "fakeaddress".to_string(),
                    channel_id: previous_channel.clone().unwrap().get_id().clone(),
                    channel_epoch: previous_channel.clone().unwrap().channel_epoch.clone(),
                };

                initialize_commitment(&mut db, &PEERS_PRIVS[0], &channel_info)
                    .await
                    .map_err(|e| PacketDbError(DbError::GenericError(e.to_string())))?;
            }

            previous_channel = channel;
        }

        Ok(())
    }

    fn spawn_ack_receive<Db: HoprCoreEthereumDbActions + 'static, const PEER_NUM: usize>(
        interaction: Arc<AcknowledgementInteraction<Db>>,
    ) {
        async_std::task::spawn_local(async move {
            while let Some(msgs) = retrieve_transport_msgs_as_peer::<ACK_PROTOCOL, PEER_NUM>() {
                for payload in msgs.into_iter().map(|m| Payload {
                    remote_peer: m.from,
                    data: m.data,
                }) {
                    debug!(
                        "received ack from {}: {}",
                        payload.remote_peer,
                        hex::encode(&payload.data)
                    );
                    interaction
                        .received_acknowledgement(payload, false)
                        .await
                        .expect("failed to receive ack");
                }
                async_std::task::sleep(Duration::from_millis(200)).await;
            }
        });
    }

    fn spawn_ack_send<Db: HoprCoreEthereumDbActions + 'static, const PEER_NUM: usize>(
        interaction: Arc<AcknowledgementInteraction<Db>>,
    ) {
        async_std::task::spawn_local(async move {
            interaction
                .handle_outgoing_acknowledgements(&send_transport_as_peer::<ACK_PROTOCOL, PEER_NUM>)
                .await;
        });
    }

    fn spawn_pkt_receive<Db: HoprCoreEthereumDbActions + 'static, const PEER_NUM: usize>(
        interaction: Arc<PacketInteraction<Db>>,
    ) {
        async_std::task::spawn_local(async move {
            while let Some(msgs) = retrieve_transport_msgs_as_peer::<MSG_PROTOCOL, PEER_NUM>() {
                for payload in msgs.into_iter().map(|m| Payload {
                    remote_peer: m.from,
                    data: m.data,
                }) {
                    debug!(
                        "received packet from {}: {}",
                        payload.remote_peer,
                        hex::encode(&payload.data)
                    );
                    interaction
                        .received_packet(payload, false)
                        .await
                        .expect("failed to receive ack");
                }
                async_std::task::sleep(Duration::from_millis(200)).await;
            }
        });
    }

    fn spawn_pkt_send<Db: HoprCoreEthereumDbActions + 'static, const PEER_NUM: usize>(
        interaction: Arc<PacketInteraction<Db>>,
    ) {
        async_std::task::spawn_local(async move {
            interaction
                .handle_outgoing_packets(&send_transport_as_peer::<MSG_PROTOCOL, PEER_NUM>)
                .await;
        });
    }

    fn spawn_pkt_handling<Db: HoprCoreEthereumDbActions + 'static, const PEER_NUM: usize>(
        pkt_interaction: Arc<PacketInteraction<Db>>,
        ack_interaction: Arc<AcknowledgementInteraction<Db>>,
    ) {
        async_std::task::spawn_local(async move {
            pkt_interaction
                .handle_incoming_packets(ack_interaction, &send_transport_as_peer::<MSG_PROTOCOL, PEER_NUM>)
                .await;
        });
    }

    fn spawn_ack_handling<Db: HoprCoreEthereumDbActions + 'static>(
        ack_interaction: Arc<AcknowledgementInteraction<Db>>,
    ) {
        async_std::task::spawn_local(async move { ack_interaction.handle_incoming_acknowledgements().await });
    }

    #[serial]
    #[async_std::test]
    pub async fn test_packet_acknowledgement_sender_workflow() {
        let _ = env_logger::builder().is_test(true).try_init();
        const TIMEOUT_SECONDS: u64 = 10;

        init_transport();

        let (done_tx, done_rx) = async_std::channel::unbounded();

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
        let ack_interaction_sender = Arc::new(AcknowledgementInteraction::new(
            core_dbs[0].clone(),
            Some(done_tx),
            None,
        ));
        spawn_ack_receive::<_, 0>(ack_interaction_sender.clone());
        spawn_ack_handling(ack_interaction_sender.clone());

        // Peer 2: Recipient of the packet and sender of the acknowledgement
        let ack_interaction_counterparty = Arc::new(AcknowledgementInteraction::new(core_dbs[1].clone(), None, None));

        // Peer 2: start sending out outgoing acknowledgement
        spawn_ack_send::<_, 1>(ack_interaction_counterparty.clone());

        // Peer 2: does not need to process incoming acknowledgements

        ////

        let recv_kp = OffchainKeypair::from_secret(&PEERS_PRIVS[1]).unwrap();
        for (ack_key, _) in sent_challenges.iter() {
            ack_interaction_counterparty
                .send_acknowledgement(Acknowledgement::new(ack_key.clone(), &recv_kp), PEERS[0].clone(), None)
                .await
                .expect("failed to send ack");
        }

        let finish = async move {
            for i in 1..PENDING_ACKS + 1 {
                let ack = done_rx.recv().await.expect("failed finalize ack");
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

        terminate_transport();
        ack_interaction_sender.stop();
        ack_interaction_counterparty.stop();
        async_std::task::sleep(Duration::from_secs(1)).await; // Let everything shutdown

        assert!(succeeded, "test timed out after {TIMEOUT_SECONDS} seconds");
    }

    #[serial]
    #[async_std::test]
    pub async fn test_packet_acknowledgement_relayer_workflow() {
        let _ = env_logger::builder().is_test(true).try_init();

        const TIMEOUT_SECONDS: u64 = 20;

        init_transport();

        let (pkt_tx, pkt_rx) = async_std::channel::unbounded();
        let (ack_tx, ack_rx) = async_std::channel::unbounded();

        let dbs = create_dbs(3);

        create_minimal_topology(&dbs)
            .await
            .expect("failed to create minimal channel topology");

        let core_dbs = create_core_dbs(&dbs);

        // Begin test
        debug!("peer 1 (sender)    = {}", PEERS[0]);
        debug!("peer 2 (relayer)   = {}", PEERS[1]);
        debug!("peer 3 (recipient) = {}", PEERS[2]);

        const PENDING_PACKETS: usize = 5;

        let packet_path = Path::new_valid(PEERS[1..=2].to_vec());
        assert_eq!(2, packet_path.length(), "path has invalid length");

        // Peer 1 (sender): just sends packets over Peer 2 to Peer 3, ignores acknowledgements from Peer 2
        let packet_sender = Arc::new(PacketInteraction::new(
            core_dbs[0].clone(),
            None,
            PacketInteractionConfig {
                check_unrealized_balance: true,
                packet_keypair: OffchainKeypair::from_secret(&PEERS_PRIVS[0]).unwrap(),
                chain_keypair: ChainKeypair::from_secret(&PEERS_PRIVS[0]).unwrap(),
                mixer: MixerConfig::default(),
            },
        ));
        spawn_pkt_send::<_, 0>(packet_sender.clone());

        // Peer 2 (relayer): relays packets to Peer 3 and awaits acknowledgements of relayer packets to Peer 3
        let ack_interaction_relayer =
            Arc::new(AcknowledgementInteraction::new(core_dbs[1].clone(), None, Some(ack_tx)));
        let pkt_interaction_relayer = Arc::new(PacketInteraction::new(
            core_dbs[1].clone(),
            None,
            PacketInteractionConfig {
                check_unrealized_balance: true,
                packet_keypair: OffchainKeypair::from_secret(&PEERS_PRIVS[1]).unwrap(),
                chain_keypair: ChainKeypair::from_secret(&PEERS_PRIVS[1]).unwrap(),
                mixer: MixerConfig::default(),
            },
        ));

        // Peer 2: start packets handling as a relayer and start receiving of incoming acknowledgements
        spawn_pkt_handling::<_, 1>(pkt_interaction_relayer.clone(), ack_interaction_relayer.clone());
        spawn_pkt_receive::<_, 1>(pkt_interaction_relayer.clone());
        spawn_ack_handling(ack_interaction_relayer.clone());
        spawn_ack_receive::<_, 1>(ack_interaction_relayer.clone());

        // Peer 3: Recipient of the packet and sender of the acknowledgement
        let ack_interaction_counterparty = Arc::new(AcknowledgementInteraction::new(core_dbs[2].clone(), None, None));
        let pkt_interaction_counterparty = Arc::new(PacketInteraction::new(
            core_dbs[2].clone(),
            Some(pkt_tx),
            PacketInteractionConfig {
                check_unrealized_balance: true,
                packet_keypair: OffchainKeypair::from_secret(&PEERS_PRIVS[2]).unwrap(),
                chain_keypair: ChainKeypair::from_secret(&PEERS_PRIVS[2]).unwrap(),
                mixer: MixerConfig::default(),
            },
        ));

        // Peer 3: start packet interaction at the recipient and start sending out acknowledgement
        spawn_pkt_handling::<_, 2>(
            pkt_interaction_counterparty.clone(),
            ack_interaction_counterparty.clone(),
        );
        spawn_pkt_receive::<_, 2>(pkt_interaction_counterparty.clone());
        spawn_ack_send::<_, 2>(ack_interaction_counterparty.clone());

        // Peer 1: start sending out packets
        for _ in 0..PENDING_PACKETS {
            packet_sender
                .send_packet(&TEST_MESSAGE, Some(42), packet_path.clone(), None)
                .await
                .unwrap();
        }

        ////

        // Check that we received all acknowledgements and packets
        let finish = async move {
            let (mut acks, mut pkts) = (0, 0);
            for _ in 1..2 * PENDING_PACKETS + 1 {
                match select(ack_rx.recv(), pkt_rx.recv()).await {
                    Either::Left((ack, _)) => {
                        debug!("relayer has received acknowledged ticket from {}", ack.unwrap().signer);
                        acks += 1;
                    }
                    Either::Right((pkt, _)) => {
                        let msg = pkt.unwrap();
                        debug!("received message: {}", hex::encode(&msg.plain_text));
                        assert_eq!(TEST_MESSAGE, msg.as_ref(), "received packet payload must match");
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
                true
            }
            Either::Right(_) => false,
        };

        terminate_transport();
        pkt_interaction_relayer.stop();
        ack_interaction_relayer.stop();
        pkt_interaction_counterparty.stop();
        ack_interaction_counterparty.stop();
        async_std::task::sleep(Duration::from_secs(1)).await; // Let everything shutdown

        assert!(succeeded, "test timed out after {TIMEOUT_SECONDS} seconds");
    }

    #[serial]
    #[async_std::test]
    async fn test_packet_interaction_multirelay_workflow() {
        let _ = env_logger::builder().is_test(true).try_init();

        const TIMEOUT_SECONDS: u64 = 20;

        init_transport();

        let (pkt_tx, pkt_rx) = async_std::channel::unbounded();

        let dbs = create_dbs(5);

        create_minimal_topology(&dbs)
            .await
            .expect("failed to create minimal channel topology");

        let core_dbs = create_core_dbs(&dbs);

        // Begin test
        debug!("peer 1 (sender)    = {}", PEERS[0]);
        debug!("peer 2 (relayer 1)   = {}", PEERS[1]);
        debug!("peer 3 (relayer 2)   = {}", PEERS[2]);
        debug!("peer 4 (relayer 3)   = {}", PEERS[3]);
        debug!("peer 5 (recipient) = {}", PEERS[4]);

        const PENDING_PACKETS: usize = 5;

        let packet_path = Path::new_valid(PEERS[1..].to_vec());
        assert_eq!(4, packet_path.length(), "path has invalid length");

        let mut interactions = vec![];
        // -------------- Peer 1: sender
        let packet_sender = Arc::new(PacketInteraction::new(
            core_dbs[0].clone(),
            None,
            PacketInteractionConfig {
                check_unrealized_balance: true,
                packet_keypair: OffchainKeypair::from_secret(&PEERS_PRIVS[0]).unwrap(),
                chain_keypair: ChainKeypair::from_secret(&PEERS_PRIVS[0]).unwrap(),
                mixer: MixerConfig::default(),
            },
        ));
        spawn_pkt_send::<_, 0>(packet_sender.clone());

        interactions.push((Some(packet_sender.clone()), None));

        // -------------- Peer 2: relayer
        let ack_1 = Arc::new(AcknowledgementInteraction::new(core_dbs[1].clone(), None, None));
        let pkt_1 = Arc::new(PacketInteraction::new(
            core_dbs[1].clone(),
            None,
            PacketInteractionConfig {
                check_unrealized_balance: true,
                packet_keypair: OffchainKeypair::from_secret(&PEERS_PRIVS[1]).unwrap(),
                chain_keypair: ChainKeypair::from_secret(&PEERS_PRIVS[1]).unwrap(),
                mixer: MixerConfig::default(),
            },
        ));

        spawn_pkt_handling::<_, 1>(pkt_1.clone(), ack_1.clone());
        spawn_pkt_receive::<_, 1>(pkt_1.clone());
        spawn_ack_handling(ack_1.clone());
        spawn_ack_receive::<_, 1>(ack_1.clone());
        spawn_ack_send::<_, 1>(ack_1.clone());

        interactions.push((Some(pkt_1), Some(ack_1)));

        // -------------- Peer 3: relayer
        let ack_2 = Arc::new(AcknowledgementInteraction::new(core_dbs[2].clone(), None, None));
        let pkt_2 = Arc::new(PacketInteraction::new(
            core_dbs[2].clone(),
            None,
            PacketInteractionConfig {
                check_unrealized_balance: true,
                packet_keypair: OffchainKeypair::from_secret(&PEERS_PRIVS[2]).unwrap(),
                chain_keypair: ChainKeypair::from_secret(&PEERS_PRIVS[2]).unwrap(),
                mixer: MixerConfig::default(),
            },
        ));

        spawn_pkt_handling::<_, 2>(pkt_2.clone(), ack_2.clone());
        spawn_pkt_receive::<_, 2>(pkt_2.clone());
        spawn_ack_handling(ack_2.clone());
        spawn_ack_receive::<_, 2>(ack_2.clone());
        spawn_ack_send::<_, 2>(ack_2.clone());

        interactions.push((Some(pkt_2), Some(ack_2)));

        // -------------- Peer 4: relayer
        let ack_3 = Arc::new(AcknowledgementInteraction::new(core_dbs[3].clone(), None, None));
        let pkt_3 = Arc::new(PacketInteraction::new(
            core_dbs[3].clone(),
            None,
            PacketInteractionConfig {
                check_unrealized_balance: true,
                packet_keypair: OffchainKeypair::from_secret(&PEERS_PRIVS[3]).unwrap(),
                chain_keypair: ChainKeypair::from_secret(&PEERS_PRIVS[3]).unwrap(),
                mixer: MixerConfig::default(),
            },
        ));

        spawn_pkt_handling::<_, 3>(pkt_3.clone(), ack_3.clone());
        spawn_pkt_receive::<_, 3>(pkt_3.clone());
        spawn_ack_handling(ack_3.clone());
        spawn_ack_receive::<_, 3>(ack_3.clone());
        spawn_ack_send::<_, 3>(ack_3.clone());

        interactions.push((Some(pkt_3), Some(ack_3)));

        // -------------- Peer 5: recipient
        let ack_4 = Arc::new(AcknowledgementInteraction::new(core_dbs[4].clone(), None, None));
        let pkt_4 = Arc::new(PacketInteraction::new(
            core_dbs[4].clone(),
            Some(pkt_tx),
            PacketInteractionConfig {
                check_unrealized_balance: true,
                packet_keypair: OffchainKeypair::from_secret(&PEERS_PRIVS[4]).unwrap(),
                chain_keypair: ChainKeypair::from_secret(&PEERS_PRIVS[4]).unwrap(),
                mixer: MixerConfig::default(),
            },
        ));

        spawn_pkt_handling::<_, 4>(pkt_4.clone(), ack_4.clone());
        spawn_pkt_receive::<_, 4>(pkt_4.clone());
        spawn_ack_handling(ack_4.clone());
        //spawn_ack_receive::<_, 4>(ack_4.clone());
        spawn_ack_send::<_, 4>(ack_4.clone());

        interactions.push((Some(pkt_4), Some(ack_4)));
        // --------------

        // Start sending packets
        for _ in 0..PENDING_PACKETS {
            packet_sender
                .send_packet(
                    &TEST_MESSAGE,
                    Some(42),
                    packet_path.clone(),
                    Some(Duration::from_secs(10)),
                )
                .await
                .unwrap();
        }

        // Check that we received all packets
        let finish = async move {
            for _ in 1..PENDING_PACKETS + 1 {
                let data = pkt_rx.recv().await.expect("failed finalize packet");
                assert_eq!(TEST_MESSAGE, data.as_ref(), "message body mismatch");
                debug!("received packet at the recipient: {}", hex::encode(&data.plain_text));
            }
        };

        let timeout = async_std::task::sleep(Duration::from_secs(TIMEOUT_SECONDS));
        pin_mut!(finish, timeout);

        let succeeded = match select(finish, timeout).await {
            Either::Left(_) => true,
            Either::Right(_) => false,
        };

        terminate_transport();
        interactions.into_iter().for_each(|(pkt, ack)| {
            pkt.map(|v| v.stop());
            ack.map(|v| v.stop());
        });

        async_std::task::sleep(Duration::from_secs(1)).await; // Let everything shutdown

        assert!(succeeded, "test timed out after {TIMEOUT_SECONDS} seconds");
    }

    #[test]
    fn test_final_data() {
        let fd_1 = ApplicationData {
            application_tag: Some(65535),
            plain_text: (*b"test msg").into(),
        };
        let fd_2 = ApplicationData::from_bytes(&fd_1.to_bytes()).expect("should be able to deserialize");
        assert_eq!(fd_1, fd_2, "should be deserialized equal");
        assert!(fd_2.application_tag.is_some_and(|t| t == 65535));

        let fd_1 = ApplicationData {
            application_tag: None,
            plain_text: (*b"test msg").into(),
        };
        let fd_2 = ApplicationData::from_bytes(&fd_1.to_bytes()).expect("should be able to deserialize");
        assert_eq!(fd_1, fd_2, "should be deserialized equal");
        assert!(fd_2.application_tag.is_none());
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::interaction::{AcknowledgementInteraction, PacketInteraction, PacketInteractionConfig, Payload};
    use async_std::channel::unbounded;
    use core_crypto::types::HalfKeyChallenge;
    use core_ethereum_db::db::wasm::Database;
    use core_ethereum_db::db::CoreEthereumDb;
    use core_mixer::mixer::Mixer;
    use core_path::path::Path;
    use core_types::acknowledgement::{AcknowledgedTicket, Acknowledgement};
    use js_sys::{JsString, Uint8Array};
    use libp2p_identity::PeerId;
    use std::future::Future;
    use std::pin::Pin;
    use std::str::FromStr;
    use std::sync::Arc;
    use std::time::Duration;
    use utils_db::leveldb::wasm::LevelDbShim;
    use utils_log::{debug, error};
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::traits::BinarySerializable;
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::JsValue;

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

    macro_rules! create_transport_closure {
        ($transport_cb:ident) => {
            |msg: Box<[u8]>, peer: String| -> Pin<Box<dyn Future<Output = Result<(), String>>>> {
                Box::pin(async move {
                    let this = JsValue::null();
                    let data: JsValue = js_sys::Uint8Array::from(msg.as_ref()).into();
                    let peer: JsValue = JsString::from(peer.as_str()).into();

                    match $transport_cb.call2(&this, &data, &peer) {
                        Ok(r) => {
                            let promise = js_sys::Promise::from(r);
                            match wasm_bindgen_futures::JsFuture::from(promise).await {
                                Ok(_) => Ok(()),
                                Err(e) => {
                                    error!("Failed to send message in packet / acknowledgement interaction");
                                    error!("{:?}", e);

                                    Err("Failed to send message in packet / acknowledgement interaction".into())
                                }
                            }
                        }
                        Err(e) => {
                            error!(
                                "The message transport could not be established: {}",
                                e.as_string()
                                    .unwrap_or_else(|| { "The message transport failed with unknown error".to_owned() })
                                    .as_str()
                            );
                            Err(format!("Failed to extract transport error as string: {:?}", e))
                        }
                    }
                })
            }
        };
    }

    #[wasm_bindgen]
    pub struct WasmAckInteraction {
        w: Arc<AcknowledgementInteraction<CoreEthereumDb<LevelDbShim>>>,
    }

    #[wasm_bindgen]
    impl WasmAckInteraction {
        #[wasm_bindgen(constructor)]
        pub fn new(
            db: Database,
            on_acknowledgement: Option<js_sys::Function>,
            on_acknowledged_ticket: Option<js_sys::Function>,
        ) -> Self {
            let on_ack = on_acknowledgement.is_some().then(unbounded::<HalfKeyChallenge>).unzip();
            let on_ack_ticket = on_acknowledged_ticket
                .is_some()
                .then(unbounded::<AcknowledgedTicket>)
                .unzip();

            if let Some(ack_recv) = on_ack.1 {
                wasm_bindgen_futures::spawn_local(async move {
                    let this = JsValue::null();
                    let cb = on_acknowledgement.unwrap();
                    while let Ok(ack) = ack_recv.recv().await {
                        debug!("wasm ack recv interaction loop iteration {ack}");

                        let param: JsValue = Uint8Array::from(ack.to_bytes().as_ref()).into();
                        if let Err(e) = cb.call1(&this, &param) {
                            error!("failed to call on_ack closure: {:?}", e.as_string());
                        }
                    }
                });
            }

            if let Some(ack_tkt_recv) = on_ack_ticket.1 {
                wasm_bindgen_futures::spawn_local(async move {
                    let this = JsValue::null();
                    let cb = on_acknowledged_ticket.unwrap();
                    while let Ok(ack) = ack_tkt_recv.recv().await {
                        debug!("wasm ack ticket recv interaction loop iteration {ack}");

                        let param: JsValue = Uint8Array::from(ack.to_bytes().as_ref()).into();
                        if let Err(e) = cb.call1(&this, &param) {
                            error!("failed to call on_ack_ticket closure: {:?}", e.as_string());
                        }
                    }
                });
            }

            Self {
                w: Arc::new(AcknowledgementInteraction::new(
                    db.as_ref_counted(),
                    on_ack.0,
                    on_ack_ticket.0,
                )),
            }
        }

        pub async fn received_acknowledgement(&self, payload: Payload) -> JsResult<()> {
            ok_or_jserr!(self.w.received_acknowledgement(payload, false).await)
        }

        pub async fn send_acknowledgement(
            &self,
            ack: Acknowledgement,
            dest: String,
            timeout_secs: u64,
        ) -> JsResult<()> {
            ok_or_jserr!(
                self.w
                    .send_acknowledgement(
                        ack,
                        ok_or_jserr!(PeerId::from_str(&dest))?,
                        Some(Duration::from_secs(timeout_secs))
                    )
                    .await
            )
        }

        pub async fn handle_incoming_acknowledgements(&self) {
            self.w.handle_incoming_acknowledgements().await
        }

        pub async fn handle_outgoing_acknowledgements(&self, transport_cb: &js_sys::Function) {
            let msg_transport = create_transport_closure!(transport_cb);
            self.w.handle_outgoing_acknowledgements(&msg_transport).await
        }

        pub fn stop(&self) {
            self.w.stop()
        }
    }

    #[wasm_bindgen]
    pub struct WasmPacketInteraction {
        w: PacketInteraction<CoreEthereumDb<LevelDbShim>>,
    }

    #[wasm_bindgen]
    impl WasmPacketInteraction {
        #[wasm_bindgen(constructor)]
        pub fn new(db: Database, on_final_packet: Option<js_sys::Function>, cfg: PacketInteractionConfig) -> Self {
            let on_msg = on_final_packet
                .is_some()
                .then(unbounded::<super::ApplicationData>)
                .unzip();

            // For WASM we need to create mixer with gloo-timers
            let gloo_mixer = Mixer::new_with_gloo_timers(cfg.mixer);

            let mut w = PacketInteraction::new(db.as_ref_counted(), on_msg.0, cfg);

            w.mixer = gloo_mixer;

            if let Some(on_msg_recv) = on_msg.1 {
                wasm_bindgen_futures::spawn_local(async move {
                    let this = JsValue::null();
                    let cb = on_final_packet.unwrap();
                    while let Ok(pkt) = on_msg_recv.recv().await {
                        debug!(
                            "wasm packet interaction loop iteration {}",
                            hex::encode(&pkt.plain_text)
                        );
                        if let Ok(param) = serde_wasm_bindgen::to_value(&pkt) {
                            if let Err(e) = cb.call1(&this, &param) {
                                error!("failed to call on_msg closure: {:?}", e.as_string());
                            }
                        } else {
                            error!("failed to serialize application data")
                        }
                    }
                });
            }

            Self { w }
        }

        pub async fn received_packet(&self, payload: Payload) -> JsResult<()> {
            ok_or_jserr!(self.w.received_packet(payload, false).await)
        }

        pub async fn send_packet(
            &self,
            msg: &[u8],
            app_tag: Option<u16>,
            path: Path,
            timeout_secs: u64,
        ) -> JsResult<HalfKeyChallenge> {
            ok_or_jserr!(
                self.w
                    .send_packet(msg, app_tag, path, Some(Duration::from_secs(timeout_secs)))
                    .await
            )
        }

        pub async fn handle_outgoing_packets(&self, transport_cb: &js_sys::Function) {
            let msg_transport = create_transport_closure!(transport_cb);
            self.w.handle_outgoing_packets(&msg_transport).await
        }

        pub async fn handle_incoming_packets(
            &self,
            ack_interaction: &WasmAckInteraction,
            transport_cb: &js_sys::Function,
        ) {
            let msg_transport = create_transport_closure!(transport_cb);
            self.w
                .handle_incoming_packets(ack_interaction.w.clone(), &msg_transport)
                .await
        }

        pub fn stop(&self) {
            self.w.stop()
        }
    }
}
