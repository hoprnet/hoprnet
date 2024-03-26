use async_trait::async_trait;
use hopr_crypto_packet::chain::ChainPacketComponents;
use hopr_crypto_packet::validation::validate_unacknowledged_ticket;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use std::ops::Sub;
use tracing::{debug, instrument, trace, warn};

use crate::channels::HoprDbChannelOperations;
use crate::db::HoprDb;
use crate::errors::DbError;
use crate::info::HoprDbInfoOperations;
use crate::prelude::{HoprDbResolverOperations, HoprDbTicketOperations};
use crate::{HoprDbGeneralModelOperations, OptTx};

/// Trait defining all DB functionality needed by packet/acknowledgement processing pipeline.
#[async_trait]
pub trait HoprDbProtocolOperations {
    /// Processes the acknowledgements for the pending tickets
    ///
    /// There are three cases:
    /// 1. There is an unacknowledged ticket and we are awaiting a half key.
    /// 2. We were the creator of the packet, hence we do not wait for any half key
    /// 3. The acknowledgement is unexpected and stems from a protocol bug or an attacker
    async fn handle_acknowledgement(&self, ack: Acknowledgement, me: ChainKeypair) -> crate::errors::Result<AckResult>;

    /// Process the data into an outgoing packet
    async fn to_send(
        &self,
        data: Box<[u8]>,
        me: ChainKeypair,
        path: Vec<OffchainPublicKey>,
    ) -> crate::errors::Result<TransportPacketWithChainData>;

    /// Process the incoming packet into data
    #[allow(clippy::wrong_self_convention)]
    async fn from_recv(
        &self,
        data: Box<[u8]>,
        me: ChainKeypair,
        pkt_keypair: &OffchainKeypair,
        sender: OffchainPublicKey,
    ) -> crate::errors::Result<TransportPacketWithChainData>;
}

#[allow(clippy::large_enum_variant)] // TODO: Uses too large objects
#[derive(Debug)]
pub enum AckResult {
    Sender(HalfKeyChallenge),
    RelayerWinning(AcknowledgedTicket),
    RelayerLosing,
}

pub enum TransportPacketWithChainData {
    /// Packet is intended for us
    Final {
        packet_tag: PacketTag,
        previous_hop: OffchainPublicKey,
        plain_text: Box<[u8]>,
        ack: Acknowledgement,
    },
    /// Packet must be forwarded
    Forwarded {
        packet_tag: PacketTag,
        previous_hop: OffchainPublicKey,
        next_hop: OffchainPublicKey,
        data: Box<[u8]>,
        ack: Acknowledgement,
    },
    /// Packet that is being sent out by us
    Outgoing {
        next_hop: OffchainPublicKey,
        ack_challenge: HalfKeyChallenge,
        data: Box<[u8]>,
    },
}

#[allow(clippy::large_enum_variant)] // TODO: Uses too large objects
enum ResolvedAcknowledgement {
    Sending(HalfKeyChallenge),
    RelayingWin(AcknowledgedTicket),
    RelayingLoss(Hash),
}

impl From<ResolvedAcknowledgement> for AckResult {
    fn from(value: ResolvedAcknowledgement) -> Self {
        match value {
            ResolvedAcknowledgement::Sending(ack_challenge) => AckResult::Sender(ack_challenge),
            ResolvedAcknowledgement::RelayingWin(ack_ticket) => AckResult::RelayerWinning(ack_ticket),
            ResolvedAcknowledgement::RelayingLoss(_) => AckResult::RelayerLosing,
        }
    }
}

