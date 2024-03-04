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

use crate::db::HoprDb;
use crate::errors::DbError::DecodingError;
use crate::errors::{DbError, Result};
use crate::{HoprDbGeneralModelOperations, OptTx, SINGULAR_TABLE_FIXED_ID};

/// TODO: implement as TryFrom trait once https://github.com/hoprnet/hoprnet/pull/6018 is merged
pub fn model_to_acknowledged_ticket(
    db_ticket: ticket::Model,
    domain_separator: &Hash,
    chain_keypair: &ChainKeypair,
) -> Result<AcknowledgedTicket> {
    let response = Response::from_bytes(&db_ticket.response)?;

    // To be refactored with https://github.com/hoprnet/hoprnet/pull/6018
    let mut ticket = hopr_internal_types::channels::Ticket::default();

    ticket.channel_id = Hash::from_hex(&db_ticket.channel_id)?;
    ticket.amount = BalanceType::HOPR.zero();
    ticket.index = U256::from_be_bytes(&db_ticket.index).as_u64();
    ticket.index_offset = db_ticket.index_offset as u32;
    ticket.channel_epoch = U256::from_be_bytes(&db_ticket.channel_epoch).as_u32();
    ticket.encoded_win_prob = db_ticket.winning_probability.try_into().map_err(|_| DecodingError)?;
    ticket.challenge = response.to_challenge().to_ethereum_challenge();
    ticket.signature = Some(Signature::from_bytes(&db_ticket.signature)?);

    let signer = ticket.recover_signer(domain_separator)?.to_address();
    Ok(AcknowledgedTicket::new(
        ticket,
        response,
        signer,
        chain_keypair,
        domain_separator,
    )?)
}

