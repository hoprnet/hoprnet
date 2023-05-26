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
use utils_log::{debug, error, info};
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
    pub on_acknowledgement: fn(HalfKeyChallenge),
    pub on_acknowledged_ticket: fn(AcknowledgedTicket),
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
            on_acknowledgement: |_| {},
            on_acknowledged_ticket: |_| {},
        }
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
    pub message_emitter: fn(&[u8]),
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
            message_emitter: |_| {},
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
    use crate::errors::PacketError::PacketDbError;
    use crate::interaction::PRICE_PER_PACKET;
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
    use std::sync::{Arc, Mutex};
    use utils_db::db::DB;
    use utils_db::errors::DbError;
    use utils_db::leveldb::rusty::RustyLevelDbShim;
    use utils_types::primitives::{Balance, BalanceType, Snapshot, U256};
    use utils_types::traits::PeerIdLike;

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

    #[async_std::test]
    pub async fn test_packet_acknowledgement_sender_workflow() {
        let peers = create_peers();
        let dbs = create_dbs(peers.len());

        create_minimal_topology(&dbs, &peers)
            .await
            .expect("failed to create minimal channel topology");
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
