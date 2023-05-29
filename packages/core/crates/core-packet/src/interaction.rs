use crate::errors::PacketError::{
    AcknowledgementValidation, ChannelNotFound, InvalidPacketState, OutOfFunds, TagReplay, TicketValidation,
    TransportError,
};
use crate::errors::Result;
use crate::packet::{Packet, PacketState};
use crate::path::Path;
use async_std::channel::{unbounded, Receiver, Sender};
use core_crypto::types::{HalfKeyChallenge, Hash, PublicKey};
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_mixer::mixer::{Mixer, MixerConfig};
use core_types::acknowledgement::{AcknowledgedTicket, Acknowledgement, PendingAcknowledgement, UnacknowledgedTicket};
use core_types::channels::{ChannelEntry, ChannelStatus, Ticket};
use libp2p_identity::PeerId;
use std::ops::Mul;
use std::sync::{Arc, Mutex};
use utils_log::{debug, error, info};
//use utils_log::{debug, error, info};
use utils_types::primitives::{Balance, BalanceType, U256};
use utils_types::traits::{BinarySerializable, PeerIdLike, ToHex};

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

pub const PRICE_PER_PACKET: &str = "10000000000000000";
pub const INVERSE_TICKET_WIN_PROB: &str = "1";
const PREIMAGE_PLACE_HOLDER: [u8; Hash::SIZE] = [0xffu8; Hash::SIZE];

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct TransportTask {
    remote_peer: PeerId,
    data: Box<[u8]>,
}

pub struct AcknowledgementInteraction<Db: HoprCoreEthereumDbActions> {
    db: Arc<Mutex<Db>>,
    pub on_acknowledgement: Box<dyn Fn(HalfKeyChallenge)>,
    pub on_acknowledged_ticket: Box<dyn Fn(AcknowledgedTicket)>,
    public_key: PublicKey,
    incoming_channel: (Sender<TransportTask>, Receiver<TransportTask>),
    outgoing_channel: (Sender<TransportTask>, Receiver<TransportTask>),
}

impl<Db: HoprCoreEthereumDbActions> AcknowledgementInteraction<Db> {
    pub fn new(db: Arc<Mutex<Db>>, public_key: PublicKey) -> Self {
        Self {
            db,
            public_key,
            incoming_channel: unbounded(),
            outgoing_channel: unbounded(),
            on_acknowledgement: Box::new(|_| {}),
            on_acknowledged_ticket: Box::new(|_| {}),
        }
    }

    pub fn get_peer_id(&self) -> PeerId {
        self.public_key.to_peerid()
    }

    pub async fn received_acknowledgement(&self, task: TransportTask) -> Result<()> {
        self.incoming_channel
            .0
            .send(task)
            .await
            .map_err(|e| TransportError(e.to_string()))
    }

    pub async fn send_acknowledgement(&self, acknowledgement: Acknowledgement, destination: PeerId) -> Result<()> {
        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_SENT_ACKS.increment();

        self.outgoing_channel
            .0
            .send(TransportTask {
                remote_peer: destination,
                data: acknowledgement.to_bytes(),
            })
            .await
            .map_err(|e| TransportError(e.to_string()))
    }

    pub async fn handle_incoming_acknowledgements(&self) {
        while let Ok(ack_task) = self.incoming_channel.1.recv().await {
            match Acknowledgement::from_bytes(&ack_task.data) {
                Ok(ack) => {
                    if let Err(e) = self.handle_acknowledgement(ack, &ack_task.remote_peer).await {
                        error!(
                            "failed to process incoming acknowledgement from {}: {e}",
                            ack_task.remote_peer
                        );
                    }
                }
                Err(e) => {
                    error!("received unreadable acknowledgement from {}: {e}", ack_task.remote_peer);
                }
            }
        }
        info!("done processing incoming acknowledgements");
    }

    pub async fn handle_outgoing_acknowledgements<T, F>(&self, message_transport: &T)
    where
        T: Fn(Box<[u8]>, String) -> F,
        F: futures::Future<Output = core::result::Result<(), String>>,
    {
        while let Ok(ack_task) = self.outgoing_channel.1.recv().await {
            if let Err(e) = message_transport(ack_task.data, ack_task.remote_peer.to_string()).await {
                error!("failed to send acknowledgement to {}: {e}", ack_task.remote_peer);
            }
        }
        info!("done processing outgoing acknowledgements")
    }

    pub fn start(&self) {
        unimplemented!("implement this when migrated to rust libp2p")
    }

