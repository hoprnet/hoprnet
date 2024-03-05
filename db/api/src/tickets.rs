use async_trait::async_trait;
use futures::TryStreamExt;
use hopr_crypto_types::prelude::*;
use hopr_db_entity::prelude::{Ticket, TicketStatistics};
use hopr_db_entity::ticket;
use hopr_db_entity::ticket_statistics;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};
use std::str::FromStr;
use std::time::SystemTime;
use hopr_db_entity::conversions::tickets::model_to_acknowledged_ticket;

use crate::db::HoprDb;
use crate::errors::{DbError, Result};
use crate::{HoprDbGeneralModelOperations, OptTx, SINGULAR_TABLE_FIXED_ID};


#[async_trait]
pub trait HoprDbTicketOperations {
    async fn get_ticket<'a>(
        &'a self,
        tx: OptTx<'a>,
        channel_id: Hash,
        epoch: u32,
        ticket_index: u64,
        // To be removed with https://github.com/hoprnet/hoprnet/pull/6018
        domain_separator: Hash,
        // To be removed with https://github.com/hoprnet/hoprnet/pull/6018
        chain_keypair: &ChainKeypair,
    ) -> Result<Option<AcknowledgedTicket>>;

    async fn insert_ticket<'a>(&'a self, tx: OptTx<'a>, acknowledged_ticket: AcknowledgedTicket) -> Result<()>;

    // async fn update_ticket_status_range(channel_id: &Hash, epoch: u32, new_status: AcknowledgedTicketStatus);

    // async fn compact_tickets(compacted_ticket: AcknowledgedTicket);

    // async fn get_ticket_stats(channel_id: &Hash, epoch: u32);

    async fn mark_ticket_redeemed<'a>(&'a self, tx: OptTx<'a>, ticket: &AcknowledgedTicket) -> Result<()>;

    async fn mark_tickets_neglected_in_epoch<'a>(&'a self, tx: OptTx<'a>, channel_id: Hash, epoch: u32) -> Result<()>;

    async fn get_ticket_statistics<'a>(&'a self, tx: OptTx<'a>) -> Result<AllTicketStatistics>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AllTicketStatistics {
    pub last_updated: SystemTime,
    pub losing_tickets: u64,
    pub neglected_tickets: u64,
    pub neglected_value: Balance,
    pub redeemed_tickets: u64,
    pub redeemed_value: Balance,
    pub unredeemed_tickets: u64,
    pub unredeemed_value: Balance,
    pub rejected_tickets: u64,
    pub rejected_value: Balance,
}

