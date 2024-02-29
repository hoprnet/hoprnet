use async_trait::async_trait;
use futures::TryStreamExt;
use hopr_crypto_types::prelude::*;
use hopr_db_entity::prelude::{Ticket, TicketStatistics};
use hopr_db_entity::ticket;
use hopr_db_entity::ticket_statistics;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use sea_orm::{EntityTrait, QueryOrder, QuerySelect};
use std::ops::Add;
use std::str::FromStr;
use std::time::SystemTime;

use crate::db::HoprDb;
use crate::errors::{DbError, Result};

use crate::db::HoprDb;
use crate::errors::Result;

#[async_trait]
pub trait HoprDbTicketOperations {
    // async fn get_tickets(channel_id: &Hash, epoch: u32);

    async fn add_ticket(&self, ticket: &AcknowledgedTicket) -> Result<()>;

    async fn remove_ticket(&self, channel_id: &Hash, epoch: u32, ticket_index: u64) -> Result<()>;

    async fn get_ticket(
        &self,
        channel_id: &Hash,
        epoch: u32,
        ticket_index: u64,
        // To be removed with https://github.com/hoprnet/hoprnet/pull/6018
        domain_separator: &Hash,
        // To be removed with https://github.com/hoprnet/hoprnet/pull/6018
        chain_keypair: &ChainKeypair,
    ) -> Result<Option<AcknowledgedTicket>>;

    // async fn update_ticket_status_range(channel_id: &Hash, epoch: u32, new_status: AcknowledgedTicketStatus);

    // async fn compact_tickets(compacted_ticket: AcknowledgedTicket);

    // async fn get_ticket_stats(channel_id: &Hash, epoch: u32);

    async fn get_ticket_statistics(&self) -> Result<AllTicketStatistics>;
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
    async fn get_ticket(
        &self,
        channel_id: &Hash,
        epoch: u32,
        ticket_index: u64,
        // To be removed with https://github.com/hoprnet/hoprnet/pull/6018
        domain_separator: &Hash,
        // To be removed with https://github.com/hoprnet/hoprnet/pull/6018
        chain_keypair: &ChainKeypair,
    ) -> Result<Option<AcknowledgedTicket>> {
        // let ticket
        match ticket::Entity::find()
            .filter(ticket::Column::ChannelId.eq(channel_id.to_hex()))
            .filter(ticket::Column::ChannelEpoch.eq(u32_to_i32(epoch)))
            .filter(ticket::Column::Index.eq(u64_to_i64(ticket_index)))
            .one(&self.db)
            .await?
        {
            Some(db_ticket) => Ok(Some(
                AcknowledgedTicket::try_from_with_domain_separator_and_chain_keypair(
                    db_ticket,
                    domain_separator,
                    chain_keypair,
                )?,
            )),
            None => Ok(None),
        }
    }

    async fn remove_ticket(&self, channel_id: &Hash, epoch: u32, ticket_index: u64) -> Result<()> {
        ticket::Entity::delete_many()
            .filter(ticket::Column::ChannelId.eq(channel_id.to_hex()))
            .filter(ticket::Column::ChannelEpoch.eq(u32_to_i32(epoch)))
            .filter(ticket::Column::Index.eq(u64_to_i64(ticket_index)))
            .exec(&self.db)
            .await?;

        Ok(())
    }

    async fn add_ticket(&self, ticket: &AcknowledgedTicket) -> Result<()> {
        // ticket::Entity::insert(todo!()).exec(&self.db).await?;

        // Ok(())
        todo!()
    }

    async fn get_ticket_statistics(&self) -> Result<AllTicketStatistics> {
        let stats = TicketStatistics::find()
            .order_by_desc(ticket_statistics::Column::LastUpdated)
            .limit(1)
            .one(&self.db)
            .await?
            .ok_or(DbError::NotFound)?;

        let (unredeemed_tickets, unredeemed_value) = Ticket::find()
            .stream(&self.db)
            .await?
            .try_fold(
                (0_u64, Balance::zero(BalanceType::HOPR)),
                |(count, amount), x| async move {
                    Ok((
                        count + 1,
                        amount.add(Balance::new(U256::from_big_endian(&x.amount), BalanceType::HOPR)),
                    ))
                },
            )
            .await?;

        Ok(AllTicketStatistics {
            last_updated: chrono::DateTime::<chrono::Utc>::from_str(&stats.last_updated)
                .map_err(|_| DbError::CorruptedData)?
                .into(),
            losing_tickets: stats.losing_tickets as u64,
            neglected_tickets: stats.neglected_tickets as u64,
            neglected_value: Balance::new(U256::from_big_endian(&stats.neglected_value), BalanceType::HOPR),
            redeemed_tickets: stats.redeemed_tickets as u64,
            redeemed_value: Balance::new(U256::from_big_endian(&stats.redeemed_value), BalanceType::HOPR),
            unredeemed_tickets,
            unredeemed_value,
            rejected_tickets: stats.rejected_tickets as u64,
            rejected_value: Balance::new(U256::from_big_endian(&stats.rejected_value), BalanceType::HOPR),
        })
    }
}

#[cfg(test)]
mod tests {}