#[async_trait]
impl HoprDbProtocolOperations for HoprDb {
    #[instrument(level = "trace", skip(self))]
    async fn handle_acknowledgement(&self, ack: Acknowledgement, me: ChainKeypair) -> crate::errors::Result<AckResult> {
        let myself = self.clone();
        let me_onchain = me.public().to_address();

        let result = self
            .begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    match myself
                        .caches
                        .unacked_tickets
                        .remove(&ack.ack_challenge())
                        .await
                        .ok_or_else(|| {
                            crate::errors::DbError::AcknowledgementValidationError(format!(
                                "received unexpected acknowledgement for half key challenge {} - half key {}",
                                ack.ack_challenge().to_hex(),
                                ack.ack_key_share.to_hex()
                            ))
                        })? {
                        PendingAcknowledgement::WaitingAsSender => {
                            trace!("received acknowledgement as sender: first relayer has processed the packet.");

                            Ok(ResolvedAcknowledgement::Sending(ack.ack_challenge()))
                        }

                        PendingAcknowledgement::WaitingAsRelayer(unacknowledged) => {
                            // Try to unlock the incentive
                            unacknowledged.verify_challenge(&ack.ack_key_share).map_err(|e| {
                                crate::errors::DbError::AcknowledgementValidationError(format!(
                                    "the acknowledgement is not sufficient to solve the embedded challenge, {e}"
                                ))
                            })?;

                            if myself
                                .get_channel_by_parties(Some(tx), &unacknowledged.signer, &me_onchain)
                                .await?
                                .is_some_and(|c| c.channel_epoch.as_u32() != unacknowledged.ticket.channel_epoch)
                            {
                                return Err(crate::errors::DbError::LogicalError(format!(
                                    "no channel found for  address '{}'",
                                    unacknowledged.signer
                                )));
                            }

                            let domain_separator =
                                myself.get_indexer_data(Some(tx)).await?.channels_dst.ok_or_else(|| {
                                    crate::errors::DbError::LogicalError("domain separator missing".into())
                                })?;

                            let ack_ticket = unacknowledged.acknowledge(&ack.ack_key_share, &me, &domain_separator)?;

                            if ack_ticket.is_winning_ticket(&domain_separator) {
                                debug!(ticket = tracing::field::display(&ack_ticket), "winning ticket");
                                Ok(ResolvedAcknowledgement::RelayingWin(ack_ticket))
                            } else {
                                trace!(ticket = tracing::field::display(&ack_ticket), "losing ticket");
                                Ok(ResolvedAcknowledgement::RelayingLoss(ack_ticket.ticket.channel_id))
                            }
                        }
                    }
                })
            })
            .await?;

        match &result {
            ResolvedAcknowledgement::RelayingWin(ack_ticket) => {
                self.ticket_manager.insert_ticket(ack_ticket.clone()).await?;

                #[cfg(all(feature = "prometheus", not(test)))]
                {
                    let channel = ack_ticket.ticket.channel_id.to_string();
                    crate::tickets::METRIC_HOPR_TICKETS_INCOMING_STATISTICS.set(
                        &[&channel, "unredeemed"],
                        self.ticket_manager
                            .unrealized_value(crate::tickets::TicketSelector::new(
                                ack_ticket.ticket.channel_id,
                                ack_ticket.ticket.channel_epoch,
                            ))
                            .await?
                            .amount()
                            .as_u128() as f64,
                    );
                    crate::tickets::METRIC_HOPR_TICKETS_INCOMING_STATISTICS
                        .increment(&[&channel, "winning_count"], 1.0f64);
                }
            }
            ResolvedAcknowledgement::RelayingLoss(_channel) => {
                #[cfg(all(feature = "prometheus", not(test)))]
                {
                    crate::tickets::METRIC_HOPR_TICKETS_INCOMING_STATISTICS
                        .increment(&[&_channel.to_string(), "losing_count"], 1.0f64);
                }
            }
            _ => {}
        };

        Ok(result.into())
    }

    #[instrument(level = "trace", skip(self))]
    async fn to_send(
        &self,
        data: Box<[u8]>,
        me: ChainKeypair,
        path: Vec<OffchainPublicKey>,
    ) -> crate::errors::Result<TransportPacketWithChainData> {
        let myself = self.clone();

        let components = self
            .begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let next_peer = myself.resolve_chain_key(&path[0]).await?.ok_or_else(|| {
                        crate::errors::DbError::LogicalError(format!(
                            "failed to find channel key for packet key {} on previous hop",
                            path[0].to_peerid_str()
                        ))
                    })?;

                    let domain_separator = myself.get_indexer_data(Some(tx)).await?.channels_dst.ok_or_else(|| {
                        crate::errors::DbError::LogicalError("failed to fetch the domain separator".into())
                    })?;

                    // Decide whether to create 0-hop or multihop ticket
                    let next_ticket = if path.len() == 1 {
                        hopr_internal_types::channels::Ticket::new_zero_hop(&next_peer, &me, &domain_separator).map_err(
                            |e| {
                                crate::errors::DbError::LogicalError(format!("failed to construct a 0 hop ticket: {e}"))
                            },
                        )
                    } else {
                        myself
                            .create_multihop_ticket(Some(tx), me.public().to_address(), next_peer, path.len() as u8)
                            .await
                    }?;

                    ChainPacketComponents::into_outgoing(&data, &path, &me, next_ticket, &domain_separator).map_err(
                        |e| {
                            crate::errors::DbError::LogicalError(format!(
                                "failed to construct chain components for a packet: {e}"
                            ))
                        },
                    )
                })
            })
            .await?;

        match components {
            ChainPacketComponents::Final { .. } | ChainPacketComponents::Forwarded { .. } => Err(
                crate::errors::DbError::LogicalError("Must contain an outgoing packet type".into()),
            ),
            ChainPacketComponents::Outgoing {
                packet,
                ticket,
                next_hop,
                ack_challenge,
            } => {
                self.caches
                    .unacked_tickets
                    .insert(ack_challenge, PendingAcknowledgement::WaitingAsSender)
                    .await;

                let mut payload = Vec::with_capacity(ChainPacketComponents::SIZE);
                payload.extend_from_slice(packet.as_ref());
                payload.extend_from_slice(&ticket.to_bytes());

                Ok(TransportPacketWithChainData::Outgoing {
                    next_hop,
                    ack_challenge,
                    data: payload.into_boxed_slice(),
                })
            }
        }
    }

    #[instrument(level = "trace", skip(self))]
    async fn from_recv(
        &self,
        data: Box<[u8]>,
        me: ChainKeypair,
        pkt_keypair: &OffchainKeypair,
        sender: OffchainPublicKey,
    ) -> crate::errors::Result<TransportPacketWithChainData> {
        let me_onchain = me.public().to_address();
        match ChainPacketComponents::from_incoming(&data, pkt_keypair, sender)
            .map_err(|e| crate::errors::DbError::LogicalError(format!("failed to construct an incoming packet: {e}")))?
        {
            ChainPacketComponents::Final {
                packet_tag,
                ack_key,
                previous_hop,
                plain_text,
                ..
            } => {
                let ack = Acknowledgement::new(ack_key, pkt_keypair);

                Ok(TransportPacketWithChainData::Final {
                    packet_tag,
                    previous_hop,
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
                let myself = self.clone();

                let t = match self
                    .begin_transaction()
                    .await?
                    .perform(|tx| {
                        Box::pin(async move {
                            let chain_data = myself.get_indexer_data(Some(tx)).await?;

                            let domain_separator = chain_data.channels_dst.ok_or_else(|| {
                                crate::errors::DbError::LogicalError("failed to fetch the domain separator".into())
                            })?;
                            let ticket_price = chain_data.ticket_price.ok_or_else(|| {
                                crate::errors::DbError::LogicalError("failed to fetch the ticket price".into())
                            })?;

                            let previous_hop_addr =
                                myself.resolve_chain_key(&previous_hop).await?.ok_or_else(|| {
                                    crate::errors::DbError::LogicalError(format!(
                                        "failed to find channel key for packet key {} on previous hop",
                                        previous_hop.to_peerid_str()
                                    ))
                                })?;

                            let next_hop_addr = myself.resolve_chain_key(&next_hop).await?.ok_or_else(|| {
                                crate::errors::DbError::LogicalError(format!(
                                    "failed to find channel key for packet key {} on next hop",
                                    next_hop.to_peerid_str()
                                ))
                            })?;

                            // TODO: cache this DB call too, or use the channel graph
                            let channel = myself
                                .get_channel_by_parties(Some(tx), &previous_hop_addr, &me_onchain)
                                .await?
                                .ok_or_else(|| {
                                    crate::errors::DbError::LogicalError(format!(
                                        "no channel found for previous hop address '{previous_hop_addr}'"
                                    ))
                                })?;

                            let remaining_balance = channel
                                .balance
                                .sub(myself.ticket_manager.unrealized_value((&channel).into()).await?);

                            if let Err(e) = validate_unacknowledged_ticket(
                                &ticket,
                                &channel,
                                &previous_hop_addr,
                                ticket_price,
                                TICKET_WIN_PROB,
                                Some(remaining_balance),
                                &domain_separator,
                            )
                            .await
                            {
                                return Err(crate::errors::DbError::TicketValidationError(Box::new((
                                    ticket,
                                    e.to_string(),
                                ))));
                            }

                            myself.increment_outgoing_ticket_index(channel.get_id()).await?;

                            myself
                                .caches
                                .unacked_tickets
                                .insert(
                                    ack_challenge,
                                    PendingAcknowledgement::WaitingAsRelayer(UnacknowledgedTicket::new(
                                        ticket.clone(),
                                        own_key.clone(),
                                        previous_hop_addr,
                                    )),
                                )
                                .await;

                            // Check that the calculated path position from the ticket matches value from the packet header
                            let ticket_path_pos = ticket.get_path_position(ticket_price.amount())?;
                            if !ticket_path_pos.eq(&path_pos) {
                                return Err(crate::errors::DbError::LogicalError(format!(
                                    "path position mismatch: from ticket {ticket_path_pos}, from packet {path_pos}"
                                )));
                            }

                            // Create next ticket for the packet
                            let mut ticket = if ticket_path_pos == 1 {
                                Ok(hopr_internal_types::channels::Ticket::new_zero_hop(
                                    &next_hop_addr,
                                    &me,
                                    &domain_separator,
                                )?)
                            } else {
                                myself
                                    .create_multihop_ticket(
                                        Some(tx),
                                        me.public().to_address(),
                                        next_hop_addr,
                                        ticket_path_pos,
                                    )
                                    .await
                            }?;

                            // forward packet
                            ticket.challenge = next_challenge.to_ethereum_challenge();
                            ticket.sign(&me, &domain_separator);

                            Ok(ticket)
                        })
                    })
                    .await
                {
                    Ok(ticket) => Ok(ticket),
                    Err(crate::errors::DbError::TicketValidationError(boxed_error)) => {
                        let (rejected_ticket, error) = *boxed_error;
                        let rejected_value = rejected_ticket.amount;
                        warn!("encountered validation error during forwarding for {rejected_ticket} with value: {rejected_value}");

                        self.mark_ticket_rejected(&rejected_ticket).await.map_err(|e| {
                            crate::errors::DbError::TicketValidationError(Box::new((
                                rejected_ticket.clone(),
                                format!("during validation error '{error}' update another error occurred: {e}"),
                            )))
                        })?;

                        Err(crate::errors::DbError::TicketValidationError(Box::new((
                            rejected_ticket,
                            error,
                        ))))
                    }
                    Err(e) => Err(e),
                }?;

                let ack = Acknowledgement::new(ack_key, pkt_keypair);

                let mut payload = Vec::with_capacity(ChainPacketComponents::SIZE);
                payload.extend_from_slice(packet.as_ref());
                payload.extend_from_slice(&t.to_bytes());

                Ok(TransportPacketWithChainData::Forwarded {
                    packet_tag,
                    previous_hop,
                    next_hop,
                    data: payload.into_boxed_slice(),
                    ack,
                })
            }
            ChainPacketComponents::Outgoing { .. } => Err(crate::errors::DbError::LogicalError(
                "Cannot receive an outgoing packet".into(),
            )),
        }
    }
}

impl HoprDb {
    async fn create_multihop_ticket<'a>(
        &'a self,
        tx: OptTx<'a>,
        me_onchain: Address,
        destination: Address,
        path_pos: u8,
    ) -> crate::errors::Result<hopr_internal_types::channels::Ticket> {
        let myself = self.clone();
        let (channel, ticket_price): (ChannelEntry, U256) = self
            .nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    Ok::<_, DbError>(
                        if let Some(channel) = myself
                            .get_channel_by_parties(Some(tx), &me_onchain, &destination)
                            .await?
                        {
                            let ticket_price = myself.get_indexer_data(Some(tx)).await?.ticket_price;

                            Some((
                                channel,
                                ticket_price
                                    .ok_or(DbError::LogicalError("missing ticket price".into()))?
                                    .amount(),
                            ))
                        } else {
                            None
                        },
                    )
                })
            })
            .await?
            .ok_or(crate::errors::DbError::LogicalError(format!(
                "channel to '{destination}' not found",
            )))?;

        let amount = Balance::new(
            ticket_price.div_f64(TICKET_WIN_PROB).map_err(|e| {
                crate::errors::DbError::LogicalError(format!(
                    "winning probability outside of the allowed interval (0.0, 1.0]: {e}"
                ))
            })? * U256::from(path_pos - 1),
            BalanceType::HOPR,
        );

        if channel.balance.lt(&amount) {
            return Err(crate::errors::DbError::LogicalError(format!(
                "out of funds: {} with counterparty {destination} has balance {} < {amount}",
                channel.get_id(),
                channel.balance
            )));
        }

        let ticket = hopr_internal_types::channels::Ticket::new_partial(
            &me_onchain,
            &destination,
            &amount,
            self.increment_outgoing_ticket_index(channel.get_id()).await?.into(),
            U256::one(), // unaggregated always have index_offset == 1
            TICKET_WIN_PROB,
            channel.channel_epoch,
        )
        .map_err(|e| crate::errors::DbError::LogicalError(format!("failed to construct a ticket: {e}")))?;

        //         #[cfg(all(feature = "prometheus", not(test)))]
        //         METRIC_TICKETS_COUNT.increment();

        Ok(ticket)
    }
}

