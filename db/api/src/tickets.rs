use async_trait::async_trait;
use futures::TryStreamExt;
use hopr_crypto_types::prelude::*;
use hopr_db_entity::prelude::{Ticket, TicketStatistics};
use hopr_db_entity::ticket;
use hopr_db_entity::ticket_statistics;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter,
    Set, StreamTrait,
};
use std::str::FromStr;
use std::time::SystemTime;

use crate::db::HoprDb;
use crate::errors::{DbError, Result};

pub fn model_to_acknowledged_ticket(_ticket: &ticket::Model) -> Result<AcknowledgedTicket> {
    todo!()
}

#[async_trait]
pub trait HoprDbTicketOperations {
    async fn get_ticket<C: ConnectionTrait>(
        &self,
        txc: &C,
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

    async fn mark_ticket_redeemed<C: ConnectionTrait>(&self, txc: &C, ticket: &ticket::Model) -> Result<()>;

    async fn mark_tickets_neglected_in_epoch<C: ConnectionTrait + StreamTrait>(
        &self,
        txc: &C,
        channel_id: &Hash,
        epoch: u32,
    ) -> Result<()>;

    async fn get_ticket_statistics<C: ConnectionTrait + StreamTrait>(&self, txc: &C) -> Result<AllTicketStatistics>;
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
    async fn get_ticket<C: ConnectionTrait>(
        &self,
        txc: &C,
        channel_id: &Hash,
        epoch: u32,
        ticket_index: u64,
        // To be removed with https://github.com/hoprnet/hoprnet/pull/6018
        domain_separator: &Hash,
        // To be removed with https://github.com/hoprnet/hoprnet/pull/6018
        chain_keypair: &ChainKeypair,
    ) -> Result<Option<AcknowledgedTicket>> {
        match ticket::Entity::find()
            .filter(ticket::Column::ChannelId.eq(channel_id.to_hex()))
            .filter(ticket::Column::ChannelEpoch.eq(epoch.to_be_bytes().as_ref()))
            .filter(ticket::Column::Index.eq(ticket_index.to_be_bytes().as_ref()))
            .one(txc)
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

    async fn mark_ticket_redeemed<C: ConnectionTrait>(&self, txc: &C, ticket: &ticket::Model) -> Result<()> {
        let stats = ticket_statistics::Entity::find_by_id(1)
            .one(txc)
            .await?
            .ok_or(DbError::CorruptedData)?;

        let current_redeemed_count = stats.redeemed_tickets;
        let current_redeemed_value = U256::from_be_bytes(&stats.redeemed_value);
        let ticket_value = U256::from_be_bytes(&ticket.amount);

        let mut active_stats = stats.into_active_model();
        active_stats.redeemed_tickets = Set(current_redeemed_count + 1);
        active_stats.redeemed_value = Set((current_redeemed_value + ticket_value).to_be_bytes().into());
        active_stats.save(txc).await?;

        ticket::Entity::delete_by_id(ticket.id).exec(txc).await?;

        Ok(())
    }

    async fn mark_tickets_neglected_in_epoch<C: ConnectionTrait + StreamTrait>(
        &self,
        txc: &C,
        channel_id: &Hash,
        epoch: u32,
    ) -> Result<()> {
        let (neglectable_count, neglectable_value) = ticket::Entity::find()
            .filter(ticket::Column::ChannelId.eq(channel_id.to_hex()))
            .filter(ticket::Column::ChannelEpoch.eq(epoch.to_be_bytes().as_ref()))
            .stream(txc)
            .await?
            .try_fold((0, U256::zero()), |(count, value), t| async move {
                Ok((count + 1, value + U256::from_be_bytes(t.amount)))
            })
            .await?;

        let stats = ticket_statistics::Entity::find_by_id(1)
            .one(txc)
            .await?
            .ok_or(DbError::CorruptedData)?;

        let current_neglected_value = U256::from_be_bytes(stats.neglected_value.clone());
        let current_neglected_count = stats.neglected_tickets;

        let mut active_stats = stats.into_active_model();
        active_stats.neglected_tickets = Set(current_neglected_count + neglectable_count);
        active_stats.neglected_value = Set((current_neglected_value + neglectable_value).to_be_bytes().into());
        active_stats.save(txc).await?;

        ticket::Entity::delete_many()
            .filter(ticket::Column::ChannelId.eq(channel_id.to_hex()))
            .filter(ticket::Column::ChannelEpoch.eq(epoch.to_be_bytes().as_ref()))
            .exec(txc)
            .await?;

        Ok(())
    }

    async fn get_ticket_statistics<C: ConnectionTrait + StreamTrait>(&self, txc: &C) -> Result<AllTicketStatistics> {
        let stats = TicketStatistics::find_by_id(1)
            .one(txc)
            .await?
            .ok_or(DbError::CorruptedData)?;

        let (unredeemed_tickets, unredeemed_value) = Ticket::find()
            .stream(txc)
            .await?
            .try_fold((0_u64, U256::zero()), |(count, amount), x| async move {
                Ok((count + 1, amount + U256::from_be_bytes(x.amount)))
            })
            .await?;

        Ok(AllTicketStatistics {
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
    }
}

#[cfg(test)]
mod tests {}
