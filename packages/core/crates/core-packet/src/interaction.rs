use crate::errors::PacketError::{
    AcknowledgementValidation, ChannelNotFound, InvalidPacketState, OutOfFunds, TagReplay, TicketValidation,
    TransportError,
};
use crate::errors::Result;
use crate::packet::{Packet, PacketState};
use crate::path::Path;
use async_std::channel::{unbounded, Receiver, Sender};
use core_crypto::types::{HalfKeyChallenge, Hash, PublicKey};
use core_db::traits::HoprCoreDbActions;
use core_mixer::mixer::Mixer;
use core_types::acknowledgement::{AcknowledgedTicket, Acknowledgement, PendingAcknowledgement, UnacknowledgedTicket};
use core_types::channels::{ChannelEntry, ChannelStatus, Ticket};
use libp2p_identity::PeerId;
use std::cell::RefCell;
use std::ops::Mul;
use std::sync::Arc;
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

pub struct AcknowledgementInteraction<Db: HoprCoreDbActions> {
    db: RefCell<Db>,
    pub on_acknowledgement: Box<dyn Fn(HalfKeyChallenge)>,
    pub on_acknowledged_ticket: Box<dyn Fn(AcknowledgedTicket)>,
    public_key: PublicKey,
    incoming_channel: (Sender<TransportTask>, Receiver<TransportTask>),
    outgoing_channel: (Sender<TransportTask>, Receiver<TransportTask>),
}

impl<Db: HoprCoreDbActions> AcknowledgementInteraction<Db> {
    pub fn new(db: Db, public_key: PublicKey) -> Self {
        Self {
            db: RefCell::new(db),
            public_key,
            incoming_channel: unbounded(),
            outgoing_channel: unbounded(),
            on_acknowledgement: Box::new(|_|{}),
            on_acknowledged_ticket: Box::new(|_|{}),
        }
    }

    pub fn get_peer_id(&self) -> PeerId {
        self.public_key.to_peerid()
    }