#[async_trait]
impl HoprDbTicketOperations for HoprDb {
    async fn get_ticket<'a>(
        &'a self,
        tx: OptTx<'a>,
        channel_id: Hash,
        epoch: u32,
        ticket_index: u64,
        domain_separator: Hash,
        chain_keypair: &ChainKeypair,
    ) -> Result<Option<AcknowledgedTicket>> {
        let ticket = self
            .nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    ticket::Entity::find()
                        .filter(ticket::Column::ChannelId.eq(channel_id.to_hex()))
                        .filter(ticket::Column::ChannelEpoch.eq(U256::from(epoch).to_be_bytes().to_vec()))
                        .filter(ticket::Column::Index.eq(ticket_index.to_be_bytes().to_vec()))
                        .one(tx.as_ref())
                        .await
                })
            })
            .await?;

        match ticket {
            None => Ok(None),
            Some(ticket_model) => Ok(Some(model_to_acknowledged_ticket(
                &ticket_model,
                domain_separator,
                chain_keypair,
            )?)),
        }
    }

    async fn insert_ticket<'a>(&'a self, tx: OptTx<'a>, acknowledged_ticket: AcknowledgedTicket) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    ticket::ActiveModel::from(acknowledged_ticket)
                        .insert(tx.as_ref())
                        .await
                })
            })
            .await?;
        Ok(())
    }

    async fn mark_ticket_redeemed<'a>(&'a self, tx: OptTx<'a>, ticket: &AcknowledgedTicket) -> Result<()> {
        let channel_id = ticket.ticket.channel_id;
        let epoch: U256 = ticket.ticket.channel_epoch.into();
        let index = ticket.ticket.index;
        let ticket_value = ticket.ticket.amount.amount();

        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    // Delete the ticket first
                    let deleted = ticket::Entity::delete_many()
                        .filter(ticket::Column::ChannelId.eq(channel_id.to_hex()))
                        .filter(ticket::Column::ChannelEpoch.eq(epoch.to_be_bytes().to_vec()))
                        .filter(ticket::Column::Index.eq(index.to_be_bytes().to_vec()))
                        .exec(tx.as_ref())
                        .await?;

                    // If the ticket has been deleted, update the stats
                    if deleted.rows_affected == 1 {
                        let stats = ticket_statistics::Entity::find_by_id(SINGULAR_TABLE_FIXED_ID)
                            .one(tx.as_ref())
                            .await?
                            .ok_or(DbError::CorruptedData)?;

                        let current_redeemed_count = stats.redeemed_tickets;
                        let current_redeemed_value = U256::from_be_bytes(&stats.redeemed_value);

                        let mut active_stats = stats.into_active_model();
                        active_stats.redeemed_tickets = Set(current_redeemed_count + 1);
                        active_stats.redeemed_value = Set((current_redeemed_value + ticket_value).to_be_bytes().into());
                        active_stats.save(tx.as_ref()).await?;

                        Ok::<(), DbError>(())
                    } else {
                        Err(DbError::LogicalError(format!("ticket #{index} in {channel_id}:{epoch} could not be deleted")))
                    }
                })
            })
            .await
    }

    async fn mark_tickets_neglected_in_epoch<'a>(&'a self, tx: OptTx<'a>, channel_id: Hash, epoch: u32) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    // Obtain the amount of neglected tickets and their value
                    let (neglectable_count, neglectable_value) = ticket::Entity::find()
                        .filter(ticket::Column::ChannelId.eq(channel_id.to_hex()))
                        .filter(ticket::Column::ChannelEpoch.eq(U256::from(epoch).to_be_bytes().to_vec()))
                        .stream(tx.as_ref())
                        .await?
                        .try_fold((0, U256::zero()), |(count, value), t| async move {
                            Ok((count + 1, value + U256::from_be_bytes(t.amount)))
                        })
                        .await?;

                    if neglectable_count > 0 {
                        // Delete the neglectable tickets first
                        let deleted = ticket::Entity::delete_many()
                            .filter(ticket::Column::ChannelId.eq(channel_id.to_hex()))
                            .filter(ticket::Column::ChannelEpoch.eq(U256::from(epoch).to_be_bytes().to_vec()))
                            .exec(tx.as_ref())
                            .await?;

                        // Update the stats if successful
                        if deleted.rows_affected == neglectable_count as u64 {
                            let stats = ticket_statistics::Entity::find_by_id(SINGULAR_TABLE_FIXED_ID)
                                .one(tx.as_ref())
                                .await?
                                .ok_or(DbError::CorruptedData)?;

                            let current_neglected_value = U256::from_be_bytes(stats.neglected_value.clone());
                            let current_neglected_count = stats.neglected_tickets;

                            let mut active_stats = stats.into_active_model();
                            active_stats.neglected_tickets = Set(current_neglected_count + neglectable_count);
                            active_stats.neglected_value =
                                Set((current_neglected_value + neglectable_value).to_be_bytes().into());
                            active_stats.save(tx.as_ref()).await?;

                            Ok(())
                        } else {
                            Err(DbError::LogicalError(format!(
                                "could not mark {neglectable_count} ticket as neglected"
                            )))
                        }
                    } else {
                        // No neglectable tickets found
                        Ok(())
                    }
                })
            })
            .await
    }

    async fn get_ticket_statistics<'a>(&'a self, tx: OptTx<'a>) -> Result<AllTicketStatistics> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let stats = TicketStatistics::find_by_id(SINGULAR_TABLE_FIXED_ID)
                        .one(tx.as_ref())
                        .await?
                        .ok_or(DbError::CorruptedData)?;

                    let (unredeemed_tickets, unredeemed_value) = Ticket::find()
                        .stream(tx.as_ref())
                        .await?
                        .try_fold((0_u64, U256::zero()), |(count, amount), x| async move {
                            Ok((count + 1, amount + U256::from_be_bytes(x.amount)))
                        })
                        .await?;

                    Ok::<AllTicketStatistics, DbError>(AllTicketStatistics {
                        last_updated: chrono::DateTime::<chrono::Utc>::from_str(&stats.last_updated)
                            .map_err(|_| DbError::CorruptedData)?
                            .into(),
                        losing_tickets: stats.losing_tickets as u64,
                        neglected_tickets: stats.neglected_tickets as u64,
                        neglected_value: BalanceType::HOPR.balance_bytes(stats.neglected_value),
                        redeemed_tickets: stats.redeemed_tickets as u64,
                        redeemed_value: BalanceType::HOPR.balance_bytes(stats.redeemed_value),
                        unredeemed_tickets,
                        unredeemed_value: BalanceType::HOPR.balance(unredeemed_value),
                        rejected_tickets: stats.rejected_tickets as u64,
                        rejected_value: BalanceType::HOPR.balance_bytes(stats.rejected_value),
                    })
                })
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use hopr_crypto_types::prelude::*;
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;

    use crate::channels::HoprDbChannelOperations;
    use crate::db::HoprDb;
    use crate::errors::DbError;
    use crate::tickets::HoprDbTicketOperations;
    use crate::HoprDbGeneralModelOperations;

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).unwrap();
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).unwrap();
    }

    const TICKET_VALUE: u64 = 100_000;

    fn generate_random_ack_ticket(index: u32) -> AcknowledgedTicket {
        let hk1 = HalfKey::random();
        let hk2 = HalfKey::random();

        let cp1: CurvePoint = hk1.to_challenge().into();
        let cp2: CurvePoint = hk2.to_challenge().into();
        let cp_sum = CurvePoint::combine(&[&cp1, &cp2]);

        let ticket = Ticket::new(
            &ALICE.public().to_address(),
            &BalanceType::HOPR.balance(TICKET_VALUE),
            index.into(),
            1_u32.into(),
            1.0f64,
            4u64.into(),
            Challenge::from(cp_sum).to_ethereum_challenge(),
            &BOB,
            &Hash::default(),
        )
        .unwrap();

        let unacked_ticket = UnacknowledgedTicket::new(ticket, hk1, BOB.public().to_address());
        unacked_ticket.acknowledge(&hk2, &ALICE, &Hash::default()).unwrap()
    }

    async fn init_db_with_tickets(db: &HoprDb, count_tickets: u64) -> (ChannelEntry, Vec<AcknowledgedTicket>) {
        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(100_u32),
            (count_tickets + 1).into(),
            ChannelStatus::Open,
            4_u32.into(),
        );

        let tickets = (0..count_tickets)
            .into_iter()
            .map(|i| generate_random_ack_ticket(i as u32))
            .collect::<Vec<_>>();

        let db_clone = db.clone();
        let tickets_clone = tickets.clone();
        db.begin_transaction()
            .await
            .unwrap()
            .perform(|tx| {
                Box::pin(async move {
                    db_clone.insert_channel(Some(tx), channel).await?;
                    for t in tickets_clone {
                        db_clone.insert_ticket(Some(tx), t).await?;
                    }
                    Ok::<(), DbError>(())
                })
            })
            .await
            .expect("tx should succeed");

        (channel, tickets)
    }

    #[async_std::test]
    async fn test_insert_get_ticket() {
        let db = HoprDb::new_in_memory().await;

        let (channel, mut tickets) = init_db_with_tickets(&db, 1).await;
        let ack_ticket = tickets.pop().unwrap();

        assert_eq!(channel.get_id(), ack_ticket.ticket.channel_id, "channel ids must match");
        assert_eq!(
            channel.channel_epoch.as_u32(),
            ack_ticket.ticket.channel_epoch,
            "epochs must match"
        );

        let db_ticket = db
            .get_ticket(
                None,
                channel.get_id(),
                ack_ticket.ticket.channel_epoch,
                ack_ticket.ticket.index,
                Hash::default(),
                &ALICE,
            )
            .await
            .expect("should get ticket")
            .expect("ticket should exist");

        assert_eq!(ack_ticket, db_ticket, "tickets must be equal");
    }

    #[async_std::test]
    async fn test_mark_redeemed() {
        let db = HoprDb::new_in_memory().await;
        const COUNT_TICKETS: u64 = 10;

        let (_, tickets) = init_db_with_tickets(&db, COUNT_TICKETS).await;

        let stats = db.get_ticket_statistics(None).await.unwrap();
        assert_eq!(
            COUNT_TICKETS, stats.unredeemed_tickets,
            "must have {COUNT_TICKETS} unredeemed"
        );
        assert_eq!(
            BalanceType::HOPR.balance(TICKET_VALUE * COUNT_TICKETS),
            stats.unredeemed_value,
            "unredeemed balance must match"
        );
        assert_eq!(0, stats.redeemed_tickets, "there must be no redeemed tickets");
        assert_eq!(
            BalanceType::HOPR.zero(),
            stats.redeemed_value,
            "there must be 0 redeemed value"
        );

        const TO_REDEEM: u64 = 2;
        let db_clone = db.clone();
        db.begin_transaction()
            .await
            .unwrap()
            .perform(|tx| {
                Box::pin(async move {
                    for i in 0..TO_REDEEM as usize {
                        db_clone.mark_ticket_redeemed(Some(tx), &tickets[i]).await?;
                    }
                    Ok::<(), DbError>(())
                })
            })
            .await
            .expect("tx must not fail");

        let stats = db.get_ticket_statistics(None).await.unwrap();
        assert_eq!(
            COUNT_TICKETS - TO_REDEEM,
            stats.unredeemed_tickets,
            "must have {COUNT_TICKETS} unredeemed"
        );
        assert_eq!(
            BalanceType::HOPR.balance(TICKET_VALUE * (COUNT_TICKETS - TO_REDEEM)),
            stats.unredeemed_value,
            "unredeemed balance must match"
        );
        assert_eq!(
            TO_REDEEM, stats.redeemed_tickets,
            "there must be {TO_REDEEM} redeemed tickets"
        );
        assert_eq!(
            BalanceType::HOPR.balance(TICKET_VALUE * TO_REDEEM),
            stats.redeemed_value,
            "there must be a redeemed value"
        );
    }

    #[async_std::test]
    async fn test_mark_redeem_should_not_mark_redeem_twice() {
        let db = HoprDb::new_in_memory().await;

        let ticket = init_db_with_tickets(&db, 1).await.1.pop().unwrap();

        db.mark_ticket_redeemed(None, &ticket)
            .await
            .expect("must not fail");
        db.mark_ticket_redeemed(None, &ticket)
            .await
            .expect_err("marking as redeemed again must fail");
    }

    #[async_std::test]
    async fn test_mark_tickets_neglected() {
        let db = HoprDb::new_in_memory().await;
        const COUNT_TICKETS: u64 = 10;

        let (channel, _) = init_db_with_tickets(&db, COUNT_TICKETS).await;

        let stats = db.get_ticket_statistics(None).await.unwrap();
        assert_eq!(
            COUNT_TICKETS, stats.unredeemed_tickets,
            "must have {COUNT_TICKETS} unredeemed"
        );
        assert_eq!(
            BalanceType::HOPR.balance(TICKET_VALUE * COUNT_TICKETS),
            stats.unredeemed_value,
            "unredeemed balance must match"
        );
        assert_eq!(0, stats.neglected_tickets, "there must be no redeemed tickets");
        assert_eq!(
            BalanceType::HOPR.zero(),
            stats.neglected_value,
            "there must be 0 redeemed value"
        );

        db.mark_tickets_neglected_in_epoch(None, channel.get_id(), channel.channel_epoch.as_u32())
            .await
            .expect("should mark as neglected");

        let stats = db.get_ticket_statistics(None).await.unwrap();
        assert_eq!(0, stats.unredeemed_tickets, "must have 0 unredeemed");
        assert_eq!(
            BalanceType::HOPR.zero(),
            stats.unredeemed_value,
            "unredeemed balance must be zero"
        );
        assert_eq!(
            COUNT_TICKETS, stats.neglected_tickets,
            "there must be no redeemed tickets"
        );
        assert_eq!(
            BalanceType::HOPR.balance(TICKET_VALUE * COUNT_TICKETS),
            stats.neglected_value,
            "there must be a neglected value"
        );
    }
}