fn acknowledged_ticket_to_model(acknowledged_ticket: AcknowledgedTicket) -> ticket::ActiveModel {
    ticket::ActiveModel {
        channel_id: Set(acknowledged_ticket.ticket.channel_id.to_hex()),
        amount: Set(acknowledged_ticket.ticket.amount.amount().to_be_bytes().to_vec()),
        index: Set(acknowledged_ticket.ticket.index.to_be_bytes().to_vec()),
        index_offset: Set(acknowledged_ticket.ticket.index_offset as i32),
        winning_probability: Set(acknowledged_ticket.ticket.encoded_win_prob.to_vec()),
        channel_epoch: Set(acknowledged_ticket.ticket.channel_epoch.to_be_bytes().to_vec()),
        signature: Set(acknowledged_ticket.ticket.signature.unwrap().to_bytes().to_vec()),
        response: Set(acknowledged_ticket.response.to_bytes().to_vec()),
        ..Default::default()
    }
}

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

    async fn mark_ticket_redeemed<'a>(&'a self, tx: OptTx<'a>, ticket: ticket::Model) -> Result<()>;

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
                        .filter(ticket::Column::ChannelEpoch.eq(epoch.to_be_bytes().as_ref()))
                        .filter(ticket::Column::Index.eq(ticket_index.to_be_bytes().as_ref()))
                        .one(tx.as_ref())
                        .await
                })
            })
            .await?;

        match ticket {
            None => Ok(None),
            Some(ticket_model) => Ok(Some(model_to_acknowledged_ticket(
                ticket_model,
                &domain_separator,
                chain_keypair,
            )?)),
        }
    }

    async fn insert_ticket<'a>(&'a self, tx: OptTx<'a>, acknowledged_ticket: AcknowledgedTicket) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    acknowledged_ticket_to_model(acknowledged_ticket)
                        .insert(tx.as_ref())
                        .await
                })
            })
            .await?;
        Ok(())
    }

    async fn mark_ticket_redeemed<'a>(&'a self, tx: OptTx<'a>, ticket: ticket::Model) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let stats = ticket_statistics::Entity::find_by_id(SINGULAR_TABLE_FIXED_ID)
                        .one(tx.as_ref())
                        .await?
                        .ok_or(DbError::CorruptedData)?;

                    let current_redeemed_count = stats.redeemed_tickets;
                    let current_redeemed_value = U256::from_be_bytes(&stats.redeemed_value);
                    let ticket_value = U256::from_be_bytes(&ticket.amount);

                    let mut active_stats = stats.into_active_model();
                    active_stats.redeemed_tickets = Set(current_redeemed_count + 1);
                    active_stats.redeemed_value = Set((current_redeemed_value + ticket_value).to_be_bytes().into());
                    active_stats.save(tx.as_ref()).await?;

                    ticket::Entity::delete_by_id(ticket.id).exec(tx.as_ref()).await?;

                    Ok::<(), DbError>(())
                })
            })
            .await
    }

    async fn mark_tickets_neglected_in_epoch<'a>(&'a self, tx: OptTx<'a>, channel_id: Hash, epoch: u32) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let (neglectable_count, neglectable_value) = ticket::Entity::find()
                        .filter(ticket::Column::ChannelId.eq(channel_id.to_hex()))
                        .filter(ticket::Column::ChannelEpoch.eq(epoch.to_be_bytes().as_ref()))
                        .stream(tx.as_ref())
                        .await?
                        .try_fold((0, U256::zero()), |(count, value), t| async move {
                            Ok((count + 1, value + U256::from_be_bytes(t.amount)))
                        })
                        .await?;

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

                    ticket::Entity::delete_many()
                        .filter(ticket::Column::ChannelId.eq(channel_id.to_hex()))
                        .filter(ticket::Column::ChannelEpoch.eq(epoch.to_be_bytes().as_ref()))
                        .exec(tx.as_ref())
                        .await?;

                    Ok::<(), DbError>(())
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
    use crate::channels::HoprDbChannelOperations;
    use crate::db::HoprDb;
    use crate::tickets::HoprDbTicketOperations;
    use crate::HoprDbGeneralModelOperations;
    use hex_literal::hex;
    use hopr_crypto_types::prelude::*;
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).unwrap();
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).unwrap();
    }

    fn generate_random_ack_ticket() -> AcknowledgedTicket {
        let counterparty = &BOB;
        let hk1 = HalfKey::random();
        let hk2 = HalfKey::random();

        let cp1: CurvePoint = hk1.to_challenge().into();
        let cp2: CurvePoint = hk2.to_challenge().into();
        let cp_sum = CurvePoint::combine(&[&cp1, &cp2]);

        let ticket = Ticket::new(
            &ALICE.public().to_address(),
            &Balance::new(100_u32, BalanceType::HOPR),
            0_u32.into(),
            1_u32.into(),
            1.0f64,
            4u64.into(),
            Challenge::from(cp_sum).to_ethereum_challenge(),
            counterparty,
            &Hash::default(),
        )
        .unwrap();

        let unacked_ticket = UnacknowledgedTicket::new(ticket, hk1, counterparty.public().to_address());
        unacked_ticket.acknowledge(&hk2, &ALICE, &Hash::default()).unwrap()
    }

    /*#[async_std::test]
    async fn test_insert_get_ticket() {
        let db = HoprDb::new_in_memory().await;

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(100_u32),
            1_u32.into(),
            ChannelStatus::Open,
            4_u32.into()
        );

        let ack_ticket = generate_random_ack_ticket();

        assert_eq!(channel.get_id(), ack_ticket.ticket.channel_id, "channel ids must match");
        assert_eq!(channel.channel_epoch.as_u32(), ack_ticket.ticket.channel_epoch, "epochs must match");

        let db_clone = db.clone();
        let ack_clone = ack_ticket.clone();
        db.begin_transaction().await.unwrap().perform(|tx| Box::pin(async move {
            db_clone.insert_channel(Some(tx), channel).await?;
            db_clone.insert_ticket(Some(tx), ack_clone).await
        }))
        .await.expect("tx should succeed");

        let db_ticket = db.get_ticket(None, channel.get_id(), 4, 0, Hash::default(), &ALICE)
            .await
            .expect("should get ticket")
            .expect("ticket should exist");

        assert_eq!(ack_ticket, db_ticket, "tickets must be equal");
    }*/
}