    pub async fn received_acknowledgement(&self, task: TransportTask) -> Result<()> {
        self.incoming_channel.0.send(task).await.map_err(|e| TransportError(e.to_string()))
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
        if !ack.validate(&self.public_key, &PublicKey::from_peerid(remote_peer)?) {
            return Err(AcknowledgementValidation(
                "could not validate the acknowledgement".to_string(),
            ));
        }

        println!("got incoming ack from: {}", remote_peer);

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
            .borrow()
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
                debug!("Received acknowledgement as sender. First relayer has processed the packet.");
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
                    .borrow()
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
                debug!("Acknowledging ticket. Using response {}", response.to_hex());

                let ack_ticket = AcknowledgedTicket::new(
                    unackowledged.ticket,
                    response,
                    Hash::new(&PREIMAGE_PLACE_HOLDER),
                    unackowledged.signer,
                );

                // replace the un-acked ticket with acked ticket.
                self.db
                    .borrow_mut()
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
pub struct PacketInteractionConfig {
    pub check_unrealized_balance: bool,
    pub private_key: Box<[u8]>,
}

pub struct PacketInteraction<Db>
where
    Db: HoprCoreDbActions,
{
    db: RefCell<Db>,
    channel: (Sender<TransportTask>, Receiver<TransportTask>),
    ack_interaction: Arc<AcknowledgementInteraction<Db>>,
    pub message_emitter: Box<dyn Fn(&[u8])>,
    cfg: PacketInteractionConfig,
}

impl<Db> PacketInteraction<Db>
where
    Db: HoprCoreDbActions,
{
    pub fn new(db: Db, ack_interaction: Arc<AcknowledgementInteraction<Db>>, cfg: PacketInteractionConfig) -> Self {
        Self {
            db: RefCell::new(db),
            channel: unbounded(),
            ack_interaction,
            message_emitter: Box::new(|_|{}),
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
                .borrow()
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
            .borrow()
            .get_current_ticket_index(channel_id)
            .await?
            .unwrap_or(U256::one());

        self.db
            .borrow_mut()
            .set_current_ticket_index(channel_id, current_ticket_index.addn(1))
            .await?;

        Ok(current_ticket_index)
    }

    async fn create_multihop_ticket(&self, destination: PublicKey, path_pos: u8) -> Result<Ticket> {
        let channel = self
            .db
            .borrow()
            .get_channel_to(&destination)
            .await?
            .ok_or(ChannelNotFound(destination.to_string()))?;

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
            .borrow()
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

        self.db.borrow_mut().mark_pending(&ticket).await?;

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
                    .borrow_mut()
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

    async fn handle_mixed_packet<T, F>(&self, mut packet: Packet, message_transport: &T) -> Result<()>
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
                if !self.db.borrow_mut().check_and_set_packet_tag(packet_tag).await? {
                    return Err(TagReplay);
                }

                // We're the destination of the packet, so emit the packet contents
                (self.message_emitter)(plain_text.as_ref());

                // And create acknowledgement
                let ack = packet.create_acknowledgement(&self.cfg.private_key).unwrap();
                self.ack_interaction
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
                if !self.db.borrow_mut().check_and_set_packet_tag(packet_tag).await? {
                    return Err(TagReplay);
                }

                let inverse_win_prob = U256::new(INVERSE_TICKET_WIN_PROB);

                // Find the corresponding channel
                let channel = self
                    .db
                    .borrow()
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
                    self.db.borrow_mut().mark_rejected(&packet.ticket).await?;
                    return Err(e);
                }

                self.db
                    .borrow_mut()
                    .set_current_ticket_index(&channel.get_id().hash(), packet.ticket.index)
                    .await?;

                // Store the unacknowledged ticket
                self.db
                    .borrow_mut()
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
        self.ack_interaction.send_acknowledgement(ack, previous_peer).await?;

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_FWD_MESSAGE_COUNT.increment();

        Ok(())
    }

    pub async fn handle_packets<T, F>(&self, message_transport: &T)
    where
        T: Fn(Box<[u8]>, String) -> F,
        F: futures::Future<Output = core::result::Result<(), String>>,
    {
        let mixer = Mixer::<TransportTask>::default();
        while let Ok(task) = self.channel.1.recv().await {
            // Add some random delay via mixer
            let mixed_packet = mixer.mix(task).await;
            match Packet::from_bytes(&mixed_packet.data, &self.cfg.private_key, &mixed_packet.remote_peer) {
                Ok(packet) => {
                    if let Err(e) = self.handle_mixed_packet(packet, message_transport).await {
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

    pub async fn push_received_packet(&mut self, packet_task: TransportTask) -> Result<()> {
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
    use std::collections::HashMap;
    use std::future::Future;
    use crate::errors::PacketError::PacketDbError;
    use crate::interaction::{AcknowledgementInteraction, PRICE_PER_PACKET, TransportTask};
    use async_trait::async_trait;
    use core_crypto::random::random_bytes;
    use core_crypto::types::{Hash, PublicKey};
    use core_ethereum_db::db::CoreEthereumDb;
    use core_ethereum_db::traits::HoprCoreEthereumDbActions;
    use core_ethereum_misc::commitment::{initialize_commitment, ChainCommitter, ChannelCommitmentInfo};
    use core_types::channels::{ChannelEntry, ChannelStatus};
    use hex_literal::hex;
    use libp2p_identity::PeerId;
    use std::ops::Mul;
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use async_std::channel::{Recv, RecvError};
    use futures::future::{Either, select};
    use futures::pin_mut;
    use lazy_static::lazy_static;
    use core_crypto::derivation::derive_ack_key_share;
    use core_db::db::CoreDb;
    use core_db::traits::HoprCoreDbActions;
    use core_types::acknowledgement::{Acknowledgement, AcknowledgementChallenge, PendingAcknowledgement};
    use utils_db::db::DB;
    use utils_db::errors::DbError;
    use utils_db::leveldb::rusty::RustyLevelDbShim;
    use utils_types::primitives::{Balance, BalanceType, Snapshot, U256};
    use utils_types::traits::{BinarySerializable, PeerIdLike, ToHex};
    use crate::por::ProofOfRelayValues;

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

    const SELF_PRIV: [u8; 32] = hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775");
    const RELAY0_PRIV: [u8; 32] = hex!("5bf21ea8cccd69aa784346b07bf79c84dac606e00eecaa68bf8c31aff397b1ca");
    const RELAY1_PRIV: [u8; 32] = hex!("3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa");
    const REALY2_PRIV: [u8; 32] = hex!("db7e3e8fcac4c817aa4cecee1d6e2b4d53da51f9881592c0e1cc303d8a012b92");
    const COUNTERPARTY_PRIV: [u8; 32] = hex!("0726a9704d56a013980a9077d195520a61b5aed28f92d89c50bca6e0e0c48cfc");

    fn create_peers() -> Vec<PeerId> {
        vec![SELF_PRIV, RELAY0_PRIV, RELAY1_PRIV, REALY2_PRIV, COUNTERPARTY_PRIV]
            .into_iter()
            .map(|private| PublicKey::from_privkey(&private).unwrap().to_peerid())
            .collect()
    }

    fn create_dbs(amount: usize) -> Vec<Arc<Mutex<DB<RustyLevelDbShim>>>> {
        (0..amount)
            .map(|i| {
                Arc::new(Mutex::new(DB::new(RustyLevelDbShim::new(
                    rusty_leveldb::DB::open(format!("test_db_{i}"), rusty_leveldb::in_memory()).unwrap(),
                ))))
            })
            .collect()
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

    async fn create_minimal_topology(
        dbs: &Vec<Arc<Mutex<DB<RustyLevelDbShim>>>>,
        nodes: &Vec<PeerId>,
    ) -> crate::errors::Result<()> {
        let testing_snapshot = Snapshot::new(U256::zero(), U256::zero(), U256::zero());
        let mut previous_channel: Option<ChannelEntry> = None;

        for (index, peer_id) in nodes.iter().enumerate() {
            let mut db = CoreEthereumDb::new(dbs[index].clone(), PublicKey::from_peerid(&peer_id).unwrap());

            let mut channel: Option<ChannelEntry> = None;

            if index < nodes.len() - 1 {
                channel = Some(create_dummy_channel(&peer_id, &nodes[index + 1]));

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

                initialize_commitment(&mut db, &SELF_PRIV, &channel_info, &mut EmptyChainCommiter {})
                    .await
                    .map_err(|e| PacketDbError(DbError::GenericError(e.to_string())))?;
            }

            previous_channel = channel;
        }

        Ok(())
    }

    #[derive(Clone, Eq, PartialEq, Debug)]
    struct Msg<T> {
        pub from: T,
        pub to: T,
        pub data: Box<[u8]>
    }

    lazy_static! {
        static ref MESSAGES: Mutex<HashMap<PeerId, Vec<(PeerId, Box<[u8]>)>>> = Mutex::new(HashMap::<PeerId, Vec<(PeerId,Box<[u8]>)>>::new());
    }

    pub async fn send_transport_as_peer<const N: usize>(msg: Box<[u8]>, dst: String) -> std::result::Result<(), String> {
        let sender = create_peers()[N];
        MESSAGES.lock().unwrap().get_mut(&PeerId::from_str(&dst).unwrap()).expect("non existent channel")
            .push((sender, msg));
        Ok(())
    }

    #[async_std::test]
    pub async fn test_packet_acknowledgement_sender_workflow() {
        let _ = env_logger::builder().is_test(true).try_init();

        let peers = create_peers();
        let dbs = create_dbs(peers.len());

        create_minimal_topology(&dbs, &peers)
            .await
            .expect("failed to create minimal channel topology");

        MESSAGES.lock().unwrap().clear();
        peers.iter().for_each(|peer| { MESSAGES.lock().unwrap().insert(peer.clone(), Vec::new()); });

        let mut core_dbs = dbs
            .iter()
            .enumerate().map(|(i, db)| CoreDb::new(
                db.clone(),
                PublicKey::from_peerid(&peers[i]).unwrap()
            )).collect::<Vec<_>>();


        // Begin test

        let secrets = (0..2).into_iter().map(|_| random_bytes::<32>()).collect::<Vec<_>>();
        let porv = ProofOfRelayValues::new(&secrets[0], Some(&secrets[1]))
            .expect("failed to create Proof of Relay values");

        // Mimics that the packet sender has sent a packet and now it has a pending acknowledgement in it's DB
        core_dbs[0].store_pending_acknowledgment(porv.ack_challenge, PendingAcknowledgement::WaitingAsSender).await
            .expect("failed to store pending ack");


        // Peer 1: This is mimics the ACK interaction of the packet sender
        let mut tmp_ack = AcknowledgementInteraction::new(core_dbs.remove(0), PublicKey::from_peerid(&peers[0]).unwrap());

        // ... which is just waiting to get an acknowledgement from the counterparty
        let (done_tx, done_rx) = async_std::channel::unbounded();
        tmp_ack.on_acknowledgement = Box::new(move |ack| {
            println!("sender has received acknowledgement: {}", ack.to_hex());
            if ack == porv.ack_challenge {
                // If it matches, set a signal that the test has finished
                done_tx.send_blocking(()).expect("send failed");
            }
        });

        let ack_interaction_sender = Arc::new(tmp_ack);

        // Peer 1: hookup receiving of acknowledgements
        let ack_1_clone_1 = ack_interaction_sender.clone();
        async_std::task::spawn_local(async move {
            loop {
                let tt ;
                {
                    if let Some(r) = MESSAGES.lock().unwrap().get_mut(&ack_1_clone_1.get_peer_id()) {
                        tt = r.iter().map(|(sender, data)| {
                            println!("received ack from {sender}: {}", hex::encode(&data));
                            TransportTask {
                                remote_peer: sender.clone(),
                                data: data.clone()
                            }
                        }).collect::<Vec<_>>();
                        r.clear();
                    } else {
                        break;
                    }
                }
                for task in tt {
                    ack_1_clone_1.received_acknowledgement(task).await.expect("failed to receive ack");
                }
                async_std::task::sleep(Duration::from_millis(200)).await;
            }
        });

        // Peer 1: start processing incoming acknowledgements on the packet sender
        let ack_1_clone_2 = ack_interaction_sender.clone();
        async_std::task::spawn_local(async move {
            ack_1_clone_2.handle_incoming_acknowledgements().await
        });

        // Peer 2: Recipient of the packet and sender of the acknowledgement
        let ack_interaction_counterparty = Arc::new(AcknowledgementInteraction::new(core_dbs.remove(0), PublicKey::from_peerid(&peers[1]).unwrap()));


        // Peer 2: start sending out outgoing acknowledgement
        let ack_2_clone = ack_interaction_counterparty.clone();
        async_std::task::spawn_local(async move {
            ack_2_clone.handle_outgoing_acknowledgements(&send_transport_as_peer::<1>).await;
        });

        ////

        let ack_key = derive_ack_key_share(&secrets[0]);
        let ack_msg = AcknowledgementChallenge::new(&ack_key.to_challenge(), &SELF_PRIV);

        assert!(ack_msg.solve(&ack_key.to_bytes()), "acknowledgement key must solve acknowledgement challenge");

        ack_interaction_counterparty.send_acknowledgement(
            Acknowledgement::new(ack_msg, ack_key, &COUNTERPARTY_PRIV),
            peers[0]
        ).await.expect("failed to send ack");


        let finish = done_rx.recv();
        let timeout = async_std::task::sleep(Duration::from_secs(10));
        pin_mut!(finish, timeout);

        match select(finish, timeout).await {
            Either::Left(_) => {}
            Either::Right(_) => {
                panic!("test timed out in 10 seconds")
            }
        }

        MESSAGES.lock().unwrap().clear();
        ack_interaction_sender.stop();
        ack_interaction_counterparty.stop();
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::interaction::{AcknowledgementInteraction, PacketInteraction, PacketInteractionConfig, TransportTask};
    use core_crypto::types::PublicKey;
    use core_db::db::CoreDb;
    use js_sys::JsString;
    use libp2p_identity::PeerId;
    use std::future::Future;
    use std::pin::Pin;
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};
    use utils_db::db::DB;
    use utils_db::leveldb::{LevelDb, LevelDbShim};
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;
    use core_types::acknowledgement::Acknowledgement;

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
        w: Arc<AcknowledgementInteraction<CoreDb<LevelDbShim>>>,
    }

    #[wasm_bindgen]
    impl WasmAckInteraction {
        pub fn new(db: LevelDb, chain_key: PublicKey) -> Self {
            Self {
                w: Arc::new(AcknowledgementInteraction::new(
                    CoreDb::new(Arc::new(Mutex::new(DB::new(LevelDbShim::new(db)))), chain_key.clone()),
                    chain_key,
                )),
            }
        }

        pub async fn received_new_acknowledgement(&self, task: TransportTask) -> JsResult<()> {
            ok_or_jserr!(self.w.received_acknowledgement(task).await)
        }

        pub async fn send_new_acknowledgement(&self, ack: Acknowledgement, dest: String) -> JsResult<()> {
            ok_or_jserr!(self.w.send_acknowledgement(ack, ok_or_jserr!(PeerId::from_str(&dest))?).await)
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
        w: PacketInteraction<CoreDb<LevelDbShim>>,
    }

    #[wasm_bindgen]
    impl WasmPacketInteraction {
        #[wasm_bindgen(constructor)]
        pub fn new(db: LevelDb, cfg: PacketInteractionConfig, ack: &WasmAckInteraction) -> Self {
            Self {
                w: PacketInteraction::new(
                    CoreDb::new(
                        Arc::new(Mutex::new(DB::new(LevelDbShim::new(db)))),
                        PublicKey::from_privkey(&cfg.private_key).expect("invalid private key"),
                    ),
                    ack.w.clone(),
                    cfg,
                ),
            }
        }

        pub async fn handle_packets(&self, transport_cb: &js_sys::Function) {
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
            self.w.handle_packets(&msg_transport).await
        }

        pub fn stop(&self) {
            self.w.stop()
        }
    }
}