    pub fn stop(&self) {
        self.incoming_channel.0.close();
        self.incoming_channel.1.close();
        self.outgoing_channel.0.close();
        self.outgoing_channel.1.close();
    }

    async fn handle_acknowledgement(&self, mut ack: Acknowledgement, remote_peer: &PeerId) -> Result<()> {
        debug!(
            "own_key = {}, remote = {}",
            self.public_key,
            PublicKey::from_peerid(remote_peer).unwrap()
        );
        if !ack.validate(&self.public_key, &PublicKey::from_peerid(remote_peer)?) {
            return Err(AcknowledgementValidation(
                "could not validate the acknowledgement".to_string(),
            ));
        }

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
            .lock()
            .unwrap()
            .get_pending_acknowledgement(&ack.ack_challenge())
            .await?
            .ok_or_else(|| {
                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_RECEIVED_FAILED_ACKS.increment();

                return AcknowledgementValidation(format!(
                    "received unexpected acknowledgement for half key challenge {} - half key {}",
                    ack.ack_challenge().to_hex(),
                    ack.ack_key_share.to_hex()
                ));
            })?;

        match pending {
            PendingAcknowledgement::WaitingAsSender => {
                // No pending ticket, nothing to do.
                debug!("received acknowledgement as sender: first relayer has processed the packet.");
                (self.on_acknowledgement)(ack.ack_challenge());

                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_RECEIVED_SUCCESSFUL_ACKS.increment();
            }

            PendingAcknowledgement::WaitingAsRelayer(unackowledged) => {
                // Try to unlock our incentive
                unackowledged.verify_challenge(&ack.ack_key_share).map_err(|e| {
                    #[cfg(all(feature = "prometheus", not(test)))]
                    METRIC_RECEIVED_FAILED_ACKS.increment();

                    return AcknowledgementValidation(format!(
                        "the acknowledgement is not sufficient to solve the embedded challenge, {e}"
                    ));
                })?;

                self.db
                    .lock()
                    .unwrap()
                    .get_channel_from(&unackowledged.signer)
                    .await
                    .map_err(|e| {
                        #[cfg(all(feature = "prometheus", not(test)))]
                        METRIC_RECEIVED_FAILED_ACKS.increment();

                        return AcknowledgementValidation(format!(
                            "acknowledgement received for channel that does not exist, {e}"
                        ));
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
                self.db
                    .lock()
                    .unwrap()
                    .replace_unack_with_ack(&ack.ack_challenge(), ack_ticket.clone())
                    .await?;

                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_ACKED_TICKETS.increment();

                (self.on_acknowledged_ticket)(ack_ticket);
            }
        }
        Ok(())
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
#[derive(Clone, Debug)]
pub struct PacketInteractionConfig {
    pub check_unrealized_balance: bool,
    pub private_key: Box<[u8]>,
    pub mixer: MixerConfig,
}

pub struct PacketInteraction<Db>
where
    Db: HoprCoreEthereumDbActions,
{
    db: Arc<Mutex<Db>>,
    channel: (Sender<TransportTask>, Receiver<TransportTask>),
    pub message_emitter: Box<dyn Fn(&[u8])>,
    pub mixer: Mixer<TransportTask>,
    cfg: PacketInteractionConfig,
}

impl<Db> PacketInteraction<Db>
where
    Db: HoprCoreEthereumDbActions,
{
    pub fn new(db: Arc<Mutex<Db>>, cfg: PacketInteractionConfig) -> Self {
        Self {
            db,
            channel: unbounded(),
            message_emitter: Box::new(|_| {}),
            mixer: Mixer::new(cfg.mixer.clone()),
            cfg,
        }
    }

    pub async fn validate_unacknowledged_ticket(
        &self,
        sender: &PublicKey,
        min_ticket_amount: Balance,
        req_inverse_ticket_win_prob: U256,
        ticket: &Ticket,
        channel: &ChannelEntry,
    ) -> Result<()> {
        let required_win_prob = U256::from_inverse_probability(req_inverse_ticket_win_prob)?;

        // ticket signer MUST be the sender
        ticket
            .verify(sender)
            .map_err(|e| TicketValidation(format!("ticket signer does not match the sender: {e}")))?;

        // ticket amount MUST be greater or equal to minTicketAmount
        if !ticket.amount.gte(&min_ticket_amount) {
            return Err(TicketValidation(format!(
                "ticket amount {} in not at least {min_ticket_amount}",
                ticket.amount
            )));
        }

        // ticket MUST have match X winning probability
        if !ticket.win_prob.eq(&required_win_prob) {
            return Err(TicketValidation(format!(
                "ticket winning probability {} is not equal to {required_win_prob}",
                ticket.win_prob
            )));
        }

        // channel MUST be open or pending to close
        if channel.status == ChannelStatus::Closed {
            return Err(TicketValidation(format!(
                "payment channel with {sender} is not opened or pending to close"
            )));
        }

        // ticket's epoch MUST match our channel's epoch
        if !ticket.epoch.eq(&channel.ticket_epoch) {
            return Err(TicketValidation(format!(
                "ticket epoch {} does not match our account epoch {} of channel {}",
                ticket.epoch,
                channel.ticket_epoch,
                channel.get_id()
            )));
        }

        // ticket's channelEpoch MUST match the current channel's epoch
        if !ticket.channel_epoch.eq(&channel.channel_epoch) {
            return Err(TicketValidation(format!(
                "ticket was created for a different channel iteration {} != {} of channel {}",
                ticket.channel_epoch,
                channel.channel_epoch,
                channel.get_id()
            )));
        }

        if self.cfg.check_unrealized_balance {
            info!("checking unrealized balances for channel {}", channel.get_id());

            let unrealized_balance = self
                .db
                .lock()
                .unwrap()
                .get_tickets(sender)
                .await? // all tickets from sender
                .into_iter()
                .filter(|t| t.epoch.eq(&channel.ticket_epoch) && t.channel_epoch.eq(&channel.channel_epoch))
                .fold(channel.balance, |result, t| result.sub(&t.amount));

            // ensure sender has enough funds
            if ticket.amount.gt(&unrealized_balance) {
                return Err(OutOfFunds(channel.get_id().to_string()));
            }
        }

        Ok(())
    }

    async fn bump_ticket_index(&self, channel_id: &Hash) -> Result<U256> {
        let current_ticket_index = self
            .db
            .lock()
            .unwrap()
            .get_current_ticket_index(channel_id)
            .await?
            .unwrap_or(U256::one());

        self.db
            .lock()
            .unwrap()
            .set_current_ticket_index(channel_id, current_ticket_index.addn(1))
            .await?;

        Ok(current_ticket_index)
    }

    async fn create_multihop_ticket(&self, destination: PublicKey, path_pos: u8) -> Result<Ticket> {
        let channel = self
            .db
            .lock()
            .unwrap()
            .get_channel_to(&destination)
            .await?
            .ok_or(ChannelNotFound(destination.to_peerid().to_string()))?;

        let channel_id = channel.get_id();
        let current_index = self.bump_ticket_index(&channel_id).await?;
        let amount = Balance::new(
            U256::new(PRICE_PER_PACKET)
                .mul(U256::new(INVERSE_TICKET_WIN_PROB))
                .muln(path_pos as u32 - 1),
            BalanceType::HOPR,
        );

        let outstanding_balance = self
            .db
            .lock()
            .unwrap()
            .get_pending_balance_to(&destination.to_address())
            .await?;

        let channel_balance = channel.balance.sub(&outstanding_balance);

        info!(
            "balances {} - {outstanding_balance} = {channel_balance} should >= {amount} in channel open to {}",
            channel.balance, channel.destination
        );

        if channel_balance.lt(&amount) {
            return Err(OutOfFunds(format!("{channel_id} with counterparty {destination}")));
        }

        let ticket = Ticket::new(
            destination.to_address(),
            None,
            channel.ticket_epoch,
            current_index,
            amount,
            U256::from_inverse_probability(U256::new(INVERSE_TICKET_WIN_PROB))?,
            channel.channel_epoch,
            &self.cfg.private_key,
        );

        self.db.lock().unwrap().mark_pending(&ticket).await?;

        debug!(
            "Creating ticket in channel {channel_id}. Ticket data: {}",
            ticket.to_hex()
        );

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_TICKETS_COUNT.increment();

        Ok(ticket)
    }

    pub async fn send_outgoing_packet<T, F>(
        &self,
        msg: &[u8],
        complete_valid_path: Path,
        message_transport: &T,
    ) -> Result<HalfKeyChallenge>
    where
        T: Fn(Box<[u8]>, String) -> F,
        F: futures::Future<Output = core::result::Result<(), String>>,
    {
        // Decide whether to create 0-hop or multihop ticket
        let next_peer = PublicKey::from_peerid(&complete_valid_path.hops()[0])?;
        let next_ticket = if complete_valid_path.length() == 1 {
            Ticket::new_zero_hop(next_peer, None, &self.cfg.private_key)
        } else {
            self.create_multihop_ticket(next_peer, complete_valid_path.length() as u8)
                .await?
        };

        // Create the packet
        let packet = Packet::new(msg, &complete_valid_path.hops(), &self.cfg.private_key, next_ticket)?;
        match packet.state() {
            PacketState::Outgoing { ack_challenge, .. } => {
                self.db
                    .lock()
                    .unwrap()
                    .store_pending_acknowledgment(ack_challenge.clone(), PendingAcknowledgement::WaitingAsSender)
                    .await?;

                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_PACKETS_COUNT.increment();

                // Send the packet
                message_transport(packet.to_bytes(), complete_valid_path.hops()[0].to_string())
                    .await
                    .map_err(|e| TransportError(e))?;

                Ok(ack_challenge.clone())
            }
            _ => panic!("invalid packet state"),
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
                if self.db.lock().unwrap().check_and_set_packet_tag(packet_tag).await? {
                    return Err(TagReplay);
                }

                // We're the destination of the packet, so emit the packet contents
                (self.message_emitter)(plain_text.as_ref());

                // And create acknowledgement
                let ack = packet.create_acknowledgement(&self.cfg.private_key).unwrap();
                ack_interaction
                    .send_acknowledgement(ack, previous_hop.to_peerid())
                    .await?;

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
                if self.db.lock().unwrap().check_and_set_packet_tag(packet_tag).await? {
                    return Err(TagReplay);
                }

                let inverse_win_prob = U256::new(INVERSE_TICKET_WIN_PROB);

                // Find the corresponding channel
                let channel = self
                    .db
                    .lock()
                    .unwrap()
                    .get_channel_from(&previous_hop)
                    .await?
                    .ok_or(ChannelNotFound(previous_hop.to_string()))?;

                // Validate the ticket first
                if let Err(e) = self
                    .validate_unacknowledged_ticket(
                        &previous_hop,
                        Balance::from_str(PRICE_PER_PACKET, BalanceType::HOPR),
                        inverse_win_prob,
                        &packet.ticket,
                        &channel,
                    )
                    .await
                {
                    // Mark as reject and passthrough the error
                    self.db.lock().unwrap().mark_rejected(&packet.ticket).await?;
                    return Err(e);
                }

                self.db
                    .lock()
                    .unwrap()
                    .set_current_ticket_index(&channel.get_id().hash(), packet.ticket.index)
                    .await?;

                // Store the unacknowledged ticket
                self.db
                    .lock()
                    .unwrap()
                    .store_pending_acknowledgment(
                        ack_challenge.clone(),
                        PendingAcknowledgement::WaitingAsRelayer(UnacknowledgedTicket::new(
                            packet.ticket.clone(),
                            own_key.clone(),
                            previous_hop.clone(),
                        )),
                    )
                    .await?;

                let path_pos = packet
                    .ticket
                    .get_path_position(U256::new(PRICE_PER_PACKET), inverse_win_prob);

                // Create next ticket for the packet
                next_ticket = if path_pos == 1 {
                    Ticket::new_zero_hop(next_hop.clone(), None, &self.cfg.private_key)
                } else {
                    self.create_multihop_ticket(next_hop.clone(), path_pos).await?
                };
                previous_peer = previous_hop.to_peerid();
                next_peer = next_hop.to_peerid();
            }
        }

        // Transform the packet for forwarding using the next ticket
        packet.forward(&self.cfg.private_key, next_ticket)?;

        // Forward the packet to the next hop
        message_transport(packet.to_bytes(), next_peer.to_string())
            .await
            .map_err(|e| TransportError(e))?;

        // Acknowledge to the previous hop that we forwarded the packet
        let ack = packet.create_acknowledgement(&self.cfg.private_key).unwrap();
        ack_interaction.send_acknowledgement(ack, previous_peer).await?;

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_FWD_MESSAGE_COUNT.increment();

        Ok(())
    }

    pub async fn handle_packets<T, F>(
        &self,
        ack_interaction: Arc<AcknowledgementInteraction<Db>>,
        message_transport: &T,
    ) where
        T: Fn(Box<[u8]>, String) -> F,
        F: futures::Future<Output = core::result::Result<(), String>>,
    {
        while let Ok(task) = self.channel.1.recv().await {
            // Add some random delay via mixer
            let mixed_packet = self.mixer.mix(task).await;
            match Packet::from_bytes(&mixed_packet.data, &self.cfg.private_key, &mixed_packet.remote_peer) {
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

    pub async fn push_received_packet(&self, packet_task: TransportTask) -> Result<()> {
        self.channel
            .0
            .send(packet_task)
            .await
            .map_err(|e| TransportError(e.to_string()))
    }

    pub fn start(&self) {
        unimplemented!("implement this when migrated to rust libp2p")
    }

    pub fn stop(&self) {
        self.channel.0.close();
        self.channel.1.close();
    }
}

#[cfg(all(not(target_arch = "wasm32"), test))]
mod tests {
    use crate::errors::PacketError::PacketDbError;
    use crate::interaction::{
        AcknowledgementInteraction, PacketInteraction, PacketInteractionConfig, TransportTask, PRICE_PER_PACKET,
    };
    use crate::path::Path;
    use crate::por::ProofOfRelayValues;
    use async_trait::async_trait;
    use core_crypto::derivation::derive_ack_key_share;
    use core_crypto::random::random_bytes;
    use core_crypto::types::{Hash, PublicKey};
    use core_ethereum_db::db::CoreEthereumDb;
    use core_ethereum_db::traits::HoprCoreEthereumDbActions;
    use core_ethereum_misc::commitment::{initialize_commitment, ChainCommitter, ChannelCommitmentInfo};
    use core_mixer::mixer::MixerConfig;
    use core_types::acknowledgement::{Acknowledgement, AcknowledgementChallenge, PendingAcknowledgement};
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
            .map(|private| PublicKey::from_privkey(private).unwrap().to_peerid())
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
    ) -> std::result::Result<(), String> {
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

    fn create_dummy_channel(from: &PeerId, to: &PeerId) -> ChannelEntry {
        ChannelEntry::new(
            PublicKey::from_peerid(from).unwrap(),
            PublicKey::from_peerid(to).unwrap(),
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

    fn create_core_dbs(dbs: &Vec<Arc<Mutex<rusty_leveldb::DB>>>) -> Vec<Arc<Mutex<CoreEthereumDb<RustyLevelDbShim>>>> {
        dbs.iter()
            .enumerate()
            .map(|(i, db)| {
                Arc::new(Mutex::new(CoreEthereumDb::new(
                    DB::new(RustyLevelDbShim::new(db.clone())),
                    PublicKey::from_peerid(&PEERS[i]).unwrap(),
                )))
            })
            .collect::<Vec<_>>()
    }

    struct EmptyChainCommiter {}

    #[async_trait(? Send)]
    impl ChainCommitter for EmptyChainCommiter {
        async fn get_commitment(&self) -> Option<Hash> {
            None
        }

        async fn set_commitment(&mut self, _commitment: &Hash) -> String {
            "".to_string()
        }
    }

    async fn create_minimal_topology(dbs: &Vec<Arc<Mutex<rusty_leveldb::DB>>>) -> crate::errors::Result<()> {
        let testing_snapshot = Snapshot::new(U256::zero(), U256::zero(), U256::zero());
        let mut previous_channel: Option<ChannelEntry> = None;

        for (index, peer_id) in PEERS.iter().enumerate().take(dbs.len()) {
            let mut db = CoreEthereumDb::new(
                DB::new(RustyLevelDbShim::new(dbs[index].clone())),
                PublicKey::from_peerid(&peer_id).unwrap(),
            );

            let mut channel: Option<ChannelEntry> = None;

            if index < PEERS.len() - 1 {
                channel = Some(create_dummy_channel(&peer_id, &PEERS[index + 1]));

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

                initialize_commitment(&mut db, &PEERS_PRIVS[0], &channel_info, &mut EmptyChainCommiter {})
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
                for task in msgs.into_iter().map(|m| TransportTask {
                    remote_peer: m.from,
                    data: m.data,
                }) {
                    debug!("received ack from {}: {}", task.remote_peer, hex::encode(&task.data));
                    interaction
                        .received_acknowledgement(task)
                        .await
                        .expect("failed to receive ack");
                }
                async_std::task::sleep(Duration::from_millis(200)).await;
            }
        });
    }

    fn spawn_pkt_receive<Db: HoprCoreEthereumDbActions + 'static, const PEER_NUM: usize>(
        interaction: Arc<PacketInteraction<Db>>,
    ) {
        async_std::task::spawn_local(async move {
            while let Some(msgs) = retrieve_transport_msgs_as_peer::<MSG_PROTOCOL, PEER_NUM>() {
                for task in msgs.into_iter().map(|m| TransportTask {
                    remote_peer: m.from,
                    data: m.data,
                }) {
                    debug!("received packet from {}: {}", task.remote_peer, hex::encode(&task.data));
                    interaction
                        .push_received_packet(task)
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

    fn spawn_pkt_handling<Db: HoprCoreEthereumDbActions + 'static, const PEER_NUM: usize>(
        pkt_interaction: Arc<PacketInteraction<Db>>,
        ack_interaction: Arc<AcknowledgementInteraction<Db>>,
    ) {
        async_std::task::spawn_local(async move {
            pkt_interaction
                .handle_packets(ack_interaction, &send_transport_as_peer::<MSG_PROTOCOL, PEER_NUM>)
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
            let secrets = (0..2).into_iter().map(|_| random_bytes::<32>()).collect::<Vec<_>>();
            let porv = ProofOfRelayValues::new(&secrets[0], Some(&secrets[1]))
                .expect("failed to create Proof of Relay values");

            // Mimics that the packet sender has sent a packet and now it has a pending acknowledgement in it's DB
            core_dbs[0]
                .lock()
                .unwrap()
                .store_pending_acknowledgment(porv.ack_challenge.clone(), PendingAcknowledgement::WaitingAsSender)
                .await
                .expect("failed to store pending ack");

            let ack_key = derive_ack_key_share(&secrets[0]);
            let ack_msg = AcknowledgementChallenge::new(&porv.ack_challenge, &PEERS_PRIVS[0]);

            sent_challenges.push((ack_key, ack_msg));
        }

        // Peer 1: This is mimics the ACK interaction of the packet sender
        let mut tmp_ack =
            AcknowledgementInteraction::new(core_dbs[0].clone(), PublicKey::from_peerid(&PEERS[0]).unwrap());

        // ... which is just waiting to get an acknowledgement from the counterparty
        let expected_challenges = sent_challenges.clone();
        tmp_ack.on_acknowledgement = Box::new(move |ack| {
            debug!("sender has received acknowledgement: {}", ack.to_hex());
            if let Some((ack_key, ack_msg)) = expected_challenges
                .iter()
                .find(|(_, chal)| chal.ack_challenge.unwrap().eq(&ack))
            {
                assert!(
                    ack_msg.solve(&ack_key.to_bytes()),
                    "acknowledgement key must solve acknowledgement challenge"
                );

                // If it matches, set a signal that the test has finished
                done_tx.send_blocking(()).expect("send failed");
                debug!("peer 1 received expected ack");
            }
        });

        // Peer 1: hookup receiving of acknowledgements and start processing them
        let ack_interaction_sender = Arc::new(tmp_ack);
        spawn_ack_receive::<_, 0>(ack_interaction_sender.clone());
        spawn_ack_handling(ack_interaction_sender.clone());

        // Peer 2: Recipient of the packet and sender of the acknowledgement
        let ack_interaction_counterparty = Arc::new(AcknowledgementInteraction::new(
            core_dbs[1].clone(),
            PublicKey::from_peerid(&PEERS[1]).unwrap(),
        ));

        // Peer 2: start sending out outgoing acknowledgement
        spawn_ack_send::<_, 1>(ack_interaction_counterparty.clone());

        // Peer 2: does not need to process incoming acknowledgements

        ////

        for (ack_key, ack_msg) in sent_challenges {
            ack_interaction_counterparty
                .send_acknowledgement(
                    Acknowledgement::new(ack_msg, ack_key, &PEERS_PRIVS[1]),
                    PEERS[0].clone(),
                )
                .await
                .expect("failed to send ack");
        }

        let finish = async move {
            for i in 1..PENDING_ACKS + 1 {
                done_rx.recv().await.expect("failed finalize ack");
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

        enum AckOrPacket {
            Ack,
            Packet,
        }

        let (done_tx, done_rx) = async_std::channel::unbounded();

        let dbs = create_dbs(3);

        create_minimal_topology(&dbs)
            .await
            .expect("failed to create minimal channel topology");

        let core_dbs = create_core_dbs(&dbs);

        // Begin test
        debug!("peer 1 (sender)    = {}", PEERS[0]);
        debug!("peer 2 (relayer)   = {}", PEERS[1]);
        debug!("peer 3 (recipient) = {}", PEERS[2]);

        // Peer 1 (sender): just sends packets over Peer 2 to Peer 3, ignores acknowledgements from Peer 2
        let packet_path = Path::new_valid(PEERS[1..=2].to_vec());
        assert_eq!(2, packet_path.length(), "path must have length 2");

        let packet_sender = PacketInteraction::new(
            core_dbs[0].clone(),
            PacketInteractionConfig {
                check_unrealized_balance: true,
                private_key: PEERS_PRIVS[0].into(),
                mixer: MixerConfig::default(),
            },
        );

        // Peer 2 (relayer): awaits acknowledgements of relayer packets to Peer 3
        let mut tmp_ack =
            AcknowledgementInteraction::new(core_dbs[1].clone(), PublicKey::from_peerid(&PEERS[1]).unwrap());
        let done_tx_clone = done_tx.clone();
        tmp_ack.on_acknowledged_ticket = Box::new(move |ack| {
            debug!("relayer has received acknowledged ticket from {}", ack.signer);
            done_tx_clone
                .send_blocking(AckOrPacket::Ack)
                .expect("failed to confirm ack");
        });

        // Peer 2: packet interaction for the relayer
        let ack_interaction_relayer = Arc::new(tmp_ack);
        let pkt_interaction_relayer = Arc::new(PacketInteraction::new(
            core_dbs[1].clone(),
            PacketInteractionConfig {
                check_unrealized_balance: true,
                private_key: PEERS_PRIVS[1].into(),
                mixer: MixerConfig::default(),
            },
        ));

        // Peer 2: start packets handling as a relayer and start receiving of incoming acknowledgements
        spawn_pkt_handling::<_, 1>(pkt_interaction_relayer.clone(), ack_interaction_relayer.clone());
        spawn_pkt_receive::<_, 1>(pkt_interaction_relayer.clone());
        spawn_ack_handling(ack_interaction_relayer.clone());
        spawn_ack_receive::<_, 1>(ack_interaction_relayer.clone());

        // Peer 3: Recipient of the packet and sender of the acknowledgement
        let mut tmp_pkt_counterparty = PacketInteraction::new(
            core_dbs[2].clone(),
            PacketInteractionConfig {
                check_unrealized_balance: true,
                private_key: PEERS_PRIVS[2].into(),
                mixer: MixerConfig::default(),
            },
        );
        tmp_pkt_counterparty.message_emitter = Box::new(move |msg: &[u8]| {
            debug!("received message: {}", hex::encode(msg));
            assert_eq!(TEST_MESSAGE, msg, "received packet payload must match");
            done_tx
                .send_blocking(AckOrPacket::Packet)
                .expect("failed to confirm pkt");
        });

        let ack_interaction_counterparty = Arc::new(AcknowledgementInteraction::new(
            core_dbs[2].clone(),
            PublicKey::from_peerid(&PEERS[2]).unwrap(),
        ));
        let pkt_interaction_counterparty = Arc::new(tmp_pkt_counterparty);

        // Peer 3: start packet interaction at the recipient and start sending out acknowledgement
        spawn_pkt_handling::<_, 2>(
            pkt_interaction_counterparty.clone(),
            ack_interaction_counterparty.clone(),
        );
        spawn_pkt_receive::<_, 2>(pkt_interaction_counterparty.clone());
        spawn_ack_send::<_, 2>(ack_interaction_counterparty.clone());

        // Peer 1: Start sending out packets
        const PENDING_PACKETS: usize = 5;
        for _ in 0..PENDING_PACKETS {
            packet_sender
                .send_outgoing_packet(
                    &TEST_MESSAGE,
                    packet_path.clone(),
                    &send_transport_as_peer::<MSG_PROTOCOL, 0>,
                )
                .await
                .unwrap();
        }

        ////

        // Check that we received all acknowledgements and packets
        let finish = async move {
            let (mut acks, mut pkts) = (0, 0);
            for _ in 1..2 * PENDING_PACKETS + 1 {
                match done_rx.recv().await.expect("failed finalize ack") {
                    AckOrPacket::Ack => {
                        acks += 1;
                        debug!("done ack tickets #{acks} out of {PENDING_PACKETS}");
                    }
                    AckOrPacket::Packet => {
                        pkts += 1;
                        debug!("done msg #{pkts} out of {PENDING_PACKETS}");
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
        ack_interaction_relayer.stop();
        ack_interaction_counterparty.stop();
        async_std::task::sleep(Duration::from_secs(1)).await; // Let everything shutdown

        assert!(succeeded, "test timed out after {TIMEOUT_SECONDS} seconds");
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::interaction::{AcknowledgementInteraction, PacketInteraction, PacketInteractionConfig, TransportTask};
    use core_crypto::types::PublicKey;
    use core_ethereum_db::db::CoreEthereumDb;
    use core_mixer::mixer::Mixer;
    use core_types::acknowledgement::Acknowledgement;
    use js_sys::JsString;
    use libp2p_identity::PeerId;
    use std::future::Future;
    use std::pin::Pin;
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};
    use utils_db::db::DB;
    use utils_db::leveldb::{LevelDb, LevelDbShim};
    use utils_log::error;
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;

    #[wasm_bindgen]
    impl TransportTask {
        #[wasm_bindgen(constructor)]
        pub fn _new(peer_id: &str, packet_data: Box<[u8]>) -> JsResult<TransportTask> {
            Ok(Self {
                remote_peer: ok_or_jserr!(PeerId::from_str(peer_id))?,
                data: packet_data,
            })
        }
    }

    #[wasm_bindgen]
    pub struct WasmAckInteraction {
        w: Arc<AcknowledgementInteraction<CoreEthereumDb<LevelDbShim>>>,
    }

    #[wasm_bindgen]
    impl WasmAckInteraction {
        pub fn new(db: LevelDb, chain_key: PublicKey) -> Self {
            Self {
                w: Arc::new(AcknowledgementInteraction::new(
                    Arc::new(Mutex::new(CoreEthereumDb::new(
                        DB::new(LevelDbShim::new(db)),
                        chain_key.clone(),
                    ))),
                    chain_key,
                )),
            }
        }

        pub async fn received_new_acknowledgement(&self, task: TransportTask) -> JsResult<()> {
            ok_or_jserr!(self.w.received_acknowledgement(task).await)
        }

        pub async fn send_new_acknowledgement(&self, ack: Acknowledgement, dest: String) -> JsResult<()> {
            ok_or_jserr!(
                self.w
                    .send_acknowledgement(ack, ok_or_jserr!(PeerId::from_str(&dest))?)
                    .await
            )
        }

        pub async fn start_handle_incoming_acks(&self) {
            self.w.handle_incoming_acknowledgements().await
        }

        pub async fn start_handle_outgoing_acks(&self, transport_cb: &js_sys::Function) {
            let msg_transport = |msg: Box<[u8]>, peer: String| -> Pin<Box<dyn Future<Output = Result<(), String>>>> {
                Box::pin(async move {
                    let this = JsValue::null();
                    let data: JsValue = js_sys::Uint8Array::from(msg.as_ref()).into();
                    let peer: JsValue = JsString::from(peer.as_str()).into();

                    match transport_cb.call2(&this, &data, &peer) {
                        Ok(r) => {
                            let promise = js_sys::Promise::from(r);
                            wasm_bindgen_futures::JsFuture::from(promise)
                                .await
                                .map(|_| ())
                                .map_err(|x| {
                                    x.dyn_ref::<JsString>()
                                        .map_or("Failed to send ping message".to_owned(), |x| -> String { x.into() })
                                })
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
            };
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
        pub fn new(db: LevelDb, cfg: PacketInteractionConfig) -> Self {
            // For WASM we need to create mixer with gloo-timers
            let gloo_mixer = Mixer::new_with_gloo_timers(cfg.mixer.clone());
            let mut w = PacketInteraction::new(
                Arc::new(Mutex::new(CoreEthereumDb::new(
                    DB::new(LevelDbShim::new(db)),
                    PublicKey::from_privkey(&cfg.private_key).expect("invalid private key"),
                ))),
                cfg,
            );
            w.mixer = gloo_mixer;
            Self { w }
        }

        pub async fn handle_packets(&self, ack_interaction: &WasmAckInteraction, transport_cb: &js_sys::Function) {
            let msg_transport = |msg: Box<[u8]>, peer: String| -> Pin<Box<dyn Future<Output = Result<(), String>>>> {
                Box::pin(async move {
                    let this = JsValue::null();
                    let data: JsValue = js_sys::Uint8Array::from(msg.as_ref()).into();
                    let peer: JsValue = JsString::from(peer.as_str()).into();

                    match transport_cb.call2(&this, &data, &peer) {
                        Ok(r) => {
                            let promise = js_sys::Promise::from(r);
                            wasm_bindgen_futures::JsFuture::from(promise)
                                .await
                                .map(|_| ())
                                .map_err(|x| {
                                    x.dyn_ref::<JsString>()
                                        .map_or("Failed to send ping message".to_owned(), |x| -> String { x.into() })
                                })
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
            };
            self.w.handle_packets(ack_interaction.w.clone(), &msg_transport).await
        }

        pub fn stop(&self) {
            self.w.stop()
        }
    }
}