// TODO: think about incorporating these tests
// #[async_std::test]
// async fn test_ticket_workflow() {
//     let mut db = CoreEthereumDb::new(
//         DB::new(CurrentDbShim::new_in_memory().await),
//         SENDER_PRIV_KEY.public().to_address(),
//     );

//     let hkc = HalfKeyChallenge::new(&random_bytes::<{ HalfKeyChallenge::SIZE }>());
//     let unack = UnacknowledgedTicket::new(
//         create_valid_ticket(),
//         HalfKey::new(&random_bytes::<{ HalfKey::SIZE }>()),
//         SENDER_PRIV_KEY.public().to_address(),
//     );

//     db.store_pending_acknowledgment(hkc, PendingAcknowledgement::WaitingAsRelayer(unack))
//         .await
//         .unwrap();
//     let num_tickets = db.get_tickets(None).await.unwrap();
//     assert_eq!(1, num_tickets.len(), "db should find one ticket");

//     let pending = db
//         .get_pending_acknowledgement(&hkc)
//         .await
//         .unwrap()
//         .expect("db should contain pending ack");
//     match pending {
//         PendingAcknowledgement::WaitingAsSender => panic!("must not be pending as sender"),
//         PendingAcknowledgement::WaitingAsRelayer(ticket) => {
//             let ack = ticket
//                 .acknowledge(&HalfKey::default(), &TARGET_PRIV_KEY, &Hash::default())
//                 .unwrap();
//             db.replace_unack_with_ack(&hkc, ack).await.unwrap();

//             let num_tickets = db.get_tickets(None).await.unwrap().len();
//             let num_unack = db.get_unacknowledged_tickets(None).await.unwrap().len();
//             let num_ack = db.get_acknowledged_tickets(None).await.unwrap().len();
//             assert_eq!(1, num_tickets, "db should find one ticket");
//             assert_eq!(0, num_unack, "db should not contain any unacknowledged tickets");
//             assert_eq!(1, num_ack, "db should contain exactly one acknowledged ticket");
//         }
//     }
// }
